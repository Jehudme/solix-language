# `manifest.yaml` Reference Wiki

`manifest.yaml` (or `manifest.yml`) is the declarative project manifest for every Solix application, library, and native-backed package. It replaces legacy `manifest.toml` and `scripts/build.py` files with strict schema validation, reusable YAML variables, self-contained profiles, automatic platform expansion, lifecycle command hooks, and auto-detected Cargo/CMake native subprojects.

> [!NOTE]
> Solix still parses legacy `manifest.toml` inside older installed archives during migration, while `manifest.yaml` / `manifest.yml` takes priority whenever present.

---

## Complete Example

```yaml
vars:
  project_version: &proj_ver "1.2.0"
  desktop_platforms: &desktop_targets
    - linux
    - macos
    - windows
  common_sources: &src_dirs
    - src

configuration:
  name: game_engine
  version: *proj_ver
  description: Cross-platform game engine built with Solix
  license: Apache-2.0
  authors:
    - Solix Contributors

profiles:
  release:
    entrypoint: Main::main
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    link: static
    sources: *src_dirs
    platforms: *desktop_targets
    pre_compile:
      - cmake --version
    post_compile: []
    natives:
      renderer_bridge:
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
    link: dynamic
    sources: *src_dirs
    platforms:
      - host
    natives:
      renderer_bridge:
        profile: debug
    dependencies:
      solix:
        version: "2.0.0"
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
    dependencies:
      solix:
        version: "2.0.0"
        profile: release
```

---

## Reusable Variables (`vars`, YAML Anchors & Merge Keys)

Because `manifest.yaml` uses standard YAML 1.2 parsing (`serde-saphyr`), you can define shared scalars, lists, and mappings once and reuse them anywhere:

1. **Top-level `vars:` and `x-*` sections:** Permitted alongside `configuration:` and `profiles:` without triggering strict schema errors.
2. **Anchors (`&name`) & Aliases (`*name`):** Share versions (`*proj_ver`), platform lists (`*desktop_targets`), or dependency maps across self-contained profiles.
3. **YAML Merge Keys (`<<: *anchor`):** Merge shared dependency maps or native blocks cleanly into multiple profiles while adding profile-specific entries.

---

## Top-Level Sections

### 1. `configuration` (Required)

| Key | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `name` | String | **Yes** | — | Unique package identifier (alphanumeric, `_`, `-`) |
| `version` | String | No | `"0.1.0"` | SemVer 2.0 package version (`MAJOR.MINOR.PATCH`) |
| `description` | String | No | `None` | Short project summary |
| `license` | String | No | `None` (`"Apache-2.0"` in `solix new`) | SPDX license identifier |
| `authors` | List of Strings | No | `[]` | Author names or `Name <email>` entries |

---

### 2. `profiles` (Self-Contained Profiles)

Every profile under `profiles.<profile_name>` is **strictly self-contained**:
* **Zero inheritance:** `inherits` is rejected with a manifest validation error.
* Built-in profile names (`release`, `debug`, `test`) receive curated defaults for omitted fields; any custom profile name (such as `cmake_backend` or `rust_backend`) uses the `debug` baseline (`entrypoint: "Main::main"`, `sources: ["src"]`, `platforms: ["host"]`).

#### Profile Fields Reference

| Field | Type | `release` Default | `debug` Default | `test` Default | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `entrypoint` | String | `"Main::main"` | `"Main::main"` | `"TestRunner::test"` | Canonical `Class::method` entrypoint symbol |
| `optimization_level` | Integer (`0..=3`) | `3` | `0` | `0` | Compiler optimizer level (alias: `opt_level`) |
| `inline_threshold` | Integer | `16` | `8` | `8` | Max statement count for AST inlining |
| `debug_symbols` | Boolean | `false` | `true` | `true` | Emits line tables and debug symbols in `.slxb` |
| `static` | Boolean | `true` | `false` | `false` | Relocates dependencies statically |
| `link` | `"dynamic"` \| `"static"` | `"static"` | `"dynamic"` | `"dynamic"` | Dependency link mode override |
| `emit_disassembly` | Boolean | `false` | `false` | `false` | Emits `.slxasm` disassembly artifacts |
| `warnings_as_errors` | Boolean | `false` | `false` | `false` | Promotes compiler warnings to hard errors |
| `verify_checksum` | Boolean | `false` (static) / `true` (dynamic) | `false` | `false` | Enforces SHA-256 verification on dependencies (bypassed on static, debug, and test profiles for speed; enforced on dynamic release) |
| `memory_limit_mb` | Integer | `None` | `None` | `None` | Optional runtime heap cap in MiB |
| `sources` | List of Strings | `["src"]` | `["src"]` | `["src", "tests"]` | Directories scanned recursively for `.slx` files |
| `platforms` | List of Strings | `["all"]` | `["host"]` | `["host"]` | Target OS tokens, `<os>-<arch>` pairs, or Rust triples |
| `pre_compile` | List of Strings | `[]` | `[]` | `[]` | Commands executed sequentially before native & Solix compilation |
| `post_compile` | List of Strings | `[]` | `[]` | `[]` | Commands executed sequentially after `.slxp` packaging |
| `dependencies` | Mapping | `{}` | `{}` | `{}` | Profile-scoped dependency requirements |
| `natives` | Mapping | `{}` | `{}` | `{}` | Auto-detected native subprojects compiled for this profile |
| `libraries` | List of Strings | `[]` | `[]` | `[]` | Prebuilt native libraries verified in `bin/<target_triple>/` |
| `dynamic_libraries` | List of Strings | `[]` | `[]` | `[]` | Runtime dynamic shared libraries staged alongside the package |

---

### 3. `platforms` (OS-to-All-Architectures Expansion)

Supplying an OS token automatically expands to compile every architecture Solix supports for that OS:

| Token | Expanded Architecture Targets |
| :--- | :--- |
| `linux` | `linux-x86_64`, `linux-aarch64` |
| `macos` / `apple` / `darwin` | `macos-x86_64`, `macos-aarch64` |
| `windows` / `win` | `windows-x86_64` |
| `all` | All 5 desktop targets (`linux-x86_64`, `linux-aarch64`, `windows-x86_64`, `macos-x86_64`, `macos-aarch64`) |
| `host` | Current host target triple (`x86_64-pc-windows-msvc` on Windows MSVC, etc.) |

---

### 4. `pre_compile` & `post_compile` Command Hooks

Commands run in sequence from the project root directory with helper environment variables injected by `solix build`:
* `SOLIX_PROJECT_ROOT` — Project root directory
* `SOLIX_PROFILE` — Active profile name
* `SOLIX_BIN` — Path to the active `solix` executable
* `SOLIX_STAGE_DIR` — Build staging directory under `build/.cache/stage/<profile>/`
* `SOLIX_CACHE_DIR` — Build cache directory (`build/.cache/`)
* `SOLIX_OUTPUT_SLXP` — Destination path of the generated `.slxp` package

---

### 5. `natives` (Auto-Detected Cargo & CMake Subprojects)

Entries under `profiles.<name>.natives.<key>` never require a `type` field. Solix inspects the subproject directory (`path`, defaulting to `natives/<key>`):
1. `Cargo.toml` present -> compiled with Cargo (`--features solix-native` added automatically when declared under `[features]`, plus `SOLIX_NATIVE_BUILD=1`).
2. `CMakeLists.txt` present -> configured & built with CMake (`-DSOLIX_NATIVE_BUILD=ON` and `SOLIX_NATIVE_BUILD=1`).

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `path` | String | `natives/<key>` | Relative path from project root to native crate/CMake directory |
| `profile` | String | Parent Solix profile | Native optimization profile (`debug` or `release`) |
| `platforms` | List of Strings | Parent `platforms` | Optional target list overriding the profile's `platforms` |
| `flags` | List of Strings | `[]` | Extra flags passed directly to Cargo or CMake (`-D...`, `--features ...`) |
| `strategy` | String | `"docker"` | Containerized compilation strategy (`"docker"` / `"container"`) |
| `docker` | Boolean | `true` | Enables containerized cross-compilation (`cross` / `dockcross`) |
| `docker_image` | String | Standard image | Custom Docker container image override |

---

### 6. `dependencies` Syntax

Dependencies under `profiles.<name>.dependencies` map package name -> metadata or shorthand version string. Version constraints accept exact semver (`"2.0.0"`), minimum/compatible prefixes (`">=2.0.0"`, `"^2.0.0"`, `"~2.0.0"`, `"=2.0.0"`), or wildcard (`"*"`). Unsupported operators such as `=<` are rejected during manifest validation:

```yaml
dependencies:
  solix:
    version: "2.2.1"
    profile: release
  local_math:
    version: "0.1.0"
```

---

### 7. Generated Package Integrity Fields

During `solix build`, packaged `.slxp` packages receive automatically stamped `supported_platforms` and `native_library_checksums` mappings (FIPS 180-4 SHA-256 digests keyed by `bin/<target_triple>/<library_filename>`). Companion native shared libraries must follow the `__native_*` (or `lib__native_*`) naming convention for runtime dynamic loading.

---

### 8. Diagnostic & Validation Errors

When a manifest contains errors or missing required fields, `solix build` and `solix compile` emit compiler-grade diagnostic output with exact line numbers, column numbers, hierarchical field path context, and visual pointer snippets:

```text
Error: Failed to parse manifest.yaml: missing required field `version` for dependency 'raylib' in profile 'debug'
  --> manifest.yaml:10:9
   |
 8 |     dependencies:
 9 |       raylib:
10 |         profile: debug
   |         ^ missing field `version`
```

Common validation checks:
* **Missing `profiles.release`**: Manifests must declare a release profile (`E0403`).
* **Deprecated `inherits`**: Cross-profile inheritance via `inherits:` has been removed in favor of explicit YAML anchors (`E0402`).
* **Unknown Root Fields**: Unrecognized fields at the root of `manifest.yaml` must be prefixed with `x-` or placed in `vars:` (`E0404`).

---

## Related Documentation
* [Formal `manifest.yaml` Schema Specification](../spec/tools/cli/manifest.md)
* [CLI Reference: `compile` & `build`](commands/compile-and-build.md)
* [CLI Reference: `new`, `new native` & Package Commands](commands/package-commands.md)
* [FFI Wiki Overview](ffi/README.md)
