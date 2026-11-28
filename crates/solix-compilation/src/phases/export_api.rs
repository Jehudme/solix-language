//! Generation of local-only, namespace-qualified API headers and pre-parsed AST binary payloads.
//!
//! Milestone 023 Pillar 2: Pre-generated API header (`api_header: String`) and serialized
//! AST bytes (`exported_ast_bytes: Vec<u8>`) enable sub-millisecond dependency ingestion
//! and eliminate multi-package namespace collisions.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::phases::binder::passes::pass1_symbol_discovery::compute_effective_package;
use crate::phases::lexing::Span;
use crate::phases::parser::statements::declarations::{
    ClassDeclaration, ClassMember, Declaration, EnumDeclaration, FieldDeclaration,
    GenericParameter, InterfaceDeclaration, MethodDeclaration, Modifier, Parameter,
};

fn get_span_text(span: Span, source: &str) -> &str {
    let start_index = span.start as usize;
    let end_index = span.end as usize;
    if start_index <= end_index && end_index <= source.len() {
        &source[start_index..end_index]
    } else {
        ""
    }
}

fn extract_package_path(spans: &[Span], source_text: &str) -> String {
    let segments: Vec<&str> = spans
        .iter()
        .map(|span| get_span_text(*span, source_text))
        .filter(|segment| !segment.is_empty())
        .collect();
    segments.join("::")
}

fn extract_namespace_path(spans: &[Span], source_text: &str) -> String {
    let segments: Vec<&str> = spans
        .iter()
        .map(|span| get_span_text(*span, source_text))
        .filter(|segment| !segment.is_empty())
        .collect();
    segments.join("::")
}

fn format_generic_parameters(parameters: &[GenericParameter], source_text: &str) -> String {
    if parameters.is_empty() {
        return String::new();
    }
    let formatted: Vec<String> = parameters
        .iter()
        .map(|parameter| {
            let parameter_name = get_span_text(parameter.name_span, source_text);
            if parameter.is_variadic {
                format!("{parameter_name}...")
            } else if !parameter.bounds.is_empty() {
                let bounds_str = parameter
                    .bounds
                    .iter()
                    .map(|bound| get_span_text(bound.span(), source_text))
                    .collect::<Vec<_>>()
                    .join(" + ");
                format!("{parameter_name} : {bounds_str}")
            } else {
                parameter_name.to_string()
            }
        })
        .collect();
    format!("<{}>", formatted.join(", "))
}

fn format_parameters(parameters: &[Parameter], source_text: &str) -> String {
    parameters
        .iter()
        .map(|parameter| {
            let type_string = get_span_text(parameter.param_type.span(), source_text);
            let name_string = get_span_text(parameter.name_span, source_text);
            if parameter.is_variadic {
                format!("{type_string}... {name_string}")
            } else {
                format!("{type_string} {name_string}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn append_declaration_modifiers(
    output_buffer: &mut String,
    modifiers: &[Modifier],
    allowed_modifiers: &[Modifier],
) {
    for allowed_modifier in allowed_modifiers {
        if modifiers.contains(allowed_modifier) {
            output_buffer.push_str(allowed_modifier.as_str());
            output_buffer.push(' ');
        }
    }
}

/// Generates a clean, namespace-qualified API header source string from local compilation sources.
pub fn generate_api_header(context: &CompilationContext) -> String {
    let mut header_output = String::new();
    let package_name = if !context.options.package_name.is_empty() {
        &context.options.package_name
    } else {
        "module"
    };
    let package_version = if !context.options.package_version.is_empty() {
        &context.options.package_version
    } else {
        "0.1.0"
    };

    header_output.push_str(&format!(
        "// Auto-generated Solix API declarations for {package_name} v{package_version}\n"
    ));
    if !context.options.package_description.is_empty() {
        header_output.push_str(&format!("// {}\n", context.options.package_description));
    }
    header_output.push('\n');
    let is_debug = context.options.profile == crate::CompilationContext::BuildProfile::Debug;

    for (source_index, ast_option) in context.ast.iter().enumerate() {
        let Ok(source_id) = u16::try_from(source_index) else {
            break;
        };
        if context.is_external_source(source_id) {
            continue;
        }

        let program_ast = match ast_option {
            Some(ast) => ast,
            None => continue,
        };
        let source_code = match context.sources.get(source_index) {
            Some(code) => code.as_str(),
            None => continue,
        };

        let file_package = if let Some(package_statement) = &program_ast.package {
            extract_package_path(&package_statement.path, source_code)
        } else if !context.options.package_name.is_empty() {
            context.options.package_name.replace('.', "::")
        } else {
            String::new()
        };

        // 1. Export public aliases
        for alias_statement in &program_ast.aliases {
            let decl_namespace =
                extract_namespace_path(&alias_statement.namespace_path, source_code);
            let effective_package = compute_effective_package(&file_package, &decl_namespace);
            let alias_name = get_span_text(alias_statement.name_span, source_code);
            let qualified_name = if effective_package.is_empty() {
                alias_name.to_string()
            } else {
                format!("{effective_package}::{alias_name}")
            };

            let generic_part = if !alias_statement.generic_params.is_empty() {
                let params = alias_statement
                    .generic_params
                    .iter()
                    .map(|param_span| get_span_text(*param_span, source_code))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("<{params}>")
            } else {
                String::new()
            };

            let target_type = get_span_text(alias_statement.target_type.span(), source_code);
            header_output.push_str(&format!(
                "alias {qualified_name}{generic_part} = {target_type};\n\n"
            ));
        }

        // 2. Export declarations
        for declaration in &program_ast.declarations {
            match declaration {
                Declaration::Class(class_declaration) => {
                    emit_class_header(
                        &mut header_output,
                        class_declaration,
                        &file_package,
                        source_code,
                        is_debug,
                    );
                }
                Declaration::Interface(interface_declaration) => {
                    emit_interface_header(
                        &mut header_output,
                        interface_declaration,
                        &file_package,
                        source_code,
                    );
                }
                Declaration::Enum(enum_declaration) => {
                    emit_enum_header(
                        &mut header_output,
                        enum_declaration,
                        &file_package,
                        source_code,
                    );
                }
                Declaration::Method(method_declaration) => {
                    emit_top_level_method(
                        &mut header_output,
                        method_declaration,
                        &file_package,
                        source_code,
                        is_debug,
                    );
                }
                Declaration::Field(field_declaration) => {
                    emit_top_level_field(
                        &mut header_output,
                        field_declaration,
                        &file_package,
                        source_code,
                        is_debug,
                    );
                }
                _ => {}
            }
        }
    }

    header_output
}

fn emit_class_header(
    output_buffer: &mut String,
    class_declaration: &ClassDeclaration,
    file_package: &str,
    source_code: &str,
    is_debug: bool,
) {
    if class_declaration.modifiers.contains(&Modifier::Extern) {
        return;
    }
    if !is_debug {
        if class_declaration.modifiers.contains(&Modifier::Private)
            || class_declaration.modifiers.contains(&Modifier::Internal)
        {
            return;
        }
        if !class_declaration.modifiers.contains(&Modifier::Public) {
            return;
        }
    }

    let decl_namespace = extract_namespace_path(&class_declaration.namespace_path, source_code);
    let effective_package = compute_effective_package(file_package, &decl_namespace);
    let class_name = get_span_text(class_declaration.name_span, source_code);
    let qualified_name = if effective_package.is_empty() {
        class_name.to_string()
    } else {
        format!("{effective_package}::{class_name}")
    };

    let is_generic_class = !class_declaration.generic_params.is_empty();
    let generics_string = format_generic_parameters(&class_declaration.generic_params, source_code);

    let super_clause = class_declaration
        .super_class
        .as_ref()
        .map(|super_type| format!(" extends {}", get_span_text(super_type.span(), source_code)))
        .unwrap_or_default();

    let implements_clause = if !class_declaration.interfaces.is_empty() {
        let interface_names = class_declaration
            .interfaces
            .iter()
            .map(|interface_type| get_span_text(interface_type.span(), source_code))
            .collect::<Vec<_>>()
            .join(", ");
        format!(" implements {interface_names}")
    } else {
        String::new()
    };

    output_buffer.push_str("extern ");
    append_declaration_modifiers(
        output_buffer,
        &class_declaration.modifiers,
        &[Modifier::Public, Modifier::Abstract],
    );
    output_buffer.push_str(&format!(
        "class {qualified_name}{generics_string}{super_clause}{implements_clause} {{\n"
    ));

    for member in &class_declaration.members {
        match member {
            ClassMember::Field(field_declaration) => {
                let is_static = field_declaration.modifiers.contains(&Modifier::Static);
                if is_static
                    && !field_declaration.modifiers.contains(&Modifier::Public)
                    && !field_declaration.modifiers.contains(&Modifier::Protected)
                {
                    continue;
                }
                output_buffer.push_str("    ");
                append_declaration_modifiers(
                    output_buffer,
                    &field_declaration.modifiers,
                    &[
                        Modifier::Public,
                        Modifier::Protected,
                        Modifier::Private,
                        Modifier::Static,
                        Modifier::Const,
                    ],
                );
                let field_type_str =
                    get_span_text(field_declaration.field_type.span(), source_code);
                let field_name_str = get_span_text(field_declaration.name_span, source_code);
                output_buffer.push_str(&format!("{field_type_str} {field_name_str};\n"));
            }
            ClassMember::Method(method_declaration) => {
                if !is_debug && method_declaration.modifiers.contains(&Modifier::Private) {
                    continue;
                }
                let method_name = get_span_text(method_declaration.name_span, source_code);
                if method_name.starts_with('~') {
                    continue;
                }

                let has_variadic_param = method_declaration
                    .parameters
                    .iter()
                    .any(|param| param.is_variadic);
                let is_generic_method =
                    !method_declaration.generic_params.is_empty() || has_variadic_param;
                let should_preserve_body = is_generic_class
                    || is_generic_method
                    || method_declaration.modifiers.contains(&Modifier::Inline);

                output_buffer.push_str("    ");
                append_declaration_modifiers(
                    output_buffer,
                    &method_declaration.modifiers,
                    &[
                        Modifier::Public,
                        Modifier::Protected,
                        Modifier::Static,
                        Modifier::Virtual,
                        Modifier::Override,
                        Modifier::Abstract,
                        Modifier::Inline,
                    ],
                );
                let return_type_str =
                    get_span_text(method_declaration.return_type.span(), source_code);
                let method_generics =
                    format_generic_parameters(&method_declaration.generic_params, source_code);
                let parameters_str = format_parameters(&method_declaration.parameters, source_code);

                if should_preserve_body {
                    if let Some(body_block) = &method_declaration.body {
                        let body_str = get_span_text(body_block.span, source_code);
                        output_buffer.push_str(&format!(
                            "{return_type_str} {method_name}{method_generics}({parameters_str}) {body_str}\n"
                        ));
                    } else {
                        output_buffer.push_str(&format!(
                            "{return_type_str} {method_name}{method_generics}({parameters_str});\n"
                        ));
                    }
                } else {
                    output_buffer.push_str(&format!(
                        "{return_type_str} {method_name}{method_generics}({parameters_str});\n"
                    ));
                }
            }
            ClassMember::Constructor(constructor_declaration) => {
                if constructor_declaration
                    .modifiers
                    .contains(&Modifier::Private)
                {
                    continue;
                }
                output_buffer.push_str("    ");
                append_declaration_modifiers(
                    output_buffer,
                    &constructor_declaration.modifiers,
                    &[Modifier::Public, Modifier::Protected, Modifier::Inline],
                );
                let parameters_str =
                    format_parameters(&constructor_declaration.parameters, source_code);
                if is_generic_class {
                    let body_str = get_span_text(constructor_declaration.body.span, source_code);
                    output_buffer.push_str(&format!("{class_name}({parameters_str}) {body_str}\n"));
                } else {
                    output_buffer.push_str(&format!("{class_name}({parameters_str});\n"));
                }
            }
            ClassMember::Destructor(destructor_declaration) => {
                if is_generic_class {
                    output_buffer.push_str("    ");
                    let body_str = get_span_text(destructor_declaration.body.span, source_code);
                    output_buffer.push_str(&format!("~{class_name}() {body_str}\n"));
                }
            }
            ClassMember::Operator(operator_declaration) => {
                if operator_declaration.modifiers.contains(&Modifier::Private) {
                    continue;
                }
                let op_str = get_span_text(operator_declaration.operator_span, source_code);
                output_buffer.push_str("    ");
                append_declaration_modifiers(
                    output_buffer,
                    &operator_declaration.modifiers,
                    &[Modifier::Public, Modifier::Protected, Modifier::Static],
                );
                let return_type_str =
                    get_span_text(operator_declaration.return_type.span(), source_code);
                let parameters_str =
                    format_parameters(&operator_declaration.parameters, source_code);
                if is_generic_class {
                    let body_str = get_span_text(operator_declaration.body.span, source_code);
                    output_buffer.push_str(&format!(
                        "{return_type_str} operator{op_str}({parameters_str}) {body_str}\n"
                    ));
                } else {
                    output_buffer.push_str(&format!(
                        "{return_type_str} operator{op_str}({parameters_str});\n"
                    ));
                }
            }
            ClassMember::Enum(enum_declaration) => {
                if enum_declaration.modifiers.contains(&Modifier::Public) {
                    emit_enum_header(output_buffer, enum_declaration, "", source_code);
                }
            }
            ClassMember::Class(_) | ClassMember::Interface(_) => {}
        }
    }

    output_buffer.push_str("}\n\n");
}

fn emit_interface_header(
    output_buffer: &mut String,
    interface_declaration: &InterfaceDeclaration,
    file_package: &str,
    source_code: &str,
) {
    if interface_declaration.modifiers.contains(&Modifier::Extern)
        || !interface_declaration.modifiers.contains(&Modifier::Public)
    {
        return;
    }

    let decl_namespace = extract_namespace_path(&interface_declaration.namespace_path, source_code);
    let effective_package = compute_effective_package(file_package, &decl_namespace);
    let iface_name = get_span_text(interface_declaration.name_span, source_code);
    let qualified_name = if effective_package.is_empty() {
        iface_name.to_string()
    } else {
        format!("{effective_package}::{iface_name}")
    };

    let generics_string =
        format_generic_parameters(&interface_declaration.generic_params, source_code);
    let extends_clause = if !interface_declaration.super_interfaces.is_empty() {
        let parent_names = interface_declaration
            .super_interfaces
            .iter()
            .map(|super_iface| get_span_text(super_iface.span(), source_code))
            .collect::<Vec<_>>()
            .join(", ");
        format!(" extends {parent_names}")
    } else {
        String::new()
    };

    output_buffer.push_str("extern public ");
    output_buffer.push_str(&format!(
        "interface {qualified_name}{generics_string}{extends_clause} {{\n"
    ));

    for method in &interface_declaration.methods {
        let method_name = get_span_text(method.name_span, source_code);
        let return_type_str = get_span_text(method.return_type.span(), source_code);
        let method_generics = format_generic_parameters(&method.generic_params, source_code);
        let parameters_str = format_parameters(&method.parameters, source_code);
        output_buffer.push_str(&format!(
            "    {return_type_str} {method_name}{method_generics}({parameters_str});\n"
        ));
    }

    output_buffer.push_str("}\n\n");
}

fn emit_enum_header(
    output_buffer: &mut String,
    enum_declaration: &EnumDeclaration,
    file_package: &str,
    source_code: &str,
) {
    if !enum_declaration.modifiers.contains(&Modifier::Public) {
        return;
    }

    let decl_namespace = extract_namespace_path(&enum_declaration.namespace_path, source_code);
    let effective_package = compute_effective_package(file_package, &decl_namespace);
    let enum_name = get_span_text(enum_declaration.name_span, source_code);
    let qualified_name = if effective_package.is_empty() {
        enum_name.to_string()
    } else {
        format!("{effective_package}::{enum_name}")
    };

    output_buffer.push_str(&format!("public enum {qualified_name} {{\n"));
    for (variant_index, variant) in enum_declaration.variants.iter().enumerate() {
        let variant_name = get_span_text(variant.name_span, source_code);
        output_buffer.push_str("    ");
        output_buffer.push_str(variant_name);
        if let Some(explicit_val) = &variant.explicit_value {
            output_buffer.push_str(&format!(
                " = {}",
                get_span_text(explicit_val.span(), source_code)
            ));
        }
        if variant_index + 1 < enum_declaration.variants.len() {
            output_buffer.push_str(",\n");
        } else {
            output_buffer.push('\n');
        }
    }
    output_buffer.push_str("}\n\n");
}

fn emit_top_level_method(
    output_buffer: &mut String,
    method_declaration: &MethodDeclaration,
    file_package: &str,
    source_code: &str,
    is_debug: bool,
) {
    if method_declaration.modifiers.contains(&Modifier::Extern)
        || (!is_debug && !method_declaration.modifiers.contains(&Modifier::Public))
    {
        return;
    }

    let decl_namespace = extract_namespace_path(&method_declaration.namespace_path, source_code);
    let effective_package = compute_effective_package(file_package, &decl_namespace);
    let func_name = get_span_text(method_declaration.name_span, source_code);
    let qualified_name = if effective_package.is_empty() {
        func_name.to_string()
    } else {
        format!("{effective_package}::{func_name}")
    };

    let return_type_str = get_span_text(method_declaration.return_type.span(), source_code);
    let generics_str = format_generic_parameters(&method_declaration.generic_params, source_code);
    let parameters_str = format_parameters(&method_declaration.parameters, source_code);

    let has_variadic_param = method_declaration
        .parameters
        .iter()
        .any(|param| param.is_variadic);
    let is_generic_method = !method_declaration.generic_params.is_empty() || has_variadic_param;
    let should_preserve_body =
        is_generic_method || method_declaration.modifiers.contains(&Modifier::Inline);

    output_buffer.push_str("extern public ");
    if should_preserve_body {
        if let Some(body_block) = &method_declaration.body {
            let body_str = get_span_text(body_block.span, source_code);
            output_buffer.push_str(&format!(
                "{return_type_str} {qualified_name}{generics_str}({parameters_str}) {body_str}\n\n"
            ));
        } else {
            output_buffer.push_str(&format!(
                "{return_type_str} {qualified_name}{generics_str}({parameters_str});\n\n"
            ));
        }
    } else {
        output_buffer.push_str(&format!(
            "{return_type_str} {qualified_name}{generics_str}({parameters_str});\n\n"
        ));
    }
}

fn emit_top_level_field(
    output_buffer: &mut String,
    field_declaration: &FieldDeclaration,
    file_package: &str,
    source_code: &str,
    is_debug: bool,
) {
    let is_const = field_declaration.modifiers.contains(&Modifier::Const);
    if field_declaration.modifiers.contains(&Modifier::Extern)
        || (!is_debug && !is_const && !field_declaration.modifiers.contains(&Modifier::Public))
    {
        return;
    }

    let decl_namespace = extract_namespace_path(&field_declaration.namespace_path, source_code);
    let effective_package = compute_effective_package(file_package, &decl_namespace);
    let field_name = get_span_text(field_declaration.name_span, source_code);
    let qualified_name = if effective_package.is_empty() {
        field_name.to_string()
    } else {
        format!("{effective_package}::{field_name}")
    };

    let field_type_str = get_span_text(field_declaration.field_type.span(), source_code);
    let const_prefix = if is_const { "const " } else { "" };
    output_buffer.push_str(&format!(
        "extern {const_prefix}{field_type_str} {qualified_name};\n\n"
    ));
}

/// Magic header identifying lazy-indexed Section-7 AST payloads (`SLXAST01`).
pub const INDEXED_AST_MAGIC: &[u8; 8] = b"SLXAST01";

/// Minimum declaration count before emitting indexed Section-7 AST payloads.
const INDEXED_AST_THRESHOLD: usize = 16;

/// Metadata index entry for a single serialized declaration inside an `SLXAST01` payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexedDeclEntry {
    /// Simple declaration identifier (empty for constructors/destructors/operators).
    pub simple_name: String,
    /// Namespace-qualified declaration identifier.
    pub qualified_name: String,
    /// Effective namespace path.
    pub namespace: String,
    /// Byte offset inside the concatenated declaration blob buffer.
    pub byte_offset: u32,
    /// Byte length of this declaration's bincode payload.
    pub byte_len: u32,
    /// True for declarations that cannot be pruned by name (constructors/destructors/operators).
    pub always_keep: bool,
    /// Deduplicated identifiers referenced in this declaration's API signature/body.
    pub referenced_names: Vec<String>,
}

/// Header for an indexed Section-7 AST payload enabling selective lazy deserialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexedAstHeader {
    /// Original source identifier at emission time.
    pub source_id: u16,
    /// Overall source coordinate span.
    pub span: Span,
    /// Optional package declaration.
    pub package: Option<crate::phases::parser::statements::PackageStatement>,
    /// Package imports.
    pub imports: Vec<crate::phases::parser::statements::ImportStatement>,
    /// Package type aliases.
    pub aliases: Vec<crate::phases::parser::statements::AliasStatement>,
    /// Index of serialized declarations.
    pub entries: Vec<IndexedDeclEntry>,
}

fn collect_decl_identifiers(slice: &str, own_name: &str) -> Vec<String> {
    let mut seen = rustc_hash::FxHashSet::default();
    let mut identifiers = Vec::new();
    let bytes = slice.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        let byte = bytes[idx];
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = idx;
            idx += 1;
            while idx < bytes.len() && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'_') {
                idx += 1;
            }
            let word = &slice[start..idx];
            if word != own_name && seen.insert(word) {
                identifiers.push(word.to_string());
            }
        } else {
            idx += 1;
        }
    }
    identifiers
}

fn extract_decl_index_names(
    declaration: &Declaration,
    source_code: &str,
) -> (String, String, String, bool) {
    let (name_span, namespace_path, always_keep) = match declaration {
        Declaration::Class(class_decl) => (
            class_decl.name_span,
            class_decl.namespace_path.as_slice(),
            false,
        ),
        Declaration::Interface(iface_decl) => (
            iface_decl.name_span,
            iface_decl.namespace_path.as_slice(),
            false,
        ),
        Declaration::Enum(enum_decl) => (
            enum_decl.name_span,
            enum_decl.namespace_path.as_slice(),
            false,
        ),
        Declaration::Method(method_decl) => (
            method_decl.name_span,
            method_decl.namespace_path.as_slice(),
            false,
        ),
        Declaration::Field(field_decl) => (
            field_decl.name_span,
            field_decl.namespace_path.as_slice(),
            false,
        ),
        Declaration::Constructor(_) | Declaration::Destructor(_) | Declaration::Operator(_) => {
            return (String::new(), String::new(), String::new(), true);
        }
    };
    let simple_name = get_span_text(name_span, source_code).to_string();
    let namespace = extract_namespace_path(namespace_path, source_code);
    let qualified_name = if namespace.is_empty() {
        simple_name.clone()
    } else {
        format!("{namespace}::{simple_name}")
    };
    (simple_name, qualified_name, namespace, always_keep)
}

/// Serializes a `ProgramAst` into `SLXAST01` indexed format when it has `>= 16` declarations.
#[must_use]
pub fn encode_indexed_program_ast(
    program_ast: &crate::phases::parser::statements::ProgramAst,
    api_header: &str,
) -> Option<Vec<u8>> {
    let mut entries = Vec::with_capacity(program_ast.declarations.len());
    let mut decl_blobs = Vec::new();

    for declaration in &program_ast.declarations {
        let blob = bincode::serialize(declaration).ok()?;
        let byte_offset = u32::try_from(decl_blobs.len()).ok()?;
        let byte_len = u32::try_from(blob.len()).ok()?;
        decl_blobs.extend_from_slice(&blob);

        let (simple_name, qualified_name, namespace, always_keep) =
            extract_decl_index_names(declaration, api_header);
        let decl_slice = get_span_text(declaration.span(), api_header);
        let referenced_names = collect_decl_identifiers(decl_slice, &simple_name);
        entries.push(IndexedDeclEntry {
            simple_name,
            qualified_name,
            namespace,
            byte_offset,
            byte_len,
            always_keep,
            referenced_names,
        });
    }

    let header = IndexedAstHeader {
        source_id: program_ast.source_id,
        span: program_ast.span,
        package: program_ast.package.clone(),
        imports: program_ast.imports.clone(),
        aliases: program_ast.aliases.clone(),
        entries,
    };
    let header_bytes = bincode::serialize(&header).ok()?;
    let header_len = u32::try_from(header_bytes.len()).ok()?;

    let mut output =
        Vec::with_capacity(INDEXED_AST_MAGIC.len() + 4 + header_bytes.len() + decl_blobs.len());
    output.extend_from_slice(INDEXED_AST_MAGIC);
    output.extend_from_slice(&header_len.to_le_bytes());
    output.extend_from_slice(&header_bytes);
    output.extend_from_slice(&decl_blobs);
    Some(output)
}

/// Generates the public API header source string and its pre-parsed serialized AST bytes.
pub fn generate_api_header_and_ast(context: &CompilationContext) -> (String, Vec<u8>) {
    let api_header = generate_api_header(context);
    if api_header.trim().is_empty() {
        return (api_header, Vec::new());
    }

    let (tokens, lex_errors) = crate::phases::lexing::lexer::tokenize_source(0, &api_header);
    if !lex_errors.is_empty() {
        return (api_header, Vec::new());
    }

    match crate::phases::parser::parse_source(0, &tokens, &api_header) {
        Ok(program_ast) => {
            if program_ast.declarations.len() >= INDEXED_AST_THRESHOLD
                && let Some(indexed_bytes) = encode_indexed_program_ast(&program_ast, &api_header)
            {
                return (api_header, indexed_bytes);
            }
            match bincode::serialize(&program_ast) {
                Ok(ast_bytes) => (api_header, ast_bytes),
                Err(_) => (api_header, Vec::new()),
            }
        }
        Err(_) => (api_header, Vec::new()),
    }
}
