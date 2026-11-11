//! Compiled function representation, exception tables, and instruction source mapping.

#![forbid(unsafe_code)]

use crate::bytecode::instruction::Instruction;
use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Zero-cost exception handler table entry defining PC ranges and catch/finally targets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExceptionHandlerEntry {
    /// Start instruction index of the protected `try` block (inclusive).
    pub try_start_pc: u32,
    /// End instruction index of the protected `try` block (exclusive).
    pub try_end_pc: u32,
    /// Catch type symbol name, or `None` if this entry handles all exceptions (catch-all).
    pub catch_type_symbol: Option<String>,
    /// Register allocated to hold the caught exception object in the handler block.
    pub catch_reg: u16,
    /// Instruction PC of the catch handler block.
    pub handler_pc: u32,
    /// Instruction PC of the finally block for stack unwinding, if present.
    pub finally_pc: Option<u32>,
    /// Registers allocated for reference-counted values inside this try block for RAII unwinding.
    pub try_ref_regs: Vec<u16>,
}

/// Source coordinate mapping linking an emitted instruction PC to its original source code span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceMapEntry {
    /// Zero-based instruction offset within the compiled function.
    pub instruction_pc: u32,
    /// Source code token span corresponding to the instruction.
    pub span: Span,
    /// 1-based source line number.
    #[serde(default)]
    pub line: u32,
    /// 1-based source column number.
    #[serde(default)]
    pub column: u32,
}

/// Maps a virtual register within a compiled function to its source variable name and type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalVariableDebugInfo {
    /// Virtual register index (`r0..rN`).
    pub register: u16,
    /// User-declared variable or parameter name.
    pub name: String,
    /// Static type name (e.g. `"int32"`, `"sample_workspace::Point"`).
    pub type_name: String,
    /// Static compiler `Type`.
    pub var_type: crate::types::Type,
    /// Inclusive starting bytecode PC where this variable enters scope.
    pub start_pc: u32,
    /// Exclusive ending bytecode PC where this variable exits scope.
    pub end_pc: u32,
    /// Whether this register is a function/method parameter (`r0` = `this` for instance methods).
    pub is_parameter: bool,
}

/// Self-contained compiled function metadata and bytecode chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledFunction {
    /// Fully qualified function signature symbol (e.g. `solix::models::User::calculate_total(int64, f64)`).
    pub symbol: String,
    /// Identifier of the source file from which this function was compiled.
    pub source_id: u16,
    /// Number of parameters accepted by this function.
    pub param_count: u16,
    /// Total peak register frame size allocated for this function's call frame.
    pub num_registers: u16,
    /// Base offset into the runtime's contiguous static memory partition.
    pub static_base_offset: u32,
    /// 64-bit aligned virtual machine instructions.
    pub instructions: Vec<Instruction>,
    /// Sorted list of register indices that hold reference-counted heap values (used for ARC stack cleanup).
    pub reference_registers: Vec<u16>,
    /// Table of zero-cost exception handlers.
    pub exception_handlers: Vec<ExceptionHandlerEntry>,
    /// Instruction-to-source debug mapping.
    pub source_map: Vec<SourceMapEntry>,
    /// Local variables and parameters debug information.
    #[serde(default)]
    pub local_variables: Vec<LocalVariableDebugInfo>,
}

impl CompiledFunction {
    /// Creates a new compiled function with the specified metadata.
    #[must_use]
    pub fn new(
        symbol: impl Into<String>,
        source_id: u16,
        param_count: u16,
        num_registers: u16,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            source_id,
            param_count,
            num_registers,
            static_base_offset: 0,
            instructions: Vec::new(),
            reference_registers: Vec::new(),
            exception_handlers: Vec::new(),
            source_map: Vec::new(),
            local_variables: Vec::new(),
        }
    }

    /// Appends an instruction and records its optional source span in the source map.
    pub fn emit(&mut self, instruction: Instruction, span: Option<Span>) -> u32 {
        self.emit_with_location(instruction, span, 0, 0)
    }

    /// Appends an instruction and records its source span, line, and column coordinates.
    pub fn emit_with_location(
        &mut self,
        instruction: Instruction,
        span: Option<Span>,
        line: u32,
        column: u32,
    ) -> u32 {
        let pc = self.instructions.len() as u32;
        self.instructions.push(instruction);
        if let Some(source_span) = span {
            self.source_map.push(SourceMapEntry {
                instruction_pc: pc,
                span: source_span,
                line,
                column,
            });
        }
        pc
    }
}
