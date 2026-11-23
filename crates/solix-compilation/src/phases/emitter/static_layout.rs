//! Static memory partitioning and module initializer (@init) synthesis.

#![forbid(unsafe_code)]

use crate::phases::binder::layout::ClassLayout;
use crate::phases::emitter::constant_pool::ConstantPoolBuilder;
use crate::phases::emitter::emitter::Emitter;
use crate::phases::emitter::register_allocator::RegisterAllocator;
use crate::phases::parser::statements::expressions::Expression;
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode};
use solix_common::diagnostics::EmissionError;
use solix_common::span::Span;
use solix_common::types::Type;
use std::collections::HashMap;

/// Declaration of a static variable with its optional initializer expression.
#[derive(Debug, Clone)]
pub struct StaticVariableDecl {
    /// Fully-qualified static variable symbol name (e.g. `solix::models::User::count`).
    pub symbol: String,
    /// Source code span of the declaration.
    pub span: Span,
    /// Optional initialization expression.
    pub initializer: Option<Expression>,
}

/// Static memory partition manager maintaining local static slot indices and exported symbols.
#[derive(Debug, Clone, Default)]
pub struct StaticLayout {
    /// Mapping of fully-qualified static symbols to local 0-based static slot offsets.
    pub symbol_to_slot: HashMap<String, u32>,
    /// Exported symbol map for binary container metadata.
    pub static_symbols_map: Vec<(String, u32)>,
    /// Total number of static memory slots required by this module.
    pub static_space_size: u32,
}

impl StaticLayout {
    /// Creates a new empty static layout manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns the next sequential static slot index to `symbol`.
    pub fn assign_static(&mut self, symbol: impl Into<String>) -> u32 {
        let sym = symbol.into();
        if let Some(&existing_slot) = self.symbol_to_slot.get(&sym) {
            return existing_slot;
        }
        let slot = self.static_space_size;
        self.static_space_size = self.static_space_size.saturating_add(1);
        self.symbol_to_slot.insert(sym.clone(), slot);
        self.static_symbols_map.push((sym, slot));
        slot
    }

    /// Looks up the local static slot assigned to `symbol`.
    #[must_use]
    pub fn lookup_slot(&self, symbol: &str) -> Option<u32> {
        self.symbol_to_slot.get(symbol).copied()
    }
}

fn collect_expr_symbol_references(
    expression: &Expression,
    sources: Option<&[String]>,
    referenced_symbols: &mut Vec<String>,
) {
    let get_span_text = |span: Span| -> Option<&str> {
        let source_list = sources?;
        let source_index = span.id as usize;
        let source_code = source_list.get(source_index)?;
        let start_offset = span.start as usize;
        let end_offset = span.end as usize;
        if start_offset <= end_offset && end_offset <= source_code.len() {
            Some(&source_code[start_offset..end_offset])
        } else {
            None
        }
    };

    match expression {
        Expression::MemberAccess {
            object,
            member_span,
            ..
        } => {
            if let Some(member_name) = get_span_text(*member_span)
                && let Expression::Identifier { span: target_span } = &**object
                && let Some(object_name) = get_span_text(*target_span)
            {
                referenced_symbols.push(format!("{object_name}::{member_name}"));
            }
            collect_expr_symbol_references(object, sources, referenced_symbols);
        }
        Expression::Identifier { span } => {
            if let Some(name) = get_span_text(*span) {
                referenced_symbols.push(name.to_string());
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_expr_symbol_references(left, sources, referenced_symbols);
            collect_expr_symbol_references(right, sources, referenced_symbols);
        }
        Expression::Unary { operand, .. } => {
            collect_expr_symbol_references(operand, sources, referenced_symbols);
        }
        Expression::Assignment { target, value, .. } => {
            collect_expr_symbol_references(target, sources, referenced_symbols);
            collect_expr_symbol_references(value, sources, referenced_symbols);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_symbol_references(condition, sources, referenced_symbols);
            collect_expr_symbol_references(then_branch, sources, referenced_symbols);
            collect_expr_symbol_references(else_branch, sources, referenced_symbols);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            collect_expr_symbol_references(callee, sources, referenced_symbols);
            for argument in arguments {
                collect_expr_symbol_references(argument, sources, referenced_symbols);
            }
        }
        _ => {}
    }
}

fn topologically_sort_static_variables(
    variables: &[StaticVariableDecl],
    sources: Option<&[String]>,
) -> Vec<StaticVariableDecl> {
    if variables.len() <= 1 {
        return variables.to_vec();
    }

    let mut symbol_to_index: HashMap<String, usize> = HashMap::new();
    for (index, variable) in variables.iter().enumerate() {
        symbol_to_index.insert(variable.symbol.clone(), index);
        let segments: Vec<&str> = variable.symbol.split("::").collect();
        if segments.len() >= 2 {
            let class_field = format!(
                "{}::{}",
                segments[segments.len() - 2],
                segments[segments.len() - 1]
            );
            symbol_to_index.entry(class_field).or_insert(index);
        }
    }

    let mut in_degree: Vec<usize> = vec![0; variables.len()];
    let mut adjacency_list: Vec<Vec<usize>> = vec![Vec::new(); variables.len()];

    for (index, variable) in variables.iter().enumerate() {
        if let Some(initializer) = &variable.initializer {
            let mut references = Vec::new();
            collect_expr_symbol_references(initializer, sources, &mut references);
            let mut seen_dependencies = std::collections::HashSet::new();
            for reference in references {
                if let Some(&dep_index) = symbol_to_index.get(&reference)
                    && dep_index != index
                    && seen_dependencies.insert(dep_index)
                {
                    adjacency_list[dep_index].push(index);
                    in_degree[index] += 1;
                }
            }
        }
    }

    let mut ready_queue = std::collections::VecDeque::new();
    for (index, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            ready_queue.push_back(index);
        }
    }

    let mut sorted_indices = Vec::with_capacity(variables.len());
    while let Some(current_index) = ready_queue.pop_front() {
        sorted_indices.push(current_index);
        for &dependent_index in &adjacency_list[current_index] {
            in_degree[dependent_index] = in_degree[dependent_index].saturating_sub(1);
            if in_degree[dependent_index] == 0 {
                ready_queue.push_back(dependent_index);
            }
        }
    }

    if sorted_indices.len() < variables.len() {
        for index in 0..variables.len() {
            if !sorted_indices.contains(&index) {
                sorted_indices.push(index);
            }
        }
    }

    sorted_indices
        .into_iter()
        .map(|index| variables[index].clone())
        .collect()
}

/// Synthesizes the module static initializer function `@init` (Function ID 0).
///
/// `@init` executes all global and static variable initializers in dependency order,
/// emitting `SET_STATIC slot, reg` to populate the allocated static partition before `main()` executes.
///
/// # Errors
/// Returns `EmissionError` if expression evaluation exceeds register or pool limits.
pub fn synthesize_module_init(
    variables: &[StaticVariableDecl],
    layout: &StaticLayout,
    pool: &mut ConstantPoolBuilder,
    node_types: Option<&HashMap<Span, Type>>,
    sources: Option<&[String]>,
    function_ids: Option<&HashMap<String, u32>>,
    class_layouts: Option<&HashMap<String, ClassLayout>>,
) -> Result<CompiledFunction, EmissionError> {
    let mut init_func = CompiledFunction::new("@init", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);

    {
        let mut emitter =
            Emitter::new(&mut init_func, &mut allocator, pool).with_static_layout(layout);
        if let Some(types) = node_types {
            emitter = emitter.with_node_types(types);
        }
        if let Some(srcs) = sources {
            emitter = emitter.with_sources(srcs);
        }
        if let Some(fids) = function_ids {
            emitter = emitter.with_function_ids(fids);
        }
        if let Some(layouts) = class_layouts {
            emitter = emitter.with_class_layouts(layouts);
        }

        let sorted_variables = topologically_sort_static_variables(variables, sources);

        for variable_decl in &sorted_variables {
            if let Some(init_expr) = &variable_decl.initializer {
                let slot = layout.lookup_slot(&variable_decl.symbol).ok_or_else(|| {
                    EmissionError::SymbolNotFound {
                        symbol: variable_decl.symbol.clone(),
                    }
                })?;

                let result_register = emitter.emit_expression(init_expr, None)?;
                // SET_STATIC src_reg, slot_offset
                emitter.emit_instruction(
                    Instruction::format_b(OpCode::SetStatic, 0, result_register, slot),
                    Some(variable_decl.span),
                );
                if emitter.query_type(init_expr.span()).is_reference_type()
                    && !matches!(init_expr, Expression::Identifier { .. })
                {
                    emitter.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, result_register, 0, 0),
                        Some(variable_decl.span),
                    );
                }
                emitter.allocator.free_temp(result_register);
            }
        }

        // Module @init ends with void return
        emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
    }

    init_func.num_registers = allocator.peak_registers();
    Ok(init_func)
}
