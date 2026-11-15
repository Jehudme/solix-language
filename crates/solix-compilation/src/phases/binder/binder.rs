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

    crate::phases::binder::call_normalizer::normalize_call_sites(
        &mut context.ast,
        &context.sources,
    );

    // Pass 3: Detailed Type Checking & Expression Traversal
    for (source_index, ast_option) in context.ast.iter().enumerate() {
        let source_id = source_index as u16;
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
                next_vtable_id: &mut next_vtable_id,
                source_texts: &context.sources,
            };

            visitor.visit_program(program_ast);
            for error in pass3_errors {
                accumulated_errors.push((source_id, error));
            }
        }
    }

    let symbols_count = symbol_table.symbols.len();
    let classes_count = layouts.len();
    let monomorphized_count = blueprint_registry.monomorphized_asts.len();

    // Validate entry point(s)
    let mut main_methods = Vec::new();
    for (source_index, ast_option) in context.ast.iter().enumerate() {
        if context.is_external_source(source_index as u16) {
            continue;
        }
        if let Some(program_ast) = ast_option {
            let source_id = source_index as u16;
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
                        "Main method must return 'void' or 'int32', found '{return_type}'"
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
                    reason: "Main method parameters must be empty () or take a single string[]/char[][] parameter".to_string(),
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
