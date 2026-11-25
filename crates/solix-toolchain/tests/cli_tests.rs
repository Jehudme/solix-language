//! Integration test suite for the Solix command-line driver (`solix`).

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_toolchain::cli::{Cli, Commands};
use solix_toolchain::run;

#[test]
fn test_hermetic_workspace_initialization() {
    let workspace =
        HermeticCliWorkspace::create().expect("Failed to initialize isolated temporary workspace");
    assert!(workspace.root_path().exists());
}

#[test]
fn test_cli_compile_single_source_and_run_exit_code() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "main.slx",
            r#"
class Main {
    public static int32 main() {
        return 42;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("main.slxb");

    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "-o",
        bin.to_str().unwrap(),
    ])
    .expect("Failed to parse compile CLI arguments");

    assert!(run(cli).is_ok());
    assert!(bin.exists());

    let run_cli = Cli::try_parse_from(["solix", "run", bin.to_str().unwrap()])
        .expect("Failed to parse run CLI arguments");

    let exit_code = run(run_cli).expect("Execution failed");
    assert_eq!(exit_code, 42);
}

#[test]
fn test_cli_compile_simple_program() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "hello.slx",
            r#"
class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("hello.slxb");

    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "-o",
        bin.to_str().unwrap(),
    ])
    .unwrap();

    assert!(run(cli).is_ok());
    assert!(bin.exists());

    let run_cli = Cli::try_parse_from(["solix", "run", bin.to_str().unwrap()]).unwrap();
    let exit_code = run(run_cli).expect("Execution failed");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_cli_compile_check_only_does_not_emit_binary() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "check_target.slx",
            r#"
class Main {
    public static int32 main() {
        return 1;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("check_target.slxb");

    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "--check",
        "--time",
        "-o",
        bin.to_str().unwrap(),
    ])
    .unwrap();

    let result = run(cli);
    assert!(result.is_ok());
    // --check must not write the binary container
    assert!(!bin.exists());
}

#[test]
fn test_cli_compile_emit_api_interface() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "math.slx",
            r#"
public class MathLib {
    public static int32 add(int32 a, int32 b) {
        return a + b;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("math.slxb");
    let api_out = ws.file_path("math_api.slx");

    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "--name",
        "my_math",
        "--version",
        "1.2.0",
        "--emit-api",
        api_out.to_str().unwrap(),
        "-o",
        bin.to_str().unwrap(),
    ])
    .unwrap();

    assert!(run(cli).is_ok());
    assert!(bin.exists());
    assert!(api_out.exists());

    let api_content = ws.read_file("math_api.slx").unwrap();
    assert!(api_content.contains("class MathLib"));
    assert!(api_content.contains("add"));
}

#[test]
fn test_cli_compile_dynamic_dependency_lib_auto_extract_and_multi_binary_run() {
    let ws = HermeticCliWorkspace::create().unwrap();

    // 1. Compile library binary
    let lib_src = ws
        .write_file(
            "greeter_lib.slx",
            r#"
public class Greeter {
    public static int32 calculate_code(int32 seed) {
        return seed * 3;
    }
}
"#,
        )
        .unwrap();

    let lib_bin = ws.file_path("greeter.slxb");
    let lib_compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        lib_src.to_str().unwrap(),
        "--name",
        "greeter",
        "-o",
        lib_bin.to_str().unwrap(),
    ])
    .unwrap();

    assert!(run(lib_compile_cli).is_ok());
    assert!(lib_bin.exists());

    // 2. Compile application consuming greeter via `-l, --lib`
    let app_src = ws
        .write_file(
            "app.slx",
            r#"
class Main {
    public static int32 main() {
        return Greeter::calculate_code(14);
    }
}
"#,
        )
        .unwrap();

    let app_bin = ws.file_path("app.slxb");
    let app_compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        app_src.to_str().unwrap(),
        "-l",
        lib_bin.to_str().unwrap(),
        "-o",
        app_bin.to_str().unwrap(),
    ])
    .unwrap();

    assert!(run(app_compile_cli).is_ok());
    assert!(app_bin.exists());

    // 3. Run VM with both binaries loaded as equal peers
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        app_bin.to_str().unwrap(),
        lib_bin.to_str().unwrap(),
    ])
    .unwrap();

    let exit_code = run(run_cli).expect("Execution failed");
    assert_eq!(exit_code, 42); // 14 * 3 = 42
}

#[test]
fn test_cli_run_multi_binary_peer_order_invariance() {
    let ws = HermeticCliWorkspace::create().unwrap();

    let lib_src = ws
        .write_file(
            "calc.slx",
            r#"
public class Calc {
    public static int32 get_answer() {
        return 84;
    }
}
"#,
        )
        .unwrap();

    let lib_bin = ws.file_path("calc.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            lib_src.to_str().unwrap(),
            "-o",
            lib_bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    let app_src = ws
        .write_file(
            "runner.slx",
            r#"
class Main {
    public static int32 main() {
        return Calc::get_answer();
    }
}
"#,
        )
        .unwrap();

    let app_bin = ws.file_path("runner.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            app_src.to_str().unwrap(),
            "-l",
            lib_bin.to_str().unwrap(),
            "-o",
            app_bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    // Pass library binary FIRST, application binary SECOND:
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        lib_bin.to_str().unwrap(),
        app_bin.to_str().unwrap(),
    ])
    .unwrap();

    let exit_code = run(run_cli).expect("Execution failed with reversed binary load order");
    assert_eq!(exit_code, 84);
}

#[test]
fn test_cli_run_entry_point_override() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "multi_entry.slx",
            r#"
class App {
    public static int32 main() {
        return 10;
    }

    public static int32 secondary() {
        return 99;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("multi_entry.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            src.to_str().unwrap(),
            "-o",
            bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    // Default entry returns 10
    let run_default = Cli::try_parse_from(["solix", "run", bin.to_str().unwrap()]).unwrap();
    assert_eq!(run(run_default).unwrap(), 10);

    // Overridden entry returns 99
    let run_override = Cli::try_parse_from([
        "solix",
        "run",
        bin.to_str().unwrap(),
        "--entry",
        "App::secondary",
    ])
    .unwrap();
    assert_eq!(run(run_override).unwrap(), 99);
}

#[test]
fn test_cli_run_heap_size_and_verbose() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "heap_test.slx",
            r#"
class Main {
    public static int32 main() {
        return 7;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("heap_test.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            src.to_str().unwrap(),
            "-o",
            bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        bin.to_str().unwrap(),
        "--heap-size",
        "16MB",
        "--max-stack",
        "512",
        "--verbose",
    ])
    .unwrap();

    assert_eq!(run(run_cli).unwrap(), 7);
}

#[test]
fn test_cli_run_passes_cli_arguments_to_entry_point() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "args_test.slx",
            r#"
class Main {
    public static int32 main(string[] args) {
        return 0;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("args_test.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            src.to_str().unwrap(),
            "-o",
            bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        bin.to_str().unwrap(),
        "--",
        "alpha",
        "beta",
        "gamma",
    ])
    .unwrap();

    assert_eq!(run(run_cli).unwrap(), 0);
}

#[test]
fn test_cli_info_summary_json_api_and_disasm_to_file() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "inspect_target.slx",
            r#"
public class Service {
    public static int32 compute(int32 x) {
        return x * 2;
    }
}

class Main {
    public static int32 main() {
        return Service::compute(21);
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("demo.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            src.to_str().unwrap(),
            "--name",
            "demo_pkg",
            "--version",
            "0.2.0",
            "--description",
            "Inspection demo package",
            "-o",
            bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    // 1. Info summary
    let info_cli = Cli::try_parse_from(["solix", "info", bin.to_str().unwrap()]).unwrap();
    assert!(run(info_cli).is_ok());

    // 2. Info JSON
    let info_json =
        Cli::try_parse_from(["solix", "info", bin.to_str().unwrap(), "--json"]).unwrap();
    assert!(run(info_json).is_ok());

    // 3. Info API
    let info_api = Cli::try_parse_from(["solix", "info", bin.to_str().unwrap(), "--api"]).unwrap();
    assert!(run(info_api).is_ok());

    // 4. Info Disasm to file
    let disasm_file = ws.file_path("disasm_output.txt");
    let info_disasm = Cli::try_parse_from([
        "solix",
        "info",
        bin.to_str().unwrap(),
        "--disasm",
        "-o",
        disasm_file.to_str().unwrap(),
    ])
    .unwrap();

    assert!(run(info_disasm).is_ok());
    assert!(disasm_file.exists());

    let disasm_content = ws.read_file("disasm_output.txt").unwrap();
    assert!(!disasm_content.is_empty());
    assert!(disasm_content.contains("Function") || disasm_content.contains("compute"));
}

#[test]
fn test_cli_compile_negative_syntax_error_fails() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "syntax_error.slx",
            r#"
class BadSyntax {
    public void broken() {
        let x = ; // missing expression
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("broken.slxb");
    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "-o",
        bin.to_str().unwrap(),
    ])
    .unwrap();

    let res = run(cli);
    assert!(res.is_err());
    assert!(!bin.exists());
}

#[test]
fn test_cli_compile_negative_missing_source_file() {
    let cli = Cli::try_parse_from(["solix", "compile", "non_existent_source.slx"]).unwrap();
    assert!(run(cli).is_err());
}

#[test]
fn test_cli_compile_negative_missing_lib_file() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "app.slx",
            r#"
class Main {
    public static int32 main() { return 0; }
}
"#,
        )
        .unwrap();

    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "-l",
        "non_existent_lib.slxb",
    ])
    .unwrap();

    assert!(run(cli).is_err());
}

#[test]
fn test_cli_run_negative_missing_binary() {
    let cli = Cli::try_parse_from(["solix", "run", "missing.slxb"]).unwrap();
    assert!(run(cli).is_err());
}

#[test]
fn test_cli_run_negative_missing_entry_point() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "no_main.slx",
            r#"
class Helper {
    public int32 x;
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("no_main.slxb");
    assert!(
        run(Cli::try_parse_from([
            "solix",
            "compile",
            src.to_str().unwrap(),
            "-o",
            bin.to_str().unwrap(),
        ])
        .unwrap())
        .is_ok()
    );

    let run_cli = Cli::try_parse_from(["solix", "run", bin.to_str().unwrap()]).unwrap();
    assert!(run(run_cli).is_err());
}

#[test]
fn test_cli_info_negative_corrupted_binary() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let corrupt_bin = ws
        .write_file("corrupt.slxb", "CORRUPTED_NOT_SLXB_CONTAINER")
        .unwrap();

    let info_cli = Cli::try_parse_from(["solix", "info", corrupt_bin.to_str().unwrap()]).unwrap();
    assert!(run(info_cli).is_err());
}

#[test]
fn test_cli_parse_flags_and_defaults() {
    // 1. Compile defaults
    let compile_cli = Cli::try_parse_from(["solix", "compile", "app.slx"]).unwrap();
    if let Commands::Compile(args) = compile_cli.command {
        assert_eq!(args.sources.len(), 1);
        assert_eq!(args.version, "0.1.0");
        assert!(!args.check);
        assert!(!args.time);
    } else {
        panic!("Expected Compile subcommand");
    }

    // 2. Compile customized
    let compile_custom = Cli::try_parse_from([
        "solix",
        "compile",
        "a.slx",
        "b.slx",
        "-o",
        "out.slxb",
        "-l",
        "dep1.slxb",
        "--lib",
        "dep2.slxb",
        "--name",
        "my_package",
        "-V",
        "2.0.0",
        "--description",
        "Cool app",
        "--check",
        "--time",
    ])
    .unwrap();
    if let Commands::Compile(args) = compile_custom.command {
        assert_eq!(args.sources.len(), 2);
        assert_eq!(args.libs.len(), 2);
        assert_eq!(args.name.as_deref(), Some("my_package"));
        assert_eq!(args.version, "2.0.0");
        assert_eq!(args.description.as_deref(), Some("Cool app"));
        assert!(args.check);
        assert!(args.time);
    } else {
        panic!("Expected Compile subcommand");
    }

    // 3. Run flags
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        "bin1.slxb",
        "bin2.slxb",
        "--entry",
        "Custom::run",
        "--heap-size",
        "128MB",
        "--max-stack",
        "4096",
        "--verbose",
        "--safe",
        "--",
        "arg1",
        "arg2",
    ])
    .unwrap();
    if let Commands::Run(args) = run_cli.command {
        assert_eq!(args.binaries.len(), 2);
        assert_eq!(args.entry.as_deref(), Some("Custom::run"));
        assert_eq!(args.heap_size, "128MB");
        assert_eq!(args.max_stack, 4096);
        assert!(args.verbose);
        assert!(args.safe);
        assert_eq!(args.args, vec!["arg1", "arg2"]);
    } else {
        panic!("Expected Run subcommand");
    }

    // 4. Info flags
    let info_cli =
        Cli::try_parse_from(["solix", "info", "app.slxb", "-o", "info.txt", "--disasm"]).unwrap();
    if let Commands::Info(args) = info_cli.command {
        assert_eq!(args.target.to_str().unwrap(), "app.slxb");
        assert!(args.output.is_some());
        assert!(args.disasm);
        assert!(!args.json);
        assert!(!args.api);
    } else {
        panic!("Expected Info subcommand");
    }
}

#[test]
fn test_cli_run_safe_mode_execution() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "safe_main.slx",
            r#"
class SafeApp {
    public static int32 main() {
        return 77;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("safe_main.slxb");

    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "-o",
        bin.to_str().unwrap(),
    ])
    .expect("Failed to parse compile CLI arguments");

    assert!(run(compile_cli).is_ok());

    let run_cli = Cli::try_parse_from(["solix", "run", "--safe", bin.to_str().unwrap()])
        .expect("Failed to parse run CLI arguments with --safe");

    let result = run(run_cli);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 77);
}

#[test]
fn test_cli_run_direct_source_execution_single_file() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let source_path = workspace
        .write_file(
            "direct_main.slx",
            r#"
class DirectApp {
    public static int32 main() {
        return 42;
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from(["solix", "run", source_path.to_str().unwrap()])
        .expect("Failed to parse run CLI arguments for direct source execution");

    let execution_result = run(run_cli);
    assert!(execution_result.is_ok());
    assert_eq!(execution_result.unwrap(), 42);
}

#[test]
fn test_cli_run_direct_source_execution_multi_file() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let helper_path = workspace
        .write_file(
            "helper.slx",
            r#"
package my.calc;

public class MathHelper {
    public static int32 add(int32 left_operand, int32 right_operand) {
        return left_operand + right_operand;
    }
}
"#,
        )
        .unwrap();

    let main_path = workspace
        .write_file(
            "main.slx",
            r#"
import my.calc.MathHelper;

class MainApp {
    public static int32 main() {
        return MathHelper::add(15, 25);
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        helper_path.to_str().unwrap(),
        main_path.to_str().unwrap(),
    ])
    .expect("Failed to parse multi-source run CLI arguments");

    let execution_result = run(run_cli);
    assert!(execution_result.is_ok());
    assert_eq!(execution_result.unwrap(), 40);
}

#[test]
fn test_cli_run_direct_source_execution_with_dependency_lib() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let lib_source_path = workspace
        .write_file(
            "shared_lib.slx",
            r#"
public class Greeter {
    public static int32 greet_code() {
        return 99;
    }
}
"#,
        )
        .unwrap();

    let lib_binary_path = workspace.file_path("shared_lib.slxb");

    let compile_lib_cli = Cli::try_parse_from([
        "solix",
        "compile",
        lib_source_path.to_str().unwrap(),
        "--name",
        "greeter",
        "-o",
        lib_binary_path.to_str().unwrap(),
    ])
    .expect("Failed to parse compile lib CLI arguments");
    assert!(run(compile_lib_cli).is_ok());

    let app_source_path = workspace
        .write_file(
            "consumer_app.slx",
            r#"
class ConsumerApp {
    public static int32 main() {
        return Greeter::greet_code();
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        app_source_path.to_str().unwrap(),
        lib_binary_path.to_str().unwrap(),
    ])
    .expect("Failed to parse run CLI arguments with positional dependency lib");

    let execution_result = run(run_cli);
    assert!(execution_result.is_ok());
    assert_eq!(execution_result.unwrap(), 99);
}

#[test]
fn test_cli_run_direct_source_negative_compilation_error() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let broken_source_path = workspace
        .write_file(
            "broken.slx",
            r#"
class BrokenApp {
    public static int32 main() {
        int32 syntax_error = ;
        return 0;
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from(["solix", "run", broken_source_path.to_str().unwrap()])
        .expect("Failed to parse run CLI arguments");

    let execution_result = run(run_cli);
    assert!(execution_result.is_err());
    let error_message = format!("{:?}", execution_result.err().unwrap());
    assert!(error_message.contains("Compilation aborted due to previous error(s)"));
}

#[test]
fn test_cli_run_direct_source_execution_multi_source_and_multi_lib_positional() {
    let workspace = HermeticCliWorkspace::create().unwrap();

    // 1. First library: lib1.slxb
    let lib1_source = workspace
        .write_file(
            "lib1.slx",
            r#"
public class LibOne {
    public static int32 value_one() {
        return 100;
    }
}
"#,
        )
        .unwrap();
    let lib1_bin = workspace.file_path("lib1.slxb");
    let compile_lib1 = Cli::try_parse_from([
        "solix",
        "compile",
        lib1_source.to_str().unwrap(),
        "--name",
        "lib1",
        "-o",
        lib1_bin.to_str().unwrap(),
    ])
    .unwrap();
    assert!(run(compile_lib1).is_ok());

    // 2. Second library: lib2.slxb
    let lib2_source = workspace
        .write_file(
            "lib2.slx",
            r#"
public class LibTwo {
    public static int32 value_two() {
        return 200;
    }
}
"#,
        )
        .unwrap();
    let lib2_bin = workspace.file_path("lib2.slxb");
    let compile_lib2 = Cli::try_parse_from([
        "solix",
        "compile",
        lib2_source.to_str().unwrap(),
        "--name",
        "lib2",
        "-o",
        lib2_bin.to_str().unwrap(),
    ])
    .unwrap();
    assert!(run(compile_lib2).is_ok());

    // 3. First source file: source1.slx
    let source1_file = workspace
        .write_file(
            "source1.slx",
            r#"
package my.app;

public class AppHelper {
    public static int32 helper_value() {
        return 50;
    }
}
"#,
        )
        .unwrap();

    // 4. Second source file: source2.slx (contains entry point main)
    let source2_file = workspace
        .write_file(
            "source2.slx",
            r#"
import my.app.AppHelper;

class MainConsumer {
    public static int32 main() {
        int32 sum = LibOne::value_one() + LibTwo::value_two() + AppHelper::helper_value();
        return sum == 350 ? 0 : 1;
    }
}
"#,
        )
        .unwrap();

    // Execute: solix run source1.slx source2.slx lib1.slxb lib2.slxb
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        source1_file.to_str().unwrap(),
        source2_file.to_str().unwrap(),
        lib1_bin.to_str().unwrap(),
        lib2_bin.to_str().unwrap(),
    ])
    .expect("Failed to parse run CLI arguments for positional multi-source and multi-lib");

    let execution_result = run(run_cli);
    assert!(execution_result.is_ok());
    assert_eq!(execution_result.unwrap(), 0);
}

#[test]
fn test_cli_run_installed_package_spec_resolution() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let package_store = workspace.file_path("test_store");
    std::fs::create_dir_all(&package_store).unwrap();

    let source_path = workspace
        .write_file(
            "app.slx",
            r#"
class App {
    public static int32 main() {
        return 42;
    }
}
"#,
        )
        .unwrap();

    let binary_artifact = workspace.file_path("my-tool.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().unwrap(),
        "--name",
        "my-tool",
        "-V",
        "1.0.0",
        "-o",
        binary_artifact.to_str().unwrap(),
    ])
    .unwrap();
    assert!(run(compile_cli).is_ok());

    let install_cli = Cli::try_parse_from([
        "solix",
        "install",
        binary_artifact.to_str().unwrap(),
        "--pkg-dir",
        package_store.to_str().unwrap(),
    ])
    .unwrap();
    assert!(run(install_cli).is_ok());

    // 1. Run via bare name
    let run_bare = Cli::try_parse_from([
        "solix",
        "run",
        "my-tool",
        "--pkg-dir",
        package_store.to_str().unwrap(),
    ])
    .unwrap();
    let exit_code = run(run_bare).expect("Execution of bare package name failed");
    assert_eq!(exit_code, 42);

    // 2. Run via name@version
    let run_versioned = Cli::try_parse_from([
        "solix",
        "run",
        "my-tool@1.0.0",
        "--pkg-dir",
        package_store.to_str().unwrap(),
    ])
    .unwrap();
    let exit_code_versioned = run(run_versioned).expect("Execution of versioned package failed");
    assert_eq!(exit_code_versioned, 42);

    // 3. Run via name:profile@version
    let run_colon = Cli::try_parse_from([
        "solix",
        "run",
        "my-tool:release@1.0.0",
        "--pkg-dir",
        package_store.to_str().unwrap(),
    ])
    .unwrap();
    let exit_code_colon = run(run_colon).expect("Execution of colon package spec failed");
    assert_eq!(exit_code_colon, 42);

    // 4. Run via name with --profile flag
    let run_profile_flag = Cli::try_parse_from([
        "solix",
        "run",
        "my-tool",
        "--profile",
        "release",
        "--pkg-dir",
        package_store.to_str().unwrap(),
    ])
    .unwrap();
    let exit_code_flag = run(run_profile_flag).expect("Execution with --profile flag failed");
    assert_eq!(exit_code_flag, 42);
}

#[test]
fn test_cli_compile_metrics_flag() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let source_file = workspace
        .write_file(
            "metrics_sample.slx",
            r#"
class Sample {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .unwrap();
    let output_bin = workspace.file_path("metrics_sample.slxb");

    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().unwrap(),
        "-o",
        output_bin.to_str().unwrap(),
        "--metrics",
    ])
    .expect("Failed to parse compile with --metrics");

    assert!(run(compile_cli).is_ok());
    assert!(output_bin.exists());
}

#[test]
fn test_cli_compile_metrics_json_flag() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let source_file = workspace
        .write_file(
            "json_sample.slx",
            r#"
class Sample {
    public static int32 compute(int32 operand) {
        return operand * 2;
    }
}
"#,
        )
        .unwrap();
    let output_bin = workspace.file_path("json_sample.slxb");

    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().unwrap(),
        "-o",
        output_bin.to_str().unwrap(),
        "--metrics-json",
    ])
    .expect("Failed to parse compile with --metrics-json");

    assert!(run(compile_cli).is_ok());
    assert!(output_bin.exists());
}

#[test]
fn test_cli_run_metrics_flag() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let source_file = workspace
        .write_file(
            "run_sample.slx",
            r#"
class Main {
    public static int32 main() {
        return 77;
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from(["solix", "run", source_file.to_str().unwrap(), "--metrics"])
        .expect("Failed to parse run with --metrics");

    let result = run(run_cli).expect("Execution failed");
    assert_eq!(result, 77);
}

#[test]
fn test_cli_run_metrics_json_flag() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let source_file = workspace
        .write_file(
            "fib_test.slx",
            r#"
class Fib {
    public static int32 fib(int32 n) {
        if (n <= 1) {
            return n;
        }
        return Fib::fib(n - 1) + Fib::fib(n - 2);
    }
    public static int32 main() {
        return Fib::fib(8);
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        source_file.to_str().unwrap(),
        "--metrics-json",
    ])
    .expect("Failed to parse run with --metrics-json");

    let result = run(run_cli).expect("Execution failed");
    assert_eq!(result, 21);
}

#[test]
fn test_process_telemetry_capture() {
    let telemetry = solix_toolchain::capture_process_telemetry();
    #[cfg(target_os = "linux")]
    {
        assert!(
            telemetry.peak_rss_kb > 0,
            "peak_rss_kb was {}",
            telemetry.peak_rss_kb
        );
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = telemetry;
    }
}

#[test]
fn test_cli_compile_directory_recursive_discovery() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let source_dir = workspace.root_path().join("src");
    let nested_dir = source_dir.join("models");
    std::fs::create_dir_all(&nested_dir).expect("Failed to create nested directory");

    std::fs::write(
        nested_dir.join("user.slx"),
        r#"
package app.models;

public class User {
    public int32 id;
    public User() {
        this.id = 42;
    }
}
"#,
    )
    .expect("Failed to write nested file");

    std::fs::write(
        source_dir.join("main.slx"),
        r#"
package app;
import app.models.User;

class Main {
    public static int32 main() {
        User user = new User();
        return user.id == 42 ? 0 : 1;
    }
}
"#,
    )
    .expect("Failed to write main file");

    let binary_output = workspace.file_path("app.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_dir.to_str().expect("valid path"),
        "--profile",
        "release",
        "-o",
        binary_output.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile directory command");

    assert_eq!(run(compile_cli).expect("Compilation failed"), 0);
    assert!(binary_output.exists());

    let run_cli =
        Cli::try_parse_from(["solix", "run", binary_output.to_str().expect("valid path")])
            .expect("Failed to parse run command");
    assert_eq!(run(run_cli).expect("Execution failed"), 0);
}

#[test]
fn test_cli_compile_profiles_debug_and_release() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let source_file = workspace
        .write_file(
            "calc.slx",
            r#"
class Calculator {
    public static int32 add(int32 alpha, int32 beta) {
        return alpha + beta;
    }
    public static int32 main() {
        return Calculator::add(10, 32) == 42 ? 0 : 1;
    }
}
"#,
        )
        .expect("Failed to write file");

    let debug_binary = workspace.file_path("calc_debug.slxb");
    let compile_debug_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().expect("valid path"),
        "--profile",
        "debug",
        "-o",
        debug_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile debug command");
    assert_eq!(run(compile_debug_cli).expect("Debug compile failed"), 0);

    let release_binary = workspace.file_path("calc_release.slxb");
    let compile_release_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_file.to_str().expect("valid path"),
        "--profile",
        "release",
        "-o",
        release_binary.to_str().expect("valid path"),
    ])
    .expect("Failed to parse compile release command");
    assert_eq!(run(compile_release_cli).expect("Release compile failed"), 0);

    let run_debug =
        Cli::try_parse_from(["solix", "run", debug_binary.to_str().expect("valid path")])
            .expect("Parse run debug");
    assert_eq!(run(run_debug).expect("Run debug failed"), 0);

    let run_release =
        Cli::try_parse_from(["solix", "run", release_binary.to_str().expect("valid path")])
            .expect("Parse run release");
    assert_eq!(run(run_release).expect("Run release failed"), 0);
}

#[test]
fn test_cli_compile_empty_directory_fails() {
    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let empty_dir = workspace.root_path().join("empty_src");
    std::fs::create_dir_all(&empty_dir).expect("Failed to create empty directory");

    let compile_cli =
        Cli::try_parse_from(["solix", "compile", empty_dir.to_str().expect("valid path")])
            .expect("Failed to parse compile command");
    assert!(run(compile_cli).is_err());
}

#[test]
fn test_cli_info_slxarc_archive_inspection_and_json() {
    use solix_common::project::SlxarcArchive;

    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let stage_directory = workspace.root_path().join("stage");
    let bin_directory = stage_directory.join("bin");
    let lib_directory = stage_directory.join("lib");
    std::fs::create_dir_all(&bin_directory).expect("create bin dir");
    std::fs::create_dir_all(&lib_directory).expect("create lib dir");

    let manifest_toml = r#"
[configuration]
name = "archive_info_test"
version = "2.5.0"
description = "Test inspecting slxarc archive container"
license = "Apache-2.0"
authors = ["Tester <tester@example.com>"]

[profiles.release]
entrypoint = "Main::main"

[profiles.test]
inherits = "release"
entrypoint = "TestRunner::main"
"#;
    std::fs::write(stage_directory.join("manifest.toml"), manifest_toml).expect("write manifest");

    // Compile a real binary to put in bin/
    let source_path = workspace
        .write_file(
            "app.slx",
            r#"
class Main {
    public static int32 main() {
        return 77;
    }
}
"#,
        )
        .expect("write source");

    let bin_path = bin_directory.join("archive_info_test.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().expect("valid path"),
        "-o",
        bin_path.to_str().expect("valid path"),
    ])
    .expect("parse compile");
    assert_eq!(run(compile_cli).expect("compile success"), 0);

    // Write a dummy native lib into lib/
    std::fs::write(lib_directory.join("libsample.so"), b"sample_native_content")
        .expect("write lib");

    let archive_path = workspace.file_path("archive_info_test.slxarc");
    SlxarcArchive::pack_directory(&stage_directory, &archive_path).expect("pack slxarc");
    assert!(archive_path.exists());

    // 1. Text info inspection
    let info_cli =
        Cli::try_parse_from(["solix", "info", archive_path.to_str().expect("valid path")])
            .expect("parse info");
    assert_eq!(run(info_cli).expect("info run success"), 0);

    // 2. JSON info inspection to output file
    let json_output_path = workspace.file_path("archive_info.json");
    let info_json_cli = Cli::try_parse_from([
        "solix",
        "info",
        archive_path.to_str().expect("valid path"),
        "--json",
        "-o",
        json_output_path.to_str().expect("valid path"),
    ])
    .expect("parse info json");
    assert_eq!(run(info_json_cli).expect("info json success"), 0);
    assert!(json_output_path.exists());

    let json_content = std::fs::read_to_string(&json_output_path).expect("read json");
    let parsed_json: serde_json::Value = serde_json::from_str(&json_content).expect("parse json");
    assert_eq!(
        parsed_json["manifest"]["configuration"]["name"],
        "archive_info_test"
    );
    assert_eq!(parsed_json["manifest"]["configuration"]["version"], "2.5.0");
    assert_eq!(
        parsed_json["manifest"]["profiles"]["release"]["entrypoint"],
        "Main::main"
    );
}

#[test]
fn test_cli_run_slxarc_archive_directly() {
    use solix_common::project::SlxarcArchive;

    let workspace = HermeticCliWorkspace::create().expect("Failed to create workspace");
    let stage_directory = workspace.root_path().join("stage_run");
    let bin_directory = stage_directory.join("bin");
    std::fs::create_dir_all(&bin_directory).expect("create bin dir");

    let manifest_toml = r#"
[configuration]
name = "runner_app"
version = "1.0.0"

[profiles.release]
entrypoint = "Main::main"
"#;
    std::fs::write(stage_directory.join("manifest.toml"), manifest_toml).expect("write manifest");

    let source_path = workspace
        .write_file(
            "entry.slx",
            r#"
class Main {
    public static int32 main() {
        return 99;
    }
}
"#,
        )
        .expect("write source");

    let bin_path = bin_directory.join("runner_app.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().expect("valid path"),
        "-o",
        bin_path.to_str().expect("valid path"),
    ])
    .expect("parse compile");
    assert_eq!(run(compile_cli).expect("compile success"), 0);

    let archive_path = workspace.file_path("runner_app.slxarc");
    SlxarcArchive::pack_directory(&stage_directory, &archive_path).expect("pack slxarc");

    let run_cli = Cli::try_parse_from(["solix", "run", archive_path.to_str().expect("valid path")])
        .expect("parse run");

    let exit_code = run(run_cli).expect("run archive success");
    assert_eq!(exit_code, 99);
}

#[test]
fn test_cli_clean_full_and_cache_only() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let build_directory = workspace.root_path().join("build");
    let cache_directory = build_directory.join(".cache");
    let release_directory = build_directory.join("release");

    std::fs::create_dir_all(&cache_directory).expect("create cache dir");
    std::fs::create_dir_all(&release_directory).expect("create release dir");

    let cache_file = cache_directory.join("temp_cache.bin");
    let release_artifact = release_directory.join("app.slxarc");
    std::fs::write(&cache_file, b"cached_bytes").expect("write cache file");
    std::fs::write(&release_artifact, b"release_archive_bytes").expect("write artifact");

    assert!(cache_file.exists());
    assert!(release_artifact.exists());

    // 1. Test clean --cache-only
    let clean_cache_cli = Cli::try_parse_from([
        "solix",
        "clean",
        workspace.root_path().to_str().expect("valid path"),
        "--cache-only",
    ])
    .expect("parse clean cache-only");
    assert_eq!(run(clean_cache_cli).expect("clean cache-only success"), 0);

    assert!(!cache_directory.exists());
    assert!(release_artifact.exists());
    assert!(build_directory.exists());

    // 2. Test full clean
    let clean_full_cli = Cli::try_parse_from([
        "solix",
        "clean",
        workspace.root_path().to_str().expect("valid path"),
    ])
    .expect("parse clean full");
    assert_eq!(run(clean_full_cli).expect("clean full success"), 0);

    assert!(!build_directory.exists());

    // 3. Test clean when build/ does not exist (idempotent noop)
    let clean_noop_cli = Cli::try_parse_from([
        "solix",
        "clean",
        workspace.root_path().to_str().expect("valid path"),
    ])
    .expect("parse clean noop");
    assert_eq!(run(clean_noop_cli).expect("clean noop success"), 0);
}

#[test]
fn test_cli_info_enriched_fields() {
    use solix_common::project::SlxarcArchive;

    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let stage_directory = workspace.root_path().join("stage_info_enriched");
    let bin_directory = stage_directory.join("bin");
    let lib_linux_directory = stage_directory.join("lib").join("linux-x86_64");
    let lib_macos_directory = stage_directory.join("lib").join("macos-arm64");

    std::fs::create_dir_all(&bin_directory).expect("create bin dir");
    std::fs::create_dir_all(&lib_linux_directory).expect("create linux lib dir");
    std::fs::create_dir_all(&lib_macos_directory).expect("create macos lib dir");

    let manifest_toml = r#"
[configuration]
name = "enriched_pkg"
version = "3.2.1"
description = "Enriched diagnostic test package"

[profiles.release]
entrypoint = "Main::main"
"#;
    std::fs::write(stage_directory.join("manifest.toml"), manifest_toml).expect("write manifest");

    let source_path = workspace
        .write_file(
            "enriched_entry.slx",
            r#"
class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .expect("write source");

    let bin_path = bin_directory.join("enriched_pkg.slxb");
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        source_path.to_str().expect("valid path"),
        "-o",
        bin_path.to_str().expect("valid path"),
    ])
    .expect("parse compile");
    assert_eq!(run(compile_cli).expect("compile success"), 0);

    std::fs::write(
        lib_linux_directory.join("libnative.so"),
        b"sample_linux_native",
    )
    .expect("write linux lib");
    std::fs::write(
        lib_macos_directory.join("libnative.dylib"),
        b"sample_macos_native",
    )
    .expect("write macos lib");

    let archive_path = workspace.file_path("enriched_pkg.slxarc");
    SlxarcArchive::pack_directory(&stage_directory, &archive_path).expect("pack slxarc");
    assert!(archive_path.exists());

    // 1. Check JSON format inspection
    let json_output_path = workspace.file_path("enriched_info.json");
    let info_json_cli = Cli::try_parse_from([
        "solix",
        "info",
        archive_path.to_str().expect("valid path"),
        "--json",
        "-o",
        json_output_path.to_str().expect("valid path"),
    ])
    .expect("parse info json");
    assert_eq!(run(info_json_cli).expect("info json success"), 0);

    let json_content = std::fs::read_to_string(&json_output_path).expect("read json");
    let parsed_json: serde_json::Value = serde_json::from_str(&json_content).expect("parse json");

    assert!(parsed_json["file_size_bytes"].as_u64().unwrap_or(0) > 0);
    assert!(
        !parsed_json["file_size_formatted"]
            .as_str()
            .unwrap_or("")
            .is_empty()
    );

    let platforms_array = parsed_json["bundled_platforms"]
        .as_array()
        .expect("platforms array");
    let platform_names: Vec<String> = platforms_array
        .iter()
        .map(|item| item.as_str().unwrap_or("").to_string())
        .collect();
    assert!(platform_names.contains(&"linux-x86_64".to_string()));
    assert!(platform_names.contains(&"macos-arm64".to_string()));

    // 2. Check text format output
    let text_output_path = workspace.file_path("enriched_info.txt");
    let info_text_cli = Cli::try_parse_from([
        "solix",
        "info",
        archive_path.to_str().expect("valid path"),
        "-o",
        text_output_path.to_str().expect("valid path"),
    ])
    .expect("parse info text");
    assert_eq!(run(info_text_cli).expect("info text success"), 0);

    let text_content = std::fs::read_to_string(&text_output_path).expect("read text");
    assert!(text_content.contains("File Size:"));
    assert!(text_content.contains("Platforms:"));
    assert!(text_content.contains("linux-x86_64, macos-arm64"));
    assert!(text_content.contains("Exported Symbols:"));
}

#[test]
fn test_cli_compile_upward_manifest_auto_discovery() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let package_store = workspace.root_path().join("pkg_store");
    std::fs::create_dir_all(&package_store).expect("create pkg_store");
    let store_string = package_store.to_str().expect("valid store path");

    // 1. Compile and install dependency math_lib
    let math_source = workspace
        .write_file(
            "math.slx",
            r#"
package math_lib;

public class MathLib {
    public static int32 square(int32 operand) {
        return operand * operand;
    }
}
"#,
        )
        .expect("write math");

    let math_bin = workspace.file_path("math_lib.slxb");
    let compile_math = Cli::try_parse_from([
        "solix",
        "compile",
        math_source.to_str().expect("valid path"),
        "--name",
        "math_lib",
        "--version",
        "1.0.0",
        "-o",
        math_bin.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
        "--install",
    ])
    .expect("parse compile math");
    assert_eq!(run(compile_math).expect("compile math succeeds"), 0);

    // 2. Set up project with manifest.toml in parent folder
    let project_dir = workspace.root_path().join("my_game");
    let source_dir = project_dir.join("src");
    std::fs::create_dir_all(&source_dir).expect("create src dir");

    let manifest_content = r#"
[configuration]
name = "my_game"
version = "1.2.0"
description = "Showcase game"

[profiles.release]
entrypoint = "Game::main"

[profiles.release.dependencies]
math_lib = { version = "1.0.0", profile = "release" }
"#;
    std::fs::write(project_dir.join("manifest.toml"), manifest_content).expect("write manifest");

    let main_source_path = source_dir.join("main.slx");
    std::fs::write(
        &main_source_path,
        r#"
import math_lib.*;

public class Game {
    public static int32 main() {
        return MathLib::square(6);
    }
}
"#,
    )
    .expect("write main source");

    let output_binary_path = workspace.file_path("my_game.slxb");

    // Compile referencing only src/main.slx, WITHOUT --deps, --name, or --entry
    let compile_cli = Cli::try_parse_from([
        "solix",
        "compile",
        main_source_path.to_str().expect("valid path"),
        "-o",
        output_binary_path.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("parse compile command");

    assert_eq!(
        run(compile_cli).expect("compile with upward manifest discovery succeeds"),
        0
    );
    assert!(output_binary_path.exists());

    // 3. Run the compiled binary and verify execution
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        output_binary_path.to_str().expect("valid path"),
        "--pkg-dir",
        store_string,
    ])
    .expect("parse run command");

    assert_eq!(run(run_cli).expect("run succeeds"), 36);
}

#[test]
fn test_cli_new_package_and_native_subcommands() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let package_dir = workspace.file_path("my_scaffold_pkg");

    // 1. solix new package <path>
    let new_package_cli = Cli::try_parse_from([
        "solix",
        "new",
        "package",
        package_dir.to_str().expect("valid path"),
        "--name",
        "my_scaffold_pkg",
        "--no-native",
    ])
    .expect("parse new package cli");
    assert_eq!(run(new_package_cli).expect("execute new package"), 0);
    assert!(package_dir.join("manifest.yaml").exists());
    assert!(package_dir.join("src").join("main.slx").exists());

    // 2. solix new native <name> inside existing project
    let new_native_cli = Cli::try_parse_from([
        "solix",
        "new",
        "native",
        "my_native_math",
        "--project",
        package_dir.to_str().expect("valid path"),
        "--lang",
        "rust",
    ])
    .expect("parse new native cli");
    assert_eq!(run(new_native_cli).expect("execute new native"), 0);
    assert!(
        package_dir
            .join("natives")
            .join("my_native_math")
            .join("Cargo.toml")
            .exists()
    );
}

#[test]
fn test_cli_compile_source_with_all_dep_flags_and_compile_package() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");

    // 1. Create a helper library source and compile it to a binary container (.slxb)
    let helper_src = workspace
        .write_file(
            "helper.slx",
            r#"
public class MathHelper {
    public static int32 add_five(int32 value) {
        return value + 5;
    }
}
"#,
        )
        .expect("write helper.slx");

    let helper_bin = workspace.file_path("helper.slxb");
    let compile_helper_cli = Cli::try_parse_from([
        "solix",
        "compile",
        "source",
        helper_src.to_str().expect("valid path"),
        "-o",
        helper_bin.to_str().expect("valid path"),
    ])
    .expect("parse compile helper");
    assert_eq!(run(compile_helper_cli).expect("compile helper"), 0);
    assert!(helper_bin.exists());

    // 2. Create another helper source to test --deps-source
    let source_dep = workspace
        .write_file(
            "source_dep.slx",
            r#"
public class SourceHelper {
    public static int32 double_it(int32 value) {
        return value * 2;
    }
}
"#,
        )
        .expect("write source_dep.slx");

    // 3. Compile main.slx linking via --deps-source and --deps-binaries
    let main_src = workspace
        .write_file(
            "main.slx",
            r#"
class Main {
    public static int32 main() {
        int32 step_one = MathHelper::add_five(10);
        return SourceHelper::double_it(step_one);
    }
}
"#,
        )
        .expect("write main.slx");

    let main_bin = workspace.file_path("main.slxb");
    let compile_main_cli = Cli::try_parse_from([
        "solix",
        "compile",
        "source",
        main_src.to_str().expect("valid path"),
        "--deps-source",
        source_dep.to_str().expect("valid path"),
        "--deps-binaries",
        helper_bin.to_str().expect("valid path"),
        "-o",
        main_bin.to_str().expect("valid path"),
    ])
    .expect("parse compile main with deps");
    assert_eq!(run(compile_main_cli).expect("compile main with deps"), 0);
    assert!(main_bin.exists());

    // Run the compiled binary container to verify logic ( (10 + 5) * 2 = 30 )
    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        main_bin.to_str().expect("valid path"),
        "-l",
        helper_bin.to_str().expect("valid path"),
    ])
    .expect("parse run main");
    assert_eq!(run(run_cli).expect("run main"), 30);

    // 4. Test `solix compile package ./pkg_dir`
    let pkg_dir = workspace.file_path("package_to_compile");
    let new_pkg_cli = Cli::try_parse_from([
        "solix",
        "new",
        "package",
        pkg_dir.to_str().expect("valid path"),
        "--name",
        "package_to_compile",
        "--no-native",
    ])
    .expect("parse new package");
    assert_eq!(run(new_pkg_cli).expect("new package"), 0);

    let compile_pkg_cli = Cli::try_parse_from([
        "solix",
        "compile",
        "package",
        pkg_dir.to_str().expect("valid path"),
    ])
    .expect("parse compile package");
    assert_eq!(run(compile_pkg_cli).expect("compile package"), 0);
    assert!(pkg_dir.join("binaries.slxb").exists());
}

#[test]
fn test_cli_run_package_and_source_subcommands() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");

    // 1. solix run source <file.slx>
    let standalone_src = workspace
        .write_file(
            "standalone.slx",
            r#"
class Main {
    public static int32 main() {
        return 55;
    }
}
"#,
        )
        .expect("write standalone.slx");

    let run_source_cli = Cli::try_parse_from([
        "solix",
        "run",
        "source",
        standalone_src.to_str().expect("valid path"),
    ])
    .expect("parse run source cli");
    assert_eq!(run(run_source_cli).expect("run source succeeds"), 55);

    // 2. solix run package <package_dir>
    let package_dir = workspace.file_path("executable_pkg");
    let new_package_cli = Cli::try_parse_from([
        "solix",
        "new",
        "package",
        package_dir.to_str().expect("valid path"),
        "--name",
        "executable_pkg",
        "--no-native",
    ])
    .expect("parse new package cli");
    assert_eq!(run(new_package_cli).expect("new package project"), 0);

    // Replace src/main.slx with an exit code of 77
    let main_slx_path = package_dir.join("src").join("main.slx");
    std::fs::write(
        &main_slx_path,
        r#"
public class Main {
    public static int32 main() {
        return 77;
    }
}
"#,
    )
    .expect("write updated main.slx");

    let run_package_cli = Cli::try_parse_from([
        "solix",
        "run",
        "package",
        package_dir.to_str().expect("valid path"),
    ])
    .expect("parse run package cli");
    assert_eq!(run(run_package_cli).expect("run package succeeds"), 77);
}

#[test]
fn test_cli_build_platform_option() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let package_dir = workspace.file_path("platform_pkg");

    let new_package_cli = Cli::try_parse_from([
        "solix",
        "new",
        "package",
        package_dir.to_str().expect("valid path"),
        "--name",
        "platform_pkg",
        "--no-native",
    ])
    .expect("parse new package cli");
    assert_eq!(run(new_package_cli).expect("new package project"), 0);

    // solix build --platform windows --check-env
    let build_platform_cli = Cli::try_parse_from([
        "solix",
        "build",
        package_dir.to_str().expect("valid path"),
        "--platform",
        "windows",
        "--check-env",
    ])
    .expect("parse build platform cli");
    assert_eq!(
        run(build_platform_cli).expect("build platform check-env succeeds"),
        0
    );
}

#[test]
fn test_cli_fmt_skips_ignored_directories() {
    let workspace = HermeticCliWorkspace::create().expect("create workspace");
    let src_file = workspace
        .write_file("src/main.slx", "class Main{int32 value_one=1;}")
        .expect("write src");
    let build_file = workspace
        .write_file(
            "build/release/stage/generated.slx",
            "class Generated{int32 value_two=2;}",
        )
        .expect("write build");
    let git_file = workspace
        .write_file(".git/hooks/hook.slx", "class Hook{int32 value_three=3;}")
        .expect("write git");

    let fmt_cli = Cli::try_parse_from([
        "solix",
        "fmt",
        workspace.root_path().to_str().expect("valid path"),
    ])
    .expect("parse fmt cli");

    assert_eq!(run(fmt_cli).expect("fmt succeeds"), 0);

    let src_content = std::fs::read_to_string(&src_file).expect("read src");
    assert!(
        src_content.contains("class Main {"),
        "Expected src/main.slx to be formatted"
    );

    let build_content = std::fs::read_to_string(&build_file).expect("read build");
    assert_eq!(
        build_content, "class Generated{int32 value_two=2;}",
        "Expected build/ file to be untouched"
    );

    let git_content = std::fs::read_to_string(&git_file).expect("read git");
    assert_eq!(
        git_content, "class Hook{int32 value_three=3;}",
        "Expected .git/ file to be untouched"
    );
}
