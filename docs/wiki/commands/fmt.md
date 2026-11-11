# CLI Reference: `fmt`

The `solix fmt` subcommand formats Solix source files (`*.slx`) adhering to the official Solix style specification (4-space indentation, normalized brace positioning, preserved inline/doc comments, and space collapse).

---

## 1. Synopsis

```bash
solix fmt [files...] [options]
```

---

## 2. Key Options & Flags

* `[files...]`: List of `.slx` source files or project directories to format. If omitted, recursively formats all `.slx` files in the current directory and its subdirectories.
* `--check`: Runs formatter in verification mode. Exits with status code `1` if any file requires formatting without writing changes to disk (ideal for CI pipelines).
* `-h, --help`: Prints command help information.

---

## 3. Formatting Rules

1. **Indentation**: Exactly 4 spaces per indentation level. Tabs are automatically converted to spaces.
2. **Braces**: Opening braces `{` follow standard Solix syntax formatting; closing braces `}` align with the declaration or enclosing statement level.
3. **Trailing Whitespace**: All trailing whitespace at the end of lines is stripped.
4. **Consecutive Spaces**: Redundant consecutive spaces outside string literals and comments are collapsed to a single space.
5. **Comments Preservation**: Inline comments (`//`) and multiline doc comments (`///` and `/** */`) are preserved verbatim with matching block indentation.
6. **Trailing Newline**: Files are guaranteed to terminate with exactly one trailing newline `\n`.

---

## 4. Practical Examples

```bash
# Format all Solix source files in the current workspace
solix fmt

# Format specific source files
solix fmt src/main.slx src/engine/renderer.slx

# Verify formatting compliance in Continuous Integration (CI)
solix fmt --check
```

---

## 5. Exit Codes

* `0`: Success. All files formatted, or in `--check` mode, all files are already properly formatted.
* `1`: Formatting violations detected in `--check` mode, or I/O error reading/writing source files.

---

## 6. Related Documentation

* [Language Server Command Reference](file:///home/jehud/Projects/solix-language/docs/wiki/commands/lsp.md)
* [Solix Getting Started Guide](file:///home/jehud/Projects/solix-language/docs/guide/01_getting_started.md)
