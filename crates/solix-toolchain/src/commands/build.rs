//! The `solix build` command driver implementing the locked declarative `manifest.yaml`
//! pipeline and automatic Cargo/CMake native compilation.

#![forbid(unsafe_code)]

use crate::cli::{BuildArgs, CompileSourceArgs};
use crate::commands::compile::collect_slx_sources_recursively;
use crate::solver::{ClientModuleSolver, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::hash::sha256_file;
use solix_common::project::{
    NativeTargetConfig, ProjectManifest, ResolvedProfile, expand_platform_list,
    expand_platforms_to_triples, find_manifest_path, host_platform_id, host_target_triple,
    normalize_platform_token, triple_to_platform_id, validate_archive_relative_path,
};
use solix_common::solver::DependencyRequirement;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

/// Automatically detected native build backend based on marker files in `natives/<name>/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBackend {
    Cargo,
    Cmake,
}

impl NativeBackend {
    fn label(self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Cmake => "cmake",
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedNativeEntry {
    name: String,
    directory: PathBuf,
    backend: NativeBackend,
    native_profile: String,
    target_triples: Vec<String>,
    flags: Vec<String>,
    has_solix_native_feature: bool,
    strategy: Option<String>,
    docker: bool,
    docker_image: Option<String>,
}

impl ResolvedNativeEntry {
    fn is_docker_strategy(&self) -> bool {
        self.docker
            || self.strategy.as_deref().is_some_and(|strategy_name| {
                strategy_name.eq_ignore_ascii_case("docker")
                    || strategy_name.eq_ignore_ascii_case("container")
            })
    }
}

/// Resolves the canonical dockcross container image name for a target triple.
#[must_use]
pub fn dockcross_image_for_triple(triple: &str) -> Option<&'static str> {
    match triple {
        "x86_64-unknown-linux-gnu" => Some("dockcross/linux-x64"),
        "aarch64-unknown-linux-gnu" => Some("dockcross/linux-arm64"),
        "x86_64-pc-windows-gnu" | "x86_64-pc-windows-msvc" => Some("dockcross/windows-static-x64"),
        "x86_64-apple-darwin" => Some("dockcross/darwin-x64"),
        "aarch64-apple-darwin" => Some("dockcross/darwin-arm64"),
        _ => None,
    }
}

/// Resolves the canonical cross-rs container image name for a target triple.
#[must_use]
pub fn cross_image_for_triple(triple: &str) -> String {
    format!("ghcr.io/cross-rs/{triple}:edge")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CargoBuildStrategy {
    NativeClang,
    Zigbuild,
    CargoXwin,
    SystemCross,
    HostCargo,
    Docker,
}

/// Executes the `build` subcommand using declarative `manifest.yaml` (or legacy fallback).
///
/// # Errors
/// Returns an error if the project manifest is missing, environment verification fails,
/// hooks or native compilation fail, or Solix compilation/packaging fails.
pub fn execute_build(args: BuildArgs) -> Result<()> {
    let project_root = solix_common::canonicalize_host_path(&args.path).with_context(|| {
        format!(
            "failed to resolve project directory '{}'",
            args.path.display()
        )
    })?;

    let manifest_path = find_manifest_path(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot build Solix project at '{}': manifest.yaml (or manifest.toml) not found",
            project_root.display()
        )
    })?;

    let is_toml_manifest = manifest_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext_str| ext_str.eq_ignore_ascii_case("toml"));
    let legacy_script_path = project_root.join("scripts").join("build.py");

    let manifest_data =
        ProjectManifest::from_file(&manifest_path).map_err(|error| anyhow::anyhow!("{error}"))?;

    let effective_profile = resolve_cli_profile(&args);
    let resolved_profile = manifest_data
        .resolve_profile(effective_profile)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let solix_bin = resolve_solix_binary();

    // Transitional compatibility for unmigrated legacy manifest.toml + scripts/build.py templates.
    if is_toml_manifest && legacy_script_path.is_file() {
        return execute_legacy_python_build(
            &args,
            &project_root,
            &legacy_script_path,
            &manifest_data,
            &resolved_profile,
            &solix_bin,
        );
    }

    execute_declarative_build(
        &args,
        &project_root,
        &manifest_path,
        &manifest_data,
        &resolved_profile,
        &solix_bin,
    )
}

fn resolve_cli_profile(args: &BuildArgs) -> &str {
    if let Some(ref prof) = args.profile {
        prof.as_str()
    } else if args.release {
        "release"
    } else if args.test {
        "test"
    } else if args.debug {
        "debug"
    } else {
        "release"
    }
}

fn execute_declarative_build(
    args: &BuildArgs,
    project_root: &Path,
    manifest_path: &Path,
    manifest_data: &ProjectManifest,
    resolved_profile: &ResolvedProfile,
    solix_bin: &Path,
) -> Result<()> {
    println!(
        "Building Solix project '{}' (profile: {})...",
        project_root.display(),
        resolved_profile.name
    );

    let native_entries = discover_native_entries(
        project_root,
        resolved_profile,
        args.target.as_deref(),
        args.platform.as_deref(),
        manifest_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext_str| {
                ext_str.eq_ignore_ascii_case("yaml") || ext_str.eq_ignore_ascii_case("yml")
            }),
    )?;

    verify_build_environment(&native_entries, resolved_profile)?;
    if args.check_env {
        println!("  [check] Pre-flight environment verification passed.");
        return Ok(());
    }

    let build_root = project_root.join("build");
    let cache_dir = build_root.join(".cache");
    let profile_dir = build_root.join(&resolved_profile.name);
    let stage_dir = profile_dir.join("stage");
    let stage_bin_dir = stage_dir.join("bin");
    let stage_lib_dir = stage_dir.join("lib");
    let stage_resources_dir = stage_dir.join("resources");

    if stage_dir.exists() {
        fs::remove_dir_all(&stage_dir).with_context(|| {
            format!(
                "failed to reset staging directory '{}'",
                stage_dir.display()
            )
        })?;
    }
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&stage_bin_dir)?;
    fs::create_dir_all(&stage_lib_dir)?;
    fs::create_dir_all(&stage_resources_dir)?;

    let hook_env = build_hook_environment(
        project_root,
        resolved_profile,
        solix_bin,
        &stage_dir,
        &cache_dir,
        None,
    );
    if !resolved_profile.pre_compile.is_empty() {
        println!(
            "=== [Phase 1] Pre-Compile Hooks ({}) ===",
            resolved_profile.pre_compile.len()
        );
        run_hook_commands(
            &resolved_profile.pre_compile,
            "pre_compile",
            project_root,
            &hook_env,
        )?;
    }

    println!(
        "=== Building Solix Project: {} [{}] ===",
        manifest_data.configuration.name, resolved_profile.name
    );

    for entry in &native_entries {
        match entry.backend {
            NativeBackend::Cargo => {
                compile_cargo_native(entry, project_root, &cache_dir, &stage_lib_dir)?;
            }
            NativeBackend::Cmake => {
                compile_cmake_native(entry, project_root, &cache_dir, &stage_lib_dir)?;
            }
        }
    }

    copy_project_tree_filtered(&project_root.join("lib"), &stage_lib_dir, true)?;
    copy_project_tree_filtered(&project_root.join("resources"), &stage_resources_dir, false)?;

    let mut resolved_profile = resolved_profile.clone();
    let required_platforms = resolve_required_platform_ids(
        &resolved_profile,
        args.target.as_deref(),
        args.platform.as_deref(),
    );
    let has_explicit_cli_target = args.target.is_some() || args.platform.is_some();
    stage_dependency_native_libraries(
        &mut resolved_profile,
        args.pkg_dir.as_deref(),
        &stage_lib_dir,
        &required_platforms,
        has_explicit_cli_target,
    )?;

    copy_project_tree_filtered(&project_root.join("bin"), &stage_bin_dir, true)?;
    copy_project_tree_filtered(&stage_lib_dir, &stage_bin_dir, true)?;

    let out_slxb = stage_dir.join("binaries.slxb");
    compile_solix_project_sources(
        project_root,
        manifest_data,
        &resolved_profile,
        &stage_lib_dir,
        &out_slxb,
        args.pkg_dir.as_deref(),
    )?;
    let legacy_slxb = stage_bin_dir.join(format!("{}.slxb", manifest_data.configuration.name));
    let _ = fs::copy(&out_slxb, &legacy_slxb);

    let (checksums, supported_platforms) = collect_native_checksums_and_platforms(&stage_lib_dir)?;
    let mut staged_manifest = manifest_data.clone();
    staged_manifest.native_library_checksums = checksums;
    staged_manifest.supported_platforms = supported_platforms;
    staged_manifest.dynamic_libraries = resolved_profile.dynamic_libraries.clone();

    let staged_manifest_yaml = staged_manifest
        .to_yaml_string()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    fs::write(stage_dir.join("manifest.yaml"), staged_manifest_yaml)?;

    let output_slxp = profile_dir.join(format!("{}.slxp", manifest_data.configuration.name));
    let output_slxarc = profile_dir.join(format!("{}.slxarc", manifest_data.configuration.name));
    if output_slxp.exists() {
        let _ = fs::remove_file(&output_slxp);
    }
    if output_slxarc.exists() {
        let _ = fs::remove_file(&output_slxarc);
    }
    solix_common::project::SlxpPackage::pack_directory(&stage_dir, &output_slxp)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let _ = fs::copy(&output_slxp, &output_slxarc);

    println!("  [package] Emitted package: {}", output_slxp.display());

    if !resolved_profile.post_compile.is_empty() {
        println!(
            "=== [Phase 4] Post-Compile Hooks ({}) ===",
            resolved_profile.post_compile.len()
        );
        let post_env = build_hook_environment(
            project_root,
            &resolved_profile,
            solix_bin,
            &stage_dir,
            &cache_dir,
            Some(&output_slxarc),
        );
        run_hook_commands(
            &resolved_profile.post_compile,
            "post_compile",
            project_root,
            &post_env,
        )?;
    }

    if args.install {
        crate::commands::pkg::install_package_artifact(
            &output_slxarc,
            Some(project_root.to_path_buf()),
            args.force,
            Some(&resolved_profile.name),
            args.pkg_dir.as_deref(),
        )?;
    }

    Ok(())
}

fn build_hook_environment(
    project_root: &Path,
    resolved_profile: &ResolvedProfile,
    solix_bin: &Path,
    stage_dir: &Path,
    cache_dir: &Path,
    output_slxarc: Option<&Path>,
) -> Vec<(String, String)> {
    let mut env_pairs = vec![
        (
            "SOLIX_PROJECT_ROOT".to_string(),
            project_root.display().to_string(),
        ),
        ("SOLIX_PROFILE".to_string(), resolved_profile.name.clone()),
        ("SOLIX_BIN".to_string(), solix_bin.display().to_string()),
        (
            "SOLIX_STAGE_DIR".to_string(),
            stage_dir.display().to_string(),
        ),
        (
            "SOLIX_CACHE_DIR".to_string(),
            cache_dir.display().to_string(),
        ),
    ];
    if let Some(archive_path) = output_slxarc {
        env_pairs.push((
            "SOLIX_OUTPUT_SLXARC".to_string(),
            archive_path.display().to_string(),
        ));
    }
    env_pairs
}

#[cfg(windows)]
fn build_shell_hook_command(raw_command: &str) -> Command {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("cmd");
    // Use /S /C with raw_arg so Windows cmd.exe strips only the single outer quote pair
    // instead of having Rust's standard arg escaper backslash-escape embedded quotes.
    cmd.raw_arg("/S")
        .raw_arg("/C")
        .raw_arg(format!("\"{raw_command}\""));
    cmd
}

#[cfg(not(windows))]
fn build_shell_hook_command(raw_command: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", raw_command]);
    cmd
}

fn run_hook_commands(
    commands: &[String],
    hook_name: &str,
    project_root: &Path,
    env_pairs: &[(String, String)],
) -> Result<()> {
    for raw_command in commands {
        let trimmed = raw_command.trim();
        if trimmed.is_empty() {
            continue;
        }
        println!("  [{hook_name}] {trimmed}");
        let mut command = build_shell_hook_command(trimmed);
        command.current_dir(project_root);
        for (key, value) in env_pairs {
            command.env(key, value);
        }
        let status = command
            .status()
            .with_context(|| format!("failed to spawn {hook_name} command '{trimmed}'"))?;
        if !status.success() {
            let code = status.code().unwrap_or(1);
            bail!("{hook_name} command '{trimmed}' exited with status {code}");
        }
    }
    Ok(())
}

fn discover_native_entries(
    project_root: &Path,
    resolved_profile: &ResolvedProfile,
    cli_target: Option<&str>,
    cli_platform: Option<&str>,
    is_yaml_manifest: bool,
) -> Result<Vec<ResolvedNativeEntry>> {
    let mut entries = Vec::new();
    let mut native_configs: BTreeMap<String, NativeTargetConfig> = resolved_profile.natives.clone();

    if native_configs.is_empty() && !is_yaml_manifest {
        let natives_root = project_root.join("natives");
        if natives_root.is_dir()
            && let Ok(read_dir) = fs::read_dir(&natives_root)
        {
            for dir_entry in read_dir.flatten() {
                let entry_path = dir_entry.path();
                if entry_path.is_dir()
                    && (entry_path.join("Cargo.toml").is_file()
                        || entry_path.join("CMakeLists.txt").is_file())
                    && let Some(dir_name) = entry_path.file_name().and_then(|name| name.to_str())
                {
                    native_configs.insert(dir_name.to_string(), NativeTargetConfig::default());
                }
            }
        }
    }

    for (native_name, native_cfg) in native_configs {
        let native_dir = match native_cfg.path {
            Some(ref rel_or_abs) => {
                let path_obj = PathBuf::from(rel_or_abs);
                if path_obj.is_absolute() {
                    path_obj
                } else {
                    project_root.join(path_obj)
                }
            }
            None => project_root.join("natives").join(&native_name),
        };

        if !native_dir.is_dir() {
            bail!(
                "Declared native project '{}' directory does not exist at '{}'",
                native_name,
                native_dir.display()
            );
        }

        let backend = detect_native_backend(&native_dir, &native_name)?;
        let native_profile = native_cfg
            .profile
            .clone()
            .unwrap_or_else(|| resolved_profile.name.clone());

        let target_triples = resolve_native_triples(
            native_cfg.platforms.as_deref(),
            &resolved_profile.platforms,
            cli_target,
            cli_platform,
        );

        let has_solix_native_feature = match backend {
            NativeBackend::Cargo => cargo_has_solix_native_feature(&native_dir.join("Cargo.toml")),
            NativeBackend::Cmake => false,
        };

        let is_docker = native_cfg.is_docker_strategy();
        entries.push(ResolvedNativeEntry {
            name: native_name,
            directory: native_dir,
            backend,
            native_profile,
            target_triples,
            flags: native_cfg.flags,
            has_solix_native_feature,
            strategy: native_cfg.strategy,
            docker: is_docker,
            docker_image: native_cfg.docker_image,
        });
    }

    Ok(entries)
}

fn detect_native_backend(native_dir: &Path, native_name: &str) -> Result<NativeBackend> {
    if native_dir.join("Cargo.toml").is_file() {
        Ok(NativeBackend::Cargo)
    } else if native_dir.join("CMakeLists.txt").is_file() {
        Ok(NativeBackend::Cmake)
    } else {
        bail!(
            "Cannot auto-detect native backend for '{}' at '{}': expected Cargo.toml or CMakeLists.txt",
            native_name,
            native_dir.display()
        )
    }
}

fn cargo_has_solix_native_feature(cargo_toml_path: &Path) -> bool {
    let Ok(raw_text) = fs::read_to_string(cargo_toml_path) else {
        return false;
    };
    let Ok(parsed_toml) = toml::from_str::<toml::Value>(&raw_text) else {
        return false;
    };
    parsed_toml
        .get("features")
        .and_then(|features| features.as_table())
        .is_some_and(|table| table.contains_key("solix-native"))
}

fn resolve_native_triples(
    native_platforms: Option<&[String]>,
    profile_platforms: &[String],
    cli_target: Option<&str>,
    cli_platform: Option<&str>,
) -> Vec<String> {
    let env_target = std::env::var("SOLIX_TARGET_TRIPLE")
        .ok()
        .map(|val| val.trim().to_string())
        .filter(|val| !val.is_empty());
    if let Some(target_override) = cli_target.or(env_target.as_deref()) {
        let trimmed = target_override.trim();
        let resolved = if trimmed.eq_ignore_ascii_case("host") {
            host_target_triple().to_string()
        } else {
            trimmed.to_string()
        };
        return vec![resolved];
    }

    let chosen_platforms = native_platforms.unwrap_or(profile_platforms);
    let all_triples = expand_platforms_to_triples(chosen_platforms);

    if let Some(platform_filter) = cli_platform {
        let platform_triples = expand_platforms_to_triples(&[platform_filter.to_string()]);
        let filtered: Vec<String> = all_triples
            .into_iter()
            .filter(|triple| {
                platform_triples.contains(triple)
                    || triple
                        .to_ascii_lowercase()
                        .contains(&platform_filter.to_ascii_lowercase())
            })
            .collect();
        if filtered.is_empty() {
            platform_triples
        } else {
            filtered
        }
    } else {
        all_triples
    }
}

fn verify_build_environment(
    native_entries: &[ResolvedNativeEntry],
    resolved_profile: &ResolvedProfile,
) -> Result<()> {
    println!(
        "=== [Phase 0] Pre-Flight Environment Verification [{}] ===",
        resolved_profile.name
    );

    if native_entries.is_empty() {
        let expanded = expand_platforms_to_triples(&resolved_profile.platforms);
        println!(
            "  [platforms] Pure Solix archive (targets: universal / {})",
            expanded.join(", ")
        );
        return Ok(());
    }

    let host_triple = host_target_triple();
    let has_cargo = native_entries
        .iter()
        .any(|entry| entry.backend == NativeBackend::Cargo);
    let has_cmake = native_entries
        .iter()
        .any(|entry| entry.backend == NativeBackend::Cmake);

    if has_cargo {
        let cargo_ver = require_tool_version("cargo", &["--version"])?;
        let rustc_ver = require_tool_version("rustc", &["--version"])?;
        println!("  [check] {cargo_ver}");
        println!("  [check] {rustc_ver}");
    }

    if has_cmake {
        let cmake_ver = require_tool_version("cmake", &["--version"])?;
        println!("  [check] {cmake_ver}");
    }

    for entry in native_entries {
        println!(
            "  [native] {} ({}) targets: {}",
            entry.name,
            entry.backend.label(),
            entry.target_triples.join(", ")
        );
        for triple in &entry.target_triples {
            let is_cross = triple != host_triple;
            if entry.is_docker_strategy() {
                if entry.backend == NativeBackend::Cargo && find_executable("cross").is_some() {
                    let cross_version = require_tool_version("cross", &["--version"])?;
                    println!("  [check] {cross_version} (for {triple} via cross-rs container)");
                } else {
                    let docker_version = require_tool_version("docker", &["--version"])?;
                    println!("  [check] {docker_version} (for {triple} via Docker container)");
                }
                continue;
            }
            match entry.backend {
                NativeBackend::Cargo => {
                    if is_cross {
                        let strategy = select_cargo_strategy(triple, host_triple, false)?;
                        if matches!(strategy, CargoBuildStrategy::Zigbuild) {
                            let zig_ver = require_tool_version("zig", &["version"])?;
                            println!("  [check] zig {zig_ver} (for {triple})");
                        }
                    }
                }
                NativeBackend::Cmake => {
                    if is_cross {
                        let zig_ver = require_tool_version("zig", &["version"])?;
                        println!("  [check] zig {zig_ver} (for {triple})");
                    }
                }
            }
        }
    }

    Ok(())
}

fn build_search_path_env() -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let Some(home_dir) = dirs_home() else {
        return existing;
    };
    let cargo_bin = home_dir.join(".cargo").join("bin");
    let mut paths: Vec<PathBuf> = std::env::split_paths(&existing).collect();
    if cargo_bin.is_dir() && !paths.iter().any(|item| item == &cargo_bin) {
        paths.insert(0, cargo_bin);
    }
    std::env::join_paths(paths).unwrap_or(existing)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn find_executable(tool_name: &str) -> Option<PathBuf> {
    let search_path = build_search_path_env();
    crate::process::find_executable_in_search_paths(tool_name, &search_path)
}

fn require_tool_version(tool_name: &str, version_args: &[&str]) -> Result<String> {
    let exe_path = find_executable(tool_name).ok_or_else(|| {
        anyhow::anyhow!("[verify] Required build tool '{tool_name}' was not found on PATH")
    })?;
    let mut version_command = crate::process::create_process_command(&exe_path);
    let output = version_command
        .args(version_args)
        .env("PATH", build_search_path_env())
        .output()
        .with_context(|| format!("failed to execute '{}'", exe_path.display()))?;
    if !output.status.success() {
        bail!(
            "[verify] Tool '{}' returned non-zero status on '{}'",
            exe_path.display(),
            version_args.join(" ")
        );
    }
    let stdout_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or(tool_name)
        .trim()
        .to_string();
    Ok(format!("{stdout_line} ({})", exe_path.display()))
}

fn select_cargo_strategy(
    target_triple: &str,
    host_triple: &str,
    is_docker: bool,
) -> Result<CargoBuildStrategy> {
    if is_docker {
        return Ok(CargoBuildStrategy::Docker);
    }
    let is_exact_host = target_triple == host_triple;
    let is_darwin_on_darwin = cfg!(target_os = "macos") && target_triple.contains("apple-darwin");
    let is_msvc_target = target_triple.contains("-pc-windows-msvc");
    let has_zigbuild = find_executable("cargo-zigbuild").is_some();
    let has_zig = find_executable("zig").is_some();
    let has_xwin = find_executable("cargo-xwin").is_some();

    if is_darwin_on_darwin {
        return Ok(CargoBuildStrategy::NativeClang);
    }
    if is_exact_host {
        return Ok(CargoBuildStrategy::HostCargo);
    }
    if has_zigbuild && has_zig && !is_msvc_target {
        return Ok(CargoBuildStrategy::Zigbuild);
    }
    if has_xwin && is_msvc_target && !cfg!(target_os = "windows") {
        return Ok(CargoBuildStrategy::CargoXwin);
    }
    if !is_exact_host && system_cross_gcc_for(target_triple).is_some() {
        return Ok(CargoBuildStrategy::SystemCross);
    }

    if is_msvc_target {
        bail!(
            "Cannot cross-compile Cargo target '{target_triple}' on host '{host_triple}': requires Windows MSVC host or cargo-xwin"
        );
    }
    if has_zigbuild && !has_zig {
        bail!(
            "Cannot cross-compile Cargo target '{target_triple}': cargo-zigbuild found but 'zig' executable is missing in PATH"
        );
    }
    bail!(
        "Cannot cross-compile Cargo target '{target_triple}' on host '{host_triple}': install zig + cargo-zigbuild"
    )
}

fn system_cross_gcc_for(target_triple: &str) -> Option<(&'static str, &'static str)> {
    let (cc_name, ar_name) = match target_triple {
        "aarch64-unknown-linux-gnu" => ("aarch64-linux-gnu-gcc", "aarch64-linux-gnu-ar"),
        "x86_64-pc-windows-gnu" => ("x86_64-w64-mingw32-gcc", "x86_64-w64-mingw32-ar"),
        _ => return None,
    };
    if find_executable(cc_name).is_some() {
        Some((cc_name, ar_name))
    } else {
        None
    }
}

fn ensure_rustup_target_installed(
    target_triple: &str,
    search_path: &std::ffi::OsStr,
) -> Result<()> {
    let auto_rustup_disabled = std::env::var("SOLIX_NO_AUTO_RUSTUP")
        .map(|val| {
            matches!(
                val.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false);
    let Some(rustup_exe) = find_executable("rustup") else {
        return Ok(());
    };
    let mut list_cmd = crate::process::create_process_command(&rustup_exe);
    let list_output = list_cmd
        .args(["target", "list", "--installed"])
        .env("PATH", search_path)
        .output();
    let Ok(list_res) = list_output else {
        return Ok(());
    };
    if !list_res.status.success() {
        return Ok(());
    }
    let installed_text = String::from_utf8_lossy(&list_res.stdout);
    if installed_text
        .lines()
        .any(|line| line.trim() == target_triple)
    {
        return Ok(());
    }
    if auto_rustup_disabled {
        bail!(
            "Rust target '{target_triple}' is not installed (SOLIX_NO_AUTO_RUSTUP=1); run `rustup target add {target_triple}`"
        );
    }
    println!("  [rustup] Installing target stdlib '{target_triple}'...");
    let mut add_cmd = crate::process::create_process_command(&rustup_exe);
    let add_status = add_cmd
        .args(["target", "add", target_triple])
        .env("PATH", search_path)
        .status()
        .context("failed to invoke rustup target add")?;
    if !add_status.success() {
        bail!("Failed to install Rust target '{target_triple}' via rustup");
    }
    Ok(())
}

#[must_use]
pub fn resolve_cargo_target_cache_dir(cache_dir: &Path, is_docker: bool) -> PathBuf {
    if is_docker {
        cache_dir.join("cargo-docker")
    } else {
        cache_dir.join("cargo")
    }
}

#[must_use]
pub fn resolve_cmake_target_cache_dir(
    cache_dir: &Path,
    native_name: &str,
    profile_name: &str,
    target_triple: &str,
    is_docker: bool,
) -> PathBuf {
    let cache_subdir = if is_docker { "cmake-docker" } else { "cmake" };
    cache_dir
        .join(cache_subdir)
        .join(native_name)
        .join(profile_name)
        .join(target_triple)
}

fn compile_cargo_native(
    entry: &ResolvedNativeEntry,
    project_root: &Path,
    cache_dir: &Path,
    stage_lib_dir: &Path,
) -> Result<()> {
    let host_triple = host_target_triple();
    let cargo_manifest = entry.directory.join("Cargo.toml");
    let is_release = entry.native_profile.eq_ignore_ascii_case("release");
    let profile_subdir = if is_release { "release" } else { "debug" };
    let search_path = build_search_path_env();
    let flags_specify_features = entry.flags.iter().any(|flag| {
        flag == "--features" || flag.starts_with("--features=") || flag == "--all-features"
    });

    let ffi_rust_source_dir = resolve_ffi_rust_source_dir(project_root);
    let uses_git_solix_ffi = cargo_uses_git_solix_ffi(&cargo_manifest);
    let zig_exe_option = find_executable("zig");

    for triple in &entry.target_triples {
        let strategy = select_cargo_strategy(triple, host_triple, entry.is_docker_strategy())?;
        let is_docker = matches!(strategy, CargoBuildStrategy::Docker);
        let cargo_target_dir = resolve_cargo_target_cache_dir(cache_dir, is_docker);
        let is_exact_host = triple == host_triple;
        if !is_exact_host && !is_docker {
            ensure_rustup_target_installed(triple, &search_path)?;
        }

        let subcommand = match strategy {
            CargoBuildStrategy::Zigbuild => "zigbuild",
            CargoBuildStrategy::CargoXwin => "xwin",
            _ => "build",
        };
        println!(
            "  [cargo] Building '{}' ({}, target: {}, strategy: {:?})...",
            entry.name, entry.native_profile, triple, strategy
        );

        let mut command = if matches!(strategy, CargoBuildStrategy::Docker) {
            if let Some(cross_executable) = find_executable("cross") {
                let mut cmd = crate::process::create_process_command(&cross_executable);
                cmd.arg("build");
                cmd.arg("--manifest-path")
                    .arg(&cargo_manifest)
                    .arg("--target-dir")
                    .arg(&cargo_target_dir);
                cmd
            } else if let Some(docker_executable) = find_executable("docker") {
                let image = entry
                    .docker_image
                    .clone()
                    .unwrap_or_else(|| cross_image_for_triple(triple));
                let mut cmd = crate::process::create_process_command(&docker_executable);
                let manifest_relative = cargo_manifest
                    .strip_prefix(project_root)
                    .unwrap_or(&cargo_manifest);
                let target_dir_relative = cargo_target_dir
                    .strip_prefix(project_root)
                    .unwrap_or(&cargo_target_dir);
                cmd.args(["run", "--rm"]);
                cmd.arg("-v")
                    .arg(format!("{}:/work", project_root.display()));
                cmd.args(["-w", "/work"]);
                cmd.arg(&image);
                cmd.args(["cargo", "build"]);
                cmd.arg("--manifest-path").arg(manifest_relative);
                cmd.arg("--target-dir").arg(target_dir_relative);
                cmd
            } else {
                bail!(
                    "Cannot build containerized native dependency '{}' for target '{triple}': neither 'cross' nor 'docker' executable was found on PATH",
                    entry.name
                );
            }
        } else {
            let cargo_executable =
                find_executable("cargo").unwrap_or_else(|| PathBuf::from("cargo"));
            let mut cmd = crate::process::create_process_command(&cargo_executable);
            if matches!(strategy, CargoBuildStrategy::CargoXwin) {
                cmd.args(["xwin", "build"]);
            } else {
                cmd.arg(subcommand);
            }
            cmd.arg("--manifest-path")
                .arg(&cargo_manifest)
                .arg("--target-dir")
                .arg(&cargo_target_dir);
            cmd
        };
        if is_release {
            command.arg("--release");
            command.env("CARGO_PROFILE_RELEASE_STRIP", "symbols");
        }
        if entry.has_solix_native_feature && !flags_specify_features {
            command.args(["--features", "solix-native"]);
        }
        command.env("SOLIX_NATIVE_BUILD", "1");
        if uses_git_solix_ffi && let Some(ref rust_ffi_dir) = ffi_rust_source_dir {
            let patch_path = cmake_path_string(rust_ffi_dir);
            command.arg("--config").arg(format!(
                "patch.\"https://github.com/Jehudme/solix-language.git\".solix-ffi.path=\"{patch_path}\""
            ));
        }
        let uses_explicit_target_flag = !(matches!(strategy, CargoBuildStrategy::HostCargo)
            && cfg!(target_os = "linux")
            && is_exact_host);
        if uses_explicit_target_flag {
            command.args(["--target", triple]);
        }
        for flag in &entry.flags {
            command.arg(flag);
        }

        command
            .current_dir(project_root)
            .env("PATH", &search_path)
            .env("SOLIX_NATIVE_BUILD", "1")
            .env("CARGO_TARGET_DIR", &cargo_target_dir);

        if matches!(strategy, CargoBuildStrategy::HostCargo)
            && cfg!(target_os = "linux")
            && is_exact_host
        {
            let existing_flags = std::env::var("RUSTFLAGS").unwrap_or_default();
            command.env(
                "RUSTFLAGS",
                format!("-C link-arg=-L/usr/lib {existing_flags}"),
            );
        }

        if matches!(strategy, CargoBuildStrategy::SystemCross)
            && let Some((cross_cc, cross_ar)) = system_cross_gcc_for(triple)
            && let Some(cc_path) = find_executable(cross_cc)
        {
            let underscored = triple.replace('-', "_");
            let upper = underscored.to_ascii_uppercase();
            command.env(format!("CARGO_TARGET_{upper}_LINKER"), &cc_path);
            command.env(format!("CC_{underscored}"), &cc_path);
            if let Some(ar_path) = find_executable(cross_ar) {
                command.env(format!("AR_{underscored}"), ar_path);
            }
        }

        if matches!(strategy, CargoBuildStrategy::Zigbuild)
            && !is_exact_host
            && let Some(ref zig_exe) = zig_exe_option
        {
            let toolchains_dir = cargo_target_dir.join("toolchains");
            if let Ok(toolchain_path) = write_zig_toolchain_file(&toolchains_dir, triple, zig_exe) {
                let bin_dir = toolchains_dir.join("bin");
                let wrapper_ext = if cfg!(windows) { ".cmd" } else { "" };
                let cc_wrapper = bin_dir.join(format!("zig-cc-{triple}{wrapper_ext}"));
                let cxx_wrapper = bin_dir.join(format!("zig-cxx-{triple}{wrapper_ext}"));
                let ar_wrapper = bin_dir.join(format!("zig-ar{wrapper_ext}"));
                let ranlib_wrapper = bin_dir.join(format!("zig-ranlib{wrapper_ext}"));

                let underscored = triple.replace('-', "_");
                command.env(format!("CC_{underscored}"), &cc_wrapper);
                command.env(format!("CXX_{underscored}"), &cxx_wrapper);
                command.env(format!("AR_{underscored}"), &ar_wrapper);
                command.env(format!("RANLIB_{underscored}"), &ranlib_wrapper);
                command.env(
                    format!("CMAKE_TOOLCHAIN_FILE_{underscored}"),
                    &toolchain_path,
                );
                command.env("CMAKE_TOOLCHAIN_FILE", &toolchain_path);

                let current_path = std::env::var("PATH").unwrap_or_default();
                let path_separator = if cfg!(windows) { ";" } else { ":" };
                command.env(
                    "PATH",
                    format!("{}{path_separator}{current_path}", bin_dir.display()),
                );
            }
        }

        let status = command.status().with_context(|| {
            format!("failed to invoke cargo for native project '{}'", entry.name)
        })?;
        if !status.success() {
            bail!(
                "Cargo native build failed for '{}' (target: '{triple}')",
                entry.name
            );
        }

        let emitted_dir = if uses_explicit_target_flag {
            cargo_target_dir.join(triple).join(profile_subdir)
        } else {
            cargo_target_dir.join(profile_subdir)
        };
        let stage_triple_dir = stage_lib_dir.join(triple);
        let staged_count = copy_shared_libs_flat(&emitted_dir, &stage_triple_dir)?;
        if staged_count == 0 {
            bail!(
                "Cargo build succeeded for '{}' ({triple}) but emitted no shared library (.so/.dll/.dylib) in '{}'",
                entry.name,
                emitted_dir.display()
            );
        }
    }
    Ok(())
}

fn cmake_path_string(path: &Path) -> String {
    let raw = path.display().to_string();
    let stripped = if let Some(unc) = raw.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else if let Some(drive) = raw.strip_prefix(r"\\?\") {
        drive.to_string()
    } else {
        raw
    };
    stripped.replace('\\', "/")
}

fn resolve_local_ffi_subdir(
    project_root: &Path,
    env_var: &str,
    subdir: &str,
    marker_file: &str,
) -> Option<PathBuf> {
    if let Ok(explicit_dir) = std::env::var(env_var) {
        let candidate = PathBuf::from(explicit_dir);
        if candidate.join(marker_file).is_file() {
            return Some(candidate);
        }
    }

    let mut search_roots = Vec::new();
    search_roots.push(project_root.to_path_buf());
    if let Ok(exe_path) = std::env::current_exe() {
        search_roots.push(exe_path);
    }
    search_roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    for start_path in search_roots {
        let mut cursor = Some(start_path.as_path());
        while let Some(current_dir) = cursor {
            let ffi_sub = current_dir.join("ffi").join(subdir);
            if ffi_sub.join(marker_file).is_file() {
                return Some(ffi_sub);
            }
            cursor = current_dir.parent();
        }
    }
    None
}

fn resolve_ffi_c_source_dir(project_root: &Path) -> Option<PathBuf> {
    resolve_local_ffi_subdir(project_root, "SOLIX_FFI_SOURCE_DIR", "c", "CMakeLists.txt")
}

fn resolve_ffi_rust_source_dir(project_root: &Path) -> Option<PathBuf> {
    resolve_local_ffi_subdir(project_root, "SOLIX_FFI_RUST_DIR", "rust", "Cargo.toml")
}

fn cargo_uses_git_solix_ffi(cargo_manifest_path: &Path) -> bool {
    let Ok(raw_text) = fs::read_to_string(cargo_manifest_path) else {
        return false;
    };
    raw_text.contains("solix-ffi") && raw_text.contains("github.com/Jehudme/solix-language")
}

fn cmake_target_descriptor(triple: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match triple {
        "x86_64-unknown-linux-gnu" => Some(("Linux", "x86_64", "x86_64-linux-gnu.2.28")),
        "aarch64-unknown-linux-gnu" => Some(("Linux", "aarch64", "aarch64-linux-gnu.2.28")),
        "x86_64-pc-windows-gnu" | "x86_64-pc-windows-msvc" => {
            Some(("Windows", "x86_64", "x86_64-windows-gnu"))
        }
        "x86_64-apple-darwin" => Some(("Darwin", "x86_64", "x86_64-macos")),
        "aarch64-apple-darwin" => Some(("Darwin", "arm64", "aarch64-macos")),
        _ => None,
    }
}

pub fn write_zig_toolchain_file_for_test(
    toolchain_dir: &Path,
    triple: &str,
    zig_exe: &Path,
) -> Result<PathBuf> {
    write_zig_toolchain_file(toolchain_dir, triple, zig_exe)
}

fn write_zig_toolchain_file(toolchain_dir: &Path, triple: &str, zig_exe: &Path) -> Result<PathBuf> {
    let Some((system_name, processor_name, zig_target)) = cmake_target_descriptor(triple) else {
        bail!("Unsupported CMake cross-compilation target triple '{triple}'");
    };

    fs::create_dir_all(toolchain_dir)?;
    let bin_dir = toolchain_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let zig_path_str = zig_exe.display().to_string().replace('\\', "/");
    let wrapper_ext = if cfg!(windows) { ".cmd" } else { "" };

    let cc_wrapper = bin_dir.join(format!("zig-cc-{triple}{wrapper_ext}"));
    let cxx_wrapper = bin_dir.join(format!("zig-cxx-{triple}{wrapper_ext}"));
    let ar_wrapper = bin_dir.join(format!("zig-ar{wrapper_ext}"));
    let ranlib_wrapper = bin_dir.join(format!("zig-ranlib{wrapper_ext}"));

    if cfg!(windows) {
        fs::write(
            &cc_wrapper,
            format!(
                "@echo off\r\n\"{}\" cc -target {zig_target} %*\r\n",
                zig_exe.display()
            ),
        )?;
        fs::write(
            &cxx_wrapper,
            format!(
                "@echo off\r\n\"{}\" c++ -target {zig_target} %*\r\n",
                zig_exe.display()
            ),
        )?;
        fs::write(
            &ar_wrapper,
            format!("@echo off\r\n\"{}\" ar %*\r\n", zig_exe.display()),
        )?;
        fs::write(
            &ranlib_wrapper,
            format!("@echo off\r\n\"{}\" ranlib %*\r\n", zig_exe.display()),
        )?;
    } else {
        write_executable_script(
            &cc_wrapper,
            &format!("#!/bin/sh\nexec \"{zig_path_str}\" cc -target {zig_target} \"$@\"\n"),
        )?;
        write_executable_script(
            &cxx_wrapper,
            &format!("#!/bin/sh\nexec \"{zig_path_str}\" c++ -target {zig_target} \"$@\"\n"),
        )?;
        write_executable_script(
            &ar_wrapper,
            &format!("#!/bin/sh\nexec \"{zig_path_str}\" ar \"$@\"\n"),
        )?;
        write_executable_script(
            &ranlib_wrapper,
            &format!("#!/bin/sh\nexec \"{zig_path_str}\" ranlib \"$@\"\n"),
        )?;
    }

    let cmake_toolchain_path = toolchain_dir.join(format!("toolchain-{triple}.cmake"));
    let host_matches_target = triple == host_target_triple();
    let find_mode = if host_matches_target { "BOTH" } else { "ONLY" };

    let darwin_extra = if system_name == "Darwin" {
        "set(CMAKE_OSX_DEPLOYMENT_TARGET \"11.0\")\nset(CMAKE_INSTALL_RPATH \"@loader_path\")\n"
    } else if system_name == "Linux" {
        "set(CMAKE_INSTALL_RPATH \"$ORIGIN\")\n"
    } else {
        ""
    };

    let root_path_setting = if host_matches_target {
        String::new()
    } else {
        "set(CMAKE_FIND_ROOT_PATH \"/__solix_no_sysroot__\")\n".to_string()
    };

    let toolchain_content = format!(
        "set(CMAKE_SYSTEM_NAME {system_name})\n\
         set(CMAKE_SYSTEM_PROCESSOR {processor_name})\n\
         set(CMAKE_C_COMPILER \"{}\")\n\
         set(CMAKE_CXX_COMPILER \"{}\")\n\
         set(CMAKE_AR \"{}\")\n\
         set(CMAKE_RANLIB \"{}\")\n\
         {darwin_extra}\
         set(CMAKE_BUILD_WITH_INSTALL_RPATH ON)\n\
         {root_path_setting}\
         set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)\n\
         set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY {find_mode})\n\
         set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE {find_mode})\n\
         set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE {find_mode})\n",
        cc_wrapper.display().to_string().replace('\\', "/"),
        cxx_wrapper.display().to_string().replace('\\', "/"),
        ar_wrapper.display().to_string().replace('\\', "/"),
        ranlib_wrapper.display().to_string().replace('\\', "/"),
    );
    fs::write(&cmake_toolchain_path, toolchain_content)?;
    Ok(cmake_toolchain_path)
}

#[cfg(unix)]
fn write_executable_script(script_path: &Path, body: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(script_path, body)?;
    let mut perms = fs::metadata(script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(script_path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_executable_script(script_path: &Path, body: &str) -> Result<()> {
    fs::write(script_path, body)?;
    Ok(())
}

fn compile_cmake_native(
    entry: &ResolvedNativeEntry,
    project_root: &Path,
    cache_dir: &Path,
    stage_lib_dir: &Path,
) -> Result<()> {
    let cmake_exe = find_executable("cmake")
        .ok_or_else(|| anyhow::anyhow!("Required build tool 'cmake' not found in PATH"))?;
    let host_triple = host_target_triple();
    let is_release = entry.native_profile.eq_ignore_ascii_case("release");
    let cmake_build_type = if is_release { "Release" } else { "Debug" };
    let search_path = build_search_path_env();
    let ffi_c_source_dir = resolve_ffi_c_source_dir(project_root);
    let zig_exe_option = find_executable("zig");
    let ninja_available = find_executable("ninja").is_some();

    for triple in &entry.target_triples {
        let is_cross = triple != host_triple;
        println!(
            "  [cmake] Building '{}' ({cmake_build_type}, target: {triple})...",
            entry.name
        );

        let cmake_build_dir = resolve_cmake_target_cache_dir(
            cache_dir,
            &entry.name,
            &entry.native_profile,
            triple,
            entry.is_docker_strategy(),
        );
        let stage_triple_dir = stage_lib_dir.join(triple);
        let archive_cache_dir = cache_dir.join("archives").join(&entry.name).join(triple);
        fs::create_dir_all(&cmake_build_dir)?;
        fs::create_dir_all(&stage_triple_dir)?;
        fs::create_dir_all(&archive_cache_dir)?;

        if entry.is_docker_strategy() {
            let docker_exe = find_executable("docker").ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot cross-compile CMake project '{}' for target '{triple}': 'docker' not found on PATH",
                    entry.name
                )
            })?;
            let dockcross_image = entry
                .docker_image
                .clone()
                .or_else(|| dockcross_image_for_triple(triple).map(str::to_string))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No default dockcross image known for target '{triple}'. Please specify 'docker_image: <image>' in manifest.yaml"
                    )
                })?;

            let stage_triple_cmake = cmake_path_string(&stage_triple_dir);
            let archive_cache_cmake = cmake_path_string(&archive_cache_dir);
            let mut cfg_cmd = crate::process::create_process_command(&docker_exe);
            cfg_cmd.args(["run", "--rm"]);
            cfg_cmd
                .arg("-v")
                .arg(format!("{}:/work", project_root.display()));
            cfg_cmd.args(["-w", "/work"]);
            cfg_cmd.arg(&dockcross_image);
            cfg_cmd.args(["cmake", "-S"]);
            cfg_cmd.arg(cmake_path_string(&entry.directory));
            cfg_cmd.arg("-B");
            cfg_cmd.arg(cmake_path_string(&cmake_build_dir));
            cfg_cmd.arg(format!("-DCMAKE_BUILD_TYPE={cmake_build_type}"));
            cfg_cmd.arg("-DSOLIX_NATIVE_BUILD=ON");
            cfg_cmd.arg("-DBUILD_SHARED_LIBS=ON");
            cfg_cmd.arg("-DCMAKE_POSITION_INDEPENDENT_CODE=ON");
            cfg_cmd.arg("-DBUILD_TESTING=OFF");
            cfg_cmd.arg(format!(
                "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={stage_triple_cmake}"
            ));
            cfg_cmd.arg(format!(
                "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY_RELEASE={stage_triple_cmake}"
            ));
            cfg_cmd.arg(format!(
                "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY_DEBUG={stage_triple_cmake}"
            ));
            cfg_cmd.arg(format!(
                "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY={stage_triple_cmake}"
            ));
            cfg_cmd.arg(format!(
                "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE={stage_triple_cmake}"
            ));
            cfg_cmd.arg(format!(
                "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY_DEBUG={stage_triple_cmake}"
            ));
            cfg_cmd.arg(format!(
                "-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={archive_cache_cmake}"
            ));
            for flag in &entry.flags {
                cfg_cmd.arg(flag);
            }
            let cfg_status = cfg_cmd.status().with_context(|| {
                format!(
                    "failed to configure CMake project '{}' via dockcross container",
                    entry.name
                )
            })?;
            if !cfg_status.success() {
                bail!(
                    "dockcross cmake configure failed for '{}' (target: '{triple}')",
                    entry.name
                );
            }

            let mut build_cmd = crate::process::create_process_command(&docker_exe);
            build_cmd.args(["run", "--rm"]);
            build_cmd
                .arg("-v")
                .arg(format!("{}:/work", project_root.display()));
            build_cmd.args(["-w", "/work"]);
            build_cmd.arg(&dockcross_image);
            build_cmd.args([
                "cmake",
                "--build",
                &cmake_path_string(&cmake_build_dir),
                "--config",
                cmake_build_type,
            ]);
            let build_status = build_cmd.status().with_context(|| {
                format!(
                    "failed to build CMake project '{}' via dockcross container",
                    entry.name
                )
            })?;
            if !build_status.success() {
                bail!(
                    "dockcross cmake build failed for '{}' (target: '{triple}')",
                    entry.name
                );
            }

            let mut staged_count = count_shared_libs_in_dir(&stage_triple_dir);
            if staged_count == 0 {
                staged_count += copy_shared_libs_recursive(&cmake_build_dir, &stage_triple_dir)?;
            }
            if staged_count == 0 {
                bail!(
                    "dockcross CMake build succeeded for '{}' ({triple}) but emitted no shared library (.so/.dll/.dylib)",
                    entry.name
                );
            }
            continue;
        }

        let toolchain_option = if is_cross {
            let zig_exe = zig_exe_option.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot cross-compile CMake project '{}' for target '{triple}' without 'zig' in PATH",
                    entry.name
                )
            })?;
            Some(write_zig_toolchain_file(
                &cache_dir.join("toolchains"),
                triple,
                zig_exe,
            )?)
        } else {
            None
        };

        let mut cfg_cmd = crate::process::create_process_command(&cmake_exe);
        let stage_triple_cmake = cmake_path_string(&stage_triple_dir);
        let archive_cache_cmake = cmake_path_string(&archive_cache_dir);
        cfg_cmd
            .arg("-S")
            .arg(cmake_path_string(&entry.directory))
            .arg("-B")
            .arg(cmake_path_string(&cmake_build_dir))
            .arg(format!("-DCMAKE_BUILD_TYPE={cmake_build_type}"))
            .arg("-DSOLIX_NATIVE_BUILD=ON")
            .arg("-DBUILD_SHARED_LIBS=ON")
            .arg("-DCMAKE_POSITION_INDEPENDENT_CODE=ON")
            .arg("-DBUILD_TESTING=OFF")
            .arg(format!(
                "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={stage_triple_cmake}"
            ))
            .arg(format!(
                "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY_RELEASE={stage_triple_cmake}"
            ))
            .arg(format!(
                "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY_DEBUG={stage_triple_cmake}"
            ))
            .arg(format!(
                "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY={stage_triple_cmake}"
            ))
            .arg(format!(
                "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE={stage_triple_cmake}"
            ))
            .arg(format!(
                "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY_DEBUG={stage_triple_cmake}"
            ))
            .arg(format!(
                "-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={archive_cache_cmake}"
            ));

        if ninja_available && (!cfg!(windows) || is_cross) {
            cfg_cmd.args(["-G", "Ninja"]);
        }
        if let Some(ref toolchain_path) = toolchain_option {
            cfg_cmd.arg(format!(
                "-DCMAKE_TOOLCHAIN_FILE={}",
                cmake_path_string(toolchain_path)
            ));
        }
        if is_release && !triple.contains("apple-darwin") && !triple.contains("-msvc") {
            cfg_cmd.arg("-DCMAKE_SHARED_LINKER_FLAGS=-s");
        }
        for flag in &entry.flags {
            cfg_cmd.arg(flag);
        }

        cfg_cmd.current_dir(project_root).env("PATH", &search_path);
        cfg_cmd.env("SOLIX_NATIVE_BUILD", "1");
        if let Some(ref ffi_dir) = ffi_c_source_dir {
            cfg_cmd.env("SOLIX_FFI_SOURCE_DIR", cmake_path_string(ffi_dir));
        }

        let cfg_status = cfg_cmd
            .status()
            .with_context(|| format!("failed to invoke cmake configure for '{}'", entry.name))?;
        if !cfg_status.success() {
            bail!(
                "CMake configure failed for '{}' (target: '{triple}')",
                entry.name
            );
        }

        let mut build_cmd = crate::process::create_process_command(&cmake_exe);
        build_cmd
            .arg("--build")
            .arg(&cmake_build_dir)
            .arg("--config")
            .arg(cmake_build_type)
            .arg("--parallel")
            .current_dir(project_root)
            .env("PATH", &search_path);

        let build_status = build_cmd
            .status()
            .with_context(|| format!("failed to invoke cmake build for '{}'", entry.name))?;
        if !build_status.success() {
            bail!(
                "CMake build failed for '{}' (target: '{triple}')",
                entry.name
            );
        }

        let mut staged_count = count_shared_libs_in_dir(&stage_triple_dir);
        if staged_count == 0 {
            staged_count += copy_shared_libs_recursive(&cmake_build_dir, &stage_triple_dir)?;
        }
        if staged_count == 0 {
            bail!(
                "CMake build succeeded for '{}' ({triple}) but emitted no shared library (.so/.dll/.dylib)",
                entry.name
            );
        }
    }

    Ok(())
}

fn is_shared_library_file(path: &Path) -> bool {
    let Some(name_str) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let lower = name_str.to_ascii_lowercase();
    lower.ends_with(".so") || lower.ends_with(".dll") || lower.ends_with(".dylib")
}

fn copy_shared_libs_flat(source_dir: &Path, dest_dir: &Path) -> Result<usize> {
    if !source_dir.is_dir() {
        return Ok(0);
    }
    fs::create_dir_all(dest_dir)?;
    let mut copied = 0usize;
    for entry_result in fs::read_dir(source_dir)? {
        let entry = entry_result?;
        let src_path = entry.path();
        if src_path.is_file()
            && is_shared_library_file(&src_path)
            && let Some(file_name) = src_path.file_name()
        {
            fs::copy(&src_path, dest_dir.join(file_name))?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn count_shared_libs_in_dir(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| entry.path().is_file() && is_shared_library_file(&entry.path()))
        .count()
}

fn copy_shared_libs_recursive(source_dir: &Path, dest_dir: &Path) -> Result<usize> {
    if !source_dir.is_dir() {
        return Ok(0);
    }
    fs::create_dir_all(dest_dir)?;
    let mut copied = 0usize;
    for entry_result in fs::read_dir(source_dir)? {
        let entry = entry_result?;
        let src_path = entry.path();
        if src_path.is_dir() {
            if src_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|dir_name| dir_name == "CMakeFiles" || dir_name == "_deps")
            {
                continue;
            }
            copied += copy_shared_libs_recursive(&src_path, dest_dir)?;
        } else if src_path.is_file()
            && is_shared_library_file(&src_path)
            && let Some(file_name) = src_path.file_name()
        {
            fs::copy(&src_path, dest_dir.join(file_name))?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn copy_project_tree_filtered(source_dir: &Path, dest_dir: &Path, only_libs: bool) -> Result<()> {
    if !source_dir.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dest_dir)?;
    for entry_result in fs::read_dir(source_dir)? {
        let entry = entry_result?;
        let src_path = entry.path();
        let Some(name_str) = src_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name_str.starts_with('.') {
            continue;
        }
        let dst_path = dest_dir.join(name_str);
        if src_path.is_dir() {
            copy_project_tree_filtered(&src_path, &dst_path, only_libs)?;
        } else if src_path.is_file() {
            let should_copy = if only_libs {
                name_str.ends_with(".slxb")
                    || name_str.ends_with(".slxarc")
                    || is_shared_library_file(&src_path)
            } else {
                true
            };
            if should_copy {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

fn resolve_required_platform_ids(
    resolved_profile: &ResolvedProfile,
    cli_target: Option<&str>,
    cli_platform: Option<&str>,
) -> Vec<String> {
    let required_triples =
        resolve_native_triples(None, &resolved_profile.platforms, cli_target, cli_platform);
    let mut platform_ids = Vec::new();
    for triple in required_triples {
        let normalized = normalize_platform_token(&triple);
        if !normalized.is_empty() && !platform_ids.contains(&normalized) {
            platform_ids.push(normalized);
        }
    }
    platform_ids
}

fn stage_dependency_native_libraries(
    resolved_profile: &mut ResolvedProfile,
    pkg_dir_override: Option<&Path>,
    stage_lib_dir: &Path,
    required_platforms: &[String],
    has_explicit_cli_target: bool,
) -> Result<()> {
    if resolved_profile.dependencies.is_empty() {
        return Ok(());
    }

    let store_dir = resolve_package_store_dir(pkg_dir_override);
    if !store_dir.is_dir() {
        return Ok(());
    }

    let solver = ClientModuleSolver::with_store_dir(store_dir);
    let mut pending: Vec<DependencyRequirement> = Vec::new();
    for (dep_name, dep_spec) in &resolved_profile.dependencies {
        let spec_str = if dep_spec.profile.is_empty() {
            format!("{dep_name}@{}", dep_spec.version)
        } else {
            format!("{dep_name}@{}#{}", dep_spec.version, dep_spec.profile)
        };
        if let Ok(req) = DependencyRequirement::parse(&spec_str) {
            pending.push(req);
        }
    }

    let mut visited_archives: HashSet<PathBuf> = HashSet::new();
    let mut index = 0usize;
    while index < pending.len() {
        let req = pending[index].clone();
        index += 1;

        let lookup_name = match &req.profile {
            Some(prof) => format!("{}-{prof}", req.name),
            None => req.name.clone(),
        };
        let Ok(archive_path) = solver.resolve_container_path(&lookup_name, req.version) else {
            continue;
        };
        let is_container = archive_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext_str| {
                ext_str.eq_ignore_ascii_case("slxarc") || ext_str.eq_ignore_ascii_case("slxp")
            });
        if !is_container || !visited_archives.insert(archive_path.clone()) {
            continue;
        }

        let archive_file = File::open(&archive_path)?;
        let mut zip_reader = ZipArchive::new(archive_file).with_context(|| {
            format!(
                "failed to open dependency archive '{}'",
                archive_path.display()
            )
        })?;

        let mut qualified_basenames: HashSet<String> = HashSet::new();
        let mut packaged_platforms: BTreeSet<String> = BTreeSet::new();
        let mut has_any_native_libs = false;
        let mut has_unqualified_libs = false;
        for zip_idx in 0..zip_reader.len() {
            let entry = zip_reader.by_index(zip_idx)?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name()?.replace('\\', "/");
            let stripped_name = name
                .strip_prefix("bin/")
                .or_else(|| name.strip_prefix("lib/"));
            if let Some(rel) = stripped_name
                && is_shared_library_file(Path::new(&name))
            {
                has_any_native_libs = true;
                if let Some((subdir, _)) = rel.split_once('/') {
                    let normalized = normalize_platform_token(subdir);
                    if !normalized.is_empty() {
                        packaged_platforms.insert(normalized);
                    }
                    if let Some(file_name) =
                        Path::new(&name).file_name().and_then(|val| val.to_str())
                    {
                        qualified_basenames.insert(file_name.to_string());
                    }
                } else {
                    has_unqualified_libs = true;
                }
            }
        }

        let inspection = solix_common::project::SlxpPackage::inspect(&archive_path).ok();
        let dep_manifest = inspection.and_then(|insp| insp.manifest);

        if has_unqualified_libs
            && packaged_platforms.is_empty()
            && let Some(ref manifest) = dep_manifest
        {
            for plat in expand_platform_list(&manifest.supported_platforms) {
                if !plat.is_empty() {
                    packaged_platforms.insert(plat);
                }
            }
        }

        if has_any_native_libs
            && !packaged_platforms.contains("universal")
            && !packaged_platforms.contains("all")
        {
            let is_universal_all_requested = resolved_profile
                .platforms
                .iter()
                .any(|plat| plat.eq_ignore_ascii_case("all"))
                && !has_explicit_cli_target;

            let missing_platforms: Vec<String> = required_platforms
                .iter()
                .filter(|req_plat| !packaged_platforms.contains(*req_plat))
                .cloned()
                .collect();

            let host_platform = host_platform_id();
            let host_covered = packaged_platforms.contains(host_platform);

            if !missing_platforms.is_empty() {
                if is_universal_all_requested && host_covered {
                    println!(
                        "  [platforms] Note: Dependency '{}@{}' covers [{}] (omitting unavailable targets: [{}])",
                        req.name,
                        req.version,
                        packaged_platforms
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", "),
                        missing_platforms.join(", ")
                    );
                } else {
                    let packaged_display = if packaged_platforms.is_empty() {
                        "none".to_string()
                    } else {
                        packaged_platforms
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    bail!(
                        "Dependency '{}@{}' (profile '{}') does not support requested target platform(s) [{}]: missing native shared library (.so/.dll/.dylib) (packaged platforms: [{}]). Run 'solix install --bundled --force' for universal bundled libraries or rebuild the dependency with matching 'platforms'.",
                        req.name,
                        req.version,
                        req.profile.as_deref().unwrap_or("release"),
                        missing_platforms.join(", "),
                        packaged_display
                    );
                }
            }
        }

        if resolved_profile.link == "static" {
            for zip_idx in 0..zip_reader.len() {
                let mut entry = zip_reader.by_index(zip_idx)?;
                if entry.is_dir() {
                    continue;
                }
                let entry_name = entry.name()?.replace('\\', "/");
                let stripped_subpath = entry_name
                    .strip_prefix("bin/")
                    .or_else(|| entry_name.strip_prefix("lib/"));
                if let Some(rel_subpath) = stripped_subpath
                    && is_shared_library_file(Path::new(&entry_name))
                {
                    let safe_rel = validate_archive_relative_path(rel_subpath)
                        .map_err(|error| anyhow::anyhow!("{error}"))?;
                    let is_qualified = rel_subpath.contains('/');
                    if !is_qualified
                        && Path::new(&entry_name)
                            .file_name()
                            .and_then(|val| val.to_str())
                            .is_some_and(|file_name| qualified_basenames.contains(file_name))
                    {
                        continue;
                    }
                    let dst_path = stage_lib_dir.join(&safe_rel);
                    if dst_path.exists() {
                        continue;
                    }
                    if let Some(parent_dir) = dst_path.parent() {
                        fs::create_dir_all(parent_dir)?;
                    }
                    let mut payload = Vec::new();
                    entry.read_to_end(&mut payload)?;
                    fs::write(&dst_path, payload)?;
                }
            }
        }

        if let Some(ref manifest) = dep_manifest {
            for dep_lib in &manifest.dynamic_libraries {
                if !resolved_profile.dynamic_libraries.contains(dep_lib) {
                    resolved_profile.dynamic_libraries.push(dep_lib.clone());
                }
            }
            let target_prof = req.profile.as_deref().unwrap_or("release");
            if let Ok(dep_resolved) = manifest.resolve_profile(target_prof) {
                for dep_lib in &dep_resolved.dynamic_libraries {
                    if !resolved_profile.dynamic_libraries.contains(dep_lib) {
                        resolved_profile.dynamic_libraries.push(dep_lib.clone());
                    }
                }
                for native_name in dep_resolved.natives.keys() {
                    if !resolved_profile.dynamic_libraries.contains(native_name) {
                        resolved_profile.dynamic_libraries.push(native_name.clone());
                    }
                }
                for (trans_name, trans_spec) in dep_resolved.dependencies {
                    let trans_spec_str =
                        format!("{trans_name}@{}#{}", trans_spec.version, trans_spec.profile);
                    if let Ok(trans_req) = DependencyRequirement::parse(&trans_spec_str) {
                        pending.push(trans_req);
                    }
                }
            }
        }
    }

    Ok(())
}

fn compile_solix_project_sources(
    project_root: &Path,
    manifest_data: &ProjectManifest,
    resolved_profile: &ResolvedProfile,
    stage_lib_dir: &Path,
    output_slxb: &Path,
    pkg_dir_override: Option<&Path>,
) -> Result<()> {
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

    let mut libs = Vec::new();
    collect_slxb_libs_recursively(stage_lib_dir, &mut libs);

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

    println!(
        "  [solix] Compiling Solix sources for '{}' ({})...",
        manifest_data.configuration.name, resolved_profile.name
    );

    crate::commands::compile::execute_compile_source(CompileSourceArgs {
        sources: discovered_files,
        output: Some(output_slxb.to_path_buf()),
        deps_source: Vec::new(),
        deps_package: Vec::new(),
        deps_binaries: Vec::new(),
        deps: dep_specs,
        libs,
        name: Some(manifest_data.configuration.name.clone()),
        version: manifest_data.configuration.version.clone(),
        description: manifest_data.configuration.description.clone(),
        profile: compiler_profile,
        emit_api: None,
        check: false,
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

fn collect_slxb_libs_recursively(dir: &Path, collected: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_slxb_libs_recursively(&entry_path, collected);
        } else if entry_path.is_file()
            && entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext_str| ext_str == "slxb" || ext_str == "slxarc")
        {
            collected.push(entry_path);
        }
    }
}

fn collect_native_checksums_and_platforms(
    stage_lib_dir: &Path,
) -> Result<(std::collections::HashMap<String, String>, Vec<String>)> {
    let mut checksums = std::collections::HashMap::new();
    let mut platform_set: BTreeSet<String> = BTreeSet::new();
    let mut has_unqualified_native = false;

    let mut native_files = Vec::new();
    collect_native_files_recursively(stage_lib_dir, &mut native_files);
    native_files.sort();

    for file_path in native_files {
        let rel_path = file_path
            .strip_prefix(stage_lib_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .replace('\\', "/");
        let digest = sha256_file(&file_path)
            .with_context(|| format!("failed to hash native library '{}'", file_path.display()))?;
        checksums.insert(format!("bin/{rel_path}"), digest.clone());
        checksums.insert(format!("lib/{rel_path}"), digest);

        let parts: Vec<&str> = rel_path.split('/').collect();
        if parts.len() > 1 {
            if let Some(canonical_platform) = triple_to_platform_id(parts[0]) {
                platform_set.insert(canonical_platform.to_string());
            } else {
                platform_set.insert(parts[0].to_string());
            }
        } else {
            has_unqualified_native = true;
        }
    }

    let supported_platforms = if checksums.is_empty() {
        vec!["universal".to_string()]
    } else {
        if platform_set.is_empty() && has_unqualified_native {
            platform_set.insert(host_platform_id().to_string());
        }
        platform_set.into_iter().collect()
    };

    Ok((checksums, supported_platforms))
}

fn collect_native_files_recursively(dir: &Path, collected: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_native_files_recursively(&entry_path, collected);
        } else if entry_path.is_file() && is_shared_library_file(&entry_path) {
            collected.push(entry_path);
        }
    }
}

fn resolve_solix_binary() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| "solix".into());
    let binary_name = if cfg!(windows) { "solix.exe" } else { "solix" };
    if let Ok(custom_bin) = std::env::var("SOLIX_BIN") {
        PathBuf::from(custom_bin)
    } else if let Ok(cargo_bin_env) = std::env::var("CARGO_BIN_EXE_solix") {
        PathBuf::from(cargo_bin_env)
    } else if current_exe
        .file_name()
        .is_some_and(|name| name == "solix" || name == "solix.exe")
    {
        current_exe
    } else if let Some(parent_dir) = current_exe.parent() {
        let sibling_candidate = parent_dir.join(binary_name);
        if sibling_candidate.exists() {
            sibling_candidate
        } else if let Some(grandparent_dir) = parent_dir.parent() {
            let ancestor_candidate = grandparent_dir.join(binary_name);
            if ancestor_candidate.exists() {
                ancestor_candidate
            } else {
                current_exe
            }
        } else {
            current_exe
        }
    } else {
        current_exe
    }
}

fn execute_legacy_python_build(
    args: &BuildArgs,
    project_root: &Path,
    build_script_path: &Path,
    manifest_data: &ProjectManifest,
    resolved_profile: &ResolvedProfile,
    solix_bin: &Path,
) -> Result<()> {
    let effective_profile = resolved_profile.name.as_str();
    let effective_link_mode = if resolved_profile.is_static {
        "static"
    } else {
        "dynamic"
    };

    println!(
        "Building Solix project '{}' (profile: {effective_profile})...",
        project_root.display()
    );

    let python_binary = std::env::var("PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut command = Command::new(python_binary);
    command
        .arg(build_script_path)
        .arg("--profile")
        .arg(effective_profile)
        .current_dir(project_root)
        .env("SOLIX_BIN", solix_bin)
        .env("SOLIX_PROFILE", effective_profile)
        .env("SOLIX_LINK_MODE", effective_link_mode)
        .env(
            "SOLIX_INLINE_THRESHOLD",
            resolved_profile.inline_threshold.to_string(),
        )
        .env(
            "SOLIX_VERIFY_CHECKSUM",
            resolved_profile.verify_checksum.to_string(),
        );

    if args.check_env {
        command.arg("--check-env");
        command.env("SOLIX_VERIFY_ONLY", "1");
    }

    if let Some(ref platform) = args.platform {
        command.arg("--platform").arg(platform);
        command.env("SOLIX_PLATFORM", platform);
    }

    if let Some(ref target_override) = args.target {
        let resolved_triple = if target_override.eq_ignore_ascii_case("host") {
            host_target_triple().to_string()
        } else {
            target_override.clone()
        };
        command.arg("--target").arg(&resolved_triple);
        command.env("SOLIX_TARGET_TRIPLE", resolved_triple);
    }

    if let Some(ref package_store_directory) = args.pkg_dir {
        command.env("SOLIX_PKG_DIR", package_store_directory);
    }

    let output = command.output().with_context(|| {
        format!(
            "failed to execute python3 build script at '{}'",
            build_script_path.display()
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
            Some(project_root.to_path_buf()),
            args.force,
            Some(effective_profile),
            args.pkg_dir.as_deref(),
        )?;
    }

    Ok(())
}
