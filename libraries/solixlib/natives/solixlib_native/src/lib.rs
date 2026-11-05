//! Rust native bridge extension crate for Solix Standard Library `solixlib` (`solixlib_native`).

#![allow(clippy::missing_safety_doc)]

pub mod console_io;
pub mod crypto;
pub mod fs_path;
pub mod math_time;
pub mod string_prim;
pub mod sys_process;

use std::ffi::{c_char, c_void};

/// Stable C-ABI tagged value passed across native shared library boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolixCValue {
    /// Tag: 0 = Null, 1 = Bool, 2 = I32, 3 = I64, 4 = F64, 5 = Ref, 6 = FnPtr
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

impl SolixCValue {
    #[must_use]
    pub const fn null() -> Self {
        Self {
            tag: 0,
            flags: 0,
            payload: 0,
        }
    }

    #[must_use]
    pub const fn bool_val(truth: bool) -> Self {
        Self {
            tag: 1,
            flags: 0,
            payload: if truth { 1 } else { 0 },
        }
    }

    #[must_use]
    pub const fn i32_val(val: i32) -> Self {
        Self {
            tag: 2,
            flags: 0,
            payload: (val as i64) as u64,
        }
    }

    #[must_use]
    pub const fn i64_val(val: i64) -> Self {
        Self {
            tag: 3,
            flags: 0,
            payload: val as u64,
        }
    }

    #[must_use]
    pub fn f64_val(val: f64) -> Self {
        Self {
            tag: 4,
            flags: 0,
            payload: val.to_bits(),
        }
    }

    #[must_use]
    pub const fn ref_val(addr: u32) -> Self {
        Self {
            tag: 5,
            flags: 0,
            payload: addr as u64,
        }
    }

    #[must_use]
    pub fn as_i32(&self) -> i32 {
        self.payload as i32
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.payload as i64
    }

    #[must_use]
    pub fn as_f64(&self) -> f64 {
        f64::from_bits(self.payload)
    }

    #[must_use]
    pub fn as_bool(&self) -> bool {
        self.payload != 0
    }

    #[must_use]
    pub fn as_ref(&self) -> u32 {
        self.payload as u32
    }
}

pub const SOLIX_VM_CONTEXT_MAGIC: u64 = 0x534F_4C49_5856_4D43;

/// Solix VM execution context passed into native C functions.
#[repr(C)]
pub struct SolixVmContext {
    pub magic: u64,
    pub vm_ptr: *mut c_void,
    pub get_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    pub set_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32, i32),
    pub read_bytes:
        unsafe extern "C" fn(vm_ptr: *mut c_void, addr: u32, out_buf: *mut u8, cap: u32) -> u32,
    pub alloc_string: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,
    pub alloc_chars: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,
    pub alloc_byte_array:
        unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,
    pub alloc_array: unsafe extern "C" fn(vm_ptr: *mut c_void, len: u32, type_tag: u8) -> u32,
    pub array_len: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32) -> u32,
    pub array_get_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32) -> u64,
    pub array_set_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32, raw_val: u64),
}

pub type NativeCFunctionPtr = unsafe extern "C" fn(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;

pub type RegisterNativeCallback = unsafe extern "C" fn(
    collector_context: *mut c_void,
    symbol: *const c_char,
    function_ptr: NativeCFunctionPtr,
);

/// Reads a Solix `string` or `char[]` at `address` into a Rust `String`.
pub unsafe fn read_solix_chars(vm_context: *mut c_void, address: u32) -> String {
    if address == 0 || vm_context.is_null() {
        return String::new();
    }
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return String::new();
    }
    let length = unsafe { (context_ref.array_len)(context_ref.vm_ptr, address) };
    if length == 0 {
        return String::new();
    }
    let mut buffer = vec![0u8; length as usize];
    let actual_read = unsafe {
        (context_ref.read_bytes)(context_ref.vm_ptr, address, buffer.as_mut_ptr(), length)
    };
    buffer.truncate(actual_read as usize);
    String::from_utf8(buffer).unwrap_or_default()
}

/// Reads a Solix byte array or `char[]` at `address` into a Rust `Vec<u8>`.
pub unsafe fn read_solix_bytes(vm_context: *mut c_void, address: u32) -> Vec<u8> {
    if address == 0 || vm_context.is_null() {
        return Vec::new();
    }
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return Vec::new();
    }
    let length = unsafe { (context_ref.array_len)(context_ref.vm_ptr, address) };
    if length == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0u8; length as usize];
    let actual_read = unsafe {
        (context_ref.read_bytes)(context_ref.vm_ptr, address, buffer.as_mut_ptr(), length)
    };
    buffer.truncate(actual_read as usize);
    buffer
}

/// Allocates a new `char[]` on the guest heap from a Rust string slice.
pub unsafe fn alloc_solix_chars(vm_context: *mut c_void, text: &str) -> u32 {
    if vm_context.is_null() {
        return 0;
    }
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return 0;
    }
    unsafe { (context_ref.alloc_chars)(context_ref.vm_ptr, text.as_ptr(), text.len() as u32) }
}

/// Allocates a new byte array (`int8[]`) on the guest heap.
pub unsafe fn alloc_solix_bytes(vm_context: *mut c_void, bytes: &[u8]) -> u32 {
    if vm_context.is_null() {
        return 0;
    }
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return 0;
    }
    unsafe {
        (context_ref.alloc_byte_array)(context_ref.vm_ptr, bytes.as_ptr(), bytes.len() as u32)
    }
}

/// Writes a 64-bit integer into an out-box array at index 0.
pub unsafe fn write_out_box_i64(vm_context: *mut c_void, out_box_address: u32, value: i64) {
    if out_box_address == 0 || vm_context.is_null() {
        return;
    }
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return;
    }
    unsafe {
        (context_ref.array_set_u64)(context_ref.vm_ptr, out_box_address, 0, value as u64);
    }
}

/// Writes a 64-bit float into an out-box array at index 0.
pub unsafe fn write_out_box_f64(vm_context: *mut c_void, out_box_address: u32, value: f64) {
    if out_box_address == 0 || vm_context.is_null() {
        return;
    }
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return;
    }
    unsafe {
        (context_ref.array_set_u64)(context_ref.vm_ptr, out_box_address, 0, value.to_bits());
    }
}

/// Registers a symbol with all canonical and mangled aliases.
pub unsafe fn register_symbol_variants(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
    variants: &[&str],
    function_ptr: NativeCFunctionPtr,
) {
    for variant in variants {
        let null_terminated = format!("{variant}\0");
        unsafe {
            callback(
                collector,
                null_terminated.as_ptr() as *const c_char,
                function_ptr,
            );
        }
    }
}

/// Main entrypoint exported by `solixlib_native` shared library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut c_void,
    register_callback: RegisterNativeCallback,
) {
    console_io::register_console_io_natives(collector_context, register_callback);
    fs_path::register_fs_path_natives(collector_context, register_callback);
    string_prim::register_string_prim_natives(collector_context, register_callback);
    math_time::register_math_time_natives(collector_context, register_callback);
    sys_process::register_sys_process_natives(collector_context, register_callback);
    crypto::register_crypto_natives(collector_context, register_callback);
}
