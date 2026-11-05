//! The `solix new` project scaffolding command driver.

#![forbid(unsafe_code)]

use crate::cli::NewArgs;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Attempts to discover the canonical `scripts/init_project.py` script path.
fn resolve_generator_script_path() -> Option<PathBuf> {
    if let Ok(custom_script) = std::env::var("SOLIX_INIT_PROJECT_PY") {
        let candidate_path = PathBuf::from(custom_script);
        if candidate_path.exists() {
            return Some(candidate_path);
        }
    }

    // Check current working directory
    let local_candidate = Path::new("scripts").join("init_project.py");
    if local_candidate.exists() {
        return Some(local_candidate);
    }

    // Check relative to current executable location
    if let Ok(current_executable) = std::env::current_exe() {
        let mut search_directory = current_executable.parent();
        for _ in 0..5 {
            if let Some(directory) = search_directory {
                let candidate = directory.join("scripts").join("init_project.py");
                if candidate.exists() {
                    return Some(candidate);
                }
                search_directory = directory.parent();
            } else {
                break;
            }
        }
    }

    None
}

/// Embedded Python generator script as a fallback when running standalone without repository files.
const EMBEDDED_INIT_PROJECT_SCRIPT: &str = include_str!("../../../../scripts/init_project.py");

/// Executes the `solix new` subcommand by invoking the Python generator script.
///
/// # Errors
/// Returns an error if the generator script execution fails or returns non-zero.
pub fn execute_new(args: NewArgs) -> Result<()> {
    let mut command = Command::new("python3");

    if let Some(discovered_path) = resolve_generator_script_path() {
        command.arg(discovered_path);
    } else {
        // Fall back to running embedded script directly via python3 -c
        command.arg("-c").arg(EMBEDDED_INIT_PROJECT_SCRIPT);
    }

    command.arg(&args.path);

    if let Some(name) = &args.name {
        command.arg("--name").arg(name);
    }
    command.arg("--version").arg(&args.version);
    command.arg("--description").arg(&args.description);
    command.arg("--license").arg(&args.license);

    for author in &args.authors {
        command.arg("--author").arg(author);
    }

    command.arg("--main").arg(&args.main);
    command.arg("--test").arg(&args.test);

    if args.lib {
        command.arg("--lib");
    }
    if args.no_native {
        command.arg("--no-native");
    }
    for dependency in &args.deps {
        command.arg("--deps").arg(dependency);
    }
    if args.force {
        command.arg("--force");
    }

    let status = command.status().context("failed to execute python3")?;
    if !status.success() {
        let exit_code = status.code().unwrap_or(1);
        bail!("Project initialization failed with exit code {exit_code}");
    }

    Ok(())
}
