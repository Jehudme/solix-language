# Solix Project Manifest (`manifest.yaml`) Specification

The `manifest.yaml` file is the declarative configuration descriptor for Solix projects, applications, and libraries. It defines package identity, self-contained build profiles, platform target expansion, pre/post compilation command hooks, auto-detected Cargo and CMake native subprojects, and dependency requirements.

> [!NOTE]
> `solix` also supports reading legacy `manifest.toml` manifests inside existing packages during migration, while `manifest.yaml` (and `manifest.yml`) takes priority when present.

---

## Schema Overview

A complete `manifest.yaml` document structure:

```yaml
vars:
  project_version: &app_version "0.1.0"
  default_platforms: &all_desktop
    - linux
    - macos
    - windows

configuration:
  name: my_project
  version: *app_version
  description: High-performance Solix application with native bindings
  license: Apache-2.0
  authors:
    - Jane Developer <jane@example.com>
    - Solix Core Team

profiles:
  release:
    entrypoint: Main::main
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    sources:
      - src
    platforms: *all_desktop
    pre_compile:
      - cmake --version
    post_compile: []
    natives:
      my_native:
        profile: release
    dependencies:
      solix:
        version: "2.0.0"
        profile: release
      raylib:
        version: "5.5.0"
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
      my_native:
        profile: debug

  test:
    entrypoint: TestRunner::test
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    sources:
      - tests
    platforms:
      - host
    dependencies:
      testlib:
        version: "1.0.0"
        profile: release
```

---

## YAML Reusable Variables (`vars`, Anchors & Aliases)

Because `manifest.yaml` parses standard YAML, you can declare reusable values once in the optional top-level `vars:` mapping (or any `x-*` vendor extension mapping) using YAML anchors (`&name`), aliases (`*name`), and YAML merge keys (`<<:`):

* `vars:` and `x-*` top-level mappings are ignored by strict schema validation (`deny_unknown_fields`).
* Anchors and aliases avoid duplicating version strings, shared dependency maps, or platform lists across self-contained profiles.

---

## Section & Field Specifications

### 1. `configuration` Mapping (Required)

Contains foundational metadata identifying the package and its authors.

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | String | **Yes** | — | Unique package identifier. Must be a valid identifier (alphanumeric, underscores, hyphens). |
| `version` | String | **Yes** | `"0.1.0"` | Semantic version string adhering to SemVer 2.0 (`MAJOR.MINOR.PATCH`). |
| `description` | String | No | `""` | Human-readable summary of the package. |
| `license` | String | No | `"Apache-2.0"` | SPDX license identifier (e.g., `Apache-2.0`, `MIT`). |
| `authors` | List of Strings | No | `[]` | List of author names or name/email pairs. |

---

### 2. `profiles` Mapping & Self-Contained Profiles (Required)

Profiles govern compiler optimization, AST function inlining, debug symbol emission, linking modes, source directory discovery, compilation hooks, native subprojects, and profile-specific dependencies.

#### Self-Contained Profile Rules (Zero Inheritance):
1. **No `inherits`:** Every profile under `profiles.<name>` is completely self-contained. Declaring `inherits` in `manifest.yaml` is explicitly rejected with a schema error.
2. **Baseline Defaults per Profile Name:** When fields are omitted in a profile:
   * `release`: `optimization_level: 3`, `inline_threshold: 16`, `debug_symbols: false`, `static: true`, `link: "dynamic"`, `sources: ["src"]`, `platforms: ["host"]`, `entrypoint: None`
   * `debug`: `optimization_level: 0`, `inline_threshold: 8`, `debug_symbols: true`, `static: false`, `link: "dynamic"`, `sources: ["src"]`, `platforms: ["host"]`, `entrypoint: None`
   * `test`: `optimization_level: 0`, `inline_threshold: 8`, `debug_symbols: true`, `static: false`, `link: "dynamic"`, `sources: ["tests"]`, `platforms: ["host"]`, `entrypoint: "TestRunner::test"`
   * Custom profile names: use the release baseline with `entrypoint: None` and `sources: ["src"]`.
3. **Profile Validation:** Requesting an undeclared custom profile name (not `release`, `debug`, or `test`) produces an immediate resolution error.

#### Profile Options:

| Key | Type | Default | Description |
|---|---|---|---|
| `entrypoint` | String | `None` (`"TestRunner::test"` for `test`) | Canonical fully-qualified entry point method. |
| `optimization_level` | Integer | `3` (`release`) / `0` (`debug`/`test`) | Bytecode optimizer level (`0..=3`). Alias: `opt_level`. |
| `inline_threshold` | Integer | `16` (`release`) / `8` (`debug`/`test`) | Maximum statement count for AST inlining. |
| `debug_symbols` | Boolean | `false` (`release`) / `true` (`debug`/`test`) | Emits source line mappings and debug symbols in `.slxb`. |
| `static` | Boolean | `true` (`release`) / `false` (`debug`/`test`) | Static linking mode (relocates dependency bytecode). Alias: `is_static`. |
| `link` | String | `"dynamic"` | Linking mode for dependencies: `"dynamic"` or `"static"`. |
| `emit_disassembly` | Boolean | `false` | Emits human-readable register VM disassembly files during compilation. |
| `warnings_as_errors` | Boolean | `false` | Treats compiler warnings as fatal errors. |
| `verify_checksum` | Boolean | `true` | Validates dependency cryptographic SHA-256 hashes against pinned manifests. |
| `memory_limit_mb` | Integer | `None` | Maximum virtual heap allocation limit in megabytes. |
| `sources` | List of Strings | `["src"]` (`["tests"]` for `test`) | Recursive directory paths to discover `.slx` source files. |
| `platforms` | List of Strings | `["host"]` | OS names or architecture targets (see Platform Expansion below). |
| `pre_compile` | List of Strings | `[]` | Shell/tool commands run sequentially before native and Solix compilation. |
| `post_compile` | List of Strings | `[]` | Shell/tool commands run sequentially after `.slxp` package packaging. |
| `dependencies` | Mapping | `{}` | Profile-scoped dependency map (`name -> { version, profile }`). |
| `natives` | Mapping | `{}` | Native subprojects compiled automatically for this profile. |
| `libraries` | List of Strings | `[]` | External pre-compiled shared libraries (`.so`, `.dll`, `.dylib`) verified across all profile target platforms under `bin/<target_triple>/`. |

---

### 3. OS-to-All-Architectures Platform Expansion (`platforms`)

Specifying an OS name under `platforms` automatically expands to compile every architecture supported for that OS:

| Token | Expanded Targets |
|---|---|
| `linux` | `linux-x86_64`, `linux-aarch64` (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`) |
| `macos` / `apple` / `darwin` | `macos-x86_64`, `macos-aarch64` (`x86_64-apple-darwin`, `aarch64-apple-darwin`) |
| `windows` / `win` | `windows-x86_64` (`x86_64-pc-windows-gnu`) |
| `all` | All 5 desktop targets (`linux-x86_64`, `linux-aarch64`, `windows-x86_64`, `macos-x86_64`, `macos-aarch64`) |
| `host` | The current host target |

Explicit `<os>-<arch>` identifiers (such as `linux-aarch64`) or full triples are also accepted.

---

### 4. Auto-Detected Native Subprojects (`natives`)

Under `profiles.<name>.natives.<key>`, each native entry omits any explicit `type` field — Solix detects the backend automatically from the native directory (`path`, defaulting to `natives/<key>`):

1. If `Cargo.toml` exists in the native directory -> built with Cargo. When `[features]` defines `solix-native`, `solix build` automatically enables `--features solix-native` and sets `SOLIX_NATIVE_BUILD=1`.
2. Else if `CMakeLists.txt` exists in the native directory -> built with CMake. `solix build` automatically configures `-DSOLIX_NATIVE_BUILD=ON` and sets `SOLIX_NATIVE_BUILD=1` (plus `SOLIX_FFI_SOURCE_DIR` if a local checkout `ffi/c` exists).

Native subprojects strictly inherit the parent profile's `platforms` setting. All built and pre-compiled native shared libraries are staged exclusively into `bin/<target_triple>/`.

| Native Key Field | Type | Default | Description |
|---|---|---|---|
| `path` | String | `natives/<key>` | Relative path from project root to the native Cargo or CMake directory. |
| `profile` | String | Parent Solix profile | Native optimization profile (`debug` or `release`). |
| `flags` | List of Strings | `[]` | Extra CMake (`-D...`) or Cargo (`--features ...`) flags. |
| `strategy` | String | Auto-detected | Compilation strategy (`"host"`, `"zigbuild"`, `"docker"` / `"container"`). |
| `docker` | Boolean | `false` | Shorthand to force containerized cross-compilation (`cross` for Rust, `dockcross` for CMake). |
| `docker_image` | String | Standard image | Custom Docker container image override. |

> [!NOTE]
> **Native Autoloading Convention (`__native_<name>`):**
> Solix automatically registers native FFI modules whose library filenames begin with `__native_<name>.<extension>` (or `lib__native_<name>.<extension>`). Auxiliary shared libraries packaged in `bin/<target_triple>/` that do not follow this prefix are copied for dynamic linking but are not registered as Solix FFI modules.

> [!NOTE]
> **Containerized Cross-Compilation (`strategy: docker`):**
> Containerized compilation executes native builds inside `cross-rs` or `dockcross` containers. While containerized compilation is not the most optimal or lightweight mechanism for cross-compilation compared to hermetic native Zigbuild, it is practical and sufficient for complex native libraries with extensive system library requirements (such as X11, Wayland, or audio drivers).
> Containerized toolchain caches (`build/.cache/cargo-docker` and `build/.cache/cmake-docker`) are isolated from host compilation directories to prevent host-compiled build scripts from leaking into container environments.

---

### 5. `supported_platforms` & `native_library_checksums` Metadata (Generated)

During `.slxp` packaging, `solix build` and `scripts/release/merge_bundled_packages.py` embed top-level `supported_platforms` and cryptographic FIPS 180-4 SHA-256 digests (`native_library_checksums`) inside the packaged `manifest.yaml` across host and cross-compiled target directories (`bin/<target_triple>/`):

```yaml
supported_platforms:
  - linux-x86_64
  - linux-aarch64
  - windows-x86_64
  - macos-x86_64
  - macos-aarch64

native_library_checksums:
  "bin/x86_64-unknown-linux-gnu/native_solixlib.so": "af5f19f16fac4342322dc64262fc55e49b4119122e690957773c78aa25405a44"
```

When installing or loading an `.slxp` (`solix install`, `ClientModuleSolver`, `RuntimeContext::new`, or `SlxpPackage::extract_and_load`), Solix verifies both that the host `<os>-<arch>` is included in `supported_platforms` (or `"universal"`) and that extracted native libraries match their SHA-256 digests (checksum verification can be bypassed with `--no-verify` / `--ignore-checksum`). Companion native shared libraries must follow the `native_*` naming convention for dynamic loading into the VM FFI table.
