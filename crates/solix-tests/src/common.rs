//! Common utilities and runners for Solix language conformance and integration testing.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_runtime::exceptions::errors::RuntimeErrorCode;
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

/// Compiles a single Solix source file into an executable [`BinaryProgram`].
///
/// # Panics
/// Panics if lexing, parsing, binding, or bytecode emission fails.
pub fn compile_source(source_code: &str) -> BinaryProgram {
    let options = CompilationOptions::new().with_source("test_main.slx", source_code);
    let mut context = CompilationContext::new(options);

    context.execute().expect("Compilation failed");

    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Bytecode emission artifacts must be present");
    artifacts.program.clone()
}

/// Compiles multiple Solix source files into an executable [`BinaryProgram`].
///
/// # Panics
/// Panics if any stage of multi-source compilation fails.
pub fn compile_multi_sources(sources: &[(&str, &str)]) -> BinaryProgram {
    let mut options = CompilationOptions::new();
    for (name, code) in sources {
        options = options.with_source(*name, *code);
    }
    let mut context = CompilationContext::new(options);

    context.execute().expect("Compilation failed");

    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Bytecode emission artifacts must be present");
    artifacts.program.clone()
}

/// Executes a compiled binary in the virtual machine with standard native bindings,
/// automatically verifying that all allocated heap memory is cleanly collected upon completion.
///
/// # Panics
/// Panics if VM execution fails or memory leaks are detected.
pub fn run_binary(program: BinaryProgram, entry_point: Option<&str>) -> (Value, VirtualMachine) {
    let mut options = RuntimeOptions::new()
        .with_binary(program)
        .with_native_function("println", builtin_println)
        .with_native_function("print", builtin_print)
        .with_native_function("Console::println", builtin_println)
        .with_native_function("Console::print", builtin_print);

    if let Some(entry) = entry_point {
        options = options.with_entry_point(entry);
    }

    let context = RuntimeContext::new(options).expect("RuntimeContext creation failed");
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("VM execution failed");

    if let Err(ref e) = vm.verify_clean_memory() {
        panic!("CLEAN MEMORY FAILED: {:?}", e);
    }

    (result, vm)
}

/// Compiles and executes a single Solix source file, auto-detecting the entry point and verifying clean memory.
///
/// # Panics
/// Panics if compilation, VM execution, or memory verification fails.
pub fn compile_and_run(source_code: &str) -> (Value, VirtualMachine) {
    let program = compile_source(source_code);

    let entry_opt = program
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::main") || f.symbol == "main")
        .map(|f| f.symbol.clone());

    run_binary(program, entry_opt.as_deref())
}

/// Compiles and executes multiple Solix source files, auto-detecting the entry point and verifying clean memory.
///
/// # Panics
/// Panics if multi-source compilation, VM execution, or memory verification fails.
pub fn compile_and_run_multi(sources: &[(&str, &str)]) -> (Value, VirtualMachine) {
    let program = compile_multi_sources(sources);

    let entry_opt = program
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::main") || f.symbol == "main")
        .map(|f| f.symbol.clone());

    run_binary(program, entry_opt.as_deref())
}

/// Compiles and executes a single Solix source file with an explicit entry point name.
///
/// # Panics
/// Panics if compilation, VM execution, or memory verification fails.
pub fn compile_and_run_entry(source_code: &str, entry_point: &str) -> (Value, VirtualMachine) {
    let program = compile_source(source_code);
    run_binary(program, Some(entry_point))
}

/// Asserts that compiling `source_code` fails with a diagnostic containing `expected_error_or_code`.
///
/// # Panics
/// Panics if compilation succeeds, if raw `SourceSpan {` debug formatting leaks into error messages,
/// or if the error does not contain the expected pattern / code.
pub fn assert_compile_error(source_code: &str, expected_error_or_code: &str) {
    let options = CompilationOptions::new().with_source("error_test.slx", source_code);
    let mut context = CompilationContext::new(options);

    match context.execute() {
        Ok(_) => panic!(
            "Expected compilation to fail with error matching '{expected_error_or_code}', but compilation succeeded!",
        ),
        Err(diagnostics) => {
            for diagnostic in &diagnostics {
                let display_message = format!("{diagnostic}");
                assert!(
                    !display_message.contains("SourceSpan {"),
                    "Diagnostic Display leaked raw SourceSpan debug formatting:\n{display_message}"
                );
            }

            let combined_message = diagnostics
                .iter()
                .map(|diagnostic| format!("{} {}", diagnostic.error_code(), diagnostic))
                .collect::<Vec<_>>()
                .join("\n");

            let matches_code = diagnostics.iter().any(|diagnostic| {
                diagnostic.error_code() == expected_error_or_code
                    || diagnostic.error_code().to_string() == expected_error_or_code
            });

            let matches_text = combined_message
                .to_lowercase()
                .contains(&expected_error_or_code.to_lowercase())
                || combined_message.contains(expected_error_or_code);

            assert!(
                matches_code || matches_text,
                "Expected error matching '{expected_error_or_code}', got:\n{combined_message}"
            );
        }
    }
}

/// Asserts that compiling multiple Solix sources fails with a diagnostic containing `expected_error_or_code`.
///
/// # Panics
/// Panics if compilation succeeds, if raw `SourceSpan {` debug formatting leaks into error messages,
/// or if the error does not contain the expected pattern / code.
pub fn assert_compile_sources_error(sources: &[(&str, &str)], expected_error_or_code: &str) {
    let mut options = CompilationOptions::new();
    for (name, code) in sources {
        options = options.with_source(*name, *code);
    }
    let mut context = CompilationContext::new(options);

    match context.execute() {
        Ok(_) => panic!(
            "Expected multi-source compilation to fail with error matching '{expected_error_or_code}', but compilation succeeded!",
        ),
        Err(diagnostics) => {
            for diagnostic in &diagnostics {
                let display_message = format!("{diagnostic}");
                assert!(
                    !display_message.contains("SourceSpan {"),
                    "Diagnostic Display leaked raw SourceSpan debug formatting:\n{display_message}"
                );
            }

            let combined_message = diagnostics
                .iter()
                .map(|diagnostic| format!("{} {}", diagnostic.error_code(), diagnostic))
                .collect::<Vec<_>>()
                .join("\n");

            let matches_code = diagnostics.iter().any(|diagnostic| {
                diagnostic.error_code() == expected_error_or_code
                    || diagnostic.error_code().to_string() == expected_error_or_code
            });

            let matches_text = combined_message
                .to_lowercase()
                .contains(&expected_error_or_code.to_lowercase())
                || combined_message.contains(expected_error_or_code);

            assert!(
                matches_code || matches_text,
                "Expected error matching '{expected_error_or_code}', got:\n{combined_message}"
            );
        }
    }
}

/// Asserts that running `source_code` in the VM aborts with the specified [`RuntimeErrorCode`].
///
/// # Panics
/// Panics if compilation fails, VM succeeds normally, or a different error code is returned.
pub fn assert_runtime_error(source_code: &str, expected_code: RuntimeErrorCode) {
    let program = compile_source(source_code);
    let entry_opt = program
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::main") || f.symbol == "main")
        .map(|f| f.symbol.clone());

    let mut options = RuntimeOptions::new().with_binary(program);
    if let Some(entry) = entry_opt.as_deref() {
        options = options.with_entry_point(entry);
    }

    let context = RuntimeContext::new(options).expect("RuntimeContext creation failed");
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    match res {
        Ok(val) => {
            let actual_code = vm.exit_code.unwrap_or(0);
            assert_eq!(
                actual_code, expected_code as i32,
                "Expected runtime error {:?} ({}), but VM returned Ok({:?}) with exit code {}",
                expected_code, expected_code as i32, val, actual_code
            );
        }
        Err(err) => {
            assert_eq!(
                err.error_code(),
                expected_code,
                "Expected runtime error {:?}, got {:?}",
                expected_code,
                err
            );
        }
    }
}
