# `solix new` — Project & Native Subproject Scaffolding

The `new` subcommand initializes Solix project workspaces configured with `manifest.yaml` and provides `solix new native <name>` for scaffolding dual-mode C, C++, or Rust native libraries directly in Rust with zero Python build scripts.

---

## Synopsis

```bash
solix new [path] [options]
solix new package <path> [options]
solix new native <name> [options]
```

---

## Subcommand 1: `solix new package <path>` (Project Scaffolding)

### Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<path>` | Path | **Yes** | Destination directory path for the new project. Created if it does not exist. |

### Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--name <name>` | | String | Target directory name | Project and package identifier name. |
| `--version <version>` | | String | `"0.1.0"` | Initial semantic version for the package. |
| `--description <desc>` | | String | `"A Solix project"` (`"A Solix package"` for `new package`) | Human-readable package description. |
| `--license <id>` | | String | `"Apache-2.0"` | SPDX license identifier. |
| `--author <name>` | | String (Repeated) | `[]` | Author attribution name (can be repeated). |
| `--main <symbol>` | | String | `"Main::main"` | Canonical application entry point symbol. |
| `--test <symbol>` | | String | `"TestRunner::test"` | Canonical test runner entry point symbol. |
| `--lib` | | Flag | `false` | Scaffolds a reusable library project (`src/lib.slx`) instead of an executable application. |
| `--native [LANG]` | | String (Optional Value) | `"rust"` | Scaffolds a dual-mode native bridge in `natives/` (`rust`, `c`, or `cpp`). |
| `--no-native` | | Flag | `false` | Omits the native bridge subproject and creates an empty `natives/.gitkeep`. |
| `--no-std` | `--no-default-deps` | Flag | `false` | Omits the automatic standard library dependency (`solix`). |
| `--deps <pkg@ver>` | `-d` | String (Repeated) | `[]` | Declares a `profiles.release.dependencies` package requirement (e.g. `-d raylib@5.5.0`). |
| `--force` | | Flag | `false` | Overwrites an existing `manifest.yaml` (or `manifest.toml`) if present. |

---

## Subcommand 2: `solix new native <name>` (Dual-Mode Native Bridge Scaffolding)

Creates a standalone and Solix-compatible native library in C (CMake C11), C++ (CMake C++17), or Rust (Cargo), optionally registering it inside the parent project's `manifest.yaml`.

### Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | String | **Yes** | Canonical native module name (folder name and default Solix class prefix). |

### Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--project <path>` | `-p` | Path | `.` (if `manifest.yaml` exists and `--output` omitted) | Existing Solix project root where the native crate is placed under `natives/` and registered in `manifest.yaml`. |
| `--name <pkg>` | | String | snake_case `<name>` | Explicit Cargo `[package] name` or CMake `project(...)` library identifier. |
| `--version <ver>` | | String | `"0.1.0"` | Initial semantic version stamped into `Cargo.toml` or `CMakeLists.txt`. |
| `--lang <lang>` | `--backend` | String | `"rust"` | Backend implementation language: `rust`, `c`, or `cpp` (`c++` / `cxx`). |
| `--output <dir>` | `-o` | Path | `<project>/natives/<name>` or `./<name>` | Custom directory where native source files are written. |
| `--platforms <list>` | | Comma-separated / Repeated | `[]` | Target OS/architecture platforms recorded under the native entry in `manifest.yaml` (`host`, `linux`, `windows`, `macos`, `all`). |
| `--profile <list>` | | Comma-separated / Repeated | `release,debug` (existing) | Profile names in `manifest.yaml` to register (`release`, `debug`, `test`, or `all`). |
| `--description <text>` | | String | `"Dual-mode native library for Solix"` | Description written to `Cargo.toml` or `CMakeLists.txt`. |
| `--license <spdx>` | | String | `"Apache-2.0"` | SPDX license identifier for `Cargo.toml`. |
| `--author <name>` | | String (Repeated) | `[]` | Author name written to `Cargo.toml`. |
| `--no-register` | | Flag | `false` | Scaffolds the native directory without editing `manifest.yaml`. |
| `--force` | | Flag | `false` | Overwrites existing native files in the target directory. |

### Dual-Mode CMake & Cargo Behavior
* **C / C++ (CMake):** Generated `CMakeLists.txt` defines `option(SOLIX_NATIVE_BUILD ... OFF)`. When built directly via `cmake -B build && cmake --build build`, it produces a standalone C/C++ library with CTest unit tests (`ctest`). When built via `solix build`, Solix passes `-DSOLIX_NATIVE_BUILD=ON`, which fetches `solix::ffi` via `FetchContent` (`GIT_TAG dev` or release tag `vX.Y.Z`, or `SOLIX_FFI_SOURCE_DIR` locally) and compiles the `SHARED` Solix plugin.
* **Rust (Cargo):** Generated `Cargo.toml` uses `crate-type = ["cdylib", "rlib"]` with an optional `solix-native = ["dep:solix-ffi"]` feature so `cargo build` / `cargo test` work standalone, while `solix build` automatically passes `--features solix-native`. Generated native entrypoints use `extern "C-unwind"` via `solix_export!` and explicit `*_by_index` struct field accessors for layout-safe FFI interop.

---

## Generated Project Layout

### 1. Application Project (Default)

Executing `solix new my_app`:

```text
my_app/
├── manifest.yaml             # Declarative configuration, profiles, and natives
├── .gitignore                # Ignores /build/, *.slxb, *.slxp
├── src/
│   └── main.slx              # Starter application source
├── tests/
│   └── test_main.slx         # Automated test entrypoint
├── resources/                # Application assets directory
├── bin/                      # Pre-populated platform targets (x86_64, aarch64, windows, macos)
│   ├── x86_64-unknown-linux-gnu/.gitkeep
│   ├── aarch64-unknown-linux-gnu/.gitkeep
│   ├── x86_64-pc-windows-msvc/.gitkeep
│   ├── x86_64-apple-darwin/.gitkeep
│   └── aarch64-apple-darwin/.gitkeep
└── natives/
    └── __native_my_app/      # Dual-mode native bridge (Cargo or CMake with __native_* prefix)
        ├── Cargo.toml        # Declares [lib] name = "__native_my_app" and [features] solix-native
        └── src/lib.rs        # Core Rust API + #[cfg(feature = "solix-native")] bridge
```

#### Generated `manifest.yaml`

```yaml
configuration:
  name: my_app
  version: "0.1.0"
  description: "A Solix project"
  license: "Apache-2.0"
  authors: []

profiles:
  release:
    entrypoint: Main::main
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    sources:
      - src
    platforms:
      - all
    natives:
      my_app_bridge:
        profile: release
    dependencies:
      solix:
        version: "2.2.1"
        profile: release

  debug:
    entrypoint: Main::main
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    sources:
      - src
    platforms:
      - host
    natives:
      my_app_bridge:
        profile: debug
    dependencies:
      solix:
        version: "2.2.1"
        profile: release

  test:
    entrypoint: TestRunner::test
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    sources:
      - src
      - tests
    platforms:
      - host
    natives:
      my_app_bridge:
        profile: debug
    dependencies:
      solix:
        version: "2.2.1"
        profile: release
```

---

## Examples

### Scaffolding an Application with a C++ CMake Bridge
```bash
solix new game_app --native cpp --description "C++ backed Solix app" -d raylib@5.5.0
```

### Adding a Dual-Mode C11 Native Module to an Existing Project
```bash
solix new native fast_audio --project ./game_app --lang c --version 1.0.0 --platforms linux,windows,macos
```
