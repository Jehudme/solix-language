//! Unit and integration tests for Phase 4: Control Flow, Loops, Jump Backpatching & Disassembler.

use solix_common::bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ExceptionHandlerEntry, Instruction, OpCode,
};
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_compilation::phases::emitter::{
    ConstantPoolBuilder, Emitter, RegisterAllocator, disassemble_function, disassemble_program,
};
use solix_compilation::phases::parser::statements::Statement;
use solix_compilation::phases::parser::statements::control_flow::{
    BlockStatement, BreakStatement, CaseClause, ContinueStatement, DoWhileStatement, ForStatement,
    IfStatement, ReturnStatement, SwitchStatement, VariableDeclarationStatement, WhileStatement,
};
use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_if_else_branching_and_jump_offsets() {
    let mut func = CompiledFunction::new("test_if_else()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // if (true) { return 1; } else { return 2; }
    let if_stmt = IfStatement {
        span: make_span(0, 50),
        condition: Expression::Literal {
            span: make_span(4, 8),
            value: LiteralValue::Bool(true),
        },
        then_branch: Box::new(Statement::Return(ReturnStatement {
            span: make_span(10, 20),
            value: Some(Expression::Literal {
                span: make_span(17, 18),
                value: LiteralValue::Int(1),
            }),
        })),
        else_branch: Some(Box::new(Statement::Return(ReturnStatement {
            span: make_span(30, 40),
            value: Some(Expression::Literal {
                span: make_span(37, 38),
                value: LiteralValue::Int(2),
            }),
        }))),
    };

    emitter
        .emit_statement(&Statement::If(if_stmt), &mut loop_stack)
        .expect("emit if-else");

    // Verify JUMP_IF_FALSE and JUMP exit instructions
    let mut jump_false_opt = None;
    let mut jump_exit_opt = None;

    for (idx, inst) in emitter.function.instructions.iter().enumerate() {
        if let Ok(op) = inst.opcode(idx) {
            if op == OpCode::JumpIfFalse {
                jump_false_opt = Some((idx, inst.jump_offset()));
            } else if op == OpCode::Jump {
                jump_exit_opt = Some((idx, inst.jump_offset()));
            }
        }
    }

    let (jump_false_pc, false_offset) = jump_false_opt.expect("must have JUMP_IF_FALSE");
    let (jump_exit_pc, exit_offset) = jump_exit_opt.expect("must have JUMP exit");

    assert!(false_offset > 0, "JumpIfFalse must jump forward to else");
    assert!(exit_offset > 0, "Jump must jump forward past else");
    assert!(
        jump_false_pc as i32 + false_offset > jump_exit_pc as i32,
        "False target must be past then block"
    );
}

#[test]
fn test_while_loop_backward_jump_and_break_backpatching() {
    let mut func = CompiledFunction::new("test_while()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // while (true) { break; }
    let while_stmt = WhileStatement {
        span: make_span(0, 30),
        condition: Expression::Literal {
            span: make_span(7, 11),
            value: LiteralValue::Bool(true),
        },
        body: Box::new(Statement::Block(BlockStatement {
            span: make_span(13, 29),
            statements: vec![Statement::Break(BreakStatement {
                span: make_span(15, 21),
            })],
        })),
    };

    emitter
        .emit_statement(&Statement::While(while_stmt), &mut loop_stack)
        .expect("emit while");

    // The last instruction must be the loop backward jump with negative offset
    let last_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(last_inst.opcode(0).unwrap(), OpCode::Jump);
    assert!(
        last_inst.jump_offset() < 0,
        "Loop backward jump must have negative offset"
    );

    // Verify the break jump was backpatched to point past the loop
    let total_insts = emitter.function.instructions.len() as i32;
    let mut found_break = false;
    for (idx, inst) in emitter.function.instructions.iter().enumerate() {
        if matches!(inst.opcode(idx), Ok(OpCode::Jump)) && inst.jump_offset() > 0 {
            // Break jump
            assert_eq!(idx as i32 + inst.jump_offset(), total_insts);
            found_break = true;
        }
    }
    assert!(found_break, "Break jump must be backpatched to loop exit");
}

#[test]
fn test_do_while_loop_execution_and_continue() {
    let mut func = CompiledFunction::new("test_do_while()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // do { continue; } while (true);
    let do_while_stmt = DoWhileStatement {
        span: make_span(0, 35),
        body: Box::new(Statement::Block(BlockStatement {
            span: make_span(3, 20),
            statements: vec![Statement::Continue(ContinueStatement {
                span: make_span(5, 14),
            })],
        })),
        condition: Expression::Literal {
            span: make_span(27, 31),
            value: LiteralValue::Bool(true),
        },
    };

    emitter
        .emit_statement(&Statement::DoWhile(do_while_stmt), &mut loop_stack)
        .expect("emit do-while");

    // The condition test is JUMP_IF_TRUE jumping backwards to instruction 0
    let last_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(last_inst.opcode(0).unwrap(), OpCode::JumpIfTrue);
    assert!(
        last_inst.jump_offset() < 0,
        "Do-while backward jump must have negative offset"
    );
}

#[test]
fn test_for_loop_with_init_cond_update_and_break() {
    let mut func = CompiledFunction::new("test_for_loop()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["i".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // for (int32 i = 0; i < 10; ++i) { if (i == 5) break; }
    let for_stmt = ForStatement {
        span: make_span(0, 60),
        initializer: Some(Box::new(Statement::VariableDeclaration(
            VariableDeclarationStatement {
                span: make_span(5, 17),
                name_span: make_span(0, 1), // "i"
                is_const: false,
                var_type: solix_compilation::phases::parser::statements::types::TypeAnnotation::Primitive {
                    span: make_span(5, 10),
                    kind: TokenKind::PrimitiveInt32,
                },
                initializer: Some(Expression::Literal {
                    span: make_span(15, 16),
                    value: LiteralValue::Int(0),
                }),
            },
        ))),
        condition: Some(Expression::Binary {
            span: make_span(18, 24),
            left: Box::new(Expression::Identifier {
                span: make_span(0, 1),
            }),
            operator: TokenKind::OperatorLessThan,
            right: Box::new(Expression::Literal {
                span: make_span(22, 24),
                value: LiteralValue::Int(10),
            }),
        }),
        update: Some(Expression::Unary {
            span: make_span(26, 29),
            operator: TokenKind::OperatorIncrement,
            operand: Box::new(Expression::Identifier {
                span: make_span(0, 1),
            }),
            is_prefix: true,
        }),
        body: Box::new(Statement::Break(BreakStatement {
            span: make_span(35, 41),
        })),
    };

    emitter
        .emit_statement(&Statement::For(for_stmt), &mut loop_stack)
        .expect("emit for");

    // Verify backward jump at loop end
    let last_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(last_inst.opcode(0).unwrap(), OpCode::Jump);
    assert!(last_inst.jump_offset() < 0);
}

#[test]
fn test_switch_statement_cases_and_break_backpatching() {
    let mut func = CompiledFunction::new("test_switch()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);
    let mut loop_stack = Vec::new();

    // switch (x) { case 1: break; default: break; }
    let switch_stmt = SwitchStatement {
        span: make_span(0, 60),
        discriminant: Expression::Literal {
            span: make_span(8, 9),
            value: LiteralValue::Int(1),
        },
        cases: vec![CaseClause {
            span: make_span(12, 25),
            case_value: Expression::Literal {
                span: make_span(17, 18),
                value: LiteralValue::Int(1),
            },
            statements: vec![Statement::Break(BreakStatement {
                span: make_span(20, 26),
            })],
        }],
        default_case: Some(vec![Statement::Break(BreakStatement {
            span: make_span(35, 41),
        })]),
    };

    emitter
        .emit_statement(&Statement::Switch(switch_stmt), &mut loop_stack)
        .expect("emit switch");

    // All breaks in switch must jump to exit
    let exit_pc = emitter.function.instructions.len() as i32;
    for (idx, inst) in emitter.function.instructions.iter().enumerate() {
        if let Ok(OpCode::Jump) = inst.opcode(idx) {
            let target = idx as i32 + inst.jump_offset();
            assert_eq!(target, exit_pc);
        }
    }
}

#[test]
fn test_disassembler_formatting_and_inspection() {
    let mut func = CompiledFunction::new("solix::models::User::calculate_total()", 0, 2, 4);
    func.reference_registers = vec![0];
    func.exception_handlers.push(ExceptionHandlerEntry {
        try_start_pc: 2,
        try_end_pc: 4,
        catch_type_symbol: Some("std::Exception".to_string()),
        catch_reg: 3,
        handler_pc: 5,
        finally_pc: Some(7),
    });

    func.emit(Instruction::format_b(OpCode::LoadI32, 0, 0, 42), None);
    func.emit(Instruction::format_b(OpCode::LoadConst, 0, 1, 0), None);
    func.emit(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1), None);
    func.emit(Instruction::format_c(OpCode::JumpIfFalse, 0, 2, 2), None);
    func.emit(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0), None);

    let mut pool = ConstantPoolBuilder::new();
    pool.add_string("Discount").unwrap();
    let compiled_pool = pool.finish();

    let disassembled = disassemble_function(&func, &compiled_pool);

    // Verify key assembly annotations appear cleanly
    assert!(disassembled.contains(".function solix::models::User::calculate_total()"));
    assert!(disassembled.contains("frame_size: 4"));
    assert!(disassembled.contains(".handlers:"));
    assert!(disassembled.contains("TRY: 0002..0004 -> CATCH 'std::Exception' -> @0005"));
    assert!(disassembled.contains("LOAD_I32"));
    assert!(disassembled.contains("LOAD_CONST"));
    assert!(disassembled.contains("\"Discount\""));
    assert!(disassembled.contains("ADD_I32"));
    assert!(disassembled.contains("JUMP_IF_FALSE"));
    assert!(disassembled.contains("RETURN_VALUE"));

    // Verify program-level disassembly
    let mut program = BinaryProgram::new();
    program.entry_point_symbol = Some("solix::models::User::calculate_total()".to_string());
    program.entry_point_id = Some(0);
    program.constant_pool = compiled_pool;
    let mut cls = CompiledClass::new("solix::models::User", 0, None, 32);
    cls.vtable = vec![0];
    program.classes.push(cls);
    program.functions.push(func);

    let program_disasm = disassemble_program(&program);
    assert!(program_disasm.contains("Solix SLXB Binary Disassembly"));
    assert!(program_disasm.contains(".constant_pool:"));
    assert!(program_disasm.contains(".classes:"));
    assert!(program_disasm.contains(".functions (1 total):"));
}
