//! Pluggable package extract cache contracts and memory implementations.

#![forbid(unsafe_code)]

use crate::bytecode::BinaryProgram;
use crate::diagnostics::EmissionError;
use crate::hash::sha256_file;
use crate::solver::PackageVersion;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::RwLock;

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
