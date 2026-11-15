//! Exhaustive test suite for semantic versioning, dependency requirements,
//! manifest serialization, and the module solver engine.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, ExportManifest};
use solix_common::project::ProjectManifest;
use solix_common::solver::{
    DependencyRequirement, InMemoryModuleSolver, ModuleSolver, PackageVersion, SolvedPackage,
    SolverError,
};

// ============================================================================
// 1. PackageVersion Parsing & Display Tests
// ============================================================================

#[test]
fn test_package_version_parsing_valid() {
    let standard_version =
        PackageVersion::parse("1.2.3").expect("Parsing standard version must succeed");
    assert_eq!(standard_version.major, 1);
    assert_eq!(standard_version.medium, 2);
    assert_eq!(standard_version.minor, 3);
    assert_eq!(standard_version.to_string(), "1.2.3");

    let prefixed_version =
        PackageVersion::parse("v2.10.5").expect("Parsing v-prefixed version must succeed");
    assert_eq!(prefixed_version.major, 2);
    assert_eq!(prefixed_version.medium, 10);
    assert_eq!(prefixed_version.minor, 5);

    let zero_version = PackageVersion::parse("0.0.0").expect("Parsing zero version must succeed");
    assert_eq!(zero_version.major, 0);
    assert_eq!(zero_version.medium, 0);
    assert_eq!(zero_version.minor, 0);
}

#[test]
fn test_package_version_parsing_invalid() {
    assert!(PackageVersion::parse("1.2").is_err());
    assert!(PackageVersion::parse("1").is_err());
    assert!(PackageVersion::parse("1.2.3.4").is_err());
    assert!(PackageVersion::parse("a.b.c").is_err());
    assert!(PackageVersion::parse("1.-2.3").is_err());
    assert!(PackageVersion::parse("").is_err());
}

// ============================================================================
// 2. SemVer Compatibility Law Tests
// ============================================================================

#[test]
fn test_semver_compatibility_law() {
    let baseline_required = PackageVersion::new(1, 2, 0);

    // Exact match is compatible
    assert!(PackageVersion::new(1, 2, 0).is_compatible_with(&baseline_required));

    // Same major, same medium, higher minor is compatible
    assert!(PackageVersion::new(1, 2, 5).is_compatible_with(&baseline_required));

    // Same major, same medium, lower minor is incompatible
    let strict_required = PackageVersion::new(1, 2, 5);
    assert!(!PackageVersion::new(1, 2, 1).is_compatible_with(&strict_required));

    // Same major, strictly higher medium allows any minor (even lower than required!)
    let high_minor_required = PackageVersion::new(1, 2, 9);
    let higher_medium_candidate = PackageVersion::new(1, 4, 0);
    assert!(higher_medium_candidate.is_compatible_with(&high_minor_required));

    // Same major, lower medium is incompatible
    assert!(!PackageVersion::new(1, 1, 9).is_compatible_with(&baseline_required));

    // Different major is always incompatible (breaking change)
    assert!(!PackageVersion::new(2, 0, 0).is_compatible_with(&baseline_required));
    assert!(!PackageVersion::new(0, 2, 0).is_compatible_with(&baseline_required));
}

#[test]
fn test_package_version_ordering() {
    let version_one = PackageVersion::new(1, 0, 0);
    let version_two = PackageVersion::new(1, 2, 0);
    let version_three = PackageVersion::new(1, 2, 5);
    let version_four = PackageVersion::new(2, 0, 0);

    assert!(version_one < version_two);
    assert!(version_two < version_three);
    assert!(version_three < version_four);
}

// ============================================================================
// 3. DependencyRequirement Parsing & Display Tests
// ============================================================================

#[test]
fn test_dependency_requirement_parsing() {
    let requirement =
        DependencyRequirement::parse("raylib@1.2.0").expect("Parsing requirement must succeed");
    assert_eq!(requirement.name, "raylib");
    assert_eq!(requirement.version, PackageVersion::new(1, 2, 0));
    assert_eq!(requirement.to_string(), "raylib@1.2.0");

    assert!(DependencyRequirement::parse("raylib").is_err());
    assert!(DependencyRequirement::parse("@1.2.0").is_err());
    assert!(DependencyRequirement::parse("raylib@1.2").is_err());
}

// ============================================================================
// 4. InMemoryModuleSolver Resolution Tests
// ============================================================================

#[test]
fn test_in_memory_module_solver_resolves_highest_compatible() {
    let dummy_program = BinaryProgram::default();

    let package_v100 = SolvedPackage::new(
        "engine",
        PackageVersion::new(1, 0, 0),
        dummy_program.clone(),
    );
    let package_v120 = SolvedPackage::new(
        "engine",
        PackageVersion::new(1, 2, 0),
        dummy_program.clone(),
    );
    let package_v125 = SolvedPackage::new(
        "engine",
        PackageVersion::new(1, 2, 5),
        dummy_program.clone(),
    );
    let package_v200 = SolvedPackage::new(
        "engine",
        PackageVersion::new(2, 0, 0),
        dummy_program.clone(),
    );

    let solver = InMemoryModuleSolver::new()
        .with_package(package_v100)
        .with_package(package_v120)
        .with_package(package_v125)
        .with_package(package_v200);

    // Requesting minimum 1.2.0 should select highest compatible version (1.2.5), not 2.0.0
    let resolved_compatible = solver
        .resolve_compatible("engine", PackageVersion::new(1, 2, 0))
        .expect("Compatible resolution must succeed");
    assert_eq!(resolved_compatible.version, PackageVersion::new(1, 2, 5));

    // Requesting latest should select absolute highest version (2.0.0)
    let resolved_latest = solver
        .resolve_latest("engine")
        .expect("Latest resolution must succeed");
    assert_eq!(resolved_latest.version, PackageVersion::new(2, 0, 0));

    // Requesting incompatible version should return NoCompatibleVersion error
    let incompatible_result = solver.resolve_compatible("engine", PackageVersion::new(3, 0, 0));
    assert!(matches!(
        incompatible_result,
        Err(SolverError::NoCompatibleVersion { .. })
    ));

    // Requesting missing package should return PackageNotFound error
    let missing_result = solver.resolve_compatible("non_existent", PackageVersion::new(1, 0, 0));
    assert!(matches!(
        missing_result,
        Err(SolverError::PackageNotFound { .. })
    ));
}

// ============================================================================
// 5. Manifest Serialization with Dependencies Tests
// ============================================================================

#[test]
fn test_export_manifest_serialization_with_dependencies() {
    let manifest = ExportManifest {
        package_name: "test_pkg".to_string(),
        package_version: "1.0.0".to_string(),
        package_description: "A test package".to_string(),
        dependencies: vec![
            DependencyRequirement::new("math", PackageVersion::new(2, 1, 0)),
            DependencyRequirement::new("logging", PackageVersion::new(0, 4, 2)),
        ],
        ..Default::default()
    };

    let serialized_json =
        serde_json::to_string_pretty(&manifest).expect("Manifest serialization must succeed");
    let deserialized_manifest: ExportManifest =
        serde_json::from_str(&serialized_json).expect("Manifest deserialization must succeed");

    assert_eq!(manifest.dependencies, deserialized_manifest.dependencies);
    assert_eq!(deserialized_manifest.dependencies.len(), 2);
    assert_eq!(deserialized_manifest.dependencies[0].name, "math");
    assert_eq!(
        deserialized_manifest.dependencies[0].version,
        PackageVersion::new(2, 1, 0)
    );
}

#[test]
fn test_project_manifest_dependencies_parsing() {
    let manifest_toml = r#"
[info]
name = "game_project"
version = "0.5.0"

[dependencies]
raylib = "1.2.0"
math_utils = "2.0.0"
"#;

    let project_manifest =
        ProjectManifest::from_toml_str(manifest_toml).expect("Parsing manifest.toml must succeed");
    assert_eq!(project_manifest.info.name, "game_project");
    assert_eq!(project_manifest.dependencies.len(), 2);
    assert_eq!(
        project_manifest
            .dependencies
            .get("raylib")
            .map(String::as_str),
        Some("1.2.0")
    );
    assert_eq!(
        project_manifest
            .dependencies
            .get("math_utils")
            .map(String::as_str),
        Some("2.0.0")
    );
}
