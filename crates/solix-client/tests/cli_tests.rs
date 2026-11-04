//! Integration test suite for the Solix command-line driver (`solix`).

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_client::cli::{Cli, Commands};
use solix_client::run;

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
fn test_cli_compile_with_builtins_blt_io() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "hello.slx",
            r#"
import blt.io.println;

class Main {
    public static int32 main() {
        println("Hello from Solix builtins!");
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
fn test_cli_compile_no_builtins_flag_rejects_blt_io() {
    let ws = HermeticCliWorkspace::create().unwrap();
    let src = ws
        .write_file(
            "no_builtins.slx",
            r#"
import blt.io.println;

class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .unwrap();

    let bin = ws.file_path("no_builtins.slxb");

    let cli = Cli::try_parse_from([
        "solix",
        "compile",
        src.to_str().unwrap(),
        "--no-builtins",
        "-o",
        bin.to_str().unwrap(),
    ])
    .unwrap();

    // With --no-builtins, blt.io cannot be resolved
    assert!(run(cli).is_err());
    assert!(!bin.exists());
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
        assert!(!args.no_builtins);
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
        "--no-builtins",
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
        assert!(args.no_builtins);
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
        assert_eq!(args.binary.to_str().unwrap(), "app.slxb");
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
import blt.io.println;

class DirectApp {
    public static int32 main() {
        println("Direct source execution works!");
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
        "--lib",
        lib_binary_path.to_str().unwrap(),
    ])
    .expect("Failed to parse run CLI arguments with dependency lib");

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
fn test_cli_run_direct_source_no_builtins_flag() {
    let workspace = HermeticCliWorkspace::create().unwrap();
    let source_path = workspace
        .write_file(
            "builtins_test.slx",
            r#"
import blt.io.println;

class AppWithBuiltin {
    public static int32 main() {
        println("Should fail if builtins disabled");
        return 0;
    }
}
"#,
        )
        .unwrap();

    let run_cli = Cli::try_parse_from([
        "solix",
        "run",
        "--no-builtins",
        source_path.to_str().unwrap(),
    ])
    .expect("Failed to parse run CLI arguments");

    let execution_result = run(run_cli);
    assert!(execution_result.is_err());
}
