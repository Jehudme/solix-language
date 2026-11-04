//! Unit tests for Object model, fields, dynamic vtables, and instanceof checks.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::FLAG_WEAK;
use solix_common::bytecode::{BinaryProgram, CompiledClass, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_alloc_object_and_field_mutation() {
    let mut binary = BinaryProgram::new();
    let class = CompiledClass::new("Point", 1, None, 24);
    binary.classes.push(class);

    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = AllocObject(vtable_id=1, size=24)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 1, 24));
    // r[1] = 42
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 42));
    // SetField(r_val=1, r_obj=0, offset=8)
    func.instructions
        .push(Instruction::format_a(OpCode::SetField, 0, 1, 0, 8));
    // r[2] = 88
    func.instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 88));
    // SetField(r_val=2, r_obj=0, offset=16)
    func.instructions
        .push(Instruction::format_a(OpCode::SetField, 0, 2, 0, 16));

    // r[3] = GetField(r_obj=0, offset=8)
    func.instructions
        .push(Instruction::format_a(OpCode::GetField, 0, 3, 0, 8));
    // r[4] = GetField(r_obj=0, offset=16)
    func.instructions
        .push(Instruction::format_a(OpCode::GetField, 0, 4, 0, 16));
    // r[5] = r[3] + r[4] (130)
    func.instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 5, 3, 4));
    // ReturnValue(r[5])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 5, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(130));
}

#[test]
fn test_weak_vs_strong_field_reference_counting() {
    let mut binary = BinaryProgram::new();
    let class = CompiledClass::new("Node", 1, None, 24);
    binary.classes.push(class);

    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = parent object
    func.instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 1, 24));
    // r[1] = child object
    func.instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 1, 1, 24));
    // Strong SetField(r_val=1, r_obj=0, offset=8): child ref_count increments to 2
    func.instructions
        .push(Instruction::format_a(OpCode::SetField, 0, 1, 0, 8));
    // Weak SetField(r_val=1, r_obj=0, offset=16, FLAG_WEAK): child ref_count stays 2
    func.instructions
        .push(Instruction::format_a(OpCode::SetField, FLAG_WEAK, 1, 0, 16));
    // ReturnValue(r[1])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    let child_addr = res.as_ref().unwrap();

    // Verify child ref count is 2 (initial 1 + 1 strong ref, weak ref did not increment)
    let header = vm.heap.read_header(child_addr).unwrap();
    assert_eq!(header.ref_count, 2);
}

#[test]
fn test_dynamic_vtable_dispatch_call_virtual() {
    let mut binary = BinaryProgram::new();

    // Function 0: Shape::area(this) -> returns 0
    let mut shape_area = CompiledFunction::new("Shape::area", 0, 1, 2);
    shape_area
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    shape_area
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    // Function 1: Circle::area(this) -> returns 314
    let mut circle_area = CompiledFunction::new("Circle::area", 0, 1, 2);
    circle_area
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 314));
    circle_area
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    // Class 1: Shape (vtable: [shape_area])
    let mut shape_class = CompiledClass::new("Shape", 1, None, 16);
    shape_class.vtable.push(0);

    // Class 2: Circle (inherits Shape, vtable: [circle_area])
    let mut circle_class = CompiledClass::new("Circle", 2, Some(1), 16);
    circle_class.vtable.push(1);

    binary.classes.push(shape_class);
    binary.classes.push(circle_class);
    binary.functions.push(shape_area);
    binary.functions.push(circle_area);

    // Function 2: main() -> instantiates Circle, calls CallVirtual(slot 0)
    let mut main_func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = AllocObject(Circle, vtable_id=2)
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 2, 16));
    // r[1] = CallVirtual(slot=0, dst=1, arg_base=0, count=1)
    main_func
        .instructions
        .push(Instruction::format_call(OpCode::CallVirtual, 1, 0, 0, 1));
    // ReturnValue(r[1])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.functions.push(main_func);
    binary.entry_point_id = Some(2);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(314));
}

#[test]
fn test_instanceof_and_cast_check() {
    let mut binary = BinaryProgram::new();

    // Animal (vtable 1) -> Dog (vtable 2, base 1) -> Puppy (vtable 3, base 2)
    let animal_class = CompiledClass::new("Animal", 1, None, 16);
    let dog_class = CompiledClass::new("Dog", 2, Some(1), 16);
    let puppy_class = CompiledClass::new("Puppy", 3, Some(2), 16);

    binary.classes.push(animal_class);
    binary.classes.push(dog_class);
    binary.classes.push(puppy_class);

    let mut func = CompiledFunction::new("main", 0, 0, 8);
    // r[0] = AllocObject(Puppy, vtable 3)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 3, 16));
    // r[1] = Instanceof(r_obj=0, vtable 3) (Puppy is Puppy -> true)
    func.instructions
        .push(Instruction::format_a(OpCode::Instanceof, 0, 1, 0, 3));
    // r[2] = Instanceof(r_obj=0, vtable 1) (Puppy is Animal -> true)
    func.instructions
        .push(Instruction::format_a(OpCode::Instanceof, 0, 2, 0, 1));
    // r[3] = Instanceof(r_obj=0, vtable 99) (Puppy is unknown -> false)
    func.instructions
        .push(Instruction::format_a(OpCode::Instanceof, 0, 3, 0, 99));
    // r[4] = CastCheck(r_obj=0, vtable 1) (Puppy cast to Animal -> Ok)
    func.instructions
        .push(Instruction::format_a(OpCode::CastCheck, 0, 4, 0, 1));
    // ReturnValue(r[2])
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::Bool(true));
}

#[test]
fn test_cast_check_failure_throws_runtime_error() {
    let mut binary = BinaryProgram::new();
    let class_a = CompiledClass::new("A", 1, None, 16);
    let class_b = CompiledClass::new("B", 2, None, 16);
    binary.classes.push(class_a);
    binary.classes.push(class_b);

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = AllocObject(A, vtable 1)
    func.instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 1, 16));
    // r[1] = CastCheck(r[0], vtable 2) -> fails! A is not B
    func.instructions
        .push(Instruction::format_a(OpCode::CastCheck, 0, 1, 0, 2));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run();
    assert!(matches!(res, Err(RuntimeError::InvalidCast { .. })));
}
