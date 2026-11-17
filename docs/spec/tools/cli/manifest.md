# Solix Project Manifest (`manifest.toml`) Specification

The `manifest.toml` file is the declarative configuration descriptor for Solix projects, applications, and libraries. It defines package identity, author attribution, application and test entrypoints, profile-scoped source roots, and dependency requirements.

---

## Schema Overview

A complete `manifest.toml` document structure:

```toml
[configuration]
name = "my_project"
version = "0.1.0"
description = "High-performance Solix application with native bindings"
license = "Apache-2.0"
authors = ["Jane Developer <jane@example.com>", "Solix Core Team"]

[profiles.release]
entrypoint = "Main::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]

[profiles.release.dependencies]
solix = { version = "2.0.0", profile = "release" }
raylib = { version = "5.5.0", profile = "release" }

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false

[profiles.test]
inherits = "debug"
entrypoint = "TestRunner::test"
sources = ["tests"]

[profiles.test.dependencies]
testlib = { version = "1.0.0", profile = "release" }
```

---

## Section & Field Specifications

### 1. `[configuration]` Table (Required)

Contains foundational metadata identifying the package and its authors.

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | String | **Yes** | — | Unique package identifier. Must be a valid identifier (alphanumeric, underscores, hyphens). |
| `version` | String | **Yes** | `"0.1.0"` | Semantic version string adhering to SemVer 2.0 (`MAJOR.MINOR.PATCH`). |
| `description` | String | No | `""` | Human-readable summary of the package. |
| `license` | String | No | `"Apache-2.0"` | SPDX license identifier (e.g., `Apache-2.0`, `MIT`). |
| `authors` | Array of Strings | No | `[]` | List of author names or name/email pairs. |

```toml
[configuration]
name = "network_engine"
version = "1.2.0"
description = "Asynchronous networking primitives for Solix"
license = "MIT"
authors = ["Jane Developer"]
```

---

### 2. `[profiles]` Table & Profile Inheritance (Required)

Profiles govern compiler optimization, AST function inlining, debug symbol emission, linking modes, source directory discovery, and profile-specific dependencies.

#### Mandatory Release Profile & Automatic Inheritance Rules:
1. **Mandatory Profile**: The **only mandatory profile** that must exist in `manifest.toml` is `[profiles.release]`.
2. **Default Parent**: If any profile omits `inherits`, it **automatically defaults to `"release"` as its parent**.
3. **Cumulative Sources**: The `sources` option specifies a list of directory paths (relative to project root) to recursively discover `.slx` source files. Child profiles inherit parent source directories and append additional ones preserving order. For `release`, `sources` defaults to `["src"]`.
4. **Inherited Dependencies**: Child profiles inherit parent dependencies and can add or override dependencies.
5. **Profile Validation**: When requesting a build profile via the CLI or build system, the target profile must either be a standard profile (`release`, `debug`, `test`) or explicitly declared in `[profiles.<name>]`. Requesting an unregistered profile produces an immediate profile resolution error.

#### Profile Options:

| Key | Type | Default | Description |
|---|---|---|---|
| `inherits` | String | `"release"` (or `None` for release) | Parent profile name to inherit settings and dependencies from. |
| `entrypoint` | String | `"Main::main"` | Canonical fully-qualified entry point method. |
| `optimization_level` | Integer | `3` (release) / `0` (debug) | Bytecode optimizer level (0..=3). Alias: `opt_level`. |
| `inline_threshold` | Integer | `16` (release) / `8` (debug) | Maximum statement count for AST inlining. |
| `debug_symbols` | Boolean | `false` (release) / `true` (debug) | Emits source line mappings and debug symbols in `.slxb`. |
| `static` | Boolean | `true` (release) / `false` (debug) | Static linking mode (relocates dependency bytecode). Alias: `is_static`. |
| `sources` | Array of Strings | `["src"]` | Recursive directory paths to discover `.slx` source files. |
| `emit_disassembly` | Boolean | `false` | Emits human-readable register VM disassembly files during compilation. |
| `warnings_as_errors`| Boolean | `false` | Treats compiler warnings as fatal errors. |
| `verify_checksum` | Boolean | `true` | Validates dependency cryptographic SHA-256 hashes against pinned manifests. |
| `link` | String | `"dynamic"` | Linking mode for dependencies: `"dynamic"` or `"static"`. |
| `memory_limit_mb` | Integer | `None` | Maximum virtual heap allocation limit in megabytes. |

```toml
[profiles.release]
entrypoint = "Main::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false

[profiles.test]
inherits = "debug"
entrypoint = "TestRunner::test"
sources = ["tests"]
```

---

### 3. Profile-Scoped `dependencies`

Dependencies are declared within profiles (e.g. `[profiles.release.dependencies]` or `[profiles.test.dependencies]`). Dependencies strictly follow table notation:

```toml
[profiles.release.dependencies]
solix = { version = "2.0.0", profile = "release" }
raylib = { version = "5.5.0", profile = "release" }

[profiles.test.dependencies]
testlib = { version = "1.0.0", profile = "release" }
```

Child profiles inherit parent dependencies and can override version or build profile.

---

### 4. `[native_library_checksums]` Table (Generated / Optional)

Defines cryptographic FIPS 180-4 SHA-256 hex digests for native shared libraries bundled inside `.slxarc` archives. Automatically staged by `scripts/solix_build.py` during packaging (including native libraries carried transitively from statically linked dependencies when `static = true`) to ensure binary tamper resistance across host and cross-compiled desktop target directories (`lib/<target_triple>/`):

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

1. **Scaffolding (`solix new`)**: Creates `manifest.toml` populated with `[configuration]` and `[profiles.release]`, `[profiles.debug]`, `[profiles.test]`.
2. **Building (`solix build`)**: `scripts/build.py` via `solix_build.py` reads `manifest.toml`, resolves the target profile with inheritance, compiles the profile's accumulated `sources`, forwards configured entrypoint and linking modes, and packages `manifest.toml` at the root of the resulting `.slxarc` archive.
3. **Execution (`solix run`)**: When running a `.slxarc` archive, the VM runtime reads `manifest.toml` from the archive container to determine entry points (`entrypoint` in the resolved profile), validate visibility/return signatures, and resolve bundled resources.
4. **Inspection (`solix info`)**: Reads and formats `manifest.toml` when displaying container metadata.
