//! Object layout and member field offset metadata.

#![forbid(unsafe_code)]

/// Object instance payload header byte size containing the 32-bit vtable index.
pub const OBJECT_HEADER_PAYLOAD_SIZE: usize = 8;
