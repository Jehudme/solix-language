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
    assert_eq!(requirement.profile, None);
    assert_eq!(requirement.version, PackageVersion::new(1, 2, 0));
    assert_eq!(requirement.to_string(), "raylib@1.2.0");

    let debug_requirement = DependencyRequirement::parse("raylib-debug@1.2.0")
        .expect("Parsing debug requirement must succeed");
    assert_eq!(debug_requirement.name, "raylib");
    assert_eq!(debug_requirement.profile.as_deref(), Some("debug"));
    assert_eq!(debug_requirement.version, PackageVersion::new(1, 2, 0));
    assert_eq!(debug_requirement.to_string(), "raylib-debug@1.2.0");

    let release_requirement = DependencyRequirement::parse("raylib-release@5.5.0")
        .expect("Parsing release requirement must succeed");
    assert_eq!(release_requirement.name, "raylib");
    assert_eq!(release_requirement.profile.as_deref(), Some("release"));
    assert_eq!(release_requirement.version, PackageVersion::new(5, 5, 0));

    let hyphenated_requirement = DependencyRequirement::parse("my-cool-package@2.0.0")
        .expect("Hyphenated package without profile must succeed");
    assert_eq!(hyphenated_requirement.name, "my-cool-package");
    assert_eq!(hyphenated_requirement.profile, None);
    assert_eq!(hyphenated_requirement.version, PackageVersion::new(2, 0, 0));

    let hyphenated_debug_requirement = DependencyRequirement::parse("my-cool-package-debug@2.0.0")
        .expect("Hyphenated package with debug profile must succeed");
    assert_eq!(hyphenated_debug_requirement.name, "my-cool-package");
    assert_eq!(
        hyphenated_debug_requirement.profile.as_deref(),
        Some("debug")
    );
    assert_eq!(
        hyphenated_debug_requirement.version,
        PackageVersion::new(2, 0, 0)
    );

    let hash_profile_requirement = DependencyRequirement::parse("raylib@1.2.0#release")
        .expect("Parsing #profile must succeed");
    assert_eq!(hash_profile_requirement.name, "raylib");
    assert_eq!(hash_profile_requirement.profile.as_deref(), Some("release"));

    assert!(DependencyRequirement::parse("raylib").is_err());
    assert!(DependencyRequirement::parse("@1.2.0").is_err());
    assert!(DependencyRequirement::parse("-debug@1.2.0").is_err());
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
            .map(|dep| dep.version()),
        Some("1.2.0")
    );
    assert_eq!(
        project_manifest
            .dependencies
            .get("math_utils")
            .map(|dep| dep.version()),
        Some("2.0.0")
    );
}

#[test]
fn test_project_manifest_build_config_parsing() {
    let manifest_toml = r#"
[info]
name = "breakout"
version = "0.1.0"

[build]
profile = "release"
link = "static"
pkg_dir = "/custom/pkg"
install = true
force_install = true
verify_checksum = false
inline_threshold = 12
"#;

    let project_manifest =
        ProjectManifest::from_toml_str(manifest_toml).expect("Parsing manifest.toml must succeed");
    assert_eq!(project_manifest.build.profile, "release");
    assert_eq!(project_manifest.build.link, "static");
    assert_eq!(project_manifest.build.pkg_dir, "/custom/pkg");
    assert!(project_manifest.build.install);
    assert!(project_manifest.build.force_install);
    assert!(!project_manifest.build.verify_checksum);
    assert_eq!(project_manifest.build.inline_threshold, 12);
}

#[test]
fn test_hierarchical_profile_inheritance_and_overrides() {
    let manifest_toml = r#"
[info]
name = "engine_demo"
version = "1.0.0"

[profile.default]
warnings_as_errors = false
inline_threshold = 10

[profile.release]
opt_level = 3
inline_threshold = 16

[profile.bench]
inherits = "release"
opt_level = 3
inline_threshold = 32
memory_limit_mb = 1024

[profile.ci]
inherits = "bench"
warnings_as_errors = true
"#;

    let project_manifest =
        ProjectManifest::from_toml_str(manifest_toml).expect("Parsing manifest.toml must succeed");

    // 1. Resolve default profile
    let default_profile = project_manifest
        .resolve_profile("default")
        .expect("Default profile resolution must succeed");
    assert_eq!(default_profile.name, "default");
    assert_eq!(default_profile.opt_level, 0);
    assert_eq!(default_profile.inline_threshold, 10);
    assert!(!default_profile.warnings_as_errors);

    // 2. Resolve release profile (inherits from default, overrides opt_level and inline_threshold)
    let release_profile = project_manifest
        .resolve_profile("release")
        .expect("Release profile resolution must succeed");
    assert_eq!(release_profile.name, "release");
    assert_eq!(release_profile.opt_level, 3);
    assert_eq!(release_profile.inline_threshold, 16);
    assert!(!release_profile.debug_symbols);

    // 3. Resolve bench profile (inherits from release -> default)
    let bench_profile = project_manifest
        .resolve_profile("bench")
        .expect("Bench profile resolution must succeed");
    assert_eq!(bench_profile.name, "bench");
    assert_eq!(bench_profile.opt_level, 3);
    assert_eq!(bench_profile.inline_threshold, 32);
    assert_eq!(bench_profile.memory_limit_mb, Some(1024));
    assert!(!bench_profile.debug_symbols); // inherited from release

    // 4. Resolve ci profile (inherits from bench -> release -> default)
    let ci_profile = project_manifest
        .resolve_profile("ci")
        .expect("CI profile resolution must succeed");
    assert_eq!(ci_profile.name, "ci");
    assert_eq!(ci_profile.opt_level, 3);
    assert_eq!(ci_profile.inline_threshold, 32); // inherited from bench
    assert!(ci_profile.warnings_as_errors); // overridden by ci
    assert_eq!(ci_profile.memory_limit_mb, Some(1024)); // inherited from bench
}

#[test]
fn test_cyclic_profile_inheritance_detection() {
    let cyclic_manifest_toml = r#"
[info]
name = "broken_project"
version = "0.1.0"

[profile.alpha]
inherits = "beta"

[profile.beta]
inherits = "gamma"

[profile.gamma]
inherits = "alpha"
"#;

    let project_manifest = ProjectManifest::from_toml_str(cyclic_manifest_toml)
        .expect("Parsing manifest syntax must succeed");
    let resolution_result = project_manifest.resolve_profile("alpha");
    assert!(
        resolution_result.is_err(),
        "Cyclic profile inheritance must be detected and rejected"
    );
}

#[test]
fn test_profile_aware_dependency_specifications() {
    let manifest_toml = r#"
[info]
name = "full_stack_app"
version = "2.0.0"

[dependencies]
math = "1.0.0"
raylib-debug = "5.5.0"
solixlib = { version = "2.0.0", profile = "release" }
crypto-test = "0.4.0"
"#;

    let project_manifest =
        ProjectManifest::from_toml_str(manifest_toml).expect("Parsing manifest.toml must succeed");
    let resolved_dependencies = project_manifest.resolved_dependencies();
    assert_eq!(resolved_dependencies.len(), 4);

    let math_dep = resolved_dependencies
        .iter()
        .find(|dep| dep.package_name == "math")
        .expect("math dependency must exist");
    assert_eq!(math_dep.version, "1.0.0");
    assert_eq!(math_dep.build_profile, "release"); // defaults to release

    let raylib_dep = resolved_dependencies
        .iter()
        .find(|dep| dep.package_name == "raylib")
        .expect("raylib dependency must exist");
    assert_eq!(raylib_dep.version, "5.5.0");
    assert_eq!(raylib_dep.build_profile, "debug"); // extracted from -debug suffix

    let solixlib_dep = resolved_dependencies
        .iter()
        .find(|dep| dep.package_name == "solixlib")
        .expect("solixlib dependency must exist");
    assert_eq!(solixlib_dep.version, "2.0.0");
    assert_eq!(solixlib_dep.build_profile, "release"); // explicit table profile

    let crypto_dep = resolved_dependencies
        .iter()
        .find(|dep| dep.package_name == "crypto")
        .expect("crypto dependency must exist");
    assert_eq!(crypto_dep.version, "0.4.0");
    assert_eq!(crypto_dep.build_profile, "test"); // extracted from -test suffix
}

#[test]
fn test_client_module_solver_profile_fallback() {
    let temp_dir = std::env::temp_dir().join(format!("solix_test_solver_{}", std::process::id()));
    let store_dir = temp_dir.join("packages");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&store_dir).expect("create store dir");

    // 1. Create raylib-debug@5.0.0 and raylib-release@5.5.0 in store
    let debug_dir = store_dir.join("raylib-debug@5.0.0");
    std::fs::create_dir_all(&debug_dir).expect("create debug dir");
    let dummy_program = BinaryProgram {
        package_name: "raylib".to_string(),
        package_version: "5.0.0".to_string(),
        build_profile: "debug".to_string(),
        ..Default::default()
    };
    std::fs::write(debug_dir.join("raylib.slxb"), dummy_program.to_bytes())
        .expect("write debug binary");

    let release_dir = store_dir.join("raylib-release@5.5.0");
    std::fs::create_dir_all(&release_dir).expect("create release dir");
    let release_program = BinaryProgram {
        package_name: "raylib".to_string(),
        package_version: "5.5.0".to_string(),
        build_profile: "release".to_string(),
        ..Default::default()
    };
    std::fs::write(release_dir.join("raylib.slxb"), release_program.to_bytes())
        .expect("write release binary");

    let solver = solix_common::solver::ClientModuleSolver::with_store_dir(store_dir);

    // Resolving without profile must choose release 5.5.0
    let resolved_default = solver
        .resolve_compatible("raylib", PackageVersion::new(5, 0, 0))
        .expect("must resolve release");
    assert_eq!(resolved_default.version, PackageVersion::new(5, 5, 0));

    // Resolving explicitly for debug must choose debug 5.0.0
    let resolved_debug = solver
        .resolve_compatible("raylib-debug", PackageVersion::new(5, 0, 0))
        .expect("must resolve debug");
    assert_eq!(resolved_debug.version, PackageVersion::new(5, 0, 0));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
