# `solix fmt` — Source Code Formatter

The `fmt` subcommand formats Solix source files (`*.slx`) deterministically using 4-space indentation, standardized operator and brace spacing, inline/doc comment preservation, consecutive space collapse outside literals/comments, and single trailing newlines.

---

## Synopsis

```bash
solix fmt [paths...] [options]
```

---

## Positional Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[paths...]` | Paths | No | `.` (Current Directory) | Target `.slx` source files or directories to scan recursively (skipping `.git`, `build`, `target`, `.cache`). |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--check` | | Flag | `false` | Checks formatting compliance without modifying files and exits with status `1` if any file needs formatting. |
| `--help` | `-h` | Flag | `false` | Prints CLI help summary. |

---

## Formatting Rules

1. **Indentation:** 4 spaces per structural brace nesting level (tabs are normalized to spaces).
2. **Braces & Mid-Line Continuation Blocks:** Opening braces stay attached cleanly; `} else {`, `} else if (...) {`, `} catch (...) {`, and `} finally {` align with the opening control-flow block.
3. **Literal & Comment Safety:** Spaces, tabs, and braces inside strings (`"..."`), characters (`'...'`), line comments (`// ...`), and block/doc comments (`/* ... */`, `/** ... */`) are preserved verbatim.
4. **Trailing Newlines:** Formatted files end with a single `\n` (empty files remain empty).

---

## Practical Examples

```bash
# Format every .slx file in the current workspace
solix fmt

# Format specific source directories or files
solix fmt src/ tests/

# Verify formatting in Continuous Integration (exits non-zero on drift)
solix fmt --check
```

---

## Exit Codes

| Exit Code | Meaning |
|---|---|
| `0` | Success (all files formatted cleanly, or compliant under `--check`). |
| `1` | One or more files require reformatting under `--check`, a target path could not be read, or writing failed. |
