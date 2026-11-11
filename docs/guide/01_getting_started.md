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

### Prerequisites

| Tool | Minimum Version | Purpose |
|---|---|---|
| Rust / Cargo | 1.80+ (Edition 2021/2024) | Compiler workspace and CLI build toolchain |
| Python | 3.10+ | Project build automation (`scripts/build.py`) |
| Git | Recent version | Source version control |

### Clone and Build the Toolchain

```bash
git clone https://github.com/yourorg/solix-language.git
cd solix-language
cargo build --release
```

After compilation finishes, the unified CLI executable is located at:
- **`target/release/solix`**

You can install the executable system-wide or add `target/release` to your `PATH`:

```bash
cargo install --path crates/solix-client
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
| `new` | Scaffolds a project workspace with `manifest.toml`, build automation, and tests | [docs/spec/tools/cli/new.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/new.md) |
| `compile` | Directly compiles source files into executable bytecode (`.slxb`) | [docs/spec/tools/cli/compile.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/compile.md) |
| `build` | Builds multi-file projects into distribution archive packages (`.slxarc`) | [docs/spec/tools/cli/build.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/build.md) |
| `run` | Executes bytecode containers, archive packages, or runs `.slx` files on-the-fly | [docs/spec/tools/cli/run.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/run.md) |
| `info` | Inspects container metadata, disassembles bytecode, or reconstructs APIs | [docs/spec/tools/cli/info.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/info.md) |
| `install` | Installs `.slxb` binaries or `.slxarc` archives into the local package store | [docs/spec/tools/cli/install.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/install.md) |
| `uninstall` | Removes installed packages or specific versions from the local store | [docs/spec/tools/cli/uninstall.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/uninstall.md) |
| `list` | Lists all installed packages and versions | [docs/spec/tools/cli/list.md](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/list.md) |

---

## Your First Project (`solix new`)

To scaffold a new Solix project:

```bash
solix new hello_solix
cd hello_solix
```

### Application Project Layout

By default, `solix new` generates a fully featured project structure:

```text
hello_solix/
├── manifest.toml             # Project metadata, entry points, and dependencies
├── .gitignore                # Build and cache exclusion patterns
├── src/
│   └── main.slx              # Application entry point source
├── tests/
│   └── test_main.slx         # Automated test suite
├── resources/                # Static application assets
├── lib/                      # Directory for precompiled native shared objects
├── natives/
│   └── hello_solix_bridge/   # Rust cdylib extension crate for native interop
│       ├── Cargo.toml
│       └── src/lib.rs
└── scripts/
    ├── build.py              # Build orchestration script
    └── solix_toolbox.py      # Build toolbox helper module
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

#### The Project Manifest (`manifest.toml`)

```toml
[info]
name = "hello_solix"
version = "0.1.0"
description = "A Solix project"
license = "Apache-2.0"
authors = []

[entrypoints]
main = "Main::main"
test = "TestRunner::test"

[dependencies]
# solixlib = "2.0.0"
```

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

For projects with `manifest.toml`, use `solix build` to build all native extensions, stage resources, compile Solix source code, and assemble a single distributable archive container (`.slxarc`):

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

## Local Package Management

Solix includes local package store management commands (`install`, `list`, `uninstall`) for sharing libraries across projects on your local machine.

The local package store resides at `~/.solix/packages/` (or `%USERPROFILE%\.solix\packages` on Windows).

### 1. Install a Package
```bash
# Install a library binary
solix install build/release/math_utils.slxb

# Install an archive container with native libraries
solix install build/release/raylib_solix.slxarc
```

### 2. List Installed Packages
```bash
solix list
```

Outputs a tabular inventory of installed packages:
```text
Installed Solix Packages:
--------------------------------------------------------------------------------
Package              Version        Format     Location
--------------------------------------------------------------------------------
math_utils           0.1.0          slxb       ~/.solix/packages/math_utils/0.1.0/math_utils.slxb
raylib_solix         1.0.0          slxarc     ~/.solix/packages/raylib_solix/1.0.0/raylib_solix.slxarc
--------------------------------------------------------------------------------
Total: 2 package(s) installed.
```

### 3. Uninstall a Package
```bash
# Uninstall an exact version
solix uninstall math_utils@0.1.0

# Uninstall all versions of a package
solix uninstall math_utils
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

- [Variables & Types Guide](file:///home/jehud/Projects/solix-language/docs/guide/02_variables_and_types.md): Variables, primitive types, operators, and control flow.
- [Classes & Object-Oriented Programming](file:///home/jehud/Projects/solix-language/docs/guide/04_classes_and_oop.md): Classes, inheritance, abstract methods, and interfaces.
- [Memory Management & ARC](file:///home/jehud/Projects/solix-language/docs/guide/05_memory_management_and_arc.md): Automatic Reference Counting, weak references, and deterministic destructors.
- [Native C & Rust Interop](file:///home/jehud/Projects/solix-language/docs/guide/09_native_plugins.md): Binding C and Rust shared libraries using `SolixCValue` and FFI callbacks.
- [CLI Command Wiki Reference](file:///home/jehud/Projects/solix-language/docs/wiki/commands/README.md): Quick reference cheat sheets for all CLI commands and options.
