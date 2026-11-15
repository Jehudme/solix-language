//! Persistent filesystem extraction cache for authenticated package binaries.

#![forbid(unsafe_code)]

pub use solix_common::cache::{
    CachedPackageMeta, FilesystemPackageCache, InMemoryPackageCache, NoopPackageCache, PackageCache,
    resolve_packages_cache_dir,
};
