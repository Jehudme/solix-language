//! Comprehensive unit and integration tests for Phase 1: Instruction representation and SLXB container.

use solix_common::bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ConstantPool, ConstantPoolValue,
    DeclarationModifier, ExceptionHandlerEntry, ExportManifest, ExportedClass, ExportedField,
    ExportedFunction, ExportedGenericTemplate, ExportedParameter, ExportedStatic, Instruction,
    MAGIC, OpCode, VERSION_MAJOR, extract_api_manifest,
};
use solix_common::diagnostics::EmissionError;
use solix_common::span::Span;
use solix_common::types::{PrimitiveKind, Type};

#[test]
fn test_format_a_3_register_packing_and_unpacking() {
    // Opcode, flags, dst, src1, src2
    let inst = Instruction::format_a(OpCode::AddI32, 0x42, 12, 34, 56);

    assert_eq!(inst.opcode(0).expect("valid opcode"), OpCode::AddI32);
    assert_eq!(inst.raw_opcode(), OpCode::AddI32 as u8);
    assert_eq!(inst.flags(), 0x42);
    assert_eq!(inst.dst(), 12);
    assert_eq!(inst.src1(), 34);
    assert_eq!(inst.src2(), 56);

    // Test register boundary limits: 0, 255, 256, 65535
    let inst_boundary = Instruction::format_a(OpCode::MulF64, 0xFF, 0, 256, u16::MAX);
    assert_eq!(
        inst_boundary.opcode(0).expect("valid opcode"),
        OpCode::MulF64
    );
    assert_eq!(inst_boundary.flags(), 0xFF);
    assert_eq!(inst_boundary.dst(), 0);
    assert_eq!(inst_boundary.src1(), 256);
    assert_eq!(inst_boundary.src2(), 65535);

    // Endianness byte round-trip
    let bytes = inst_boundary.to_le_bytes();
    let decoded = Instruction::from_le_bytes(bytes);
    assert_eq!(inst_boundary, decoded);
}

#[test]
fn test_format_b_register_immediate_packing_and_unpacking() {
    // Test unsigned 32-bit immediate
    let inst_u32 = Instruction::format_b(OpCode::LoadConst, 0, 10, 0xDEADBEEF);
    assert_eq!(inst_u32.opcode(0).expect("valid opcode"), OpCode::LoadConst);
    assert_eq!(inst_u32.dst(), 10);
    assert_eq!(inst_u32.imm32(), 0xDEADBEEF);

    // Test signed 32-bit immediate boundaries (i32::MIN, -1, 0, 1, i32::MAX)
    let test_cases = [i32::MIN, -100, -1, 0, 1, 100, i32::MAX];
    for &imm in &test_cases {
        let inst = Instruction::format_b_signed(OpCode::LoadI32, 0, 42, imm);
        assert_eq!(inst.opcode(0).expect("valid opcode"), OpCode::LoadI32);
        assert_eq!(inst.dst(), 42);
        assert_eq!(inst.imm32_signed(), imm);
    }
}

#[test]
fn test_format_c_branch_and_jump_packing_and_unpacking() {
    // Test backward jumps (loops) and forward jumps (if-else branches)
    let jump_offsets = [
        -65536,
        -1024,
        -8,
        -1,
        0,
        1,
        8,
        1024,
        65536,
        i32::MAX,
        i32::MIN,
    ];
    for &offset in &jump_offsets {
        let inst = Instruction::format_c(OpCode::JumpIfFalse, 0x01, 7, offset);
        assert_eq!(inst.opcode(0).expect("valid opcode"), OpCode::JumpIfFalse);
        assert_eq!(inst.flags(), 0x01);
        assert_eq!(inst.cond_reg(), 7);
        assert_eq!(inst.jump_offset(), offset);
    }
}

#[test]
fn test_format_call_windowed_packing_and_unpacking() {
    // dst: 5, target_id: 128, arg_base: 16, arg_count: 4
    let inst = Instruction::format_call(OpCode::CallDirect, 5, 128, 16, 4);
    assert_eq!(inst.opcode(0).expect("valid opcode"), OpCode::CallDirect);
    assert_eq!(inst.call_dst(), 5);
    assert_eq!(inst.call_target(), 128);
    assert_eq!(inst.arg_base(), 16);
    assert_eq!(inst.arg_count(), 4);

    // Test max boundaries: dst=u16::MAX, target=u16::MAX, arg_base=u16::MAX, arg_count=u8::MAX
    let max_call =
        Instruction::format_call(OpCode::CallVirtual, u16::MAX, u16::MAX, u16::MAX, u8::MAX);
    assert_eq!(max_call.call_dst(), u16::MAX);
    assert_eq!(max_call.call_target(), u16::MAX);
    assert_eq!(max_call.arg_base(), u16::MAX);
    assert_eq!(max_call.arg_count(), u8::MAX);
}

#[test]
fn test_instruction_formatting_and_display() {
    let call_inst = Instruction::format_call(OpCode::CallDirect, 2, 10, 4, 3);
    let jump_inst = Instruction::format_c(OpCode::JumpIfTrue, 0, 1, 24);
    let load_inst = Instruction::format_b(OpCode::LoadI32, 0, 5, 100);

    let call_str = format!("{call_inst}");
    let jump_str = format!("{jump_inst}");
    let load_str = format!("{load_inst}");

    assert!(call_str.contains("CALL_DIRECT"));
    assert!(call_str.contains("r2"));
    assert!(jump_str.contains("JUMP_IF_TRUE"));
    assert!(jump_str.contains("+24"));
    assert!(load_str.contains("LOAD_I32"));
    assert!(load_str.contains("100"));
}

#[test]
fn test_invalid_opcode_returns_diagnostic_error() {
    let invalid_instruction = Instruction(0xFE); // 0xFE is not an assigned opcode
    let err = invalid_instruction.opcode(42);
    assert!(err.is_err());
    match err {
        Err(EmissionError::InvalidOpcode {
            raw_byte,
            instruction_index,
        }) => {
            assert_eq!(raw_byte, 0xFE);
            assert_eq!(instruction_index, 42);
        }
        other => panic!("Expected InvalidOpcode error, got {other:?}"),
    }
}

#[test]
fn test_constant_pool_deduplication() {
    let mut pool = ConstantPool::new();

    let idx1 = pool.add_i64(100).expect("add i64");
    let idx2 = pool.add_i64(200).expect("add i64");
    let idx3 = pool.add_i64(100).expect("dedup i64");
    assert_eq!(idx1, idx3);
    assert_ne!(idx1, idx2);

    let s1 = pool.add_string("hello").expect("add str");
    let s2 = pool.add_string("world").expect("add str");
    let s3 = pool.add_string("hello").expect("dedup str");
    assert_eq!(s1, s3);
    assert_ne!(s1, s2);

    let f1 = pool.add_f64(42.125).expect("add f64");
    let f2 = pool.add_f64(42.125).expect("dedup f64");
    assert_eq!(f1, f2);

    assert_eq!(pool.len(), 5); // 100, 200, "hello", "world", 42.125
    assert_eq!(
        pool.get(s1 as usize),
        Some(&ConstantPoolValue::String("hello".to_string()))
    );
}

#[test]
fn test_slxb_binary_container_serialization_round_trip() {
    let mut program = BinaryProgram::new();
    program.flags = 0x01;
    program.entry_point_id = Some(1);
    program.entry_point_symbol = Some("solix::main()".to_string());
    program.static_space_size = 16;
    program
        .static_symbols_map
        .push(("solix::global_counter".to_string(), 0));
    program
        .static_symbols_map
        .push(("solix::models::User::MAX_COUNT".to_string(), 1));

    // Constant pool entries
    program.constant_pool.add_i64(42).unwrap();
    program.constant_pool.add_f64(100.5).unwrap();
    program
        .constant_pool
        .add_string("Welcome to Solix!")
        .unwrap();

    // Compiled Class
    let mut user_class = CompiledClass::new("solix::models::User", 0, None, 32);
    user_class.reference_field_offsets = vec![8, 16];
    user_class.vtable = vec![1, 2];
    user_class.itable = vec![(10, vec![1]), (20, vec![2])];
    user_class.destructor_id = Some(3);
    program.classes.push(user_class);

    // Compiled Functions
    // Function 0: @init
    let mut init_func = CompiledFunction::new("@init", 0, 0, 2);
    init_func.emit(
        Instruction::format_b(OpCode::LoadI32, 0, 0, 100),
        Some(Span {
            id: 0,
            start: 0,
            end: 10,
        }),
    );
    init_func.emit(
        Instruction::format_b(OpCode::SetStatic, 0, 0, 0),
        Some(Span {
            id: 0,
            start: 11,
            end: 25,
        }),
    );
    init_func.emit(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
    program.functions.push(init_func);

    // Function 1: main
    let mut main_func = CompiledFunction::new("solix::main()", 0, 0, 8);
    main_func.reference_registers = vec![1, 4];
    main_func.exception_handlers.push(ExceptionHandlerEntry {
        try_start_pc: 1,
        try_end_pc: 3,
        catch_type_symbol: Some("std::Exception".to_string()),
        catch_reg: 2,
        handler_pc: 4,
        finally_pc: Some(6),
        try_ref_regs: vec![1],
    });
    main_func.emit(
        Instruction::format_call(OpCode::CallDirect, 0, 2, 1, 1),
        None,
    );
    main_func.emit(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0), None);
    program.functions.push(main_func);

    // Round-trip serialization
    let serialized_bytes = program.to_bytes();
    assert!(!serialized_bytes.is_empty());
    assert_eq!(&serialized_bytes[0..4], MAGIC);

    let deserialized_program =
        BinaryProgram::from_bytes(&serialized_bytes).expect("successful deserialization");

    // Exact equality check
    assert_eq!(program.magic, deserialized_program.magic);
    assert_eq!(program.version_major, deserialized_program.version_major);
    assert_eq!(program.version_minor, deserialized_program.version_minor);
    assert_eq!(program.flags, deserialized_program.flags);
    assert_eq!(program.entry_point_id, deserialized_program.entry_point_id);
    assert_eq!(
        program.entry_point_symbol,
        deserialized_program.entry_point_symbol
    );
    assert_eq!(
        program.static_space_size,
        deserialized_program.static_space_size
    );
    assert_eq!(
        program.static_symbols_map,
        deserialized_program.static_symbols_map
    );
    assert_eq!(program.constant_pool, deserialized_program.constant_pool);
    assert_eq!(program.classes, deserialized_program.classes);
    assert_eq!(program.functions, deserialized_program.functions);
}

#[test]
fn test_corrupted_magic_header_rejection() {
    let program = BinaryProgram::new();
    let mut bytes = program.to_bytes();
    bytes[0] = b'X'; // Corrupt header magic

    let result = BinaryProgram::from_bytes(&bytes);
    assert!(result.is_err());
    match result {
        Err(EmissionError::InvalidMagic { found }) => {
            assert_eq!(found[0], b'X');
        }
        other => panic!("Expected InvalidMagic error, got {other:?}"),
    }
}

#[test]
fn test_unsupported_version_rejection() {
    let program = BinaryProgram::new();
    let mut bytes = program.to_bytes();
    // Overwrite major version at offset 4..6 to 999
    bytes[4..6].copy_from_slice(&999u16.to_le_bytes());

    let result = BinaryProgram::from_bytes(&bytes);
    assert!(result.is_err());
    match result {
        Err(EmissionError::UnsupportedVersion {
            expected_major,
            found_major,
            ..
        }) => {
            assert_eq!(expected_major, VERSION_MAJOR);
            assert_eq!(found_major, 999);
        }
        other => panic!("Expected UnsupportedVersion error, got {other:?}"),
    }
}

#[test]
fn test_truncated_buffer_unexpected_eof_handling() {
    let program = BinaryProgram::new();
    let bytes = program.to_bytes();

    // Truncate to just 6 bytes (incomplete header)
    let truncated = &bytes[0..6];
    let result = BinaryProgram::from_bytes(truncated);
    assert!(result.is_err());
    match result {
        Err(EmissionError::UnexpectedEof { section, .. }) => {
            assert_eq!(section, "Header");
        }
        other => panic!("Expected UnexpectedEof error, got {other:?}"),
    }
}

#[test]
fn test_export_manifest_json_extraction() {
    let mut program = BinaryProgram::new();
    program.package_name = "math_lib".to_string();
    program.package_version = "1.2.0".to_string();
    program.package_description = "Standard mathematics and linear algebra library".to_string();

    let exported_func = ExportedFunction {
        symbol: "math_lib::add/2".to_string(),
        simple_name: "add".to_string(),
        modifiers: vec![DeclarationModifier::Public, DeclarationModifier::Inline],
        parameters: vec![
            ExportedParameter {
                name: "a".to_string(),
                param_type: Type::Primitive(PrimitiveKind::Int32),
            },
            ExportedParameter {
                name: "b".to_string(),
                param_type: Type::Primitive(PrimitiveKind::Int32),
            },
        ],
        return_type: Type::Primitive(PrimitiveKind::Int32),
        is_constructor: false,
        is_destructor: false,
    };

    let exported_static = ExportedStatic {
        symbol: "math_lib::PI".to_string(),
        name: "PI".to_string(),
        var_type: Type::Primitive(PrimitiveKind::Float64),
        modifiers: vec![
            DeclarationModifier::Public,
            DeclarationModifier::Static,
            DeclarationModifier::Const,
        ],
        slot: 0,
    };

    let exported_class = ExportedClass {
        symbol: "math_lib::Vector2D".to_string(),
        name: "Vector2D".to_string(),
        modifiers: vec![DeclarationModifier::Public],
        is_interface: false,
        base_class: None,
        interfaces: vec!["math_lib::IPrintable".to_string()],
        generic_parameters: Vec::new(),
        fields: vec![
            ExportedField {
                name: "x".to_string(),
                field_type: Type::Primitive(PrimitiveKind::Float64),
                modifiers: vec![DeclarationModifier::Public],
                offset: 8,
            },
            ExportedField {
                name: "y".to_string(),
                field_type: Type::Primitive(PrimitiveKind::Float64),
                modifiers: vec![DeclarationModifier::Public],
                offset: 16,
            },
        ],
        methods: vec![ExportedFunction {
            symbol: "math_lib::Vector2D::magnitude/0".to_string(),
            simple_name: "magnitude".to_string(),
            modifiers: vec![DeclarationModifier::Public],
            parameters: Vec::new(),
            return_type: Type::Primitive(PrimitiveKind::Float64),
            is_constructor: false,
            is_destructor: false,
        }],
    };

    let generic_template = ExportedGenericTemplate {
        name: "Container".to_string(),
        parameter_names: vec!["T".to_string()],
        template_source: "class Container<T> { public T item; }".to_string(),
    };

    program.export_manifest = ExportManifest {
        package_name: program.package_name.clone(),
        package_version: program.package_version.clone(),
        package_description: program.package_description.clone(),
        dependencies: Vec::new(),
        exported_classes: vec![exported_class],
        exported_functions: vec![exported_func],
        exported_statics: vec![exported_static],
        generic_templates: vec![generic_template],
    };

    let manifest_json = extract_api_manifest(&program).expect("json extraction succeeds");
    assert!(manifest_json.contains("\"package_name\": \"math_lib\""));
    assert!(manifest_json.contains("\"package_version\": \"1.2.0\""));
    assert!(manifest_json.contains("\"simple_name\": \"add\""));
    assert!(manifest_json.contains("\"name\": \"Vector2D\""));
    assert!(manifest_json.contains("\"name\": \"Container\""));
    assert!(
        manifest_json.contains("\"template_source\": \"class Container<T> { public T item; }\"")
    );
}

#[test]
fn test_binary_program_manifest_round_trip() {
    let mut program = BinaryProgram::new();
    program.package_name = "network".to_string();
    program.package_version = "2.0.1".to_string();
    program.package_description = "Networking library".to_string();

    program.export_manifest = ExportManifest {
        package_name: "network".to_string(),
        package_version: "2.0.1".to_string(),
        package_description: "Networking library".to_string(),
        dependencies: Vec::new(),
        exported_classes: Vec::new(),
        exported_functions: vec![ExportedFunction {
            symbol: "network::connect/1".to_string(),
            simple_name: "connect".to_string(),
            modifiers: vec![DeclarationModifier::Public],
            parameters: vec![ExportedParameter {
                name: "url".to_string(),
                param_type: Type::Primitive(PrimitiveKind::Int32),
            }],
            return_type: Type::Primitive(PrimitiveKind::Bool),
            is_constructor: false,
            is_destructor: false,
        }],
        exported_statics: Vec::new(),
        generic_templates: Vec::new(),
    };

    let bytes = program.to_bytes();
    let deserialized =
        BinaryProgram::from_bytes(&bytes).expect("round-trip deserialization succeeds");

    assert_eq!(deserialized.package_name, "network");
    assert_eq!(deserialized.package_version, "2.0.1");
    assert_eq!(deserialized.package_description, "Networking library");
    assert_eq!(deserialized.export_manifest, program.export_manifest);
}
