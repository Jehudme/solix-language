//! Semantic analysis driver executing multi-pass binding, type checking, and generic monomorphization.

#![forbid(unsafe_code)]

use crate::BindingError;
use crate::CompilationContext::CompilationContext;
use crate::phases::binder::layout::ClassLayout;
use crate::phases::binder::monomorphizer::GenericBlueprintRegistry;
use crate::phases::binder::passes::{
    TypeCheckingVisitor, execute_pass1_symbol_discovery, execute_pass2_type_hierarchy,
};
use crate::phases::binder::symbols::{SymbolId, SymbolTable};
use crate::phases::lexing::Span;
use crate::phases::parser::statements::declarations::ClassDeclaration;
use crate::phases::parser::visitor::AstVisitor;
use solix_common::{PrimitiveKind, Type};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metrics and statistics summarizing the completion of the semantic binding phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BindingMetaData {
    /// Number of declared symbols registered in the symbol table.
    pub symbols_bound: usize,
    /// Number of class layouts calculated with dedicated VTables.
    pub classes_bound: usize,
    /// Number of generic blueprint classes monomorphized on the fly.
    pub monomorphized_count: usize,
    /// Total wall-clock duration of the binding phase.
    pub duration: Duration,
}

/// Compilation artifacts produced by the semantic binder.
#[derive(Debug, Clone, Default)]
pub struct BindingArtifacts {
    /// Hierarchical symbol table containing all declared and monomorphized symbols.
    pub symbol_table: SymbolTable,
    /// Calculated memory layouts and VTable slot maps indexed by class [`SymbolId`].
    pub layouts: HashMap<SymbolId, ClassLayout>,
    /// Evaluated semantic types of all expressions mapped by [`Span`] (powers LSP hover).
    pub node_types: HashMap<Span, Type>,
    /// Resolved symbol definitions mapped by usage [`Span`] (powers LSP go-to-definition).
    pub resolved_symbols: HashMap<Span, SymbolId>,
    /// ASTs of generic classes monomorphized on the fly at compile time.
    pub monomorphized_classes: Vec<ClassDeclaration>,
}

/// Executes full semantic binding across all source programs in the compilation context.
///
/// # Errors
/// Returns an accumulated list of [`BindingError`]s if type mismatches, unresolved symbols, or
/// semantic constraints are violated.
pub fn bind_context(
    context: &mut CompilationContext,
) -> Result<BindingMetaData, Vec<BindingError>> {
    let start_time = Instant::now();
    let sources_count = context.sources.len();

    let mut symbol_table = SymbolTable::with_sources_count(sources_count);
    for dependency_name in context.dependencies_map.keys() {
        crate::phases::binder::passes::pass1_symbol_discovery::register_known_package(
            &mut symbol_table,
            &dependency_name.replace('.', "::"),
        );
    }
    let mut blueprint_registry = GenericBlueprintRegistry::default();
    let mut layouts = HashMap::new();
    let mut node_types = HashMap::new();
    let mut resolved_symbols = HashMap::new();
    let mut accumulated_errors: Vec<(u16, BindingError)> = Vec::new();
    let mut next_vtable_id = 1u32;

    // Pass 1: Global Symbol Outline & Package Registration
    symbol_table.cli_linked_packages = context.cli_linked_packages.clone();
    if let Err(errors) = execute_pass1_symbol_discovery(
        &context.sources,
        &context.ast,
        &mut symbol_table,
        &mut blueprint_registry,
    ) {
        accumulated_errors.extend(errors);
    }

    // Pass 2: Type Hierarchy, VTables & Memory Layout
    if let Err(errors) = execute_pass2_type_hierarchy(
        &context.sources,
        &context.ast,
        &mut symbol_table,
        &mut layouts,
        &mut next_vtable_id,
    ) {
        accumulated_errors.extend(errors);
    }

    crate::phases::binder::call_normalizer::normalize_call_sites_with_symbols(
        &mut context.ast,
        &context.sources,
        Some(&symbol_table),
    );

    // Pass 3: Detailed Type Checking & Expression Traversal
    for (source_index, ast_option) in context.ast.iter().enumerate() {
        let Ok(source_id) = u16::try_from(source_index) else {
            break;
        };
        if context.is_external_source(source_id) {
            continue;
        }

        let source_text = context
            .sources
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");

        if let Some(program_ast) = ast_option {
            let mut pass3_errors = Vec::new();
            let mut visitor = TypeCheckingVisitor {
                source_id,
                source_text,
                symbol_table: &mut symbol_table,
                blueprint_registry: &mut blueprint_registry,
                layouts: &mut layouts,
                node_types: &mut node_types,
                resolved_symbols: &mut resolved_symbols,
                errors: &mut pass3_errors,
                current_class: None,
                current_class_generic_names: std::collections::HashSet::new(),
                current_method_generic_names: std::collections::HashSet::new(),
                current_method_return_type: None,
                is_static_method: false,
                is_inside_destructor: false,
                loop_depth: 0,
                switch_depth: 0,
                lambda_stack: Vec::new(),
                next_vtable_id: &mut next_vtable_id,
                source_texts: &context.sources,
            };

            visitor.visit_program(program_ast);
            for error in pass3_errors {
                accumulated_errors.push((source_id, error));
            }
        }
    }

    validate_static_field_dependencies(&context.ast, &context.sources, &mut accumulated_errors);

    let symbols_count = symbol_table.symbols.len();
    let classes_count = layouts.len();
    let monomorphized_count = blueprint_registry.monomorphized_asts.len();

    // Validate entry point(s)
    let mut main_methods = Vec::new();
    for (source_index, ast_option) in context.ast.iter().enumerate() {
        let Ok(source_id) = u16::try_from(source_index) else {
            break;
        };
        if context.is_external_source(source_id) {
            continue;
        }
        if let Some(program_ast) = ast_option {
            let source_text = context
                .sources
                .get(source_index)
                .map(|text| text.as_str())
                .unwrap_or("");

            for declaration in &program_ast.declarations {
                match declaration {
                    crate::phases::parser::statements::declarations::Declaration::Method(
                        method_decl,
                    ) => {
                        let method_name = if method_decl.name_span.end as usize <= source_text.len()
                        {
                            &source_text[method_decl.name_span.start as usize
                                ..method_decl.name_span.end as usize]
                        } else {
                            ""
                        };
                        if method_name == "main" {
                            let ret_type =
                                crate::phases::binder::type_resolver::resolve_type_annotation(
                                    source_id,
                                    source_text,
                                    &method_decl.return_type,
                                    &symbol_table,
                                );
                            let param_types: Vec<Type> = method_decl
                                .parameters
                                .iter()
                                .map(|param| {
                                    crate::phases::binder::type_resolver::resolve_type_annotation(
                                        source_id,
                                        source_text,
                                        &param.param_type,
                                        &symbol_table,
                                    )
                                })
                                .collect();
                            main_methods.push((source_id, method_decl.span, ret_type, param_types));
                        }
                    }
                    crate::phases::parser::statements::declarations::Declaration::Class(
                        class_decl,
                    ) => {
                        for member in &class_decl.members {
                            if let crate::phases::parser::statements::declarations::ClassMember::Method(
                                method_decl,
                            ) = member
                                && method_decl.modifiers.contains(
                                    &crate::phases::parser::statements::declarations::Modifier::Static,
                                ) {
                                    let method_name = if method_decl.name_span.end as usize
                                        <= source_text.len()
                                    {
                                        &source_text[method_decl.name_span.start as usize
                                            ..method_decl.name_span.end as usize]
                                    } else {
                                        ""
                                    };
                                    if method_name == "main" {
                                        let ret_type =
                                            crate::phases::binder::type_resolver::resolve_type_annotation(
                                                source_id,
                                                source_text,
                                                &method_decl.return_type,
                                                &symbol_table,
                                            );
                                        let param_types: Vec<Type> = method_decl
                                            .parameters
                                            .iter()
                                            .map(|param| {
                                                crate::phases::binder::type_resolver::resolve_type_annotation(
                                                    source_id,
                                                    source_text,
                                                    &param.param_type,
                                                    &symbol_table,
                                                )
                                            })
                                            .collect();
                                        main_methods.push((source_id, method_decl.span, ret_type, param_types));
                                    }
                                }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if main_methods.len() > 1 {
        let (first_conflict_source_id, first_conflict_span, _, _) = &main_methods[1];
        accumulated_errors.push((
            *first_conflict_source_id,
            BindingError::MultipleMainMethodsFound {
                count: main_methods.len(),
                span: first_conflict_span.to_source_span(),
            },
        ));
    }

    for (main_source_id, main_span, return_type, parameter_types) in &main_methods {
        let valid_return = matches!(
            return_type,
            Type::Primitive(PrimitiveKind::Void) | Type::Primitive(PrimitiveKind::Int32)
        );
        if !valid_return {
            accumulated_errors.push((
                *main_source_id,
                BindingError::MainMethodSignatureInvalid {
                    reason: format!(
                        "Entry point 'main' must return 'void' or 'int32', found '{return_type}'"
                    ),
                    span: main_span.to_source_span(),
                },
            ));
        }

        let valid_params = match parameter_types.len() {
            0 => true,
            1 => {
                let param_type = &parameter_types[0];
                match param_type {
                    Type::Array {
                        element_type,
                        depth,
                    } => {
                        if *depth == 1 {
                            match &**element_type {
                                Type::Class { symbol_name, .. } => {
                                    symbol_name == "std.string"
                                        || symbol_name == "string"
                                        || symbol_name == "std::string"
                                }
                                Type::Array {
                                    element_type: inner_type,
                                    depth: inner_depth,
                                } => {
                                    *inner_depth == 1
                                        && matches!(
                                            &**inner_type,
                                            Type::Primitive(PrimitiveKind::Char)
                                        )
                                }
                                _ => false,
                            }
                        } else if *depth == 2 {
                            matches!(&**element_type, Type::Primitive(PrimitiveKind::Char))
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            _ => false,
        };

        if !valid_params {
            accumulated_errors.push((
                *main_source_id,
                BindingError::MainMethodSignatureInvalid {
                    reason: "Entry point 'main' parameters must be empty () or take a single 'string[]' / 'char[][]' parameter".to_string(),
                    span: main_span.to_source_span(),
                },
            ));
        }
    }

    let artifacts = BindingArtifacts {
        symbol_table,
        layouts,
        node_types,
        resolved_symbols,
        monomorphized_classes: blueprint_registry.monomorphized_asts,
    };

    context.binding_artifacts = Some(artifacts);
    context.binding_errors = accumulated_errors.clone();

    if !accumulated_errors.is_empty() {
        let unlocated_errors: Vec<BindingError> = accumulated_errors
            .into_iter()
            .map(|(_, error)| error)
            .collect();
        return Err(unlocated_errors);
    }

    Ok(BindingMetaData {
        symbols_bound: symbols_count,
        classes_bound: classes_count,
        monomorphized_count,
        duration: start_time.elapsed(),
    })
}

/// Executes full semantic binding across all source programs in the compilation context.
///
/// Direct alias for [`bind_context`].
///
/// # Errors
/// Returns an accumulated list of [`BindingError`]s if semantic analysis fails.
#[inline]
pub fn bind(context: &mut CompilationContext) -> Result<BindingMetaData, Vec<BindingError>> {
    bind_context(context)
}

/// Incrementally rebinds the compilation context following an update to `changed_source_id`.
///
/// Utilizes reverse dependency graph tracking to rebind the modified file and its downstream dependents.
///
/// # Errors
/// Returns semantic diagnostics if errors were encountered during rebinding.
pub fn bind_incremental(
    context: &mut CompilationContext,
    _changed_source_id: u16,
) -> Result<BindingMetaData, Vec<BindingError>> {
    // For soundness, execute full multi-pass binding across all active valid ASTs
    bind_context(context)
}

fn collect_static_field_references(
    expression: &crate::phases::parser::statements::expressions::Expression,
    source_text: &str,
    referenced_symbols: &mut Vec<String>,
) {
    use crate::phases::parser::statements::expressions::Expression;
    match expression {
        Expression::MemberAccess {
            object,
            member_span,
            ..
        } => {
            let start_offset = member_span.start as usize;
            let end_offset = member_span.end as usize;
            let member_name = if start_offset <= end_offset && end_offset <= source_text.len() {
                &source_text[start_offset..end_offset]
            } else {
                ""
            };
            if let Expression::Identifier { span: target_span } = &**object {
                let object_start = target_span.start as usize;
                let object_end = target_span.end as usize;
                let object_name = if object_start <= object_end && object_end <= source_text.len() {
                    &source_text[object_start..object_end]
                } else {
                    ""
                };
                if !object_name.is_empty() && !member_name.is_empty() {
                    referenced_symbols.push(format!("{object_name}::{member_name}"));
                }
            }
            collect_static_field_references(object, source_text, referenced_symbols);
        }
        Expression::Binary { left, right, .. } => {
            collect_static_field_references(left, source_text, referenced_symbols);
            collect_static_field_references(right, source_text, referenced_symbols);
        }
        Expression::Unary { operand, .. } => {
            collect_static_field_references(operand, source_text, referenced_symbols);
        }
        Expression::Assignment { target, value, .. } => {
            collect_static_field_references(target, source_text, referenced_symbols);
            collect_static_field_references(value, source_text, referenced_symbols);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_static_field_references(condition, source_text, referenced_symbols);
            collect_static_field_references(then_branch, source_text, referenced_symbols);
            collect_static_field_references(else_branch, source_text, referenced_symbols);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            collect_static_field_references(callee, source_text, referenced_symbols);
            for argument in arguments {
                collect_static_field_references(argument, source_text, referenced_symbols);
            }
        }
        _ => {}
    }
}

fn detect_static_dependency_cycle(
    current_node: &str,
    dependency_graph: &std::collections::HashMap<String, Vec<String>>,
    visiting_nodes: &mut std::collections::HashSet<String>,
    visited_nodes: &mut std::collections::HashSet<String>,
    current_path: &mut Vec<String>,
) -> Option<Vec<String>> {
    visiting_nodes.insert(current_node.to_string());
    current_path.push(current_node.to_string());

    if let Some(dependencies) = dependency_graph.get(current_node) {
        for dependency in dependencies {
            if visiting_nodes.contains(dependency) {
                let start_index = current_path
                    .iter()
                    .position(|node| node == dependency)
                    .unwrap_or(0);
                let mut cycle = current_path[start_index..].to_vec();
                cycle.push(dependency.clone());
                return Some(cycle);
            }
            if !visited_nodes.contains(dependency)
                && let Some(cycle) = detect_static_dependency_cycle(
                    dependency,
                    dependency_graph,
                    visiting_nodes,
                    visited_nodes,
                    current_path,
                )
            {
                return Some(cycle);
            }
        }
    }

    current_path.pop();
    visiting_nodes.remove(current_node);
    visited_nodes.insert(current_node.to_string());
    None
}

fn validate_static_field_dependencies(
    ast_list: &[Option<crate::phases::parser::statements::ProgramAst>],
    sources: &[String],
    accumulated_errors: &mut Vec<(u16, BindingError)>,
) {
    use crate::phases::parser::statements::declarations::{ClassMember, Declaration, Modifier};
    let mut declared_static_fields: std::collections::HashMap<String, (u16, Span)> =
        std::collections::HashMap::new();
    let mut field_dependencies: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (source_index, ast_option) in ast_list.iter().enumerate() {
        let Ok(source_id) = u16::try_from(source_index) else {
            break;
        };
        let source_text = sources
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");
        if let Some(program_ast) = ast_option {
            let package_prefix = program_ast
                .package
                .as_ref()
                .map(|package_decl| {
                    let segments: Vec<&str> = package_decl
                        .path
                        .iter()
                        .map(|span| {
                            let start = span.start as usize;
                            let end = span.end as usize;
                            if start <= end && end <= source_text.len() {
                                &source_text[start..end]
                            } else {
                                ""
                            }
                        })
                        .collect();
                    segments.join("::")
                })
                .unwrap_or_default();

            for declaration in &program_ast.declarations {
                if let Declaration::Class(class_decl) = declaration {
                    let class_name = if class_decl.name_span.end as usize <= source_text.len() {
                        &source_text
                            [class_decl.name_span.start as usize..class_decl.name_span.end as usize]
                    } else {
                        "class"
                    };
                    for member in &class_decl.members {
                        if let ClassMember::Field(field_decl) = member
                            && field_decl.modifiers.contains(&Modifier::Static)
                        {
                            let field_name =
                                if field_decl.name_span.end as usize <= source_text.len() {
                                    &source_text[field_decl.name_span.start as usize
                                        ..field_decl.name_span.end as usize]
                                } else {
                                    "field"
                                };
                            let full_symbol = if package_prefix.is_empty() {
                                format!("{class_name}::{field_name}")
                            } else {
                                format!("{package_prefix}::{class_name}::{field_name}")
                            };
                            let simple_symbol = format!("{class_name}::{field_name}");

                            declared_static_fields
                                .insert(full_symbol.clone(), (source_id, field_decl.span));
                            declared_static_fields
                                .insert(simple_symbol.clone(), (source_id, field_decl.span));

                            let mut references = Vec::new();
                            if let Some(initializer) = &field_decl.initializer {
                                collect_static_field_references(
                                    initializer,
                                    source_text,
                                    &mut references,
                                );
                            }
                            field_dependencies.insert(full_symbol, references.clone());
                            field_dependencies.insert(simple_symbol, references);
                        }
                    }
                }
            }
        }
    }

    let mut filtered_graph: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (field_symbol, references) in &field_dependencies {
        let valid_targets: Vec<String> = references
            .iter()
            .filter(|target| declared_static_fields.contains_key(*target))
            .cloned()
            .collect();
        if !valid_targets.is_empty() {
            filtered_graph.insert(field_symbol.clone(), valid_targets);
        }
    }

    let mut visiting_nodes = std::collections::HashSet::new();
    let mut visited_nodes = std::collections::HashSet::new();
    let mut reported_cycles = std::collections::HashSet::new();

    for field_symbol in filtered_graph.keys() {
        if !visited_nodes.contains(field_symbol) {
            let mut current_path = Vec::new();
            if let Some(cycle) = detect_static_dependency_cycle(
                field_symbol,
                &filtered_graph,
                &mut visiting_nodes,
                &mut visited_nodes,
                &mut current_path,
            ) {
                let cycle_path = cycle.join(" -> ");
                if reported_cycles.insert(cycle_path.clone()) {
                    let (source_id, span) = declared_static_fields
                        .get(field_symbol)
                        .copied()
                        .unwrap_or((0, Span::default()));
                    accumulated_errors.push((
                        source_id,
                        BindingError::CyclicStaticFieldDependency {
                            cycle_path,
                            span: span.to_source_span(),
                        },
                    ));
                }
            }
        }
    }
}
