//! The `solix info` command driver.

#![forbid(unsafe_code)]

use crate::cache::FilesystemPackageCache;
use crate::cli::InfoArgs;
use crate::commands::clean::format_human_size;
use crate::registry::{PackageRecord, PackageRegistry};
use crate::solver::resolve_package_store_dir;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_common::cache::PackageCache;
use solix_common::hash::{sha256_file, sha256_hex};
use solix_common::project::SlxarcInspection;
use solix_common::solver::PackageVersion;
use solix_compilation::manifest_to_source;
use solix_compilation::phases::emitter::disassemble_program;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

/// Executes the `info` subcommand.
///
/// # Errors
/// Returns an error if reading the binary container, archive inspection, or output writing fails.
pub fn execute_info(args: InfoArgs) -> Result<()> {
    let store_directory = resolve_package_store_dir(args.pkg_dir.as_deref());

    // Resolve target: check filesystem first, then installed package registry
    let (target_path, resolved_record): (PathBuf, Option<PackageRecord>) = if args.target.exists() {
        let registry = PackageRegistry::load_or_sync(&store_directory).ok();
        let canonical_target = args
            .target
            .canonicalize()
            .unwrap_or_else(|_| args.target.clone());
        let record = registry.and_then(|reg| {
            reg.packages.into_iter().find(|pkg_record| {
                pkg_record
                    .artifact_path
                    .canonicalize()
                    .unwrap_or_else(|_| pkg_record.artifact_path.clone())
                    == canonical_target
            })
        });
        (args.target.clone(), record)
    } else {
        let registry = PackageRegistry::load_or_sync(&store_directory)?;
        let target_str = args.target.to_string_lossy();
        let found_record = registry.get(&target_str, None);

        if let Some(record) = found_record {
            (record.artifact_path.clone(), Some(record.clone()))
        } else {
            bail!(
                "Target '{}' was not found as a file or registered package in '{}'",
                args.target.display(),
                store_directory.display()
            );
        }
    };

    if target_path.is_dir() {
        return execute_project_dir_info(&target_path, &args);
    }

    let file_size_bytes = fs::metadata(&target_path)
        .map(|meta| meta.len())
        .unwrap_or(0);
    let formatted_file_size = format_human_size(file_size_bytes);

    let is_archive = target_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|ext_str| {
            ext_str.eq_ignore_ascii_case("slxp") || ext_str.eq_ignore_ascii_case("slxarc")
        })
        .unwrap_or(false);

    if is_archive {
        let inspection =
            solix_common::project::SlxpPackage::inspect(&target_path).with_context(|| {
                format!(
                    "Failed to inspect package container from '{}'",
                    target_path.display()
                )
            })?;

        let computed_sha256 = sha256_file(&target_path).unwrap_or_default();

        let detected_profile = resolved_record
            .as_ref()
            .map(|record| record.build_profile.clone())
            .filter(|profile_str| !profile_str.is_empty())
            .or_else(|| {
                inspection
                    .binaries
                    .iter()
                    .find(|(_, program)| !program.build_profile.is_empty())
                    .map(|(_, program)| program.build_profile.clone())
            })
            .unwrap_or_else(|| "unspecified".to_string());

        let source_directory = resolved_record
            .as_ref()
            .and_then(|record| record.source.as_ref())
            .map(|path| path.display().to_string());

        let mut bundled_platforms: Vec<String> = Vec::new();
        for entry in &inspection.file_entries {
            let clean_path = entry.path.trim_start_matches('/');
            let stripped = clean_path
                .strip_prefix("bin/")
                .or_else(|| clean_path.strip_prefix("lib/"));
            if let Some(rest) = stripped {
                let mut segments = rest.split('/');
                if let Some(target_platform) = segments.next()
                    && !target_platform.is_empty()
                    && (rest.contains('/') || entry.is_directory)
                {
                    let target_string = target_platform.to_string();
                    if !bundled_platforms.contains(&target_string) {
                        bundled_platforms.push(target_string);
                    }
                }
            }
        }
        bundled_platforms.sort();

        let total_exported_symbols: usize = inspection
            .binaries
            .iter()
            .map(|(_, program)| {
                program.export_manifest.exported_functions.len()
                    + program.export_manifest.exported_classes.len()
            })
            .sum();

        // Check authenticated extraction cache status
        let cache = FilesystemPackageCache::new();
        let cache_status = if let Some(manifest) = &inspection.manifest {
            if let Ok(parsed_version) = PackageVersion::parse(&manifest.configuration.version) {
                match cache.get_cached_binary(
                    &manifest.configuration.name,
                    parsed_version,
                    &target_path,
                ) {
                    Ok(Some(_)) => "Cached (verified SHA-256 match in ~/.solix/cache/)".to_string(),
                    Ok(None) => "Not cached (will extract on first compile)".to_string(),
                    Err(cache_err) => format!("Cache invalid: {cache_err}"),
                }
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        let content = if args.disasm {
            let mut disassembly = String::new();
            for (name, program) in &inspection.binaries {
                let _ = writeln!(disassembly, "=== Disassembly of '{name}' ===");
                disassembly.push_str(&disassemble_program(program));
                disassembly.push('\n');
            }
            disassembly
        } else if args.api {
            let mut reconstructed_apis = String::new();
            for (name, program) in &inspection.binaries {
                let api_source = if !program.export_manifest.api_header.is_empty() {
                    program.export_manifest.api_header.clone()
                } else {
                    let manifest_json = extract_api_manifest(program)
                        .with_context(|| format!("Failed to extract API manifest from '{name}'"))?;
                    let (_, fallback_source) = manifest_to_source(&manifest_json)
                        .map_err(|error| anyhow::anyhow!("{error:?}"))?;
                    fallback_source
                };
                let _ = writeln!(reconstructed_apis, "// === Interface API for '{name}' ===");
                reconstructed_apis.push_str(&api_source);
                reconstructed_apis.push('\n');
            }
            reconstructed_apis
        } else if args.json {
            let native_checksums = inspection
                .manifest
                .as_ref()
                .map(|manifest| manifest.native_library_checksums.clone())
                .unwrap_or_default();

            let json_value = serde_json::json!({
                "archive": target_path.display().to_string(),
                "file_size_bytes": file_size_bytes,
                "file_size_formatted": formatted_file_size,
                "build_profile": detected_profile,
                "source_directory": source_directory,
                "bundled_platforms": bundled_platforms,
                "supported_platforms": inspection.manifest.as_ref().map(|manifest| manifest.supported_platforms.clone()).unwrap_or_default(),
                "exported_symbols_count": total_exported_symbols,
                "sha256": computed_sha256,
                "cache_status": cache_status,
                "manifest": inspection.manifest,
                "native_library_checksums": native_checksums,
                "contained_files": inspection.file_entries.iter().map(|entry| {
                    serde_json::json!({
                        "path": entry.path,
                        "uncompressed_size": entry.uncompressed_size,
                        "compressed_size": entry.compressed_size,
                        "is_directory": entry.is_directory,
                    })
                }).collect::<Vec<_>>(),
                "binaries": inspection.binaries.iter().map(|(name, program)| {
                    serde_json::json!({
                        "path": name,
                        "package_name": program.package_name,
                        "package_version": program.package_version,
                        "build_profile": program.build_profile,
                        "sha256": program.compute_sha256(),
                        "functions_count": program.functions.len(),
                        "classes_count": program.classes.len(),
                        "static_space_size": program.static_space_size,
                    })
                }).collect::<Vec<_>>(),
                "native_libraries": inspection.native_libraries,
            });
            serde_json::to_string_pretty(&json_value)?
        } else {
            format_slxarc_info_summary(
                &target_path,
                &inspection,
                &computed_sha256,
                &cache_status,
                file_size_bytes,
                &formatted_file_size,
                &detected_profile,
                source_directory.as_deref(),
                &bundled_platforms,
                total_exported_symbols,
            )
        };

        if let Some(out_path) = &args.output {
            if let Some(parent) = out_path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            fs::write(out_path, content)?;
            println!("Info written to {}", out_path.display());
        } else {
            println!("{content}");
        }

        return Ok(());
    }

    let bytes = fs::read(&target_path).with_context(|| {
        format!(
            "Failed to read binary container from '{}'",
            target_path.display()
        )
    })?;
    let computed_sha256 = sha256_hex(&bytes);
    let program = BinaryProgram::from_bytes(&bytes).with_context(|| {
        format!(
            "Failed to decode binary container from '{}'",
            target_path.display()
        )
    })?;

    let detected_profile = resolved_record
        .as_ref()
        .map(|record| record.build_profile.clone())
        .filter(|profile_str| !profile_str.is_empty())
        .unwrap_or_else(|| {
            if !program.build_profile.is_empty() {
                program.build_profile.clone()
            } else {
                "unspecified".to_string()
            }
        });

    let source_directory = resolved_record
        .as_ref()
        .and_then(|record| record.source.as_ref())
        .map(|path| path.display().to_string());

    let total_exported_symbols = program.export_manifest.exported_functions.len()
        + program.export_manifest.exported_classes.len();

    let content = if args.disasm {
        disassemble_program(&program)
    } else if args.api {
        if !program.export_manifest.api_header.is_empty() {
            program.export_manifest.api_header.clone()
        } else {
            let manifest_json = extract_api_manifest(&program)
                .with_context(|| "Failed to extract API manifest from binary")?;
            let (_, api_source) =
                manifest_to_source(&manifest_json).map_err(|error| anyhow::anyhow!("{error:?}"))?;
            api_source
        }
    } else if args.json {
        let json_value = serde_json::json!({
            "target": target_path.display().to_string(),
            "kind": "slxb",
            "file_size_bytes": file_size_bytes,
            "file_size_formatted": formatted_file_size,
            "build_profile": detected_profile,
            "source_directory": source_directory,
            "exported_symbols_count": total_exported_symbols,
            "sha256": computed_sha256,
            "package_name": program.package_name,
            "package_version": program.package_version,
            "description": program.package_description,
            "static_space_size": program.static_space_size,
            "functions_count": program.functions.len(),
            "classes_count": program.classes.len(),
            "native_library_checksums": program.native_library_checksums,
            "export_manifest": program.export_manifest,
        });
        serde_json::to_string_pretty(&json_value)?
    } else {
        format_info_summary(
            &target_path,
            &program,
            &computed_sha256,
            file_size_bytes,
            &formatted_file_size,
            &detected_profile,
            source_directory.as_deref(),
            total_exported_symbols,
        )
    };

    if let Some(out_path) = &args.output {
        if let Some(parent) = out_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(out_path, content)?;
        println!("Info written to {}", out_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn format_slxarc_info_summary(
    archive_path: &Path,
    inspection: &SlxarcInspection,
    sha256_digest: &str,
    cache_status: &str,
    file_size_bytes: u64,
    formatted_file_size: &str,
    build_profile: &str,
    source_directory: Option<&str>,
    bundled_platforms: &[String],
    total_exported_symbols: usize,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Solix Archive Container Inspection (.slxarc) ===");
    let _ = writeln!(out, "Archive:          {}", archive_path.display());
    let _ = writeln!(
        out,
        "File Size:        {formatted_file_size} ({file_size_bytes} bytes)"
    );
    let _ = writeln!(out, "SHA-256:          {sha256_digest}");
    let _ = writeln!(out, "Build Profile:    {build_profile}");
    if let Some(source_path) = source_directory {
        let _ = writeln!(out, "Source Dir:       {source_path}");
    }
    if !bundled_platforms.is_empty() {
        let _ = writeln!(out, "Platforms:        {}", bundled_platforms.join(", "));
    }
    let _ = writeln!(out, "Exported Symbols: {total_exported_symbols}");
    let _ = writeln!(out, "Cache Status:     {cache_status}");

    if let Some(manifest) = &inspection.manifest {
        let _ = writeln!(
            out,
            "Package:          {} v{}",
            manifest.configuration.name, manifest.configuration.version
        );
        if let Some(description) = &manifest.configuration.description {
            let _ = writeln!(out, "Description:      {description}");
        }
        if let Some(license) = &manifest.configuration.license {
            let _ = writeln!(out, "License:          {license}");
        }
        if !manifest.configuration.authors.is_empty() {
            let _ = writeln!(
                out,
                "Authors:          {}",
                manifest.configuration.authors.join(", ")
            );
        }
        if !manifest.supported_platforms.is_empty() {
            let _ = writeln!(
                out,
                "Supported Targets: {}",
                manifest.supported_platforms.join(", ")
            );
        }
        let _ = writeln!(out, "\n--- Entry Points ---");
        let _ = writeln!(
            out,
            "  • main: {}",
            manifest.main_entrypoint().as_deref().unwrap_or("<none>")
        );
        let _ = writeln!(
            out,
            "  • test: {}",
            manifest.test_entrypoint().as_deref().unwrap_or("<none>")
        );

        if !manifest.native_library_checksums.is_empty() {
            let _ = writeln!(
                out,
                "\n--- Native Shared Libraries ({}) ---",
                manifest.native_library_checksums.len()
            );
            for (lib_path, digest) in &manifest.native_library_checksums {
                let _ = writeln!(out, "  • {lib_path} [SHA-256: {digest}]");
            }
        }
    } else {
        let _ = writeln!(out, "Manifest:         <none>");
    }

    let _ = writeln!(
        out,
        "\n--- Contained Files ({}) ---",
        inspection.file_entries.len()
    );
    for entry in &inspection.file_entries {
        if entry.is_directory {
            let _ = writeln!(out, "  • {}/ [dir]", entry.path.trim_end_matches('/'));
        } else {
            let _ = writeln!(
                out,
                "  • {} (size: {}B, compressed: {}B)",
                entry.path, entry.uncompressed_size, entry.compressed_size
            );
        }
    }

    let _ = writeln!(
        out,
        "\n--- Compiled Binaries ({}) ---",
        inspection.binaries.len()
    );
    for (name, program) in &inspection.binaries {
        let profile_str = if !program.build_profile.is_empty() {
            &program.build_profile
        } else {
            "unspecified"
        };
        let _ = writeln!(
            out,
            "  • {} (profile: {}, functions: {}, classes: {}, static: {}B, SHA-256: {})",
            name,
            profile_str,
            program.functions.len(),
            program.classes.len(),
            program.static_space_size,
            program.compute_sha256()
        );
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn format_info_summary(
    binary_path: &Path,
    program: &BinaryProgram,
    sha256_digest: &str,
    file_size_bytes: u64,
    formatted_file_size: &str,
    build_profile: &str,
    source_directory: Option<&str>,
    total_exported_symbols: usize,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Solix Binary Container Inspection (.slxb) ===");
    let _ = writeln!(out, "Binary:           {}", binary_path.display());
    let _ = writeln!(
        out,
        "File Size:        {formatted_file_size} ({file_size_bytes} bytes)"
    );
    let _ = writeln!(out, "SHA-256:          {sha256_digest}");

    let pkg_name = if program.package_name.is_empty() {
        "<unnamed>"
    } else {
        &program.package_name
    };
    let pkg_ver = if program.package_version.is_empty() {
        "0.1.0"
    } else {
        &program.package_version
    };

    let _ = writeln!(out, "Package:          {pkg_name} v{pkg_ver}");
    let _ = writeln!(out, "Build Profile:    {build_profile}");
    if let Some(source_path) = source_directory {
        let _ = writeln!(out, "Source Dir:       {source_path}");
    }
    let _ = writeln!(out, "Exported Symbols: {total_exported_symbols}");
    if !program.package_description.is_empty() {
        let _ = writeln!(out, "Description:      {}", program.package_description);
    }

    let _ = writeln!(out, "Static Space:     {} bytes", program.static_space_size);

    if !program.native_library_checksums.is_empty() {
        let _ = writeln!(
            out,
            "\n--- Native Shared Libraries ({}) ---",
            program.native_library_checksums.len()
        );
        for (lib_path, digest) in &program.native_library_checksums {
            let _ = writeln!(out, "  • {lib_path} [SHA-256: {digest}]");
        }
    }

    let _ = writeln!(
        out,
        "\n--- Declared Classes ({}) ---",
        program.classes.len()
    );
    for class in &program.classes {
        let _ = writeln!(
            out,
            "  • {} (instance_size: {}B, vtable_slots: {})",
            class.symbol,
            class.instance_size,
            class.vtable.len()
        );
    }

    let _ = writeln!(
        out,
        "\n--- Compiled Functions ({}) ---",
        program.functions.len()
    );
    for function in &program.functions {
        let _ = writeln!(
            out,
            "  • {} (params: {}, registers: {}, instructions: {})",
            function.symbol,
            function.param_count,
            function.num_registers,
            function.instructions.len()
        );
    }

    let _ = writeln!(
        out,
        "\n--- Export Manifest (Exported Symbols: {}) ---",
        program.export_manifest.exported_functions.len()
            + program.export_manifest.exported_classes.len()
    );
    for function in &program.export_manifest.exported_functions {
        let _ = writeln!(
            out,
            "  • [func] {} (params: {}, return: {})",
            function.symbol,
            function.parameters.len(),
            function.return_type
        );
    }
    for class in &program.export_manifest.exported_classes {
        let _ = writeln!(out, "  • [class] {}", class.symbol);
    }

    out
}

fn execute_project_dir_info(dir_path: &Path, args: &InfoArgs) -> Result<()> {
    let directory_name = dir_path.file_name().map(|name| name.to_string_lossy());
    let is_build_or_stage_target = directory_name.as_ref().is_some_and(|name| {
        name.eq_ignore_ascii_case("build") || name.eq_ignore_ascii_case("stage")
    });
    if is_build_or_stage_target {
        bail!(
            "'solix info' cannot be run on build or staging directories ('{}'). Run 'solix info' on the project root directory or on a compiled package container (.slxp, .slxarc, .slxb).",
            dir_path.display()
        );
    }

    let manifest_path = solix_common::project::find_manifest_path(dir_path).ok_or_else(|| {
        anyhow::anyhow!(
            "Target directory '{}' does not contain a Solix project manifest (manifest.yaml / manifest.toml)",
            dir_path.display()
        )
    })?;

    let manifest =
        solix_common::project::ProjectManifest::from_file(&manifest_path).with_context(|| {
            format!(
                "Failed to load project manifest at '{}'",
                manifest_path.display()
            )
        })?;

    let build_dir = dir_path.join("build");
    let mut built_artifacts: Vec<serde_json::Value> = Vec::new();
    let mut artifact_summaries: Vec<String> = Vec::new();
    if build_dir.is_dir() {
        for profile_name in ["release", "debug", "test"] {
            let profile_dir = build_dir.join(profile_name);
            if !profile_dir.is_dir() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&profile_dir) {
                for entry_result in entries.flatten() {
                    let entry_path = entry_result.path();
                    let extension = entry_path
                        .extension()
                        .and_then(|ext_val| ext_val.to_str())
                        .unwrap_or("");
                    if extension.eq_ignore_ascii_case("slxp")
                        || extension.eq_ignore_ascii_case("slxarc")
                        || extension.eq_ignore_ascii_case("slxb")
                    {
                        let size_bytes = fs::metadata(&entry_path)
                            .map(|metadata_val| metadata_val.len())
                            .unwrap_or(0);
                        let sha256_digest = sha256_file(&entry_path).unwrap_or_default();
                        let inspection =
                            solix_common::project::SlxpPackage::inspect(&entry_path).ok();
                        let supported = inspection
                            .as_ref()
                            .and_then(|insp| insp.manifest.as_ref())
                            .map(|sub_manifest| sub_manifest.supported_platforms.clone())
                            .unwrap_or_default();
                        built_artifacts.push(serde_json::json!({
                            "path": entry_path.display().to_string(),
                            "profile": profile_name,
                            "kind": extension,
                            "size_bytes": size_bytes,
                            "size_formatted": format_human_size(size_bytes),
                            "sha256": sha256_digest,
                            "supported_platforms": supported,
                        }));
                        let rel_path = entry_path
                            .strip_prefix(dir_path)
                            .unwrap_or(&entry_path)
                            .display();
                        let plat_desc = if supported.is_empty() {
                            "universal".to_string()
                        } else {
                            supported.join(", ")
                        };
                        artifact_summaries.push(format!(
                            "  • {} ({}, profile: {}, platforms: {})",
                            rel_path,
                            format_human_size(size_bytes),
                            profile_name,
                            plat_desc
                        ));
                    }
                }
            }
        }
    }

    let content = if args.json {
        let json_val = serde_json::json!({
            "target": dir_path.display().to_string(),
            "kind": "project",
            "manifest_path": manifest_path.display().to_string(),
            "configuration": manifest.configuration,
            "profiles": manifest.profiles,
            "supported_platforms": manifest.supported_platforms,
            "built_artifacts": built_artifacts,
        });
        serde_json::to_string_pretty(&json_val)?
    } else {
        let mut out = String::new();
        let _ = writeln!(out, "=== Solix Project Inspection ===");
        let _ = writeln!(
            out,
            "Project:          {} v{}",
            manifest.configuration.name, manifest.configuration.version
        );
        let _ = writeln!(out, "Directory:        {}", dir_path.display());
        let _ = writeln!(out, "Manifest:         {}", manifest_path.display());
        if let Some(ref description) = manifest.configuration.description {
            let _ = writeln!(out, "Description:      {description}");
        }
        if let Some(ref license) = manifest.configuration.license {
            let _ = writeln!(out, "License:          {license}");
        }
        if !manifest.configuration.authors.is_empty() {
            let _ = writeln!(
                out,
                "Authors:          {}",
                manifest.configuration.authors.join(", ")
            );
        }
        let _ = writeln!(out, "\n--- Profiles ({}) ---", manifest.profiles.len());
        for profile_name in manifest.profiles.keys() {
            if let Ok(resolved) = manifest.resolve_profile(profile_name) {
                let _ = writeln!(out, "  • {profile_name}:");
                let _ = writeln!(out, "      Optimization: Level {}", resolved.opt_level);
                let _ = writeln!(
                    out,
                    "      Inlining:     Threshold {}",
                    resolved.inline_threshold
                );
                let expanded_targets =
                    solix_common::project::expand_platform_list(&resolved.platforms);
                let plat_str = if resolved.platforms.is_empty() {
                    "universal".to_string()
                } else {
                    format!(
                        "{} (expanded: {})",
                        resolved.platforms.join(", "),
                        expanded_targets.join(", ")
                    )
                };
                let _ = writeln!(out, "      Platforms:    {plat_str}");
                if let Some(ref entry) = resolved.entrypoint {
                    let _ = writeln!(out, "      Entrypoint:   {entry}");
                }
                let _ = writeln!(out, "      Static:       {}", resolved.is_static);
                if !resolved.sources.is_empty() {
                    let _ = writeln!(out, "      Sources:      {}", resolved.sources.join(", "));
                }
                if !resolved.dependencies.is_empty() {
                    let dep_strs: Vec<String> = resolved
                        .dependencies
                        .iter()
                        .map(|(dep_name, dep_spec)| {
                            format!("{dep_name}@{} ({})", dep_spec.version, dep_spec.profile)
                        })
                        .collect();
                    let _ = writeln!(out, "      Dependencies: {}", dep_strs.join(", "));
                }
                if !resolved.natives.is_empty() {
                    let native_strs: Vec<String> = resolved.natives.keys().cloned().collect();
                    let _ = writeln!(out, "      Natives:      {}", native_strs.join(", "));
                }
            }
        }

        if !artifact_summaries.is_empty() {
            let _ = writeln!(
                out,
                "\n--- Built Artifacts ({}) ---",
                artifact_summaries.len()
            );
            for summary_line in &artifact_summaries {
                let _ = writeln!(out, "{summary_line}");
            }
        }

        out
    };

    if let Some(ref out_path) = args.output {
        if let Some(parent) = out_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(out_path, content)?;
        println!("Info written to {}", out_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}
