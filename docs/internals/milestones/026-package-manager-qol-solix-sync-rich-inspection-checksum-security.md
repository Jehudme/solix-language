# Milestone 026: Package Manager Modernization, `solix sync`, Rich `info`/`list` Inspection, Authenticated Extraction Cache, Binary AST Serialization & Cryptographic Checksum Pinning

## Document Metadata

* **Milestone ID:** MS-026
* **Target Version:** `v0.1.0-alpha.26`
* **Milestone Branch:** `milestone/026-package-manager-sync-and-checksum-security`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/020-package-manager-solver-and-dependency-resolution.md` (MS-020)
  * `docs/internals/milestones/023-compiler-stabilization-qualified-declarations-and-dual-linking.md` (MS-023)
  * `docs/internals/milestones/025-workspace-hygiene-crate-reorganization-and-build-cache.md` (MS-025)

---

## 1. Executive Summary & Architectural Objectives

While Milestone 020 introduced basic package installation (`solix install <file.slxarc>`), uninstallation, listing, and SemVer resolution, real-world library development, high-frequency developer iteration loops, and secure package distribution require stronger safety guarantees, sub-10 ms compilation latency, rich metadata inspection, and end-to-end cryptographic integrity verification:

1. **Accidental Overwrite Hazard on `solix install`:**
   * Currently, running `solix install` silently overwrites an already-installed package version. It **MUST** refuse to overwrite an existing installation unless `--force` (`-f`) is explicitly supplied.
2. **Manual Two-Step Build-Then-Install Friction:**
   * Developers currently have to run `solix build <folder> --release` and then manually type `solix install <folder>/build/release/<name>.slxarc`.
   * Developers should be able to run **`solix install <folder> [--profile <debug|release|test>] [--force]`** directly, or pass **`--install` / `--sync`** to `solix build` and `solix compile`, or run **`solix sync [name | --all]`** to automatically rebuild installed packages from their recorded source directory and profile!
3. **Limited `solix info` and `solix list` Introspection:**
   * `solix info` currently only accepts a filesystem path to a `.slxb` or `.slxarc` file—it cannot inspect an installed module by name (`solix info solix` or `solix info solix@2.0.0`).
   * Neither `solix info` nor `solix list` displays the **build profile** (`debug`/`release`/`test`), **installation timestamp**, **original source directory path**, **binary size**, or **SHA-256 checksum**, and `solix list` lacks filtering (`--filter`, `--profile`, `--kind`) and sorting (`--sort name|version|date|size|profile`, `--reverse`).
4. **The SemVer Solver Security & ABI Integrity Vulnerability:**
   * Relying purely on loose SemVer compatibility (`1.x.y`) without cryptographic verification is a security and stability hazard: if a developer compiles `app.slxb` against `lib@1.0.0`, and at runtime a mutated, corrupted, or tampered `lib@1.0.0` (or `lib@1.2.0` with altered vtable layouts or native hooks) is loaded, the VM can execute untrusted code or corrupt memory.
   * By recording the exact **`(name, version, sha256)`** of every resolved dependency inside the compiled `.slxb` binary and enforcing **exact version + exact SHA-256 checksum verification** by default (with an explicit **`--yolo` / `--ignore-checksum`** flag to bypass verification when intentionally testing hot-swapped libraries), Solix guarantees 100% reproducible, tamper-proof execution.
5. **Cold-Start Compilation Overhead on Package Imports (~112 ms Bottleneck):**
   * Detailed profiling of `solix compile` importing `solix.io.Console` revealed three critical performance bottlenecks accounting for > 95% of total compilation latency:
     * **Archive Re-Decompression (~22 ms):** On every cold compile, [`SlxarcArchive::extract_and_load`](file:///home/jehud/Projects/solix-language/crates/solix-common/src/project.rs#L315) opens `solix.slxarc` (6.16 MB `.slxb`) and inflates all entries from scratch.
     * **Manifest & AST JSON Bloat (~52 ms):** Serializing `exported_ast_bytes: Vec<u8>` inside the JSON `ExportManifest` turns raw bytes into a 4.5 MB ASCII JSON array `[123, 34, ...]`, expanding `ExportManifest` to 5.38 MB and forcing two multi-megabyte JSON deserialization passes.
     * **Monolithic Symbol Registration (~18 ms):** Ingesting the entire `solix` standard library binds all 97 declarations (crypto, collections, math, time) even when only `Console` is imported.
   * Milestone 026 eliminates this entire latency cliff via:
     * **Authenticated Extract Cache (-22 ms):** Caching extracted `.slxb` artifacts directly in `~/.solix/cache/packages/` authenticated by the milestone's SHA-256 checksum system.
     * **Binary AST Container Encoding (-45 ms):** Moving AST serialization to a dedicated binary section in `BinaryProgram` (or `bincode`/length-prefixed raw binary bytes), eliminating the 4.5 MB JSON string expansion and reducing `.slxb` size from 6.16 MB to ~1.6 MB.
     * **Selective AST Ingestion & Symbol Tree-Shaking (-15 ms):** Parsing and binding only the imported module slice (`solix.io.Console` and its transitive types) rather than 97 classes.
     * **Native Shared Library Checksum Authentication:** Storing and verifying SHA-256 digests for native dynamic libraries bundled in `.slxarc` archives.

---

## 2. Detailed Architectural Specifications

### 2.1 Binary Build Profile, SHA-256 Metadata & Native Library Checksums

In `crates/solix-common/src/bytecode/manifest.rs` and `binary.rs`:

1. **Build Profile Recorded in `.slxb`:**
   * Add `#[serde(default)] pub build_profile: String` (`"debug"`, `"release"`, or `"test"`) to `ExportManifest` (and `BinaryProgram`).
2. **Deterministic SHA-256 Checksums (`compute_sha256_hex`):**
   * Implement a self-contained, high-throughput FIPS 180-4 SHA-256 utility in `crates/solix-common/src/hash.rs` (`sha256_bytes`, `sha256_hex`, `sha256_file`).
   * Every `.slxb` binary computes its canonical 64-character hex SHA-256 digest over its serialized container bytes.
3. **Native Shared Library Checksum Authentication in `.slxarc` (`native_library_checksums`):**
   * Add `native_library_checksums: HashMap<String, String>` to `ExportManifest`, `BinaryProgram`, and `ProjectManifest` (`manifest.toml`), mapping bundled native library paths (e.g. `"x86_64-linux/libsolixlib_native.so"` or `"host/native.so"`) to their exact SHA-256 hex digest.
   * When building `.slxarc` packages, the toolchain computes and records the digest of every native binary packaged into `lib/`.
   * At runtime (`SlxarcArchive::extract_and_load` and `RuntimeContext::new`), the VM computes the SHA-256 of extracted native libraries and verifies them against `native_library_checksums` before passing them to the OS dynamic linker (`dlopen`/`LoadLibrary`).
4. **Pinned Dependency Requirements (`DependencyRequirement`):**
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub struct DependencyRequirement {
       pub name: String,
       pub version: PackageVersion,
       /// Exact SHA-256 hex digest of the dependency `.slxb` binary at compile time.
       #[serde(default)]
       pub sha256: String,
   }
   ```

---

### 2.2 Cross-Platform Package Registry Manifest (`$SOLIX_PKG_DIR/registry.json`)

Alongside the package files in `$SOLIX_PKG_DIR/<name>/<version>/<name>.<ext>`, the package manager maintains a cross-platform JSON registry manifest at `$SOLIX_PKG_DIR/registry.json`:

```json
{
  "schema_version": 1,
  "packages": [
    {
      "name": "solix",
      "version": "2.0.0",
      "kind": "slxarc",
      "build_profile": "release",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "size_bytes": 1648210,
      "installed_at_unix_secs": 1762809000,
      "installed_at_iso8601": "2026-11-10T21:10:00Z",
      "source_project_dir": "/home/user/Projects/solix/libraries/solixlib",
      "artifact_path": "/home/user/.solix/packages/solix/2.0.0/solix.slxarc",
      "native_libraries": {
        "x86_64-linux/libsolixlib_native.so": "a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0"
      },
      "description": "The Official Solix v2.0 Standard Library",
      "license": "Apache-2.0",
      "authors": ["Solix Team"]
    }
  ]
}
```

* **Automatic Backfill / Self-Healing:** If a user manually places a `.slxb` or `.slxarc` inside `$SOLIX_PKG_DIR/<name>/<version>/` or upgrades from MS-020, `PackageRegistry::load_or_sync()` automatically scans the directory, extracts metadata/profile/SHA-256 from any unlisted `.slxb`/`.slxarc` files, and synchronizes `registry.json`.

---

### 2.3 Modular Authenticated Extract Cache (`PackageCache` & `~/.solix/cache/packages/`)

To eliminate the **~22 ms archive decompression overhead** on every cold compilation:

1. **Pluggable `PackageCache` Trait (`crates/solix-common/src/cache.rs`):**
   ```rust
   pub trait PackageCache: Send + Sync + std::fmt::Debug {
       /// Attempts to load an extracted binary program from cache, validating that
       /// the cached artifact's fingerprint matches the source archive.
       fn get_cached_binary(
           &self,
           package_name: &str,
           version: PackageVersion,
           source_archive_path: &std::path::Path,
       ) -> Result<Option<BinaryProgram>, solix_common::diagnostics::EmissionError>;

       /// Atomically stores an extracted binary program into the cache associated with
       /// the source archive's cryptographic SHA-256 fingerprint.
       fn put_cached_binary(
           &self,
           package_name: &str,
           version: PackageVersion,
           source_archive_path: &std::path::Path,
           program: &BinaryProgram,
           raw_binary_bytes: &[u8],
       ) -> Result<(), solix_common::diagnostics::EmissionError>;
   }
   ```
2. **Cryptographic Authenticity Verification:**
   * Alongside the cached binary at `~/.solix/cache/packages/<name>-<version>/bin/<name>.slxb`, the cache writes a cryptographic sidecar manifest `meta.json`:
     ```json
     {
       "package_name": "solix",
       "package_version": "2.0.0",
       "source_archive_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
       "source_archive_size_bytes": 812617,
       "cached_binary_sha256": "9f83c61b17b2f3498a4d52e5a7b6c8d9e0f1234567890abcdef1234567890abc"
     }
     ```
   * **Stale / Tamper Detection:** Before serving a cached `.slxb`, `FilesystemPackageCache` checks the source archive size and SHA-256. If the source `.slxarc` has been updated, rebuilt, or altered, the cache entry is immediately purged and safely re-extracted.
3. **Atomic File Staging:**
   * All cache writes are written to a temporary sibling file (`.bin.tmp.<pid>`) and moved using `std::fs::rename` to prevent race conditions or corrupted partial reads.
4. **Hermetic Test Isolation:**
   * Unit tests use `NoopPackageCache` or `InMemoryPackageCache`, maintaining hermetic, filesystem-independent test runs.

---

### 2.4 Dedicated Binary AST Container Section (Eliminating JSON Bloat)

To eliminate the **~52 ms JSON parsing overhead**:

1. **Current Failure Mode:**
   * In Milestone 023, `exported_ast_bytes: Vec<u8>` was stored as a field on `ExportManifest`. Because `ExportManifest` is serialized via `serde_json`, `serde` serializes `Vec<u8>` as a raw JSON array of integers (`[123, 34, 115, ...]`).
   * A 1.42 MB raw AST expanded to **4.5 MB of ASCII numbers and commas**, ballooning `ExportManifest` to 5.38 MB and total `.slxb` size to 6.16 MB.
   * `BinaryProgram::from_bytes` spent ~32 ms running `serde_json::from_slice` on 5.38 MB of JSON, followed by `ingest_package_api_source` spending another ~20 ms parsing the 1.42 MB AST JSON.
2. **Binary Section Migration in `.slxb` Container:**
   * Add a dedicated, length-prefixed binary container section to the `SLXB` binary specification:
     * **Section 7: `ExportedAst`:**
       * `ast_len: u32` (4 bytes, little-endian).
       * `ast_bytes: [u8; ast_len]` (compact binary encoded AST, e.g. via `bincode` or compact length-prefixed binary AST).
   * `ExportManifest` strips `exported_ast_bytes` from its JSON payload, shrinking the manifest JSON from 5.38 MB down to < 40 KB.
   * Container deserialization in `BinaryProgram::from_bytes` reads the raw binary slice in **< 0.1 ms**, and `bincode::deserialize::<ProgramAst>` deserializes the AST into memory in **< 1 ms** (a **~45 ms reduction**).

---

### 2.5 Selective Dependency AST Ingestion & Symbol Tree-Shaking

To eliminate the **~15 ms semantic binding overhead**:

1. **Current Bottleneck:**
   * Importing `solix.io.Console` currently ingests the entire `solix@2.0.0` AST containing all 97 declarations (AES/SHA crypto, Red-Black Trees, Queues, HashMaps, Math, Time, System).
   * [`BinderPhase`](file:///home/jehud/Projects/solix-language/crates/solix-compilation/src/phases/standard.rs#L170) binds all 97 classes, computes field layouts, vtables, and itables for 95 unused classes.
2. **Selective AST Ingestion Strategy:**
   * In [`DependencyResolutionPhase`](file:///home/jehud/Projects/solix-language/crates/solix-compilation/src/phases/dependency.rs), when resolving an `extern import`:
     * Inspect the imported symbol paths (e.g. `solix.io.Console`).
     * Extract only the target class declaration AST node (`Console`) and its transitive reachability closure (`solix.core.String`, `IStringable`, primitive wrappers).
     * Omit unreferenced collection templates and cryptographic modules from `context.ast`.
   * Semantic binding duration drops from **~18 ms down to ~2 ms** (-15 ms).

---

### 2.6 Native Shared Library Checksum Authentication in `.slxarc`

1. **Compile & Package Time (`solix build` / `solix pack`):**
   * For every native shared library bundled into `lib/<triple>/<name>.<ext>`, the packager computes its SHA-256 digest and records it into `ExportManifest.native_library_checksums`.
2. **Runtime Verification (`SlxarcArchive::extract_and_load` & `RuntimeContext::new`):**
   * Before extracting or loading any native library (`.so`, `.dll`, `.dylib`) into `RuntimeOptions.native_libraries`, the runtime computes the SHA-256 digest of the file.
   * If the digest does not match `native_library_checksums`, the runtime immediately aborts execution with **`RuntimeError::NativeLibraryChecksumMismatch`**:
     ```text
     error: native library checksum verification failed for 'lib/x86_64-linux/libsolixlib_native.so'
       expected: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
       found:    8c9f7a... (library file may have been corrupted or tampered with)
     ```
   * Can be bypassed only when intentionally testing hot-swapped native binaries via `--yolo` / `--ignore-checksum`.

---

### 2.7 Enhanced CLI Commands & Developer Workflows (`solix-toolchain`)

```
┌────────────────────────────────────────────────────────────────────────────┐
│                Milestone 026 Package & Build CLI Matrix                    │
├────────────────────────────────────────────────────────────────────────────┤
│ 1. `solix install <target> [--profile|--debug|--release|--test] [--force]` │
│    • If `<target>` is a `.slxb` or `.slxarc` file:                         │
│      - Checks if `<name>@<version>` is already installed.                  │
│      - If installed and `--force` (`-f`) is NOT passed -> ERROR!           │
│      - Computes SHA-256, validates native library checksums, warms cache.  │
│      - Records `build_profile`, `sha256`, `installed_at`, `size_bytes`.    │
│    • If `<target>` is a project directory (contains `manifest.toml`):      │
│      - Automatically builds `<target>` with requested profile.             │
│      - Installs resulting `.slxarc` and records `source_project_dir`       │
│        in `registry.json`!                                                 │
├────────────────────────────────────────────────────────────────────────────┤
│ 2. `solix sync [<package_name> | --all] [--profile <override>]`            │
│    • Looks up `<package_name>` (or all packages with `source_project_dir`) │
│      in `registry.json`.                                                   │
│    • Rebuilds from source with recorded profile, re-computes checksums,    │
│      refreshes extract cache, and updates `registry.json`!                 │
├────────────────────────────────────────────────────────────────────────────┤
│ 3. `solix build [path] ... --install` (alias `--sync`)                     │
│    `solix compile ... --install` (alias `--sync`)                          │
│    • Automatically installs/updates the resulting `.slxarc` or `.slxb` in  │
│      the store and refreshes the authenticated extract cache!              │
├────────────────────────────────────────────────────────────────────────────┤
│ 4. `solix info <file_path | pkg_name | pkg_name@version> [flags]`          │
│    • Resolves `<pkg_name>` or `<pkg_name>@<version>` directly from store.  │
│    • Displays: Name, Version, Kind, Build Profile, SHA-256 Checksums       │
│      (bytecode + native libraries), Cached Status, Source Project Path.   │
├────────────────────────────────────────────────────────────────────────────┤
│ 5. `solix list [flags]`                                                    │
│    • Columns: NAME, VERSION, KIND, PROFILE, SIZE, INSTALLED AT, SHA256,    │
│      SOURCE DIR.                                                           │
│    • Filtering: `--filter <substr>`, `--profile <prof>`, `--kind <kind>`.  │
│    • Ordering: `--sort <name|version|date|size|profile>`, `--reverse`.     │
│    • Machine output: `--json`.                                             │
└────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.8 Cryptographic Checksum Pinning & `--yolo` (`--ignore-checksum`) Mode

#### 1. Compile-Time Recording & Enforcement
* When compiling against `solix@2.0.0`:
  * `DependencyResolutionPhase` records `DependencyRequirement { name: "solix", version: 2.0.0, sha256: "<64-char-hex>" }` inside the output binary.
  * If a dependency requirement's recorded SHA-256 does not match the resolved package artifact, compilation halts with diagnostic **`E0291` (`DependencyChecksumMismatch`)** unless `--yolo` is supplied.

#### 2. Runtime Verification (`RuntimeContext::new`)
* By default (`RuntimeOptions.verify_checksums = true`):
  * Verifies both `.slxb` bytecode checksums and native shared library checksums against their pinned digests.
  * If any checksum mismatches, execution is aborted with `RuntimeError::DependencyError` or `RuntimeError::NativeLibraryChecksumMismatch`.
* **YOLO Mode (`solix run --yolo` or `solix run --ignore-checksum`):**
  * Bypasses checksum enforcement while printing an explicit warning:
    `warning: checksum verification bypassed (--yolo); loading unverified dependency 'solix@2.0.0'`.

---

## 3. Phased Implementation Plan

### Phase 1: SHA-256 Hashing, Binary AST Container Section & Checksum Fields (`solix-common`)
* [x] `[P0]` **Task 1.1:** Implement self-contained SHA-256 calculation (`sha256_bytes`, `sha256_hex`, `sha256_file`) in `crates/solix-common/src/hash.rs` and `BinaryProgram::compute_sha256(&self) -> String`.
* [x] `[P0]` **Task 1.2:** Add `pub build_profile: String` to `ExportManifest` and `BinaryProgram`, and `pub sha256: String` to `DependencyRequirement`.
* [x] `[P0]` **Task 1.3:** Add `pub native_library_checksums: HashMap<String, String>` to `ExportManifest`, `BinaryProgram`, and `ProjectManifest`.
* [x] `[P0]` **Task 1.4:** Migrate `exported_ast_bytes` out of the JSON `ExportManifest` into Section 7 (`ExportedAst`) of `BinaryProgram`, replacing JSON array encoding with compact binary serialization (`bincode`).

### Phase 2: Authenticated `PackageCache` & Extracted Artifact Store (`solix-common` & `solix-toolchain`)
* [x] `[P0]` **Task 2.1:** Define `PackageCache` trait in `crates/solix-common/src/cache.rs` with `NoopPackageCache` and `InMemoryPackageCache`.
* [x] `[P0]` **Task 2.2:** Implement `FilesystemPackageCache` in `crates/solix-toolchain/src/cache.rs` managing `~/.solix/cache/packages/<name>-<version>/bin/<name>.slxb` with atomic temp-file staging and cryptographic `meta.json` validation.
* [x] `[P0]` **Task 2.3:** Integrate `PackageCache` into `ClientModuleSolver::load_package_from_dir`, bypassing `.slxarc` decompression when the cached `.slxb` matches the source archive's SHA-256.

### Phase 3: Selective AST Ingestion & Symbol Tree-Shaking (`solix-compilation`)
* [x] `[P0]` **Task 3.1:** In `DependencyResolutionPhase::ingest_package_api_source`, inspect requested `extern import` symbol paths and selectively prune non-transitive class/function declarations from the ingested AST.
* [x] `[P0]` **Task 3.2:** Benchmark `solix compile` on `extern import solix.io.Console;` to verify semantic binding latency drops from ~18 ms to < 3 ms.

### Phase 4: Cross-Platform `PackageRegistry` (`registry.json`) & Safe `solix install` (`solix-toolchain`)
* [x] `[P0]` **Task 4.1:** Implement `PackageRegistry` (`crates/solix-toolchain/src/registry.rs`) reading/writing `$SOLIX_PKG_DIR/registry.json` with `installed_at`, `build_profile`, `sha256`, `size_bytes`, `native_libraries`, and `source_project_dir: Option<PathBuf>`.
* [x] `[P0]` **Task 4.2:** Enforce the overwrite guard in `solix install`: if `<name>@<version>` is already installed and `--force` (`-f`) is not set, return an error instructing the user to pass `--force`.
* [x] `[P0]` **Task 4.3:** Support `solix install <project_dir> [--profile <debug|release|test>] [--debug] [--release] [--test] [--force]`: builds the project folder with the requested profile, computes all checksums, warms the extract cache, and stores `source_project_dir` in `registry.json`.

### Phase 5: `solix sync` Command & `--install` / `--sync` Flags on `build` and `compile` (`solix-toolchain`)
* [x] `[P1]` **Task 5.1:** Add `--install` (alias `--sync`) to `solix build` and `solix compile`.
* [x] `[P1]` **Task 5.2:** Implement `solix sync [<name> | --all] [--profile <override>]` in `crates/solix-toolchain/src/commands/pkg.rs`, rebuilding each target package from its `source_project_dir` using its recorded `build_profile`, refreshing cached binaries, and updating `registry.json`.

### Phase 6: Upgraded `solix info` (Installed Modules) & `solix list` (Filter/Sort/Columns) (`solix-toolchain`)
* [x] `[P1]` **Task 6.1:** Update `solix info <target>` (`crates/solix-toolchain/src/commands/info.rs`) so `<target>` can be a file path OR an installed package name (`solix` or `solix@2.0.0`), displaying profile, SHA-256, native library checksums, cached extraction status, install timestamp, source folder path, dependencies, `--api`, `--disasm`, and `--json`.
* [x] `[P1]` **Task 6.2:** Upgrade `solix list` with `PROFILE`, `INSTALLED AT`, `SIZE`, `SHA256`, and `SOURCE` columns, plus `--filter <query>`, `--profile <profile>`, `--kind <slxb|slxarc>`, `--sort <name|version|date|size|profile>`, `--reverse`, and `--json`.

### Phase 7: Exact Checksum Enforcement, Native Library Verification & `--yolo` Mode (`solix-compilation` & `solix-runtime`)
* [x] `[P0]` **Task 7.1:** Add `E0291` (`DependencyChecksumMismatch`) to `BindingError` and update `DependencyResolutionPhase` to verify pinned SHA-256 digests.
* [x] `[P0]` **Task 7.2:** Add `RuntimeError::NativeLibraryChecksumMismatch` and verify native shared library checksums in `SlxarcArchive::extract_and_load` and `RuntimeContext::new`.
* [x] `[P0]` **Task 7.3:** Add `--yolo` (alias `--ignore-checksum`) to `solix run` and `solix compile` (mapped to `CompilationOptions.verify_checksums` and `RuntimeOptions.verify_checksums`).

### Phase 8: Exhaustive Tests, Latency Benchmarks & Pre-Merge Documentation Sync (`GL-0013`)
* [x] `[P0]` **Task 8.1:** Add integration tests in `crates/solix-toolchain/tests/` covering `solix install` collision protection (`--force`), `solix install <folder>`, `solix sync`, `solix build --sync`, `solix info <installed_pkg>`, `solix list --filter --sort`, checksum tamper detection (bytecode + native libraries), extract cache invalidation, and `--yolo` bypass.
* [x] `[P0]` **Task 8.2:** Execute compilation latency benchmarks on `extern import solix.io.Console;` to confirm total cold compilation latency drops below **10 ms** (from 112 ms).
* [x] `[P1]` **Task 8.3:** Run `git diff main...HEAD` and update `docs/wiki/commands/`, `docs/wiki/errors/`, `docs/spec/package-solver-and-linking.md`, and `docs/guide/01-getting-started-and-toolchain.md` per `GL-0013`.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **`solix install` Overwrite Guard** | Refuses overwrite without `--force`; succeeds with `--force` | `cargo test -p solix-toolchain` | [x] Completed |
| **Direct Folder Install & Profile** | `solix install <dir> --release` builds & records `source_project_dir` | `cargo test -p solix-toolchain` | [x] Completed |
| **`solix sync` & `build --sync`** | Rebuilds installed package from `source_project_dir` with recorded profile | `cargo test -p solix-toolchain` | [x] Completed |
| **`solix info <installed_pkg>`** | Inspects installed modules by name/version (`--api`, `--disasm`, `--json`) | `cargo test -p solix-toolchain` | [x] Completed |
| **`solix list` Filter & Sort** | Shows profile, date, size, source; supports `--filter` & `--sort` | `cargo test -p solix-toolchain` | [x] Completed |
| **Authenticated Extract Cache** | Caches extracted `.slxb` in `~/.solix/cache/packages/`; re-verifies via SHA-256 | `cargo test -p solix-toolchain` | [x] Completed |
| **Binary AST Encoding Section** | `.slxb` container shrinks from > 6 MB to < 2 MB; eliminates JSON AST array | `cargo test -p solix-common -p solix-compilation` | [x] Completed |
| **Selective AST Ingestion** | Ingests only imported class slice (`Console`), pruning 95 unused classes | `cargo test -p solix-compilation` | [x] Completed |
| **Sub-10 ms Compilation Latency** | `extern import solix.io.Console;` compiles in < 10 ms (down from 112 ms) | `solix compile bench.slx --metrics-json` | [x] Completed |
| **SHA-256 Bytecode & Native Auth** | Rejects tampered `.slxb` and `.so` libraries; allows with `--yolo` | `cargo test -p solix-runtime -p solix-toolchain` | [x] Completed |
| **Documentation Sync (`GL-0013`)** | Wiki, Specs, and Guides updated for `sync`, `info`, `list`, `--yolo`, `E0291` | `git diff main...HEAD -- docs/` | [x] Completed |
| **Clippy & Formatting Clean** | Zero warnings across all crates | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |
