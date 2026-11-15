//! Package manager command implementations (`install`, `reinstall`, `sync`, `uninstall`, `list`).

#![forbid(unsafe_code)]

use crate::cache::FilesystemPackageCache;
use crate::cli::{BuildArgs, InstallArgs, ListArgs, ReinstallArgs, SyncArgs, UninstallArgs};
use crate::commands::execute_build;
use crate::registry::{PackageRecord, PackageRegistry, format_iso8601_timestamp};
use crate::solver::{resolve_native_cache_dir, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::bytecode::BinaryProgram;
use solix_common::cache::PackageCache;
use solix_common::hash::{sha256_file, sha256_hex};
use solix_common::project::{ProjectManifest, SlxarcArchive};
use solix_common::solver::PackageVersion;
use std::fmt::Write as FmtWrite;
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

        let folder_name = format!("{package_name}-{effective_profile}@{package_version}");
        let target_dir = store_directory.join(&folder_name);
        let target_file = target_dir.join(format!("{package_name}.slxb"));

        let already_installed = registry.packages.iter().any(|existing| {
            existing.name == package_name
                && existing.build_profile == effective_profile
                && existing.version == package_version.to_string()
        }) || target_file.exists();

        if already_installed && !force {
            bail!(
                "Package '{package_name}-{effective_profile}@{package_version}' is already installed. Use '--force' ('-f') to overwrite."
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
            source: source_project_dir,
            artifact_path: target_file.clone(),
            native_libraries: program.native_library_checksums.clone(),
            description: program.package_description.clone(),
            license: String::new(),
            authors: Vec::new(),
        };

        registry.register(record.clone())?;

        println!(
            "Installed '{package_name}-{effective_profile}@{package_version}' (.slxb) to {}",
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

        let folder_name = format!("{package_name}-{effective_profile}@{package_version}");
        let target_dir = store_directory.join(&folder_name);
        let target_file = target_dir.join(format!("{package_name}.slxarc"));

        let already_installed = registry.packages.iter().any(|existing| {
            existing.name == package_name
                && existing.build_profile == effective_profile
                && existing.version == package_version.to_string()
        }) || target_file.exists();

        if already_installed && !force {
            bail!(
                "Package '{package_name}-{effective_profile}@{package_version}' is already installed. Use '--force' ('-f') to overwrite."
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
            source: source_project_dir,
            artifact_path: target_file.clone(),
            native_libraries: manifest.native_library_checksums.clone(),
            description: manifest.info.description.unwrap_or_default(),
            license: manifest.info.license.unwrap_or_default(),
            authors: manifest.info.authors,
        };

        registry.register(record.clone())?;

        println!(
            "Installed '{package_name}-{effective_profile}@{package_version}' (.slxarc) to {}",
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
    if args.bundled {
        let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
        let installed_count =
            crate::assets::install_bundled_packages(&store_directory, args.force)?;
        println!(
            "Installed {installed_count} bundled default package(s) into '{}'",
            store_directory.display()
        );
        return Ok(());
    }

    let input_path = args.path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Missing package path to install. Use 'solix install <path>' or 'solix install --bundled'"
        )
    })?;

    let resolved_path = input_path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve package path at '{}'",
            input_path.display()
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
            profile: Some(profile.to_string()),
            debug: false,
            release: false,
            test: false,
            pkg_dir: args.pkg_dir.clone(),
            install: false,
            force: args.force,
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

/// Reinstalls and rebuilds installed packages from their recorded source directories.
///
/// # Errors
/// Returns an error if rebuilding, packaging, or registry persistence fails.
pub fn execute_reinstall(args: ReinstallArgs) -> Result<()> {
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let registry = PackageRegistry::load_or_sync(&store_directory)?;

    let mut targets_to_reinstall: Vec<PackageRecord> = Vec::new();

    if args.all {
        for record in &registry.packages {
            if record.source.is_some() {
                targets_to_reinstall.push(record.clone());
            }
        }
        if targets_to_reinstall.is_empty() {
            println!("No registered packages with source project directories found to reinstall.");
            return Ok(());
        }
    } else if let Some(ref spec) = args.spec {
        let target_spec = spec.trim();
        let (spec_name_part, spec_version_part) =
            if let Some((left, right)) = target_spec.split_once('@') {
                (left, Some(right))
            } else {
                (target_spec, None)
            };

        let (package_name, profile_filter) =
            if let Some(base) = spec_name_part.strip_suffix("-debug") {
                (base, Some("debug"))
            } else if let Some(base) = spec_name_part.strip_suffix("-release") {
                (base, Some("release"))
            } else if let Some(base) = spec_name_part.strip_suffix("-test") {
                (base, Some("test"))
            } else {
                (spec_name_part, None)
            };

        let mut matching_records: Vec<&PackageRecord> = registry
            .packages
            .iter()
            .filter(|record| record.name == package_name)
            .collect();

        if let Some(prof) = profile_filter {
            matching_records.retain(|record| record.build_profile == prof);
        }
        if let Some(version_str) = spec_version_part {
            matching_records.retain(|record| record.version == version_str);
        }

        if matching_records.is_empty() {
            bail!("Package '{spec}' is not registered in the package store.");
        }

        for record in matching_records {
            if record.source.is_some() {
                targets_to_reinstall.push(record.clone());
            } else {
                bail!(
                    "Package '{}-{}@{}' does not have a recorded source project directory to reinstall from.",
                    record.name,
                    record.build_profile,
                    record.version
                );
            }
        }
    } else {
        let current_directory = std::env::current_dir()?;
        let manifest_path = current_directory.join("manifest.toml");
        if manifest_path.exists() {
            let manifest = ProjectManifest::from_file(&manifest_path)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let matching_records = registry.find_all(&manifest.info.name);
            for record in matching_records {
                targets_to_reinstall.push(record.clone());
            }
            if targets_to_reinstall.is_empty() {
                execute_install(InstallArgs {
                    path: Some(current_directory),
                    bundled: false,
                    force: args.force,
                    profile: args.profile,
                    pkg_dir: args.pkg_dir,
                })?;
                return Ok(());
            }
        } else {
            bail!("Must specify a package name or '--all' to reinstall packages.");
        }
    }

    for record in targets_to_reinstall {
        let project_directory = record.source.as_ref().unwrap();
        if !project_directory.exists() {
            eprintln!(
                "warning: Source directory for '{}-{}@{}' at '{}' no longer exists; skipping.",
                record.name,
                record.build_profile,
                record.version,
                project_directory.display()
            );
            continue;
        }

        let manifest_path = project_directory.join("manifest.toml");
        if !manifest_path.exists() {
            eprintln!(
                "warning: Source manifest for '{}-{}@{}' at '{}' no longer exists; skipping.",
                record.name,
                record.build_profile,
                record.version,
                manifest_path.display()
            );
            continue;
        }

        let source_manifest = ProjectManifest::from_file(&manifest_path)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if source_manifest.info.name != record.name {
            bail!(
                "Source directory manifest name '{}' does not match package name '{}'",
                source_manifest.info.name,
                record.name
            );
        }
        if source_manifest.info.version != record.version {
            bail!(
                "Source directory manifest version '{}' does not match package version '{}'",
                source_manifest.info.version,
                record.version
            );
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
            "Reinstalling '{}-{}@{}' from '{}' (profile: {effective_profile})...",
            record.name,
            record.build_profile,
            record.version,
            project_directory.display()
        );

        execute_install(InstallArgs {
            path: Some(project_directory.clone()),
            bundled: false,
            force: true,
            profile: Some(effective_profile.to_string()),
            pkg_dir: args.pkg_dir.clone(),
        })?;
    }

    Ok(())
}

/// Backward compatibility alias for `execute_reinstall`.
///
/// # Errors
/// Returns an error if rebuilding, packaging, or registry persistence fails.
pub fn execute_sync(args: SyncArgs) -> Result<()> {
    execute_reinstall(args)
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

    let (spec_name_part, spec_version_part) =
        if let Some((left, right)) = specification.split_once('@') {
            (left, Some(right))
        } else {
            (specification, None)
        };

    let (package_name, profile_filter) = if let Some(base) = spec_name_part.strip_suffix("-debug") {
        (base, Some("debug"))
    } else if let Some(base) = spec_name_part.strip_suffix("-release") {
        (base, Some("release"))
    } else if let Some(base) = spec_name_part.strip_suffix("-test") {
        (base, Some("test"))
    } else {
        (spec_name_part, None)
    };

    let removed_records = registry.unregister(package_name, spec_version_part, profile_filter)?;

    let mut any_folder_removed = false;
    if store_directory.exists()
        && let Ok(store_entries) = fs::read_dir(&store_directory)
    {
        for entry_result in store_entries.flatten() {
            let entry_path = entry_result.path();
            let entry_name = entry_result.file_name().to_string_lossy().to_string();

            let should_remove = if let Some(version_target) = spec_version_part {
                if let Some(prof_target) = profile_filter {
                    entry_name == format!("{package_name}-{prof_target}@{version_target}")
                } else {
                    entry_name == format!("{package_name}@{version_target}")
                        || (entry_name.starts_with(&format!("{package_name}-"))
                            && entry_name.ends_with(&format!("@{version_target}")))
                }
            } else if let Some(prof_target) = profile_filter {
                entry_name.starts_with(&format!("{package_name}-{prof_target}@"))
            } else {
                entry_name == package_name
                    || entry_name.starts_with(&format!("{package_name}-"))
                    || entry_name.starts_with(&format!("{package_name}@"))
            };

            if should_remove && entry_path.exists() {
                let _ = fs::remove_dir_all(&entry_path);
                any_folder_removed = true;
            }
        }
    }

    // Also check legacy nested structure: store_directory/<name>/<version>
    let legacy_package_folder = store_directory.join(package_name);
    if legacy_package_folder.exists() {
        if let Some(version_target) = spec_version_part {
            let legacy_version_folder = legacy_package_folder.join(version_target);
            if legacy_version_folder.exists() {
                let _ = fs::remove_dir_all(&legacy_version_folder);
                any_folder_removed = true;
            }
        } else {
            let _ = fs::remove_dir_all(&legacy_package_folder);
            any_folder_removed = true;
        }
        if let Ok(mut remaining) = fs::read_dir(&legacy_package_folder)
            && remaining.next().is_none()
        {
            let _ = fs::remove_dir(&legacy_package_folder);
        }
    }

    // Clean up native cache
    if native_cache_directory.exists()
        && let Ok(entries) = fs::read_dir(&native_cache_directory)
    {
        for entry_result in entries.flatten() {
            let folder_name = entry_result.file_name().to_string_lossy().to_string();
            let should_remove = if let Some(version_target) = spec_version_part {
                if let Some(prof_target) = profile_filter {
                    folder_name == format!("{package_name}-{prof_target}@{version_target}")
                        || folder_name == format!("{package_name}-{version_target}")
                } else {
                    folder_name.starts_with(&format!("{package_name}-"))
                        && (folder_name.ends_with(&format!("@{version_target}"))
                            || folder_name.ends_with(&format!("-{version_target}")))
                }
            } else {
                folder_name.starts_with(&format!("{package_name}-"))
            };
            if should_remove {
                let _ = fs::remove_dir_all(entry_result.path());
            }
        }
    }

    if removed_records.is_empty() && !any_folder_removed {
        bail!("Package '{specification}' is not installed");
    }

    println!("Uninstalled '{specification}'");
    Ok(())
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

    let mut output_content = String::new();
    if args.json {
        output_content = serde_json::to_string_pretty(&filtered_packages)?;
    } else {
        let _ = writeln!(
            output_content,
            "Installed Solix Packages ({}):",
            store_directory.display()
        );
        if filtered_packages.is_empty() {
            let _ = writeln!(output_content, "  (no packages found)");
        } else {
            let _ = writeln!(
                output_content,
                "{:<16} {:<10} {:<7} {:<8} {:<10} {:<20} {:<15} SOURCE",
                "NAME", "VERSION", "KIND", "PROFILE", "SIZE", "INSTALLED AT", "SHA256"
            );
            let _ = writeln!(
                output_content,
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
                    .source
                    .as_ref()
                    .map_or("-", |dir_path| dir_path.to_str().unwrap_or("-"));

                let _ = writeln!(
                    output_content,
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
    }

    if let Some(ref output_path) = args.output {
        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, &output_content).with_context(|| {
            format!("Failed to write list output to '{}'", output_path.display())
        })?;
        println!("Wrote package list to '{}'", output_path.display());
    } else {
        print!("{output_content}");
    }

    Ok(())
}
