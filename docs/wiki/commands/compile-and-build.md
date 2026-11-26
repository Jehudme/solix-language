# CLI Reference: `compile` & `build`

---

## 1. `solix compile`

Compiles Solix source files into an executable `.slxb` container, or compiles a package into `.slxp`:

```bash
solix compile source <sources...> [options]
solix compile package [path] [options]
```

### Key Options & Flags (`compile source`)
* `-o, --output <path>`: Specifies destination `.slxb` output file path [default: `<name>.slxb`].
* `-d, --deps <spec>`: External package dependency requirement formatted as `<name>[:<profile>][@<version>]` (can be repeated).
* `--deps-source <path>`: External source dependency file (`.slx`) to include directly.
* `--deps-package <path>`: External package container (`.slxp`) to link against.
* `--deps-binaries <path>`: External binary container (`.slxb`) to link against.
* `--deps-file <path>`: External binary (`.slxb`) or package archive (`.slxp`) dependency file path (shorthand `-l`).
* `-n, --name <name>`: Package or module identifier name.
* `-V, --version <semver>`: Semantic version embedded into binary metadata [default: `0.1.0`].
* `--description <desc>`: Package description string.
* `--profile <debug|release|test>`: Compilation profile [default: `release`].
* `-e, --entry <symbol>`: Overrides default entry point symbol (e.g. `Main::main`).
* `--emit-api <file.slx>`: Synthesizes and writes public external API signatures to a `.slx` file.
* `--check`: Performs syntax and semantic binder verification without emitting bytecode.
* `--time`: Prints wall-clock duration for each compiler pipeline phase.
* `--metrics [<format>]`: Prints compiler throughput metrics and phase duration breakdown (`text` or `json`).
* `--inline-threshold <N>`: Inlining statement threshold limit for `AstInlinerPhase` [default: `8`].
* `--link <dynamic|static>`: Linking mode for dependencies and emission [default: `dynamic`].
* `--static`: Shorthand flag for static linking mode (`--link static`), bundling all dependencies into one binary.
* `--no-verify`: Bypasses dependency cryptographic SHA-256 checksum verification (visible alias `--ignore-checksum`).
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].
* `--install`: Automatically installs compiled binary into package store.
* `-f, --force-install`: Force overwrites existing packages in store when `--install` is active.

### Practical Examples
```bash
# Compile a single source file into an executable binary
solix compile source src/main.slx -o bin/app.slxb

# Multi-file compilation with external dependencies
solix compile source src/main.slx src/utils.slx -d raylib@1.0.0 -o bin/game.slxb

# Check syntax and semantics without emitting bytecode
solix compile source src/main.slx --check

# Compile with static dependency linking and metrics
solix compile source src/main.slx -d math_utils@0.2.0 --static --metrics
```

### Upward Manifest Auto-Discovery
If `solix compile source` or `solix compile package` is executed inside a project tree without explicit `--deps`, `--entry`, or `--name` parameters, it automatically traverses parent directories (up to 12 levels) to discover `manifest.yaml` (or legacy `manifest.toml`). Dependencies and entry points from `profiles`, and project metadata from `configuration` are inherited automatically.

---

## 2. `solix build`

Orchestrates multi-file, multi-language projects governed by `manifest.yaml` (pure Rust build pipeline):

```bash
solix build [path] [options]
```

### Key Options & Flags
* `[path]`: Project root directory containing `manifest.yaml` (or legacy `manifest.toml`) [default: `.`].
* `--profile <name>`: Target self-contained build profile (e.g., `release`, `debug`, `test`, or custom declared profile) [default: `release`].
* `--debug`: Shorthand for `--profile debug`.
* `--release`: Shorthand for `--profile release` (optimizations enabled).
* `--test`: Shorthand for `--profile test` (includes `tests/` sources and fixtures).
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].
* `-i, --install`: Automatically installs the built `.slxp` into `~/.solix/packages`.
* `-f, --force`: Overwrites existing installed package when paired with `--install`.
* `--check-env` / `--verify-only`: Runs only Phase 0 pre-flight environment & toolchain verification without compiling.
* `--target <triple>`: Compiles native subprojects for a single target triple or `"host"`.
* `--platform <name>`: Builds all target architectures for the specified platform (`linux`, `windows`, `macos`, or `all`).

### Project Binary Directory Layout & Native Naming
* All pre-compiled and built native shared libraries reside under `bin/<target_triple>/`. The legacy `lib/` directory is deprecated and eliminated.
* Solix runtime autoloading automatically discovers modules whose library filenames match `__native_<name>.<extension>` (or `lib__native_<name>.<extension>`).
* Profiles can specify external pre-compiled shared libraries with `libraries: [lib01, lib02]`, which are verified across all active profile targets under `bin/<target_triple>/` before compilation.

### Practical Examples
```bash
# Build the project in the current directory with debug profile
solix build

# Build optimized release package container (build/release/<name>.slxp)
solix build --release

# Build test package including test suite fixtures (build/test/<name>.slxp)
solix build --test

# Build all targets for Linux
solix build --platform linux

# Build and auto-install into local package store, overwriting any previous version
solix build --release --install --force
```

---

## Related Specifications
* Formal Spec: [`docs/spec/tools/cli/compile.md`](../../spec/tools/cli/compile.md)
* Formal Spec: [`docs/spec/tools/cli/build.md`](../../spec/tools/cli/build.md)
