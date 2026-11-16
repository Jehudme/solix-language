//! Virtual machine context accessor and heap allocator function pointers.

use std::ffi::{c_char, c_void};

/// Magic identifier validating a valid `SolixVmContext` table.
pub const SOLIX_VM_CONTEXT_MAGIC: u64 = 0x534F4C4958564D31; // "SOLIXVM1"

/// C-ABI function pointer table passed to native extension callbacks.
#[repr(C)]
pub struct SolixVmContext {
    /// Validation magic number (`SOLIX_VM_CONTEXT_MAGIC`).
    pub magic: u64,
    /// Opaque pointer to the host `VirtualMachine` instance.
    pub vm_ptr: *mut c_void,
    /// Reads a 32-bit integer field from an object at the given offset.
    pub get_field_i32: unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, field_index: u32) -> i32,
    /// Writes a 32-bit integer field to an object at the given offset.
    pub set_field_i32: unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, field_index: u32, value: i32),
    /// Reads raw bytes from a heap buffer or string into `out_buffer`.
    pub read_bytes: unsafe extern "C" fn(vm_ptr: *mut c_void, address: u32, out_buffer: *mut u8, capacity: u32) -> u32,
    /// Allocates an immutable string object on the ARC heap from UTF-8 bytes.
    pub alloc_string: unsafe extern "C" fn(vm_ptr: *mut c_void, pointer: *const u8, length: u32) -> u32,
    /// Allocates a mutable character buffer on the ARC heap.
    pub alloc_chars: unsafe extern "C" fn(vm_ptr: *mut c_void, pointer: *const u8, length: u32) -> u32,
    /// Allocates a raw byte array on the ARC heap.
    pub alloc_byte_array: unsafe extern "C" fn(vm_ptr: *mut c_void, pointer: *const u8, length: u32) -> u32,
    /// Allocates an array of the given length and element type tag.
    pub alloc_array: unsafe extern "C" fn(vm_ptr: *mut c_void, length: u32, type_tag: u8) -> u32,
    /// Returns the length of an array object.
    pub array_len: unsafe extern "C" fn(vm_ptr: *mut c_void, array_address: u32) -> u32,
    /// Reads an element from an array as a 64-bit raw value.
    pub array_get_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, array_address: u32, index: u32) -> u64,
    /// Writes an element to an array as a 64-bit raw value.
    pub array_set_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, array_address: u32, index: u32, raw_value: u64),
    /// Allocates an instance of a nominal class object with the specified vtable.
    pub alloc_object: unsafe extern "C" fn(vm_ptr: *mut c_void, class_vtable_id: u32, field_count: u32) -> u32,
    /// Reads an arbitrary 64-bit field from an object instance.
    pub read_field_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, offset: u32) -> u64,
    /// Writes an arbitrary 64-bit field to an object instance.
    pub write_field_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, object_address: u32, offset: u32, raw_value: u64),
    /// Resolves the vtable ID for a class by name.
    pub get_class_vtable_id: unsafe extern "C" fn(vm_ptr: *mut c_void, name_pointer: *const c_char) -> u32,
    /// Evaluates a runtime execution or environment metric by identifier (0..=55).
    pub get_vm_metric: unsafe extern "C" fn(vm_ptr: *mut c_void, metric_id: u32) -> u64,
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
            (self.read_bytes)(self.vm_ptr, heap_address, buffer.as_mut_ptr(), buffer.len() as u32)
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
        unsafe {
            (self.alloc_string)(self.vm_ptr, text.as_ptr(), text.len() as u32)
        }
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
    pub unsafe fn get_metric(&self, metric_id: u32) -> u64 {
        unsafe { (self.get_vm_metric)(self.vm_ptr, metric_id) }
    }
}
