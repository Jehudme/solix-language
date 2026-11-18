//! The `solix new` project scaffolding command driver.

#![forbid(unsafe_code)]

use crate::assets::{EMBEDDED_SOLIX_SCAFFOLD_SCRIPT, resolve_scaffold_script_path};
use crate::cli::NewArgs;
use anyhow::{Context, Result, bail};
use std::process::Command;

/// Executes the `solix new` subcommand by invoking the Python generator script.
///
/// # Errors
/// Returns an error if the generator script execution fails or returns non-zero.
pub fn execute_new(args: NewArgs) -> Result<()> {
    let mut command = Command::new("python3");

    if let Ok(discovered_path) = resolve_scaffold_script_path() {
        command.arg(discovered_path);
    } else {
        // Fall back to running embedded script directly via python3 -c
        command.arg("-c").arg(EMBEDDED_SOLIX_SCAFFOLD_SCRIPT);
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
    if args.native {
        command.arg("--native");
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
    command.arg("--script-lang").arg(&args.script_lang);

    let status = command.status().context("failed to execute python3")?;
    if !status.success() {
        let exit_code = status.code().unwrap_or(1);
        bail!("Project initialization failed with exit code {exit_code}");
    }

    Ok(())
}
