//! Pass 1: Global symbol discovery, package outlines, import cataloging, and generic blueprint registration.

#![forbid(unsafe_code)]

use crate::phases::binder::monomorphizer::{GenericBlueprint, GenericBlueprintRegistry};
use crate::phases::binder::symbols::{ImportEntry, SymbolEntry, SymbolId, SymbolTable};
use crate::phases::binder::type_resolver::resolve_type_annotation;
use crate::phases::lexing::Span;
use crate::phases::parser::statements::ProgramAst;
use crate::phases::parser::statements::declarations::{ClassMember, Declaration, Modifier};
use crate::phases::parser::statements::types::TypeAnnotation;
use solix_common::{BindingError, PrimitiveKind, Type};
use std::collections::{HashMap, HashSet};

struct RawAliasInfo {
    source_id: u16,
    package_prefix: String,
    alias_name: String,
    fully_qualified_name: String,
    target_type_name: String,
    is_primitive: bool,
    parameter_names: Vec<String>,
    span: Span,
}

fn get_type_annotation_base_name<'a>(
    annotation: &'a TypeAnnotation,
    source_text: &'a str,
) -> (String, bool) {
    match annotation {
        TypeAnnotation::Primitive { .. } => (String::new(), true),
        TypeAnnotation::Named { path, .. } => {
            let segments: Vec<&str> = path
                .iter()
                .map(|s| {
                    let start = s.start as usize;
                    let end = s.end as usize;
                    if end <= source_text.len() {
                        &source_text[start..end]
                    } else {
                        ""
                    }
                })
                .collect();
            (segments.join("::"), false)
        }
        TypeAnnotation::Generic { base, .. } => get_type_annotation_base_name(base, source_text),
        _ => (String::new(), false),
    }
}

/// Extracts the namespace segments joined by `::` from a sequence of identifier spans.
pub(crate) fn extract_namespace_path(namespace_path: &[Span], source_text: &str) -> String {
    let segments: Vec<&str> = namespace_path
        .iter()
        .map(|span| {
            let start = span.start as usize;
            let end = span.end as usize;
            if end <= source_text.len() {
                &source_text[start..end]
            } else {
                ""
            }
        })
        .collect();
    segments.join("::")
}

/// Computes the effective package prefix combining a file package and declaration namespace.
pub(crate) fn compute_effective_package(file_package: &str, declaration_namespace: &str) -> String {
    if declaration_namespace.is_empty() {
        file_package.to_string()
    } else if file_package.is_empty()
        || declaration_namespace == file_package
        || declaration_namespace.starts_with(&format!("{file_package}::"))
    {
        declaration_namespace.to_string()
    } else {
        format!("{file_package}::{declaration_namespace}")
    }
}

/// Registers a package name into the symbol table's known packages set.
pub(crate) fn register_known_package(symbol_table: &mut SymbolTable, package_name: &str) {
    if !package_name.is_empty() {
        symbol_table.known_packages.insert(package_name.to_string());
    }
}

/// Executes Pass 1 of semantic binding across all source programs in the compilation context.
///
/// Discovers top-level packages, imports, classes, interfaces, enums, aliases, and generic blueprints.
///
/// # Errors
/// Returns an accumulated list of [`BindingError`]s if duplicate symbol definitions were encountered.
pub fn execute_pass1_symbol_discovery(
    source_texts: &[String],
    source_asts: &[Option<ProgramAst>],
    symbol_table: &mut SymbolTable,
    blueprint_registry: &mut GenericBlueprintRegistry,
) -> Result<(), Vec<BindingError>> {
    let mut accumulated_errors = Vec::new();
    let mut raw_aliases = Vec::new();

    for (source_index, ast_option) in source_asts.iter().enumerate() {
        let source_id = source_index as u16;
        let source_text = source_texts
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");

        let program_ast = match ast_option {
            Some(ast) => ast,
            None => continue,
        };

        // 1. Process package statement
        let mut package_prefix = String::new();
        if let Some(package_statement) = &program_ast.package {
            let segments: Vec<&str> = package_statement
                .path
                .iter()
                .map(|span| {
                    let start = span.start as usize;
                    let end = span.end as usize;
                    if end <= source_text.len() {
                        &source_text[start..end]
                    } else {
                        ""
                    }
                })
                .collect();
            package_prefix = segments.join("::");
            register_known_package(symbol_table, &package_prefix);
        }
        if source_index < symbol_table.file_packages.len() {
            symbol_table.file_packages[source_index] = package_prefix.clone();
        }

        // 2. Process import statements
        for import_statement in &program_ast.imports {
            let segments: Vec<&str> = import_statement
                .path
                .iter()
                .map(|span| {
                    let start = span.start as usize;
                    let end = span.end as usize;
                    if end <= source_text.len() {
                        &source_text[start..end]
                    } else {
                        ""
                    }
                })
                .collect();
            let full_import_path = segments.join("::");
            let short_symbol_name = segments.last().copied().unwrap_or("").to_string();

            let alias_name = import_statement.alias.map(|alias_span| {
                let start = alias_span.start as usize;
                let end = alias_span.end as usize;
                if end <= source_text.len() {
                    source_text[start..end].to_string()
                } else {
                    String::new()
                }
            });

            let import_entry = ImportEntry {
                source_id,
                path: full_import_path,
                symbol_name: short_symbol_name,
                alias: alias_name,
                is_wildcard: import_statement.is_wildcard,
                span: import_statement.span,
            };

            if source_index < symbol_table.file_imports.len() {
                symbol_table.file_imports[source_index].push(import_entry);
            }
        }

        // 3. Process aliases
        for alias_statement in &program_ast.aliases {
            let alias_name = if alias_statement.name_span.end as usize <= source_text.len() {
                &source_text[alias_statement.name_span.start as usize
                    ..alias_statement.name_span.end as usize]
            } else {
                "alias"
            };

            let decl_namespace =
                extract_namespace_path(&alias_statement.namespace_path, source_text);
            let effective_package = compute_effective_package(&package_prefix, &decl_namespace);
            register_known_package(symbol_table, &effective_package);

            let fully_qualified_name = if effective_package.is_empty() {
                alias_name.to_string()
            } else {
                format!("{effective_package}::{alias_name}")
            };

            let target_type = resolve_type_annotation(
                source_id,
                source_text,
                &alias_statement.target_type,
                symbol_table,
            );

            let alias_symbol = SymbolEntry {
                id: SymbolId(0),
                name: alias_name.to_string(),
                fully_qualified_name: fully_qualified_name.clone(),
                source_id,
                span: alias_statement.span,
                modifiers: Vec::new(),
                symbol_type: Type::Alias {
                    alias_name: alias_name.to_string(),
                    target_type: Box::new(target_type),
                },
                parent_symbol: None,
                members: Vec::new(),
                is_static: false,
                is_const: false,
                slot_index: None,
            };

            if symbol_table
                .global_names
                .contains_key(&fully_qualified_name)
            {
                accumulated_errors.push(BindingError::DuplicateSymbol {
                    symbol_name: fully_qualified_name.clone(),
                    span: alias_statement.span.to_source_span(),
                });
            } else {
                symbol_table.insert_symbol(alias_symbol);
            }

            let (target_type_name, is_primitive) =
                get_type_annotation_base_name(&alias_statement.target_type, source_text);
            let parameter_names: Vec<String> = alias_statement
                .generic_params
                .iter()
                .map(|p_span| {
                    let start = p_span.start as usize;
                    let end = p_span.end as usize;
                    if end <= source_text.len() {
                        source_text[start..end].to_string()
                    } else {
                        "T".to_string()
                    }
                })
                .collect();

            raw_aliases.push(RawAliasInfo {
                source_id,
                package_prefix: effective_package.clone(),
                alias_name: alias_name.to_string(),
                fully_qualified_name,
                target_type_name,
                is_primitive,
                parameter_names,
                span: alias_statement.span,
            });
        }

        // 4. Process top-level declarations
        for declaration in &program_ast.declarations {
            match declaration {
                Declaration::Class(class_declaration) => {
                    let class_name =
                        if class_declaration.name_span.end as usize <= source_text.len() {
                            &source_text[class_declaration.name_span.start as usize
                                ..class_declaration.name_span.end as usize]
                        } else {
                            "class"
                        };

                    let decl_namespace =
                        extract_namespace_path(&class_declaration.namespace_path, source_text);
                    let effective_package =
                        compute_effective_package(&package_prefix, &decl_namespace);
                    register_known_package(symbol_table, &effective_package);

                    let fully_qualified_name = if effective_package.is_empty() {
                        class_name.to_string()
                    } else {
                        format!("{effective_package}::{class_name}")
                    };

                    // If generic, register as blueprint
                    if !class_declaration.generic_params.is_empty() {
                        let parameter_names: Vec<String> = class_declaration
                            .generic_params
                            .iter()
                            .map(|param| {
                                let start = param.name_span.start as usize;
                                let end = param.name_span.end as usize;
                                if end <= source_text.len() {
                                    source_text[start..end].to_string()
                                } else {
                                    "T".to_string()
                                }
                            })
                            .collect();

                        let blueprint = GenericBlueprint {
                            source_id,
                            package_prefix: effective_package.clone(),
                            name: class_name.to_string(),
                            fully_qualified_name: fully_qualified_name.clone(),
                            parameter_names,
                            class_ast: class_declaration.clone(),
                        };
                        blueprint_registry.register_blueprint(blueprint);
                    }

                    if symbol_table
                        .global_names
                        .contains_key(&fully_qualified_name)
                    {
                        accumulated_errors.push(BindingError::DuplicateSymbol {
                            symbol_name: fully_qualified_name,
                            span: class_declaration.span.to_source_span(),
                        });
                        continue;
                    }

                    let class_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: class_name.to_string(),
                        fully_qualified_name: fully_qualified_name.clone(),
                        source_id,
                        span: class_declaration.span,
                        modifiers: class_declaration.modifiers.clone(),
                        symbol_type: Type::Class {
                            symbol_name: class_name.to_string(),
                            type_arguments: Vec::new(),
                        },
                        parent_symbol: None,
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                    };
                    let class_id = symbol_table.insert_symbol(class_symbol);

                    for member in &class_declaration.members {
                        if let ClassMember::Enum(enum_declaration) = member {
                            let enum_name =
                                if enum_declaration.name_span.end as usize <= source_text.len() {
                                    &source_text[enum_declaration.name_span.start as usize
                                        ..enum_declaration.name_span.end as usize]
                                } else {
                                    "enum"
                                };
                            let nested_fqn = format!("{fully_qualified_name}::{enum_name}");
                            let nested_sym = SymbolEntry {
                                id: SymbolId(0),
                                name: enum_name.to_string(),
                                fully_qualified_name: nested_fqn.clone(),
                                source_id,
                                span: enum_declaration.span,
                                modifiers: enum_declaration.modifiers.clone(),
                                symbol_type: Type::Enum {
                                    symbol_name: nested_fqn.clone(),
                                },
                                parent_symbol: Some(class_id),
                                members: Vec::new(),
                                is_static: true,
                                is_const: false,
                                slot_index: None,
                            };
                            let enum_id = symbol_table.insert_symbol(nested_sym);
                            symbol_table
                                .global_names
                                .entry(enum_name.to_string())
                                .or_insert(enum_id);
                            if let Some(cls_entry) = symbol_table.get_symbol_mut(class_id) {
                                cls_entry.members.push(enum_id);
                            }
                        }
                        if let ClassMember::Class(nested_class) = member {
                            let nested_class_name =
                                if nested_class.name_span.end as usize <= source_text.len() {
                                    &source_text[nested_class.name_span.start as usize
                                        ..nested_class.name_span.end as usize]
                                } else {
                                    "class"
                                };
                            let nested_fqn = format!("{fully_qualified_name}::{nested_class_name}");
                            let nested_dot = format!("{fully_qualified_name}.{nested_class_name}");
                            let nested_sym = SymbolEntry {
                                id: SymbolId(0),
                                name: nested_class_name.to_string(),
                                fully_qualified_name: nested_fqn.clone(),
                                source_id,
                                span: nested_class.span,
                                modifiers: nested_class.modifiers.clone(),
                                symbol_type: Type::Class {
                                    symbol_name: nested_fqn.clone(),
                                    type_arguments: Vec::new(),
                                },
                                parent_symbol: Some(class_id),
                                members: Vec::new(),
                                is_static: false,
                                is_const: false,
                                slot_index: None,
                            };
                            let nested_id = symbol_table.insert_symbol(nested_sym);
                            symbol_table
                                .global_names
                                .entry(nested_class_name.to_string())
                                .or_insert(nested_id);
                            symbol_table
                                .global_names
                                .entry(nested_fqn.clone())
                                .or_insert(nested_id);
                            symbol_table
                                .global_names
                                .entry(nested_dot.clone())
                                .or_insert(nested_id);
                            if let Some(cls_entry) = symbol_table.get_symbol_mut(class_id) {
                                cls_entry.members.push(nested_id);
                            }
                        }
                        if let ClassMember::Interface(nested_iface) = member {
                            let nested_iface_name =
                                if nested_iface.name_span.end as usize <= source_text.len() {
                                    &source_text[nested_iface.name_span.start as usize
                                        ..nested_iface.name_span.end as usize]
                                } else {
                                    "interface"
                                };
                            let nested_fqn = format!("{fully_qualified_name}::{nested_iface_name}");
                            let nested_dot = format!("{fully_qualified_name}.{nested_iface_name}");
                            let nested_sym = SymbolEntry {
                                id: SymbolId(0),
                                name: nested_iface_name.to_string(),
                                fully_qualified_name: nested_fqn.clone(),
                                source_id,
                                span: nested_iface.span,
                                modifiers: nested_iface.modifiers.clone(),
                                symbol_type: Type::Interface {
                                    symbol_name: nested_fqn.clone(),
                                    type_arguments: Vec::new(),
                                },
                                parent_symbol: Some(class_id),
                                members: Vec::new(),
                                is_static: false,
                                is_const: false,
                                slot_index: None,
                            };
                            let nested_id = symbol_table.insert_symbol(nested_sym);
                            symbol_table
                                .global_names
                                .entry(nested_iface_name.to_string())
                                .or_insert(nested_id);
                            symbol_table
                                .global_names
                                .entry(nested_fqn.clone())
                                .or_insert(nested_id);
                            symbol_table
                                .global_names
                                .entry(nested_dot.clone())
                                .or_insert(nested_id);
                            if let Some(cls_entry) = symbol_table.get_symbol_mut(class_id) {
                                cls_entry.members.push(nested_id);
                            }
                        }
                    }
                }
                Declaration::Interface(interface_declaration) => {
                    let interface_name =
                        if interface_declaration.name_span.end as usize <= source_text.len() {
                            &source_text[interface_declaration.name_span.start as usize
                                ..interface_declaration.name_span.end as usize]
                        } else {
                            "interface"
                        };

                    let decl_namespace =
                        extract_namespace_path(&interface_declaration.namespace_path, source_text);
                    let effective_package =
                        compute_effective_package(&package_prefix, &decl_namespace);
                    register_known_package(symbol_table, &effective_package);

                    let fully_qualified_name = if effective_package.is_empty() {
                        interface_name.to_string()
                    } else {
                        format!("{effective_package}::{interface_name}")
                    };

                    if symbol_table
                        .global_names
                        .contains_key(&fully_qualified_name)
                    {
                        accumulated_errors.push(BindingError::DuplicateSymbol {
                            symbol_name: fully_qualified_name,
                            span: interface_declaration.span.to_source_span(),
                        });
                        continue;
                    }

                    let interface_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: interface_name.to_string(),
                        fully_qualified_name: fully_qualified_name.clone(),
                        source_id,
                        span: interface_declaration.span,
                        modifiers: interface_declaration.modifiers.clone(),
                        symbol_type: Type::Interface {
                            symbol_name: interface_name.to_string(),
                            type_arguments: Vec::new(),
                        },
                        parent_symbol: None,
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                    };
                    symbol_table.insert_symbol(interface_symbol);
                }
                Declaration::Enum(enum_declaration) => {
                    let enum_name = if enum_declaration.name_span.end as usize <= source_text.len()
                    {
                        &source_text[enum_declaration.name_span.start as usize
                            ..enum_declaration.name_span.end as usize]
                    } else {
                        "enum"
                    };

                    let decl_namespace =
                        extract_namespace_path(&enum_declaration.namespace_path, source_text);
                    let effective_package =
                        compute_effective_package(&package_prefix, &decl_namespace);
                    register_known_package(symbol_table, &effective_package);

                    let fully_qualified_name = if effective_package.is_empty() {
                        enum_name.to_string()
                    } else {
                        format!("{effective_package}::{enum_name}")
                    };

                    if symbol_table
                        .global_names
                        .contains_key(&fully_qualified_name)
                    {
                        accumulated_errors.push(BindingError::DuplicateSymbol {
                            symbol_name: fully_qualified_name,
                            span: enum_declaration.span.to_source_span(),
                        });
                        continue;
                    }

                    let enum_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: enum_name.to_string(),
                        fully_qualified_name: fully_qualified_name.clone(),
                        source_id,
                        span: enum_declaration.span,
                        modifiers: enum_declaration.modifiers.clone(),
                        symbol_type: Type::Enum {
                            symbol_name: enum_name.to_string(),
                        },
                        parent_symbol: None,
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                    };
                    let enum_id = symbol_table.insert_symbol(enum_symbol);

                    // Register enum variants
                    let mut seen_variants = std::collections::HashSet::new();
                    for (variant_index, variant) in enum_declaration.variants.iter().enumerate() {
                        let variant_name = if variant.name_span.end as usize <= source_text.len() {
                            &source_text
                                [variant.name_span.start as usize..variant.name_span.end as usize]
                        } else {
                            "variant"
                        };

                        if !seen_variants.insert(variant_name.to_string()) {
                            accumulated_errors.push(BindingError::DuplicateSymbol {
                                symbol_name: format!("{fully_qualified_name}::{variant_name}"),
                                span: variant.span.to_source_span(),
                            });
                            continue;
                        }

                        let variant_symbol = SymbolEntry {
                            id: SymbolId(0),
                            name: variant_name.to_string(),
                            fully_qualified_name: format!("{fully_qualified_name}::{variant_name}"),
                            source_id,
                            span: variant.span,
                            modifiers: enum_declaration.modifiers.clone(),
                            symbol_type: Type::Enum {
                                symbol_name: enum_name.to_string(),
                            },
                            parent_symbol: Some(enum_id),
                            members: Vec::new(),
                            is_static: true,
                            is_const: true,
                            slot_index: Some(variant_index as u32),
                        };
                        let variant_id = symbol_table.insert_symbol(variant_symbol);
                        if let Some(parent_entry) = symbol_table.get_symbol_mut(enum_id) {
                            parent_entry.members.push(variant_id);
                        }
                    }
                }
                Declaration::Method(method_declaration) => {
                    let method_name =
                        if method_declaration.name_span.end as usize <= source_text.len() {
                            &source_text[method_declaration.name_span.start as usize
                                ..method_declaration.name_span.end as usize]
                        } else {
                            "function"
                        };

                    let decl_namespace =
                        extract_namespace_path(&method_declaration.namespace_path, source_text);
                    let effective_package =
                        compute_effective_package(&package_prefix, &decl_namespace);
                    register_known_package(symbol_table, &effective_package);

                    let fully_qualified_name = if effective_package.is_empty() {
                        method_name.to_string()
                    } else {
                        format!("{effective_package}::{method_name}")
                    };

                    let return_type = resolve_type_annotation(
                        source_id,
                        source_text,
                        &method_declaration.return_type,
                        symbol_table,
                    );
                    let parameter_types = method_declaration
                        .parameters
                        .iter()
                        .map(|param| {
                            if param.is_variadic {
                                Type::Primitive(PrimitiveKind::TuplePrimitive)
                            } else {
                                resolve_type_annotation(
                                    source_id,
                                    source_text,
                                    &param.param_type,
                                    symbol_table,
                                )
                            }
                        })
                        .collect();

                    let method_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: method_name.to_string(),
                        fully_qualified_name: fully_qualified_name.clone(),
                        source_id,
                        span: method_declaration.span,
                        modifiers: method_declaration.modifiers.clone(),
                        symbol_type: Type::FunctionPointer {
                            return_type: Box::new(return_type),
                            parameter_types,
                        },
                        parent_symbol: None,
                        members: Vec::new(),
                        is_static: true,
                        is_const: false,
                        slot_index: None,
                    };
                    symbol_table.insert_symbol(method_symbol);
                }
                Declaration::Field(field_declaration) => {
                    let field_name =
                        if field_declaration.name_span.end as usize <= source_text.len() {
                            &source_text[field_declaration.name_span.start as usize
                                ..field_declaration.name_span.end as usize]
                        } else {
                            "global"
                        };

                    let decl_namespace =
                        extract_namespace_path(&field_declaration.namespace_path, source_text);
                    let effective_package =
                        compute_effective_package(&package_prefix, &decl_namespace);
                    register_known_package(symbol_table, &effective_package);

                    let fully_qualified_name = if effective_package.is_empty() {
                        field_name.to_string()
                    } else {
                        format!("{effective_package}::{field_name}")
                    };
                    let field_type = resolve_type_annotation(
                        source_id,
                        source_text,
                        &field_declaration.field_type,
                        symbol_table,
                    );
                    let is_const = field_declaration.modifiers.contains(&Modifier::Const);
                    let field_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: field_name.to_string(),
                        fully_qualified_name: fully_qualified_name.clone(),
                        source_id,
                        span: field_declaration.span,
                        modifiers: field_declaration.modifiers.clone(),
                        symbol_type: field_type,
                        parent_symbol: None,
                        members: Vec::new(),
                        is_static: true,
                        is_const,
                        slot_index: None,
                    };
                    let fid = symbol_table.insert_symbol(field_symbol);
                    symbol_table
                        .global_names
                        .entry(field_name.to_string())
                        .or_insert(fid);
                }
                _ => {}
            }
        }
    }

    // 5. Post-Pass Validation: Aliases
    let mut alias_targets: HashMap<String, String> = HashMap::new();
    let mut all_alias_names: HashSet<String> = HashSet::new();
    for alias in &raw_aliases {
        all_alias_names.insert(alias.alias_name.clone());
        all_alias_names.insert(alias.fully_qualified_name.clone());
        alias_targets.insert(alias.alias_name.clone(), alias.target_type_name.clone());
        alias_targets.insert(
            alias.fully_qualified_name.clone(),
            alias.target_type_name.clone(),
        );
    }

    let is_primitive = |name: &str| -> bool {
        matches!(
            name,
            "string"
                | "void"
                | "bool"
                | "char"
                | "int8"
                | "byte"
                | "int16"
                | "short"
                | "int32"
                | "int"
                | "int64"
                | "long"
                | "uint8"
                | "ubyte"
                | "uint16"
                | "ushort"
                | "uint32"
                | "uint"
                | "uint64"
                | "ulong"
                | "float32"
                | "float"
                | "float64"
                | "double"
        )
    };

    for alias in &raw_aliases {
        let target = &alias.target_type_name;
        if !alias.is_primitive && !target.is_empty() && !is_primitive(target) {
            let target_dot = target.replace("::", ".");
            let is_resolved = all_alias_names.contains(target)
                || all_alias_names.contains(&target_dot)
                || symbol_table.global_names.contains_key(target)
                || symbol_table.lookup_global(target).is_some()
                || symbol_table
                    .resolve_symbol(alias.source_id, target)
                    .is_some()
                || symbol_table
                    .resolve_symbol(alias.source_id, &target_dot)
                    .is_some()
                || blueprint_registry.has_blueprint(target)
                || blueprint_registry.has_blueprint(&target_dot);

            if !is_resolved {
                accumulated_errors.push(BindingError::UnresolvedAliasTarget {
                    target_name: target.clone(),
                    span: alias.span.to_source_span(),
                });
            }
        }

        // Circular alias cycle check
        let mut visited = HashSet::new();
        let mut curr = alias.alias_name.clone();
        visited.insert(curr.clone());
        while let Some(next) = alias_targets.get(&curr) {
            if next.is_empty() || is_primitive(next) {
                break;
            }
            if visited.contains(next) {
                accumulated_errors.push(BindingError::CircularAlias {
                    alias_name: alias.alias_name.clone(),
                    span: alias.span.to_source_span(),
                });
                break;
            }
            visited.insert(next.clone());
            curr = next.clone();
        }

        // Register generic blueprint for parameterized generic alias
        if !alias.parameter_names.is_empty() {
            let target_blueprint_opt = blueprint_registry
                .blueprints
                .get(&alias.target_type_name)
                .cloned()
                .or_else(|| {
                    let target_dot = alias.target_type_name.replace("::", ".");
                    blueprint_registry.blueprints.get(&target_dot).cloned()
                });

            if let Some(target_blueprint) = target_blueprint_opt {
                let alias_blueprint = GenericBlueprint {
                    source_id: alias.source_id,
                    package_prefix: alias.package_prefix.clone(),
                    name: alias.alias_name.clone(),
                    fully_qualified_name: alias.fully_qualified_name.clone(),
                    parameter_names: alias.parameter_names.clone(),
                    class_ast: target_blueprint.class_ast,
                };
                blueprint_registry.register_blueprint(alias_blueprint);
            }
        }
    }

    // 6. Post-Pass Validation: Imports
    for imports in &symbol_table.file_imports {
        for import_entry in imports {
            if import_entry.is_wildcard {
                let path = &import_entry.path;
                let is_std = path.starts_with("std") || path.starts_with("solix");
                let is_known_pkg = symbol_table.known_packages.contains(path)
                    || symbol_table
                        .file_packages
                        .iter()
                        .any(|pkg_name| pkg_name == path);
                if !is_std && !is_known_pkg {
                    accumulated_errors.push(BindingError::CannotResolveImport {
                        import_path: path.replace("::", "."),
                        span: import_entry.span.to_source_span(),
                    });
                }
            } else {
                let path = &import_entry.path;
                let is_resolved = symbol_table.lookup_global(path).is_some();
                if !is_resolved {
                    let parts: Vec<&str> = path.split("::").collect();
                    if parts.len() > 1 {
                        let package_name = parts[..parts.len() - 1].join("::");
                        let pkg_exists = symbol_table.known_packages.contains(&package_name)
                            || symbol_table
                                .file_packages
                                .iter()
                                .any(|pkg_name| pkg_name == &package_name);
                        if pkg_exists {
                            accumulated_errors.push(BindingError::undefined_symbol(
                                format!(
                                    "Symbol '{}' not found in package '{}'",
                                    import_entry.symbol_name,
                                    package_name.replace("::", ".")
                                ),
                                import_entry.span,
                            ));
                        } else {
                            accumulated_errors.push(BindingError::CannotResolveImport {
                                import_path: path.replace("::", "."),
                                span: import_entry.span.to_source_span(),
                            });
                        }
                    } else {
                        accumulated_errors.push(BindingError::CannotResolveImport {
                            import_path: path.replace("::", "."),
                            span: import_entry.span.to_source_span(),
                        });
                    }
                }
            }
        }
    }

    if !accumulated_errors.is_empty() {
        return Err(accumulated_errors);
    }

    Ok(())
}
