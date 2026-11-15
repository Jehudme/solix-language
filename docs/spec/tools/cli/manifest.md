# Solix Project Manifest (`manifest.toml`) Specification

The `manifest.toml` file is the declarative configuration descriptor for Solix projects, applications, and libraries. It defines package identity, author attribution, application and test entrypoints, and dependency requirements.

---

## Schema Overview

A complete `manifest.toml` document structure:

```toml
[info]
name = "my_project"
version = "0.1.0"
description = "High-performance Solix application with native bindings"
license = "Apache-2.0"
authors = ["Jane Developer <jane@example.com>", "Solix Core Team"]

[entrypoints]
main = "Main::main"
test = "TestRunner::test"

[profile.default]
opt_level = 0
inline_threshold = 8
debug_symbols = true

[profile.release]
opt_level = 3
debug_symbols = false

[dependencies]
solixlib = "2.0.0"
raylib = { version = ">=1.0.0", profile = "release" }
math_matrix-debug = "0.3.1"
```

---

## Section & Field Specifications

### 1. `[info]` Table (Required)

Contains foundational metadata identifying the package and its authors.

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | String | **Yes** | — | Unique package identifier. Must be a valid identifier (alphanumeric, underscores, hyphens). |
| `version` | String | **Yes** | `"0.1.0"` | Semantic version string adhering to SemVer 2.0 (`MAJOR.MINOR.PATCH`). |
| `description` | String | No | `""` | Human-readable summary of the package. |
| `license` | String | No | `"Apache-2.0"` | SPDX license identifier (e.g., `Apache-2.0`, `MIT`). |
| `authors` | Array of Strings | No | `[]` | List of author names or name/email pairs. |

```toml
[info]
name = "network_engine"
version = "1.2.0"
description = "Asynchronous networking primitives for Solix"
license = "MIT"
authors = ["Jane Developer"]
```

---

### 2. `[entrypoints]` Table (Optional for Libraries, Recommended for Apps)

Defines canonical execution entry points for running the program and its test harness.

| Field | Type | Default | Description |
|---|---|---|---|
| `main` | String | `"Main::main"` | Canonical fully-qualified method name invoked when executing the application (`solix run <target.slxarc>`). |
| `test` | String | `"TestRunner::test"` | Fully-qualified test runner method executed when invoking with `--test` (`solix run --test <target.slxarc>`). |

```toml
[entrypoints]
main = "App::main"
test = "SuiteRunner::run_tests"
```

> [!NOTE]
> Entry point signatures must return an integer (`int32` or `int64`) representing the OS process exit code, or return `void`.

---

### 3. `[dependencies]` Table (Optional)

Defines external package dependencies resolved against the local Solix package store (`~/.solix/packages`).

Dependencies are strictly declared using table notation:

```toml
[dependencies]
raylib = { version = "5.5.0", profile = "debug" }
solixlib = { version = "2.0.0", profile = "release" }
math_matrix = { version = "0.3.1", profile = "release" }
```

#### Table Specification Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `version` | String | *(Required)* | SemVer version requirement string (e.g. `"5.5.0"`, `"^1.2.0"`, `"*"`). |
| `profile` | String | `"release"` | Target build profile to resolve against the package repository or store (`"release"`, `"debug"`, or `"test"`). |

#### Version Requirement Syntax

| Syntax | Type | Matches | Example |
|---|---|---|---|
| `"1.2.3"` | Exact / Compatible | `>= 1.2.3, < 2.0.0` (SemVer caret semantics) | `"1.2.3"` matches `1.2.4`, rejects `2.0.0` |
| `"^1.2.0"` | Caret Requirement | Compatible non-breaking updates | `"^1.2.0"` matches `1.3.0`, rejects `2.0.0` |
| `">=1.0.0"` | Comparison Range | Greater than or equal to version | `">=1.0.0"` matches any version `>= 1.0.0` |
| `"*"` | Wildcard | Any available installed version | `"*"` matches any installed version |

> [!NOTE]
> Dependency declarations in `manifest.toml` strictly require table notation: `package_name = { version = "<version>", profile = "<profile>" }`. Shorthand string syntax and suffix notation are rejected by the manifest parser.

---

### 4. `[profile.<name>]` Tables (Hierarchical Profile System)

Defines granular compilation, optimization, and code emission options per profile with single inheritance:

```toml
[profile.default]
opt_level = 0
inline_threshold = 8
debug_symbols = true
emit_disassembly = false
warnings_as_errors = false
verify_checksum = true
link = "dynamic"

[profile.release]
opt_level = 3
debug_symbols = false

[profile.bench]
inherits = "release"
opt_level = 3
inline_threshold = 32
memory_limit_mb = 1024

[profile.ci]
inherits = "bench"
warnings_as_errors = true
```

#### Profile Keys & Inheritance

| Key | Type | Default | Description |
|---|---|---|---|
| `inherits` | String | `None` | Parent profile name to inherit options from (e.g. `"default"`, `"release"`). |
| `opt_level` | Integer | `0` (debug) / `3` (release) | Bytecode optimizer level (0..=3). |
| `inline_threshold` | Integer | `8` | Maximum statement count for AST inlining. |
| `debug_symbols` | Boolean | `true` (debug) / `false` (release) | Emits source line mappings and debug symbols in `.slxb`. |
| `emit_disassembly` | Boolean | `false` | Emits human-readable register VM disassembly files during compilation. |
| `warnings_as_errors`| Boolean | `false` | Treats compiler warnings as fatal errors. |
| `verify_checksum` | Boolean | `true` | Validates dependency cryptographic SHA-256 hashes against pinned manifests. |
| `link` | String | `"dynamic"` | Linking mode for dependencies: `"dynamic"` or `"static"`. |
| `memory_limit_mb` | Integer | `None` | Maximum virtual heap allocation limit in megabytes. |

---

### 5. `[build]` Table (Optional / Backward Compatibility)

Configures default compilation, linking, inlining, and package store behavior when executing `solix build` or invoking build scripts:

```toml
[build]
profile = "release"           # Default target profile: "debug", "release", or "test"
link = "dynamic"              # Linking mode: "dynamic" (default) or "static"
pkg_dir = ""                  # Optional package store override path (defaults to ~/.solix/packages)
install = false               # Automatically install archive into store upon successful build
force_install = false         # Force overwrite if package is already installed
verify_checksum = true        # Cryptographically verify SHA-256 dependency digests
inline_threshold = 8          # Statement threshold limit for AST function inlining
```

| Key | Type | Default | Description |
|---|---|---|---|
| `profile` | String | `"release"` | Target compilation profile (`"debug"`, `"release"`, `"test"`). Can be overridden by `--profile` CLI flag. |
| `link` | String | `"dynamic"` | Linking mode for dependencies: `"dynamic"` or `"static"`. |
| `pkg_dir` | String | `""` | Package store directory override path. Defaults to `~/.solix/packages`. |
| `install` | Boolean | `false` | When `true`, automatically installs `.slxarc` into the package store after building. |
| `force_install` | Boolean | `false` | When `true`, overwrites existing package versions during installation. |
| `verify_checksum` | Boolean | `true` | Validates dependency cryptographic SHA-256 hashes against pinned manifests. |
| `inline_threshold` | Integer | `8` | Maximum statement count for `AstInlinerPhase` inlining. |

---

### 6. `[native_library_checksums]` Table (Generated / Optional)

Defines cryptographic FIPS 180-4 SHA-256 hex digests for native shared libraries bundled inside `.slxarc` archives. Automatically staged by `scripts/solix_build.py` during packaging to ensure binary tamper resistance across host and cross-compiled desktop target directories (`lib/<target_triple>/`):

```toml
[native_library_checksums]
"lib/libsolixlib_native.so" = "af5f19f16fac4342322dc64262fc55e49b4119122e690957773c78aa25405a44"
"lib/x86_64-unknown-linux-gnu/libsolixlib_native.so" = "af5f19f16fac4342322dc64262fc55e49b4119122e690957773c78aa25405a44"
```

When loading native dynamic libraries into the virtual machine (`RuntimeContext::new` or `SlxarcArchive::extract_and_load`), the runtime validates that the extracted library's SHA-256 digest matches this pin. Any mismatch halts execution unless `--no-verify` (`--ignore-checksum`) is passed.

---

## Directory Conventions Around `manifest.toml`

A standard Solix project structured around `manifest.toml`:

```text
my_project/
├── manifest.toml             # Authoritative configuration descriptor
├── .gitignore                # Ignores /build/, /target/, *.slxb, *.slxarc
├── src/
│   └── main.slx              # Application source code
├── tests/
│   └── test_main.slx         # Automated test suite
├── resources/                # Embedded static assets (images, audio, config)
├── lib/                      # Precompiled native shared libraries (.so / .dll / .dylib)
├── natives/                  # Rust cdylib bridge extension crates
│   └── my_project_bridge/
│       ├── Cargo.toml
│       └── src/lib.rs
└── scripts/
    ├── build.py              # Build automation entrypoint
    └── solix_build.py        # Build engine helper module
```

---

## Role in Build & Distribution Lifecycle

1. **Scaffolding (`solix new`)**: Creates `manifest.toml` populated with CLI arguments (`--name`, `--version`, `--license`, `--author`, `--deps`).
2. **Building (`solix build`)**: `scripts/build.py` via `solix_build.py` reads `manifest.toml`, extracts metadata and dependencies, forwards configured entrypoints (`--entry`) and linking modes (`SOLIX_LINK_MODE`), invokes `solix compile` (resolving `solix` or `solix.exe` cross-platform, configurable via `SOLIX_BIN` and `PYTHON`), manages native Cargo target path delimiters via `os.pathsep` across both MSVC and GNU Windows platforms, and packages `manifest.toml` at the root of the resulting `.slxarc` archive.
3. **Execution (`solix run`)**: When running a `.slxarc` archive, the VM runtime reads `manifest.toml` from the archive container to determine entry points (`[entrypoints].main` or `[entrypoints].test`), validate visibility/return signatures, and resolve bundled resources.
4. **Inspection (`solix info`)**: Reads and formats `manifest.toml` when displaying container metadata.
