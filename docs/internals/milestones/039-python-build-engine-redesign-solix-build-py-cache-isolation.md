# Milestone 039: Python Build Engine Redesign (`solix_build.py`), Rich Diagnostics & `.cache` Isolation

## Document Metadata
* **Milestone ID:** MS-039
* **Target Version:** `v0.1.0-rc.4`
* **Milestone Branch:** `milestone/039-python-build-engine-and-cache-isolation`
* **Status:** DRAFT
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-22
* **Related Specifications:**
  * `docs/features/build-system.md`
  * `docs/internals/specs/toolchain/project-manifest.md`
  * `docs/internals/specs/runtime/native-ffi-bridge.md`

---

## 1. Executive Summary & Objectives

Redesign and standardize the Python build and project-scaffolding engines across the Solix ecosystem by renaming `solix_toolbox.py` to `solix_build.py` and `init_project.py` to `solix_scaffold.py`, introducing structured terminal diagnostics (`BuildError`), enforcing strict hermetic artifact isolation inside `build/.cache/` for all native Rust and C builds, and authoring the comprehensive build system specification in `docs/features/build-system.md`.

Previously, build failures inside `solix_toolbox.py` (such as a failed CMake step, missing `zig` compiler, or failed `solix compile` invocation) produced raw Python `subprocess.CalledProcessError` tracebacks, and native Cargo builds in `libraries/solixlib` or scaffolded templates could create `natives/*/target/` directories outside `build/.cache/`. This milestone eliminates raw Python tracebacks in favor of actionable diagnostic panels and guarantees 100% of intermediate native and bytecode build artifacts reside under `build/.cache/`.

## 2. Scope & Crates Affected
* `crates/solix-common`: Not affected.
* `crates/solix-compilation`: Not affected.
* `crates/solix-runtime`: Not affected.
* `crates/solix-toolchain`:
  * `assets/scripts/solix_build.py` (renamed from `solix_toolbox.py`)
  * `assets/scripts/solix_scaffold.py` (renamed from `init_project.py`)
  * `assets/templates/untitled-rust/scripts/solix_build.py` and `build.py`
  * `src/assets.rs`, `src/commands/new.rs`, and `build.rs`
* `libraries/solixlib`: `build.py`, `scripts/solix_build.py`, and `natives/solix_native/.cargo/config.toml`.
* `libraries/raylib`: `build.py`, `scripts/solix_build.py`, and `natives/raylib/.cargo/config.toml`.
* `docs/`: Author `docs/features/build-system.md`.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [ ] `[P0]` **Rename & Unify `solix_build.py` and `solix_scaffold.py` Across Workspace:**
  1. Rename `solix_toolbox.py` $\to$ `solix_build.py` in `crates/solix-toolchain/assets/scripts/`, `crates/solix-toolchain/assets/templates/untitled-rust/scripts/`, `libraries/solixlib/scripts/`, and `libraries/raylib/scripts/`.
  2. Rename `init_project.py` $\to$ `solix_scaffold.py` in `crates/solix-toolchain/assets/scripts/`.
  3. Update all imports in `build.py` files (`from scripts.solix_build import ...`) and all script path references in `crates/solix-toolchain/src/assets.rs`, `crates/solix-toolchain/src/commands/new.rs`, and `crates/solix-toolchain/build.rs`.
- [ ] `[P0]` **Structured Build Error Reporting & Diagnostic UI (`solix_build.py`):**
  1. Implement `BuildError(Exception)` encapsulating `phase: str`, `message: str`, `command: list[str] | None`, `cwd: Path | None`, `exit_code: int | None`, `stderr: str | None`, and `hint: str | None`.
  2. Implement `run_command(cmd, *, cwd, env, phase, hint)` and `run_cli(main_fn)` so any failed CMake, `rlparser`, IR codegen, Cargo/`zigbuild`, or `solix compile` subprocess renders a clean, structured `[SOLIX BUILD ERROR]` diagnostic report with the failed phase, command, working directory, compiler error output, and remediation hint—suppressing raw Python tracebacks unless `SOLIX_BUILD_TRACEBACK=1` is set.
- [ ] `[P0]` **Strict `build/.cache/` Isolation for All Native C & Rust Crates:**
  1. Enforce `CARGO_TARGET_DIR = <project_root>/build/.cache/cargo` across all Cargo and `cargo zigbuild` invocations in `solix_build.py`.
  2. Add `.cargo/config.toml` (`[build]\ntarget-dir = "../../build/.cache/cargo"`) inside `libraries/solixlib/natives/solix_native/`, `libraries/raylib/natives/raylib/`, and `crates/solix-toolchain/assets/templates/untitled-rust/natives/untitled_native/` (and generate it in `solix_scaffold.py`) so direct `cargo` or `rust-analyzer` invocations inside `natives/*` never create `natives/*/target/`.
  3. Delete any leftover `natives/*/target/` directories from the repository.

### 3.2 P1: High Priority (Milestone Core)
- [ ] `[P1]` **Full `manifest.toml` `[build]` Integration in `solix_build.py`:** Read `[build]` settings from `manifest.toml` (`profile`, `link`, `pkg_dir`, `install`, `force_install`, `verify_checksum`, `inline_threshold`) merged with `SOLIX_*` environment variables passed by `solix build`. When `install = true` (or `SOLIX_AUTO_INSTALL=1`), automatically invoke `solix install <artifact> --profile <profile> [--force] [--pkg-dir <dir>]` upon build completion.
- [ ] `[P1]` **Rebuild & Verify `solixlib`, `raylib`, and `examples/breakout`:**
  1. Build and install `libraries/solixlib` in both `debug` (`solix-debug@2.0.0`) and `release` (`solix-release@2.0.0`) profiles.
  2. Build and install `libraries/raylib` in `debug` (`raylib-debug@5.5.0`) and `release` (`raylib-release@5.5.0`) profiles.
  3. Verify `examples/breakout` builds and runs cleanly against the installed packages.
- [ ] `[P1]` **Author Build System Documentation (`docs/features/build-system.md`):** Document the `solix build` architecture, `solix_build.py` Tier-1 primitives and Tier-2 `build_project()` orchestrator, custom pre-build hooks (CMake + `rlparser` + IR codegen), `[build]` manifest configuration, `build/.cache/` isolation layout, multi-target Zig cross-compilation, and structured error diagnostics.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [ ] `[P2]` **Phase Timing Summary in `solix_build.py`:** Print elapsed duration per build phase (Native C/Rust compilation, Solix bytecode compilation, `.slxarc` packaging, and store installation).

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [ ] `[P3]` **Python Type Annotations (`solix_build.py`, `solix_scaffold.py`):** Ensure full Python 3.10+ type hints (`Path`, `dataclass`, union types) across all helper functions.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending |
| **Documentation Sync** | `docs/features/build-system.md` verified to GL-0010 | GL-0010 audit checklist | [ ] Pending |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** `solix new` tests or build integration tests may look for `solix_toolbox.py` or `init_project.py` by filename.
  * *Mitigation:* Update all references in `crates/solix-toolchain/src/assets.rs`, `src/commands/new.rs`, `build.rs`, and integration tests simultaneously, and verify `cargo test --workspace` passes 100%.
* **Risk 2:** Relative `target-dir = "../../build/.cache/cargo"` in `.cargo/config.toml` must resolve relative to the native crate root (`natives/<crate>/`).
  * *Mitigation:* Cargo resolves relative `build.target-dir` paths in `.cargo/config.toml` relative to the directory containing `.cargo/`, so `../../build/.cache/cargo` inside `natives/<crate>/.cargo/config.toml` deterministically resolves to `<project_root>/build/.cache/cargo`.