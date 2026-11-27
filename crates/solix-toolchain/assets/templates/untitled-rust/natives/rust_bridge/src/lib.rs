//! Rust native bridge extension crate for Solix.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

pub type NativeCFunctionPtr = unsafe extern "C-unwind" fn(
    *mut std::ffi::c_void,
    *const SolixCValue,
    u32,
    *mut SolixCValue,
) -> SolixCValue;

pub type RegisterNativeCallback = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const std::ffi::c_char,
    NativeCFunctionPtr,
);

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn MathBridge_fast_add(
    _vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue {
            tag: 0,
            flags: 0,
            payload: 0,
        };
    }
    let first = unsafe { (*args.add(0)).payload as i32 };
    let second = unsafe { (*args.add(1)).payload as i32 };
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: ((first + second) as i64) as u64,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut std::ffi::c_void,
    register_callback: RegisterNativeCallback,
) {
    let add_symbol = c"MathBridge::fast_add";
    unsafe {
        register_callback(
            collector_context,
            add_symbol.as_ptr(),
            MathBridge_fast_add,
        );
    }
}
