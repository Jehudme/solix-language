//! Unit and integration tests for Phase 5: Static Memory Partitioning & Synthesized Module Initializer (@init).

use solix_common::bytecode::{CompiledFunction, OpCode};
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_compilation::phases::emitter::{
    ConstantPoolBuilder, Emitter, RegisterAllocator, StaticLayout, StaticVariableDecl,
    disassemble_function, synthesize_module_init,
};
use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_synthesize_module_init_sequence() {
    let mut layout = StaticLayout::new();
    let slot_count = layout.assign_static("solix::models::User::count");
    let slot_limit = layout.assign_static("solix::models::User::limit");

    assert_eq!(slot_count, 0);
    assert_eq!(slot_limit, 1);
    assert_eq!(layout.static_space_size, 2);

    let mut pool = ConstantPoolBuilder::new();

    // 1. count = 100
    let decl_count = StaticVariableDecl {
        symbol: "solix::models::User::count".to_string(),
        span: make_span(0, 20),
        initializer: Some(Expression::Literal {
            span: make_span(10, 13),
            value: LiteralValue::Int(100),
        }),
    };

    // 2. limit = 200
    let decl_limit = StaticVariableDecl {
        symbol: "solix::models::User::limit".to_string(),
        span: make_span(25, 45),
        initializer: Some(Expression::Literal {
            span: make_span(35, 38),
            value: LiteralValue::Int(200),
        }),
    };

    let variables = vec![decl_count, decl_limit];
    let init_func = synthesize_module_init(&variables, &layout, &mut pool, None, None, None, None)
        .expect("synthesize @init");

    assert_eq!(init_func.symbol, "@init");
    assert_eq!(init_func.param_count, 0);

    // Verify SET_STATIC instructions
    let mut set_static_slots = Vec::new();
    let mut has_return = false;

    for (idx, inst) in init_func.instructions.iter().enumerate() {
        if let Ok(op) = inst.opcode(idx) {
            if op == OpCode::SetStatic {
                set_static_slots.push(inst.imm32());
            } else if op == OpCode::Return {
                has_return = true;
            }
        }
    }

    assert_eq!(set_static_slots, vec![0, 1]);
    assert!(has_return, "@init must end with RETURN");

    // Test disassembly of @init
    let compiled_pool = pool.finish();
    let disasm = disassemble_function(&init_func, &compiled_pool);
    assert!(disasm.contains(".function @init"));
    assert!(disasm.contains("SET_STATIC"));
    assert!(disasm.contains("RETURN"));
}

#[test]
fn test_intra_module_static_get_and_set() {
    let mut layout = StaticLayout::new();
    let slot = layout.assign_static("User::counter");
    assert_eq!(slot, 0);

    let mut func = CompiledFunction::new("test_static_access()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["User counter".to_string()];

    // Span for "User" is (0, 4), span for "counter" is (5, 12)
    let user_span = make_span(0, 4);
    let counter_span = make_span(5, 12);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_static_layout(&layout);

    // 1. Read: User::counter
    let expr_read = Expression::MemberAccess {
        span: make_span(0, 12),
        object: Box::new(Expression::Identifier { span: user_span }),
        member_span: counter_span,
        is_scope_resolution: true,
    };

    let read_dst = emitter
        .emit_expression(&expr_read, None)
        .expect("emit static read");
    let get_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(get_inst.opcode(0).unwrap(), OpCode::GetStatic);
    assert_eq!(get_inst.dst(), read_dst);
    assert_eq!(get_inst.imm32(), 0);

    // 2. Write: User::counter = 42
    let expr_assign = Expression::Assignment {
        span: make_span(0, 20),
        target: Box::new(expr_read),
        operator: TokenKind::OperatorAssign,
        value: Box::new(Expression::Literal {
            span: make_span(15, 17),
            value: LiteralValue::Int(42),
        }),
    };

    emitter
        .emit_expression(&expr_assign, None)
        .expect("emit static write");
    let mut has_set_static = false;
    for inst in &emitter.function.instructions {
        if let Ok(OpCode::SetStatic) = inst.opcode(0) {
            has_set_static = true;
            assert_eq!(inst.imm32(), 0); // target slot
        }
    }
    assert!(
        has_set_static,
        "Must emit SET_STATIC for static field assignment"
    );
}

#[test]
fn test_cross_module_static_get_global() {
    let layout = StaticLayout::new(); // Empty local layout
    let mut func = CompiledFunction::new("test_cross_module()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["Config TIMEOUT".to_string()];

    let config_span = make_span(0, 6);
    let timeout_span = make_span(7, 14);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_static_layout(&layout);

    // Read: Config::TIMEOUT (not present in local layout)
    let expr_global = Expression::MemberAccess {
        span: make_span(0, 14),
        object: Box::new(Expression::Identifier { span: config_span }),
        member_span: timeout_span,
        is_scope_resolution: true,
    };

    let dst = emitter
        .emit_expression(&expr_global, None)
        .expect("emit global static read");
    let inst = emitter.function.instructions.last().unwrap();
    assert_eq!(inst.opcode(0).unwrap(), OpCode::GetStaticGlobal);
    assert_eq!(inst.dst(), dst);
}
