//! Integration tests for package manager commands (`install`, `uninstall`, `list`)
//! and the concrete [`ClientModuleSolver`].

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_common::project::SlxarcArchive;
use solix_common::solver::{ModuleSolver, PackageVersion, SolverError};
use solix_toolchain::cli::Cli;
use solix_toolchain::run;
use solix_toolchain::solver::ClientModuleSolver;
use std::fs;

#[test]
fn test_package_install_list_and_uninstall_lifecycle() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("solix_packages");
    fs::create_dir_all(&package_store_directory).expect("Failed to create package store dir");
    let store_str = package_store_directory.to_str().expect("valid path");

    // 1. Compile math_utils.slxb
    let math_source = workspace
        .write_file(
            "math_utils.slx",
            r#"
public class MathUtils {
    public static int32 square(int32 number) {
        return number * number;
    }
}
"#,
        )
        .expect("Failed to write math source");

    let math_binary = workspace.file_path("math_utils.slxb");
    let compile_math_cli = Cli::try_parse_from([
        "solix",
        "compile",
        math_source.to_str().expect("valid path"),
        "--name",
        "math_utils",
        "--version",
        "1.0.0",
        "--description",
        "Mathematical utilities",
        "-o",
        math_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile CLI");
    assert_eq!(run(compile_math_cli).expect("Compile must succeed"), 0);

    // 2. Install math_utils.slxb using --pkg-dir
    let install_math_cli = Cli::try_parse_from([
        "solix",
        "install",
        math_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("Failed to parse install CLI");
    assert_eq!(run(install_math_cli).expect("Install must succeed"), 0);

    let installed_math_file = package_store_directory
        .join("math_utils-release@1.0.0")
        .join("math_utils.slxb");
    assert!(
        installed_math_file.exists(),
        "Expected installed binary at '{}'",
        installed_math_file.display()
    );

    // 3. Create and install raylib.slxarc version 1.0.0 and 1.2.0
    let stage_v100 = workspace.root_path().join("stage_raylib_100");
    let bin_v100 = stage_v100.join("bin");
    fs::create_dir_all(&bin_v100).expect("Failed to create bin dir");
    fs::write(
        stage_v100.join("manifest.toml"),
        r#"
[info]
name = "raylib"
version = "1.0.0"
description = "Raylib graphics"
"#,
    )
    .expect("Failed to write manifest");

    let dummy_source = workspace
        .write_file(
            "dummy_raylib.slx",
            r#"
public class Window {
    public static int32 init() { return 1; }
}
"#,
        )
        .expect("Failed to write dummy raylib source");

    let raylib_v100_binary = bin_v100.join("raylib.slxb");
    let compile_raylib_v100 = Cli::try_parse_from([
        "solix",
        "compile",
        dummy_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "1.0.0",
        "-o",
        raylib_v100_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse raylib compile");
    assert_eq!(run(compile_raylib_v100).expect("Compile must succeed"), 0);

    let archive_v100 = workspace.file_path("raylib_100.slxarc");
    SlxarcArchive::pack_directory(&stage_v100, &archive_v100).expect("Failed to pack v100");

    let install_v100_cli = Cli::try_parse_from([
        "solix",
        "install",
        archive_v100.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("Failed to parse install v100");
    assert_eq!(run(install_v100_cli).expect("Install v100 must succeed"), 0);

    // Version 1.2.0
    let stage_v120 = workspace.root_path().join("stage_raylib_120");
    let bin_v120 = stage_v120.join("bin");
    fs::create_dir_all(&bin_v120).expect("Failed to create bin dir");
    fs::write(
        stage_v120.join("manifest.toml"),
        r#"
[info]
name = "raylib"
version = "1.2.0"
description = "Raylib graphics updated"
"#,
    )
    .expect("Failed to write manifest");

    let raylib_v120_binary = bin_v120.join("raylib.slxb");
    let compile_raylib_v120 = Cli::try_parse_from([
        "solix",
        "compile",
        dummy_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "1.2.0",
        "-o",
        raylib_v120_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse raylib compile");
    assert_eq!(run(compile_raylib_v120).expect("Compile must succeed"), 0);

    let archive_v120 = workspace.file_path("raylib_120.slxarc");
    SlxarcArchive::pack_directory(&stage_v120, &archive_v120).expect("Failed to pack v120");

    let install_v120_cli = Cli::try_parse_from([
        "solix",
        "install",
        archive_v120.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("Failed to parse install v120");
    assert_eq!(run(install_v120_cli).expect("Install v120 must succeed"), 0);

    // 4. Test solix list (text table)
    let list_cli = Cli::try_parse_from(["solix", "list", "--pkg-dir", store_str])
        .expect("Failed to parse list CLI");
    assert_eq!(run(list_cli).expect("List command must succeed"), 0);

    // Test solix list --json
    let list_json_cli = Cli::try_parse_from(["solix", "list", "--json", "--pkg-dir", store_str])
        .expect("Failed to parse list json CLI");
    assert_eq!(
        run(list_json_cli).expect("List json command must succeed"),
        0
    );

    // Test ClientModuleSolver against installed packages
    let solver = ClientModuleSolver::with_store_dir(package_store_directory.clone());

    // Should resolve highest compatible version (1.2.0), not 1.0.0
    let resolved_compatible = solver
        .resolve_compatible("raylib", PackageVersion::new(1, 1, 0))
        .expect("Compatible resolution must succeed");
    assert_eq!(resolved_compatible.version, PackageVersion::new(1, 2, 0));

    // Should resolve latest version
    let resolved_latest = solver
        .resolve_latest("raylib")
        .expect("Latest resolution must succeed");
    assert_eq!(resolved_latest.version, PackageVersion::new(1, 2, 0));

    // Incompatible major version should fail
    let incompatible_major_result =
        solver.resolve_compatible("raylib", PackageVersion::new(2, 0, 0));
    assert!(matches!(
        incompatible_major_result,
        Err(SolverError::NoCompatibleVersion { .. })
    ));

    // 5. Test uninstall specific version: raylib@1.0.0
    let uninstall_v100_cli =
        Cli::try_parse_from(["solix", "uninstall", "raylib@1.0.0", "--pkg-dir", store_str])
            .expect("Failed to parse uninstall specific version CLI");
    assert_eq!(
        run(uninstall_v100_cli).expect("Uninstall specific version must succeed"),
        0
    );

    let raylib_v100_path = package_store_directory.join("raylib-release@1.0.0");
    assert!(!raylib_v100_path.exists(), "raylib 1.0.0 must be deleted");

    let raylib_v120_path = package_store_directory.join("raylib-release@1.2.0");
    assert!(
        raylib_v120_path.exists(),
        "raylib 1.2.0 must still remain installed"
    );

    // 6. Test uninstall entire package: raylib
    let uninstall_all_cli =
        Cli::try_parse_from(["solix", "uninstall", "raylib", "--pkg-dir", store_str])
            .expect("Failed to parse uninstall package CLI");
    assert_eq!(
        run(uninstall_all_cli).expect("Uninstall entire package must succeed"),
        0
    );

    assert!(
        !raylib_v120_path.exists(),
        "Entire raylib package folder must be deleted"
    );

    // 7. Uninstall non-existent package should fail
    let uninstall_nonexistent_cli = Cli::try_parse_from([
        "solix",
        "uninstall",
        "non_existent_package",
        "--pkg-dir",
        store_str,
    ])
    .expect("Failed to parse uninstall non-existent CLI");
    assert!(run(uninstall_nonexistent_cli).is_err());
}

#[test]
fn test_install_invalid_file_format_fails() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let invalid_file = workspace
        .write_file("readme.txt", "This is just a text file.")
        .expect("Failed to write text file");

    let install_cli = Cli::try_parse_from([
        "solix",
        "install",
        invalid_file.to_str().expect("valid path"),
    ])
    .expect("Failed to parse install CLI");

    let install_result = run(install_cli);
    assert!(
        install_result.is_err(),
        "Installing text file must fail with unsupported format error"
    );
}

#[test]
fn test_side_by_side_profiles_installation_and_resolution() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("solix_packages");
    fs::create_dir_all(&package_store_directory).expect("Failed to create package store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    let source_path = workspace
        .write_file(
            "physics.slx",
            r#"
public class Physics {
    public static int32 gravity() {
        return 98;
    }
}
"#,
        )
        .expect("write physics source");

    let debug_binary = workspace.file_path("physics_debug.slxb");
    let compile_debug = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().expect("valid path"),
        "--name",
        "physics",
        "--version",
        "1.0.0",
        "--profile",
        "debug",
        "-o",
        debug_binary.to_str().expect("valid path"),
    ])
    .expect("compile debug cli");
    assert_eq!(run(compile_debug).expect("compile debug"), 0);

    let release_binary = workspace.file_path("physics_release.slxb");
    let compile_release = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().expect("valid path"),
        "--name",
        "physics",
        "--version",
        "1.0.0",
        "--profile",
        "release",
        "-o",
        release_binary.to_str().expect("valid path"),
    ])
    .expect("compile release cli");
    assert_eq!(run(compile_release).expect("compile release"), 0);

    // Install debug build
    let install_debug = Cli::try_parse_from([
        "solix",
        "install",
        debug_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
        "--profile",
        "debug",
    ])
    .expect("install debug cli");
    assert_eq!(run(install_debug).expect("install debug"), 0);

    // Install release build side-by-side
    let install_release = Cli::try_parse_from([
        "solix",
        "install",
        release_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
        "--profile",
        "release",
    ])
    .expect("install release cli");
    assert_eq!(run(install_release).expect("install release"), 0);

    let debug_folder = package_store_directory.join("physics-debug@1.0.0");
    let release_folder = package_store_directory.join("physics-release@1.0.0");
    assert!(debug_folder.exists(), "Debug folder must exist");
    assert!(release_folder.exists(), "Release folder must exist");

    // ClientModuleSolver without profile preference picks release
    let solver = ClientModuleSolver::with_store_dir(package_store_directory.clone());
    let default_resolved = solver
        .resolve_latest("physics")
        .expect("resolve latest physics");
    assert_eq!(default_resolved.program.build_profile, "release");

    // ClientModuleSolver with explicit debug requirement picks debug
    let debug_resolved = solver
        .resolve_latest("physics-debug")
        .expect("resolve debug requirement");
    assert_eq!(debug_resolved.program.build_profile, "debug");

    // Uninstall debug profile only
    let uninstall_debug = Cli::try_parse_from([
        "solix",
        "uninstall",
        "physics-debug@1.0.0",
        "--pkg-dir",
        store_string,
    ])
    .expect("uninstall debug cli");
    assert_eq!(run(uninstall_debug).expect("uninstall debug"), 0);

    assert!(!debug_folder.exists(), "Debug folder must be deleted");
    assert!(release_folder.exists(), "Release folder must still exist");

    // After debug is uninstalled, solver fallback still finds release
    let fallback_resolved = solver
        .resolve_latest("physics")
        .expect("resolve latest physics");
    assert_eq!(fallback_resolved.program.build_profile, "release");
}

#[test]
fn test_list_output_file_redirection() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("solix_packages");
    fs::create_dir_all(&package_store_directory).expect("Failed to create package store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    let source_path = workspace
        .write_file("util.slx", "public class Util {}")
        .expect("write util");
    let util_binary = workspace.file_path("util.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().expect("valid path"),
        "--name",
        "util",
        "--version",
        "1.0.0",
        "-o",
        util_binary.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
        "--install",
    ])
    .expect("compile cli");
    assert_eq!(run(compile_cli).expect("compile"), 0);

    let output_text_path = workspace.file_path("package_list.txt");
    let list_text_cli = Cli::try_parse_from([
        "solix",
        "list",
        "--pkg-dir",
        store_string,
        "-o",
        output_text_path.to_str().expect("valid path"),
    ])
    .expect("list text cli");
    assert_eq!(run(list_text_cli).expect("list text"), 0);

    assert!(output_text_path.exists(), "List text file must exist");
    let text_content = fs::read_to_string(&output_text_path).expect("read text");
    assert!(text_content.contains("util"));
    assert!(text_content.contains("SOURCE"));

    let output_json_path = workspace.file_path("package_list.json");
    let list_json_cli = Cli::try_parse_from([
        "solix",
        "list",
        "--json",
        "--pkg-dir",
        store_string,
        "-o",
        output_json_path.to_str().expect("valid path"),
    ])
    .expect("list json cli");
    assert_eq!(run(list_json_cli).expect("list json"), 0);

    assert!(output_json_path.exists(), "List json file must exist");
    let json_content = fs::read_to_string(&output_json_path).expect("read json");
    let parsed_json: serde_json::Value = serde_json::from_str(&json_content).expect("parse json");
    assert!(parsed_json.is_array());
    assert_eq!(parsed_json.as_array().unwrap().len(), 1);
    assert_eq!(parsed_json[0]["name"], "util");
}

#[test]
fn test_reinstall_command_with_source_validation() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("solix_packages");
    fs::create_dir_all(&package_store_directory).expect("Failed to create package store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    let project_dir = workspace.root_path().join("my_lib");
    let new_cli = Cli::try_parse_from([
        "solix",
        "new",
        project_dir.to_str().expect("valid path"),
        "--name",
        "my_lib",
        "--version",
        "1.0.0",
        "--no-native",
    ])
    .expect("new cli");
    assert_eq!(run(new_cli).expect("new project"), 0);

    // Install from project directory
    let install_cli = Cli::try_parse_from([
        "solix",
        "install",
        project_dir.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install cli");
    assert_eq!(run(install_cli).expect("install project"), 0);

    let installed_archive = package_store_directory
        .join("my_lib-release@1.0.0")
        .join("my_lib.slxarc");
    assert!(installed_archive.exists(), "Installed archive must exist");

    // Reinstall by package name
    let reinstall_cli =
        Cli::try_parse_from(["solix", "reinstall", "my_lib", "--pkg-dir", store_string])
            .expect("reinstall cli");
    assert_eq!(run(reinstall_cli).expect("reinstall project"), 0);

    // Tamper the manifest name: reinstall must fail due to source mismatch
    let manifest_path = project_dir.join("manifest.toml");
    let tampered_manifest = r#"
[info]
name = "tampered_name"
version = "1.0.0"
"#;
    fs::write(&manifest_path, tampered_manifest).expect("write tampered manifest");

    let reinstall_mismatch =
        Cli::try_parse_from(["solix", "reinstall", "my_lib", "--pkg-dir", store_string])
            .expect("reinstall cli");
    let mismatch_result = run(reinstall_mismatch);
    assert!(
        mismatch_result.is_err(),
        "Reinstall with mismatched manifest name must fail"
    );
}
