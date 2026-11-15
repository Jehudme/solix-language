# CLI Reference: `compile` & `build`

---

## 1. `solix compile`

Compiles one or more Solix source files (`*.slx`) into an executable `.slxb` container:

```bash
solix compile <sources...> [options]
```

### Key Options & Flags
* `-o, --output <path>`: Specifies destination `.slxb` output file path [default: `<name>.slxb`].
* `-d, --deps <pkg@version>`: External package dependency requirement formatted as `<name>[-<profile>]@<version>` (can be repeated).
* `-l, --deps-file <path>`: External library binary (`.slxb`) or archive (`.slxarc`) to link against (visible alias `--lib`, can be repeated).
* `-n, --name <name>`: Package or module identifier name.
* `-V, --version <semver>`: Semantic version embedded into binary metadata [default: `0.1.0`].
* `--description <desc>`: Package description string.
* `--profile <debug|release|test>`: Compilation profile [default: `release`].
* `-e, --entry <symbol>`: Overrides default entry point symbol (e.g. `Main::main`).
* `--emit-api <file.slx>`: Synthesizes and writes public external API signatures to a `.slx` file.
* `--no-builtins`: Disables automatic inclusion of built-in standard library definitions (`blt.io`).
* `--check`: Performs syntax and semantic binder verification without emitting bytecode.
* `--time`: Prints wall-clock duration for each compiler pipeline phase.
* `--metrics [<format>]`: Prints compiler throughput metrics and phase duration breakdown (`text` or `json`).
* `--inline-threshold <N>`: Inlining statement threshold limit for `AstInlinerPhase` [default: `8`].
* `--link <dynamic|static>`: Linking mode for dependencies and emission [default: `dynamic`].
* `--static`: Shorthand flag for static linking mode (`--link static`), bundling all dependencies into one binary.
* `--no-verify`: Bypasses dependency cryptographic SHA-256 checksum verification (visible alias `--ignore-checksum`).
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].
* `--install`: Automatically installs compiled binary into package store (visible alias `--sync`).
* `-f, --force-install`: Force overwrites existing packages in store when `--install` is active.

### Practical Examples
```bash
# Compile a single source file into an executable binary
solix compile src/main.slx -o bin/app.slxb

# Multi-file compilation with external dependencies
solix compile src/main.slx src/utils.slx -d raylib@1.0.0 -o bin/game.slxb

# Check syntax and semantics without emitting bytecode
solix compile src/main.slx --check

# Compile with static dependency linking and metrics
solix compile src/main.slx -d math_utils@0.2.0 --static --metrics
```

---

## 2. `solix build`

Orchestrates multi-file, multi-language projects governed by `manifest.toml`:

```bash
solix build [path] [options]
```

### Key Options & Flags
* `[path]`: Project root directory containing `manifest.toml` and `scripts/build.py` [default: `.`].
* `--profile <debug|release|test>`: Target build profile [default: `debug`].
* `--debug`: Shorthand for `--profile debug`.
* `--release`: Shorthand for `--profile release` (optimizations enabled).
* `--test`: Shorthand for `--profile test` (includes `tests/` sources and fixtures).
* `--link <dynamic|static>`: Linking mode for dependencies and emission [default: `dynamic`].
* `--static`: Shorthand for static linking mode (`--link static`).
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].

### Practical Examples
```bash
# Build the project in the current directory with debug profile
solix build

# Build optimized release archive container (build/release/<name>.slxarc)
solix build --release

# Build test package including test suite fixtures (build/test/<name>.slxarc)
solix build --test

# Build a project located in another directory with static linking
solix build ../my_game --release --static
```

---

## Related Specifications
* Formal Spec: [`docs/spec/tools/cli/compile.md`](../../spec/tools/cli/compile.md)
* Formal Spec: [`docs/spec/tools/cli/build.md`](../../spec/tools/cli/build.md)
