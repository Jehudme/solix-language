//! Virtual machine call frame stack structure.

#![forbid(unsafe_code)]

use solix_common::bytecode::Instruction;
use std::num::NonZeroU32;

/// Activation record representing an in-flight function call (compact 32-byte layout).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallFrame {
    /// Global function index within `RuntimeContext.all_functions`.
    pub function_id: u32,
    /// Instruction program counter within the function's bytecode chunk.
    pub pc: u32,
    /// Offset in the unified `register_stack` where this frame's registers start.
    pub register_base: u32,
    /// Total register count allocated for this function frame.
    pub register_count: u32,
    /// Cached base pointer to the instruction slice of `function_id` (eliminates `func_code` lookups on return).
    pub inst_ptr: *const Instruction,
    /// Optional non-null heap payload address of the closure environment associated with this frame.
    pub active_closure: Option<NonZeroU32>,
    /// Binary container index that defined this function.
    pub binary_id: u16,
    /// Destination register index in the caller's frame where the return value is written.
    pub caller_dst: u16,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<CallFrame>() == 32);

impl CallFrame {
    /// Creates a new `CallFrame`.
    #[must_use]
    pub fn new(
        function_id: u32,
        binary_id: usize,
        register_base: usize,
        register_count: usize,
        inst_ptr: *const Instruction,
        caller_dst: u16,
    ) -> Self {
        Self {
            function_id,
            pc: 0,
            register_base: register_base as u32,
            register_count: register_count as u32,
            inst_ptr,
            active_closure: None,
            binary_id: binary_id as u16,
            caller_dst,
        }
    }
}
