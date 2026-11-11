# Milestone 025: Workspace Hygiene, Crate Reorganization (`solix-toolchain`), Self-Contained Embedded Assets, `solix-common` De-Bloat & `build/.cache` Relocation

## Document Metadata

* **Milestone ID:** MS-025
* **Target Version:** `v0.1.0-alpha.25`
* **Milestone Branch:** `milestone/025-workspace-hygiene-and-crate-reorganization`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/023-compiler-stabilization-qualified-declarations-and-dual-linking.md` (MS-023)
  * `docs/internals/milestones/024-exhaustive-documentation-specs-wiki-and-guidelines.md` (MS-024)

---

## 1. Executive Summary & Architectural Objectives

As the Solix workspace evolved through Milestones 001–023, several structural hygiene issues accumulated across the repository:
1. **Cluttered Repository Root (`scripts/` and `templates/`):**
   * The repository root contains `scripts/init_project.py` and `templates/untitled-cmake` + `templates/untitled-rust`.
   * Scaffolding scripts and project templates belong inside the CLI toolchain crate—not loose in the repository root where they can get out of sync or fail to be found when the `solix` binary is installed to `/usr/local/bin` or `~/.cargo/bin`!
2. **Misleading Crate Name (`solix-client`):**
   * `crates/solix-client` is not a network client; it is the unified **Solix CLI Driver & Toolchain (`solix-toolchain`)**.
3. **Bloated `solix-common` Crate:**
   * `crates/solix-common` currently holds `LexingError`, `ParsingError`, `BindingError`, and `CompilationDiagnostic` (`E0000`–`E0290`). However, `solix-runtime` never uses compiler AST/lexing/parsing/binding diagnostics! Any crate that needs compiler diagnostics (`solix-toolchain`, `solix-lsp`, `solix-tests`) already depends on `solix-compilation`. Keeping compiler diagnostics in `solix-common` bloats `solix-common` and couples the runtime to compiler internals.
4. **Top-Level Project `cache/` Directory Pollution:**
   * Building a Solix project currently creates both `<project_root>/cache/` and `<project_root>/build/`, cluttering user projects with two separate artifact folders. Moving the project cache inside `<project_root>/build/.cache/` (or `<project_root>/build/cache/`) keeps the project root completely clean with a single `build/` artifact directory.

**Milestone 025** cleans up the entire workspace through **Four Core Deliverables**:

1. **Rename `crates/solix-client` $\to$ `crates/solix-toolchain` (Binary `solix`):**
   * Renames the crate to `solix-toolchain` while keeping the compiled binary executable name as `solix`.
2. **Eliminate Root `scripts/` and `templates/` via Rust `build.rs` Asset Staging & Embedded Resources:**
   * Removes `/scripts/` and `/templates/` from the repository root completely.
   * Relocates `solix_toolbox.py` and `init_project.py` into `crates/solix-toolchain/assets/scripts/`.
   * Uses **both** compile-time embedding (`include_str!`) **AND** a `crates/solix-toolchain/build.rs` script that automatically copies `assets/` alongside the compiled `solix` binary in `target/<profile>/assets/` during `cargo build` (and falls back to extracting from `include_str!` if `solix` is moved as a standalone single binary!).
3. **De-Bloat `crates/solix-common`:**
   * Moves `LexingError`, `ParsingError`, `BindingError`, and `CompilationDiagnostic` from `crates/solix-common/src/diagnostics/` into **`crates/solix-compilation/src/diagnostics/`** (and re-exports them from `solix_compilation`).
   * Keeps `solix-common` strictly minimal: `bytecode` (`OpCode`, `Instruction`, `BinaryProgram`), `Span`, `Type`, `ProjectManifest`, `SlxarcArchive`, and `ModuleSolver`.
4. **Consolidate Project Build Cache into `<project_root>/build/.cache/`:**
   * Updates `solix_toolbox.py`, `SlxarcArchive`, and `solix-toolchain` so all intermediate native builds (`build/.cache/natives/`), staging trees (`build/.cache/stage/`), and runtime native extractions (`build/.cache/runtime_natives/`) live cleanly inside `build/.cache/`.

---

## 2. Detailed Architectural Design

### 2.1 Self-Contained Asset Management in `crates/solix-toolchain`

```
crates/solix-toolchain/
├── Cargo.toml                  # name = "solix-toolchain", [[bin]] name = "solix"
├── build.rs                    # Automatically stages assets/ into target/<profile>/assets/
├── assets/
│   └── scripts/
│       ├── init_project.py     # Project scaffolding generator
│       └── solix_toolbox.py    # Two-Tier Build Toolbox library
└── src/
    ├── assets.rs               # Resolves staged assets or materializes embedded `include_str!`
    ├── cli.rs
    ├── main.rs
    ├── solver.rs
    └── commands/
        ├── build.rs
        ├── compile.rs
        ├── info.rs
        ├── new.rs
        ├── pkg.rs
        └── run.rs
```

* **How `build.rs` + `assets.rs` Works:**
  1. During `cargo build`, `crates/solix-toolchain/build.rs` watches `assets/` (`cargo:rerun-if-changed=assets`) and copies `assets/scripts/*.py` into `<OUT_DIR>/../../../assets/scripts/` (next to the `target/<profile>/solix` binary).
  2. In `crates/solix-toolchain/src/assets.rs`, ` include_str!("../assets/scripts/init_project.py")` and `include_str!("../assets/scripts/solix_toolbox.py")` are also compiled directly into the executable's `.rodata` segment.
  3. When `solix new` or `solix build` runs, it checks `<exe_dir>/assets/scripts/` first; if the `solix` binary was copied to `/usr/local/bin/solix` without the `assets/` folder, `assets.rs` automatically writes the embedded scripts to `$SOLIX_HOME/cache/scripts/` and executes them seamlessly!
  4. **Result:** Root `/scripts/` and `/templates/` are 100% gone from the workspace root, and `solix` works anywhere on any machine!

### 2.2 De-Bloating `crates/solix-common`

```
┌────────────────────────────────────────────────────────────────────────────┐
│                   Crate Dependency & Ownership Boundary                    │
├────────────────────────────────────────────────────────────────────────────┤
│  crates/solix-common (Minimal Shared Core)                                 │
│    • bytecode::{OpCode, Instruction, BinaryProgram, ExportManifest}        │
│    • types::{Type, PrimitiveKind}                                          │
│    • span::Span                                                            │
│    • project::{ProjectManifest, SlxarcArchive}                             │
│    • solver::{PackageVersion, DependencyRequirement, ModuleSolver}         │
│    • diagnostics::emission::EmissionError (used by BinaryProgram/Archive)  │
├────────────────────────────────────────────────────────────────────────────┤
│  crates/solix-compilation (Depends on solix-common)                        │
│    • diagnostics::{LexingError, ParsingError, BindingError,                │
│                    CompilationDiagnostic}  <-- MOVED HERE!                 │
│    • phases::{dependency, lexing, parser, binder, export_api,              │
│               optimizer, emitter}                                          │
├────────────────────────────────────────────────────────────────────────────┤
│  crates/solix-runtime (Depends ONLY on solix-common)                       │
│    • exceptions::{RuntimeError, RuntimeErrorCode}                          │
│    • memory::VirtualHeap, vm::VirtualMachine, native::ffi                  │
└────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Clean Project Cache Inside `build/.cache/`

Every Solix project generated by `solix new` (and `libraries/solixlib`) uses `<project_root>/build/.cache/` instead of `<project_root>/cache/`:

```text
<project_root>/
├── manifest.toml
├── scripts/
│   ├── build.py
│   └── solix_toolbox.py
├── src/
├── tests/
├── natives/
├── resources/
├── lib/
└── build/
    ├── .cache/                        # <-- All intermediates consolidated inside build/.cache/
    │   ├── natives/<profile>/
    │   └── stage/<profile>/
    ├── debug/
    │   └── <name>.slxarc
    ├── release/
    │   └── <name>.slxarc
    └── test/
        └── <name>.slxarc
```

---

## 3. Phased Implementation Plan

### Phase 1: Rename `solix-client` to `solix-toolchain` & Embed `assets/` with `build.rs`
* [x] `[P0]` **Task 1.1:** Rename `crates/solix-client` to `crates/solix-toolchain` and update root `Cargo.toml` workspace members.
* [x] `[P0]` **Task 1.2:** Move `scripts/init_project.py` and `solix_toolbox.py` into `crates/solix-toolchain/assets/scripts/`.
* [x] `[P0]` **Task 1.3:** Implement `crates/solix-toolchain/build.rs` and `crates/solix-toolchain/src/assets.rs` to stage `assets/` to the target output directory on `cargo build` and provide embedded fallback via `include_str!`.
* [x] `[P0]` **Task 1.4:** Delete the root `/scripts/` and `/templates/` directories and remove any references to `untitled-cmake`.

### Phase 2: Move Compiler Diagnostics Out of `solix-common` into `solix-compilation`
* [x] `[P0]` **Task 2.1:** Move `lexing.rs`, `parsing.rs`, and `binding.rs` (along with `CompilationDiagnostic`) from `crates/solix-common/src/diagnostics/` into `crates/solix-compilation/src/diagnostics/`.
* [x] `[P0]` **Task 2.2:** Re-export `LexingError`, `ParsingError`, `BindingError`, `EmissionError`, and `CompilationDiagnostic` from `crates/solix-compilation/src/lib.rs`.
* [x] `[P0]` **Task 2.3:** Update imports across `solix-compilation`, `solix-toolchain`, `solix-lsp`, and `solix-tests`.

### Phase 3: Relocate Project Cache into `build/.cache/`
* [x] `[P1]` **Task 3.1:** Update `solix_toolbox.py` (in `crates/solix-toolchain/assets/scripts/` and `libraries/solixlib/scripts/`) so `prepare_staging_dirs` uses `project_root / "build" / ".cache"` instead of `project_root / "cache"`, and `.gitignore` ignores `/build/`.
* [x] `[P1]` **Task 3.2:** Delete the old `libraries/solixlib/cache/` directory and verify `solix build libraries/solixlib --test` places all staging and native build intermediates inside `libraries/solixlib/build/.cache/`.

### Phase 4: Mandatory Documentation Sync (`GL-0013` Pre-Merge Audit)
* [x] `[P1]` **Task 4.1:** Run `git diff main...HEAD` and update all references to `solix-client`, root `scripts/`/`templates/`, and `build/.cache/` across `docs/spec/` and `docs/guide/`.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Root Hygiene** | `/scripts` and `/templates` removed from repo root | `ls -la` at workspace root | [x] Completed |
| **Crate Renamed to `solix-toolchain`** | `crates/solix-toolchain` builds binary `solix` | `cargo build -p solix-toolchain` | [x] Completed |
| **Automatic Asset Staging** | `build.rs` + `include_str!` in `solix-toolchain` | `cargo test -p solix-toolchain` | [x] Completed |
| **`solix-common` De-Bloated** | Compiler error enums moved to `solix-compilation` | `cargo check --workspace` | [x] Completed |
| **Cache Inside `build/.cache/`** | No `<project>/cache/` created; uses `build/.cache/` | `solix build libraries/solixlib --debug` | [x] Completed |
| **Documentation Updated (`GL-0013`)** | All docs reflect `solix-toolchain` and `build/.cache/` | `git diff main...HEAD -- docs/` | [x] Completed |
| **Clippy & Formatting Clean** | Zero warnings across all crates | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |