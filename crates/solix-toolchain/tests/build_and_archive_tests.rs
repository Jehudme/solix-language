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
[configuration]
name = "demo_package"
version = "1.2.3"
description = "A demo package for testing"
license = "Apache-2.0"
authors = ["Tester"]

[profiles.release]
entrypoint = "Demo::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]

[profiles.test]
inherits = "release"
entrypoint = "DemoTest::main"
sources = ["tests"]
"#;
    let manifest = ProjectManifest::from_toml_str(valid_toml).expect("Valid manifest should parse");
    assert_eq!(manifest.configuration.name, "demo_package");
    assert_eq!(manifest.configuration.version, "1.2.3");
    assert_eq!(
        manifest.configuration.description.as_deref(),
        Some("A demo package for testing")
    );
    assert_eq!(
        manifest.configuration.license.as_deref(),
        Some("Apache-2.0")
    );
    assert_eq!(manifest.main_entrypoint().as_deref(), Some("Demo::main"));
    assert_eq!(
        manifest.test_entrypoint().as_deref(),
        Some("DemoTest::main")
    );

    // 2. Minimal valid manifest
    let minimal_toml = r#"
[configuration]
name = "minimal_package"

[profiles.release]
"#;
    let minimal_manifest =
        ProjectManifest::from_toml_str(minimal_toml).expect("Minimal manifest should parse");
    assert_eq!(minimal_manifest.configuration.name, "minimal_package");
    assert_eq!(minimal_manifest.configuration.version, "0.1.0");

    // 3. Negative: missing [configuration] section
    let missing_configuration_toml = r#"
[profiles.release]
entrypoint = "Main::main"
"#;
    assert!(
        ProjectManifest::from_toml_str(missing_configuration_toml).is_err(),
        "Manifest without [configuration] must fail to parse"
    );

    // 4. Negative: missing 'name' in [configuration] section
    let missing_name_toml = r#"
[configuration]
version = "1.0.0"

[profiles.release]
"#;
    assert!(
        ProjectManifest::from_toml_str(missing_name_toml).is_err(),
        "Manifest without [configuration.name] must fail to parse"
    );

    // 5. Negative: missing [profiles.release] section
    let missing_release_toml = r#"
[configuration]
name = "no_release"
"#;
    assert!(
        ProjectManifest::from_toml_str(missing_release_toml).is_err(),
        "Manifest without mandatory [profiles.release] profile must fail to parse"
    );

    // 6. Negative: invalid TOML syntax
    let syntax_error_toml = r#"
[configuration
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
    assert_eq!(manifest.configuration.name, "untitled");
    assert_eq!(manifest.configuration.version, "0.1.0");
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

    // 2. Build --release (host target on single-OS CI runner)
    let build_release_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--release",
        "--target",
        "host",
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

    // Run the test archive to execute the test profile's baked entrypoint
    let run_test_cli =
        Cli::try_parse_from(["solix", "run", test_archive.to_str().expect("valid path")])
            .expect("Failed to parse run test archive CLI");
    let run_test_exit_code =
        run(run_test_cli).expect("Executing test .slxarc archive must succeed");
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

    assert_eq!(parsed_json["manifest"]["configuration"]["name"], "untitled");
    assert_eq!(parsed_json["manifest"]["configuration"]["version"], "0.1.0");
    assert_eq!(
        parsed_json["manifest"]["profiles"]["release"]["entrypoint"],
        "Main::main"
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
[configuration]
name = "deps_project"
version = "0.1.0"
description = "Project with dependencies"
license = "Apache-2.0"
authors = ["Tester"]

[profiles.release]
entrypoint = "Main::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]
dependencies = { math = { version = "1.0.0", profile = "release" } }

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false
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

// ============================================================================
// Test B.9: Test Profile Baked Entrypoint & CLI Entry/Profile Overrides
// ============================================================================

#[test]
fn test_test_profile_and_entry_point_override_execute_distinct_exit_codes() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("entry_override_app");

    // Copy only scripts/ from template so we have a clean self-contained project without natives.
    fs::create_dir_all(project_root.join("src")).expect("create src");
    fs::create_dir_all(project_root.join("tests")).expect("create tests");
    copy_project_tree(
        &template_source.join("scripts"),
        &project_root.join("scripts"),
    )
    .expect("copy scripts");

    let manifest_toml = r#"
[configuration]
name = "entry_override_app"
version = "0.1.0"
description = "Verifies profile and CLI entrypoint precedence"
license = "Apache-2.0"
authors = ["Tester"]

[profiles.release]
entrypoint = "Main::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false

[profiles.test]
inherits = "debug"
entrypoint = "TestRunner::test"
sources = ["tests"]
"#;
    fs::write(project_root.join("manifest.toml"), manifest_toml).expect("write manifest");

    let main_slx = r#"
public class Main {
    public static int32 main() {
        return 11;
    }

    public static int32 alternate() {
        return 33;
    }
}
"#;
    fs::write(project_root.join("src").join("main.slx"), main_slx).expect("write src/main.slx");

    let test_slx = r#"
public class TestRunner {
    public static int32 test() {
        return 22;
    }

    public static int32 alt_test() {
        return 44;
    }
}
"#;
    fs::write(project_root.join("tests").join("test_main.slx"), test_slx)
        .expect("write tests/test_main.slx");

    let store_dir = workspace.root_path().join("store");
    fs::create_dir_all(&store_dir).expect("create store");
    let store_str = store_dir.to_str().expect("valid path");

    // 1. Build --release and run without flags -> Main::main() = 11
    let build_release = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--release",
        "--pkg-dir",
        store_str,
    ])
    .expect("parse build release");
    assert_eq!(run(build_release).expect("build release"), 0);

    let release_archive = project_root
        .join("build")
        .join("release")
        .join("entry_override_app.slxarc");
    let run_release = Cli::try_parse_from([
        "solix",
        "run",
        release_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run release");
    assert_eq!(run(run_release).expect("run release"), 11);

    // 2. Build --test and run WITHOUT --profile or --entry -> MUST run TestRunner::test() = 22
    let build_test = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--test",
        "--pkg-dir",
        store_str,
    ])
    .expect("parse build test");
    assert_eq!(run(build_test).expect("build test"), 0);

    let test_archive = project_root
        .join("build")
        .join("test")
        .join("entry_override_app.slxarc");
    let run_test_default = Cli::try_parse_from([
        "solix",
        "run",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run test archive without flags");
    assert_eq!(
        run(run_test_default).expect("run test archive default"),
        22,
        "Running a --test .slxarc without flags must execute TestRunner::test (22), not release Main::main (11)"
    );

    // 3. Explicit --profile overrides on test archive
    let run_with_test_profile = Cli::try_parse_from([
        "solix",
        "run",
        "--profile",
        "test",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run -p test");
    assert_eq!(run(run_with_test_profile).expect("run -p test"), 22);

    let run_with_release_profile = Cli::try_parse_from([
        "solix",
        "run",
        "--profile",
        "release",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run -p release");
    assert_eq!(run(run_with_release_profile).expect("run -p release"), 11);

    // 4. Explicit -e / --entry overrides on test archive
    let run_entry_alt = Cli::try_parse_from([
        "solix",
        "run",
        "-e",
        "Main::alternate",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run -e Main::alternate");
    assert_eq!(run(run_entry_alt).expect("run -e Main::alternate"), 33);

    let run_entry_alt_test = Cli::try_parse_from([
        "solix",
        "run",
        "-e",
        "TestRunner::alt_test",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run -e TestRunner::alt_test");
    assert_eq!(
        run(run_entry_alt_test).expect("run -e TestRunner::alt_test"),
        44
    );

    // 5. Unknown --profile on archive must fail loudly instead of silently running release entrypoint
    let run_unknown_profile = Cli::try_parse_from([
        "solix",
        "run",
        "--profile",
        "does_not_exist",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run -p unknown");
    assert!(
        run(run_unknown_profile).is_err(),
        "Unknown --profile on .slxarc must return an error"
    );

    // 6. `solix compile --profile test` respects [profiles.test].entrypoint automatically
    let compiled_test_bin = workspace.file_path("compiled_test.slxb");
    let compile_test = Cli::try_parse_from([
        "solix",
        "compile",
        project_root
            .join("src")
            .join("main.slx")
            .to_str()
            .expect("valid path"),
        project_root
            .join("tests")
            .join("test_main.slx")
            .to_str()
            .expect("valid path"),
        "--profile",
        "test",
        "-o",
        compiled_test_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse compile --profile test");
    assert_eq!(run(compile_test).expect("compile --profile test"), 0);

    let run_compiled_test = Cli::try_parse_from([
        "solix",
        "run",
        compiled_test_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run compiled test slxb");
    assert_eq!(
        run(run_compiled_test).expect("run compiled test slxb"),
        22,
        "`solix compile --profile test` must bake TestRunner::test entrypoint"
    );

    // 7. `solix run --profile test tests/test_main.slx` discovers parent manifest test profile
    let run_source_test_profile = Cli::try_parse_from([
        "solix",
        "run",
        "--profile",
        "test",
        project_root
            .join("tests")
            .join("test_main.slx")
            .to_str()
            .expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run -p test test_main.slx");
    assert_eq!(
        run(run_source_test_profile).expect("run -p test test_main.slx"),
        22
    );
}

// ============================================================================
// Test B.10: Static Release Build with Native Dependency + Standalone Execution
// ============================================================================

#[test]
fn test_static_release_build_bundles_dependency_natives_and_runs_standalone() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let store_dir = workspace.root_path().join("pkg_store");
    let empty_store_dir = workspace.root_path().join("empty_pkg_store");
    fs::create_dir_all(&store_dir).expect("create store dir");
    fs::create_dir_all(&empty_store_dir).expect("create empty store dir");

    let template_source = locate_template_source();

    // 1. Build and install a native-backed dependency package `widget_dep` with overloaded
    //    constructors, virtual methods, static globals, and native `CallSymbol` bridge.
    let dep_root = workspace.root_path().join("widget_dep");
    copy_project_tree(&template_source, &dep_root).expect("copy template for widget_dep");

    let dep_manifest = r#"
[configuration]
name = "widget_dep"
version = "0.1.0"
description = "Dependency with native bridge, overloaded ctors, vtable, and static global"
license = "Apache-2.0"
authors = ["Tester"]

[profiles.release]
optimization_level = 3
inline_threshold = 0
debug_symbols = false
static = true
sources = ["src"]

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 0
debug_symbols = true
static = false
"#;
    fs::write(dep_root.join("manifest.toml"), dep_manifest).expect("write dep manifest");

    let dep_source = r#"
package widget_dep;

public class MathBridge {
    public static native int32 fast_add(int32 left, int32 right);
}

public class WidgetConfig {
    public static int32 BIAS = 4;
}

public class Widget {
    public int32 first;
    public int32 second;

    public Widget() {
        this.first = 1;
        this.second = 1;
    }

    public Widget(int32 init_first, int32 init_second) {
        this.first = init_first;
        this.second = init_second;
    }

    public virtual int32 compute(int32 delta) {
        return MathBridge::fast_add(this.first, this.second) + WidgetConfig::BIAS + delta;
    }

    public static int32 unused_main() {
        return 0;
    }
}
"#;
    fs::write(dep_root.join("src").join("main.slx"), dep_source).expect("write dep main.slx");

    let store_str = store_dir.to_str().expect("valid path");
    let build_install_dep = Cli::try_parse_from([
        "solix",
        "build",
        dep_root.to_str().expect("valid path"),
        "--release",
        "--target",
        "host",
        "--install",
        "--force",
        "--pkg-dir",
        store_str,
    ])
    .expect("parse dep build --install");
    assert_eq!(run(build_install_dep).expect("build+install widget_dep"), 0);

    // 2. Create consumer project linking statically in `[profiles.release]` against `widget_dep`
    //    and defining its own classes + distinct `[profiles.test]` entrypoint.
    let app_root = workspace.root_path().join("static_app");
    fs::create_dir_all(app_root.join("src")).expect("create app src");
    fs::create_dir_all(app_root.join("tests")).expect("create app tests");
    copy_project_tree(&template_source.join("scripts"), &app_root.join("scripts"))
        .expect("copy app scripts");

    let app_manifest = r#"
[configuration]
name = "static_app"
version = "0.1.0"
description = "Consumer app testing static release build with native dep"
license = "Apache-2.0"
authors = ["Tester"]

[profiles.release]
entrypoint = "App::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]
dependencies = { widget_dep = { version = "0.1.0", profile = "release" } }

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false

[profiles.test]
inherits = "debug"
entrypoint = "TestRunner::test"
sources = ["tests"]
"#;
    fs::write(app_root.join("manifest.toml"), app_manifest).expect("write app manifest");

    let app_main = r#"
import widget_dep.*;

public class LocalFirst {
    public int32 val;
    public LocalFirst(int32 v_in) {
        this.val = v_in;
    }
    public virtual int32 read_val() {
        return this.val;
    }
}

public class LocalSecond {
    public int32 extra;
    public LocalSecond(int32 e_in) {
        this.extra = e_in;
    }
    public virtual int32 read_extra() {
        return this.extra + 2;
    }
}

public class App {
    public static int32 main() {
        LocalFirst alpha = new LocalFirst(3);
        LocalSecond beta = new LocalSecond(5);
        Widget widget = new Widget(10, 20);
        // fast_add(10, 20) = 30, + BIAS(4) + delta(6) = 40; + alpha(3) + beta(7) = 50
        return widget.compute(6) + alpha.read_val() + beta.read_extra();
    }
}
"#;
    fs::write(app_root.join("src").join("main.slx"), app_main).expect("write app main.slx");

    let app_test = r#"
import widget_dep.*;

public class TestRunner {
    public static int32 test() {
        Widget widget = new Widget(4, 5);
        // fast_add(4, 5) = 9, + BIAS(4) + delta(0) = 13
        return widget.compute(0);
    }
}
"#;
    fs::write(app_root.join("tests").join("test_main.slx"), app_test)
        .expect("write app test_main.slx");

    let build_app_release = Cli::try_parse_from([
        "solix",
        "build",
        app_root.to_str().expect("valid path"),
        "--release",
        "--target",
        "host",
        "--pkg-dir",
        store_str,
    ])
    .expect("parse app release build");
    assert_eq!(run(build_app_release).expect("build static_app release"), 0);

    let release_archive = app_root
        .join("build")
        .join("release")
        .join("static_app.slxarc");
    assert!(release_archive.exists(), "release archive must exist");

    let inspection = SlxarcArchive::inspect(&release_archive).expect("inspect static release");
    let expected_native = host_cdylib_filename("rust_bridge");
    assert!(
        inspection
            .native_libraries
            .iter()
            .any(|lib_path| lib_path.ends_with(&expected_native)),
        "Static release archive must bundle dependency native library '{expected_native}', found {:?}",
        inspection.native_libraries
    );
    let primary_binary = &inspection.binaries[0].1;
    assert!(
        primary_binary.export_manifest.dependencies.is_empty(),
        "Static release binary must not keep external runtime bytecode dependencies"
    );

    // 3. Execute release archive using an EMPTY package store to prove 100% standalone operation!
    let empty_store_str = empty_store_dir.to_str().expect("valid path");
    let run_standalone = Cli::try_parse_from([
        "solix",
        "run",
        release_archive.to_str().expect("valid path"),
        "--pkg-dir",
        empty_store_str,
    ])
    .expect("parse run standalone static release");
    assert_eq!(
        run(run_standalone).expect("run standalone static release archive"),
        50
    );

    // 4. Build & run consumer test profile -> must execute TestRunner::test (13), not App::main (50)
    let build_app_test = Cli::try_parse_from([
        "solix",
        "build",
        app_root.to_str().expect("valid path"),
        "--test",
        "--pkg-dir",
        store_str,
    ])
    .expect("parse app test build");
    assert_eq!(run(build_app_test).expect("build static_app test"), 0);

    let test_archive = app_root
        .join("build")
        .join("test")
        .join("static_app.slxarc");
    let run_test = Cli::try_parse_from([
        "solix",
        "run",
        test_archive.to_str().expect("valid path"),
        "--pkg-dir",
        store_str,
    ])
    .expect("parse run static_app test archive");
    assert_eq!(run(run_test).expect("run static_app test archive"), 13);
}

#[test]
fn test_solix_build_scripts_remain_in_sync_across_repo() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|crates_dir| crates_dir.parent())
        .expect("resolve repository root");

    let canonical_path = manifest_dir
        .join("assets")
        .join("scripts")
        .join("solix_build.py");
    let canonical_bytes = fs::read(&canonical_path).expect("read canonical solix_build.py");

    let synced_locations = [manifest_dir
        .join("assets")
        .join("templates")
        .join("untitled-rust")
        .join("scripts")
        .join("solix_build.py")];

    for synced_path in synced_locations {
        let copy_bytes = fs::read(&synced_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", synced_path.display()));
        assert_eq!(
            copy_bytes,
            canonical_bytes,
            "solix_build.py at '{}' drifted from canonical '{}'",
            synced_path.display(),
            canonical_path.display()
        );
    }

    // First-party packages migrated to declarative manifest.yaml must no longer ship solix_build.py.
    for migrated_dir in [
        repo_root.join("examples").join("breakout"),
        repo_root.join("libraries").join("raylib"),
        repo_root.join("libraries").join("solixlib"),
    ] {
        assert!(
            migrated_dir.join("manifest.yaml").is_file(),
            "Migrated package '{}' must contain manifest.yaml",
            migrated_dir.display()
        );
        assert!(
            !migrated_dir.join("scripts").join("solix_build.py").exists(),
            "Migrated package '{}' must not contain legacy scripts/solix_build.py",
            migrated_dir.display()
        );
    }
}

#[test]
fn test_solix_build_engine_target_strategy_and_cross_dependency_bundling() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_dir = manifest_dir.join("assets").join("scripts");
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let python_exe = if cfg!(windows) { "python" } else { "python3" };

    let test_script = r#"
import sys
import zipfile
from pathlib import Path

scripts_dir = Path(sys.argv[1])
work_dir = Path(sys.argv[2])
sys.path.insert(0, str(scripts_dir))
import solix_build

# 1. Verify zigbuild enables Linux GNU, Windows GNU, AND both macOS Darwin targets
project_dir = work_dir / "native_proj"
target_dir = work_dir / "target"
env = {"PATH": ""}

for target_triple in (
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
):
    cmd, _, explicit_dir, label = solix_build._select_release_target_plan(
        target_triple,
        "x86_64-unknown-linux-gnu",
        project_dir,
        target_dir,
        env,
        has_cargo_zigbuild=True,
        has_zig=True,
        has_cargo_xwin=False,
        forced_target=False,
    )
    assert cmd is not None, f"Expected zigbuild plan for {target_triple}, got {label}"
    assert cmd[:2] == ["cargo", "zigbuild"] or (sys.platform == "darwin" and "apple-darwin" in target_triple), cmd
    assert explicit_dir is True

# 2. Verify cargo-xwin enables Windows MSVC on non-Windows hosts
msvc_cmd, _, msvc_explicit, _ = solix_build._select_release_target_plan(
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    project_dir,
    target_dir,
    env,
    has_cargo_zigbuild=False,
    has_zig=False,
    has_cargo_xwin=True,
    forced_target=False,
)
if sys.platform != "win32":
    assert msvc_cmd is not None and msvc_cmd[:3] == ["cargo", "xwin", "build"], msvc_cmd
    assert msvc_explicit is True

# 3. Verify stage_dependency_native_libraries preserves all 5 architectures and strips duplicate unqualified fallback
store_dir = work_dir / "store"
dep_archive_dir = store_dir / "raylib-release@5.5.0"
dep_archive_dir.mkdir(parents=True, exist_ok=True)
dep_archive = dep_archive_dir / "raylib.slxarc"
dep_manifest = '[configuration]\nname = "raylib"\nversion = "5.5.0"\n[profiles.release]\nsources = ["src"]\n'

with zipfile.ZipFile(dep_archive, "w") as archive_writer:
    archive_writer.writestr("manifest.toml", dep_manifest)
    archive_writer.writestr("lib/libraylib_native.so", b"dup_fallback")
    archive_writer.writestr("lib/x86_64-unknown-linux-gnu/libraylib_native.so", b"lin64")
    archive_writer.writestr("lib/aarch64-unknown-linux-gnu/libraylib_native.so", b"linarm64")
    archive_writer.writestr("lib/x86_64-pc-windows-msvc/raylib_native.dll", b"win64")
    archive_writer.writestr("lib/x86_64-apple-darwin/libraylib_native.dylib", b"mac64")
    archive_writer.writestr("lib/aarch64-apple-darwin/libraylib_native.dylib", b"macarm64")

consumer_manifest = {
    "configuration": {"name": "breakout", "version": "0.1.0"},
    "profiles": {
        "release": {
            "static": True,
            "dependencies": {"raylib": {"version": "5.5.0", "profile": "release"}},
        }
    },
}
stage_lib_dir = work_dir / "stage_lib"
stage_lib_dir.mkdir(parents=True, exist_ok=True)

import os
old_pkg_dir = os.environ.get("SOLIX_PKG_DIR")
os.environ["SOLIX_PKG_DIR"] = str(store_dir)
try:
    solix_build.stage_dependency_native_libraries(consumer_manifest, "release", stage_lib_dir)
finally:
    if old_pkg_dir is None:
        os.environ.pop("SOLIX_PKG_DIR", None)
    else:
        os.environ["SOLIX_PKG_DIR"] = old_pkg_dir

assert (stage_lib_dir / "x86_64-unknown-linux-gnu" / "libraylib_native.so").read_bytes() == b"lin64"
assert (stage_lib_dir / "aarch64-unknown-linux-gnu" / "libraylib_native.so").read_bytes() == b"linarm64"
assert (stage_lib_dir / "x86_64-pc-windows-msvc" / "raylib_native.dll").read_bytes() == b"win64"
assert (stage_lib_dir / "x86_64-apple-darwin" / "libraylib_native.dylib").read_bytes() == b"mac64"
assert (stage_lib_dir / "aarch64-apple-darwin" / "libraylib_native.dylib").read_bytes() == b"macarm64"
assert not (stage_lib_dir / "libraylib_native.so").exists(), "Duplicate unqualified fallback must be omitted"
"#;

    let status = std::process::Command::new(python_exe)
        .arg("-c")
        .arg(test_script)
        .arg(&scripts_dir)
        .arg(workspace.root_path())
        .status()
        .expect("execute python build engine check");
    assert!(status.success(), "solix_build.py strategy check failed");
}

#[test]
fn test_merge_bundled_packages_script_unifies_all_platforms_and_checksums() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|crates_dir| crates_dir.parent())
        .expect("resolve repository root");
    let merge_script = repo_root
        .join("scripts")
        .join("release")
        .join("merge_bundled_packages.py");
    assert!(
        merge_script.is_file(),
        "merge_bundled_packages.py must exist"
    );

    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let platform_targets = [
        (
            "x86_64-unknown-linux-gnu",
            "libdemo_native.so",
            b"linux_x64_payload".as_slice(),
        ),
        (
            "aarch64-unknown-linux-gnu",
            "libdemo_native.so",
            b"linux_arm64_payload".as_slice(),
        ),
        (
            "x86_64-pc-windows-msvc",
            "demo_native.dll",
            b"win_msvc_payload".as_slice(),
        ),
        (
            "x86_64-apple-darwin",
            "libdemo_native.dylib",
            b"mac_x64_payload".as_slice(),
        ),
        (
            "aarch64-apple-darwin",
            "libdemo_native.dylib",
            b"mac_arm64_payload".as_slice(),
        ),
    ];

    let mut input_dirs: Vec<PathBuf> = Vec::new();
    for (triple, lib_filename, lib_payload) in &platform_targets {
        let input_root = workspace.root_path().join(format!("in_{triple}"));
        let stage_dir = workspace.root_path().join(format!("stage_{triple}"));
        let bin_dir = stage_dir.join("bin");
        let target_lib_dir = stage_dir.join("lib").join(triple);
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::create_dir_all(&target_lib_dir).expect("create target lib dir");

        let plat_id =
            solix_common::project::triple_to_platform_id(triple).unwrap_or("linux-x86_64");
        fs::write(
            stage_dir.join("manifest.yaml"),
            format!(
                "configuration:\n  name: demo\n  version: \"1.0.0\"\nprofiles:\n  release:\n    sources:\n      - src\nsupported_platforms:\n- {plat_id}\nnative_library_checksums:\n  \"lib/{triple}/{lib_filename}\": \"0000\"\n"
            ),
        )
        .expect("write serde-saphyr-formatted manifest.yaml");
        fs::write(
            bin_dir.join("demo.slxb"),
            solix_common::BinaryProgram::new().to_bytes(),
        )
        .expect("write bytecode");
        fs::write(stage_dir.join("lib").join(lib_filename), b"unqualified_dup")
            .expect("write unqualified dup");
        fs::write(target_lib_dir.join(lib_filename), lib_payload).expect("write target lib");

        let pkg_dir = input_root.join("demo-release@1.0.0");
        fs::create_dir_all(&pkg_dir).expect("create pkg dir");
        SlxarcArchive::pack_directory(&stage_dir, &pkg_dir.join("demo.slxarc"))
            .expect("pack per-platform archive");
        input_dirs.push(input_root);
    }

    let merged_output_dir = workspace.root_path().join("merged_packages");
    let python_exe = if cfg!(windows) { "python" } else { "python3" };
    let mut cmd = std::process::Command::new(python_exe);
    cmd.arg(&merge_script).arg("--input-dirs");
    for input_dir in &input_dirs {
        cmd.arg(input_dir);
    }
    cmd.arg("--output-dir").arg(&merged_output_dir).arg("--require-targets").arg(
        "x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu,x86_64-pc-windows-msvc,x86_64-apple-darwin,aarch64-apple-darwin",
    );
    let merge_output = cmd.output().expect("run merge_bundled_packages.py");
    assert!(
        merge_output.status.success(),
        "merge_bundled_packages.py failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&merge_output.stdout),
        String::from_utf8_lossy(&merge_output.stderr)
    );

    let universal_archive = merged_output_dir
        .join("demo-release@1.0.0")
        .join("demo.slxarc");
    assert!(universal_archive.is_file(), "Universal archive must exist");

    let inspection = SlxarcArchive::inspect(&universal_archive).expect("inspect universal archive");
    for (triple, lib_filename, _) in &platform_targets {
        let expected_entry = format!("lib/{triple}/{lib_filename}");
        assert!(
            inspection
                .native_libraries
                .iter()
                .any(|entry| entry == &expected_entry),
            "Universal archive must contain '{expected_entry}', got {:?}",
            inspection.native_libraries
        );
    }
    assert!(
        !inspection
            .native_libraries
            .iter()
            .any(|entry| entry == "lib/libdemo_native.so"),
        "Unqualified duplicate must be stripped when target-qualified entries exist"
    );

    let merged_manifest = inspection.manifest.expect("universal manifest");
    assert_eq!(merged_manifest.native_library_checksums.len(), 5);

    let json_info_path = workspace.file_path("universal_info.json");
    let info_cli = Cli::try_parse_from([
        "solix",
        "info",
        universal_archive.to_str().expect("valid utf8"),
        "--json",
        "-o",
        json_info_path.to_str().expect("valid utf8"),
    ])
    .expect("parse info cli");
    assert_eq!(run(info_cli).expect("solix info must succeed"), 0);

    let parsed_info: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_info_path).expect("read json"))
            .expect("parse info json");
    let bundled_platforms: Vec<String> = parsed_info["bundled_platforms"]
        .as_array()
        .expect("bundled_platforms array")
        .iter()
        .map(|entry| entry.as_str().unwrap_or("").to_string())
        .collect();
    for (expected_platform, _, _) in &platform_targets {
        assert!(
            bundled_platforms.contains(&(*expected_platform).to_string()),
            "Expected platform '{expected_platform}' in {bundled_platforms:?}"
        );
    }

    assert!(merged_output_dir.join("bundled_manifest.json").is_file());

    // Verify downstream consumer release build automatically inherits all 5 bundled platforms
    let pkg_store = workspace.root_path().join("consumer_store");
    let pkg_store_str = pkg_store.to_str().expect("valid utf8");
    let install_merged = Cli::try_parse_from([
        "solix",
        "install",
        universal_archive.to_str().expect("valid utf8"),
        "--profile",
        "release",
        "--pkg-dir",
        pkg_store_str,
    ])
    .expect("parse install universal dep");
    assert_eq!(run(install_merged).expect("install universal dep"), 0);

    let consumer_root = workspace.root_path().join("consumer_app");
    let consumer_new = Cli::try_parse_from([
        "solix",
        "new",
        consumer_root.to_str().expect("valid utf8"),
        "--name",
        "consumer_app",
        "--no-native",
        "--deps",
        "demo@1.0.0",
    ])
    .expect("parse consumer new");
    assert_eq!(run(consumer_new).expect("scaffold consumer_app"), 0);

    let consumer_build = Cli::try_parse_from([
        "solix",
        "build",
        consumer_root.to_str().expect("valid utf8"),
        "--release",
        "--pkg-dir",
        pkg_store_str,
    ])
    .expect("parse consumer release build");
    assert_eq!(run(consumer_build).expect("build consumer_app release"), 0);

    let consumer_archive = consumer_root
        .join("build")
        .join("release")
        .join("consumer_app.slxarc");
    let consumer_inspection =
        SlxarcArchive::inspect(&consumer_archive).expect("inspect consumer release");
    for (triple, lib_filename, _) in &platform_targets {
        let expected_entry = format!("lib/{triple}/{lib_filename}");
        assert!(
            consumer_inspection
                .native_libraries
                .iter()
                .any(|entry| entry == &expected_entry),
            "Static consumer release archive must inherit '{expected_entry}' from dependency, got {:?}",
            consumer_inspection.native_libraries
        );
    }

    // Also verify unqualified debug-style archives under `bundled-packages-<triple>/` infer their target triple
    let debug_merge_workspace =
        HermeticCliWorkspace::create().expect("create debug merge workspace");
    let mut inferred_input_dirs: Vec<PathBuf> = Vec::new();
    for (triple, lib_filename, lib_payload) in &platform_targets {
        let bundled_dir = debug_merge_workspace
            .root_path()
            .join(format!("bundled-packages-{triple}"));
        let stage_dir = debug_merge_workspace
            .root_path()
            .join(format!("dbg_stage_{triple}"));
        fs::create_dir_all(stage_dir.join("bin")).expect("create dbg bin dir");
        fs::create_dir_all(stage_dir.join("lib")).expect("create dbg lib dir");
        fs::write(
            stage_dir.join("manifest.toml"),
            "[configuration]\nname = \"demo\"\nversion = \"1.0.0\"\n[profiles.release]\nsources = [\"src\"]\n[profiles.debug]\ninherits = \"release\"\nstatic = false\n",
        )
        .expect("write dbg manifest");
        fs::write(
            stage_dir.join("bin").join("demo.slxb"),
            solix_common::BinaryProgram::new().to_bytes(),
        )
        .expect("write dbg bytecode");
        fs::write(stage_dir.join("lib").join(lib_filename), lib_payload).expect("write dbg lib");

        let pkg_dir = bundled_dir.join("demo-debug@1.0.0");
        fs::create_dir_all(&pkg_dir).expect("create dbg pkg dir");
        SlxarcArchive::pack_directory(&stage_dir, &pkg_dir.join("demo.slxarc"))
            .expect("pack debug archive");
        inferred_input_dirs.push(bundled_dir);
    }

    let inferred_output_dir = debug_merge_workspace.root_path().join("merged_debug");
    let mut inferred_cmd = std::process::Command::new(python_exe);
    inferred_cmd.arg(&merge_script).arg("--input-dirs");
    for input_dir in &inferred_input_dirs {
        inferred_cmd.arg(input_dir);
    }
    inferred_cmd.arg("--output-dir").arg(&inferred_output_dir).arg("--require-targets").arg(
        "x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu,x86_64-pc-windows-msvc,x86_64-apple-darwin,aarch64-apple-darwin",
    );
    let inferred_out = inferred_cmd.output().expect("run inferred debug merge");
    assert!(
        inferred_out.status.success(),
        "Inferred debug merge failed: stderr={}",
        String::from_utf8_lossy(&inferred_out.stderr)
    );
}

// ============================================================================
// Tests B.13-B.16: Per-Profile Platforms, Release Cross-Platform Gate & Preflight
// ============================================================================

#[test]
fn test_manifest_per_profile_platforms_and_release_defaults() {
    let manifest_toml = r#"
[configuration]
name = "platform_spec_test"
version = "0.2.0"

[profiles.release]
sources = ["src"]

[profiles.debug]
inherits = "release"

[profiles.test]
inherits = "debug"
sources = ["tests"]

[profiles.win_only]
inherits = "release"
platforms = ["windows-x86_64"]

[profiles.pure]
inherits = "release"
platforms = ["universal"]
"#;

    let manifest = ProjectManifest::from_toml_str(manifest_toml).expect("parse manifest");
    let release_profile = manifest.resolve_profile("release").expect("release");
    assert_eq!(release_profile.platforms, vec!["all".to_string()]);
    let expanded_release = solix_common::project::expand_platform_list(&release_profile.platforms);
    assert_eq!(expanded_release.len(), 5);
    assert!(expanded_release.contains(&"linux-x86_64".to_string()));
    assert!(expanded_release.contains(&"windows-x86_64".to_string()));
    assert!(expanded_release.contains(&"macos-aarch64".to_string()));

    let debug_profile = manifest.resolve_profile("debug").expect("debug");
    assert_eq!(debug_profile.platforms, vec!["host".to_string()]);
    let test_profile = manifest.resolve_profile("test").expect("test");
    assert_eq!(test_profile.platforms, vec!["host".to_string()]);

    let win_profile = manifest.resolve_profile("win_only").expect("win_only");
    assert_eq!(win_profile.platforms, vec!["windows-x86_64".to_string()]);

    let pure_profile = manifest.resolve_profile("pure").expect("pure");
    assert_eq!(pure_profile.platforms, vec!["universal".to_string()]);
}

#[test]
fn test_solix_build_release_fails_when_required_cross_targets_missing() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_dir = manifest_dir.join("assets").join("scripts");
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let python_exe = if cfg!(windows) { "python" } else { "python3" };

    // Copy template with natives/ (requires all 5 desktop platforms on release)
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("native_cross_gate");
    copy_project_tree(&template_source, &project_root).expect("copy template");

    // Attempt compile_rust_project with impossible custom tool-free PATH (cross targets cannot build)
    let check_script = r#"
import os, sys
from pathlib import Path

scripts_dir = Path(sys.argv[1])
project_root = Path(sys.argv[2])
sys.path.insert(0, str(scripts_dir))
import solix_build as tb

manifest = tb.load_manifest(project_root)
platforms = tb.get_required_platforms(manifest, "release", project_root=project_root)
assert len(platforms) == 5, f"Expected all 5 release platforms, got {platforms}"
triples = tb.resolve_target_triples(manifest, "release", project_root=project_root)
assert len(triples) == 5, f"Expected 5 target triples, got {triples}"

# Confirm verify_environment / require_tool raises BuildError on missing tool / version mismatch
try:
    tb.require_tool("solix_nonexistent_tool_xyz", min_version="1.0.0")
    raise AssertionError("Expected BuildError for missing tool")
except tb.BuildError as err:
    assert "solix_nonexistent_tool_xyz" in err.message

# Confirm require_tool fails on unsatisfied minimum version
try:
    tb.require_tool(sys.executable, min_version="999.0.0")
    raise AssertionError("Expected BuildError for unsatisfied min_version")
except tb.BuildError as err:
    assert "999.0.0" in err.message

# Confirm auto_build hard-fails when a non-host target is required and unavailable
manifest["profiles"]["release"]["platforms"] = ["linux-x86_64", "macos-aarch64"]
old_path = os.environ.get("PATH", "")
os.environ["PATH"] = ""
try:
    tb.auto_build("release", project_root=project_root)
    raise AssertionError("Expected fatal BuildError when cross-platform target unavailable")
except tb.BuildError as err:
    assert err.phase in ("Pre-Flight Verification", "Cross-Platform Release Gate", "Native Rust Build"), err.phase
finally:
    os.environ["PATH"] = old_path
"#;

    let status = std::process::Command::new(python_exe)
        .arg("-c")
        .arg(check_script)
        .arg(&scripts_dir)
        .arg(&project_root)
        .status()
        .expect("execute python release gate check");
    assert!(status.success(), "Cross-platform release gate test failed");
}

#[test]
fn test_solix_build_check_env_flag_executes_preflight_only() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let template_source = locate_template_source();
    let project_root = workspace.root_path().join("preflight_proj");
    copy_project_tree(&template_source, &project_root).expect("copy template");

    let check_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--debug",
        "--check-env",
    ])
    .expect("parse build --check-env");
    assert_eq!(run(check_cli).expect("build --check-env must pass"), 0);

    // Must NOT have produced build/debug/*.slxarc since --check-env only runs Phase 0
    assert!(
        !project_root
            .join("build")
            .join("debug")
            .join("untitled.slxarc")
            .exists(),
        "--check-env must skip compilation and packaging"
    );
}

#[test]
fn test_slxarc_runtime_and_install_reject_incompatible_host_platform() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let stage_dir = workspace.root_path().join("incompat_stage");
    let bin_dir = stage_dir.join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin dir");

    let incompatible_manifest = r#"
supported_platforms = ["solix-nonexistent-os-arm99"]

[configuration]
name = "incompat_pkg"
version = "0.1.0"

[profiles.release]
sources = ["src"]
"#;
    fs::write(stage_dir.join("manifest.toml"), incompatible_manifest).expect("write manifest");
    let program = solix_common::BinaryProgram::new();
    fs::write(bin_dir.join("incompat_pkg.slxb"), program.to_bytes()).expect("write bytecode");

    let archive_path = workspace.file_path("incompat_pkg.slxarc");
    SlxarcArchive::pack_directory(&stage_dir, &archive_path).expect("pack incompatible archive");

    // 1. SlxarcArchive::extract_and_load must reject incompatible host
    let extract_cache = workspace.root_path().join("extract_cache");
    let extract_err = SlxarcArchive::extract_and_load(&archive_path, &extract_cache)
        .expect_err("extract_and_load must reject incompatible host platform");
    let extract_msg = extract_err.to_string();
    assert!(
        extract_msg.contains("solix-nonexistent-os-arm99") && extract_msg.contains("incompatible"),
        "Expected clear platform incompatibility message, got: {extract_msg}"
    );

    // 2. solix install must reject incompatible host
    let store_dir = workspace.root_path().join("store");
    let install_cli = Cli::try_parse_from([
        "solix",
        "install",
        archive_path.to_str().expect("valid path"),
        "--pkg-dir",
        store_dir.to_str().expect("valid path"),
    ])
    .expect("parse solix install");
    let install_result = run(install_cli);
    assert!(
        install_result.is_err(),
        "solix install must reject archive built for an incompatible platform"
    );

    // 3. solix run must reject incompatible host
    let run_cli = Cli::try_parse_from(["solix", "run", archive_path.to_str().expect("valid path")])
        .expect("parse solix run");
    let run_result = run(run_cli);
    assert!(
        run_result.is_err(),
        "solix run must reject archive built for an incompatible platform"
    );
}

#[test]
fn test_manifest_yaml_declarative_cmake_project_build_and_hooks() {
    let cmake_probe = std::process::Command::new("cmake")
        .arg("--version")
        .output();
    if cmake_probe.is_err() {
        return;
    }

    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let project_root = workspace.root_path().join("cmake_yaml_demo");
    fs::create_dir_all(project_root.join("src")).expect("create src dir");
    fs::create_dir_all(project_root.join("natives").join("cmake_calc"))
        .expect("create cmake native dir");

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf();
    let ffi_c_dir = repo_root.join("ffi").join("c");

    let cmakelists = r#"cmake_minimum_required(VERSION 3.20)
project(cmake_calc C)

option(SOLIX_NATIVE_BUILD "Build as Solix native shared library plugin" OFF)

if(SOLIX_NATIVE_BUILD)
    include(FetchContent)
    if(DEFINED ENV{SOLIX_FFI_SOURCE_DIR} AND EXISTS "$ENV{SOLIX_FFI_SOURCE_DIR}/CMakeLists.txt")
        FetchContent_Declare(solix_ffi SOURCE_DIR "$ENV{SOLIX_FFI_SOURCE_DIR}")
    else()
        FetchContent_Declare(solix_ffi SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
    endif()
    FetchContent_MakeAvailable(solix_ffi)
    add_library(cmake_calc SHARED calc.c)
    target_link_libraries(cmake_calc PRIVATE solix::ffi)
    target_compile_definitions(cmake_calc PRIVATE SOLIX_PLUGIN_MODE=1)
else()
    add_library(cmake_calc STATIC calc.c)
endif()
"#;
    fs::write(
        project_root
            .join("natives")
            .join("cmake_calc")
            .join("CMakeLists.txt"),
        cmakelists,
    )
    .expect("write CMakeLists.txt");

    let calc_c = r#"#include <stdint.h>

#ifdef SOLIX_PLUGIN_MODE
#include "solix_ffi.h"

static SolixCValue native_calc_add_mul(
    void* raw_vm_context,
    const SolixCValue* args,
    uint32_t argc,
    SolixCValue* out_error
) {
    (void)raw_vm_context;
    (void)out_error;
    if (argc < 2) {
        return solix_cvalue_i32(0);
    }
    int32_t lhs = solix_cvalue_as_i32(args[0]);
    int32_t rhs = solix_cvalue_as_i32(args[1]);
    return solix_cvalue_i32((lhs + rhs) * 2);
}

static const SolixNativeEntry CMAKE_CALC_ENTRIES[] = {
    {"CmakeCalc::add_mul", native_calc_add_mul},
};

SOLIX_REGISTER_MODULE(CMAKE_CALC_ENTRIES)
#else
int32_t standalone_calc_add_mul(int32_t lhs, int32_t rhs) {
    return (lhs + rhs) * 2;
}
#endif
"#;
    fs::write(
        project_root
            .join("natives")
            .join("cmake_calc")
            .join("calc.c"),
        calc_c,
    )
    .expect("write calc.c");

    let main_slx = r#"
public class CmakeCalc {
    native public static int32 add_mul(int32 left_val, int32 right_val);
}

public class Main {
    public static int32 main() {
        return CmakeCalc::add_mul(10, 11);
    }
}
"#;
    fs::write(project_root.join("src").join("main.slx"), main_slx).expect("write main.slx");

    let python_exe = if cfg!(windows) { "python" } else { "python3" };
    let manifest_yaml = format!(
        r#"vars:
  pkg_ver: &proj_ver "0.4.2"

configuration:
  name: cmake_yaml_demo
  version: *proj_ver
  description: Declarative CMake YAML test
  license: Apache-2.0

profiles:
  release:
    entrypoint: Main::main
    sources:
      - src
    platforms:
      - host
    natives:
      cmake_calc:
        profile: release

  custom_rel:
    entrypoint: Main::main
    optimization_level: 2
    debug_symbols: false
    static: true
    sources:
      - src
    platforms:
      - host
    pre_compile:
      - "{python_exe} -c \"import pathlib; pathlib.Path('pre_marker.txt').write_text('ok')\""
    post_compile:
      - "{python_exe} -c \"import pathlib; assert pathlib.Path('build/custom_rel/cmake_yaml_demo.slxarc').is_file(); pathlib.Path('post_marker.txt').write_text('ok')\""
    natives:
      cmake_calc:
        profile: release
"#
    );
    fs::write(project_root.join("manifest.yaml"), manifest_yaml).expect("write manifest.yaml");
    assert!(ffi_c_dir.is_dir(), "expected ffi/c directory in repo");

    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--profile",
        "custom_rel",
    ])
    .expect("parse build custom_rel");
    assert_eq!(
        run(build_cli).expect("manifest.yaml CMake build must pass"),
        0
    );

    assert_eq!(
        fs::read_to_string(project_root.join("pre_marker.txt")).expect("pre marker written"),
        "ok"
    );
    assert_eq!(
        fs::read_to_string(project_root.join("post_marker.txt")).expect("post marker written"),
        "ok"
    );

    let archive_path = project_root
        .join("build")
        .join("custom_rel")
        .join("cmake_yaml_demo.slxarc");
    let inspection = SlxarcArchive::inspect(&archive_path).expect("inspect YAML .slxarc");
    let parsed_manifest = inspection.manifest.expect("manifest parsed inside .slxarc");
    assert_eq!(parsed_manifest.version(), "0.4.2");
    assert!(!parsed_manifest.native_library_checksums.is_empty());

    let run_cli = Cli::try_parse_from(["solix", "run", archive_path.to_str().expect("valid path")])
        .expect("parse run cmake archive");
    let exit_code = run(run_cli).expect("running CMake FFI archive must succeed");
    assert_eq!(exit_code, 42);
}

#[test]
fn test_manifest_yaml_auto_detects_cargo_with_solix_native_feature() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let project_root = workspace.root_path().join("cargo_yaml_demo");
    fs::create_dir_all(project_root.join("src")).expect("create src dir");
    fs::create_dir_all(project_root.join("natives").join("rust_calc").join("src"))
        .expect("create rust native dir");

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf();
    let ffi_rust_dir = repo_root.join("ffi").join("rust");
    let ffi_rust_str = ffi_rust_dir.to_string_lossy().replace('\\', "/");

    let cargo_toml = format!(
        r#"[package]
name = "rust_calc"
version = "0.1.0"
edition = "2024"

[workspace]

[lib]
crate-type = ["cdylib", "rlib"]

[features]
solix-native = ["dep:solix-ffi"]

[dependencies]
solix-ffi = {{ path = "{ffi_rust_str}", optional = true }}
"#
    );
    fs::write(
        project_root
            .join("natives")
            .join("rust_calc")
            .join("Cargo.toml"),
        cargo_toml,
    )
    .expect("write Cargo.toml");

    let lib_rs = r#"pub fn standalone_square(value: i32) -> i32 {
    value * value
}

#[cfg(feature = "solix-native")]
solix_ffi::solix_export! {
    "RustCalc::square" => native_square,
}

#[cfg(feature = "solix-native")]
pub unsafe extern "C" fn native_square(
    _vm: *mut std::ffi::c_void,
    args: *const solix_ffi::SolixCValue,
    argc: u32,
    _out_error: *mut solix_ffi::SolixCValue,
) -> solix_ffi::SolixCValue {
    if argc < 1 || args.is_null() {
        return solix_ffi::SolixCValue::from_i32(0);
    }
    let input = unsafe { (*args).as_i32() };
    solix_ffi::SolixCValue::from_i32(standalone_square(input))
}
"#;
    fs::write(
        project_root
            .join("natives")
            .join("rust_calc")
            .join("src")
            .join("lib.rs"),
        lib_rs,
    )
    .expect("write lib.rs");

    let main_slx = r#"
public class RustCalc {
    native public static int32 square(int32 value);
}

public class Main {
    public static int32 main() {
        return RustCalc::square(7);
    }
}
"#;
    fs::write(project_root.join("src").join("main.slx"), main_slx).expect("write main.slx");

    let manifest_yaml = r#"configuration:
  name: cargo_yaml_demo
  version: "0.1.0"

profiles:
  release:
    entrypoint: Main::main
    sources:
      - src
    platforms:
      - host
    natives:
      rust_calc:
        profile: release
"#;
    fs::write(project_root.join("manifest.yaml"), manifest_yaml).expect("write manifest.yaml");

    let build_cli = Cli::try_parse_from([
        "solix",
        "build",
        project_root.to_str().expect("valid path"),
        "--profile",
        "release",
    ])
    .expect("parse build release");
    assert_eq!(
        run(build_cli).expect("manifest.yaml Cargo build with solix-native must pass"),
        0
    );

    let archive_path = project_root
        .join("build")
        .join("release")
        .join("cargo_yaml_demo.slxarc");
    let run_cli = Cli::try_parse_from(["solix", "run", archive_path.to_str().expect("valid path")])
        .expect("parse run cargo archive");
    let exit_code = run(run_cli).expect("running Cargo solix-native archive must succeed");
    assert_eq!(exit_code, 49);
}

#[test]
fn test_build_fails_when_native_dependency_missing_requested_platform() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let pkg_store = workspace.root_path().join("pkg_store");
    let pkg_store_str = pkg_store.to_str().expect("valid utf8");

    // Package a host-only native dependency `.slxarc` (only host triple native lib).
    let dep_stage = workspace.root_path().join("dep_stage");
    let host_triple = solix_common::project::host_target_triple();
    let host_lib_name = if cfg!(windows) {
        "demo_host_native.dll"
    } else if cfg!(target_os = "macos") {
        "libdemo_host_native.dylib"
    } else {
        "libdemo_host_native.so"
    };
    fs::create_dir_all(dep_stage.join("bin")).expect("create dep bin");
    fs::create_dir_all(dep_stage.join("lib").join(host_triple)).expect("create dep host lib");
    fs::write(
        dep_stage.join("manifest.yaml"),
        "configuration:\n  name: single_plat_dep\n  version: \"1.0.0\"\nprofiles:\n  release:\n    sources:\n      - src\n",
    )
    .expect("write dep manifest");
    fs::write(
        dep_stage.join("bin").join("single_plat_dep.slxb"),
        solix_common::BinaryProgram::new().to_bytes(),
    )
    .expect("write dep slxb");
    fs::write(
        dep_stage.join("lib").join(host_triple).join(host_lib_name),
        b"host_only_native_bytes",
    )
    .expect("write dep native lib");

    let dep_archive = workspace.file_path("single_plat_dep.slxarc");
    SlxarcArchive::pack_directory(&dep_stage, &dep_archive).expect("pack dep archive");

    let install_cli = Cli::try_parse_from([
        "solix",
        "install",
        dep_archive.to_str().expect("valid utf8"),
        "--profile",
        "release",
        "--pkg-dir",
        pkg_store_str,
    ])
    .expect("parse dep install");
    assert_eq!(run(install_cli).expect("install host-only dep"), 0);

    // Consumer declares `platforms: [linux, windows]` which is NOT covered by a single-OS dependency.
    let consumer_root = workspace.root_path().join("multi_plat_consumer");
    fs::create_dir_all(consumer_root.join("src")).expect("create consumer src");
    fs::write(
        consumer_root.join("src").join("main.slx"),
        "public class Main {\n    public static int32 main() { return 0; }\n}\n",
    )
    .expect("write consumer main.slx");
    fs::write(
        consumer_root.join("manifest.yaml"),
        r#"configuration:
  name: multi_plat_consumer
  version: "0.1.0"
profiles:
  release:
    entrypoint: Main::main
    static: true
    sources:
      - src
    platforms:
      - linux
      - windows
    dependencies:
      single_plat_dep:
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
        pkg_store_str,
    ])
    .expect("parse consumer build cli");
    let build_err = run(build_cli).expect_err(
        "building multi-platform consumer against host-only native dependency must fail",
    );
    let err_msg = format!("{build_err:#}");
    assert!(
        err_msg.contains("single_plat_dep@1.0.0")
            && err_msg.contains("does not support requested target platform"),
        "Unexpected error message: {err_msg}"
    );
}

// ============================================================================
// Test B.17: .slxp Package Architecture, Root binaries.slxb, and Native Loader Filtering
// ============================================================================

#[test]
fn test_slxp_package_root_binaries_and_bin_native_filtering() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let host_triple = solix_common::project::host_target_triple();
    let stage_directory = workspace.file_path("stage_slxp");
    let stage_bin_triple = stage_directory.join("bin").join(host_triple);
    fs::create_dir_all(&stage_bin_triple).expect("create stage bin target dir");

    let manifest_yaml = r#"configuration:
  name: modern_slxp_pkg
  version: "1.0.0"
  description: "Test .slxp package container format"
profiles:
  release:
    entrypoint: Main::main
"#;
    fs::write(stage_directory.join("manifest.yaml"), manifest_yaml).expect("write manifest.yaml");

    let binary_program = solix_common::bytecode::BinaryProgram::new();
    let binary_bytes = binary_program.to_bytes();
    fs::write(stage_directory.join("binaries.slxb"), &binary_bytes)
        .expect("write root binaries.slxb");

    #[cfg(target_os = "windows")]
    let native_module_name = "native_calc.dll";
    #[cfg(target_os = "macos")]
    let native_module_name = "libnative_calc.dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let native_module_name = "libnative_calc.so";

    #[cfg(target_os = "windows")]
    let transitive_lib_name = "glfw3.dll";
    #[cfg(target_os = "macos")]
    let transitive_lib_name = "libglfw.dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let transitive_lib_name = "libglfw.so";

    fs::write(
        stage_bin_triple.join(native_module_name),
        b"NATIVE_MODULE_STUB_BYTES",
    )
    .expect("write native module");
    fs::write(
        stage_bin_triple.join(transitive_lib_name),
        b"TRANSITIVE_LIB_STUB_BYTES",
    )
    .expect("write transitive lib");

    let package_path = workspace.file_path("modern_slxp_pkg.slxp");
    solix_common::project::SlxpPackage::pack_directory(&stage_directory, &package_path)
        .expect("pack slxp package");
    assert!(package_path.is_file(), "Emitted .slxp package must exist");

    let inspection =
        solix_common::project::SlxpPackage::inspect(&package_path).expect("inspect slxp package");
    assert!(
        inspection.manifest.is_some(),
        "Package manifest must be parsed"
    );
    assert_eq!(
        inspection.binaries.len(),
        1,
        "Package must contain root binaries.slxb"
    );
    assert_eq!(
        inspection.binaries[0].0, "binaries.slxb",
        "Primary binary must be root binaries.slxb"
    );

    let extract_cache = workspace.file_path("extract_cache");
    let loaded =
        solix_common::project::SlxpPackage::extract_and_load(&package_path, &extract_cache)
            .expect("extract and load slxp package");

    assert_eq!(
        loaded.binaries.len(),
        1,
        "Loaded package must have primary binary"
    );

    let loaded_native_names: Vec<String> = loaded
        .native_libraries
        .iter()
        .filter_map(|lib_path| {
            lib_path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(str::to_string)
        })
        .collect();
    assert!(
        loaded_native_names.contains(&native_module_name.to_string()),
        "native_* module must be registered in loaded native_libraries"
    );
    assert!(
        !loaded_native_names.contains(&transitive_lib_name.to_string()),
        "Transitive support library must NOT be registered in loaded native_libraries"
    );

    assert!(
        extract_cache.join(transitive_lib_name).exists(),
        "Transitive support library must still be extracted to disk for dynamic linker resolution"
    );
}

#[test]
fn test_slxp_package_dynamic_libraries_declaration_permits_non_prefixed_modules() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let host_triple = solix_common::project::host_target_triple();
    let stage_directory = workspace.file_path("stage_declared");
    let stage_bin_triple = stage_directory.join("bin").join(host_triple);
    fs::create_dir_all(&stage_bin_triple).expect("create stage bin target dir");

    #[cfg(target_os = "windows")]
    let custom_driver_name = "custom_driver.dll";
    #[cfg(target_os = "macos")]
    let custom_driver_name = "libcustom_driver.dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let custom_driver_name = "libcustom_driver.so";

    let manifest_yaml = format!(
        r#"configuration:
  name: declared_driver_pkg
  version: "1.0.0"
dynamic_libraries:
  - "{custom_driver_name}"
profiles:
  release:
    entrypoint: Main::main
"#
    );
    fs::write(stage_directory.join("manifest.yaml"), manifest_yaml).expect("write manifest.yaml");

    let mut binary_program = solix_common::bytecode::BinaryProgram::new();
    binary_program
        .dynamic_libraries
        .push(custom_driver_name.to_string());
    let binary_bytes = binary_program.to_bytes();
    fs::write(stage_directory.join("binaries.slxb"), &binary_bytes)
        .expect("write root binaries.slxb");

    fs::write(
        stage_bin_triple.join(custom_driver_name),
        b"CUSTOM_DRIVER_STUB_BYTES",
    )
    .expect("write custom driver");

    let package_path = workspace.file_path("declared_driver_pkg.slxp");
    solix_common::project::SlxpPackage::pack_directory(&stage_directory, &package_path)
        .expect("pack slxp package");

    let extract_cache = workspace.file_path("driver_extract_cache");
    let loaded =
        solix_common::project::SlxpPackage::extract_and_load(&package_path, &extract_cache)
            .expect("extract and load slxp package");

    let loaded_native_names: Vec<String> = loaded
        .native_libraries
        .iter()
        .filter_map(|lib_path| {
            lib_path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(str::to_string)
        })
        .collect();
    assert!(
        loaded_native_names.contains(&custom_driver_name.to_string()),
        "Declared non-native_ dynamic library must be registered when explicitly declared in metadata"
    );
}
