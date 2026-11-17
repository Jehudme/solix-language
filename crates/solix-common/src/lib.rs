//! Solix common types, source coordinate spans, and shared diagnostic error models.

#![forbid(unsafe_code)]

pub mod bytecode;
pub mod cache;
pub mod diagnostics;
pub mod hash;
pub mod project;
pub mod solver;
pub mod span;
pub mod tokens;
pub mod types;

pub use bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ConstantPool, ConstantPoolValue,
    DeclarationModifier, ExceptionHandlerEntry, ExportManifest, ExportedClass, ExportedField,
    ExportedFunction, ExportedGenericTemplate, ExportedParameter, ExportedStatic, Instruction,
    OpCode, SourceMapEntry, extract_api_manifest,
};
pub use cache::{
    CachedPackageMeta, FilesystemPackageCache, InMemoryPackageCache, NoopPackageCache,
    PackageCache, resolve_packages_cache_dir,
};
pub use diagnostics::EmissionError;
pub use hash::{Sha256Hasher, format_hex_digest, sha256_bytes, sha256_file, sha256_hex};
pub use project::{
    DependencySpec, LoadedSlxarc, ProfileConfig, ProjectConfiguration, ProjectInfo,
    ProjectManifest, ResolvedDependency, ResolvedProfile, SlxarcArchive, SlxarcFileEntry,
    SlxarcInspection, canonicalize_host_path, matches_native_target_subdir,
    native_checksum_key_applies_to_host, resolve_expected_native_checksum,
    strip_windows_verbatim_prefix,
};
pub use solver::{
    ClientModuleSolver, DependencyRequirement, InMemoryModuleSolver, ModuleSolver, PackageVersion,
    SolvedPackage, SolverError, resolve_native_cache_dir, resolve_package_store_dir,
};
pub use span::Span;
pub use tokens::{KEYWORDS, Token, TokenKind, lookup_keyword};
pub use types::{PrimitiveKind, Type};
