//! Dynamic array layout and bounds verification.

#![forbid(unsafe_code)]

/// Array payload header byte size: 4 bytes length + 4 bytes element size.
pub const ARRAY_HEADER_PAYLOAD_SIZE: usize = 8;
