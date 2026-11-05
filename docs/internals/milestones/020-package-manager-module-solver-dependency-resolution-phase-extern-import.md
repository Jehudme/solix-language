# Milestone 020: Cross-Platform Package Manager (`install`/`uninstall`/`list`), Pluggable `ModuleSolver`, Pre-Lex `DependencyResolutionPhase` & `extern import`

## Document Metadata

* **Milestone ID:** MS-020
* **Target Version:** `v0.1.0-alpha.20`
* **Milestone Branch:** `milestone/020-package-manager-and-solver`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/011-multi-binary-support-and-dynamic-linking.md` (MS-011)
  * `docs/internals/milestones/012-compiler-pipeline-modularization-and-stabilization.md` (MS-012)
  * `docs/internals/milestones/019-project-build-system-slxarc-and-native-loader.md` (MS-019)

---

## 1. Executive Summary & Architectural Audit of Your Plan

After analyzing `solix-common`, `solix-compilation`, `solix-runtime`, and `solix-client`, here is how your compiler currently works and why **your new plan is the exact right architecture for Solix** (with 4 subtle edge cases refined below to make it 100% foolproof):

### 1.1 Why Your Plan Fits the Existing Codebase Perfectly
1. **Why `manifest_to_source` + Lexing/Parsing the API is Required:**
   * In `solix-compilation/src/manifest.rs`, `manifest_to_source(manifest_json)` converts a binary's `ExportManifest` into a synthetic Solix interface file (`<pkg>@<version>.slx`) with `extern` declarations and generic template sources.
   * Your `BinderPhase` (`pass1_symbol_discovery.rs`, `pass2_type_hierarchy.rs`, `pass3_type_checking.rs`) and `GenericBlueprintRegistry` require AST declarations (`ProgramAst`) in `context.ast` to resolve types, check inheritance, and monomorphize generics.
   * Therefore, extracting the API from `.slxb` (and `.slxb` inside `.slxarc`) via `manifest_to_source` and lexing/parsing that synthesized `.slx` source into `CompilationContext` is 100% consistent with your compiler!
2. **Why `extern import` (`KeywordExtern` + `KeywordImport`) Is a Great Idea for Solix:**
   * You already have `TokenKind::KeywordExtern` (`extern`) in `solix-common/src/tokens.rs`!
   * Right now, in `parser.rs:344-351`, any modifier before `import` triggers `E0185` (`ModifierOnModuleStatement`).
   * By allowing **`extern import raylib.*;`** or **`extern import raylib.initWindow;`** (while keeping `public import` etc. forbidden with `E0185`), you give the programmer and the parser an **explicit, unambiguous instruction**:
     * Regular `import foo.Bar;` $\to$ looks up symbols already present in the current project or declared in `manifest.toml` (`--deps`).
     * `extern import raylib.initWindow;` $\to$ tells the compiler during `ParserPhase`: *"If `raylib` is not already in `context.dependencies_map`, immediately ask the `ModuleSolver` for the highest installed version of `raylib`, validate its transitive dependency tree, synthesize its API source via `manifest_to_source`, lex and parse it on the fly, and record `raylib@<version>` in `context.dependencies_map`!"*
   * This solves standalone single-file script compilation (`solix compile game.slx` or `solix run game.slx`) without needing a `manifest.toml` or CLI `--deps` flags, while project builds can also declare `[dependencies]` in `manifest.toml`!

---

### 1.2 Four Crucial Edge Cases Fixed to Make Your Plan 100% Foolproof

While reviewing the codebase against your plan, I found **4 critical edge cases** that we must handle so you never hit hidden bugs:

#### Edge Case 1: Package Namespace Qualification in `manifest_to_source` & Root Facade Aliasing (`raylib.initWindow` vs `raylib.raylib.initWindow`)
* **The Bug in Current Code (`manifest.rs:19-121`):**
  Look at `crates/solix-compilation/src/manifest.rs`: right now, `manifest_to_source` **does NOT emit `package <pkg_name>;` at the top of the synthesized `<pkg_name>@<version>.slx` file!** Because there is no `package` statement, `pass1_symbol_discovery.rs` registers all exported functions/classes in the **global root namespace** (`initWindow` instead of `raylib::initWindow`), which means `import raylib.initWindow;` or `import raylib.*;` actually **fails with `E0232: CannotResolveImport`**!
* **The Foolproof Fix:**
  1. Update `manifest_to_source` so that if `manifest.package_name` is non-empty (e.g., `"raylib"`), it emits `package raylib;` at the top of `<pkg_name>@<version>.slx`.
  2. **Eliminating the `raylib.raylib.initWindow` Stutter:**
     In `pass1_symbol_discovery.rs` and `symbols.rs`:
     * When `import raylib.*;` or `extern import raylib.*;` is used, `raylib`'s public functions (`initWindow`) and classes (`Vector2`) are brought into scope so you can call `initWindow(...)` directly, **OR** call `raylib.initWindow(...)` / `raylib::initWindow(...)` via package-prefix qualification!
     * And if a library `raylib` has a class `raylib` (or package `raylib`), `BinderPhase` automatically aliases `raylib.<member>` to the root exported symbol so the user **never** has to write `raylib.raylib.initWindow(...)`!

#### Edge Case 2: Deduping & Upgrading Transitive Dependencies When Version 1.0.0 is Replaced by 1.2.0
* **The Trap:**
  Suppose `--deps` (or `extern import`) first loads `math@1.0.0`, and then another package requires `math@1.2.0` (which is compatible: same major `1`, higher minor `2 >= 0`). If we already lexed and parsed `math@1.0.0.slx` into `context.sources`, and then we *also* lex and parse `math@1.2.0.slx`, `BinderPhase` will fail with **`E0202: DuplicateSymbol`** because `math`'s classes/functions exist twice in `context.ast`!
* **The Foolproof Fix:**
  1. **In `DependencyResolutionPhase` (Phase 0, before Lexing):**
     First, traverse the **entire** transitive dependency graph (BFS/DFS) using `ModuleSolver` and build `context.dependencies_map: HashMap<String, ResolvedDependency>` in memory **before** generating any API source!
     During graph traversal, if `math@1.0.0` is in the map and `math@1.2.0` is encountered:
     * Check compatibility: `1 == 1` (same major) $\to$ compatible! Keep the highest version (`1.2.0`).
     * Only **after** the entire dependency tree is resolved and deduplicated to the single highest compatible version per package do we call `manifest_to_source` once per unique package and inject `<name>@<winning_version>.slx` into `context`!
  2. **In `ParserPhase` (when encountering `extern import pkg...`):**
     * Check if `pkg` is already in `context.dependencies_map`.
     * If `pkg` is **already** in `context.dependencies_map`, we don't need to re-fetch or re-parse it—its winning version is already loaded!
     * If `pkg` is **not** in `context.dependencies_map`, query `ModuleSolver::resolve_latest("pkg")`, resolve its transitive dependencies against `context.dependencies_map` (checking compatibility), and for any newly added package (or upgraded package, replacing the old AST slot in `context.ast`), synthesize its API via `manifest_to_source`, lex it, and parse it immediately!

#### Edge Case 3: Native Shared Libraries (`.so` / `.dll` / `.dylib`) Inside Installed `.slxarc` Packages
* **The Trap:**
  When a library like `raylib@1.0.0.slxarc` is installed and imported by `app.slxb`, `app.slxb` only stores dependency metadata (`dependencies: [raylib@1.0.0]`). When `solix run app.slxb` runs on PC B, `RuntimeContext::new` uses the `ModuleSolver` to fetch `raylib@1.0.0.slxarc`. It must load **both** the bytecode (`bin/raylib.slxb`) **AND** extract/load the host OS native shared library (`lib/libraylib.so` / `raylib.dll` / `libraylib.dylib`)!
* **The Foolproof Fix:**
  * When `ModuleSolver` resolves an installed `.slxarc` archive:
    * For **Compilation** (`DependencyResolutionPhase` / `ParserPhase`): It only needs the `BinaryProgram` (`bin/*.slxb`) to extract `ExportManifest` and `dependencies`.
    * For **Runtime** (`RuntimeContext::new`): It extracts `bin/*.slxb` **AND** extracts the host platform's native shared libraries (`.so`, `.dll`, `.dylib`) into the native cache directory (`~/.solix/cache/natives/<name>-<version>/`) and registers them into `RuntimeOptions.native_libraries`!

#### Edge Case 4: Keeping `CompilationOptions` Ergonomic for Both CLI (`--deps`) and In-Memory Tests
* **Your Requirement:**
  * `solix compile` replaces `-l <path>` with `--deps <name>@<version>` (e.g. `solix compile src/ --deps raylib@1.0.0`).
  * `CompilationOptions` has a vector of pre-loaded `BinaryProgram`s (`dependency_binaries: Vec<BinaryProgram>`) **and** dependency specs (`dependency_specs: Vec<DependencySpec>`) + an `Option<Arc<dyn ModuleSolver>>`.
  * Before lexing, `DependencyResolutionPhase` inspects all `dependency_binaries` and `dependency_specs`, extracts their `package_name`, `package_version`, and transitive `dependencies`, uses `ModuleSolver` to fetch all transitive `.slxb`/`.slxarc` dependencies, verifies SemVer compatibility across the entire tree, populates `context.dependencies_map`, and injects their synthesized API `.slx` sources!

---

## 2. Exact SemVer Compatibility Law

Every package version in Solix follows strict 3-component Semantic Versioning:
$$\text{Version} = \text{Major}.\text{Medium}.\text{Minor} \quad (\text{e.g. } 1.2.3)$$
*(Note: In standard SemVer terminology, these three numbers are often called `major.minor.patch`; in our codebase we will support both names `.medium()` / `.minor()` so the rule is 100% explicit).*

### 2.1 Compatibility Predicate: `is_compatible_with(required, candidate)`
Given a required dependency version $V_{\text{req}} = (M_{\text{req}}, D_{\text{req}}, m_{\text{req}})$ and an installed/candidate version $V_{\text{cand}} = (M_{\text{cand}}, D_{\text{cand}}, m_{\text{cand}})$:

1. **Major Version Must Match Exactly:**
   $$M_{\text{cand}} == M_{\text{req}}$$
   If $M_{\text{cand}} \neq M_{\text{req}}$, the versions are **incompatible**!
2. **Medium Version Must Be Equal or Higher:**
   $$D_{\text{cand}} \ge D_{\text{req}}$$
   If $D_{\text{cand}} < D_{\text{req}}$, the candidate is **incompatible** (too old)!
3. **Minor Version Rule:**
   * If $D_{\text{cand}} == D_{\text{req}}$, then $m_{\text{cand}} \ge m_{\text{req}}$ (cannot be lower than the required minor version when medium is the same!).
   * If $D_{\text{cand}} > D_{\text{req}}$, any $m_{\text{cand}} \ge 0$ is valid, and the solver **always selects the highest available $(D_{\text{cand}}, m_{\text{cand}})$**.

### 2.2 Deduplication / Merging Rule in the Dependency Map
When multiple packages in the dependency tree depend on the same package `P` with versions $V_1$ and $V_2$:
1. If $V_1.\text{major} \neq V_2.\text{major}$:
   * **Immediate Error!** Emit compiler diagnostic `E0289` (`IncompatibleDependencyVersion`) or runtime error `RuntimeError::IncompatibleDependency` detailing the conflicting versions and which parent packages requested them.
2. If $V_1.\text{major} == V_2.\text{major}$:
   * Compute $V_{\max} = \max(V_1, V_2)$ using lexicographical tuple comparison $(M, D, m)$.
   * Because $V_{\max}$ has the same major version and $\ge$ medium/minor of both $V_1$ and $V_2$, $V_{\max}$ is guaranteed compatible with both!
   * Keep only $V_{\max}$ in `dependencies_map` (and ask the `ModuleSolver` for the highest installed version in the store that is compatible with $V_{\max}$).

---

## 3. Architecture of the 6 Subsystems in Milestone 020

### 3.1 Subsystem 1: Cross-Platform Package Store & `install` / `uninstall` / `list` CLI (`solix-client`)

#### Cross-Platform Store Location:
Resolved deterministically across Linux, macOS, and Windows:
1. If environment variable `SOLIX_PKG_DIR` is set $\to$ use `$SOLIX_PKG_DIR` (essential for hermetic parallel testing!).
2. Else if `SOLIX_HOME` is set $\to$ use `$SOLIX_HOME/packages`.
3. Else on Windows (`cfg!( windows )`) $\to$ `%USERPROFILE%\.solix\packages` (or `%LOCALAPPDATA%\solix\packages`).
4. Else on Linux / macOS $\to$ `$HOME/.solix/packages`.

#### Store Directory Structure:
```text
~/.solix/packages/
├── raylib/
│   ├── 1.0.0/
│   │   └── raylib.slxarc
│   └── 1.2.0/
│       └── raylib.slxarc
└── math_utils/
    └── 2.1.0/
        └── math_utils.slxb
```

#### New CLI Subcommands in `solix-client`:
1. **`solix install <PATH>`:**
   * Accepts a path to a `.slxb` or `.slxarc` file.
   * If `.slxb`: deserializes `BinaryProgram::from_bytes`, validates `package_name` (must not be empty) and `package_version` (must be valid `Major.Medium.Minor`), and copies the file to `<store>/<name>/<version>/<name>.slxb`.
   * If `.slxarc`: inspects `SlxarcArchive::inspect`, reads `manifest.toml` (`info.name`, `info.version`) and verifies the embedded `.slxb`, then copies the archive to `<store>/<name>/<version>/<name>.slxarc`.
2. **`solix uninstall <SPEC>`:**
   * Accepts `<name>@<version>` (e.g. `solix uninstall raylib@1.0.0` — removes only that version) or `<name>` (e.g. `solix uninstall raylib` — removes all installed versions of `raylib`).
   * Cleans up any cached native libraries for that package version in `<store>/../cache/natives/`.
3. **`solix list [--json]`:**
   * Scans `<store>` and prints a clean, deterministic table of all installed packages, their versions (sorted ascending), kind (`slxb` vs `slxarc`), and description (or machine-readable JSON when `--json` is passed).

---

### 3.2 Subsystem 2: The `ModuleSolver` Trait & `ClientModuleSolver` (`solix-common` & `solix-client`)

In `crates/solix-common/src/solver.rs`:
```rust
/// Parsed semantic version `Major.Medium.Minor` (e.g. `1.2.3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageVersion {
    pub major: u32,
    pub medium: u32,
    pub minor: u32,
}

/// A dependency requirement recorded in `ExportManifest`, `manifest.toml`, or `--deps`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyRequirement {
    pub name: String,
    pub version: PackageVersion,
}

/// The payload returned by a `ModuleSolver` when a package is fetched.
#[derive(Debug, Clone)]
pub struct SolvedPackage {
    pub name: String,
    pub version: PackageVersion,
    /// Primary compiled bytecode container (.slxb) of the package.
    pub program: BinaryProgram,
    /// Secondary .slxb binaries if bundled inside a .slxarc archive.
    pub secondary_programs: Vec<BinaryProgram>,
    /// Extracted host-platform native shared libraries (.dll / .so / .dylib) if this was a .slxarc.
    pub native_libraries: Vec<PathBuf>,
}

/// Pluggable trait for fetching external packages during compilation and runtime.
pub trait ModuleSolver: Send + Sync + std::fmt::Debug {
    /// Finds and loads the highest installed version of `package_name` that is compatible
    /// with `min_version` (same major, >= medium, and >= minor if same medium).
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError>;

    /// Finds and loads the absolute highest installed version of `package_name`
    /// (used by `extern import <pkg>.*` when no version constraint was pre-specified).
    fn resolve_latest(&self, package_name: &str) -> Result<SolvedPackage, SolverError>;
}
```

* **`ClientModuleSolver` (in `solix-client` / `solix-common`):**
  * Looks inside the cross-platform installed packages folder (`~/.solix/packages/<package_name>/`).
  * Lists all installed `<version>` subdirectories, parses their `PackageVersion`, filters by `v.is_compatible_with(min_version)` (or all versions for `resolve_latest`), picks the **maximum** version, and loads the `.slxb` or `.slxarc` (extracting native libraries to `~/.solix/cache/natives/<name>-<version>/`).
* **`InMemoryModuleSolver` (in `solix-common`):**
  * A solver backed by a `Vec<SolvedPackage>` in memory so unit tests in `solix-compilation`, `solix-runtime`, and `solix-tests` can test custom solver behavior without touching the filesystem!

---

### 3.3 Subsystem 3: Storing Dependencies in `.slxb` (`ExportManifest` & `BinaryProgram`)

In `crates/solix-common/src/bytecode/manifest.rs`:
* Add `#[serde(default)] pub dependencies: Vec<DependencyRequirement>` to `ExportManifest` (and `BinaryProgram`).
* Because `ExportManifest` is serialized as JSON in Section 6 of `.slxb` (`binary.rs:347`), adding `#[serde(default)] pub dependencies: Vec<DependencyRequirement>` is 100% backward-compatible with existing `.slxb` tests and automatically persists the resolved dependency map into every compiled `.slxb`!

---

### 3.4 Subsystem 4: `manifest.toml` `[dependencies]`, `solix_toolbox.py`, and `solix compile --deps`

1. **`ProjectManifest` (`crates/solix-common/src/project.rs`):**
   Update `manifest.toml` schema to support the `[dependencies]` section just like Cargo:
   ```toml
   [info]
   name = "my_game"
   version = "0.1.0"

   [entrypoints]
   main = "Main::main"
   test = "TestRunner::test"

   [dependencies]
   raylib = "1.0.0"
   math_utils = "2.1.0"
   ```
   In `ProjectManifest`:
   ```rust
   #[serde(default)]
   pub dependencies: BTreeMap<String, String>,
   ```
2. **`solix compile --deps <name>@<version>` (`crates/solix-client/src/cli.rs` & `compile.rs`):**
   * Replace `-l, --lib <PATH>` on `solix compile` with `--deps <NAME>@<VERSION>` (accepting multiple `--deps raylib@1.0.0 --deps math@2.1.0` or `-d raylib@1.0.0`).
   * `execute_compile` parses each `<name>@<version>` into a `DependencyRequirement`, attaches `ClientModuleSolver` to `CompilationOptions`, and lets `DependencyResolutionPhase` resolve and validate everything!
3. **`solix_toolbox.py` (`templates/untitled-rust/scripts/solix_toolbox.py`):**
   * In `load_manifest()`, read `manifest.get("dependencies", {})`.
   * In `compile_solix_sources()`, for each `dep_name, dep_ver` in `manifest["dependencies"]`, automatically append `["--deps", f"{dep_name}@{dep_ver}"]` to the `solix compile` command!

---

### 3.5 Subsystem 5: Compiler Refactor — `DependencyResolutionPhase` & `extern import`

```
┌────────────────────────────────────────────────────────────────────────────┐
│                   Updated Solix Compilation Pipeline                       │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 0: DependencyResolutionPhase (NEW - Runs BEFORE Lexing!)             │
│   1. Inspects `options.dependency_binaries` (raw .slxb/.slxarc data)       │
│      and `options.dependency_specs` (`--deps raylib@1.0.0`).               │
│   2. Uses `options.solver` to fetch root & transitive dependencies.        │
│   3. Verifies SemVer compatibility across the entire dependency tree:      │
│      - Same major, >= medium, highest minor -> keeps most recent version.  │
│      - Different major or lower incompatible version -> emits E0289/E0290! │
│   4. Populates `context.dependencies_map`.                                 │
│   5. Extracts APIs via `manifest_to_source` & registers synthetic sources. │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 1: LexerPhase                                                        │
│   Tokenizes user sources + pre-loaded dependency API sources.              │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: ParserPhase (Enhanced with `extern import`)                       │
│   1. Parses `extern import raylib.initWindow;` or `extern import raylib.*;`│
│   2. When `is_extern == true`:                                             │
│      - Extracts first identifier (`raylib`).                               │
│      - Checks `context.dependencies_map`; if not already loaded, queries   │
│        `solver.resolve_latest("raylib")`.                                  │
│      - Validates transitive dependencies of `raylib` against               │
│        `context.dependencies_map` (checking SemVer compatibility!).        │
│      - Synthesizes API source via `manifest_to_source`, lexes & parses it  │
│        immediately, and appends `raylib@<ver>` to `dependencies_map`!      │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: BinderPhase (Root Facade Support)                                 │
│   Supports both `raylib.raylib.initWindow` AND clean `raylib.initWindow`   │
│   or direct `initWindow` when imported!                                    │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 4 & 5: AstInlinerPhase & EmitterPhase                                │
│   Writes `context.dependencies_map` into `program.export_manifest.deps`!   │
└────────────────────────────────────────────────────────────────────────────┘
```

#### New Diagnostic Error Codes in `BindingError` / `CompilationDiagnostic`:
* **`E0289` (`IncompatibleDependencyVersion`):** Dependency conflict in tree (e.g., package A requires `raylib@1.2.0` and package B requires `raylib@2.0.0`, or only `raylib@1.0.0` is installed when `1.2.0` is required).
* **`E0290` (`ExternalPackageNotFound`):** `ModuleSolver` could not find requested package `<name>` in the installed packages store.

---

### 3.6 Subsystem 6: Runtime Dependency Tree Verification & Auto-Loading (`solix-runtime`)

When running on PC B (`solix run app.slxb` or `solix run app.slxarc` or multiple `.slxb`/`.slxarc` files):
1. `RuntimeOptions` includes `pub solver: Option<Arc<dyn ModuleSolver>>` (and `solix run` attaches `ClientModuleSolver` by default).
2. At the start of `RuntimeContext::new(options)`:
   * Collects all root binaries loaded in `options.binaries` (and `options.binary_paths`).
   * Builds a unified runtime `resolved_dependencies: HashMap<String, PackageVersion>` across all loaded root binaries and their `export_manifest.dependencies`.
   * Uses `options.solver` to recursively fetch all required `.slxb` and `.slxarc` dependencies (and their transitive dependencies) that are not already among the root binaries.
   * **Strict Compatibility Verification:** Verifies that every loaded binary and every transitive dependency satisfies the exact SemVer law ($M_{\text{cand}} == M_{\text{req}}$, $D_{\text{cand}} \ge D_{\text{req}}$, and $m_{\text{cand}} \ge m_{\text{req}}$ when $D_{\text{cand}} == D_{\text{req}}$).
   * If any dependency is missing on PC B or has an incompatible version (e.g. major mismatch or installed version is older than what PC A compiled against), `RuntimeContext::new` immediately halts before executing a single instruction and returns `Err(RuntimeError::DependencyError { reason })` (mapped to `RuntimeErrorCode::DependencyError = 23`)!
   * For all resolved `.slxarc` dependencies, automatically registers their extracted native shared libraries (`.dll`, `.so`, `.dylib`) into `options.native_libraries` and mounts their `.slxb` programs into `options.binaries`!

---

## 4. Phased Implementation Plan (Step-by-Step Execution Blueprint)

### Phase 1: SemVer Engine, `ModuleSolver` Trait & `.slxb` Dependency Metadata (`solix-common`)
* **Files to Create / Modify:**
  1. `crates/solix-common/src/solver.rs` (New):
     * Implement `PackageVersion` (`parse`, `Display`, `is_compatible_with`, `Ord`).
     * Implement `DependencyRequirement`, `SolvedPackage`, `SolverError`, `ModuleSolver` trait, and `InMemoryModuleSolver`.
  2. `crates/solix-common/src/bytecode/manifest.rs` & `binary.rs`:
     * Add `#[serde(default)] pub dependencies: Vec<DependencyRequirement>` to `ExportManifest`.
  3. `crates/solix-common/src/project.rs`:
     * Add `#[serde(default)] pub dependencies: BTreeMap<String, String>` to `ProjectManifest`.
  4. `crates/solix-common/src/diagnostics/binding.rs`:
     * Add `E0289` (`IncompatibleDependencyVersion`) and `E0290` (`ExternalPackageNotFound`).

### Phase 2: Package Store & CLI Commands `install`, `uninstall`, `list` (`solix-client`)
* **Files to Create / Modify:**
  1. `crates/solix-client/src/solver.rs` (New):
     * Implement cross-platform package store directory resolution (`resolve_package_store_dir()`, honoring `SOLIX_PKG_DIR` and `SOLIX_HOME`).
     * Implement `ClientModuleSolver` implementing `ModuleSolver` for both `.slxb` and `.slxarc` packages.
  2. `crates/solix-client/src/cli.rs`:
     * Add `Install(InstallArgs)`, `Uninstall(UninstallArgs)`, and `List(ListArgs)` to `Commands`.
     * Replace `-l, --lib <PathBuf>` on `CompileArgs` with `-d, --deps <String>` (`Vec<String>` of `<name>@<version>`).
  3. `crates/solix-client/src/commands/pkg.rs` (New):
     * Implement `execute_install`, `execute_uninstall`, and `execute_list`.

### Phase 3: Compiler `DependencyResolutionPhase`, `extern import`, and Root Facade Binding (`solix-compilation`)
* **Files to Create / Modify:**
  1. `crates/solix-compilation/src/CompilationContext.rs`:
     * Add `dependency_binaries: Vec<BinaryProgram>`, `dependency_specs: Vec<DependencyRequirement>`, and `solver: Option<Arc<dyn ModuleSolver>>` to `CompilationOptions`.
     * Add `dependencies_map: HashMap<String, DependencyRequirement>` and `loaded_api_packages: HashSet<String>` to `CompilationContext`.
  2. `crates/solix-compilation/src/manifest.rs`:
     * Emit `package <pkg_name>;` at the top of `manifest_to_source` when `package_name` is non-empty.
  3. `crates/solix-compilation/src/phases/dependency.rs` (New):
     * Implement `DependencyResolutionPhase` running at the start of `CompilationPipeline`:
       * Extracts metadata and dependencies from `options.dependency_binaries` and `options.dependency_specs`.
       * Traverses the transitive dependency graph via `solver`, verifying SemVer compatibility and deduplicating to the highest compatible version.
       * Synthesizes API sources via `manifest_to_source` and registers them in `context` before `LexerPhase` runs.
  4. `crates/solix-compilation/src/phases/parser/statements/modules.rs` & `parser.rs`:
     * Add `pub is_extern: bool` to `ImportStatement`.
     * Allow `extern import <path>;` in `parse_top_level_element` and `parse_import_statement` (while still rejecting other modifiers like `public import` with `E0185`).
     * In `ParserPhase::run`: when an AST contains an `ImportStatement` with `is_extern == true`, extract the first identifier (`pkg_name`), query `solver.resolve_latest(pkg_name)` (if not already in `dependencies_map`), verify transitive compatibility with `dependencies_map`, synthesize the API source, lex and parse it immediately into `context.ast`, and record the version in `dependencies_map`!
  5. `crates/solix-compilation/src/phases/binder/passes/pass1_symbol_discovery.rs` & `symbols.rs`:
     * Support clean root-facade access so `import raylib.*;` or `extern import raylib.initWindow;` allows both `initWindow(...)` and `raylib.initWindow(...)` / `raylib::initWindow(...)` without requiring `raylib.raylib.initWindow(...)`.
  6. `crates/solix-compilation/src/phases/emitter/pipeline.rs`:
     * Populate `program.export_manifest.dependencies` from `context.dependencies_map`.

### Phase 4: Runtime Dependency Verification & Auto-Linking (`solix-runtime`)
* **Files to Modify:**
  1. `crates/solix-runtime/src/exceptions/errors.rs`:
     * Add `RuntimeError::DependencyError { reason: String }` and `RuntimeErrorCode::DependencyError = 23`.
  2. `crates/solix-runtime/src/options.rs`:
     * Add `pub solver: Option<Arc<dyn ModuleSolver>>` and `.with_solver(...)` to `RuntimeOptions`.
  3. `crates/solix-runtime/src/context.rs`:
     * At the start of `RuntimeContext::new`, inspect all loaded binaries and their `export_manifest.dependencies`.
     * Recursively resolve missing dependencies via `options.solver`, validate SemVer compatibility across all root and transitive binaries, mount secondary `.slxb` programs, and load extracted `.slxarc` native libraries!

### Phase 5: Project Toolbox (`solix_toolbox.py`) & CLI Wiring (`solix-client`)
* **Files to Modify:**
  1. `crates/solix-client/src/commands/compile.rs` & `run.rs`:
     * Wire `ClientModuleSolver` into `CompilationOptions` and `RuntimeOptions`.
     * Parse `--deps <name>@<version>` in `execute_compile`.
  2. `templates/untitled-rust/scripts/solix_toolbox.py`:
     * Read `[dependencies]` from `manifest.toml` and pass `--deps <name>@<version>` to `solix compile`.

---

## 5. Exhaustive Test Plan & Verification Matrix

Every single feature and edge case in your plan will be tested across `solix-common`, `solix-compilation`, `solix-runtime`, `solix-client`, and `solix-tests`:

1. **SemVer Compatibility & Tree Resolution Unit Tests (`solix-common`):**
   * `1.0.0` vs `1.0.0` (exact match $\to$ compatible).
   * `1.2.0` required vs `1.2.5` installed (same major, same medium, higher minor $\to$ compatible, picks `1.2.5`).
   * `1.2.3` required vs `1.2.1` installed (same medium, lower minor $\to$ **rejected!**).
   * `1.2.0` required vs `1.4.0` installed (higher medium, lower minor `0 < 2` $\to$ **compatible**, picks `1.4.0`).
   * `1.2.0` required vs `1.1.9` installed (lower medium $\to$ **rejected!**).
   * `1.0.0` required vs `2.0.0` installed (different major $\to$ **rejected!**).
2. **CLI Package Manager Tests (`solix-client/tests/package_manager_tests.rs`):**
   * `solix install math.slxb` and `solix install raylib.slxarc` into a hermetic `SOLIX_PKG_DIR`.
   * `solix list` and `solix list --json` verifying multiple versions (`1.0.0`, `1.2.0`) are listed accurately.
   * `solix uninstall raylib@1.0.0` removes only `1.0.0` while keeping `1.2.0`.
   * `solix uninstall raylib` removes all versions.
3. **`DependencyResolutionPhase` & `--deps` Compilation Tests (`solix-tests`):**
   * Pre-lexing resolution from `options.dependency_binaries` and `--deps raylib@1.0.0`.
   * Transitive dependency chain: `App` depends on `Engine@1.0.0`, which depends on `Math@1.1.0`. Both `Math@1.1.0` and `Math@1.3.0` are installed $\to$ compiler automatically resolves `Math@1.3.0` and embeds `[Engine@1.0.0, Math@1.3.0]` in `App.slxb`'s manifest!
   * Diamond dependency conflict: `App` depends on `LibA@1.0.0` (needs `Core@1.2.0`) and `LibB@1.0.0` (needs `Core@2.0.0`) $\to$ fails in `DependencyResolutionPhase` before lexing with **`E0289` (`IncompatibleDependencyVersion`)**!
   * Diamond compatible upgrade: `LibA` needs `Core@1.1.0` and `LibB` needs `Core@1.3.0` $\to$ deduplicates cleanly to `Core@1.3.0` with zero `DuplicateSymbol` errors!
4. **`extern import` Parser & Root Facade Tests (`solix-tests`):**
   * Standalone file with `extern import raylib.initWindow;` and `extern import raylib.*;` (no `--deps` flag passed): parser triggers solver, fetches highest version of `raylib` (`1.2.0`), lexes & parses its synthesized API on the fly, and supports calling both `raylib.initWindow(800, 600)` and `initWindow(800, 600)` without `raylib.raylib.initWindow`!
   * `extern import` of a package with transitive dependencies $\to$ fetches and parses all transitive APIs and checks compatibility.
5. **Cross-PC Runtime Dependency Validation & Auto-Loading Tests (`solix-tests` & `solix-client`):**
   * **PC A $\to$ PC B Simulation:**
     * Compile `app.slxb` in Environment A with `raylib@1.2.0.slxarc` installed (which includes a Rust `cdylib` native bridge).
     * Move **only** `app.slxb` to Environment B:
       * **Scenario 1 (Missing on PC B):** Running `solix run app.slxb` on PC B when `raylib` is not installed fails immediately with `RuntimeError::DependencyError` (`Missing required dependency 'raylib@1.2.0'`).
       * **Scenario 2 (Older incompatible version on PC B):** Install `raylib@1.1.0.slxarc` on PC B $\to$ `solix run app.slxb` fails with `RuntimeError::DependencyError` (`Installed version 1.1.0 is incompatible with required 1.2.0`).
       * **Scenario 3 (Major mismatch on PC B):** Install `raylib@2.0.0.slxarc` on PC B $\to$ `solix run app.slxb` fails with `RuntimeError::DependencyError`.
       * **Scenario 4 (Compatible newer version on PC B):** Install `raylib@1.4.2.slxarc` on PC B $\to$ `solix run app.slxb` automatically fetches `raylib@1.4.2.slxarc`, extracts the native `.so`/`.dll`/`.dylib`, links `raylib.slxb`, executes the native function, and exits with `0`!

---

## 6. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **`install` / `uninstall` / `list`** | Cross-platform package store for `.slxb` & `.slxarc` | `cargo test -p solix-client` | [ ] Pending |
| **`ModuleSolver` & `ClientModuleSolver`** | Trait in `solix-common` + disk/memory implementations | `cargo test -p solix-common` | [ ] Pending |
| **`DependencyResolutionPhase`** | Pre-lexing dependency tree validation & API extraction | `cargo test -p solix-compilation` | [ ] Pending |
| **`extern import` & Root Facade** | On-the-fly solver fetch in parser + `pkg.fn()` without `pkg.pkg.fn()` | `cargo test -p solix-tests` | [ ] Pending |
| **`--deps <name>@<ver>` & `manifest.toml`** | Replaces `-l` on `compile`; `solix_toolbox.py` passes `--deps` | `cargo test -p solix-client` | [ ] Pending |
| **Runtime Dependency Verification** | Auto-loads `.slxb`/`.slxarc` + native libs; rejects missing/incompatible | `cargo test -p solix-runtime` | [ ] Pending |
| **Clippy & Formatting** | Zero warnings across all workspace crates | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |