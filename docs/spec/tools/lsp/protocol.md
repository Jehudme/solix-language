# Solix Language Server Protocol (LSP) Specification

The Solix Language Server (`solix lsp`) implements the standard Language Server Protocol (LSP 3.17) over standard I/O (`stdin`/`stdout`). It provides editor-agnostic language intelligence, live diagnostic squiggles, and native project manifest integration (`manifest.yaml`) with support for `solix-ffi` native bridges.

---

## 1. Transport & Framing

Communication adheres to the JSON-RPC 2.0 protocol using HTTP-style header framing:

```
Content-Length: <byte_length>\r\n
\r\n
<json_payload>
```

### Lifecycle Messages

1. **`initialize` (Request)**:
   - Parameters:
     - `rootUri` (string): Absolute `file://` URI to the project workspace root.
     - `workspaceFolders` (array): List of open workspace folders.
     - `capabilities` (object): Client capability flags.
   - Response:
     - Server capabilities:
       ```json
       {
         "capabilities": {
           "textDocumentSync": 1,
           "hoverProvider": true,
           "definitionProvider": true,
           "typeDefinitionProvider": true,
           "implementationProvider": true,
           "referencesProvider": true,
           "documentHighlightProvider": true,
           "documentSymbolProvider": true,
           "workspaceSymbolProvider": true,
           "callHierarchyProvider": true,
           "typeHierarchyProvider": true,
           "codeActionProvider": true,
           "codeLensProvider": {
             "resolveProvider": false
           },
           "documentFormattingProvider": true,
           "documentRangeFormattingProvider": true,
           "renameProvider": {
             "prepareProvider": true
           },
           "foldingRangeProvider": true,
           "selectionRangeProvider": true,
           "inlayHintProvider": true,
           "documentLinkProvider": {
             "resolveProvider": false
           },
           "signatureHelpProvider": {
             "triggerCharacters": ["(", ","]
           },
           "completionProvider": {
             "resolveProvider": true,
             "triggerCharacters": [".", ":"]
           },
           "semanticTokensProvider": {
             "legend": {
               "tokenTypes": [],
               "tokenModifiers": []
             },
             "full": true
           }
         },
         "serverInfo": {
           "name": "solix-lsp",
           "version": "0.1.0"
         }
       }
       ```

2. **`initialized` (Notification)**:
   - Acknowledges handshake readiness.

3. **`shutdown` (Request)**:
   - Signals server shutdown. Returns `null` result.

4. **`exit` (Notification)**:
   - Exits the server process with exit code `0` if shutdown was received, or `1` if unexpected exit.

---

## 2. Document Synchronization

The Solix Language Server maintains an in-memory document cache (`DocumentStore`) to provide live feedback without requiring users to save files to disk.

- **`textDocument/didOpen`**:
  - Registers open file buffers and immediately triggers frontend analysis.
- **`textDocument/didChange`**:
  - Updates buffer contents in memory and re-evaluates syntax and semantic rules.
- **`textDocument/didClose`**:
  - Unregisters file buffer and emits an empty `textDocument/publishDiagnostics` list to clear editor squiggles.
- **`textDocument/didSave`**:
  - Synchronizes disk changes and triggers re-analysis.

---

## 3. Project Manifest Integration (`manifest.yaml`)

When `initialize` provides a `rootUri`, the server automatically discovers `manifest.yaml` (with `manifest.toml` fallback) in the root workspace folder:

1. **Dependency Resolution**:
   - Uses `ClientModuleSolver` and `solix_common::project::ProjectManifest` to resolve:
     - Standard Library (`solix`) from the local package store (`~/.solix/packages`).
     - Profile-scoped package dependencies declared under `profiles.<name>.dependencies`.
2. **Buffer Overrides**:
   - Any open virtual document in the editor takes precedence over on-disk files.
3. **Multi-File Context**:
   - Symbols across all referenced packages and modules are bound together, preventing false-positive undeclared symbol warnings.

---

## 4. Live Diagnostics (`textDocument/publishDiagnostics`)

On every document modification, the server runs the compiler frontend pipeline:
$$\text{Lexer} \longrightarrow \text{Parser} \longrightarrow \text{Binder}$$

Errors and warnings captured by `solix::Diagnostic` are transformed into LSP diagnostic objects:

```json
{
  "method": "textDocument/publishDiagnostics",
  "params": {
    "uri": "file:///path/to/File.slx",
    "diagnostics": [
      {
        "severity": 1,
        "code": "E0101",
        "source": "solix",
        "message": "Expected ';' after expression",
        "range": {
          "start": { "line": 4, "character": 12 },
          "end": { "line": 4, "character": 13 }
        }
      }
    ]
  }
}
```

- **Coordinates**: Converted from 1-indexed compiler source positions to LSP 0-indexed positions.
- **Spans**: Utilizes accurate start and end token coordinates (`end_line`, `end_column`).
- **Diagnostic Isolation**: Diagnostics generated across multi-file compilation pipelines and imported dependencies are strictly filtered by originating `source_id`. Diagnostics whose `source_id` does not match the active document are prevented from leaking across file boundaries, eliminating invalid character offset spans (`OutOfBounds`).

---

## 5. Navigation & Spatial AST Indexing

The Language Server populates a spatial interval index (`SpatialAstIndex`) mapping source coordinate intervals `(line, column) -> Node*` across all parsed compilation units, complemented by a flattened `local_symbols` registry for parameter and variable scope tracking.

### 5.1 Definition & Type-Definition Resolution (`textDocument/definition`, `textDocument/typeDefinition`)
- **Single- & Multi-File Resolution**: When navigating definitions across distinct source units (e.g. `Main.slx` calling a method in `Helper.slx`), target nodes are resolved to their originating file URI via `node->source` and context-registered AST sources.
- **Dangling Source Guard**: Persistent compilation options (`last_opts_`) guarantee source string pointers remain valid across interactive requests without memory corruption.
- **Fallbacks**: If AST nodes lack exact source pointers, the server queries registered compilation unit paths to construct well-formed `file://` URIs.

### 5.2 Hover Tooltips (`textDocument/hover`)
- Hover requests inspect AST nodes and `local_symbols` at target line/column coordinates and render Markdown-formatted symbol representations, method signatures, parameter metadata, and type structures.
- Whitespace and unmapped token spans safely resolve to `null` responses.

---

## 6. Syntax Grammar & TextMate Scopes

The Solix VS Code extension embeds `solix.tmLanguage.json` for syntax tokenization.

- **Class Declarations**: Matches `class|interface` declarations, styling the declared type identifier with `entity.name.type.class.solix`.
- **Reserved Keywords**: Comprehensive keyword coverage including `implements`, `extends`, `operator`, `assert`, `exit`, `weak`, `instanceof`, `sizeof`, control flow (`match`, `case`, `defer`, `return`, `throw`), and type keywords (`int8` through `uint64`, `f32`, `f64`, `bool`, `string`, `void`, `auto`).

---

## 7. Performance & UTF-8 Memory Safety Invariants

### 7.1 Multibyte UTF-8 Character Slicing Safety
All text offset conversions from LSP coordinates (UTF-16/character columns) to byte indices clamp boundaries to valid UTF-8 character boundaries (`is_char_boundary` / `clamp_to_char_boundary`). Slicing within multi-byte UTF-8 sequences in strings or comments is strictly prevented.

### 7.2 External Package Index Caching
To maintain sub-millisecond response latency during interactive typing, the LSP maintains an in-memory cache of resolved external package symbol indices (`PackageExportIndex`) keyed by `(package_name, profile, version, archive_mtime_secs)`. Modifications to installed `.slxp` packages immediately invalidate cache entries via filesystem mtime detection.

### 7.3 Semantic Tokens Drift Resilience & Coordinate Sanitization
Semantic tokens (`textDocument/semanticTokens/full`) utilize LSP 3.17 relative delta encoding (`delta_line`, `delta_start_character`, `length`, `token_type`, `token_modifiers`). To prevent semantic token desynchronization and mid-word color fragmentation during syntax errors:
1. **UTF-16 Code Unit Precision**: Character offsets and lengths are computed using UTF-16 code unit counts conforming strictly to the LSP specification.
2. **Multi-line Token Splitting**: Spans spanning line breaks are split at newline boundaries, ensuring no token has multi-line span length.
3. **Strict Non-Overlapping Invariant**: Tokens are strictly sorted by line and character offset and sanitized so subsequent tokens never start before the preceding token ends.
4. **Resilient Token Fallback**: If syntax errors prevent full AST generation, the lexer token stream provides robust lexical classification (prioritizing call patterns `ident(` and `::ident(` as method/function tokens before initial-casing type heuristics), while strictly distinguishing variable instances holding nominal class types from type declarations, preventing token drift and color fragmentation across remaining source lines.
5. **CRLF Visual Alignment**: Character coordinates ignore `\r` carriage return bytes in UTF-16 code unit accumulation, ensuring column positioning remains invariant between LF and CRLF line terminations.

### 7.4 Formal Parameter Preservation for Inlay Hints & Signatures
Parameter names and types are preserved across all compiler passes (`SymbolEntry::parameter_names`).
1. **Inlay Hints**: Inlay hints (`textDocument/inlayHint`) query live editor document snapshots (`get_live_snapshot`), resolve callee method declarations via direct AST `resolved_symbols` binding, and display true declared parameter names (`brick_rect:`, `active_color:`) rather than synthesized `arg{index}:` placeholders, accompanied by hover tooltips providing full type annotations (`Rectangle rec`, `Color color`).
2. **Signature Help Resilience**: Active typing of unclosed invocations (e.g. `DrawRectangleRounded(brick_rect, `) invokes a backward lexical scanner that extracts the callee symbol and active parameter index even when resilient AST generation produces empty call nodes.
3. **Completion Label Details**: In autocompletion lists (`textDocument/completion`), method and function entries utilize LSP 3.17 `CompletionItemLabelDetails` to render parameter signatures in `detail` (`(Rectangle rec, float32 roundness, int32 segments, Color color)`) and return types in `description` (`void`), while field/variable entries display their evaluated type.
4. **Chained Call and Indexing Receiver Resolution**: In autocompletion lists (`textDocument/completion`), receiver expressions containing chained method invocations followed by array index brackets (e.g. `receiver.getItems()[0].`) correctly resolve the underlying element type and present accurate member completions without identifier truncation.
5. **UTF-8 Character Boundary Validation**: All string slicing operations throughout AST symbol extraction, hover, completion, and signature help validate UTF-8 codepoint boundaries (`is_char_boundary`), preventing server panics when documents contain multi-byte Unicode characters.

### 7.5 Strict Import Resolution Parity
The Language Server enforces diagnostic parity with the CLI compiler (`solix compile` / `solix build`):
1. **External Packages**: Any external package (such as `solix.*` or `raylib.*`) requires an explicit dependency entry in `manifest.yaml` or an `extern import` declaration. Wildcard imports targeting undeclared external packages consistently emit `CannotResolveImport` (`E0232`).

### 7.6 Workspace Manifest Dependency Table Resolution
The LSP `WorkspaceManager` locates `manifest.yaml` in project parent directories and deserializes `profiles.<name>.dependencies` using strictly-typed mapping notation:
- Dependencies are declared as `<package_name>: { version: "<version>", profile: "<profile>" }`.
- When `profile` is omitted, the profile defaults to `"release"`.
- Each resolved dependency propagates both semantic version requirements and build profiles into the language server compilation options, ensuring background diagnostics match CLI build output.

### 7.7 Internal Workspace Distribution
The `solix-lsp` crate operates as an internal daemon embedded into the single `solix` executable (`solix lsp`). Publication to public registries is restricted (`publish = false`), and it is distributed exclusively as part of the official toolchain binaries.

### 7.8 Doc Comment & Parameter Documentation Intelligence
The Language Server extracts doc comments (`///` and `/** ... */`) directly into LSP metadata:
1. **Completion Item Documentation**: Declarations in `textDocument/completion` results attach their doc comment string directly in `CompletionItem.documentation` upfront, allowing code completions in VS Code to immediately display formatted markdown descriptions without requiring a secondary `completionItem/resolve` round-trip.
2. **Signature Help Parameter Documentation**: When signature help triggers on active parameter typing (`textDocument/signatureHelp`), structured `@param <name> <description>` tags are extracted from the callee's doc comments and attached to the corresponding `ParameterInformation.documentation`. Editors render the active parameter's description alongside its type in the signature tooltip.

### 7.9 Multi-File Workspace Path Normalization & Windows Verbatim Prefix Stripping
When discovering and analyzing project source files across workspace directories (`analyze_document`), file paths and document URIs are normalized using `solix_common::strip_windows_verbatim_prefix`. On Windows environments where path canonicalization injects `\\?\` prefixes (e.g. `\\?\C:\project\...`), stripping verbatim prefixes guarantees that target file comparisons and document dictionary keys align consistently between `Url::from_file_path` conversions and disk paths, preventing duplicate document registration and spurious duplicate symbol diagnostics.

### 7.10 Cached-Dependency Import Parity & Scoped Enum Completion Filtering
1. **Cached-Dependency Import Enforcement**: `WorkspaceManager` sets `implicit_link_dependency_binaries = false` on `CompilationOptions` so preloaded in-memory dependency archives cached after `textDocument/didOpen` are not promoted to `cli_linked_packages`. Subsequent `textDocument/didChange` passes continue reporting `E0201` diagnostics for unimported dependency symbols, matching `solix build`.
2. **Scoped Enum & Member Filtering**: Bare identifier completion (`handle_general_completion`) excludes `EnumVariant` entries (parented by `Type::Enum`) as well as out-of-scope class members, constructors, destructors, and operators. Enum variants such as `WHITE` are suggested exclusively after `EnumName::` in `handle_scope_resolution_completion`.

### 7.11 Multi-Level Namespace Quick Fixes & Error Wiki Routing
`textDocument/codeAction` provides diagnostic-driven Quick Fixes (`CodeActionKind::QUICKFIX`) with `package`-aware edit placement:
- **`E0201` (Undefined Symbol)**: Discovers matching public classes, interfaces, enums, functions, and static fields across workspace snapshots and resolved dependency package indices (`PackageNamespaceIndex`), offering both exact (`import pkg.ns.Symbol;`) and namespace wildcard (`import pkg.ns.*;`) imports inserted after the `package` statement and existing `import` declarations, plus Levenshtein typo replacement candidates.
- **`E0212` (Missing Return)**: Inserts `return 0;` (or type-appropriate default `return`) before the closing brace of non-void methods/functions.
- **`E0226` / `E0227` (Access Delimiter)**: Rewrites `.` <-> `::` for static vs instance member access.
- **`E0207` / `E0268` / `E0282` / `E0283` (Override & Visibility)**: Adds or removes `override` accurately (without misrouting `E0284`) and widens narrowing override visibility to `public`.
- **`E0206` / `E0265` / `E0306` / `E0271` / `W0001` / `E0101` / `E0112` / `E0153` / `E0192`**: Generates missing method stubs, inserts `default` switch arms, removes unreachable statements, inserts/removes semicolons, removes duplicate `package` statements, and converts multi-character `'...'` literals to `"..."` strings.
- **Documentation Links**: `make_code_description` and phase-specific diagnostic builders route `E00*` to `lexer-errors.md`, `E01*` to `parser-errors.md`, `E02*` and binder `E0301`-`E0316` to `binder-errors.md`, and `E03*` runtime diagnostics to `runtime-errors.md`.

### 7.12 Formatter Lexical Brace Tracking, Qualified Signature Help, and Scoped Inlay Hints / CodeLens
- **Lexical Formatter Brace Counting (`formatter.rs`)**: `count_code_braces` and `format_line_content` track string/char escape parity (`escape_active`), `//` line comments, and `/* ... */` block comments so braces inside comments, strings, or trailing escaped backslashes (`"C:\\\\"`) never corrupt indentation.
- **Qualified Signature Help (`signature_help.rs`)**: Preserves AST receiver/class qualifiers and restricts fallback symbol lookup to in-scope or public symbols.
- **Scoped RAII Inlay Hints & CodeLens (`inlay_hints.rs`, `code_lens.rs`, `diagnostics.rs`)**: Emits `~RAII` inlay hints only for blocks declaring locals whose class defines a destructor, traverses `Destructor`/`Operator`/`Try`/`Switch`, checks zero-parameter arity for `test*` CodeLens actions, counts references by `fully_qualified_name`, and reports `W0001` unreachable code inside `try`/`switch` and after exhaustive `if`/`else` returns.
- **Safe Symbol Rename (`rename.rs`)**: Blocks renaming `main` and external/read-only symbols in `prepare_rename`, detects scope collisions in `rename_symbol_checked`, and surfaces descriptive JSON-RPC `InvalidParams` errors.




