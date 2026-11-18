//! Unit tests for extended `SolixVmContext` native FFI host callbacks.

use solix_common::bytecode::BinaryProgram;
use solix_runtime::context::RuntimeContext;
use solix_runtime::native::ffi::{
    SOLIX_VM_CONTEXT_MAGIC, SolixVmContext, solix_vm_alloc_array, solix_vm_alloc_byte_array,
    solix_vm_alloc_chars, solix_vm_alloc_object, solix_vm_alloc_string, solix_vm_array_get_u64,
    solix_vm_array_len, solix_vm_array_set_u64, solix_vm_collect_cycles,
    solix_vm_get_class_vtable_id, solix_vm_get_field_i32, solix_vm_get_metric, solix_vm_read_bytes,
    solix_vm_read_field_u64, solix_vm_set_cycle_threshold, solix_vm_set_field_i32,
    solix_vm_write_field_u64,
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
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
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
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
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
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
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
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let target_vtable_id: u32 = 42;
    let field_slot_count: u32 = 4;
    let object_address =
        unsafe { (context_wrapper.alloc_object)(context_ptr, target_vtable_id, field_slot_count) };
    assert_ne!(
        object_address, 0,
        "Object allocation must return non-zero address"
    );

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

#[test]
fn test_ffi_get_class_vtable_id_lookup() {
    let mut binary_program = BinaryProgram::new();
    binary_program
        .classes
        .push(solix_common::CompiledClass::new(
            "models::Vector2",
            105,
            None,
            16,
        ));
    binary_program
        .classes
        .push(solix_common::CompiledClass::new("Color", 106, None, 16));

    let options = RuntimeOptions::new().with_binary(binary_program);
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
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    let vector2_vtable =
        unsafe { (context_wrapper.get_class_vtable_id)(context_ptr, c"Vector2".as_ptr()) };
    assert_eq!(vector2_vtable, 105);

    let full_symbol_vtable =
        unsafe { (context_wrapper.get_class_vtable_id)(context_ptr, c"models::Vector2".as_ptr()) };
    assert_eq!(full_symbol_vtable, 105);

    let color_vtable =
        unsafe { (context_wrapper.get_class_vtable_id)(context_ptr, c"Color".as_ptr()) };
    assert_eq!(color_vtable, 106);

    let unknown_vtable =
        unsafe { (context_wrapper.get_class_vtable_id)(context_ptr, c"NonExistentClass".as_ptr()) };
    assert_eq!(unknown_vtable, 0);
}

#[test]
fn test_ffi_get_vm_metric_telemetry() {
    let mut binary_program = BinaryProgram::new();
    binary_program
        .functions
        .push(solix_common::bytecode::CompiledFunction::new(
            "test::sample_function",
            0,
            0,
            4,
        ));

    let options = RuntimeOptions::new()
        .with_binary(binary_program)
        .with_container_kind("slxpac")
        .with_container_resources(5, 4096);
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
        get_class_vtable_id: solix_vm_get_class_vtable_id,
        get_vm_metric: solix_vm_get_metric,
        collect_cycles: solix_vm_collect_cycles,
        set_cycle_threshold: solix_vm_set_cycle_threshold,
    };
    let context_ptr = &mut context_wrapper as *mut SolixVmContext as *mut c_void;

    // Metric 28: func_total_registered
    let func_count = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 28) };
    assert_eq!(func_count, 1);

    // Metric 36: container_is_slxpac
    let is_slxpac = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 36) };
    assert_eq!(is_slxpac, 1);

    // Metric 37: container_is_slxarc
    let is_slxarc = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 37) };
    assert_eq!(is_slxarc, 0);

    // Metric 40: container_resource_count
    let resource_count = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 40) };
    assert_eq!(resource_count, 5);

    // Metric 41: container_resource_total_bytes
    let resource_bytes = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 41) };
    assert_eq!(resource_bytes, 4096);

    // Metric 44: process_pid
    let process_pid = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 44) };
    assert_eq!(process_pid, std::process::id() as u64);

    // Metric 52: sys_available_parallelism
    let parallelism = unsafe { (context_wrapper.get_vm_metric)(context_ptr, 52) };
    assert!(parallelism >= 1);
}

#[test]
fn test_call_symbol_does_not_quicken_native_wrapper_stub_into_self_recursion() {
    use solix_common::bytecode::{
        BinaryProgram, CompiledFunction, ConstantPoolValue, Instruction, OpCode,
    };

    // Build a binary containing a package-scoped native stub wrapper `demo_pkg::println`
    // that delegates via CallSymbol("demo_pkg::println"), plus `demo_pkg::main` calling it twice.
    let mut program = BinaryProgram::new();
    program.package_name = "demo_pkg".to_string();
    program
        .constant_pool
        .values
        .push(ConstantPoolValue::String("demo_pkg::println".to_string()));
    program
        .constant_pool
        .values
        .push(ConstantPoolValue::String("hello_once".to_string()));
    program
        .constant_pool
        .values
        .push(ConstantPoolValue::String("hello_twice".to_string()));
    program.entry_point = Some("demo_pkg::main".to_string());
    program.entry_point_id = Some(1);

    let mut stub_func = CompiledFunction::new("demo_pkg::println", 0, 1, 2);
    stub_func.instructions = vec![
        Instruction::format_call(OpCode::CallSymbol, 1, 0, 0, 1),
        Instruction::format_a(OpCode::Return, 0, 0, 0, 0),
    ];
    program.functions.push(stub_func);

    let mut main_func = CompiledFunction::new("demo_pkg::main", 0, 0, 2);
    main_func.is_public = true;
    main_func.instructions = vec![
        Instruction::format_b(OpCode::LoadConst, 0, 0, 1),
        Instruction::format_call(OpCode::CallDirect, 1, 0, 0, 1),
        Instruction::format_b(OpCode::LoadConst, 0, 0, 2),
        Instruction::format_call(OpCode::CallDirect, 1, 0, 0, 1),
        Instruction::format_a(OpCode::Return, 0, 0, 0, 0),
    ];
    program.functions.push(main_func);

    let options = RuntimeOptions::new().with_binary(program);
    let context = RuntimeContext::new(options).expect("construct runtime context");
    let mut vm_instance = VirtualMachine::new(context);
    vm_instance
        .run()
        .expect("repeated calls to package-scoped native stub must not stack overflow");
    assert_eq!(
        vm_instance.stdout,
        vec!["hello_once".to_string(), "hello_twice".to_string()]
    );
}
