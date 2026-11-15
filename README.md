# The Solix Programming Language

[![CI](https://github.com/jehudme/solix-language/actions/workflows/ci.yml/badge.svg)](https://github.com/jehudme/solix-language/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-v1.1.0-green.svg)](https://github.com/jehudme/solix-language/releases)

Solix is an expressive, memory-safe, statically typed systems and application programming language. Built entirely in safe Rust (`#![forbid(unsafe_code)]`), Solix combines an Ahead-of-Time AST compiler with a high-throughput **64-bit Register Virtual Machine**, deterministic **Automatic Reference Counting (ARC)** with immediate free-block recycling, and a turnkey hermetic build pipeline.

---

## ⚡ Quick Start: Instant Installation

Install the official pre-built Solix toolchain and bundled standard packages with a single command:

### Linux & macOS (bash / zsh)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Jehudme/solix-language/releases/latest/download/solix-toolchain-installer.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://github.com/Jehudme/solix-language/releases/latest/download/solix-toolchain-installer.ps1 | iex
```

### Homebrew (macOS & Linux)
```bash
brew install solix-lang/tap/solix
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

---

## ✨ Code at a Glance

```solix
import blt.io.println;

class Particle {
    public float32 coord_x;
    public float32 coord_y;
    public uint8   alpha;

    public Particle(float32 initial_x, float32 initial_y) {
        this.coord_x = initial_x;
        this.coord_y = initial_y;
        this.alpha = 255;          // Context-typed numeric literal coercion
    }

    public void update(float32 delta_time) {
        this.coord_x += 10.5 * delta_time;
        this.coord_y += 5.0 * delta_time;
    }
}

class Main {
    public static int32 main() {
        Particle spark = new Particle(100.0, 200.0);
        spark.update(0.016);
        println("Particle position: " + spark.coord_x + ", " + spark.coord_y);
        return 0;
    }
}
```

---

## 🛠️ Unified Toolchain CLI (`solix`)

The single, consolidated `solix` executable manages all phases of project development, compilation, execution, debugging, and packaging:

| Command | Purpose | Example |
|:---|:---|:---|
| `solix new` | Scaffolds a new application or library project | `solix new my_game --deps raylib@5.5.0` |
| `solix compile` | Compiles `.slx` source files into a `.slxb` container | `solix compile src/main.slx -o bin/app.slxb` |
| `solix build` | Builds a project into an isolated `.slxarc` package | `solix build --profile release` |
| `solix run` | Executes `.slxb` binaries, `.slxarc` archives, or `.slx` sources | `solix run build/release/my_game.slxarc` |
| `solix clean` | Cleans project build artifacts (`build/`) or caches (`.cache/`) | `solix clean --cache-only` |
| `solix install` | Installs an archive or project into the local package store | `solix install ../libraries/raylib --profile release` |
| `solix reinstall` | Rebuilds and reinstalls packages from recorded source paths | `solix reinstall --all` |
| `solix uninstall` | Removes packages or specific versions from the local store | `solix uninstall raylib@5.5.0` |
| `solix list` | Lists all installed packages with sizes, profiles, and digests | `solix list --json` |
| `solix info` | Inspects container metadata, disassembly, or exported APIs | `solix info build/release/app.slxarc` |
| `solix lsp` | Starts the standard Language Server Protocol (LSP 3.17) | `solix lsp --stdio` |
| `solix dap` | Starts the Debug Adapter Protocol (DAP) service | `solix dap --port 45678` |
| `solix fmt` | Formats Solix source files according to official style | `solix fmt src/` |

---

## 🧩 Architectural Highlights

* **64-Bit Register Abstract Machine**: Uniform 8-byte instruction words (`Format A`, `Format B`, `Format C`, `Format Call`), sliding register windows, and linear scan allocation for high instruction cache locality.
* **Deterministic ARC with Segregated Free Lists**: Memory is reclaimed immediately when reference counts drop to zero, executing RAII destructors (`~ClassName()`) deterministically with zero GC pauses.
* **Hermetic Python Build Engine (`solix_build.py`)**: All native C/Zig/CMake and Cargo builds occur strictly inside `build/.cache/` without root contamination or host tool leakage.
* **Profile-Aware Multi-Version Package Store**: Supports simultaneous side-by-side installations of packages under `<name>-<profile>@<version>` with cryptographic SHA-256 tamper verification.
* **Comprehensive Tooling**: First-class Language Server Protocol (`solix-lsp`), Debug Adapter Protocol (`solix-dap`), and an official Visual Studio Code extension.

---

## 📚 Documentation Navigation

Comprehensive guides, specifications, and architecture deep dives are available in [`docs/`](docs/README.md):

* [**Master Documentation Hub**](docs/README.md) — Complete taxonomy and architecture guide.
* [**Getting Started Tutorial**](docs/guide/01_getting_started.md) — First steps with Solix, installation, and project layout.
* [**Hermetic Build System & Native Bridge**](docs/features/build-system.md) — Architecture of `solix_build.py`, `.cache/` isolation, and Zig/CMake pipelines.
* [**VS Code Extension & Debugging**](docs/guide/editors_vscode.md) — Editor setup, CodeLens execution, and interactive DAP debugging.
* [**CLI Toolchain Reference**](docs/spec/tools/cli/README.md) — Detailed option flags and execution contracts for all `solix` subcommands.
* [**Engineering Milestones Archive**](docs/internals/milestones/README.md) — Roadmap delivery logs and audit sign-off matrices (`MS-001` through `MS-043`).

---

## 🛡️ License

The Solix Programming Language and its standard toolchain are open-source software licensed under the [Apache License, Version 2.0](LICENSE).
