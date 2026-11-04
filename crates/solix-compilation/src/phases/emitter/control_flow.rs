//! Control-flow bytecode generation and jump backpatching engine.

#![forbid(unsafe_code)]

use crate::phases::emitter::emitter::Emitter;
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::control_flow::{
    BlockStatement, DoWhileStatement, ForStatement, IfStatement, ReturnStatement, SwitchStatement,
    VariableDeclarationStatement, WhileStatement,
};
use crate::phases::parser::statements::expressions::Expression;
use solix_common::bytecode::{Instruction, OpCode};
use solix_common::diagnostics::EmissionError;

/// Active loop context tracking forward break jumps and loop continue targets.
#[derive(Debug, Clone, Default)]
pub struct LoopContext {
    /// Jump instruction PCs that break out of the loop and need patching to loop exit.
    pub break_patches: Vec<usize>,
    /// Jump instruction PCs from continue statements needing patching to loop update/cond.
    pub continue_patches: Vec<usize>,
    /// Scope depth in the allocator when this loop context was pushed.
    pub scope_depth: usize,
}

impl<'a> Emitter<'a> {
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
                self.emit_active_finally_blocks()?;

                if let Some(active_loop) = loop_stack.last() {
                    let loop_depth = active_loop.scope_depth;
                    for reg in self.allocator.scopes_allocated_from(loop_depth) {
                        if self.function.reference_registers.contains(&reg) {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::DecRef, 0, reg, 0, 0),
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
                self.emit_active_finally_blocks()?;

                let patch_pc = self.emit_instruction(
                    Instruction::format_c(OpCode::Jump, 0, 0, 0),
                    Some(continue_stmt.span),
                );
                if let Some(active_loop) = loop_stack.last_mut() {
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
                let temp = self.emit_expression(&expr_stmt.expression, None)?;
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
                self.allocator.free_temp(temp);
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
        let r_cond = self.emit_expression(&assert_stmt.condition, None)?;
        let jump_patch_pc = self.emit_instruction(
            Instruction::format_c(OpCode::JumpIfTrue, 0, r_cond, 0),
            Some(assert_stmt.span),
        );
        self.allocator.free_temp(r_cond);

        let r_err = self.allocator.alloc_temp()?;
        if let Some(msg_expr) = &assert_stmt.message {
            let r_msg = self.emit_expression(msg_expr, Some(r_err))?;
            if r_msg != r_err {
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, r_err, r_msg, 0),
                    Some(assert_stmt.span),
                );
                self.allocator.free_temp(r_msg);
            }
        } else {
            let const_idx = self.pool.add_string("Assertion failed")?;
            self.emit_instruction(
                Instruction::format_b(OpCode::LoadConst, 0, r_err, const_idx as u32),
                Some(assert_stmt.span),
            );
        }
        self.emit_instruction(
            Instruction::format_a(OpCode::Throw, 0, r_err, 0, 0),
            Some(assert_stmt.span),
        );
        self.allocator.free_temp(r_err);

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
            let temp = self.emit_expression(update, None)?;
            self.allocator.free_temp(temp);
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
        let r_disc = self.emit_expression(&switch_stmt.discriminant, None)?;
        let mut case_jump_patches = Vec::new();
        let mut switch_break_ctx = LoopContext {
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.allocator.scope_depth(),
        };

        loop_stack.push(switch_break_ctx);

        for case in &switch_stmt.cases {
            let r_val = self.emit_expression(&case.case_value, None)?;
            let r_cmp = self.allocator.alloc_temp()?;
            self.emit_instruction(
                Instruction::format_a(OpCode::EqI32, 0, r_cmp, r_disc, r_val),
                Some(case.span),
            );
            self.allocator.free_temp(r_val);

            let next_case_jump = self.emit_instruction(
                Instruction::format_c(OpCode::JumpIfFalse, 0, r_cmp, 0),
                Some(case.span),
            );
            self.allocator.free_temp(r_cmp);

            for stmt in &case.statements {
                self.emit_statement(stmt, loop_stack)?;
            }

            let exit_jump = self.emit_instruction(
                Instruction::format_c(OpCode::Jump, 0, 0, 0),
                Some(case.span),
            );
            case_jump_patches.push(exit_jump as usize);

            let next_case_pc = self.function.instructions.len() as i32;
            let offset = next_case_pc - next_case_jump as i32;
            self.patch_jump_offset(next_case_jump as usize, offset);
        }

        if let Some(default_stmts) = &switch_stmt.default_case {
            for stmt in default_stmts {
                self.emit_statement(stmt, loop_stack)?;
            }
        }

        let exit_pc = self.function.instructions.len() as i32;
        for patch in case_jump_patches {
            let offset = exit_pc - patch as i32;
            self.patch_jump_offset(patch, offset);
        }

        switch_break_ctx = loop_stack.pop().unwrap_or_default();
        for break_pc in switch_break_ctx.break_patches {
            let offset = exit_pc - break_pc as i32;
            self.patch_jump_offset(break_pc, offset);
        }

        self.allocator.free_temp(r_disc);
        Ok(())
    }

    /// Compiles a return statement (`return [expr];`).
    pub fn emit_return_statement(
        &mut self,
        return_stmt: &ReturnStatement,
    ) -> Result<(), EmissionError> {
        self.emit_active_finally_blocks()?;

        if let Some(val) = &return_stmt.value {
            let r_val = self.emit_expression(val, None)?;
            if self.query_type(val.span()).is_reference_type() && self.is_borrowed_reference(val) {
                self.emit_instruction(
                    Instruction::format_a(OpCode::IncRef, 0, r_val, 0, 0),
                    Some(return_stmt.span),
                );
            }
            self.emit_return_arc_cleanup(Some(r_val));
            self.emit_instruction(
                Instruction::format_a(OpCode::ReturnValue, 0, r_val, 0, 0),
                Some(return_stmt.span),
            );
            self.allocator.free_temp(r_val);
        } else {
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
        let local_reg = self.allocator.alloc_local(name)?;

        let var_type = self.query_type(var_decl.span);
        let type_name = self.get_lexeme(var_decl.var_type.span());
        let is_ref = var_type.is_reference_type()
            || matches!(
                var_decl.var_type,
                crate::phases::parser::statements::types::TypeAnnotation::Named { .. }
                    | crate::phases::parser::statements::types::TypeAnnotation::Array { .. }
                    | crate::phases::parser::statements::types::TypeAnnotation::Tuple { .. }
                    | crate::phases::parser::statements::types::TypeAnnotation::FunctionPointer { .. }
            )
            || (!type_name.is_empty()
                && !matches!(
                    type_name,
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
            self.emit_expression(init, Some(local_reg))?;
            if is_ref && self.is_borrowed_reference(init) {
                self.emit_instruction(
                    Instruction::format_a(OpCode::IncRef, 0, local_reg, 0, 0),
                    Some(var_decl.span),
                );
            }
        } else {
            self.emit_instruction(
                Instruction::format_b(OpCode::LoadNull, 0, local_reg, 0),
                Some(var_decl.span),
            );
        }
        Ok(())
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
