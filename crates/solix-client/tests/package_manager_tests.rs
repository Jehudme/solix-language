//! Integration tests for package manager commands (`install`, `uninstall`, `list`)
//! and the concrete [`ClientModuleSolver`].

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_client::cli::Cli;
use solix_client::run;
use solix_client::solver::ClientModuleSolver;
use solix_common::project::SlxarcArchive;
use solix_common::solver::{ModuleSolver, PackageVersion, SolverError};
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
        .join("math_utils")
        .join("1.0.0")
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

    let raylib_v100_path = package_store_directory.join("raylib").join("1.0.0");
    assert!(!raylib_v100_path.exists(), "raylib 1.0.0 must be deleted");

    let raylib_v120_path = package_store_directory.join("raylib").join("1.2.0");
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

    let raylib_package_folder = package_store_directory.join("raylib");
    assert!(
        !raylib_package_folder.exists(),
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
