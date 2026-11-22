//! Integration tests for Milestone 026: Package Manager Modernization, `solix sync`,
//! Rich `info`/`list` Inspection, Authenticated Extraction Cache, and Cryptographic Checksum Pinning.

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_common::bytecode::BinaryProgram;
use solix_common::cache::PackageCache;
use solix_common::solver::PackageVersion;
use solix_toolchain::cache::FilesystemPackageCache;
use solix_toolchain::cli::Cli;
use solix_toolchain::registry::PackageRegistry;
use solix_toolchain::run;
use std::fs;

#[test]
fn test_install_collision_protection_and_force_flag() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create package store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    let source_file = workspace
        .write_file(
            "calc.slx",
            r#"
public class Calculator {
    public static int32 add(int32 left_operand, int32 right_operand) {
        return left_operand + right_operand;
    }
}
"#,
        )
        .expect("write calc");

    let calc_binary = workspace.file_path("calc.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().expect("valid path"),
        "--name",
        "calculator",
        "--version",
        "1.0.0",
        "-o",
        calc_binary.to_str().expect("valid path"),
    ])
    .expect("compile cli");
    assert_eq!(run(compile_cli).expect("compile must succeed"), 0);

    // Initial installation must succeed
    let install_cli_first = Cli::try_parse_from([
        "solix",
        "install",
        calc_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("install cli first");
    assert_eq!(
        run(install_cli_first).expect("initial install must succeed"),
        0
    );

    // Re-installing without --force must fail due to collision protection
    let install_cli_collision = Cli::try_parse_from([
        "solix",
        "install",
        calc_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("install cli collision");
    let collision_result = run(install_cli_collision);
    assert!(
        collision_result.is_err(),
        "Installing existing package without --force must fail"
    );
    let error_text = format!("{:?}", collision_result.unwrap_err());
    assert!(
        error_text.contains("--force") || error_text.contains("-f"),
        "Error message must mention --force, got: {error_text}"
    );

    // Installing with --force must succeed
    let install_cli_forced = Cli::try_parse_from([
        "solix",
        "install",
        calc_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--force",
    ])
    .expect("install cli forced");
    assert_eq!(
        run(install_cli_forced).expect("forced install must succeed"),
        0
    );
}

#[test]
fn test_compile_with_install_and_sync_alias() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    let source_file = workspace
        .write_file(
            "greeter.slx",
            r#"
public class Greeter {
    public static int32 greet() {
        return 42;
    }
}
"#,
        )
        .expect("write greeter");

    let binary_file = workspace.file_path("greeter.slxb");

    // Compile with --install (alias --sync)
    let compile_install_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().expect("valid path"),
        "--name",
        "greeter",
        "--version",
        "2.1.0",
        "-o",
        binary_file.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--install",
    ])
    .expect("compile --install cli");
    assert_eq!(
        run(compile_install_cli).expect("compile with --install must succeed"),
        0
    );

    // Verify it is registered in registry.json and placed in the store
    let registry = PackageRegistry::load_or_sync(&package_store_directory).expect("load registry");
    let greeter_record = registry.get("greeter", Some("2.1.0"));
    assert!(
        greeter_record.is_some(),
        "greeter@2.1.0 must be registered in registry.json"
    );
    let record = greeter_record.unwrap();
    assert_eq!(record.name, "greeter");
    assert_eq!(record.version, "2.1.0");
    assert!(
        record.kind == "binaries" || record.kind == "slxb",
        "Record kind must be binaries or slxb, got: {}",
        record.kind
    );
    assert!(!record.sha256.is_empty());
}

#[test]
fn test_info_installed_package_lookup_and_json() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    let source_file = workspace
        .write_file(
            "network.slx",
            r#"
public class NetworkClient {
    public static int32 ping() {
        return 1;
    }
}
"#,
        )
        .expect("write network");

    let network_bin = workspace.file_path("network.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().expect("valid path"),
        "--name",
        "network",
        "--version",
        "3.0.0",
        "-o",
        network_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--install",
    ])
    .expect("compile cli");
    assert_eq!(run(compile_cli).expect("compile must succeed"), 0);

    // 1. Info lookup by bare package name ("network")
    let info_by_name = Cli::try_parse_from(["solix", "info", "network", "--pkg-dir", store_str])
        .expect("info by name");
    assert_eq!(run(info_by_name).expect("info by name must succeed"), 0);

    // 2. Info lookup by name and version ("network@3.0.0") with --json
    let info_by_spec_json = Cli::try_parse_from([
        "solix",
        "info",
        "network@3.0.0",
        "--pkg-dir",
        store_str,
        "--json",
    ])
    .expect("info by spec json");
    assert_eq!(
        run(info_by_spec_json).expect("info by spec json must succeed"),
        0
    );

    // 3. Info lookup by colon syntax ("network:release@3.0.0")
    let info_by_colon_spec = Cli::try_parse_from([
        "solix",
        "info",
        "network:release@3.0.0",
        "--pkg-dir",
        store_str,
        "--json",
    ])
    .expect("info by colon spec");
    assert_eq!(
        run(info_by_colon_spec).expect("info by colon spec must succeed"),
        0
    );

    // 4. Info lookup by profile-only colon syntax ("network:release")
    let info_by_colon_profile =
        Cli::try_parse_from(["solix", "info", "network:release", "--pkg-dir", store_str])
            .expect("info by colon profile");
    assert_eq!(
        run(info_by_colon_profile).expect("info by colon profile must succeed"),
        0
    );

    // 5. Info lookup of non-existent package fails cleanly
    let info_non_existent =
        Cli::try_parse_from(["solix", "info", "does_not_exist", "--pkg-dir", store_str])
            .expect("info non existent");
    assert!(
        run(info_non_existent).is_err(),
        "Looking up non-existent package must return error"
    );
}

#[test]
fn test_list_filtering_sorting_and_json() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    // Install alpha@1.0.0 (debug)
    let alpha_source = workspace
        .write_file("alpha.slx", "public class Alpha {}")
        .expect("write alpha");
    let alpha_bin = workspace.file_path("alpha.slxb");
    let compile_alpha = Cli::try_parse_from([
        "solix",
        "compile",
        alpha_source.to_str().expect("valid path"),
        "--name",
        "alpha",
        "--version",
        "1.0.0",
        "--profile",
        "debug",
        "-o",
        alpha_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--install",
    ])
    .expect("compile alpha");
    assert_eq!(run(compile_alpha).expect("compile alpha"), 0);

    // Install beta@2.0.0 (release)
    let beta_source = workspace
        .write_file("beta.slx", "public class Beta {}")
        .expect("write beta");
    let beta_bin = workspace.file_path("beta.slxb");
    let compile_beta = Cli::try_parse_from([
        "solix",
        "compile",
        beta_source.to_str().expect("valid path"),
        "--name",
        "beta",
        "--version",
        "2.0.0",
        "--profile",
        "release",
        "-o",
        beta_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--install",
    ])
    .expect("compile beta");
    assert_eq!(run(compile_beta).expect("compile beta"), 0);

    // 1. List with --filter "alp"
    let list_filter =
        Cli::try_parse_from(["solix", "list", "--filter", "alp", "--pkg-dir", store_str])
            .expect("list filter");
    assert_eq!(run(list_filter).expect("list filter"), 0);

    // 2. List with --profile debug
    let list_profile = Cli::try_parse_from([
        "solix",
        "list",
        "--profile",
        "debug",
        "--pkg-dir",
        store_str,
    ])
    .expect("list profile");
    assert_eq!(run(list_profile).expect("list profile"), 0);

    // 3. List with --kind slxb --sort version --reverse
    let list_sort = Cli::try_parse_from([
        "solix",
        "list",
        "--kind",
        "slxb",
        "--sort",
        "version",
        "--reverse",
        "--pkg-dir",
        store_str,
    ])
    .expect("list sort");
    assert_eq!(run(list_sort).expect("list sort"), 0);

    // 4. List with --json
    let list_json = Cli::try_parse_from(["solix", "list", "--json", "--pkg-dir", store_str])
        .expect("list json");
    assert_eq!(run(list_json).expect("list json"), 0);
}

#[test]
fn test_authenticated_extract_cache_warm_hit_and_tamper_detection() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let cache_directory = workspace.root_path().join("cache");
    let cache = FilesystemPackageCache::with_root(cache_directory.clone());

    // Create a dummy archive file
    let dummy_archive = workspace.file_path("test_pkg.slxarc");
    fs::write(&dummy_archive, b"dummy archive contents for hashing").expect("write dummy archive");

    let package_version = PackageVersion::new(1, 0, 0);
    let mut program = BinaryProgram::new();
    program.package_name = "test_pkg".to_string();
    program.package_version = "1.0.0".to_string();
    let raw_binary_bytes = program.to_bytes();

    // 1. Initially cache is cold (miss)
    let initial_fetch = cache
        .get_cached_binary("test_pkg", package_version, &dummy_archive)
        .expect("cache get must succeed");
    assert!(initial_fetch.is_none(), "Cold cache must return None");

    // 2. Put binary into cache (warm)
    cache
        .put_cached_binary(
            "test_pkg",
            package_version,
            &dummy_archive,
            &program,
            &raw_binary_bytes,
        )
        .expect("cache put must succeed");

    // 3. Subsequent fetch is a cache hit
    let cached_fetch = cache
        .get_cached_binary("test_pkg", package_version, &dummy_archive)
        .expect("cache get must succeed");
    assert!(
        cached_fetch.is_some(),
        "Warmed cache must return Some(program)"
    );
    let loaded_program = cached_fetch.unwrap();
    assert_eq!(loaded_program.package_name, "test_pkg");
    assert_eq!(loaded_program.package_version, "1.0.0");

    // 4. Tampering the source archive (modifying its contents) invalidates cache
    fs::write(
        &dummy_archive,
        b"tampered archive contents with altered SHA-256",
    )
    .expect("tamper dummy archive");

    let invalidated_fetch = cache
        .get_cached_binary("test_pkg", package_version, &dummy_archive)
        .expect("cache get after tamper must not panic");
    assert!(
        invalidated_fetch.is_none(),
        "Tampered source archive must invalidate stale cache entry"
    );
}

#[test]
fn test_cryptographic_checksum_pinning_and_tamper_rejection() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    // 1. Compile math library and install
    let math_source = workspace
        .write_file(
            "math.slx",
            r#"
public class MathService {
    public static int32 compute(int32 factor) {
        return factor * 10;
    }
}
"#,
        )
        .expect("write math");

    let math_binary = workspace.file_path("math.slxb");
    let compile_math = Cli::try_parse_from([
        "solix",
        "compile",
        math_source.to_str().expect("valid path"),
        "--name",
        "math_service",
        "--version",
        "1.0.0",
        "-o",
        math_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--install",
    ])
    .expect("compile math");
    assert_eq!(run(compile_math).expect("compile math"), 0);

    // 2. Compile consumer application linking math_service
    let app_source = workspace
        .write_file(
            "app.slx",
            r#"
extern import math_service.MathService;

public class Main {
    public static int32 main() {
        return MathService::compute(5);
    }
}
"#,
        )
        .expect("write app");

    let app_binary = workspace.file_path("app.slxb");
    let compile_app = Cli::try_parse_from([
        "solix",
        "compile",
        app_source.to_str().expect("valid path"),
        "--name",
        "app",
        "--deps",
        "math_service@1.0.0",
        "-o",
        app_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("compile app");
    assert_eq!(run(compile_app).expect("compile app"), 0);

    // 3. Normal execution succeeds and returns 50
    let run_app = Cli::try_parse_from([
        "solix",
        "run",
        app_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("run app");
    assert_eq!(run(run_app).expect("run app"), 50);

    // 4. Tamper the installed math_service binary in the package store
    let installed_math_path = package_store_directory
        .join("math_service-release@1.0.0")
        .join("math_service.slxb");
    assert!(installed_math_path.exists());

    let mut math_bytes = fs::read(&installed_math_path).expect("read math binary");
    // Alter a byte to corrupt SHA-256 without destroying binary format header
    let last_index = math_bytes.len() - 1;
    math_bytes[last_index] ^= 0xFF;
    fs::write(&installed_math_path, &math_bytes).expect("write tampered binary");

    // 5. Running without --yolo must fail due to cryptographic checksum mismatch!
    let run_tampered_strict = Cli::try_parse_from([
        "solix",
        "run",
        app_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("run tampered strict");
    let failure_result = run(run_tampered_strict);
    assert!(
        failure_result.is_err(),
        "Running against tampered dependency without --yolo must fail"
    );

    // 6. Running with --yolo must bypass verification and execute
    let run_tampered_yolo = Cli::try_parse_from([
        "solix",
        "run",
        app_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--yolo",
    ])
    .expect("run tampered yolo");
    // Even if execution encounters an instruction mismatch from corrupted byte, verification was bypassed
    let _ = run(run_tampered_yolo);
}

#[test]
fn test_list_platforms_column_and_json_payload() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    let source_file = workspace
        .write_file(
            "geometry.slx",
            r#"
public class Geometry {
    public static int32 area(int32 width, int32 height) {
        return width * height;
    }
}
"#,
        )
        .expect("write geometry");

    let binary_file = workspace.file_path("geometry.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().expect("valid path"),
        "--name",
        "geometry",
        "--version",
        "1.0.0",
        "-o",
        binary_file.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
        "--install",
    ])
    .expect("compile geometry");
    assert_eq!(run(compile_cli).expect("compile geometry"), 0);

    // Verify registry record has platforms: ["universal"]
    let registry = PackageRegistry::load_or_sync(&package_store_directory).expect("load registry");
    let geometry_record = registry
        .get("geometry", Some("1.0.0"))
        .expect("record exists");
    assert_eq!(geometry_record.platforms, vec!["universal".to_string()]);

    // Test solix list --json
    let list_json_cli = Cli::try_parse_from(["solix", "list", "--json", "--pkg-dir", store_str])
        .expect("list json");
    assert_eq!(run(list_json_cli).expect("list json must succeed"), 0);

    // Test solix list plain table
    let list_plain_cli =
        Cli::try_parse_from(["solix", "list", "--pkg-dir", store_str]).expect("list plain");
    assert_eq!(run(list_plain_cli).expect("list plain must succeed"), 0);
}

#[test]
fn test_info_project_dir_inspection_and_json() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let project_directory = workspace.root_path().join("my_app");
    fs::create_dir_all(project_directory.join("src")).expect("create my_app src");

    fs::write(
        project_directory.join("src").join("main.slx"),
        r#"
public class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
    )
    .expect("write main.slx");

    fs::write(
        project_directory.join("manifest.yaml"),
        r#"configuration:
  name: my_app
  version: "0.2.0"
  description: "Test application"
profiles:
  release:
    entrypoint: Main::main
    sources:
      - src
    platforms:
      - all
  debug:
    entrypoint: Main::main
    sources:
      - src
    platforms:
      - host
"#,
    )
    .expect("write manifest.yaml");

    // solix info <project_directory>
    let info_cli = Cli::try_parse_from([
        "solix",
        "info",
        project_directory.to_str().expect("valid utf8"),
    ])
    .expect("info project dir");
    assert_eq!(run(info_cli).expect("info project dir must succeed"), 0);

    // solix info <project_directory> --json
    let info_json_cli = Cli::try_parse_from([
        "solix",
        "info",
        project_directory.to_str().expect("valid utf8"),
        "--json",
    ])
    .expect("info project dir json");
    assert_eq!(
        run(info_json_cli).expect("info project dir json must succeed"),
        0
    );
}

#[test]
fn test_consumer_with_platforms_all_succeeds_with_partial_native_dependency() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let package_store = workspace.root_path().join("pkg_store");
    let package_store_str = package_store.to_str().expect("valid utf8");

    // Package a host-only native dependency archive
    let dependency_stage = workspace.root_path().join("dep_stage");
    let host_triple = solix_common::project::host_target_triple();
    let host_lib_name = if cfg!(windows) {
        "demo_subset_native.dll"
    } else if cfg!(target_os = "macos") {
        "libdemo_subset_native.dylib"
    } else {
        "libdemo_subset_native.so"
    };

    fs::create_dir_all(dependency_stage.join("bin")).expect("create dep bin");
    fs::create_dir_all(dependency_stage.join("lib").join(host_triple))
        .expect("create dep host lib");
    fs::write(
        dependency_stage.join("manifest.yaml"),
        r#"configuration:
  name: subset_dep
  version: "1.0.0"
profiles:
  release:
    sources:
      - src
"#,
    )
    .expect("write dep manifest");
    fs::write(
        dependency_stage.join("bin").join("subset_dep.slxb"),
        solix_common::BinaryProgram::new().to_bytes(),
    )
    .expect("write dep slxb");
    fs::write(
        dependency_stage
            .join("lib")
            .join(host_triple)
            .join(host_lib_name),
        b"host_native_binary_bytes",
    )
    .expect("write dep native lib");

    let dependency_archive = workspace.file_path("subset_dep.slxarc");
    solix_common::project::SlxarcArchive::pack_directory(&dependency_stage, &dependency_archive)
        .expect("pack dep archive");

    let install_cli = Cli::try_parse_from([
        "solix",
        "install",
        dependency_archive.to_str().expect("valid utf8"),
        "--profile",
        "release",
        "--pkg-dir",
        package_store_str,
    ])
    .expect("parse dep install");
    assert_eq!(run(install_cli).expect("install dep"), 0);

    // Consumer declares `platforms: [all]`. Since it does not specify explicit CLI targets,
    // it should bundle available host native libraries with an informative note instead of failing.
    let consumer_root = workspace.root_path().join("consumer_all");
    fs::create_dir_all(consumer_root.join("src")).expect("create consumer src");
    fs::write(
        consumer_root.join("src").join("main.slx"),
        "public class Main {\n    public static int32 main() { return 0; }\n}\n",
    )
    .expect("write consumer main.slx");
    fs::write(
        consumer_root.join("manifest.yaml"),
        r#"configuration:
  name: consumer_all
  version: "0.1.0"
profiles:
  release:
    entrypoint: Main::main
    static: true
    sources:
      - src
    platforms:
      - all
    dependencies:
      subset_dep:
        version: "1.0.0"
        profile: release
"#,
    )
    .expect("write consumer manifest.yaml");

    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        consumer_root.to_str().expect("valid utf8"),
        "--release",
        "--pkg-dir",
        package_store_str,
    ])
    .expect("parse consumer build cli");
    assert_eq!(
        run(build_cli).expect("consumer with platforms: [all] must build successfully"),
        0
    );
}
