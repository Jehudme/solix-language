# `solix install` — Package Installation

The `install` subcommand installs compiled Solix binary containers (`.slxb`), archive packages (`.slxarc`), or project source directories into the local package store, making them available for dependency resolution across Solix projects.

---

## Synopsis

```bash
solix install <path> [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<path>` | Path | **Yes** | Path to a `.slxb` binary container, `.slxarc` archive file, or project root directory containing `manifest.toml`. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--force` | `-f` | Flag | `false` | Force overwrite if package version is already installed in the package store. |
| `--profile <prof>` | | String | `"release"` | Target build profile when installing from a project directory (`debug`, `release`, or `test`). |
| `--debug` | | Flag | `false` | Shorthand for `--profile debug`. |
| `--release` | | Flag | `true` | Shorthand for `--profile release`. |
| `--test` | | Flag | `false` | Shorthand for `--profile test`. |
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

### 2. Collision & Overwrite Protection

Solix protects against accidental package overwriting. If `<name>@<version>` is already registered or installed on disk, `solix install` halts with an error instructing the user to supply `--force` (`-f`):

```bash
# Overwrite existing installation
solix install ../libraries/solixlib --release --force
```

### 3. Project Directory Installation

When `<path>` points to a project directory containing `manifest.toml`:
1. Solix builds the project with the requested profile (`solix build <dir> --<profile>`).
2. Locates the emitted `.slxarc` archive in `build/<profile>/<name>.slxarc`.
3. Validates native library SHA-256 checksums and warms the authenticated extract cache at `~/.solix/cache/packages/`.
4. Copies the archive to `<pkg_dir>/<name>/<version>/<name>.slxarc`.
5. Records the canonical `source_project_dir`, `build_profile`, `sha256`, `size_bytes`, and timestamps into `$SOLIX_PKG_DIR/registry.json`.

```bash
solix install ./libraries/solixlib --release
```

### 4. Container Installation (`.slxb` & `.slxarc`)

When installing a `.slxb` or `.slxarc` file directly:
1. Solix verifies the container header, computes the artifact's SHA-256 digest, and parses metadata.
2. If `.slxarc`, extracts and validates bundled native library checksums against `manifest.toml` and warms `~/.solix/cache/packages/`.
3. Copies the container artifact into `<pkg_dir>/<name>/<version>/<name>.<ext>`.
4. Registers the package entry in `registry.json`.

```bash
solix install build/release/raylib.slxarc --force
```
