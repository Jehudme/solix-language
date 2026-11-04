//! Heterogeneous tuple layout and slot indexing.

#![forbid(unsafe_code)]

/// Tuple payload header byte size: 4 bytes length + 4 bytes slot stride.
pub const TUPLE_HEADER_PAYLOAD_SIZE: usize = 8;
