//! Register-based virtual machine execution engine and instruction dispatch.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod debug;
pub mod engine;
pub mod fast;
#[cfg(all(
    feature = "jit",
    any(target_arch = "x86_64", target_arch = "aarch64"),
    not(target_family = "wasm")
))]
pub mod jit;

pub use debug::{DebugSignal, VmDebugHook};
pub use engine::{VirtualMachine, VmTelemetry};
