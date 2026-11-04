//! Register-based virtual machine execution engine and instruction dispatch.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod engine;
pub mod fast;

pub use engine::{VirtualMachine, VmTelemetry};
