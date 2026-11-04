//! Object model, member fields, dynamic arrays, tuples, and closures.

#![forbid(unsafe_code)]

pub mod array;
pub mod closure;
pub mod object;
pub mod tuple;
pub mod vtable;

pub use array::ARRAY_HEADER_PAYLOAD_SIZE;
pub use closure::CLOSURE_HEADER_PAYLOAD_SIZE;
pub use object::OBJECT_HEADER_PAYLOAD_SIZE;
pub use tuple::TUPLE_HEADER_PAYLOAD_SIZE;
pub use vtable::{resolve_interface_method, resolve_virtual_method};
