# `solix build` — Project Manifest & Container Build

The `build` subcommand orchestrates project building and packaging according to `manifest.toml`. It executes project build automation (`scripts/build.py`), compiles Rust native bridge extension crates, stages resources, compiles Solix source files, and produces a distributable archive container (`build/<profile>/<name>.slxarc`).

---

## Synopsis

```bash
solix build [path] [options]
```

---

## Positional Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `[path]` | Path | `.` (Current Directory) | Project root directory containing `manifest.toml` and `scripts/build.py`. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--profile <mode>` | | String | `"debug"` | Target build profile: `debug`, `release`, or `test`. |
| `--debug` | | Flag | `false` | Shorthand for debug build profile (`--profile debug`). |
| `--release` | | Flag | `false` | Shorthand for release build profile (`--profile release`). |
| `--test` | | Flag | `false` | Shorthand for test build profile (`--profile test`), including `tests/` sources in the compilation. |
| `--link <mode>` | | String | `"dynamic"` | Linking mode for dependencies and emission: `dynamic` (default) or `static`. |
| `--static` | | Flag | `false` | Shorthand for static linking mode (`--link static`). |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for resolving package dependencies. |
| `--check-env` | `--verify-only` | Flag | `false` | Run Phase 0 pre-flight environment and toolchain verification without compiling or packaging. |
| `--target <triple>` | | String | `None` | Compile native crates for a single Rust target triple or `"host"`. |

---

## Build Automation Pipeline

When `solix build` is invoked:

```mermaid
flowchart TD
    A["solix build [path]"] --> B["Validate manifest.toml & scripts/build.py"]
    B --> C["Resolve Profile & platforms constraint"]
    C --> V["Phase 0: verify_environment()<br>(tools, versions, rustup targets, verify hook)"]
    V --> D["Invoke python3 scripts/build.py --profile <profile>"]
    D --> E["tb.prepare_staging_dirs()<br>(build/.cache/stage/<profile>/)"]
    E --> F["tb.compile_rust_project()<br>(enforce required platforms -> stage/lib/)"]
    F --> G["tb.stage_resources()<br>(resources/ -> stage/resources/)"]
    G --> H["tb.compile_solix_sources()<br>(solix compile src/ -> stage/bin/<name>.slxb)"]
    H --> I["tb.package_slxarc()<br>(embed supported_platforms + checksums)"]
```

### Pre-Flight Verification & Cross-Platform Release Gate
1. **Phase 0 (`verify_environment`):** Checks host toolchain dependencies (`cargo`, `rustc`, `zig >= 0.11.0`, `cargo-zigbuild`, required `rustup` targets) and invokes custom `verify(ctx)` hooks using `require_tool`, `require_rust_targets`, and `require_env`.
2. **Toolbox Helpers:** Custom `scripts/build.py` scripts can query `solix_build.get_required_platforms(manifest, profile)` and `solix_build.resolve_target_triples(manifest, profile)`.
3. **Hard Release Failure:** When any platform required by `[profiles.<name>].platforms` (defaulting `[profiles.release]` to all 5 desktop architectures for native projects, and `["universal"]` for pure-Solix projects) fails or is missing unless `--target` is passed, `solix_build.py` raises a fatal `BuildError` and halts compilation.

### Staged Artifact Directory Structure

During the build lifecycle, artifacts are assembled deterministically:

```text
build/
├── .cache/
│   ├── cargo/                # Cargo target directory for native bridge crates
│   └── stage/
│       └── <profile>/
│           ├── manifest.toml # Packaged copy of project manifest
│           ├── bin/
│           │   └── <name>.slxb # Compiled Solix application bytecode
│           ├── lib/
│           │   ├── <name>_bridge.so # Native shared library (.so / .dll / .dylib)
│           │   └── (dependencies)
│           └── resources/    # Bundled data assets
└── <profile>/
    └── <name>.slxarc         # Final distribution archive
```

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Project built and `.slxarc` archive successfully created. |
| `1` | Error | Missing `manifest.toml`, missing `scripts/build.py`, native cargo compilation error, or Solix compilation error. |

---

## Examples

### 1. Default Debug Build
```bash
solix build
```

### 2. Optimized Release Build
```bash
solix build --release
```

### 3. Test Build
```bash
solix build --test
```

### 4. Build Project in Another Directory with Static Linking
```bash
solix build ../game_project --release --static
```
