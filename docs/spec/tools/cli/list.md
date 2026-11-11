# `solix list` — Local Package Inventory

The `list` subcommand scans and reports all packages and versions installed in the local Solix package store.

---

## Synopsis

```bash
solix list [options]
```

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--json` | | Flag | `false` | Output the inventory as machine-readable JSON. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store. Can also be set via `SOLIX_PKG_DIR`. |

---

## Output Formats

### 1. Human-Readable Table (Default)

By default, `solix list` prints a formatted table listing each discovered package, its installed versions, package format (`.slxb` or `.slxarc`), and file location:

```text
Installed Solix Packages:
--------------------------------------------------------------------------------
Package              Version        Format     Location
--------------------------------------------------------------------------------
raylib               1.0.0          slxarc     ~/.solix/packages/raylib/1.0.0/raylib.slxarc
math_utils           0.2.1          slxb       ~/.solix/packages/math_utils/0.2.1/math_utils.slxb
--------------------------------------------------------------------------------
Total: 2 package(s) installed.
```

### 2. JSON Output (`--json`)

When invoked with `--json`, `solix list` outputs structured JSON suitable for CI scripts and IDE tooling:

```bash
solix list --json
```

```json
[
  {
    "name": "raylib",
    "version": "1.0.0",
    "format": "slxarc",
    "path": "/home/user/.solix/packages/raylib/1.0.0/raylib.slxarc"
  },
  {
    "name": "math_utils",
    "version": "0.2.1",
    "format": "slxb",
    "path": "/home/user/.solix/packages/math_utils/0.2.1/math_utils.slxb"
  }
]
```

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Package store successfully scanned and inventory displayed. |
| `1` | Error | Package store directory inaccessible or unreadable. |
