# `solix doc` — Static API Documentation Generator

The `doc` subcommand extracts doc comments (`///` and `/** ... */`) and structured doc tags (`@param`, `@return`, `@throws`, `@deprecated`) from Solix source files, generating searchable, responsive static HTML documentation or clean Markdown documentation.

---

## Synopsis

```bash
solix doc [path] [options]
```

---

## Positional Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `[path]` | Path | `.` (Current Directory) | Project root containing `manifest.yaml` (or legacy `manifest.toml`) or directory containing `.slx` source files. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <dir>` | `-o` | Path | `docs/api` | Directory where generated documentation artifacts are written. |
| `--format <format>` | `-f` | String | `"html"` | Output documentation format: `html` (responsive web app) or `markdown` (`md`). |
| `--open` | | Flag | `false` | Automatically opens `index.html` in the user's default web browser upon completion. |
| `--profile <mode>` | `-p` | String | `"release"` | Target build profile to resolve dependencies and source manifests. |

---

## Supported Doc Comments & Tags

`solix doc` parses both single-line triple-slash (`///`) doc comments and multi-line C-style (`/** ... */`) block doc comments attached to declarations.

### Structured Tags

| Tag | Syntax | Description |
|---|---|---|
| `@param` | `@param <name> <description>` | Documents formal parameter name and description. Rendered in parameter tables and LSP signature help tooltips. |
| `@return` | `@return <description>` | Documents method or function return value semantics. |
| `@throws` | `@throws <type> <description>` | Documents exceptional conditions and error types thrown by the routine. |
| `@deprecated` | `@deprecated <reason>` | Marks the symbol as deprecated with an explanation and migration advice. |

### Source Example

```solix
package math;

/// Fast mathematical utilities for vector processing.
public class VectorMath {
    /// Computes the dot product of two 3D vectors.
    ///
    /// @param vector_a First operand vector
    /// @param vector_b Second operand vector
    /// @return The scalar dot product of vector_a and vector_b
    /// @throws MathError If vector dimensions mismatch
    public static float64 dot(Vector3 vector_a, Vector3 vector_b) {
        return vector_a.x * vector_b.x + vector_a.y * vector_b.y + vector_a.z * vector_b.z;
    }
}
```

---

## Output Formats

### 1. HTML Documentation (`--format html`)

Generates a static documentation bundle including:
- `index.html`: Responsive documentation portal with quick navigation.
- Package and class pages detailing member fields, methods, signatures, and structured doc tags.
- Responsive styling supporting both dark and light reader themes.
- Client-side navigation sidebar for rapid symbol lookup.

### 2. Markdown Documentation (`--format markdown`)

Generates pure Markdown files (`.md`) formatted for GitHub wikis, Git repositories, or documentation static site generators (such as MkDocs or mdBook):
- `index.md`: Table of contents and symbol index.
- `<Package>.<Symbol>.md`: Individual symbol markdown pages with tables for `@param`, `@return`, and `@throws`.

---

## Examples

### 1. Default HTML Generation
```bash
solix doc
```

### 2. Generate and Open in Browser
```bash
solix doc --open
```

### 3. Generate Markdown Documentation to Custom Directory
```bash
solix doc --format markdown --output docs/reference
```
