//! First-class workflow subcommands: `solix check`, `solix test`, `solix init`, and `solix add`.

#![forbid(unsafe_code)]

use crate::cli::{AddArgs, CheckArgs, CompilePackageArgs, InitArgs, TestArgs};
use crate::commands::compile::{collect_slx_sources_recursively, execute_compile_package};
use crate::diagnostics::render_diagnostics;
use crate::registry::parse_package_spec;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::ModuleSolver;
use solix_common::project::{DependencySpec, ProjectManifest, find_manifest_path};
use solix_common::solver::DependencyRequirement;
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Executes `solix check [PATH]` using `CompilationPipeline::check()` without emitting bytecode.
///
/// # Errors
/// Returns an error if the manifest is missing or compilation diagnostics fail.
pub fn execute_check(args: CheckArgs) -> Result<i32> {
    let compile_args = CompilePackageArgs {
        path: args.path,
        profile: args.profile,
        output: None,
        pkg_dir: args.pkg_dir,
        check: true,
        time: false,
        metrics: None,
        no_verify: args.no_verify,
    };
    execute_compile_package(compile_args)?;
    Ok(0)
}

/// Executes `solix test [PATH] [--filter <pattern>]` by compiling with the `test` profile
/// and running all discovered zero-argument `test_*` functions/methods.
///
/// # Errors
/// Returns an error if compilation fails or any executed test fails.
pub fn execute_test(args: TestArgs) -> Result<i32> {
    let project_root = args.path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve project directory '{}'",
            args.path.display()
        )
    })?;
    let manifest_path = find_manifest_path(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "No manifest.yaml (or manifest.toml) found in '{}'",
            project_root.display()
        )
    })?;
    let manifest = ProjectManifest::from_file(&manifest_path)
        .map_err(|err| anyhow::anyhow!("Failed to parse manifest: {err}"))?;
    let resolved_profile = manifest
        .resolve_profile(&args.profile)
        .map_err(|err| anyhow::anyhow!("Failed to resolve profile '{}': {err}", args.profile))?;

    let mut source_files: Vec<PathBuf> = Vec::new();
    for source_root in &resolved_profile.sources {
        let candidate_path = project_root.join(source_root);
        if candidate_path.is_dir() {
            collect_slx_sources_recursively(&candidate_path, &mut source_files)?;
        } else if candidate_path.is_file() {
            source_files.push(candidate_path);
        }
    }
    if source_files.is_empty() {
        bail!("No .slx source files found for profile '{}'", args.profile);
    }

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_directory));

    let mut options = CompilationOptions::new()
        .with_profile(BuildProfile::Test)
        .with_package_metadata(
            &manifest.configuration.name,
            &manifest.configuration.version,
            manifest.configuration.description.as_deref().unwrap_or(""),
        )
        .with_solver(solver.clone())
        .with_verify_checksums(!args.no_verify);

    if let Some(entry_sym) = resolved_profile.entrypoint.clone() {
        options = options.with_entry_point(Some(entry_sym));
    }
    for (dep_name, dep_spec) in &resolved_profile.dependencies {
        if let Ok(parsed_ver) = solix_common::solver::PackageVersion::parse(&dep_spec.version) {
            let mut req = DependencyRequirement::new(dep_name.clone(), parsed_ver);
            if !dep_spec.profile.is_empty() {
                req = req.with_profile(dep_spec.profile.clone());
            }
            options = options.with_dependency_spec(req);
        }
    }
    for source_path in &source_files {
        let content = fs::read_to_string(source_path)?;
        options = options.with_source(source_path.display().to_string(), content);
    }

    let mut context = CompilationContext::new(options);
    let mut pipeline = CompilationPipeline::standard();
    if let Err(diagnostics) = pipeline.execute(&mut context) {
        render_diagnostics(&context.source_names, &context.sources, &diagnostics);
        bail!(
            "Test compilation aborted due to {} error(s).",
            diagnostics.len()
        );
    }

    let program = context
        .emission_artifacts
        .take()
        .ok_or_else(|| anyhow::anyhow!("Test compilation produced no bytecode artifacts"))?
        .program;

    // Discover zero-arg test_* functions or fall back to configured test entrypoint
    let mut test_candidates: Vec<(u32, String)> = Vec::new();
    for (func_idx, func_info) in program.functions.iter().enumerate() {
        let short_name = func_info
            .symbol
            .rsplit("::")
            .next()
            .unwrap_or(&func_info.symbol);
        let is_test_fn = short_name.starts_with("test_") && func_info.param_count == 0;
        let matches_filter = args
            .filter
            .as_ref()
            .is_none_or(|pattern| func_info.symbol.contains(pattern.as_str()));
        if is_test_fn && matches_filter {
            test_candidates.push((func_idx as u32, func_info.symbol.clone()));
        }
    }

    if test_candidates.is_empty()
        && let Some(entry_sym) = resolved_profile.entrypoint
        && let Some(entry_idx) = program.functions.iter().position(|func| {
            func.symbol == entry_sym || func.symbol.ends_with(&format!("::{entry_sym}"))
        })
    {
        test_candidates.push((
            entry_idx as u32,
            program.functions[entry_idx].symbol.clone(),
        ));
    }

    if test_candidates.is_empty() {
        println!("No matching test functions found.");
        return Ok(0);
    }

    println!("Running {} test(s)...", test_candidates.len());
    let mut passed_count = 0usize;
    let mut failed_count = 0usize;

    for (func_id, func_name) in &test_candidates {
        let runtime_options = RuntimeOptions::new()
            .with_heap_size(32 * 1024 * 1024)
            .with_solver(solver.clone())
            .with_native_function("println", builtin_println)
            .with_native_function("print", builtin_print)
            .with_native_function("Console::println", builtin_println)
            .with_native_function("Console::print", builtin_print)
            .with_verify_checksums(!args.no_verify)
            .with_binary(program.clone());

        let runtime_ctx = RuntimeContext::new(runtime_options)
            .map_err(|err| anyhow::anyhow!("Runtime initialization failed: {err}"))?;
        let mut virtual_machine = VirtualMachine::new(runtime_ctx);
        let start_time = Instant::now();
        let exec_result = virtual_machine
            .push_frame(*func_id, 0, &[])
            .and_then(|()| virtual_machine.run());
        let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        match exec_result {
            Ok(_) => {
                passed_count += 1;
                println!("  [PASS] {func_name} ({elapsed_ms:.2}ms)");
            }
            Err(runtime_err) => {
                failed_count += 1;
                eprintln!("  [FAIL] {func_name} ({elapsed_ms:.2}ms): {runtime_err}");
            }
        }
    }

    println!(
        "\nTest result: {} passed, {} failed.",
        passed_count, failed_count
    );
    Ok(i32::from(failed_count > 0))
}

/// Executes `solix init [PATH]` to initialize a `manifest.yaml` in an existing directory
/// without overwriting existing source files.
///
/// # Errors
/// Returns an error if `manifest.yaml` already exists (without `--force`) or I/O fails.
pub fn execute_init(args: InitArgs) -> Result<()> {
    let target_dir = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)?;
    }

    if !args.force
        && let Some(existing) = find_manifest_path(&target_dir)
    {
        bail!(
            "Manifest already exists at '{}'. Use --force to overwrite.",
            existing.display()
        );
    }

    let project_name = args.name.unwrap_or_else(|| {
        target_dir
            .file_name()
            .and_then(|val| val.to_str())
            .unwrap_or("solix_project")
            .to_string()
    });
    let solix_version = crate::assets::default_bundled_solix_version();

    let manifest_text = format!(
        "vars:\n  sources: &sources\n    - src\n  dependencies: &dependencies\n    solix:\n      version: \"{solix_version}\"\n      profile: release\n\nconfiguration:\n  name: {project_name}\n  version: \"0.1.0\"\n  description: \"A Solix project\"\n  license: \"Apache-2.0\"\n  authors: []\n\nprofiles:\n  release:\n    entrypoint: Main::main\n    optimization_level: 3\n    inline_threshold: 16\n    debug_symbols: false\n    static: true\n    sources: *sources\n    platforms:\n      - all\n    dependencies: *dependencies\n\n  debug:\n    entrypoint: Main::main\n    optimization_level: 0\n    inline_threshold: 8\n    debug_symbols: true\n    static: false\n    sources: *sources\n    platforms:\n      - host\n    dependencies: *dependencies\n"
    );

    let manifest_path = target_dir.join("manifest.yaml");
    fs::write(&manifest_path, manifest_text)?;
    println!(
        "Initialized Solix manifest at '{}'.",
        manifest_path.display()
    );
    Ok(())
}

/// Executes `solix add <PACKAGE>[@VERSION]` to add a dependency to `manifest.yaml`.
///
/// # Errors
/// Returns an error if `manifest.yaml` cannot be found/parsed or written.
pub fn execute_add(args: AddArgs) -> Result<()> {
    let project_root = args.path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve project directory '{}'",
            args.path.display()
        )
    })?;
    let manifest_path = find_manifest_path(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "No manifest.yaml (or manifest.toml) found in '{}'",
            project_root.display()
        )
    })?;

    let (package_name, requested_ver, _requested_profile) = parse_package_spec(&args.package);
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);
    let solver = ClientModuleSolver::with_store_dir(store_directory);

    let resolved_version = if let Some(explicit_ver) = requested_ver {
        explicit_ver
    } else if let Ok(solved) = solver.resolve_latest(&package_name) {
        format!(">={}", solved.version)
    } else {
        "*".to_string()
    };

    let mut manifest = ProjectManifest::from_file(&manifest_path)
        .map_err(|err| anyhow::anyhow!("Failed to parse manifest: {err}"))?;

    let dep_entry = DependencySpec {
        version: resolved_version.clone(),
        profile: args.dep_profile.clone(),
    };

    if manifest.profiles.is_empty() {
        bail!(
            "Manifest '{}' defines no profiles to attach dependency '{package_name}'.",
            manifest_path.display()
        );
    }
    let mut updated_profiles = BTreeMap::new();
    for (profile_name, mut profile_cfg) in manifest.profiles {
        profile_cfg
            .dependencies
            .insert(package_name.clone(), dep_entry.clone());
        updated_profiles.insert(profile_name, profile_cfg);
    }
    manifest.profiles = updated_profiles;

    let serialized = manifest
        .to_yaml_string()
        .map_err(|err| anyhow::anyhow!("Failed to serialize manifest: {err}"))?;
    fs::write(&manifest_path, serialized)?;
    println!(
        "Added dependency '{package_name}' ({resolved_version}) to '{}'.",
        manifest_path.display()
    );
    Ok(())
}
