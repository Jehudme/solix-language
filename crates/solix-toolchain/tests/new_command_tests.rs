//! Integration tests for `solix new` and `solix new native` scaffolding commands.

#![forbid(unsafe_code)]

use clap::Parser;
use solix_common::project::ProjectManifest;
use solix_toolchain::{BuildArgs, Cli, Commands, NewArgs, execute_build, run};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_solix_new_default_application() {
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let target_project_directory = temporary_directory.path().join("sample_application");

    let new_arguments = NewArgs {
        subcommand: None,
        path: Some(target_project_directory.clone()),
        name: Some("sample_application".to_string()),
        version: "0.1.0".to_string(),
        description: "A sample Solix test project".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec!["Alice Doe <alice@example.com>".to_string()],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        native: None,
        no_native: false,
        no_std: false,
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

    let manifest_path = target_project_directory.join("manifest.yaml");
    assert!(manifest_path.exists(), "manifest.yaml must exist");
    assert!(
        !target_project_directory.join("manifest.toml").exists(),
        "new projects must use manifest.yaml instead of manifest.toml"
    );
    assert!(
        !target_project_directory
            .join("scripts")
            .join("build.py")
            .exists(),
        "new projects must not generate legacy scripts/build.py"
    );
    assert!(
        !target_project_directory
            .join("scripts")
            .join("solix_build.py")
            .exists(),
        "new projects must not generate legacy scripts/solix_build.py"
    );

    let parsed_manifest =
        ProjectManifest::from_file(&manifest_path).expect("generated manifest.yaml must parse");
    assert_eq!(parsed_manifest.name(), "sample_application");
    assert_eq!(parsed_manifest.version(), "0.1.0");
    assert_eq!(
        parsed_manifest.authors(),
        &["Alice Doe <alice@example.com>"]
    );
    assert_eq!(
        parsed_manifest.main_entrypoint().as_deref(),
        Some("Main::main")
    );
    assert_eq!(
        parsed_manifest.test_entrypoint().as_deref(),
        Some("TestRunner::test")
    );

    let release_profile = parsed_manifest
        .resolve_profile("release")
        .expect("release profile");
    assert_eq!(release_profile.platforms, vec!["all"]);
    let debug_profile = parsed_manifest
        .resolve_profile("debug")
        .expect("debug profile");
    assert_eq!(debug_profile.platforms, vec!["host"]);

    let main_slx = fs::read_to_string(target_project_directory.join("src").join("main.slx"))
        .expect("read src/main.slx");
    assert!(
        main_slx.contains("Console::println"),
        "Default starter src/main.slx should use Console::println"
    );
    assert!(
        target_project_directory
            .join("tests")
            .join("test_main.slx")
            .exists()
    );
    let raw_manifest = fs::read_to_string(target_project_directory.join("manifest.yaml"))
        .expect("read manifest.yaml");
    assert!(
        raw_manifest.contains("vars:")
            && raw_manifest.contains("&sources")
            && raw_manifest.contains("&dependencies"),
        "manifest.yaml must use vars: YAML anchors"
    );
    assert!(
        !target_project_directory
            .join("natives")
            .join("sample_application_bridge")
            .exists(),
        "Pure Solix default project must not scaffold a native bridge subproject without --native"
    );
    assert!(
        !target_project_directory.join("bin").exists(),
        "Pure Solix default project must not scaffold empty bin/ directories without --native"
    );
    assert!(
        !target_project_directory.join("lib").exists(),
        "Legacy lib/ directory must not be created"
    );

    // Verify solix check and solix test work cleanly on a freestanding project (always runs)
    let freestanding_dir = temporary_directory.path().join("freestanding_check");
    solix_toolchain::execute_new(NewArgs {
        subcommand: None,
        path: Some(freestanding_dir.clone()),
        name: None,
        version: "0.1.0".to_string(),
        description: "Freestanding".to_string(),
        license: "Apache-2.0".to_string(),
        authors: Vec::new(),
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        native: None,
        no_native: false,
        no_std: true,
        deps: Vec::new(),
        force: false,
    })
    .expect("scaffold freestanding");
    assert_eq!(
        solix_toolchain::execute_check(solix_toolchain::CheckArgs {
            path: freestanding_dir.clone(),
            profile: "debug".to_string(),
            pkg_dir: None,
            no_verify: true,
        })
        .expect("execute_check on freestanding project"),
        0
    );
    assert_eq!(
        solix_toolchain::execute_test(solix_toolchain::TestArgs {
            path: freestanding_dir,
            profile: "test".to_string(),
            filter: None,
            pkg_dir: None,
            no_verify: true,
        })
        .expect("execute_test on freestanding project"),
        0
    );

    // Build debug profile via pure-Rust builder when Docker/cross containerization is available
    if !is_cross_container_available() {
        eprintln!(
            "[test] Skipping build verification in test_solix_new_default_application: Docker/cross container environment is not active."
        );
        return;
    }

    let build_arguments = BuildArgs {
        path: target_project_directory.clone(),
        platform: None,
        profile: Some("debug".to_string()),
        debug: true,
        release: false,
        test: false,
        pkg_dir: None,
        install: false,
        force: false,
        check_env: false,
        target: None,
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

    let run_cli =
        Cli::try_parse_from(["solix", "run", output_archive.to_str().expect("valid path")])
            .expect("parse run cli");
    assert_eq!(run(run_cli).expect("run debug archive"), 0);
}

#[test]
fn test_solix_new_library_no_native_and_deps() {
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let target_project_directory = temporary_directory.path().join("math_utilities");

    let new_arguments = NewArgs {
        subcommand: None,
        path: Some(target_project_directory.clone()),
        name: Some("math_utilities".to_string()),
        version: "1.2.3".to_string(),
        description: "Math utilities library".to_string(),
        license: "MIT".to_string(),
        authors: vec!["Bob Developer".to_string()],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: true,
        native: None,
        no_native: true,
        no_std: false,
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

    let manifest_path = target_project_directory.join("manifest.yaml");
    let manifest = ProjectManifest::from_file(&manifest_path).expect("parse generated manifest");
    assert_eq!(manifest.name(), "math_utilities");
    assert_eq!(manifest.version(), "1.2.3");
    let release_profile = manifest
        .resolve_profile("release")
        .expect("release profile");
    assert_eq!(
        release_profile
            .dependencies
            .get("solix")
            .map(|d| d.version.as_str()),
        Some("2.0.0")
    );
    assert_eq!(
        release_profile
            .dependencies
            .get("extra_helpers")
            .map(|d| d.version.as_str()),
        Some("0.5.1")
    );

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
        subcommand: None,
        path: Some(target_project_directory.clone()),
        name: Some("conflict_project".to_string()),
        version: "0.1.0".to_string(),
        description: "Initial".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec![],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        native: None,
        no_native: true,
        no_std: false,
        deps: vec![],
        force: false,
    };

    assert!(
        run(Cli {
            command: Commands::New(initial_arguments.clone()),
        })
        .is_ok()
    );
    assert!(
        run(Cli {
            command: Commands::New(initial_arguments),
        })
        .is_err(),
        "Expected error when overwriting without force"
    );

    let forced_arguments = NewArgs {
        subcommand: None,
        path: Some(target_project_directory.clone()),
        name: Some("conflict_project".to_string()),
        version: "0.2.0".to_string(),
        description: "Updated".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec![],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        native: None,
        no_native: true,
        no_std: false,
        deps: vec![],
        force: true,
    };

    assert!(
        run(Cli {
            command: Commands::New(forced_arguments),
        })
        .is_ok(),
        "Expected success when overwriting with --force"
    );

    let manifest_path = target_project_directory.join("manifest.yaml");
    let manifest = ProjectManifest::from_file(&manifest_path).expect("parse manifest");
    assert_eq!(manifest.version(), "0.2.0");
}

#[test]
fn test_solix_new_explicit_native_flag() {
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let target_project_directory = temporary_directory.path().join("native_app");

    let new_arguments = NewArgs {
        subcommand: None,
        path: Some(target_project_directory.clone()),
        name: Some("native_app".to_string()),
        version: "0.1.0".to_string(),
        description: "Native test project".to_string(),
        license: "Apache-2.0".to_string(),
        authors: vec![],
        main: "Main::main".to_string(),
        test: "TestRunner::test".to_string(),
        lib: false,
        native: Some("rust".to_string()),
        no_native: false,
        no_std: false,
        deps: vec![],
        force: false,
    };

    assert!(
        run(Cli {
            command: Commands::New(new_arguments),
        })
        .is_ok()
    );

    let bridge_directory = target_project_directory
        .join("natives")
        .join("native_app_bridge");
    assert!(bridge_directory.join("Cargo.toml").exists());
    assert!(bridge_directory.join("src").join("lib.rs").exists());
    let cargo_toml =
        fs::read_to_string(bridge_directory.join("Cargo.toml")).expect("read bridge Cargo.toml");
    assert!(cargo_toml.contains("solix-ffi"));
    assert!(cargo_toml.contains("solix-native = [\"dep:solix-ffi\"]"));
    assert!(cargo_toml.contains("github.com/Jehudme/solix-language.git"));
}

#[test]
fn test_solix_new_native_subcommand_c_cpp_and_rust_dual_mode() {
    let cmake_available = Command::new("cmake").arg("--version").output().is_ok();
    let temporary_directory = tempdir().expect("failed to create temp dir");
    let project_root = temporary_directory.path().join("multi_native_app");

    // 1. Create a base pure-Solix project
    let init_cli = Cli::try_parse_from([
        "solix",
        "new",
        project_root.to_str().expect("valid path"),
        "--no-native",
    ])
    .expect("parse solix new --no-native");
    assert_eq!(run(init_cli).expect("solix new --no-native"), 0);

    // 2. Add a Rust dual-mode library via `solix new native` (without requiring network!)
    let add_rust_cli = Cli::try_parse_from([
        "solix",
        "new",
        "native",
        "FastMath",
        "--project",
        project_root.to_str().expect("valid path"),
        "--backend",
        "rust",
        "--version",
        "0.3.0",
        "--platforms",
        "host",
    ])
    .expect("parse solix new native rust");
    assert_eq!(run(add_rust_cli).expect("solix new native rust"), 0);

    let rust_dir = project_root.join("natives").join("fastmath");
    assert!(rust_dir.join("Cargo.toml").is_file());
    let cargo_toml_content =
        fs::read_to_string(rust_dir.join("Cargo.toml")).expect("read rust Cargo.toml");
    assert!(
        cargo_toml_content.contains("name = \"__native_fastmath\""),
        "Generated Cargo.toml must name the library __native_fastmath"
    );
    // Standalone Cargo unit tests pass without --features solix-native
    let cargo_test_status = Command::new("cargo")
        .args(["test", "--manifest-path"])
        .arg(rust_dir.join("Cargo.toml"))
        .status()
        .expect("run cargo test on generated rust native crate");
    assert!(
        cargo_test_status.success(),
        "standalone cargo test must pass"
    );

    // 3. Add a C dual-mode CMake library
    let add_c_cli = Cli::try_parse_from([
        "solix",
        "new",
        "native",
        "CalcC",
        "--project",
        project_root.to_str().expect("valid path"),
        "--lang",
        "c",
        "--version",
        "1.2.0",
        "--platforms",
        "host",
    ])
    .expect("parse solix new native c");
    assert_eq!(run(add_c_cli).expect("solix new native c"), 0);

    // 4. Add a C++ dual-mode CMake library
    let add_cpp_cli = Cli::try_parse_from([
        "solix",
        "new",
        "native",
        "CalcCpp",
        "--project",
        project_root.to_str().expect("valid path"),
        "--lang",
        "cpp",
        "--platforms",
        "host",
    ])
    .expect("parse solix new native cpp");
    assert_eq!(run(add_cpp_cli).expect("solix new native cpp"), 0);

    let c_cmakelists = fs::read_to_string(
        project_root
            .join("natives")
            .join("calcc")
            .join("CMakeLists.txt"),
    )
    .expect("read C CMakeLists.txt");
    assert!(
        c_cmakelists.contains("add_library(__native_calcc SHARED"),
        "C CMakeLists.txt must create library __native_calcc"
    );
    let cpp_cmakelists = fs::read_to_string(
        project_root
            .join("natives")
            .join("calccpp")
            .join("CMakeLists.txt"),
    )
    .expect("read C++ CMakeLists.txt");
    assert!(
        cpp_cmakelists.contains("add_library(__native_calccpp SHARED"),
        "C++ CMakeLists.txt must create library __native_calccpp"
    );

    let updated_manifest =
        ProjectManifest::from_file(project_root.join("manifest.yaml")).expect("reload manifest");
    let release_configuration = updated_manifest
        .resolve_profile("release")
        .expect("release profile");
    assert!(release_configuration.natives.contains_key("fastmath"));
    assert!(release_configuration.natives.contains_key("calcc"));
    assert!(release_configuration.natives.contains_key("calccpp"));
    assert_eq!(release_configuration.platforms, vec!["host"]);

    // Wire Solix main.slx to call all three native libraries (Rust + C + C++)
    let combined_main = r#"
public class FastmathNative {
    native public static int32 add(int32 left_val, int32 right_val);
}

public class CalccNative {
    native public static int32 add(int32 left_val, int32 right_val);
}

public class CalccppNative {
    native public static int32 add(int32 left_val, int32 right_val);
}

public class Main {
    public static int32 main() {
        return FastmathNative::add(10, 5)
            + CalccNative::add(12, 3)
            + CalccppNative::add(8, 4);
    }
}
"#;
    fs::write(project_root.join("src").join("main.slx"), combined_main).expect("write main.slx");

    if cmake_available {
        // Standalone CMake build + CTest without SOLIX_NATIVE_BUILD on C and C++ templates
        for native_name in ["calcc", "calccpp"] {
            let native_src = project_root.join("natives").join(native_name);
            let standalone_build = temporary_directory
                .path()
                .join(format!("standalone_{native_name}"));
            let cfg_ok = Command::new("cmake")
                .args(["-S"])
                .arg(&native_src)
                .args(["-B"])
                .arg(&standalone_build)
                .arg("-DBUILD_TESTING=ON")
                .status()
                .expect("cmake configure standalone")
                .success();
            assert!(
                cfg_ok,
                "standalone cmake configure failed for {native_name}"
            );

            let build_ok = Command::new("cmake")
                .args(["--build"])
                .arg(&standalone_build)
                .args(["--config", "Release"])
                .status()
                .expect("cmake build standalone")
                .success();
            assert!(build_ok, "standalone cmake build failed for {native_name}");

            let ctest_ok = Command::new("ctest")
                .args(["--test-dir"])
                .arg(&standalone_build)
                .args(["-C", "Release", "--output-on-failure"])
                .status()
                .expect("ctest standalone")
                .success();
            assert!(ctest_ok, "standalone ctest failed for {native_name}");
        }

        // Full Solix plugin build & execution combining Rust + C + C++!
        if !is_cross_container_available() {
            eprintln!(
                "[test] Skipping containerized build verification in test_solix_new_native_subcommand_c_cpp_and_rust_dual_mode: Docker/cross container environment is not active."
            );
            return;
        }

        let build_cli = Cli::try_parse_from([
            "solix",
            "build",
            project_root.to_str().expect("valid path"),
            "--release",
            "--target",
            "host",
        ])
        .expect("parse solix build");
        assert_eq!(run(build_cli).expect("solix build release"), 0);

        let arc_path = project_root
            .join("build")
            .join("release")
            .join("multi_native_app.slxarc");
        let run_cli = Cli::try_parse_from(["solix", "run", arc_path.to_str().expect("valid path")])
            .expect("parse solix run");
        assert_eq!(run(run_cli).expect("run combined multi-native archive"), 42);

        let slxp_path = project_root
            .join("build")
            .join("release")
            .join("multi_native_app.slxp");
        assert!(
            slxp_path.is_file(),
            "Expected .slxp package container to be emitted"
        );
        let run_slxp_cli =
            Cli::try_parse_from(["solix", "run", slxp_path.to_str().expect("valid path")])
                .expect("parse solix run slxp");
        assert_eq!(
            run(run_slxp_cli).expect("run combined multi-native slxp package"),
            42
        );
    }
}

fn is_cross_container_available() -> bool {
    if solix_toolchain::verify_docker_daemon_active().is_err() {
        return false;
    }
    let Some(docker_exe) = solix_toolchain::find_executable_in_path("docker") else {
        return false;
    };
    let Ok(output) = solix_toolchain::create_process_command(&docker_exe)
        .args(["version", "--format", "{{.Server.Os}}"])
        .output()
    else {
        return false;
    };
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "linux" {
        return false;
    }
    if solix_toolchain::find_executable_in_path("cross").is_some() {
        return true;
    }
    let home_dir = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    home_dir.is_some_and(|home| {
        let cargo_bin = std::path::PathBuf::from(home).join(".cargo").join("bin");
        solix_toolchain::find_executable_in_search_paths("cross", cargo_bin.as_os_str()).is_some()
    })
}

#[test]
fn test_fmt_missing_path_errors_and_clean_dry_run_preserves_build() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let missing_file = temp_dir.path().join("does_not_exist.slx");
    let fmt_res = solix_toolchain::execute_fmt(solix_toolchain::FmtArgs {
        files: vec![missing_file],
        check: false,
    });
    assert!(
        fmt_res.is_err(),
        "solix fmt must return an error on missing path instead of formatting cwd"
    );

    let build_dir = temp_dir.path().join("build");
    fs::create_dir_all(&build_dir).expect("create build dir");
    let dummy_file = build_dir.join("sample.slxp");
    fs::write(&dummy_file, "artifact").expect("write dummy");

    solix_toolchain::execute_clean(solix_toolchain::CleanArgs {
        path: temp_dir.path().to_path_buf(),
        cache_only: false,
        dry_run: true,
    })
    .expect("clean --dry-run");
    assert!(
        dummy_file.exists(),
        "clean --dry-run must preserve build artifacts"
    );
}

#[test]
fn test_solix_init_and_add_dependency() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src");
    let main_file = src_dir.join("main.slx");
    fs::write(
        &main_file,
        "public class Main { public static int32 main() { return 0; } }\n",
    )
    .expect("write existing main.slx");

    solix_toolchain::execute_init(solix_toolchain::InitArgs {
        path: temp_dir.path().to_path_buf(),
        name: Some("existing_app".to_string()),
        force: false,
    })
    .expect("execute_init");
    assert!(temp_dir.path().join("manifest.yaml").exists());
    assert!(
        fs::read_to_string(&main_file)
            .expect("read main.slx")
            .contains("return 0;"),
        "solix init must not overwrite existing source files"
    );

    solix_toolchain::execute_add(solix_toolchain::AddArgs {
        package: "raylib@5.5.0".to_string(),
        path: temp_dir.path().to_path_buf(),
        dep_profile: "release".to_string(),
        pkg_dir: None,
    })
    .expect("execute_add");
    let manifest =
        solix_common::project::ProjectManifest::from_file(temp_dir.path().join("manifest.yaml"))
            .expect("parse updated manifest");
    let release = manifest.resolve_profile("release").expect("release");
    assert!(
        release.dependencies.contains_key("raylib"),
        "solix add must add raylib dependency to manifest.yaml"
    );

    // Invalid constraint operator must fail immediately with a clear diagnostic
    let invalid_add = solix_toolchain::execute_add(solix_toolchain::AddArgs {
        package: "raylib@=<6.0.1".to_string(),
        path: temp_dir.path().to_path_buf(),
        dep_profile: "release".to_string(),
        pkg_dir: None,
    });
    assert!(
        invalid_add.is_err(),
        "solix add with invalid operator '=<' must fail"
    );
    let invalid_msg = invalid_add.expect_err("error").to_string();
    assert!(
        invalid_msg.contains("Unsupported version constraint operator '=<'"),
        "Unexpected solix add error message: {invalid_msg}"
    );
}

#[test]
fn test_solix_add_writes_exact_semver_matching_solver_when_installed() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let store_dir = tempfile::tempdir().expect("store_dir");

    // Seed an installed package in store_dir
    let pkg_dir = store_dir.path().join("demo_pkg-release@1.2.3");
    fs::create_dir_all(&pkg_dir).expect("create pkg dir");
    let mut dummy_prog = solix_common::bytecode::BinaryProgram::new();
    dummy_prog.package_name = "demo_pkg".to_string();
    dummy_prog.build_profile = "release".to_string();
    fs::write(pkg_dir.join("demo_pkg.slxb"), dummy_prog.to_bytes()).expect("write slxb");

    solix_toolchain::execute_init(solix_toolchain::InitArgs {
        path: temp_dir.path().to_path_buf(),
        name: Some("consumer_app".to_string()),
        force: false,
    })
    .expect("execute_init");

    solix_toolchain::execute_add(solix_toolchain::AddArgs {
        package: "demo_pkg".to_string(),
        path: temp_dir.path().to_path_buf(),
        dep_profile: "release".to_string(),
        pkg_dir: Some(store_dir.path().to_path_buf()),
    })
    .expect("execute_add");

    let manifest =
        solix_common::project::ProjectManifest::from_file(temp_dir.path().join("manifest.yaml"))
            .expect("parse manifest");
    let release = manifest.resolve_profile("release").expect("release");
    let dep = release
        .dependencies
        .get("demo_pkg")
        .expect("demo_pkg present");
    assert_eq!(
        dep.version, "1.2.3",
        "solix add must emit clean semver without incompatible '>=' prefix mismatch"
    );

    let spec = format!("{}@{}#{}", "demo_pkg", dep.version, dep.profile);
    let parsed = solix_common::solver::DependencyRequirement::parse(&spec)
        .expect("DependencyRequirement::parse must accept version written by solix add");
    assert_eq!(parsed.version, solix_common::PackageVersion::new(1, 2, 3));
}
