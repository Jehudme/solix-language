//! Manifest-to-Source code generator transforming binary API manifests into compilable Solix interface sources.

#![forbid(unsafe_code)]

use solix_common::bytecode::manifest::{DeclarationModifier, ExportManifest};

fn append_modifiers(out: &mut String, modifiers: &[DeclarationModifier]) {
    let mut seen_modifiers = std::collections::HashSet::new();
    for modifier in modifiers {
        if *modifier != DeclarationModifier::Extern && seen_modifiers.insert(*modifier) {
            out.push_str(modifier.as_str());
            out.push(' ');
        }
    }
}

/// Converts an export manifest JSON string into compilable Solix source text and canonical file name.
///
/// Returns `Ok((source_name, source_code))` where `source_name` follows `<package_name>@<version>.slx`.
///
/// # Errors
/// Returns `serde_json::Error` if `manifest_json` cannot be deserialized into `ExportManifest`.
pub fn manifest_to_source(manifest_json: &str) -> Result<(String, String), serde_json::Error> {
    let manifest: ExportManifest = serde_json::from_str(manifest_json)?;
    let pkg_name = if manifest.package_name.is_empty() {
        "lib".to_string()
    } else {
        manifest.package_name.clone()
    };
    let pkg_version = if manifest.package_version.is_empty() {
        "0.1.0".to_string()
    } else {
        manifest.package_version.clone()
    };
    let source_name = format!("{pkg_name}@{pkg_version}.slx");

    let mut out = String::new();
    out.push_str(&format!(
        "// Auto-generated Solix API declarations for {pkg_name} v{pkg_version}\n"
    ));
    if !manifest.package_description.is_empty() {
        out.push_str(&format!("// {}\n", manifest.package_description));
    }
    out.push('\n');

    if !manifest.package_name.is_empty() {
        out.push_str(&format!("package {};\n\n", manifest.package_name));
    }

    // 1. Generic templates (preserved source code for monomorphization)
    for template in &manifest.generic_templates {
        out.push_str(&template.template_source);
        out.push_str("\n\n");
    }

    // 2. Exported static variables
    for stat in &manifest.exported_statics {
        out.push_str("extern ");
        append_modifiers(&mut out, &stat.modifiers);
        out.push_str(&format!("{} {};\n", stat.var_type, stat.name));
    }
    if !manifest.exported_statics.is_empty() {
        out.push('\n');
    }

    // Exported enums
    for exported_enum in &manifest.exported_enums {
        append_modifiers(&mut out, &exported_enum.modifiers);
        out.push_str(&format!("enum {} {{\n", exported_enum.name));
        for (variant_index, (variant_name, discriminant_option)) in
            exported_enum.variants.iter().enumerate()
        {
            out.push_str("    ");
            out.push_str(variant_name);
            if let Some(discriminant) = discriminant_option {
                out.push_str(&format!(" = {discriminant}"));
            }
            if variant_index + 1 < exported_enum.variants.len() {
                out.push_str(",\n");
            } else {
                out.push('\n');
            }
        }
        out.push_str("}\n\n");
    }

    // 3. Exported top-level functions
    for func in &manifest.exported_functions {
        out.push_str("extern ");
        append_modifiers(&mut out, &func.modifiers);
        out.push_str(&format!("{} {}(", func.return_type, func.simple_name));
        for (param_index, param) in func.parameters.iter().enumerate() {
            if param_index > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{} {}", param.param_type, param.name));
        }
        out.push_str(");\n");
    }
    if !manifest.exported_functions.is_empty() {
        out.push('\n');
    }

    // 4. Exported classes and interfaces
    for cls in &manifest.exported_classes {
        out.push_str("extern ");
        append_modifiers(&mut out, &cls.modifiers);
        if cls.is_interface {
            out.push_str(&format!("interface {}", cls.name));
        } else {
            out.push_str(&format!("class {}", cls.name));
        }
        if let Some(base) = &cls.base_class {
            out.push_str(&format!(" extends {base}"));
        }
        if !cls.interfaces.is_empty() {
            out.push_str(" implements ");
            out.push_str(&cls.interfaces.join(", "));
        }
        out.push_str(" {\n");

        // Fields
        for field in &cls.fields {
            out.push_str("    ");
            append_modifiers(&mut out, &field.modifiers);
            out.push_str(&format!("{} {};\n", field.field_type, field.name));
        }

        // Methods / constructors — destructors and private methods are skipped because:
        // - Destructors cannot have an `extern` bodiless declaration with an access modifier.
        // - Private methods are implementation details not callable from downstream code.
        for method in &cls.methods {
            // Skip destructors — they are managed by the VM and cannot be declared extern.
            if method.is_destructor {
                continue;
            }
            // Skip private methods — they must not appear in public API headers.
            if method.modifiers.contains(&DeclarationModifier::Private) {
                continue;
            }

            out.push_str("    ");
            append_modifiers(&mut out, &method.modifiers);
            if method.is_constructor {
                out.push_str(&format!("{}(", cls.name));
            } else if method.is_operator {
                let operator_name = if method.simple_name.starts_with("operator") {
                    method.simple_name.clone()
                } else {
                    format!("operator{}", method.simple_name)
                };
                out.push_str(&format!("{} {operator_name}(", method.return_type));
            } else {
                out.push_str(&format!("{} {}(", method.return_type, method.simple_name));
            }

            for (param_index, param) in method.parameters.iter().enumerate() {
                if param_index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("{} {}", param.param_type, param.name));
            }
            out.push_str(");\n");
        }

        out.push_str("}\n\n");
    }

    Ok((source_name, out))
}
