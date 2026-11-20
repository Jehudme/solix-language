# Getting Started with Solix

Solix is a statically typed, high-performance object-oriented programming language designed for predictable memory management through Automatic Reference Counting (ARC) and a register-based virtual machine.

This guide provides a practical, hands-on walkthrough for setting up the toolchain, scaffolding new projects, running code directly on-the-fly, building distributable archive packages (`.slxarc`), inspecting binary containers (`.slxb`), and managing local dependencies with the `solix` CLI.

---

## Table of Contents

1. [Prerequisites & Building from Source](#prerequisites--building-from-source)
2. [CLI Toolchain Overview](#cli-toolchain-overview)
3. [Your First Project (`solix new`)](#your-first-project-solix-new)
   - [Application Project Layout](#application-project-layout)
   - [Scaffolding Libraries (`--lib`)](#scaffolding-libraries---lib)
   - [Pure Solix Projects (`--no-native`)](#pure-solix-projects---no-native)
4. [Instant Execution (`solix run`)](#instant-execution-solix-run)
5. [Compiling Standalone Binaries (`solix compile`)](#compiling-standalone-binaries-solix-compile)
6. [Building Projects (`solix build`)](#building-projects-solix-build)
7. [Running Tests (`solix run --test`)](#running-tests-solix-run---test)
8. [Inspecting Containers & Disassembly (`solix info`)](#inspecting-containers--disassembly-solix-info)
9. [Local Package Management (`install`, `list`, `uninstall`)](#local-package-management)
10. [Passing Arguments & Exit Code Propagation](#passing-arguments--exit-code-propagation)
11. [Next Steps & Further Reading](#next-steps--further-reading)

---

## Prerequisites & Building from Source

### Quick Installation (Prebuilt Binaries)

Install the official prebuilt Solix binary toolchain directly on Linux, macOS, or Windows:

#### Linux & macOS (Universal Shell Script)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Jehudme/solix-language/releases/latest/download/solix-toolchain-installer.sh | sh
```

#### Windows (PowerShell)
```powershell
irm https://github.com/Jehudme/solix-language/releases/latest/download/solix-toolchain-installer.ps1 | iex
```

> [!NOTE]
> All official release packages include first-party default libraries (`solix` standard library and `raylib` 2D graphics bindings) pre-bundled inside `assets/packages/`. On first run, Solix automatically bootstraps these libraries into your local package store (`~/.solix/packages`). You can also manually re-extract them at any time via `solix install --bundled --force`.

---

### Prerequisites & Building from Source

| Tool | Minimum Version | Purpose |
|---|---|---|
| Rust / Cargo | 1.80+ (Edition 2021/2024) | Compiler workspace, CLI toolchain, and Rust `solix-ffi` extensions |
| CMake + C11/C++17 Compiler | 3.20+ (Optional) | Dual-mode C/C++ native extensions using `solix::ffi` |
| Git | Recent version | Source version control |

### Clone and Build the Toolchain

```bash
git clone https://github.com/Jehudme/solix-language.git
cd solix-language
cargo build --release
```

After compilation finishes, the unified CLI executable is located at:
- **`target/release/solix`**

You can install the executable system-wide or add `target/release` to your `PATH`:

```bash
cargo install --path crates/solix-toolchain
```

Verify the installation:
```bash
solix --version
```

---

## CLI Toolchain Overview

The `solix` executable is the single entry point for all development tasks:

```bash
solix <subcommand> [options] [arguments...]
```

| Subcommand | Function | Reference |
|---|---|---|
| `new` | Scaffolds a project workspace (`manifest.yaml`, tests, optional `c`/`cpp`/`rust` native) or adds a native library via `solix new native` | [docs/spec/tools/cli/new.md](../spec/tools/cli/new.md) |
| `compile` | Directly compiles source files into executable bytecode (`.slxb`) | [docs/spec/tools/cli/compile.md](../spec/tools/cli/compile.md) |
| `build` | Builds multi-file projects into distribution archive packages (`.slxarc`) | [docs/spec/tools/cli/build.md](../spec/tools/cli/build.md) |
| `run` | Executes bytecode containers, archive packages, or runs `.slx` files on-the-fly | [docs/spec/tools/cli/run.md](../spec/tools/cli/run.md) |
| `info` | Inspects container metadata, disassembles bytecode, or reconstructs APIs | [docs/spec/tools/cli/info.md](../spec/tools/cli/info.md) |
| `install` | Installs `.slxb` binaries or `.slxarc` archives into the local package store | [docs/spec/tools/cli/install.md](../spec/tools/cli/install.md) |
| `uninstall` | Removes installed packages or specific versions from the local store | [docs/spec/tools/cli/uninstall.md](../spec/tools/cli/uninstall.md) |
| `list` | Lists all installed packages and versions | [docs/spec/tools/cli/list.md](../spec/tools/cli/list.md) |
| `doc` | Generates static HTML or Markdown API documentation from source doc comments | [docs/spec/tools/cli/doc.md](../spec/tools/cli/doc.md) |
| `wiki` | Opens the bundled offline HTML documentation portal matching your `solix` version | [docs/spec/tools/cli/wiki.md](../spec/tools/cli/wiki.md) |

---

## Your First Project (`solix new`)

To scaffold a new Solix project:

```bash
solix new hello_solix
cd hello_solix
```

### Application Project Layout

By default, `solix new` generates a fully featured project structure with pure-Rust `manifest.yaml` configuration:

```text
hello_solix/
├── manifest.yaml             # Project metadata, YAML vars/profiles, natives & hooks
├── .gitignore                # Build and cache exclusion patterns
├── src/
│   └── main.slx              # Application entry point source
├── tests/
│   └── test_main.slx         # Automated test suite
├── resources/                # Static application assets
├── lib/                      # Directory for precompiled native shared objects
└── natives/
    └── hello_solix_bridge/   # Dual-mode Rust (`solix-ffi`) or C/C++ (`solix::ffi`) bridge
        ├── Cargo.toml        # (or CMakeLists.txt when using `--native c` / `--native cpp`)
        └── src/lib.rs
```

#### The Starter Source (`src/main.slx`)

```solix
class Main {
    public static int32 main() {
        return 0;
    }
}
```

In Solix, executable programs define a `main` method (by default `Main::main`). The integer return value (`0` for success) is returned directly to the host operating system shell.

#### The Project Manifest (`manifest.yaml`)

```yaml
vars:
  project_version: &app_version "0.1.0"
  default_sources: &src_dirs
    - src

configuration:
  name: hello_solix
  version: *app_version
  description: A Solix project
  license: Apache-2.0
  authors: []

profiles:
  release:
    entrypoint: Main::main
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    sources: *src_dirs
    platforms:
      - host
    natives:
      hello_solix_bridge:
        profile: release

  debug:
    entrypoint: Main::main
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    sources: *src_dirs
    platforms:
      - host
    natives:
      hello_solix_bridge:
        profile: debug

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
```

> [!TIP]
> Pass `--native c` or `--native cpp` to `solix new` to generate a dual-mode CMake project instead of Rust, or add a native library to an existing project later using `solix new native <name> --project . --lang <rust|c|cpp>`. See the [`manifest.yaml` Wiki](../wiki/manifest.md).

### Scaffolding Libraries (`--lib`)

When creating a reusable library rather than an executable application, supply the `--lib` flag:

```bash
solix new math_utils --lib
```

This generates `src/lib.slx` with package declaration instead of `src/main.slx`:

```solix
package math_utils;

public class MathUtils {
    public static int32 version() {
        return 1;
    }
}
```

### Pure Solix Projects (`--no-native`)

If your project does not require Rust native bridge shared libraries, omit the `natives/` crate:

```bash
solix new simple_tool --no-native
```

---

## Instant Execution (`solix run`)

During development, you can execute Solix source files directly without a separate build step:

```bash
solix run src/main.slx
```

The runtime compiles `src/main.slx` into memory and launches the VM instantly.

You can also pass arguments to the application using `--`:

```bash
solix run src/main.slx -- --verbose --input data.txt
```

---

## Compiling Standalone Binaries (`solix compile`)

To compile source files directly into a standalone binary container (`.slxb`):

```bash
solix compile src/main.slx -o bin/hello_solix.slxb
```

Execute the compiled container:
```bash
solix run bin/hello_solix.slxb
```

### Useful Compilation Flags
- **`--check`**: Validates syntax and semantic types without generating bytecode:
  ```bash
  solix compile src/main.slx --check
  ```
- **`--static`**: Merges external dependencies statically, producing a self-contained binary:
  ```bash
  solix compile src/main.slx -d math_utils@0.1.0 --static -o bin/bundled.slxb
  ```
- **`--metrics`**: Prints compiler throughput and phase timings:
  ```bash
  solix compile src/main.slx --metrics
  ```

---

## Building Projects (`solix build`)

For projects with `manifest.yaml` (or legacy `manifest.toml`), use `solix build` to execute `pre_compile` hooks, auto-detect and build CMake/Cargo native extensions, stage resources, compile Solix source code, and assemble a single distributable archive container (`.slxarc`):

### 1. Debug Build (Default)
```bash
solix build
```
Creates: `build/debug/hello_solix.slxarc`.

### 2. Optimized Release Build
```bash
solix build --release
```
Creates: `build/release/hello_solix.slxarc`.

### Running Built Archives
Execute the archive package directly with `solix run`:
```bash
solix run build/release/hello_solix.slxarc
```

The VM runtime automatically extracts the manifest, stages native libraries in memory, configures resource access, and executes the `[entrypoints].main` method.

---

## Running Tests (`solix run --test`)

Every project scaffolded by `solix new` includes a test suite in `tests/test_main.slx`:

```solix
class TestRunner {
    public static int32 test() {
        return 0;
    }
}
```

To build and run tests:

```bash
# Build test package
solix build --test

# Execute test suite entrypoint
solix run --test build/test/hello_solix.slxarc
```

---

## Inspecting Containers & Disassembly (`solix info`)

The `solix info` command inspects compiled `.slxb` binaries and `.slxarc` archive containers.

### Inspect Container Summary
```bash
solix info build/release/hello_solix.slxarc
```

Displays package metadata, entry points, contained files with compressed and uncompressed byte sizes, and native shared libraries.

### Disassemble Virtual Machine Instructions (`--disasm`)
```bash
solix info build/release/hello_solix.slxarc --disasm
```

Outputs the disassembled 64-bit Register VM opcodes (`Format A`, `Format B`, `Format Call`), registers, and constant pool entries.

### Extract Reconstructed API Declarations (`--api`)
```bash
solix info bin/hello_solix.slxb --api
```

Emits the public API interface source code (`.slx`) reconstructed directly from the binary's `ExportManifest`.

### Output JSON Metadata (`--json`)
```bash
solix info build/release/hello_solix.slxarc --json
```

---

## Local Package Management (`install`, `sync`, `list`, `uninstall`)

Solix includes local package store management commands (`install`, `sync`, `list`, `uninstall`) for sharing libraries across projects on your local machine with end-to-end cryptographic SHA-256 verification.

The local package store resides at `~/.solix/packages/` (or `%USERPROFILE%\.solix\packages` on Windows), managed by an automatic self-healing registry manifest at `registry.json`.

### 1. Install a Package (`solix install`)
You can install precompiled `.slxb` binaries, `.slxarc` archives, or install directly from a project folder:

```bash
# Install directly from a library project directory (builds and registers source folder)
solix install ../libraries/solixlib --release

# Automatically install on build or compile
solix build --release --install
solix compile src/math.slx --name math_utils --version 1.0.0 --install

# Install precompiled containers
solix install build/release/raylib.slxarc
solix install build/release/math_utils.slxb
```

> [!NOTE]
> Solix guards against accidental package overwrites. If `<name>@<version>` is already installed, `solix install` refuses to overwrite unless `--force` (`-f`) is explicitly provided.

### 2. Export Packages for Distribution (`solix export`)
To package libraries or binaries for distribution, use `solix export`:

```bash
# Export a built package archive container
solix export package ./solixlib --output ./dist/solixlib.slxp

# Export standalone bytecode binaries
solix export binaries ./build/release/solixlib.slxb --output ./dist/solixlib.slxb
```

### 3. List Installed Packages (`solix list`)
`solix list` displays an enriched inventory of installed packages with build profiles, sizes, installation timestamps, SHA-256 digests, and source directories:

```bash
solix list

# Filter and sort packages
solix list --filter math
solix list --profile release
solix list --kind slxarc --sort size --reverse
solix list --json
```

Output:
```text
Installed Solix Packages (/home/user/.solix/packages):
NAME             VERSION    KIND    PROFILE  SIZE       INSTALLED AT         SHA256          SOURCE DIR
---------------- ---------- ------- -------- ---------- -------------------- --------------- --------------------
solix            2.0.0      slxarc  release  681.8 KB   2026-11-11T05:44:54Z 46747ea15ac6... /home/user/solixlib
raylib           1.2.0      slxarc  release  1.2 MB     2026-11-11T02:15:30Z 9d1da08c4ede... -
```

### 4. Inspect Packages & Modules (`solix info`)
`solix info` inspects container artifacts or looks up installed modules directly by name:

```bash
# Look up installed package by name or name@version
solix info solix
solix info raylib@1.2.0

# Disassemble bytecode or reconstruct interface source
solix info solix --api
solix info solix --disasm
solix info solix --json
```

### 5. Uninstall a Package (`solix uninstall`)
```bash
# Uninstall an exact version
solix uninstall math_utils@0.1.0

# Uninstall all versions of a package
solix uninstall math_utils
```

### 6. Cryptographic Verification & Bypassing Checks (`--no-verify`)
By default, the Solix toolchain cryptographically verifies that every resolved dependency and native shared library matches the exact SHA-256 digest recorded at compile time.

To temporarily bypass checksum verification when hot-swapping or debugging experimental libraries:
```bash
solix run app.slxb --no-verify
# or: solix compile src/main.slx --no-verify
```

---

## Passing Arguments & Exit Code Propagation

The Solix runtime propagates the return value of `main()` directly to the host operating system shell.

```solix
class Main {
    public static int32 main() {
        return 42;
    }
}
```

```bash
solix run src/main.slx
echo $?
# Output: 42
```

Arguments passed after `--` are delivered directly to the guest application:

```bash
solix run src/main.slx -- --host localhost --port 8080
```

---

## Next Steps & Further Reading

### Next Steps
- [Variables & Types Guide](02_variables_and_types.md): Learn about variable declarations, primitive types, type coercion, and reference strings.

### Further Reading
- [Classes & Object-Oriented Programming](04_classes_and_oop.md): Classes, inheritance, abstract methods, and interfaces.
- [Memory Management & ARC](05_memory_management_and_arc.md): Automatic Reference Counting, weak references, and deterministic destructors.
- [CLI Reference: compile & build](../wiki/commands/compile-and-build.md): Compiler invocation, options, and archive packaging.
- [Built-in Primitives Reference](../wiki/primitives/README.md): Bit-exact data types and scalar representations.
- [Native C & Rust Interop](09_native_plugins.md): Binding C and Rust shared libraries using `SolixCValue` and FFI callbacks.
- [CLI Command Wiki Reference](../wiki/commands/README.md): Quick reference cheat sheets for all CLI commands and options.
