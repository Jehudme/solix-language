//! The `solix info` command driver.

#![forbid(unsafe_code)]

use crate::cache::FilesystemPackageCache;
use crate::cli::InfoArgs;
use crate::registry::PackageRegistry;
use crate::solver::resolve_package_store_dir;
use anyhow::{Context, Result, bail};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_common::cache::PackageCache;
use solix_common::hash::{sha256_file, sha256_hex};
use solix_common::project::{SlxarcArchive, SlxarcInspection};
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
    let target_path: PathBuf = if args.target.exists() {
        args.target.clone()
    } else {
        let registry = PackageRegistry::load_or_sync(&store_directory)?;
        let target_str = args.target.to_string_lossy();

        let resolved_record = if target_str.contains('@') {
            let mut parts = target_str.splitn(2, '@');
            let package_name = parts.next().unwrap_or("");
            let package_version = parts.next().unwrap_or("");
            registry.get(package_name, Some(package_version))
        } else {
            registry.get(&target_str, None)
        };

        if let Some(record) = resolved_record {
            record.artifact_path.clone()
        } else {
            bail!(
                "Target '{}' was not found as a file or registered package in '{}'",
                args.target.display(),
                store_directory.display()
            );
        }
    };

    let is_archive = target_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|ext_str| ext_str.eq_ignore_ascii_case("slxarc"))
        .unwrap_or(false);

    if is_archive {
        let inspection = SlxarcArchive::inspect(&target_path).with_context(|| {
            format!(
                "Failed to inspect .slxarc archive container from '{}'",
                target_path.display()
            )
        })?;

        let computed_sha256 = sha256_file(&target_path).unwrap_or_default();

        // Check authenticated extraction cache status
        let cache = FilesystemPackageCache::new();
        let cache_status = if let Some(manifest) = &inspection.manifest {
            if let Ok(parsed_version) = PackageVersion::parse(&manifest.info.version) {
                match cache.get_cached_binary(&manifest.info.name, parsed_version, &target_path) {
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
            "sha256": computed_sha256,
            "package_name": program.package_name,
            "package_version": program.package_version,
            "build_profile": program.build_profile,
            "description": program.package_description,
            "static_space_size": program.static_space_size,
            "functions_count": program.functions.len(),
            "classes_count": program.classes.len(),
            "native_library_checksums": program.native_library_checksums,
            "export_manifest": program.export_manifest,
        });
        serde_json::to_string_pretty(&json_value)?
    } else {
        format_info_summary(&target_path, &program, &computed_sha256)
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

fn format_slxarc_info_summary(
    archive_path: &Path,
    inspection: &SlxarcInspection,
    sha256_digest: &str,
    cache_status: &str,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Solix Archive Container Inspection (.slxarc) ===");
    let _ = writeln!(out, "Archive:      {}", archive_path.display());
    let _ = writeln!(out, "SHA-256:      {sha256_digest}");
    let _ = writeln!(out, "Cache Status: {cache_status}");

    if let Some(manifest) = &inspection.manifest {
        let _ = writeln!(
            out,
            "Package:      {} v{}",
            manifest.info.name, manifest.info.version
        );
        if let Some(description) = &manifest.info.description {
            let _ = writeln!(out, "Description:  {description}");
        }
        if let Some(license) = &manifest.info.license {
            let _ = writeln!(out, "License:      {license}");
        }
        if !manifest.info.authors.is_empty() {
            let _ = writeln!(out, "Authors:      {}", manifest.info.authors.join(", "));
        }
        let _ = writeln!(out, "\n--- Entry Points ---");
        let _ = writeln!(
            out,
            "  • main: {}",
            manifest.entrypoints.main.as_deref().unwrap_or("<none>")
        );
        let _ = writeln!(
            out,
            "  • test: {}",
            manifest.entrypoints.test.as_deref().unwrap_or("<none>")
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
        let _ = writeln!(out, "Manifest:     <none>");
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

fn format_info_summary(
    binary_path: &Path,
    program: &BinaryProgram,
    sha256_digest: &str,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Solix Binary Container Inspection (.slxb) ===");
    let _ = writeln!(out, "Binary:       {}", binary_path.display());
    let _ = writeln!(out, "SHA-256:      {sha256_digest}");

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
    let profile_str = if !program.build_profile.is_empty() {
        &program.build_profile
    } else {
        "unspecified"
    };

    let _ = writeln!(out, "Package:      {pkg_name} v{pkg_ver}");
    let _ = writeln!(out, "Profile:      {profile_str}");
    if !program.package_description.is_empty() {
        let _ = writeln!(out, "Description:  {}", program.package_description);
    }

    let _ = writeln!(
        out,
        "Static Space: {} bytes",
        program.static_space_size
    );

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

    let _ = writeln!(out, "\n--- Declared Classes ({}) ---", program.classes.len());
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
