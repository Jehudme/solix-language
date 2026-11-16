//! Comprehensive integration test suite for Solix project build system,
//! `.slxarc` archive containers, native shared library loading, and CLI commands.

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_common::project::{ProjectManifest, SlxarcArchive};
use solix_toolchain::cli::Cli;
use solix_toolchain::run;
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively copies a directory tree while excluding ephemeral target and cache directories.
fn copy_project_tree(source_directory: &Path, destination_directory: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination_directory)?;
    for entry_result in fs::read_dir(source_directory)? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        let name_string = file_name.to_string_lossy();

        if name_string == "target"
            || name_string == "cache"
            || name_string == "build"
            || name_string == "__pycache__"
        {
            continue;
        }

        let child_source = entry.path();
        let child_destination = destination_directory.join(&file_name);

        if file_type.is_dir() {
            copy_project_tree(&child_source, &child_destination)?;
        } else {
            fs::copy(&child_source, &child_destination)?;
        }
    }
    Ok(())
}

/// Locates the untitled-rust project template inside the crate assets.
fn locate_template_source() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("assets")
        .join("templates")
        .join("untitled-rust")
}

/// Returns the platform-specific shared library filename for a Rust cdylib crate.
fn host_cdylib_filename(base_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{base_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{base_name}.dylib")
    } else {
        format!("lib{base_name}.so")
    }
}

// ============================================================================
// Test B.1: manifest.toml Parsing & Minimal Schema Enforcement
// ============================================================================

#[test]
fn test_manifest_toml_parsing_and_validation() {
    // 1. Valid full manifest
    let valid_toml = r#"
[info]
name = "demo_package"
version = "1.2.3"
description = "A demo package for testing"
license = "Apache-2.0"
authors = ["Tester"]

[entrypoints]
main = "Demo::main"
test = "DemoTest::main"
"#;
    let manifest = ProjectManifest::from_toml_str(valid_toml).expect("Valid manifest should parse");
    assert_eq!(manifest.info.name, "demo_package");
    assert_eq!(manifest.info.version, "1.2.3");
    assert_eq!(
        manifest.info.description.as_deref(),
        Some("A demo package for testing")
    );
    assert_eq!(manifest.info.license.as_deref(), Some("Apache-2.0"));
    assert_eq!(manifest.entrypoints.main.as_deref(), Some("Demo::main"));
    assert_eq!(manifest.entrypoints.test.as_deref(), Some("DemoTest::main"));

    // 2. Minimal valid manifest
    let minimal_toml = r#"
[info]
name = "minimal_package"
"#;
    let minimal_manifest =
        ProjectManifest::from_toml_str(minimal_toml).expect("Minimal manifest should parse");
    assert_eq!(minimal_manifest.info.name, "minimal_package");
    assert_eq!(minimal_manifest.info.version, "0.1.0");
    assert!(minimal_manifest.entrypoints.main.is_none());
    assert!(minimal_manifest.entrypoints.test.is_none());

    // 3. Negative: missing [info] section
    let missing_info_toml = r#"
[entrypoints]
main = "Main::main"
"#;
    assert!(
        ProjectManifest::from_toml_str(missing_info_toml).is_err(),
        "Manifest without [info] must fail to parse"
    );

    // 4. Negative: missing 'name' in [info] section
    let missing_name_toml = r#"
[info]
version = "1.0.0"
"#;
    assert!(
        ProjectManifest::from_toml_str(missing_name_toml).is_err(),
        "Manifest without [info.name] must fail to parse"
    );

    // 5. Negative: invalid TOML syntax
    let syntax_error_toml = r#"
[info
name = broken
"#;
    assert!(
        ProjectManifest::from_toml_str(syntax_error_toml).is_err(),
        "Malformed TOML syntax must fail to parse"
    );
}

// ============================================================================
// Test B.2: Directory Recursion Compilation & Build Profiles
// ============================================================================

#[test]
fn test_compile_directory_recursion_and_profiles() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let source_dir = workspace.root_path().join("src");
    let math_subdir = source_dir.join("math");
    fs::create_dir_all(&math_subdir).expect("Failed to create math directory");

    let math_source = r#"
public class MathOps {
    public static int32 add(int32 operand_a, int32 operand_b) {
        return operand_a + operand_b;
    }
}
"#;
    fs::write(math_subdir.join("operations.slx"), math_source).expect("Failed to write math ops");

    let main_source = r#"
public class Main {
    public static int32 main() {
        return MathOps.add(15, 27);
    }
}
"#;
    fs::write(source_dir.join("main.slx"), main_source).expect("Failed to write main");

    // 1. Compile with --profile debug
    let debug_binary = workspace.file_path("app_debug.slxb");
    let debug_compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_dir.to_str().expect("valid path"),
        "--profile",
        "debug",
        "-o",
        debug_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile debug CLI");

    let compile_exit_code = run(debug_compile_cli).expect("Debug compile must succeed");
    assert_eq!(compile_exit_code, 0);
    assert!(debug_binary.exists());

    let run_debug_cli =
        Cli::try_parse_from(["solix", "run", debug_binary.to_str().expect("valid path")])
            .expect("Failed to parse run debug CLI");
    let debug_result = run(run_debug_cli).expect("Debug binary execution must succeed");
    assert_eq!(debug_result, 42);

    // 2. Compile with --profile release
    let release_binary = workspace.file_path("app_release.slxb");
    let release_compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_dir.to_str().expect("valid path"),
        "--profile",
        "release",
        "-o",
        release_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile release CLI");

    let release_exit_code = run(release_compile_cli).expect("Release compile must succeed");
    assert_eq!(release_exit_code, 0);
    assert!(release_binary.exists());

    let run_release_cli =
        Cli::try_parse_from(["solix", "run", release_binary.to_str().expect("valid path")])
            .expect("Failed to parse run release CLI");
    let release_result = run(run_release_cli).expect("Release binary execution must succeed");
    assert_eq!(release_result, 42);

    // 3. Compile with --profile test
    let test_binary = workspace.file_path("app_test.slxb");
    let test_compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_dir.to_str().expect("valid path"),
        "--profile",
        "test",
        "-o",
        test_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile test CLI");

    let test_exit_code = run(test_compile_cli).expect("Test profile compile must succeed");
    assert_eq!(test_exit_code, 0);
    assert!(test_binary.exists());

    let run_test_cli =
        Cli::try_parse_from(["solix", "run", test_binary.to_str().expect("valid path")])
            .expect("Failed to parse run test CLI");
    let test_result = run(run_test_cli).expect("Test binary execution must succeed");
    assert_eq!(test_result, 42);
}

// ============================================================================
// Test B.3: End-to-End untitled-rust Template Build & Execution
// ============================================================================

#[test]
fn test_end_to_end_untitled_rust_template_build_and_run_all_profiles() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("untitled-rust");

    copy_project_tree(&template_source, &project_root).expect("Failed to copy template tree");

    // 1. Build --debug
    let build_debug_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--debug",
    ])
    .expect("Failed to parse build debug CLI");
    let build_debug_code = run(build_debug_cli).expect("solix build --debug must succeed");
    assert_eq!(build_debug_code, 0);

    let debug_archive = project_root
        .join("build")
        .join("debug")
        .join("untitled.slxarc");
    assert!(
        debug_archive.exists(),
        "Expected debug archive at '{}'",
        debug_archive.display()
    );

    // Inspect the debug archive structure
    let debug_inspection =
        SlxarcArchive::inspect(&debug_archive).expect("Inspection of debug archive must succeed");
    let manifest = debug_inspection
        .manifest
        .as_ref()
        .expect("manifest must be present in archive");
    assert_eq!(manifest.info.name, "untitled");
    assert_eq!(manifest.info.version, "0.1.0");
    assert!(
        debug_inspection
            .file_entries
            .iter()
            .any(|item| item.path == "manifest.toml")
    );
    assert!(
        debug_inspection
            .file_entries
            .iter()
            .any(|item| item.path == "bin/untitled.slxb")
    );
    let expected_cdylib_name = host_cdylib_filename("rust_bridge");
    assert!(
        debug_inspection
            .file_entries
            .iter()
            .any(|item| item.path.starts_with("lib/") && item.path.ends_with(&expected_cdylib_name))
    );
    assert!(
        debug_inspection
            .file_entries
            .iter()
            .any(|item| item.path == "resources/config.txt")
    );

    // Run the debug archive
    let run_debug_cli =
        Cli::try_parse_from(["solix", "run", debug_archive.to_str().expect("valid path")])
            .expect("Failed to parse run debug archive CLI");
    let run_debug_exit_code =
        run(run_debug_cli).expect("Executing debug .slxarc archive must succeed");
    assert_eq!(run_debug_exit_code, 0);

    // 2. Build --release
    let build_release_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--release",
    ])
    .expect("Failed to parse build release CLI");
    let build_release_code = run(build_release_cli).expect("solix build --release must succeed");
    assert_eq!(build_release_code, 0);

    let release_archive = project_root
        .join("build")
        .join("release")
        .join("untitled.slxarc");
    assert!(
        release_archive.exists(),
        "Expected release archive at '{}'",
        release_archive.display()
    );

    let run_release_cli = Cli::try_parse_from([
        "solix",
        "run",
        release_archive.to_str().expect("valid path"),
    ])
    .expect("Failed to parse run release archive CLI");
    let run_release_exit_code =
        run(run_release_cli).expect("Executing release .slxarc archive must succeed");
    assert_eq!(run_release_exit_code, 0);

    // 3. Build --test
    let build_test_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--test",
    ])
    .expect("Failed to parse build test CLI");
    let build_test_code = run(build_test_cli).expect("solix build --test must succeed");
    assert_eq!(build_test_code, 0);

    let test_archive = project_root
        .join("build")
        .join("test")
        .join("untitled.slxarc");
    assert!(
        test_archive.exists(),
        "Expected test archive at '{}'",
        test_archive.display()
    );

    // Run the test archive with --test flag to execute TestRunner::main
    let run_test_cli = Cli::try_parse_from([
        "solix",
        "run",
        "--test",
        test_archive.to_str().expect("valid path"),
    ])
    .expect("Failed to parse run test archive CLI");
    let run_test_exit_code =
        run(run_test_cli).expect("Executing test .slxarc archive with --test must succeed");
    assert_eq!(run_test_exit_code, 0);
}

// ============================================================================
// Test B.4: Direct Native Library Loading via -l / --lib CLI Flag
// ============================================================================

#[test]
fn test_run_with_direct_native_library_flag() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("template_native_build");

    copy_project_tree(&template_source, &project_root).expect("Failed to copy template tree");

    // Build debug archive to compile the native library
    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--debug",
    ])
    .expect("Failed to parse build CLI");
    assert_eq!(run(build_cli).expect("Build must succeed"), 0);

    // Locate the emitted native cdylib in build/.cache/stage/debug/lib
    let native_lib_path = project_root
        .join("build")
        .join(".cache")
        .join("stage")
        .join("debug")
        .join("lib")
        .join(host_cdylib_filename("rust_bridge"));
    assert!(
        native_lib_path.exists(),
        "Native bridge library must exist at '{}'",
        native_lib_path.display()
    );

    // Write a separate, standalone Solix application that calls MathBridge::fast_add
    let independent_source = workspace
        .write_file(
            "independent_app.slx",
            r#"
public class MathBridge {
    public static native int32 fast_add(int32 a, int32 b);
}

public class Main {
    public static int32 main() {
        return MathBridge::fast_add(100, 200);
    }
}
"#,
        )
        .expect("Failed to write independent source");

    let independent_binary = workspace.file_path("independent_app.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        independent_source.to_str().expect("valid path"),
        "-o",
        independent_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile CLI");
    assert_eq!(run(compile_cli).expect("Compile must succeed"), 0);

    // Run independent binary mounting the native library with -l
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        "-l",
        native_lib_path.to_str().expect("valid path"),
        independent_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse run CLI with -l flag");

    let exit_code = run(run_cli).expect("Run with native library must succeed");
    assert_eq!(exit_code, 300);
}

#[test]
fn test_run_with_missing_native_library_flag_fails() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let dummy_source = workspace
        .write_file(
            "dummy.slx",
            r#"
public class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .expect("Failed to write dummy source");

    let dummy_binary = workspace.file_path("dummy.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        dummy_source.to_str().expect("valid path"),
        "-o",
        dummy_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile CLI");
    assert_eq!(run(compile_cli).expect("Compile must succeed"), 0);

    let nonexistent_lib = workspace.file_path("does_not_exist_lib.so");
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        "-l",
        nonexistent_lib.to_str().expect("valid path"),
        dummy_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse run CLI");

    let run_result = run(run_cli);
    assert!(
        run_result.is_err(),
        "Run with nonexistent native library must fail"
    );
}

// ============================================================================
// Test B.5: solix info on .slxarc Archives (Text, JSON, Disasm, API)
// ============================================================================

#[test]
fn test_info_command_on_slxarc_archive() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("info_test_project");

    copy_project_tree(&template_source, &project_root).expect("Failed to copy template tree");

    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--debug",
    ])
    .expect("Failed to parse build CLI");
    assert_eq!(run(build_cli).expect("Build must succeed"), 0);

    let archive_path = project_root
        .join("build")
        .join("debug")
        .join("untitled.slxarc");
    assert!(archive_path.exists());

    // 1. Text info summary
    let info_text_cli =
        Cli::try_parse_from(["solix", "info", archive_path.to_str().expect("valid path")])
            .expect("Failed to parse info text CLI");
    assert_eq!(run(info_text_cli).expect("Info text must succeed"), 0);

    // 2. JSON info export
    let json_output_file = workspace.file_path("archive_info.json");
    let info_json_cli = Cli::try_parse_from([
        "solix",
        "info",
        archive_path.to_str().expect("valid path"),
        "--json",
        "-o",
        json_output_file.to_str().expect("valid path"),
    ])
    .expect("Failed to parse info json CLI");
    assert_eq!(run(info_json_cli).expect("Info JSON must succeed"), 0);
    assert!(json_output_file.exists());

    let json_text = fs::read_to_string(&json_output_file).expect("Failed to read JSON output");
    let parsed_json: serde_json::Value =
        serde_json::from_str(&json_text).expect("Failed to deserialize JSON");

    assert_eq!(parsed_json["manifest"]["info"]["name"], "untitled");
    assert_eq!(parsed_json["manifest"]["info"]["version"], "0.1.0");
    assert_eq!(parsed_json["manifest"]["entrypoints"]["main"], "Main::main");
    assert_eq!(
        parsed_json["manifest"]["entrypoints"]["test"],
        "TestRunner::test"
    );

    let files_array = parsed_json["contained_files"]
        .as_array()
        .expect("contained_files must be an array");
    assert!(
        files_array
            .iter()
            .any(|item| item["path"] == "manifest.toml")
    );
    assert!(
        files_array
            .iter()
            .any(|item| item["path"] == "bin/untitled.slxb")
    );
    assert!(
        files_array
            .iter()
            .any(|item| item["path"] == "resources/config.txt")
    );

    let natives_array = parsed_json["native_libraries"]
        .as_array()
        .expect("native_libraries must be an array");
    assert!(!natives_array.is_empty());

    // 3. Disassembly inspection on .slxarc
    let info_disasm_cli = Cli::try_parse_from([
        "solix",
        "info",
        archive_path.to_str().expect("valid path"),
        "--disasm",
    ])
    .expect("Failed to parse info disasm CLI");
    assert_eq!(run(info_disasm_cli).expect("Info disasm must succeed"), 0);

    // 4. API interface inspection on .slxarc
    let info_api_cli = Cli::try_parse_from([
        "solix",
        "info",
        archive_path.to_str().expect("valid path"),
        "--api",
    ])
    .expect("Failed to parse info api CLI");
    assert_eq!(run(info_api_cli).expect("Info API must succeed"), 0);
}

// ============================================================================
// Test B.6: Edge Cases & Error Points of Failure
// ============================================================================

#[test]
fn test_info_command_on_corrupt_slxarc_fails() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let corrupt_path = workspace.file_path("corrupt.slxarc");
    fs::write(&corrupt_path, b"not_a_valid_zip_archive_header")
        .expect("Failed to write corrupt file");

    let info_cli =
        Cli::try_parse_from(["solix", "info", corrupt_path.to_str().expect("valid path")])
            .expect("Failed to parse info CLI");

    let info_result = run(info_cli);
    assert!(
        info_result.is_err(),
        "Info on corrupt archive must return an error"
    );
}

#[test]
fn test_build_missing_build_py_fails() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let empty_project_dir = workspace.file_path("empty_project");
    fs::create_dir_all(&empty_project_dir).expect("Failed to create empty project dir");

    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        empty_project_dir.to_str().expect("valid path"),
    ])
    .expect("Failed to parse build CLI");

    let build_result = run(build_cli);
    assert!(
        build_result.is_err(),
        "Build on project without scripts/build.py must return an error"
    );
}

#[test]
fn test_run_slxarc_missing_manifest_fails() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let stage_dir = workspace.root_path().join("stage_no_manifest");
    let bin_dir = stage_dir.join("bin");
    fs::create_dir_all(&bin_dir).expect("Failed to create bin directory");

    let dummy_source = workspace
        .write_file(
            "dummy_app.slx",
            r#"
public class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .expect("Failed to write dummy source");

    let dummy_binary = bin_dir.join("app.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        dummy_source.to_str().expect("valid path"),
        "-o",
        dummy_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile CLI");
    assert_eq!(run(compile_cli).expect("Compile must succeed"), 0);

    let archive_path = workspace.file_path("no_manifest.slxarc");
    SlxarcArchive::pack_directory(&stage_dir, &archive_path).expect("Failed to pack directory");

    let run_cli = Cli::try_parse_from(["solix", "run", archive_path.to_str().expect("valid path")])
        .expect("Failed to parse run CLI");

    let run_result = run(run_cli);
    assert!(
        run_result.is_err(),
        "Running .slxarc without manifest.toml must return an error"
    );
}

#[test]
fn test_build_project_with_dependencies_in_manifest() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");

    // 1. Compile and install dependency math 1.0.0
    let math_source = workspace
        .write_file(
            "math.slx",
            r#"
public class Math {
    public static int32 compute(int32 operand) {
        return operand * 10;
    }
}
"#,
        )
        .expect("write math");
    let math_bin = workspace.file_path("math.slxb");
    let compile_math = Cli::try_parse_from([
        "solix",
        "compile",
        math_source.to_str().expect("valid path"),
        "--name",
        "math",
        "--version",
        "1.0.0",
        "-o",
        math_bin.to_str().expect("valid path"),
        "--pkg-dir",
        package_store_directory.to_str().expect("valid path"),
    ])
    .expect("compile math");
    assert_eq!(run(compile_math).expect("compile math"), 0);

    let install_math = Cli::try_parse_from([
        "solix",
        "install",
        math_bin.to_str().expect("valid path"),
        "--pkg-dir",
        package_store_directory.to_str().expect("valid path"),
    ])
    .expect("install math");
    assert_eq!(run(install_math).expect("install math"), 0);

    // 2. Setup project from template
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("deps_project");
    copy_project_tree(&template_source, &project_root).expect("copy template");

    // 3. Update manifest.toml to declare dependency
    let manifest_content = r#"
[info]
name = "deps_project"
version = "0.1.0"
description = "Project with dependencies"
license = "Apache-2.0"
authors = ["Tester"]

[entrypoints]
main = "Main::main"

[dependencies]
math = { version = "1.0.0", profile = "release" }
"#;
    fs::write(project_root.join("manifest.toml"), manifest_content).expect("write manifest");

    // 4. Update src/main.slx to use Math::compute
    let main_content = r#"
import math.*;

public class Main {
    public static int32 main() {
        return Math::compute(7);
    }
}
"#;
    fs::write(project_root.join("src").join("main.slx"), main_content).expect("write main.slx");

    // 5. Build project with debug profile and explicit package store directory
    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--debug",
        "--pkg-dir",
        package_store_directory.to_str().expect("valid path"),
    ])
    .expect("parse build");
    assert_eq!(run(build_cli).expect("build project"), 0);

    let built_archive = project_root
        .join("build")
        .join("debug")
        .join("deps_project.slxarc");
    assert!(
        built_archive.exists(),
        "Built archive must exist at '{}'",
        built_archive.display()
    );

    // 6. Run the built archive: Math::compute(7) = 70
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        built_archive.to_str().expect("valid path"),
        "--pkg-dir",
        package_store_directory.to_str().expect("valid path"),
    ])
    .expect("parse run");
    assert_eq!(run(run_cli).expect("run built archive"), 70);
}
