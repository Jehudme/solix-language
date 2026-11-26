//! Control-flow bytecode generation and jump backpatching engine.

#![forbid(unsafe_code)]

use crate::phases::emitter::emitter::Emitter;
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::control_flow::{
    BlockStatement, DoWhileStatement, ForStatement, IfStatement, ReturnStatement, SwitchStatement,
    VariableDeclarationStatement, WhileStatement,
};
use crate::phases::parser::statements::expressions::{Expression, LiteralValue};
use solix_common::bytecode::{Instruction, OpCode};
use solix_common::diagnostics::EmissionError;
use solix_common::types::{PrimitiveKind, Type};

/// Active loop context tracking forward break jumps and loop continue targets.
#[derive(Debug, Clone, Default)]
pub struct LoopContext {
    /// Jump instruction PCs that break out of the loop and need patching to loop exit.
    pub break_patches: Vec<usize>,
    /// Jump instruction PCs from continue statements needing patching to loop update/cond.
    pub continue_patches: Vec<usize>,
    /// Scope depth in the allocator when this loop context was pushed.
    pub scope_depth: usize,
    /// Active finally stack depth at loop entry.
    pub finally_depth: usize,
    /// Indicates whether this context represents a switch statement rather than a loop.
    pub is_switch: bool,
}

impl<'ctx> Emitter<'ctx> {
    /// Recursively compiles an imperative AST statement into bytecode instructions.
    ///
    /// # Errors
    /// Returns `EmissionError` if register, constant, or jump offset bounds are exceeded.
    pub fn emit_statement(
        &mut self,
        statement: &Statement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        match statement {
            Statement::Block(block) => self.emit_block_statement(block, loop_stack),
            Statement::If(if_stmt) => self.emit_if_statement(if_stmt, loop_stack),
            Statement::While(while_stmt) => self.emit_while_statement(while_stmt, loop_stack),
            Statement::DoWhile(do_while) => self.emit_do_while_statement(do_while, loop_stack),
            Statement::For(for_stmt) => self.emit_for_statement(for_stmt, loop_stack),
            Statement::Switch(switch_stmt) => self.emit_switch_statement(switch_stmt, loop_stack),
            Statement::Break(break_stmt) => {
                let target_finally_depth = loop_stack
                    .last()
                    .map(|context_item| context_item.finally_depth)
                    .unwrap_or(0);
                self.emit_active_finally_blocks_up_to(target_finally_depth)?;

                if let Some(active_loop) = loop_stack.last() {
                    let loop_depth = active_loop.scope_depth;
                    for register in self.allocator.scopes_allocated_from(loop_depth) {
                        if self.function.reference_registers.contains(&register) {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::DecRef, 0, register, 0, 0),
                                Some(break_stmt.span),
                            );
                        }
                    }
                }

                let patch_pc = self.emit_instruction(
                    Instruction::format_c(OpCode::Jump, 0, 0, 0),
                    Some(break_stmt.span),
                );
                if let Some(active_loop) = loop_stack.last_mut() {
                    active_loop.break_patches.push(patch_pc as usize);
                }
                Ok(())
            }
            Statement::Continue(continue_stmt) => {
                let target_finally_depth = loop_stack
                    .iter()
                    .rev()
                    .find(|context_item| !context_item.is_switch)
                    .map(|context_item| context_item.finally_depth)
                    .unwrap_or(0);
                self.emit_active_finally_blocks_up_to(target_finally_depth)?;

                if let Some(active_loop) = loop_stack
                    .iter()
                    .rev()
                    .find(|context_item| !context_item.is_switch)
                {
                    let loop_depth = active_loop.scope_depth;
                    for register in self.allocator.scopes_allocated_from(loop_depth) {
                        if self.function.reference_registers.contains(&register) {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::DecRef, 0, register, 0, 0),
                                Some(continue_stmt.span),
                            );
                        }
                    }
                }

                let patch_pc = self.emit_instruction(
                    Instruction::format_c(OpCode::Jump, 0, 0, 0),
                    Some(continue_stmt.span),
                );
                if let Some(active_loop) = loop_stack
                    .iter_mut()
                    .rev()
                    .find(|context_item| !context_item.is_switch)
                {
                    active_loop.continue_patches.push(patch_pc as usize);
                }
                Ok(())
            }
            Statement::Return(return_stmt) => self.emit_return_statement(return_stmt),
            Statement::VariableDeclaration(var_decl) => {
                self.emit_variable_declaration_statement(var_decl)
            }
            Statement::Try(try_stmt) => self.emit_try_statement(try_stmt, loop_stack),
            Statement::Throw(throw_stmt) => self.emit_throw_statement(throw_stmt),
            Statement::Expression(expr_stmt) => {
                if matches!(&expr_stmt.expression, Expression::Assignment { .. }) {
                    self.emit_discarded_expression(&expr_stmt.expression)?;
                    return Ok(());
                }
                let target_hint = self.resolve_assignment_target_local(&expr_stmt.expression);
                let temp = self.emit_expression(&expr_stmt.expression, target_hint)?;
                if self
                    .query_type(expr_stmt.expression.span())
                    .is_reference_type()
                    && matches!(
                        expr_stmt.expression,
                        Expression::MethodCall { .. }
                            | Expression::NewInstance { .. }
                            | Expression::ArrayCreation { .. }
                            | Expression::ArrayLiteral { .. }
                            | Expression::TupleCreation { .. }
                            | Expression::Lambda { .. }
                            | Expression::Block { .. }
                    )
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, temp, 0, 0),
                        Some(expr_stmt.span),
                    );
                }
                if target_hint != Some(temp) {
                    self.allocator.free_temp(temp);
                }
                Ok(())
            }
            Statement::Assert(assert_stmt) => self.emit_assert_statement(assert_stmt),
            Statement::Exit(exit_stmt) => self.emit_exit_statement(exit_stmt),
            _ => Ok(()),
        }
    }

    fn emit_exit_statement(
        &mut self,
        exit_stmt: &crate::phases::parser::statements::control_flow::ExitStatement,
    ) -> Result<(), EmissionError> {
        let r_code = self.emit_expression(&exit_stmt.code, None)?;
        self.emit_instruction(
            Instruction::format_a(OpCode::Exit, 0, r_code, 0, 0),
            Some(exit_stmt.span),
        );
        self.allocator.free_temp(r_code);
        Ok(())
    }

    fn emit_assert_statement(
        &mut self,
        assert_stmt: &crate::phases::parser::statements::control_flow::AssertStatement,
    ) -> Result<(), EmissionError> {
        let condition_register = self.emit_expression(&assert_stmt.condition, None)?;
        let jump_patch_pc = self.emit_instruction(
            Instruction::format_c(OpCode::JumpIfTrue, 0, condition_register, 0),
            Some(assert_stmt.span),
        );
        self.allocator.free_temp(condition_register);

        let error_register = self.allocator.alloc_temp()?;
        if let Some(message_expression) = &assert_stmt.message {
            if let Expression::Literal {
                value: LiteralValue::String(literal_string),
                span: literal_span,
            } = message_expression
            {
                let formatted_message = format!("AssertionError: {literal_string}");
                let constant_index = self.pool.add_string(&formatted_message)?;
                self.emit_instruction(
                    Instruction::format_b(
                        OpCode::LoadConst,
                        0,
                        error_register,
                        constant_index as u32,
                    ),
                    Some(*literal_span),
                );
            } else {
                let message_temp = self.emit_expression(message_expression, None)?;
                let prefix_index = self.pool.add_string("AssertionError: ")?;
                let prefix_register = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_b(
                        OpCode::LoadConst,
                        0,
                        prefix_register,
                        prefix_index as u32,
                    ),
                    Some(assert_stmt.span),
                );
                let window_size = 2;
                let argument_base = self.allocator.alloc_window(window_size)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, argument_base, prefix_register, 0),
                    Some(assert_stmt.span),
                );
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, argument_base + 1, message_temp, 0),
                    Some(assert_stmt.span),
                );
                let symbol_index = self.pool.add_string("__builtin_str_concat")?;
                self.emit_instruction(
                    Instruction::format_call(
                        OpCode::CallSymbol,
                        error_register,
                        symbol_index,
                        argument_base,
                        window_size as u8,
                    ),
                    Some(assert_stmt.span),
                );
                self.allocator.free_window(argument_base, window_size);
                self.allocator.free_temp(prefix_register);
                self.allocator.free_temp(message_temp);
            }
        } else {
            let constant_index = self.pool.add_string("AssertionError: Assertion failed")?;
            self.emit_instruction(
                Instruction::format_b(OpCode::LoadConst, 0, error_register, constant_index as u32),
                Some(assert_stmt.span),
            );
        }
        self.emit_instruction(
            Instruction::format_a(OpCode::Throw, 0, error_register, 0, 0),
            Some(assert_stmt.span),
        );
        self.allocator.free_temp(error_register);

        let pass_pc = self.function.instructions.len() as i32;
        let offset = pass_pc - jump_patch_pc as i32;
        self.patch_jump_offset(jump_patch_pc as usize, offset);
        Ok(())
    }

    /// Compiles a scoped block statement.
    pub fn emit_block_statement(
        &mut self,
        block: &BlockStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        self.allocator.enter_scope();
        for stmt in &block.statements {
            self.emit_statement(stmt, loop_stack)?;
        }
        self.emit_block_exit_arc_cleanup();
        self.allocator.exit_scope();
        Ok(())
    }

    /// Compiles an if-then-else branch with jump backpatching.
    pub fn emit_if_statement(
        &mut self,
        if_stmt: &IfStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        let r_cond = self.emit_expression(&if_stmt.condition, None)?;
        let jump_false_pc = self.emit_instruction(
            Instruction::format_c(OpCode::JumpIfFalse, 0, r_cond, 0),
            Some(if_stmt.span),
        );
        self.allocator.free_temp(r_cond);

        self.emit_statement(&if_stmt.then_branch, loop_stack)?;

        if let Some(else_branch) = &if_stmt.else_branch {
            let jump_exit_pc = self.emit_instruction(
                Instruction::format_c(OpCode::Jump, 0, 0, 0),
                Some(if_stmt.span),
            );

            // Backpatch jump_false to point to start of else branch
            let else_start_pc = self.function.instructions.len() as i32;
            let false_offset = else_start_pc - jump_false_pc as i32;
            self.patch_jump_offset(jump_false_pc as usize, false_offset);

            self.emit_statement(else_branch, loop_stack)?;

            // Backpatch jump_exit to point past the else branch
            let exit_pc = self.function.instructions.len() as i32;
            let exit_offset = exit_pc - jump_exit_pc as i32;
            self.patch_jump_offset(jump_exit_pc as usize, exit_offset);
        } else {
            // No else branch: backpatch jump_false to point past then branch
            let exit_pc = self.function.instructions.len() as i32;
            let false_offset = exit_pc - jump_false_pc as i32;
            self.patch_jump_offset(jump_false_pc as usize, false_offset);
        }

        Ok(())
    }

    /// Compiles a while loop (`while (cond) body`).
    pub fn emit_while_statement(
        &mut self,
        while_stmt: &WhileStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        let header_pc = self.function.instructions.len() as i32;
        loop_stack.push(LoopContext {
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.allocator.scope_depth(),
            finally_depth: self.finally_stack.len(),
            is_switch: false,
        });

        let r_cond = self.emit_expression(&while_stmt.condition, None)?;
        let exit_jump_pc = self.emit_instruction(
            Instruction::format_c(OpCode::JumpIfFalse, 0, r_cond, 0),
            Some(while_stmt.span),
        );
        self.allocator.free_temp(r_cond);

        self.emit_statement(&while_stmt.body, loop_stack)?;

        // Continue statements jump to header_pc
        let loop_ctx = loop_stack.pop().unwrap_or_default();
        for continue_pc in loop_ctx.continue_patches {
            let offset = header_pc - continue_pc as i32;
            self.patch_jump_offset(continue_pc, offset);
        }

        // Backward jump to header
        let loop_end_pc = self.function.instructions.len() as i32;
        let backward_offset = header_pc - loop_end_pc;
        self.emit_instruction(
            Instruction::format_c(OpCode::Jump, 0, 0, backward_offset),
            Some(while_stmt.span),
        );

        let exit_pc = self.function.instructions.len() as i32;
        let exit_offset = exit_pc - exit_jump_pc as i32;
        self.patch_jump_offset(exit_jump_pc as usize, exit_offset);

        // Backpatch break statements
        for break_pc in loop_ctx.break_patches {
            let offset = exit_pc - break_pc as i32;
            self.patch_jump_offset(break_pc, offset);
        }

        Ok(())
    }

    /// Compiles a do-while loop (`do body while (cond);`).
    pub fn emit_do_while_statement(
        &mut self,
        do_while: &DoWhileStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        let body_pc = self.function.instructions.len() as i32;
        loop_stack.push(LoopContext {
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.allocator.scope_depth(),
            finally_depth: self.finally_stack.len(),
            is_switch: false,
        });

        self.emit_statement(&do_while.body, loop_stack)?;

        let cond_pc = self.function.instructions.len() as i32;
        let loop_ctx = loop_stack.pop().unwrap_or_default();
        for continue_pc in loop_ctx.continue_patches {
            let offset = cond_pc - continue_pc as i32;
            self.patch_jump_offset(continue_pc, offset);
        }

        let r_cond = self.emit_expression(&do_while.condition, None)?;
        let cond_inst_pc = self.function.instructions.len() as i32;
        let backward_offset = body_pc - cond_inst_pc;
        self.emit_instruction(
            Instruction::format_c(OpCode::JumpIfTrue, 0, r_cond, backward_offset),
            Some(do_while.span),
        );
        self.allocator.free_temp(r_cond);

        let exit_pc = self.function.instructions.len() as i32;
        for break_pc in loop_ctx.break_patches {
            let offset = exit_pc - break_pc as i32;
            self.patch_jump_offset(break_pc, offset);
        }

        Ok(())
    }

    /// Compiles a standard for loop (`for (init; cond; update) body`).
    pub fn emit_for_statement(
        &mut self,
        for_stmt: &ForStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        self.allocator.enter_scope();

        if let Some(init) = &for_stmt.initializer {
            self.emit_statement(init, loop_stack)?;
        }

        let header_pc = self.function.instructions.len() as i32;
        loop_stack.push(LoopContext {
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.allocator.scope_depth(),
            finally_depth: self.finally_stack.len(),
            is_switch: false,
        });

        let exit_jump_pc = if let Some(cond) = &for_stmt.condition {
            let r_cond = self.emit_expression(cond, None)?;
            let pc = self.emit_instruction(
                Instruction::format_c(OpCode::JumpIfFalse, 0, r_cond, 0),
                Some(for_stmt.span),
            );
            self.allocator.free_temp(r_cond);
            Some(pc)
        } else {
            None
        };

        self.emit_statement(&for_stmt.body, loop_stack)?;

        let update_pc = self.function.instructions.len() as i32;
        let loop_ctx = loop_stack.pop().unwrap_or_default();
        for continue_pc in loop_ctx.continue_patches {
            let offset = update_pc - continue_pc as i32;
            self.patch_jump_offset(continue_pc, offset);
        }

        if let Some(update) = &for_stmt.update {
            self.emit_discarded_expression(update)?;
        }

        let loop_end_pc = self.function.instructions.len() as i32;
        let backward_offset = header_pc - loop_end_pc;
        self.emit_instruction(
            Instruction::format_c(OpCode::Jump, 0, 0, backward_offset),
            Some(for_stmt.span),
        );

        let exit_pc = self.function.instructions.len() as i32;
        if let Some(exit_pc_idx) = exit_jump_pc {
            let offset = exit_pc - exit_pc_idx as i32;
            self.patch_jump_offset(exit_pc_idx as usize, offset);
        }

        for break_pc in loop_ctx.break_patches {
            let offset = exit_pc - break_pc as i32;
            self.patch_jump_offset(break_pc, offset);
        }

        self.allocator.exit_scope();
        Ok(())
    }

    /// Compiles a switch statement with case comparisons and jump targets.
    pub fn emit_switch_statement(
        &mut self,
        switch_stmt: &SwitchStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        let reg_discriminant = self.emit_expression(&switch_stmt.discriminant, None)?;
        let mut case_exit_patches = Vec::new();
        let mut pending_body_jumps: Vec<usize> = Vec::new();
        let mut switch_break_context = LoopContext {
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.allocator.scope_depth(),
            finally_depth: self.finally_stack.len(),
            is_switch: true,
        };

        loop_stack.push(switch_break_context);

        let discriminant_type = self.query_type(switch_stmt.discriminant.span());

        for case_clause in &switch_stmt.cases {
            let mut case_failure_jumps = Vec::new();

            match &case_clause.case_value {
                Expression::Instanceof { target_type, .. } => {
                    if let crate::phases::parser::statements::types::TypeAnnotation::Tuple {
                        elements,
                        ..
                    } = target_type
                    {
                        for (element_index, element_annotation) in elements.iter().enumerate() {
                            let element_type_name =
                                self.get_lexeme(element_annotation.span()).to_string();
                            let type_tag = self.resolve_type_tag(&element_type_name);
                            let reg_index = self.allocator.alloc_temp()?;
                            self.emit_instruction(
                                Instruction::format_b_signed(
                                    OpCode::LoadI32,
                                    0,
                                    reg_index,
                                    element_index as i32,
                                ),
                                Some(case_clause.span),
                            );
                            let reg_match = self.allocator.alloc_temp()?;
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::TupleInstanceof,
                                    type_tag,
                                    reg_match,
                                    reg_discriminant,
                                    reg_index,
                                ),
                                Some(case_clause.span),
                            );
                            self.allocator.free_temp(reg_index);

                            let fail_jump = self.emit_instruction(
                                Instruction::format_c(OpCode::JumpIfFalse, 0, reg_match, 0),
                                Some(case_clause.span),
                            );
                            self.allocator.free_temp(reg_match);
                            case_failure_jumps.push(fail_jump as usize);
                        }
                    } else {
                        let type_name = self.get_lexeme(target_type.span()).to_string();
                        let simple_type_name =
                            type_name.rsplit(['.', ':']).next().unwrap_or(&type_name);
                        let vtable_id = self
                            .class_layouts
                            .and_then(|layouts| {
                                layouts
                                    .get(&type_name)
                                    .or_else(|| layouts.get(simple_type_name))
                            })
                            .map(|layout| layout.vtable_id)
                            .unwrap_or(0);

                        let reg_match = self.allocator.alloc_temp()?;
                        self.emit_instruction(
                            Instruction::format_a(
                                OpCode::Instanceof,
                                0,
                                reg_match,
                                reg_discriminant,
                                vtable_id as u16,
                            ),
                            Some(case_clause.span),
                        );
                        let fail_jump = self.emit_instruction(
                            Instruction::format_c(OpCode::JumpIfFalse, 0, reg_match, 0),
                            Some(case_clause.span),
                        );
                        self.allocator.free_temp(reg_match);
                        case_failure_jumps.push(fail_jump as usize);
                    }
                }

                Expression::TupleCreation { elements, .. } => {
                    for (element_index, element_expr) in elements.iter().enumerate() {
                        let reg_disc_element = self.allocator.alloc_temp()?;
                        self.emit_instruction(
                            Instruction::format_a(
                                OpCode::GetTuple,
                                0,
                                reg_disc_element,
                                reg_discriminant,
                                element_index as u16,
                            ),
                            Some(case_clause.span),
                        );

                        let reg_case_element = self.emit_expression(element_expr, None)?;
                        let element_type = self.query_type(element_expr.span());
                        let comparison_opcode = if matches!(
                            &element_type,
                            Type::Class { symbol_name, .. }
                                if symbol_name == "std::string"
                                    || symbol_name == "std.string"
                                    || symbol_name == "string"
                        ) {
                            OpCode::EqString
                        } else if matches!(
                            &element_type,
                            Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                        ) {
                            OpCode::EqI64
                        } else if element_type.is_floating_point() {
                            OpCode::EqF64
                        } else if matches!(&element_type, Type::Primitive(PrimitiveKind::Bool)) {
                            OpCode::EqBool
                        } else {
                            OpCode::EqI32
                        };

                        let reg_compare = self.allocator.alloc_temp()?;
                        self.emit_instruction(
                            Instruction::format_a(
                                comparison_opcode,
                                0,
                                reg_compare,
                                reg_disc_element,
                                reg_case_element,
                            ),
                            Some(case_clause.span),
                        );
                        self.allocator.free_temp(reg_case_element);
                        self.allocator.free_temp(reg_disc_element);

                        let fail_jump = self.emit_instruction(
                            Instruction::format_c(OpCode::JumpIfFalse, 0, reg_compare, 0),
                            Some(case_clause.span),
                        );
                        self.allocator.free_temp(reg_compare);
                        case_failure_jumps.push(fail_jump as usize);
                    }
                }

                _ => {
                    let reg_case_value = self.emit_expression(&case_clause.case_value, None)?;
                    let reg_compare = self.allocator.alloc_temp()?;

                    let class_symbol_name = match &discriminant_type {
                        Type::Class { symbol_name, .. }
                            if symbol_name != "std::string"
                                && symbol_name != "std.string"
                                && symbol_name != "string" =>
                        {
                            Some(symbol_name)
                        }
                        _ => None,
                    };

                    if let Some(symbol_name) = class_symbol_name {
                        let simple_class_name =
                            symbol_name.rsplit(['.', ':']).next().unwrap_or(symbol_name);
                        let colon_class_name = symbol_name.replace('.', "::");
                        let dot_class_name = symbol_name.replace("::", ".");

                        let mut operator_eq_candidates = Vec::new();
                        let mut operator_ne_candidates = Vec::new();
                        for prefix in [
                            &colon_class_name,
                            &dot_class_name,
                            symbol_name,
                            &simple_class_name.to_string(),
                        ] {
                            operator_eq_candidates.push(format!("{prefix}::operator_=="));
                            operator_ne_candidates.push(format!("{prefix}::operator_!="));
                        }

                        let equal_function_id = self.function_ids.and_then(|ids| {
                            operator_eq_candidates
                                .iter()
                                .find_map(|candidate| ids.get(candidate).copied())
                        });
                        let not_equal_function_id = self.function_ids.and_then(|ids| {
                            operator_ne_candidates
                                .iter()
                                .find_map(|candidate| ids.get(candidate).copied())
                        });

                        if let Some(function_id) = equal_function_id {
                            let window_size = 2;
                            let arg_base = self.allocator.alloc_window(window_size)?;
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::Mov,
                                    0,
                                    arg_base,
                                    reg_discriminant,
                                    0,
                                ),
                                Some(case_clause.span),
                            );
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::Mov,
                                    0,
                                    arg_base + 1,
                                    reg_case_value,
                                    0,
                                ),
                                Some(case_clause.span),
                            );
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallDirect,
                                    reg_compare,
                                    function_id as u16,
                                    arg_base,
                                    window_size as u8,
                                ),
                                Some(case_clause.span),
                            );
                            self.allocator.free_window(arg_base, window_size);
                        } else if let Some(function_id) = not_equal_function_id {
                            let window_size = 2;
                            let arg_base = self.allocator.alloc_window(window_size)?;
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::Mov,
                                    0,
                                    arg_base,
                                    reg_discriminant,
                                    0,
                                ),
                                Some(case_clause.span),
                            );
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::Mov,
                                    0,
                                    arg_base + 1,
                                    reg_case_value,
                                    0,
                                ),
                                Some(case_clause.span),
                            );
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallDirect,
                                    reg_compare,
                                    function_id as u16,
                                    arg_base,
                                    window_size as u8,
                                ),
                                Some(case_clause.span),
                            );
                            self.allocator.free_window(arg_base, window_size);
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Not, 0, reg_compare, reg_compare, 0),
                                Some(case_clause.span),
                            );
                        } else {
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::EqRef,
                                    0,
                                    reg_compare,
                                    reg_discriminant,
                                    reg_case_value,
                                ),
                                Some(case_clause.span),
                            );
                        }
                    } else {
                        let comparison_opcode = if matches!(
                            &discriminant_type,
                            Type::Class { symbol_name, .. }
                                if symbol_name == "std::string"
                                    || symbol_name == "std.string"
                                    || symbol_name == "string"
                        ) {
                            OpCode::EqString
                        } else if matches!(
                            &discriminant_type,
                            Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                        ) {
                            OpCode::EqI64
                        } else if discriminant_type.is_floating_point() {
                            OpCode::EqF64
                        } else if matches!(&discriminant_type, Type::Primitive(PrimitiveKind::Bool))
                        {
                            OpCode::EqBool
                        } else if matches!(
                            &case_clause.case_value,
                            Expression::Literal {
                                value: LiteralValue::Null,
                                ..
                            }
                        ) {
                            OpCode::EqRef
                        } else {
                            OpCode::EqI32
                        };

                        self.emit_instruction(
                            Instruction::format_a(
                                comparison_opcode,
                                0,
                                reg_compare,
                                reg_discriminant,
                                reg_case_value,
                            ),
                            Some(case_clause.span),
                        );
                    }

                    self.allocator.free_temp(reg_case_value);

                    let fail_jump = self.emit_instruction(
                        Instruction::format_c(OpCode::JumpIfFalse, 0, reg_compare, 0),
                        Some(case_clause.span),
                    );
                    self.allocator.free_temp(reg_compare);
                    case_failure_jumps.push(fail_jump as usize);
                }
            }

            if case_clause.statements.is_empty() {
                // Grouped empty label (e.g. `case 1: case 2: ...`): jump over subsequent case
                // comparisons directly into the next non-empty case or `default` body.
                let body_jump = self.emit_instruction(
                    Instruction::format_c(OpCode::Jump, 0, 0, 0),
                    Some(case_clause.span),
                );
                pending_body_jumps.push(body_jump as usize);
            } else {
                let body_start_pc = self.function.instructions.len() as i32;
                for body_jump_pc in pending_body_jumps.drain(..) {
                    let offset = body_start_pc - body_jump_pc as i32;
                    self.patch_jump_offset(body_jump_pc, offset);
                }
                for stmt in &case_clause.statements {
                    self.emit_statement(stmt, loop_stack)?;
                }

                let exit_jump = self.emit_instruction(
                    Instruction::format_c(OpCode::Jump, 0, 0, 0),
                    Some(case_clause.span),
                );
                case_exit_patches.push(exit_jump as usize);
            }

            let next_case_pc = self.function.instructions.len() as i32;
            for fail_jump_pc in case_failure_jumps {
                let offset = next_case_pc - fail_jump_pc as i32;
                self.patch_jump_offset(fail_jump_pc, offset);
            }
        }

        let default_or_exit_pc = self.function.instructions.len() as i32;
        for body_jump_pc in pending_body_jumps.drain(..) {
            let offset = default_or_exit_pc - body_jump_pc as i32;
            self.patch_jump_offset(body_jump_pc, offset);
        }

        if let Some(default_stmts) = &switch_stmt.default_case {
            for stmt in default_stmts {
                self.emit_statement(stmt, loop_stack)?;
            }
        }

        let exit_pc = self.function.instructions.len() as i32;
        for patch in case_exit_patches {
            let offset = exit_pc - patch as i32;
            self.patch_jump_offset(patch, offset);
        }

        switch_break_context = loop_stack.pop().unwrap_or_default();
        for break_pc in switch_break_context.break_patches {
            let offset = exit_pc - break_pc as i32;
            self.patch_jump_offset(break_pc, offset);
        }

        self.allocator.free_temp(reg_discriminant);
        Ok(())
    }

    /// Compiles a return statement (`return [expr];`).
    pub fn emit_return_statement(
        &mut self,
        return_stmt: &ReturnStatement,
    ) -> Result<(), EmissionError> {
        if let Some(return_value_expression) = &return_stmt.value {
            let mut evaluated_return_register =
                self.emit_expression(return_value_expression, None)?;
            let expression_type = self.query_type(return_value_expression.span());
            if self.function.return_type.is_floating_point()
                && (expression_type.is_signed_integer() || expression_type.is_unsigned_integer())
            {
                let converted_register = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(
                        OpCode::ConvIToF,
                        0,
                        converted_register,
                        evaluated_return_register,
                        0,
                    ),
                    Some(return_stmt.span),
                );
                self.allocator.free_temp(evaluated_return_register);
                evaluated_return_register = converted_register;
            }
            let aliases_local = matches!(
                return_value_expression,
                Expression::Identifier { span }
                    if self.allocator.lookup_local(self.get_lexeme(*span))
                        == Some(evaluated_return_register)
            );
            let is_returning_owned_local = aliases_local
                && self
                    .function
                    .reference_registers
                    .contains(&evaluated_return_register);

            // When `finally` blocks run after evaluating a local return variable, snapshot the
            // value into a dedicated temp (and retain if reference-typed) so `finally` mutations or
            // nullifications cannot corrupt or free the return value before `ReturnValue`.
            let mut active_return_register = evaluated_return_register;
            let mut allocated_saved_return_temp = false;
            if !self.finally_stack.is_empty() && aliases_local {
                let saved_reg = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, saved_reg, evaluated_return_register, 0),
                    Some(return_stmt.span),
                );
                if self
                    .function
                    .reference_registers
                    .contains(&evaluated_return_register)
                    || self
                        .query_type(return_value_expression.span())
                        .is_reference_type()
                    || matches!(
                        self.query_type(return_value_expression.span()),
                        solix_common::types::Type::GenericParameter { .. }
                            | solix_common::types::Type::Unresolved(_)
                    )
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::IncRef, 0, saved_reg, 0, 0),
                        Some(return_stmt.span),
                    );
                }
                active_return_register = saved_reg;
                allocated_saved_return_temp = true;
            } else if !is_returning_owned_local
                && (self
                    .query_type(return_value_expression.span())
                    .is_reference_type()
                    || matches!(
                        self.query_type(return_value_expression.span()),
                        solix_common::types::Type::GenericParameter { .. }
                            | solix_common::types::Type::Unresolved(_)
                    ))
                && self.is_borrowed_reference(return_value_expression)
            {
                self.emit_instruction(
                    Instruction::format_a(OpCode::IncRef, 0, evaluated_return_register, 0, 0),
                    Some(return_stmt.span),
                );
            }

            self.emit_active_finally_blocks_up_to(0)?;

            self.emit_return_arc_cleanup(Some(active_return_register));
            self.emit_instruction(
                Instruction::format_a(OpCode::ReturnValue, 0, active_return_register, 0, 0),
                Some(return_stmt.span),
            );
            if allocated_saved_return_temp {
                self.allocator.free_temp(active_return_register);
            }
            self.allocator.free_temp(evaluated_return_register);
        } else {
            self.emit_active_finally_blocks_up_to(0)?;
            self.emit_return_arc_cleanup(None);
            self.emit_instruction(
                Instruction::format_a(OpCode::Return, 0, 0, 0, 0),
                Some(return_stmt.span),
            );
        }
        Ok(())
    }

    /// Compiles a local variable declaration statement.
    pub fn emit_variable_declaration_statement(
        &mut self,
        var_decl: &VariableDeclarationStatement,
    ) -> Result<(), EmissionError> {
        let name = self.get_lexeme(var_decl.name_span).to_string();
        let tracked_name = name.clone();
        let local_reg = self.allocator.alloc_local(name.clone())?;

        let var_type = self.query_type(var_decl.span);
        let type_name_string = self.get_lexeme(var_decl.var_type.span()).to_string();

        self.function
            .local_variables
            .push(solix_common::bytecode::LocalVariableDebugInfo {
                register: local_reg,
                name,
                type_name: type_name_string.clone(),
                var_type: var_type.clone(),
                start_pc: self.function.instructions.len() as u32,
                end_pc: u32::MAX,
                is_parameter: false,
            });
        let is_ref = var_type.is_reference_type()
            || matches!(
                var_decl.var_type,
                crate::phases::parser::statements::types::TypeAnnotation::Named { .. }
                    | crate::phases::parser::statements::types::TypeAnnotation::Array { .. }
                    | crate::phases::parser::statements::types::TypeAnnotation::Tuple { .. }
                    | crate::phases::parser::statements::types::TypeAnnotation::FunctionPointer { .. }
            )
            || (!type_name_string.is_empty()
                && !matches!(
                    type_name_string.as_str(),
                    "int8"
                        | "int16"
                        | "int32"
                        | "int64"
                        | "uint8"
                        | "uint16"
                        | "uint32"
                        | "uint64"
                        | "float32"
                        | "float64"
                        | "bool"
                        | "char"
                        | "void"
                ));

        if is_ref {
            self.track_reference_register(local_reg);
        }

        if let Some(init) = &var_decl.initializer {
            let init_type = self.query_type(init.span());
            if var_type.is_floating_point()
                && (init_type.is_signed_integer() || init_type.is_unsigned_integer())
            {
                let temp_reg = self.emit_expression(init, None)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::ConvIToF, 0, local_reg, temp_reg, 0),
                    Some(var_decl.span),
                );
                self.allocator.free_temp(temp_reg);
            } else {
                self.emit_expression(init, Some(local_reg))?;
            }
            if is_ref && self.is_borrowed_reference(init) {
                self.emit_instruction(
                    Instruction::format_a(OpCode::IncRef, 0, local_reg, 0, 0),
                    Some(var_decl.span),
                );
            }
            if !self.unsafe_non_null_locals.contains(&tracked_name)
                && self.is_guaranteed_non_null(init)
            {
                self.non_null_locals.insert(tracked_name);
            }
        } else {
            self.emit_instruction(
                Instruction::format_b(OpCode::LoadNull, 0, local_reg, 0),
                Some(var_decl.span),
            );
        }
        Ok(())
    }

    /// Returns the local variable register index when `expr` is an identifier assignment or inc/dec targeting a local.
    pub(super) fn resolve_assignment_target_local(&self, expr: &Expression) -> Option<u16> {
        match expr {
            Expression::Assignment { target, .. } => {
                if let Expression::Identifier { span } = target.as_ref() {
                    let name = self.get_lexeme(*span);
                    return self.allocator.lookup_local(name);
                }
                None
            }
            Expression::Unary {
                operator:
                    solix_common::tokens::TokenKind::OperatorIncrement
                    | solix_common::tokens::TokenKind::OperatorDecrement,
                operand,
                ..
            } => {
                if let Expression::Identifier { span } = operand.as_ref() {
                    let name = self.get_lexeme(*span);
                    return self.allocator.lookup_local(name);
                }
                None
            }
            _ => None,
        }
    }

    pub fn patch_jump_offset(&mut self, instruction_index: usize, offset: i32) {
        if instruction_index < self.function.instructions.len() {
            let old = self.function.instructions[instruction_index];
            let raw_op = old.raw_opcode();
            if let Ok(opcode) = OpCode::from_u8(raw_op, instruction_index) {
                let flags = old.flags();
                let cond_reg = old.cond_reg();
                self.function.instructions[instruction_index] =
                    Instruction::format_c(opcode, flags, cond_reg, offset);
            }
        }
    }
}
