//! Unit tests for ARC retain, release, class destructors, and cascade drops.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledClass, CompiledFunction, Instruction, OpCode};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_arc_destructor_execution_on_zero_ref_count() {
    let mut binary = BinaryProgram::new();
    binary.static_space_size = 1;

    // Function 0: ~Resource(this) -> writes 999 to static slot 0
    let mut dtor_func = CompiledFunction::new("Resource::~Resource", 0, 1, 4);
    dtor_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 999));
    dtor_func
        .instructions
        .push(Instruction::format_b(OpCode::SetStatic, 0, 1, 0));
    dtor_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    // Class 1: Resource with destructor function 0
    let mut res_class = CompiledClass::new("Resource", 1, None, 16);
    res_class.destructor_id = Some(0);

    // Function 1: main() -> allocates Resource, decrements ref, asserts destructor ran
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = AllocObject(Resource, 1) -> ref_count = 1
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 1, 16));
    // DecRef(r[0]) -> drops ref_count to 0, triggers destructor!
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::DecRef, 0, 0, 0, 0));
    // r[1] = GetStatic(slot 0) -> should be 999!
    main_func
        .instructions
        .push(Instruction::format_b(OpCode::GetStatic, 0, 1, 0));
    // ReturnValue(r[1])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));

    binary.classes.push(res_class);
    binary.functions.push(dtor_func);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(1);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(999));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_arc_cascade_drop_child_references() {
    let mut binary = BinaryProgram::new();

    // Class 1: Child
    let child_class = CompiledClass::new("Child", 1, None, 16);

    // Class 2: Parent with reference_field_offsets = [8]
    let mut parent_class = CompiledClass::new("Parent", 2, None, 24);
    parent_class.reference_field_offsets.push(8);

    // Function 0: main()
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = Alloc Parent
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 2, 24));
    // r[1] = Alloc Child
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::AllocObject, 0, 1, 1, 16));
    // SetField(r_val=1, r_obj=0, offset=8) -> Parent holds strong ref to Child (Child rc = 2)
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::SetField, 0, 1, 0, 8));
    // DecRef(r[1]) -> Child local dropped (Child rc = 1, owned by Parent)
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::DecRef, 0, 1, 0, 0));
    // DecRef(r[0]) -> Parent dropped (Parent rc = 0 -> drops Child -> Child rc = 0!)
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::DecRef, 0, 0, 0, 0));
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary.classes.push(child_class);
    binary.classes.push(parent_class);
    binary.functions.push(main_func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let _ = vm.run().expect("Execution failed");
    // Both Parent and Child must be completely freed with zero leaks!
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_cycle_collector_symmetric_refcount_restoration() {
    use solix_runtime::memory::cycle_collector::{run_cycle_collection, scan_black};
    use solix_runtime::memory::layout::{BlockTag, GcColor, MAX_REF_COUNT};

    let mut binary = BinaryProgram::new();
    let mut node_class = CompiledClass::new("Node", 1, None, 24);
    node_class.reference_field_offsets = vec![8, 16];
    binary.classes.push(node_class);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("build context");
    let mut vm = VirtualMachine::new(context);

    let root_addr = vm.heap.allocate(24, BlockTag::Object).expect("alloc root");
    let near_max_addr = vm
        .heap
        .allocate(24, BlockTag::Object)
        .expect("alloc near_max");
    let zero_rc_addr = vm
        .heap
        .allocate(24, BlockTag::Object)
        .expect("alloc zero_rc");

    vm.heap.write_u64(root_addr, 1).unwrap();
    vm.heap.write_u64(near_max_addr, 1).unwrap();
    vm.heap.write_u64(zero_rc_addr, 1).unwrap();

    // root holds references to near_max_addr (field +8) and zero_rc_addr (field +16).
    vm.heap
        .write_u64(root_addr + 8, Value::Ref(near_max_addr).to_raw_u64())
        .unwrap();
    vm.heap
        .write_u64(root_addr + 16, Value::Ref(zero_rc_addr).to_raw_u64())
        .unwrap();

    // root has an external reference (ref_count = 2) and is marked Purple.
    let mut root_hdr = vm.heap.read_header(root_addr).unwrap();
    root_hdr.ref_count = 2;
    root_hdr.color = GcColor::Purple;
    vm.heap.write_header(root_addr, root_hdr).unwrap();

    // near_max sits one increment below MAX_REF_COUNT; must NOT latch to MAX_REF_COUNT.
    let mut near_hdr = vm.heap.read_header(near_max_addr).unwrap();
    near_hdr.ref_count = MAX_REF_COUNT - 1;
    near_hdr.color = GcColor::Black;
    vm.heap.write_header(near_max_addr, near_hdr).unwrap();

    // zero_rc starts at 0; mark_gray skips decrementing 0, so scan_black must NOT inflate to 1.
    let mut zero_hdr = vm.heap.read_header(zero_rc_addr).unwrap();
    zero_hdr.ref_count = 0;
    zero_hdr.color = GcColor::Black;
    vm.heap.write_header(zero_rc_addr, zero_hdr).unwrap();

    let (reclaimed, _deferred) =
        run_cycle_collection(&mut vm.heap, &vm.context, &[root_addr], None)
            .expect("cycle collection succeeds");
    assert_eq!(reclaimed, 0);

    let after_near_hdr = vm.heap.read_header(near_max_addr).unwrap();
    assert_eq!(
        after_near_hdr.ref_count,
        MAX_REF_COUNT - 1,
        "MAX_REF_COUNT - 1 must restore symmetrically without latching to immortal MAX_REF_COUNT"
    );

    let after_zero_hdr = vm.heap.read_header(zero_rc_addr).unwrap();
    assert_eq!(
        after_zero_hdr.ref_count, 0,
        "Un-decremented 0-ref child must not inflate to ref_count = 1 during scan_black"
    );

    // Direct standalone scan_black must also never latch MAX_REF_COUNT - 1 into MAX_REF_COUNT.
    scan_black(&mut vm.heap, &vm.context, root_addr).expect("standalone scan_black succeeds");
    assert_eq!(
        vm.heap.read_header(near_max_addr).unwrap().ref_count,
        MAX_REF_COUNT - 1
    );
    assert_eq!(vm.heap.read_header(zero_rc_addr).unwrap().ref_count, 0);
}
