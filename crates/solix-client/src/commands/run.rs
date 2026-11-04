//! The `solix run` command driver.

#![forbid(unsafe_code)]

use crate::cli::RunArgs;
use crate::diagnostics::render_diagnostics;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::manifest::manifest_to_source;
use solix_compilation::pipeline::CompilationPipeline;
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};
use std::fs;
use std::path::PathBuf;

/// Parses a human-readable memory size string (e.g., `"64MB"`, `"16KB"`, `"1048576"`) into bytes.
fn parse_byte_size(size_str: &str) -> Result<usize> {
    let trimmed = size_str.trim();
    if trimmed.is_empty() {
        return Ok(32 * 1024 * 1024);
    }

    let upper = trimmed.to_uppercase();
    if let Some(num_str) = upper.strip_suffix("GB") {
        let byte_value: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(byte_value * 1024 * 1024 * 1024)
    } else if let Some(num_str) = upper.strip_suffix("MB") {
        let byte_value: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(byte_value * 1024 * 1024)
    } else if let Some(num_str) = upper.strip_suffix("KB") {
        let byte_value: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(byte_value * 1024)
    } else if let Some(num_str) = upper.strip_suffix('B') {
        let byte_value: usize = num_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(byte_value)
    } else {
        let byte_value: usize = trimmed
            .parse()
            .with_context(|| format!("Invalid number in size '{size_str}'"))?;
        Ok(byte_value)
    }
}

/// Executes the `run` subcommand.
///
/// If any `.slx` source files are passed, they are automatically compiled on-the-fly
/// into an in-memory binary container, merged with any external libraries or binaries,
/// and executed directly in the virtual machine.
///
/// # Errors
/// Returns an error if source compilation, binary loading, VM initialization, or execution fails.
pub fn execute_run(args: RunArgs) -> Result<i32> {
    if args.binaries.is_empty() {
        bail!("No binary container or source files provided to execute");
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

    // Partition target paths into source files (*.slx) and precompiled binaries (*.slxb)
    let mut source_paths: Vec<&PathBuf> = Vec::new();
    let mut binary_paths: Vec<&PathBuf> = Vec::new();

    for target_path in &args.binaries {
        let is_source = target_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|ext_str| ext_str.eq_ignore_ascii_case("slx"))
            .unwrap_or(false);

        if is_source {
            source_paths.push(target_path);
        } else {
            binary_paths.push(target_path);
        }
    }

    // If source files are provided, compile them on-the-fly in memory
    if !source_paths.is_empty() {
        let first_stem = source_paths
            .first()
            .and_then(|source_path| source_path.file_stem())
            .and_then(|stem| stem.to_str())
            .unwrap_or("app");

        let mut compilation_options = CompilationOptions::new()
            .with_package_metadata(first_stem, "0.1.0", "")
            .with_builtins(!args.no_builtins);

        for source_path in &source_paths {
            let source_code = fs::read_to_string(source_path).with_context(|| {
                format!("Failed to read source file '{}'", source_path.display())
            })?;
            compilation_options =
                compilation_options.with_source(source_path.display().to_string(), source_code);
        }

        // Auto-extract and mount dependency API headers from any `-l, --lib` binaries
        for lib_path in &args.libs {
            let lib_bytes = fs::read(lib_path).with_context(|| {
                format!("Failed to read dependency binary '{}'", lib_path.display())
            })?;
            let lib_bin = BinaryProgram::from_bytes(&lib_bytes).with_context(|| {
                format!(
                    "Failed to decode binary container from '{}'",
                    lib_path.display()
                )
            })?;
            let manifest_json = extract_api_manifest(&lib_bin).with_context(|| {
                format!(
                    "Failed to extract export manifest from '{}'",
                    lib_path.display()
                )
            })?;
            let (api_filename, api_source) =
                manifest_to_source(&manifest_json).map_err(|error| anyhow::anyhow!("{error:?}"))?;

            compilation_options = compilation_options.with_source(api_filename, api_source);
        }

        let mut pipeline = CompilationPipeline::standard();
        let mut compilation_context = CompilationContext::new(compilation_options);

        let compile_result = pipeline.execute(&mut compilation_context);

        if let Err(diagnostics) = compile_result {
            render_diagnostics(
                &compilation_context.source_names,
                &compilation_context.sources,
                &diagnostics,
            );
            bail!("Compilation aborted due to previous error(s)");
        }

        let artifacts = compilation_context
            .emission_artifacts
            .take()
            .ok_or_else(|| anyhow::anyhow!("Bytecode emission phase produced no artifacts"))?;

        runtime_options = runtime_options.with_binary(artifacts.program);
    }

    // Load any explicit library binary containers (*.slxb) into the VM
    for lib_path in &args.libs {
        let bytes = fs::read(lib_path).with_context(|| {
            format!("Failed to read dependency library '{}'", lib_path.display())
        })?;
        let program = BinaryProgram::from_bytes(&bytes).with_context(|| {
            format!(
                "Failed to deserialize dependency binary from '{}'",
                lib_path.display()
            )
        })?;
        runtime_options = runtime_options.with_binary(program);
    }

    // Load precompiled binary containers
    for binary_path in &binary_paths {
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
        .map_err(|error| anyhow::anyhow!("Runtime initialization failed: {error}"))?;

    let mut vm = VirtualMachine::new(context);

    // Run virtual machine
    if let Err(runtime_error) = vm.run() {
        bail!("Runtime error: {runtime_error}");
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
