//! Unit tests for extended `SolixVmContext` native FFI host callbacks.

use solix_common::bytecode::BinaryProgram;
use solix_runtime::context::RuntimeContext;
use solix_runtime::native::ffi::{
    SOLIX_VM_CONTEXT_MAGIC, SolixVmContext, solix_vm_alloc_array, solix_vm_alloc_byte_array,
    solix_vm_alloc_chars, solix_vm_alloc_object, solix_vm_alloc_string, solix_vm_array_get_u64,
    solix_vm_array_len, solix_vm_array_set_u64, solix_vm_get_field_i32, solix_vm_read_bytes,
    solix_vm_read_field_u64, solix_vm_set_field_i32, solix_vm_write_field_u64,
};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::vm::VirtualMachine;
use std::ffi::c_void;

#[test]
fn test_ffi_string_allocation_and_read() {
    let options = RuntimeOptions::new().with_binary(BinaryProgram::new());
    let context = RuntimeContext::new(options).expect("failed to construct context");
    let mut virtual_machine = VirtualMachine::new(context);

    let vm_pointer = &mut virtual_machine as *mut VirtualMachine as *mut c_void;
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
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let sample_text = b"Hello Solix Standard Library!";
    let string_address = unsafe {
        (context_wrapper.alloc_string)(context_ptr, sample_text.as_ptr(), sample_text.len() as u32)
    };
    assert_ne!(string_address, 0, "String address must not be zero");

    let string_length = unsafe { (context_wrapper.array_len)(context_ptr, string_address) };
    assert_eq!(string_length as usize, sample_text.len());

    let mut read_buffer = vec![0u8; sample_text.len()];
    let bytes_read = unsafe {
        (context_wrapper.read_bytes)(
            context_ptr,
            string_address,
            read_buffer.as_mut_ptr(),
            read_buffer.len() as u32,
        )
    };
    assert_eq!(bytes_read as usize, sample_text.len());
    assert_eq!(&read_buffer[..], sample_text);
}

#[test]
fn test_ffi_char_and_byte_arrays() {
    let options = RuntimeOptions::new().with_binary(BinaryProgram::new());
    let context = RuntimeContext::new(options).expect("failed to construct context");
    let mut virtual_machine = VirtualMachine::new(context);

    let vm_pointer = &mut virtual_machine as *mut VirtualMachine as *mut c_void;
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
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let sample_chars = b"SolixChars";
    let char_array_address = unsafe {
        (context_wrapper.alloc_chars)(
            context_ptr,
            sample_chars.as_ptr(),
            sample_chars.len() as u32,
        )
    };
    assert_ne!(char_array_address, 0);

    let char_array_length = unsafe { (context_wrapper.array_len)(context_ptr, char_array_address) };
    assert_eq!(char_array_length as usize, sample_chars.len());

    let mut char_buffer = vec![0u8; sample_chars.len()];
    let read_count = unsafe {
        (context_wrapper.read_bytes)(
            context_ptr,
            char_array_address,
            char_buffer.as_mut_ptr(),
            char_buffer.len() as u32,
        )
    };
    assert_eq!(read_count as usize, sample_chars.len());
    assert_eq!(&char_buffer[..], sample_chars);

    // Test byte array (int8[])
    let sample_bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    let byte_array_address = unsafe {
        (context_wrapper.alloc_byte_array)(
            context_ptr,
            sample_bytes.as_ptr(),
            sample_bytes.len() as u32,
        )
    };
    assert_ne!(byte_array_address, 0);

    let mut byte_buffer = vec![0u8; sample_bytes.len()];
    let bytes_read = unsafe {
        (context_wrapper.read_bytes)(
            context_ptr,
            byte_array_address,
            byte_buffer.as_mut_ptr(),
            byte_buffer.len() as u32,
        )
    };
    assert_eq!(bytes_read as usize, sample_bytes.len());
    assert_eq!(&byte_buffer[..], &sample_bytes);
}

#[test]
fn test_ffi_generic_array_manipulation() {
    let options = RuntimeOptions::new().with_binary(BinaryProgram::new());
    let context = RuntimeContext::new(options).expect("failed to construct context");
    let mut virtual_machine = VirtualMachine::new(context);

    let vm_pointer = &mut virtual_machine as *mut VirtualMachine as *mut c_void;
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
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    // Allocate an array of 5 int64 elements (type tag 4)
    let array_address = unsafe { (context_wrapper.alloc_array)(context_ptr, 5, 4) };
    assert_ne!(array_address, 0);

    let length = unsafe { (context_wrapper.array_len)(context_ptr, array_address) };
    assert_eq!(length, 5);

    unsafe {
        (context_wrapper.array_set_u64)(context_ptr, array_address, 0, 100);
        (context_wrapper.array_set_u64)(context_ptr, array_address, 1, 200);
        (context_wrapper.array_set_u64)(context_ptr, array_address, 2, 300);
        (context_wrapper.array_set_u64)(context_ptr, array_address, 3, 400);
        (context_wrapper.array_set_u64)(context_ptr, array_address, 4, 500);
    }

    assert_eq!(
        unsafe { (context_wrapper.array_get_u64)(context_ptr, array_address, 0) },
        100
    );
    assert_eq!(
        unsafe { (context_wrapper.array_get_u64)(context_ptr, array_address, 1) },
        200
    );
    assert_eq!(
        unsafe { (context_wrapper.array_get_u64)(context_ptr, array_address, 2) },
        300
    );
    assert_eq!(
        unsafe { (context_wrapper.array_get_u64)(context_ptr, array_address, 3) },
        400
    );
    assert_eq!(
        unsafe { (context_wrapper.array_get_u64)(context_ptr, array_address, 4) },
        500
    );
    assert_eq!(
        unsafe { (context_wrapper.array_get_u64)(context_ptr, array_address, 5) },
        0
    );
}

#[test]
fn test_ffi_object_allocation_and_u64_fields() {
    let options = RuntimeOptions::new().with_binary(BinaryProgram::new());
    let context = RuntimeContext::new(options).expect("failed to construct context");
    let mut virtual_machine = VirtualMachine::new(context);

    let vm_pointer = &mut virtual_machine as *mut VirtualMachine as *mut c_void;
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
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let target_vtable_id: u32 = 42;
    let field_slot_count: u32 = 4;
    let object_address = unsafe {
        (context_wrapper.alloc_object)(context_ptr, target_vtable_id, field_slot_count)
    };
    assert_ne!(object_address, 0, "Object allocation must return non-zero address");

    // Write 64-bit field values at indices 0, 1, 2, 3
    unsafe {
        (context_wrapper.write_field_u64)(context_ptr, object_address, 0, 0x1122_3344_5566_7788);
        (context_wrapper.write_field_u64)(context_ptr, object_address, 1, 0xAABB_CCDD_EEFF_0011);
        (context_wrapper.write_field_u64)(context_ptr, object_address, 2, 1024);
        (context_wrapper.write_field_u64)(context_ptr, object_address, 3, 2048);
    }

    assert_eq!(
        unsafe { (context_wrapper.read_field_u64)(context_ptr, object_address, 0) },
        0x1122_3344_5566_7788
    );
    assert_eq!(
        unsafe { (context_wrapper.read_field_u64)(context_ptr, object_address, 1) },
        0xAABB_CCDD_EEFF_0011
    );
    assert_eq!(
        unsafe { (context_wrapper.read_field_u64)(context_ptr, object_address, 2) },
        1024
    );
    assert_eq!(
        unsafe { (context_wrapper.read_field_u64)(context_ptr, object_address, 3) },
        2048
    );
}
