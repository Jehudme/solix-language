# `solix install` — Package Installation

The `install` subcommand installs compiled Solix binary containers (`.slxb`) or archive packages (`.slxarc`) into the local package store, making them available for dependency resolution across Solix projects.

---

## Synopsis

```bash
solix install <path> [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<path>` | Path | **Yes** | Path to the `.slxb` binary container or `.slxarc` archive file to install. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store. Can also be set via `SOLIX_PKG_DIR`. |

---

## Installation Mechanics

### 1. Store Directory Resolution

The local package store directory is resolved in the following priority:
1. Explicit `--pkg-dir <path>` command-line option.
2. `SOLIX_PKG_DIR` environment variable.
3. System default directory:
   - **Linux / macOS**: `~/.solix/packages/`
   - **Windows**: `%USERPROFILE%\.solix\packages\`

### 2. Binary Container (`.slxb`) Installation

When installing a `.slxb` file:
1. Solix verifies the container header and decodes binary metadata (`package_name`, `package_version`).
2. Validates that `package_name` is non-empty and `package_version` conforms to Semantic Versioning (`major.minor.patch`).
3. Creates destination directory: `<pkg_dir>/<package_name>/<version>/`.
4. Copies the binary file into `<pkg_dir>/<package_name>/<version>/<package_name>.slxb`.

```bash
# Install a compiled library binary
solix install build/release/math_utils.slxb
```

### 3. Archive Container (`.slxarc`) Installation

When installing a `.slxarc` archive:
1. Solix inspects the archive container and extracts the root `manifest.toml`.
2. Validates package metadata (`name`, `version` in `[info]`).
3. Creates destination directory: `<pkg_dir>/<package_name>/<version>/`.
4. Copies the complete archive container into `<pkg_dir>/<package_name>/<version>/<package_name>.slxarc`.
5. Unpacks bundled `.slxb` binaries, staging headers, and native shared libraries into the solver store and native cache.

```bash
# Install a packaged distribution archive
solix install build/release/raylib_solix.slxarc
```

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Package successfully validated and installed into the local store. |
| `1` | Error | Target file not found, malformed container, missing `manifest.toml`, or write permission failure. |
