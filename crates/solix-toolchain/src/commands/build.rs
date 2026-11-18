//! The `solix build` command driver.

#![forbid(unsafe_code)]

use crate::cli::BuildArgs;
use crate::diagnostics::render_diagnostics;
use crate::registry::PackageRegistry;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::hash::sha256_file;
use solix_common::project::{
    ProjectManifest, ResolvedProfile, SlxarcArchive, expand_platform_list, host_platform_id,
    host_target_triple, triple_to_platform_id, validate_archive_relative_path,
};
use solix_common::solver::{DependencyRequirement, PackageVersion};
use solix_compilation::CompilationContext::{
    BuildProfile, CompilationContext, CompilationOptions,
};
use solix_compilation::CompilationPipeline;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use zip::ZipArchive;

/// Resolves the canonical path to the currently running `solix` CLI binary.
fn resolve_solix_binary() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("solix"));
    let binary_name = if cfg!(windows) { "solix.exe" } else { "solix" };
    if let Ok(custom_bin) = std::env::var("SOLIX_BIN")
        && !custom_bin.trim().is_empty()
    {
        return PathBuf::from(custom_bin);
    }
    if let Ok(cargo_bin_env) = std::env::var("CARGO_BIN_EXE_solix")
        && !cargo_bin_env.trim().is_empty()
    {
        return PathBuf::from(cargo_bin_env);
    }
    if current_exe
        .file_name()
        .is_some_and(|name| name == "solix" || name == "solix.exe")
    {
        return current_exe;
    }
    if let Some(parent_dir) = current_exe.parent() {
        let sibling_candidate = parent_dir.join(binary_name);
        if sibling_candidate.exists() {
            return sibling_candidate;
        }
        if let Some(grandparent_dir) = parent_dir.parent() {
            let ancestor_candidate = grandparent_dir.join(binary_name);
            if ancestor_candidate.exists() {
                return ancestor_candidate;
            }
        }
    }
    current_exe
}

/// Resolves a `--target` CLI override or `SOLIX_TARGET_TRIPLE` environment value.
fn resolve_effective_target_override(target_arg: Option<&str>) -> Option<String> {
    let raw = target_arg
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("SOLIX_TARGET_TRIPLE")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })?;
    if raw.eq_ignore_ascii_case("host") {
        Some(host_target_triple().to_string())
    } else {
        Some(raw)
    }
}

/// Discovers Rust (`Cargo.toml`) and C/CMake (`CMakeLists.txt` or `.c`) directories under `natives/`.
fn discover_native_dirs(project_root: &Path) -> (Vec<String>, Vec<String>) {
    let natives_dir = project_root.join("natives");
    let mut rust_dirs = Vec::new();
    let mut c_dirs = Vec::new();
    if !natives_dir.is_dir() {
        return (rust_dirs, c_dirs);
    }
    let Ok(entries) = fs::read_dir(&natives_dir) else {
        return (rust_dirs, c_dirs);
    };
    let mut subdirs: Vec<PathBuf> = entries
        .filter_map(|entry_res| entry_res.ok().map(|entry| entry.path()))
        .filter(|candidate| {
            candidate.is_dir()
                && !candidate
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with('.'))
        })
        .collect();
    subdirs.sort();

    for subdir in subdirs {
        let rel_path = format!(
            "natives/{}",
            subdir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
        );
        if subdir.join("Cargo.toml").is_file() {
            rust_dirs.push(rel_path);
        } else if subdir.join("CMakeLists.txt").is_file() {
            c_dirs.push(rel_path);
        } else if let Ok(inner_entries) = fs::read_dir(&subdir) {
            let has_c_source = inner_entries.filter_map(|entry| entry.ok()).any(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "c")
            });
            if has_c_source {
                c_dirs.push(rel_path);
            }
        }
    }
    (rust_dirs, c_dirs)
}

/// Checks whether a directory tree contains any native shared library binaries.
fn dir_contains_shared_libs(dir_path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir_path) else {
        return false;
    };
    for entry in entries.filter_map(|res| res.ok()) {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if dir_contains_shared_libs(&entry_path) {
                return true;
            }
        } else if entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "so" | "dll" | "dylib"))
        {
            return true;
        }
    }
    false
}

/// Resolves the installed archive path for a declared dependency from the local package store.
fn find_dependency_archive(
    registry: &PackageRegistry,
    package_name: &str,
    version_req: &str,
    preferred_profile: Option<&str>,
) -> Option<PathBuf> {
    let req_spec = if version_req.is_empty() || version_req == "*" {
        package_name.to_string()
    } else {
        format!("{package_name}@{version_req}")
    };
    let parsed_req = DependencyRequirement::parse(&req_spec).ok();

    let mut matches: Vec<(&crate::registry::PackageRecord, PackageVersion)> = registry
        .packages
        .iter()
        .filter(|record| record.name == package_name && record.artifact_path.is_file())
        .filter_map(|record| {
            let parsed_ver = PackageVersion::parse(&record.version).ok()?;
            if let Some(ref req) = parsed_req
                && version_req != "*"
                && !version_req.is_empty()
                && !parsed_ver.is_compatible_with(&req.version)
            {
                return None;
            }
            Some((record, parsed_ver))
        })
        .collect();

    matches.sort_by(|left, right| {
        let left_pref = preferred_profile
            .map(|prof| left.0.build_profile == prof)
            .unwrap_or_else(|| left.0.build_profile == "release");
        let right_pref = preferred_profile
            .map(|prof| right.0.build_profile == prof)
            .unwrap_or_else(|| right.0.build_profile == "release");
        left_pref.cmp(&right_pref).then(left.1.cmp(&right.1))
    });

    matches
        .last()
        .map(|(record, _)| record.artifact_path.clone())
}

/// Checks whether the project requires platform-specific native code for `profile`.
fn project_has_native_code(
    project_root: &Path,
    resolved_profile: &ResolvedProfile,
    package_store: &Path,
) -> bool {
    let (rust_dirs, c_dirs) = discover_native_dirs(project_root);
    if !rust_dirs.is_empty() || !c_dirs.is_empty() {
        return true;
    }
    let lib_dir = project_root.join("lib");
    if lib_dir.is_dir() && dir_contains_shared_libs(&lib_dir) {
        return true;
    }
    if resolved_profile.is_static
        && !resolved_profile.dependencies.is_empty()
        && package_store.is_dir()
        && let Ok(registry) = PackageRegistry::load_or_sync(package_store)
    {
        for dep in resolved_profile.resolved_dependencies() {
            let preferred_profile = if dep.build_profile.is_empty() {
                None
            } else {
                Some(dep.build_profile.as_str())
            };
            if let Some(archive_path) = find_dependency_archive(
                &registry,
                &dep.package_name,
                &dep.version,
                preferred_profile,
            ) && let Ok(archive_file) = fs::File::open(archive_path)
                && let Ok(mut zip_reader) = ZipArchive::new(archive_file)
            {
                for idx in 0..zip_reader.len() {
                    if let Ok(entry) = zip_reader.by_index(idx)
                        && let Ok(name) = entry.name()
                        && name.starts_with("lib/")
                        && (name.ends_with(".so")
                            || name.ends_with(".dll")
                            || name.ends_with(".dylib"))
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Computes canonical required platforms and target triples for `profile`.
fn compute_required_platforms_and_triples(
    project_root: &Path,
    manifest: &ProjectManifest,
    resolved_profile: &ResolvedProfile,
    effective_profile: &str,
    target_override: Option<&str>,
    package_store: &Path,
) -> (Vec<String>, Vec<String>) {
    if let Some(forced_triple) = target_override {
        let plat = triple_to_platform_id(forced_triple)
            .map(str::to_string)
            .unwrap_or_else(|| forced_triple.to_ascii_lowercase());
        return (vec![plat], vec![forced_triple.to_string()]);
    }

    let raw_platforms: Vec<String> = manifest
        .profiles
        .get(effective_profile)
        .and_then(|profile_cfg| profile_cfg.platforms.clone())
        .unwrap_or_else(|| vec!["all".to_string()]);
    let explicit_platforms = manifest
        .profiles
        .get(effective_profile)
        .and_then(|profile_cfg| profile_cfg.platforms.as_ref())
        .is_some();

    let required_platforms = if !explicit_platforms
        && effective_profile == "release"
        && !project_has_native_code(project_root, resolved_profile, package_store)
    {
        vec!["universal".to_string()]
    } else {
        expand_platform_list(&raw_platforms)
    };

    let has_cargo_xwin = Command::new("cargo-xwin")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success());
    let mut target_triples = Vec::new();
    for platform_id in &required_platforms {
        match platform_id.as_str() {
            "universal" => {}
            "linux-x86_64" => target_triples.push("x86_64-unknown-linux-gnu".to_string()),
            "linux-aarch64" => target_triples.push("aarch64-unknown-linux-gnu".to_string()),
            "windows-x86_64" => {
                if cfg!(windows) || has_cargo_xwin {
                    target_triples.push("x86_64-pc-windows-msvc".to_string());
                } else {
                    target_triples.push("x86_64-pc-windows-gnu".to_string());
                }
            }
            "macos-x86_64" => target_triples.push("x86_64-apple-darwin".to_string()),
            "macos-aarch64" => target_triples.push("aarch64-apple-darwin".to_string()),
            _ => {}
        }
    }

    (required_platforms, target_triples)
}

/// Emits resolved manifest, profile, and platform targets as JSON for `BuildToolbox.slx`.
fn execute_inspect_manifest_json(
    project_root: &Path,
    manifest_path: &Path,
    manifest: &ProjectManifest,
    resolved_profile: &ResolvedProfile,
    effective_profile: &str,
    args: &BuildArgs,
) -> Result<()> {
    let package_store = resolve_package_store_dir(args.pkg_dir.as_deref());
    let target_override = resolve_effective_target_override(args.target.as_deref());
    let (required_platforms, target_triples) = compute_required_platforms_and_triples(
        project_root,
        manifest,
        resolved_profile,
        effective_profile,
        target_override.as_deref(),
        &package_store,
    );
    let (rust_native_dirs, c_native_dirs) = discover_native_dirs(project_root);
    let has_native_sources = !rust_native_dirs.is_empty() || !c_native_dirs.is_empty();

    let stage_dir = project_root
        .join("build")
        .join(effective_profile)
        .join("stage");
    let output_slxarc = project_root
        .join("build")
        .join(effective_profile)
        .join(format!("{}.slxarc", manifest.configuration.name));

    let dependency_specs: Vec<String> = resolved_profile
        .resolved_dependencies()
        .into_iter()
        .map(|dep| {
            if dep.build_profile.is_empty() {
                format!("{}@{}", dep.package_name, dep.version)
            } else {
                format!(
                    "{}@{}#{}",
                    dep.package_name, dep.version, dep.build_profile
                )
            }
        })
        .collect();

    let payload = serde_json::json!({
        "project_root": project_root.to_string_lossy(),
        "manifest_path": manifest_path.to_string_lossy(),
        "profile": effective_profile,
        "package_name": manifest.configuration.name,
        "package_version": manifest.configuration.version,
        "package_description": manifest.configuration.description.as_deref().unwrap_or(""),
        "entrypoint": resolved_profile.entrypoint.as_deref().unwrap_or("Main::main"),
        "link_mode": if resolved_profile.is_static { "static" } else { "dynamic" },
        "optimization_level": resolved_profile.opt_level,
        "inline_threshold": resolved_profile.inline_threshold,
        "debug_symbols": resolved_profile.debug_symbols,
        "verify_checksum": resolved_profile.verify_checksum,
        "emit_disassembly": resolved_profile.emit_disassembly,
        "host_platform": host_platform_id(),
        "host_triple": host_target_triple(),
        "target_triple": target_override.unwrap_or_default(),
        "pkg_dir": package_store.to_string_lossy(),
        "has_native_sources": has_native_sources,
        "rust_native_dirs": rust_native_dirs,
        "c_native_dirs": c_native_dirs,
        "sources": resolved_profile.sources,
        "dependencies": dependency_specs,
        "required_platforms": required_platforms,
        "target_triples": target_triples,
        "stage_dir": stage_dir.to_string_lossy(),
        "stage_bin_dir": stage_dir.join("bin").to_string_lossy(),
        "stage_lib_dir": stage_dir.join("lib").to_string_lossy(),
        "stage_resources_dir": stage_dir.join("resources").to_string_lossy(),
        "output_slxarc": output_slxarc.to_string_lossy(),
    });
    println!(
        "{}",
        serde_json::to_string(&payload).context("failed to serialize manifest context JSON")?
    );
    Ok(())
}

/// Stages native shared libraries from statically linked dependency `.slxarc` packages.
fn stage_dependency_natives(
    project_root: &Path,
    resolved_profile: &ResolvedProfile,
    effective_profile: &str,
    package_store: &Path,
) -> Result<()> {
    if !resolved_profile.is_static || resolved_profile.dependencies.is_empty() {
        return Ok(());
    }
    if !package_store.is_dir() {
        return Ok(());
    }

    let stage_lib_dir = project_root
        .join("build")
        .join(effective_profile)
        .join("stage")
        .join("lib");
    fs::create_dir_all(&stage_lib_dir)?;

    let registry = PackageRegistry::load_or_sync(package_store)?;
    let mut visited = BTreeSet::new();
    let mut pending = resolved_profile.resolved_dependencies();

    while let Some(dep) = pending.pop() {
        let key = format!("{}@{}", dep.package_name, dep.version);
        if !visited.insert(key) {
            continue;
        }
        let preferred_profile = if dep.build_profile.is_empty() {
            None
        } else {
            Some(dep.build_profile.as_str())
        };
        let Some(archive_path) =
            find_dependency_archive(&registry, &dep.package_name, &dep.version, preferred_profile)
        else {
            continue;
        };

        let archive_file = fs::File::open(&archive_path)?;
        let mut zip_reader = ZipArchive::new(archive_file)?;
        let mut native_entries = Vec::new();
        let mut qualified_basenames = BTreeSet::new();

        for idx in 0..zip_reader.len() {
            let entry = zip_reader.by_index(idx)?;
            let name = entry
                .name()
                .map_err(|zip_err| anyhow::anyhow!("Invalid zip entry name: {zip_err}"))?
                .to_string();
            if name.starts_with("lib/")
                && (name.ends_with(".so") || name.ends_with(".dll") || name.ends_with(".dylib"))
            {
                let rel_sub = &name["lib/".len()..];
                if let Some((_, file_name)) = rel_sub.rsplit_once('/') {
                    qualified_basenames.insert(file_name.to_string());
                }
                native_entries.push(name);
            }
        }

        for entry_name in native_entries {
            let rel_sub = &entry_name["lib/".len()..];
            let safe_rel = validate_archive_relative_path(rel_sub)
                .map_err(|err| anyhow::anyhow!("Unsafe dependency archive path: {err}"))?;
            let is_qualified = rel_sub.contains('/');
            let base_name = safe_rel
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !is_qualified && qualified_basenames.contains(base_name) {
                continue;
            }
            let dest_path = stage_lib_dir.join(&safe_rel);
            if let Some(parent_dir) = dest_path.parent() {
                fs::create_dir_all(parent_dir)?;
            }
            let mut zip_entry = zip_reader.by_name(&entry_name)?;
            let mut bytes = Vec::new();
            zip_entry.read_to_end(&mut bytes)?;
            fs::write(&dest_path, &bytes)?;
            println!(
                "  [native] Staged dependency library '{}': {}",
                dep.package_name,
                dest_path.display()
            );
        }
    }

    Ok(())
}

/// Recursively collects native shared library digests under `stage_dir/lib/`.
fn collect_staged_native_digests(
    stage_dir: &Path,
    current_dir: &Path,
    checksums: &mut BTreeMap<String, String>,
) -> Result<()> {
    if !current_dir.is_dir() {
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(current_dir)?
        .filter_map(|res| res.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    for entry_path in entries {
        if entry_path.is_dir() {
            collect_staged_native_digests(stage_dir, &entry_path, checksums)?;
        } else if entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "so" | "dll" | "dylib"))
        {
            let rel_path = entry_path
                .strip_prefix(stage_dir)
                .with_context(|| format!("failed to relativize '{}'", entry_path.display()))?;
            let key = rel_path.to_string_lossy().replace('\\', "/");
            let digest = sha256_file(&entry_path)
                .with_context(|| format!("failed to hash native '{}'", entry_path.display()))?;
            checksums.insert(key, digest);
        }
    }
    Ok(())
}

/// Validates platform compatibility, injects manifest checksums, and packs `.slxarc`.
fn execute_pack_archive(
    project_root: &Path,
    manifest: &ProjectManifest,
    resolved_profile: &ResolvedProfile,
    effective_profile: &str,
    args: &BuildArgs,
) -> Result<()> {
    let package_store = resolve_package_store_dir(args.pkg_dir.as_deref());
    let target_override = resolve_effective_target_override(args.target.as_deref());
    let (required_platforms, _triples) = compute_required_platforms_and_triples(
        project_root,
        manifest,
        resolved_profile,
        effective_profile,
        target_override.as_deref(),
        &package_store,
    );

    let stage_dir = project_root
        .join("build")
        .join(effective_profile)
        .join("stage");
    let stage_lib_dir = stage_dir.join("lib");
    let mut native_checksums = BTreeMap::new();
    collect_staged_native_digests(&stage_dir, &stage_lib_dir, &mut native_checksums)?;

    let mut packaged_targets = BTreeSet::new();
    let mut covered_platforms = BTreeSet::new();
    for rel_key in native_checksums.keys() {
        let parts: Vec<&str> = rel_key.split('/').collect();
        if parts.len() >= 3 && parts[0] == "lib" {
            let triple_name = parts[1];
            packaged_targets.insert(triple_name.to_string());
            if let Some(plat_id) = triple_to_platform_id(triple_name) {
                covered_platforms.insert(plat_id.to_string());
            }
        } else if parts.len() == 2 && parts[0] == "lib" {
            covered_platforms.insert(host_platform_id().to_string());
        }
    }

    if !native_checksums.is_empty() && target_override.is_none() {
        let missing_platforms: Vec<String> = required_platforms
            .iter()
            .filter(|plat| plat.as_str() != "universal" && !covered_platforms.contains(*plat))
            .cloned()
            .collect();
        if !missing_platforms.is_empty() {
            bail!(
                "Cross-Platform Release Gate failed for '{}' [{}]: missing native binaries for {}",
                manifest.configuration.name,
                effective_profile,
                missing_platforms.join(", ")
            );
        }
    }

    let explicit_platforms = manifest
        .profiles
        .get(effective_profile)
        .and_then(|profile_cfg| profile_cfg.platforms.as_ref())
        .is_some();
    let supported_platforms: Vec<String> = if let Some(forced) = target_override {
        vec![
            triple_to_platform_id(&forced)
                .map(str::to_string)
                .unwrap_or_else(|| forced.to_ascii_lowercase()),
        ]
    } else if !native_checksums.is_empty() {
        covered_platforms.into_iter().collect()
    } else if explicit_platforms {
        required_platforms
    } else {
        vec!["universal".to_string()]
    };

    let staged_manifest_path = stage_dir.join("manifest.toml");
    let source_manifest = if staged_manifest_path.is_file() {
        staged_manifest_path.clone()
    } else {
        project_root.join("manifest.toml")
    };
    if source_manifest.is_file() {
        let raw_text = fs::read_to_string(&source_manifest)?;
        let body: String = raw_text
            .lines()
            .filter(|line| !line.trim_start().starts_with("supported_platforms"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        let plat_list = supported_platforms
            .iter()
            .map(|plat| format!("\"{plat}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let mut updated = format!("supported_platforms = [{plat_list}]\n\n{body}\n");
        if !native_checksums.is_empty() {
            updated.push_str("\n[native_library_checksums]\n");
            for (rel_key, digest) in &native_checksums {
                updated.push_str(&format!("\"{rel_key}\" = \"{digest}\"\n"));
            }
        }
        fs::write(&staged_manifest_path, updated)?;
    }

    let output_slxarc = project_root
        .join("build")
        .join(effective_profile)
        .join(format!("{}.slxarc", manifest.configuration.name));
    if output_slxarc.exists() {
        fs::remove_file(&output_slxarc)?;
    }
    SlxarcArchive::pack_directory(&stage_dir, &output_slxarc)
        .map_err(|err| anyhow::anyhow!("failed to package .slxarc archive: {err}"))?;
    println!("  [package] Emitted archive: {}", output_slxarc.display());
    Ok(())
}

/// Applies shared `SOLIX_*` build environment variables onto a child process `Command`.
fn apply_build_env_vars(
    command: &mut Command,
    project_root: &Path,
    solix_bin: &Path,
    effective_profile: &str,
    resolved_profile: &ResolvedProfile,
    args: &BuildArgs,
) {
    let effective_link_mode = if resolved_profile.is_static {
        "static"
    } else {
        "dynamic"
    };
    command
        .current_dir(project_root)
        .env("SOLIX_PROJECT_ROOT", project_root)
        .env("SOLIX_BIN", solix_bin)
        .env("SOLIX_PROFILE", effective_profile)
        .env("SOLIX_LINK_MODE", effective_link_mode)
        .env(
            "SOLIX_INLINE_THRESHOLD",
            resolved_profile.inline_threshold.to_string(),
        )
        .env(
            "SOLIX_VERIFY_CHECKSUM",
            if resolved_profile.verify_checksum {
                "1"
            } else {
                "0"
            },
        );

    if args.check_env {
        command.env("SOLIX_VERIFY_ONLY", "1");
    }
    if let Some(resolved_triple) = resolve_effective_target_override(args.target.as_deref()) {
        command.env("SOLIX_TARGET_TRIPLE", resolved_triple);
    }
    if let Some(ref package_store_directory) = args.pkg_dir {
        command.env("SOLIX_PKG_DIR", package_store_directory);
    }
}

/// Compiles (with mtime caching) and executes a Solix-native `scripts/build.slx` script.
fn execute_build_slx(
    project_root: &Path,
    script_path: &Path,
    solix_bin: &Path,
    effective_profile: &str,
    resolved_profile: &ResolvedProfile,
    args: &BuildArgs,
) -> Result<()> {
    let cache_dir = project_root.join("build").join(".cache");
    fs::create_dir_all(&cache_dir)?;
    let cached_slxb = cache_dir.join("build_script.slxb");

    let needs_recompile = if cached_slxb.is_file() {
        let script_mtime = fs::metadata(script_path).and_then(|meta| meta.modified()).ok();
        let cache_mtime = fs::metadata(&cached_slxb).and_then(|meta| meta.modified()).ok();
        match (script_mtime, cache_mtime) {
            (Some(src_time), Some(bin_time)) => src_time > bin_time,
            _ => true,
        }
    } else {
        true
    };

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let _ = crate::assets::ensure_bundled_packages_installed(&store_directory);

    if needs_recompile {
        let source_code = fs::read_to_string(script_path).with_context(|| {
            format!(
                "failed to read Solix build script '{}'",
                script_path.display()
            )
        })?;
        let mut comp_opts = CompilationOptions::new()
            .with_profile(BuildProfile::Release)
            .with_source(script_path.display().to_string(), source_code)
            .with_solver(Arc::new(ClientModuleSolver::with_store_dir(
                store_directory.clone(),
            )));
        let req = DependencyRequirement::parse("solix@2.0.0#release")
            .map_err(|err| anyhow::anyhow!("invalid default solixlib requirement: {err}"))?;
        comp_opts = comp_opts.with_dependency_spec(req);
        let mut pipeline = CompilationPipeline::standard();
        let mut comp_ctx = CompilationContext::new(comp_opts);
        if let Err(diagnostics) = pipeline.execute(&mut comp_ctx) {
            render_diagnostics(
                &comp_ctx.source_names,
                &comp_ctx.sources,
                &diagnostics,
            );
            bail!(
                "Failed to compile Solix build script '{}'",
                script_path.display()
            );
        }
        let binary = comp_ctx
            .emission_artifacts
            .take()
            .ok_or_else(|| anyhow::anyhow!("Missing emission artifacts for build.slx"))?
            .program;
        fs::write(&cached_slxb, binary.to_bytes())?;
    }

    let mut run_cmd = Command::new(solix_bin);
    run_cmd
        .arg("run")
        .arg(&cached_slxb)
        .arg("--profile")
        .arg("release")
        .arg("--pkg-dir")
        .arg(&store_directory);
    apply_build_env_vars(
        &mut run_cmd,
        project_root,
        solix_bin,
        effective_profile,
        resolved_profile,
        args,
    );

    let output = run_cmd.output().with_context(|| {
        format!(
            "failed to invoke '{}' for Solix build script '{}'",
            solix_bin.display(),
            script_path.display()
        )
    })?;

    let stdout_text = String::from_utf8_lossy(&output.stdout);
    let stderr_text = String::from_utf8_lossy(&output.stderr);
    if !stdout_text.is_empty() {
        print!("{stdout_text}");
    }
    if !stderr_text.is_empty() {
        eprint!("{stderr_text}");
    }

    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(1);
        bail!(
            "Solix build script terminated with non-zero exit code: {exit_code}\n[stdout]:\n{stdout_text}\n[stderr]:\n{stderr_text}"
        );
    }

    Ok(())
}

/// Executes a legacy Python `scripts/build.py` script.
fn execute_build_py(
    project_root: &Path,
    script_path: &Path,
    solix_bin: &Path,
    effective_profile: &str,
    resolved_profile: &ResolvedProfile,
    args: &BuildArgs,
) -> Result<()> {
    let python_binary = std::env::var("PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut command = Command::new(python_binary);
    command
        .arg(script_path)
        .arg("--profile")
        .arg(effective_profile);
    if args.check_env {
        command.arg("--check-env");
    }
    if let Some(resolved_triple) = resolve_effective_target_override(args.target.as_deref()) {
        command.arg("--target").arg(resolved_triple);
    }
    apply_build_env_vars(
        &mut command,
        project_root,
        solix_bin,
        effective_profile,
        resolved_profile,
        args,
    );

    let output = command.output().with_context(|| {
        format!(
            "failed to execute python3 build script at '{}'",
            script_path.display()
        )
    })?;

    let stdout_text = String::from_utf8_lossy(&output.stdout);
    let stderr_text = String::from_utf8_lossy(&output.stderr);
    if !stdout_text.is_empty() {
        print!("{stdout_text}");
    }
    if !stderr_text.is_empty() {
        eprint!("{stderr_text}");
    }

    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(1);
        bail!(
            "Build script terminated with non-zero exit code: {exit_code}\n[stdout]:\n{stdout_text}\n[stderr]:\n{stderr_text}"
        );
    }
    Ok(())
}

/// Executes the `build` subcommand by delegating to `scripts/build.slx` or `scripts/build.py`.
///
/// # Errors
/// Returns an error if `manifest.toml` is missing, the build script is missing, or the build fails.
pub fn execute_build(args: BuildArgs) -> Result<()> {
    let project_root = solix_common::canonicalize_host_path(&args.path).with_context(|| {
        format!(
            "failed to resolve project directory '{}'",
            args.path.display()
        )
    })?;

    let manifest_path = project_root.join("manifest.toml");
    if !manifest_path.exists() {
        bail!(
            "Cannot build Solix project at '{}': manifest.toml not found",
            project_root.display()
        );
    }

    let manifest_data =
        ProjectManifest::from_file(&manifest_path).map_err(|error| anyhow::anyhow!("{error}"))?;

    let effective_profile = args.resolved_profile();
    let resolved_profile = manifest_data
        .resolve_profile(effective_profile)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if args.inspect_manifest_json {
        return execute_inspect_manifest_json(
            &project_root,
            &manifest_path,
            &manifest_data,
            &resolved_profile,
            effective_profile,
            &args,
        );
    }

    if args.stage_deps_only {
        let package_store = resolve_package_store_dir(args.pkg_dir.as_deref());
        return stage_dependency_natives(
            &project_root,
            &resolved_profile,
            effective_profile,
            &package_store,
        );
    }

    if args.pack_archive {
        return execute_pack_archive(
            &project_root,
            &manifest_data,
            &resolved_profile,
            effective_profile,
            &args,
        );
    }

    let build_script_path = if let Some(ref explicit_script) = args.script {
        let resolved_script = if explicit_script.is_absolute() {
            explicit_script.clone()
        } else {
            project_root.join(explicit_script)
        };
        if !resolved_script.exists() {
            bail!(
                "Explicit build script '{}' not found",
                resolved_script.display()
            );
        }
        resolved_script
    } else {
        let slx_script = project_root.join("scripts").join("build.slx");
        let py_script = project_root.join("scripts").join("build.py");
        if slx_script.exists() {
            slx_script
        } else if py_script.exists() {
            py_script
        } else {
            bail!(
                "Cannot build Solix project at '{}': neither scripts/build.slx nor scripts/build.py found",
                project_root.display()
            );
        }
    };

    let solix_bin = resolve_solix_binary();
    println!(
        "Building Solix project '{}' (profile: {effective_profile})...",
        project_root.display()
    );

    if build_script_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("slx"))
    {
        execute_build_slx(
            &project_root,
            &build_script_path,
            &solix_bin,
            effective_profile,
            &resolved_profile,
            &args,
        )?;
    } else {
        execute_build_py(
            &project_root,
            &build_script_path,
            &solix_bin,
            effective_profile,
            &resolved_profile,
            &args,
        )?;
    }

    if args.check_env {
        return Ok(());
    }

    if args.install {
        let archive_candidate = project_root
            .join("build")
            .join(effective_profile)
            .join(format!("{}.slxarc", manifest_data.configuration.name));

        let archive_path = if archive_candidate.exists() {
            archive_candidate
        } else {
            let fallback_archive = project_root
                .join("build")
                .join(format!("{}.slxarc", manifest_data.configuration.name));
            if fallback_archive.exists() {
                fallback_archive
            } else {
                bail!(
                    "Cannot install build output: archive not found at '{}'",
                    archive_candidate.display()
                );
            }
        };

        crate::commands::pkg::install_package_artifact(
            &archive_path,
            Some(project_root),
            args.force,
            Some(effective_profile),
            args.pkg_dir.as_deref(),
        )?;
    }

    Ok(())
}
