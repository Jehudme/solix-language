//! Unit and integration tests for Phase 8: First-Class Lambdas, Closures & Function Pointers.

#![forbid(unsafe_code)]

use solix_common::bytecode::OpCode;
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_compilation::phases::emitter::{
    ConstantPoolBuilder, Emitter, RegisterAllocator, find_free_variables,
};
use solix_compilation::phases::parser::statements::declarations::Parameter;
use solix_compilation::phases::parser::statements::expressions::{
    Expression, LambdaBody, LiteralValue,
};
use solix_compilation::phases::parser::statements::types::TypeAnnotation;
use std::collections::HashMap;

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_free_variable_capture_analysis() {
    let source = "let x = 10; let y = 20; let z = 30; (a, b) => x + a * y;".to_string();
    let sources = vec![source];

    let mut allocator = RegisterAllocator::new(0);
    allocator.enter_scope();
    allocator.alloc_local("x").expect("alloc x");
    allocator.alloc_local("y").expect("alloc y");
    allocator.alloc_local("z").expect("alloc z");

    let lambda = Expression::Lambda {
        span: make_span(36, 56),
        captures: Vec::new(),
        parameters: vec![
            Parameter {
                span: make_span(37, 38),
                name_span: make_span(37, 38), // "a"
                param_type: TypeAnnotation::Primitive {
                    span: make_span(0, 0),
                    kind: TokenKind::PrimitiveInt64,
                },
                default_value: None,
                is_variadic: false,
            },
            Parameter {
                span: make_span(40, 41),
                name_span: make_span(40, 41), // "b"
                param_type: TypeAnnotation::Primitive {
                    span: make_span(0, 0),
                    kind: TokenKind::PrimitiveInt64,
                },
                default_value: None,
                is_variadic: false,
            },
        ],
        body: LambdaBody::Expression(Box::new(Expression::Binary {
            span: make_span(46, 55),
            left: Box::new(Expression::Identifier {
                span: make_span(4, 5), // "x"
            }),
            operator: TokenKind::OperatorPlus,
            right: Box::new(Expression::Binary {
                span: make_span(50, 55),
                left: Box::new(Expression::Identifier {
                    span: make_span(37, 38), // "a"
                }),
                operator: TokenKind::OperatorMultiply,
                right: Box::new(Expression::Identifier {
                    span: make_span(16, 17), // "y"
                }),
            }),
        })),
    };

    let free_vars = find_free_variables(&lambda, &allocator, Some(&sources));
    assert_eq!(free_vars, vec!["x".to_string(), "y".to_string()]);
}

#[test]
fn test_closure_packing_and_synthesized_lambda() {
    let source = "x y z (val) => x + val".to_string();
    let sources = vec![source];

    let mut outer_func = solix_common::bytecode::CompiledFunction::new("outer()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    allocator.enter_scope();
    let r_x = allocator.alloc_local("x").expect("alloc x");
    let _r_y = allocator.alloc_local("y").expect("alloc y");

    let mut pool = ConstantPoolBuilder::new();
    let mut synthesized_funcs = Vec::new();

    let lambda = Expression::Lambda {
        span: make_span(6, 22),
        captures: Vec::new(),
        parameters: vec![Parameter {
            span: make_span(7, 10),
            name_span: make_span(7, 10), // "val"
            param_type: TypeAnnotation::Primitive {
                span: make_span(0, 0),
                kind: TokenKind::PrimitiveInt64,
            },
            default_value: None,
            is_variadic: false,
        }],
        body: LambdaBody::Expression(Box::new(Expression::Binary {
            span: make_span(15, 22),
            left: Box::new(Expression::Identifier {
                span: make_span(0, 1), // "x"
            }),
            operator: TokenKind::OperatorPlus,
            right: Box::new(Expression::Identifier {
                span: make_span(7, 10), // "val"
            }),
        })),
    };

    let r_closure = {
        let mut emitter = Emitter::new(&mut outer_func, &mut allocator, &mut pool)
            .with_sources(&sources)
            .with_synthesized_functions(&mut synthesized_funcs);

        emitter
            .emit_expression(&lambda, None)
            .expect("emit lambda closure")
    };

    assert_eq!(r_closure, 2);

    // Verify outer function emitted MOV x to capture window and PACK_CLOSURE
    let instructions = &outer_func.instructions;
    assert!(instructions.len() >= 2);

    let mov_inst = instructions[0];
    assert_eq!(mov_inst.opcode(0).expect("op"), OpCode::Mov);
    assert_eq!(mov_inst.src1(), r_x);

    let pack_inst = instructions[1];
    assert_eq!(pack_inst.opcode(1).expect("op"), OpCode::PackClosure);
    assert_eq!(pack_inst.call_dst(), r_closure);
    assert_eq!(pack_inst.call_target(), 1); // lambda_id 1
    assert_eq!(pack_inst.arg_count(), 1); // 1 captured variable ("x")

    // Verify synthesized lambda function
    assert_eq!(synthesized_funcs.len(), 1);
    let lambda_func = &synthesized_funcs[0];
    assert_eq!(lambda_func.symbol, "<lambda_1>");
    assert_eq!(lambda_func.param_count, 1);

    // Lambda first instruction must unpack captures
    let lambda_instructions = &lambda_func.instructions;
    assert!(!lambda_instructions.is_empty());

    let unpack_inst = lambda_instructions[0];
    assert_eq!(unpack_inst.opcode(0).expect("op"), OpCode::UnpackCaptures);
    assert_eq!(unpack_inst.arg_count(), 1);

    // Lambda must end with RETURN_VALUE
    let last_inst = *lambda_instructions.last().expect("last inst");
    assert_eq!(last_inst.opcode(0).expect("op"), OpCode::ReturnValue);
}

#[test]
fn test_callable_invocation_and_function_pointer() {
    let source = "my_calc fn_var 42".to_string();
    let sources = vec![source];

    let mut func = solix_common::bytecode::CompiledFunction::new("test_call()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    allocator.enter_scope();
    let r_fn = allocator.alloc_local("fn_var").expect("alloc fn_var");

    let mut pool = ConstantPoolBuilder::new();
    let mut func_ids = HashMap::new();
    func_ids.insert("my_calc".to_string(), 77u32);

    // Test 1: Referencing "my_calc" as a first-class function pointer
    let fn_ptr_expr = Expression::Identifier {
        span: make_span(0, 7), // "my_calc"
    };

    let r_ptr = {
        let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
            .with_sources(&sources)
            .with_function_ids(&func_ids);

        emitter
            .emit_expression(&fn_ptr_expr, None)
            .expect("emit function pointer")
    };

    let ptr_pack_inst = *func.instructions.last().expect("pack inst");
    assert_eq!(ptr_pack_inst.opcode(0).expect("op"), OpCode::PackClosure);
    assert_eq!(ptr_pack_inst.call_dst(), r_ptr);
    assert_eq!(ptr_pack_inst.call_target(), 77); // function ID
    assert_eq!(ptr_pack_inst.arg_count(), 0); // 0 captures for bare function pointer

    // Test 2: Invoking the local callable variable fn_var(42)
    let call_expr = Expression::MethodCall {
        span: make_span(8, 17),
        callee: Box::new(Expression::Identifier {
            span: make_span(8, 14), // "fn_var"
        }),
        generic_arguments: Vec::new(),
        arguments: vec![Expression::Literal {
            span: make_span(15, 17),
            value: LiteralValue::Int(42),
        }],
    };

    let r_call_res = {
        let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
            .with_sources(&sources)
            .with_function_ids(&func_ids);

        emitter
            .emit_expression(&call_expr, None)
            .expect("emit callable invocation")
    };

    let call_inst = *func.instructions.last().expect("call inst");
    assert_eq!(call_inst.opcode(0).expect("op"), OpCode::CallCallable);
    assert_eq!(call_inst.call_dst(), r_call_res);
    assert_eq!(call_inst.call_target(), r_fn); // callable register target
    assert_eq!(call_inst.arg_count(), 1); // 1 argument passed
}
