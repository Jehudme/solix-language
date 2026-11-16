//! Integration tests for Milestone 053:
//! Toolchain CLI Refactoring, Entrypoint Validation (E0301), and Documentation Intelligence.

#![forbid(unsafe_code)]

mod common;

use clap::Parser;
use common::HermeticCliWorkspace;
use solix_toolchain::cli::Cli;
use solix_toolchain::run;

#[test]
fn test_run_valid_entrypoint_override() {
    let workspace = HermeticCliWorkspace::create().expect("Create test workspace");
    let source_path = workspace
        .write_file(
            "src/main.slx",
            r#"
package main;

public class Main {
    public static int32 main() {
        return 10;
    }

    public static int32 custom_entry() {
        return 42;
    }
}
"#,
        )
        .expect("write source");

    let cli_command = Cli::try_parse_from([
        "solix",
        "run",
        source_path.to_str().expect("valid path"),
        "--entry",
        "Main::custom_entry",
    ])
    .expect("parse run CLI");

    let exit_code = run(cli_command).expect("run custom entry");
    assert_eq!(exit_code, 42);
}

#[test]
fn test_run_missing_entrypoint_override_fails_e0301() {
    let workspace = HermeticCliWorkspace::create().expect("Create test workspace");
    let source_path = workspace
        .write_file(
            "src/main.slx",
            r#"
package main;

public class Main {
    public static int32 main() {
        return 0;
    }
}
"#,
        )
        .expect("write source");

    let cli_command = Cli::try_parse_from([
        "solix",
        "run",
        source_path.to_str().expect("valid path"),
        "--entry",
        "NonExistent::method",
    ])
    .expect("parse run CLI");

    let run_result = run(cli_command);
    assert!(run_result.is_err(), "Must fail when entrypoint is absent");
    let error_message = format!("{:?}", run_result.unwrap_err());
    assert!(
        error_message.contains("[E0301]"),
        "Error message must contain [E0301], got: {error_message}"
    );
    assert!(
        error_message.contains("NonExistent::method"),
        "Error message must specify requested symbol, got: {error_message}"
    );
}

#[test]
fn test_doc_generator_html_output() {
    let workspace = HermeticCliWorkspace::create().expect("Create test workspace");
    workspace
        .write_file(
            "manifest.toml",
            r#"
[configuration]
name = "math_lib"
version = "1.0.0"
description = "Mathematical operations library"
license = "Apache-2.0"
authors = ["Solix Team"]

[profiles.release]
sources = ["src"]
"#,
        )
        .expect("write manifest");

    workspace
        .write_file(
            "src/Calculator.slx",
            r#"
package math;

/// Advanced mathematical calculator.
/// Provides elementary arithmetic operations.
public class Calculator {
    /// Constructs a new Calculator instance.
    public Calculator() {}

    /// Adds two numbers together.
    ///
    /// @param first_number Left-hand operand
    /// @param second_number Right-hand operand
    /// @return Sum of operands
    /// @throws ArithmeticException When overflow occurs
    public int32 add(int32 first_number, int32 second_number) {
        return first_number + second_number;
    }
}

/// Supported calculation modes.
public enum Mode {
    /// Standard precision mode.
    Standard,
    /// High-precision scientific mode.
    Scientific,
}
"#,
        )
        .expect("write Calculator.slx");

    let output_directory = workspace.file_path("docs_out");

    let doc_command = Cli::try_parse_from([
        "solix",
        "doc",
        workspace.root_path().to_str().expect("valid path"),
        "--output",
        output_directory.to_str().expect("valid path"),
        "--format",
        "html",
    ])
    .expect("parse doc CLI");

    let doc_result = run(doc_command);
    assert!(doc_result.is_ok(), "solix doc html must succeed");

    let index_file = output_directory.join("index.html");
    let css_file = output_directory.join("style.css");
    let class_file = output_directory.join("math_Calculator.html");
    let enum_file = output_directory.join("math_Mode.html");

    assert!(index_file.exists(), "index.html must be generated");
    assert!(css_file.exists(), "style.css must be generated");
    assert!(class_file.exists(), "math_Calculator.html must be generated");
    assert!(enum_file.exists(), "math_Mode.html must be generated");

    let class_html = std::fs::read_to_string(&class_file).expect("read class html");
    assert!(class_html.contains("Advanced mathematical calculator"));
    assert!(class_html.contains("first_number"));
    assert!(class_html.contains("Right-hand operand"));
    assert!(class_html.contains("ArithmeticException"));
}

#[test]
fn test_doc_generator_markdown_output() {
    let workspace = HermeticCliWorkspace::create().expect("Create test workspace");
    workspace
        .write_file(
            "src/Logger.slx",
            r#"
package logging;

/// System logger for recording diagnostics.
public class Logger {
    /// Logs a debug message.
    /// @param message Message text to log
    public static void log(string message) {}
}
"#,
        )
        .expect("write Logger.slx");

    let output_directory = workspace.file_path("md_out");

    let doc_command = Cli::try_parse_from([
        "solix",
        "doc",
        workspace.root_path().to_str().expect("valid path"),
        "--output",
        output_directory.to_str().expect("valid path"),
        "--format",
        "markdown",
    ])
    .expect("parse doc CLI");

    let doc_result = run(doc_command);
    assert!(doc_result.is_ok(), "solix doc markdown must succeed");

    let readme_file = output_directory.join("README.md");
    let logger_file = output_directory.join("logging_Logger.md");

    assert!(readme_file.exists(), "README.md must be generated");
    assert!(logger_file.exists(), "logging_Logger.md must be generated");

    let logger_md = std::fs::read_to_string(&logger_file).expect("read logger md");
    assert!(logger_md.contains("System logger for recording diagnostics"));
    assert!(logger_md.contains("`message`: Message text to log"));
}

#[test]
fn test_doc_generator_empty_sources_fails_e0303() {
    let workspace = HermeticCliWorkspace::create().expect("Create test workspace");
    let output_directory = workspace.file_path("empty_out");

    let doc_command = Cli::try_parse_from([
        "solix",
        "doc",
        workspace.root_path().to_str().expect("valid path"),
        "--output",
        output_directory.to_str().expect("valid path"),
    ])
    .expect("parse doc CLI");

    let doc_result = run(doc_command);
    assert!(doc_result.is_err(), "Must fail when no .slx sources exist");
    let error_message = format!("{:?}", doc_result.unwrap_err());
    assert!(
        error_message.contains("[E0303]"),
        "Must contain error code [E0303], got: {error_message}"
    );
}
