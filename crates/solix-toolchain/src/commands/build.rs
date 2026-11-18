//! The `solix build` command driver.

#![forbid(unsafe_code)]

use crate::cli::BuildArgs;
use anyhow::{Context, Result, bail};
use std::process::Command;

/// Executes the `build` subcommand by delegating to the project build script.
///
/// # Errors
/// Returns an error if `manifest.toml` is missing, `python3` fails, or the build script fails.
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

    let build_script_path = project_root.join("scripts").join("build.py");
    if !build_script_path.exists() {
        bail!(
            "Cannot build Solix project at '{}': scripts/build.py not found",
            project_root.display()
        );
    }

    let manifest_data = solix_common::project::ProjectManifest::from_file(&manifest_path)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let effective_profile = if let Some(ref prof) = args.profile {
        prof.as_str()
    } else if args.release {
        "release"
    } else if args.test {
        "test"
    } else if args.debug {
        "debug"
    } else {
        "release"
    };

    let resolved_profile = manifest_data
        .resolve_profile(effective_profile)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let effective_link_mode = if resolved_profile.is_static {
        "static"
    } else {
        "dynamic"
    };

    let effective_pkg_dir = args.pkg_dir.clone();

    let current_exe = std::env::current_exe().unwrap_or_else(|_| "solix".into());
    let binary_name = if cfg!(windows) { "solix.exe" } else { "solix" };
    let solix_bin = if let Ok(custom_bin) = std::env::var("SOLIX_BIN") {
        std::path::PathBuf::from(custom_bin)
    } else if let Ok(cargo_bin_env) = std::env::var("CARGO_BIN_EXE_solix") {
        std::path::PathBuf::from(cargo_bin_env)
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
        .arg(&build_script_path)
        .arg("--profile")
        .arg(effective_profile)
        .current_dir(&project_root)
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

    if let Some(ref target_override) = args.target {
        let resolved_triple = if target_override.eq_ignore_ascii_case("host") {
            solix_common::project::host_target_triple().to_string()
        } else {
            target_override.clone()
        };
        command.arg("--target").arg(&resolved_triple);
        command.env("SOLIX_TARGET_TRIPLE", resolved_triple);
    }

    if let Some(ref package_store_directory) = effective_pkg_dir {
        command.env("SOLIX_PKG_DIR", package_store_directory);
    }

    if effective_link_mode == "static" {
        command.env("SOLIX_LINK_MODE", "static");
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

    let should_install = args.install;
    let force_install = args.force;

    if should_install {
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
            force_install,
            Some(effective_profile),
            effective_pkg_dir.as_deref(),
        )?;
    }

    Ok(())
}
