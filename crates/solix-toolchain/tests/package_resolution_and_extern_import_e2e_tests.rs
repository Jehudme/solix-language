//! End-to-end integration tests for the Solix package manager, module solver,
//! dependency resolution phase, `extern import` dynamic loading, and cross-PC runtime verification.

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_common::bytecode::BinaryProgram;
use solix_common::project::SlxarcArchive;
use solix::cli::Cli;
use solix::run;
use std::fs;

#[test]
fn test_compile_with_deps_and_transitive_auto_upgrade() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    // 1. Compile and install math 1.1.0
    let math_v110_source = workspace
        .write_file(
            "math_110.slx",
            r#"
public class Math {
    public static int32 compute(int32 value) {
        return value * 2;
    }
}
"#,
        )
        .expect("Failed to write math 1.1.0 source");
    let math_v110_bin = workspace.file_path("math_110.slxb");
    let compile_math_v110 = Cli::try_parse_from([
        "solix",
        "compile",
        math_v110_source.to_str().expect("valid path"),
        "--name",
        "math",
        "--version",
        "1.1.0",
        "-o",
        math_v110_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse compile");
    assert_eq!(run(compile_math_v110).expect("Compile math 1.1.0"), 0);

    let install_math_v110 = Cli::try_parse_from([
        "solix",
        "install",
        math_v110_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse install");
    assert_eq!(run(install_math_v110).expect("Install math 1.1.0"), 0);

    // 2. Compile and install math 1.3.0 (compatible upgrade)
    let math_v130_source = workspace
        .write_file(
            "math_130.slx",
            r#"
public class Math {
    public static int32 compute(int32 value) {
        return value * 3;
    }
}
"#,
        )
        .expect("Failed to write math 1.3.0 source");
    let math_v130_bin = workspace.file_path("math_130.slxb");
    let compile_math_v130 = Cli::try_parse_from([
        "solix",
        "compile",
        math_v130_source.to_str().expect("valid path"),
        "--name",
        "math",
        "--version",
        "1.3.0",
        "-o",
        math_v130_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse compile");
    assert_eq!(run(compile_math_v130).expect("Compile math 1.3.0"), 0);

    let install_math_v130 = Cli::try_parse_from([
        "solix",
        "install",
        math_v130_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse install");
    assert_eq!(run(install_math_v130).expect("Install math 1.3.0"), 0);

    // 3. Compile and install engine 1.0.0 which depends on math@1.1.0
    let engine_source = workspace
        .write_file(
            "engine.slx",
            r#"
import math.*;

public class Engine {
    public static int32 run_engine(int32 seed) {
        return Math.compute(seed) + 5;
    }
}
"#,
        )
        .expect("Failed to write engine source");
    let engine_bin = workspace.file_path("engine.slxb");
    let compile_engine = Cli::try_parse_from([
        "solix",
        "compile",
        engine_source.to_str().expect("valid path"),
        "--name",
        "engine",
        "--version",
        "1.0.0",
        "--deps",
        "math@1.1.0",
        "-o",
        engine_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse compile engine");
    assert_eq!(run(compile_engine).expect("Compile engine"), 0);

    let install_engine = Cli::try_parse_from([
        "solix",
        "install",
        engine_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse install engine");
    assert_eq!(run(install_engine).expect("Install engine"), 0);

    // 4. Compile app depending on engine@1.0.0
    let app_source = workspace
        .write_file(
            "app.slx",
            r#"
import engine.*;

public class MainApp {
    public static int32 main() {
        return Engine.run_engine(10);
    }
}
"#,
        )
        .expect("Failed to write app source");
    let app_bin = workspace.file_path("app.slxb");
    let compile_app = Cli::try_parse_from([
        "solix",
        "compile",
        app_source.to_str().expect("valid path"),
        "--name",
        "app",
        "--deps",
        "engine@1.0.0",
        "-o",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse compile app");
    assert_eq!(run(compile_app).expect("Compile app"), 0);

    // Verify dependencies embedded in app.slxb
    let app_bytes = fs::read(&app_bin).expect("Read app.slxb");
    let app_program = BinaryProgram::from_bytes(&app_bytes).expect("Deserialize app.slxb");
    let dependency_names: Vec<String> = app_program
        .export_manifest
        .dependencies
        .iter()
        .map(|req| format!("{}@{}", req.name, req.version))
        .collect();

    assert!(
        dependency_names.contains(&"engine@1.0.0".to_string()),
        "App must embed engine@1.0.0 dependency requirement: {dependency_names:?}"
    );
    assert!(
        dependency_names.contains(&"math@1.3.0".to_string()),
        "App must embed upgraded math@1.3.0 dependency requirement: {dependency_names:?}"
    );

    // Run app.slxb: Engine.run_engine(10) -> Math.compute(10) [30] + 5 = 35
    let run_app = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("Failed to parse run app");
    assert_eq!(run(run_app).expect("Run app"), 35);
}

#[test]
fn test_diamond_dependency_conflict_fails_with_incompatible_version() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    // Core 1.2.0
    let core_v120_source = workspace
        .write_file(
            "core_v120.slx",
            "public class Core { public static int32 val() { return 120; } }",
        )
        .expect("write core v120");
    let core_v120_bin = workspace.file_path("core_v120.slxb");
    let compile_core_v120 = Cli::try_parse_from([
        "solix",
        "compile",
        core_v120_source.to_str().expect("valid path"),
        "--name",
        "core",
        "--version",
        "1.2.0",
        "-o",
        core_v120_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile core v120");
    assert_eq!(run(compile_core_v120).expect("compile"), 0);

    let install_core_v120 = Cli::try_parse_from([
        "solix",
        "install",
        core_v120_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install core v120");
    assert_eq!(run(install_core_v120).expect("install"), 0);

    // Core 2.0.0 (conflicting major)
    let core_v200_source = workspace
        .write_file(
            "core_v200.slx",
            "public class Core { public static int32 val() { return 200; } }",
        )
        .expect("write core v200");
    let core_v200_bin = workspace.file_path("core_v200.slxb");
    let compile_core_v200 = Cli::try_parse_from([
        "solix",
        "compile",
        core_v200_source.to_str().expect("valid path"),
        "--name",
        "core",
        "--version",
        "2.0.0",
        "-o",
        core_v200_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile core v200");
    assert_eq!(run(compile_core_v200).expect("compile"), 0);

    let install_core_v200 = Cli::try_parse_from([
        "solix",
        "install",
        core_v200_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install core v200");
    assert_eq!(run(install_core_v200).expect("install"), 0);

    // LibA depends on core@1.2.0
    let lib_a_source = workspace
        .write_file(
            "lib_a.slx",
            "import core.*; public class LibA { public static int32 a() { return Core.val(); } }",
        )
        .expect("write lib a");
    let lib_a_bin = workspace.file_path("lib_a.slxb");
    let compile_lib_a = Cli::try_parse_from([
        "solix",
        "compile",
        lib_a_source.to_str().expect("valid path"),
        "--name",
        "lib_a",
        "--version",
        "1.0.0",
        "--deps",
        "core@1.2.0",
        "-o",
        lib_a_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile lib a");
    assert_eq!(run(compile_lib_a).expect("compile lib a"), 0);

    let install_lib_a = Cli::try_parse_from([
        "solix",
        "install",
        lib_a_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install lib a");
    assert_eq!(run(install_lib_a).expect("install lib a"), 0);

    // LibB depends on core@2.0.0
    let lib_b_source = workspace
        .write_file(
            "lib_b.slx",
            "import core.*; public class LibB { public static int32 b() { return Core.val(); } }",
        )
        .expect("write lib b");
    let lib_b_bin = workspace.file_path("lib_b.slxb");
    let compile_lib_b = Cli::try_parse_from([
        "solix",
        "compile",
        lib_b_source.to_str().expect("valid path"),
        "--name",
        "lib_b",
        "--version",
        "1.0.0",
        "--deps",
        "core@2.0.0",
        "-o",
        lib_b_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile lib b");
    assert_eq!(run(compile_lib_b).expect("compile lib b"), 0);

    let install_lib_b = Cli::try_parse_from([
        "solix",
        "install",
        lib_b_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install lib b");
    assert_eq!(run(install_lib_b).expect("install lib b"), 0);

    // App attempts to depend on both lib_a@1.0.0 and lib_b@1.0.0
    let app_conflict_source = workspace
        .write_file(
            "app_conflict.slx",
            "import lib_a.*; import lib_b.*; public class App { public static int32 main() { return LibA.a() + LibB.b(); } }",
        )
        .expect("write app conflict");
    let app_conflict_bin = workspace.file_path("app_conflict.slxb");
    let compile_conflict = Cli::try_parse_from([
        "solix",
        "compile",
        app_conflict_source.to_str().expect("valid path"),
        "--name",
        "app_conflict",
        "--deps",
        "lib_a@1.0.0",
        "--deps",
        "lib_b@1.0.0",
        "-o",
        app_conflict_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("parse compile conflict");

    let compile_result = run(compile_conflict);
    assert!(
        compile_result.is_err(),
        "Compilation must fail due to conflicting major version requirements in dependency tree"
    );
}

#[test]
fn test_diamond_compatible_deduplication_cleanly_resolves() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    // Core 1.1.0 and Core 1.3.0
    let core_v110_source = workspace
        .write_file(
            "core_v110.slx",
            "public class Core { public static int32 val() { return 10; } }",
        )
        .expect("write core v110");
    let core_v110_bin = workspace.file_path("core_v110.slxb");
    let compile_core_v110 = Cli::try_parse_from([
        "solix",
        "compile",
        core_v110_source.to_str().expect("valid path"),
        "--name",
        "core",
        "--version",
        "1.1.0",
        "-o",
        core_v110_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile core v110");
    assert_eq!(run(compile_core_v110).expect("compile"), 0);

    let install_core_v110 = Cli::try_parse_from([
        "solix",
        "install",
        core_v110_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install core v110");
    assert_eq!(run(install_core_v110).expect("install"), 0);

    let core_v130_source = workspace
        .write_file(
            "core_v130.slx",
            "public class Core { public static int32 val() { return 20; } }",
        )
        .expect("write core v130");
    let core_v130_bin = workspace.file_path("core_v130.slxb");
    let compile_core_v130 = Cli::try_parse_from([
        "solix",
        "compile",
        core_v130_source.to_str().expect("valid path"),
        "--name",
        "core",
        "--version",
        "1.3.0",
        "-o",
        core_v130_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile core v130");
    assert_eq!(run(compile_core_v130).expect("compile"), 0);

    let install_core_v130 = Cli::try_parse_from([
        "solix",
        "install",
        core_v130_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install core v130");
    assert_eq!(run(install_core_v130).expect("install"), 0);

    // LibA requires core@1.1.0
    let lib_a_source = workspace
        .write_file(
            "lib_a.slx",
            "import core.*; public class LibA { public static int32 a() { return Core.val(); } }",
        )
        .expect("write lib a");
    let lib_a_bin = workspace.file_path("lib_a.slxb");
    let compile_lib_a = Cli::try_parse_from([
        "solix",
        "compile",
        lib_a_source.to_str().expect("valid path"),
        "--name",
        "lib_a",
        "--version",
        "1.0.0",
        "--deps",
        "core@1.1.0",
        "-o",
        lib_a_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile lib a");
    assert_eq!(run(compile_lib_a).expect("compile lib a"), 0);

    let install_lib_a = Cli::try_parse_from([
        "solix",
        "install",
        lib_a_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install lib a");
    assert_eq!(run(install_lib_a).expect("install lib a"), 0);

    // LibB requires core@1.3.0
    let lib_b_source = workspace
        .write_file(
            "lib_b.slx",
            "import core.*; public class LibB { public static int32 b() { return Core.val() + 1; } }",
        )
        .expect("write lib b");
    let lib_b_bin = workspace.file_path("lib_b.slxb");
    let compile_lib_b = Cli::try_parse_from([
        "solix",
        "compile",
        lib_b_source.to_str().expect("valid path"),
        "--name",
        "lib_b",
        "--version",
        "1.0.0",
        "--deps",
        "core@1.3.0",
        "-o",
        lib_b_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile lib b");
    assert_eq!(run(compile_lib_b).expect("compile lib b"), 0);

    let install_lib_b = Cli::try_parse_from([
        "solix",
        "install",
        lib_b_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install lib b");
    assert_eq!(run(install_lib_b).expect("install lib b"), 0);

    // App depends on both LibA and LibB -> deduplicates to core@1.3.0 with zero duplicate symbols
    let app_diamond_source = workspace
        .write_file(
            "app_diamond.slx",
            r#"
import lib_a.*;
import lib_b.*;

public class AppDiamond {
    public static int32 main() {
        return LibA.a() + LibB.b();
    }
}
"#,
        )
        .expect("write app diamond");
    let app_diamond_bin = workspace.file_path("app_diamond.slxb");
    let compile_diamond = Cli::try_parse_from([
        "solix",
        "compile",
        app_diamond_source.to_str().expect("valid path"),
        "--name",
        "app_diamond",
        "--deps",
        "lib_a@1.0.0",
        "--deps",
        "lib_b@1.0.0",
        "-o",
        app_diamond_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("parse compile diamond");
    assert_eq!(
        run(compile_diamond).expect("compile diamond must succeed"),
        0
    );

    // Run diamond app: Core.val() is 20, LibA.a() is 20, LibB.b() is 21 -> 20 + 21 = 41
    let run_diamond = Cli::try_parse_from([
        "solix",
        "run",
        app_diamond_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("parse run diamond");
    assert_eq!(run(run_diamond).expect("run diamond"), 41);
}

#[test]
fn test_extern_import_on_the_fly_solver_fetching_and_root_facade() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let package_store_directory = workspace.root_path().join("store");
    fs::create_dir_all(&package_store_directory).expect("Failed to create store dir");
    let store_string = package_store_directory.to_str().expect("valid path");

    // Install raylib package into the store with top-level package function initWindow
    let raylib_source = workspace
        .write_file(
            "raylib.slx",
            r#"
package raylib;

public int32 initWindow(int32 width, int32 height) {
    return width + height;
}
"#,
        )
        .expect("write raylib source");
    let raylib_bin = workspace.file_path("raylib.slxb");
    let compile_raylib = Cli::try_parse_from([
        "solix",
        "compile",
        raylib_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "1.2.0",
        "-o",
        raylib_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile raylib");
    assert_eq!(run(compile_raylib).expect("compile raylib"), 0);

    let install_raylib = Cli::try_parse_from([
        "solix",
        "install",
        raylib_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("install raylib");
    assert_eq!(run(install_raylib).expect("install raylib"), 0);

    // User source with extern import and NO --deps flag
    let game_source = workspace
        .write_file(
            "game.slx",
            r#"
extern import raylib.initWindow;
extern import raylib.*;

public class GameApp {
    public static int32 main() {
        int32 size_one = raylib.initWindow(800, 600);
        int32 size_two = initWindow(100, 50);
        return size_one + size_two;
    }
}
"#,
        )
        .expect("write game source");
    let game_bin = workspace.file_path("game.slxb");

    // Notice: NO --deps flag passed! Parser phase dynamically queries solver for raylib
    let compile_game = Cli::try_parse_from([
        "solix",
        "compile",
        game_source.to_str().expect("valid path"),
        "--name",
        "game",
        "-o",
        game_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("compile game");
    assert_eq!(run(compile_game).expect("compile game"), 0);

    // Verify dependencies recorded in game.slxb
    let game_bytes = fs::read(&game_bin).expect("Read game.slxb");
    let game_program = BinaryProgram::from_bytes(&game_bytes).expect("Deserialize game.slxb");
    assert!(
        game_program
            .export_manifest
            .dependencies
            .iter()
            .any(|req| req.name == "raylib" && req.version.to_string() == "1.2.0"),
        "game.slxb must record raylib@1.2.0 dependency"
    );

    // Execute game.slxb: (800 + 600) + (100 + 50) = 1400 + 150 = 1550
    let run_game = Cli::try_parse_from([
        "solix",
        "run",
        game_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("run game");
    assert_eq!(run(run_game).expect("run game"), 1550);
}

#[test]
fn test_cross_pc_runtime_dependency_validation_all_scenarios() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");

    // Environment A: Compile machine (PC A)
    let pc_a_store = workspace.root_path().join("store_pc_a");
    fs::create_dir_all(&pc_a_store).expect("create pc_a store");
    let pc_a_store_str = pc_a_store.to_str().expect("valid path");

    // Create raylib 1.2.0 archive on PC A
    let raylib_source = workspace
        .write_file(
            "raylib.slx",
            r#"
package raylib;

public int32 run_graphics() {
    return 77;
}
"#,
        )
        .expect("write raylib");
    let raylib_stage = workspace.root_path().join("raylib_stage");
    let raylib_bin_dir = raylib_stage.join("bin");
    fs::create_dir_all(&raylib_bin_dir).expect("create raylib bin");
    fs::write(
        raylib_stage.join("manifest.toml"),
        r#"
[info]
name = "raylib"
version = "1.2.0"
description = "Raylib cross-pc"
"#,
    )
    .expect("write raylib manifest");

    let raylib_v120_bin = raylib_bin_dir.join("raylib.slxb");
    let compile_raylib = Cli::try_parse_from([
        "solix",
        "compile",
        raylib_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "1.2.0",
        "-o",
        raylib_v120_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_a_store_str,
    ])
    .expect("compile raylib");
    assert_eq!(run(compile_raylib).expect("compile raylib"), 0);

    let raylib_v120_archive = workspace.file_path("raylib_120.slxarc");
    SlxarcArchive::pack_directory(&raylib_stage, &raylib_v120_archive).expect("pack archive");

    let install_raylib_pc_a = Cli::try_parse_from([
        "solix",
        "install",
        raylib_v120_archive.to_str().expect("valid path"),
        "--pkg-dir",
        pc_a_store_str,
    ])
    .expect("install on pc a");
    assert_eq!(run(install_raylib_pc_a).expect("install raylib pc a"), 0);

    // Compile app.slxb on PC A
    let app_source = workspace
        .write_file(
            "app_target.slx",
            r#"
import raylib.*;

public class App {
    public static int32 main() {
        return raylib.run_graphics();
    }
}
"#,
        )
        .expect("write app");
    let app_bin = workspace.file_path("app_target.slxb");
    let compile_app = Cli::try_parse_from([
        "solix",
        "compile",
        app_source.to_str().expect("valid path"),
        "--name",
        "app_target",
        "--deps",
        "raylib@1.2.0",
        "-o",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_a_store_str,
    ])
    .expect("compile app on pc a");
    assert_eq!(run(compile_app).expect("compile app"), 0);

    // Environment B: Target machine (PC B)
    let pc_b_store = workspace.root_path().join("store_pc_b");
    fs::create_dir_all(&pc_b_store).expect("create pc_b store");
    let pc_b_store_str = pc_b_store.to_str().expect("valid path");

    // SCENARIO 1: raylib missing on PC B -> fails immediately
    let run_scenario_1 = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("parse run scenario 1");
    let result_scenario_1 = run(run_scenario_1);
    assert!(
        result_scenario_1.is_err(),
        "Scenario 1: Running on PC B with missing dependency must fail"
    );
    let error_message_1 = format!("{:?}", result_scenario_1.err());
    assert!(
        error_message_1.contains("raylib") && error_message_1.contains("1.2.0"),
        "Scenario 1 error must mention missing raylib@1.2.0: {error_message_1}"
    );

    // SCENARIO 2: Older incompatible version raylib 1.1.0 on PC B -> fails
    let raylib_v110_stage = workspace.root_path().join("raylib_v110_stage");
    let raylib_v110_bin_dir = raylib_v110_stage.join("bin");
    fs::create_dir_all(&raylib_v110_bin_dir).expect("create v110 bin dir");
    fs::write(
        raylib_v110_stage.join("manifest.toml"),
        r#"
[info]
name = "raylib"
version = "1.1.0"
"#,
    )
    .expect("write v110 manifest");
    let raylib_v110_bin = raylib_v110_bin_dir.join("raylib.slxb");
    let compile_v110 = Cli::try_parse_from([
        "solix",
        "compile",
        raylib_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "1.1.0",
        "-o",
        raylib_v110_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("compile v110");
    assert_eq!(run(compile_v110).expect("compile"), 0);

    let raylib_v110_archive = workspace.file_path("raylib_110.slxarc");
    SlxarcArchive::pack_directory(&raylib_v110_stage, &raylib_v110_archive).expect("pack v110");
    let install_v110 = Cli::try_parse_from([
        "solix",
        "install",
        raylib_v110_archive.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("install v110");
    assert_eq!(run(install_v110).expect("install"), 0);

    let run_scenario_2 = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("parse run scenario 2");
    let result_scenario_2 = run(run_scenario_2);
    assert!(
        result_scenario_2.is_err(),
        "Scenario 2: Older installed version 1.1.0 must fail against required 1.2.0"
    );

    // Clean up v110 from PC B
    let uninstall_v110 =
        Cli::try_parse_from(["solix", "uninstall", "raylib", "--pkg-dir", pc_b_store_str])
            .expect("uninstall v110");
    assert_eq!(run(uninstall_v110).expect("uninstall"), 0);

    // SCENARIO 3: Major version mismatch raylib 2.0.0 on PC B -> fails
    let raylib_v200_stage = workspace.root_path().join("raylib_v200_stage");
    let raylib_v200_bin_dir = raylib_v200_stage.join("bin");
    fs::create_dir_all(&raylib_v200_bin_dir).expect("create v200 bin dir");
    fs::write(
        raylib_v200_stage.join("manifest.toml"),
        r#"
[info]
name = "raylib"
version = "2.0.0"
"#,
    )
    .expect("write v200 manifest");
    let raylib_v200_bin = raylib_v200_bin_dir.join("raylib.slxb");
    let compile_v200 = Cli::try_parse_from([
        "solix",
        "compile",
        raylib_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "2.0.0",
        "-o",
        raylib_v200_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("compile v200");
    assert_eq!(run(compile_v200).expect("compile"), 0);

    let raylib_v200_archive = workspace.file_path("raylib_200.slxarc");
    SlxarcArchive::pack_directory(&raylib_v200_stage, &raylib_v200_archive).expect("pack v200");
    let install_v200 = Cli::try_parse_from([
        "solix",
        "install",
        raylib_v200_archive.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("install v200");
    assert_eq!(run(install_v200).expect("install"), 0);

    let run_scenario_3 = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("parse run scenario 3");
    let result_scenario_3 = run(run_scenario_3);
    assert!(
        result_scenario_3.is_err(),
        "Scenario 3: Major version 2.0.0 must fail against required 1.2.0"
    );

    // Clean up v200 from PC B
    let uninstall_v200 =
        Cli::try_parse_from(["solix", "uninstall", "raylib", "--pkg-dir", pc_b_store_str])
            .expect("uninstall v200");
    assert_eq!(run(uninstall_v200).expect("uninstall"), 0);

    // SCENARIO 4: Newer compatible version raylib 1.4.2 on PC B -> succeeds
    let raylib_v142_stage = workspace.root_path().join("raylib_v142_stage");
    let raylib_v142_bin_dir = raylib_v142_stage.join("bin");
    fs::create_dir_all(&raylib_v142_bin_dir).expect("create v142 bin dir");
    fs::write(
        raylib_v142_stage.join("manifest.toml"),
        r#"
[info]
name = "raylib"
version = "1.4.2"
"#,
    )
    .expect("write v142 manifest");
    let raylib_v142_bin = raylib_v142_bin_dir.join("raylib.slxb");
    let compile_v142 = Cli::try_parse_from([
        "solix",
        "compile",
        raylib_source.to_str().expect("valid path"),
        "--name",
        "raylib",
        "--version",
        "1.4.2",
        "-o",
        raylib_v142_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("compile v142");
    assert_eq!(run(compile_v142).expect("compile"), 0);

    let raylib_v142_archive = workspace.file_path("raylib_142.slxarc");
    SlxarcArchive::pack_directory(&raylib_v142_stage, &raylib_v142_archive).expect("pack v142");
    let install_v142 = Cli::try_parse_from([
        "solix",
        "install",
        raylib_v142_archive.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("install v142");
    assert_eq!(run(install_v142).expect("install"), 0);

    // Scenario 4a: Without --yolo, running app pinned to raylib@1.2.0 against raylib@1.4.2
    // must fail cryptographic checksum verification (Milestone 026 security invariant).
    let run_scenario_4_strict = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
    ])
    .expect("parse run scenario 4 strict");
    assert!(
        run(run_scenario_4_strict).is_err(),
        "Scenario 4 strict: Running without --yolo must fail checksum verification on upgraded package"
    );

    // Scenario 4b: With --yolo, checksum verification is bypassed, allowing SemVer compatible raylib@1.4.2 to run.
    let run_scenario_4_yolo = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().expect("valid path"),
        "--pkg-dir",
        pc_b_store_str,
        "--yolo",
    ])
    .expect("parse run scenario 4 yolo");
    assert_eq!(
        run(run_scenario_4_yolo).expect("Scenario 4 with --yolo must succeed"),
        77,
        "Scenario 4: Executing app against installed raylib@1.4.2 with --yolo must return 77"
    );
}
