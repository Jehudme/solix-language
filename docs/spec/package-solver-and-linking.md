# Formal Specification: Package Solver, SemVer & Dual-Mode Linking

## Document Metadata

* **Specification ID:** SPEC-0010
* **Scope:** Package Manager, Dependency Resolution & SemVer Laws (`crates/solix-common/src/solver.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-common/src/solver.rs`](../../crates/solix-common/src/solver.rs)

---

## 1. Semantic Versioning Specification (`PackageVersion`)

Package versions follow the canonical three-component format `Major.Medium.Minor` (`1.2.3`):

* **Major:** Incremented on backward-incompatible ABI or API changes. Different major versions are mutually incompatible.
* **Medium:** Incremented on backward-compatible feature additions.
* **Minor:** Incremented on backward-compatible bug fixes and security patches.

### SemVer Compatibility Law
A candidate version $V_{c}$ is compatible with a required minimum version $V_{r}$ if and only if:
1. $V_{c}.\text{major} == V_{r}.\text{major}$
2. $V_{c}.\text{medium} > V_{r}.\text{medium}$ OR ($V_{c}.\text{medium} == V_{r}.\text{medium}$ AND $V_{c}.\text{minor} \ge V_{r}.\text{minor}$)

---

## 2. Pluggable `ModuleSolver` Architecture

The [`ModuleSolver`](../../crates/solix-common/src/solver.rs) trait abstracts package discovery, version solving, and available package cataloging:

```rust
pub trait ModuleSolver: Send + Sync + std::fmt::Debug {
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError>;

    fn resolve_exact(
        &self,
        package_name: &str,
        version: PackageVersion,
        expected_sha256: Option<&str>,
    ) -> Result<SolvedPackage, SolverError>;

    fn resolve_latest(
        &self,
        package_name: &str,
    ) -> Result<SolvedPackage, SolverError>;

    fn list_packages(&self) -> Vec<String>;
}
```

### 2.1 Concrete Solvers
* **`ClientModuleSolver` (`solix-toolchain`):** Scans the local filesystem package store (`~/.solix/packages/`), reads version directories, extracts archives, loads `.slxb` containers, and enumerates available package names via `list_packages()`.
* **`InMemoryModuleSolver` (`solix-common`):** In-memory dictionary solver used for fast, hermetic compiler and language server unit tests without filesystem I/O.

---

## 3. Package Store Topology

The local package store organizes artifacts using strict versioned directory paths:

```text
~/.solix/packages/
├── solix/
│   ├── 1.0.0/
│   │   └── solix.slxarc
│   └── 2.0.0/
│       └── solix.slxarc
└── raylib/
    └── 5.0.0/
        └── raylib.slxarc
```

Resolution Priority:
1. Explicit CLI override (`--pkg-dir <path>`)
2. `SOLIX_PKG_DIR` environment variable
3. `SOLIX_HOME/packages` environment variable
4. Platform standard: `%USERPROFILE%\.solix\packages` (Windows) or `$HOME/.solix/packages` (Unix/macOS).

---

## 4. Solver-Backed Autocompletion Intelligence

Language tooling (`solix-lsp`) leverages `ModuleSolver::list_packages()` and cached package indexes to provide hierarchical `import` and `extern import` completion:

1. **Segment 0 (Root Package):**
   When the user types `import ` or `extern import `, the completion engine queries:
   * Available installed packages via `solver.list_packages()` (e.g., `solix`, `raylib`).
   * Compiler built-in package `blt`.
   * Workspace-local project package names.

2. **Segment 1..N (Subpackages & Exported Symbols):**
   When typing subsequent dot-separated segments (e.g., `import solix.` or `import solix.io.`):
   * The LSP server fetches the solved package's `ExportedAst` or `ExportManifest` via `solver.resolve_latest(package_name)`.
   * Immediate child namespaces are suggested as `CompletionItemKind::MODULE` (e.g., `io`, `math`, `core`).
   * Exported public symbols (classes, interfaces, enums, free functions) are suggested with terminal semicolon deduplication (e.g., `Console;`, `File;`).
   * Wildcard import `*;` is provided for batch symbol import.

---

## 5. Dual-Mode Linking (`SOLIX_LINK_MODE`)

Solix supports two complementary binary linking strategies:

1. **Dynamic Linking (`--link dynamic`, default):**
   * Dependencies are resolved at runtime via the local package store (`~/.solix/packages`).
   * Compiled `.slxb` and `.slxarc` archives only bundle project-local bytecode and native extensions.

2. **Static Linking (`--link static`):**
   * Transitive bytecode dependencies are merged into a single consolidated binary payload.
   * Controlled via the `SOLIX_LINK_MODE=static` environment variable or the `--link static` CLI compilation flag.
   * `solix_build.py` checks `os.environ.get("SOLIX_LINK_MODE")` during project build scripts and forwards the flag to `solix compile`.
