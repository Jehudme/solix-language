# Milestone 038: Profile-Aware Package Store (`<name>-<profile>@<version>`) & CLI Modernization

## Document Metadata
* **Milestone ID:** MS-038
* **Target Version:** `v0.1.0-rc.3`
* **Milestone Branch:** `milestone/038-profile-aware-package-store-and-cli-modernization`
* **Status:** DRAFT
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-20
* **Related Specifications:**
  * `docs/internals/specs/toolchain/package-manager-and-solver.md`
  * `docs/internals/specs/toolchain/cli-reference.md`
  * `docs/internals/specs/toolchain/project-manifest.md`

---

## 1. Executive Summary & Objectives

Upgrade the Solix package store, dependency solver (`ClientModuleSolver`), project manifest (`manifest.toml`), and `solix` CLI interface (`solix-toolchain`) to support side-by-side profile-qualified packages (`<name>-<profile>@<version>`), declarative `[build]` configuration in `manifest.toml`, and a unified, ergonomic set of CLI flags and subcommands.

Previously, the package store indexed packages solely by `<name>@<version>`, causing a `debug` installation of a package (`raylib@5.5.0`) to overwrite its `release` installation. This milestone introduces the shell-safe `<name>-<profile>@<version>` package specifier and directory layout (`$SOLIX_PKG_DIR/<name>-<profile>@<version>/<name>.<ext>`), implements automatic profile fallback in `ClientModuleSolver` (`release` $\to$ `debug` $\to$ `test` when no profile is explicitly requested), adds the `[build]` table to `manifest.toml`, renames `solix sync` to `solix reinstall` with source validation, and streamlines all CLI flags across `compile`, `run`, `build`, `info`, `install`, `reinstall`, `uninstall`, and `list`.

## 2. Scope & Crates Affected
* `crates/solix-common`:
  * `src/solver.rs`: Parse `<name>-<profile>@<version>` (and `<name>@<version>#<profile>` alias), add `profile: Option<String>` to `DependencyRequirement`, and implement `release` $\to$ `debug` $\to$ `test` fallback in `ClientModuleSolver`.
  * `src/registry.rs`: Key package store directories and `registry.json` records by `(name, build_profile, version)` under `$SOLIX_PKG_DIR/<name>-<profile>@<version>/`, and rename `source_project_dir` to `source` (with `#[serde(alias = "source_project_dir")]`).
  * `src/project.rs`: Add `[build]` configuration section (`BuildConfig`) to `ProjectManifest`.
* `crates/solix-compilation`: Pass profile-aware `DependencyRequirement` specifications through compilation pipeline dependency resolution.
* `crates/solix-runtime`: Support profile-aware dependency resolution when loading dynamic dependencies at runtime.
* `crates/solix-toolchain`: Refactor `src/cli.rs` and `src/commands/` (`compile.rs`, `run.rs`, `build.rs`, `info.rs`, `install.rs`, `reinstall.rs`, `uninstall.rs`, `list.rs`).
* `crates/solix-lsp`: Resolve workspace dependencies using the updated `ClientModuleSolver` profile fallback.
* `crates/solix-dap`: Not affected.
* `docs/`: Update CLI reference and package management specifications.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [ ] `[P0]` **Profile-Aware Dependency Specifier & Store Layout (`solver.rs`, `registry.rs`):**
  1. Extend `DependencyRequirement` with `pub profile: Option<String>`.
  2. Parse `<name>-<profile>@<version>` where `<profile>` $\in \{\text{"debug"}, \text{"release"}, \text{"test"}\}$ (ensuring hyphenated package names like `my-pkg@1.0.0` parse as `name = "my-pkg", profile = None`, whereas `my-pkg-debug@1.0.0` parses as `name = "my-pkg", profile = Some("debug")`), as well as `<name>@<version>#<profile>` and `<name>@<version>`.
  3. Store installed packages in `$SOLIX_PKG_DIR/<name>-<profile>@<version>/<name>.<ext>` (e.g., `solix-release@2.0.0/solix.slxarc` and `solix-debug@2.0.0/solix.slxarc`).
  4. Key `PackageRegistry` (`registry.json`) entries uniquely by `(name, build_profile, version)` while retaining backward-compatible loading for legacy `<name>@<version>` entries.
- [ ] `[P0]` **Deterministic Profile Fallback in `ClientModuleSolver` (`solver.rs`):**
  1. When `DependencyRequirement.profile` is `Some(req_profile)`, resolve strictly against installed packages matching `(name, req_profile)` and the SemVer requirement.
  2. When `DependencyRequirement.profile` is `None`, search for compatible versions under `release` first; if none exist, automatically fall back to `debug`, and then `test`.
- [ ] `[P0]` **Declarative `[build]` Configuration in `manifest.toml` (`project.rs`, `commands/build.rs`):**
  Add `pub build: BuildConfig` (`#[serde(default)]`) to `ProjectManifest` supporting:
  ```toml
  [build]
  profile = "debug"           # "debug", "release", or "test"
  link = "dynamic"            # "dynamic" or "static"
  pkg_dir = ""                # optional custom package store directory
  install = false             # auto-install artifact into store after build
  force_install = false       # overwrite existing store entry on install
  verify_checksum = true      # set false to bypass SHA-256 verification
  inline_threshold = 8        # AST inliner statement threshold
  ```

### 3.2 P1: High Priority (Milestone Core)
- [ ] `[P1]` **Overhaul `solix compile` and `solix run` Flags (`cli.rs`, `compile.rs`, `run.rs`):**
  1. Remove manual `[default: 8]` text from the `inline_threshold` doc comment in `CompileArgs` and `RunArgs` to eliminate the duplicate `[default: 8] [default: 8]` output in `--help`.
  2. Merge `--metrics` and `--metrics-json` into a single option `--metrics [<FORMAT>]` (`#[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]`), printing human-readable metrics when passed as `--metrics` or `--metrics text`, and JSON when passed as `--metrics json` / `--metrics=json`.
  3. Rename `--yolo` to `--no-verify` (with `visible_alias = "ignore-checksum"` and hidden `alias = "yolo"`).
  4. In `CompileArgs`, keep `-d, --deps <SPEC>` for package-store dependencies (`<name>[-<profile>]@<version>`), rename `--lib` to `--deps-file <PATH>` (retaining `-l` short flag and `visible_alias = "lib"`), and rename `--force` to `--force-install` (retaining `-f` short flag and `alias = "force"`).
- [ ] `[P1]` **Simplify `solix build` & `solix install` (`cli.rs`, `build.rs`, `install.rs`):**
  1. In `BuildArgs`, remove `--debug`, `--release`, `--test`, `--link`, `--static`, `--pkg-dir`, `--install`, `-f/--force`, and `--yolo`. Keep only `path` (default `"."`) and `--profile <PROFILE>` (optional override of `[build].profile` in `manifest.toml`). Forward resolved `[build]` configuration to `build.py` via `SOLIX_*` environment variables (and apply directly when `build.py` is absent).
  2. In `InstallArgs`, remove `--debug`, `--release`, and `--test` shorthand flags; keep `path`, `--profile <PROFILE>`, `-f, --force`, and `--pkg-dir <DIR>`.
- [ ] `[P1]` **Rename `solix sync` to `solix reinstall` with Source Validation (`cli.rs`, `reinstall.rs`, `registry.rs`):**
  1. Rename `PackageRecord.source_project_dir` to `source` (`#[serde(alias = "source_project_dir")]`) in `registry.json`, `solix list --json`, and `solix info --json`.
  2. Rename subcommand `Sync` to `Reinstall` (`solix reinstall [<SPEC>] [--all] [--pkg-dir <DIR>]`, keeping `sync` as a visible alias).
  3. For each targeted package record in `registry.json`, validate that `record.source` is recorded, the directory exists on disk, and `<source>/manifest.toml` matches `record.name` and `record.version`; then invoke `solix build <source> --profile <record.build_profile>` and force-install the rebuilt archive back into `$SOLIX_PKG_DIR/<name>-<profile>@<version>/`.
- [ ] `[P1]` **Upgrade `solix info`, `solix uninstall`, and `solix list` (`info.rs`, `uninstall.rs`, `list.rs`):**
  1. `solix info <TARGET>`: Accept file paths, `<name>-<profile>@<version>`, `<name>@<version>`, `<name>-<profile>`, or `<name>` (resolving `release` first then falling back to `debug` when profile is omitted).
  2. `solix uninstall <SPEC>`: Support removing a specific profile+version (`raylib-debug@5.5.0`), all profiles of a version (`raylib@5.5.0`), all versions of a profile (`raylib-debug`), or all versions and profiles of a package (`raylib`).
  3. `solix list`: Add `-o, --output <FILE>` to write the formatted table or JSON output directly to a file.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [ ] `[P2]` **CLI Help Text Standardization (`cli.rs`):** Audit and refine every command and argument description in `crates/solix-toolchain/src/cli.rs` so `solix --help` and `solix <subcommand> --help` are clear, consistent, and free of redundant phrasing.

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [ ] `[P3]` **Legacy Registry Migration Helper:** Automatically normalize legacy `<name>@<version>` entries in `registry.json` on load without failing if older packages exist in `~/.solix/packages`.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending |
| **Documentation Sync** | Specs and guides updated to GL-0010 | GL-0010 audit checklist | [ ] Pending |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** Package names containing hyphens (e.g., `my-math-lib@1.0.0`) could be misparsed as having a profile named `lib`.
  * *Mitigation:* Only split `-<suffix>` before `@` as a build profile when `<suffix>` is an exact match for a known profile (`"debug"`, `"release"`, or `"test"`). All other hyphens remain part of the package name.
* **Risk 2:** Existing CLI tests in `crates/solix-toolchain/tests/` may invoke old flags (`--yolo`, `--metrics-json`, `solix build --debug`).
  * *Mitigation:* Retain hidden backward-compatible aliases where appropriate (`alias = "yolo"`, `visible_alias = "lib"`, `visible_alias = "sync"`) and update all integration tests to exercise the new canonical flags and store layout.