//! Foreign Function Interface (FFI) dynamic shared library bridge and C-ABI definitions.

use crate::exceptions::RuntimeError;
use crate::memory::BlockTag;
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

/// Context structure passed to native C callbacks providing safe field and heap accessor function pointers.
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

    // --- New Host Callbacks Appended for Standard Library & Native Crates ---
    /// Reads a guest `string`, `char[]`, or `int8[]` at `addr` into `out_buf` (up to `cap` bytes).
    /// Returns the total byte/element length of the string or array.
    pub read_bytes:
        unsafe extern "C" fn(vm_ptr: *mut c_void, addr: u32, out_buf: *mut u8, cap: u32) -> u32,

    /// Allocates a new `BlockTag::String` on the guest heap from `(ptr, len)` and returns its `u32` address.
    pub alloc_string: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,

    /// Allocates a new `char[]` (`BlockTag::ArrayRef`, elem_size=8, type_tag=12) from `(ptr, len)`
    /// so each ASCII/UTF-8 byte is stored as a `char` element, returning its `u32` address.
    pub alloc_chars: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,

    /// Allocates a new `int8[]` (`BlockTag::ArrayRef`, elem_size=8, type_tag=1) from `(ptr, len)`.
    pub alloc_byte_array:
        unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,

    /// Allocates a new `ArrayRef` of `len` elements with `type_tag` (e.g. 4 = int64, 10 = float64, 13 = string).
    pub alloc_array: unsafe extern "C" fn(vm_ptr: *mut c_void, len: u32, type_tag: u8) -> u32,

    /// Returns the element length of an `ArrayRef` or byte length of a `String`.
    pub array_len: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32) -> u32,

    /// Gets a 64-bit element (int64, float64 bits, or ref u32) at `index` in a guest `ArrayRef`.
    pub array_get_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32) -> u64,

    /// Sets a 64-bit element (int64, float64 bits, or ref u32) at `index` in a guest `ArrayRef`.
    pub array_set_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32, raw_val: u64),

    /// Allocates an object on the guest heap with `class_vtable_id` and `field_count` 8-byte slots.
    pub alloc_object:
        unsafe extern "C" fn(vm_ptr: *mut c_void, class_vtable_id: u32, field_count: u32) -> u32,

    /// Reads a 64-bit field from an object at a field index or direct byte offset.
    pub read_field_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, offset: u32) -> u64,

    /// Writes a 64-bit field to an object at a field index or direct byte offset.
    pub write_field_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, offset: u32, raw_val: u64),

    /// Looks up a registered class VTable ID by qualified or simple name. Returns 0 if not found.
    pub get_class_vtable_id:
        unsafe extern "C" fn(vm_ptr: *mut c_void, name_ptr: *const c_char) -> u32,

    /// Evaluates a runtime execution or environment metric by identifier (0..=55).
    pub get_vm_metric: unsafe extern "C" fn(vm_ptr: *mut c_void, metric_id: u32) -> u64,

    /// Triggers synchronous Bacon & Rajan cycle collection across the ARC heap.
    pub collect_cycles: unsafe extern "C" fn(vm_ptr: *mut c_void) -> u64,

    /// Sets the cycle collector candidate threshold.
    pub set_cycle_threshold: unsafe extern "C" fn(vm_ptr: *mut c_void, threshold: u32),
    /// Reads a 32-bit signed integer field from an object handle by explicit 0-based field index.
    pub get_field_i32_by_index: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    /// Writes a 32-bit signed integer field to an object handle by explicit 0-based field index.
    pub set_field_i32_by_index: unsafe extern "C" fn(*mut c_void, u32, u32, i32),
    /// Reads a 32-bit signed integer field from an object handle by explicit byte offset.
    pub get_field_i32_by_offset: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    /// Writes a 32-bit signed integer field to an object handle by explicit byte offset.
    pub set_field_i32_by_offset: unsafe extern "C" fn(*mut c_void, u32, u32, i32),
    /// Reads a 64-bit field from an object by explicit 0-based field index.
    pub read_field_u64_by_index:
        unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, field_index: u32) -> u64,
    /// Writes a 64-bit field to an object by explicit 0-based field index.
    pub write_field_u64_by_index:
        unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, field_index: u32, raw_val: u64),
    /// Reads a 64-bit field from an object by explicit byte offset.
    pub read_field_u64_by_offset:
        unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, byte_offset: u32) -> u64,
    /// Writes a 64-bit field to an object by explicit byte offset.
    pub write_field_u64_by_offset:
        unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, byte_offset: u32, raw_val: u64),
}

/// Stable C-ABI native function pointer signature.
pub type NativeCFunctionPtr = unsafe extern "C-unwind" fn(
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
        read_bytes: solix_vm_read_bytes,
        alloc_string: solix_vm_alloc_string,
        alloc_chars: solix_vm_alloc_chars,
        alloc_byte_array: solix_vm_alloc_byte_array,
        alloc_array: solix_vm_alloc_array,
        array_len: solix_vm_array_len,
        array_get_u64: solix_vm_array_get_u64,
        array_set_u64: solix_vm_array_set_u64,
        alloc_object: solix_vm_alloc_object,
        read_field_u64: solix_vm_read_field_u64,
        write_field_u64: solix_vm_write_field_u64,
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
        get_field_i32_by_index: solix_vm_get_field_i32_by_index,
        set_field_i32_by_index: solix_vm_set_field_i32_by_index,
        get_field_i32_by_offset: solix_vm_get_field_i32_by_offset,
        set_field_i32_by_offset: solix_vm_set_field_i32_by_offset,
        read_field_u64_by_index: solix_vm_read_field_u64_by_index,
        write_field_u64_by_index: solix_vm_write_field_u64_by_index,
        read_field_u64_by_offset: solix_vm_read_field_u64_by_offset,
        write_field_u64_by_offset: solix_vm_write_field_u64_by_offset,
    };
    let context_wrapper_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let execution_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        function_ptr(
            context_wrapper_ptr,
            converted_arguments.as_ptr(),
            converted_arguments.len() as u32,
            &mut out_error,
        )
    }));

    let result = match execution_result {
        Ok(call_result) => call_result,
        Err(_) => {
            return Err(RuntimeError::NativeFunctionError {
                name: "native_c_call".to_string(),
                reason: "native callback panicked across C-ABI boundary".to_string(),
            });
        }
    };

    if out_error.tag != 0 {
        let reason_text = if out_error.tag == 5 && out_error.payload != 0 {
            let error_address = out_error.payload as u32;
            vm.heap
                .read_string(error_address)
                .unwrap_or_else(|_| "error signaled by native C callback".to_string())
        } else {
            "error signaled by native C callback".to_string()
        };
        return Err(RuntimeError::NativeFunctionError {
            name: "native_c_call".to_string(),
            reason: reason_text,
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
        let candidates = if clean_symbol.contains([':', '.']) {
            vec![clean_symbol, &mangled]
        } else {
            vec![clean_symbol, &mangled, unqualified]
        };
        for candidate in candidates {
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

    /// Discovers and loads native shared libraries from a target directory (such as `bin/<target_triple>/`).
    ///
    /// Only libraries whose basename matches `native_*` or `libnative_*` (or are explicitly declared in
    /// `declared_libraries`) are loaded into the symbol table. Transitive or support libraries (e.g.
    /// `libglfw.so`, `glfw3.dll`) are ignored.
    pub fn load_directory_filtered(
        &mut self,
        target_directory: &Path,
        declared_libraries: &[String],
    ) -> Result<Vec<std::path::PathBuf>, RuntimeError> {
        let mut loaded_paths = Vec::new();
        if !target_directory.is_dir() {
            return Ok(loaded_paths);
        }
        let directory_entries = std::fs::read_dir(target_directory).map_err(|io_error| {
            RuntimeError::SharedLibraryError {
                path: target_directory.display().to_string(),
                reason: format!("failed to read directory: {io_error}"),
            }
        })?;

        for entry_result in directory_entries.flatten() {
            let entry_path = entry_result.path();
            if !entry_path.is_file() {
                continue;
            }
            let file_name = entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let extension = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            let is_shared_library = extension.eq_ignore_ascii_case("so")
                || extension.eq_ignore_ascii_case("dll")
                || extension.eq_ignore_ascii_case("dylib");
            if !is_shared_library {
                continue;
            }

            let is_declared = declared_libraries.iter().any(|declared_lib| {
                solix_common::project::matches_declared_dynamic_library(file_name, declared_lib)
            });
            let is_native_module =
                solix_common::project::is_solix_native_module_name(file_name) || is_declared;

            if !is_native_module {
                continue;
            }

            self.load_library(&entry_path)?;
            loaded_paths.push(entry_path);
        }
        Ok(loaded_paths)
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
fn lookup_class_descriptor(
    vm_instance: &VirtualMachine,
    object_address: u32,
) -> Option<&solix_common::CompiledClass> {
    vm_instance
        .heap
        .read_u32(object_address)
        .ok()
        .and_then(|vtable_identifier| {
            vm_instance
                .context
                .get_class_by_vtable_id(vtable_identifier)
        })
}

fn resolve_field_by_index(
    vm_instance: &VirtualMachine,
    object_address: u32,
    field_index: u32,
    default_stride: u32,
) -> Option<(u32, u8)> {
    if let Some(class_descriptor) = lookup_class_descriptor(vm_instance, object_address)
        && !class_descriptor.field_types.is_empty()
    {
        let mut sorted_field_list = class_descriptor.field_types.clone();
        sorted_field_list.sort_by_key(|&(field_offset, _)| field_offset);
        return sorted_field_list
            .get(field_index as usize)
            .map(|&(field_offset, resolved_tag)| (field_offset as u32, resolved_tag));
    }
    field_index
        .checked_mul(default_stride)
        .and_then(|slot_offset| 8u32.checked_add(slot_offset))
        .map(|computed_offset| (computed_offset, 0))
}

fn resolve_field_by_offset(
    vm_instance: &VirtualMachine,
    object_address: u32,
    byte_offset: u32,
) -> (u32, u8) {
    let field_tag = lookup_class_descriptor(vm_instance, object_address)
        .and_then(|class_descriptor| class_descriptor.get_field_type(byte_offset as usize))
        .unwrap_or(0);
    (byte_offset, field_tag)
}

fn read_field_i32_resolved(
    vm_instance: &VirtualMachine,
    object_address: u32,
    actual_offset: u32,
    field_tag: u8,
) -> i32 {
    let Some(field_addr) = object_address.checked_add(actual_offset) else {
        return 0;
    };
    match field_tag {
        1 => vm_instance.heap.read_u8(field_addr).unwrap_or(0) as i8 as i32,
        2 | 11 => vm_instance.heap.read_u8(field_addr).unwrap_or(0) as i32,
        3 => vm_instance.heap.read_u16(field_addr).unwrap_or(0) as i16 as i32,
        4 => vm_instance.heap.read_u16(field_addr).unwrap_or(0) as i32,
        _ => vm_instance
            .heap
            .read_u32(field_addr)
            .map(|value| value as i32)
            .unwrap_or(0),
    }
}

fn write_field_i32_resolved(
    vm_instance: &mut VirtualMachine,
    object_address: u32,
    actual_offset: u32,
    field_tag: u8,
    value: i32,
) {
    let Some(field_addr) = object_address.checked_add(actual_offset) else {
        return;
    };
    match field_tag {
        1 | 2 => {
            let _ = vm_instance.heap.write_u8(field_addr, value as u8);
        }
        3 | 4 => {
            let _ = vm_instance.heap.write_u16(field_addr, value as u16);
        }
        11 => {
            let _ = vm_instance.heap.write_u8(field_addr, u8::from(value != 0));
        }
        _ => {
            let _ = vm_instance.heap.write_u32(field_addr, value as u32);
        }
    }
}

/// Legacy helper reading a 32-bit integer field using field index `< 8` or byte offset `>= 8`.
///
/// Prefer `solix_vm_get_field_i32_by_index` or `solix_vm_get_field_i32_by_offset` to avoid
/// collisions on classes with small offsets or >= 8 fields.
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
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return 0;
    };
    if object_address == 0 {
        return 0;
    }
    let Some((actual_offset, field_tag)) = (if offset < 8 {
        resolve_field_by_index(vm_instance, object_address, offset, 4)
    } else {
        Some(resolve_field_by_offset(vm_instance, object_address, offset))
    }) else {
        return 0;
    };
    read_field_i32_resolved(vm_instance, object_address, actual_offset, field_tag)
}

/// Legacy helper writing a 32-bit integer field using field index `< 8` or byte offset `>= 8`.
///
/// Prefer `solix_vm_set_field_i32_by_index` or `solix_vm_set_field_i32_by_offset` to avoid
/// collisions on classes with small offsets or >= 8 fields.
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
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return;
    };
    if object_address == 0 {
        return;
    }
    let Some((actual_offset, field_tag)) = (if offset < 8 {
        resolve_field_by_index(vm_instance, object_address, offset, 4)
    } else {
        Some(resolve_field_by_offset(vm_instance, object_address, offset))
    }) else {
        return;
    };
    write_field_i32_resolved(vm_instance, object_address, actual_offset, field_tag, value);
}

/// Reads a 32-bit signed integer field from an object instance using its explicit 0-based `field_index`.
///
/// # Safety
/// The caller must ensure `vm_context` is null or points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_get_field_i32_by_index(
    vm_context: *mut c_void,
    object_address: u32,
    field_index: u32,
) -> i32 {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return 0;
    };
    if object_address == 0 {
        return 0;
    }
    let Some((actual_offset, field_tag)) =
        resolve_field_by_index(vm_instance, object_address, field_index, 4)
    else {
        return 0;
    };
    read_field_i32_resolved(vm_instance, object_address, actual_offset, field_tag)
}

/// Writes a 32-bit signed integer field to an object instance using its explicit 0-based `field_index`.
///
/// # Safety
/// The caller must ensure `vm_context` is null or points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_set_field_i32_by_index(
    vm_context: *mut c_void,
    object_address: u32,
    field_index: u32,
    value: i32,
) {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return;
    };
    if object_address == 0 {
        return;
    }
    let Some((actual_offset, field_tag)) =
        resolve_field_by_index(vm_instance, object_address, field_index, 4)
    else {
        return;
    };
    write_field_i32_resolved(vm_instance, object_address, actual_offset, field_tag, value);
}

/// Reads a 32-bit signed integer field from an object instance using its explicit `byte_offset`.
///
/// # Safety
/// The caller must ensure `vm_context` is null or points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_get_field_i32_by_offset(
    vm_context: *mut c_void,
    object_address: u32,
    byte_offset: u32,
) -> i32 {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return 0;
    };
    if object_address == 0 {
        return 0;
    }
    let (actual_offset, field_tag) =
        resolve_field_by_offset(vm_instance, object_address, byte_offset);
    read_field_i32_resolved(vm_instance, object_address, actual_offset, field_tag)
}

/// Writes a 32-bit signed integer field to an object instance using its explicit `byte_offset`.
///
/// # Safety
/// The caller must ensure `vm_context` is null or points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_set_field_i32_by_offset(
    vm_context: *mut c_void,
    object_address: u32,
    byte_offset: u32,
    value: i32,
) {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return;
    };
    if object_address == 0 {
        return;
    }
    let (actual_offset, field_tag) =
        resolve_field_by_offset(vm_instance, object_address, byte_offset);
    write_field_i32_resolved(vm_instance, object_address, actual_offset, field_tag, value);
}

/// Safely resolves a `*mut c_void` context pointer into an optional `VirtualMachine` reference.
unsafe fn resolve_vm<'vm>(vm_context: *mut c_void) -> Option<&'vm mut VirtualMachine> {
    if vm_context.is_null() {
        return None;
    }
    let actual_vm_ptr = {
        let context_pointer = vm_context as *mut SolixVmContext;
        if !context_pointer.is_null()
            && context_pointer.is_aligned()
            && unsafe { (*context_pointer).magic } == SOLIX_VM_CONTEXT_MAGIC
        {
            unsafe { (*context_pointer).vm_ptr }
        } else {
            vm_context
        }
    };
    let vm_pointer = actual_vm_ptr as *mut VirtualMachine;
    if actual_vm_ptr.is_null() || !vm_pointer.is_aligned() {
        None
    } else {
        Some(unsafe { &mut *vm_pointer })
    }
}

/// Reads bytes from a guest `string`, `char[]`, or `int8[]` into `out_buffer` (up to `capacity` bytes).
/// For `char[]` arrays holding Unicode scalar values (`type_tag == 12`, `element_size >= 4`),
/// transcodes elements to UTF-8 and returns the full UTF-8 byte length.
///
/// # Safety
/// The caller must ensure `out_buffer` is valid for writing `capacity` bytes if non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_read_bytes(
    vm_context: *mut c_void,
    source_address: u32,
    out_buffer: *mut u8,
    capacity: u32,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    if source_address == 0 {
        return 0;
    }
    let header_result = vm_instance.heap.read_header(source_address);
    let header = match header_result {
        Ok(valid_header) => valid_header,
        Err(_) => return 0,
    };

    if header.tag == BlockTag::String {
        let total_length = header.size;
        if !out_buffer.is_null() && capacity > 0 {
            let copy_length = (capacity as usize).min(total_length as usize);
            let start_offset = source_address as usize;
            if let Some(end_offset) = start_offset.checked_add(copy_length)
                && end_offset <= vm_instance.heap.buffer.len()
            {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        vm_instance.heap.buffer[start_offset..end_offset].as_ptr(),
                        out_buffer,
                        copy_length,
                    );
                }
            }
        }
        total_length
    } else if header.tag == BlockTag::ArrayRef {
        let total_length = vm_instance.heap.read_u32(source_address).unwrap_or(0);
        let array_offset = source_address as usize;
        let element_size = array_offset
            .checked_add(4)
            .and_then(|meta_offset| vm_instance.heap.buffer.get(meta_offset).copied())
            .unwrap_or(8)
            .max(1) as usize;
        let type_tag = array_offset
            .checked_add(7)
            .and_then(|tag_offset| vm_instance.heap.buffer.get(tag_offset).copied())
            .unwrap_or(0);
        let Some(data_start) = array_offset.checked_add(8) else {
            return 0;
        };

        if type_tag == 12 && element_size >= 4 {
            let mut total_utf8_bytes = 0usize;
            let mut written_bytes = 0usize;
            let target_cap = if out_buffer.is_null() {
                0
            } else {
                capacity as usize
            };
            let heap_len = vm_instance.heap.buffer.len();
            let mut utf8_scratch = [0u8; 4];

            for element_index in 0..total_length as usize {
                let Some(element_offset) = element_index
                    .checked_mul(element_size)
                    .and_then(|relative_offset| data_start.checked_add(relative_offset))
                else {
                    break;
                };
                let Some(element_end) = element_offset.checked_add(element_size) else {
                    break;
                };
                if element_end > heap_len {
                    break;
                }
                let raw_word = if element_size >= 8 {
                    let bytes: [u8; 8] = vm_instance.heap.buffer
                        [element_offset..element_offset + 8]
                        .try_into()
                        .unwrap_or([0; 8]);
                    u64::from_le_bytes(bytes)
                } else {
                    let bytes: [u8; 4] = vm_instance.heap.buffer
                        [element_offset..element_offset + 4]
                        .try_into()
                        .unwrap_or([0; 4]);
                    u64::from(u32::from_le_bytes(bytes))
                };
                let code_point = raw_word as u32;
                if code_point < 0x80 {
                    if written_bytes < target_cap {
                        unsafe {
                            *out_buffer.add(written_bytes) = code_point as u8;
                        }
                        written_bytes += 1;
                    }
                    total_utf8_bytes = total_utf8_bytes.saturating_add(1);
                } else {
                    let unicode_char =
                        char::from_u32(code_point).unwrap_or(char::REPLACEMENT_CHARACTER);
                    let encoded_bytes = unicode_char.encode_utf8(&mut utf8_scratch).as_bytes();
                    let remaining_cap = target_cap.saturating_sub(written_bytes);
                    let copy_bytes = remaining_cap.min(encoded_bytes.len());
                    if copy_bytes > 0 {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                encoded_bytes.as_ptr(),
                                out_buffer.add(written_bytes),
                                copy_bytes,
                            );
                        }
                        written_bytes += copy_bytes;
                    }
                    total_utf8_bytes = total_utf8_bytes.saturating_add(encoded_bytes.len());
                }
            }
            return u32::try_from(total_utf8_bytes).unwrap_or(u32::MAX);
        }

        if !out_buffer.is_null() && capacity > 0 {
            let copy_count = (capacity as usize).min(total_length as usize);
            let heap_len = vm_instance.heap.buffer.len();
            for element_index in 0..copy_count {
                let Some(element_offset) = element_index
                    .checked_mul(element_size)
                    .and_then(|relative_offset| data_start.checked_add(relative_offset))
                else {
                    break;
                };
                if element_offset < heap_len {
                    let byte_value = vm_instance.heap.buffer[element_offset];
                    unsafe {
                        *out_buffer.add(element_index) = byte_value;
                    }
                }
            }
        }
        total_length
    } else {
        0
    }
}

/// Allocates a new `BlockTag::String` on the guest heap from bytes `(source_pointer, byte_count)` and returns its `u32` address.
///
/// # Safety
/// The caller must ensure `source_pointer` points to at least `byte_count` readable bytes if `byte_count > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_alloc_string(
    vm_context: *mut c_void,
    source_pointer: *const u8,
    byte_count: u32,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    let string_address = match vm_instance
        .heap
        .allocate(byte_count as usize, BlockTag::String)
    {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    if !source_pointer.is_null() && byte_count > 0 {
        let source_slice =
            unsafe { std::slice::from_raw_parts(source_pointer, byte_count as usize) };
        let _ = vm_instance.heap.write_bytes(string_address, source_slice);
    }
    string_address
}

/// Allocates a new `char[]` (`BlockTag::ArrayRef`, elem_size=8, type_tag=12) from bytes `(source_pointer, char_count)`
/// so each byte/character is stored in an 8-byte array slot, returning its `u32` address.
///
/// # Safety
/// The caller must ensure `source_pointer` points to at least `char_count` readable bytes if `char_count > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_alloc_chars(
    vm_context: *mut c_void,
    source_pointer: *const u8,
    char_count: u32,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };

    if !source_pointer.is_null() && char_count > 0 {
        let source_slice =
            unsafe { std::slice::from_raw_parts(source_pointer, char_count as usize) };
        if source_slice.is_ascii() {
            let element_count = source_slice.len();
            let payload_size = match element_count
                .checked_mul(8)
                .and_then(|computed_bytes| computed_bytes.checked_add(8))
            {
                Some(calculated_size) if calculated_size <= u32::MAX as usize => calculated_size,
                _ => return 0,
            };
            let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
                Ok(allocated_address) => allocated_address,
                Err(_) => return 0,
            };
            let _ = vm_instance
                .heap
                .write_u32(array_address, element_count as u32);
            if let Some(meta_addr) = array_address.checked_add(4) {
                let _ = vm_instance.heap.write_u8(meta_addr, 8);
            }
            if let Some(tag_addr) = array_address.checked_add(7) {
                let _ = vm_instance.heap.write_u8(tag_addr, 12); // Type tag 12 = Char
            }
            for (character_index, &character_byte) in source_slice.iter().enumerate() {
                let Some(slot_address) = (character_index as u32)
                    .checked_mul(8)
                    .and_then(|relative_offset| relative_offset.checked_add(8))
                    .and_then(|offset_bytes| array_address.checked_add(offset_bytes))
                else {
                    break;
                };
                let _ = vm_instance
                    .heap
                    .write_u64(slot_address, character_byte as u64);
            }
            return array_address;
        }

        let decoded_text = String::from_utf8_lossy(source_slice);
        let code_points: Vec<u32> = decoded_text
            .chars()
            .map(|unicode_char| unicode_char as u32)
            .collect();
        let element_count = code_points.len();
        let payload_size = match element_count
            .checked_mul(8)
            .and_then(|computed_bytes| computed_bytes.checked_add(8))
        {
            Some(calculated_size) if calculated_size <= u32::MAX as usize => calculated_size,
            _ => return 0,
        };
        let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
            Ok(allocated_address) => allocated_address,
            Err(_) => return 0,
        };
        let _ = vm_instance
            .heap
            .write_u32(array_address, element_count as u32);
        if let Some(meta_addr) = array_address.checked_add(4) {
            let _ = vm_instance.heap.write_u8(meta_addr, 8);
        }
        if let Some(tag_addr) = array_address.checked_add(7) {
            let _ = vm_instance.heap.write_u8(tag_addr, 12); // Type tag 12 = Char
        }
        for (character_index, code_point) in code_points.into_iter().enumerate() {
            let Some(slot_address) = (character_index as u32)
                .checked_mul(8)
                .and_then(|relative_offset| relative_offset.checked_add(8))
                .and_then(|offset_bytes| array_address.checked_add(offset_bytes))
            else {
                break;
            };
            let _ = vm_instance.heap.write_u64(slot_address, code_point as u64);
        }
        return array_address;
    }

    let payload_size = match (char_count as usize)
        .checked_mul(8)
        .and_then(|computed_bytes| computed_bytes.checked_add(8))
    {
        Some(calculated_size) if calculated_size <= u32::MAX as usize => calculated_size,
        _ => return 0,
    };
    let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(array_address, char_count);
    if let Some(meta_addr) = array_address.checked_add(4) {
        let _ = vm_instance.heap.write_u8(meta_addr, 8);
    }
    if let Some(tag_addr) = array_address.checked_add(7) {
        let _ = vm_instance.heap.write_u8(tag_addr, 12); // Type tag 12 = Char
    }
    array_address
}

/// Allocates a new `int8[]` (`BlockTag::ArrayRef`, elem_size=8, type_tag=1) from bytes `(source_pointer, byte_count)`
/// so each byte is stored in an 8-byte array slot, returning its `u32` address.
///
/// # Safety
/// The caller must ensure `source_pointer` points to at least `byte_count` readable bytes if `byte_count > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_alloc_byte_array(
    vm_context: *mut c_void,
    source_pointer: *const u8,
    byte_count: u32,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    let payload_size = match (byte_count as usize)
        .checked_mul(8)
        .and_then(|computed_bytes| computed_bytes.checked_add(8))
    {
        Some(calculated_size) if calculated_size <= u32::MAX as usize => calculated_size,
        _ => return 0,
    };
    let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(array_address, byte_count);
    if let Some(meta_addr) = array_address.checked_add(4) {
        let _ = vm_instance.heap.write_u8(meta_addr, 8);
    }
    if let Some(tag_addr) = array_address.checked_add(7) {
        let _ = vm_instance.heap.write_u8(tag_addr, 1); // Type tag 1 = Int8
    }

    if !source_pointer.is_null() && byte_count > 0 {
        let source_slice =
            unsafe { std::slice::from_raw_parts(source_pointer, byte_count as usize) };
        for (byte_index, &raw_byte) in source_slice.iter().enumerate() {
            let Some(slot_address) = (byte_index as u32)
                .checked_mul(8)
                .and_then(|relative_offset| relative_offset.checked_add(8))
                .and_then(|offset_bytes| array_address.checked_add(offset_bytes))
            else {
                break;
            };
            let _ = vm_instance
                .heap
                .write_u64(slot_address, (raw_byte as i8 as i64) as u64);
        }
    }
    array_address
}

/// Allocates a new empty `ArrayRef` of `element_count` elements with `type_tag`.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_alloc_array(
    vm_context: *mut c_void,
    element_count: u32,
    type_tag: u8,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    let payload_size = match (element_count as usize)
        .checked_mul(8)
        .and_then(|computed_bytes| computed_bytes.checked_add(8))
    {
        Some(calculated_size) if calculated_size <= u32::MAX as usize => calculated_size,
        _ => return 0,
    };
    let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(array_address, element_count);
    if let Some(meta_addr) = array_address.checked_add(4) {
        let _ = vm_instance.heap.write_u8(meta_addr, 8);
    }
    if let Some(tag_addr) = array_address.checked_add(7) {
        let _ = vm_instance.heap.write_u8(tag_addr, type_tag);
    }
    array_address
}

/// Returns the element length of an `ArrayRef` or byte length of a `String`.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_array_len(vm_context: *mut c_void, array_address: u32) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    if array_address == 0 {
        return 0;
    }
    let header_result = vm_instance.heap.read_header(array_address);
    match header_result {
        Ok(header) if header.tag == BlockTag::String => header.size,
        _ => vm_instance.heap.read_u32(array_address).unwrap_or(0),
    }
}

/// Gets a 64-bit element at `element_index` from an array on the guest heap.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_array_get_u64(
    vm_context: *mut c_void,
    array_address: u32,
    element_index: u32,
) -> u64 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    if array_address == 0 {
        return 0;
    }
    let total_length = vm_instance.heap.read_u32(array_address).unwrap_or(0);
    if element_index >= total_length {
        return 0;
    }
    let element_size = array_address
        .checked_add(4)
        .and_then(|meta_addr| vm_instance.heap.read_u8(meta_addr).ok())
        .unwrap_or(8)
        .max(1) as u32;
    let type_tag = array_address
        .checked_add(7)
        .and_then(|tag_addr| vm_instance.heap.read_u8(tag_addr).ok())
        .unwrap_or(0);
    let Some(slot_address) = element_index
        .checked_mul(element_size)
        .and_then(|relative_offset| array_address.checked_add(8)?.checked_add(relative_offset))
    else {
        return 0;
    };
    if type_tag == 9 {
        let raw_u32 = vm_instance.heap.read_u32(slot_address).unwrap_or(0);
        let float_val = f32::from_bits(raw_u32);
        (float_val as f64).to_bits()
    } else {
        match element_size {
            1 => vm_instance.heap.read_u8(slot_address).unwrap_or(0) as u64,
            2 => vm_instance.heap.read_u16(slot_address).unwrap_or(0) as u64,
            4 => vm_instance.heap.read_u32(slot_address).unwrap_or(0) as u64,
            _ => vm_instance.heap.read_u64(slot_address).unwrap_or(0),
        }
    }
}

/// Sets a 64-bit element at `element_index` into an array on the guest heap.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_array_set_u64(
    vm_context: *mut c_void,
    array_address: u32,
    element_index: u32,
    raw_value: u64,
) {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return,
    };
    if array_address == 0 {
        return;
    }
    let total_length = vm_instance.heap.read_u32(array_address).unwrap_or(0);
    if element_index >= total_length {
        return;
    }
    let element_size = array_address
        .checked_add(4)
        .and_then(|meta_addr| vm_instance.heap.read_u8(meta_addr).ok())
        .unwrap_or(8)
        .max(1) as u32;
    let type_tag = array_address
        .checked_add(7)
        .and_then(|tag_addr| vm_instance.heap.read_u8(tag_addr).ok())
        .unwrap_or(0);
    let Some(slot_address) = element_index
        .checked_mul(element_size)
        .and_then(|relative_offset| array_address.checked_add(8)?.checked_add(relative_offset))
    else {
        return;
    };
    if type_tag == 9 {
        let float_val = f64::from_bits(raw_value) as f32;
        let _ = vm_instance
            .heap
            .write_u32(slot_address, float_val.to_bits());
    } else {
        match element_size {
            1 => {
                let _ = vm_instance.heap.write_u8(slot_address, raw_value as u8);
            }
            2 => {
                let _ = vm_instance.heap.write_u16(slot_address, raw_value as u16);
            }
            4 => {
                let _ = vm_instance.heap.write_u32(slot_address, raw_value as u32);
            }
            _ => {
                let _ = vm_instance.heap.write_u64(slot_address, raw_value);
            }
        }
    }
}

/// Allocates an object on the guest heap with `class_vtable_id` and `field_count` 8-byte slots.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_alloc_object(
    vm_context: *mut c_void,
    class_vtable_id: u32,
    field_count: u32,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    let min_size = match (field_count as usize)
        .checked_mul(8)
        .and_then(|computed_bytes| computed_bytes.checked_add(8))
    {
        Some(calculated_size) if calculated_size <= u32::MAX as usize => calculated_size,
        _ => return 0,
    };
    let class_descriptor = vm_instance.context.get_class_by_vtable_id(class_vtable_id);
    let payload_size = class_descriptor.map_or(min_size, |target_class| {
        target_class.instance_size.max(min_size)
    });
    let object_address = match vm_instance.heap.allocate(payload_size, BlockTag::Object) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(object_address, class_vtable_id);
    object_address
}

/// Reads a 64-bit field from an object at a field index or direct byte offset.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
fn read_field_u64_impl(
    vm_instance: &VirtualMachine,
    object_address: u32,
    actual_offset: u32,
    field_tag: u8,
) -> u64 {
    let Some(field_addr) = object_address.checked_add(actual_offset) else {
        return 0;
    };
    match field_tag {
        1 => vm_instance.heap.read_u8(field_addr).unwrap_or(0) as i8 as i64 as u64,
        2 => vm_instance.heap.read_u8(field_addr).unwrap_or(0) as u64,
        3 => vm_instance.heap.read_u16(field_addr).unwrap_or(0) as i16 as i64 as u64,
        4 => vm_instance.heap.read_u16(field_addr).unwrap_or(0) as u64,
        5 => vm_instance.heap.read_u32(field_addr).unwrap_or(0) as i32 as i64 as u64,
        6 | 9 => vm_instance.heap.read_u32(field_addr).unwrap_or(0) as u64,
        7 | 8 | 10 => vm_instance.heap.read_u64(field_addr).unwrap_or(0),
        11 => u64::from(vm_instance.heap.read_u8(field_addr).unwrap_or(0) != 0),
        12 => {
            let raw_word = vm_instance.heap.read_u64(field_addr).unwrap_or(0);
            match Value::from_raw_u64(raw_word) {
                Value::Ref(addr) => addr as u64,
                Value::Null => 0,
                _ => raw_word & 0xFFFF_FFFF,
            }
        }
        _ => vm_instance.heap.read_u64(field_addr).unwrap_or(0),
    }
}

fn write_field_u64_impl(
    vm_instance: &mut VirtualMachine,
    object_address: u32,
    actual_offset: u32,
    field_tag: u8,
    raw_value: u64,
) {
    let Some(field_addr) = object_address.checked_add(actual_offset) else {
        return;
    };
    match field_tag {
        1 | 2 => {
            let _ = vm_instance.heap.write_u8(field_addr, raw_value as u8);
        }
        3 | 4 => {
            let _ = vm_instance.heap.write_u16(field_addr, raw_value as u16);
        }
        5 | 6 | 9 => {
            let _ = vm_instance.heap.write_u32(field_addr, raw_value as u32);
        }
        7 | 8 | 10 => {
            let _ = vm_instance.heap.write_u64(field_addr, raw_value);
        }
        11 => {
            let _ = vm_instance
                .heap
                .write_u8(field_addr, u8::from(raw_value != 0));
        }
        12 => {
            let tagged_word = if raw_value == 0 {
                Value::Null.to_raw_u64()
            } else if (raw_value & Value::TAG_MASK) == Value::TAG_REF {
                raw_value
            } else {
                Value::Ref(raw_value as u32).to_raw_u64()
            };
            let _ = vm_instance.heap.write_u64(field_addr, tagged_word);
        }
        _ => {
            let _ = vm_instance.heap.write_u64(field_addr, raw_value);
        }
    }
}

/// Legacy helper reading a 64-bit field from an object using field index `< 64` or byte offset.
///
/// Prefer `solix_vm_read_field_u64_by_index` or `solix_vm_read_field_u64_by_offset` to avoid
/// ambiguity on classes with small offsets (`< 64`) or >= 8 fields.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_read_field_u64(
    vm_context: *mut c_void,
    object_address: u32,
    offset: u32,
) -> u64 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    if object_address == 0 {
        return 0;
    }
    if let Some(class_descriptor) = lookup_class_descriptor(vm_instance, object_address)
        && !class_descriptor.field_types.is_empty()
    {
        let Some((actual_offset, field_tag)) =
            (if offset < 64 && (offset as usize) < class_descriptor.field_types.len() {
                resolve_field_by_index(vm_instance, object_address, offset, 8)
            } else {
                Some(resolve_field_by_offset(vm_instance, object_address, offset))
            })
        else {
            return 0;
        };
        read_field_u64_impl(vm_instance, object_address, actual_offset, field_tag)
    } else {
        let Some(actual_offset) = (if offset < 8 {
            offset.checked_mul(8).and_then(|rel| 8u32.checked_add(rel))
        } else {
            Some(offset)
        }) else {
            return 0;
        };
        read_field_u64_impl(vm_instance, object_address, actual_offset, 0)
    }
}

/// Legacy helper writing a 64-bit field to an object using field index `< 64` or byte offset.
///
/// Prefer `solix_vm_write_field_u64_by_index` or `solix_vm_write_field_u64_by_offset` to avoid
/// ambiguity on classes with small offsets (`< 64`) or >= 8 fields.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_write_field_u64(
    vm_context: *mut c_void,
    object_address: u32,
    offset: u32,
    raw_value: u64,
) {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return,
    };
    if object_address == 0 {
        return;
    }
    if let Some(class_descriptor) = lookup_class_descriptor(vm_instance, object_address)
        && !class_descriptor.field_types.is_empty()
    {
        let Some((actual_offset, field_tag)) =
            (if offset < 64 && (offset as usize) < class_descriptor.field_types.len() {
                resolve_field_by_index(vm_instance, object_address, offset, 8)
            } else {
                Some(resolve_field_by_offset(vm_instance, object_address, offset))
            })
        else {
            return;
        };
        write_field_u64_impl(
            vm_instance,
            object_address,
            actual_offset,
            field_tag,
            raw_value,
        );
    } else {
        let Some(actual_offset) = (if offset < 8 {
            offset.checked_mul(8).and_then(|rel| 8u32.checked_add(rel))
        } else {
            Some(offset)
        }) else {
            return;
        };
        write_field_u64_impl(vm_instance, object_address, actual_offset, 0, raw_value);
    }
}

/// Reads a 64-bit (or sub-64-bit normalized) field from an object using its explicit 0-based `field_index`.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_read_field_u64_by_index(
    vm_context: *mut c_void,
    object_address: u32,
    field_index: u32,
) -> u64 {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return 0;
    };
    if object_address == 0 {
        return 0;
    }
    let Some((actual_offset, field_tag)) =
        resolve_field_by_index(vm_instance, object_address, field_index, 8)
    else {
        return 0;
    };
    read_field_u64_impl(vm_instance, object_address, actual_offset, field_tag)
}

/// Writes a 64-bit (or sub-64-bit normalized) field to an object using its explicit 0-based `field_index`.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_write_field_u64_by_index(
    vm_context: *mut c_void,
    object_address: u32,
    field_index: u32,
    raw_value: u64,
) {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return;
    };
    if object_address == 0 {
        return;
    }
    let Some((actual_offset, field_tag)) =
        resolve_field_by_index(vm_instance, object_address, field_index, 8)
    else {
        return;
    };
    write_field_u64_impl(
        vm_instance,
        object_address,
        actual_offset,
        field_tag,
        raw_value,
    );
}

/// Reads a 64-bit (or sub-64-bit normalized) field from an object using its explicit `byte_offset`.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_read_field_u64_by_offset(
    vm_context: *mut c_void,
    object_address: u32,
    byte_offset: u32,
) -> u64 {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return 0;
    };
    if object_address == 0 {
        return 0;
    }
    let (actual_offset, field_tag) =
        resolve_field_by_offset(vm_instance, object_address, byte_offset);
    read_field_u64_impl(vm_instance, object_address, actual_offset, field_tag)
}

/// Writes a 64-bit (or sub-64-bit normalized) field to an object using its explicit `byte_offset`.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_write_field_u64_by_offset(
    vm_context: *mut c_void,
    object_address: u32,
    byte_offset: u32,
    raw_value: u64,
) {
    let Some(vm_instance) = (unsafe { resolve_vm(vm_context) }) else {
        return;
    };
    if object_address == 0 {
        return;
    }
    let (actual_offset, field_tag) =
        resolve_field_by_offset(vm_instance, object_address, byte_offset);
    write_field_u64_impl(
        vm_instance,
        object_address,
        actual_offset,
        field_tag,
        raw_value,
    );
}

/// Looks up a registered class VTable ID by qualified or simple name. Returns 0 if not found.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext` and `name_ptr` points to a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_get_class_vtable_id(
    vm_context: *mut c_void,
    name_ptr: *const c_char,
) -> u32 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    if name_ptr.is_null() {
        return 0;
    }
    let class_name = match unsafe { CStr::from_ptr(name_ptr) }.to_str() {
        Ok(valid_name) => valid_name,
        Err(_) => return 0,
    };
    for class_descriptor in &vm_instance.context.all_classes {
        if class_descriptor.symbol == class_name
            || class_descriptor
                .symbol
                .ends_with(&format!("::{class_name}"))
        {
            return class_descriptor.vtable_id;
        }
    }
    0
}

/// Queries a runtime metric by identifier.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_get_metric(vm_context: *mut c_void, metric_id: u32) -> u64 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    vm_instance.get_metric_by_id(metric_id)
}

/// Triggers synchronous Bacon & Rajan cycle collection.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_collect_cycles(vm_context: *mut c_void) -> u64 {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return 0,
    };
    vm_instance.collect_cycles().unwrap_or(0)
}

/// Sets the cycle collector candidate threshold.
///
/// # Safety
/// The caller must ensure `vm_context` points to a valid `VirtualMachine` or `SolixVmContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_vm_set_cycle_threshold(vm_context: *mut c_void, threshold: u32) {
    let vm_instance = match unsafe { resolve_vm(vm_context) } {
        Some(resolved_instance) => resolved_instance,
        None => return,
    };
    vm_instance.cycle_threshold = threshold.max(1) as usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RuntimeContext;
    use crate::options::RuntimeOptions;

    unsafe extern "C-unwind" fn panicking_native_function(
        _context: *mut c_void,
        _arguments: *const SolixCValue,
        _count: u32,
        _error_out: *mut SolixCValue,
    ) -> SolixCValue {
        panic!("simulated unhandled panic inside foreign native plugin");
    }

    #[test]
    fn test_invoke_c_native_catches_unwind_safely() {
        let mut virtual_machine = VirtualMachine::new(
            RuntimeContext::new(RuntimeOptions::new())
                .expect("Context initialization must succeed"),
        );

        let invocation_result =
            invoke_c_native(panicking_native_function, &mut virtual_machine, &[]);

        assert!(
            matches!(
                invocation_result,
                Err(RuntimeError::NativeFunctionError { .. })
            ),
            "Foreign plugin panic must be caught cleanly and converted to NativeFunctionError"
        );
    }
}
