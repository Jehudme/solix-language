//! Semantic versioning, dependency requirement specifications, and the pluggable `ModuleSolver` trait.

#![forbid(unsafe_code)]

use crate::bytecode::binary::BinaryProgram;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Parsed semantic version following the Solix `Major.Medium.Minor` format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageVersion {
    /// Major version component (breaking API changes).
    pub major: u32,
    /// Medium version component (backward-compatible feature additions).
    pub medium: u32,
    /// Minor version component (backward-compatible bug fixes and patches).
    pub minor: u32,
}

impl PackageVersion {
    /// Creates a new explicit version triple.
    #[must_use]
    pub const fn new(major: u32, medium: u32, minor: u32) -> Self {
        Self {
            major,
            medium,
            minor,
        }
    }

    /// Parses a semantic version string in `"Major.Medium.Minor"` format.
    ///
    /// # Errors
    /// Returns a [`SolverError::InvalidVersionString`] if parsing fails or components are missing.
    pub fn parse(version_string: &str) -> Result<Self, SolverError> {
        let trimmed_version = version_string
            .trim()
            .strip_prefix('v')
            .unwrap_or(version_string.trim());
        let components: Vec<&str> = trimmed_version.split('.').collect();

        if components.len() != 3 {
            return Err(SolverError::InvalidVersionString {
                raw: version_string.to_string(),
                reason: format!(
                    "Expected exactly three components (major.medium.minor), found {}",
                    components.len()
                ),
            });
        }

        let major_number =
            components[0]
                .parse::<u32>()
                .map_err(|error| SolverError::InvalidVersionString {
                    raw: version_string.to_string(),
                    reason: format!("Invalid major version '{}': {error}", components[0]),
                })?;

        let medium_number =
            components[1]
                .parse::<u32>()
                .map_err(|error| SolverError::InvalidVersionString {
                    raw: version_string.to_string(),
                    reason: format!("Invalid medium version '{}': {error}", components[1]),
                })?;

        let minor_number =
            components[2]
                .parse::<u32>()
                .map_err(|error| SolverError::InvalidVersionString {
                    raw: version_string.to_string(),
                    reason: format!("Invalid minor version '{}': {error}", components[2]),
                })?;

        Ok(Self {
            major: major_number,
            medium: medium_number,
            minor: minor_number,
        })
    }

    /// Evaluates whether this candidate version is backward-compatible with a required minimum version.
    ///
    /// # SemVer Compatibility Rules:
    /// 1. Major version must match exactly (`candidate.major == required.major`).
    /// 2. Medium version must be equal or higher (`candidate.medium >= required.medium`).
    /// 3. If medium version matches, minor version must be equal or higher.
    /// 4. If medium version is strictly higher, any minor version is compatible.
    #[must_use]
    pub const fn is_compatible_with(&self, required: &Self) -> bool {
        if self.major != required.major {
            return false;
        }

        if self.medium < required.medium {
            return false;
        }

        if self.medium == required.medium {
            self.minor >= required.minor
        } else {
            true
        }
    }
}

impl fmt::Display for PackageVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.medium, self.minor)
    }
}

/// A named package dependency requirement (e.g. `raylib@1.2.0` or `solix@2.0.0#e3b0c4...`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DependencyRequirement {
    /// Package canonical identifier name.
    pub name: String,
    /// Required minimum compatible semantic version.
    pub version: PackageVersion,
    /// Exact SHA-256 hex digest of the dependency `.slxb` binary at compile time.
    #[serde(default)]
    pub sha256: String,
}

impl DependencyRequirement {
    /// Creates a new dependency requirement.
    pub fn new(name: impl Into<String>, version: PackageVersion) -> Self {
        Self {
            name: name.into(),
            version,
            sha256: String::new(),
        }
    }

    /// Associates an expected cryptographic SHA-256 checksum with this requirement.
    #[must_use]
    pub fn with_sha256(mut self, sha256: impl Into<String>) -> Self {
        self.sha256 = sha256.into();
        self
    }

    /// Parses a requirement specification string formatted as `name@version` or `name@version#sha256`.
    ///
    /// # Errors
    /// Returns a [`SolverError`] if the format is invalid or version parsing fails.
    pub fn parse(specification: &str) -> Result<Self, SolverError> {
        let trimmed_spec = specification.trim();
        let parts: Vec<&str> = trimmed_spec.split('@').collect();

        if parts.len() != 2 {
            return Err(SolverError::InvalidVersionString {
                raw: specification.to_string(),
                reason: "Dependency specification must be formatted as 'name@version' or 'name@version#sha256'".to_string(),
            });
        }

        let package_name = parts[0].trim();
        if package_name.is_empty() {
            return Err(SolverError::InvalidVersionString {
                raw: specification.to_string(),
                reason: "Package name cannot be empty".to_string(),
            });
        }

        let (version_str, sha256_str) = if let Some((ver, sha)) = parts[1].split_once('#') {
            (ver.trim(), sha.trim().to_string())
        } else {
            (parts[1].trim(), String::new())
        };

        let parsed_version = PackageVersion::parse(version_str)?;

        Ok(Self {
            name: package_name.to_string(),
            version: parsed_version,
            sha256: sha256_str,
        })
    }
}

impl fmt::Display for DependencyRequirement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sha256.is_empty() {
            write!(formatter, "{}@{}", self.name, self.version)
        } else {
            write!(formatter, "{}@{}#{}", self.name, self.version, self.sha256)
        }
    }
}

/// The payload returned by a [`ModuleSolver`] when an installed package is resolved.
#[derive(Debug, Clone)]
pub struct SolvedPackage {
    /// Canonical package name.
    pub name: String,
    /// Exact resolved package version.
    pub version: PackageVersion,
    /// Primary compiled bytecode container (.slxb) of the package.
    pub program: BinaryProgram,
    /// Secondary .slxb binaries if bundled inside a .slxarc archive container.
    pub secondary_programs: Vec<BinaryProgram>,
    /// Extracted host-platform native shared libraries (.so / .dll / .dylib) if available.
    pub native_libraries: Vec<PathBuf>,
}

impl SolvedPackage {
    /// Creates a new solved package payload.
    pub fn new(name: impl Into<String>, version: PackageVersion, program: BinaryProgram) -> Self {
        Self {
            name: name.into(),
            version,
            program,
            secondary_programs: Vec::new(),
            native_libraries: Vec::new(),
        }
    }

    /// Appends a secondary binary container.
    #[must_use]
    pub fn with_secondary_program(mut self, program: BinaryProgram) -> Self {
        self.secondary_programs.push(program);
        self
    }

    /// Appends a native shared library path.
    #[must_use]
    pub fn with_native_library(mut self, library_path: PathBuf) -> Self {
        self.native_libraries.push(library_path);
        self
    }
}

/// Errors emitted during package resolution, version solving, or artifact extraction.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SolverError {
    #[error("Invalid version string '{raw}': {reason}")]
    InvalidVersionString { raw: String, reason: String },

    #[error("Package '{name}' was not found in installed packages store")]
    PackageNotFound { name: String },

    #[error(
        "No compatible version found for package '{name}': required minimum {required}, available versions: [{}]",
        available.iter().map(|item| item.to_string()).collect::<Vec<_>>().join(", ")
    )]
    NoCompatibleVersion {
        name: String,
        required: PackageVersion,
        available: Vec<PackageVersion>,
    },

    #[error(
        "Incompatible dependency versions requested for package '{name}' in dependency tree: {first_version} vs {second_version}"
    )]
    IncompatibleVersionsInTree {
        name: String,
        first_version: PackageVersion,
        second_version: PackageVersion,
    },

    #[error("Package store I/O error at '{path}': {reason}")]
    IoError { path: PathBuf, reason: String },

    #[error("Failed to extract package archive: {reason}")]
    ExtractionError { reason: String },
}

/// Pluggable trait for fetching external packages during compilation and runtime.
pub trait ModuleSolver: Send + Sync + fmt::Debug {
    /// Finds and loads the highest installed version of `package_name` that is compatible
    /// with `min_version` (same major, >= medium, and >= minor if same medium).
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError>;

    /// Finds and loads the absolute highest installed version of `package_name`.
    fn resolve_latest(&self, package_name: &str) -> Result<SolvedPackage, SolverError>;

    /// Returns a sorted list of all available top-level package names in the solver store.
    fn list_packages(&self) -> Vec<String> {
        Vec::new()
    }
}

/// An in-memory [`ModuleSolver`] implementation used for hermetic compiler and runtime testing.
#[derive(Debug, Default, Clone)]
pub struct InMemoryModuleSolver {
    packages: HashMap<String, Vec<SolvedPackage>>,
}

impl InMemoryModuleSolver {
    /// Creates an empty in-memory module solver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Registers a solved package into the in-memory store.
    pub fn add_package(&mut self, package: SolvedPackage) {
        self.packages
            .entry(package.name.clone())
            .or_default()
            .push(package);
    }

    /// Builder method to register a solved package.
    #[must_use]
    pub fn with_package(mut self, package: SolvedPackage) -> Self {
        self.add_package(package);
        self
    }
}

impl ModuleSolver for InMemoryModuleSolver {
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError> {
        let package_entries =
            self.packages
                .get(package_name)
                .ok_or_else(|| SolverError::PackageNotFound {
                    name: package_name.to_string(),
                })?;

        let mut compatible_entries: Vec<&SolvedPackage> = package_entries
            .iter()
            .filter(|candidate| candidate.version.is_compatible_with(&min_version))
            .collect();

        if compatible_entries.is_empty() {
            let available_versions: Vec<PackageVersion> =
                package_entries.iter().map(|entry| entry.version).collect();
            return Err(SolverError::NoCompatibleVersion {
                name: package_name.to_string(),
                required: min_version,
                available: available_versions,
            });
        }

        compatible_entries.sort_by_key(|entry| entry.version);
        let winning_entry = compatible_entries.last().expect("non-empty collection");
        Ok((*winning_entry).clone())
    }

    fn resolve_latest(&self, package_name: &str) -> Result<SolvedPackage, SolverError> {
        let package_entries =
            self.packages
                .get(package_name)
                .ok_or_else(|| SolverError::PackageNotFound {
                    name: package_name.to_string(),
                })?;

        if package_entries.is_empty() {
            return Err(SolverError::PackageNotFound {
                name: package_name.to_string(),
            });
        }

        let mut sorted_entries = package_entries.clone();
        sorted_entries.sort_by_key(|entry| entry.version);
        let winning_entry = sorted_entries.last().expect("non-empty collection");
        Ok(winning_entry.clone())
    }

    fn list_packages(&self) -> Vec<String> {
        let mut package_names: Vec<String> = self.packages.keys().cloned().collect();
        package_names.sort();
        package_names
    }
}

/// Resolves the canonical root directory of the installed packages store.
#[must_use]
pub fn resolve_package_store_dir(override_directory: Option<&Path>) -> PathBuf {
    if let Some(custom_directory) = override_directory {
        return custom_directory.to_path_buf();
    }

    if let Ok(pkg_dir) = std::env::var("SOLIX_PKG_DIR") {
        let trimmed_pkg_dir = pkg_dir.trim();
        if !trimmed_pkg_dir.is_empty() {
            return PathBuf::from(trimmed_pkg_dir);
        }
    }

    if let Ok(home_dir) = std::env::var("SOLIX_HOME") {
        let trimmed_home_dir = home_dir.trim();
        if !trimmed_home_dir.is_empty() {
            return PathBuf::from(trimmed_home_dir).join("packages");
        }
    }

    if cfg!(windows) {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let trimmed_profile = user_profile.trim();
            if !trimmed_profile.is_empty() {
                return PathBuf::from(trimmed_profile)
                    .join(".solix")
                    .join("packages");
            }
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let trimmed_app_data = local_app_data.trim();
            if !trimmed_app_data.is_empty() {
                return PathBuf::from(trimmed_app_data)
                    .join("solix")
                    .join("packages");
            }
        }
    }

    if let Ok(home_dir) = std::env::var("HOME") {
        let trimmed_home = home_dir.trim();
        if !trimmed_home.is_empty() {
            return PathBuf::from(trimmed_home).join(".solix").join("packages");
        }
    }

    PathBuf::from(".solix").join("packages")
}

/// Resolves the cache directory for extracted native shared libraries.
#[must_use]
pub fn resolve_native_cache_dir(store_directory: &Path) -> PathBuf {
    store_directory
        .parent()
        .map(|parent_path| parent_path.join("cache").join("natives"))
        .unwrap_or_else(|| PathBuf::from(".solix").join("cache").join("natives"))
}

/// Concrete [`ModuleSolver`] resolving packages from the local filesystem package store.
#[derive(Debug, Clone)]
pub struct ClientModuleSolver {
    store_directory: PathBuf,
    native_cache_directory: PathBuf,
    package_cache: std::sync::Arc<dyn crate::cache::PackageCache>,
}

impl Default for ClientModuleSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientModuleSolver {
    /// Creates a client module solver using standard environment directories.
    #[must_use]
    pub fn new() -> Self {
        let store_directory = resolve_package_store_dir(None);
        let native_cache_directory = resolve_native_cache_dir(&store_directory);
        let cache_root = crate::cache::resolve_packages_cache_dir(None);
        Self {
            store_directory,
            native_cache_directory,
            package_cache: std::sync::Arc::new(crate::cache::FilesystemPackageCache::with_root(cache_root)),
        }
    }

    /// Creates a client module solver with explicit store and native cache directories.
    #[must_use]
    pub fn with_store_dir(store_directory: PathBuf) -> Self {
        let native_cache_directory = resolve_native_cache_dir(&store_directory);
        let cache_root = store_directory
            .parent()
            .map(|parent_path| parent_path.join("cache").join("packages"))
            .unwrap_or_else(|| crate::cache::resolve_packages_cache_dir(None));
        Self {
            store_directory,
            native_cache_directory,
            package_cache: std::sync::Arc::new(crate::cache::FilesystemPackageCache::with_root(cache_root)),
        }
    }

    /// Configures an explicit package extraction cache for this solver.
    #[must_use]
    pub fn with_package_cache(mut self, package_cache: std::sync::Arc<dyn crate::cache::PackageCache>) -> Self {
        self.package_cache = package_cache;
        self
    }

    /// Returns the root path of the managed package store.
    #[must_use]
    pub fn store_directory(&self) -> &std::path::Path {
        &self.store_directory
    }

    /// Returns the directory path for extracted native libraries.
    #[must_use]
    pub fn native_cache_directory(&self) -> &std::path::Path {
        &self.native_cache_directory
    }

    /// Returns a reference to the active package extraction cache.
    #[must_use]
    pub fn package_cache(&self) -> &std::sync::Arc<dyn crate::cache::PackageCache> {
        &self.package_cache
    }

    /// Scans the package store for all installed versions of `package_name`.
    fn scan_installed_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<(PackageVersion, PathBuf)>, SolverError> {
        let package_folder = self.store_directory.join(package_name);
        if !package_folder.exists() || !package_folder.is_dir() {
            return Err(SolverError::PackageNotFound {
                name: package_name.to_string(),
            });
        }

        let mut discovered_versions = Vec::new();
        let read_entries = std::fs::read_dir(&package_folder).map_err(|error| SolverError::IoError {
            path: package_folder.clone(),
            reason: error.to_string(),
        })?;

        for entry_result in read_entries {
            let entry = entry_result.map_err(|error| SolverError::IoError {
                path: package_folder.clone(),
                reason: error.to_string(),
            })?;

            if entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
                let folder_name = entry.file_name();
                let version_string = folder_name.to_string_lossy();
                if let Ok(parsed_version) = PackageVersion::parse(&version_string) {
                    discovered_versions.push((parsed_version, entry.path()));
                }
            }
        }

        if discovered_versions.is_empty() {
            return Err(SolverError::PackageNotFound {
                name: package_name.to_string(),
            });
        }

        discovered_versions.sort_by_key(|(version, _)| *version);
        Ok(discovered_versions)
    }

    /// Loads a package from its version directory containing `<name>.slxarc` or `<name>.slxb`.
    fn load_package_from_dir(
        &self,
        package_name: &str,
        version: PackageVersion,
        version_dir: &std::path::Path,
    ) -> Result<SolvedPackage, SolverError> {
        let slxarc_candidate = version_dir.join(format!("{package_name}.slxarc"));
        let slxb_candidate = version_dir.join(format!("{package_name}.slxb"));

        if slxarc_candidate.exists() {
            let extraction_target = self
                .native_cache_directory
                .join(format!("{package_name}-{version}"));

            // 1. Check authenticated extraction cache first to avoid re-inflating archive
            if let Ok(Some(cached_program)) =
                self.package_cache
                    .get_cached_binary(package_name, version, &slxarc_candidate)
            {
                let mut solved_package = SolvedPackage::new(package_name, version, cached_program);

                let native_lib_dir = extraction_target.join("lib");
                if native_lib_dir.exists() && native_lib_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&native_lib_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                solved_package = solved_package.with_native_library(path);
                            }
                        }
                    }
                    return Ok(solved_package);
                }
            }

            // 2. Cache miss: extract archive from scratch
            let loaded_archive =
                crate::project::SlxarcArchive::extract_and_load(&slxarc_candidate, &extraction_target).map_err(
                    |error| SolverError::ExtractionError {
                        reason: format!(
                            "Failed to extract '{package_name}' archive at '{}': {error}",
                            slxarc_candidate.display()
                        ),
                    },
                )?;

            let primary_program = loaded_archive.binaries.into_iter().next().ok_or_else(|| {
                SolverError::ExtractionError {
                    reason: format!(
                        "Archive '{}' does not contain a primary binary in bin/",
                        slxarc_candidate.display()
                    ),
                }
            })?;

            // 3. Populate authenticated extraction cache
            let primary_bytes = primary_program.to_bytes();
            let _ = self.package_cache.put_cached_binary(
                package_name,
                version,
                &slxarc_candidate,
                &primary_program,
                &primary_bytes,
            );

            let mut solved_package = SolvedPackage::new(package_name, version, primary_program);
            for secondary in loaded_archive.secondary_binaries {
                solved_package = solved_package.with_secondary_program(secondary);
            }
            for native_lib in loaded_archive.native_libraries {
                solved_package = solved_package.with_native_library(native_lib);
            }

            Ok(solved_package)
        } else if slxb_candidate.exists() {
            let binary_bytes = std::fs::read(&slxb_candidate).map_err(|error| SolverError::IoError {
                path: slxb_candidate.clone(),
                reason: error.to_string(),
            })?;

            let program = BinaryProgram::from_bytes(&binary_bytes).map_err(|error| {
                SolverError::ExtractionError {
                    reason: format!(
                        "Failed to deserialize binary at '{}': {error}",
                        slxb_candidate.display()
                    ),
                }
            })?;

            Ok(SolvedPackage::new(package_name, version, program))
        } else {
            Err(SolverError::IoError {
                path: version_dir.to_path_buf(),
                reason: format!(
                    "Neither '{package_name}.slxarc' nor '{package_name}.slxb' found in '{}'",
                    version_dir.display()
                ),
            })
        }
    }
}

impl ModuleSolver for ClientModuleSolver {
    fn resolve_compatible(
        &self,
        package_name: &str,
        min_version: PackageVersion,
    ) -> Result<SolvedPackage, SolverError> {
        let discovered_versions = self.scan_installed_versions(package_name)?;

        let mut compatible_versions: Vec<(PackageVersion, PathBuf)> = discovered_versions
            .into_iter()
            .filter(|(version, _)| version.is_compatible_with(&min_version))
            .collect();

        if compatible_versions.is_empty() {
            let available_versions = self
                .scan_installed_versions(package_name)?
                .into_iter()
                .map(|(version, _)| version)
                .collect();
            return Err(SolverError::NoCompatibleVersion {
                name: package_name.to_string(),
                required: min_version,
                available: available_versions,
            });
        }

        compatible_versions.sort_by_key(|(version, _)| *version);
        let (winning_version, winning_dir) = compatible_versions
            .last()
            .expect("compatible versions non-empty");

        self.load_package_from_dir(package_name, *winning_version, winning_dir)
    }

    fn resolve_latest(&self, package_name: &str) -> Result<SolvedPackage, SolverError> {
        let mut discovered_versions = self.scan_installed_versions(package_name)?;
        discovered_versions.sort_by_key(|(version, _)| *version);

        let (winning_version, winning_dir) = discovered_versions
            .last()
            .expect("discovered versions non-empty");

        self.load_package_from_dir(package_name, *winning_version, winning_dir)
    }

    fn list_packages(&self) -> Vec<String> {
        let mut package_names = Vec::new();
        if let Ok(read_dir) = fs::read_dir(&self.store_directory) {
            for entry_result in read_dir.flatten() {
                let package_path = entry_result.path();
                if package_path.is_dir() {
                    if let Some(folder_name) = package_path.file_name().and_then(|name| name.to_str()) {
                        if let Ok(version_read_dir) = fs::read_dir(&package_path) {
                            let mut has_version = false;
                            for version_entry_result in version_read_dir.flatten() {
                                let version_candidate = version_entry_result.path();
                                if version_candidate.is_dir() {
                                    if let Some(candidate_name) = version_candidate.file_name().and_then(|name| name.to_str()) {
                                        if PackageVersion::parse(candidate_name).is_ok() {
                                            has_version = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if has_version {
                                package_names.push(folder_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        package_names.sort();
        package_names.dedup();
        package_names
    }
}
