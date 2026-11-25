# `solix install` — Package Installation

The `install` subcommand installs compiled Solix binary containers (`.slxb`), packages (`.slxp`), or project source directories into the local package store, making them available for dependency resolution across Solix projects.

---

## Synopsis

```bash
solix install package <path> [options]
solix install binaries <path.slxb> [options]
solix install [<path>] [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<path>` | Path | **Yes** (for `package`/`binaries`) | Path to a `.slxp` package, project directory, or `.slxb` standalone binary container. |
| `[path]` | Path | Optional | Path for top-level install fallback; required unless `--bundled` or `--vscode` is specified. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--bundled` | | Flag | `false` | Install or re-extract all bundled first-party default libraries (`solix`, `raylib`) from `assets/packages/`. |
| `--vscode` | | Flag | `false` | Discover installed editor CLIs (`code`, `codium`, `cursor`) and automatically install the bundled official Solix extension (`.vsix`). |
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
solix install ../libraries/solixlib --profile release --force
```

### 3. Project Directory Installation

When `<path>` points to a project directory containing `manifest.yaml` (or legacy `manifest.toml`):
1. Solix builds the project with the requested profile (`solix build <dir> --profile <profile>`).
2. Locates the emitted `.slxp` package in `build/<profile>/<name>.slxp`.
3. Validates native library SHA-256 checksums and warms the authenticated extract cache at `~/.solix/cache/packages/`.
4. Copies the package to `<pkg_dir>/<name>-<profile>@<version>/<name>.slxp`.
5. Records the canonical `source`, `build_profile`, `sha256`, `size_bytes`, and timestamps into `$SOLIX_PKG_DIR/registry.json`.

```bash
solix install package ./libraries/solixlib --profile release
```

### 4. Container Installation (`.slxb` & `.slxp`)

When installing a `.slxb` or `.slxp` file directly:
1. Solix verifies the container header, computes the artifact's SHA-256 digest, and parses metadata.
2. If `.slxp`, extracts and validates bundled native library checksums against `manifest.yaml` (`manifest.toml` for legacy archives) and warms `~/.solix/cache/packages/`.
3. Copies the container artifact into `<pkg_dir>/<name>-<profile>@<version>/<name>.<ext>`.
4. Registers the package entry in `registry.json`.

```bash
solix install package build/release/raylib.slxp --force
```

### 5. Bundled Standard Libraries Installation (`--bundled`)

When invoked with `--bundled`, Solix locates bundled first-party packages (checking `SOLIX_BUNDLED_PACKAGES_DIR`, sibling `<exe_dir>/assets/packages/`, workspace assets, or materializing the compile-time embedded archives into `~/.solix/cache/bundled_packages/`) and installs the `release` profile packages of `solix` and `raylib` into `$SOLIX_PKG_DIR`.

```bash
# Re-extract bundled standard libraries
solix install --bundled --force
```

First-party libraries are also automatically bootstrapped on first run when invoking `solix list`, `solix compile`, or `solix run` against the default package store (unless `SOLIX_NO_AUTO_BOOTSTRAP=1` is set). Custom stores passed via `--pkg-dir` remain isolated and do not auto-bootstrap.

### 6. VS Code Extension Installation (`--vscode`)

When invoked with `--vscode`, Solix locates the bundled `.vsix` extension archive (`SOLIX_VSCODE_VSIX_PATH`, `<exe_dir>/assets/vscode/solix-language.vsix`, workspace assets, or by materializing the compile-time embedded extension into `~/.solix/cache/vscode/solix-language.vsix`), discovers installed editor command-line tools (`code`, `codium`, `cursor`) in the system `PATH`, and executes the extension installation:

```bash
solix install --vscode
```

Pass `--force` (`solix install --vscode --force`) to force overwrite an existing extension installation or allow downgrading.

On Windows, editor CLI commands such as `code` or `cursor` are frequently installed as batch scripts (`code.cmd`) alongside extensionless POSIX shell wrappers. Solix automatically inspects `PATHEXT` to prioritize Windows executable extensions (`.exe`, `.cmd`, `.bat`) over extensionless files, and executes batch scripts via `cmd.exe /c` to prevent Win32 error 193 execution defects.

If no supported editor CLI is found in `PATH`, the command prints the absolute path to the `.vsix` file along with instructions for manual graphical installation.
