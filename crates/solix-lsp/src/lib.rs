//! Solix Language Server Protocol implementation library.

#![forbid(unsafe_code)]

pub mod diagnostics;

pub use diagnostics::{
    binding_error_to_diagnostic, byte_offset_to_position, lexing_error_to_diagnostic,
    parsing_error_to_diagnostic, span_to_range,
};
