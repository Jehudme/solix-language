//! Pass 2: Type hierarchy resolution, inheritance cycle detection, VTable/ITable generation, and memory layout.

#![forbid(unsafe_code)]

use crate::BindingError;
use crate::phases::binder::layout::ClassLayout;
use crate::phases::binder::passes::pass1_symbol_discovery::{
    compute_effective_package, extract_namespace_path,
};
use crate::phases::binder::symbols::{SymbolEntry, SymbolId, SymbolTable};
use crate::phases::binder::type_resolver::resolve_type_annotation;
use crate::phases::parser::statements::ProgramAst;
use crate::phases::parser::statements::declarations::{
    ClassDeclaration, ClassMember, Declaration, InterfaceDeclaration, Modifier,
};
use crate::phases::parser::statements::expressions::{Expression, LiteralValue};
use solix_common::{PrimitiveKind, TokenKind, Type};
use std::collections::{HashMap, HashSet};

fn evaluate_integer_constant(expr: &Expression) -> Option<i64> {
    match expr {
        Expression::Literal {
            value: LiteralValue::Int(value),
            ..
        } => Some(*value),
        Expression::Literal {
            value: LiteralValue::UInt(value),
            ..
        } => Some(*value as i64),
        Expression::Unary {
            operator: TokenKind::OperatorMinus,
            operand,
            ..
        } => evaluate_integer_constant(operand).map(|val| -val),
        Expression::Unary {
            operator: TokenKind::OperatorPlus,
            operand,
            ..
        } => evaluate_integer_constant(operand),
        _ => None,
    }
}

fn find_base_method(
    symbol_table: &SymbolTable,
    class_inheritance_edges: &HashMap<SymbolId, SymbolId>,
    class_symbol_id: SymbolId,
    method_name: &str,
) -> Option<SymbolEntry> {
    let mut current_class_id = class_inheritance_edges.get(&class_symbol_id).copied();
    while let Some(base_id) = current_class_id {
        if let Some(base_entry) = symbol_table.get_symbol(base_id) {
            for &member_id in &base_entry.members {
                if let Some(member_entry) = symbol_table.get_symbol(member_id)
                    && member_entry.name == method_name
                {
                    return Some(member_entry.clone());
                }
            }
        }
        current_class_id = class_inheritance_edges.get(&base_id).copied();
    }
    None
}

/// Executes Pass 2 of semantic binding across all source programs in the compilation context.
///
/// Resolves superclasses, detects cyclic inheritance, constructs VTables, validates interface conformance,
/// and computes class memory layouts.
/// Internal diagnostic collector tracking error occurrences per originating source file.
struct HierarchyErrorCollector {
    current_source_id: u16,
    errors: Vec<(u16, BindingError)>,
}

impl HierarchyErrorCollector {
    fn new() -> Self {
        Self {
            current_source_id: 0,
            errors: Vec::new(),
        }
    }

    fn push(&mut self, error: BindingError) {
        self.errors.push((self.current_source_id, error));
    }

    fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

///
/// # Errors
/// Returns an accumulated list of [`BindingError`]s if cyclic inheritance, invalid base types, or
/// unfulfilled interface contracts are detected.
pub fn execute_pass2_type_hierarchy(
    source_texts: &[String],
    source_asts: &[Option<ProgramAst>],
    symbol_table: &mut SymbolTable,
    layouts: &mut HashMap<SymbolId, ClassLayout>,
    next_vtable_id: &mut u32,
) -> Result<(), Vec<(u16, BindingError)>> {
    let mut accumulated_errors = HierarchyErrorCollector::new();

    // Map: class SymbolId -> super class SymbolId
    let mut class_inheritance_edges: HashMap<SymbolId, SymbolId> = HashMap::new();
    let mut class_ast_map: HashMap<SymbolId, (u16, &ClassDeclaration)> = HashMap::new();
    let mut interface_by_name: HashMap<String, (u16, &InterfaceDeclaration)> = HashMap::new();

    // 1. Resolve base classes and interfaces
    for (source_index, ast_option) in source_asts.iter().enumerate() {
        let source_id = source_index as u16;
        accumulated_errors.current_source_id = source_id;
        let source_text = source_texts
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");

        let program_ast = match ast_option {
            Some(ast) => ast,
            None => continue,
        };

        let file_package = symbol_table
            .file_packages
            .get(source_index)
            .cloned()
            .unwrap_or_default();

        for declaration in &program_ast.declarations {
            if let Declaration::Interface(iface_decl) = declaration {
                let iface_name = if iface_decl.name_span.end as usize <= source_text.len() {
                    &source_text
                        [iface_decl.name_span.start as usize..iface_decl.name_span.end as usize]
                } else {
                    "interface"
                };

                let decl_namespace =
                    extract_namespace_path(&iface_decl.namespace_path, source_text);
                let effective_package = compute_effective_package(&file_package, &decl_namespace);
                let fqn = if effective_package.is_empty() {
                    iface_name.to_string()
                } else {
                    format!("{effective_package}::{iface_name}")
                };
                let previous_active_package = symbol_table.active_package.clone();
                if !effective_package.is_empty() {
                    symbol_table.active_package = Some(effective_package);
                }

                // Check modifiers on interface
                for modifier in &iface_decl.modifiers {
                    if *modifier == Modifier::Native {
                        accumulated_errors.push(BindingError::NativeOnNonMethodOrFunction {
                            span: iface_decl.span.to_source_span(),
                        });
                    }
                    if *modifier == Modifier::Abstract {
                        accumulated_errors.push(BindingError::ConflictingModifierCombination {
                            modifier1: "abstract".to_string(),
                            modifier2: "interface".to_string(),
                            span: iface_decl.span.to_source_span(),
                        });
                    }
                }

                // Validate super interfaces (extends)
                for super_iface_annot in &iface_decl.super_interfaces {
                    let resolved = resolve_type_annotation(
                        source_id,
                        source_text,
                        super_iface_annot,
                        symbol_table,
                    );
                    match resolved {
                        Type::Interface { .. } => {}
                        Type::Unresolved(ref name) => {
                            accumulated_errors.push(BindingError::ImplementedTypeMustBeInterface {
                                found: name.clone(),
                                span: super_iface_annot.span().to_source_span(),
                            });
                        }
                        other => {
                            let type_name = if let Some(unres) = other.first_unresolved_name() {
                                unres.to_string()
                            } else {
                                other.to_string()
                            };
                            accumulated_errors.push(BindingError::ImplementedTypeMustBeInterface {
                                found: type_name,
                                span: super_iface_annot.span().to_source_span(),
                            });
                        }
                    }
                }

                let iface_symbol_id_opt = symbol_table
                    .lookup_global(&fqn)
                    .or_else(|| symbol_table.resolve_symbol(source_id, iface_name));

                if let Some(iface_symbol_id) = iface_symbol_id_opt {
                    let iface_vtable_id = *next_vtable_id;
                    *next_vtable_id += 1;
                    let full_name = symbol_table
                        .get_symbol(iface_symbol_id)
                        .map(|entry| entry.fully_qualified_name.clone())
                        .unwrap_or_else(|| iface_name.to_string());
                    let mut iface_layout =
                        ClassLayout::new(iface_symbol_id, full_name.clone(), iface_vtable_id);
                    let mut iface_child_ids = Vec::new();

                    let iface_generic_names: HashSet<String> = iface_decl
                        .generic_params
                        .iter()
                        .filter_map(|param| {
                            let span = param.name_span;
                            if span.end as usize <= source_text.len() {
                                Some(
                                    source_text[span.start as usize..span.end as usize].to_string(),
                                )
                            } else {
                                None
                            }
                        })
                        .collect();

                    for (slot_index, method) in iface_decl.methods.iter().enumerate() {
                        let method_name = if method.name_span.end as usize <= source_text.len() {
                            &source_text
                                [method.name_span.start as usize..method.name_span.end as usize]
                        } else {
                            "method"
                        };
                        let method_return_type = resolve_type_annotation(
                            source_id,
                            source_text,
                            &method.return_type,
                            symbol_table,
                        );
                        if let Some(unres) = method_return_type.first_unresolved_name()
                            && !iface_generic_names.contains(unres)
                        {
                            accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                                type_name: unres.to_string(),
                                span: method.return_type.span().to_source_span(),
                            });
                        }
                        let mut method_param_types = Vec::new();
                        for param in &method.parameters {
                            let param_type = if param.is_variadic {
                                Type::Primitive(PrimitiveKind::TuplePrimitive)
                            } else {
                                resolve_type_annotation(
                                    source_id,
                                    source_text,
                                    &param.param_type,
                                    symbol_table,
                                )
                            };
                            if param_type == Type::Primitive(PrimitiveKind::Void) {
                                accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                                    span: param.span.to_source_span(),
                                });
                            }
                            if let Some(unres) = param_type.first_unresolved_name()
                                && !iface_generic_names.contains(unres)
                            {
                                accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                                    type_name: unres.to_string(),
                                    span: param.span.to_source_span(),
                                });
                            }
                            method_param_types.push(param_type);
                        }
                        for modifier in &method.modifiers {
                            if matches!(
                                modifier,
                                Modifier::Private
                                    | Modifier::Protected
                                    | Modifier::Static
                                    | Modifier::Native
                                    | Modifier::Inline
                                    | Modifier::Virtual
                                    | Modifier::Override
                                    | Modifier::Const
                                    | Modifier::Weak
                            ) {
                                accumulated_errors.push(
                                    BindingError::ModifierNotApplicableToDeclaration {
                                        modifier: modifier.as_str().to_string(),
                                        declaration_kind: "interface method".to_string(),
                                        span: method.span.to_source_span(),
                                    },
                                );
                            }
                        }
                        iface_layout
                            .method_slots
                            .insert(method_name.to_string(), slot_index);

                        let method_symbol = SymbolEntry {
                            id: SymbolId(0),
                            name: method_name.to_string(),
                            fully_qualified_name: format!("{full_name}::{method_name}"),
                            source_id,
                            span: method.span,
                            modifiers: method.modifiers.clone(),
                            symbol_type: Type::FunctionPointer {
                                return_type: Box::new(method_return_type),
                                parameter_types: method_param_types,
                            },
                            parent_symbol: Some(iface_symbol_id),
                            members: Vec::new(),
                            is_static: false,
                            is_const: false,
                            slot_index: Some(slot_index as u32),
                        };
                        let method_id = symbol_table.insert_symbol(method_symbol);
                        iface_child_ids.push(method_id);
                    }

                    if let Some(iface_sym) = symbol_table.get_symbol_mut(iface_symbol_id) {
                        iface_sym.members = iface_child_ids;
                    }

                    interface_by_name.insert(iface_name.to_string(), (source_id, iface_decl));
                    interface_by_name.insert(full_name, (source_id, iface_decl));
                    layouts.insert(iface_symbol_id, iface_layout);
                }

                symbol_table.active_package = previous_active_package;
            }

            if let Declaration::Method(method_declaration) = declaration {
                let method_name = if method_declaration.name_span.end as usize <= source_text.len()
                {
                    &source_text[method_declaration.name_span.start as usize
                        ..method_declaration.name_span.end as usize]
                } else {
                    "function"
                };

                let decl_namespace =
                    extract_namespace_path(&method_declaration.namespace_path, source_text);
                let effective_package = compute_effective_package(&file_package, &decl_namespace);
                let fqn = if effective_package.is_empty() {
                    method_name.to_string()
                } else {
                    format!("{effective_package}::{method_name}")
                };
                let previous_active_package = symbol_table.active_package.clone();
                if !effective_package.is_empty() {
                    symbol_table.active_package = Some(effective_package);
                }

                let method_generic_names: HashSet<String> = method_declaration
                    .generic_params
                    .iter()
                    .filter_map(|param| {
                        let span = param.name_span;
                        if span.end as usize <= source_text.len() {
                            Some(source_text[span.start as usize..span.end as usize].to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                let return_type = resolve_type_annotation(
                    source_id,
                    source_text,
                    &method_declaration.return_type,
                    symbol_table,
                );
                if let Some(unres) = return_type.first_unresolved_name()
                    && !method_generic_names.contains(unres)
                {
                    accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                        type_name: unres.to_string(),
                        span: method_declaration.return_type.span().to_source_span(),
                    });
                }
                let parameter_types: Vec<Type> = method_declaration
                    .parameters
                    .iter()
                    .map(|param| {
                        let param_type = if param.is_variadic {
                            Type::Primitive(PrimitiveKind::TuplePrimitive)
                        } else {
                            resolve_type_annotation(
                                source_id,
                                source_text,
                                &param.param_type,
                                symbol_table,
                            )
                        };
                        if param_type == Type::Primitive(PrimitiveKind::Void) {
                            accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                                span: param.span.to_source_span(),
                            });
                        }
                        if let Some(unres) = param_type.first_unresolved_name()
                            && !method_generic_names.contains(unres)
                        {
                            accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                                type_name: unres.to_string(),
                                span: param.span.to_source_span(),
                            });
                        }
                        param_type
                    })
                    .collect();

                // Modifiers on free function
                for modifier in &method_declaration.modifiers {
                    if matches!(modifier, Modifier::Const | Modifier::Weak) {
                        accumulated_errors.push(BindingError::ModifierNotApplicableToDeclaration {
                            modifier: modifier.as_str().to_string(),
                            declaration_kind: "function".to_string(),
                            span: method_declaration.span.to_source_span(),
                        });
                    }
                    if *modifier == Modifier::Virtual {
                        accumulated_errors.push(BindingError::VirtualOnNonMethod {
                            span: method_declaration.span.to_source_span(),
                        });
                    }
                    if *modifier == Modifier::Override {
                        accumulated_errors.push(BindingError::OverrideOnNonMethod {
                            span: method_declaration.span.to_source_span(),
                        });
                    }
                }

                let method_symbol_id_opt = symbol_table
                    .lookup_global(&fqn)
                    .or_else(|| symbol_table.resolve_symbol(source_id, method_name));
                if let Some(sym_id) = method_symbol_id_opt
                    && let Some(entry) = symbol_table.get_symbol_mut(sym_id)
                {
                    entry.symbol_type = Type::FunctionPointer {
                        return_type: Box::new(return_type),
                        parameter_types: parameter_types.clone(),
                    };
                }

                symbol_table.active_package = previous_active_package;
            }

            if let Declaration::Field(field_declaration) = declaration {
                let decl_namespace =
                    extract_namespace_path(&field_declaration.namespace_path, source_text);
                let effective_package = compute_effective_package(&file_package, &decl_namespace);
                let previous_active_package = symbol_table.active_package.clone();
                if !effective_package.is_empty() {
                    symbol_table.active_package = Some(effective_package);
                }

                let field_type = resolve_type_annotation(
                    source_id,
                    source_text,
                    &field_declaration.field_type,
                    symbol_table,
                );
                if field_type == Type::Primitive(PrimitiveKind::Void) {
                    accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                        span: field_declaration.span.to_source_span(),
                    });
                }
                if let Some(unres) = field_type.first_unresolved_name() {
                    accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                        type_name: unres.to_string(),
                        span: field_declaration.span.to_source_span(),
                    });
                }
                for modifier in &field_declaration.modifiers {
                    if matches!(
                        modifier,
                        Modifier::Virtual
                            | Modifier::Override
                            | Modifier::Abstract
                            | Modifier::Inline
                            | Modifier::Weak
                    ) {
                        accumulated_errors.push(BindingError::ModifierNotApplicableToDeclaration {
                            modifier: modifier.as_str().to_string(),
                            declaration_kind: "field".to_string(),
                            span: field_declaration.span.to_source_span(),
                        });
                    }
                }

                symbol_table.active_package = previous_active_package;
            }

            if let Declaration::Enum(enum_declaration) = declaration {
                let enum_name = if enum_declaration.name_span.end as usize <= source_text.len() {
                    &source_text[enum_declaration.name_span.start as usize
                        ..enum_declaration.name_span.end as usize]
                } else {
                    "enum"
                };

                let decl_namespace =
                    extract_namespace_path(&enum_declaration.namespace_path, source_text);
                let effective_package = compute_effective_package(&file_package, &decl_namespace);
                let fqn = if effective_package.is_empty() {
                    enum_name.to_string()
                } else {
                    format!("{effective_package}::{enum_name}")
                };
                let previous_active_package = symbol_table.active_package.clone();
                if !effective_package.is_empty() {
                    symbol_table.active_package = Some(effective_package);
                }

                for modifier in &enum_declaration.modifiers {
                    if *modifier == Modifier::Native {
                        accumulated_errors.push(BindingError::NativeOnNonMethodOrFunction {
                            span: enum_declaration.span.to_source_span(),
                        });
                    }
                    if *modifier == Modifier::Extern {
                        accumulated_errors.push(BindingError::ModifierNotApplicableToDeclaration {
                            modifier: "extern".to_string(),
                            declaration_kind: "enum".to_string(),
                            span: enum_declaration.span.to_source_span(),
                        });
                    }
                }
                let mut seen_values = HashSet::new();
                let mut current_val = 0i64;
                for (variant_index, variant) in enum_declaration.variants.iter().enumerate() {
                    let variant_name = if variant.name_span.end as usize <= source_text.len() {
                        &source_text
                            [variant.name_span.start as usize..variant.name_span.end as usize]
                    } else {
                        "variant"
                    };
                    if let Some(explicit) = &variant.explicit_value {
                        if let Some(int_val) = evaluate_integer_constant(explicit) {
                            current_val = int_val;
                        } else {
                            accumulated_errors.push(BindingError::EnumExplicitValueNotInteger {
                                variant_name: variant_name.to_string(),
                                found: format!("{explicit:?}"),
                                span: variant.span.to_source_span(),
                            });
                        }
                    } else if variant_index > 0 {
                        current_val += 1;
                    }
                    if !seen_values.insert(current_val) {
                        accumulated_errors.push(BindingError::DuplicateEnumVariant {
                            variant_name: variant_name.to_string(),
                            enum_name: enum_name.to_string(),
                            span: variant.span.to_source_span(),
                        });
                    }
                    let sym_name = format!("{fqn}::{variant_name}");
                    let variant_sym_id_opt = symbol_table.lookup_global(&sym_name).or_else(|| {
                        symbol_table
                            .resolve_symbol(source_id, &format!("{enum_name}::{variant_name}"))
                    });
                    if let Some(variant_sym_id) = variant_sym_id_opt
                        && let Some(variant_entry) = symbol_table.get_symbol_mut(variant_sym_id)
                    {
                        variant_entry.slot_index = Some(current_val as u32);
                    }
                }

                symbol_table.active_package = previous_active_package;
            }

            if let Declaration::Class(class_declaration) = declaration {
                let class_name = if class_declaration.name_span.end as usize <= source_text.len() {
                    &source_text[class_declaration.name_span.start as usize
                        ..class_declaration.name_span.end as usize]
                } else {
                    "class"
                };

                let decl_namespace =
                    extract_namespace_path(&class_declaration.namespace_path, source_text);
                let effective_package = compute_effective_package(&file_package, &decl_namespace);
                let fqn = if effective_package.is_empty() {
                    class_name.to_string()
                } else {
                    format!("{effective_package}::{class_name}")
                };

                // Class modifier checks
                for modifier in &class_declaration.modifiers {
                    if *modifier == Modifier::Native {
                        accumulated_errors.push(BindingError::NativeOnNonMethodOrFunction {
                            span: class_declaration.span.to_source_span(),
                        });
                    }
                    if matches!(modifier, Modifier::Const | Modifier::Weak) {
                        accumulated_errors.push(BindingError::ModifierNotApplicableToDeclaration {
                            modifier: modifier.as_str().to_string(),
                            declaration_kind: "class".to_string(),
                            span: class_declaration.span.to_source_span(),
                        });
                    }
                    if *modifier == Modifier::Virtual {
                        accumulated_errors.push(BindingError::VirtualOnNonMethod {
                            span: class_declaration.span.to_source_span(),
                        });
                    }
                    if *modifier == Modifier::Override {
                        accumulated_errors.push(BindingError::OverrideOnNonMethod {
                            span: class_declaration.span.to_source_span(),
                        });
                    }
                }

                let class_symbol_id = match symbol_table
                    .lookup_global(&fqn)
                    .or_else(|| symbol_table.resolve_symbol(source_id, class_name))
                {
                    Some(id) => id,
                    None => continue,
                };

                class_ast_map.insert(class_symbol_id, (source_id, class_declaration));

                for member in &class_declaration.members {
                    if let ClassMember::Class(nested_class) = member {
                        let nested_name =
                            if nested_class.name_span.end as usize <= source_text.len() {
                                &source_text[nested_class.name_span.start as usize
                                    ..nested_class.name_span.end as usize]
                            } else {
                                "class"
                            };
                        let nested_fqn = format!("{class_name}::{nested_name}");
                        let nested_dot = format!("{class_name}.{nested_name}");
                        if let Some(nested_id) = symbol_table
                            .resolve_symbol(source_id, &nested_fqn)
                            .or_else(|| symbol_table.resolve_symbol(source_id, &nested_dot))
                            .or_else(|| symbol_table.resolve_symbol(source_id, nested_name))
                        {
                            class_ast_map.insert(nested_id, (source_id, nested_class));
                        }
                    }
                    if let ClassMember::Interface(nested_iface) = member {
                        let nested_name =
                            if nested_iface.name_span.end as usize <= source_text.len() {
                                &source_text[nested_iface.name_span.start as usize
                                    ..nested_iface.name_span.end as usize]
                            } else {
                                "interface"
                            };
                        let nested_fqn = format!("{class_name}::{nested_name}");
                        let nested_dot = format!("{class_name}.{nested_name}");
                        interface_by_name
                            .insert(nested_name.to_string(), (source_id, nested_iface));
                        interface_by_name.insert(nested_fqn.clone(), (source_id, nested_iface));
                        interface_by_name.insert(nested_dot.clone(), (source_id, nested_iface));

                        if let Some(iface_symbol_id) = symbol_table
                            .resolve_symbol(source_id, &nested_fqn)
                            .or_else(|| symbol_table.resolve_symbol(source_id, &nested_dot))
                            .or_else(|| symbol_table.resolve_symbol(source_id, nested_name))
                        {
                            let iface_vtable_id = *next_vtable_id;
                            *next_vtable_id += 1;
                            let mut iface_layout = ClassLayout::new(
                                iface_symbol_id,
                                nested_fqn.clone(),
                                iface_vtable_id,
                            );
                            let mut iface_child_ids = Vec::new();
                            for (slot, m) in nested_iface.methods.iter().enumerate() {
                                let m_name = if m.name_span.end as usize <= source_text.len() {
                                    &source_text
                                        [m.name_span.start as usize..m.name_span.end as usize]
                                } else {
                                    "method"
                                };
                                let m_ret = resolve_type_annotation(
                                    source_id,
                                    source_text,
                                    &m.return_type,
                                    symbol_table,
                                );
                                let mut m_param_types = Vec::new();
                                for param in &m.parameters {
                                    let p_type = if param.is_variadic {
                                        Type::Primitive(PrimitiveKind::TuplePrimitive)
                                    } else {
                                        resolve_type_annotation(
                                            source_id,
                                            source_text,
                                            &param.param_type,
                                            symbol_table,
                                        )
                                    };
                                    m_param_types.push(p_type);
                                }
                                iface_layout.method_slots.insert(m_name.to_string(), slot);
                                let method_symbol = SymbolEntry {
                                    id: SymbolId(0),
                                    name: m_name.to_string(),
                                    fully_qualified_name: format!("{nested_fqn}::{m_name}"),
                                    source_id,
                                    span: m.span,
                                    modifiers: m.modifiers.clone(),
                                    symbol_type: Type::FunctionPointer {
                                        return_type: Box::new(m_ret),
                                        parameter_types: m_param_types,
                                    },
                                    parent_symbol: Some(iface_symbol_id),
                                    members: Vec::new(),
                                    is_static: false,
                                    is_const: false,
                                    slot_index: Some(slot as u32),
                                };
                                let m_id = symbol_table.insert_symbol(method_symbol);
                                iface_child_ids.push(m_id);
                            }
                            if let Some(iface_sym) = symbol_table.get_symbol_mut(iface_symbol_id) {
                                iface_sym.members = iface_child_ids;
                            }
                            layouts.insert(iface_symbol_id, iface_layout);
                        }
                    }
                }

                // Check superclass
                if let Some(super_annotation) = &class_declaration.super_class {
                    let resolved_super = resolve_type_annotation(
                        source_id,
                        source_text,
                        super_annotation,
                        symbol_table,
                    );

                    match resolved_super {
                        Type::Class { symbol_name, .. } => {
                            if let Some(super_symbol_id) =
                                symbol_table.resolve_symbol(source_id, &symbol_name)
                            {
                                class_inheritance_edges.insert(class_symbol_id, super_symbol_id);
                            } else {
                                accumulated_errors.push(BindingError::undefined_symbol(
                                    symbol_name,
                                    super_annotation.span(),
                                ));
                            }
                        }
                        Type::Interface { symbol_name, .. } => {
                            accumulated_errors.push(BindingError::InvalidBaseClass {
                                base_name: symbol_name,
                                reason: "Classes may only inherit from concrete or abstract classes, not interfaces".to_string(),
                                span: super_annotation.span().to_source_span(),
                            });
                        }
                        Type::Enum { symbol_name } => {
                            accumulated_errors.push(BindingError::InvalidBaseClass {
                                base_name: symbol_name,
                                reason: "Classes may only inherit from concrete or abstract classes, not enums".to_string(),
                                span: super_annotation.span().to_source_span(),
                            });
                        }
                        Type::Unresolved(name) => {
                            accumulated_errors.push(BindingError::undefined_symbol(
                                name,
                                super_annotation.span(),
                            ));
                        }
                        other => {
                            if let Some(unres) = other.first_unresolved_name() {
                                accumulated_errors.push(BindingError::undefined_symbol(
                                    unres.to_string(),
                                    super_annotation.span(),
                                ));
                            } else {
                                accumulated_errors.push(BindingError::InvalidBaseClass {
                                    base_name: other.to_string(),
                                    reason: format!("Type '{}' cannot be used as a superclass; expected a class", other),
                                    span: super_annotation.span().to_source_span(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Detect cyclic inheritance
    for &class_id in class_inheritance_edges.keys() {
        let mut visited = HashSet::new();
        let mut current_id = class_id;

        while let Some(&super_id) = class_inheritance_edges.get(&current_id) {
            if super_id == class_id || visited.contains(&super_id) {
                let class_name = symbol_table
                    .get_symbol(class_id)
                    .map(|entry| entry.name.clone())
                    .unwrap_or_else(|| "Class".to_string());
                let span = symbol_table
                    .get_symbol(class_id)
                    .map(|entry| entry.span)
                    .unwrap_or_default();

                accumulated_errors.current_source_id = span.id;
                accumulated_errors.push(BindingError::CyclicInheritance {
                    class_name,
                    span: span.to_source_span(),
                });
                break;
            }
            visited.insert(super_id);
            current_id = super_id;
        }
    }

    // Transitively inherit methods from super-interfaces into interface symbols
    for (iface_name, &(source_file_id, iface_decl)) in &interface_by_name {
        accumulated_errors.current_source_id = source_file_id;
        if let Some(iface_id) = symbol_table.resolve_symbol(source_file_id, iface_name) {
            let mut visited_interfaces = HashSet::new();
            visited_interfaces.insert(iface_name.clone());
            let mut interface_queue = Vec::new();
            let source_text_slice = source_texts
                .get(source_file_id as usize)
                .map(|text_string| text_string.as_str())
                .unwrap_or("");
            for super_annotation in &iface_decl.super_interfaces {
                let super_resolution = resolve_type_annotation(
                    source_file_id,
                    source_text_slice,
                    super_annotation,
                    symbol_table,
                );
                if let Type::Interface {
                    ref symbol_name, ..
                } = super_resolution
                {
                    interface_queue.push(symbol_name.clone());
                    if let Some(super_sym_id) =
                        symbol_table.resolve_symbol(source_file_id, symbol_name)
                        && let Some(iface_entry) = symbol_table.get_symbol_mut(iface_id)
                    {
                        iface_entry.parent_symbol = Some(super_sym_id);
                    }
                }
            }

            let mut inherited_members = Vec::new();
            while let Some(current_super_interface) = interface_queue.pop() {
                if !visited_interfaces.insert(current_super_interface.clone()) {
                    continue;
                }
                if let Some(super_symbol_id) =
                    symbol_table.resolve_symbol(source_file_id, &current_super_interface)
                    && let Some(super_entry) = symbol_table.get_symbol(super_symbol_id)
                {
                    for &member_symbol_id in &super_entry.members {
                        if !inherited_members.contains(&member_symbol_id) {
                            inherited_members.push(member_symbol_id);
                        }
                    }
                }
                if let Some(&(super_source_id, super_decl)) =
                    interface_by_name.get(&current_super_interface)
                {
                    let super_source_text = source_texts
                        .get(super_source_id as usize)
                        .map(|text_string| text_string.as_str())
                        .unwrap_or("");
                    for super_super_annot in &super_decl.super_interfaces {
                        let super_super_res = resolve_type_annotation(
                            super_source_id,
                            super_source_text,
                            super_super_annot,
                            symbol_table,
                        );
                        if let Type::Interface {
                            ref symbol_name, ..
                        } = super_super_res
                        {
                            interface_queue.push(symbol_name.clone());
                        }
                    }
                }
            }

            if let Some(iface_entry) = symbol_table.get_symbol_mut(iface_id) {
                for member_symbol_id in &inherited_members {
                    if !iface_entry.members.contains(member_symbol_id) {
                        iface_entry.members.push(*member_symbol_id);
                    }
                }
            }
            if let Some(iface_layout) = layouts.get_mut(&iface_id) {
                for member_symbol_id in &inherited_members {
                    if let Some(member_sym) = symbol_table.get_symbol(*member_symbol_id) {
                        let next_slot = iface_layout.method_slots.len();
                        iface_layout
                            .method_slots
                            .entry(member_sym.name.clone())
                            .or_insert(next_slot);
                    }
                }
            }
        }
    }

    // 3. Construct class layouts and virtual dispatch tables in topological order (base classes before derived)
    let compute_depth = |mut current_id: SymbolId| -> usize {
        let mut depth = 0usize;
        let mut visited = HashSet::new();
        while let Some(&super_id) = class_inheritance_edges.get(&current_id) {
            if !visited.insert(current_id) {
                break;
            }
            depth += 1;
            current_id = super_id;
        }
        depth
    };

    let mut sorted_class_ids: Vec<SymbolId> = class_ast_map.keys().copied().collect();
    sorted_class_ids.sort_by_key(|&class_id| compute_depth(class_id));

    for class_symbol_id in sorted_class_ids {
        let (source_id, class_declaration) = class_ast_map[&class_symbol_id];
        accumulated_errors.current_source_id = source_id;
        let source_text = source_texts
            .get(source_id as usize)
            .map(|text| text.as_str())
            .unwrap_or("");

        let class_name = symbol_table
            .get_symbol(class_symbol_id)
            .map(|entry| entry.name.clone())
            .unwrap_or_else(|| "Class".to_string());

        let current_vtable_id = *next_vtable_id;
        *next_vtable_id += 1;

        let class_fqn = symbol_table
            .get_symbol(class_symbol_id)
            .map(|entry| entry.fully_qualified_name.clone())
            .unwrap_or_default();
        let class_package = if let Some((package_prefix, _)) = class_fqn.rsplit_once("::") {
            Some(package_prefix.to_string())
        } else {
            None
        };
        let previous_active_package = symbol_table.active_package.clone();
        if class_package.is_some() {
            symbol_table.active_package = class_package;
        }

        let mut class_layout =
            ClassLayout::new(class_symbol_id, class_name.clone(), current_vtable_id);
        class_layout.is_extern = class_declaration.modifiers.contains(&Modifier::Extern);
        let mut current_field_offset = 8usize; // Initial 8 bytes for object metadata header
        let mut child_symbol_ids = Vec::new();

        // If inheriting from a base class, copy base field offsets and vtable
        if let Some(&super_class_id) = class_inheritance_edges.get(&class_symbol_id)
            && let Some(super_layout) = layouts.get(&super_class_id)
        {
            class_layout.base_vtable_id = Some(super_layout.vtable_id);
            class_layout.field_offsets = super_layout.field_offsets.clone();
            class_layout.reference_field_offsets = super_layout.reference_field_offsets.clone();
            class_layout.weak_field_offsets = super_layout.weak_field_offsets.clone();
            class_layout.vtable_methods = super_layout.vtable_methods.clone();
            class_layout.method_slots = super_layout.method_slots.clone();
            class_layout.itable_methods = super_layout.itable_methods.clone();
            class_layout.field_types = super_layout.field_types.clone();
            current_field_offset = super_layout.instance_size_bytes;
        }

        // Register class members
        let class_generic_names: HashSet<String> = class_declaration
            .generic_params
            .iter()
            .filter_map(|param| {
                let span = param.name_span;
                if span.end as usize <= source_text.len() {
                    Some(source_text[span.start as usize..span.end as usize].to_string())
                } else {
                    None
                }
            })
            .collect();

        for member in &class_declaration.members {
            match member {
                ClassMember::Field(field_declaration) => {
                    let field_name =
                        if field_declaration.name_span.end as usize <= source_text.len() {
                            &source_text[field_declaration.name_span.start as usize
                                ..field_declaration.name_span.end as usize]
                        } else {
                            "field"
                        };

                    if class_layout.field_offsets.contains_key(field_name) {
                        accumulated_errors.push(BindingError::DuplicateSymbol {
                            symbol_name: format!("{class_name}::{field_name}"),
                            span: field_declaration.span.to_source_span(),
                        });
                        continue;
                    }

                    let field_type = resolve_type_annotation(
                        source_id,
                        source_text,
                        &field_declaration.field_type,
                        symbol_table,
                    );

                    if field_type == Type::Primitive(PrimitiveKind::Void) {
                        accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                            span: field_declaration.span.to_source_span(),
                        });
                    }
                    if let Some(unres) = field_type.first_unresolved_name()
                        && !class_generic_names.contains(unres)
                    {
                        accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                            type_name: unres.to_string(),
                            span: field_declaration.span.to_source_span(),
                        });
                    }

                    for modifier in &field_declaration.modifiers {
                        if matches!(
                            modifier,
                            Modifier::Virtual
                                | Modifier::Override
                                | Modifier::Abstract
                                | Modifier::Inline
                        ) {
                            accumulated_errors.push(
                                BindingError::ModifierNotApplicableToDeclaration {
                                    modifier: modifier.as_str().to_string(),
                                    declaration_kind: "field".to_string(),
                                    span: field_declaration.span.to_source_span(),
                                },
                            );
                        }
                        if *modifier == Modifier::Native {
                            accumulated_errors.push(BindingError::NativeOnNonMethodOrFunction {
                                span: field_declaration.span.to_source_span(),
                            });
                        }
                    }

                    let unwrapped_field_type = field_type.unwrap_alias();
                    let is_ref = unwrapped_field_type.is_reference_type();
                    // `std.string` (BlockTag::String) is a raw byte buffer managed by its own
                    // internal allocator — it is not an ARC object and must not appear in
                    // `reference_field_offsets` or be passed to `release_reference`.
                    let is_gc_managed_ref = unwrapped_field_type.is_gc_managed_reference_type();
                    let field_size = match unwrapped_field_type {
                        Type::Primitive(prim) => prim.byte_size(),
                        Type::Enum { .. } => 4,
                        _ => 8,
                    };

                    class_layout
                        .field_offsets
                        .insert(field_name.to_string(), current_field_offset);
                    if field_declaration.modifiers.contains(&Modifier::Weak) {
                        if is_ref {
                            class_layout.weak_field_offsets.push(current_field_offset);
                        } else {
                            accumulated_errors.push(BindingError::InvalidWeakModifier {
                                found: field_type.to_string(),
                                span: field_declaration.span.to_source_span(),
                            });
                        }
                    } else if is_gc_managed_ref {
                        class_layout
                            .reference_field_offsets
                            .push(current_field_offset);
                    }

                    let field_type_tag: u8 = match unwrapped_field_type {
                        Type::Primitive(prim) => match prim {
                            PrimitiveKind::Int8 => 1,
                            PrimitiveKind::UInt8 | PrimitiveKind::Char => 2,
                            PrimitiveKind::Int16 => 3,
                            PrimitiveKind::UInt16 => 4,
                            PrimitiveKind::Int32 => 5,
                            PrimitiveKind::UInt32 => 6,
                            PrimitiveKind::Int64 => 7,
                            PrimitiveKind::UInt64 => 8,
                            PrimitiveKind::Float32 => 9,
                            PrimitiveKind::Float64 => 10,
                            PrimitiveKind::Bool => 11,
                            PrimitiveKind::TuplePrimitive => 12,
                            PrimitiveKind::Void => 0,
                        },
                        Type::Enum { .. } => 5,
                        _ => 12,
                    };
                    class_layout
                        .field_types
                        .insert(current_field_offset, field_type_tag);

                    current_field_offset += field_size.max(4);

                    let field_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: field_name.to_string(),
                        fully_qualified_name: format!("{class_name}::{field_name}"),
                        source_id,
                        span: field_declaration.span,
                        modifiers: field_declaration.modifiers.clone(),
                        symbol_type: field_type,
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: field_declaration.modifiers.contains(&Modifier::Static),
                        is_const: field_declaration.modifiers.contains(&Modifier::Const),
                        slot_index: None,
                    };
                    let field_id = symbol_table.insert_symbol(field_symbol);
                    child_symbol_ids.push(field_id);
                }
                ClassMember::Method(method_declaration) => {
                    let method_name =
                        if method_declaration.name_span.end as usize <= source_text.len() {
                            &source_text[method_declaration.name_span.start as usize
                                ..method_declaration.name_span.end as usize]
                        } else {
                            "method"
                        };

                    let method_generic_names: HashSet<String> = method_declaration
                        .generic_params
                        .iter()
                        .filter_map(|param| {
                            let span = param.name_span;
                            if span.end as usize <= source_text.len() {
                                Some(
                                    source_text[span.start as usize..span.end as usize].to_string(),
                                )
                            } else {
                                None
                            }
                        })
                        .collect();

                    let return_type = resolve_type_annotation(
                        source_id,
                        source_text,
                        &method_declaration.return_type,
                        symbol_table,
                    );

                    if let Some(unres) = return_type.first_unresolved_name()
                        && !class_generic_names.contains(unres)
                        && !method_generic_names.contains(unres)
                    {
                        accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                            type_name: unres.to_string(),
                            span: method_declaration.return_type.span().to_source_span(),
                        });
                    }

                    let parameter_types: Vec<Type> = method_declaration
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let p_type = if parameter.is_variadic {
                                Type::Primitive(PrimitiveKind::TuplePrimitive)
                            } else {
                                resolve_type_annotation(
                                    source_id,
                                    source_text,
                                    &parameter.param_type,
                                    symbol_table,
                                )
                            };
                            if p_type == Type::Primitive(PrimitiveKind::Void) {
                                accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                                    span: parameter.span.to_source_span(),
                                });
                            }
                            if let Some(unres) = p_type.first_unresolved_name()
                                && !class_generic_names.contains(unres)
                                && !method_generic_names.contains(unres)
                            {
                                accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                                    type_name: unres.to_string(),
                                    span: parameter.span.to_source_span(),
                                });
                            }
                            p_type
                        })
                        .collect();

                    let is_static = method_declaration.modifiers.contains(&Modifier::Static);

                    // Validate method modifiers
                    for modifier in &method_declaration.modifiers {
                        if matches!(modifier, Modifier::Const | Modifier::Weak) {
                            accumulated_errors.push(
                                BindingError::ModifierNotApplicableToDeclaration {
                                    modifier: modifier.as_str().to_string(),
                                    declaration_kind: "method".to_string(),
                                    span: method_declaration.span.to_source_span(),
                                },
                            );
                        }
                    }
                    if method_declaration.modifiers.contains(&Modifier::Native) {
                        if !method_declaration.generic_params.is_empty() {
                            accumulated_errors.push(BindingError::ConflictingModifierCombination {
                                modifier1: "native".to_string(),
                                modifier2: "generic <T>".to_string(),
                                span: method_declaration.span.to_source_span(),
                            });
                        }
                        for modifier in &method_declaration.modifiers {
                            if matches!(
                                modifier,
                                Modifier::Abstract
                                    | Modifier::Virtual
                                    | Modifier::Override
                                    | Modifier::Inline
                                    | Modifier::Extern
                            ) {
                                accumulated_errors.push(
                                    BindingError::ConflictingModifierCombination {
                                        modifier1: "native".to_string(),
                                        modifier2: modifier.as_str().to_string(),
                                        span: method_declaration.span.to_source_span(),
                                    },
                                );
                            }
                        }
                    }
                    if method_declaration.modifiers.contains(&Modifier::Abstract) {
                        if !class_declaration.modifiers.contains(&Modifier::Abstract) {
                            accumulated_errors.push(
                                BindingError::AbstractMethodInNonAbstractClass {
                                    method_name: method_name.to_string(),
                                    class_name: class_name.clone(),
                                    span: method_declaration.span.to_source_span(),
                                },
                            );
                        }
                        for modifier in &method_declaration.modifiers {
                            if matches!(
                                modifier,
                                Modifier::Static
                                    | Modifier::Private
                                    | Modifier::Inline
                                    | Modifier::Native
                                    | Modifier::Extern
                                    | Modifier::Virtual
                            ) {
                                accumulated_errors.push(
                                    BindingError::ConflictingModifierCombination {
                                        modifier1: "abstract".to_string(),
                                        modifier2: modifier.as_str().to_string(),
                                        span: method_declaration.span.to_source_span(),
                                    },
                                );
                            }
                        }
                    }
                    if is_static {
                        for modifier in &method_declaration.modifiers {
                            if matches!(
                                modifier,
                                Modifier::Virtual | Modifier::Override | Modifier::Abstract
                            ) {
                                accumulated_errors.push(
                                    BindingError::ConflictingModifierCombination {
                                        modifier1: "static".to_string(),
                                        modifier2: modifier.as_str().to_string(),
                                        span: method_declaration.span.to_source_span(),
                                    },
                                );
                            }
                        }
                    }
                    if method_declaration.modifiers.contains(&Modifier::Private) {
                        for modifier in &method_declaration.modifiers {
                            if matches!(modifier, Modifier::Virtual | Modifier::Override) {
                                accumulated_errors.push(
                                    BindingError::ConflictingModifierCombination {
                                        modifier1: "private".to_string(),
                                        modifier2: modifier.as_str().to_string(),
                                        span: method_declaration.span.to_source_span(),
                                    },
                                );
                            }
                        }
                    }

                    // Virtual and override rules
                    if method_declaration.modifiers.contains(&Modifier::Override) {
                        match find_base_method(
                            symbol_table,
                            &class_inheritance_edges,
                            class_symbol_id,
                            method_name,
                        ) {
                            None => {
                                accumulated_errors.push(BindingError::OverrideWithoutBaseMethod {
                                    method_name: method_name.to_string(),
                                    span: method_declaration.span.to_source_span(),
                                });
                            }
                            Some(base_entry) => {
                                let mut is_virtual_or_abstract =
                                    base_entry.modifiers.contains(&Modifier::Virtual)
                                        || base_entry.modifiers.contains(&Modifier::Abstract);
                                if !is_virtual_or_abstract
                                    && let Some(base_class_id) = base_entry.parent_symbol
                                    && let Some(&(base_src_id, base_class_decl)) =
                                        class_ast_map.get(&base_class_id)
                                {
                                    let base_src_text = source_texts
                                        .get(base_src_id as usize)
                                        .map(|s| s.as_str())
                                        .unwrap_or("");
                                    for iface_annot in &base_class_decl.interfaces {
                                        let resolved_iface = resolve_type_annotation(
                                            base_src_id,
                                            base_src_text,
                                            iface_annot,
                                            symbol_table,
                                        );
                                        if let Type::Interface {
                                            ref symbol_name, ..
                                        } = resolved_iface
                                            && let Some(&(if_src, iface_decl)) =
                                                interface_by_name.get(symbol_name)
                                        {
                                            let if_text = source_texts
                                                .get(if_src as usize)
                                                .map(|s| s.as_str())
                                                .unwrap_or("");
                                            if iface_decl.methods.iter().any(|m| {
                                                let m_name =
                                                    if m.name_span.end as usize <= if_text.len() {
                                                        &if_text[m.name_span.start as usize
                                                            ..m.name_span.end as usize]
                                                    } else {
                                                        ""
                                                    };
                                                m_name == method_name
                                            }) {
                                                is_virtual_or_abstract = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                                let base_class_name = base_entry
                                    .parent_symbol
                                    .and_then(|parent_id| symbol_table.get_symbol(parent_id))
                                    .map(|parent_sym| parent_sym.name.clone())
                                    .unwrap_or_else(|| "Base".to_string());
                                if !is_virtual_or_abstract {
                                    accumulated_errors.push(
                                        BindingError::OverrideNonVirtualMethod {
                                            method_name: method_name.to_string(),
                                            base_class: base_class_name.clone(),
                                            span: method_declaration.span.to_source_span(),
                                        },
                                    );
                                }
                                if let Type::FunctionPointer {
                                    return_type: ref base_ret,
                                    parameter_types: ref base_params,
                                } = base_entry.symbol_type
                                    && (base_params != &parameter_types
                                        || **base_ret != return_type)
                                {
                                    let expected_params = base_params
                                        .iter()
                                        .map(|param_type| param_type.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    let found_params = parameter_types
                                        .iter()
                                        .map(|param_type| param_type.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    accumulated_errors.push(
                                            BindingError::MethodOverrideMismatch {
                                                method_name: method_name.to_string(),
                                                super_class: base_class_name,
                                                reason: format!(
                                                    "expected {method_name}({expected_params}): {base_ret}, found {method_name}({found_params}): {return_type}"
                                                ),
                                                span: method_declaration.span.to_source_span(),
                                            },
                                        );
                                }
                            }
                        }
                    } else if !is_static
                        && !method_declaration.modifiers.contains(&Modifier::Private)
                        && let Some(base_entry) = find_base_method(
                            symbol_table,
                            &class_inheritance_edges,
                            class_symbol_id,
                            method_name,
                        )
                    {
                        let is_virtual = base_entry.modifiers.contains(&Modifier::Virtual);
                        if is_virtual
                            && let Type::FunctionPointer {
                                parameter_types: ref base_params,
                                ..
                            } = base_entry.symbol_type
                            && base_params == &parameter_types
                        {
                            let base_class_name = base_entry
                                .parent_symbol
                                .and_then(|parent_id| symbol_table.get_symbol(parent_id))
                                .map(|parent_sym| parent_sym.name.clone())
                                .unwrap_or_else(|| "Base".to_string());
                            accumulated_errors.push(BindingError::MethodOverrideMismatch {
                                method_name: method_name.to_string(),
                                super_class: base_class_name,
                                reason:
                                    "missing 'override' modifier on virtual method implementation"
                                        .to_string(),
                                span: method_declaration.span.to_source_span(),
                            });
                        }
                    }

                    let param_sig = parameter_types
                        .iter()
                        .map(|param_type| param_type.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let method_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: method_name.to_string(),
                        fully_qualified_name: format!("{class_name}::{method_name}({param_sig})"),
                        source_id,
                        span: method_declaration.span,
                        modifiers: method_declaration.modifiers.clone(),
                        symbol_type: Type::FunctionPointer {
                            return_type: Box::new(return_type.clone()),
                            parameter_types: parameter_types.clone(),
                        },
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static,
                        is_const: false,
                        slot_index: None,
                    };
                    let method_id = symbol_table.insert_symbol(method_symbol.clone());
                    child_symbol_ids.push(method_id);

                    if !is_static {
                        // Check if overriding existing virtual method slot
                        let mut overridden = false;
                        for (slot_idx, slot) in class_layout.vtable_methods.iter_mut().enumerate() {
                            if let Some(existing_entry) = symbol_table.get_symbol(*slot)
                                && existing_entry.name == method_name
                                && existing_entry.symbol_type == method_symbol.symbol_type
                            {
                                let old_id = *slot;
                                *slot = method_id;
                                let param_sig = parameter_types
                                    .iter()
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()
                                    .join(",");
                                class_layout
                                    .method_slots
                                    .insert(format!("{method_name}({param_sig})"), slot_idx);
                                class_layout.method_slots.insert(
                                    format!("{method_name}/{}", parameter_types.len()),
                                    slot_idx,
                                );
                                class_layout
                                    .method_slots
                                    .insert(method_name.to_string(), slot_idx);
                                for methods in class_layout.itable_methods.values_mut() {
                                    for method_slot in methods.iter_mut() {
                                        if *method_slot == old_id {
                                            *method_slot = method_id;
                                        }
                                    }
                                }
                                overridden = true;
                                break;
                            }
                        }
                        if !overridden {
                            let slot_idx = class_layout.vtable_methods.len();
                            class_layout.vtable_methods.push(method_id);
                            let param_sig = parameter_types
                                .iter()
                                .map(|param_type| param_type.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            let mangled_slot = format!("{method_name}({param_sig})");
                            let arity_slot = format!("{method_name}/{}", parameter_types.len());
                            class_layout.method_slots.insert(mangled_slot, slot_idx);
                            class_layout.method_slots.insert(arity_slot, slot_idx);
                            class_layout
                                .method_slots
                                .entry(method_name.to_string())
                                .or_insert(slot_idx);
                        }
                    }
                }
                ClassMember::Constructor(constructor_declaration) => {
                    let arity = constructor_declaration.parameters.len();
                    for modifier in &constructor_declaration.modifiers {
                        if matches!(
                            modifier,
                            Modifier::Static
                                | Modifier::Virtual
                                | Modifier::Override
                                | Modifier::Abstract
                                | Modifier::Const
                                | Modifier::Weak
                        ) {
                            accumulated_errors.push(
                                BindingError::ModifierNotApplicableToDeclaration {
                                    modifier: modifier.as_str().to_string(),
                                    declaration_kind: "constructor".to_string(),
                                    span: constructor_declaration.span.to_source_span(),
                                },
                            );
                        }
                        if *modifier == Modifier::Native {
                            accumulated_errors.push(BindingError::NativeOnNonMethodOrFunction {
                                span: constructor_declaration.span.to_source_span(),
                            });
                        }
                    }

                    let param_types: Vec<Type> = constructor_declaration
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let p_type = resolve_type_annotation(
                                source_id,
                                source_text,
                                &parameter.param_type,
                                symbol_table,
                            );
                            if p_type == Type::Primitive(PrimitiveKind::Void) {
                                accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                                    span: parameter.span.to_source_span(),
                                });
                            }
                            if let Some(unres) = p_type.first_unresolved_name()
                                && !class_generic_names.contains(unres)
                            {
                                accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                                    type_name: unres.to_string(),
                                    span: parameter.span.to_source_span(),
                                });
                            }
                            p_type
                        })
                        .collect();
                    let constructor_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: "constructor".to_string(),
                        fully_qualified_name: format!("{class_name}::constructor/{arity}"),
                        source_id,
                        span: constructor_declaration.span,
                        modifiers: constructor_declaration.modifiers.clone(),
                        symbol_type: Type::FunctionPointer {
                            parameter_types: param_types,
                            return_type: Box::new(Type::Primitive(PrimitiveKind::Void)),
                        },
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                    };
                    let constructor_id = symbol_table.insert_symbol(constructor_symbol);
                    child_symbol_ids.push(constructor_id);
                }
                ClassMember::Destructor(destructor_declaration) => {
                    let destructor_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: "destructor".to_string(),
                        fully_qualified_name: format!("{class_name}::~destructor"),
                        source_id,
                        span: destructor_declaration.span,
                        modifiers: Vec::new(),
                        symbol_type: Type::Primitive(PrimitiveKind::Void),
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                    };
                    let destructor_id = symbol_table.insert_symbol(destructor_symbol);
                    child_symbol_ids.push(destructor_id);
                }
                ClassMember::Operator(operator_declaration) => {
                    for modifier in &operator_declaration.modifiers {
                        if *modifier == Modifier::Native {
                            accumulated_errors.push(BindingError::NativeOnNonMethodOrFunction {
                                span: operator_declaration.span.to_source_span(),
                            });
                        }
                        if *modifier == Modifier::Extern {
                            accumulated_errors.push(
                                BindingError::ModifierNotApplicableToDeclaration {
                                    modifier: "extern".to_string(),
                                    declaration_kind: "operator".to_string(),
                                    span: operator_declaration.span.to_source_span(),
                                },
                            );
                        }
                    }

                    let operator_name = format!(
                        "operator_{}",
                        operator_declaration.operator_token.as_static_str()
                    );
                    let return_type = resolve_type_annotation(
                        source_id,
                        source_text,
                        &operator_declaration.return_type,
                        symbol_table,
                    );
                    if let Some(unres) = return_type.first_unresolved_name()
                        && !class_generic_names.contains(unres)
                    {
                        accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                            type_name: unres.to_string(),
                            span: operator_declaration.return_type.span().to_source_span(),
                        });
                    }

                    let parameter_types: Vec<Type> = operator_declaration
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let p_type = resolve_type_annotation(
                                source_id,
                                source_text,
                                &parameter.param_type,
                                symbol_table,
                            );
                            if p_type == Type::Primitive(PrimitiveKind::Void) {
                                accumulated_errors.push(BindingError::VoidTypeNotAllowedHere {
                                    span: parameter.span.to_source_span(),
                                });
                            }
                            if let Some(unres) = p_type.first_unresolved_name()
                                && !class_generic_names.contains(unres)
                            {
                                accumulated_errors.push(BindingError::UnknownTypeInDeclaration {
                                    type_name: unres.to_string(),
                                    span: parameter.span.to_source_span(),
                                });
                            }
                            p_type
                        })
                        .collect();

                    if parameter_types.len() != 1 {
                        accumulated_errors.push(BindingError::InvalidOperatorOverload {
                            operator_name: operator_name.clone(),
                            reason:
                                "Binary operator declared in class must take exactly 1 parameter"
                                    .to_string(),
                            span: operator_declaration.span.to_source_span(),
                        });
                    }

                    let operator_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: operator_name.clone(),
                        fully_qualified_name: format!("{class_name}::{operator_name}"),
                        source_id,
                        span: operator_declaration.span,
                        modifiers: operator_declaration.modifiers.clone(),
                        symbol_type: Type::FunctionPointer {
                            return_type: Box::new(return_type),
                            parameter_types: parameter_types.clone(),
                        },
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                    };
                    let operator_id = symbol_table.insert_symbol(operator_symbol);
                    child_symbol_ids.push(operator_id);

                    let slot_idx = class_layout.vtable_methods.len();
                    class_layout.vtable_methods.push(operator_id);
                    let param_sig = parameter_types
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    class_layout
                        .method_slots
                        .insert(format!("{operator_name}({param_sig})"), slot_idx);
                    class_layout.method_slots.insert(
                        format!("{operator_name}/{}", parameter_types.len()),
                        slot_idx,
                    );
                    class_layout.method_slots.insert(operator_name, slot_idx);
                }
                ClassMember::Enum(enum_declaration) => {
                    let enum_name = if enum_declaration.name_span.end as usize <= source_text.len()
                    {
                        &source_text[enum_declaration.name_span.start as usize
                            ..enum_declaration.name_span.end as usize]
                    } else {
                        "enum"
                    };
                    let nested_fqn = format!("{class_name}::{enum_name}");
                    let enum_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: enum_name.to_string(),
                        fully_qualified_name: nested_fqn.clone(),
                        source_id,
                        span: enum_declaration.span,
                        modifiers: enum_declaration.modifiers.clone(),
                        symbol_type: Type::Enum {
                            symbol_name: nested_fqn.clone(),
                        },
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: true,
                        is_const: false,
                        slot_index: None,
                    };
                    let enum_id = symbol_table.insert_symbol(enum_symbol);
                    child_symbol_ids.push(enum_id);

                    let mut seen_values = HashSet::new();
                    let mut current_val = 0i64;
                    for (variant_index, variant) in enum_declaration.variants.iter().enumerate() {
                        let variant_name = if variant.name_span.end as usize <= source_text.len() {
                            &source_text
                                [variant.name_span.start as usize..variant.name_span.end as usize]
                        } else {
                            "variant"
                        };
                        if let Some(explicit) = &variant.explicit_value {
                            if let Some(int_val) = evaluate_integer_constant(explicit) {
                                current_val = int_val;
                            } else {
                                accumulated_errors.push(
                                    BindingError::EnumExplicitValueNotInteger {
                                        variant_name: variant_name.to_string(),
                                        found: "non-integer".to_string(),
                                        span: variant.span.to_source_span(),
                                    },
                                );
                            }
                        } else if variant_index > 0 {
                            current_val += 1;
                        }
                        if !seen_values.insert(current_val) {
                            accumulated_errors.push(BindingError::DuplicateEnumVariant {
                                variant_name: variant_name.to_string(),
                                enum_name: enum_name.to_string(),
                                span: variant.span.to_source_span(),
                            });
                        }
                        let variant_symbol = SymbolEntry {
                            id: SymbolId(0),
                            name: variant_name.to_string(),
                            fully_qualified_name: format!("{nested_fqn}::{variant_name}"),
                            source_id,
                            span: variant.span,
                            modifiers: enum_declaration.modifiers.clone(),
                            symbol_type: Type::Enum {
                                symbol_name: nested_fqn.clone(),
                            },
                            parent_symbol: Some(enum_id),
                            members: Vec::new(),
                            is_static: true,
                            is_const: true,
                            slot_index: Some(current_val as u32),
                        };
                        let variant_id = symbol_table.insert_symbol(variant_symbol);
                        if let Some(parent_entry) = symbol_table.get_symbol_mut(enum_id) {
                            parent_entry.members.push(variant_id);
                        }
                    }
                }
                ClassMember::Class(_) | ClassMember::Interface(_) => {}
            }
        }

        // Register implemented interfaces
        for iface_annotation in &class_declaration.interfaces {
            let resolved_iface =
                resolve_type_annotation(source_id, source_text, iface_annotation, symbol_table);
            match resolved_iface {
                Type::Interface {
                    ref symbol_name, ..
                } => {
                    let simple = symbol_name
                        .rsplit(['.', ':'])
                        .next()
                        .unwrap_or(symbol_name)
                        .to_string();

                    let mut required_methods: Vec<String> = Vec::new();
                    let mut iface_queue = vec![symbol_name.clone(), simple.clone()];
                    let mut visited_ifaces = HashSet::new();

                    while let Some(current_iface_name) = iface_queue.pop() {
                        if !visited_ifaces.insert(current_iface_name.clone()) {
                            continue;
                        }
                        if let Some(&(iface_src_id, iface_decl)) =
                            interface_by_name.get(&current_iface_name)
                        {
                            let iface_src_text = source_texts
                                .get(iface_src_id as usize)
                                .map(|s| s.as_str())
                                .unwrap_or("");
                            for method in &iface_decl.methods {
                                let method_name =
                                    if method.name_span.end as usize <= iface_src_text.len() {
                                        &iface_src_text[method.name_span.start as usize
                                            ..method.name_span.end as usize]
                                    } else {
                                        ""
                                    };
                                required_methods.push(method_name.to_string());
                            }
                            for super_annot in &iface_decl.super_interfaces {
                                let super_res = resolve_type_annotation(
                                    iface_src_id,
                                    iface_src_text,
                                    super_annot,
                                    symbol_table,
                                );
                                if let Type::Interface {
                                    symbol_name: ref super_sym_name,
                                    ..
                                } = super_res
                                {
                                    iface_queue.push(super_sym_name.clone());
                                }
                            }
                        }
                    }

                    let mut iface_methods = Vec::new();
                    for required_method_name in &required_methods {
                        if let Some(&cls_slot) = class_layout.method_slots.get(required_method_name)
                            && cls_slot < class_layout.vtable_methods.len()
                        {
                            iface_methods.push(class_layout.vtable_methods[cls_slot]);
                        } else if !class_declaration.modifiers.contains(&Modifier::Abstract) {
                            accumulated_errors.push(BindingError::UnimplementedInterfaceMethod {
                                class_name: class_name.to_string(),
                                interface_name: symbol_name.clone(),
                                method_signature: required_method_name.clone(),
                                span: class_declaration.span.to_source_span(),
                            });
                        }
                    }

                    if iface_methods.is_empty() {
                        iface_methods = class_layout.vtable_methods.clone();
                    }

                    for visited_name in &visited_ifaces {
                        if let Some(&(vis_src_id, vis_decl)) = interface_by_name.get(visited_name) {
                            let vis_text = source_texts
                                .get(vis_src_id as usize)
                                .map(|s| s.as_str())
                                .unwrap_or("");
                            let mut vis_methods = Vec::new();
                            for m in &vis_decl.methods {
                                let m_name = if m.name_span.end as usize <= vis_text.len() {
                                    &vis_text[m.name_span.start as usize..m.name_span.end as usize]
                                } else {
                                    ""
                                };
                                if let Some(&cls_slot) = class_layout.method_slots.get(m_name)
                                    && cls_slot < class_layout.vtable_methods.len()
                                {
                                    vis_methods.push(class_layout.vtable_methods[cls_slot]);
                                }
                            }
                            // Also include inherited methods from super interfaces of this interface
                            for super_annot in &vis_decl.super_interfaces {
                                let super_res = resolve_type_annotation(
                                    vis_src_id,
                                    vis_text,
                                    super_annot,
                                    symbol_table,
                                );
                                if let Type::Interface {
                                    ref symbol_name, ..
                                } = super_res
                                    && let Some(&(s_src, s_decl)) =
                                        interface_by_name.get(symbol_name)
                                {
                                    let s_text = source_texts
                                        .get(s_src as usize)
                                        .map(|s| s.as_str())
                                        .unwrap_or("");
                                    for sm in &s_decl.methods {
                                        let sm_name = if sm.name_span.end as usize <= s_text.len() {
                                            &s_text[sm.name_span.start as usize
                                                ..sm.name_span.end as usize]
                                        } else {
                                            ""
                                        };
                                        if let Some(&cls_slot) =
                                            class_layout.method_slots.get(sm_name)
                                            && cls_slot < class_layout.vtable_methods.len()
                                            && !vis_methods
                                                .contains(&class_layout.vtable_methods[cls_slot])
                                        {
                                            vis_methods.push(class_layout.vtable_methods[cls_slot]);
                                        }
                                    }
                                }
                            }

                            let visited_simple = visited_name
                                .rsplit(['.', ':'])
                                .next()
                                .unwrap_or(visited_name)
                                .to_string();
                            class_layout
                                .itable_methods
                                .insert(visited_name.clone(), vis_methods.clone());
                            class_layout
                                .itable_methods
                                .insert(visited_simple, vis_methods);
                        }
                    }
                    class_layout
                        .itable_methods
                        .insert(symbol_name.clone(), iface_methods.clone());
                    class_layout.itable_methods.insert(simple, iface_methods);
                }
                Type::Unresolved(ref name) => {
                    accumulated_errors.push(BindingError::ImplementedTypeMustBeInterface {
                        found: name.clone(),
                        span: iface_annotation.span().to_source_span(),
                    });
                }
                other => {
                    let type_name = if let Some(unres) = other.first_unresolved_name() {
                        unres.to_string()
                    } else {
                        other.to_string()
                    };
                    accumulated_errors.push(BindingError::ImplementedTypeMustBeInterface {
                        found: type_name,
                        span: iface_annotation.span().to_source_span(),
                    });
                }
            }
        }

        if !class_declaration.modifiers.contains(&Modifier::Abstract) {
            for &method_id in &class_layout.vtable_methods {
                if let Some(method_entry) = symbol_table.get_symbol(method_id)
                    && method_entry.modifiers.contains(&Modifier::Abstract)
                {
                    accumulated_errors.push(BindingError::UnimplementedInterfaceMethod {
                        class_name: class_name.to_string(),
                        interface_name: method_entry
                            .parent_symbol
                            .and_then(|parent_id| symbol_table.get_symbol(parent_id))
                            .map(|parent_sym| parent_sym.name.clone())
                            .unwrap_or_else(|| "Base".to_string()),
                        method_signature: method_entry.name.clone(),
                        span: class_declaration.span.to_source_span(),
                    });
                }
            }
        }

        class_layout.instance_size_bytes = current_field_offset;
        layouts.insert(class_symbol_id, class_layout);

        if let Some(class_entry) = symbol_table.get_symbol_mut(class_symbol_id) {
            class_entry.parent_symbol = class_inheritance_edges.get(&class_symbol_id).copied();
            class_entry.members = child_symbol_ids;
        }

        symbol_table.active_package = previous_active_package;
    }

    if !accumulated_errors.is_empty() {
        return Err(accumulated_errors.errors);
    }

    Ok(())
}
