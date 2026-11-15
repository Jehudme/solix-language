//! Function pointer types and registration helpers for native extension libraries.

use crate::value::SolixCValue;
use std::ffi::{c_char, c_void, CString};

/// Stable C-ABI function pointer signature implemented by all native Solix bridge functions.
pub type NativeCFunctionPtr = unsafe extern "C" fn(
    vm_context: *mut c_void,
    arguments: *const SolixCValue,
    argument_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;

/// Host callback function invoked during extension initialization to bind exported symbols.
pub type RegisterNativeCallback = unsafe extern "C" fn(
    collector_context: *mut c_void,
    symbol_name_pointer: *const c_char,
    function_pointer: NativeCFunctionPtr,
);

/// Safe registration builder provided to native modules during `solix_register_natives`.
pub struct NativeRegistry {
    collector_context: *mut c_void,
    register_callback: RegisterNativeCallback,
}

impl NativeRegistry {
    /// Creates a new native registration scope from host callback pointers.
    #[must_use]
    pub fn new(collector_context: *mut c_void, register_callback: RegisterNativeCallback) -> Self {
        Self {
            collector_context,
            register_callback,
        }
    }

    /// Registers a native function symbol with the Solix virtual machine.
    ///
    /// # Panics
    /// Panics if `symbol_name` contains an embedded interior null byte.
    pub fn register(&mut self, symbol_name: &str, function_pointer: NativeCFunctionPtr) {
        let c_symbol = CString::new(symbol_name)
            .expect("Symbol name cannot contain embedded null characters");
        unsafe {
            (self.register_callback)(
                self.collector_context,
                c_symbol.as_ptr(),
                function_pointer,
            );
        }
    }
}

/// Convenience macro for declaring and exporting the `solix_register_natives` entrypoint.
///
/// # Example
/// ```rust,no_run
/// use solix_ffi::{solix_export, NativeCFunctionPtr, SolixCValue};
///
/// pub unsafe extern "C" fn fast_add(
///     _context: *mut std::ffi::c_void,
///     arguments: *const SolixCValue,
///     count: u32,
///     _error: *mut SolixCValue,
/// ) -> SolixCValue {
///     if count < 2 || arguments.is_null() {
///         return SolixCValue::null();
///     }
///     let value_a = unsafe { (*arguments.add(0)).as_i32() };
///     let value_b = unsafe { (*arguments.add(1)).as_i32() };
///     SolixCValue::from_i32(value_a + value_b)
/// }
///
/// solix_export! {
///     "MathBridge::fast_add" => fast_add,
/// }
/// ```
#[macro_export]
macro_rules! solix_export {
    ($($symbol:expr => $function:ident),* $(,)?) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn solix_register_natives(
            collector_context: *mut std::ffi::c_void,
            register_callback: $crate::RegisterNativeCallback,
        ) {
            let mut registry = $crate::NativeRegistry::new(collector_context, register_callback);
            $(
                registry.register($symbol, $function);
            )*
        }
    };
}
