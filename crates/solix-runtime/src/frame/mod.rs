//! Virtual machine call frame stack and activation records.

#![forbid(unsafe_code)]

pub mod call_frame;

pub use call_frame::CallFrame;
