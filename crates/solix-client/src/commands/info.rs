//! The `solix info` command driver.

#![forbid(unsafe_code)]

use crate::cli::InfoArgs;
use anyhow::{Context, Result};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_common::project::{SlxarcArchive, SlxarcInspection};
use solix_compilation::manifest_to_source;
use solix_compilation::phases::emitter::disassemble_program;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

/// Executes the `info` subcommand.
///
/// # Errors
/// Returns an error if reading the binary container, archive inspection, or output writing fails.
pub fn execute_info(args: InfoArgs) -> Result<()> {
    let is_archive = args
        .binary
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|ext_str| ext_str.eq_ignore_ascii_case("slxarc"))
        .unwrap_or(false);

    if is_archive {
        let inspection = SlxarcArchive::inspect(&args.binary).with_context(|| {
            format!(
                "Failed to inspect .slxarc archive container from '{}'",
                args.binary.display()
            )
        })?;

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
                let manifest_json = extract_api_manifest(program)
                    .with_context(|| format!("Failed to extract API manifest from '{name}'"))?;
                let (_, api_source) = manifest_to_source(&manifest_json)
                    .map_err(|error| anyhow::anyhow!("{error:?}"))?;
                let _ = writeln!(reconstructed_apis, "// === Interface API for '{name}' ===");
                reconstructed_apis.push_str(&api_source);
                reconstructed_apis.push('\n');
            }
            reconstructed_apis
        } else if args.json {
            let json_value = serde_json::json!({
                "archive": args.binary.display().to_string(),
                "manifest": inspection.manifest,
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
                        "functions_count": program.functions.len(),
                        "classes_count": program.classes.len(),
                        "static_space_size": program.static_space_size,
                    })
                }).collect::<Vec<_>>(),
                "native_libraries": inspection.native_libraries,
            });
            serde_json::to_string_pretty(&json_value)?
        } else {
            format_slxarc_info_summary(&args.binary, &inspection)
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

    let bytes = fs::read(&args.binary).with_context(|| {
        format!(
            "Failed to read binary container from '{}'",
            args.binary.display()
        )
    })?;
    let program = BinaryProgram::from_bytes(&bytes).with_context(|| {
        format!(
            "Failed to decode binary container from '{}'",
            args.binary.display()
        )
    })?;

    let content = if args.disasm {
        disassemble_program(&program)
    } else if args.api {
        let manifest_json = extract_api_manifest(&program)
            .with_context(|| "Failed to extract API manifest from binary")?;
        let (_, api_source) =
            manifest_to_source(&manifest_json).map_err(|error| anyhow::anyhow!("{error:?}"))?;
        api_source
    } else if args.json {
        extract_api_manifest(&program)
            .with_context(|| "Failed to extract JSON manifest from binary")?
    } else {
        format_info_summary(&program)
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

fn format_slxarc_info_summary(archive_path: &Path, inspection: &SlxarcInspection) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Solix Archive Container Inspection (.slxarc) ===");
    let _ = writeln!(out, "Archive:     {}", archive_path.display());

    if let Some(manifest) = &inspection.manifest {
        let _ = writeln!(
            out,
            "Package:     {} v{}",
            manifest.info.name, manifest.info.version
        );
        if let Some(description) = &manifest.info.description {
            let _ = writeln!(out, "Description: {description}");
        }
        if let Some(license) = &manifest.info.license {
            let _ = writeln!(out, "License:     {license}");
        }
        if !manifest.info.authors.is_empty() {
            let _ = writeln!(out, "Authors:     {}", manifest.info.authors.join(", "));
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
    } else {
        let _ = writeln!(out, "Manifest:    <none>");
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

    if !inspection.binaries.is_empty() {
        let _ = writeln!(
            out,
            "\n--- Packaged Bytecode Binaries ({}) ---",
            inspection.binaries.len()
        );
        for (name, program) in &inspection.binaries {
            let _ = writeln!(
                out,
                "  • {}: {} functions, {} classes, {} static vars",
                name,
                program.functions.len(),
                program.classes.len(),
                program.static_space_size
            );
        }
    }

    if !inspection.native_libraries.is_empty() {
        let _ = writeln!(
            out,
            "\n--- Packaged Native Libraries ({}) ---",
            inspection.native_libraries.len()
        );
        for native_lib in &inspection.native_libraries {
            let _ = writeln!(out, "  • {native_lib}");
        }
    }

    out
}

fn format_info_summary(program: &BinaryProgram) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== Solix Binary Container Inspection ===");
    let _ = writeln!(
        out,
        "Package:     {} v{}",
        if program.package_name.is_empty() {
            "<unnamed>"
        } else {
            &program.package_name
        },
        program.package_version
    );
    if !program.package_description.is_empty() {
        let _ = writeln!(out, "Description: {}", program.package_description);
    }
    let _ = writeln!(out, "Functions:   {}", program.functions.len());
    let _ = writeln!(out, "Classes:     {}", program.classes.len());
    let _ = writeln!(out, "Static Vars: {}", program.static_space_size);
    let _ = writeln!(out, "Constants:   {}", program.constant_pool.len());

    let manifest = &program.export_manifest;
    let _ = writeln!(out, "\n--- Export Manifest ---");
    let _ = writeln!(
        out,
        "Exported Functions: {}",
        manifest.exported_functions.len()
    );
    for func in &manifest.exported_functions {
        let _ = writeln!(out, "  • {}", func.symbol);
    }

    let _ = writeln!(
        out,
        "Exported Classes:   {}",
        manifest.exported_classes.len()
    );
    for class_item in &manifest.exported_classes {
        let _ = writeln!(out, "  • class {}", class_item.name);
        for method in &class_item.methods {
            let _ = writeln!(out, "      fn {}", method.simple_name);
        }
    }

    let _ = writeln!(
        out,
        "Exported Statics:   {}",
        manifest.exported_statics.len()
    );
    for static_item in &manifest.exported_statics {
        let _ = writeln!(
            out,
            "  • {} (slot: {})",
            static_item.symbol, static_item.slot
        );
    }

    let _ = writeln!(
        out,
        "Generic Templates:  {}",
        manifest.generic_templates.len()
    );
    for template in &manifest.generic_templates {
        let _ = writeln!(out, "  • <template> {}", template.name);
    }

    out
}
