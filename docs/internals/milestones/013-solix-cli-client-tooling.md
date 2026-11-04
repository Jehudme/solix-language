# Milestone 013: Solix CLI Tooling, Builtin Subsystems & Multi-Binary Host Driver

## Document Metadata
* **Milestone ID:** MS-013
* **Target Version:** `v0.1.0-alpha.13`
* **Milestone Branch:** `milestone/013-solix-cli-client`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-20
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/011-multi-binary-support-and-dynamic-linking.md` (MS-011)
  * `docs/internals/milestones/012-compiler-pipeline-modularization-and-stabilization.md` (MS-012)

---

## 1. Executive Summary & Purpose

Following the implementation of multi-binary dynamic linking (MS-011) and the modular compilation pipeline (MS-012), the developer experience requires an industrial-grade command-line interface (`solix`).

This milestone delivers:
1. **Built-in Standard Library Subsystem (`blt.io`)**:
   - Bundling standard I/O builtins inside `solix-compilation` (`package blt.io;`) providing top-level `print` and `println` native functions.
   - Controllable via `CompilationOptions.include_builtins` and the `--no-builtins` CLI flag.
   - Synchronized runtime registration in `solix-runtime`.
2. **`solix compile` Driver**:
   - Compiling multi-source projects with package metadata (`--name`, `--version`, `--description`).
   - Automated dependency resolution via `-l, --lib`: extracting manifests (`extract_api_manifest`) and synthesizing `.slx` API headers (`manifest_to_source`) from `.slxb` containers on the fly.
   - `--emit-api <PATH>` for publishing library headers and `--check` for fast semantic verification.
3. **`solix run` Multi-Binary Equal Host**:
   - Peer multi-binary execution loading any number of `.slxb` containers into a single virtual machine context without arbitrary "main" vs "lib" distinctions.
   - Entry point discovery (`Main::main` / `main` or overridden via `--entry`).
   - Forwarding CLI arguments (`args: Vec<String>`) to the Solix entry point.
4. **`solix info` Container Inspection**:
   - Printing human-readable package metadata and export summaries.
   - Emitting JSON manifests (`--json`), synthesized Solix interface code (`--api`), or disassembled bytecode instructions (`--disasm`).
   - Writing output directly to a file via `-o, --output <PATH>` or standard output.
5. **High-Fidelity Terminal Diagnostics (`miette`)**:
   - Professional colorized diagnostic error output with line numbers, source snippets, carets, error codes (`E0101`, `E0203`, etc.), and remediation help.
6. **Hermetic Integration Test Suite (`crates/solix-client/tests`)**:
   - Exhaustive CLI test coverage verifying compilation, dependency auto-extraction, execution, inspection, and error handling.

---

## 2. Command Architecture & Design Specification

### 2.1 The CLI Command Structure (`clap`)

```rust
#[derive(Parser)]
#[command(
    name = "solix",
    author = "Jehud Metellus",
    version,
    about = "The Solix programming language compiler and runtime toolchain",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compiles Solix source files into an executable binary container (.slxb)
    Compile {
        /// Input Solix source files (*.slx)
        #[arg(required = true)]
        sources: Vec<PathBuf>,

        /// Output compiled binary file path [default: <name>.slxb]
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Paths to library binary containers (*.slxb) whose API interfaces should be extracted
        #[arg(short = 'l', long = "lib", alias = "dep")]
        libs: Vec<PathBuf>,

        /// Package or library name
        #[arg(short, long)]
        name: Option<String>,

        /// Package semantic version [default: 0.1.0]
        #[arg(short = 'V', long = "version", default_value = "0.1.0")]
        version: String,

        /// Human-readable package description
        #[arg(long)]
        description: Option<String>,

        /// Save synthesized API interface source to disk (*.slx)
        #[arg(long)]
        emit_api: Option<PathBuf>,

        /// Do not automatically include built-in standard library definitions (blt.io)
        #[arg(long)]
        no_builtins: bool,

        /// Perform syntax and semantic checks without emitting bytecode
        #[arg(long)]
        check: bool,

        /// Print wall-clock duration for each compilation phase
        #[arg(long)]
        time: bool,
    },

    /// Executes compiled Solix binary containers in the virtual machine
    Run {
        /// Executable binary containers to load (*.slxb)
        #[arg(required = true)]
        binaries: Vec<PathBuf>,

        /// Override default entry point symbol (e.g. "Main::main")
        #[arg(short, long)]
        entry: Option<String>,

        /// Virtual heap size limit (e.g., "32MB", "64MB") [default: 32MB]
        #[arg(long, default_value = "32MB")]
        heap_size: String,

        /// Maximum call stack depth limit [default: 1024]
        #[arg(long, default_value_t = 1024)]
        max_stack: usize,

        /// Print execution statistics and memory report upon completion
        #[arg(short, long)]
        verbose: bool,

        /// Arguments passed to the Solix application entry point
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Inspects metadata, export manifests, and bytecode of an SLXB binary container
    Info {
        /// Target binary container file (*.slxb)
        #[arg(required = true)]
        binary: PathBuf,

        /// Output file destination for inspection data [default: stdout]
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output the raw export manifest as formatted JSON
        #[arg(long)]
        json: bool,

        /// Print the reconstructed Solix API interface source (.slx)
        #[arg(long)]
        api: bool,

        /// Disassemble bytecode instructions
        #[arg(long)]
        disasm: bool,
    },
}
```

---

## 3. Definition of Done (DoD) Matrix

| Requirement | Acceptance Criteria | Verification Method |
| :--- | :--- | :--- |
| **Builtins Subsystem** | `package blt.io;` provides top-level `print` and `println`; toggleable via `include_builtins`. | Unit & integration tests |
| **`compile` Command** | Compiles single/multi sources, auto-extracts API from `-l` binaries, generates `.slxb` & `--emit-api`. | CLI tests in `solix-client` |
| **`run` Command** | Loads multiple `.slxb` binaries as equal peers, detects entry point, passes CLI args, returns exit code. | CLI tests in `solix-client` |
| **`info` Command** | Dumps metadata, `--json`, `--api`, and `--disasm` to stdout or file via `-o, --output`. | CLI tests in `solix-client` |
| **Diagnostic Rendering** | `miette` outputs colorized diagnostics with line numbers, carets, error codes, and help messages. | CLI tests in `solix-client` |
| **Zero Warnings & Safe Code** | `#![forbid(unsafe_code)]`, `cargo clippy -- -D warnings`, `cargo fmt --check`. | Workspace automated checks |

---

## 4. Phased Roadmap

* [x] **Step 0: Specification Document & Milestone Branch Inception**
* [x] **Phase 1: Builtins Subsystem (`blt.io` in `solix-compilation` & `solix-runtime`)**
* [x] **Phase 2: CLI Model & Argument Parser (`solix-client`)**
* [x] **Phase 3: High-Fidelity Diagnostic Terminal Reporter (`solix-client`)**
* [x] **Phase 4: The `compile` Command & Dynamic Dependency API Extraction (`solix-client`)**
* [x] **Phase 5: The `run` Command & Peer Multi-Binary Execution (`solix-client`)**
* [x] **Phase 6: The `info` Command with Output & Inspection Flags (`solix-client`)**
* [x] **Phase 7: Hermetic Integration Test Suite & Verification (`solix-client/tests`)**
