//! Semantic versioning, dependency requirement specifications, and the pluggable `ModuleSolver` trait.

#![forbid(unsafe_code)]

use crate::bytecode::binary::BinaryProgram;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

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

/// A named package dependency requirement (e.g. `raylib@1.2.0`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DependencyRequirement {
    /// Package canonical identifier name.
    pub name: String,
    /// Required minimum compatible semantic version.
    pub version: PackageVersion,
}

impl DependencyRequirement {
    /// Creates a new dependency requirement.
    pub fn new(name: impl Into<String>, version: PackageVersion) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }

    /// Parses a requirement specification string formatted as `name@version`.
    ///
    /// # Errors
    /// Returns a [`SolverError`] if the format is invalid or version parsing fails.
    pub fn parse(specification: &str) -> Result<Self, SolverError> {
        let trimmed_spec = specification.trim();
        let parts: Vec<&str> = trimmed_spec.split('@').collect();

        if parts.len() != 2 {
            return Err(SolverError::InvalidVersionString {
                raw: specification.to_string(),
                reason: "Dependency specification must be formatted as 'name@version'".to_string(),
            });
        }

        let package_name = parts[0].trim();
        if package_name.is_empty() {
            return Err(SolverError::InvalidVersionString {
                raw: specification.to_string(),
                reason: "Package name cannot be empty".to_string(),
            });
        }

        let parsed_version = PackageVersion::parse(parts[1].trim())?;

        Ok(Self {
            name: package_name.to_string(),
            version: parsed_version,
        })
    }
}

impl fmt::Display for DependencyRequirement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.name, self.version)
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
}
