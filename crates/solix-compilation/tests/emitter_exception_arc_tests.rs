//! Unit and integration tests for Phase 9: Safe ARC Tracking, Destructor Wiring & Exception Tables.

#![forbid(unsafe_code)]

use solix_common::bytecode::{CompiledFunction, OpCode};
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_compilation::phases::emitter::{ConstantPoolBuilder, Emitter, RegisterAllocator};
use solix_compilation::phases::parser::statements::Statement;
use solix_compilation::phases::parser::statements::control_flow::{
    BlockStatement, BreakStatement, CatchClause, ReturnStatement, ThrowStatement, TryStatement,
    VariableDeclarationStatement, WhileStatement,
};
use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};
use solix_compilation::phases::parser::statements::types::TypeAnnotation;

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_arc_retain_release_placement() {
    let source = "User u1 u2 null".to_string();
    let sources = vec![source];

    let mut func = CompiledFunction::new("test_arc()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // Block with User u2 (reg 0), User u1 = u2 (reg 1), then u1 = null;
    let block = BlockStatement {
        span: make_span(0, 50),
        statements: vec![
            // User u2; (allocated reg 0)
            Statement::VariableDeclaration(VariableDeclarationStatement {
                span: make_span(0, 7),
                name_span: make_span(8, 10), // "u2"
                is_const: false,
                var_type: TypeAnnotation::Named {
                    span: make_span(0, 4), // "User"
                    path: vec![make_span(0, 4)],
                },
                initializer: None,
            }),
            // User u1 = u2;
            Statement::VariableDeclaration(VariableDeclarationStatement {
                span: make_span(0, 10),
                name_span: make_span(5, 7), // "u1"
                is_const: false,
                var_type: TypeAnnotation::Named {
                    span: make_span(0, 4), // "User"
                    path: vec![make_span(0, 4)],
                },
                initializer: Some(Expression::Identifier {
                    span: make_span(8, 10), // "u2"
                }),
            }),
            // u1 = null;
            Statement::Expression(
                solix_compilation::phases::parser::statements::control_flow::ExpressionStatement {
                    span: make_span(11, 20),
                    expression: Expression::Assignment {
                        span: make_span(11, 20),
                        target: Box::new(Expression::Identifier {
                            span: make_span(5, 7), // "u1"
                        }),
                        operator: TokenKind::OperatorAssign,
                        value: Box::new(Expression::Literal {
                            span: make_span(11, 15), // "null"
                            value: LiteralValue::Null,
                        }),
                    },
                },
            ),
        ],
    };

    emitter
        .emit_block_statement(&block, &mut loop_stack)
        .expect("emit arc block");

    let instructions = &func.instructions;
    // 1. u2 init: LOAD_NULL r0
    assert_eq!(instructions[0].opcode(0).expect("op"), OpCode::LoadNull);
    assert_eq!(instructions[0].dst(), 0);

    // 2. u1 = u2: MOV r1, r0, INC_REF r1
    assert_eq!(instructions[1].opcode(1).expect("op"), OpCode::Mov);
    assert_eq!(instructions[1].dst(), 1); // r_u1
    assert_eq!(instructions[1].src1(), 0); // r_u2
    assert_eq!(instructions[2].opcode(2).expect("op"), OpCode::IncRef);
    assert_eq!(instructions[2].dst(), 1); // r_u1

    // 3. u1 = null: LOAD_NULL r3, DEC_REF r1, MOV r1, r3
    assert_eq!(instructions[3].opcode(3).expect("op"), OpCode::LoadNull);
    assert_eq!(instructions[4].opcode(4).expect("op"), OpCode::DecRef);
    assert_eq!(instructions[4].dst(), 1); // r_u1
    assert_eq!(instructions[5].opcode(5).expect("op"), OpCode::Mov);
    assert_eq!(instructions[5].dst(), 1); // r_u1

    // 4. Block exit: DEC_REF on all block-local reference variables (r0 and r1)
    let exit_decrefs: Vec<u16> = instructions[6..]
        .iter()
        .filter(|inst| inst.opcode(0).ok() == Some(OpCode::DecRef))
        .map(|inst| inst.dst())
        .collect();
    assert!(exit_decrefs.contains(&0));
    assert!(exit_decrefs.contains(&1));
}

#[test]
fn test_loop_break_arc_cleanup() {
    let source = "User item while true break".to_string();
    let sources = vec![source];

    let mut func = CompiledFunction::new("test_break()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // while (true) { User item; break; }
    let while_stmt = WhileStatement {
        span: make_span(0, 50),
        condition: Expression::Literal {
            span: make_span(16, 20),
            value: LiteralValue::Bool(true),
        },
        body: Box::new(Statement::Block(BlockStatement {
            span: make_span(0, 40),
            statements: vec![
                Statement::VariableDeclaration(VariableDeclarationStatement {
                    span: make_span(0, 9),
                    name_span: make_span(5, 9), // "item"
                    is_const: false,
                    var_type: TypeAnnotation::Named {
                        span: make_span(0, 4), // "User"
                        path: vec![make_span(0, 4)],
                    },
                    initializer: None,
                }),
                Statement::Break(BreakStatement {
                    span: make_span(21, 26),
                }),
            ],
        })),
    };

    emitter
        .emit_statement(&Statement::While(while_stmt), &mut loop_stack)
        .expect("emit while with break");

    // Before the break jump instruction, DEC_REF r0 (item) must be emitted
    let mut found_break_decref = false;
    for (idx, inst) in func.instructions.iter().enumerate() {
        if inst.opcode(idx).ok() == Some(OpCode::DecRef) && inst.dst() == 0 {
            found_break_decref = true;
            // The subsequent instruction must be JUMP (break)
            assert_eq!(
                func.instructions[idx + 1].opcode(idx + 1).expect("op"),
                OpCode::Jump
            );
            break;
        }
    }
    assert!(found_break_decref, "Expected DEC_REF prior to break jump");
}

#[test]
fn test_function_return_arc_cleanup_preserves_returned_register() {
    let source = "User a b".to_string();
    let sources = vec![source];

    let mut func = CompiledFunction::new("test_return()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    allocator.enter_scope();
    let r_a = allocator.alloc_local("a").expect("alloc a");
    let r_b = allocator.alloc_local("b").expect("alloc b");

    func.reference_registers = vec![r_a, r_b];

    let mut pool = ConstantPoolBuilder::new();
    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    // return a;
    let ret_stmt = ReturnStatement {
        span: make_span(0, 10),
        value: Some(Expression::Identifier {
            span: make_span(5, 6), // "a"
        }),
    };

    emitter
        .emit_return_statement(&ret_stmt)
        .expect("emit return");

    // Verify DEC_REF r_b is emitted, but r_a is NOT decremented
    let decref_regs: Vec<u16> = func
        .instructions
        .iter()
        .filter(|inst| inst.opcode(0).ok() == Some(OpCode::DecRef))
        .map(|inst| inst.dst())
        .collect();

    assert!(decref_regs.contains(&r_b));
    assert!(!decref_regs.contains(&r_a));

    let last_inst = *func.instructions.last().expect("last inst");
    assert_eq!(last_inst.opcode(0).expect("op"), OpCode::ReturnValue);
    assert_eq!(last_inst.dst(), r_a);
}

#[test]
fn test_zero_cost_exception_tables_and_pc_bounds() {
    let source = "try Error e work handle".to_string();
    let sources = vec![source];

    let mut func = CompiledFunction::new("test_try_catch()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // try { return 1; } catch (Error e) { return 2; }
    let try_stmt = TryStatement {
        span: make_span(0, 50),
        try_block: BlockStatement {
            span: make_span(0, 20),
            statements: vec![Statement::Return(ReturnStatement {
                span: make_span(5, 15),
                value: Some(Expression::Literal {
                    span: make_span(12, 13),
                    value: LiteralValue::Int(1),
                }),
            })],
        },
        catch_clauses: vec![CatchClause {
            span: make_span(25, 45),
            exception_type: TypeAnnotation::Named {
                span: make_span(4, 9), // "Error"
                path: vec![make_span(4, 9)],
            },
            variable_name: make_span(10, 11), // "e"
            block: BlockStatement {
                span: make_span(25, 45),
                statements: vec![Statement::Return(ReturnStatement {
                    span: make_span(30, 40),
                    value: Some(Expression::Literal {
                        span: make_span(37, 38),
                        value: LiteralValue::Int(2),
                    }),
                })],
            },
        }],
        finally_block: None,
    };

    emitter
        .emit_statement(&Statement::Try(try_stmt), &mut loop_stack)
        .expect("emit try catch");

    assert_eq!(func.exception_handlers.len(), 1);
    let handler = &func.exception_handlers[0];

    // Assert try_start_pc points to first instruction of try block (0)
    assert_eq!(handler.try_start_pc, 0);
    // Assert try_end_pc points immediately after try block (before normal jump)
    assert!(handler.try_end_pc > handler.try_start_pc);
    assert_eq!(handler.catch_type_symbol, Some("Error".to_string()));
    assert_eq!(handler.handler_pc, handler.try_end_pc + 1); // handler is after normal exit jump
}

#[test]
fn test_dual_emission_finally_normal_and_early_return() {
    let source = "try finally clean".to_string();
    let sources = vec![source];

    let mut func = CompiledFunction::new("test_finally()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // try { return 10; } finally { let x = 99; }
    let try_stmt = TryStatement {
        span: make_span(0, 50),
        try_block: BlockStatement {
            span: make_span(0, 20),
            statements: vec![Statement::Return(ReturnStatement {
                span: make_span(5, 15),
                value: Some(Expression::Literal {
                    span: make_span(12, 14),
                    value: LiteralValue::Int(10),
                }),
            })],
        },
        catch_clauses: Vec::new(),
        finally_block: Some(BlockStatement {
            span: make_span(25, 45),
            statements: vec![Statement::Expression(
                solix_compilation::phases::parser::statements::control_flow::ExpressionStatement {
                    span: make_span(30, 40),
                    expression: Expression::Literal {
                        span: make_span(35, 37),
                        value: LiteralValue::Int(99),
                    },
                },
            )],
        }),
    };

    emitter
        .emit_statement(&Statement::Try(try_stmt), &mut loop_stack)
        .expect("emit try finally");

    // 1. Verify ExceptionHandlerEntry has finally_pc populated for unwinding
    assert!(!func.exception_handlers.is_empty());
    let handler = &func.exception_handlers[0];
    assert!(handler.finally_pc.is_some());

    // 2. Verify that in the early return path inside the try block,
    // the finally block statement (evaluating 99) executes BEFORE the ReturnValue instruction!
    let mut return_idx = None;
    let mut load_99_idx = None;

    for (idx, inst) in func.instructions.iter().enumerate() {
        if inst.opcode(idx).ok() == Some(OpCode::LoadI32) && inst.imm32_signed() == 99 {
            load_99_idx = Some(idx);
        }
        if inst.opcode(idx).ok() == Some(OpCode::ReturnValue) {
            return_idx = Some(idx);
            break;
        }
    }

    assert!(
        load_99_idx.is_some(),
        "Finally block should be emitted before early return"
    );
    assert!(
        return_idx.is_some(),
        "ReturnValue instruction should be emitted"
    );
    assert!(
        load_99_idx.unwrap() < return_idx.unwrap(),
        "Finally must execute before ReturnValue"
    );
}

#[test]
fn test_throw_statement_emission() {
    let source = "throw err".to_string();
    let sources = vec![source];

    let mut func = CompiledFunction::new("test_throw()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    allocator.enter_scope();
    let r_err = allocator.alloc_local("err").expect("alloc err");

    let mut pool = ConstantPoolBuilder::new();
    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    let throw_stmt = ThrowStatement {
        span: make_span(0, 9),
        expression: Expression::Identifier {
            span: make_span(6, 9), // "err"
        },
    };

    emitter
        .emit_throw_statement(&throw_stmt)
        .expect("emit throw");

    let last_inst = *func.instructions.last().expect("last inst");
    assert_eq!(last_inst.opcode(0).expect("op"), OpCode::Throw);
    assert_eq!(last_inst.dst(), r_err);
}
