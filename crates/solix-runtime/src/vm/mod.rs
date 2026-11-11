//! Register-based virtual machine execution engine and instruction dispatch.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod debug;
pub mod engine;
pub mod fast;

pub use debug::{DebugSignal, VmDebugHook};
pub use engine::{VirtualMachine, VmTelemetry};
