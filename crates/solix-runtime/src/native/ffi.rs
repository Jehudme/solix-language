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

/// Safely resolves a `*mut c_void` context pointer into an optional `VirtualMachine` reference.
unsafe fn resolve_vm<'vm>(vm_context: *mut c_void) -> Option<&'vm mut VirtualMachine> {
    if vm_context.is_null() {
        return None;
    }
    let actual_vm_ptr = {
        let context_pointer = vm_context as *mut SolixVmContext;
        if !context_pointer.is_null()
            && unsafe { (*context_pointer).magic } == SOLIX_VM_CONTEXT_MAGIC
        {
            unsafe { (*context_pointer).vm_ptr }
        } else {
            vm_context
        }
    };
    if actual_vm_ptr.is_null() {
        None
    } else {
        Some(unsafe { &mut *(actual_vm_ptr as *mut VirtualMachine) })
    }
}

/// Reads bytes from a guest `string`, `char[]`, or `int8[]` into `out_buffer` (up to `capacity` bytes).
/// Returns the total byte/element length of the string or array.
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
            let end_offset = start_offset + copy_length;
            if end_offset <= vm_instance.heap.buffer.len() {
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
        let element_size = vm_instance
            .heap
            .buffer
            .get(array_offset + 4)
            .copied()
            .unwrap_or(8)
            .max(1) as usize;

        if !out_buffer.is_null() && capacity > 0 {
            let copy_count = (capacity as usize).min(total_length as usize);
            for element_index in 0..copy_count {
                let element_offset = array_offset + 8 + (element_index * element_size);
                if element_offset < vm_instance.heap.buffer.len() {
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
    vm_instance.interned_string_addrs.push(string_address);
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
    let payload_size = 8 + (char_count as usize) * 8;
    let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(array_address, char_count);
    let _ = vm_instance.heap.write_u8(array_address + 4, 8);
    let _ = vm_instance.heap.write_u8(array_address + 7, 12); // Type tag 12 = Char

    if !source_pointer.is_null() && char_count > 0 {
        let source_slice =
            unsafe { std::slice::from_raw_parts(source_pointer, char_count as usize) };
        for (character_index, &character_byte) in source_slice.iter().enumerate() {
            let slot_address = array_address + 8 + (character_index as u32 * 8);
            let _ = vm_instance
                .heap
                .write_u64(slot_address, character_byte as u64);
        }
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
    let payload_size = 8 + (byte_count as usize) * 8;
    let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(array_address, byte_count);
    let _ = vm_instance.heap.write_u8(array_address + 4, 8);
    let _ = vm_instance.heap.write_u8(array_address + 7, 1); // Type tag 1 = Int8

    if !source_pointer.is_null() && byte_count > 0 {
        let source_slice =
            unsafe { std::slice::from_raw_parts(source_pointer, byte_count as usize) };
        for (byte_index, &raw_byte) in source_slice.iter().enumerate() {
            let slot_address = array_address + 8 + (byte_index as u32 * 8);
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
    let payload_size = 8 + (element_count as usize) * 8;
    let array_address = match vm_instance.heap.allocate(payload_size, BlockTag::ArrayRef) {
        Ok(allocated_address) => allocated_address,
        Err(_) => return 0,
    };
    let _ = vm_instance.heap.write_u32(array_address, element_count);
    let _ = vm_instance.heap.write_u8(array_address + 4, 8);
    let _ = vm_instance.heap.write_u8(array_address + 7, type_tag);
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
    let element_size = vm_instance
        .heap
        .read_u8(array_address + 4)
        .unwrap_or(8)
        .max(1) as usize;
    let type_tag = vm_instance.heap.read_u8(array_address + 7).unwrap_or(0);
    let slot_address = array_address + 8 + (element_index * element_size as u32);
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
    let element_size = vm_instance
        .heap
        .read_u8(array_address + 4)
        .unwrap_or(8)
        .max(1) as usize;
    let type_tag = vm_instance.heap.read_u8(array_address + 7).unwrap_or(0);
    let slot_address = array_address + 8 + (element_index * element_size as u32);
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
    let min_size = 8 + (field_count as usize) * 8;
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
    let class_descriptor_opt =
        vm_instance
            .heap
            .read_u32(object_address)
            .ok()
            .and_then(|vtable_identifier| {
                vm_instance
                    .context
                    .get_class_by_vtable_id(vtable_identifier)
            });

    if let Some(class_descriptor) = class_descriptor_opt
        && !class_descriptor.field_types.is_empty()
    {
        let (actual_offset, field_tag) =
            if offset < 64 && (offset as usize) < class_descriptor.field_types.len() {
                let mut sorted_field_list = class_descriptor.field_types.clone();
                sorted_field_list.sort_by_key(|&(field_offset, _)| field_offset);
                let (field_offset, resolved_tag) = sorted_field_list[offset as usize];
                (field_offset as u32, resolved_tag)
            } else {
                let direct_offset = offset as usize;
                let resolved_tag = class_descriptor.get_field_type(direct_offset).unwrap_or(0);
                (offset, resolved_tag)
            };

        let field_addr = object_address + actual_offset;
        match field_tag {
            1 => vm_instance.heap.read_u8(field_addr).unwrap_or(0) as i8 as i64 as u64,
            2 => vm_instance.heap.read_u8(field_addr).unwrap_or(0) as u64,
            3 => vm_instance.heap.read_u16(field_addr).unwrap_or(0) as i16 as i64 as u64,
            4 => vm_instance.heap.read_u16(field_addr).unwrap_or(0) as u64,
            5 => vm_instance.heap.read_u32(field_addr).unwrap_or(0) as i32 as i64 as u64,
            6 | 9 => vm_instance.heap.read_u32(field_addr).unwrap_or(0) as u64,
            7 | 8 | 10 => vm_instance.heap.read_u64(field_addr).unwrap_or(0),
            11 => {
                if vm_instance.heap.read_u8(field_addr).unwrap_or(0) != 0 {
                    1
                } else {
                    0
                }
            }
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
    } else {
        let actual_offset = if offset < 8 { 8 + offset * 8 } else { offset };
        vm_instance
            .heap
            .read_u64(object_address + actual_offset)
            .unwrap_or(0)
    }
}

/// Writes a 64-bit field to an object at a field index or direct byte offset.
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
    let class_descriptor_opt =
        vm_instance
            .heap
            .read_u32(object_address)
            .ok()
            .and_then(|vtable_identifier| {
                vm_instance
                    .context
                    .get_class_by_vtable_id(vtable_identifier)
            });

    if let Some(class_descriptor) = class_descriptor_opt
        && !class_descriptor.field_types.is_empty()
    {
        let (actual_offset, field_tag) =
            if offset < 64 && (offset as usize) < class_descriptor.field_types.len() {
                let mut sorted_field_list = class_descriptor.field_types.clone();
                sorted_field_list.sort_by_key(|&(field_offset, _)| field_offset);
                let (field_offset, resolved_tag) = sorted_field_list[offset as usize];
                (field_offset as u32, resolved_tag)
            } else {
                let direct_offset = offset as usize;
                let resolved_tag = class_descriptor.get_field_type(direct_offset).unwrap_or(0);
                (offset, resolved_tag)
            };

        let field_addr = object_address + actual_offset;
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
                    .write_u8(field_addr, if raw_value != 0 { 1 } else { 0 });
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
    } else {
        let actual_offset = if offset < 8 { 8 + offset * 8 } else { offset };
        let _ = vm_instance
            .heap
            .write_u64(object_address + actual_offset, raw_value);
    }
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
