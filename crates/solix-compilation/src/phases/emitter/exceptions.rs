//! Exception handling bytecode generation and zero-cost exception table construction.

#![forbid(unsafe_code)]

use crate::phases::emitter::control_flow::LoopContext;
use crate::phases::emitter::emitter::Emitter;
use crate::phases::parser::statements::control_flow::{ThrowStatement, TryStatement};
use solix_common::bytecode::{ExceptionHandlerEntry, Instruction, OpCode};
use solix_common::diagnostics::EmissionError;

impl<'a> Emitter<'a> {
    /// Compiles an exception-handling statement (`try { ... } catch (...) { ... } finally { ... }`).
    ///
    /// # Errors
    /// Returns `EmissionError` if register or jump offset bounds are exceeded.
    pub fn emit_try_statement(
        &mut self,
        try_stmt: &TryStatement,
        loop_stack: &mut Vec<LoopContext>,
    ) -> Result<(), EmissionError> {
        // Zero-cost: zero instructions emitted at try entry!
        let try_start_pc = self.function.instructions.len() as u32;

        if let Some(finally_block) = &try_stmt.finally_block {
            self.finally_stack.push(finally_block.clone());
        }

        self.emit_block_statement(&try_stmt.try_block, loop_stack)?;

        if try_stmt.finally_block.is_some() {
            self.finally_stack.pop();
        }

        let try_end_pc = self.function.instructions.len() as u32;

        // Normal execution path jumps over catch handlers to finally or exit
        let normal_jump = self.emit_instruction(
            Instruction::format_c(OpCode::Jump, 0, 0, 0),
            Some(try_stmt.span),
        );

        let mut catch_jumps = Vec::new();

        // Compile catch clauses
        for catch in &try_stmt.catch_clauses {
            let handler_pc = self.function.instructions.len() as u32;

            self.allocator.enter_scope();
            let var_name = self.get_lexeme(catch.variable_name).to_string();
            let catch_reg = self.allocator.alloc_local(var_name)?;
            self.track_reference_register(catch_reg);

            let type_sym = self.get_lexeme(catch.exception_type.span()).to_string();
            let catch_type_symbol = if type_sym.is_empty() {
                None
            } else {
                Some(type_sym)
            };

            self.function
                .exception_handlers
                .push(ExceptionHandlerEntry {
                    try_start_pc,
                    try_end_pc,
                    catch_type_symbol,
                    catch_reg,
                    handler_pc,
                    finally_pc: None,
                });

            if let Some(finally_block) = &try_stmt.finally_block {
                self.finally_stack.push(finally_block.clone());
            }

            self.emit_block_statement(&catch.block, loop_stack)?;

            if try_stmt.finally_block.is_some() {
                self.finally_stack.pop();
            }

            self.emit_block_exit_arc_cleanup();
            self.allocator.exit_scope();

            let exit_catch = self.emit_instruction(
                Instruction::format_c(OpCode::Jump, 0, 0, 0),
                Some(catch.span),
            );
            catch_jumps.push(exit_catch as usize);
        }

        // If there's a finally block, compile it
        if let Some(finally_block) = &try_stmt.finally_block {
            let finally_pc = self.function.instructions.len() as u32;

            // Update finally_pc on all handlers registered for this try block
            for handler in &mut self.function.exception_handlers {
                if handler.try_start_pc == try_start_pc && handler.try_end_pc == try_end_pc {
                    handler.finally_pc = Some(finally_pc);
                }
            }

            // Also register an unwinding handler for unhandled exceptions
            self.function
                .exception_handlers
                .push(ExceptionHandlerEntry {
                    try_start_pc,
                    try_end_pc,
                    catch_type_symbol: None,
                    catch_reg: 0,
                    handler_pc: finally_pc,
                    finally_pc: Some(finally_pc),
                });

            // Patch normal jumps to point to the normal finally execution
            let normal_target = self.function.instructions.len() as i32;
            let offset = normal_target - normal_jump as i32;
            self.patch_jump_offset(normal_jump as usize, offset);

            for cj in catch_jumps {
                let off = normal_target - cj as i32;
                self.patch_jump_offset(cj, off);
            }

            // Emit inline finally block
            self.emit_block_statement(finally_block, loop_stack)?;
        } else {
            // No finally block: patch normal jumps to point past all catch handlers
            let exit_pc = self.function.instructions.len() as i32;
            let offset = exit_pc - normal_jump as i32;
            self.patch_jump_offset(normal_jump as usize, offset);

            for cj in catch_jumps {
                let off = exit_pc - cj as i32;
                self.patch_jump_offset(cj, off);
            }
        }

        Ok(())
    }

    /// Compiles an exception throw statement (`throw expr;`).
    ///
    /// # Errors
    /// Returns `EmissionError` if expression compilation fails.
    pub fn emit_throw_statement(
        &mut self,
        throw_stmt: &ThrowStatement,
    ) -> Result<(), EmissionError> {
        let r_val = self.emit_expression(&throw_stmt.expression, None)?;
        self.emit_active_finally_blocks()?;
        self.emit_instruction(
            Instruction::format_a(OpCode::Throw, 0, r_val, 0, 0),
            Some(throw_stmt.span),
        );
        self.allocator.free_temp(r_val);
        Ok(())
    }
}
