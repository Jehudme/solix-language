//! Virtual machine context accessor and heap allocator function pointers.

use std::ffi::{c_char, c_void};

/// Magic identifier validating a valid `SolixVmContext` table (`"SOLIXVMC"`).
pub const SOLIX_VM_CONTEXT_MAGIC: u64 = 0x534F_4C49_5856_4D43;

/// Element tag for `int8[]` arrays passed to [`SolixVmContext::alloc_array`].
pub const SOLIX_ARRAY_TAG_I8: u8 = 1;
/// Element tag for `int64[]` arrays passed to [`SolixVmContext::alloc_array`].
pub const SOLIX_ARRAY_TAG_I64: u8 = 4;
/// Element tag for `float32[]` arrays passed to [`SolixVmContext::alloc_array`].
pub const SOLIX_ARRAY_TAG_F32: u8 = 9;
/// Element tag for `float64[]` arrays passed to [`SolixVmContext::alloc_array`].
pub const SOLIX_ARRAY_TAG_F64: u8 = 10;
/// Element tag for `char[]` arrays passed to [`SolixVmContext::alloc_array`].
pub const SOLIX_ARRAY_TAG_CHAR: u8 = 12;
/// Element tag for `string[]` / reference arrays passed to [`SolixVmContext::alloc_array`].
pub const SOLIX_ARRAY_TAG_REF: u8 = 13;

/// C-ABI function pointer table passed to native extension callbacks.
#[repr(C)]
pub struct SolixVmContext {
    /// Validation magic number (`SOLIX_VM_CONTEXT_MAGIC`).
    pub magic: u64,
    /// Opaque pointer to the host `VirtualMachine` instance.
    pub vm_ptr: *mut c_void,
    /// Reads a 32-bit integer field from an object at the given offset.
    pub get_field_i32:
        unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, field_index: u32) -> i32,
    /// Writes a 32-bit integer field to an object at the given offset.
    pub set_field_i32: unsafe extern "C" fn(
        vm_ptr: *mut c_void,
        object_address: u32,
        field_index: u32,
        value: i32,
    ),
    /// Reads raw bytes from a heap buffer or string into `out_buffer`.
    pub read_bytes: unsafe extern "C" fn(
        vm_ptr: *mut c_void,
        address: u32,
        out_buffer: *mut u8,
        capacity: u32,
    ) -> u32,
    /// Allocates an immutable string object on the ARC heap from UTF-8 bytes.
    pub alloc_string:
        unsafe extern "C" fn(vm_ptr: *mut c_void, pointer: *const u8, length: u32) -> u32,
    /// Allocates a mutable character buffer on the ARC heap.
    pub alloc_chars:
        unsafe extern "C" fn(vm_ptr: *mut c_void, pointer: *const u8, length: u32) -> u32,
    /// Allocates a raw byte array on the ARC heap.
    pub alloc_byte_array:
        unsafe extern "C" fn(vm_ptr: *mut c_void, pointer: *const u8, length: u32) -> u32,
    /// Allocates an array of the given length and element type tag.
    pub alloc_array: unsafe extern "C" fn(vm_ptr: *mut c_void, length: u32, type_tag: u8) -> u32,
    /// Returns the length of an array object.
    pub array_len: unsafe extern "C" fn(vm_ptr: *mut c_void, array_address: u32) -> u32,
    /// Reads an element from an array as a 64-bit raw value.
    pub array_get_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, array_address: u32, index: u32) -> u64,
    /// Writes an element to an array as a 64-bit raw value.
    pub array_set_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, array_address: u32, index: u32, raw_value: u64),
    /// Allocates an instance of a nominal class object with the specified vtable.
    pub alloc_object:
        unsafe extern "C" fn(vm_ptr: *mut c_void, class_vtable_id: u32, field_count: u32) -> u32,
    /// Reads an arbitrary 64-bit field from an object instance.
    pub read_field_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, offset: u32) -> u64,
    /// Writes an arbitrary 64-bit field to an object instance.
    pub write_field_u64:
        unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, offset: u32, raw_value: u64),
    /// Resolves the vtable ID for a class by name.
    pub get_class_vtable_id:
        unsafe extern "C" fn(vm_ptr: *mut c_void, name_pointer: *const c_char) -> u32,
    /// Evaluates a runtime execution or environment metric by identifier (0..=55).
    pub get_vm_metric: unsafe extern "C" fn(vm_ptr: *mut c_void, metric_id: u32) -> u64,
    /// Triggers synchronous cycle collection across the ARC heap and returns count of collected objects.
    pub collect_cycles: unsafe extern "C" fn(vm_ptr: *mut c_void) -> u64,
    /// Updates the cycle collector candidate threshold.
    pub set_cycle_threshold: unsafe extern "C" fn(vm_ptr: *mut c_void, threshold: u32),
}

impl SolixVmContext {
    /// Safely reads a UTF-8 string from the virtual machine heap.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn read_string(&self, heap_address: u32) -> Option<String> {
        if heap_address == 0 {
            return None;
        }
        let mut buffer = vec![0u8; 1024];
        let bytes_read = unsafe {
            (self.read_bytes)(
                self.vm_ptr,
                heap_address,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            )
        };
        if bytes_read as usize > buffer.len() {
            buffer.resize(bytes_read as usize, 0);
            unsafe {
                (self.read_bytes)(self.vm_ptr, heap_address, buffer.as_mut_ptr(), bytes_read);
            }
        } else {
            buffer.truncate(bytes_read as usize);
        }
        String::from_utf8(buffer).ok()
    }

    /// Allocates a new UTF-8 string on the virtual machine ARC heap.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn alloc_string(&self, text: &str) -> u32 {
        unsafe { (self.alloc_string)(self.vm_ptr, text.as_ptr(), text.len() as u32) }
    }

    /// Reads raw bytes from a heap address into `destination_buffer`.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn read_bytes(&self, heap_address: u32, destination_buffer: &mut [u8]) -> usize {
        let count = unsafe {
            (self.read_bytes)(
                self.vm_ptr,
                heap_address,
                destination_buffer.as_mut_ptr(),
                destination_buffer.len() as u32,
            )
        };
        count as usize
    }

    /// Safely evaluates a VM runtime metric by its 0-indexed identifier (0..=55).
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn get_vm_metric(&self, metric_id: u32) -> u64 {
        unsafe { (self.get_vm_metric)(self.vm_ptr, metric_id) }
    }

    /// Safely triggers cycle collection and returns the number of reclaimed objects.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn collect_cycles(&self) -> u64 {
        unsafe { (self.collect_cycles)(self.vm_ptr) }
    }

    /// Safely sets the cycle collector candidate threshold.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn set_cycle_threshold(&self, threshold: u32) {
        unsafe { (self.set_cycle_threshold)(self.vm_ptr, threshold) }
    }

    /// Validates an opaque `vm_context` pointer received from the Solix VM.
    ///
    /// # Safety
    /// If non-null, `raw_context` must point to a valid initialized `SolixVmContext`.
    pub unsafe fn validate<'ctx>(raw_context: *mut c_void) -> Option<&'ctx Self> {
        if raw_context.is_null() {
            return None;
        }
        let context_ref = unsafe { &*(raw_context as *const Self) };
        if context_ref.magic == SOLIX_VM_CONTEXT_MAGIC && !context_ref.vm_ptr.is_null() {
            Some(context_ref)
        } else {
            None
        }
    }

    /// Reads a 32-bit signed integer field from an object instance.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn get_field_i32(&self, object_address: u32, field_index: u32) -> i32 {
        unsafe { (self.get_field_i32)(self.vm_ptr, object_address, field_index) }
    }

    /// Writes a 32-bit signed integer field to an object instance.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn set_field_i32(&self, object_address: u32, field_index: u32, field_value: i32) {
        unsafe { (self.set_field_i32)(self.vm_ptr, object_address, field_index, field_value) }
    }

    /// Allocates a mutable `char[]` array on the guest heap from UTF-8 bytes.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn alloc_chars(&self, text: &str) -> u32 {
        unsafe { (self.alloc_chars)(self.vm_ptr, text.as_ptr(), text.len() as u32) }
    }

    /// Allocates an `int8[]` array on the guest heap from raw bytes.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn alloc_byte_array(&self, raw_bytes: &[u8]) -> u32 {
        unsafe { (self.alloc_byte_array)(self.vm_ptr, raw_bytes.as_ptr(), raw_bytes.len() as u32) }
    }

    /// Allocates an `ArrayRef` of `element_count` slots with the specified `type_tag`.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn alloc_array(&self, element_count: u32, type_tag: u8) -> u32 {
        unsafe { (self.alloc_array)(self.vm_ptr, element_count, type_tag) }
    }

    /// Returns the element length of a guest array or byte length of a guest string.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn array_len(&self, array_address: u32) -> u32 {
        unsafe { (self.array_len)(self.vm_ptr, array_address) }
    }

    /// Reads an element from a guest array as raw 64-bit bits.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn array_get_u64(&self, array_address: u32, element_index: u32) -> u64 {
        unsafe { (self.array_get_u64)(self.vm_ptr, array_address, element_index) }
    }

    /// Writes a 64-bit raw value into a guest array element slot.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn array_set_u64(&self, array_address: u32, element_index: u32, raw_value: u64) {
        unsafe { (self.array_set_u64)(self.vm_ptr, array_address, element_index, raw_value) }
    }

    /// Looks up a registered class VTable identifier by qualified or simple class name.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn get_class_vtable_id(&self, class_name: &str) -> u32 {
        let Ok(c_name) = std::ffi::CString::new(class_name) else {
            return 0;
        };
        unsafe { (self.get_class_vtable_id)(self.vm_ptr, c_name.as_ptr()) }
    }

    /// Allocates an object instance for `class_vtable_id` with `field_count` field slots.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn alloc_object(&self, class_vtable_id: u32, field_count: u32) -> u32 {
        unsafe { (self.alloc_object)(self.vm_ptr, class_vtable_id, field_count) }
    }

    /// Reads an arbitrary 64-bit field slot from an object instance.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn read_field_u64(&self, object_address: u32, field_offset: u32) -> u64 {
        unsafe { (self.read_field_u64)(self.vm_ptr, object_address, field_offset) }
    }

    /// Writes an arbitrary 64-bit field slot into an object instance.
    ///
    /// # Safety
    /// `self.vm_ptr` must point to a valid, live VM instance.
    pub unsafe fn write_field_u64(&self, object_address: u32, field_offset: u32, raw_value: u64) {
        unsafe { (self.write_field_u64)(self.vm_ptr, object_address, field_offset, raw_value) }
    }
}
