//! The `solix compile` command driver.

#![forbid(unsafe_code)]

use crate::cli::{CompileArgs, CompilePackageArgs, CompileSourceArgs, CompileSubcommand};
use crate::diagnostics::render_diagnostics;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use crate::telemetry::CompileMetricsReport;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_common::project::ProjectManifest;
use solix_common::solver::DependencyRequirement;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_compilation::manifest_to_source;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Recursively walks `directory_path` collecting all files with `.slx` extension in sorted order.
pub(crate) fn collect_slx_sources_recursively(
    directory_path: &Path,
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

/// Searches upward from the target source path looking for an enclosing `manifest.yaml` (or `manifest.toml`).
pub(crate) fn find_parent_manifest(initial_path: &Path) -> Option<PathBuf> {
    let mut current_directory = if initial_path.is_dir() {
        initial_path.to_path_buf()
    } else {
        initial_path.parent()?.to_path_buf()
    };

    let home_dir = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);

    for _ in 0..12 {
        if let Some(manifest_candidate) =
            solix_common::project::find_manifest_path(&current_directory)
        {
            return Some(manifest_candidate);
        }
        if current_directory.join(".git").exists()
            || home_dir
                .as_deref()
                .is_some_and(|home| home == current_directory.as_path())
        {
            break;
        }
        if !current_directory.pop() {
            break;
        }
    }
    None
}

/// Executes the `compile` subcommand, routing to `compile source`, `compile package`, or direct compilation.
///
/// # Errors
/// Returns an error if file I/O fails or compilation encounters diagnostics.
pub fn execute_compile(args: CompileArgs) -> Result<()> {
    if let Some(subcommand) = args.subcommand {
        return match subcommand {
            CompileSubcommand::Source(source_args) => execute_compile_source(source_args),
            CompileSubcommand::Package(package_args) => execute_compile_package(package_args),
        };
    }

    let source_args = CompileSourceArgs {
        sources: args.sources,
        output: args.output,
        deps_source: args.deps_source,
        deps_package: args.deps_package,
        deps_binaries: args.deps_binaries,
        deps: args.deps,
        libs: args.libs,
        name: args.name,
        version: args.version,
        description: args.description,
        profile: args.profile,
        emit_api: args.emit_api,
        check: args.check,
        time: args.time,
        metrics: args.metrics,
        metrics_json: args.metrics_json,
        inline_threshold: args.inline_threshold,
        entry: args.entry,
        link: args.link,
        r#static: args.r#static,
        pkg_dir: args.pkg_dir,
        install: args.install,
        force: args.force,
        no_verify: args.no_verify,
    };

    execute_compile_source(source_args)
}

/// Executes `solix compile source <files...>`.
///
/// # Errors
/// Returns an error if reading sources, resolving dependencies, or compilation fails.
pub fn execute_compile_source(args: CompileSourceArgs) -> Result<()> {
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

    // 2b. Attempt to discover parent manifest if within a Solix project
    let parent_manifest = resolved_sources
        .first()
        .and_then(|source_path| find_parent_manifest(source_path))
        .and_then(|manifest_path| {
            solix_common::project::ProjectManifest::from_file(&manifest_path).ok()
        });

    // 3. Resolve package name, linking mode, and output destination
    let link_mode = args.resolved_link_mode();
    let package_name = args.name.unwrap_or_else(|| {
        if let Some(ref manifest) = parent_manifest
            && !manifest.configuration.name.is_empty()
        {
            manifest.configuration.name.clone()
        } else {
            resolved_sources
                .first()
                .and_then(|path| path.file_stem())
                .and_then(|stem| stem.to_str())
                .unwrap_or("app")
                .to_string()
        }
    });

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{package_name}.slxb")));

    // 4. Initialize compilation options and attach package store solver
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_directory));

    let package_version = if args.version == "0.1.0"
        && let Some(ref manifest) = parent_manifest
        && !manifest.configuration.version.is_empty()
    {
        &manifest.configuration.version
    } else {
        &args.version
    };

    let package_description = match args.description.as_deref() {
        Some(explicit_description) => explicit_description,
        None => parent_manifest
            .as_ref()
            .and_then(|manifest| manifest.configuration.description.as_deref())
            .unwrap_or(""),
    };

    let embed_hashes = parent_manifest
        .as_ref()
        .and_then(|manifest| manifest.resolve_profile(&args.profile).ok())
        .map_or(
            !args.no_verify && link_mode != solix_compilation::LinkMode::Static,
            |resolved| {
                resolved.embed_hashes
                    && !args.no_verify
                    && link_mode != solix_compilation::LinkMode::Static
            },
        );

    let mut options = CompilationOptions::new()
        .with_profile(profile)
        .with_link_mode(link_mode)
        .with_package_metadata(&package_name, package_version, package_description)
        .with_solver(solver)
        .with_verify_checksums(!args.no_verify)
        .with_embed_dependency_hashes(embed_hashes);

    let effective_entry = args.entry.or_else(|| {
        parent_manifest.as_ref().and_then(|manifest| {
            manifest
                .resolve_profile(&args.profile)
                .ok()
                .and_then(|resolved| resolved.entrypoint)
                .or_else(|| manifest.main_entrypoint())
        })
    });

    if let Some(entry_sym) = effective_entry {
        options = options.with_entry_point(Some(entry_sym));
    }

    let declared_dynamic_libs = parent_manifest
        .as_ref()
        .map(|manifest| {
            manifest
                .resolve_profile(&args.profile)
                .map(|resolved| resolved.dynamic_libraries)
                .unwrap_or_else(|_| manifest.dynamic_libraries.clone())
        })
        .unwrap_or_default();
    if !declared_dynamic_libs.is_empty() {
        options = options.with_dynamic_libraries(declared_dynamic_libs);
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

    // 5b. Register explicit source dependencies passed via --deps-source
    for dependency_source_path in &args.deps_source {
        let mut source_files = Vec::new();
        if dependency_source_path.is_dir() {
            collect_slx_sources_recursively(dependency_source_path, &mut source_files)?;
        } else {
            source_files.push(dependency_source_path.clone());
        }
        for path_entry in source_files {
            let code = fs::read_to_string(&path_entry).with_context(|| {
                format!(
                    "Failed to read dependency source file '{}'",
                    path_entry.display()
                )
            })?;
            options = options.with_source(path_entry.display().to_string(), code);
        }
    }

    // 6. Register explicit dependency specifications (--deps <name>@<version>) or parent manifest dependencies
    for dependency_specification in &args.deps {
        let requirement =
            DependencyRequirement::parse(dependency_specification).map_err(|parse_error| {
                anyhow::anyhow!(
                    "Invalid dependency requirement '{dependency_specification}': {parse_error}"
                )
            })?;
        options = options.with_dependency_spec(requirement);
    }

    if args.deps.is_empty()
        && let Some(ref manifest) = parent_manifest
    {
        let resolved_deps = manifest
            .resolved_dependencies_for_profile(&args.profile)
            .unwrap_or_else(|_| manifest.resolved_dependencies());
        for resolved_dependency in resolved_deps {
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
            let requirement =
                DependencyRequirement::parse(&spec_string).map_err(|parse_error| {
                    anyhow::anyhow!(
                        "Invalid dependency requirement '{spec_string}' in manifest: {parse_error}"
                    )
                })?;
            options = options.with_dependency_spec(requirement);
        }
    }

    // 7. Mount dependency library binaries or archives passed via --deps-package, --deps-binaries, or `-l, --lib`
    let mut temp_archive_directories: Vec<tempfile::TempDir> = Vec::new();

    // 7a. Mount package dependencies (--deps-package)
    for package_path in &args.deps_package {
        let temp_dir = tempfile::Builder::new()
            .prefix("solix_compile_pkg_")
            .tempdir()
            .map_err(|error| {
                anyhow::anyhow!(
                    "Failed to create temporary directory for package extraction: {error}"
                )
            })?;
        let loaded_archive =
            solix_common::project::SlxpPackage::extract_and_load(package_path, temp_dir.path())
                .map_err(|load_error| {
                    anyhow::anyhow!(
                        "Failed to extract dependency package '{}': {load_error}",
                        package_path.display()
                    )
                })?;
        for binary_program in loaded_archive.binaries {
            options = options.with_dependency_binary(binary_program);
        }
        for secondary_program in loaded_archive.secondary_binaries {
            options = options.with_dependency_binary(secondary_program);
        }
        temp_archive_directories.push(temp_dir);
    }

    // 7b. Mount standalone binary dependencies (--deps-binaries)
    for binary_path in &args.deps_binaries {
        let lib_bytes = fs::read(binary_path).with_context(|| {
            format!(
                "Failed to read dependency binary '{}'",
                binary_path.display()
            )
        })?;
        let lib_binary = BinaryProgram::from_bytes(&lib_bytes).with_context(|| {
            format!(
                "Failed to decode binary container from '{}'",
                binary_path.display()
            )
        })?;
        options = options.with_dependency_binary(lib_binary);
    }

    // 7c. Mount legacy `-l, --lib` files
    for lib_path in &args.libs {
        if lib_path
            .extension()
            .is_some_and(|extension| extension == "slxp" || extension == "slxarc")
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
                solix_common::project::SlxpPackage::extract_and_load(lib_path, temp_dir.path())
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

    // 9. Output handling
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

    let binary_bytes = artifacts
        .program
        .try_to_bytes()
        .context("Failed to serialize compiled binary program")?;

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

/// Executes `solix compile package [path]`.
///
/// # Errors
/// Returns an error if project manifest is missing or compilation fails.
pub fn execute_compile_package(args: CompilePackageArgs) -> Result<()> {
    let project_root = solix_common::canonicalize_host_path(&args.path)
        .with_context(|| format!("Failed to resolve package path '{}'", args.path.display()))?;

    let manifest_path = find_parent_manifest(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot compile package at '{}': manifest.yaml not found",
            project_root.display()
        )
    })?;

    let manifest =
        ProjectManifest::from_file(&manifest_path).map_err(|error| anyhow::anyhow!("{error}"))?;

    let output_path = match args.output {
        Some(explicit_out) => explicit_out,
        None => project_root.join("binaries.slxb"),
    };

    execute_compile_package_to_output(
        &project_root,
        &manifest,
        &args.profile,
        &output_path,
        args.pkg_dir.as_deref(),
        args.check,
    )?;

    if !args.check {
        println!(
            "Compiled package '{}' to {}",
            manifest.configuration.name,
            output_path.display()
        );
    }

    Ok(())
}

/// Compiles a package's sources directly into a destination binary container (`binaries.slxb`).
///
/// # Errors
/// Returns an error if source files are missing or compilation fails.
pub fn execute_compile_package_to_output(
    project_root: &Path,
    manifest_data: &ProjectManifest,
    profile_name: &str,
    output_slxb: &Path,
    pkg_dir_override: Option<&Path>,
    check_only: bool,
) -> Result<()> {
    let resolved_profile = manifest_data
        .resolve_profile(profile_name)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let mut source_roots = Vec::new();
    for rel_src in &resolved_profile.sources {
        let candidate = if Path::new(rel_src).is_absolute() {
            PathBuf::from(rel_src)
        } else {
            project_root.join(rel_src)
        };
        if candidate.exists() {
            source_roots.push(candidate);
        }
    }
    if source_roots.is_empty() {
        bail!(
            "No Solix source directories exist for profile '{}' (configured: {})",
            resolved_profile.name,
            resolved_profile.sources.join(", ")
        );
    }

    let mut discovered_files = Vec::new();
    for source_root in &source_roots {
        if source_root.is_dir() {
            collect_slx_sources_recursively(source_root, &mut discovered_files)?;
        } else if source_root.is_file() {
            discovered_files.push(source_root.clone());
        }
    }
    if discovered_files.is_empty() {
        bail!(
            "No Solix source files (*.slx) found in profile '{}' sources",
            resolved_profile.name
        );
    }

    let mut dep_specs = Vec::new();
    for (dep_name, dep_spec) in &resolved_profile.dependencies {
        let formatted = if dep_spec.profile.is_empty() {
            format!("{dep_name}@{}", dep_spec.version)
        } else {
            format!("{dep_name}@{}#{}", dep_spec.version, dep_spec.profile)
        };
        dep_specs.push(formatted);
    }

    let compiler_profile = if resolved_profile.name == "test" {
        "test".to_string()
    } else if resolved_profile.opt_level >= 2 && !resolved_profile.debug_symbols {
        "release".to_string()
    } else {
        "debug".to_string()
    };

    execute_compile_source(CompileSourceArgs {
        sources: discovered_files,
        output: Some(output_slxb.to_path_buf()),
        deps_source: Vec::new(),
        deps_package: Vec::new(),
        deps_binaries: Vec::new(),
        deps: dep_specs,
        libs: Vec::new(),
        name: Some(manifest_data.configuration.name.clone()),
        version: manifest_data.configuration.version.clone(),
        description: manifest_data.configuration.description.clone(),
        profile: compiler_profile,
        emit_api: None,
        check: check_only,
        time: false,
        metrics: None,
        metrics_json: false,
        inline_threshold: resolved_profile.inline_threshold,
        entry: resolved_profile.entrypoint.clone(),
        link: resolved_profile.link.clone(),
        r#static: resolved_profile.is_static,
        pkg_dir: pkg_dir_override.map(Path::to_path_buf),
        install: false,
        force: false,
        no_verify: !resolved_profile.verify_checksum,
    })
}

#[cfg(test)]
mod tests {
    use super::find_parent_manifest;
    use std::fs;

    #[test]
    fn test_find_parent_manifest_stops_at_git_boundary() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let outer_manifest = temp_dir.path().join("manifest.yaml");
        fs::write(
            &outer_manifest,
            "configuration:\n  name: outer\n  version: \"0.1.0\"\n",
        )
        .expect("write outer manifest");

        let repo_root = temp_dir.path().join("repo");
        let nested_src = repo_root.join("examples").join("standalone");
        fs::create_dir_all(repo_root.join(".git")).expect("create .git");
        fs::create_dir_all(&nested_src).expect("create nested dir");

        let standalone_file = nested_src.join("main.slx");
        fs::write(&standalone_file, "int32 main() { return 0; }").expect("write source");

        assert!(
            find_parent_manifest(&standalone_file).is_none(),
            "Traversal must stop at .git boundary instead of walking past the repository root"
        );
    }
}
