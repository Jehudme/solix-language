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

    let colon_debug_requirement = DependencyRequirement::parse("raylib:debug@1.2.0")
        .expect("Parsing colon debug requirement must succeed");
    assert_eq!(colon_debug_requirement.name, "raylib");
    assert_eq!(colon_debug_requirement.profile.as_deref(), Some("debug"));
    assert_eq!(
        colon_debug_requirement.version,
        PackageVersion::new(1, 2, 0)
    );

    let colon_hyphenated_requirement =
        DependencyRequirement::parse("my-cool-package:release@2.0.0")
            .expect("Parsing colon hyphenated package requirement must succeed");
    assert_eq!(colon_hyphenated_requirement.name, "my-cool-package");
    assert_eq!(
        colon_hyphenated_requirement.profile.as_deref(),
        Some("release")
    );
    assert_eq!(
        colon_hyphenated_requirement.version,
        PackageVersion::new(2, 0, 0)
    );

    let colon_after_version_requirement =
        DependencyRequirement::parse("my-cool-package@2.0.0:debug")
            .expect("Parsing colon after version must succeed");
    assert_eq!(colon_after_version_requirement.name, "my-cool-package");
    assert_eq!(
        colon_after_version_requirement.profile.as_deref(),
        Some("debug")
    );
    assert_eq!(
        colon_after_version_requirement.version,
        PackageVersion::new(2, 0, 0)
    );

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
    let manifest_yaml = r#"
vars:
  shared_deps: &release_deps
    raylib: { version: "1.2.0", profile: release }

configuration:
  name: game_project
  version: "0.1.0"

profiles:
  release:
    dependencies: *release_deps

  debug:
    dependencies:
      <<: *release_deps
      math_utils: { version: "2.0.0", profile: debug }
"#;

    let project_manifest =
        ProjectManifest::from_yaml_str(manifest_yaml).expect("Parsing manifest.yaml must succeed");
    assert_eq!(project_manifest.name(), "game_project");

    let release_deps = project_manifest
        .resolved_dependencies_for_profile("release")
        .expect("resolve release deps");
    assert_eq!(release_deps.len(), 1);
    assert_eq!(release_deps[0].package_name, "raylib");

    let debug_deps = project_manifest
        .resolved_dependencies_for_profile("debug")
        .expect("resolve debug deps");
    assert_eq!(debug_deps.len(), 2);
    assert!(debug_deps.iter().any(|dep| dep.package_name == "raylib"));
    assert!(
        debug_deps
            .iter()
            .any(|dep| dep.package_name == "math_utils")
    );
}

#[test]
fn test_project_manifest_build_config_parsing() {
    let manifest_yaml = r#"
configuration:
  name: breakout
  version: "0.1.0"

profiles:
  release:
    entrypoint: Breakout::main
    optimization_level: 3
    inline_threshold: 12
    static: true
    verify_checksum: false
    sources: [src]
"#;

    let project_manifest =
        ProjectManifest::from_yaml_str(manifest_yaml).expect("Parsing manifest.yaml must succeed");
    let resolved_profile = project_manifest
        .resolve_profile("release")
        .expect("resolve profile");
    assert_eq!(resolved_profile.opt_level, 3);
    assert_eq!(resolved_profile.inline_threshold, 12);
    assert!(resolved_profile.is_static);
    assert!(!resolved_profile.verify_checksum);
    assert_eq!(resolved_profile.sources, vec!["src"]);
}

#[test]
fn test_independent_profiles_with_yaml_anchors() {
    let manifest_yaml = r#"
vars:
  base_entry: &main_entry "Main::main"
  base_sources: &base_src [src]

configuration:
  name: engine_demo
  version: "1.0.0"

profiles:
  release:
    entrypoint: *main_entry
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    sources: *base_src
    platforms: [all]

  bench:
    entrypoint: *main_entry
    optimization_level: 3
    inline_threshold: 32
    memory_limit_mb: 1024
    sources: *base_src

  ci:
    entrypoint: *main_entry
    optimization_level: 0
    warnings_as_errors: true
    sources: [src, ci_tests]
"#;

    let project_manifest =
        ProjectManifest::from_yaml_str(manifest_yaml).expect("Parsing manifest.yaml must succeed");

    // 1. Resolve release profile
    let release_profile = project_manifest
        .resolve_profile("release")
        .expect("Release profile resolution must succeed");
    assert_eq!(release_profile.name, "release");
    assert_eq!(release_profile.opt_level, 3);
    assert_eq!(release_profile.inline_threshold, 16);
    assert!(!release_profile.debug_symbols);
    assert_eq!(release_profile.sources, vec!["src"]);

    // 2. Resolve bench profile independently (does not leak warnings_as_errors or ci_tests)
    let bench_profile = project_manifest
        .resolve_profile("bench")
        .expect("Bench profile resolution must succeed");
    assert_eq!(bench_profile.name, "bench");
    assert_eq!(bench_profile.opt_level, 3);
    assert_eq!(bench_profile.inline_threshold, 32);
    assert_eq!(bench_profile.memory_limit_mb, Some(1024));
    assert_eq!(bench_profile.sources, vec!["src"]);

    // 3. Resolve ci profile independently
    let ci_profile = project_manifest
        .resolve_profile("ci")
        .expect("CI profile resolution must succeed");
    assert_eq!(ci_profile.name, "ci");
    assert_eq!(ci_profile.opt_level, 0);
    assert!(ci_profile.warnings_as_errors);
    assert_eq!(ci_profile.sources, vec!["src", "ci_tests"]);
}

#[test]
fn test_inherits_keyword_rejected_in_yaml() {
    let invalid_yaml = r#"
configuration:
  name: broken_project
  version: "0.1.0"

profiles:
  release: {}
  debug:
    inherits: release
"#;

    let parse_result = ProjectManifest::from_yaml_str(invalid_yaml);
    assert!(
        parse_result.is_err(),
        "Profile inheritance via inherits MUST be rejected in manifest.yaml"
    );
}

#[test]
fn test_profile_aware_dependency_specifications() {
    let manifest_yaml = r#"
configuration:
  name: full_stack_app
  version: "2.0.0"

profiles:
  release:
    dependencies:
      math: { version: "1.0.0", profile: release }
      solixlib: { version: "2.0.0" }

  debug:
    dependencies:
      raylib: { version: "5.5.0", profile: debug }

  test:
    dependencies:
      math: { version: "1.0.0", profile: release }
      solixlib: { version: "2.0.0" }
      raylib: { version: "5.5.0", profile: debug }
      crypto: { version: "0.4.0", profile: test }
"#;

    let project_manifest =
        ProjectManifest::from_yaml_str(manifest_yaml).expect("Parsing manifest.yaml must succeed");
    let debug_dependencies = project_manifest
        .resolved_dependencies_for_profile("debug")
        .expect("resolve debug dependencies");
    assert_eq!(
        debug_dependencies.len(),
        1,
        "debug profile must NOT inherit release dependencies"
    );

    let resolved_dependencies = project_manifest
        .resolved_dependencies_for_profile("test")
        .expect("resolve test dependencies");
    assert_eq!(resolved_dependencies.len(), 4);

    let math_dep = resolved_dependencies
        .iter()
        .find(|dependency| dependency.package_name == "math")
        .expect("math dependency must exist");
    assert_eq!(math_dep.version, "1.0.0");
    assert_eq!(math_dep.build_profile, "release");

    let raylib_dep = resolved_dependencies
        .iter()
        .find(|dependency| dependency.package_name == "raylib")
        .expect("raylib dependency must exist");
    assert_eq!(raylib_dep.version, "5.5.0");
    assert_eq!(raylib_dep.build_profile, "debug");

    let solixlib_dep = resolved_dependencies
        .iter()
        .find(|dependency| dependency.package_name == "solixlib")
        .expect("solixlib dependency must exist");
    assert_eq!(solixlib_dep.version, "2.0.0");
    assert_eq!(solixlib_dep.build_profile, "release");

    let crypto_dep = resolved_dependencies
        .iter()
        .find(|dependency| dependency.package_name == "crypto")
        .expect("crypto dependency must exist");
    assert_eq!(crypto_dep.version, "0.4.0");
    assert_eq!(crypto_dep.build_profile, "test");
}

#[test]
fn test_legacy_sections_rejected() {
    let legacy_info_toml = r#"
[info]
name = "legacy_project"
version = "1.0.0"

[dependencies]
raylib = "5.5.0"
"#;
    let parse_result = ProjectManifest::from_toml_str(legacy_info_toml);
    assert!(
        parse_result.is_err(),
        "Legacy sections [info] and root [dependencies] must be rejected"
    );
}

#[test]
fn test_missing_release_profile_rejected() {
    let no_release_toml = r#"
[configuration]
name = "no_release"
version = "1.0.0"

[profiles.debug]
optimization_level = 0
"#;
    let parse_result = ProjectManifest::from_toml_str(no_release_toml);
    assert!(
        parse_result.is_err(),
        "Manifest without mandatory [profiles.release] profile must be rejected"
    );
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

#[test]
fn test_unknown_profile_rejected() {
    let toml = r#"
[configuration]
name = "audit_app"
version = "1.0.0"

[profiles.release]
entrypoint = "Main::main"
"#;
    let manifest = ProjectManifest::from_toml_str(toml).expect("parse manifest");
    let unknown_result = manifest.resolve_profile("nonexistent");
    assert!(unknown_result.is_err());
    let err_msg = format!("{:?}", unknown_result.err().unwrap());
    assert!(err_msg.contains("Unknown build profile 'nonexistent'"));
}

#[test]
fn test_positive_native_package_cache_hit_and_parallel_safety() {
    use solix_common::project::SlxarcArchive;
    use solix_common::solver::ClientModuleSolver;

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let stage_dir = temp_dir.path().join("stage");
    let bin_dir = stage_dir.join("bin");
    let lib_dir = stage_dir.join("lib");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    std::fs::create_dir_all(&lib_dir).expect("create lib dir");

    let manifest_content = r#"
[configuration]
name = "nativepkg"
version = "1.0.0"

[profiles.release]
"#;
    std::fs::write(stage_dir.join("manifest.toml"), manifest_content).expect("write manifest");

    #[cfg(target_os = "windows")]
    let host_lib_name = "libnativepkg.dll";
    #[cfg(target_os = "macos")]
    let host_lib_name = "libnativepkg.dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let host_lib_name = "libnativepkg.so";

    let lib_bytes = b"simulated_native_library_payload_for_atomic_cache_test";
    let lib_path = lib_dir.join(host_lib_name);
    std::fs::write(&lib_path, lib_bytes).expect("write dummy native lib");
    let expected_hash = solix_common::hash::sha256_hex(lib_bytes);

    let mut program = BinaryProgram::new();
    program
        .native_library_checksums
        .insert(format!("lib/{host_lib_name}"), expected_hash.clone());
    std::fs::write(bin_dir.join("nativepkg.slxb"), program.to_bytes()).expect("write binary");

    let store_dir = temp_dir.path().join("packages");
    let pkg_dir = store_dir.join("nativepkg-release@1.0.0");
    std::fs::create_dir_all(&pkg_dir).expect("create pkg dir");
    let archive_path = pkg_dir.join("nativepkg.slxarc");
    SlxarcArchive::pack_directory(&stage_dir, &archive_path).expect("pack archive");

    // Resolve across parallel threads against the shared store and native cache
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let store_clone = store_dir.clone();
            std::thread::spawn(move || {
                let solver = ClientModuleSolver::with_store_dir(store_clone);
                solver
                    .resolve_compatible("nativepkg", PackageVersion::new(1, 0, 0))
                    .expect("parallel native package resolution must succeed")
            })
        })
        .collect();

    for handle in handles {
        let resolved = handle.join().expect("thread join");
        assert_eq!(resolved.native_libraries.len(), 1);
        assert!(resolved.native_libraries[0].exists());
    }

    // Subsequent resolution hits authenticated cache with preserved native library
    let solver = ClientModuleSolver::with_store_dir(store_dir);
    let cached = solver
        .resolve_compatible("nativepkg", PackageVersion::new(1, 0, 0))
        .expect("cache-hit resolution must succeed");
    assert_eq!(cached.native_libraries.len(), 1);
    assert_eq!(
        solix_common::hash::sha256_file(&cached.native_libraries[0]).expect("hash cached lib"),
        expected_hash
    );
}
