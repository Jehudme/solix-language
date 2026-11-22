//! Package manager command implementations (`install`, `reinstall`, `sync`, `uninstall`, `list`).

#![forbid(unsafe_code)]

use crate::cache::FilesystemPackageCache;
use crate::cli::{
    BuildArgs, InstallArgs, InstallBinariesArgs, InstallPackageArgs, InstallSubcommand, ListArgs,
    UninstallArgs, UninstallBinariesArgs, UninstallPackageArgs, UninstallSubcommand,
};
use crate::commands::execute_build;
use crate::registry::{
    PackageRecord, PackageRegistry, format_iso8601_timestamp, parse_package_spec,
};
use crate::solver::{resolve_native_cache_dir, resolve_package_store_dir};
use anyhow::{Context, Result, bail};
use solix_common::bytecode::BinaryProgram;
use solix_common::cache::PackageCache;
use solix_common::hash::{sha256_file, sha256_hex};
use solix_common::project::ProjectManifest;
use solix_common::solver::PackageVersion;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Resolves the store directory for standalone binaries (~/.solix/bin/ or $SOLIX_BIN_DIR or override).
pub fn resolve_bin_store_dir(pkg_dir_override: Option<&Path>) -> PathBuf {
    if let Some(custom_dir) = pkg_dir_override {
        let canonical = custom_dir.to_path_buf();
        if canonical.file_name().and_then(|val| val.to_str()) == Some("packages")
            && let Some(parent) = canonical.parent()
        {
            return parent.join("bin");
        }
        return canonical.join("bin");
    }

    if let Ok(bin_dir) = std::env::var("SOLIX_BIN_DIR") {
        let trimmed = bin_dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(home_dir) = std::env::var("SOLIX_HOME") {
        let trimmed = home_dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("bin");
        }
    }

    if cfg!(windows) {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let trimmed = user_profile.trim();
            if !trimmed.is_empty() {
                return PathBuf::from(trimmed).join(".solix").join("bin");
            }
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join(".solix").join("bin");
        }
    }

    PathBuf::from(".solix").join("bin")
}

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
    install_package_artifact_with_options(
        artifact_path,
        source_project_dir,
        force,
        profile_override,
        pkg_dir,
        false,
    )
}

/// Internal implementation of [`install_package_artifact`] with optional quiet mode for auto-bootstrap.
///
/// # Errors
/// Returns an error if reading the artifact fails, collision occurs without `--force`, or writing fails.
pub(crate) fn install_package_artifact_with_options(
    artifact_path: &Path,
    source_project_dir: Option<PathBuf>,
    force: bool,
    profile_override: Option<&str>,
    pkg_dir: Option<&Path>,
    quiet: bool,
) -> Result<PackageRecord> {
    let resolved_artifact =
        solix_common::canonicalize_host_path(artifact_path).with_context(|| {
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
            kind: "binaries".to_string(),
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
            platforms: vec!["universal".to_string()],
        };

        registry.register(record.clone())?;

        if !quiet {
            println!(
                "Installed '{package_name}-{effective_profile}@{package_version}' (.slxb) to {}",
                target_file.display()
            );
        }
        Ok(record)
    } else if extension.eq_ignore_ascii_case("slxp") || extension.eq_ignore_ascii_case("slxarc") {
        let is_slxp = extension.eq_ignore_ascii_case("slxp");
        let computed_sha256 = sha256_file(&resolved_artifact).with_context(|| {
            format!(
                "Failed to compute SHA-256 for archive '{}'",
                resolved_artifact.display()
            )
        })?;
        let inspection = solix_common::project::SlxpPackage::inspect(&resolved_artifact)
            .with_context(|| {
                format!(
                    "Failed to inspect archive container at '{}'",
                    resolved_artifact.display()
                )
            })?;

        let manifest = inspection.manifest.ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot install archive '{}': missing or invalid 'manifest.yaml' / 'manifest.toml'",
                resolved_artifact.display()
            )
        })?;

        solix_common::project::validate_archive_host_compatibility(
            &manifest,
            &resolved_artifact.display().to_string(),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;

        let package_name = manifest.configuration.name.trim();
        if package_name.is_empty() {
            bail!("Cannot install archive with empty package name in manifest.toml");
        }

        let package_version =
            PackageVersion::parse(&manifest.configuration.version).with_context(|| {
                format!(
                    "Cannot install package '{}': invalid version '{}' in manifest.toml",
                    package_name, manifest.configuration.version
                )
            })?;

        let effective_profile = profile_override.unwrap_or("release");

        let folder_name = format!("{package_name}-{effective_profile}@{package_version}");
        let target_dir = store_directory.join(&folder_name);
        let target_ext = if is_slxp { "slxp" } else { "slxarc" };
        let target_file = target_dir.join(format!("{package_name}.{target_ext}"));

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

        let dual_ext = if is_slxp { "slxarc" } else { "slxp" };
        let dual_file = target_dir.join(format!("{package_name}.{dual_ext}"));
        let _ = fs::copy(&resolved_artifact, &dual_file);

        let file_size = fs::metadata(&target_file)?.len();

        // Warm the authenticated extract cache
        let cache = FilesystemPackageCache::new();
        let primary_binary_key = format!("bin/{package_name}.slxb");
        if let Some((_, primary_binary)) = inspection.binaries.iter().find(|(entry_name, _)| {
            entry_name == "binaries.slxb" || entry_name == &primary_binary_key
        }) {
            let binary_raw_bytes = primary_binary.to_bytes();
            let _ = cache.put_cached_binary(
                package_name,
                package_version,
                &target_file,
                primary_binary,
                &binary_raw_bytes,
            );
        }

        let record_platforms = if !manifest.supported_platforms.is_empty() {
            manifest.supported_platforms.clone()
        } else if !manifest.native_library_checksums.is_empty() {
            let mut extracted_targets: Vec<String> = Vec::new();
            for library_key in manifest.native_library_checksums.keys() {
                let clean_key = library_key.trim_start_matches('/');
                let stripped_prefix = clean_key
                    .strip_prefix("bin/")
                    .or_else(|| clean_key.strip_prefix("lib/"));
                if let Some(tail_path) = stripped_prefix
                    && let Some((target_segment, _)) = tail_path.split_once('/')
                {
                    let normalized_target =
                        solix_common::project::normalize_platform_token(target_segment);
                    if !normalized_target.is_empty()
                        && !extracted_targets.contains(&normalized_target)
                    {
                        extracted_targets.push(normalized_target);
                    }
                }
            }
            if extracted_targets.is_empty() {
                vec!["universal".to_string()]
            } else {
                extracted_targets.sort();
                extracted_targets
            }
        } else {
            vec!["universal".to_string()]
        };

        let record = PackageRecord {
            name: package_name.to_string(),
            version: package_version.to_string(),
            kind: "package".to_string(),
            build_profile: effective_profile.to_string(),
            sha256: computed_sha256,
            size_bytes: file_size,
            installed_at_unix_secs: current_timestamp,
            installed_at_iso8601: iso8601_string,
            source: source_project_dir,
            artifact_path: target_file.clone(),
            native_libraries: manifest.native_library_checksums.clone(),
            description: manifest.configuration.description.unwrap_or_default(),
            license: manifest.configuration.license.unwrap_or_default(),
            authors: manifest.configuration.authors,
            platforms: record_platforms,
        };

        registry.register(record.clone())?;

        if !quiet {
            println!(
                "Installed '{package_name}-{effective_profile}@{package_version}' (.{target_ext}) to {}",
                target_file.display()
            );
        }
        Ok(record)
    } else {
        bail!(
            "Unsupported package format '{}'. Only .slxp, .slxb, and .slxarc packages can be installed.",
            resolved_artifact.display()
        );
    }
}

/// Installs a package (.slxp) into the package store.
///
/// # Errors
/// Returns an error if building fails or installation fails.
pub fn execute_install_package(args: InstallPackageArgs) -> Result<()> {
    let resolved_path = solix_common::canonicalize_host_path(&args.path).with_context(|| {
        format!(
            "Failed to resolve package path at '{}'",
            args.path.display()
        )
    })?;

    if !resolved_path.exists() {
        bail!("Package path not found at '{}'", resolved_path.display());
    }

    if resolved_path.is_dir() {
        let manifest_path =
            solix_common::project::find_manifest_path(&resolved_path).ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot install project at '{}': manifest.yaml (or manifest.toml) not found",
                    resolved_path.display()
                )
            })?;

        let manifest = ProjectManifest::from_file(&manifest_path)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let package_name = manifest.configuration.name.clone();
        let profile = args.resolved_profile();

        execute_build(BuildArgs {
            path: resolved_path.clone(),
            platform: None,
            profile: Some(profile.to_string()),
            debug: false,
            release: false,
            test: false,
            pkg_dir: args.pkg_dir.clone(),
            install: false,
            force: args.force,
            check_env: false,
            target: None,
        })?;

        let candidate_slxp = resolved_path
            .join("build")
            .join(profile)
            .join(format!("{package_name}.slxp"));
        let candidate_archive = resolved_path
            .join("build")
            .join(profile)
            .join(format!("{package_name}.slxarc"));
        let archive_path = if candidate_slxp.exists() {
            candidate_slxp
        } else if candidate_archive.exists() {
            candidate_archive
        } else {
            bail!(
                "Build succeeded but expected package archive was not found at '{}'",
                candidate_slxp.display()
            );
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

/// Installs a standalone binary (.slxb) into ~/.solix/bin/.
///
/// # Errors
/// Returns an error if reading the binary or copying to the binary store fails.
pub fn execute_install_binaries(args: InstallBinariesArgs) -> Result<()> {
    let resolved_path = solix_common::canonicalize_host_path(&args.path)
        .with_context(|| format!("Failed to resolve binary path at '{}'", args.path.display()))?;

    if !resolved_path.exists() {
        bail!("Binary path not found at '{}'", resolved_path.display());
    }

    let file_bytes = fs::read(&resolved_path).with_context(|| {
        format!(
            "Failed to read binary file at '{}'",
            resolved_path.display()
        )
    })?;
    let program = BinaryProgram::from_bytes(&file_bytes).with_context(|| {
        format!(
            "Failed to deserialize binary container from '{}'",
            resolved_path.display()
        )
    })?;

    let binary_name = if !program.package_name.trim().is_empty() {
        program.package_name.trim().to_string()
    } else {
        resolved_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("binary")
            .to_string()
    };

    let bin_store_dir = resolve_bin_store_dir(args.pkg_dir.as_deref());
    fs::create_dir_all(&bin_store_dir).with_context(|| {
        format!(
            "Failed to create binary store directory at '{}'",
            bin_store_dir.display()
        )
    })?;

    let destination_file = bin_store_dir.join(format!("{binary_name}.slxb"));
    if destination_file.exists() && !args.force {
        bail!(
            "Binary '{}' is already installed at '{}'. Use '--force' to overwrite.",
            binary_name,
            destination_file.display()
        );
    }

    fs::copy(&resolved_path, &destination_file)
        .with_context(|| format!("Failed to copy binary to '{}'", destination_file.display()))?;

    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let mut registry = PackageRegistry::load_or_sync(&store_directory)?;

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let iso8601_string = format_iso8601_timestamp(current_timestamp);
    let computed_sha256 = sha256_hex(&file_bytes);
    let file_size = destination_file.metadata()?.len();

    let record = PackageRecord {
        name: binary_name.clone(),
        version: if !program.package_version.trim().is_empty() {
            program.package_version.clone()
        } else {
            "0.1.0".to_string()
        },
        kind: "binaries".to_string(),
        build_profile: args.resolved_profile().to_string(),
        sha256: computed_sha256,
        size_bytes: file_size,
        installed_at_unix_secs: current_timestamp,
        installed_at_iso8601: iso8601_string,
        source: None,
        artifact_path: destination_file.clone(),
        native_libraries: program.native_library_checksums.clone(),
        description: program.package_description.clone(),
        license: String::new(),
        authors: Vec::new(),
        platforms: vec!["universal".to_string()],
    };

    registry.register(record)?;
    println!(
        "Installed binary '{}' to {}",
        binary_name,
        destination_file.display()
    );
    Ok(())
}

/// Installs a `.slxb` binary container, `.slxarc` archive, or project directory into the package store.
///
/// # Errors
/// Returns an error if building fails, package cannot be read, metadata is missing, or copy operations fail.
pub fn execute_install(args: InstallArgs) -> Result<()> {
    if let Some(subcmd) = args.subcommand {
        return match subcmd {
            InstallSubcommand::Package(package_args) => execute_install_package(package_args),
            InstallSubcommand::Binaries(binaries_args) => execute_install_binaries(binaries_args),
        };
    }
    if args.vscode {
        return execute_install_vscode();
    }

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

    let resolved_path = solix_common::canonicalize_host_path(input_path).with_context(|| {
        format!(
            "Failed to resolve package path at '{}'",
            input_path.display()
        )
    })?;

    if !resolved_path.exists() {
        bail!("Package path not found at '{}'", resolved_path.display());
    }

    if resolved_path.is_dir() {
        let manifest_path =
            solix_common::project::find_manifest_path(&resolved_path).ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot install project at '{}': manifest.yaml (or manifest.toml) not found",
                    resolved_path.display()
                )
            })?;

        let manifest = ProjectManifest::from_file(&manifest_path)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let package_name = manifest.configuration.name.clone();
        let profile = args.resolved_profile();

        execute_build(BuildArgs {
            path: resolved_path.clone(),
            platform: None,
            profile: Some(profile.to_string()),
            debug: false,
            release: false,
            test: false,
            pkg_dir: args.pkg_dir.clone(),
            install: false,
            force: args.force,
            check_env: false,
            target: None,
        })?;

        let candidate_slxp = resolved_path
            .join("build")
            .join(profile)
            .join(format!("{package_name}.slxp"));
        let candidate_archive = resolved_path
            .join("build")
            .join(profile)
            .join(format!("{package_name}.slxarc"));
        let archive_path = if candidate_slxp.exists() {
            candidate_slxp
        } else if candidate_archive.exists() {
            candidate_archive
        } else {
            let alt_slxp = resolved_path
                .join("build")
                .join(format!("{package_name}.slxp"));
            let alt_candidate = resolved_path
                .join("build")
                .join(format!("{package_name}.slxarc"));
            if alt_slxp.exists() {
                alt_slxp
            } else if alt_candidate.exists() {
                alt_candidate
            } else {
                bail!(
                    "Build succeeded but expected package archive was not found at '{}'",
                    candidate_slxp.display()
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

fn matches_installed_store_folder(
    folder_name: &str,
    target_package: &str,
    version_filter: Option<&str>,
    profile_filter: Option<&str>,
) -> bool {
    if folder_name == target_package && version_filter.is_none() && profile_filter.is_none() {
        return true;
    }

    if let Some((name_part, version_part)) = folder_name.split_once('@') {
        if let Some(target_version) = version_filter
            && version_part != target_version
        {
            return false;
        }

        let (package_candidate, candidate_profile) =
            if let Some(base_name) = name_part.strip_suffix("-debug") {
                (base_name, Some("debug"))
            } else if let Some(base_name) = name_part.strip_suffix("-release") {
                (base_name, Some("release"))
            } else if let Some(base_name) = name_part.strip_suffix("-test") {
                (base_name, Some("test"))
            } else {
                (name_part, None)
            };

        if package_candidate != target_package {
            return false;
        }

        if let Some(target_profile) = profile_filter {
            candidate_profile == Some(target_profile)
        } else {
            true
        }
    } else {
        false
    }
}

fn matches_native_cache_folder(
    folder_name: &str,
    target_package: &str,
    version_filter: Option<&str>,
    profile_filter: Option<&str>,
) -> bool {
    if matches_installed_store_folder(folder_name, target_package, version_filter, profile_filter) {
        return true;
    }

    if let Some((name_part, version_part)) = folder_name.rsplit_once('-') {
        if name_part != target_package {
            return false;
        }
        if let Some(target_version) = version_filter {
            version_part == target_version
        } else {
            profile_filter.is_none()
        }
    } else {
        false
    }
}

/// Uninstalls a package with exact matching.
///
/// # Errors
/// Returns an error if the package is not installed or deletion fails.
pub fn execute_uninstall_package(args: UninstallPackageArgs) -> Result<()> {
    let specification = args.spec.trim();
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let native_cache_directory = resolve_native_cache_dir(&store_directory);
    let mut registry = PackageRegistry::load_or_sync(&store_directory)?;

    let (package_name, profile_filter, spec_version_part) = parse_package_spec(specification);
    let version_target_ref = spec_version_part.as_deref();
    let profile_target_ref = profile_filter.as_deref();

    let removed_records = registry.unregister_matching(
        &package_name,
        version_target_ref,
        profile_target_ref,
        Some("package"),
    )?;

    let mut any_folder_removed = false;
    if store_directory.exists()
        && let Ok(store_entries) = fs::read_dir(&store_directory)
    {
        for entry_result in store_entries.flatten() {
            let entry_path = entry_result.path();
            let entry_name = entry_result.file_name().to_string_lossy().to_string();

            let should_remove = matches_installed_store_folder(
                &entry_name,
                &package_name,
                version_target_ref,
                profile_target_ref,
            );

            if should_remove && entry_path.exists() {
                let _ = fs::remove_dir_all(&entry_path);
                any_folder_removed = true;
            }
        }
    }

    let legacy_package_folder = store_directory.join(&package_name);
    if legacy_package_folder.exists() {
        if let Some(version_target) = version_target_ref {
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

    if native_cache_directory.exists()
        && let Ok(entries) = fs::read_dir(&native_cache_directory)
    {
        for entry_result in entries.flatten() {
            let folder_name = entry_result.file_name().to_string_lossy().to_string();
            let should_remove = matches_native_cache_folder(
                &folder_name,
                &package_name,
                version_target_ref,
                profile_target_ref,
            );
            if should_remove {
                let _ = fs::remove_dir_all(entry_result.path());
            }
        }
    }

    if removed_records.is_empty() && !any_folder_removed {
        bail!("Package '{specification}' is not currently installed");
    }

    println!("Uninstalled package '{specification}' successfully.");
    Ok(())
}

/// Uninstalls a binary with exact matching.
///
/// # Errors
/// Returns an error if the binary is not installed or deletion fails.
pub fn execute_uninstall_binaries(args: UninstallBinariesArgs) -> Result<()> {
    let specification = args.spec.trim();
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let bin_store_dir = resolve_bin_store_dir(args.pkg_dir.as_deref());
    let mut registry = PackageRegistry::load_or_sync(&store_directory)?;

    let (binary_name, profile_filter, spec_version_part) = parse_package_spec(specification);

    let removed_records = registry.unregister_matching(
        &binary_name,
        spec_version_part.as_deref(),
        profile_filter.as_deref(),
        Some("binaries"),
    )?;

    let binary_file = bin_store_dir.join(format!("{binary_name}.slxb"));
    let mut file_removed = false;
    if binary_file.exists() {
        let _ = fs::remove_file(&binary_file);
        file_removed = true;
    }

    if removed_records.is_empty() && !file_removed {
        bail!(
            "Binary '{specification}' is not currently installed in '{}'",
            bin_store_dir.display()
        );
    }

    println!("Uninstalled binary '{specification}' successfully.");
    Ok(())
}

/// Uninstalls a package version or an entire package from the local package store.
///
/// # Errors
/// Returns an error if the package is not installed or filesystem deletion fails.
pub fn execute_uninstall(args: UninstallArgs) -> Result<()> {
    if let Some(subcmd) = args.subcommand {
        return match subcmd {
            UninstallSubcommand::Package(package_args) => execute_uninstall_package(package_args),
            UninstallSubcommand::Binaries(binaries_args) => {
                execute_uninstall_binaries(binaries_args)
            }
        };
    }

    let Some(specification_raw) = args.spec else {
        bail!(
            "Missing package or binary name to uninstall. Usage: `solix uninstall package <NAME>` or `solix uninstall binaries <NAME>`"
        );
    };
    let specification = specification_raw.trim();
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());
    let native_cache_directory = resolve_native_cache_dir(&store_directory);
    let mut registry = PackageRegistry::load_or_sync(&store_directory)?;

    let (package_name, profile_filter, spec_version_part) = parse_package_spec(specification);
    let version_target_ref = spec_version_part.as_deref();
    let profile_target_ref = profile_filter.as_deref();

    let removed_records =
        registry.unregister(&package_name, version_target_ref, profile_target_ref)?;

    let mut any_folder_removed = false;
    if store_directory.exists()
        && let Ok(store_entries) = fs::read_dir(&store_directory)
    {
        for entry_result in store_entries.flatten() {
            let entry_path = entry_result.path();
            let entry_name = entry_result.file_name().to_string_lossy().to_string();

            let should_remove = matches_installed_store_folder(
                &entry_name,
                &package_name,
                version_target_ref,
                profile_target_ref,
            );

            if should_remove && entry_path.exists() {
                let _ = fs::remove_dir_all(&entry_path);
                any_folder_removed = true;
            }
        }
    }

    // Also check legacy nested structure: store_directory/<name>/<version>
    let legacy_package_folder = store_directory.join(&package_name);
    if legacy_package_folder.exists() {
        if let Some(version_target) = version_target_ref {
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
            let should_remove = matches_native_cache_folder(
                &folder_name,
                &package_name,
                version_target_ref,
                profile_target_ref,
            );
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
    if let Err(bootstrap_error) = crate::assets::ensure_bundled_packages_installed(&store_directory)
    {
        eprintln!("Warning: Failed to auto-bootstrap bundled packages: {bootstrap_error}");
    }
    let registry = PackageRegistry::load_or_sync(&store_directory)?;

    let mut filtered_packages: Vec<PackageRecord> = registry.packages;

    for package in &mut filtered_packages {
        if package.kind.eq_ignore_ascii_case("slxb")
            || package.kind.eq_ignore_ascii_case("binaries")
        {
            package.kind = "binaries".to_string();
        } else {
            package.kind = "package".to_string();
        }
    }

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
        let normalized_filter = if kind_filter.eq_ignore_ascii_case("binaries")
            || kind_filter.eq_ignore_ascii_case("slxb")
        {
            "binaries"
        } else {
            "package"
        };
        filtered_packages
            .retain(|package_record| package_record.kind.eq_ignore_ascii_case(normalized_filter));
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
                "{:<16} {:<10} {:<8} {:<8} {:<28} {:<10} {:<20} {:<15} SOURCE",
                "NAME", "VERSION", "KIND", "PROFILE", "PLATFORMS", "SIZE", "INSTALLED AT", "SHA256"
            );
            let _ = writeln!(
                output_content,
                "{:-<16} {:-<10} {:-<8} {:-<8} {:-<28} {:-<10} {:-<20} {:-<15} {:-<20}",
                "", "", "", "", "", "", "", "", ""
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
                let raw_platforms = if package.platforms.is_empty() {
                    "universal".to_string()
                } else {
                    package.platforms.join(", ")
                };
                let platforms_display = if raw_platforms.len() > 28 {
                    format!("{}...", &raw_platforms[..25])
                } else {
                    raw_platforms
                };

                let _ = writeln!(
                    output_content,
                    "{:<16} {:<10} {:<8} {:<8} {:<28} {:<10} {:<20} {:<15} {}",
                    package.name,
                    package.version,
                    package.kind,
                    package.build_profile,
                    platforms_display,
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

/// Discovers supported editor CLIs (`code`, `codium`, `cursor`) and installs the Solix extension.
///
/// # Errors
/// Returns an error if the bundled `.vsix` cannot be located or installation fails.
pub fn execute_install_vscode() -> Result<()> {
    let extension_path = crate::assets::resolve_bundled_vscode_extension_path().ok_or_else(|| {
        anyhow::anyhow!(
            "Could not locate bundled Solix VS Code extension (.vsix) on disk or embedded in this binary.\n\
             Please ensure 'assets/vscode/solix-language.vsix' exists or set SOLIX_VSCODE_VSIX_PATH."
        )
    })?;

    println!(
        "Found Solix VS Code extension at '{}'",
        extension_path.display()
    );

    let editor_binaries = ["code", "codium", "cursor"];
    let mut installed_count: usize = 0;
    let mut discovered_editors: Vec<PathBuf> = Vec::new();

    for editor_name in editor_binaries {
        if let Some(editor_path) = find_executable_in_path(editor_name) {
            discovered_editors.push(editor_path);
        }
    }

    if discovered_editors.is_empty() {
        println!(
            "\nNo supported editor command-line tool ('code', 'codium', 'cursor') found in PATH.\n\
             \nTo install manually into Visual Studio Code / VSCodium / Cursor:\n\
             1. Open your editor.\n\
             2. Press Ctrl+Shift+P (or Cmd+Shift+P on macOS).\n\
             3. Select 'Extensions: Install from VSIX...'\n\
             4. Choose the extension file:\n\
                {}\n",
            extension_path.display()
        );
        return Ok(());
    }

    for editor_executable in &discovered_editors {
        let editor_display = editor_executable
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("editor");

        println!("Installing extension using '{editor_display}'...");
        let mut install_command = crate::process::create_process_command(editor_executable);
        let installation_status = install_command
            .arg("--install-extension")
            .arg(&extension_path)
            .status();

        match installation_status {
            Ok(status) if status.success() => {
                println!("Successfully installed Solix extension into '{editor_display}'!");
                installed_count += 1;
            }
            Ok(status) => {
                eprintln!(
                    "Warning: '{editor_display} --install-extension' exited with code {:?}",
                    status.code()
                );
            }
            Err(error_details) => {
                eprintln!("Warning: Failed to execute '{editor_display}': {error_details}");
            }
        }
    }

    if installed_count > 0 {
        println!(
            "\nExtension installation complete. Restart your editor or run 'Developer: Reload Window' to activate."
        );
    }

    Ok(())
}

use crate::process::find_executable_in_path;
