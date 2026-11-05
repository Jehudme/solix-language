//! Solix Virtual Machine Runtime & Virtual Memory System.
//!
//! Provides a register-based virtual machine execution engine, byte-addressed
//! virtual heap, relocatable multi-binary module coordination, and table-driven
//! zero-cost exception handling.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod context;
pub mod exceptions;
pub mod frame;
pub mod memory;
pub mod native;
pub mod objects;
pub mod options;
pub mod value;
pub mod verifier;
pub mod vm;

pub use context::{LoadedBinary, RuntimeContext};
pub use exceptions::{RuntimeError, RuntimeErrorCode};
pub use frame::CallFrame;
pub use memory::{
    ALIGNMENT, BlockHeader, BlockTag, FreeListManager, HEADER_SIZE, HeapTelemetry,
    LiveAllocationInfo, MAX_REF_COUNT, VirtualHeap, align_up, dec_ref, inc_ref,
};
pub use native::{
    LoadedSharedLibrary, NativeCFunctionPtr, NativeCallback, NativeFunction, NativeRegistry,
    SharedLibraryLoader, SolixCValue, SolixVmContext,
};
pub use options::{DEFAULT_HEAP_SIZE, DEFAULT_MAX_CALL_STACK_DEPTH, RuntimeOptions};
pub use value::Value;
pub use vm::{VirtualMachine, VmTelemetry};
