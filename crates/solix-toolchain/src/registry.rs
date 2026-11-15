//! Cross-platform package registry descriptor and manifest (`registry.json`).

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solix_common::bytecode::BinaryProgram;
use solix_common::hash::{sha256_file, sha256_hex};
use solix_common::project::SlxarcArchive;
use solix_common::solver::PackageVersion;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn default_schema_version() -> u32 {
    1
}

/// Metadata record for an installed package registered in `$SOLIX_PKG_DIR/registry.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRecord {
    /// Canonical package name.
    pub name: String,
    /// Semantic version string (e.g. "2.0.0").
    pub version: String,
    /// Container format kind ("slxb" or "slxarc").
    pub kind: String,
    /// Recorded build profile ("debug", "release", or "test").
    pub build_profile: String,
    /// Cryptographic SHA-256 digest of the installed binary or archive artifact.
    pub sha256: String,
    /// File size in bytes of the installed package artifact.
    pub size_bytes: u64,
    /// Installation Unix epoch timestamp in seconds.
    pub installed_at_unix_secs: u64,
    /// Human-readable ISO 8601 UTC timestamp string.
    pub installed_at_iso8601: String,
    /// Optional filesystem path to the original source project directory for rebuilding via `solix sync`.
    #[serde(default)]
    pub source_project_dir: Option<PathBuf>,
    /// Filesystem path to the installed package artifact in the store.
    pub artifact_path: PathBuf,
    /// Cryptographic SHA-256 digests of bundled native shared libraries.
    #[serde(default)]
    pub native_libraries: HashMap<String, String>,
    /// Package description summary.
    #[serde(default)]
    pub description: String,
    /// SPDX license identifier.
    #[serde(default)]
    pub license: String,
    /// Author / maintainer names.
    #[serde(default)]
    pub authors: Vec<String>,
}

/// Package registry container representing `$SOLIX_PKG_DIR/registry.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRegistry {
    /// Registry schema format version.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Registered package entries.
    #[serde(default)]
    pub packages: Vec<PackageRecord>,
    /// Absolute filesystem path to the managed `registry.json` file.
    #[serde(skip)]
    registry_file_path: PathBuf,
}

impl Default for PackageRegistry {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            packages: Vec::new(),
            registry_file_path: PathBuf::new(),
        }
    }
}

impl PackageRegistry {
    /// Loads `$SOLIX_PKG_DIR/registry.json` or creates a new one, self-healing by
    /// backfilling any installed packages present on disk but missing from the manifest.
    pub fn load_or_sync(store_directory: &Path) -> Result<Self> {
        let registry_path = store_directory.join("registry.json");
        let mut registry = if registry_path.exists() {
            let manifest_content = fs::read_to_string(&registry_path).with_context(|| {
                format!(
                    "Failed to read package registry at '{}'",
                    registry_path.display()
                )
            })?;
            let mut parsed_registry: PackageRegistry = serde_json::from_str(&manifest_content)
                .with_context(|| {
                    format!(
                        "Failed to parse package registry JSON at '{}'",
                        registry_path.display()
                    )
                })?;
            parsed_registry.registry_file_path = registry_path.clone();
            parsed_registry
        } else {
            Self {
                schema_version: default_schema_version(),
                packages: Vec::new(),
                registry_file_path: registry_path.clone(),
            }
        };

        // Automatic self-healing: scan store directory for unlisted packages
        let mut modified = false;
        if store_directory.exists()
            && store_directory.is_dir()
            && let Ok(package_dir_entries) = fs::read_dir(store_directory)
        {
            for package_entry_result in package_dir_entries.flatten() {
                let package_dir = package_entry_result.path();
                if !package_dir.is_dir() {
                    continue;
                }
                let package_name = package_entry_result
                    .file_name()
                    .to_string_lossy()
                    .to_string();
                if package_name.starts_with('.') || package_name == "registry.json" {
                    continue;
                }

                if let Ok(version_dir_entries) = fs::read_dir(&package_dir) {
                    for version_entry_result in version_dir_entries.flatten() {
                        let version_dir = version_entry_result.path();
                        if !version_dir.is_dir() {
                            continue;
                        }
                        let version_string = version_entry_result
                            .file_name()
                            .to_string_lossy()
                            .to_string();
                        if PackageVersion::parse(&version_string).is_err() {
                            continue;
                        }

                        let already_registered = registry.packages.iter().any(|existing| {
                            existing.name == package_name && existing.version == version_string
                        });

                        if !already_registered
                            && let Some(backfilled_record) = backfill_package_from_dir(
                                &package_name,
                                &version_string,
                                &version_dir,
                            )
                        {
                            registry.packages.push(backfilled_record);
                            modified = true;
                        }
                    }
                }
            }
        }

        if modified || !registry_path.exists() {
            registry.save()?;
        }

        Ok(registry)
    }

    /// Atomically writes the package registry manifest to disk.
    pub fn save(&self) -> Result<()> {
        if self.registry_file_path.as_os_str().is_empty() {
            return Ok(());
        }

        if let Some(parent_directory) = self.registry_file_path.parent() {
            fs::create_dir_all(parent_directory).with_context(|| {
                format!(
                    "Failed to create registry directory '{}'",
                    parent_directory.display()
                )
            })?;
        }

        let json_string =
            serde_json::to_string_pretty(self).context("Failed to serialize registry manifest")?;

        let temporary_path = self
            .registry_file_path
            .with_extension(format!("json.tmp.{}", std::process::id()));

        {
            let mut file_handle = File::create(&temporary_path).with_context(|| {
                format!(
                    "Failed to create temporary registry file at '{}'",
                    temporary_path.display()
                )
            })?;
            file_handle
                .write_all(json_string.as_bytes())
                .context("Failed to write registry content")?;
            file_handle
                .flush()
                .context("Failed to flush registry file")?;
        }

        fs::rename(&temporary_path, &self.registry_file_path).with_context(|| {
            format!(
                "Failed to rename temporary registry file to '{}'",
                self.registry_file_path.display()
            )
        })?;

        Ok(())
    }

    /// Finds a registered package record by exact name and optional version (or highest SemVer version if omitted).
    #[must_use]
    pub fn get(
        &self,
        package_name: &str,
        requested_version: Option<&str>,
    ) -> Option<&PackageRecord> {
        if let Some(target_version) = requested_version {
            self.packages
                .iter()
                .find(|record| record.name == package_name && record.version == target_version)
        } else {
            // Find highest SemVer version
            let mut matching_records: Vec<&PackageRecord> = self
                .packages
                .iter()
                .filter(|record| record.name == package_name)
                .collect();

            matching_records.sort_by(|record_alpha, record_beta| {
                let parsed_alpha = PackageVersion::parse(&record_alpha.version).ok();
                let parsed_beta = PackageVersion::parse(&record_beta.version).ok();
                parsed_alpha.cmp(&parsed_beta)
            });

            matching_records.last().copied()
        }
    }

    /// Returns all registered package records matching `package_name`.
    #[must_use]
    pub fn find_all(&self, package_name: &str) -> Vec<&PackageRecord> {
        self.packages
            .iter()
            .filter(|record| record.name == package_name)
            .collect()
    }

    /// Registers or updates a package record and persists changes to disk.
    pub fn register(&mut self, record: PackageRecord) -> Result<()> {
        if let Some(existing_index) = self
            .packages
            .iter()
            .position(|existing| existing.name == record.name && existing.version == record.version)
        {
            self.packages[existing_index] = record;
        } else {
            self.packages.push(record);
        }
        self.save()
    }

    /// Removes a package record from the registry and persists changes to disk.
    pub fn unregister(
        &mut self,
        package_name: &str,
        package_version: &str,
    ) -> Result<Option<PackageRecord>> {
        if let Some(existing_index) = self.packages.iter().position(|existing| {
            existing.name == package_name && existing.version == package_version
        }) {
            let removed = self.packages.remove(existing_index);
            self.save()?;
            Ok(Some(removed))
        } else {
            Ok(None)
        }
    }
}

/// Helper function to construct a backfilled `PackageRecord` from an unlisted directory.
fn backfill_package_from_dir(
    package_name: &str,
    version_string: &str,
    version_dir: &Path,
) -> Option<PackageRecord> {
    let slxarc_candidate = version_dir.join(format!("{package_name}.slxarc"));
    let slxb_candidate = version_dir.join(format!("{package_name}.slxb"));

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let iso8601_string = format_iso8601_timestamp(current_timestamp);

    if slxarc_candidate.exists() {
        let file_metadata = fs::metadata(&slxarc_candidate).ok()?;
        let computed_sha256 = sha256_file(&slxarc_candidate).ok()?;
        let inspection = SlxarcArchive::inspect(&slxarc_candidate).ok()?;

        let description = inspection
            .manifest
            .as_ref()
            .and_then(|manifest| manifest.info.description.clone())
            .unwrap_or_default();
        let license = inspection
            .manifest
            .as_ref()
            .and_then(|manifest| manifest.info.license.clone())
            .unwrap_or_default();
        let authors = inspection
            .manifest
            .as_ref()
            .map(|manifest| manifest.info.authors.clone())
            .unwrap_or_default();
        let native_libraries = inspection
            .manifest
            .as_ref()
            .map(|manifest| manifest.native_library_checksums.clone())
            .unwrap_or_default();

        Some(PackageRecord {
            name: package_name.to_string(),
            version: version_string.to_string(),
            kind: "slxarc".to_string(),
            build_profile: "release".to_string(),
            sha256: computed_sha256,
            size_bytes: file_metadata.len(),
            installed_at_unix_secs: current_timestamp,
            installed_at_iso8601: iso8601_string,
            source_project_dir: None,
            artifact_path: slxarc_candidate,
            native_libraries,
            description,
            license,
            authors,
        })
    } else if slxb_candidate.exists() {
        let file_metadata = fs::metadata(&slxb_candidate).ok()?;
        let file_bytes = fs::read(&slxb_candidate).ok()?;
        let computed_sha256 = sha256_hex(&file_bytes);
        let program = BinaryProgram::from_bytes(&file_bytes).ok()?;

        Some(PackageRecord {
            name: package_name.to_string(),
            version: version_string.to_string(),
            kind: "slxb".to_string(),
            build_profile: program.build_profile.clone(),
            sha256: computed_sha256,
            size_bytes: file_metadata.len(),
            installed_at_unix_secs: current_timestamp,
            installed_at_iso8601: iso8601_string,
            source_project_dir: None,
            artifact_path: slxb_candidate,
            native_libraries: program.native_library_checksums.clone(),
            description: program.package_description.clone(),
            license: String::new(),
            authors: Vec::new(),
        })
    } else {
        None
    }
}

/// Formats a Unix timestamp into an approximate ISO 8601 UTC string (`YYYY-MM-DDTHH:MM:SSZ`).
#[must_use]
pub fn format_iso8601_timestamp(unix_seconds: u64) -> String {
    let seconds_per_day = 86400u64;
    let days_since_epoch = unix_seconds / seconds_per_day;
    let seconds_in_day = unix_seconds % seconds_per_day;

    let hour = seconds_in_day / 3600;
    let minute = (seconds_in_day % 3600) / 60;
    let second = seconds_in_day % 60;

    // Civil day calculation
    let shifted_days = days_since_epoch + 719468;
    let era = shifted_days / 146097;
    let day_of_era = shifted_days - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_internal = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_internal + 2) / 5 + 1;
    let month = if month_internal < 10 {
        month_internal + 3
    } else {
        month_internal - 9
    };
    let adjusted_year = if month <= 2 { year + 1 } else { year };

    format!("{adjusted_year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
