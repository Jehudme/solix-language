//! Client module solver and cross-platform package store manager.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_common::project::SlxarcArchive;
use solix_common::solver::{ModuleSolver, PackageVersion, SolvedPackage, SolverError};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolves the canonical root directory of the installed packages store.
///
/// Priority:
/// 1. Explicit `override_directory` if supplied.
/// 2. `SOLIX_PKG_DIR` environment variable.
/// 3. `SOLIX_HOME` environment variable -> `$SOLIX_HOME/packages`.
/// 4. Windows: `%USERPROFILE%\.solix\packages`.
/// 5. Unix/macOS: `$HOME/.solix/packages`.
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
        Self {
            store_directory,
            native_cache_directory,
        }
    }

    /// Creates a client module solver with explicit store and native cache directories.
    #[must_use]
    pub fn with_store_dir(store_directory: PathBuf) -> Self {
        let native_cache_directory = resolve_native_cache_dir(&store_directory);
        Self {
            store_directory,
            native_cache_directory,
        }
    }

    /// Returns the root path of the managed package store.
    #[must_use]
    pub fn store_directory(&self) -> &Path {
        &self.store_directory
    }

    /// Returns the directory path for extracted native libraries.
    #[must_use]
    pub fn native_cache_directory(&self) -> &Path {
        &self.native_cache_directory
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
        let read_entries = fs::read_dir(&package_folder).map_err(|error| SolverError::IoError {
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
        version_dir: &Path,
    ) -> Result<SolvedPackage, SolverError> {
        let slxarc_candidate = version_dir.join(format!("{package_name}.slxarc"));
        let slxb_candidate = version_dir.join(format!("{package_name}.slxb"));

        if slxarc_candidate.exists() {
            let extraction_target = self
                .native_cache_directory
                .join(format!("{package_name}-{version}"));

            let loaded_archive =
                SlxarcArchive::extract_and_load(&slxarc_candidate, &extraction_target).map_err(
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

            let mut solved_package = SolvedPackage::new(package_name, version, primary_program);
            for secondary in loaded_archive.secondary_binaries {
                solved_package = solved_package.with_secondary_program(secondary);
            }
            for native_lib in loaded_archive.native_libraries {
                solved_package = solved_package.with_native_library(native_lib);
            }

            Ok(solved_package)
        } else if slxb_candidate.exists() {
            let binary_bytes = fs::read(&slxb_candidate).map_err(|error| SolverError::IoError {
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
}
