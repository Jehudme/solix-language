//! The `solix compile` command driver.

#![forbid(unsafe_code)]

use crate::cli::CompileArgs;
use crate::diagnostics::render_diagnostics;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use crate::telemetry::CompileMetricsReport;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_common::solver::DependencyRequirement;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_compilation::manifest_to_source;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Recursively walks `directory_path` collecting all files with `.slx` extension in sorted order.
fn collect_slx_sources_recursively(
    directory_path: &std::path::Path,
    collected_sources: &mut Vec<PathBuf>,
) -> Result<()> {
    let mut entries = Vec::new();
    for entry_result in fs::read_dir(directory_path)
        .with_context(|| format!("Failed to read directory '{}'", directory_path.display()))?
    {
        let entry = entry_result?;
        entries.push(entry.path());
    }
    entries.sort();

    for child_path in entries {
        if child_path.is_dir() {
            collect_slx_sources_recursively(&child_path, collected_sources)?;
        } else if child_path
            .extension()
            .is_some_and(|extension| extension == "slx")
        {
            collected_sources.push(child_path);
        }
    }
    Ok(())
}

/// Executes the `compile` subcommand.
///
/// # Errors
/// Returns an error if file I/O fails or compilation encounters diagnostics.
pub fn execute_compile(args: CompileArgs) -> Result<()> {
    let start_total = Instant::now();

    // 1. Resolve build profile
    let profile = args
        .profile
        .parse::<solix_compilation::BuildProfile>()
        .map_err(|error_message| anyhow::anyhow!("{error_message}"))?;

    // 2. Resolve user source files (expanding directories recursively)
    let mut resolved_sources = Vec::new();
    for source_path in &args.sources {
        if source_path.is_dir() {
            collect_slx_sources_recursively(source_path, &mut resolved_sources)?;
        } else {
            resolved_sources.push(source_path.clone());
        }
    }

    if resolved_sources.is_empty() {
        bail!("No Solix source files (*.slx) found to compile");
    }

    // 3. Resolve package name, linking mode, and output destination
    let link_mode = args.resolved_link_mode();
    let package_name = args.name.unwrap_or_else(|| {
        resolved_sources
            .first()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .unwrap_or("app")
            .to_string()
    });

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{package_name}.slxb")));

    // 4. Initialize compilation options and attach package store solver
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_directory));

    let mut options = CompilationOptions::new()
        .with_profile(profile)
        .with_link_mode(link_mode)
        .with_package_metadata(
            &package_name,
            &args.version,
            args.description.as_deref().unwrap_or(""),
        )
        .with_builtins(!args.no_builtins)
        .with_solver(solver)
        .with_verify_checksums(!args.no_verify);

    if let Some(entry_sym) = args.entry {
        options = options.with_entry_point(Some(entry_sym));
    }

    if args.inline_threshold != 8 {
        options = options.with_inline_threshold(args.inline_threshold);
    }

    // 5. Register user source files
    for source_path in &resolved_sources {
        let code = fs::read_to_string(source_path)
            .with_context(|| format!("Failed to read source file '{}'", source_path.display()))?;
        options = options.with_source(source_path.display().to_string(), code);
    }

    // 6. Register explicit dependency specifications (--deps <name>@<version>)
    for dependency_specification in &args.deps {
        let requirement =
            DependencyRequirement::parse(dependency_specification).map_err(|parse_error| {
                anyhow::anyhow!(
                    "Invalid dependency requirement '{dependency_specification}': {parse_error}"
                )
            })?;
        options = options.with_dependency_spec(requirement);
    }

    // 7. Mount dependency library binaries or archives passed via `-l, --lib`
    let mut temp_archive_directories: Vec<tempfile::TempDir> = Vec::new();
    for lib_path in &args.libs {
        if lib_path
            .extension()
            .is_some_and(|extension| extension == "slxarc")
        {
            let temp_dir = tempfile::Builder::new()
                .prefix("solix_compile_")
                .tempdir()
                .map_err(|error| {
                    anyhow::anyhow!(
                        "Failed to create temporary directory for archive extraction: {error}"
                    )
                })?;
            let loaded_archive =
                solix_common::project::SlxarcArchive::extract_and_load(lib_path, temp_dir.path())
                    .map_err(|load_error| {
                    anyhow::anyhow!(
                        "Failed to extract dependency archive '{}': {load_error}",
                        lib_path.display()
                    )
                })?;
            for binary_program in loaded_archive.binaries {
                options = options.with_dependency_binary(binary_program);
            }
            for secondary_program in loaded_archive.secondary_binaries {
                options = options.with_dependency_binary(secondary_program);
            }
            temp_archive_directories.push(temp_dir);
        } else {
            let lib_bytes = fs::read(lib_path).with_context(|| {
                format!("Failed to read dependency library '{}'", lib_path.display())
            })?;
            let lib_binary = BinaryProgram::from_bytes(&lib_bytes).with_context(|| {
                format!(
                    "Failed to decode binary container from '{}'",
                    lib_path.display()
                )
            })?;
            options = options.with_dependency_binary(lib_binary);
        }
    }

    // 8. Build and execute pipeline
    let mut pipeline = if args.check {
        CompilationPipeline::check()
    } else {
        CompilationPipeline::for_profile(profile)
    };

    let mut context = CompilationContext::new(options);

    let compile_result = pipeline.execute(&mut context);

    if let Err(diagnostics) = compile_result {
        render_diagnostics(&context.source_names, &context.sources, &diagnostics);
        bail!("Compilation aborted due to previous error(s)");
    }

    let elapsed = start_total.elapsed();

    // 6. Output handling
    if args.check {
        let is_json_metrics = args.metrics_json || args.metrics.as_deref() == Some("json");
        let is_text_metrics = args.metrics.as_deref() == Some("text");

        if is_json_metrics {
            let telemetry = context.telemetry.unwrap_or_default();
            let report = CompileMetricsReport {
                package_name,
                status: "success".to_string(),
                total_time_us: telemetry.total_time_us,
                throughput_mb_s: telemetry.throughput_mb_s,
                source_bytes: telemetry.source_bytes,
                total_tokens: telemetry.total_tokens,
                total_instructions: telemetry.total_instructions,
                binary_size_bytes: telemetry.binary_size_bytes,
                phases: telemetry.phases,
            };
            let json_output = serde_json::to_string_pretty(&report)
                .context("Failed to serialize compiler metrics to JSON")?;
            println!("{json_output}");
        } else if is_text_metrics {
            let telemetry = context.telemetry.unwrap_or_default();
            println!("=== Solix Compiler Metrics ===");
            println!("Package:      {package_name}");
            println!("Status:       check passed");
            println!(
                "Total Time:   {:.2} ms",
                telemetry.total_time_us as f64 / 1000.0
            );
            println!("Throughput:   {:.2} MB/s", telemetry.throughput_mb_s);
            println!(
                "Sources:      {} bytes, {} tokens",
                telemetry.source_bytes, telemetry.total_tokens
            );
            println!("Phases:");
            for phase in &telemetry.phases {
                println!("  - {:<32} : {} µs", phase.name, phase.duration_us);
            }
        } else if args.time {
            println!(
                "Check passed in {:.2}ms (0 errors)",
                elapsed.as_secs_f64() * 1000.0
            );
        } else {
            println!("Check passed (0 errors)");
        }
        return Ok(());
    }

    let artifacts = context
        .emission_artifacts
        .as_ref()
        .context("Missing emission artifacts")?;

    let binary_bytes = artifacts.program.to_bytes();

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory '{}'", parent.display()))?;
    }

    fs::write(&output_path, &binary_bytes)
        .with_context(|| format!("Failed to write binary output '{}'", output_path.display()))?;

    // Optional API header export
    if let Some(api_out_path) = args.emit_api {
        let manifest_json = extract_api_manifest(&artifacts.program)
            .with_context(|| "Failed to serialize export manifest")?;
        let (_, api_source) =
            manifest_to_source(&manifest_json).map_err(|error| anyhow::anyhow!("{error:?}"))?;

        if let Some(parent) = api_out_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(&api_out_path, api_source).with_context(|| {
            format!(
                "Failed to write API header output '{}'",
                api_out_path.display()
            )
        })?;
        let is_json_metrics = args.metrics_json || args.metrics.as_deref() == Some("json");
        if !is_json_metrics {
            println!("Exported API header -> {}", api_out_path.display());
        }
    }

    let is_json_metrics = args.metrics_json || args.metrics.as_deref() == Some("json");
    let is_text_metrics = args.metrics.as_deref() == Some("text");

    if is_json_metrics {
        let telemetry = context.telemetry.unwrap_or_default();
        let report = CompileMetricsReport {
            package_name,
            status: "success".to_string(),
            total_time_us: telemetry.total_time_us,
            throughput_mb_s: telemetry.throughput_mb_s,
            source_bytes: telemetry.source_bytes,
            total_tokens: telemetry.total_tokens,
            total_instructions: telemetry.total_instructions,
            binary_size_bytes: telemetry.binary_size_bytes,
            phases: telemetry.phases,
        };
        let json_output = serde_json::to_string_pretty(&report)
            .context("Failed to serialize compiler metrics to JSON")?;
        println!("{json_output}");
    } else if is_text_metrics {
        let telemetry = context.telemetry.unwrap_or_default();
        println!("=== Solix Compiler Metrics ===");
        println!("Package:      {package_name}");
        println!("Status:       success");
        println!(
            "Total Time:   {:.2} ms",
            telemetry.total_time_us as f64 / 1000.0
        );
        println!("Throughput:   {:.2} MB/s", telemetry.throughput_mb_s);
        println!(
            "Sources:      {} bytes, {} tokens, {} instructions",
            telemetry.source_bytes, telemetry.total_tokens, telemetry.total_instructions
        );
        println!("Binary Size:  {} bytes", telemetry.binary_size_bytes);
        println!("Phases:");
        for phase in &telemetry.phases {
            println!("  - {:<32} : {} µs", phase.name, phase.duration_us);
        }
    } else if args.time {
        println!(
            "Compiled '{}' -> {} in {:.2}ms",
            package_name,
            output_path.display(),
            elapsed.as_secs_f64() * 1000.0
        );
    } else {
        println!("Compiled '{}' -> {}", package_name, output_path.display());
    }

    if args.install {
        crate::commands::pkg::install_package_artifact(
            &output_path,
            None,
            args.force,
            Some(&args.profile),
            args.pkg_dir.as_deref(),
        )?;
    }

    Ok(())
}
