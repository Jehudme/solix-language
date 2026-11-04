//! Exception handling, zero-cost unwinding, and runtime diagnostic types.

#![forbid(unsafe_code)]

pub mod errors;
pub mod unwinder;

pub use errors::{RuntimeError, RuntimeErrorCode};
pub use unwinder::find_matching_handler;
