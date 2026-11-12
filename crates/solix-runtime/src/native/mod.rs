//! Host native FFI and core built-in functions.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod builtins;
pub mod ffi;
pub mod registry;

pub use ffi::{
    LoadedSharedLibrary, NativeCFunctionPtr, RegisterNativeCallback, SharedLibraryLoader,
    SolixCValue, SolixRegisterNativesHook, SolixVmContext, invoke_c_native, solix_vm_alloc_object,
    solix_vm_get_field_i32, solix_vm_read_field_u64, solix_vm_set_field_i32,
    solix_vm_write_field_u64, validate_platform_library_path,
};
pub use registry::{NativeCallback, NativeFunction, NativeRegistry};
