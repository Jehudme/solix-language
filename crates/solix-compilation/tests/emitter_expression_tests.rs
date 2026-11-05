//! Unit and integration tests for Phase 3: Expression bytecode emission.

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
fn test_literal_expression_emission() {
    let mut func = CompiledFunction::new("test_literals()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // 1. Small i32 literal
    let expr_i32 = Expression::Literal {
        span: make_span(0, 2),
        value: LiteralValue::Int(42),
    };
    let reg1 = emitter.emit_expression(&expr_i32, None).unwrap();
    assert_eq!(reg1, 0);
    let inst1 = emitter.function.instructions[0];
    assert_eq!(inst1.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst1.dst(), 0);
    assert_eq!(inst1.imm32_signed(), 42);

    // 2. Large i64 literal > i32::MAX (constant pool)
    let expr_i64 = Expression::Literal {
        span: make_span(3, 10),
        value: LiteralValue::Int(10_000_000_000),
    };
    let reg2 = emitter.emit_expression(&expr_i64, None).unwrap();
    assert_eq!(reg2, 1);
    let inst2 = emitter.function.instructions[1];
    assert_eq!(inst2.opcode(1).unwrap(), OpCode::LoadConst);
    assert_eq!(inst2.dst(), 1);

    // 3. Float literal (emitted as LoadF64 from constant pool)
    let expr_f64 = Expression::Literal {
        span: make_span(11, 15),
        value: LiteralValue::Float(99.5),
    };
    let reg3 = emitter.emit_expression(&expr_f64, None).unwrap();
    let inst3 = emitter.function.instructions[2];
    assert_eq!(inst3.opcode(2).unwrap(), OpCode::LoadF64);
    assert_eq!(inst3.dst(), reg3);

    // 4. String literal (constant pool)
    let expr_str = Expression::Literal {
        span: make_span(16, 23),
        value: LiteralValue::String("Solix".to_string()),
    };
    let reg4 = emitter.emit_expression(&expr_str, None).unwrap();
    let inst4 = emitter.function.instructions[3];
    assert_eq!(inst4.opcode(3).unwrap(), OpCode::LoadConst);
    assert_eq!(inst4.dst(), reg4);

    // 5. Bool literal
    let expr_bool = Expression::Literal {
        span: make_span(24, 28),
        value: LiteralValue::Bool(true),
    };
    let _reg5 = emitter.emit_expression(&expr_bool, None).unwrap();
    let inst5 = emitter.function.instructions[4];
    assert_eq!(inst5.opcode(4).unwrap(), OpCode::LoadBool);
    assert_eq!(inst5.imm32(), 1);

    // 6. Null literal
    let expr_null = Expression::Literal {
        span: make_span(29, 33),
        value: LiteralValue::Null,
    };
    let reg6 = emitter.emit_expression(&expr_null, None).unwrap();
    let inst6 = emitter.function.instructions[5];
    assert_eq!(inst6.opcode(5).unwrap(), OpCode::LoadNull);
    assert_eq!(inst6.dst(), reg6);
}

#[test]
fn test_binary_arithmetic_integer_and_float_dispatch() {
    let mut func = CompiledFunction::new("test_arithmetic()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    // Node types map: left is float64
    let mut node_types = HashMap::new();
    let float_span = make_span(0, 5);
    node_types.insert(float_span, Type::Primitive(PrimitiveKind::Float64));

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types);

    // Integer: 10 + 20
    let expr_int_add = Expression::Binary {
        span: make_span(10, 20),
        left: Box::new(Expression::Literal {
            span: make_span(10, 12),
            value: LiteralValue::Int(10),
        }),
        operator: TokenKind::OperatorPlus,
        right: Box::new(Expression::Literal {
            span: make_span(15, 17),
            value: LiteralValue::Int(20),
        }),
    };
    let dst_int = emitter.emit_expression(&expr_int_add, None).unwrap();
    // Verify instruction is ADD_I32
    let add_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(add_inst.opcode(0).unwrap(), OpCode::AddI32);
    assert_eq!(add_inst.dst(), dst_int);

    // Float: 1.5 + 2.5
    let expr_float_add = Expression::Binary {
        span: make_span(30, 40),
        left: Box::new(Expression::Literal {
            span: float_span,
            value: LiteralValue::Float(1.5),
        }),
        operator: TokenKind::OperatorPlus,
        right: Box::new(Expression::Literal {
            span: make_span(35, 38),
            value: LiteralValue::Float(2.5),
        }),
    };
    let dst_float = emitter.emit_expression(&expr_float_add, None).unwrap();
    let float_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(float_inst.opcode(0).unwrap(), OpCode::AddF64);
    assert_eq!(float_inst.dst(), dst_float);
}

#[test]
fn test_short_circuit_logic_emission() {
    let mut func = CompiledFunction::new("test_short_circuit()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // a && b
    let expr_and = Expression::Binary {
        span: make_span(0, 10),
        left: Box::new(Expression::Literal {
            span: make_span(0, 4),
            value: LiteralValue::Bool(true),
        }),
        operator: TokenKind::OperatorLogicalAnd,
        right: Box::new(Expression::Literal {
            span: make_span(8, 10),
            value: LiteralValue::Bool(false),
        }),
    };

    let dst = emitter.emit_expression(&expr_and, None).unwrap();

    // Verify JUMP_IF_FALSE instruction was emitted and backpatched
    let jump_inst = emitter.function.instructions[1];
    assert_eq!(jump_inst.opcode(1).unwrap(), OpCode::JumpIfFalse);
    assert_eq!(jump_inst.cond_reg(), dst);
    // Jump offset must be positive forward jump
    assert!(jump_inst.jump_offset() > 0);
}

#[test]
fn test_unary_operations_and_increments() {
    let mut func = CompiledFunction::new("test_unary()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["x".to_string()];

    allocator.enter_scope();
    let x_reg = allocator.alloc_local("x").unwrap();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // 1. Not (!flag)
    let expr_not = Expression::Unary {
        span: make_span(0, 1),
        operator: TokenKind::OperatorLogicalNot,
        operand: Box::new(Expression::Literal {
            span: make_span(1, 2),
            value: LiteralValue::Bool(true),
        }),
        is_prefix: true,
    };
    let not_reg = emitter.emit_expression(&expr_not, None).unwrap();
    let not_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(not_inst.opcode(0).unwrap(), OpCode::Not);
    assert_eq!(not_inst.dst(), not_reg);

    // 2. Prefix increment (++x)
    let expr_pre_inc = Expression::Unary {
        span: make_span(0, 1),
        operator: TokenKind::OperatorIncrement,
        operand: Box::new(Expression::Identifier {
            span: make_span(0, 1), // "x"
        }),
        is_prefix: true,
    };
    let pre_dst = emitter.emit_expression(&expr_pre_inc, None).unwrap();
    // Result evaluated into pre_dst, which was copied from x_reg (r0)
    let last_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(last_inst.opcode(0).unwrap(), OpCode::Mov);
    assert_eq!(last_inst.dst(), pre_dst);
    assert_eq!(last_inst.src1(), x_reg);
}

#[test]
fn test_ternary_expression_conditional_jumps() {
    let mut func = CompiledFunction::new("test_ternary()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // cond ? 10 : 20
    let expr_ternary = Expression::Ternary {
        span: make_span(0, 20),
        condition: Box::new(Expression::Literal {
            span: make_span(0, 4),
            value: LiteralValue::Bool(true),
        }),
        then_branch: Box::new(Expression::Literal {
            span: make_span(7, 9),
            value: LiteralValue::Int(10),
        }),
        else_branch: Box::new(Expression::Literal {
            span: make_span(12, 14),
            value: LiteralValue::Int(20),
        }),
    };

    let dst = emitter.emit_expression(&expr_ternary, None).unwrap();

    // Verify JUMP_IF_FALSE and unconditional JUMP are both emitted
    let mut has_jump_if_false = false;
    let mut has_unconditional_jump = false;
    for inst in &emitter.function.instructions {
        if let Ok(op) = inst.opcode(0) {
            if op == OpCode::JumpIfFalse {
                has_jump_if_false = true;
                assert!(inst.jump_offset() > 0);
            } else if op == OpCode::Jump {
                has_unconditional_jump = true;
                assert!(inst.jump_offset() > 0);
            }
        }
    }

    assert!(has_jump_if_false);
    assert!(has_unconditional_jump);
    assert_eq!(dst, 0);
}

#[test]
fn test_numeric_type_cast_conversions() {
    let mut func = CompiledFunction::new("test_casts()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let cast_span = make_span(0, 10);
    let inner_span = make_span(5, 7);

    // Map inner expression to Int32 and cast expression to Float64
    let mut node_types = HashMap::new();
    node_types.insert(inner_span, Type::Primitive(PrimitiveKind::Int32));
    node_types.insert(cast_span, Type::Primitive(PrimitiveKind::Float64));

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types);

    let expr_cast = Expression::Cast {
        span: cast_span,
        expression: Box::new(Expression::Literal {
            span: inner_span,
            value: LiteralValue::Int(42),
        }),
        target_type:
            solix_compilation::phases::parser::statements::types::TypeAnnotation::Primitive {
                span: cast_span,
                kind: TokenKind::PrimitiveFloat64,
            },
    };

    let dst = emitter.emit_expression(&expr_cast, None).unwrap();
    let cast_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(cast_inst.opcode(0).unwrap(), OpCode::ConvIToF);
    assert_eq!(cast_inst.dst(), dst);
}
