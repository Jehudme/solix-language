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
    let project_root = args.path.canonicalize().with_context(|| {
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

    let profile = args.resolved_profile();
    let current_exe = std::env::current_exe().unwrap_or_else(|_| "solix".into());
    let solix_bin = if let Ok(custom_bin) = std::env::var("SOLIX_BIN") {
        std::path::PathBuf::from(custom_bin)
    } else if current_exe
        .file_name()
        .is_some_and(|name| name == "solix" || name == "solix.exe")
    {
        current_exe
    } else if let Some(parent_dir) = current_exe.parent() {
        let sibling_candidate = parent_dir.join("solix");
        if sibling_candidate.exists() {
            sibling_candidate
        } else if let Some(grandparent_dir) = parent_dir.parent() {
            let ancestor_candidate = grandparent_dir.join("solix");
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
        "Building Solix project '{}' (profile: {profile})...",
        project_root.display()
    );

    let mut command = Command::new("python3");
    command
        .arg("scripts/build.py")
        .arg("--profile")
        .arg(profile)
        .current_dir(&project_root)
        .env("SOLIX_BIN", solix_bin);

    if let Some(package_store_directory) = &args.pkg_dir {
        command.env("SOLIX_PKG_DIR", package_store_directory);
    }

    if args.resolved_link_mode() == solix_compilation::LinkMode::Static {
        command.env("SOLIX_LINK_MODE", "static");
    }

    let status = command.status().with_context(|| {
        format!(
            "failed to execute python3 build script at '{}'",
            build_script_path.display()
        )
    })?;

    if !status.success() {
        let exit_code = status.code().unwrap_or(1);
        bail!("Build script terminated with non-zero exit code: {exit_code}");
    }

    Ok(())
}
