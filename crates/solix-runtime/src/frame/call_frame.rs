//! Virtual machine call frame stack structure.

#![forbid(unsafe_code)]

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
    /// Contiguous static memory base slot offset for intra-binary static accesses.
    pub static_base_offset: u32,
    /// Intra-binary function index offset for zero-cost direct calls.
    pub function_id_offset: u32,
    /// Optional non-null heap payload address of the closure environment associated with this frame.
    pub active_closure: Option<NonZeroU32>,
    /// Binary container index that defined this function.
    pub binary_id: u16,
    /// Destination register index in the caller's frame where the return value is written.
    pub caller_dst: u16,
}

const _: () = assert!(std::mem::size_of::<CallFrame>() == 32);

impl CallFrame {
    /// Creates a new `CallFrame`.
    #[must_use]
    pub fn new(
        function_id: u32,
        binary_id: usize,
        register_base: usize,
        register_count: usize,
        static_base_offset: u32,
        function_id_offset: u32,
        caller_dst: u16,
    ) -> Self {
        Self {
            function_id,
            pc: 0,
            register_base: register_base as u32,
            register_count: register_count as u32,
            static_base_offset,
            function_id_offset,
            active_closure: None,
            binary_id: binary_id as u16,
            caller_dst,
        }
    }
}
