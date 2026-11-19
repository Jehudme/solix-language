# CLI Reference: Package Management Commands

This section covers project initialization and package store management commands: `solix new`, `solix install`, `solix uninstall`, and `solix list`.

---

## 1. `solix new` & `solix new native`

### `solix new <path>` (Project Scaffolding)

Scaffolds a complete Solix project workspace configured with `manifest.yaml` (`vars:` anchors, self-contained profiles, pure-Rust `solix build`), test suites, and an optional dual-mode C, C++, or Rust native bridge:

```bash
solix new <path> [options]
```

#### Key Options & Flags
* `<path>`: Target directory for the new project.
* `--name <name>`: Project identifier name [default: directory name].
* `--version <version>`: Initial semantic version [default: `0.1.0`].
* `--description <desc>`: Project description [default: `A Solix project`].
* `--license <id>`: SPDX license identifier [default: `Apache-2.0`].
* `--author <name>`: Author attribution (can be repeated).
* `--main <symbol>`: Main entry point symbol [default: `Main::main`].
* `--test <symbol>`: Test entry point symbol [default: `TestRunner::test`].
* `--lib`: Generates a library project (`src/lib.slx`) instead of an application.
* `--native [LANG]`: Scaffolds a dual-mode native subproject in `natives/` (`rust` [default], `c`, or `cpp`).
* `--no-native`: Omits the native subproject in `natives/`.
* `-d, --deps <pkg@ver>`: Adds dependency in `pkg@ver` format (can be repeated).
* `--force`: Overwrites existing `manifest.yaml` (or legacy `manifest.toml`) if present in the target directory.

### `solix new native <name>` (Native Subproject Scaffolding)

Scaffolds a dual-mode native library in **C11 (CMake)**, **C++17 (CMake)**, or **Rust (Cargo)** that builds both as a standalone library/test suite (`cmake` / `cargo test`) and as a Solix VM plugin (`solix build`), automatically registering it under `natives:` in `manifest.yaml`:

```bash
solix new native <name> [options]
```

#### Key Options & Flags
* `<name>`: Canonical module identifier (directory name and default Solix class prefix).
* `-p, --project <path>`: Existing Solix project root containing `manifest.yaml` [default: `.` when `manifest.yaml` exists and `--output` is omitted].
* `--name <pkg>`: Explicit Cargo package or CMake `project(...)` library name [default: `snake_case(<name>)`].
* `--version <ver>`: Initial semantic version stamped into `Cargo.toml` or `CMakeLists.txt` [default: `0.1.0`].
* `--lang, --backend <lang>`: Native implementation backend: `rust` [default], `c`, or `cpp` (`c++` / `cxx`).
* `-o, --output <dir>`: Custom output directory [default: `<project>/natives/<name>` or `./<name>`].
* `--platforms <list>`: Comma-separated or repeated target platforms recorded on the `natives:` entry (`host`, `linux`, `windows`, `macos`, `all`).
* `--profile <list>`: Manifest profile(s) to register the native subproject under [default: existing `release` / `debug`].
* `--description <text>`: Description written into `Cargo.toml` or `CMakeLists.txt`.
* `--license <spdx>`: SPDX license identifier [default: `Apache-2.0`].
* `--author <name>`: Author name written to `Cargo.toml` (repeatable).
* `--no-register`: Writes the native subproject without modifying `manifest.yaml`.
* `--force`: Overwrites existing files in the native directory.

#### Practical Examples
```bash
# Scaffold a standard application project with a Rust native bridge
solix new my_app

# Scaffold an application with a C++17 CMake native bridge
solix new physics_demo --native cpp

# Add a C11 CMake library directly into an existing project and register it for all desktop platforms
solix new native audio_dsp --project . --lang c --platforms linux,macos,windows

# Scaffold a reusable library with dependencies and no native subproject
solix new math_matrix --lib --no-native --deps "vector_math@0.1.0"
```

---

## 2. `solix install`

Installs a compiled `.slxb` binary container or `.slxarc` archive package into the local package store (`~/.solix/packages`):

```bash
solix install <path> [options]
```

### Key Options & Flags
* `<path>`: Path to the target `.slxb` or `.slxarc` file to install.
* `--bundled`: Installs or re-extracts bundled first-party packages (`solix`, `raylib`) from `assets/packages/`.
* `--vscode`: Automatically installs the bundled official Solix extension into VS Code, VSCodium, or Cursor.
* `-f, --force`: Overwrites existing package version in the store.
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].

### Practical Examples
```bash
# Install the official Solix VS Code / VSCodium / Cursor extension
solix install --vscode

# Re-extract bundled standard packages
solix install --bundled --force

# Install a compiled binary library into the package store
solix install build/release/math_utils.slxb

# Install a packaged distribution archive with native shared libraries
solix install build/release/raylib_solix.slxarc
```

---

## 3. `solix uninstall`

Removes a specific version or all versions of a package from the local package store:

```bash
solix uninstall <spec> [options]
```

### Key Options & Flags
* `<spec>`: Package name (e.g. `raylib` to remove all versions) or package@version (e.g. `raylib@1.0.0` to remove a specific version).
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].

### Practical Examples
```bash
# Remove an exact version of a package
solix uninstall raylib@1.0.0

# Remove all installed versions of a package
solix uninstall raylib
```

---

## 4. `solix list`

Lists all packages and versions installed in the local package store:

```bash
solix list [options]
```

### Key Options & Flags
* `--json`: Emits the package inventory as machine-readable JSON.
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].

### Practical Examples
```bash
# Print formatted tabular list of installed packages
solix list

# Output package inventory as JSON
solix list --json
```

---

## Related Specifications
* Formal Spec: [`docs/spec/tools/cli/new.md`](../../spec/tools/cli/new.md)
* Formal Spec: [`docs/spec/tools/cli/install.md`](../../spec/tools/cli/install.md)
* Formal Spec: [`docs/spec/tools/cli/uninstall.md`](../../spec/tools/cli/uninstall.md)
* Formal Spec: [`docs/spec/tools/cli/list.md`](../../spec/tools/cli/list.md)
* Formal Spec: [`docs/spec/tools/cli/manifest.md`](../../spec/tools/cli/manifest.md)
