//! Static API documentation generator for Solix projects and libraries.

#![forbid(unsafe_code)]

use crate::cli::DocArgs;
use anyhow::{Context, Result, bail};
use solix_common::project::ProjectManifest;
use solix_common::span::Span;
use solix_compilation::CompilationContext::CompilationContext;
use solix_compilation::CompilationOptions;
use solix_compilation::phases::lexing::lexer::lex_context;
use solix_compilation::phases::parser::parser::parse_context_resilient;
use solix_compilation::phases::parser::statements::{
    ClassDeclaration, ClassMember, ConstructorDeclaration, Declaration, EnumDeclaration,
    InterfaceDeclaration, MethodDeclaration,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Extracted documentation tag from a doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocTag {
    /// `@param <name> <description>`
    Param {
        name: String,
        description: String,
    },
    /// `@return <description>`
    Return { description: String },
    /// `@throws <type> <description>`
    Throws {
        exception_type: String,
        description: String,
    },
    /// `@deprecated <reason>`
    Deprecated { reason: String },
}

/// Parsed doc comment containing narrative summary and structured tags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedDocComment {
    /// Initial summary line.
    pub summary: String,
    /// Detailed markdown description body.
    pub description: String,
    /// Parsed tags (`@param`, `@return`, `@throws`, `@deprecated`).
    pub tags: Vec<DocTag>,
}

/// Documented class or interface member.
#[derive(Debug, Clone)]
pub struct DocumentedMember {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub modifiers: Vec<String>,
    pub doc: ParsedDocComment,
}

/// Documented type (class, interface, enum) or top-level function.
#[derive(Debug, Clone)]
pub struct DocumentedType {
    pub package_name: String,
    pub name: String,
    pub kind: String,
    pub modifiers: Vec<String>,
    pub extends_name: Option<String>,
    pub implements_names: Vec<String>,
    pub doc: ParsedDocComment,
    pub members: Vec<DocumentedMember>,
    pub enum_variants: Vec<(String, Option<String>)>,
}

/// Executes the `solix doc` command.
///
/// # Errors
/// Returns an error if source parsing or file writing fails.
pub fn execute_doc(args: DocArgs) -> Result<()> {
    let workspace_path = args.path.canonicalize().unwrap_or(args.path.clone());
    let manifest_path = workspace_path.join("manifest.toml");

    let mut source_paths = Vec::new();
    let mut project_name = "Untitled Project".to_string();
    let mut project_version = "1.0.0".to_string();
    let mut project_description = "Solix Application".to_string();

    if manifest_path.exists() {
        let manifest_content = fs::read_to_string(&manifest_path).with_context(|| {
            format!(
                "[E0303] Failed to read project manifest at '{}'",
                manifest_path.display()
            )
        })?;
        let project_manifest = ProjectManifest::from_toml_str(&manifest_content).with_context(|| {
            format!(
                "[E0303] Failed to parse project manifest at '{}'",
                manifest_path.display()
            )
        })?;

        project_name = project_manifest.configuration.name.clone();
        project_version = project_manifest.configuration.version.clone();
        project_description = project_manifest
            .configuration
            .description
            .clone()
            .unwrap_or_else(|| "Solix Application".to_string());

        let effective_profile = args.profile.as_deref().unwrap_or("release");
        let resolved_profile = project_manifest
            .resolve_profile(effective_profile)
            .ok()
            .or_else(|| project_manifest.resolve_profile("default").ok());

        let source_directories = resolved_profile
            .map(|profile| profile.sources)
            .unwrap_or_else(|| vec!["src".to_string()]);

        for source_directory in &source_directories {
            let directory_path = workspace_path.join(source_directory);
            if directory_path.is_dir() {
                collect_solix_sources(&directory_path, &mut source_paths)?;
            }
        }
    } else {
        let src_dir = workspace_path.join("src");
        if src_dir.is_dir() {
            collect_solix_sources(&src_dir, &mut source_paths)?;
        } else {
            collect_solix_sources(&workspace_path, &mut source_paths)?;
        }
    }

    if source_paths.is_empty() {
        bail!(
            "[E0303] No Solix source files (*.slx) found for documentation in '{}'",
            workspace_path.display()
        );
    }

    let mut compilation_options = CompilationOptions::new();
    let mut file_sources = Vec::new();

    for source_path in &source_paths {
        let source_content = fs::read_to_string(source_path).with_context(|| {
            format!(
                "[E0303] Failed to read source file '{}'",
                source_path.display()
            )
        })?;
        let relative_display = source_path
            .strip_prefix(&workspace_path)
            .unwrap_or(source_path)
            .display()
            .to_string();

        compilation_options = compilation_options.with_source(relative_display.clone(), source_content.clone());
        file_sources.push((relative_display, source_content));
    }

    let mut compilation_context = CompilationContext::new(compilation_options);
    let _ = lex_context(&mut compilation_context);
    let _parsing_diagnostics = parse_context_resilient(&mut compilation_context);

    let mut documented_types = Vec::new();

    for (source_index, program_ast_opt) in compilation_context.ast.iter().enumerate() {
        if let Some(program_ast) = program_ast_opt {
            let source_content = compilation_context
                .sources
                .get(source_index)
                .map(|content| content.as_str())
                .unwrap_or("");

            let package_name = program_ast
                .package
                .as_ref()
                .map(|package_statement| {
                    package_statement
                        .path
                        .iter()
                        .map(|segment_span| get_span_string(source_content, *segment_span))
                        .collect::<Vec<String>>()
                        .join(".")
                })
                .unwrap_or_else(|| "global".to_string());

            for declaration in &program_ast.declarations {
                if let Some(doc_type) = extract_declaration_doc(declaration, &package_name, source_content) {
                    documented_types.push(doc_type);
                }
            }
        }
    }

    fs::create_dir_all(&args.output).with_context(|| {
        format!(
            "[E0302] Failed to create documentation output directory '{}'",
            args.output.display()
        )
    })?;

    let is_markdown = args.format.eq_ignore_ascii_case("markdown") || args.format.eq_ignore_ascii_case("md");

    if is_markdown {
        generate_markdown_docs(
            &args.output,
            &project_name,
            &project_version,
            &project_description,
            &documented_types,
        )?;
    } else {
        generate_html_docs(
            &args.output,
            &project_name,
            &project_version,
            &project_description,
            &documented_types,
        )?;
    }

    println!(
        "Generated API documentation for {} types into '{}'",
        documented_types.len(),
        args.output.display()
    );

    if args.open && !is_markdown {
        let index_file = args.output.join("index.html");
        open_in_browser(&index_file);
    }

    Ok(())
}

fn collect_solix_sources(current_dir: &Path, discovered_paths: &mut Vec<PathBuf>) -> Result<()> {
    if !current_dir.exists() {
        return Ok(());
    }
    let read_dir = fs::read_dir(current_dir)
        .with_context(|| format!("[E0303] Failed to read directory '{}'", current_dir.display()))?;

    for entry_result in read_dir {
        let entry = entry_result?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let directory_name = entry.file_name().to_string_lossy().to_string();
            if !directory_name.starts_with('.') && directory_name != "target" && directory_name != "build" {
                collect_solix_sources(&entry_path, discovered_paths)?;
            }
        } else if entry_path.extension().is_some_and(|ext| ext == "slx") {
            discovered_paths.push(entry_path);
        }
    }
    Ok(())
}

fn extract_declaration_doc(
    declaration: &Declaration,
    package_name: &str,
    source_content: &str,
) -> Option<DocumentedType> {
    match declaration {
        Declaration::Class(class_decl) => Some(extract_class_doc(class_decl, package_name, source_content)),
        Declaration::Interface(iface_decl) => Some(extract_interface_doc(iface_decl, package_name, source_content)),
        Declaration::Enum(enum_decl) => Some(extract_enum_doc(enum_decl, package_name, source_content)),
        Declaration::Method(method_decl) => Some(extract_top_level_method_doc(method_decl, package_name, source_content)),
        _ => None,
    }
}

fn extract_class_doc(
    class_decl: &ClassDeclaration,
    package_name: &str,
    source_content: &str,
) -> DocumentedType {
    let raw_doc = extract_preceding_doc_comment(source_content, class_decl.span);
    let parsed_doc = parse_doc_comment_text(&raw_doc);

    let class_name = get_span_string(source_content, class_decl.name_span);
    let modifiers_list: Vec<String> = class_decl
        .modifiers
        .iter()
        .map(|modifier_item| modifier_item.as_str().to_string())
        .collect();

    let extends_name = class_decl
        .super_class
        .as_ref()
        .map(|base_type| get_span_string(source_content, base_type.span()));

    let implements_names: Vec<String> = class_decl
        .interfaces
        .iter()
        .map(|interface_type| get_span_string(source_content, interface_type.span()))
        .collect();

    let mut members_list = Vec::new();

    for member_item in &class_decl.members {
        match member_item {
            ClassMember::Method(method_decl) => {
                let member_doc = extract_preceding_doc_comment(source_content, method_decl.span);
                let method_name = get_span_string(source_content, method_decl.name_span);
                let parameters_signature = format_method_parameters(source_content, method_decl);
                let method_modifiers: Vec<String> = method_decl
                    .modifiers
                    .iter()
                    .map(|mod_item| mod_item.as_str().to_string())
                    .collect();

                let return_type_str = get_span_string(source_content, method_decl.return_type.span());
                let signature_display = format!(
                    "{return_type_str} {method_name}({parameters_signature})"
                );

                members_list.push(DocumentedMember {
                    name: method_name,
                    kind: "method".to_string(),
                    signature: signature_display,
                    modifiers: method_modifiers,
                    doc: parse_doc_comment_text(&member_doc),
                });
            }
            ClassMember::Constructor(ctor_decl) => {
                let member_doc = extract_preceding_doc_comment(source_content, ctor_decl.span);
                let ctor_name = "constructor".to_string();
                let parameters_signature = format_constructor_parameters(source_content, ctor_decl);
                let ctor_modifiers: Vec<String> = ctor_decl
                    .modifiers
                    .iter()
                    .map(|mod_item| mod_item.as_str().to_string())
                    .collect();

                let signature_display = format!("{class_name}({parameters_signature})");

                members_list.push(DocumentedMember {
                    name: ctor_name,
                    kind: "constructor".to_string(),
                    signature: signature_display,
                    modifiers: ctor_modifiers,
                    doc: parse_doc_comment_text(&member_doc),
                });
            }
            ClassMember::Field(field_decl) => {
                let member_doc = extract_preceding_doc_comment(source_content, field_decl.span);
                let field_name = get_span_string(source_content, field_decl.name_span);
                let field_modifiers: Vec<String> = field_decl
                    .modifiers
                    .iter()
                    .map(|mod_item| mod_item.as_str().to_string())
                    .collect();

                let field_type_str = get_span_string(source_content, field_decl.field_type.span());
                let signature_display = format!("{field_type_str} {field_name}");

                members_list.push(DocumentedMember {
                    name: field_name,
                    kind: "field".to_string(),
                    signature: signature_display,
                    modifiers: field_modifiers,
                    doc: parse_doc_comment_text(&member_doc),
                });
            }
            ClassMember::Operator(operator_decl) => {
                let member_doc = extract_preceding_doc_comment(source_content, operator_decl.span);
                let operator_token_str = operator_decl.operator_token.as_static_str();
                let operator_name = format!("operator{operator_token_str}");
                let operator_modifiers: Vec<String> = operator_decl
                    .modifiers
                    .iter()
                    .map(|mod_item| mod_item.as_str().to_string())
                    .collect();

                let return_type_str = get_span_string(source_content, operator_decl.return_type.span());
                let signature_display = format!(
                    "{return_type_str} operator{operator_token_str}(...)"
                );

                members_list.push(DocumentedMember {
                    name: operator_name,
                    kind: "operator".to_string(),
                    signature: signature_display,
                    modifiers: operator_modifiers,
                    doc: parse_doc_comment_text(&member_doc),
                });
            }
            _ => {}
        }
    }

    DocumentedType {
        package_name: package_name.to_string(),
        name: class_name,
        kind: "class".to_string(),
        modifiers: modifiers_list,
        extends_name,
        implements_names,
        doc: parsed_doc,
        members: members_list,
        enum_variants: Vec::new(),
    }
}

fn extract_interface_doc(
    iface_decl: &InterfaceDeclaration,
    package_name: &str,
    source_content: &str,
) -> DocumentedType {
    let raw_doc = extract_preceding_doc_comment(source_content, iface_decl.span);
    let parsed_doc = parse_doc_comment_text(&raw_doc);

    let iface_name = get_span_string(source_content, iface_decl.name_span);
    let modifiers_list: Vec<String> = iface_decl
        .modifiers
        .iter()
        .map(|modifier_item| modifier_item.as_str().to_string())
        .collect();

    let mut members_list = Vec::new();

    for method_decl in &iface_decl.methods {
        let member_doc = extract_preceding_doc_comment(source_content, method_decl.span);
        let method_name = get_span_string(source_content, method_decl.name_span);
        let parameters_signature = format_method_parameters(source_content, method_decl);
        let return_type_str = get_span_string(source_content, method_decl.return_type.span());
        let signature_display = format!(
            "{return_type_str} {method_name}({parameters_signature})"
        );

        members_list.push(DocumentedMember {
            name: method_name,
            kind: "method".to_string(),
            signature: signature_display,
            modifiers: vec!["public".to_string()],
            doc: parse_doc_comment_text(&member_doc),
        });
    }

    DocumentedType {
        package_name: package_name.to_string(),
        name: iface_name,
        kind: "interface".to_string(),
        modifiers: modifiers_list,
        extends_name: None,
        implements_names: Vec::new(),
        doc: parsed_doc,
        members: members_list,
        enum_variants: Vec::new(),
    }
}

fn extract_enum_doc(
    enum_decl: &EnumDeclaration,
    package_name: &str,
    source_content: &str,
) -> DocumentedType {
    let raw_doc = extract_preceding_doc_comment(source_content, enum_decl.span);
    let parsed_doc = parse_doc_comment_text(&raw_doc);

    let enum_name = get_span_string(source_content, enum_decl.name_span);
    let modifiers_list: Vec<String> = enum_decl
        .modifiers
        .iter()
        .map(|modifier_item| modifier_item.as_str().to_string())
        .collect();

    let mut variants_list = Vec::new();

    for variant_item in &enum_decl.variants {
        let variant_doc = extract_preceding_doc_comment(source_content, variant_item.span);
        let variant_name = get_span_string(source_content, variant_item.name_span);
        let doc_opt = if variant_doc.trim().is_empty() {
            None
        } else {
            Some(variant_doc.trim().to_string())
        };
        variants_list.push((variant_name, doc_opt));
    }

    DocumentedType {
        package_name: package_name.to_string(),
        name: enum_name,
        kind: "enum".to_string(),
        modifiers: modifiers_list,
        extends_name: None,
        implements_names: Vec::new(),
        doc: parsed_doc,
        members: Vec::new(),
        enum_variants: variants_list,
    }
}

fn extract_top_level_method_doc(
    method_decl: &MethodDeclaration,
    package_name: &str,
    source_content: &str,
) -> DocumentedType {
    let raw_doc = extract_preceding_doc_comment(source_content, method_decl.span);
    let parsed_doc = parse_doc_comment_text(&raw_doc);

    let method_name = get_span_string(source_content, method_decl.name_span);
    let parameters_signature = format_method_parameters(source_content, method_decl);
    let modifiers_list: Vec<String> = method_decl
        .modifiers
        .iter()
        .map(|modifier_item| modifier_item.as_str().to_string())
        .collect();

    let return_type_str = get_span_string(source_content, method_decl.return_type.span());
    let signature_display = format!(
        "{return_type_str} {method_name}({parameters_signature})"
    );

    let member_entry = DocumentedMember {
        name: method_name.clone(),
        kind: "function".to_string(),
        signature: signature_display,
        modifiers: modifiers_list.clone(),
        doc: parsed_doc.clone(),
    };

    DocumentedType {
        package_name: package_name.to_string(),
        name: method_name,
        kind: "function".to_string(),
        modifiers: modifiers_list,
        extends_name: None,
        implements_names: Vec::new(),
        doc: parsed_doc,
        members: vec![member_entry],
        enum_variants: Vec::new(),
    }
}

fn format_method_parameters(source_content: &str, method_decl: &MethodDeclaration) -> String {
    method_decl
        .parameters
        .iter()
        .map(|param_item| {
            let param_type_str = get_span_string(source_content, param_item.param_type.span());
            let param_name = get_span_string(source_content, param_item.name_span);
            format!("{param_type_str} {param_name}")
        })
        .collect::<Vec<String>>()
        .join(", ")
}

fn format_constructor_parameters(source_content: &str, ctor_decl: &ConstructorDeclaration) -> String {
    ctor_decl
        .parameters
        .iter()
        .map(|param_item| {
            let param_type_str = get_span_string(source_content, param_item.param_type.span());
            let param_name = get_span_string(source_content, param_item.name_span);
            format!("{param_type_str} {param_name}")
        })
        .collect::<Vec<String>>()
        .join(", ")
}

fn get_span_string(source: &str, span: Span) -> String {
    let start_index = span.start as usize;
    let end_index = span.end as usize;
    if start_index <= end_index && end_index <= source.len() {
        source[start_index..end_index].to_string()
    } else {
        String::new()
    }
}

fn extract_preceding_doc_comment(source_text: &str, span: Span) -> String {
    let start_offset = span.start as usize;
    if start_offset > source_text.len() {
        return String::new();
    }

    let prefix = &source_text[..start_offset];
    let lines: Vec<&str> = prefix.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut doc_lines = Vec::new();
    let mut index = lines.len() - 1;

    // Skip the declaration line itself if it does not contain a doc comment
    if index > 0 && !lines[index].trim().starts_with("///") && !lines[index].trim().ends_with("*/") {
        index -= 1;
    }

    while index > 0 && lines[index].trim().is_empty() {
        index -= 1;
    }

    while index < lines.len() {
        let line_trimmed = lines[index].trim();
        if let Some(stripped) = line_trimmed.strip_prefix("///") {
            doc_lines.push(stripped.trim().to_string());
            if index == 0 {
                break;
            }
            index -= 1;
        } else if line_trimmed.ends_with("*/") {
            let mut block_lines = Vec::new();
            let mut block_index = index;
            let mut found_start = false;
            loop {
                let raw_line = lines[block_index].trim();
                let cleaned = raw_line
                    .trim_start_matches("/*")
                    .trim_end_matches("*/")
                    .trim_start_matches('*')
                    .trim();
                if !cleaned.is_empty() {
                    block_lines.push(cleaned.to_string());
                }
                if raw_line.starts_with("/*") {
                    found_start = true;
                    break;
                }
                if block_index == 0 {
                    break;
                }
                block_index -= 1;
            }
            if found_start {
                block_lines.reverse();
                doc_lines.extend(block_lines);
            }
            break;
        } else {
            break;
        }
    }

    if doc_lines.is_empty() {
        String::new()
    } else {
        doc_lines.reverse();
        doc_lines.join("\n")
    }
}

/// Parses raw doc comment text into a structured `ParsedDocComment`.
#[must_use]
pub fn parse_doc_comment_text(raw_doc: &str) -> ParsedDocComment {
    let trimmed = raw_doc.trim();
    if trimmed.is_empty() {
        return ParsedDocComment::default();
    }

    let mut description_lines = Vec::new();
    let mut parsed_tags = Vec::new();

    for line in trimmed.lines() {
        let line_trimmed = line.trim();
        if let Some(rest) = line_trimmed.strip_prefix("@param") {
            let mut parts = rest.trim().splitn(2, ' ');
            let param_name = parts.next().unwrap_or("").trim().to_string();
            let param_desc = parts.next().unwrap_or("").trim().to_string();
            parsed_tags.push(DocTag::Param {
                name: param_name,
                description: param_desc,
            });
        } else if let Some(rest) = line_trimmed.strip_prefix("@return") {
            parsed_tags.push(DocTag::Return {
                description: rest.trim().to_string(),
            });
        } else if let Some(rest) = line_trimmed.strip_prefix("@throws") {
            let mut parts = rest.trim().splitn(2, ' ');
            let exception_type = parts.next().unwrap_or("").trim().to_string();
            let exception_desc = parts.next().unwrap_or("").trim().to_string();
            parsed_tags.push(DocTag::Throws {
                exception_type,
                description: exception_desc,
            });
        } else if let Some(rest) = line_trimmed.strip_prefix("@deprecated") {
            parsed_tags.push(DocTag::Deprecated {
                reason: rest.trim().to_string(),
            });
        } else {
            description_lines.push(line_trimmed);
        }
    }

    let summary = description_lines.first().copied().unwrap_or("").to_string();
    let description = description_lines.join("\n");

    ParsedDocComment {
        summary,
        description,
        tags: parsed_tags,
    }
}

fn generate_markdown_docs(
    output_dir: &Path,
    project_name: &str,
    project_version: &str,
    project_description: &str,
    types: &[DocumentedType],
) -> Result<()> {
    let mut index_content = format!(
        "# {project_name} API Reference (v{project_version})\n\n{project_description}\n\n---\n\n## Packages and Types\n\n"
    );

    let mut grouped_packages: BTreeMap<String, Vec<&DocumentedType>> = BTreeMap::new();
    for documented_type in types {
        grouped_packages
            .entry(documented_type.package_name.clone())
            .or_default()
            .push(documented_type);
    }

    for (package_name, package_types) in &grouped_packages {
        index_content.push_str(&format!("### Package `{package_name}`\n\n"));
        index_content.push_str("| Type | Kind | Summary |\n|:---|:---:|:---|\n");

        for doc_type in package_types {
            let summary_text = if doc_type.doc.summary.is_empty() {
                "*(No description provided)*"
            } else {
                &doc_type.doc.summary
            };
            index_content.push_str(&format!(
                "| [`{}`]({}_{}.md) | `{}` | {} |\n",
                doc_type.name,
                doc_type.package_name.replace('.', "_"),
                doc_type.name,
                doc_type.kind,
                summary_text
            ));
        }
        index_content.push('\n');
    }

    fs::write(output_dir.join("README.md"), index_content)
        .with_context(|| "[E0302] Failed to write README.md")?;

    for doc_type in types {
        let file_name = format!(
            "{}_{}.md",
            doc_type.package_name.replace('.', "_"),
            doc_type.name
        );
        let mut type_page = format!(
            "# {} `{}`\n\n**Package:** `{}`  \n",
            capitalize(&doc_type.kind),
            doc_type.name,
            doc_type.package_name
        );

        if let Some(ref base_class) = doc_type.extends_name {
            type_page.push_str(&format!("**Extends:** `{base_class}`  \n"));
        }
        if !doc_type.implements_names.is_empty() {
            type_page.push_str(&format!(
                "**Implements:** {}  \n",
                doc_type
                    .implements_names
                    .iter()
                    .map(|iface| format!("`{iface}`"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }

        type_page.push_str("\n---\n\n## Overview\n\n");
        if doc_type.doc.description.is_empty() {
            type_page.push_str("*(No documentation provided)*\n\n");
        } else {
            type_page.push_str(&format!("{}\n\n", doc_type.doc.description));
        }

        if !doc_type.enum_variants.is_empty() {
            type_page.push_str("## Variants\n\n| Variant | Description |\n|:---|:---|\n");
            for (variant_name, variant_doc) in &doc_type.enum_variants {
                let desc = variant_doc.as_deref().unwrap_or("*(No description)*");
                type_page.push_str(&format!("| `{variant_name}` | {desc} |\n"));
            }
            type_page.push('\n');
        }

        if !doc_type.members.is_empty() {
            type_page.push_str("## Members\n\n");
            for member in &doc_type.members {
                type_page.push_str(&format!("### `{}`\n\n", member.signature));
                if !member.doc.description.is_empty() {
                    type_page.push_str(&format!("{}\n\n", member.doc.description));
                }

                let param_tags: Vec<&DocTag> = member
                    .doc
                    .tags
                    .iter()
                    .filter(|tag| matches!(tag, DocTag::Param { .. }))
                    .collect();

                if !param_tags.is_empty() {
                    type_page.push_str("**Parameters:**\n\n");
                    for tag in param_tags {
                        if let DocTag::Param { name, description } = tag {
                            type_page.push_str(&format!("* `{name}`: {description}\n"));
                        }
                    }
                    type_page.push('\n');
                }

                for tag in &member.doc.tags {
                    match tag {
                        DocTag::Return { description } => {
                            type_page.push_str(&format!("**Returns:** {description}\n\n"));
                        }
                        DocTag::Throws { exception_type, description } => {
                            type_page.push_str(&format!("**Throws `{exception_type}`:** {description}\n\n"));
                        }
                        DocTag::Deprecated { reason } => {
                            type_page.push_str(&format!("> [!WARNING]\n> **Deprecated:** {reason}\n\n"));
                        }
                        _ => {}
                    }
                }
            }
        }

        fs::write(output_dir.join(file_name), type_page)
            .with_context(|| "[E0302] Failed to write type markdown documentation")?;
    }

    Ok(())
}

fn generate_html_docs(
    output_dir: &Path,
    project_name: &str,
    project_version: &str,
    project_description: &str,
    types: &[DocumentedType],
) -> Result<()> {
    let css_content = r#"
:root {
    --bg-primary: #121417;
    --bg-surface: #1a1d21;
    --bg-surface-elevated: #24282e;
    --border-color: #2f353d;
    --text-primary: #e6edf3;
    --text-muted: #8b949e;
    --accent: #58a6ff;
    --accent-hover: #79c0ff;
    --code-bg: #161b22;
    --badge-class: #3fb950;
    --badge-iface: #a371f7;
    --badge-enum: #f0883e;
    --badge-fn: #58a6ff;
}

body {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    display: flex;
    min-height: 100vh;
}

.sidebar {
    width: 280px;
    background-color: var(--bg-surface);
    border-right: 1px solid var(--border-color);
    padding: 24px 16px;
    box-sizing: border-box;
    position: sticky;
    top: 0;
    height: 100vh;
    overflow-y: auto;
}

.sidebar h2 {
    font-size: 1.1rem;
    margin-top: 0;
    color: var(--text-primary);
}

.sidebar .project-ver {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin-bottom: 20px;
}

.sidebar .nav-group {
    margin-bottom: 20px;
}

.sidebar .nav-title {
    font-size: 0.8rem;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    margin-bottom: 8px;
}

.sidebar ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

.sidebar li {
    margin-bottom: 6px;
}

.sidebar a {
    color: var(--text-primary);
    text-decoration: none;
    font-size: 0.9rem;
    display: flex;
    align-items: center;
    gap: 8px;
}

.sidebar a:hover {
    color: var(--accent);
}

.content {
    flex: 1;
    padding: 40px 60px;
    max-width: 1000px;
}

h1 {
    font-size: 2rem;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 12px;
    margin-top: 0;
}

.badge {
    font-size: 0.75rem;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
    text-transform: uppercase;
}

.badge-class { background-color: rgba(63, 185, 80, 0.2); color: var(--badge-class); }
.badge-interface { background-color: rgba(163, 113, 247, 0.2); color: var(--badge-iface); }
.badge-enum { background-color: rgba(240, 136, 62, 0.2); color: var(--badge-enum); }
.badge-function { background-color: rgba(88, 166, 255, 0.2); color: var(--badge-fn); }

pre {
    background-color: var(--code-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 16px;
    overflow-x: auto;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.9rem;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 20px 0;
}

th, td {
    padding: 10px 14px;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
}

th {
    background-color: var(--bg-surface);
    color: var(--text-muted);
    font-weight: 600;
    font-size: 0.85rem;
}

.member-card {
    background-color: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 18px;
    margin-bottom: 20px;
}

.member-card h3 {
    margin-top: 0;
    font-size: 1.1rem;
    font-family: monospace;
}
"#;

    fs::write(output_dir.join("style.css"), css_content)
        .with_context(|| "[E0302] Failed to write style.css")?;

    let mut sidebar_html = String::new();
    let mut grouped_packages: BTreeMap<String, Vec<&DocumentedType>> = BTreeMap::new();
    for doc_type in types {
        grouped_packages
            .entry(doc_type.package_name.clone())
            .or_default()
            .push(doc_type);
    }

    for (pkg_name, pkg_types) in &grouped_packages {
        sidebar_html.push_str(&format!(
            "<div class=\"nav-group\"><div class=\"nav-title\">{pkg_name}</div><ul>"
        ));
        for doc_type in pkg_types {
            let file_name = format!("{}_{}.html", doc_type.package_name.replace('.', "_"), doc_type.name);
            sidebar_html.push_str(&format!(
                "<li><a href=\"{file_name}\"><span class=\"badge badge-{}\">{}</span>{}</a></li>",
                doc_type.kind,
                &doc_type.kind[..1],
                doc_type.name
            ));
        }
        sidebar_html.push_str("</ul></div>");
    }

    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{project_name} API Documentation</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="sidebar">
        <h2><a href="index.html" style="color: inherit; text-decoration: none;">{project_name}</a></h2>
        <div class="project-ver">v{project_version}</div>
        {sidebar_html}
    </div>
    <div class="content">
        <h1>{project_name} API Overview</h1>
        <p>{project_description}</p>
        <h2>Discovered Types</h2>
        <table>
            <thead>
                <tr>
                    <th>Type</th>
                    <th>Kind</th>
                    <th>Package</th>
                    <th>Description</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </div>
</body>
</html>"#,
        types
            .iter()
            .map(|doc_item| {
                let file_name = format!(
                    "{}_{}.html",
                    doc_item.package_name.replace('.', "_"),
                    doc_item.name
                );
                format!(
                    "<tr><td><a href=\"{file_name}\"><code>{}</code></a></td><td><span class=\"badge badge-{}\">{}</span></td><td><code>{}</code></td><td>{}</td></tr>",
                    doc_item.name,
                    doc_item.kind,
                    doc_item.kind,
                    doc_item.package_name,
                    if doc_item.doc.summary.is_empty() { "—" } else { &doc_item.doc.summary }
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    );

    fs::write(output_dir.join("index.html"), index_html)
        .with_context(|| "[E0302] Failed to write index.html")?;

    for doc_type in types {
        let file_name = format!("{}_{}.html", doc_type.package_name.replace('.', "_"), doc_type.name);
        let mut members_html = String::new();

        if !doc_type.enum_variants.is_empty() {
            members_html.push_str("<h2>Variants</h2><table><thead><tr><th>Variant</th><th>Description</th></tr></thead><tbody>");
            for (var_name, var_doc) in &doc_type.enum_variants {
                members_html.push_str(&format!(
                    "<tr><td><code>{var_name}</code></td><td>{}</td></tr>",
                    var_doc.as_deref().unwrap_or("—")
                ));
            }
            members_html.push_str("</tbody></table>");
        }

        if !doc_type.members.is_empty() {
            members_html.push_str("<h2>Members</h2>");
            for member in &doc_type.members {
                let mut tags_html = String::new();
                for tag in &member.doc.tags {
                    match tag {
                        DocTag::Param { name, description } => {
                            tags_html.push_str(&format!("<li><strong><code>{name}</code></strong>: {description}</li>"));
                        }
                        DocTag::Return { description } => {
                            tags_html.push_str(&format!("<li><strong>Returns</strong>: {description}</li>"));
                        }
                        DocTag::Throws { exception_type, description } => {
                            tags_html.push_str(&format!("<li><strong>Throws <code>{exception_type}</code></strong>: {description}</li>"));
                        }
                        DocTag::Deprecated { reason } => {
                            tags_html.push_str(&format!("<li style=\"color: #f85149;\"><strong>Deprecated</strong>: {reason}</li>"));
                        }
                    }
                }
                if !tags_html.is_empty() {
                    tags_html = format!("<ul>{tags_html}</ul>");
                }

                members_html.push_str(&format!(
                    r#"<div class="member-card">
                        <h3>{}</h3>
                        <p>{}</p>
                        {tags_html}
                    </div>"#,
                    member.signature,
                    if member.doc.description.is_empty() { "—" } else { &member.doc.description }
                ));
            }
        }

        let type_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{} - {project_name}</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="sidebar">
        <h2><a href="index.html" style="color: inherit; text-decoration: none;">{project_name}</a></h2>
        <div class="project-ver">v{project_version}</div>
        {sidebar_html}
    </div>
    <div class="content">
        <h1><span class="badge badge-{}">{}</span> {}</h1>
        <p><strong>Package:</strong> <code>{}</code></p>
        <p>{}</p>
        {members_html}
    </div>
</body>
</html>"#,
            doc_type.name,
            doc_type.kind,
            doc_type.kind,
            doc_type.name,
            doc_type.package_name,
            if doc_type.doc.description.is_empty() { "—" } else { &doc_type.doc.description }
        );

        fs::write(output_dir.join(file_name), type_html)
            .with_context(|| "[E0302] Failed to write type HTML documentation")?;
    }

    Ok(())
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn open_in_browser(file_path: &Path) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(file_path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(file_path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", &file_path.to_string_lossy()]).spawn();
    }
}
