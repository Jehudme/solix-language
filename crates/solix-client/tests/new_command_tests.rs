//! Integration tests for the `solix new` project generator command.

#![forbid(unsafe_code)]

use solix_client::{BuildArgs, Cli, Commands, NewArgs, execute_build, run};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_solix_new_default_application() {
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let target_project_directory = temporary_directory.path().join("sample_application");

    let new_arguments = NewArgs {
        path: target_project_directory.clone(),
        name: Some("sample_application".to_string()),
        version: "0.1.0".to_string(),
        description: "A sample Solix test project".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec!["Alice Doe <alice@example.com>".to_string()],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        no_native: false,
        deps: vec![],
        force: false,
    };

    let execution_result = run(Cli {
        command: Commands::New(new_arguments),
    });
    assert!(
        execution_result.is_ok(),
        "Failed to execute solix new: {execution_result:?}"
    );

    // Verify files created
    let manifest_path = target_project_directory.join("manifest.toml");
    assert!(manifest_path.exists(), "manifest.toml must exist");

    let manifest_content =
        fs::read_to_string(&manifest_path).expect("failed to read manifest.toml");
    assert!(manifest_content.contains("name = \"sample_application\""));
    assert!(manifest_content.contains("version = \"0.1.0\""));
    assert!(manifest_content.contains("authors = [\"Alice Doe <alice@example.com>\"]"));
    assert!(manifest_content.contains("main = \"Main::main\""));
    assert!(manifest_content.contains("test = \"TestRunner::test\""));

    assert!(
        target_project_directory
            .join("scripts")
            .join("build.py")
            .exists()
    );
    assert!(
        target_project_directory
            .join("scripts")
            .join("solix_toolbox.py")
            .exists()
    );
    assert!(
        target_project_directory
            .join("src")
            .join("main.slx")
            .exists()
    );
    assert!(
        target_project_directory
            .join("tests")
            .join("test_main.slx")
            .exists()
    );
    assert!(
        target_project_directory
            .join("natives")
            .join("sample_application_bridge")
            .join("Cargo.toml")
            .exists()
    );
    assert!(
        target_project_directory
            .join("natives")
            .join("sample_application_bridge")
            .join("src")
            .join("lib.rs")
            .exists()
    );

    // Verify build works on this newly generated project
    let build_arguments = BuildArgs {
        path: target_project_directory.clone(),
        profile: "debug".to_string(),
        debug: true,
        release: false,
        test: false,
        link: "dynamic".to_string(),
        r#static: false,
        pkg_dir: None,
    };
    let build_result = execute_build(build_arguments);
    assert!(
        build_result.is_ok(),
        "Failed to build newly generated project: {build_result:?}"
    );

    let output_archive = target_project_directory
        .join("build")
        .join("debug")
        .join("sample_application.slxarc");
    assert!(
        output_archive.exists(),
        "Expected output archive at {output_archive:?}"
    );
}

#[test]
fn test_solix_new_library_no_native_and_deps() {
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let target_project_directory = temporary_directory.path().join("math_utilities");

    let new_arguments = NewArgs {
        path: target_project_directory.clone(),
        name: Some("math_utilities".to_string()),
        version: "1.2.3".to_string(),
        description: "Math utilities library".to_string(),
        license: "MIT".to_string(),
        authors: vec!["Bob Developer".to_string()],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: true,
        no_native: true,
        deps: vec!["solix@2.0.0".to_string(), "extra_helpers:0.5.1".to_string()],
        force: false,
    };

    let execution_result = run(Cli {
        command: Commands::New(new_arguments),
    });
    assert!(
        execution_result.is_ok(),
        "Failed to generate library project: {execution_result:?}"
    );

    let manifest_path = target_project_directory.join("manifest.toml");
    let manifest_content =
        fs::read_to_string(&manifest_path).expect("failed to read manifest.toml");
    assert!(manifest_content.contains("name = \"math_utilities\""));
    assert!(manifest_content.contains("version = \"1.2.3\""));
    assert!(manifest_content.contains("solix = \"2.0.0\""));
    assert!(manifest_content.contains("extra_helpers = \"0.5.1\""));

    assert!(
        target_project_directory
            .join("src")
            .join("lib.slx")
            .exists(),
        "lib.slx must exist for library project"
    );
    assert!(
        !target_project_directory
            .join("src")
            .join("main.slx")
            .exists(),
        "main.slx must not exist for library project"
    );

    assert!(
        target_project_directory
            .join("natives")
            .join(".gitkeep")
            .exists(),
        "natives/.gitkeep should exist when --no-native is specified"
    );
}

#[test]
fn test_solix_new_existing_directory_force_flag() {
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let target_project_directory = temporary_directory.path().join("conflict_project");

    let initial_arguments = NewArgs {
        path: target_project_directory.clone(),
        name: Some("conflict_project".to_string()),
        version: "0.1.0".to_string(),
        description: "Initial".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec![],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        no_native: true,
        deps: vec![],
        force: false,
    };

    let first_result = run(Cli {
        command: Commands::New(initial_arguments.clone()),
    });
    assert!(first_result.is_ok());

    // Second run without force should fail
    let second_result = run(Cli {
        command: Commands::New(initial_arguments),
    });
    assert!(
        second_result.is_err(),
        "Expected error when overwriting without force"
    );

    // Third run with force should succeed
    let mut forced_arguments = NewArgs {
        path: target_project_directory.clone(),
        name: Some("conflict_project".to_string()),
        version: "0.2.0".to_string(),
        description: "Updated".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec![],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        no_native: true,
        deps: vec![],
        force: true,
    };
    forced_arguments.force = true;

    let third_result = run(Cli {
        command: Commands::New(forced_arguments),
    });
    assert!(
        third_result.is_ok(),
        "Expected success when overwriting with --force"
    );

    let manifest_path = target_project_directory.join("manifest.toml");
    let manifest_content =
        fs::read_to_string(&manifest_path).expect("failed to read manifest.toml");
    assert!(manifest_content.contains("version = \"0.2.0\""));
}
