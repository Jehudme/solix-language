# `solix uninstall` — Package Removal

The `uninstall` subcommand removes an installed package or a specific package version from the local package store.

---

## Synopsis

```bash
solix uninstall <spec> [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<spec>` | String | **Yes** | Target specification formatted as `<name>[:<profile>][@<version>]` (e.g. `raylib`, `raylib@1.0.0`, `raylib:debug@1.0.0`, or `raylib:debug`). |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store. Can also be set via `SOLIX_PKG_DIR`. |

---

## Uninstallation Mechanics

### 1. Removing a Specific Version

When specifying `<name>@<version>`:
1. Solix locates the target version directory: `<pkg_dir>/<name>/<version>/`.
2. Deletes the version directory and its contents (`.slxb`, `.slxarc`, unpacked headers).
3. If no other versions remain inside `<pkg_dir>/<name>/`, the empty parent package directory is cleaned up.

```bash
# Remove an exact version of a package
solix uninstall raylib@1.0.0
```

### 2. Removing All Installed Versions

When specifying only `<name>`:
1. Solix locates the package directory: `<pkg_dir>/<name>/`.
2. Recursively removes the entire directory tree across all installed versions.

```bash
# Remove all versions of a package
solix uninstall raylib
```

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Package or version successfully uninstalled. |
| `1` | Error | Package not found in local store, or file system permission failure. |
