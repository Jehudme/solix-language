//! Master test entry for all Solix virtual machine and memory runtime suites.

#![forbid(unsafe_code)]

#[path = "runtime/test_virtual_heap.rs"]
mod test_virtual_heap;

#[path = "runtime/test_object_model.rs"]
mod test_object_model;

#[path = "runtime/test_error_codes_and_entry_point.rs"]
mod test_error_codes_and_entry_point;

#[path = "runtime/test_exceptions.rs"]
mod test_exceptions;

#[path = "runtime/test_call_and_returns.rs"]
mod test_call_and_returns;

#[path = "runtime/test_closures_lambdas.rs"]
mod test_closures_lambdas;

#[path = "runtime/test_options_and_context.rs"]
mod test_options_and_context;

#[path = "runtime/test_native_ffi.rs"]
mod test_native_ffi;

#[path = "runtime/test_control_flow.rs"]
mod test_control_flow;

#[path = "runtime/test_basic_dispatch.rs"]
mod test_basic_dispatch;

#[path = "runtime/test_arc_lifecycle.rs"]
mod test_arc_lifecycle;

#[path = "runtime/test_arithmetic_bitwise.rs"]
mod test_arithmetic_bitwise;

#[path = "runtime/test_arrays_tuples.rs"]
mod test_arrays_tuples;

#[path = "runtime/test_multi_binary_dynamic_linking.rs"]
mod test_multi_binary_dynamic_linking;
