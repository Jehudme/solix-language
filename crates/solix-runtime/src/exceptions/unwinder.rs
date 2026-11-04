//! Zero-cost table-driven exception unwinding search.

#![forbid(unsafe_code)]

use solix_common::bytecode::ExceptionHandlerEntry;

/// Searches an instruction PC against the compiled exception handler table in reverse.
///
/// Reverse iteration ensures innermost nested try-catch blocks are matched first.
#[must_use]
pub fn find_matching_handler(
    handlers: &[ExceptionHandlerEntry],
    pc: u32,
) -> Option<&ExceptionHandlerEntry> {
    handlers
        .iter()
        .rev()
        .find(|h| pc >= h.try_start_pc && pc < h.try_end_pc)
}
