//! Foreign Function Interface (FFI) dynamic shared library bridge and C-ABI definitions.

use crate::exceptions::RuntimeError;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use std::ffi::{CStr, c_char, c_void};
use std::path::Path;

/// Stable C-ABI tagged value passed across native shared library boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolixCValue {
    /// Tag: 0 = Null, 1 = Bool, 2 = I32, 3 = I64, 4 = F64, 5 = Ref, 6 = FnPtr
    pub tag: u32,
    /// Padding / reserved flags
    pub flags: u32,
    /// 64-bit payload (i32, i64, f64 bits, or heap u32 reference address)
    pub payload: u64,
}

impl SolixCValue {
    /// Constructs a Null representation.
    #[must_use]
    pub const fn null() -> Self {
        Self {
            tag: 0,
            flags: 0,
            payload: 0,
        }
    }

    /// Constructs a Bool representation.
    #[must_use]
    pub const fn bool_val(truth: bool) -> Self {
        Self {
            tag: 1,
            flags: 0,
            payload: if truth { 1 } else { 0 },
        }
    }

    /// Constructs an I32 representation.
    #[must_use]
    pub const fn i32_val(integer_value: i32) -> Self {
        Self {
            tag: 2,
            flags: 0,
            payload: (integer_value as i64) as u64,
        }
    }

    /// Constructs an I64 representation.
    #[must_use]
    pub const fn i64_val(integer_value: i64) -> Self {
        Self {
            tag: 3,
            flags: 0,
            payload: integer_value as u64,
        }
    }

    /// Constructs an F64 representation.
    #[must_use]
    pub fn f64_val(float_value: f64) -> Self {
        Self {
            tag: 4,
            flags: 0,
            payload: float_value.to_bits(),
        }
    }

    /// Constructs a Ref representation.
    #[must_use]
    pub const fn ref_val(address: u32) -> Self {
        Self {
            tag: 5,
            flags: 0,
            payload: address as u64,
        }
    }

    /// Constructs an FnPtr representation.
    #[must_use]
    pub const fn fn_ptr(func_id: u32) -> Self {
        Self {
            tag: 6,
            flags: 0,
            payload: func_id as u64,
        }
    }

    /// Converts a high-level runtime `Value` into a C-ABI compatible `SolixCValue`.
    #[must_use]
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Null => Self::null(),
            Value::Bool(truth) => Self::bool_val(*truth),
            Value::I32(integer_value) => Self::i32_val(*integer_value),
            Value::I64(integer_value) => Self::i64_val(*integer_value),
            Value::F64(float_value) => Self::f64_val(*float_value),
            Value::Ref(address) => Self::ref_val(*address),
            Value::FnPtr(func_id) => Self::fn_ptr(*func_id),
        }
    }

    /// Converts this `SolixCValue` back into a high-level runtime `Value`.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self.tag {
            0 => Value::Null,
            1 => Value::Bool(self.payload != 0),
            2 => Value::I32(self.payload as i32),
            3 => Value::I64(self.payload as i64),
            4 => Value::F64(f64::from_bits(self.payload)),
            5 => Value::Ref(self.payload as u32),
            6 => Value::FnPtr(self.payload as u32),
            _ => Value::Null,
        }
    }
}

/// Magic identifier constant verifying a valid `SolixVmContext` structure.
pub const SOLIX_VM_CONTEXT_MAGIC: u64 = 0x534F_4C49_5856_4D43; // "SOLIXVMC"

/// Context structure passed to native C callbacks providing safe field accessor function pointers.
#[repr(C)]
pub struct SolixVmContext {
    /// Magic verification tag (`SOLIX_VM_CONTEXT_MAGIC`).
    pub magic: u64,
    /// Opaque pointer to the executing `VirtualMachine`.
    pub vm_ptr: *mut c_void,
    /// Function pointer to read a 32-bit integer field from an object handle.
    pub get_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    /// Function pointer to write a 32-bit integer field to an object handle.
    pub set_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32, i32),
}

/// Stable C-ABI native function pointer signature.
pub type NativeCFunctionPtr = unsafe extern "C" fn(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;

/// Registration callback provided to `solix_register_natives`.
pub type RegisterNativeCallback = unsafe extern "C" fn(
    collector_context: *mut c_void,
    symbol: *const c_char,
    function_ptr: NativeCFunctionPtr,
);

/// Optional batch registration hook exported by a shared library.
pub type SolixRegisterNativesHook =
    unsafe extern "C" fn(collector_context: *mut c_void, register_callback: RegisterNativeCallback);

/// Safely validates that a requested library path conforms to the host platform format.
pub fn validate_platform_library_path(library_path: &Path) -> Result<(), RuntimeError> {
    let extension = library_path
        .extension()
        .and_then(|ext_str| ext_str.to_str())
        .unwrap_or("");
    let path_string = library_path.display().to_string();

    #[cfg(target_os = "windows")]
    let expected_extension = "dll";
    #[cfg(target_os = "macos")]
    let expected_extension = "dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let expected_extension = "so";

    let is_foreign_extension = match extension {
        "dll" => cfg!(not(target_os = "windows")),
        "so" => cfg!(target_os = "windows") || cfg!(target_os = "macos"),
        "dylib" => cfg!(not(target_os = "macos")),
        _ => false,
    };

    if is_foreign_extension {
        return Err(RuntimeError::SharedLibraryError {
            path: path_string,
            reason: format!(
                "unsupported native library format for host platform (expected '.{expected_extension}')"
            ),
        });
    }

    Ok(())
}

/// Invokes a stable C-ABI native function pointer using high-level runtime values.
pub fn invoke_c_native(
    function_ptr: NativeCFunctionPtr,
    vm: &mut VirtualMachine,
    arguments: &[Value],
) -> Result<Value, RuntimeError> {
    let converted_arguments: Vec<SolixCValue> =
        arguments.iter().map(SolixCValue::from_value).collect();
    let mut out_error = SolixCValue::null();
    let vm_pointer = vm as *mut VirtualMachine as *mut c_void;
    let mut context_wrapper = SolixVmContext {
        magic: SOLIX_VM_CONTEXT_MAGIC,
        vm_ptr: vm_pointer,
        get_field_i32: solix_vm_get_field_i32,
        set_field_i32: solix_vm_set_field_i32,
    };
    let context_wrapper_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let result = unsafe {
        function_ptr(
            context_wrapper_ptr,
            converted_arguments.as_ptr(),
            converted_arguments.len() as u32,
            &mut out_error,
        )
    };

    if out_error.tag != 0 {
        return Err(RuntimeError::NativeFunctionError {
            name: "native_c_call".to_string(),
            reason: "error signaled by native C callback".to_string(),
        });
    }

    Ok(result.to_value())
}

/// Internal C callback for collecting registered symbols from `solix_register_natives`.
unsafe extern "C" fn collect_registration_callback(
    collector_context: *mut c_void,
    symbol_ptr: *const c_char,
    function_ptr: NativeCFunctionPtr,
) {
    if collector_context.is_null() || symbol_ptr.is_null() {
        return;
    }
    let collector = unsafe { &mut *(collector_context as *mut Vec<(String, NativeCFunctionPtr)>) };
    let c_str = unsafe { CStr::from_ptr(symbol_ptr) };
    if let Ok(symbol_string) = c_str.to_str() {
        collector.push((symbol_string.to_string(), function_ptr));
    }
}

/// Thread-safe reference-counted wrapper around an open dynamic shared library handle.
#[derive(Clone)]
pub struct LoadedSharedLibrary(pub std::sync::Arc<libloading::Library>);

impl std::fmt::Debug for LoadedSharedLibrary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("LoadedSharedLibrary")
    }
}

impl LoadedSharedLibrary {
    /// Probes this library for a symbol by its canonical, mangled, or unqualified name.
    #[must_use]
    pub fn get_symbol(&self, symbol_name: &str) -> Option<NativeCFunctionPtr> {
        let clean_symbol = symbol_name.split('(').next().unwrap_or(symbol_name);
        let mangled = clean_symbol.replace("::", "_");
        let unqualified = clean_symbol
            .rsplit(['.', ':'])
            .next()
            .unwrap_or(clean_symbol);

        let candidates = [clean_symbol, &mangled, unqualified];
        for candidate in &candidates {
            let null_terminated = format!("{candidate}\0");
            unsafe {
                if let Ok(symbol_handle) =
                    self.0.get::<NativeCFunctionPtr>(null_terminated.as_bytes())
                {
                    return Some(*symbol_handle);
                }
            }
        }
        None
    }
}

/// Dynamic shared library manager responsible for loading and resolving symbols across libraries.
pub struct SharedLibraryLoader {
    /// Open shared library handles held alive for the lifetime of execution.
    pub loaded_libraries: Vec<libloading::Library>,
    /// Discovered native functions mapped by registered symbol names.
    pub registered_functions: Vec<(String, NativeCFunctionPtr)>,
}

impl Default for SharedLibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedLibraryLoader {
    /// Creates a new empty `SharedLibraryLoader`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            loaded_libraries: Vec::new(),
            registered_functions: Vec::new(),
        }
    }

    /// Loads a shared library from disk, validating its format, and querying its registration hook.
    pub fn load_library(&mut self, library_path: &Path) -> Result<(), RuntimeError> {
        let path_string = library_path.display().to_string();
        validate_platform_library_path(library_path)?;

        if !library_path.exists() {
            return Err(RuntimeError::SharedLibraryError {
                path: path_string,
                reason: "file not found".to_string(),
            });
        }

        let loaded_lib =
            unsafe { libloading::Library::new(library_path) }.map_err(|load_error| {
                RuntimeError::SharedLibraryError {
                    path: path_string.clone(),
                    reason: load_error.to_string(),
                }
            })?;

        // Query optional batch registration hook: solix_register_natives
        unsafe {
            if let Ok(init_hook) =
                loaded_lib.get::<SolixRegisterNativesHook>(b"solix_register_natives\0")
            {
                let mut collected_entries = Vec::<(String, NativeCFunctionPtr)>::new();
                let collector_ptr =
                    &mut collected_entries as *mut Vec<(String, NativeCFunctionPtr)> as *mut c_void;
                init_hook(collector_ptr, collect_registration_callback);
                self.registered_functions.extend(collected_entries);
            }
        }

        self.loaded_libraries.push(loaded_lib);
        Ok(())
    }

    /// Probes loaded libraries for a direct symbol matching `symbol_name` or its mangled alternatives.
    #[must_use]
    pub fn resolve_direct_symbol(&self, symbol_name: &str) -> Option<NativeCFunctionPtr> {
        let clean_symbol = symbol_name.split('(').next().unwrap_or(symbol_name);
        let mangled_underscore = clean_symbol.replace("::", "_");
        let unqualified_symbol = clean_symbol.rsplit("::").next().unwrap_or(clean_symbol);

        let candidate_names = [clean_symbol, &mangled_underscore, unqualified_symbol];

        for library in &self.loaded_libraries {
            for candidate in &candidate_names {
                let null_terminated = format!("{candidate}\0");
                unsafe {
                    if let Ok(symbol_handle) =
                        library.get::<NativeCFunctionPtr>(null_terminated.as_bytes())
                    {
                        return Some(*symbol_handle);
                    }
                }
            }
        }

        None
    }
}

/// Instance helper export allowing native C callbacks to read a 32-bit field from a Solix object.
///
/// # Safety
/// The caller must ensure `vm_context` is either null or points to a valid `VirtualMachine`
/// or `SolixVmContext` instance, and that `object_address` references valid heap memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_get_field_i32(
    vm_context: *mut c_void,
    object_address: u32,
    offset: u32,
) -> i32 {
    if vm_context.is_null() {
        return 0;
    }
    let actual_vm_ptr = unsafe {
        let context_ptr = vm_context as *mut SolixVmContext;
        if !context_ptr.is_null() && (*context_ptr).magic == SOLIX_VM_CONTEXT_MAGIC {
            (*context_ptr).vm_ptr
        } else {
            vm_context
        }
    };
    let vm = unsafe { &mut *(actual_vm_ptr as *mut VirtualMachine) };
    let actual_offset = if offset < 8 {
        let resolved_offset = vm
            .heap
            .read_u32(object_address)
            .ok()
            .and_then(|vtable_id| vm.context.get_class_by_vtable_id(vtable_id))
            .and_then(|class_desc| {
                let mut sorted_fields = class_desc.field_types.clone();
                sorted_fields.sort_by_key(|&(field_offset, _)| field_offset);
                sorted_fields
                    .get(offset as usize)
                    .map(|&(field_offset, _)| field_offset as u32)
            });
        resolved_offset.unwrap_or(8 + offset * 4)
    } else {
        offset
    };
    vm.heap
        .read_u32(object_address + actual_offset)
        .map(|value| value as i32)
        .unwrap_or(0)
}

/// Instance helper export allowing native C callbacks to write a 32-bit field to a Solix object.
///
/// # Safety
/// The caller must ensure `vm_context` is either null or points to a valid `VirtualMachine`
/// or `SolixVmContext` instance, and that `object_address` references valid heap memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_set_field_i32(
    vm_context: *mut c_void,
    object_address: u32,
    offset: u32,
    value: i32,
) {
    if vm_context.is_null() {
        return;
    }
    let actual_vm_ptr = unsafe {
        let context_ptr = vm_context as *mut SolixVmContext;
        if !context_ptr.is_null() && (*context_ptr).magic == SOLIX_VM_CONTEXT_MAGIC {
            (*context_ptr).vm_ptr
        } else {
            vm_context
        }
    };
    let vm = unsafe { &mut *(actual_vm_ptr as *mut VirtualMachine) };
    let actual_offset = if offset < 8 {
        let resolved_offset = vm
            .heap
            .read_u32(object_address)
            .ok()
            .and_then(|vtable_id| vm.context.get_class_by_vtable_id(vtable_id))
            .and_then(|class_desc| {
                let mut sorted_fields = class_desc.field_types.clone();
                sorted_fields.sort_by_key(|&(field_offset, _)| field_offset);
                sorted_fields
                    .get(offset as usize)
                    .map(|&(field_offset, _)| field_offset as u32)
            });
        resolved_offset.unwrap_or(8 + offset * 4)
    } else {
        offset
    };
    let _ = vm
        .heap
        .write_u32(object_address + actual_offset, value as u32);
}
