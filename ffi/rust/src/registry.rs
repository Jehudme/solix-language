//! Function pointer types and registration helpers for native extension libraries.

use crate::context::SolixVmContext;
use crate::value::SolixCValue;
use std::any::Any;
use std::ffi::{CString, c_char, c_void};

/// Stable C-ABI function pointer signature implemented by all native Solix bridge functions.
pub type NativeCFunctionPtr = unsafe extern "C-unwind" fn(
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

/// Extracts a human-readable panic description and records it in `out_error`.
fn report_panic_error(
    vm_context: *mut c_void,
    out_error: *mut SolixCValue,
    panic_payload: Box<dyn Any + Send>,
) {
    if out_error.is_null() {
        return;
    }
    let panic_detail = if let Some(static_msg) = panic_payload.downcast_ref::<&str>() {
        *static_msg
    } else if let Some(owned_msg) = panic_payload.downcast_ref::<String>() {
        owned_msg.as_str()
    } else {
        "unhandled Rust panic inside native plugin"
    };
    let formatted_reason = format!("native plugin panicked: {panic_detail}");
    if let Some(ctx) = unsafe { SolixVmContext::validate(vm_context) } {
        unsafe { ctx.set_error(out_error, &formatted_reason) };
    } else {
        unsafe {
            *out_error = SolixCValue::from_bool(false);
        }
    }
}

/// Runs a native callback inside `std::panic::catch_unwind`, converting any panic into `out_error`.
///
/// # Safety
/// If `out_error` is non-null, it must point to writable `SolixCValue` storage provided by the VM.
pub unsafe fn invoke_caught<CallbackFn>(
    vm_context: *mut c_void,
    out_error: *mut SolixCValue,
    callback_body: CallbackFn,
) -> SolixCValue
where
    CallbackFn: FnOnce() -> SolixCValue,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback_body)) {
        Ok(result) => result,
        Err(panic_payload) => {
            report_panic_error(vm_context, out_error, panic_payload);
            SolixCValue::null()
        }
    }
}

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

    /// Validates optional host registration arguments and creates a registry when non-null.
    #[must_use]
    pub fn from_raw(
        collector_context: *mut c_void,
        register_callback: Option<RegisterNativeCallback>,
    ) -> Option<Self> {
        if collector_context.is_null() {
            return None;
        }
        let callback = register_callback?;
        Some(Self::new(collector_context, callback))
    }

    /// Registers a native function symbol with the Solix virtual machine.
    /// Symbols containing embedded interior null bytes or a null collector context are ignored.
    pub fn register(&mut self, symbol_name: &str, function_pointer: NativeCFunctionPtr) {
        if self.collector_context.is_null() {
            return;
        }
        if let Ok(c_symbol) = CString::new(symbol_name) {
            unsafe {
                (self.register_callback)(
                    self.collector_context,
                    c_symbol.as_ptr(),
                    function_pointer,
                );
            }
        }
    }
}

/// Convenience macro for declaring and exporting the `solix_register_natives` entrypoint.
///
/// Automatically wraps each registered native callback in a panic-catching `extern "C-unwind"`
/// trampoline and guards against null collector/callback host pointers.
///
/// # Example
/// ```rust,no_run
/// use solix_ffi::{solix_export, NativeCFunctionPtr, SolixCValue};
///
/// pub unsafe extern "C-unwind" fn fast_add(
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
            register_callback: Option<$crate::RegisterNativeCallback>,
        ) {
            let Some(mut registry) =
                $crate::NativeRegistry::from_raw(collector_context, register_callback)
            else {
                return;
            };
            $(
                {
                    unsafe extern "C-unwind" fn __solix_export_trampoline(
                        vm_context: *mut std::ffi::c_void,
                        arguments: *const $crate::SolixCValue,
                        argument_count: u32,
                        out_error: *mut $crate::SolixCValue,
                    ) -> $crate::SolixCValue {
                        unsafe {
                            $crate::invoke_caught(vm_context, out_error, || {
                                $function(vm_context, arguments, argument_count, out_error)
                            })
                        }
                    }
                    registry.register($symbol, __solix_export_trampoline);
                }
            )*
        }
    };
}
