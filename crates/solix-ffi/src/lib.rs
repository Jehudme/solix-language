//! Official Foreign Function Interface (FFI) SDK and safe abstractions for Solix.
//!
//! Provides the stable C-ABI tagged value structure (`SolixCValue`), VM context
//! accessor functions, safe marshaling helpers, and the `solix_export!` registration
//! macro for authoring native shared library extensions in Rust.

pub mod context;
pub mod registry;
pub mod value;

pub use context::{SOLIX_VM_CONTEXT_MAGIC, SolixVmContext};
pub use registry::{NativeCFunctionPtr, NativeRegistry, RegisterNativeCallback};
pub use value::{SolixCValue, TAG_BOOL, TAG_F64, TAG_FNPTR, TAG_I32, TAG_I64, TAG_NULL, TAG_REF};
