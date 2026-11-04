//! Virtual machine call frame stack structure.

#![forbid(unsafe_code)]

/// Activation record representing an in-flight function call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    /// Global function index within `RuntimeContext.all_functions`.
    pub function_id: u32,
    /// Binary container index that defined this function.
    pub binary_id: usize,
    /// Instruction program counter within the function's bytecode chunk.
    pub pc: u32,
    /// Offset in the unified `register_stack` where this frame's registers start.
    pub register_base: usize,
    /// Total register count allocated for this function frame.
    pub register_count: usize,
    /// Contiguous static memory base slot offset for intra-binary static accesses.
    pub static_base_offset: u32,
    /// Intra-binary function index offset for zero-cost direct calls.
    pub function_id_offset: u32,
    /// Destination register index in the caller's frame where the return value is written.
    pub caller_dst: u16,
    /// Optional heap payload address of the closure environment associated with this frame.
    pub active_closure: Option<u32>,
}

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
            binary_id,
            pc: 0,
            register_base,
            register_count,
            static_base_offset,
            function_id_offset,
            caller_dst,
            active_closure: None,
        }
    }
}
