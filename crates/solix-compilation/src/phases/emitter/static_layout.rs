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

/// Synthesizes the module static initializer function `@init` (Function ID 0).
///
/// `@init` executes all global and static variable initializers in declaration order,
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
        let mut emitter = Emitter::new(&mut init_func, &mut allocator, pool);
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

        for var in variables {
            if let Some(init_expr) = &var.initializer {
                let slot = layout.lookup_slot(&var.symbol).ok_or_else(|| {
                    EmissionError::SymbolNotFound {
                        symbol: var.symbol.clone(),
                    }
                })?;

                let r_val = emitter.emit_expression(init_expr, None)?;
                // SET_STATIC src_reg, slot_offset
                emitter.emit_instruction(
                    Instruction::format_b(OpCode::SetStatic, 0, r_val, slot),
                    Some(var.span),
                );
                if emitter.query_type(init_expr.span()).is_reference_type()
                    && !matches!(init_expr, Expression::Identifier { .. })
                {
                    emitter.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, r_val, 0, 0),
                        Some(var.span),
                    );
                }
                emitter.allocator.free_temp(r_val);
            }
        }

        // Module @init ends with void return
        emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
    }

    init_func.num_registers = allocator.peak_registers();
    Ok(init_func)
}
