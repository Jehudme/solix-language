//! Host native FFI and core built-in functions.

#![forbid(unsafe_code)]

pub mod builtins;
pub mod registry;

pub use registry::{NativeFunction, NativeRegistry};
