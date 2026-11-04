//! The `solix compile` command driver.

#![forbid(unsafe_code)]

use crate::cli::CompileArgs;
use crate::diagnostics::render_diagnostics;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_compilation::manifest_to_source;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Executes the `compile` subcommand.
///
/// # Errors
/// Returns an error if file I/O fails or compilation encounters diagnostics.
pub fn execute_compile(args: CompileArgs) -> Result<()> {
    let start_total = Instant::now();

    // 1. Resolve package name and output destination
    let package_name = args.name.unwrap_or_else(|| {
        args.sources
            .first()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string()
    });

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{package_name}.slxb")));

    // 2. Initialize compilation options
    let mut options = CompilationOptions::new()
        .with_package_metadata(
            &package_name,
            &args.version,
            args.description.as_deref().unwrap_or(""),
        )
        .with_builtins(!args.no_builtins);

    // 3. Register user source files
    for source_path in &args.sources {
        let code = fs::read_to_string(source_path)
            .with_context(|| format!("Failed to read source file '{}'", source_path.display()))?;
        options = options.with_source(source_path.display().to_string(), code);
    }

    // 4. Auto-extract and mount dependency API headers from `-l, --lib` binaries
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
            manifest_to_source(&manifest_json).map_err(|e| anyhow::anyhow!("{e:?}"))?;

        options = options.with_source(api_filename, api_source);
    }

    // 5. Build and execute pipeline
    let mut pipeline = if args.check {
        CompilationPipeline::lsp()
    } else {
        CompilationPipeline::standard()
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
        if args.time {
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
            manifest_to_source(&manifest_json).map_err(|e| anyhow::anyhow!("{e:?}"))?;

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
        println!("Exported API header -> {}", api_out_path.display());
    }

    if args.time {
        println!(
            "Compiled '{}' -> {} in {:.2}ms",
            package_name,
            output_path.display(),
            elapsed.as_secs_f64() * 1000.0
        );
    } else {
        println!("Compiled '{}' -> {}", package_name, output_path.display());
    }

    Ok(())
}
