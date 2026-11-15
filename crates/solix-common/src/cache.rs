//! Pluggable package extract cache contracts and memory implementations.

#![forbid(unsafe_code)]

use crate::bytecode::BinaryProgram;
use crate::diagnostics::EmissionError;
use crate::hash::{sha256_bytes, sha256_file};
use crate::solver::PackageVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Metadata sidecar recorded alongside cached package binaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedPackageMeta {
    /// Canonical package name.
    pub package_name: String,
    /// Semantic package version.
    pub package_version: String,
    /// Cryptographic SHA-256 digest of the source `.slxarc` archive file.
    pub source_archive_sha256: String,
    /// File size in bytes of the source `.slxarc` archive file.
    pub source_archive_size_bytes: u64,
    /// Cryptographic SHA-256 digest of the cached `.slxb` primary binary.
    pub cached_binary_sha256: String,
}

/// Resolves the default cache root directory for extracted packages (`~/.solix/cache/packages`).
#[must_use]
pub fn resolve_packages_cache_dir(override_directory: Option<&Path>) -> PathBuf {
    if let Some(custom_directory) = override_directory {
        return custom_directory.to_path_buf();
    }

    if let Ok(cache_dir_env) = std::env::var("SOLIX_CACHE_DIR") {
        let trimmed_cache_dir = cache_dir_env.trim();
        if !trimmed_cache_dir.is_empty() {
            return PathBuf::from(trimmed_cache_dir).join("packages");
        }
    }

    if let Ok(home_dir_env) = std::env::var("SOLIX_HOME") {
        let trimmed_home = home_dir_env.trim();
        if !trimmed_home.is_empty() {
            return PathBuf::from(trimmed_home).join("cache").join("packages");
        }
    }

    if cfg!(windows)
        && let Ok(user_profile) = std::env::var("USERPROFILE")
    {
        let trimmed_profile = user_profile.trim();
        if !trimmed_profile.is_empty() {
            return PathBuf::from(trimmed_profile)
                .join(".solix")
                .join("cache")
                .join("packages");
        }
    }

    if let Ok(home_dir) = std::env::var("HOME") {
        let trimmed_home = home_dir.trim();
        if !trimmed_home.is_empty() {
            return PathBuf::from(trimmed_home)
                .join(".solix")
                .join("cache")
                .join("packages");
        }
    }

    PathBuf::from(".solix").join("cache").join("packages")
}

/// Pluggable extraction cache trait for bypassing redundant `.slxarc` archive decompression.
pub trait PackageCache: Send + Sync + Debug {
    /// Attempts to load an extracted binary program from cache, validating that
    /// the cached artifact's fingerprint matches the source archive.
    fn get_cached_binary(
        &self,
        package_name: &str,
        version: PackageVersion,
        source_archive_path: &Path,
    ) -> Result<Option<BinaryProgram>, EmissionError>;

    /// Atomically stores an extracted binary program into the cache associated with
    /// the source archive's cryptographic SHA-256 fingerprint.
    fn put_cached_binary(
        &self,
        package_name: &str,
        version: PackageVersion,
        source_archive_path: &Path,
        program: &BinaryProgram,
        raw_binary_bytes: &[u8],
    ) -> Result<(), EmissionError>;
}

/// No-op package cache that bypasses caching completely (always cache miss).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPackageCache;

impl PackageCache for NoopPackageCache {
    fn get_cached_binary(
        &self,
        _package_name: &str,
        _version: PackageVersion,
        _source_archive_path: &Path,
    ) -> Result<Option<BinaryProgram>, EmissionError> {
        Ok(None)
    }

    fn put_cached_binary(
        &self,
        _package_name: &str,
        _version: PackageVersion,
        _source_archive_path: &Path,
        _program: &BinaryProgram,
        _raw_binary_bytes: &[u8],
    ) -> Result<(), EmissionError> {
        Ok(())
    }
}

type CachedBinaryEntry = (String, BinaryProgram, Vec<u8>);
type CacheMap = HashMap<(String, PackageVersion), CachedBinaryEntry>;

/// Hermetic in-memory package cache used for high-speed unit tests without disk I/O.
#[derive(Debug, Default)]
pub struct InMemoryPackageCache {
    entries: RwLock<CacheMap>,
}

impl InMemoryPackageCache {
    /// Creates a new empty in-memory package cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

impl PackageCache for InMemoryPackageCache {
    fn get_cached_binary(
        &self,
        package_name: &str,
        version: PackageVersion,
        source_archive_path: &Path,
    ) -> Result<Option<BinaryProgram>, EmissionError> {
        let expected_sha256 = match sha256_file(source_archive_path) {
            Ok(digest) => digest,
            Err(_) => return Ok(None),
        };

        let read_guard = self.entries.read().map_err(|poison_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("Lock poisoned: {poison_error}"),
            }
        })?;

        let cache_key = (package_name.to_string(), version);
        if let Some((stored_sha256, program, _raw_bytes)) = read_guard.get(&cache_key)
            && stored_sha256 == &expected_sha256
        {
            return Ok(Some(program.clone()));
        }

        Ok(None)
    }

    fn put_cached_binary(
        &self,
        package_name: &str,
        version: PackageVersion,
        source_archive_path: &Path,
        program: &BinaryProgram,
        raw_binary_bytes: &[u8],
    ) -> Result<(), EmissionError> {
        let archive_sha256 = sha256_file(source_archive_path).unwrap_or_default();

        let mut write_guard = self.entries.write().map_err(|poison_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("Lock poisoned: {poison_error}"),
            }
        })?;

        let cache_key = (package_name.to_string(), version);
        write_guard.insert(
            cache_key,
            (archive_sha256, program.clone(), raw_binary_bytes.to_vec()),
        );

        Ok(())
    }
}

/// Filesystem-backed package cache managing `~/.solix/cache/packages/<name>-<version>/`.
#[derive(Debug, Clone)]
pub struct FilesystemPackageCache {
    cache_root: PathBuf,
}

impl Default for FilesystemPackageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemPackageCache {
    /// Creates a new filesystem package cache rooted at standard user directories.
    #[must_use]
    pub fn new() -> Self {
        let cache_root = resolve_packages_cache_dir(None);
        Self { cache_root }
    }

    /// Creates a new filesystem package cache rooted at an explicit directory path.
    #[must_use]
    pub fn with_root(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    /// Returns the root path of the managed cache directory.
    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    fn package_entry_dir(&self, package_name: &str, version: PackageVersion) -> PathBuf {
        self.cache_root.join(format!("{package_name}-{version}"))
    }
}

impl PackageCache for FilesystemPackageCache {
    fn get_cached_binary(
        &self,
        package_name: &str,
        version: PackageVersion,
        source_archive_path: &Path,
    ) -> Result<Option<BinaryProgram>, EmissionError> {
        let entry_dir = self.package_entry_dir(package_name, version);
        let meta_file = entry_dir.join("meta.json");
        let binary_file = entry_dir.join("bin").join(format!("{package_name}.slxb"));

        if !meta_file.exists() || !binary_file.exists() {
            return Ok(None);
        }

        // 1. Read and parse metadata sidecar
        let meta_content = match fs::read_to_string(&meta_file) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        };

        let cached_meta: CachedPackageMeta = match serde_json::from_str(&meta_content) {
            Ok(parsed) => parsed,
            Err(_) => return Ok(None),
        };

        // 2. Validate source archive existence and file size first (fast path)
        let source_metadata = match fs::metadata(source_archive_path) {
            Ok(metadata) => metadata,
            Err(_) => return Ok(None),
        };

        if source_metadata.len() != cached_meta.source_archive_size_bytes {
            let _ = fs::remove_dir_all(&entry_dir);
            return Ok(None);
        }

        // 3. Validate source archive cryptographic hash
        let actual_archive_sha256 = match sha256_file(source_archive_path) {
            Ok(digest) => digest,
            Err(_) => return Ok(None),
        };

        if actual_archive_sha256 != cached_meta.source_archive_sha256 {
            let _ = fs::remove_dir_all(&entry_dir);
            return Ok(None);
        }

        // 4. Read cached binary and verify its integrity against recorded hash
        let binary_bytes = match fs::read(&binary_file) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(None),
        };

        let actual_binary_sha256 = sha256_bytes(&binary_bytes);
        let formatted_actual_sha256 = crate::hash::format_hex_digest(&actual_binary_sha256);
        if formatted_actual_sha256 != cached_meta.cached_binary_sha256 {
            let _ = fs::remove_dir_all(&entry_dir);
            return Ok(None);
        }

        // 5. Deserialize verified binary program
        match BinaryProgram::from_bytes(&binary_bytes) {
            Ok(program) => Ok(Some(program)),
            Err(_) => {
                let _ = fs::remove_dir_all(&entry_dir);
                Ok(None)
            }
        }
    }

    fn put_cached_binary(
        &self,
        package_name: &str,
        version: PackageVersion,
        source_archive_path: &Path,
        _program: &BinaryProgram,
        raw_binary_bytes: &[u8],
    ) -> Result<(), EmissionError> {
        let entry_dir = self.package_entry_dir(package_name, version);
        let bin_dir = entry_dir.join("bin");

        fs::create_dir_all(&bin_dir).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Failed to create cache directory '{}': {io_error}",
                    bin_dir.display()
                ),
            }
        })?;

        let source_metadata = fs::metadata(source_archive_path).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Failed to read source archive metadata '{}': {io_error}",
                    source_archive_path.display()
                ),
            }
        })?;

        let source_archive_sha256 = sha256_file(source_archive_path).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Failed to hash source archive '{}': {io_error}",
                    source_archive_path.display()
                ),
            }
        })?;

        let cached_binary_sha256 = crate::hash::format_hex_digest(&sha256_bytes(raw_binary_bytes));

        let meta = CachedPackageMeta {
            package_name: package_name.to_string(),
            package_version: version.to_string(),
            source_archive_sha256,
            source_archive_size_bytes: source_metadata.len(),
            cached_binary_sha256,
        };

        let target_binary = bin_dir.join(format!("{package_name}.slxb"));
        let temp_binary = bin_dir.join(format!("{package_name}.slxb.tmp.{}", std::process::id()));

        // Write binary via atomic rename
        {
            let mut file_handle = File::create(&temp_binary).map_err(|io_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!(
                        "Failed to create temp cache file '{}': {io_error}",
                        temp_binary.display()
                    ),
                }
            })?;
            file_handle
                .write_all(raw_binary_bytes)
                .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                    reason: format!(
                        "Failed to write temp cache file '{}': {io_error}",
                        temp_binary.display()
                    ),
                })?;
            file_handle.flush().map_err(|io_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!(
                        "Failed to flush temp cache file '{}': {io_error}",
                        temp_binary.display()
                    ),
                }
            })?;
        }

        fs::rename(&temp_binary, &target_binary).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Failed to atomically rename cache file to '{}': {io_error}",
                    target_binary.display()
                ),
            }
        })?;

        // Write meta.json via atomic rename
        let meta_target = entry_dir.join("meta.json");
        let meta_temp = entry_dir.join(format!("meta.json.tmp.{}", std::process::id()));

        let meta_json_string = serde_json::to_string_pretty(&meta).map_err(|json_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("Failed to serialize CachedPackageMeta: {json_error}"),
            }
        })?;

        fs::write(&meta_temp, meta_json_string).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Failed to write temp meta file '{}': {io_error}",
                    meta_temp.display()
                ),
            }
        })?;

        fs::rename(&meta_temp, &meta_target).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Failed to atomically rename meta file to '{}': {io_error}",
                    meta_target.display()
                ),
            }
        })?;

        Ok(())
    }
}
