# Milestone 034/: Final v1.0 Pre-Release Audit, Solver-Driven Import Intelligence, Integrated Terminal DAP, Modular Multi-Entrypoint Runtime & Load-Time Symbol Collision Hardening

## Document Metadata

* **Milestone ID:** MS-034
* **Target Version:** `v1.0.0-rc.1` (Final Pre-v1.0 Stabilization Gate)
* **Milestone Branch:** `milestone/034-final-v1-pre-release-audit-and-modular-runtime`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/authoring-features-and-wiki.md` (GL-0012)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/020-package-manager-solver-and-dependency-resolution.md` (MS-020)
  * `docs/internals/milestones/023-compiler-stabilization-qualified-declarations-and-dual-linking.md` (MS-023)
  * `docs/internals/milestones/029-single-executable-consolidation-lsp-remediation-and-common-solver.md` (MS-029)
  * `docs/internals/milestones/030-industrial-debug-adapter-protocol-solix-dap-and-zero-overhead-vm-hooks.md` (MS-030)
  * `docs/internals/milestones/031-vscode-debugger-integration-release-run-debug-ux-and-automated-e2e-suite.md` (MS-031)
  * `docs/internals/milestones/032-dap-and-vscode-debugger-stabilization-and-conformance.md` (MS-032)

---

## 1. Executive Summary & Full-Stack Forensic Pre-v1.0 Audit

Before tagging **Solix v1.0**, a comprehensive full-stack architectural audit was conducted across `solix-common`, `solix-compilation`, `solix-runtime`, `solix-lsp`, `solix-dap`, `solix-toolchain`, and `editors/vscode`.

This milestone combines **5 core architectural requirements** for v1.0 readiness with **6 additional pre-release defects** discovered during the forensic codebase audit—establishing a rock-solid, zero-compromise foundation for Solix v1.0.

### 1.1 Complete Inventory of All 11 Architectural Enhancements & Remediations

| ID | Category & Target Crate(s) | Current State / Root Cause | Normative v1.0 Architectural Fix |
| :--- | :--- | :--- | :--- |
| **REQ-01** | **Import & Extern Import Intelligence**<br>`solix-common`, `solix-lsp` | `completion.rs` hardcodes `solix.io.Console;` and dumps raw class names from open snapshots without checking package paths or querying `ModuleSolver`. | **First identifier** (`import <cursor>` / `extern import <cursor>`) queries `ModuleSolver::list_packages()`, `blt`, and workspace roots. **Inner segments** (`import solix.` / `import solix.io.`) inspect the package's `ExportedAst` / `ExportManifest` to suggest child subpackages (`io`, `math`, `core`) and public symbols (`Console;`, `*;`). |
| **REQ-02** | **Integrated Terminal DAP Execution**<br>`solix-dap`, `editors/vscode` | `solix-dap` runs over `--stdio` and captures `stdout` into read-only DAP `output` events, preventing interactive `stdin` (`Console::read_line()`, `Console::input_int32()`) and ANSI terminal control. | In VS Code, launch `solix dap --port <port>` inside an **Integrated Terminal (`Solix Debug Terminal`)** and connect via `vscode.DebugAdapterServer(port)`. Guest `stdin`/`stdout`/`stderr` flow natively through the real terminal PTY while DAP traffic runs over the dedicated localhost TCP socket! |
| **REQ-03** | **Clean CodeLens & UI Labels**<br>`solix-lsp`, `editors/vscode` | `code_lens.rs` and `editors/vscode` use the `🐞` emoji (`"🐞 Debug Main"`, `"🐞 Debug Test"`). | Strip all emojis from debug buttons; render cleanly as **`Debug Main`** and **`Debug Test`** (and codicon `$(debug-alt)` in VS Code menus). |
| **REQ-04** | **Modular Multi-Entrypoint Runtime & Strict CLI Entrypoint Contract**<br>`solix-common`, `solix-compilation`, `solix-runtime`, `solix-toolchain`, `solix-lsp` | `VirtualMachine` couples static initialization with a single `run()` call, lacks reusable multi-entrypoint invocation for host game engines, and does not enforce visibility/signature rules on CLI entrypoints. | **1)** `VirtualMachine::new_initialized` runs static initializers once; `invoke_entry(symbol, &[Value])` can be called repeatedly across multiple entrypoints (`on_init`, `on_update`, `on_render`) sharing `static_slots` and `VirtualHeap`; `shutdown()` / `Drop` deterministically runs RAII destructors on statics.<br>**2)** `invoke_entry` accepts any `public` function/static method with primitive/array parameters and return types.<br>**3)** CLI `solix run` enforces `public int32 <sym>()` or `public int32 <sym>(char[] / string[])` (with `void` returning `0`), resolving `--entry` $\to$ `manifest.toml` `[entrypoints]` $\to$ `.slxb` `entry_point` $\to$ `"main"`. |
| **REQ-05** | **Strict Load-Time Duplicate Symbol Rejection**<br>`solix-runtime` | `RuntimeContext::new` loads multiple `.slxb` binaries and inserts functions, classes, and static globals into maps, allowing a later binary to overwrite/shadow an earlier binary's symbols. | Strictly verify at load time that **no two binaries define the same non-extern function symbol, class symbol, or static global symbol**. Immediately abort with `RuntimeError::DuplicateSymbolDefinition` (`R0024`). |
| **BUG-01** | **Legacy `.` Static Calls in Templates & Tests**<br>`solix-toolchain` | `assets/templates/untitled-rust/src/main.slx:8`, `tests/test_main.slx:5`, `init_project.py`, and `build_and_archive_tests.rs:125,326` still use `MathBridge.fast_add(...)` instead of `::`. | Replace all static method `.` calls across `init_project.py`, project templates, and toolchain tests with `MathBridge::fast_add(...)` in strict compliance with Solix v2.0 (`E0226`). |
| **BUG-02** | **`SOLIX_LINK_MODE` Ignored by `solix_toolbox.py`**<br>`solix-toolchain` | `build.rs:76` sets `SOLIX_LINK_MODE=static` when `solix build --static` is passed, but `solix_toolbox.py` never reads `SOLIX_LINK_MODE` or passes `--link static` to `solix compile`. | Update `compile_solix_sources` in `solix_toolbox.py` to read `os.environ.get("SOLIX_LINK_MODE")` and pass `--link <mode>` to `solix compile`. |
| **BUG-03** | **Manifest `[entrypoints]` Not Embedded by `solix_toolbox.py`**<br>`solix-toolchain` | `solix_toolbox.py` never passes `--entry <manifest_entry>` to `solix compile`, so the compiled `.slxb` inside `.slxarc` does not record the custom entrypoint in its binary metadata. | Add `--entry <SYMBOL>` to `solix compile` and pass `manifest["entrypoints"]["main"]` (or `"test"` when `profile == "test"`) from `solix_toolbox.py`. |
| **BUG-04** | **Manual Static Init Loop in `solix-dap`**<br>`solix-dap`, `solix-runtime` | `solix-dap/src/server.rs:144` manually loops over `init_function_ids` to push frames instead of using a unified VM static initialization lifecycle. | Centralize static initialization inside `VirtualMachine::initialize_statics()` (`statics_initialized: bool`) so CLI, DAP, and embedded hosts share one canonical initialization path. |
| **BUG-05** | **Legacy `solix-lsp` Binary Search in VS Code**<br>`editors/vscode` | `editors/vscode/src/extension.ts` still contains legacy dual-binary fallback logic searching for a standalone `solix-lsp` executable removed in MS-029. | Clean up `extension.ts` to strictly resolve the single unified `solix` toolchain binary (`solix lsp --stdio`, `solix dap --port <port>`). |
| **BUG-06** | **Unregistered `solix.showReferences` Command**<br>`solix-lsp`, `editors/vscode` | `code_lens.rs:51,66` emits `"command": "solix.showReferences"` on `N references` CodeLenses, which is not registered in `commands.ts`. | Register `solix.showReferences` in `editors/vscode/src/commands.ts` and delegate to `vscode.commands.executeCommand('editor.action.showReferences', ...)`. |

---

## 2. Detailed Architectural Specifications

### 2.1 Pillar 1: Solver-Backed & API-Driven `import` / `extern import` Autocompletion (`solix-common` & `solix-lsp`)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│           Hierarchical Import & Extern Import Completion Engine                  │
├──────────────────────────────────────────────────────────────────────────────────┤
│  User types: `import <cursor>` or `extern import <cursor>` (Segment 0 — Root)    │
│    ├── 1. `workspace.solver().list_packages()` -> e.g. `solix`, `raylib`         │
│    ├── 2. Built-in runtime package -> `blt`                                      │
│    └── 3. Workspace local package roots -> e.g. `sample_workspace`, `my_app`     │
├──────────────────────────────────────────────────────────────────────────────────┤
│  User types: `import solix.<cursor>` (Segment 1 — Subpackage / Root Symbol)      │
│    ├── Resolves `solix` via cached `ModuleSolver::resolve_latest("solix")`       │
│    ├── Extracts subpackage tree from `ExportedAst` (`solix::io`, `solix::math`,  │
│    │   `solix::core`, `solix::collections`, `solix::time`, `solix::system`,      │
│    │   `solix::crypto`)                                                          │
│    └── Suggests immediate child namespaces (`io`, `math`, `core`, ...) + `*;`    │
├──────────────────────────────────────────────────────────────────────────────────┤
│  User types: `import solix.io.<cursor>` (Segment N — Terminal Symbols & Subpkgs) │
│    ├── Filters exported public symbols in namespace `solix::io`                  │
│    └── Suggests: `Console;`, `File;`, `Path;`, `Directory;`, `FileStream;`,      │
│        `MemoryStream;`, `BufferedReader;`, `StreamReader;`, `StreamWriter;`,     │
│        `IStream;`, `FileMode;`, `SeekOrigin;`, `ConsoleColor;`, `*;`             │
└──────────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Extending `ModuleSolver` in `crates/solix-common/src/solver.rs`
Add `list_packages(&self) -> Vec<String>` to the `ModuleSolver` trait:
```rust
pub trait ModuleSolver: Send + Sync + std::fmt::Debug {
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError>;

    fn resolve_exact(
        &self,
        package_name: &str,
        version: PackageVersion,
        expected_sha256: Option<&str>,
    ) -> Result<SolvedPackage, SolverError> {
        let solved = self.resolve_compatible(package_name, version)?;
        if solved.version != version {
            return Err(SolverError::IncompatibleVersion {
                package: package_name.to_string(),
                required: version.to_string(),
                found: solved.version.to_string(),
            });
        }
        if let Some(expected) = expected_sha256
            && !expected.is_empty()
        {
            let actual = solved.program.compute_sha256();
            if actual != expected {
                return Err(SolverError::ChecksumMismatch {
                    package: package_name.to_string(),
                    version: version.to_string(),
                    expected: expected.to_string(),
                    actual,
                });
            }
        }
        Ok(solved)
    }

    fn resolve_latest(&self, package_name: &str) -> Result<SolvedPackage, SolverError>;

    /// Returns a sorted list of all available top-level package names in the solver store.
    fn list_packages(&self) -> Vec<String> {
        Vec::new()
    }
}
```
* **`ClientModuleSolver::list_packages(&self)`:** Scans `self.store_dir` (`~/.solix/packages`) for subdirectories containing at least one valid version folder (or entries in `registry.json`), returning their package names sorted and deduplicated.
* **`InMemoryModuleSolver::list_packages(&self)`:** Returns all unique `pkg.name` strings registered in `self.packages`, sorted alphabetically.

#### 2. Package Namespace & Symbol Index in `WorkspaceManager` (`crates/solix-lsp/src/workspace.rs`)
To avoid hitting the filesystem or re-parsing `api_header` on every keystroke during `import` completion:
* `WorkspaceManager` caches a `package_export_index: RwLock<HashMap<String, PackageNamespaceIndex>>` where:
  ```rust
  #[derive(Debug, Clone, Default)]
  pub struct PackageNamespaceIndex {
      /// All full package paths declared in this module (e.g. `"solix.io"`, `"solix.math"`).
      pub namespaces: BTreeSet<String>,
      /// Map from dot-separated namespace path (e.g. `"solix.io"`) to exported public symbols
      /// `(symbol_name, CompletionItemKind, detail_signature)`.
      pub symbols_by_namespace: HashMap<String, Vec<(String, CompletionItemKind, String)>>,
  }
  ```
* When `resolve_package_index(pkg_name)` is queried:
  1. If `pkg_name == "blt"`, returns the built-in index containing namespace `"blt.io"` with functions `print` and `println`.
  2. Otherwise, checks `package_export_index` cache; on cache miss, calls `solver.resolve_latest(pkg_name)`, inspects its deserialized `ExportedModuleAst` (from `program.export_manifest.exported_ast_bytes` or `api_header`) plus `exported_classes`, `exported_functions`, and `exported_enums`, and populates `namespaces` and `symbols_by_namespace`.
  3. Also scans all active workspace snapshots (`self.all_snapshots()`) for local `package a.b.c;` declarations and qualified declarations (`class a::b::Name`) so project-local multi-file imports autocomplete with the exact same hierarchical precision!

#### 3. Trigger Character & Prefix Extraction in `crates/solix-lsp/src/completion.rs`
* In `handle_completion`:
  * Before dispatching general completion, check if the trimmed current line up to `column` starts with `import ` or `extern import `.
  * Even when `trigger_char == Some(".")`, if the current line up to cursor is an `import` or `extern import` statement (e.g. `import solix.` or `extern import solix.io.`), route directly to `handle_import_completion(workspace, snapshot, import_prefix_after_keyword)` instead of `handle_dot_completion`!
  * **Completion Insertion Rule:**
    * When suggesting an intermediate subpackage folder (e.g., `io` after `import solix.`), `insert_text` is `"io"` (`CompletionItemKind::MODULE`) so the user can continue typing `.Console;` or `.*;`.
    * When suggesting a terminal symbol (e.g., `Console` after `import solix.io.`) or wildcard (`*`), `insert_text` is `"Console;"` or `"*;"` (unless the line already has a trailing `;` after the cursor, in which case omit the duplicate `;`!).

---

### 2.2 Pillar 2: Integrated Terminal Execution for Debugging (`solix-dap` & `editors/vscode`)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│         Integrated Terminal + TCP Socket DAP Architecture (`solix dap`)          │
├──────────────────────────────────────────────────────────────────────────────────┤
│  VS Code Extension (`editors/vscode/src/debug.ts`)                               │
│    1. Allocates free ephemeral TCP port `P` on `127.0.0.1`                       │
│    2. Spawns `solix dap --port <P>` inside VS Code Integrated Terminal           │
│       (`"Solix Debug Terminal"`) or attaches terminal PTY                        │
│    3. Returns `new vscode.DebugAdapterServer(P, "127.0.0.1")`                    │
│         │                                               │                        │
│         │ (1) DAP JSON-RPC Control Traffic              │ (2) Native Guest I/O   │
│         │     (Breakpoints, Stack, Variables, Watches)  │     (`stdin`, `stdout`,│
│         ▼                                               ▼      ANSI colors)      │
│  `solix dap --port <P>` TCP Listener             VS Code Integrated Terminal     │
└──────────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Native Terminal I/O in TCP Mode (`crates/solix-dap/src/server.rs`)
* In `crates/solix-dap/src/server.rs`:
  * Track `pub is_tcp_terminal_mode: bool` on `DapServerState` (`port.is_some()`).
  * **When `is_tcp_terminal_mode == true` (`solix dap --port <port>`):**
    * DAP JSON-RPC messages travel exclusively over the accepted `TcpStream` (`127.0.0.1:<port>`), **NOT** over process `stdin`/`stdout`!
    * Therefore, process `stdin`, `stdout`, and `stderr` are 100% free for the guest Solix program!
    * Do **NOT** attach the intercepting `output_sink` to `RuntimeOptions` when `is_tcp_terminal_mode == true`. Guest calls to `print(...)`, `println(...)`, `Console::print(...)`, `Console::println(...)`, `Console::read_line()`, and `Console::input_int32()` read and write directly to the real Integrated Terminal PTY!
  * **When `is_tcp_terminal_mode == false` (`solix dap --stdio`):**
    * Keep the `output_sink` capture enabled so headless automated tests (`dap_rpc.test.mjs` and `crates/solix-dap/tests/dap_tests.rs`) continue to capture `Event::Output` packets over stdio without needing a GUI terminal!

#### 2. `SolixDebugAdapterDescriptorFactory` in `editors/vscode/src/debug.ts`
* In `editors/vscode/src/debug.ts`:
  * When launching a debug session in VS Code:
    1. Allocate an available localhost port using Node's `net.createServer()`.
    2. Reuse or create a dedicated VS Code Integrated Terminal named `"Solix Debug"` (`vscode.window.createTerminal({ name: 'Solix Debug', cwd })`).
    3. Show the terminal (`terminal.show(true)`) and launch `"${solixBinaryPath}" dap --port ${port}` inside the terminal (or support `console: "integratedTerminal"` / `"internalConsole"` in `launch.json`, defaulting to `"integratedTerminal"`).
    4. Wait with a fast retry loop (up to 3,000 ms, polling every 25 ms) for `127.0.0.1:${port}` to accept the TCP connection, and return `new vscode.DebugAdapterServer(port, '127.0.0.1')`!

---

### 2.3 Pillar 3: Clean CodeLens & UI Labels + `solix.showReferences` (`solix-lsp` & `editors/vscode`)

1. **Remove Emoji from `crates/solix-lsp/src/code_lens.rs`:**
   * Replace `"🐞 Debug Main"` with `"Debug Main"`.
   * Replace `"🐞 Debug Test"` with `"Debug Test"`.
2. **Register `solix.showReferences` in `editors/vscode/src/commands.ts`:**
   * Register `vscode.commands.registerCommand('solix.showReferences', async (uriStr?: string, position?: { line: number; character: number }) => { ... })`:
     * Converts `uriStr` to `vscode.Uri.parse(uriStr)` and `position` to `new vscode.Position(position.line, position.character)`.
     * Queries `vscode.commands.executeCommand<vscode.Location[]>('vscode.executeReferenceProvider', uri, pos)` and invokes `vscode.commands.executeCommand('editor.action.showReferences', uri, pos, locations ?? [])`.

---

### 2.4 Pillar 4: Modular Multi-Entrypoint Runtime & Strict CLI Entrypoint Contract

#### 1. Decoupled `VirtualMachine` Lifecycle for Game Engines & Host Embeddings (`crates/solix-runtime`)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│         Reusable Multi-Entrypoint VirtualMachine Lifecycle (`solix-runtime`)     │
├──────────────────────────────────────────────────────────────────────────────────┤
│  1. Construction & One-Time Static Initialization:                               │
│     `let mut vm = VirtualMachine::new_initialized(context)?;`                    │
│     • Allocates string constants on `VirtualHeap`.                               │
│     • Executes all module `@init` (`init_function_ids`) ONCE.                    │
│     • Sets `vm.statics_initialized = true`.                                      │
│     • `vm.static_slots` now holds live initialized global/static state!          │
├──────────────────────────────────────────────────────────────────────────────────┤
│  2. Repeated Multi-Entrypoint Invocations (Sharing `static_slots` & `heap`):     │
│     • `vm.invoke_entry("Engine::on_init", &[Value::I32(800), Value::I32(600)])?` │
│     • `vm.invoke_entry("Engine::on_update", &[Value::F64(0.016)])?`              │
│     • `vm.invoke_entry("Engine::on_render", &[])?`                               │
│     • Validates target is `public` and all parameter/return types are primitive  │
│       (or 1D primitive arrays `char[]` / `string[]`).                            │
│     • Never resets `static_slots` or `heap` between calls!                       │
├──────────────────────────────────────────────────────────────────────────────────┤
│  3. Deterministic Shutdown & RAII Destructor Cleanup (`shutdown()` & `Drop`):    │
│     `vm.shutdown()?;` (Also called automatically in `impl Drop for VirtualMachine`)│
│     • Iterates `0..vm.static_slots.len()`, replaces each `Value::Ref(addr)` with │
│       `Value::Null`, and calls `vm.release_reference(addr)` (triggering any      │
│       `~ClassName()` destructors on static singleton objects!).                  │
│     • Releases string constants and verifies clean heap memory.                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

#### 2. Enriched Function Signature Metadata in `CompiledFunction` (`crates/solix-common`)
In `crates/solix-common/src/bytecode/function.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledFunction {
    pub id: u32,
    pub symbol: String,
    pub num_registers: u16,
    pub num_parameters: u8,
    pub reference_registers: Vec<u16>,
    pub instructions: Vec<Instruction>,
    pub exception_handlers: Vec<ExceptionHandlerEntry>,
    pub source_map: Vec<SourceMapEntry>,
    #[serde(default)]
    pub local_variables: Vec<LocalVariableDebugInfo>,
    /// Whether this function or method was declared `public`.
    #[serde(default = "default_true")]
    pub is_public: bool,
    /// Declared return type of this function/method.
    #[serde(default)]
    pub return_type: crate::types::Type,
    /// Declared parameter types of this function/method (excluding implicit `this`).
    #[serde(default)]
    pub parameter_types: Vec<crate::types::Type>,
}
```
And in `crates/solix-common/src/bytecode/binary.rs` (`BinaryProgram` & `ExportManifest`):
```rust
/// Optional default entrypoint symbol embedded in the binary header/manifest.
#[serde(default)]
pub entry_point: Option<String>,
```

#### 3. Package-Qualified Free Functions & Flexible Symbol Lookup (`solix-compilation` & `solix-runtime`)
* **How Free Functions Are Named in Packages (`sample_workspace::main` vs `main`):**
  * When a file declares `package sample_workspace;` and defines a top-level free function `public int32 main(char[] args)`, `EmitterPhase` (`crates/solix-compilation/src/phases/emitter/pipeline.rs`) emits its canonical symbol as `"sample_workspace::main"` (or `"main"` if in the default root package).
  * In `RuntimeContext::resolve_entry_function_id(symbol: &str) -> Result<u32, RuntimeError>`:
    1. First, check exact match in `self.function_symbols.get(symbol)`.
    2. Second, if `symbol` does not contain `"::"` (e.g., `"main"`), check if the primary binary (`binaries[0]`) has a package name `pkg` and `"<pkg>::<symbol>"` (e.g., `"sample_workspace::main"`) or `"Main::<symbol>"` exists in `self.function_symbols`.
    3. Third, if `symbol` is `"<pkg>::<fn_name>"` (e.g., `"sample_workspace::main"`), also check if `binaries[0].package_name == "<pkg>"` and `"<fn_name>"` (`"main"`) or `"<pkg>::Main::<fn_name>"` exists in `self.function_symbols`!
  * This guarantees that whether `manifest.toml` specifies `main = "sample_workspace::main"`, `main = "Main::main"`, or `main = "main"`, resolution is 100% deterministic and unambiguous!

#### 4. Runtime Entrypoint Validation vs. CLI `solix run` Entrypoint Validation
1. **In `VirtualMachine::invoke_entry(&mut self, symbol: &str, args: &[Value])` (`solix-runtime`):**
   * Resolves `func_id` via `context.resolve_entry_function_id(symbol)?`.
   * Checks `func.is_public`: if `false`, returns `Err(RuntimeError::InvalidEntryPointSignature { symbol, reason: "Entry point function must have 'public' visibility" })` (`R0020`).
   * Checks that `func.return_type` and all `func.parameter_types` are primitive types (`Void`, `Bool`, `Char`, `Int8..Int64`, `UInt8..UInt64`, `Float32`, `Float64`, `StringPrimitive`) or 1D primitive arrays (`char[]`, `string[]`, `int32[]`, etc.). If a parameter or return type is an arbitrary user `Type::Class` or `Type::Interface`, returns `Err(RuntimeError::InvalidEntryPointSignature { ... })`.
   * Checks `args.len() == func.num_parameters as usize`.
2. **In `VirtualMachine::validate_cli_entry_point(&self, func_id: u32)` (Used by `solix run` CLI):**
   * Enforces the strict CLI contract:
     * `func.is_public == true` (must be `public`).
     * `func.return_type` must be `int32` (`Type::Primitive(PrimitiveKind::Int32)`) or `void` (`Type::Primitive(PrimitiveKind::Void)`). Any other return type (e.g. `float64`, `string`, `bool`, `Point`) is rejected with `RuntimeError::InvalidEntryPointSignature` (`R0020`)!
     * `func.parameter_types` must be either:
       * Empty `()` (`0` parameters), OR
       * Single CLI argument array `(char[] args)` or `(string[] args)` or `(char[][] args)` (`1` parameter).
       * Any other parameter list (e.g. `(int32 a, int32 b)`) is rejected by `solix run` with `RuntimeError::InvalidEntryPointSignature` (`R0020`)!
3. **CLI Entrypoint Resolution Priority in `solix run` (`crates/solix-toolchain/src/commands/run.rs`):**
   * **Priority 1:** Explicit CLI flag `--entry <SYMBOL>` (if provided by user).
   * **Priority 2:** If running a `.slxarc` archive: `manifest.entrypoints.test` (when `--test` is set) or `manifest.entrypoints.main` (when non-empty).
   * **Priority 3:** If running a `.slxb` binary: `program.entry_point` embedded in `.slxb` metadata (when `Some(non_empty)`).
   * **Priority 4:** Fallback to `"main"`. If `"main"` does not exist in the binary, returns `RuntimeError::EntryPointNotFound` (`R0019`).

#### 5. Manifest-Aware Entrypoint CodeLenses in `crates/solix-lsp/src/code_lens.rs`
* In `crates/solix-lsp/src/code_lens.rs`:
  * Inspect the active workspace's `manifest.toml` (via `WorkspaceManager::project_manifest_for_uri(&snapshot.uri)`):
    * Extract `main_entry = manifest.entrypoints.main` (defaulting to `"main"` if no `manifest.toml` exists or if `main` is empty).
    * Extract `test_entry = manifest.entrypoints.test` (defaulting to `"test"` if empty).
  * Match each `Declaration::Method` (or `ClassMember::Method` with `Modifier::Public`) against `main_entry` and `test_entry` (comparing both unqualified `func_name` and qualified `<pkg_or_class>::<func_name>`):
    * Emit `▶ Run Main` and `Debug Main` on the function matching `main_entry`.
    * Emit `▶ Run Test` and `Debug Test` on the function matching `test_entry` (and on `public` `test_*` functions in `tests/`).

---

### 2.5 Pillar 5: Strict Load-Time Duplicate Symbol / Override Rejection (`solix-runtime`)

In `crates/solix-runtime/src/context.rs` (`RuntimeContext::new`):
* When linking multiple `BinaryProgram` containers (the primary program + dynamic library dependencies):
  * Maintain three load-time symbol ownership maps:
    1. `defined_functions: HashMap<String, String>` (`function_symbol -> defining_binary_name`)
    2. `defined_classes: HashMap<String, String>` (`class_symbol -> defining_binary_name`)
    3. `defined_statics: HashMap<String, String>` (`static_symbol -> defining_binary_name`)
  * For each binary `bin_idx` (`binary_label = if program.package_name.is_empty() { format!("binary#{bin_idx}") } else { format!("{}@{}", program.package_name, program.package_version) }`):
    * **Functions:** For every `CompiledFunction` in `program.functions` (excluding synthetic module initializers `@init` / `__static_init` which are scoped per module):
      * Check if `defined_functions.contains_key(&func.symbol)`.
      * If already defined by `first_pkg`, immediately return:
        ```rust
        Err(RuntimeError::DuplicateSymbolDefinition {
            kind: "function".to_string(),
            symbol: func.symbol.clone(),
            first_module: first_pkg.clone(),
            second_module: binary_label.clone(),
        })
        ```
      * Otherwise insert `(func.symbol.clone(), binary_label.clone())`.
    * **Classes:** For every `CompiledClass` in `program.classes`:
      * Check if `defined_classes.contains_key(&class.symbol)`.
      * If already defined by `first_pkg`, immediately return `Err(RuntimeError::DuplicateSymbolDefinition { kind: "class", ... })`.
    * **Static Globals:** For every `static_sym` in `program.static_symbols`:
      * Check if `defined_statics.contains_key(static_sym)`.
      * If already defined by `first_pkg`, immediately return `Err(RuntimeError::DuplicateSymbolDefinition { kind: "static variable", ... })`.
* Add `RuntimeError::DuplicateSymbolDefinition` to `crates/solix-runtime/src/exceptions/errors.rs` and `RuntimeErrorCode::DuplicateSymbolDefinition = 24` (`R0024`) to `codes.rs`.

---

## 3. Phased Implementation Plan

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Bytecode Metadata (`is_public`, `return_type`, `param_types`, `entry`)  │
│          & `ModuleSolver::list_packages` (`solix-common` & `solix-compilation`)  │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Multi-Entrypoint `VirtualMachine` Lifecycle (`initialize_statics`,      │
│          `invoke_entry`, `shutdown`/`Drop`) & Duplicate Symbol Rejection (`R0024`)│
├──────────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Strict CLI Entrypoint Resolution, `solix_toolbox.py` Fixes (`--entry`,  │
│          `SOLIX_LINK_MODE`) & Template `::` Remediation (`solix-toolchain`)      │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Solver-Backed `import` / `extern import` Completion & Manifest-Aware    │
│          Emoji-Free CodeLens (`solix-lsp`)                                       │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Integrated Terminal TCP DAP (`solix dap --port <port>`) &               │
│          `solix.showReferences` in VS Code (`solix-dap` & `editors/vscode`)      │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Rebuild `solixlib`, Run Full Test Matrix & Sync Docs (`GL-0013`)        │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Bytecode Metadata & `ModuleSolver::list_packages()` (`solix-common` & `solix-compilation`)
* [ ] `[P0]` **Task 1.1:** Add `fn list_packages(&self) -> Vec<String>` to `ModuleSolver`, `ClientModuleSolver`, and `InMemoryModuleSolver` in `crates/solix-common/src/solver.rs`.
* [ ] `[P0]` **Task 1.2:** Add `is_public: bool`, `return_type: Type`, and `parameter_types: Vec<Type>` to `CompiledFunction` (`crates/solix-common/src/bytecode/function.rs`), and `entry_point: Option<String>` to `ExportManifest` and `BinaryProgram` (`manifest.rs`, `binary.rs`).
* [ ] `[P0]` **Task 1.3:** Update `EmitterPhase` (`crates/solix-compilation/src/phases/emitter/pipeline.rs`) to populate `is_public`, `return_type`, and `parameter_types` on every emitted `CompiledFunction`, qualify top-level free functions with their file/declaration package prefix (`<pkg>::<name>` when in a non-empty package), and store `options.entry_point` into `program.entry_point`.

### Phase 2: Multi-Entrypoint VM Lifecycle & Load-Time Symbol Collision Hardening (`solix-runtime`)
* [ ] `[P0]` **Task 2.1:** Add `RuntimeError::DuplicateSymbolDefinition` (`R0024`) to `crates/solix-runtime/src/exceptions/errors.rs` and `codes.rs`.
* [ ] `[P0]` **Task 2.2:** In `RuntimeContext::new` (`crates/solix-runtime/src/context.rs`), enforce load-time duplicate symbol detection across all loaded binaries for functions, classes, and static globals (allowing identical monomorphized generic instantiations `<...>` to deduplicate cleanly while rejecting conflicting concrete definitions with `DuplicateSymbolDefinition`).
* [ ] `[P0]` **Task 2.3:** Refactor `VirtualMachine` (`crates/solix-runtime/src/vm/engine.rs`):
  - Add `statics_initialized: bool` and `is_shut_down: bool`.
  - Implement `pub fn initialize_statics(&mut self) -> Result<(), RuntimeError>` that runs all module `@init` functions once and preserves `self.static_slots`.
  - Implement `pub fn invoke_entry(&mut self, symbol: &str, args: &[Value]) -> Result<Value, RuntimeError>` that auto-initializes statics if needed, validates `is_public` and primitive parameter/return types, invokes `symbol` while preserving `self.static_slots` and `self.heap` across multiple calls, and returns the resulting `Value`.
  - Implement `pub fn shutdown(&mut self) -> Result<(), RuntimeError>` and `impl Drop for VirtualMachine` to deterministically release all `Value::Ref` entries in `self.static_slots` (running their `~ClassName()` destructors) and clean up heap allocations.
  - Implement `pub fn validate_cli_entry_point(&self, func_id: u32) -> Result<(), RuntimeError>` enforcing `public` visibility, `int32` (or `void`) return type, and `()` or `(char[] / string[])` parameters for CLI execution.

### Phase 3: CLI Entrypoint Contract, Toolbox Fixes & Template `::` Cleanup (`solix-toolchain`)
* [ ] `[P0]` **Task 3.1:** Add `--entry <SYMBOL>` to `solix compile` (`crates/solix-toolchain/src/cli.rs` and `commands/compile.rs`) and wire it to `CompilationOptions::with_entry_point`.
* [ ] `[P0]` **Task 3.2:** Update `crates/solix-toolchain/src/commands/run.rs` to resolve the target entrypoint using the strict 4-tier priority (`--entry` $\to$ `manifest.toml` `[entrypoints]` $\to$ `.slxb` `program.entry_point` $\to$ `"main"`) and call `vm.validate_cli_entry_point(entry_id)?` before executing.
* [ ] `[P0]` **Task 3.3:** Fix `solix_toolbox.py` (in `crates/solix-toolchain/assets/scripts/solix_toolbox.py`, `assets/templates/untitled-rust/scripts/solix_toolbox.py`, and `libraries/solixlib/scripts/solix_toolbox.py`):
  - Read `os.environ.get("SOLIX_LINK_MODE")` and pass `--link <mode>` to `solix compile` (`BUG-02`).
  - Pass `--entry <manifest_entry>` to `solix compile` (`BUG-03`).
* [ ] `[P0]` **Task 3.4:** Fix all legacy `.` static method calls (`MathBridge.fast_add` $\to$ `MathBridge::fast_add`) in `assets/templates/untitled-rust/src/main.slx`, `tests/test_main.slx`, `assets/scripts/init_project.py`, and `crates/solix-toolchain/tests/build_and_archive_tests.rs` (`BUG-01`).

### Phase 4: Solver-Backed Import Autocompletion & Manifest-Aware CodeLens (`solix-lsp`)
* [ ] `[P0]` **Task 4.1:** Implement `PackageNamespaceIndex` caching in `WorkspaceManager` (`crates/solix-lsp/src/workspace.rs`) backed by `ModuleSolver::list_packages()`, `ExportedModuleAst`, `blt`, and workspace snapshots.
* [ ] `[P0]` **Task 4.2:** Implement `handle_import_completion` in `crates/solix-lsp/src/completion.rs`:
  - First segment (`import <cursor>` or `extern import <cursor>`): suggests all packages from `solver.list_packages()`, `blt`, and workspace package roots.
  - Inner segments (`import solix.<cursor>` or `import solix.io.<cursor>`): suggests child subpackages (`io`, `math`, `core`, `collections`, etc.), public classes/interfaces/enums/functions in that namespace (`Console;`, `File;`, etc.), and wildcard `*;`.
* [ ] `[P0]` **Task 4.3:** Update `crates/solix-lsp/src/code_lens.rs`:
  - Remove `🐞` emoji (`"Debug Main"`, `"Debug Test"`).
  - Resolve project `[entrypoints].main` and `[entrypoints].test` from `manifest.toml` (falling back to `"main"` for standalone files) so Run/Debug CodeLenses appear on the exact configured entrypoint.

### Phase 5: Integrated Terminal DAP (`solix dap --port`) & VS Code Polish (`solix-dap` & `editors/vscode`)
* [ ] `[P0]` **Task 5.1:** Update `crates/solix-dap/src/server.rs`:
  - Use `vm.initialize_statics()` instead of manual init frame loops (`BUG-04`).
  - When running in TCP mode (`port.is_some()`), allow guest `stdin`/`stdout`/`stderr` to flow directly to the process's real terminal PTY instead of intercepting `stdout` into `pending_console_outputs`.
  - Resolve entrypoints using the same manifest/binary metadata rules as `solix run`.
* [ ] `[P0]` **Task 5.2:** Update `editors/vscode/src/debug.ts`, `src/commands.ts`, and `src/extension.ts`:
  - Launch `solix dap --port <ephemeral_port>` inside the VS Code Integrated Terminal (`Solix Debug Terminal`) and connect via `new vscode.DebugAdapterServer(port, '127.0.0.1')` so interactive `stdin` and `stdout` work natively in the terminal!
  - Register `solix.showReferences` in `src/commands.ts` (`BUG-06`).
  - Remove legacy `solix-lsp` binary search fallback in `src/extension.ts` (`BUG-05`) and remove `🐞` emoji from command labels.

### Phase 6: Rebuild `solixlib`, Exhaustive Conformance Suite & Pre-Merge Documentation Sync (`GL-0013`)
* [ ] `[P0]` **Task 6.1:** Rebuild and sync `libraries/solixlib` (`solix build libraries/solixlib --release --sync`) so `solix@2.0.0.slxarc` includes the enriched `CompiledFunction` metadata (`is_public`, `return_type`, `parameter_types`).
* [ ] `[P0]` **Task 6.2:** Add comprehensive unit and integration tests across `solix-runtime`, `solix-lsp`, `solix-dap`, `solix-toolchain`, and `editors/vscode`:
  1. `test_import_first_identifier_and_nested_namespace_completion` (`solix-lsp`).
  2. `test_multiple_entrypoints_share_static_state_and_raii_shutdown` (`solix-runtime`).
  3. `test_non_public_or_non_primitive_entrypoint_rejected` (`solix-runtime`).
  4. `test_cli_entrypoint_resolution_priority_and_signature_enforcement` (`solix-toolchain`).
  5. `test_load_time_duplicate_symbol_definition_rejected` (`solix-runtime`).
  6. `test_dap_tcp_port_mode_and_terminal_io` (`solix-dap` & `editors/vscode`).
* [ ] `[P1]` **Task 6.3:** Execute the `GL-0013` Pre-Merge Git-Diff Documentation Audit and update `docs/wiki/errors/runtime-errors.md` (`R0020`, `R0024`), `docs/spec/vm-isa-and-execution.md`, `docs/spec/package-solver-and-linking.md`, and `docs/guide/editors_vscode.md`.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Solver-Backed Import Completion** | `import ` suggests solver packages; `import solix.` suggests `io`, `math`, `core`; `import solix.io.` suggests `Console;` | `cargo test -p solix-lsp` | [ ] Pending |
| **Integrated Terminal DAP** | `solix dap --port <port>` runs in VS Code Integrated Terminal with live `stdin`/`stdout` | `cargo test -p solix-dap && cd editors/vscode && npm test` | [ ] Pending |
| **Emoji-Free CodeLens & UI** | CodeLens emits `"Debug Main"` & `"Debug Test"`; `solix.showReferences` registered | `cargo test -p solix-lsp && cd editors/vscode && npm test` | [ ] Pending |
| **Modular Multi-Entrypoint VM** | Multiple entrypoints share `static_slots` in one `VirtualMachine` & run RAII destructors on `Drop` | `cargo test -p solix-runtime` | [ ] Pending |
| **Strict CLI Entrypoint Contract** | Enforces `public int32 <sym>(char[]/string[])` & resolves `--entry` $\to$ `manifest.toml` $\to$ `.slxb` $\to$ `main` | `cargo test -p solix-toolchain` | [ ] Pending |
| **Duplicate Symbol Rejection (`R0024`)** | Loading two binaries defining the same function/class/static fails with `DuplicateSymbolDefinition` | `cargo test -p solix-runtime` | [ ] Pending |
| **Zero Warnings & Clean VSIX** | `cargo clippy --workspace --all-targets -- -D warnings` & `npm run package` pass with 0 warnings | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |