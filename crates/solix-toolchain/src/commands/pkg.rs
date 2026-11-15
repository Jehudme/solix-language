//! Package manager command implementations (`install`, `sync`, `uninstall`, `list`).

#![forbid(unsafe_code)]

use crate::cache::FilesystemPackageCache;
use crate::cli::{BuildArgs, InstallArgs, ListArgs, SyncArgs, UninstallArgs};
use crate::commands::execute_build;
use crate::registry::{PackageRecord, PackageRegistry, format_iso8601_timestamp};
use crate::solver::{resolve_native_cache_dir, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::bytecode::BinaryProgram;
use solix_common::cache::PackageCache;
use solix_common::hash::{sha256_file, sha256_hex};
use solix_common::project::{ProjectManifest, SlxarcArchive};
use solix_common::solver::{DependencyRequirement, PackageVersion};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Installs an artifact (.slxb or .slxarc) into the package store, optionally registering source project dir.
///
/// # Errors
/// Returns an error if reading the artifact fails, collision occurs without `--force`, or writing fails.
pub fn install_package_artifact(
    artifact_path: &Path,
    source_project_dir: Option<PathBuf>,
    force: bool,
    profile_override: Option<&str>,
    pkg_dir: Option<&Path>,
) -> Result<PackageRecord> {
    let resolved_artifact = artifact_path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve package artifact at '{}'",
            artifact_path.display()
        )
    })?;

    if !resolved_artifact.exists() {
        bail!(
            "Package artifact not found at '{}'",
            resolved_artifact.display()
        );
    }

    let extension = resolved_artifact
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let store_directory = resolve_package_store_dir(pkg_dir);
    let mut registry = PackageRegistry::load_or_sync(&store_directory)?;

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let iso8601_string = format_iso8601_timestamp(current_timestamp);

    if extension.eq_ignore_ascii_case("slxb") {
        let file_bytes = fs::read(&resolved_artifact).with_context(|| {
            format!(
                "Failed to read binary file at '{}'",
                resolved_artifact.display()
            )
        })?;
        let computed_sha256 = sha256_hex(&file_bytes);
        let program = BinaryProgram::from_bytes(&file_bytes).with_context(|| {
            format!(
                "Failed to deserialize binary container from '{}'",
                resolved_artifact.display()
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

        let effective_profile = profile_override.unwrap_or(if !program.build_profile.is_empty() {
            &program.build_profile
        } else {
            "release"
        });

        let target_dir = store_directory
            .join(package_name)
            .join(package_version.to_string());
        let target_file = target_dir.join(format!("{package_name}.slxb"));

        let already_installed = registry
            .get(package_name, Some(&package_version.to_string()))
            .is_some()
            || target_file.exists();

        if already_installed && !force {
            bail!(
                "Package '{package_name}@{package_version}' is already installed. Use '--force' ('-f') to overwrite."
            );
        }

        fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "Failed to create package store directory at '{}'",
                target_dir.display()
            )
        })?;

        fs::copy(&resolved_artifact, &target_file).with_context(|| {
            format!(
                "Failed to copy package binary to '{}'",
                target_file.display()
            )
        })?;

        let file_size = fs::metadata(&target_file)?.len();

        let record = PackageRecord {
            name: package_name.to_string(),
            version: package_version.to_string(),
            kind: "slxb".to_string(),
            build_profile: effective_profile.to_string(),
            sha256: computed_sha256,
            size_bytes: file_size,
            installed_at_unix_secs: current_timestamp,
            installed_at_iso8601: iso8601_string,
            source_project_dir,
            artifact_path: target_file.clone(),
            native_libraries: program.native_library_checksums.clone(),
            description: program.package_description.clone(),
            license: String::new(),
            authors: Vec::new(),
        };

        registry.register(record.clone())?;

        println!(
            "Installed '{package_name}@{package_version}' (.slxb) to {}",
            target_file.display()
        );
        Ok(record)
    } else if extension.eq_ignore_ascii_case("slxarc") {
        let computed_sha256 = sha256_file(&resolved_artifact).with_context(|| {
            format!(
                "Failed to compute SHA-256 for archive '{}'",
                resolved_artifact.display()
            )
        })?;
        let inspection = SlxarcArchive::inspect(&resolved_artifact).with_context(|| {
            format!(
                "Failed to inspect archive container at '{}'",
                resolved_artifact.display()
            )
        })?;

        let manifest = inspection.manifest.ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot install archive '{}': missing mandatory 'manifest.toml'",
                resolved_artifact.display()
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

        let effective_profile = profile_override.unwrap_or("release");

        let target_dir = store_directory
            .join(package_name)
            .join(package_version.to_string());
        let target_file = target_dir.join(format!("{package_name}.slxarc"));

        let already_installed = registry
            .get(package_name, Some(&package_version.to_string()))
            .is_some()
            || target_file.exists();

        if already_installed && !force {
            bail!(
                "Package '{package_name}@{package_version}' is already installed. Use '--force' ('-f') to overwrite."
            );
        }

        fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "Failed to create package store directory at '{}'",
                target_dir.display()
            )
        })?;

        fs::copy(&resolved_artifact, &target_file).with_context(|| {
            format!(
                "Failed to copy package archive to '{}'",
                target_file.display()
            )
        })?;

        let file_size = fs::metadata(&target_file)?.len();

        // Warm the authenticated extract cache
        let cache = FilesystemPackageCache::new();
        let primary_binary_key = format!("bin/{package_name}.slxb");
        if let Some((_, primary_binary)) = inspection
            .binaries
            .iter()
            .find(|(entry_name, _)| entry_name == &primary_binary_key)
        {
            let binary_raw_bytes = primary_binary.to_bytes();
            let _ = cache.put_cached_binary(
                package_name,
                package_version,
                &target_file,
                primary_binary,
                &binary_raw_bytes,
            );
        }

        let record = PackageRecord {
            name: package_name.to_string(),
            version: package_version.to_string(),
            kind: "slxarc".to_string(),
            build_profile: effective_profile.to_string(),
            sha256: computed_sha256,
            size_bytes: file_size,
            installed_at_unix_secs: current_timestamp,
            installed_at_iso8601: iso8601_string,
            source_project_dir,
            artifact_path: target_file.clone(),
            native_libraries: manifest.native_library_checksums.clone(),
            description: manifest.info.description.unwrap_or_default(),
            license: manifest.info.license.unwrap_or_default(),
            authors: manifest.info.authors,
        };

        registry.register(record.clone())?;

        println!(
            "Installed '{package_name}@{package_version}' (.slxarc) to {}",
            target_file.display()
        );
        Ok(record)
    } else {
        bail!(
            "Unsupported package format '{}'. Only .slxb and .slxarc packages can be installed.",
            resolved_artifact.display()
        );
    }
}

/// Installs a `.slxb` binary container, `.slxarc` archive, or project directory into the package store.
///
/// # Errors
/// Returns an error if building fails, package cannot be read, metadata is missing, or copy operations fail.
pub fn execute_install(args: InstallArgs) -> Result<()> {
    let resolved_path = args.path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve package path at '{}'",
            args.path.display()
        )
    })?;

    if !resolved_path.exists() {
        bail!("Package path not found at '{}'", resolved_path.display());
    }

    if resolved_path.is_dir() {
        let manifest_path = resolved_path.join("manifest.toml");
        if !manifest_path.exists() {
            bail!(
                "Cannot install project at '{}': manifest.toml not found",
                resolved_path.display()
            );
        }

        let manifest = ProjectManifest::from_file(&manifest_path)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let package_name = manifest.info.name.clone();
        let profile = args.resolved_profile();

        execute_build(BuildArgs {
            path: resolved_path.clone(),
            profile: profile.to_string(),
            debug: args.debug,
            release: args.release,
            test: args.test,
            link: "dynamic".to_string(),
            r#static: false,
            pkg_dir: args.pkg_dir.clone(),
            install: false,
            force: args.force,
            yolo: false,
        })?;

        let candidate_archive = resolved_path
            .join("build")
            .join(profile)
            .join(format!("{package_name}.slxarc"));
        let archive_path = if candidate_archive.exists() {
            candidate_archive
        } else {
            let alt_candidate = resolved_path
                .join("build")
                .join(format!("{package_name}.slxarc"));
            if alt_candidate.exists() {
                alt_candidate
            } else {
                bail!(
                    "Build succeeded but expected archive was not found at '{}'",
                    candidate_archive.display()
                );
            }
        };

        install_package_artifact(
            &archive_path,
            Some(resolved_path),
            args.force,
            Some(profile),
            args.pkg_dir.as_deref(),
        )?;
        Ok(())
    } else {
        install_package_artifact(
            &resolved_path,
            None,
            args.force,
            Some(args.resolved_profile()),
            args.pkg_dir.as_deref(),
        )?;
        Ok(())
    }
}

/// Synchronizes and rebuilds installed packages from their recorded source directories.
///
/// # Errors
/// Returns an error if rebuilding, packaging, or registry persistence fails.
pub fn execute_sync(args: SyncArgs) -> Result<()> {
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let registry = PackageRegistry::load_or_sync(&store_directory)?;

    let mut targets_to_sync: Vec<PackageRecord> = Vec::new();

    if args.all {
        for record in &registry.packages {
            if record.source_project_dir.is_some() {
                targets_to_sync.push(record.clone());
            }
        }
        if targets_to_sync.is_empty() {
            println!(
                "No registered packages with source project directories found to synchronize."
            );
            return Ok(());
        }
    } else if let Some(ref target_name) = args.package {
        let matching_records = registry.find_all(target_name);
        if matching_records.is_empty() {
            bail!("Package '{target_name}' is not registered in the package store.");
        }
        for record in matching_records {
            if record.source_project_dir.is_some() {
                targets_to_sync.push(record.clone());
            } else {
                bail!(
                    "Package '{target_name}@{}' does not have a recorded source project directory to synchronize from.",
                    record.version
                );
            }
        }
    } else {
        let current_dir = std::env::current_dir()?;
        let manifest_path = current_dir.join("manifest.toml");
        if manifest_path.exists() {
            let manifest = ProjectManifest::from_file(&manifest_path)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let matching_records = registry.find_all(&manifest.info.name);
            for record in matching_records {
                targets_to_sync.push(record.clone());
            }
            if targets_to_sync.is_empty() {
                execute_install(InstallArgs {
                    path: current_dir,
                    force: args.force,
                    profile: args.profile,
                    debug: false,
                    release: false,
                    test: false,
                    pkg_dir: args.pkg_dir,
                })?;
                return Ok(());
            }
        } else {
            bail!("Must specify a package name or '--all' to synchronize packages.");
        }
    }

    for record in targets_to_sync {
        let project_dir = record.source_project_dir.as_ref().unwrap();
        if !project_dir.exists() {
            eprintln!(
                "warning: Source directory for '{}@{}' at '{}' no longer exists; skipping.",
                record.name,
                record.version,
                project_dir.display()
            );
            continue;
        }

        let effective_profile =
            args.profile
                .as_deref()
                .unwrap_or(if !record.build_profile.is_empty() {
                    &record.build_profile
                } else {
                    "release"
                });

        println!(
            "Synchronizing '{}@{}' from '{}' (profile: {effective_profile})...",
            record.name,
            record.version,
            project_dir.display()
        );

        execute_install(InstallArgs {
            path: project_dir.clone(),
            force: true,
            profile: Some(effective_profile.to_string()),
            debug: false,
            release: false,
            test: false,
            pkg_dir: args.pkg_dir.clone(),
        })?;
    }

    Ok(())
}

/// Uninstalls a package version or an entire package from the local package store.
///
/// # Errors
/// Returns an error if the package is not installed or filesystem deletion fails.
pub fn execute_uninstall(args: UninstallArgs) -> Result<()> {
    let specification = args.spec.trim();
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let native_cache_directory = resolve_native_cache_dir(&store_directory);
    let mut registry = PackageRegistry::load_or_sync(&store_directory)?;

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

        let _ = registry.unregister(&requirement.name, &requirement.version.to_string());

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

        registry
            .packages
            .retain(|record| record.name != specification);
        let _ = registry.save();

        println!("Uninstalled all versions of package '{specification}'");
        Ok(())
    }
}

/// Formats a byte size into a human-readable string (B, KB, MB).
fn format_human_size(size_in_bytes: u64) -> String {
    if size_in_bytes < 1024 {
        format!("{size_in_bytes} B")
    } else if size_in_bytes < 1024 * 1024 {
        format!("{:.1} KB", size_in_bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size_in_bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Lists all packages currently installed in the package store.
///
/// # Errors
/// Returns an error if reading store directory entries fails.
pub fn execute_list(args: ListArgs) -> Result<()> {
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let registry = PackageRegistry::load_or_sync(&store_directory)?;

    let mut filtered_packages: Vec<PackageRecord> = registry.packages;

    if let Some(ref filter_query) = args.filter {
        let query_lower = filter_query.to_ascii_lowercase();
        filtered_packages.retain(|package_record| {
            package_record
                .name
                .to_ascii_lowercase()
                .contains(&query_lower)
                || package_record
                    .description
                    .to_ascii_lowercase()
                    .contains(&query_lower)
        });
    }

    if let Some(ref profile_filter) = args.profile {
        filtered_packages.retain(|package_record| {
            package_record
                .build_profile
                .eq_ignore_ascii_case(profile_filter)
        });
    }

    if let Some(ref kind_filter) = args.kind {
        filtered_packages
            .retain(|package_record| package_record.kind.eq_ignore_ascii_case(kind_filter));
    }

    match args.sort.to_ascii_lowercase().as_str() {
        "version" => {
            filtered_packages.sort_by(|record_alpha, record_beta| {
                let parsed_alpha = PackageVersion::parse(&record_alpha.version).ok();
                let parsed_beta = PackageVersion::parse(&record_beta.version).ok();
                parsed_alpha.cmp(&parsed_beta)
            });
        }
        "date" => {
            filtered_packages.sort_by_key(|record| record.installed_at_unix_secs);
        }
        "size" => {
            filtered_packages.sort_by_key(|record| record.size_bytes);
        }
        "profile" => {
            filtered_packages.sort_by(|record_alpha, record_beta| {
                record_alpha.build_profile.cmp(&record_beta.build_profile)
            });
        }
        _ => {
            filtered_packages.sort_by(|record_alpha, record_beta| {
                record_alpha
                    .name
                    .cmp(&record_beta.name)
                    .then_with(|| record_alpha.version.cmp(&record_beta.version))
            });
        }
    }

    if args.reverse {
        filtered_packages.reverse();
    }

    if args.json {
        let json_output = serde_json::to_string_pretty(&filtered_packages)?;
        println!("{json_output}");
    } else {
        println!("Installed Solix Packages ({}):", store_directory.display());
        if filtered_packages.is_empty() {
            println!("  (no packages found)");
            return Ok(());
        }

        println!(
            "{:<16} {:<10} {:<7} {:<8} {:<10} {:<20} {:<15} SOURCE DIR",
            "NAME", "VERSION", "KIND", "PROFILE", "SIZE", "INSTALLED AT", "SHA256"
        );
        println!(
            "{:-<16} {:-<10} {:-<7} {:-<8} {:-<10} {:-<20} {:-<15} {:-<20}",
            "", "", "", "", "", "", "", ""
        );
        for package in &filtered_packages {
            let short_sha256 = if package.sha256.len() >= 12 {
                format!("{}...", &package.sha256[..12])
            } else {
                package.sha256.clone()
            };
            let size_display = format_human_size(package.size_bytes);
            let source_display = package
                .source_project_dir
                .as_ref()
                .map_or("-", |dir_path| dir_path.to_str().unwrap_or("-"));

            println!(
                "{:<16} {:<10} {:<7} {:<8} {:<10} {:<20} {:<15} {}",
                package.name,
                package.version,
                package.kind,
                package.build_profile,
                size_display,
                package.installed_at_iso8601,
                short_sha256,
                source_display
            );
        }
    }

    Ok(())
}
