//! Package manager command implementations (`install`, `uninstall`, `list`).

#![forbid(unsafe_code)]

use crate::cli::{InstallArgs, ListArgs, UninstallArgs};
use crate::solver::{resolve_native_cache_dir, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use solix_common::bytecode::BinaryProgram;
use solix_common::project::SlxarcArchive;
use solix_common::solver::{DependencyRequirement, PackageVersion};
use std::fs;

/// Installs a `.slxb` binary container or `.slxarc` archive container into the package store.
///
/// # Errors
/// Returns an error if the file cannot be read, metadata is missing, or copy operations fail.
pub fn execute_install(args: InstallArgs) -> Result<()> {
    let source_path = args.path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve package file at '{}'",
            args.path.display()
        )
    })?;

    if !source_path.exists() {
        bail!("Package file not found at '{}'", source_path.display());
    }

    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());

    if extension.eq_ignore_ascii_case("slxb") {
        let file_bytes = fs::read(&source_path).with_context(|| {
            format!("Failed to read binary file at '{}'", source_path.display())
        })?;
        let program = BinaryProgram::from_bytes(&file_bytes).with_context(|| {
            format!(
                "Failed to deserialize binary container from '{}'",
                source_path.display()
            )
        })?;

        let package_name = program.package_name.trim();
        if package_name.is_empty() {
            bail!("Cannot install binary with empty package_name metadata");
        }

        let package_version =
            PackageVersion::parse(&program.package_version).with_context(|| {
                format!(
                    "Cannot install package '{}': invalid package_version '{}'",
                    package_name, program.package_version
                )
            })?;

        let target_dir = store_directory
            .join(package_name)
            .join(package_version.to_string());
        fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "Failed to create package store directory at '{}'",
                target_dir.display()
            )
        })?;

        let target_file = target_dir.join(format!("{package_name}.slxb"));
        fs::copy(&source_path, &target_file).with_context(|| {
            format!(
                "Failed to copy package binary to '{}'",
                target_file.display()
            )
        })?;

        println!(
            "Installed '{}@{}' (.slxb) to {}",
            package_name,
            package_version,
            target_file.display()
        );
        Ok(())
    } else if extension.eq_ignore_ascii_case("slxarc") {
        let inspection = SlxarcArchive::inspect(&source_path).with_context(|| {
            format!(
                "Failed to inspect archive container at '{}'",
                source_path.display()
            )
        })?;

        let manifest = inspection.manifest.ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot install archive '{}': missing mandatory 'manifest.toml'",
                source_path.display()
            )
        })?;

        let package_name = manifest.info.name.trim();
        if package_name.is_empty() {
            bail!("Cannot install archive with empty package name in manifest.toml");
        }

        let package_version = PackageVersion::parse(&manifest.info.version).with_context(|| {
            format!(
                "Cannot install package '{}': invalid version '{}' in manifest.toml",
                package_name, manifest.info.version
            )
        })?;

        let target_dir = store_directory
            .join(package_name)
            .join(package_version.to_string());
        fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "Failed to create package store directory at '{}'",
                target_dir.display()
            )
        })?;

        let target_file = target_dir.join(format!("{package_name}.slxarc"));
        fs::copy(&source_path, &target_file).with_context(|| {
            format!(
                "Failed to copy package archive to '{}'",
                target_file.display()
            )
        })?;

        println!(
            "Installed '{}@{}' (.slxarc) to {}",
            package_name,
            package_version,
            target_file.display()
        );
        Ok(())
    } else {
        bail!(
            "Unsupported package format '{}'. Only .slxb and .slxarc packages can be installed.",
            source_path.display()
        );
    }
}

/// Uninstalls a package version or an entire package from the local package store.
///
/// # Errors
/// Returns an error if the package is not installed or filesystem deletion fails.
pub fn execute_uninstall(args: UninstallArgs) -> Result<()> {
    let specification = args.spec.trim();
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let native_cache_directory = resolve_native_cache_dir(&store_directory);

    if specification.contains('@') {
        let requirement = DependencyRequirement::parse(specification)?;
        let version_folder = store_directory
            .join(&requirement.name)
            .join(requirement.version.to_string());

        if !version_folder.exists() {
            bail!(
                "Package version '{}@{}' is not installed",
                requirement.name,
                requirement.version
            );
        }

        fs::remove_dir_all(&version_folder).with_context(|| {
            format!(
                "Failed to remove package directory '{}'",
                version_folder.display()
            )
        })?;

        let cached_natives_folder =
            native_cache_directory.join(format!("{}-{}", requirement.name, requirement.version));
        if cached_natives_folder.exists() {
            let _ = fs::remove_dir_all(&cached_natives_folder);
        }

        println!("Uninstalled '{}@{}'", requirement.name, requirement.version);

        let package_folder = store_directory.join(&requirement.name);
        if let Ok(mut entries) = fs::read_dir(&package_folder)
            && entries.next().is_none()
        {
            let _ = fs::remove_dir(&package_folder);
        }
        Ok(())
    } else {
        let package_folder = store_directory.join(specification);
        if !package_folder.exists() {
            bail!("Package '{specification}' is not installed");
        }

        fs::remove_dir_all(&package_folder).with_context(|| {
            format!(
                "Failed to remove package directory '{}'",
                package_folder.display()
            )
        })?;

        if let Ok(entries) = fs::read_dir(&native_cache_directory) {
            for entry_result in entries.flatten() {
                let folder_name = entry_result.file_name();
                let name_str = folder_name.to_string_lossy();
                if name_str.starts_with(&format!("{specification}-")) {
                    let _ = fs::remove_dir_all(entry_result.path());
                }
            }
        }

        println!("Uninstalled all versions of package '{specification}'");
        Ok(())
    }
}

/// Metadata description of an installed package artifact in the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackageItem {
    pub name: String,
    pub version: String,
    pub kind: String,
    pub description: String,
    pub path: String,
}

/// Lists all packages currently installed in the package store.
///
/// # Errors
/// Returns an error if reading store directory entries fails.
pub fn execute_list(args: ListArgs) -> Result<()> {
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let mut installed_packages = Vec::new();

    if store_directory.exists() && store_directory.is_dir() {
        let package_folders = fs::read_dir(&store_directory).with_context(|| {
            format!(
                "Failed to read package store at '{}'",
                store_directory.display()
            )
        })?;

        for package_entry_result in package_folders.flatten() {
            if !package_entry_result
                .file_type()
                .is_ok_and(|file_type| file_type.is_dir())
            {
                continue;
            }
            let package_name = package_entry_result
                .file_name()
                .to_string_lossy()
                .to_string();
            let version_folders = fs::read_dir(package_entry_result.path())?;

            for version_entry_result in version_folders.flatten() {
                if !version_entry_result
                    .file_type()
                    .is_ok_and(|file_type| file_type.is_dir())
                {
                    continue;
                }
                let version_string = version_entry_result
                    .file_name()
                    .to_string_lossy()
                    .to_string();
                let version_dir = version_entry_result.path();

                let slxarc_file = version_dir.join(format!("{package_name}.slxarc"));
                let slxb_file = version_dir.join(format!("{package_name}.slxb"));

                if slxarc_file.exists() {
                    let description = if let Ok(inspection) = SlxarcArchive::inspect(&slxarc_file) {
                        inspection
                            .manifest
                            .and_then(|manifest| manifest.info.description)
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    installed_packages.push(InstalledPackageItem {
                        name: package_name.clone(),
                        version: version_string,
                        kind: "slxarc".to_string(),
                        description,
                        path: slxarc_file.display().to_string(),
                    });
                } else if slxb_file.exists() {
                    let description = if let Ok(bytes) = fs::read(&slxb_file) {
                        if let Ok(program) = BinaryProgram::from_bytes(&bytes) {
                            program.package_description
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    installed_packages.push(InstalledPackageItem {
                        name: package_name.clone(),
                        version: version_string,
                        kind: "slxb".to_string(),
                        description,
                        path: slxb_file.display().to_string(),
                    });
                }
            }
        }
    }

    installed_packages.sort_by(|item_a, item_b| {
        item_a
            .name
            .cmp(&item_b.name)
            .then_with(|| item_a.version.cmp(&item_b.version))
    });

    if args.json {
        let json_output = serde_json::to_string_pretty(&installed_packages)?;
        println!("{json_output}");
    } else {
        println!("Installed Solix Packages ({}):", store_directory.display());
        if installed_packages.is_empty() {
            println!("  (no packages installed)");
            return Ok(());
        }

        println!("{:<20} {:<12} {:<8} DESCRIPTION", "NAME", "VERSION", "KIND");
        println!("{:-<20} {:-<12} {:-<8} {:-<30}", "", "", "", "");
        for package in &installed_packages {
            println!(
                "{:<20} {:<12} {:<8} {}",
                package.name, package.version, package.kind, package.description
            );
        }
    }

    Ok(())
}
