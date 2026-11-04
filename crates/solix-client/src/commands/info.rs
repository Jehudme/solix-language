//! The `solix info` command driver.

#![forbid(unsafe_code)]

use crate::cli::InfoArgs;
use anyhow::{Context, Result};
use solix_common::bytecode::{BinaryProgram, extract_api_manifest};
use solix_compilation::manifest_to_source;
use solix_compilation::phases::emitter::disassemble_program;
use std::fmt::Write as FmtWrite;
use std::fs;

/// Executes the `info` subcommand.
///
/// # Errors
/// Returns an error if reading the binary container or output writing fails.
pub fn execute_info(args: InfoArgs) -> Result<()> {
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
            manifest_to_source(&manifest_json).map_err(|e| anyhow::anyhow!("{e:?}"))?;
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
    for f in &manifest.exported_functions {
        let _ = writeln!(out, "  • {}", f.symbol);
    }

    let _ = writeln!(
        out,
        "Exported Classes:   {}",
        manifest.exported_classes.len()
    );
    for c in &manifest.exported_classes {
        let _ = writeln!(out, "  • class {}", c.name);
        for m in &c.methods {
            let _ = writeln!(out, "      fn {}", m.simple_name);
        }
    }

    let _ = writeln!(
        out,
        "Exported Statics:   {}",
        manifest.exported_statics.len()
    );
    for s in &manifest.exported_statics {
        let _ = writeln!(out, "  • {} (slot: {})", s.symbol, s.slot);
    }

    let _ = writeln!(
        out,
        "Generic Templates:  {}",
        manifest.generic_templates.len()
    );
    for t in &manifest.generic_templates {
        let _ = writeln!(out, "  • <template> {}", t.name);
    }

    out
}
