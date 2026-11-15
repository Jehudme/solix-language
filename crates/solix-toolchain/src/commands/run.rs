//! The `solix run` command driver.

#![forbid(unsafe_code)]

use crate::cli::RunArgs;
use crate::diagnostics::render_diagnostics;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use crate::telemetry::{
    ExecutionTelemetry, MemoryTelemetry, RunMetricsReport, TimingsTelemetry,
    capture_process_telemetry,
};
use anyhow::{Context, Result, bail};
use solix_common::bytecode::BinaryProgram;
use solix_common::project::SlxarcArchive;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::pipeline::CompilationPipeline;
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

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
    let total_start = std::time::Instant::now();
    let binary_load_timer = std::time::Instant::now();

    if args.binaries.is_empty() {
        bail!("No binary container or source files provided to execute");
    }

    let heap_size = parse_byte_size(&args.heap_size)?;

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_directory));

    let mut runtime_options = RuntimeOptions::new()
        .with_heap_size(heap_size)
        .with_max_call_stack_depth(args.max_stack)
        .with_args(args.args)
        .with_safe_mode(args.safe)
        .with_solver(solver.clone())
        .with_native_function("println", builtin_println)
        .with_native_function("print", builtin_print)
        .with_native_function("blt::io::println", builtin_println)
        .with_native_function("blt::io::print", builtin_print)
        .with_native_function("Console::println", builtin_println)
        .with_native_function("Console::print", builtin_print)
        .with_verify_checksums(!args.no_verify);

    if args.no_verify {
        eprintln!(
            "warning: checksum verification bypassed (--no-verify); running without cryptographic validation"
        );
    }

    // Set optional entry point override
    if let Some(entry) = &args.entry {
        runtime_options = runtime_options.with_entry_point(entry);
    }

    let temp_cache_dir = tempfile::Builder::new()
        .prefix("solix_run_")
        .tempdir()
        .with_context(|| "Failed to create temporary directory for run extraction")?;
    let mut loaded_binary_programs = Vec::new();

    // Mount extra libraries passed via -l, --lib
    for lib_path in &args.libs {
        let extension = lib_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        if extension.eq_ignore_ascii_case("so")
            || extension.eq_ignore_ascii_case("dll")
            || extension.eq_ignore_ascii_case("dylib")
        {
            runtime_options = runtime_options.with_native_library(lib_path);
        } else if extension.eq_ignore_ascii_case("slxarc") {
            let loaded_archive = SlxarcArchive::extract_and_load(lib_path, temp_cache_dir.path())
                .with_context(|| {
                format!(
                    "Failed to extract dependency archive '{}'",
                    lib_path.display()
                )
            })?;
            for native_lib in loaded_archive.native_libraries {
                runtime_options = runtime_options.with_native_library(native_lib);
            }
            loaded_binary_programs.extend(loaded_archive.binaries);
            loaded_binary_programs.extend(loaded_archive.secondary_binaries);
        } else if extension.eq_ignore_ascii_case("slxb") {
            let bytes = fs::read(lib_path).with_context(|| {
                format!("Failed to read dependency binary '{}'", lib_path.display())
            })?;
            let program = BinaryProgram::from_bytes(&bytes).with_context(|| {
                format!(
                    "Failed to deserialize dependency binary from '{}'",
                    lib_path.display()
                )
            })?;
            loaded_binary_programs.push(program);
        }
    }

    // Partition target paths into source files (*.slx), archives (*.slxarc), and precompiled binaries (*.slxb)
    let mut source_paths: Vec<&PathBuf> = Vec::new();
    let mut binary_paths: Vec<&PathBuf> = Vec::new();

    for target_path in &args.binaries {
        let extension = target_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if extension.eq_ignore_ascii_case("slx") {
            source_paths.push(target_path);
        } else if extension.eq_ignore_ascii_case("slxarc") {
            let loaded_archive =
                SlxarcArchive::extract_and_load(target_path, temp_cache_dir.path()).with_context(
                    || {
                        format!(
                            "Failed to extract archive container '{}'",
                            target_path.display()
                        )
                    },
                )?;
            let archive_manifest = loaded_archive.manifest.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid archive container '{}': missing mandatory 'manifest.toml'",
                    target_path.display()
                )
            })?;
            for native_lib in loaded_archive.native_libraries {
                runtime_options = runtime_options.with_native_library(native_lib);
            }
            if runtime_options.entry_point.is_none() {
                let selected_entry = if args.test {
                    archive_manifest
                        .entrypoints
                        .test
                        .as_ref()
                        .or(archive_manifest.entrypoints.main.as_ref())
                } else {
                    archive_manifest.entrypoints.main.as_ref()
                };
                if let Some(target_symbol) = selected_entry {
                    runtime_options = runtime_options.with_entry_point(target_symbol);
                }
            }
            loaded_binary_programs.extend(loaded_archive.binaries);
            loaded_binary_programs.extend(loaded_archive.secondary_binaries);
        } else {
            binary_paths.push(target_path);
        }
    }

    // Pre-load and deserialize any provided binary containers (*.slxb)
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
        if runtime_options.entry_point.is_none()
            && source_paths.is_empty()
            && let Some(entry) = &program.entry_point
            && !entry.is_empty()
        {
            runtime_options = runtime_options.with_entry_point(entry);
        }
        loaded_binary_programs.push(program);
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
            .with_builtins(!args.no_builtins)
            .with_inline_threshold(args.inline_threshold)
            .with_solver(solver.clone());

        if let Some(entry) = &args.entry {
            compilation_options = compilation_options.with_entry_point(Some(entry.clone()));
        }

        for source_path in &source_paths {
            let source_code = fs::read_to_string(source_path).with_context(|| {
                format!("Failed to read source file '{}'", source_path.display())
            })?;
            compilation_options =
                compilation_options.with_source(source_path.display().to_string(), source_code);
        }

        // Mount preloaded binary programs into compilation_options as dependency binaries
        for binary_program in &loaded_binary_programs {
            compilation_options =
                compilation_options.with_dependency_binary(binary_program.clone());
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

    // Mount all precompiled binary containers (*.slxb) into the VM runtime
    for binary_program in loaded_binary_programs {
        runtime_options = runtime_options.with_binary(binary_program);
    }

    // Initialize RuntimeContext and VirtualMachine
    let context = RuntimeContext::new(runtime_options)
        .map_err(|error| anyhow::anyhow!("Runtime initialization failed: {error}"))?;

    let mut vm = VirtualMachine::new(context);
    let binary_load_time_us = binary_load_timer.elapsed().as_micros() as u64;

    let entry_id = vm
        .context
        .entry_point_function_id
        .ok_or_else(|| anyhow::anyhow!("Entry point not found in loaded binaries"))?;
    vm.validate_cli_entry_point(entry_id)
        .map_err(|error| anyhow::anyhow!("Invalid entry point signature: {error}"))?;

    // Run virtual machine
    let run_result = vm.run();
    let total_process_wall_time_us = total_start.elapsed().as_micros() as u64;

    let exit_code = vm.exit_code.unwrap_or(0);
    let vm_telemetry = vm.telemetry();
    let heap_telemetry = vm.heap.telemetry();
    let process_telemetry = capture_process_telemetry();

    let heap_amplification_ratio = if heap_telemetry.peak_allocated_bytes > 0 {
        let raw_ratio = (process_telemetry.peak_rss_kb as f64 * 1024.0)
            / (heap_telemetry.peak_allocated_bytes as f64);
        (raw_ratio * 100.0).round() / 100.0
    } else {
        1.0
    };

    let workload_name = args
        .binaries
        .first()
        .and_then(|path| path.file_stem())
        .and_then(|stem| stem.to_str())
        .unwrap_or("app")
        .to_string();

    let status_str = if run_result.is_ok() {
        "success"
    } else {
        "error"
    };

    let is_json_metrics = args.metrics_json || args.metrics.as_deref() == Some("json");
    let is_text_metrics = args.metrics.as_deref() == Some("text");

    if is_json_metrics {
        let report = RunMetricsReport {
            workload: workload_name,
            status: status_str.to_string(),
            exit_code,
            timings: TimingsTelemetry {
                total_process_wall_time_us,
                pure_vm_execution_time_us: vm_telemetry.pure_execution_time_ns / 1_000,
                binary_load_time_us,
                static_init_time_us: 0,
            },
            memory: MemoryTelemetry {
                peak_virtual_heap_bytes: heap_telemetry.peak_allocated_bytes,
                peak_os_rss_kb: process_telemetry.peak_rss_kb,
                heap_amplification_ratio,
                total_allocations: heap_telemetry.total_allocations_count,
                total_deallocations: heap_telemetry.total_deallocations_count,
                active_leaks: heap_telemetry.active_live_objects,
            },
            execution: ExecutionTelemetry {
                instructions_dispatched: vm_telemetry.instructions_dispatched,
                peak_stack_depth: vm_telemetry.peak_call_stack_depth,
                quickened_calls: vm_telemetry.quickened_instructions_count,
            },
        };
        let json_output = serde_json::to_string_pretty(&report)
            .context("Failed to serialize run metrics to JSON")?;
        println!("{json_output}");
    } else if is_text_metrics {
        println!("=== Solix VM Telemetry ===");
        println!("Workload:       {workload_name}");
        println!("Status:         {status_str} (exit code: {exit_code})");
        println!("Timings:");
        println!(
            "  Total Process Wall Time : {:.2} ms",
            total_process_wall_time_us as f64 / 1000.0
        );
        println!(
            "  Pure VM Execution Time  : {:.2} ms",
            vm_telemetry.pure_execution_time_ns as f64 / 1_000_000.0
        );
        println!(
            "  Binary Load Time        : {:.2} ms",
            binary_load_time_us as f64 / 1000.0
        );
        println!("Memory:");
        println!(
            "  Peak VirtualHeap Bytes  : {} B",
            heap_telemetry.peak_allocated_bytes
        );
        println!(
            "  Peak Host OS RSS        : {} KB",
            process_telemetry.peak_rss_kb
        );
        println!(
            "  Heap Amplification Ratio: {:.2}x",
            heap_amplification_ratio
        );
        println!(
            "  Total Allocations       : {}",
            heap_telemetry.total_allocations_count
        );
        println!(
            "  Total Deallocations     : {}",
            heap_telemetry.total_deallocations_count
        );
        println!(
            "  Active Leaks            : {}",
            heap_telemetry.active_live_objects
        );
        println!("Execution:");
        println!(
            "  Instructions Dispatched : {}",
            vm_telemetry.instructions_dispatched
        );
        println!(
            "  Peak Call Stack Depth   : {}",
            vm_telemetry.peak_call_stack_depth
        );
        println!(
            "  Quickened Instructions  : {}",
            vm_telemetry.quickened_instructions_count
        );
    }

    if let Err(runtime_error) = run_result {
        bail!("Runtime error: {runtime_error}");
    }

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
