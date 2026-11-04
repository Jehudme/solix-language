//! The `solix run` command driver.

#![forbid(unsafe_code)]

use crate::cli::RunArgs;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::BinaryProgram;
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};
use std::fs;

/// Parses a human-readable memory size string (e.g., `"64MB"`, `"16KB"`, `"1048576"`) into bytes.
fn parse_byte_size(size_str: &str) -> Result<usize> {
    let trimmed = size_str.trim();
    if trimmed.is_empty() {
        return Ok(32 * 1024 * 1024);
    }

    let upper = trimmed.to_uppercase();
    if let Some(num_str) = upper.strip_suffix("GB") {
        let n: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(n * 1024 * 1024 * 1024)
    } else if let Some(num_str) = upper.strip_suffix("MB") {
        let n: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(n * 1024 * 1024)
    } else if let Some(num_str) = upper.strip_suffix("KB") {
        let n: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(n * 1024)
    } else if let Some(num_str) = upper.strip_suffix('B') {
        let n: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(n)
    } else {
        let n: usize = trimmed
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(n)
    }
}

/// Executes the `run` subcommand.
///
/// # Errors
/// Returns an error if binary loading, VM initialization, or execution fails.
pub fn execute_run(args: RunArgs) -> Result<i32> {
    if args.binaries.is_empty() {
        bail!("No binary container files provided to execute");
    }

    let heap_size = parse_byte_size(&args.heap_size)?;

    let mut runtime_options = RuntimeOptions::new()
        .with_heap_size(heap_size)
        .with_max_call_stack_depth(args.max_stack)
        .with_args(args.args)
        .with_safe_mode(args.safe)
        .with_native_function("println", builtin_println)
        .with_native_function("print", builtin_print)
        .with_native_function("blt::io::println", builtin_println)
        .with_native_function("blt::io::print", builtin_print)
        .with_native_function("Console::println", builtin_println)
        .with_native_function("Console::print", builtin_print);

    // Set optional entry point override
    if let Some(entry) = &args.entry {
        runtime_options = runtime_options.with_entry_point(entry);
    }

    // Load all binary containers equally as peer modules
    for binary_path in &args.binaries {
        let bytes = fs::read(binary_path).with_context(|| {
            format!(
                "Failed to read binary container '{}'",
                binary_path.display()
            )
        })?;
        let program = BinaryProgram::from_bytes(&bytes).with_context(|| {
            format!(
                "Failed to deserialize binary container from '{}'",
                binary_path.display()
            )
        })?;
        runtime_options = runtime_options.with_binary(program);
    }

    // Initialize RuntimeContext and VirtualMachine
    let context = RuntimeContext::new(runtime_options)
        .map_err(|e| anyhow::anyhow!("Runtime initialization failed: {e}"))?;

    let mut vm = VirtualMachine::new(context);

    // Run virtual machine
    if let Err(err) = vm.run() {
        bail!("Runtime error: {err}");
    }

    let exit_code = vm.exit_code.unwrap_or(0);

    if args.verbose {
        println!("\n--- Solix Execution Summary ---");
        println!("Exit Code:    {exit_code}");
        println!("Return Value: {:?}", vm.last_return_value);
        if let Err(leak) = vm.verify_clean_memory() {
            eprintln!("Warning: Memory integrity check reported: {leak:?}");
        } else {
            println!("Memory Check: Clean (0 leaks detected)");
        }
    }

    Ok(exit_code)
}
