# Formal Specification: Package Solver, SemVer & Dual-Mode Linking

## Document Metadata

* **Specification ID:** SPEC-0010
* **Scope:** Package Manager, Dependency Resolution & SemVer Laws (`crates/solix-common/src/solver.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-common/src/solver.rs`](file:///home/jehud/Projects/solix-language/crates/solix-common/src/solver.rs)

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

The [`ModuleSolver`](file:///home/jehud/Projects/solix-language/crates/solix-common/src/solver.rs) trait abstracts package discovery and version solving:

```rust
pub trait ModuleSolver: Send + Sync + std::fmt::Debug {
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError>;

    fn resolve_latest(
        &self,
        package_name: &str,
    ) -> Result<SolvedPackage, SolverError>;
}
```

### 2.1 Concrete Solvers
* **`ClientModuleSolver` (`solix-client` / `solix-toolchain`):** Scans the local filesystem package store (`~/.solix/packages/`), reads version directories, extracts archives, and loads `.slxb` containers.
* **`InMemoryModuleSolver` (`solix-common`):** In-memory dictionary solver used for fast, hermetic compiler unit tests without filesystem I/O.

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
