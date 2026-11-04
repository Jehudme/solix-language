//! Unit and integration tests for Phase 7: Arrays & Heterogeneous Tuples.

use solix_common::bytecode::{CompiledFunction, OpCode};
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_common::types::{PrimitiveKind, Type};
use solix_compilation::phases::emitter::{ConstantPoolBuilder, Emitter, RegisterAllocator};
use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};
use std::collections::HashMap;

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_array_creation_and_literal_emission() {
    let mut func = CompiledFunction::new("test_arrays()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // 1. Dynamic allocation: new int32[10]
    let expr_creation = Expression::ArrayCreation {
        span: make_span(0, 15),
        element_type:
            solix_compilation::phases::parser::statements::types::TypeAnnotation::Primitive {
                span: make_span(4, 9),
                kind: TokenKind::PrimitiveInt32,
            },
        size: Box::new(Expression::Literal {
            span: make_span(10, 12),
            value: LiteralValue::Int(10),
        }),
    };

    let dst_alloc = emitter
        .emit_expression(&expr_creation, None)
        .expect("emit array creation");
    let alloc_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(alloc_inst.opcode(0).unwrap(), OpCode::AllocArray);
    assert_eq!(alloc_inst.dst(), dst_alloc);

    // 2. Inline literal: [100, 200, 300]
    let expr_literal = Expression::ArrayLiteral {
        span: make_span(20, 35),
        elements: vec![
            Expression::Literal {
                span: make_span(21, 24),
                value: LiteralValue::Int(100),
            },
            Expression::Literal {
                span: make_span(26, 29),
                value: LiteralValue::Int(200),
            },
            Expression::Literal {
                span: make_span(31, 34),
                value: LiteralValue::Int(300),
            },
        ],
    };

    let dst_lit = emitter
        .emit_expression(&expr_literal, None)
        .expect("emit array literal");

    // Verify SET_ARRAY was emitted 3 times
    let mut set_array_count = 0;
    for inst in &emitter.function.instructions {
        if let Ok(OpCode::SetArray) = inst.opcode(0) {
            set_array_count += 1;
            assert_eq!(inst.dst(), dst_lit);
        }
    }
    assert_eq!(set_array_count, 3);
}

#[test]
fn test_array_access_read_and_write() {
    let mut func = CompiledFunction::new("test_arr_access()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["arr".to_string()];

    let arr_span = make_span(0, 3);

    allocator.enter_scope();
    let arr_reg = allocator.alloc_local("arr").unwrap();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // 1. Read: arr[2]
    let expr_read = Expression::ArrayAccess {
        span: make_span(0, 6),
        array: Box::new(Expression::Identifier { span: arr_span }),
        index: Box::new(Expression::Literal {
            span: make_span(4, 5),
            value: LiteralValue::Int(2),
        }),
    };

    let dst_read = emitter
        .emit_expression(&expr_read, None)
        .expect("emit arr[2]");
    let get_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(get_inst.opcode(0).unwrap(), OpCode::GetArray);
    assert_eq!(get_inst.dst(), dst_read);
    assert_eq!(get_inst.src1(), arr_reg);

    // 2. Write: arr[2] = 42
    let expr_assign = Expression::Assignment {
        span: make_span(0, 12),
        target: Box::new(expr_read),
        operator: TokenKind::OperatorAssign,
        value: Box::new(Expression::Literal {
            span: make_span(9, 11),
            value: LiteralValue::Int(42),
        }),
    };

    emitter
        .emit_expression(&expr_assign, None)
        .expect("emit arr[2] = 42");

    let mut has_set_array = false;
    for inst in &emitter.function.instructions {
        if let Ok(OpCode::SetArray) = inst.opcode(0) {
            has_set_array = true;
            assert_eq!(inst.dst(), arr_reg); // array reference
        }
    }
    assert!(has_set_array);
}

#[test]
fn test_array_length_property() {
    let mut func = CompiledFunction::new("test_length()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["arr length".to_string()];

    let arr_span = make_span(0, 3);
    let length_span = make_span(4, 10);

    allocator.enter_scope();
    let arr_reg = allocator.alloc_local("arr").unwrap();

    let mut node_types = HashMap::new();
    node_types.insert(
        arr_span,
        Type::Array {
            element_type: Box::new(Type::Primitive(PrimitiveKind::Int32)),
            depth: 1,
        },
    );

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types);

    // arr.length
    let expr_length = Expression::MemberAccess {
        span: make_span(0, 10),
        object: Box::new(Expression::Identifier { span: arr_span }),
        member_span: length_span,
        is_scope_resolution: false,
    };

    let dst = emitter
        .emit_expression(&expr_length, None)
        .expect("emit arr.length");
    let inst = emitter.function.instructions.last().unwrap();
    assert_eq!(inst.opcode(0).unwrap(), OpCode::ArrayLength);
    assert_eq!(inst.dst(), dst);
    assert_eq!(inst.src1(), arr_reg);
}

#[test]
fn test_heterogeneous_tuple_creation_and_indexing() {
    let mut func = CompiledFunction::new("test_tuple()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["t 0 1".to_string()];

    let t_span = make_span(0, 1);
    let zero_span = make_span(2, 3);
    let one_span = make_span(4, 5);

    allocator.enter_scope();
    let t_reg = allocator.alloc_local("t").unwrap();

    let mut node_types = HashMap::new();
    node_types.insert(
        t_span,
        Type::Tuple(vec![
            Type::Primitive(PrimitiveKind::Int32),
            Type::Primitive(PrimitiveKind::Bool),
        ]),
    );

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types);

    // 1. Tuple creation: tuple(10, true)
    let expr_tuple = Expression::TupleCreation {
        span: make_span(0, 15),
        elements: vec![
            Expression::Literal {
                span: make_span(6, 8),
                value: LiteralValue::Int(10),
            },
            Expression::Literal {
                span: make_span(10, 14),
                value: LiteralValue::Bool(true),
            },
        ],
    };

    let dst_tuple = emitter
        .emit_expression(&expr_tuple, None)
        .expect("emit tuple(10, true)");
    let alloc_tuple_inst = emitter.function.instructions[0];
    assert_eq!(alloc_tuple_inst.opcode(0).unwrap(), OpCode::AllocTuple);
    assert_eq!(alloc_tuple_inst.dst(), dst_tuple);
    assert_eq!(alloc_tuple_inst.src1(), 2); // 2 elements

    // 2. Indexing: t.0 and t.1
    let expr_t0 = Expression::MemberAccess {
        span: make_span(0, 3),
        object: Box::new(Expression::Identifier { span: t_span }),
        member_span: zero_span,
        is_scope_resolution: false,
    };
    let dst_t0 = emitter.emit_expression(&expr_t0, None).expect("emit t.0");
    let inst_t0 = emitter.function.instructions.last().unwrap();
    assert_eq!(inst_t0.opcode(0).unwrap(), OpCode::GetTuple);
    assert_eq!(inst_t0.dst(), dst_t0);
    assert_eq!(inst_t0.src1(), t_reg);
    assert_eq!(inst_t0.src2(), 0); // slot 0

    let expr_t1 = Expression::MemberAccess {
        span: make_span(0, 3),
        object: Box::new(Expression::Identifier { span: t_span }),
        member_span: one_span,
        is_scope_resolution: false,
    };
    let dst_t1 = emitter.emit_expression(&expr_t1, None).expect("emit t.1");
    let inst_t1 = emitter.function.instructions.last().unwrap();
    assert_eq!(inst_t1.opcode(0).unwrap(), OpCode::GetTuple);
    assert_eq!(inst_t1.dst(), dst_t1);
    assert_eq!(inst_t1.src2(), 1); // slot 1
}
