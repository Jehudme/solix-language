//! Register-based bytecode emitter, register allocator, and binary container generation.

#![forbid(unsafe_code)]

pub mod closures;
pub mod constant_pool;
pub mod control_flow;
pub mod disassembler;
#[allow(clippy::module_inception)]
pub mod emitter;
pub mod exceptions;
pub mod object_model;
pub mod pipeline;
pub mod register_allocator;
pub mod static_layout;

pub use closures::{find_free_variables, synthesize_lambda_function};
pub use constant_pool::ConstantPoolBuilder;
pub use control_flow::LoopContext;
pub use disassembler::{disassemble_function, disassemble_instruction, disassemble_program};
pub use emitter::Emitter;
pub use object_model::compile_class_layout;
pub use pipeline::{EmissionArtifacts, emit_context};
pub use register_allocator::{MAX_ARGUMENTS_PER_CALL, MAX_REGISTERS_PER_FRAME, RegisterAllocator};
pub use static_layout::{StaticLayout, StaticVariableDecl, synthesize_module_init};
