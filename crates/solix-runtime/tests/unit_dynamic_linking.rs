//! Unit tests for dynamic linking opcodes and in-place instruction quickening.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledClass, CompiledFunction, Instruction, OpCode};
use solix_runtime::context::RuntimeContext;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::Value;
use solix_runtime::vm::VirtualMachine;

#[test]
fn test_call_extern_quickens_to_call_direct() {
    // Binary 1: Library with exported function "math::add"
    let mut lib_bin = BinaryProgram::new();
    let mut add_func = CompiledFunction::new("math::add", 0, 2, 4);
    // r[2] = r[0] + r[1]
    add_func
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
    // ReturnValue(r[2])
    add_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    lib_bin.functions.push(add_func);

    // Binary 2: Consumer calling "math::add" via CallExtern
    let mut app_bin = BinaryProgram::new();
    let sym_idx = app_bin.constant_pool.add_string("math::add").unwrap();
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = 10
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
    // r[1] = 20
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 20));
    // r[2] = CallExtern("math::add", dst=2, arg_base=0, count=2)
    main_func.instructions.push(Instruction::format_call(
        OpCode::CallExtern,
        2,
        sym_idx,
        0,
        2,
    ));
    // ReturnValue(r[2])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    app_bin.functions.push(main_func);
    app_bin.entry_point_id = Some(0);

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context construction failed");

    // Before execution, main's 3rd instruction must be CallExtern
    let main_global_id = *context.symbol_to_function_id.get("main").unwrap();
    assert_eq!(
        context.all_functions[main_global_id as usize].instructions[2]
            .opcode(0)
            .unwrap(),
        OpCode::CallExtern
    );

    let mut vm = VirtualMachine::new(context);
    let result = vm.run().expect("Execution failed");
    assert_eq!(result, Value::I32(30));
    assert!(vm.verify_clean_memory().is_ok());

    // After execution, the instruction must be quickened in-place to CallDirect
    let quickened_inst = vm.context.all_functions[main_global_id as usize].instructions[2];
    assert_eq!(quickened_inst.opcode(0).unwrap(), OpCode::CallDirect);
    assert_eq!(quickened_inst.call_target(), 0); // math::add was at index 0
}

#[test]
fn test_get_and_set_static_global_quickening() {
    // Binary 1: Library defining exported static variable "config::MAX_RETRIES"
    let mut lib_bin = BinaryProgram::new();
    lib_bin.static_space_size = 1;
    lib_bin
        .static_symbols_map
        .push(("config::MAX_RETRIES".to_string(), 0));

    // Binary 2: Consumer modifying and reading "config::MAX_RETRIES"
    let mut app_bin = BinaryProgram::new();
    let sym_idx = app_bin
        .constant_pool
        .add_string("config::MAX_RETRIES")
        .unwrap();
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // r[0] = 42
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 42));
    // SetStaticGlobal(dst=0, sym_idx)
    main_func.instructions.push(Instruction::format_b(
        OpCode::SetStaticGlobal,
        0,
        0,
        sym_idx as u32,
    ));
    // GetStaticGlobal(dst=1, sym_idx)
    main_func.instructions.push(Instruction::format_b(
        OpCode::GetStaticGlobal,
        0,
        1,
        sym_idx as u32,
    ));
    // ReturnValue(r[1])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    app_bin.functions.push(main_func);
    app_bin.entry_point_id = Some(0);

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context construction failed");
    let main_global_id = *context.symbol_to_function_id.get("main").unwrap();

    let mut vm = VirtualMachine::new(context);
    let result = vm.run().expect("Execution failed");
    assert_eq!(result, Value::I32(42));
    assert!(vm.verify_clean_memory().is_ok());

    // Verify SetStaticGlobal quickened to SetStaticDirect
    let set_inst = vm.context.all_functions[main_global_id as usize].instructions[1];
    assert_eq!(set_inst.opcode(0).unwrap(), OpCode::SetStaticDirect);
    assert_eq!(set_inst.imm32(), 0);

    // Verify GetStaticGlobal quickened to GetStaticDirect
    let get_inst = vm.context.all_functions[main_global_id as usize].instructions[2];
    assert_eq!(get_inst.opcode(0).unwrap(), OpCode::GetStaticDirect);
    assert_eq!(get_inst.imm32(), 0);
}

#[test]
fn test_alloc_object_extern_quickens_to_alloc_object() {
    // Binary 1: Library defining class "Point" with vtable and destructor
    let mut lib_bin = BinaryProgram::new();
    let mut point_cls = CompiledClass::new("geom::Point", 0, None, 16);
    let mut dtor_func = CompiledFunction::new("geom::Point::~Point", 0, 1, 2);
    dtor_func
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    lib_bin.functions.push(dtor_func);
    point_cls.destructor_id = Some(0);
    lib_bin.classes.push(point_cls);

    // Binary 2: Consumer allocating "geom::Point" via AllocObjectExtern
    let mut app_bin = BinaryProgram::new();
    let sym_idx = app_bin.constant_pool.add_string("geom::Point").unwrap();
    let mut main_func = CompiledFunction::new("main", 0, 0, 4);
    // AllocObjectExtern(dst=0, sym_idx)
    main_func.instructions.push(Instruction::format_b(
        OpCode::AllocObjectExtern,
        0,
        0,
        sym_idx as u32,
    ));
    // DecRef(r[0]) to trigger clean drop
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::DecRef, 0, 0, 0, 0));
    // r[1] = 100
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 100));
    // ReturnValue(r[1])
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
    app_bin.functions.push(main_func);
    app_bin.entry_point_id = Some(0);

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context construction failed");
    let main_global_id = *context.symbol_to_function_id.get("main").unwrap();

    let mut vm = VirtualMachine::new(context);
    let result = vm.run().expect("Execution failed");
    assert_eq!(result, Value::I32(100));
    assert!(vm.verify_clean_memory().is_ok());

    // Verify AllocObjectExtern quickened to AllocObject
    let alloc_inst = vm.context.all_functions[main_global_id as usize].instructions[0];
    assert_eq!(alloc_inst.opcode(0).unwrap(), OpCode::AllocObject);
    assert_eq!(alloc_inst.src1(), 0); // vtable_id = 0
    assert_eq!(alloc_inst.src2(), 16); // size = 16
}

#[test]
fn test_resolve_function_symbol_distinguishes_external_overloads_by_arity_and_params() {
    let mut lib_bin = BinaryProgram::new();
    lib_bin.package_name = "gfx".to_string();

    // Index 0: 0-arg constructor stored without parens (as emitted by pipeline.rs)
    let ctor_zero = CompiledFunction::new("Vec3::constructor", 0, 1, 2);
    // Index 1: 3-arg float64 constructor
    let ctor_f64 = CompiledFunction::new("Vec3::constructor(float64,float64,float64)", 0, 4, 5);
    // Index 2: 3-arg int32 constructor (registered last!)
    let ctor_i32 = CompiledFunction::new("Vec3::constructor(int32,int32,int32)", 0, 4, 5);
    // Index 3: 0-arg Camera constructor stored without parens
    let cam_zero = CompiledFunction::new("Cam::constructor", 0, 1, 2);
    // Index 4: 5-arg Camera constructor (registered last!)
    let cam_five = CompiledFunction::new("Cam::constructor(Vec3,Vec3,Vec3,float32,int32)", 0, 6, 7);

    lib_bin.functions.push(ctor_zero);
    lib_bin.functions.push(ctor_f64);
    lib_bin.functions.push(ctor_i32);
    lib_bin.functions.push(cam_zero);
    lib_bin.functions.push(cam_five);

    let options = RuntimeOptions::new().with_binary(lib_bin);
    let context = RuntimeContext::new(options).expect("Context construction failed");

    assert_eq!(
        context.resolve_function_symbol("gfx::Vec3::constructor/0"),
        Some(0)
    );
    assert_eq!(
        context.resolve_function_symbol("gfx::Vec3::constructor()"),
        Some(0)
    );
    assert_eq!(
        context.resolve_function_symbol("gfx::Vec3::constructor(float64,float64,float64)"),
        Some(1)
    );
    assert_eq!(
        context.resolve_function_symbol("gfx::Vec3::constructor(int32,int32,int32)"),
        Some(2)
    );
    assert_eq!(
        context.resolve_function_symbol("gfx::Cam::constructor/0"),
        Some(3)
    );
    assert_eq!(
        context.resolve_function_symbol("gfx::Cam::constructor(Vec3,Vec3,Vec3,float32,int32)"),
        Some(4)
    );
}
