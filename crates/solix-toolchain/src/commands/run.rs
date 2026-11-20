//! The `solix run` command driver.

#![forbid(unsafe_code)]

use crate::cli::{BuildArgs, RunArgs, RunPackageArgs, RunSourceArgs, RunSubcommand};
use crate::diagnostics::render_diagnostics;
use crate::registry::{PackageRecord, PackageRegistry, parse_package_spec};
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use crate::telemetry::{
    ExecutionTelemetry, MemoryTelemetry, RunMetricsReport, TimingsTelemetry,
    capture_process_telemetry,
};
use anyhow::{Context, Result, bail};
use solix_common::bytecode::BinaryProgram;
use solix_common::project::SlxarcArchive;
use solix_common::solver::PackageVersion;
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

/// Executes `solix run package <path>`.
///
/// # Errors
/// Returns an error if building or executing the package fails.
pub fn execute_run_package(args: RunPackageArgs) -> Result<i32> {
    let resolved_target =
        solix_common::canonicalize_host_path(&args.target).with_context(|| {
            format!(
                "Failed to resolve target package at '{}'",
                args.target.display()
            )
        })?;

    let effective_profile = args
        .resolved_profile()
        .unwrap_or_else(|| "release".to_string());

    let (package_binary, entry_override) = if resolved_target.is_dir() {
        let manifest_path = solix_common::project::find_manifest_path(&resolved_target)
            .ok_or_else(|| {
                anyhow::anyhow!("No manifest.yaml found in '{}'", resolved_target.display())
            })?;
        let manifest = solix_common::project::ProjectManifest::from_file(&manifest_path)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let package_name = manifest.configuration.name.clone();

        let slxp_candidate = resolved_target
            .join("build")
            .join(&effective_profile)
            .join(format!("{package_name}.slxp"));
        let slxarc_candidate = resolved_target
            .join("build")
            .join(&effective_profile)
            .join(format!("{package_name}.slxarc"));
        let root_binaries = resolved_target.join("binaries.slxb");

        let artifact_to_run = if slxp_candidate.exists() {
            slxp_candidate
        } else if slxarc_candidate.exists() {
            slxarc_candidate
        } else if root_binaries.exists() {
            root_binaries
        } else {
            crate::commands::execute_build(BuildArgs {
                path: resolved_target.clone(),
                platform: None,
                profile: Some(effective_profile.clone()),
                debug: false,
                release: false,
                test: false,
                pkg_dir: args.pkg_dir.clone(),
                install: false,
                force: true,
                check_env: false,
                target: None,
            })?;
            if slxp_candidate.exists() {
                slxp_candidate
            } else if slxarc_candidate.exists() {
                slxarc_candidate
            } else {
                resolved_target
                    .join("build")
                    .join(&effective_profile)
                    .join(format!("{package_name}.slxp"))
            }
        };
        let entry_point = args.entry.or_else(|| manifest.main_entrypoint());
        (artifact_to_run, entry_point)
    } else {
        (resolved_target, args.entry)
    };

    execute_run(RunArgs {
        subcommand: None,
        binaries: vec![package_binary],
        libs: Vec::new(),
        entry: entry_override,
        profile: Some(effective_profile),
        debug: args.debug,
        release: args.release,
        test: args.test,
        inline_threshold: 8,
        heap_size: args.heap_size,
        max_stack: args.max_stack,
        cycle_collector: args.cycle_collector,
        cycle_threshold: args.cycle_threshold,
        cycle_budget_us: args.cycle_budget_us,
        verbose: args.verbose,
        safe: args.safe,
        metrics: args.metrics,
        metrics_json: args.metrics_json,
        args: args.args,
        pkg_dir: args.pkg_dir,
        no_verify: args.no_verify,
    })
}

/// Executes `solix run source <file.slx>`.
///
/// # Errors
/// Returns an error if compiling or executing the source file fails.
pub fn execute_run_source(args: RunSourceArgs) -> Result<i32> {
    execute_run(RunArgs {
        subcommand: None,
        binaries: args.sources,
        libs: args.libs,
        entry: args.entry,
        profile: args.profile,
        debug: args.debug,
        release: args.release,
        test: args.test,
        inline_threshold: args.inline_threshold,
        heap_size: args.heap_size,
        max_stack: args.max_stack,
        cycle_collector: args.cycle_collector,
        cycle_threshold: args.cycle_threshold,
        cycle_budget_us: args.cycle_budget_us,
        verbose: args.verbose,
        safe: args.safe,
        metrics: args.metrics,
        metrics_json: args.metrics_json,
        args: args.args,
        pkg_dir: args.pkg_dir,
        no_verify: args.no_verify,
    })
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
    if let Some(subcmd) = args.subcommand {
        return match subcmd {
            RunSubcommand::Package(package_args) => execute_run_package(package_args),
            RunSubcommand::Source(source_args) => execute_run_source(source_args),
        };
    }

    let total_start = std::time::Instant::now();
    let binary_load_timer = std::time::Instant::now();
    let effective_profile_arg = args.resolved_profile();

    if args.binaries.is_empty() {
        bail!("No binary container or source files provided to execute");
    }

    let heap_size = parse_byte_size(&args.heap_size)?;

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let mut bundled_bootstrapped = false;
    let mut ensure_bundled = || {
        if !bundled_bootstrapped {
            let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);
            bundled_bootstrapped = true;
        }
    };
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_directory.clone()));

    let cycle_collector_enabled = !matches!(
        args.cycle_collector.to_lowercase().as_str(),
        "off" | "false" | "0"
    );
    if args.cycle_threshold == 0 {
        bail!(
            "[E0341] Invalid cycle collector configuration parameter: cycle threshold must be greater than zero."
        );
    }

    let is_json_metrics = args.metrics_json || args.metrics.as_deref() == Some("json");
    let is_text_metrics = args.metrics.as_deref() == Some("text");

    let mut runtime_options = RuntimeOptions::new()
        .with_heap_size(heap_size)
        .with_max_call_stack_depth(args.max_stack)
        .with_cycle_collector(cycle_collector_enabled)
        .with_cycle_threshold(args.cycle_threshold)
        .with_args(args.args)
        .with_safe_mode(args.safe)
        .with_metrics_tracking(is_json_metrics || is_text_metrics)
        .with_solver(solver.clone())
        .with_native_function("println", builtin_println)
        .with_native_function("print", builtin_print)
        .with_native_function("Console::println", builtin_println)
        .with_native_function("Console::print", builtin_print)
        .with_verify_checksums(!args.no_verify);

    if let Some(budget_us) = args.cycle_budget_us {
        runtime_options = runtime_options.with_cycle_budget_us(budget_us);
    }

    if args.no_verify {
        eprintln!(
            "warning: checksum verification bypassed (--no-verify); running without cryptographic validation"
        );
    }

    // Set optional entry point override
    if let Some(entry) = &args.entry {
        runtime_options = runtime_options.with_entry_point(entry);
    }

    let mut temp_cache_dir: Option<tempfile::TempDir> = None;
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
        } else if extension.eq_ignore_ascii_case("slxp")
            || extension.eq_ignore_ascii_case("slxarc")
            || extension.eq_ignore_ascii_case("slxpac")
        {
            let cache_root = ensure_temp_cache_dir(&mut temp_cache_dir)?;
            let loaded_archive = SlxarcArchive::extract_and_load(lib_path, cache_root)
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

    let mut target_binaries = args.binaries.clone();
    let mut resolved_spec_profile: Option<String> = None;

    if target_binaries.len() == 1 {
        let single_target = &target_binaries[0];
        let spec_string = single_target.to_string_lossy().to_string();
        let looks_like_package_spec = !single_target.exists()
            && (spec_string.contains(':')
                || spec_string.contains('@')
                || single_target.extension().is_none()
                || single_target.extension().is_some_and(|extension| {
                    !extension.eq_ignore_ascii_case("slx")
                        && !extension.eq_ignore_ascii_case("slxb")
                        && !extension.eq_ignore_ascii_case("slxarc")
                        && !extension.eq_ignore_ascii_case("slxpac")
                }));

        if looks_like_package_spec {
            ensure_bundled();
            let (package_name, spec_profile, spec_version) = parse_package_spec(&spec_string);
            let effective_profile = spec_profile
                .as_deref()
                .or(effective_profile_arg.as_deref())
                .unwrap_or("release");
            resolved_spec_profile = spec_profile.clone();

            let registry = PackageRegistry::load_or_sync(&store_directory)?;

            let mut candidates: Vec<&PackageRecord> = registry
                .packages
                .iter()
                .filter(|record| {
                    record.name == package_name && record.build_profile == effective_profile
                })
                .collect();

            if candidates.is_empty() && spec_profile.is_none() && effective_profile_arg.is_none() {
                candidates = registry
                    .packages
                    .iter()
                    .filter(|record| record.name == package_name)
                    .collect();
            }

            if let Some(ref version_query) = spec_version {
                candidates.retain(|record| &record.version == version_query);
            }

            candidates.sort_by(|record_alpha, record_beta| {
                let parsed_alpha = PackageVersion::parse(&record_alpha.version).ok();
                let parsed_beta = PackageVersion::parse(&record_beta.version).ok();
                parsed_beta.cmp(&parsed_alpha)
            });

            if let Some(matched_record) = candidates.first() {
                if !matched_record.artifact_path.exists() {
                    bail!(
                        "Package '{}@{}' ({}) artifact was not found at expected path '{}'",
                        matched_record.name,
                        matched_record.version,
                        matched_record.build_profile,
                        matched_record.artifact_path.display()
                    );
                }
                target_binaries = vec![matched_record.artifact_path.clone()];
            } else {
                let installed_summary: Vec<String> = registry
                    .packages
                    .iter()
                    .map(|record| {
                        format!(
                            "{}:{}@{}",
                            record.name, record.build_profile, record.version
                        )
                    })
                    .collect();
                bail!(
                    "Package '{}' (profile: '{}') was not found in installed packages store at '{}'. Available packages: [{}]",
                    package_name,
                    effective_profile,
                    store_directory.display(),
                    installed_summary.join(", ")
                );
            }
        }
    }

    // Partition target paths into source files (*.slx), archives (*.slxarc), and precompiled binaries (*.slxb)
    let mut source_paths: Vec<&PathBuf> = Vec::new();
    let mut binary_paths: Vec<&PathBuf> = Vec::new();

    for target_path in &target_binaries {
        let extension = target_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if extension.eq_ignore_ascii_case("slx") {
            source_paths.push(target_path);
        } else if extension.eq_ignore_ascii_case("slxp")
            || extension.eq_ignore_ascii_case("slxarc")
            || extension.eq_ignore_ascii_case("slxpac")
        {
            let cache_root = ensure_temp_cache_dir(&mut temp_cache_dir)?;
            let is_slxpac = extension.eq_ignore_ascii_case("slxpac");
            let is_slxp = extension.eq_ignore_ascii_case("slxp");
            let kind_string = if is_slxp {
                "slxp"
            } else if is_slxpac {
                "slxpac"
            } else {
                "slxarc"
            };
            runtime_options = runtime_options.with_container_kind(kind_string);
            let loaded_archive = SlxarcArchive::extract_and_load(target_path, cache_root)
                .with_context(|| {
                    format!(
                        "Failed to extract archive container '{}'",
                        target_path.display()
                    )
                })?;
            let archive_manifest = loaded_archive.manifest.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid archive container '{}': missing or invalid 'manifest.yaml' / 'manifest.toml'",
                    target_path.display()
                )
            })?;
            runtime_options = runtime_options
                .with_container_resource_root(cache_root.join("resources"))
                .with_container_resources(
                    loaded_archive.resource_files.len() as u32,
                    loaded_archive.resource_total_bytes,
                );
            for native_lib in loaded_archive.native_libraries {
                runtime_options = runtime_options.with_native_library(native_lib);
            }
            if runtime_options.entry_point.is_none() {
                let explicit_profile = effective_profile_arg
                    .as_deref()
                    .or(resolved_spec_profile.as_deref());
                if let Some(target_profile) = explicit_profile {
                    let resolved_profile = archive_manifest
                        .resolve_profile(target_profile)
                        .map_err(|error| anyhow::anyhow!("{error}"))?;
                    if let Some(target_symbol) = resolved_profile.entrypoint {
                        runtime_options = runtime_options.with_entry_point(target_symbol);
                    }
                } else {
                    let primary_has_baked_entry =
                        loaded_archive.binaries.first().is_some_and(|binary| {
                            binary.entry_point_id.is_some()
                                || binary
                                    .entry_point
                                    .as_deref()
                                    .is_some_and(|entry| !entry.is_empty())
                        });
                    if !primary_has_baked_entry {
                        let fallback_profile = loaded_archive
                            .binaries
                            .first()
                            .map(|binary| binary.build_profile.as_str())
                            .filter(|profile_name| !profile_name.is_empty())
                            .unwrap_or("release");
                        let selected_entry = archive_manifest
                            .resolve_profile(fallback_profile)
                            .ok()
                            .and_then(|profile| profile.entrypoint)
                            .or_else(|| archive_manifest.main_entrypoint());
                        if let Some(ref target_symbol) = selected_entry {
                            runtime_options = runtime_options.with_entry_point(target_symbol);
                        }
                    }
                }
            }
            loaded_binary_programs.extend(loaded_archive.binaries);
            loaded_binary_programs.extend(loaded_archive.secondary_binaries);
        } else {
            if extension.eq_ignore_ascii_case("slxb") {
                runtime_options = runtime_options.with_container_kind("slxb");
            }
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
            .with_inline_threshold(args.inline_threshold)
            .with_emit_export_metadata(false)
            .with_solver(solver.clone());

        let mut read_sources = Vec::with_capacity(source_paths.len());
        let mut needs_packages_or_manifest =
            !args.libs.is_empty() || !loaded_binary_programs.is_empty();
        for source_path in &source_paths {
            let source_code = fs::read_to_string(source_path).with_context(|| {
                format!("Failed to read source file '{}'", source_path.display())
            })?;
            if source_code.contains("import") || source_code.contains("extern") {
                needs_packages_or_manifest = true;
            }
            read_sources.push((source_path, source_code));
        }

        let parent_manifest = if needs_packages_or_manifest || effective_profile_arg.is_some() {
            source_paths
                .first()
                .and_then(|source_path| crate::commands::compile::find_parent_manifest(source_path))
                .and_then(|manifest_path| {
                    solix_common::project::ProjectManifest::from_file(&manifest_path).ok()
                })
        } else {
            None
        };

        if let Some(ref manifest) = parent_manifest {
            let target_profile = effective_profile_arg.as_deref().unwrap_or("release");
            let resolved_profile_config = if effective_profile_arg.is_some() {
                Some(
                    manifest
                        .resolve_profile(target_profile)
                        .map_err(|error| anyhow::anyhow!("{error}"))?,
                )
            } else {
                manifest.resolve_profile(target_profile).ok()
            };
            let deps = resolved_profile_config
                .as_ref()
                .map(|profile_cfg| profile_cfg.resolved_dependencies())
                .unwrap_or_else(|| manifest.resolved_dependencies());
            if !deps.is_empty() {
                ensure_bundled();
            }
            for resolved_dependency in deps {
                let spec_string = if resolved_dependency.build_profile.is_empty() {
                    format!(
                        "{}@{}",
                        resolved_dependency.package_name, resolved_dependency.version
                    )
                } else {
                    format!(
                        "{}@{}#{}",
                        resolved_dependency.package_name,
                        resolved_dependency.version,
                        resolved_dependency.build_profile
                    )
                };
                if let Ok(requirement) =
                    solix_common::solver::DependencyRequirement::parse(&spec_string)
                {
                    compilation_options = compilation_options.with_dependency_spec(requirement);
                }
            }
            if runtime_options.entry_point.is_none() && args.entry.is_none() {
                let selected_entry = resolved_profile_config
                    .as_ref()
                    .and_then(|profile| profile.entrypoint.clone())
                    .or_else(|| manifest.main_entrypoint());
                if let Some(ref target_symbol) = selected_entry {
                    compilation_options =
                        compilation_options.with_entry_point(Some(target_symbol.clone()));
                    runtime_options = runtime_options.with_entry_point(target_symbol);
                }
            }
        } else if needs_packages_or_manifest {
            ensure_bundled();
        }

        if let Some(entry) = &args.entry {
            compilation_options = compilation_options.with_entry_point(Some(entry.clone()));
        }

        for (source_path, source_code) in read_sources {
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

    let mut vm = VirtualMachine::try_new(context)
        .map_err(|error| anyhow::anyhow!("Virtual machine initialization failed: {error}"))?;
    let binary_load_time_us = binary_load_timer.elapsed().as_micros() as u64;

    let entry_id = vm.context.entry_point_function_id.ok_or_else(|| {
        if let Some(ref requested_entry) = args.entry {
            anyhow::anyhow!(
                "Runtime error: [E0301] Specified entrypoint symbol '{requested_entry}' was not found in the loaded program or container."
            )
        } else if let Some(ref configured_entry) = vm.context.options.entry_point {
            anyhow::anyhow!(
                "Runtime error: [E0301] Specified entrypoint symbol '{configured_entry}' was not found in the loaded program or container."
            )
        } else {
            anyhow::anyhow!("Entry point not found in loaded binaries")
        }
    })?;
    vm.validate_cli_entry_point(entry_id)
        .map_err(|error| anyhow::anyhow!("Invalid entry point signature: {error}"))?;

    // Run virtual machine
    let run_result = vm.run();
    let total_process_wall_time_us = total_start.elapsed().as_micros() as u64;
    let exit_code = vm.exit_code.unwrap_or(0);

    if is_json_metrics || is_text_metrics {
        let vm_telemetry = vm.telemetry();
        let heap_telemetry = vm.heap.telemetry();
        let active_guest_leaks = vm.active_guest_objects();
        let process_telemetry = capture_process_telemetry();

        let heap_amplification_ratio = if heap_telemetry.peak_allocated_bytes > 0 {
            let raw_ratio = (process_telemetry.peak_rss_kb as f64 * 1024.0)
                / (heap_telemetry.peak_allocated_bytes as f64);
            (raw_ratio * 100.0).round() / 100.0
        } else {
            1.0
        };

        let workload_name = target_binaries
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
                    active_leaks: active_guest_leaks,
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
        } else {
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
            println!("  Active Leaks            : {}", active_guest_leaks);
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
    }

    if let Err(runtime_error) = run_result {
        if let solix_runtime::exceptions::RuntimeError::EntryPointNotFound { symbol } =
            &runtime_error
        {
            bail!(
                "Runtime error: [E0301] Specified entrypoint symbol '{symbol}' was not found in the loaded program or container."
            );
        }
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

/// Lazily initializes the temporary archive extraction directory on first archive use.
fn ensure_temp_cache_dir(
    cache_dir_slot: &mut Option<tempfile::TempDir>,
) -> Result<&std::path::Path> {
    if cache_dir_slot.is_none() {
        let created = tempfile::Builder::new()
            .prefix("solix_run_")
            .tempdir()
            .with_context(|| "Failed to create temporary directory for run extraction")?;
        *cache_dir_slot = Some(created);
    }
    cache_dir_slot
        .as_ref()
        .map(tempfile::TempDir::path)
        .ok_or_else(|| anyhow::anyhow!("Temporary cache directory unavailable"))
}
