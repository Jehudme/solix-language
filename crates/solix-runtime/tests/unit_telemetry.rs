//! Unit tests validating HeapTelemetry and VmTelemetry recording.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_runtime::context::RuntimeContext;
use solix_runtime::memory::{BlockTag, VirtualHeap};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::Value;
use solix_runtime::vm::VirtualMachine;

#[test]
fn test_heap_telemetry_tracking() {
    let mut heap = VirtualHeap::new(1024, 65536);
    let initial_telemetry = heap.telemetry();
    assert_eq!(initial_telemetry.total_allocations_count, 0);
    assert_eq!(initial_telemetry.total_deallocations_count, 0);
    assert_eq!(initial_telemetry.current_allocated_bytes, 0);
    assert_eq!(initial_telemetry.peak_allocated_bytes, 0);
    assert_eq!(initial_telemetry.active_live_objects, 0);

    let address = heap
        .allocate(128, BlockTag::Object)
        .expect("allocation should succeed");
    let after_alloc_telemetry = heap.telemetry();
    assert_eq!(after_alloc_telemetry.total_allocations_count, 1);
    assert_eq!(after_alloc_telemetry.total_deallocations_count, 0);
    assert!(after_alloc_telemetry.current_allocated_bytes >= 128);
    assert_eq!(
        after_alloc_telemetry.peak_allocated_bytes,
        after_alloc_telemetry.current_allocated_bytes
    );
    assert_eq!(after_alloc_telemetry.active_live_objects, 1);

    heap.deallocate(address).expect("deallocate should succeed");
    let after_dealloc_telemetry = heap.telemetry();
    assert_eq!(after_dealloc_telemetry.total_allocations_count, 1);
    assert_eq!(after_dealloc_telemetry.total_deallocations_count, 1);
    assert_eq!(after_dealloc_telemetry.current_allocated_bytes, 0);
    assert!(after_dealloc_telemetry.peak_allocated_bytes >= 128);
    assert_eq!(after_dealloc_telemetry.active_live_objects, 0);
}

#[test]
fn test_vm_telemetry_instruction_and_stack_tracking() {
    let mut program = BinaryProgram::new();

    // Function helper(x) -> x + 5
    let mut helper_func = CompiledFunction::new("helper", 0, 1, 2);
    helper_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 5));
    helper_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 1));
    helper_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    program.functions.push(helper_func);

    // Function main() -> calls helper(37)
    let mut main_func = CompiledFunction::new("main", 0, 0, 3);
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 37));
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallDirect, 0, 0, 1, 1));
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    program.functions.push(main_func);
    program.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(program);
    let context = RuntimeContext::new(options).expect("Runtime context construction should succeed");
    let mut machine = VirtualMachine::new(context);

    let result = machine.run().expect("VM run should succeed");
    assert_eq!(result, Value::I32(42));

    let telemetry = machine.telemetry();
    assert!(
        telemetry.instructions_dispatched >= 6,
        "instructions dispatched was {}",
        telemetry.instructions_dispatched
    );
    assert!(
        telemetry.peak_call_stack_depth >= 2,
        "peak call stack depth was {}",
        telemetry.peak_call_stack_depth
    );
    assert!(
        telemetry.pure_execution_time_ns > 0,
        "pure execution time was {}",
        telemetry.pure_execution_time_ns
    );
}

#[test]
fn test_vm_telemetry_native_call_tracking() {
    let mut program = BinaryProgram::new();

    // Call native builtin "clock"
    let mut main_func = CompiledFunction::new("main", 0, 0, 2);
    let sym_idx = program.constant_pool.add_string("clock").unwrap();
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallSymbol, 0, sym_idx, 0, 0));
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    program.functions.push(main_func);
    program.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(program);
    let context = RuntimeContext::new(options).expect("Runtime context construction should succeed");
    let mut machine = VirtualMachine::new(context);

    let result = machine.run().expect("VM run should succeed");
    assert!(matches!(result, Value::I64(_)));

    let telemetry = machine.telemetry();
    assert_eq!(telemetry.native_calls_count, 1);
}

#[test]
fn test_vm_telemetry_quickened_instruction_tracking() {
    // Binary 1: Library with exported function "math::double"
    let mut lib_bin = BinaryProgram::new();
    let mut double_func = CompiledFunction::new("math::double", 0, 1, 2);
    double_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 0));
    double_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    lib_bin.functions.push(double_func);

    // Binary 2: Consumer calling "math::double" via CallExtern
    let mut app_bin = BinaryProgram::new();
    let sym_idx = app_bin
        .constant_pool
        .add_string("math::double")
        .expect("Symbol add should succeed");
    let mut main_func = CompiledFunction::new("main", 0, 0, 3);
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 21));
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallExtern, 0, sym_idx, 1, 1));
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    app_bin.functions.push(main_func);
    app_bin.entry_point_id = Some(0);

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Runtime context construction should succeed");
    let mut machine = VirtualMachine::new(context);

    let result = machine.run().expect("VM run should succeed");
    assert_eq!(result, Value::I32(42));

    let telemetry = machine.telemetry();
    assert!(
        telemetry.quickened_instructions_count >= 1,
        "quickened count was {}",
        telemetry.quickened_instructions_count
    );
}
