//! The `solix export` command driver for exporting package archives or standalone binaries.

#![forbid(unsafe_code)]

use crate::cli::{BuildArgs, ExportArgs, ExportBinariesArgs, ExportPackageArgs, ExportSubcommand};
use crate::commands::build::execute_build;
use crate::commands::compile::execute_compile_package_to_output;
use anyhow::{Context, Result, bail};
use solix_common::project::{ProjectManifest, find_manifest_path};
use std::fs;

/// Executes the `export` subcommand.
///
/// # Errors
/// Returns an error if project resolution, compilation, packaging, or file copying fails.
pub fn execute_export(args: ExportArgs) -> Result<()> {
    match args.subcommand {
        ExportSubcommand::Package(package_args) => execute_export_package(package_args),
        ExportSubcommand::Binaries(binaries_args) => execute_export_binaries(binaries_args),
    }
}

fn execute_export_package(args: ExportPackageArgs) -> Result<()> {
    let project_root = solix_common::canonicalize_host_path(&args.path).with_context(|| {
        format!(
            "Failed to resolve export project directory '{}'",
            args.path.display()
        )
    })?;

    let manifest_path = find_manifest_path(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot export package from '{}': manifest.yaml not found",
            project_root.display()
        )
    })?;

    let manifest = ProjectManifest::from_file(&manifest_path)
        .map_err(|manifest_error| anyhow::anyhow!("{manifest_error}"))?;
    let package_name = manifest.configuration.name.clone();
    let profile = args.profile.unwrap_or_else(|| "release".to_string());

    execute_build(BuildArgs {
        path: project_root.clone(),
        platform: None,
        profile: Some(profile.clone()),
        debug: false,
        release: false,
        test: false,
        pkg_dir: args.pkg_dir.clone(),
        install: false,
        force: true,
        check_env: false,
        target: None,
    })?;

    let candidate_slxp = project_root
        .join("build")
        .join(&profile)
        .join(format!("{package_name}.slxp"));

    if !candidate_slxp.exists() {
        bail!(
            "Build succeeded but expected package archive was not found at '{}'",
            candidate_slxp.display()
        );
    }

    let destination_path = match args.output {
        Some(explicit_output) => {
            if let Some(parent_dir) = explicit_output.parent()
                && !parent_dir.as_os_str().is_empty()
            {
                fs::create_dir_all(parent_dir)?;
            }
            fs::copy(&candidate_slxp, &explicit_output).with_context(|| {
                format!(
                    "Failed to write exported package to '{}'",
                    explicit_output.display()
                )
            })?;
            explicit_output
        }
        None => {
            let default_destination = project_root.join(format!("{package_name}.slxp"));
            if default_destination != candidate_slxp {
                fs::copy(&candidate_slxp, &default_destination)?;
            }
            default_destination
        }
    };

    println!(
        "Exported package '{}' to {}",
        package_name,
        destination_path.display()
    );
    Ok(())
}

fn execute_export_binaries(args: ExportBinariesArgs) -> Result<()> {
    let project_root = solix_common::canonicalize_host_path(&args.path).with_context(|| {
        format!(
            "Failed to resolve export project directory '{}'",
            args.path.display()
        )
    })?;

    let manifest_path = find_manifest_path(&project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot export binary from '{}': manifest.yaml not found",
            project_root.display()
        )
    })?;

    let manifest = ProjectManifest::from_file(&manifest_path)
        .map_err(|manifest_error| anyhow::anyhow!("{manifest_error}"))?;
    let package_name = manifest.configuration.name.clone();
    let profile = args.profile.unwrap_or_else(|| "release".to_string());

    let destination_path = args
        .output
        .unwrap_or_else(|| project_root.join(format!("{package_name}.slxb")));

    if let Some(parent_dir) = destination_path.parent()
        && !parent_dir.as_os_str().is_empty()
    {
        fs::create_dir_all(parent_dir)?;
    }

    execute_compile_package_to_output(
        &project_root,
        &manifest,
        &profile,
        &destination_path,
        args.pkg_dir.as_deref(),
        false,
    )?;

    println!(
        "Exported binary '{}' to {}",
        package_name,
        destination_path.display()
    );
    Ok(())
}
