# The Solix Programming Language

[![CI](https://github.com/jehudme/solix-language/actions/workflows/ci.yml/badge.svg)](https://github.com/jehudme/solix-language/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/github/v/release/Jehudme/solix-language?color=green&label=version)](https://github.com/Jehudme/solix-language/releases)

Solix is a modern, statically typed programming language designed to be fast, memory-safe, and easy to learn. It combines the clean, expressive syntax of languages like C# and Java with the predictable performance and resource control of systems languages like C++ and Rust.

Written entirely in safe Rust (`#![forbid(unsafe_code)]`), Solix is built from the ground up to provide a complete, self-contained development ecosystem: an Ahead-of-Time compiler, an optimized 64-bit register-based virtual machine, automatic reference-counted memory management without stop-the-world tracing GC pauses, a unified command-line toolchain, and first-class Visual Studio Code integration.

---

## Key Design Principles

### Memory Safety with Deterministic ARC & Bounded Cycle Collection
Solix manages heap memory primarily through deterministic **Automatic Reference Counting (ARC)**. You never have to manually allocate and free memory with pointers, and unlike languages that rely on tracing garbage collectors with unpredictable stop-the-world pauses, Solix reclaims memory immediately as soon as an object reference count drops to zero and runs its class destructor (`~ClassName()`) deterministically. For complex cyclic object graphs, Solix provides a bounded, synchronous cycle collector based on Bacon & Rajan (2001) that isolates and collects garbage cycles on demand without non-deterministic tracing pauses.

### Fast 64-Bit Register Virtual Machine
The Solix compiler translates source code into compact binary containers (`.slxb`) and standalone multi-file distribution packages (`.slxp`). These containers execute on a high-throughput 64-bit register virtual machine featuring fixed 8-byte instruction words, sliding register windows, and linear-scan register allocation. The result is fast execution, compact binary footprints, and consistent behavior across Linux, macOS, and Windows.

### Clean and Explicit Scope Syntax
Solix enforces clear visual separation between namespaces, types, and instance members:
* **Scope Resolution (`::`)**: Used strictly for namespace paths, types, static methods, and enum variants (e.g. `std::io::Console::write_line`, `solix::collections::List`, `Color::RED`, `GameState::PLAYING`).
* **Member Access (`.`)**: Used strictly for instance fields, object properties, and instance method invocations (e.g. `player.health`, `particle.update(delta_time)`).

### All-in-One Unified Toolchain
Developing with Solix requires no external build tools, Makefiles, or complicated configuration. The single, consolidated `solix` executable manages the entire development lifecycle: package creation (`solix new package`, `solix new native`), compilation (`solix compile source`, `solix compile package`), package builds (`solix build`), execution (`solix run package`, `solix run source`), code formatting (`solix fmt`), API documentation generation (`solix doc`), bundled offline documentation (`solix wiki`), package management (`solix install`, `solix uninstall`, `solix export`, `solix list`), and editor language intelligence (`solix lsp`, `solix dap`).

---

## Quick Start: Installation

Install the official pre-built Solix toolchain and bundled standard packages with a single command:

### Linux & macOS (bash / zsh)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Jehudme/solix-language/releases/latest/download/solix-toolchain-installer.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://github.com/Jehudme/solix-language/releases/latest/download/solix-toolchain-installer.ps1 | iex
```

### Dev / Nightly Channel (Bleeding-Edge from `dev` merge)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Jehudme/solix-language/releases/download/dev/solix-toolchain-installer.sh | sh
```

### Build from Source (Cargo)
```bash
git clone https://github.com/jehudme/solix-language.git
cd solix-language
cargo build --release
cp target/release/solix ~/.local/bin/
```

### First-Run Package Store Auto-Bootstrap
Upon first invocation, the Solix toolchain automatically installs pre-bundled core packages into `~/.solix/packages`:
* `solixlib` — Complete standard library (collections, math, I/O, networking, reflection).
* `raylib` — Hardware-accelerated 2D/3D graphics, windowing, audio, and physics engine.

To manually re-bootstrap or force refresh bundled packages at any time:
```bash
solix install --bundled --force
```

### Visual Studio Code Extension
To install the official Solix Language extension in Visual Studio Code, VSCodium, or Cursor, run:
```bash
solix install --vscode
```
Alternatively, download the `.vsix` bundle from the [latest GitHub release](https://github.com/Jehudme/solix-language/releases/latest) and install it with `code --install-extension solix-language.vsix`.

---

## Code at a Glance

```solix
import solix.io.Console;

public interface IEntity {
    void update(float32 delta_time);
}

public class Particle implements IEntity {
    public float32 coord_x;
    public float32 coord_y;
    public uint8   alpha;

    public Particle(float32 initial_x, float32 initial_y) {
        this.coord_x = initial_x;
        this.coord_y = initial_y;
        this.alpha = 255;          // Context-typed numeric literal coercion
    }

    ~Particle() {
        // Runs deterministically via ARC when reference count reaches zero
    }

    public void update(float32 delta_time) {
        this.coord_x += 10.5 * delta_time;
        this.coord_y += 5.0 * delta_time;
    }
}

public class Main {
    public static int32 main() {
        Particle spark = new Particle(100.0, 200.0);
        spark.update(0.016);
        Console::println("Particle updated: x=" + spark.coord_x + ", y=" + spark.coord_y);
        return 0;
    }
}
```

---

## Unified Toolchain Command Reference

The single `solix` executable manages all phases of project development, compilation, execution, debugging, and packaging:

| Command | Purpose | Example |
|:---|:---|:---|
| `solix new` | Scaffolds a new package or native project | `solix new package my_game` |
| `solix compile` | Compiles `.slx` source files or packages into `.slxb` | `solix compile source src/main.slx -o bin/app.slxb` |
| `solix build` | Builds a package into an isolated `.slxp` bundle with `--platform` target support | `solix build --profile release --platform x86_64-unknown-linux-gnu` |
| `solix run` | Executes `.slxp` packages, `.slxb` binaries, or `.slx` source files | `solix run package build/release/my_game.slxp` |
| `solix clean` | Cleans package build artifacts (`build/`) or caches (`.cache/`) | `solix clean --cache-only` |
| `solix install` | Installs `.slxp` packages, `.slxb` binaries, or VS Code extension | `solix install package build/release/my_game.slxp` |
| `solix uninstall` | Removes packages or binaries from the local store | `solix uninstall package raylib` |
| `solix export` | Exports installed packages or binaries to a destination | `solix export package raylib --output dist/` |
| `solix list` | Lists all installed items with `KIND` (`package` / `binaries`), sizes, and digests | `solix list --json` |
| `solix info` | Inspects container metadata, disassembly, or exported APIs | `solix info build/release/app.slxp` |
| `solix lsp` | Starts the standard Language Server Protocol (LSP 3.17) | `solix lsp --stdio` |
| `solix dap` | Starts the Debug Adapter Protocol (DAP) service | `solix dap --port 45678` |
| `solix fmt` | Formats Solix source files according to official style | `solix fmt src/` |
| `solix doc` | Generates Markdown API documentation from source comments | `solix doc src/ --out docs/api` |
| `solix wiki` | Opens the bundled offline HTML documentation and search portal | `solix wiki --print-path` |

---

## Architectural Overview

* **64-Bit Register Abstract Machine**: Uniform 8-byte instruction words (`Format A`, `Format B`, `Format C`, `Format Call`), sliding register windows, and linear-scan register allocation for high instruction cache locality.
* **Deterministic ARC with Segregated Free Lists & Cycle Collection**: Memory is reclaimed immediately when reference counts drop to zero, executing RAII destructors (`~ClassName()`) deterministically without stop-the-world tracing GC pauses, with bounded synchronous cycle collection for cyclic graphs.
* **Declarative Pure-Rust Build System (`manifest.yaml`)**: Executes `pre_compile` / `post_compile` hooks, auto-detects dual-mode C/C++ CMake (`solix::ffi`) and Rust Cargo (`solix-ffi`) native extensions across OS-expanded architectures inside isolated `build/.cache/` directories.
* **Profile-Aware Multi-Version Package Store**: Supports simultaneous side-by-side installations of packages under `<name>-<profile>@<version>` with cryptographic SHA-256 tamper verification.
* **Comprehensive Tooling**: First-class Language Server Protocol (`solix-lsp`), Debug Adapter Protocol (`solix-dap`), and an official Visual Studio Code extension.

---

## Documentation Navigation

Comprehensive guides, specifications, and architecture deep dives are available in [`docs/`](docs/README.md):

* [**Master Documentation Hub**](docs/README.md) — Complete taxonomy and architecture guide.
* [**Getting Started Tutorial**](docs/guide/01_getting_started.md) — First steps with Solix, installation, and project layout.
* [**Declarative Build System & `manifest.yaml`**](docs/guide/build_system.md) — Pure-Rust `solix build`, YAML anchors, `pre_compile`/`post_compile` hooks, and multi-target compilation.
* [**FFI SDK Wiki (`ffi/c` & `ffi/rust`)**](docs/wiki/ffi/README.md) — Complete C11/C++17 (`solix::ffi`) and Rust (`solix-ffi`) FFI SDK reference and [Dual-Backend Example](examples/ffi_dual_backend/README.md).
* [**`manifest.yaml` Wiki Reference**](docs/wiki/manifest.md) — YAML variables, self-contained profiles, platform expansion, and auto-detected `natives`.
* [**Disassembly Format Reference (`.slxasm`)**](docs/spec/disassembly-format-slxasm.md) — Guide to reading `.slxasm` / `.slxdasm` 64-bit Register VM output.
* [**VS Code Extension & Debugging**](docs/guide/editors_vscode.md) — Editor setup, CodeLens execution, and interactive DAP debugging.
* [**CLI Toolchain Reference**](docs/spec/tools/cli/README.md) — Detailed option flags and execution contracts for all `solix` subcommands.
* [**Contributing & Engineering Guidelines**](CONTRIBUTING.md) — Contributor workflow and normative [engineering standards](docs/guidelines/README.md).

---

## License

The Solix Programming Language and its standard toolchain are open-source software licensed under the [Apache License, Version 2.0](LICENSE).
