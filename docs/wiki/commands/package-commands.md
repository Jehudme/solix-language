# CLI Reference: Package Management Commands

This section covers project initialization and package store management commands: `solix new`, `solix install`, `solix uninstall`, and `solix list`.

---

## 1. `solix new`

Scaffolds a complete Solix project workspace with `manifest.toml`, build automation scripts, test suites, and optional native Rust bridge crates:

```bash
solix new <path> [options]
```

### Key Options & Flags
* `<path>`: Target directory for the new project.
* `--name <name>`: Project identifier name [default: directory name].
* `--version <version>`: Initial semantic version [default: `0.1.0`].
* `--description <desc>`: Project description [default: `A Solix project`].
* `--license <id>`: SPDX license identifier [default: `Apache-2.0`].
* `--author <name>`: Author attribution (can be repeated).
* `--main <symbol>`: Main entry point symbol [default: `Main::main`].
* `--test <symbol>`: Test entry point symbol [default: `TestRunner::test`].
* `--lib`: Generates a library project (`src/lib.slx`) instead of an application.
* `--no-native`: Omits the Rust native `cdylib` bridge in `natives/`.
* `-d, --deps <pkg@ver>`: Adds dependency in `pkg@ver` format (can be repeated).
* `--force`: Overwrites existing `manifest.toml` if present in the target directory.

### Practical Examples
```bash
# Scaffold a standard application project
solix new my_app

# Scaffold a reusable library with dependencies
solix new math_matrix --lib --deps "vector_math@0.1.0"

# Scaffold a pure Solix application without Rust native bridge
solix new simple_cli --no-native
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
