//! Client module solver and cross-platform package store manager.

#![forbid(unsafe_code)]

pub use solix_common::solver::{
    ClientModuleSolver, DependencyRequirement, InMemoryModuleSolver, ModuleSolver, PackageVersion,
    SolvedPackage, SolverError, resolve_native_cache_dir, resolve_package_store_dir,
};
