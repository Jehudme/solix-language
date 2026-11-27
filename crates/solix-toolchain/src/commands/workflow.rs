//! First-class workflow subcommands: `solix check`, `solix test`, `solix init`, and `solix add`.

#![forbid(unsafe_code)]

use crate::cli::{AddArgs, BuildArgs, CheckArgs, CompilePackageArgs, InitArgs, TestArgs};
use crate::commands::build::execute_build;
use crate::commands::compile::execute_compile_package;
use crate::registry::parse_package_spec;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::ModuleSolver;
use solix_common::project::{DependencySpec, ProjectManifest, SlxarcArchive, find_manifest_path};
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};
use std::collections::BTreeMap;
use std::fs;
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

fn resolve_project_directory(path: &std::path::Path) -> Result<std::path::PathBuf> {
    match solix_common::canonicalize_host_path(path) {
        Ok(resolved) => Ok(resolved),
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound && std::env::current_dir().is_err() {
                bail!(
                    "Failed to resolve project directory '{}': the current working directory no longer exists or is inaccessible (it may have been deleted or recreated). Run 'cd' into an existing directory or pass an explicit project path.",
                    path.display()
                );
            }
            Err(err).with_context(|| {
                format!("Failed to resolve project directory '{}'", path.display())
            })
        }
    }
}

/// Executes `solix test [PATH] [--filter <pattern>]` by building the `test` profile
/// (including any native libraries) and running all discovered zero-argument `test_*` functions/methods.
///
/// # Errors
/// Returns an error if compilation fails or any executed test fails.
pub fn execute_test(args: TestArgs) -> Result<i32> {
    let project_root = resolve_project_directory(&args.path)?;
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

    execute_build(BuildArgs {
        path: project_root.clone(),
        platform: None,
        profile: Some(args.profile.clone()),
        debug: false,
        release: false,
        test: false,
        pkg_dir: args.pkg_dir.clone(),
        install: false,
        force: true,
        check_env: false,
        target: None,
    })?;

    let package_name = &manifest.configuration.name;
    let build_dir = project_root.join("build").join(&args.profile);
    let slxp_path = build_dir.join(format!("{package_name}.slxp"));
    let slxarc_path = build_dir.join(format!("{package_name}.slxarc"));
    let archive_path = if slxp_path.exists() {
        slxp_path
    } else if slxarc_path.exists() {
        slxarc_path
    } else {
        bail!(
            "Built test archive not found under '{}'",
            build_dir.display()
        );
    };

    let extract_temp = tempfile::Builder::new().prefix("solix_test_").tempdir()?;
    let loaded_archive = SlxarcArchive::extract_and_load(&archive_path, extract_temp.path())?;
    let primary_program = loaded_archive
        .binaries
        .first()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Test archive contains no primary bytecode binary"))?;

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_directory));

    // Discover zero-arg test_* functions or fall back to configured test entrypoint
    let mut test_candidates: Vec<(u32, String)> = Vec::new();
    for (func_idx, func_info) in primary_program.functions.iter().enumerate() {
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
        && let Some(entry_idx) = primary_program.functions.iter().position(|func| {
            func.symbol == entry_sym || func.symbol.ends_with(&format!("::{entry_sym}"))
        })
    {
        test_candidates.push((
            entry_idx as u32,
            primary_program.functions[entry_idx].symbol.clone(),
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
        let mut runtime_options = RuntimeOptions::new()
            .with_heap_size(32 * 1024 * 1024)
            .with_solver(solver.clone())
            .with_native_function("println", builtin_println)
            .with_native_function("print", builtin_print)
            .with_native_function("Console::println", builtin_println)
            .with_native_function("Console::print", builtin_print)
            .with_verify_checksums(!args.no_verify);

        for native_lib in &loaded_archive.native_libraries {
            runtime_options = runtime_options.with_native_library(native_lib.clone());
        }
        for binary_prog in &loaded_archive.binaries {
            runtime_options = runtime_options.with_binary(binary_prog.clone());
        }
        for secondary_prog in &loaded_archive.secondary_binaries {
            runtime_options = runtime_options.with_binary(secondary_prog.clone());
        }

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
    let project_root = resolve_project_directory(&args.path)?;
    let manifest_path = find_manifest_path(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "No manifest.yaml (or manifest.toml) found in '{}'",
            project_root.display()
        )
    })?;

    let (package_name, requested_profile, requested_ver) = parse_package_spec(&args.package);
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);
    let solver = ClientModuleSolver::with_store_dir(store_directory);

    let resolved_version = if let Some(explicit_ver) = requested_ver {
        solix_common::solver::PackageVersion::parse_constraint(&explicit_ver).map_err(|err| {
            anyhow::anyhow!(
                "Invalid version constraint '{explicit_ver}' for '{package_name}': {err}"
            )
        })?;
        explicit_ver
    } else if let Ok(solved) = solver.resolve_latest(&package_name) {
        solved.version.to_string()
    } else {
        "*".to_string()
    };

    let mut manifest = ProjectManifest::from_file(&manifest_path)
        .map_err(|err| anyhow::anyhow!("Failed to parse manifest: {err}"))?;

    let dep_entry = DependencySpec {
        version: resolved_version.clone(),
        profile: requested_profile.unwrap_or_else(|| args.dep_profile.clone()),
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
