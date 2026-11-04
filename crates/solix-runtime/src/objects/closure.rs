//! Closure environment layout and capture metadata.

#![forbid(unsafe_code)]

/// Closure payload header byte size: 4 bytes function ID + 4 bytes capture count.
pub const CLOSURE_HEADER_PAYLOAD_SIZE: usize = 8;
