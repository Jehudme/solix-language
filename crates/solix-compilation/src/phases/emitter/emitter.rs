//! Register-based bytecode emitter walking bound AST expressions and generating 64-bit instructions.

#![forbid(unsafe_code)]

use crate::phases::binder::layout::ClassLayout;
use crate::phases::emitter::constant_pool::ConstantPoolBuilder;
use crate::phases::emitter::register_allocator::RegisterAllocator;
use crate::phases::emitter::static_layout::StaticLayout;
use crate::phases::lexing::TokenKind;
use crate::phases::parser::statements::expressions::{Expression, LiteralValue};
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode};
use solix_common::diagnostics::EmissionError;
use solix_common::span::Span;
use solix_common::types::{PrimitiveKind, Type};
use std::collections::{HashMap, HashSet};

/// Bytecode emitter context for transforming expressions and statements into virtual register instructions.
pub struct Emitter<'ctx> {
    /// Function descriptor currently being compiled.
    pub function: &'ctx mut CompiledFunction,
    /// Register allocator managing local variables and temporary registers.
    pub allocator: &'ctx mut RegisterAllocator,
    /// Shared constant pool manager.
    pub pool: &'ctx mut ConstantPoolBuilder,
    /// Semantic types of AST nodes mapped by span (from semantic binding phase).
    pub node_types: Option<&'ctx HashMap<Span, Type>>,
    /// Source code contents indexed by source_id.
    pub sources: Option<&'ctx [String]>,
    /// Module static memory partition layout.
    pub static_layout: Option<&'ctx StaticLayout>,
    /// Class memory layouts indexed by class name.
    pub class_layouts: Option<&'ctx HashMap<String, ClassLayout>>,
    /// Function indices indexed by function signature symbol.
    pub function_ids: Option<&'ctx HashMap<String, u32>>,
    /// Non-virtual concrete method targets indexed by `(vtable_id, slot_index) -> func_id`.
    pub direct_method_targets: Option<&'ctx HashMap<(u32, usize), u32>>,
    /// Enum variant constant values indexed by fully qualified symbol (e.g. `Color::Red` -> 0).
    pub enum_variants: Option<&'ctx HashMap<String, u32>>,
    /// Synthesized functions (such as compiled lambda closure bodies) to append to the module.
    pub synthesized_functions: Option<&'ctx mut Vec<CompiledFunction>>,
    /// Active finally block statements for inlining during early returns, breaks, or continues.
    pub finally_stack: Vec<crate::phases::parser::statements::control_flow::BlockStatement>,
    /// Optional symbol of the enclosing class currently being emitted.
    pub enclosing_class: Option<String>,
    /// Local variable names initialized with guaranteed non-null references and never reassigned.
    pub non_null_locals: HashSet<String>,
    /// Local variable names that may be reassigned or declared nullable within the current function.
    pub unsafe_non_null_locals: HashSet<String>,
}

impl<'ctx> Emitter<'ctx> {
    /// Creates a new emitter instance for the target function.
    pub fn new(
        function: &'ctx mut CompiledFunction,
        allocator: &'ctx mut RegisterAllocator,
        pool: &'ctx mut ConstantPoolBuilder,
    ) -> Self {
        Self {
            function,
            allocator,
            pool,
            node_types: None,
            sources: None,
            static_layout: None,
            class_layouts: None,
            function_ids: None,
            direct_method_targets: None,
            enum_variants: None,
            synthesized_functions: None,
            finally_stack: Vec::new(),
            enclosing_class: None,
            non_null_locals: HashSet::new(),
            unsafe_non_null_locals: HashSet::new(),
        }
    }

    /// Registers a register index as an ARC reference-counted heap pointer.
    pub fn track_reference_register(&mut self, reg: u16) {
        if !self.function.reference_registers.contains(&reg) {
            self.function.reference_registers.push(reg);
            self.function.reference_registers.sort_unstable();
        }
    }

    /// Emits all active finally blocks in LIFO order for early jumps (returns, breaks, continues).
    ///
    /// # Errors
    /// Returns `EmissionError` if finally block compilation fails.
    pub fn emit_active_finally_blocks(&mut self) -> Result<(), EmissionError> {
        self.emit_active_finally_blocks_up_to(0)
    }

    /// Emits active finally blocks down to `target_depth` in LIFO order.
    ///
    /// Temporarily truncates `finally_stack` during recursive emission to prevent re-entrant emission
    /// and compiler stack overflows when finally blocks contain `return`, `break`, or `continue`.
    ///
    /// # Errors
    /// Returns `EmissionError` if finally block compilation fails.
    pub fn emit_active_finally_blocks_up_to(
        &mut self,
        target_depth: usize,
    ) -> Result<(), EmissionError> {
        let current_depth = self.finally_stack.len();
        if current_depth <= target_depth {
            return Ok(());
        }

        // Pop one block at a time so early control flow (`return`/`break`/`continue`) inside an
        // inner `finally` block still unwinds enclosing outer `finally` blocks on the stack.
        let mut popped_blocks = Vec::with_capacity(current_depth - target_depth);
        while self.finally_stack.len() > target_depth {
            if let Some(block_statement) = self.finally_stack.pop() {
                let mut inner_loop_stack = Vec::new();
                let emit_result =
                    self.emit_block_statement(&block_statement, &mut inner_loop_stack);
                popped_blocks.push(block_statement);
                if let Err(emission_error) = emit_result {
                    while let Some(saved_block) = popped_blocks.pop() {
                        self.finally_stack.push(saved_block);
                    }
                    return Err(emission_error);
                }
            }
        }

        while let Some(saved_block) = popped_blocks.pop() {
            self.finally_stack.push(saved_block);
        }
        Ok(())
    }

    /// Emits `DEC_REF` instructions for all reference registers in the current lexical scope upon exit.
    pub fn emit_block_exit_arc_cleanup(&mut self) {
        if let Some(allocated) = self.allocator.current_scope_allocated() {
            for reg in allocated {
                if self.function.reference_registers.contains(&reg) {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, reg, 0, 0),
                        None,
                    );
                }
            }
        }
    }

    /// Emits `DEC_REF` instructions for reference registers in current lexical scope upon exit, preserving `preserve_reg`.
    pub fn emit_block_exit_arc_cleanup_except(&mut self, preserve_reg: Option<u16>) {
        if let Some(allocated) = self.allocator.current_scope_allocated() {
            for reg in allocated {
                if self.function.reference_registers.contains(&reg) && Some(reg) != preserve_reg {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, reg, 0, 0),
                        None,
                    );
                }
            }
        }
    }

    /// Emits `DEC_REF` cleanup for all active reference variables across all scopes upon return.
    ///
    /// Preserves the returned register `ret_reg` without decrementing it.
    pub fn emit_return_arc_cleanup(&mut self, ret_reg: Option<u16>) {
        for reg in self.allocator.all_allocated_registers() {
            if self.function.reference_registers.contains(&reg) && Some(reg) != ret_reg {
                self.emit_instruction(Instruction::format_a(OpCode::DecRef, 0, reg, 0, 0), None);
            }
        }
    }

    /// Attaches synthesized functions buffer for collecting lambda bodies.
    #[must_use]
    pub fn with_synthesized_functions(
        mut self,
        synthesized_functions: &'ctx mut Vec<CompiledFunction>,
    ) -> Self {
        self.synthesized_functions = Some(synthesized_functions);
        self
    }

    /// Attaches class layouts for object instantiation and field offset resolution.
    #[must_use]
    pub fn with_class_layouts(mut self, class_layouts: &'ctx HashMap<String, ClassLayout>) -> Self {
        self.class_layouts = Some(class_layouts);
        self
    }

    /// Attaches function indices for direct call targets.
    #[must_use]
    pub fn with_function_ids(mut self, function_ids: &'ctx HashMap<String, u32>) -> Self {
        self.function_ids = Some(function_ids);
        self
    }

    /// Attaches enum variant constants for direct constant loading.
    #[must_use]
    pub fn with_enum_variants(mut self, enum_variants: &'ctx HashMap<String, u32>) -> Self {
        self.enum_variants = Some(enum_variants);
        self
    }

    /// Attaches module static memory partition layout.
    #[must_use]
    pub fn with_static_layout(mut self, static_layout: &'ctx StaticLayout) -> Self {
        self.static_layout = Some(static_layout);
        self
    }

    /// Attaches semantic node types from binder artifacts.
    #[must_use]
    pub fn with_node_types(mut self, node_types: &'ctx HashMap<Span, Type>) -> Self {
        self.node_types = Some(node_types);
        self
    }

    /// Attaches source files for lexeme extraction.
    #[must_use]
    pub fn with_sources(mut self, sources: &'ctx [String]) -> Self {
        self.sources = Some(sources);
        self
    }

    /// Attaches the enclosing class symbol name for resolving `this` member accesses.
    #[must_use]
    pub fn with_enclosing_class(mut self, enclosing_class: impl Into<String>) -> Self {
        self.enclosing_class = Some(enclosing_class.into());
        self
    }

    /// Attaches precomputed devirtualized concrete method targets `(vtable_id, slot) -> func_id`.
    #[must_use]
    pub fn with_direct_method_targets(
        mut self,
        direct_method_targets: &'ctx HashMap<(u32, usize), u32>,
    ) -> Self {
        self.direct_method_targets = Some(direct_method_targets);
        self
    }

    /// Pre-scans a function block to identify any local variable names that are reassigned or declared nullable.
    pub fn populate_reassigned_locals_from_block(
        &mut self,
        block: &crate::phases::parser::statements::control_flow::BlockStatement,
    ) {
        self.populate_reassigned_locals_from_statements(&block.statements);
    }

    /// Pre-scans statements to record local variable names that are reassigned or potentially nullable.
    pub fn populate_reassigned_locals_from_statements(
        &mut self,
        statements: &[crate::phases::parser::statements::Statement],
    ) {
        use crate::phases::parser::statements::Statement;
        for statement in statements {
            match statement {
                Statement::Block(inner_block) => {
                    self.populate_reassigned_locals_from_statements(&inner_block.statements);
                }
                Statement::If(if_stmt) => {
                    self.collect_unsafe_locals_in_expr(&if_stmt.condition);
                    self.populate_reassigned_locals_from_statements(&[if_stmt
                        .then_branch
                        .as_ref()
                        .clone()]);
                    if let Some(else_branch) = &if_stmt.else_branch {
                        self.populate_reassigned_locals_from_statements(&[else_branch
                            .as_ref()
                            .clone()]);
                    }
                }
                Statement::While(while_stmt) => {
                    self.collect_unsafe_locals_in_expr(&while_stmt.condition);
                    self.populate_reassigned_locals_from_statements(&[while_stmt
                        .body
                        .as_ref()
                        .clone()]);
                }
                Statement::DoWhile(do_while) => {
                    self.collect_unsafe_locals_in_expr(&do_while.condition);
                    self.populate_reassigned_locals_from_statements(&[do_while
                        .body
                        .as_ref()
                        .clone()]);
                }
                Statement::For(for_stmt) => {
                    if let Some(init) = &for_stmt.initializer {
                        self.populate_reassigned_locals_from_statements(&[init.as_ref().clone()]);
                    }
                    if let Some(cond) = &for_stmt.condition {
                        self.collect_unsafe_locals_in_expr(cond);
                    }
                    if let Some(update) = &for_stmt.update {
                        self.collect_unsafe_locals_in_expr(update);
                    }
                    self.populate_reassigned_locals_from_statements(&[for_stmt
                        .body
                        .as_ref()
                        .clone()]);
                }
                Statement::Switch(switch_stmt) => {
                    self.collect_unsafe_locals_in_expr(&switch_stmt.discriminant);
                    for case_clause in &switch_stmt.cases {
                        self.collect_unsafe_locals_in_expr(&case_clause.case_value);
                        self.populate_reassigned_locals_from_statements(&case_clause.statements);
                    }
                    if let Some(default_stmts) = &switch_stmt.default_case {
                        self.populate_reassigned_locals_from_statements(default_stmts);
                    }
                }
                Statement::Try(try_stmt) => {
                    self.populate_reassigned_locals_from_statements(&try_stmt.try_block.statements);
                    for clause in &try_stmt.catch_clauses {
                        let catch_name = self.get_lexeme(clause.variable_name).to_string();
                        self.unsafe_non_null_locals.insert(catch_name);
                        self.populate_reassigned_locals_from_statements(&clause.block.statements);
                    }
                    if let Some(finally_block) = &try_stmt.finally_block {
                        self.populate_reassigned_locals_from_statements(&finally_block.statements);
                    }
                }
                Statement::VariableDeclaration(var_decl) => {
                    let var_name = self.get_lexeme(var_decl.name_span).to_string();
                    let non_null_init = var_decl
                        .initializer
                        .as_ref()
                        .is_some_and(is_static_non_null_literal_or_alloc);
                    if !non_null_init {
                        self.unsafe_non_null_locals.insert(var_name);
                    }
                    if let Some(init_expr) = &var_decl.initializer {
                        self.collect_unsafe_locals_in_expr(init_expr);
                    }
                }
                Statement::Expression(expr_stmt) => {
                    self.collect_unsafe_locals_in_expr(&expr_stmt.expression);
                }
                Statement::Return(ret_stmt) => {
                    if let Some(ret_val) = &ret_stmt.value {
                        self.collect_unsafe_locals_in_expr(ret_val);
                    }
                }
                Statement::Throw(throw_stmt) => {
                    self.collect_unsafe_locals_in_expr(&throw_stmt.expression);
                }
                Statement::Assert(assert_stmt) => {
                    self.collect_unsafe_locals_in_expr(&assert_stmt.condition);
                }
                Statement::Exit(exit_stmt) => {
                    self.collect_unsafe_locals_in_expr(&exit_stmt.code);
                }
                _ => {}
            }
        }
    }

    fn collect_unsafe_locals_in_expr(&mut self, expr: &Expression) {
        match expr {
            Expression::Assignment { target, value, .. } => {
                if let Expression::Identifier { span } = target.as_ref() {
                    let target_name = self.get_lexeme(*span).to_string();
                    self.unsafe_non_null_locals.insert(target_name);
                }
                self.collect_unsafe_locals_in_expr(target);
                self.collect_unsafe_locals_in_expr(value);
            }
            Expression::Unary {
                operator, operand, ..
            } => {
                if matches!(
                    operator,
                    TokenKind::OperatorIncrement | TokenKind::OperatorDecrement
                ) && let Expression::Identifier { span } = operand.as_ref()
                {
                    let target_name = self.get_lexeme(*span).to_string();
                    self.unsafe_non_null_locals.insert(target_name);
                }
                self.collect_unsafe_locals_in_expr(operand);
            }
            Expression::Binary { left, right, .. } => {
                self.collect_unsafe_locals_in_expr(left);
                self.collect_unsafe_locals_in_expr(right);
            }
            Expression::MemberAccess { object, .. } => {
                self.collect_unsafe_locals_in_expr(object);
            }
            Expression::MethodCall {
                callee, arguments, ..
            } => {
                self.collect_unsafe_locals_in_expr(callee);
                for arg in arguments {
                    self.collect_unsafe_locals_in_expr(arg);
                }
            }
            Expression::ArrayAccess { array, index, .. } => {
                self.collect_unsafe_locals_in_expr(array);
                self.collect_unsafe_locals_in_expr(index);
            }
            Expression::ArrayCreation { size, .. } => {
                self.collect_unsafe_locals_in_expr(size);
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.collect_unsafe_locals_in_expr(element);
                }
            }
            Expression::NewInstance { arguments, .. } => {
                for arg in arguments {
                    self.collect_unsafe_locals_in_expr(arg);
                }
            }
            Expression::Cast { expression, .. } | Expression::Instanceof { expression, .. } => {
                self.collect_unsafe_locals_in_expr(expression);
            }
            Expression::Ternary {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_unsafe_locals_in_expr(condition);
                self.collect_unsafe_locals_in_expr(then_branch);
                self.collect_unsafe_locals_in_expr(else_branch);
            }
            _ => {}
        }
    }

    /// Returns `true` if `expr` is provably non-null at runtime (`this`, `new ...`, array/string allocs, or tracked non-null local).
    #[must_use]
    pub fn is_guaranteed_non_null(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier { span } => {
                let name = self.get_lexeme(*span);
                name == "this" || self.non_null_locals.contains(name)
            }
            _ => is_static_non_null_literal_or_alloc(expr),
        }
    }

    /// Resolves the exact string slice from source code for the given span.
    #[must_use]
    pub fn get_lexeme(&self, span: Span) -> &str {
        if let Some(sources) = self.sources {
            let src_idx = span.id as usize;
            if src_idx < sources.len() {
                let code = &sources[src_idx];
                let start = span.start as usize;
                let end = span.end as usize;
                if start <= end && end <= code.len() {
                    return &code[start..end];
                }
            }
        }
        ""
    }

    /// Emits a single instruction and records its source span in the function's source map.
    pub fn emit_instruction(&mut self, instruction: Instruction, span: Option<Span>) -> u32 {
        self.function.emit(instruction, span)
    }

    /// Recursively evaluates an expression, storing its result into `target_reg`.
    ///
    /// If `target_reg` is `None`, a temporary scratch register is allocated and returned.
    /// The caller is responsible for freeing the returned temporary register if applicable.
    ///
    /// # Errors
    /// Returns `EmissionError` if register limits or constant limits are exceeded.
    pub fn emit_expression(
        &mut self,
        expr: &Expression,
        target_reg: Option<u16>,
    ) -> Result<u16, EmissionError> {
        match expr {
            Expression::Literal { span, value } => {
                let dst = self.resolve_dst(target_reg)?;
                self.emit_literal(dst, value, *span)?;
                Ok(dst)
            }

            Expression::Identifier { span } => {
                let name = self.get_lexeme(*span).to_string();
                if let Some(local_reg) = self.allocator.lookup_local(&name) {
                    if let Some(dst) = target_reg {
                        if dst != local_reg {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Mov, 0, dst, local_reg, 0),
                                Some(*span),
                            );
                        }
                        Ok(dst)
                    } else {
                        Ok(local_reg)
                    }
                } else if let Some(&func_id) = self.function_ids.and_then(|ids| ids.get(&name)) {
                    // Top-level or static function referenced as a first-class function pointer
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_call(OpCode::PackClosure, dst, func_id as u16, 0, 0),
                        Some(*span),
                    );
                    Ok(dst)
                } else if let Some(slot) = self
                    .static_layout
                    .and_then(|layout| layout.lookup_slot(&name))
                {
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::GetStatic, 0, dst, slot),
                        Some(*span),
                    );
                    Ok(dst)
                } else {
                    let const_idx = self.pool.add_string(&name)?;
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::GetStaticGlobal, 0, dst, const_idx as u32),
                        Some(*span),
                    );
                    Ok(dst)
                }
            }

            Expression::Binary {
                span,
                left,
                operator,
                right,
            } => {
                let dst = self.resolve_dst(target_reg)?;
                self.emit_binary(dst, left, *operator, right, *span)?;
                Ok(dst)
            }

            Expression::Unary {
                span,
                operator,
                operand,
                is_prefix,
            } => {
                let dst = self.resolve_dst(target_reg)?;
                self.emit_unary(dst, *operator, operand, *is_prefix, *span)?;
                Ok(dst)
            }

            Expression::Assignment {
                span,
                target,
                operator,
                value,
            } => {
                let dst = self.resolve_dst(target_reg)?;
                self.emit_assignment(Some(dst), target, *operator, value, *span)?;
                Ok(dst)
            }

            Expression::Ternary {
                span,
                condition,
                then_branch,
                else_branch,
            } => {
                let dst = self.resolve_dst(target_reg)?;
                self.emit_ternary(dst, condition, then_branch, else_branch, *span)?;
                Ok(dst)
            }

            Expression::Cast {
                span,
                expression,
                target_type,
            } => {
                let dst = self.resolve_dst(target_reg)?;
                let src = self.emit_expression(expression, None)?;

                let expr_type = self.query_type(expression.span());
                let result_type = self.query_type(*span);

                let is_src_int = expr_type.is_signed_integer() || expr_type.is_unsigned_integer();
                let is_res_int =
                    result_type.is_signed_integer() || result_type.is_unsigned_integer();

                if expr_type.is_floating_point() && is_res_int {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::ConvFToI, 0, dst, src, 0),
                        Some(*span),
                    );
                } else if is_src_int && result_type.is_floating_point() {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::ConvIToF, 0, dst, src, 0),
                        Some(*span),
                    );
                } else if let Type::Class {
                    ref symbol_name, ..
                }
                | Type::Interface {
                    ref symbol_name, ..
                } = result_type
                {
                    let base_name = symbol_name.split('<').next().unwrap_or(symbol_name);
                    let simple = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
                    let vtable_id = self
                        .class_layouts
                        .and_then(|layouts| {
                            layouts
                                .get(symbol_name)
                                .or_else(|| layouts.get(base_name))
                                .or_else(|| layouts.get(simple))
                        })
                        .map(|layout| layout.vtable_id as u16)
                        .unwrap_or(0);
                    if vtable_id > 0 {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::CastCheck, 0, dst, src, vtable_id),
                            Some(*span),
                        );
                    } else {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, dst, src, 0),
                            Some(*span),
                        );
                    }
                } else if matches!(
                    result_type,
                    Type::GenericParameter { .. } | Type::Unresolved(_)
                ) {
                    let type_lexeme = self.get_lexeme(target_type.span());
                    let generic_substitutions = self.get_enclosing_generic_substitutions();
                    let resolved_name = generic_substitutions
                        .get(type_lexeme)
                        .map(|name| name.as_str())
                        .unwrap_or(type_lexeme);
                    let type_tag = self.resolve_type_tag(resolved_name);
                    if type_tag != 0x80 && type_tag > 0 {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::CastCheck, type_tag, dst, src, 0),
                            Some(*span),
                        );
                    } else {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, dst, src, 0),
                            Some(*span),
                        );
                    }
                } else {
                    let type_tag = self.resolve_type_tag_for_type(&result_type);
                    self.emit_instruction(
                        Instruction::format_a(OpCode::CastCheck, type_tag, dst, src, 0),
                        Some(*span),
                    );
                }

                if target_reg.is_none() || src != dst {
                    self.allocator.free_temp(src);
                }
                Ok(dst)
            }

            Expression::MemberAccess {
                span,
                object,
                member_span,
                is_scope_resolution,
            } if *is_scope_resolution => {
                let dst = self.resolve_dst(target_reg)?;
                let obj_name = self.get_lexeme(object.span());
                let member_name = self.get_lexeme(*member_span);
                let full_sym = format!("{obj_name}::{member_name}");

                if let Some(&variant_val) = self.enum_variants.and_then(|ev| ev.get(&full_sym)) {
                    self.emit_instruction(
                        Instruction::format_b_signed(OpCode::LoadI32, 0, dst, variant_val as i32),
                        Some(*span),
                    );
                    return Ok(dst);
                }

                if let Some(&func_id) = self.function_ids.and_then(|ids| ids.get(&full_sym)) {
                    self.emit_instruction(
                        Instruction::format_call(OpCode::PackClosure, dst, func_id as u16, 0, 0),
                        Some(*span),
                    );
                    return Ok(dst);
                }

                if let Some(slot) = self
                    .static_layout
                    .and_then(|layout| layout.lookup_slot(&full_sym))
                {
                    self.emit_instruction(
                        Instruction::format_b(OpCode::GetStatic, 0, dst, slot),
                        Some(*span),
                    );
                } else {
                    let const_idx = self.pool.add_string(&full_sym)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::GetStaticGlobal, 0, dst, const_idx as u32),
                        Some(*span),
                    );
                }
                Ok(dst)
            }

            Expression::MemberAccess {
                span,
                object,
                member_span,
                is_scope_resolution: false,
            } => {
                let obj_name = self.get_lexeme(object.span());
                let member_name = self.get_lexeme(*member_span).to_string();
                let full_sym = format!("{obj_name}::{member_name}");

                if let Some(&variant_val) = self.enum_variants.and_then(|ev| ev.get(&full_sym)) {
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_b_signed(OpCode::LoadI32, 0, dst, variant_val as i32),
                        Some(*span),
                    );
                    return Ok(dst);
                }

                if let Some(&func_id) = self.function_ids.and_then(|ids| ids.get(&full_sym)) {
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_call(OpCode::PackClosure, dst, func_id as u16, 0, 0),
                        Some(*span),
                    );
                    return Ok(dst);
                }

                if let Some(slot) = self
                    .static_layout
                    .and_then(|layout| layout.lookup_slot(&full_sym))
                {
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::GetStatic, 0, dst, slot),
                        Some(*span),
                    );
                    return Ok(dst);
                }
                if member_name == "length"
                    && (matches!(self.query_type(object.span()), Type::Array { .. })
                        || matches!(self.query_type(object.span()), Type::Tuple(_))
                        || matches!(
                            self.query_type(object.span()),
                            Type::Primitive(PrimitiveKind::TuplePrimitive)
                        )
                        || matches!(
                            self.query_type(object.span()),
                            Type::Class { ref symbol_name, .. }
                                if symbol_name == "std.string"
                                    || symbol_name == "std::string"
                                    || symbol_name == "string"
                        ))
                {
                    let r_obj = self.emit_expression(object, None)?;
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_a(OpCode::ArrayLength, 0, dst, r_obj, 0),
                        Some(*span),
                    );
                    self.allocator.free_temp(r_obj);
                    return Ok(dst);
                }
                if let Ok(slot_idx) = member_name.parse::<u16>() {
                    let is_tuple = matches!(self.query_type(object.span()), Type::Tuple(_));
                    if is_tuple {
                        let r_obj = self.emit_expression(object, None)?;
                        let dst = self.resolve_dst(target_reg)?;
                        self.emit_instruction(
                            Instruction::format_a(OpCode::GetTuple, 0, dst, r_obj, slot_idx),
                            Some(*span),
                        );
                        self.allocator.free_temp(r_obj);
                        return Ok(dst);
                    }
                }
                let offset = self.resolve_field_offset(object.span(), &member_name);
                let field_tag = self.resolve_field_type_tag(object.span(), &member_name);
                let weak_flag = if self.is_weak_field(object.span(), &member_name) {
                    solix_common::bytecode::instruction::FLAG_WEAK
                } else {
                    0
                };
                let flags = (field_tag << 1) | weak_flag;
                let r_obj = self.emit_expression(object, None)?;
                let dst = self.resolve_dst(target_reg)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::GetField, flags, dst, r_obj, offset),
                    Some(*span),
                );
                if self.query_type(object.span()).is_reference_type()
                    && matches!(
                        object.as_ref(),
                        Expression::MethodCall { .. } | Expression::NewInstance { .. }
                    )
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, r_obj, 0, 0),
                        Some(*span),
                    );
                }
                self.allocator.free_temp(r_obj);
                Ok(dst)
            }

            Expression::ArrayAccess { span, array, index } => {
                let r_arr = self.emit_expression(array, None)?;
                let r_idx = self.emit_expression(index, None)?;
                let dst = self.resolve_dst(target_reg)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::GetArray, 0, dst, r_arr, r_idx),
                    Some(*span),
                );
                self.allocator.free_temp(r_arr);
                self.allocator.free_temp(r_idx);
                Ok(dst)
            }

            Expression::ArrayCreation {
                span,
                element_type,
                size,
            } => {
                let r_size = self.emit_expression(size, None)?;
                let dst = self.resolve_dst(target_reg)?;
                let element_lexeme = self.get_lexeme(element_type.span());
                let generic_substitutions = self.get_enclosing_generic_substitutions();
                let mut resolved_element_name = element_lexeme.to_string();
                if !generic_substitutions.is_empty() {
                    if let Some(substituted) = generic_substitutions.get(element_lexeme) {
                        resolved_element_name = substituted.clone();
                    } else if resolved_element_name.contains('<')
                        && let Some((base_name, args_part)) = resolved_element_name.split_once('<')
                        && let Some(args_string) = args_part.strip_suffix('>')
                    {
                        let substituted_args: Vec<String> =
                            split_top_level_generic_arguments(args_string)
                                .into_iter()
                                .map(|arg| {
                                    generic_substitutions
                                        .get(arg)
                                        .cloned()
                                        .unwrap_or_else(|| arg.to_string())
                                })
                                .collect();
                        resolved_element_name =
                            format!("{base_name}<{}>", substituted_args.join(", "));
                    }
                }
                let array_type = self.query_type(*span);
                let (type_tag, vtable_id) = match array_type {
                    Type::Array {
                        element_type: ref array_element_type,
                        depth: 1,
                    } => {
                        let inner_tag = self.resolve_type_tag_for_type(array_element_type);
                        let vtable = self.resolve_vtable_id_for_type(array_element_type);
                        if inner_tag == 0x80 {
                            let tag = self.resolve_type_tag(&resolved_element_name);
                            let vtable_from_name = self.resolve_vtable_id(&resolved_element_name);
                            (tag, if vtable > 0 { vtable } else { vtable_from_name })
                        } else {
                            (inner_tag, vtable)
                        }
                    }
                    Type::Array { depth, .. } if depth > 1 => (0x80, 0),
                    _ => {
                        let tag = self.resolve_type_tag(&resolved_element_name);
                        let vtable = self.resolve_vtable_id(&resolved_element_name);
                        (tag, vtable)
                    }
                };
                let elem_size: u16 = match type_tag {
                    1 | 5 | 11 | 12 => 1,
                    2 | 6 => 2,
                    3 | 7 | 9 => 4,
                    _ => 8,
                };
                let src2_val: u16 = if type_tag >= 0x80 && vtable_id > 0 {
                    vtable_id as u16
                } else {
                    elem_size
                };
                self.emit_instruction(
                    Instruction::format_a(OpCode::AllocArray, type_tag, dst, r_size, src2_val),
                    Some(*span),
                );
                self.allocator.free_temp(r_size);
                Ok(dst)
            }

            Expression::ArrayLiteral { span, elements } => {
                let r_len = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_b(OpCode::LoadI32, 0, r_len, elements.len() as u32),
                    Some(*span),
                );
                let dst = self.resolve_dst(target_reg)?;
                let array_type = self.query_type(*span);
                let type_tag = match array_type {
                    Type::Array {
                        ref element_type,
                        depth,
                    } => {
                        if depth > 1 {
                            0x80
                        } else {
                            self.resolve_type_tag_for_type(element_type)
                        }
                    }
                    _ => 0x80,
                };
                let elem_size: u16 = match type_tag {
                    1 | 5 | 11 | 12 => 1,
                    2 | 6 => 2,
                    3 | 7 | 9 => 4,
                    _ => 8,
                };
                let src2_val: u16 = if type_tag >= 0x80 {
                    let vtable_id = match array_type {
                        Type::Array {
                            ref element_type,
                            depth: 1,
                        } => self.resolve_vtable_id_for_type(element_type),
                        _ => 0,
                    };
                    if vtable_id > 0 {
                        vtable_id as u16
                    } else {
                        elem_size
                    }
                } else {
                    elem_size
                };
                self.emit_instruction(
                    Instruction::format_a(OpCode::AllocArray, type_tag, dst, r_len, src2_val),
                    Some(*span),
                );
                self.allocator.free_temp(r_len);

                let r_idx = self.allocator.alloc_temp()?;
                for (idx, elem) in elements.iter().enumerate() {
                    let r_val = self.emit_expression(elem, None)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadI32, 0, r_idx, idx as u32),
                        Some(*span),
                    );
                    self.emit_instruction(
                        Instruction::format_a(OpCode::SetArray, 0, dst, r_idx, r_val),
                        Some(*span),
                    );
                    if self.query_type(elem.span()).is_reference_type()
                        && !self.is_borrowed_reference(elem)
                    {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::DecRef, 0, r_val, 0, 0),
                            Some(*span),
                        );
                    }
                    self.allocator.free_temp(r_val);
                }
                self.allocator.free_temp(r_idx);
                Ok(dst)
            }

            Expression::TupleCreation { span, elements } => {
                let dst = self.resolve_dst(target_reg)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::AllocTuple, 0, dst, elements.len() as u16, 0),
                    Some(*span),
                );
                let r_idx = self.allocator.alloc_temp()?;
                for (idx, elem) in elements.iter().enumerate() {
                    let r_val = self.emit_expression(elem, None)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadI32, 0, r_idx, idx as u32),
                        Some(*span),
                    );
                    self.emit_instruction(
                        Instruction::format_a(OpCode::SetArray, 0, dst, r_idx, r_val),
                        Some(*span),
                    );
                    if self.query_type(elem.span()).is_reference_type()
                        && !self.is_borrowed_reference(elem)
                    {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::DecRef, 0, r_val, 0, 0),
                            Some(*span),
                        );
                    }
                    self.allocator.free_temp(r_val);
                }
                self.allocator.free_temp(r_idx);
                Ok(dst)
            }

            Expression::Block {
                span,
                statements,
                result,
            } => {
                self.allocator.enter_scope();
                let mut inner_loop_stack = Vec::new();
                for stmt in statements {
                    self.emit_statement(stmt, &mut inner_loop_stack)?;
                }
                let dst = if let Some(res) = result {
                    let val_reg = self.emit_expression(res, None)?;
                    self.emit_block_exit_arc_cleanup_except(Some(val_reg));
                    if let Some(target) = target_reg {
                        if target != val_reg {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Mov, 0, target, val_reg, 0),
                                Some(*span),
                            );
                            target
                        } else {
                            val_reg
                        }
                    } else {
                        val_reg
                    }
                } else {
                    self.emit_block_exit_arc_cleanup();
                    let val_reg = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_a(OpCode::LoadNull, 0, val_reg, 0, 0),
                        Some(*span),
                    );
                    val_reg
                };
                self.allocator.exit_scope();
                Ok(dst)
            }

            Expression::NewInstance {
                span,
                instance_type,
                arguments,
            } => {
                let mut resolved_class_name = match self.query_type(*span).unwrap_alias() {
                    Type::Class {
                        symbol_name,
                        type_arguments,
                    } => {
                        if !type_arguments.is_empty() && !symbol_name.contains('<') {
                            format!(
                                "{symbol_name}<{}>",
                                type_arguments
                                    .iter()
                                    .map(|type_arg| type_arg.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        } else {
                            symbol_name.clone()
                        }
                    }
                    _ => self.get_lexeme(instance_type.span()).to_string(),
                };
                let generic_substitutions = self.get_enclosing_generic_substitutions();
                if !generic_substitutions.is_empty() {
                    if let Some(substituted_name) = generic_substitutions.get(&resolved_class_name)
                    {
                        resolved_class_name = substituted_name.clone();
                    } else if resolved_class_name.contains('<')
                        && let Some((base_name, args_part)) = resolved_class_name.split_once('<')
                        && let Some(args_string) = args_part.strip_suffix('>')
                    {
                        let substituted_arguments: Vec<String> =
                            split_top_level_generic_arguments(args_string)
                                .into_iter()
                                .map(|argument| {
                                    generic_substitutions
                                        .get(argument)
                                        .cloned()
                                        .unwrap_or_else(|| argument.to_string())
                                })
                                .collect();
                        resolved_class_name =
                            format!("{base_name}<{}>", substituted_arguments.join(", "));
                    }
                }
                if resolved_class_name.contains("unresolved")
                    && let Some(enclosing_class) = self.enclosing_class.as_deref()
                    && let Some((enclosing_base, enclosing_args)) = enclosing_class.split_once('<')
                    && let Some((instance_base, _)) = resolved_class_name.split_once('<')
                    && instance_base.rsplit(['.', ':']).next()
                        == enclosing_base.rsplit(['.', ':']).next()
                {
                    resolved_class_name = format!("{enclosing_base}<{enclosing_args}");
                }
                let base_name = resolved_class_name
                    .split('<')
                    .next()
                    .unwrap_or(&resolved_class_name);
                let simple_base = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
                let generic_part = resolved_class_name
                    .split_once('<')
                    .map(|(_, args)| format!("<{args}"))
                    .unwrap_or_default();
                let simple = format!("{simple_base}{generic_part}");
                let enclosing_pkg = self.enclosing_class.as_deref().and_then(|cls| {
                    let base = cls.split('<').next().unwrap_or(cls);
                    base.rsplit_once("::").map(|(pkg, _)| pkg)
                });
                let qualified_simple = enclosing_pkg
                    .map(|pkg| format!("{pkg}::{simple}"))
                    .unwrap_or_default();
                let layout_info = self.class_layouts.and_then(|layouts| {
                    layouts
                        .get(&resolved_class_name)
                        .or_else(|| layouts.get(&simple))
                        .or_else(|| {
                            if !qualified_simple.is_empty() {
                                layouts.get(&qualified_simple)
                            } else {
                                None
                            }
                        })
                });

                let dst = self.resolve_dst(target_reg)?;
                if let Some(layout) = layout_info
                    && !layout.is_extern
                {
                    self.emit_instruction(
                        Instruction::format_a(
                            OpCode::AllocObject,
                            0,
                            dst,
                            layout.vtable_id as u16,
                            layout.instance_size_bytes as u16,
                        ),
                        Some(*span),
                    );
                } else {
                    let const_idx = self.pool.add_string(&resolved_class_name)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::AllocObjectExtern, 0, dst, const_idx as u32),
                        Some(*span),
                    );
                }

                let total_args = 1 + arguments.len();
                let arg_base = self.allocator.alloc_window(total_args)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base, dst, 0),
                    Some(*span),
                );

                for (idx, arg) in arguments.iter().enumerate() {
                    self.emit_expression(arg, Some(arg_base + 1 + idx as u16))?;
                }

                let arity = arguments.len();
                let argument_signatures = arguments
                    .iter()
                    .map(|argument| self.query_type(argument.span()).to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let argument_simple_signatures = arguments
                    .iter()
                    .map(|argument| {
                        let type_name = self.query_type(argument.span()).to_string();
                        type_name
                            .rsplit([':', '.'])
                            .next()
                            .unwrap_or(&type_name)
                            .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let ctor_sym_signature = if argument_signatures.is_empty() {
                    format!("{resolved_class_name}::constructor")
                } else {
                    format!("{resolved_class_name}::constructor({argument_signatures})")
                };
                let ctor_sym_signature_simple = if argument_simple_signatures.is_empty() {
                    format!("{resolved_class_name}::constructor")
                } else {
                    format!("{resolved_class_name}::constructor({argument_simple_signatures})")
                };
                let ctor_sym_simple_signature = if argument_signatures.is_empty() {
                    format!("{simple}::constructor")
                } else {
                    format!("{simple}::constructor({argument_signatures})")
                };
                let ctor_sym_simple_signature_simple = if argument_simple_signatures.is_empty() {
                    format!("{simple}::constructor")
                } else {
                    format!("{simple}::constructor({argument_simple_signatures})")
                };
                let ctor_sym_arity = format!("{resolved_class_name}::constructor/{arity}");
                let ctor_sym_simple_arity = format!("{simple}::constructor/{arity}");
                let ctor_sym = format!("{resolved_class_name}::constructor");
                let ctor_sym_simple = format!("{simple}::constructor");
                let ctor_id = self
                    .function_ids
                    .and_then(|ids| {
                        ids.get(&ctor_sym_signature)
                            .or_else(|| ids.get(&ctor_sym_signature_simple))
                            .or_else(|| ids.get(&ctor_sym_simple_signature))
                            .or_else(|| ids.get(&ctor_sym_simple_signature_simple))
                            .or_else(|| {
                                if !qualified_simple.is_empty() {
                                    ids.get(&format!("{qualified_simple}::constructor({argument_signatures})"))
                                        .or_else(|| ids.get(&format!("{qualified_simple}::constructor({argument_simple_signatures})")))
                                } else {
                                    None
                                }
                            })
                            .or_else(|| ids.get(&ctor_sym_arity))
                            .or_else(|| ids.get(&ctor_sym_simple_arity))
                            .or_else(|| {
                                if !qualified_simple.is_empty() {
                                    ids.get(&format!("{qualified_simple}::constructor/{arity}"))
                                } else {
                                    None
                                }
                            })
                            .or_else(|| ids.get(&ctor_sym))
                            .or_else(|| ids.get(&ctor_sym_simple))
                            .or_else(|| {
                                if !qualified_simple.is_empty() {
                                    ids.get(&format!("{qualified_simple}::constructor"))
                                } else {
                                    None
                                }
                            })
                    })
                    .copied();

                let is_extern_class = layout_info.as_ref().is_some_and(|layout| layout.is_extern)
                    || layout_info.is_none();
                let dummy_dst = self.allocator.alloc_temp()?;
                if !is_extern_class && let Some(cid) = ctor_id {
                    self.emit_instruction(
                        Instruction::format_call(
                            OpCode::CallDirect,
                            dummy_dst,
                            cid as u16,
                            arg_base,
                            total_args as u8,
                        ),
                        Some(*span),
                    );
                } else if is_extern_class || arity > 0 {
                    let sym_idx = self.pool.add_string(&ctor_sym_arity)?;
                    self.emit_instruction(
                        Instruction::format_call(
                            OpCode::CallExtern,
                            dummy_dst,
                            sym_idx as u16,
                            arg_base,
                            total_args as u8,
                        ),
                        Some(*span),
                    );
                }
                self.allocator.free_temp(dummy_dst);
                for (idx, arg) in arguments.iter().enumerate() {
                    if self.is_reference_expr(arg) && !self.is_borrowed_reference(arg) {
                        self.emit_instruction(
                            Instruction::format_a(
                                OpCode::DecRef,
                                0,
                                arg_base + 1 + idx as u16,
                                0,
                                0,
                            ),
                            Some(*span),
                        );
                    }
                }
                self.allocator.free_window(arg_base, total_args);
                Ok(dst)
            }

            Expression::MethodCall {
                span,
                callee,
                generic_arguments,
                arguments,
            } => {
                if let Expression::MemberAccess {
                    object,
                    member_span,
                    is_scope_resolution,
                    ..
                } = callee.as_ref()
                {
                    let object_name = self.get_lexeme(object.span()).to_string();
                    let method_name = self.get_lexeme(*member_span).to_string();
                    let generic_suffix = if !generic_arguments.is_empty() {
                        let args_str: Vec<String> = generic_arguments
                            .iter()
                            .map(|argument| self.get_lexeme(argument.span()).to_string())
                            .collect();
                        format!("<{}>", args_str.join(", "))
                    } else {
                        String::new()
                    };
                    let symbol_full = format!("{object_name}::{method_name}{generic_suffix}");
                    let simple_object = object_name
                        .rsplit(['.', ':'])
                        .next()
                        .unwrap_or(&object_name);
                    let symbol_simple = format!("{simple_object}::{method_name}{generic_suffix}");

                    let is_package_or_class_call =
                        matches!(object.as_ref(), Expression::Identifier { .. })
                            && self.allocator.lookup_local(&object_name).is_none()
                            && (self.class_layouts.is_some_and(|layouts| {
                                layouts.contains_key(&object_name)
                                    || layouts.contains_key(simple_object)
                            }) || !self.query_type(object.span()).is_reference_type());

                    if object_name == "super" {
                        let super_type = self.query_type(object.span());
                        let (super_name, simple_super) = match &super_type {
                            Type::Class { symbol_name, .. } => {
                                let simple =
                                    symbol_name.rsplit(['.', ':']).next().unwrap_or(symbol_name);
                                (symbol_name.as_str(), simple)
                            }
                            _ => ("", ""),
                        };
                        let super_method_sym = format!("{super_name}::{method_name}");
                        let simple_super_sym = format!("{simple_super}::{method_name}");
                        let func_id_opt = self
                            .function_ids
                            .and_then(|ids| {
                                ids.get(&super_method_sym)
                                    .or_else(|| ids.get(&simple_super_sym))
                            })
                            .copied();

                        let total_args = 1 + arguments.len();
                        let arg_base = self.allocator.alloc_window(total_args)?;
                        // Pass `this` (register 0) in argument position 0
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, arg_base, 0, 0),
                            Some(*span),
                        );
                        for (arg_index, argument) in arguments.iter().enumerate() {
                            self.emit_expression(argument, Some(arg_base + 1 + arg_index as u16))?;
                        }
                        let dst = self.resolve_dst(target_reg)?;
                        if let Some(func_id) = func_id_opt {
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallDirect,
                                    dst,
                                    func_id as u16,
                                    arg_base,
                                    total_args as u8,
                                ),
                                Some(*span),
                            );
                        } else {
                            let mangled_sym = format!("{super_method_sym}/{total_args}");
                            let sym_idx = self.pool.add_string(&mangled_sym)?;
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallExtern,
                                    dst,
                                    sym_idx as u16,
                                    arg_base,
                                    total_args as u8,
                                ),
                                Some(*span),
                            );
                        }
                        for (arg_index, argument) in arguments.iter().enumerate() {
                            if self.is_reference_expr(argument)
                                && !self.is_borrowed_reference(argument)
                            {
                                self.emit_instruction(
                                    Instruction::format_a(
                                        OpCode::DecRef,
                                        0,
                                        arg_base + 1 + arg_index as u16,
                                        0,
                                        0,
                                    ),
                                    Some(*span),
                                );
                            }
                        }
                        self.allocator.free_window(arg_base, total_args);
                        return Ok(dst);
                    }

                    let is_static_fn = *is_scope_resolution
                        || is_package_or_class_call
                        || self
                            .function_ids
                            .map(|ids| {
                                ids.contains_key(&symbol_full) || ids.contains_key(&symbol_simple)
                            })
                            .unwrap_or(false);

                    if is_static_fn {
                        // Static method call Type::method(args) or Type.method(args)
                        let argument_signatures = arguments
                            .iter()
                            .map(|argument| self.query_type(argument.span()).to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        let argument_simple_signatures = arguments
                            .iter()
                            .map(|argument| {
                                let type_name = self.query_type(argument.span()).to_string();
                                type_name
                                    .rsplit([':', '.'])
                                    .next()
                                    .unwrap_or(&type_name)
                                    .to_string()
                            })
                            .collect::<Vec<_>>()
                            .join(",");

                        let candidate_full_typed = format!("{symbol_full}({argument_signatures})");
                        let candidate_full_simple_typed =
                            format!("{symbol_full}({argument_simple_signatures})");
                        let candidate_simple_typed =
                            format!("{symbol_simple}({argument_signatures})");
                        let candidate_simple_simple_typed =
                            format!("{symbol_simple}({argument_simple_signatures})");

                        let raw_symbol_full = format!("{object_name}::{method_name}");
                        let raw_symbol_simple = format!("{simple_object}::{method_name}");
                        let func_id_opt = self
                            .function_ids
                            .and_then(|ids| {
                                ids.get(&candidate_full_typed)
                                    .or_else(|| ids.get(&candidate_full_simple_typed))
                                    .or_else(|| ids.get(&candidate_simple_typed))
                                    .or_else(|| ids.get(&candidate_simple_simple_typed))
                                    .or_else(|| ids.get(&symbol_full))
                                    .or_else(|| ids.get(&symbol_simple))
                                    .or_else(|| ids.get(&raw_symbol_full))
                                    .or_else(|| ids.get(&raw_symbol_simple))
                            })
                            .copied();

                        let arg_base = self.allocator.alloc_window(arguments.len())?;
                        for (arg_index, argument) in arguments.iter().enumerate() {
                            self.emit_expression(argument, Some(arg_base + arg_index as u16))?;
                        }
                        let dst = self.resolve_dst(target_reg)?;
                        if let Some(func_id) = func_id_opt {
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallDirect,
                                    dst,
                                    func_id as u16,
                                    arg_base,
                                    arguments.len() as u8,
                                ),
                                Some(*span),
                            );
                        } else {
                            let arity = arguments.len();
                            let mangled_sym = format!("{symbol_full}/{arity}");
                            let sym_idx = self.pool.add_string(&mangled_sym)?;
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallExtern,
                                    dst,
                                    sym_idx as u16,
                                    arg_base,
                                    arguments.len() as u8,
                                ),
                                Some(*span),
                            );
                        }
                        for (arg_index, argument) in arguments.iter().enumerate() {
                            if self.is_reference_expr(argument)
                                && !self.is_borrowed_reference(argument)
                            {
                                self.emit_instruction(
                                    Instruction::format_a(
                                        OpCode::DecRef,
                                        0,
                                        arg_base + arg_index as u16,
                                        0,
                                        0,
                                    ),
                                    Some(*span),
                                );
                            }
                        }
                        self.allocator.free_window(arg_base, arguments.len());
                        Ok(dst)
                    } else {
                        let method_name = self.get_lexeme(*member_span).to_string();
                        let is_field = {
                            let obj_type = self.query_type(object.span());
                            let cls_name = match &obj_type {
                                Type::Class { symbol_name, .. } => Some(symbol_name.as_str()),
                                _ => self.enclosing_class.as_deref(),
                            };
                            if let Some(cls) = cls_name {
                                let simple = cls.rsplit(['.', ':']).next().unwrap_or(cls);
                                self.class_layouts
                                    .and_then(|layouts| {
                                        layouts.get(cls).or_else(|| layouts.get(simple))
                                    })
                                    .map(|layout| layout.field_offsets.contains_key(&method_name))
                                    .unwrap_or(false)
                            } else {
                                false
                            }
                        };

                        if is_field {
                            let r_callee = self.emit_expression(callee, None)?;
                            let arg_base = self.allocator.alloc_window(arguments.len())?;
                            for (idx, arg) in arguments.iter().enumerate() {
                                self.emit_expression(arg, Some(arg_base + idx as u16))?;
                            }
                            let dst = self.resolve_dst(target_reg)?;
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallCallable,
                                    dst,
                                    r_callee,
                                    arg_base,
                                    arguments.len() as u8,
                                ),
                                Some(*span),
                            );
                            for (idx, arg) in arguments.iter().enumerate() {
                                if self.is_reference_expr(arg) && !self.is_borrowed_reference(arg) {
                                    self.emit_instruction(
                                        Instruction::format_a(
                                            OpCode::DecRef,
                                            0,
                                            arg_base + idx as u16,
                                            0,
                                            0,
                                        ),
                                        Some(*span),
                                    );
                                }
                            }
                            self.allocator.free_window(arg_base, arguments.len());
                            self.allocator.free_temp(r_callee);
                            return Ok(dst);
                        }

                        // Virtual / interface / devirtualized method call object.method(args)
                        let arg_types: Vec<Type> = arguments
                            .iter()
                            .map(|arg| self.query_type(arg.span()))
                            .collect();
                        let (slot, is_interface) =
                            self.resolve_method_dispatch(object.span(), &method_name, &arg_types);

                        let direct_func_id = if !is_interface
                            && self.is_guaranteed_non_null(object)
                            && let Some(vtable_id) = self.resolve_receiver_vtable_id(object.span())
                        {
                            self.direct_method_targets
                                .and_then(|targets| targets.get(&(vtable_id, slot as usize)))
                                .copied()
                        } else {
                            None
                        };

                        let total_args = 1 + arguments.len();
                        let arg_base = self.allocator.alloc_window(total_args)?;
                        self.emit_expression(object, Some(arg_base))?;

                        for (idx, arg) in arguments.iter().enumerate() {
                            self.emit_expression(arg, Some(arg_base + 1 + idx as u16))?;
                        }

                        let dst = self.resolve_dst(target_reg)?;
                        if let Some(func_id) = direct_func_id {
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallDirect,
                                    dst,
                                    func_id as u16,
                                    arg_base,
                                    total_args as u8,
                                ),
                                Some(*span),
                            );
                        } else {
                            let op = if is_interface {
                                OpCode::CallInterface
                            } else {
                                OpCode::CallVirtual
                            };
                            self.emit_instruction(
                                Instruction::format_call(op, dst, slot, arg_base, total_args as u8),
                                Some(*span),
                            );
                        }
                        if self.is_reference_expr(object) && !self.is_borrowed_reference(object) {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::DecRef, 0, arg_base, 0, 0),
                                Some(*span),
                            );
                        }
                        for (idx, arg) in arguments.iter().enumerate() {
                            if self.is_reference_expr(arg) && !self.is_borrowed_reference(arg) {
                                self.emit_instruction(
                                    Instruction::format_a(
                                        OpCode::DecRef,
                                        0,
                                        arg_base + 1 + idx as u16,
                                        0,
                                        0,
                                    ),
                                    Some(*span),
                                );
                            }
                        }
                        self.allocator.free_window(arg_base, total_args);
                        Ok(dst)
                    }
                } else if let Expression::Identifier { span: id_span } = callee.as_ref() {
                    let func_name = self.get_lexeme(*id_span).to_string();
                    let is_local = self.allocator.lookup_local(&func_name).is_some();
                    let generic_suffix = if !generic_arguments.is_empty() {
                        let args_str: Vec<String> = generic_arguments
                            .iter()
                            .map(|argument| self.get_lexeme(argument.span()).to_string())
                            .collect();
                        format!("<{}>", args_str.join(", "))
                    } else {
                        String::new()
                    };
                    let specialized_name = format!("{func_name}{generic_suffix}");
                    let func_id_opt = self
                        .function_ids
                        .and_then(|ids| {
                            ids.get(&specialized_name)
                                .or_else(|| ids.get(&func_name))
                                .or_else(|| {
                                    self.enclosing_class.as_ref().and_then(|cls| {
                                        let simple = cls.rsplit(['.', ':']).next().unwrap_or(cls);
                                        let qual1 = format!("{cls}::{specialized_name}");
                                        let qual2 = format!("{simple}::{specialized_name}");
                                        let qual3 = format!("{cls}::{func_name}");
                                        let qual4 = format!("{simple}::{func_name}");
                                        ids.get(&qual1)
                                            .or_else(|| ids.get(&qual2))
                                            .or_else(|| ids.get(&qual3))
                                            .or_else(|| ids.get(&qual4))
                                    })
                                })
                        })
                        .copied();

                    if is_local {
                        let r_callee =
                            self.allocator.lookup_local(&func_name).ok_or_else(|| {
                                EmissionError::SymbolNotFound {
                                    symbol: func_name.clone(),
                                }
                            })?;
                        let arg_base = self.allocator.alloc_window(arguments.len())?;
                        for (idx, arg) in arguments.iter().enumerate() {
                            self.emit_expression(arg, Some(arg_base + idx as u16))?;
                        }
                        let dst = self.resolve_dst(target_reg)?;
                        self.emit_instruction(
                            Instruction::format_call(
                                OpCode::CallCallable,
                                dst,
                                r_callee,
                                arg_base,
                                arguments.len() as u8,
                            ),
                            Some(*span),
                        );
                        for (idx, arg) in arguments.iter().enumerate() {
                            if self.is_reference_expr(arg) && !self.is_borrowed_reference(arg) {
                                self.emit_instruction(
                                    Instruction::format_a(
                                        OpCode::DecRef,
                                        0,
                                        arg_base + idx as u16,
                                        0,
                                        0,
                                    ),
                                    Some(*span),
                                );
                            }
                        }
                        self.allocator.free_window(arg_base, arguments.len());
                        Ok(dst)
                    } else {
                        let arg_base = self.allocator.alloc_window(arguments.len())?;
                        for (idx, arg) in arguments.iter().enumerate() {
                            self.emit_expression(arg, Some(arg_base + idx as u16))?;
                        }
                        let dst = self.resolve_dst(target_reg)?;
                        if let Some(func_id) = func_id_opt {
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallDirect,
                                    dst,
                                    func_id as u16,
                                    arg_base,
                                    arguments.len() as u8,
                                ),
                                Some(*span),
                            );
                        } else {
                            let arity = arguments.len();
                            let mangled_name = format!("{func_name}/{arity}");
                            let sym_idx = self.pool.add_string(&mangled_name)?;
                            self.emit_instruction(
                                Instruction::format_call(
                                    OpCode::CallExtern,
                                    dst,
                                    sym_idx as u16,
                                    arg_base,
                                    arguments.len() as u8,
                                ),
                                Some(*span),
                            );
                        }
                        for (idx, arg) in arguments.iter().enumerate() {
                            if self.is_reference_expr(arg) && !self.is_borrowed_reference(arg) {
                                self.emit_instruction(
                                    Instruction::format_a(
                                        OpCode::DecRef,
                                        0,
                                        arg_base + idx as u16,
                                        0,
                                        0,
                                    ),
                                    Some(*span),
                                );
                            }
                        }
                        self.allocator.free_window(arg_base, arguments.len());
                        Ok(dst)
                    }
                } else {
                    let r_callee = self.emit_expression(callee, None)?;
                    let arg_base = self.allocator.alloc_window(arguments.len())?;
                    for (idx, arg) in arguments.iter().enumerate() {
                        self.emit_expression(arg, Some(arg_base + idx as u16))?;
                    }
                    let dst = self.resolve_dst(target_reg)?;
                    self.emit_instruction(
                        Instruction::format_call(
                            OpCode::CallCallable,
                            dst,
                            r_callee,
                            arg_base,
                            arguments.len() as u8,
                        ),
                        Some(*span),
                    );
                    for (idx, arg) in arguments.iter().enumerate() {
                        if self.is_reference_expr(arg) && !self.is_borrowed_reference(arg) {
                            self.emit_instruction(
                                Instruction::format_a(
                                    OpCode::DecRef,
                                    0,
                                    arg_base + idx as u16,
                                    0,
                                    0,
                                ),
                                Some(*span),
                            );
                        }
                    }
                    self.allocator.free_window(arg_base, arguments.len());
                    self.allocator.free_temp(r_callee);
                    Ok(dst)
                }
            }

            Expression::Lambda { span, .. } => {
                let lambda_expr = expr;
                let captured_vars = crate::phases::emitter::closures::find_free_variables(
                    lambda_expr,
                    self.allocator,
                    self.sources,
                );

                let base_func_id = self
                    .function_ids
                    .and_then(|func_map| func_map.values().max().copied())
                    .map_or(1, |max_id| max_id + 1);

                let mut synth = self.synthesized_functions.take();
                let (lambda_id, placeholder_idx) = if let Some(ref mut funcs) = synth {
                    let idx = funcs.len();
                    let id = base_func_id + idx as u32;
                    funcs.push(CompiledFunction::new(
                        format!("<lambda_{id}>"),
                        span.id,
                        0,
                        0,
                    ));
                    (id, Some(idx))
                } else {
                    (base_func_id, None)
                };

                let lambda_func = crate::phases::emitter::closures::synthesize_lambda_function(
                    lambda_id,
                    lambda_expr,
                    &captured_vars,
                    self.pool,
                    self.node_types,
                    self.sources,
                    self.class_layouts,
                    self.function_ids,
                    self.direct_method_targets,
                    self.static_layout,
                    synth.as_deref_mut(),
                )?;

                if let Some(ref mut funcs) = synth {
                    if let Some(idx) = placeholder_idx {
                        funcs[idx] = lambda_func;
                    } else {
                        funcs.push(lambda_func);
                    }
                }
                self.synthesized_functions = synth;

                let dst = self.resolve_dst(target_reg)?;
                let capture_count = captured_vars.len();

                if capture_count > 0 {
                    let cap_base = self.allocator.alloc_window(capture_count)?;
                    for (idx, var_name) in captured_vars.iter().enumerate() {
                        let r_var = self.allocator.lookup_local(var_name).ok_or_else(|| {
                            EmissionError::SymbolNotFound {
                                symbol: var_name.clone(),
                            }
                        })?;
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, cap_base + idx as u16, r_var, 0),
                            Some(*span),
                        );
                    }
                    self.emit_instruction(
                        Instruction::format_call(
                            OpCode::PackClosure,
                            dst,
                            lambda_id as u16,
                            cap_base,
                            capture_count as u8,
                        ),
                        Some(*span),
                    );
                    self.allocator.free_window(cap_base, capture_count);
                } else {
                    self.emit_instruction(
                        Instruction::format_call(OpCode::PackClosure, dst, lambda_id as u16, 0, 0),
                        Some(*span),
                    );
                }
                Ok(dst)
            }

            Expression::Instanceof {
                span,
                expression,
                target_type,
            } => {
                let type_name = self.get_lexeme(target_type.span()).to_string();
                let dst = self.resolve_dst(target_reg)?;

                // Handle tuple element indexing: tuple[i] instanceof type
                if let Expression::ArrayAccess { array, index, .. } = &**expression
                    && matches!(self.query_type(array.span()), Type::Tuple(_))
                {
                    let r_tuple = self.emit_expression(array, None)?;
                    let r_idx = self.emit_expression(index, None)?;
                    let type_tag = self.resolve_type_tag(&type_name);
                    self.emit_instruction(
                        Instruction::format_a(
                            OpCode::TupleInstanceof,
                            type_tag,
                            dst,
                            r_tuple,
                            r_idx,
                        ),
                        Some(*span),
                    );
                    self.allocator.free_temp(r_idx);
                    self.allocator.free_temp(r_tuple);
                    return Ok(dst);
                }

                let simple = type_name.rsplit(['.', ':']).next().unwrap_or(&type_name);
                let vtable_id = self
                    .class_layouts
                    .and_then(|layouts| layouts.get(&type_name).or_else(|| layouts.get(simple)))
                    .map(|layout| layout.vtable_id)
                    .unwrap_or(0);

                let r_obj = self.emit_expression(expression, None)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Instanceof, 0, dst, r_obj, vtable_id as u16),
                    Some(*span),
                );
                self.allocator.free_temp(r_obj);
                Ok(dst)
            }

            Expression::Sizeof { span, target_type } => {
                let dst = self.resolve_dst(target_reg)?;
                let query = self.query_type(target_type.span());
                let byte_size = match query {
                    Type::Class {
                        ref symbol_name, ..
                    } => {
                        let short_name = symbol_name
                            .rsplit(['.', ':'])
                            .next()
                            .unwrap_or(symbol_name.as_str());
                        self.resolve_type_byte_size(short_name)
                    }
                    _ => {
                        let mut type_name = self.get_lexeme(target_type.span());
                        if type_name == "T" || type_name == "K" || type_name == "V" {
                            if let Some(enc) = &self.enclosing_class
                                && let Some(open) = enc.find('<')
                                && let Some(close) = enc.rfind('>')
                            {
                                type_name = enc[open + 1..close].trim();
                            } else if let Some(open) = self.function.symbol.find('<')
                                && let Some(close) = self.function.symbol.rfind('>')
                            {
                                type_name = self.function.symbol[open + 1..close].trim();
                            }
                        }
                        self.resolve_type_byte_size(type_name)
                    }
                };
                self.emit_instruction(
                    Instruction::format_b_signed(OpCode::LoadI32, 0, dst, byte_size as i32),
                    Some(*span),
                );
                Ok(dst)
            }

            Expression::Default { span, target_type } => {
                let dst = self.resolve_dst(target_reg)?;

                let mut extracted = None;
                if let Some(enc) = &self.enclosing_class
                    && let Some(open) = enc.find('<')
                    && let Some(close) = enc.rfind('>')
                {
                    let arg_str = enc[open + 1..close].trim();
                    let lex = self.get_lexeme(target_type.span());
                    if lex == "T" || lex == "K" || lex == "V" || !enc.starts_with(lex) {
                        extracted = Some(arg_str.to_string());
                    }
                } else if let Some(open) = self.function.symbol.find('<')
                    && let Some(close) = self.function.symbol.rfind('>')
                {
                    let arg_str = self.function.symbol[open + 1..close].trim();
                    extracted = Some(arg_str.to_string());
                }

                if let Some(type_name) = extracted {
                    match type_name.as_str() {
                        "f64" | "float64" | "f32" | "float32" => {
                            let const_idx = self.pool.add_f64(0.0)?;
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                                Some(*span),
                            );
                        }
                        "bool" => {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadBool, 0, dst, 0),
                                Some(*span),
                            );
                        }
                        "int64" | "uint64" | "i64" | "u64" => {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadI64, 0, dst, 0),
                                Some(*span),
                            );
                        }
                        "int8" | "uint8" | "i8" | "u8" | "char" | "int16" | "uint16" | "i16"
                        | "u16" | "int32" | "uint32" | "i32" | "u32" => {
                            self.emit_instruction(
                                Instruction::format_b_signed(OpCode::LoadI32, 0, dst, 0),
                                Some(*span),
                            );
                        }
                        _ => {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadNull, 0, dst, 0),
                                Some(*span),
                            );
                        }
                    }
                } else if let Some(sem_type) = self.node_types.and_then(|types| types.get(span)) {
                    if sem_type.is_floating_point() {
                        let const_idx = self.pool.add_f64(0.0)?;
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                            Some(*span),
                        );
                    } else if matches!(sem_type, Type::Primitive(PrimitiveKind::Bool)) {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadBool, 0, dst, 0),
                            Some(*span),
                        );
                    } else if matches!(
                        sem_type,
                        Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                    ) {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadI64, 0, dst, 0),
                            Some(*span),
                        );
                    } else if sem_type.is_signed_integer()
                        || sem_type.is_unsigned_integer()
                        || matches!(sem_type, Type::Primitive(PrimitiveKind::Char))
                    {
                        self.emit_instruction(
                            Instruction::format_b_signed(OpCode::LoadI32, 0, dst, 0),
                            Some(*span),
                        );
                    } else {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadNull, 0, dst, 0),
                            Some(*span),
                        );
                    }
                } else {
                    let type_name = self.get_lexeme(target_type.span());
                    match type_name {
                        "f64" | "float64" | "f32" | "float32" => {
                            let const_idx = self.pool.add_f64(0.0)?;
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                                Some(*span),
                            );
                        }
                        "bool" => {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadBool, 0, dst, 0),
                                Some(*span),
                            );
                        }
                        "int8" | "uint8" | "i8" | "u8" | "char" | "int16" | "uint16" | "i16"
                        | "u16" | "int32" | "uint32" | "i32" | "u32" | "int64" | "uint64"
                        | "i64" | "u64" => {
                            self.emit_instruction(
                                Instruction::format_b_signed(OpCode::LoadI32, 0, dst, 0),
                                Some(*span),
                            );
                        }
                        _ => {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadNull, 0, dst, 0),
                                Some(*span),
                            );
                        }
                    }
                }
                Ok(dst)
            }

            _ => {
                // Other expression variants (calls, arrays, objects) are handled in subsequent phases
                let dst = self.resolve_dst(target_reg)?;
                Ok(dst)
            }
        }
    }

    fn emit_literal(
        &mut self,
        dst: u16,
        value: &LiteralValue,
        span: Span,
    ) -> Result<(), EmissionError> {
        let resolved_type = self.query_type(span);
        match value {
            LiteralValue::Int(literal_int) => {
                if matches!(
                    resolved_type.unwrap_alias(),
                    Type::Primitive(PrimitiveKind::Float32 | PrimitiveKind::Float64)
                ) {
                    let const_idx = self.pool.add_f64(*literal_int as f64)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadF64, 0, dst, const_idx as u32),
                        Some(span),
                    );
                } else if matches!(
                    resolved_type.unwrap_alias(),
                    Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                ) {
                    let const_idx = self.pool.add_i64(*literal_int)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                        Some(span),
                    );
                } else if *literal_int >= i32::MIN as i64 && *literal_int <= i32::MAX as i64 {
                    self.emit_instruction(
                        Instruction::format_b_signed(OpCode::LoadI32, 0, dst, *literal_int as i32),
                        Some(span),
                    );
                } else {
                    let const_idx = self.pool.add_i64(*literal_int)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                        Some(span),
                    );
                }
            }
            LiteralValue::UInt(literal_uint) => {
                if matches!(
                    resolved_type.unwrap_alias(),
                    Type::Primitive(PrimitiveKind::Float32 | PrimitiveKind::Float64)
                ) {
                    let const_idx = self.pool.add_f64(*literal_uint as f64)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadF64, 0, dst, const_idx as u32),
                        Some(span),
                    );
                } else if matches!(
                    resolved_type.unwrap_alias(),
                    Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                ) {
                    let const_idx = self.pool.add_i64(*literal_uint as i64)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                        Some(span),
                    );
                } else if *literal_uint <= i32::MAX as u64 {
                    self.emit_instruction(
                        Instruction::format_b_signed(OpCode::LoadI32, 0, dst, *literal_uint as i32),
                        Some(span),
                    );
                } else {
                    let const_idx = self.pool.add_i64(*literal_uint as i64)?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                        Some(span),
                    );
                }
            }
            LiteralValue::Float(literal_float) => {
                let const_idx = self.pool.add_f64(*literal_float)?;
                self.emit_instruction(
                    Instruction::format_b(OpCode::LoadF64, 0, dst, const_idx as u32),
                    Some(span),
                );
            }
            LiteralValue::String(literal_str) => {
                let const_idx = self.pool.add_string(literal_str)?;
                self.emit_instruction(
                    Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                    Some(span),
                );
            }
            LiteralValue::Char(literal_char) => {
                self.emit_instruction(
                    Instruction::format_b(OpCode::LoadI32, 0, dst, *literal_char as u32),
                    Some(span),
                );
            }
            LiteralValue::Bool(literal_bool) => {
                self.emit_instruction(
                    Instruction::format_b(
                        OpCode::LoadBool,
                        0,
                        dst,
                        if *literal_bool { 1 } else { 0 },
                    ),
                    Some(span),
                );
            }
            LiteralValue::Null => {
                self.emit_instruction(
                    Instruction::format_b(OpCode::LoadNull, 0, dst, 0),
                    Some(span),
                );
            }
        }
        Ok(())
    }

    fn emit_binary(
        &mut self,
        dst: u16,
        left: &Expression,
        operator: TokenKind,
        right: &Expression,
        span: Span,
    ) -> Result<(), EmissionError> {
        // Short-circuit logical operations
        if matches!(operator, TokenKind::OperatorLogicalAnd) {
            self.emit_expression(left, Some(dst))?;
            let jump_pc = self.emit_instruction(
                Instruction::format_c(OpCode::JumpIfFalse, 0, dst, 0),
                Some(span),
            );
            self.emit_expression(right, Some(dst))?;
            let target_pc = self.function.instructions.len() as i32;
            let offset = target_pc - jump_pc as i32;
            self.function.instructions[jump_pc as usize] =
                Instruction::format_c(OpCode::JumpIfFalse, 0, dst, offset);
            return Ok(());
        }

        if matches!(operator, TokenKind::OperatorLogicalOr) {
            self.emit_expression(left, Some(dst))?;
            let jump_pc = self.emit_instruction(
                Instruction::format_c(OpCode::JumpIfTrue, 0, dst, 0),
                Some(span),
            );
            self.emit_expression(right, Some(dst))?;
            let target_pc = self.function.instructions.len() as i32;
            let offset = target_pc - jump_pc as i32;
            self.function.instructions[jump_pc as usize] =
                Instruction::format_c(OpCode::JumpIfTrue, 0, dst, offset);
            return Ok(());
        }

        let left_type = self.query_type(left.span());
        let right_type = self.query_type(right.span());
        let is_null_comparison = matches!(
            operator,
            TokenKind::OperatorEqual | TokenKind::OperatorNotEqual
        ) && (left_type == Type::Null
            || right_type == Type::Null
            || matches!(
                left,
                Expression::Literal {
                    value: LiteralValue::Null,
                    ..
                }
            )
            || matches!(
                right,
                Expression::Literal {
                    value: LiteralValue::Null,
                    ..
                }
            ));

        if !is_null_comparison
            && let Type::Class {
                ref symbol_name, ..
            } = left_type
        {
            let simple_class_name = symbol_name.rsplit(['.', ':']).next().unwrap_or(symbol_name);
            let dot_class_name = symbol_name.replace("::", ".");
            let colon_class_name = symbol_name.replace('.', "::");
            let operator_symbol = operator.as_static_str();
            let right_type_name = self.query_type(right.span()).to_string();
            let right_simple_type = right_type_name
                .rsplit([':', '.'])
                .next()
                .unwrap_or(&right_type_name);
            let right_alt_type = if right_simple_type == "string" {
                "String"
            } else if right_simple_type == "String" {
                "string"
            } else {
                right_simple_type
            };

            let mut operator_candidates = Vec::new();
            for class_prefix in [
                &colon_class_name,
                &dot_class_name,
                symbol_name,
                &simple_class_name.to_string(),
            ] {
                for argument_type in [
                    &right_type_name,
                    &right_simple_type.to_string(),
                    &right_alt_type.to_string(),
                ] {
                    let candidate =
                        format!("{class_prefix}::operator_{operator_symbol}({argument_type})");
                    if !operator_candidates.contains(&candidate) {
                        operator_candidates.push(candidate);
                    }
                }
            }
            // Unsuffixed fallback only if no typed overload matches
            for class_prefix in [
                &colon_class_name,
                &dot_class_name,
                symbol_name,
                &simple_class_name.to_string(),
            ] {
                let candidate = format!("{class_prefix}::operator_{operator_symbol}");
                if !operator_candidates.contains(&candidate) {
                    operator_candidates.push(candidate);
                }
            }
            let func_id = self
                .function_ids
                .and_then(|ids| {
                    operator_candidates
                        .iter()
                        .find_map(|candidate| ids.get(candidate))
                })
                .copied();
            if let Some(callee_id) = func_id {
                let register_left = self.emit_expression(left, None)?;
                let register_right = self.emit_expression(right, None)?;
                let window_size = 2; // this, rhs
                let arg_base = self.allocator.alloc_window(window_size)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base, register_left, 0),
                    Some(span),
                );
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base + 1, register_right, 0),
                    Some(span),
                );
                self.emit_instruction(
                    Instruction::format_call(
                        OpCode::CallDirect,
                        dst,
                        callee_id as u16,
                        arg_base,
                        window_size as u8,
                    ),
                    Some(span),
                );
                if self.query_type(left.span()).is_reference_type()
                    && !self.is_borrowed_reference(left)
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, register_left, 0, 0),
                        Some(span),
                    );
                }
                if self.query_type(right.span()).is_reference_type()
                    && !self.is_borrowed_reference(right)
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, register_right, 0, 0),
                        Some(span),
                    );
                }
                self.allocator.free_window(arg_base, window_size);
                self.allocator.free_temp(register_left);
                self.allocator.free_temp(register_right);
                return Ok(());
            }
        }

        if !is_null_comparison && matches!(operator, TokenKind::OperatorPlus) {
            let left_type = self.query_type(left.span());
            let right_type = self.query_type(right.span());
            let is_string_type = |type_info: &Type| {
                matches!(
                    type_info,
                    Type::Class { symbol_name, .. }
                        if symbol_name == "std::string"
                            || symbol_name == "std.string"
                            || symbol_name == "string"
                )
            };
            if is_string_type(&left_type) || is_string_type(&right_type) {
                let r_left = self.emit_expression(left, None)?;
                let r_right = self.emit_expression(right, None)?;
                let window_size = 2;
                let arg_base = self.allocator.alloc_window(window_size)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base, r_left, 0),
                    Some(span),
                );
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base + 1, r_right, 0),
                    Some(span),
                );
                let sym_idx = self.pool.add_string("__builtin_str_concat")?;
                self.emit_instruction(
                    Instruction::format_call(
                        OpCode::CallSymbol,
                        dst,
                        sym_idx,
                        arg_base,
                        window_size as u8,
                    ),
                    Some(span),
                );
                if self.query_type(left.span()).is_reference_type()
                    && !self.is_borrowed_reference(left)
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, r_left, 0, 0),
                        Some(span),
                    );
                }
                if self.query_type(right.span()).is_reference_type()
                    && !self.is_borrowed_reference(right)
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, r_right, 0, 0),
                        Some(span),
                    );
                }
                self.allocator.free_window(arg_base, window_size);
                self.allocator.free_temp(r_left);
                self.allocator.free_temp(r_right);
                return Ok(());
            }
        }

        let mut register_left = self.emit_expression(left, None)?;
        let mut register_right = self.emit_expression(right, None)?;

        let left_type = self.query_type(left.span());
        let right_type = self.query_type(right.span());
        let operand_type = if left_type.is_floating_point() || right_type.is_floating_point() {
            Type::Primitive(solix_common::types::PrimitiveKind::Float64)
        } else if left_type == Type::Null {
            right_type.clone()
        } else {
            left_type.clone()
        };

        if operand_type.is_floating_point() && !left_type.is_floating_point() {
            let converted_reg = self.allocator.alloc_temp()?;
            self.emit_instruction(
                Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, register_left, 0),
                Some(span),
            );
            self.allocator.free_temp(register_left);
            register_left = converted_reg;
        }
        if operand_type.is_floating_point() && !right_type.is_floating_point() {
            let converted_reg = self.allocator.alloc_temp()?;
            self.emit_instruction(
                Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, register_right, 0),
                Some(span),
            );
            self.allocator.free_temp(register_right);
            register_right = converted_reg;
        }
        let opcode = if is_null_comparison {
            match operator {
                TokenKind::OperatorEqual => OpCode::EqRef,
                TokenKind::OperatorNotEqual => OpCode::NeqRef,
                _ => self.select_binary_opcode(operator, &operand_type),
            }
        } else {
            self.select_binary_opcode(operator, &operand_type)
        };

        self.emit_instruction(
            Instruction::format_a(opcode, 0, dst, register_left, register_right),
            Some(span),
        );

        if self.query_type(left.span()).is_reference_type() && !self.is_borrowed_reference(left) {
            self.emit_instruction(
                Instruction::format_a(OpCode::DecRef, 0, register_left, 0, 0),
                Some(span),
            );
        }
        if self.query_type(right.span()).is_reference_type() && !self.is_borrowed_reference(right) {
            self.emit_instruction(
                Instruction::format_a(OpCode::DecRef, 0, register_right, 0, 0),
                Some(span),
            );
        }

        self.allocator.free_temp(register_left);
        self.allocator.free_temp(register_right);
        Ok(())
    }

    fn select_compound_opcode(&self, operator: TokenKind, operand_type: &Type) -> OpCode {
        let is_floating = operand_type.is_floating_point();
        let is_i64 = matches!(
            operand_type,
            Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
        );
        match operator {
            TokenKind::OperatorPlusAssign => {
                if is_floating {
                    OpCode::AddF64
                } else if is_i64 {
                    OpCode::AddI64
                } else {
                    OpCode::AddI32
                }
            }
            TokenKind::OperatorMinusAssign => {
                if is_floating {
                    OpCode::SubF64
                } else if is_i64 {
                    OpCode::SubI64
                } else {
                    OpCode::SubI32
                }
            }
            TokenKind::OperatorMultiplyAssign => {
                if is_floating {
                    OpCode::MulF64
                } else if is_i64 {
                    OpCode::MulI64
                } else {
                    OpCode::MulI32
                }
            }
            TokenKind::OperatorDivideAssign => {
                if is_floating {
                    OpCode::DivF64
                } else if is_i64 {
                    OpCode::DivI64
                } else {
                    OpCode::DivI32
                }
            }
            TokenKind::OperatorModuloAssign => {
                if is_i64 {
                    OpCode::ModI64
                } else {
                    OpCode::ModI32
                }
            }
            _ => OpCode::AddI32,
        }
    }

    fn select_binary_opcode(&self, operator: TokenKind, operand_type: &Type) -> OpCode {
        if operand_type.is_floating_point() {
            match operator {
                TokenKind::OperatorPlus => OpCode::AddF64,
                TokenKind::OperatorMinus => OpCode::SubF64,
                TokenKind::OperatorMultiply => OpCode::MulF64,
                TokenKind::OperatorDivide => OpCode::DivF64,
                TokenKind::OperatorEqual => OpCode::EqF64,
                TokenKind::OperatorNotEqual => OpCode::NeqF64,
                TokenKind::OperatorLessThan => OpCode::LtF64,
                TokenKind::OperatorLessEqual => OpCode::LteF64,
                TokenKind::OperatorGreaterThan => OpCode::GtF64,
                TokenKind::OperatorGreaterEqual => OpCode::GteF64,
                _ => OpCode::AddF64,
            }
        } else if matches!(operand_type, Type::Primitive(PrimitiveKind::UInt64)) {
            match operator {
                TokenKind::OperatorPlus => OpCode::AddI64,
                TokenKind::OperatorMinus => OpCode::SubI64,
                TokenKind::OperatorMultiply => OpCode::MulI64,
                TokenKind::OperatorDivide => OpCode::DivU64,
                TokenKind::OperatorModulo => OpCode::ModU64,
                TokenKind::PunctuationAmpersand => OpCode::Band,
                TokenKind::OperatorBitwiseOr => OpCode::Bor,
                TokenKind::OperatorBitwiseXor => OpCode::Bxor,
                TokenKind::OperatorShiftLeft => OpCode::Shl,
                TokenKind::OperatorShiftRight => OpCode::Lshr,
                TokenKind::OperatorEqual => OpCode::EqI64,
                TokenKind::OperatorNotEqual => OpCode::NeqI64,
                TokenKind::OperatorLessThan => OpCode::LtU64,
                TokenKind::OperatorLessEqual => OpCode::LteU64,
                TokenKind::OperatorGreaterThan => OpCode::GtU64,
                TokenKind::OperatorGreaterEqual => OpCode::GteU64,
                _ => OpCode::AddI64,
            }
        } else if matches!(operand_type, Type::Primitive(PrimitiveKind::Int64)) {
            match operator {
                TokenKind::OperatorPlus => OpCode::AddI64,
                TokenKind::OperatorMinus => OpCode::SubI64,
                TokenKind::OperatorMultiply => OpCode::MulI64,
                TokenKind::OperatorDivide => OpCode::DivI64,
                TokenKind::OperatorModulo => OpCode::ModI64,
                TokenKind::PunctuationAmpersand => OpCode::Band,
                TokenKind::OperatorBitwiseOr => OpCode::Bor,
                TokenKind::OperatorBitwiseXor => OpCode::Bxor,
                TokenKind::OperatorShiftLeft => OpCode::Shl,
                TokenKind::OperatorShiftRight => OpCode::Shr,
                TokenKind::OperatorEqual => OpCode::EqI64,
                TokenKind::OperatorNotEqual => OpCode::NeqI64,
                TokenKind::OperatorLessThan => OpCode::LtI64,
                TokenKind::OperatorLessEqual => OpCode::LteI64,
                TokenKind::OperatorGreaterThan => OpCode::GtI64,
                TokenKind::OperatorGreaterEqual => OpCode::GteI64,
                _ => OpCode::AddI64,
            }
        } else if matches!(operator, TokenKind::OperatorEqual) {
            if matches!(operand_type, Type::Class { symbol_name, .. } if symbol_name == "std::string" || symbol_name == "std.string" || symbol_name == "string")
            {
                OpCode::EqString
            } else if operand_type.is_reference_type() {
                OpCode::EqRef
            } else if matches!(operand_type, Type::Primitive(PrimitiveKind::Bool)) {
                OpCode::EqBool
            } else {
                OpCode::EqI32
            }
        } else if matches!(operator, TokenKind::OperatorNotEqual) {
            if matches!(operand_type, Type::Class { symbol_name, .. } if symbol_name == "std::string" || symbol_name == "std.string" || symbol_name == "string")
            {
                OpCode::NeqString
            } else if operand_type.is_reference_type() {
                OpCode::NeqRef
            } else if matches!(operand_type, Type::Primitive(PrimitiveKind::Bool)) {
                OpCode::NeqBool
            } else {
                OpCode::NeqI32
            }
        } else if matches!(
            operand_type,
            Type::Primitive(PrimitiveKind::UInt8 | PrimitiveKind::UInt16 | PrimitiveKind::UInt32)
        ) {
            match operator {
                TokenKind::OperatorPlus => OpCode::AddI32,
                TokenKind::OperatorMinus => OpCode::SubI32,
                TokenKind::OperatorMultiply => OpCode::MulI32,
                TokenKind::OperatorDivide => OpCode::DivU32,
                TokenKind::OperatorModulo => OpCode::ModU32,
                TokenKind::PunctuationAmpersand => OpCode::Band,
                TokenKind::OperatorBitwiseOr => OpCode::Bor,
                TokenKind::OperatorBitwiseXor => OpCode::Bxor,
                TokenKind::OperatorShiftLeft => OpCode::Shl,
                TokenKind::OperatorShiftRight => OpCode::Lshr,
                TokenKind::OperatorLessThan => OpCode::LtU32,
                TokenKind::OperatorLessEqual => OpCode::LteU32,
                TokenKind::OperatorGreaterThan => OpCode::GtU32,
                TokenKind::OperatorGreaterEqual => OpCode::GteU32,
                _ => OpCode::AddI32,
            }
        } else {
            // Default 32-bit integer arithmetic and bitwise
            match operator {
                TokenKind::OperatorPlus => OpCode::AddI32,
                TokenKind::OperatorMinus => OpCode::SubI32,
                TokenKind::OperatorMultiply => OpCode::MulI32,
                TokenKind::OperatorDivide => OpCode::DivI32,
                TokenKind::OperatorModulo => OpCode::ModI32,
                TokenKind::PunctuationAmpersand => OpCode::Band,
                TokenKind::OperatorBitwiseOr => OpCode::Bor,
                TokenKind::OperatorBitwiseXor => OpCode::Bxor,
                TokenKind::OperatorShiftLeft => OpCode::Shl,
                TokenKind::OperatorShiftRight => OpCode::Shr,
                TokenKind::OperatorLessThan => OpCode::LtI32,
                TokenKind::OperatorLessEqual => OpCode::LteI32,
                TokenKind::OperatorGreaterThan => OpCode::GtI32,
                TokenKind::OperatorGreaterEqual => OpCode::GteI32,
                _ => OpCode::AddI32,
            }
        }
    }

    fn emit_unary(
        &mut self,
        dst: u16,
        operator: TokenKind,
        operand: &Expression,
        is_prefix: bool,
        span: Span,
    ) -> Result<(), EmissionError> {
        match operator {
            TokenKind::OperatorLogicalNot => {
                let r_op = self.emit_expression(operand, None)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Not, 0, dst, r_op, 0),
                    Some(span),
                );
                self.allocator.free_temp(r_op);
            }
            TokenKind::OperatorMinus => {
                let negated_literal_int = match operand {
                    Expression::Literal {
                        value: LiteralValue::Int(literal_int),
                        ..
                    } => Some(*literal_int as i128),
                    Expression::Literal {
                        value: LiteralValue::UInt(literal_uint),
                        ..
                    } => Some(*literal_uint as i128),
                    _ => None,
                };
                if let Some(raw_magnitude) = negated_literal_int {
                    let negated_i128 = -raw_magnitude;
                    let target_type = self.query_type(span).unwrap_alias().clone();
                    if matches!(
                        target_type,
                        Type::Primitive(PrimitiveKind::Float32 | PrimitiveKind::Float64)
                    ) {
                        let const_idx = self.pool.add_f64(negated_i128 as f64)?;
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadF64, 0, dst, const_idx as u32),
                            Some(span),
                        );
                        return Ok(());
                    } else if matches!(
                        target_type,
                        Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                    ) {
                        let const_idx = self.pool.add_i64(negated_i128 as i64)?;
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadConst, 0, dst, const_idx as u32),
                            Some(span),
                        );
                        return Ok(());
                    } else if negated_i128 >= i32::MIN as i128 && negated_i128 <= i32::MAX as i128 {
                        self.emit_instruction(
                            Instruction::format_b_signed(
                                OpCode::LoadI32,
                                0,
                                dst,
                                negated_i128 as i32,
                            ),
                            Some(span),
                        );
                        return Ok(());
                    }
                }
                let r_op = self.emit_expression(operand, None)?;
                let operand_type = self.query_type(operand.span());
                let op = if operand_type.is_floating_point() {
                    OpCode::NegF64
                } else if matches!(
                    operand_type,
                    Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                ) {
                    OpCode::NegI64
                } else {
                    OpCode::NegI32
                };
                self.emit_instruction(Instruction::format_a(op, 0, dst, r_op, 0), Some(span));
                self.allocator.free_temp(r_op);
            }
            TokenKind::OperatorBitwiseNot => {
                let r_op = self.emit_expression(operand, None)?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::Bnot, 0, dst, r_op, 0),
                    Some(span),
                );
                self.allocator.free_temp(r_op);
            }
            TokenKind::OperatorIncrement | TokenKind::OperatorDecrement => {
                let operand_type = self.query_type(operand.span());
                let is_float = operand_type.is_floating_point();
                let is_i64 = matches!(
                    operand_type,
                    Type::Primitive(PrimitiveKind::Int64 | PrimitiveKind::UInt64)
                );

                if let Expression::Identifier { span: id_span } = operand {
                    let name = self.get_lexeme(*id_span);
                    let local_reg = self.allocator.lookup_local(name).ok_or_else(|| {
                        EmissionError::SymbolNotFound {
                            symbol: name.to_string(),
                        }
                    })?;

                    if is_i64 || is_float {
                        let temp_one = self.allocator.alloc_temp()?;
                        let op = if is_i64 {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadI64, 0, temp_one, 1),
                                Some(span),
                            );
                            if matches!(operator, TokenKind::OperatorIncrement) {
                                OpCode::AddI64
                            } else {
                                OpCode::SubI64
                            }
                        } else {
                            let const_idx = self.pool.add_f64(1.0)?;
                            self.emit_instruction(
                                Instruction::format_b(
                                    OpCode::LoadConst,
                                    0,
                                    temp_one,
                                    const_idx as u32,
                                ),
                                Some(span),
                            );
                            if matches!(operator, TokenKind::OperatorIncrement) {
                                OpCode::AddF64
                            } else {
                                OpCode::SubF64
                            }
                        };

                        if is_prefix {
                            self.emit_instruction(
                                Instruction::format_a(op, 0, local_reg, local_reg, temp_one),
                                Some(span),
                            );
                            if dst != local_reg {
                                self.emit_instruction(
                                    Instruction::format_a(OpCode::Mov, 0, dst, local_reg, 0),
                                    Some(span),
                                );
                            }
                        } else {
                            if dst != local_reg {
                                self.emit_instruction(
                                    Instruction::format_a(OpCode::Mov, 0, dst, local_reg, 0),
                                    Some(span),
                                );
                            }
                            self.emit_instruction(
                                Instruction::format_a(op, 0, local_reg, local_reg, temp_one),
                                Some(span),
                            );
                        }
                        self.allocator.free_temp(temp_one);
                    } else {
                        let imm_op = if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddI32Imm
                        } else {
                            OpCode::SubI32Imm
                        };

                        if is_prefix {
                            self.emit_instruction(
                                Instruction::format_a(imm_op, 0, local_reg, local_reg, 1),
                                Some(span),
                            );
                            if dst != local_reg {
                                self.emit_instruction(
                                    Instruction::format_a(OpCode::Mov, 0, dst, local_reg, 0),
                                    Some(span),
                                );
                            }
                        } else {
                            // Postfix: yield old value first, then mutate local
                            if dst != local_reg {
                                self.emit_instruction(
                                    Instruction::format_a(OpCode::Mov, 0, dst, local_reg, 0),
                                    Some(span),
                                );
                            }
                            self.emit_instruction(
                                Instruction::format_a(imm_op, 0, local_reg, local_reg, 1),
                                Some(span),
                            );
                        }
                    }
                } else if let Expression::MemberAccess {
                    object,
                    member_span,
                    is_scope_resolution: true,
                    ..
                } = operand
                {
                    let obj_name = self.get_lexeme(object.span());
                    let member_name = self.get_lexeme(*member_span);
                    let full_sym = format!("{obj_name}::{member_name}");

                    if let Some(slot) = self
                        .static_layout
                        .and_then(|layout| layout.lookup_slot(&full_sym))
                    {
                        let cur_val = self.allocator.alloc_temp()?;
                        self.emit_instruction(
                            Instruction::format_b(OpCode::GetStatic, 0, cur_val, slot),
                            Some(span),
                        );
                        let temp_one = self.allocator.alloc_temp()?;
                        let op = if is_i64 {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadI64, 0, temp_one, 1),
                                Some(span),
                            );
                            if matches!(operator, TokenKind::OperatorIncrement) {
                                OpCode::AddI64
                            } else {
                                OpCode::SubI64
                            }
                        } else if is_float {
                            let const_idx = self.pool.add_f64(1.0)?;
                            self.emit_instruction(
                                Instruction::format_b(
                                    OpCode::LoadConst,
                                    0,
                                    temp_one,
                                    const_idx as u32,
                                ),
                                Some(span),
                            );
                            if matches!(operator, TokenKind::OperatorIncrement) {
                                OpCode::AddF64
                            } else {
                                OpCode::SubF64
                            }
                        } else {
                            self.emit_instruction(
                                Instruction::format_b(OpCode::LoadI32, 0, temp_one, 1),
                                Some(span),
                            );
                            if matches!(operator, TokenKind::OperatorIncrement) {
                                OpCode::AddI32
                            } else {
                                OpCode::SubI32
                            }
                        };

                        if is_prefix {
                            self.emit_instruction(
                                Instruction::format_a(op, 0, cur_val, cur_val, temp_one),
                                Some(span),
                            );
                            self.emit_instruction(
                                Instruction::format_b(OpCode::SetStatic, 0, cur_val, slot),
                                Some(span),
                            );
                            if dst != cur_val {
                                self.emit_instruction(
                                    Instruction::format_a(OpCode::Mov, 0, dst, cur_val, 0),
                                    Some(span),
                                );
                            }
                        } else {
                            if dst != cur_val {
                                self.emit_instruction(
                                    Instruction::format_a(OpCode::Mov, 0, dst, cur_val, 0),
                                    Some(span),
                                );
                            }
                            self.emit_instruction(
                                Instruction::format_a(op, 0, cur_val, cur_val, temp_one),
                                Some(span),
                            );
                            self.emit_instruction(
                                Instruction::format_b(OpCode::SetStatic, 0, cur_val, slot),
                                Some(span),
                            );
                        }
                        self.allocator.free_temp(temp_one);
                        self.allocator.free_temp(cur_val);
                    }
                } else if let Expression::MemberAccess {
                    object,
                    member_span,
                    is_scope_resolution: false,
                    ..
                } = operand
                {
                    let r_obj = self.emit_expression(object, None)?;
                    let member_name = self.get_lexeme(*member_span);
                    let offset = self.resolve_field_offset(object.span(), member_name);
                    let field_tag = self.resolve_field_type_tag(object.span(), member_name);
                    let is_weak = self.is_weak_field(object.span(), member_name);
                    let set_flags = if field_tag > 0 {
                        (field_tag << 1)
                            | (if is_weak {
                                solix_common::bytecode::instruction::FLAG_WEAK
                            } else {
                                0
                            })
                    } else if is_weak {
                        solix_common::bytecode::instruction::FLAG_WEAK
                    } else {
                        0
                    };
                    let cur_val = self.allocator.alloc_temp()?;
                    let weak_flag = if is_weak {
                        solix_common::bytecode::instruction::FLAG_WEAK
                    } else {
                        0
                    };
                    self.emit_instruction(
                        Instruction::format_a(
                            OpCode::GetField,
                            (field_tag << 1) | weak_flag,
                            cur_val,
                            r_obj,
                            offset,
                        ),
                        Some(span),
                    );
                    let temp_one = self.allocator.alloc_temp()?;
                    let op = if is_i64 {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadI64, 0, temp_one, 1),
                            Some(span),
                        );
                        if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddI64
                        } else {
                            OpCode::SubI64
                        }
                    } else if is_float {
                        let const_idx = self.pool.add_f64(1.0)?;
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadConst, 0, temp_one, const_idx as u32),
                            Some(span),
                        );
                        if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddF64
                        } else {
                            OpCode::SubF64
                        }
                    } else {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadI32, 0, temp_one, 1),
                            Some(span),
                        );
                        if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddI32
                        } else {
                            OpCode::SubI32
                        }
                    };
                    if is_prefix {
                        self.emit_instruction(
                            Instruction::format_a(op, 0, cur_val, cur_val, temp_one),
                            Some(span),
                        );
                        self.emit_instruction(
                            Instruction::format_a(
                                OpCode::SetField,
                                set_flags,
                                cur_val,
                                r_obj,
                                offset,
                            ),
                            Some(span),
                        );
                        if dst != cur_val {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Mov, 0, dst, cur_val, 0),
                                Some(span),
                            );
                        }
                    } else {
                        if dst != cur_val {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Mov, 0, dst, cur_val, 0),
                                Some(span),
                            );
                        }
                        self.emit_instruction(
                            Instruction::format_a(op, 0, cur_val, cur_val, temp_one),
                            Some(span),
                        );
                        self.emit_instruction(
                            Instruction::format_a(
                                OpCode::SetField,
                                set_flags,
                                cur_val,
                                r_obj,
                                offset,
                            ),
                            Some(span),
                        );
                    }
                    self.allocator.free_temp(temp_one);
                    self.allocator.free_temp(cur_val);
                    self.allocator.free_temp(r_obj);
                } else if let Expression::ArrayAccess {
                    span: arr_span,
                    array,
                    index,
                } = operand
                {
                    let r_arr = self.emit_expression(array, None)?;
                    let r_idx = self.emit_expression(index, None)?;
                    let cur_val = self.allocator.alloc_temp()?;
                    self.emit_instruction(
                        Instruction::format_a(OpCode::GetArray, 0, cur_val, r_arr, r_idx),
                        Some(*arr_span),
                    );
                    let temp_one = self.allocator.alloc_temp()?;
                    let op = if is_i64 {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadI64, 0, temp_one, 1),
                            Some(span),
                        );
                        if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddI64
                        } else {
                            OpCode::SubI64
                        }
                    } else if is_float {
                        let const_idx = self.pool.add_f64(1.0)?;
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadConst, 0, temp_one, const_idx as u32),
                            Some(span),
                        );
                        if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddF64
                        } else {
                            OpCode::SubF64
                        }
                    } else {
                        self.emit_instruction(
                            Instruction::format_b(OpCode::LoadI32, 0, temp_one, 1),
                            Some(span),
                        );
                        if matches!(operator, TokenKind::OperatorIncrement) {
                            OpCode::AddI32
                        } else {
                            OpCode::SubI32
                        }
                    };

                    if is_prefix {
                        self.emit_instruction(
                            Instruction::format_a(op, 0, cur_val, cur_val, temp_one),
                            Some(span),
                        );
                        self.emit_instruction(
                            Instruction::format_a(OpCode::SetArray, 0, r_arr, r_idx, cur_val),
                            Some(span),
                        );
                        if dst != cur_val {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Mov, 0, dst, cur_val, 0),
                                Some(span),
                            );
                        }
                    } else {
                        if dst != cur_val {
                            self.emit_instruction(
                                Instruction::format_a(OpCode::Mov, 0, dst, cur_val, 0),
                                Some(span),
                            );
                        }
                        self.emit_instruction(
                            Instruction::format_a(op, 0, cur_val, cur_val, temp_one),
                            Some(span),
                        );
                        self.emit_instruction(
                            Instruction::format_a(OpCode::SetArray, 0, r_arr, r_idx, cur_val),
                            Some(span),
                        );
                    }
                    self.allocator.free_temp(temp_one);
                    self.allocator.free_temp(cur_val);
                    self.allocator.free_temp(r_idx);
                    self.allocator.free_temp(r_arr);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Emits an expression evaluated purely for side effects (skipping any trailing result `Mov`).
    pub(super) fn emit_discarded_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<(), EmissionError> {
        if let Expression::Assignment {
            span,
            target,
            operator,
            value,
        } = expr
        {
            return self.emit_assignment(None, target, *operator, value, *span);
        }
        let target_hint = self.resolve_assignment_target_local(expr);
        let temp = self.emit_expression(expr, target_hint)?;
        if target_hint != Some(temp) {
            self.allocator.free_temp(temp);
        }
        Ok(())
    }

    fn emit_assignment(
        &mut self,
        dst: Option<u16>,
        target: &Expression,
        operator: TokenKind,
        value: &Expression,
        span: Span,
    ) -> Result<(), EmissionError> {
        if let Expression::Identifier { span: id_span } = target {
            let name = self.get_lexeme(*id_span).to_string();
            if let Some(local_reg) = self.allocator.lookup_local(&name) {
                if matches!(operator, TokenKind::OperatorAssign) {
                    let is_ref = self.function.reference_registers.contains(&local_reg)
                        || self.query_type(target.span()).is_reference_type();
                    let value_type = self.query_type(value.span());
                    let target_type = self.query_type(target.span());
                    let needs_conv = target_type.is_floating_point()
                        && (value_type.is_signed_integer() || value_type.is_unsigned_integer());
                    let direct_target =
                        (!is_ref && !needs_conv && is_direct_eval_safe(value)).then_some(local_reg);

                    let mut r_val = self.emit_expression(value, direct_target)?;
                    if needs_conv {
                        let converted_reg = self.allocator.alloc_temp()?;
                        self.emit_instruction(
                            Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, r_val, 0),
                            Some(span),
                        );
                        self.allocator.free_temp(r_val);
                        r_val = converted_reg;
                    }
                    if is_ref && self.is_borrowed_reference(value) {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::IncRef, 0, r_val, 0, 0),
                            Some(span),
                        );
                    }
                    if is_ref {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::DecRef, 0, local_reg, 0, 0),
                            Some(span),
                        );
                    }
                    if r_val != local_reg {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, local_reg, r_val, 0),
                            Some(span),
                        );
                    }
                    if let Some(dst_reg) = dst
                        && dst_reg != local_reg
                    {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, dst_reg, local_reg, 0),
                            Some(span),
                        );
                    }
                    if r_val != local_reg {
                        self.allocator.free_temp(r_val);
                    }
                    return Ok(());
                }

                let target_type = self.query_type(target.span());
                if matches!(operator, TokenKind::OperatorPlusAssign)
                    && matches!(
                        &target_type,
                        Type::Class { symbol_name, .. }
                            if symbol_name == "std::string"
                                || symbol_name == "std.string"
                                || symbol_name == "string"
                    )
                {
                    let r_val = self.emit_expression(value, None)?;
                    let window_size = 2;
                    let arg_base = self.allocator.alloc_window(window_size)?;
                    self.emit_instruction(
                        Instruction::format_a(OpCode::Mov, 0, arg_base, local_reg, 0),
                        Some(span),
                    );
                    self.emit_instruction(
                        Instruction::format_a(OpCode::Mov, 0, arg_base + 1, r_val, 0),
                        Some(span),
                    );
                    let sym_idx = self.pool.add_string("__builtin_str_concat")?;
                    self.emit_instruction(
                        Instruction::format_call(
                            OpCode::CallSymbol,
                            local_reg,
                            sym_idx,
                            arg_base,
                            window_size as u8,
                        ),
                        Some(span),
                    );
                    self.allocator.free_window(arg_base, window_size);
                    self.allocator.free_temp(r_val);
                    if let Some(dst_reg) = dst
                        && dst_reg != local_reg
                    {
                        self.emit_instruction(
                            Instruction::format_a(OpCode::Mov, 0, dst_reg, local_reg, 0),
                            Some(span),
                        );
                    }
                    return Ok(());
                }

                let is_narrow_or_i32 = matches!(
                    target_type,
                    Type::Primitive(
                        PrimitiveKind::Int8
                            | PrimitiveKind::Int16
                            | PrimitiveKind::Int32
                            | PrimitiveKind::UInt8
                            | PrimitiveKind::UInt16
                            | PrimitiveKind::UInt32
                            | PrimitiveKind::Char
                    )
                );

                if let Expression::Literal {
                    value: LiteralValue::Int(imm_val),
                    ..
                } = value
                    && is_narrow_or_i32
                    && *imm_val >= -32768
                    && *imm_val <= 32767
                    && matches!(
                        operator,
                        TokenKind::OperatorPlusAssign | TokenKind::OperatorMinusAssign
                    )
                {
                    let imm_op = if matches!(operator, TokenKind::OperatorPlusAssign) {
                        OpCode::AddI32Imm
                    } else {
                        OpCode::SubI32Imm
                    };
                    self.emit_instruction(
                        Instruction::format_a(
                            imm_op,
                            0,
                            local_reg,
                            local_reg,
                            *imm_val as i16 as u16,
                        ),
                        Some(span),
                    );
                } else {
                    let mut r_val = self.emit_expression(value, None)?;
                    let value_type = self.query_type(value.span());
                    if target_type.is_floating_point()
                        && (value_type.is_signed_integer() || value_type.is_unsigned_integer())
                    {
                        let converted_reg = self.allocator.alloc_temp()?;
                        self.emit_instruction(
                            Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, r_val, 0),
                            Some(span),
                        );
                        self.allocator.free_temp(r_val);
                        r_val = converted_reg;
                    }
                    let op = self.select_compound_opcode(operator, &target_type);
                    self.emit_instruction(
                        Instruction::format_a(op, 0, local_reg, local_reg, r_val),
                        Some(span),
                    );
                    self.allocator.free_temp(r_val);
                }

                if let Some(dst_reg) = dst
                    && dst_reg != local_reg
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::Mov, 0, dst_reg, local_reg, 0),
                        Some(span),
                    );
                }
            } else if let Some(slot) = self
                .static_layout
                .and_then(|layout| layout.lookup_slot(&name))
            {
                let mut r_val = self.emit_expression(value, None)?;
                let value_type = self.query_type(value.span());
                let target_type = self.query_type(target.span());
                if target_type.is_floating_point()
                    && (value_type.is_signed_integer() || value_type.is_unsigned_integer())
                {
                    let converted_reg = self.allocator.alloc_temp()?;
                    self.emit_instruction(
                        Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, r_val, 0),
                        Some(span),
                    );
                    self.allocator.free_temp(r_val);
                    r_val = converted_reg;
                }
                self.emit_instruction(
                    Instruction::format_b(OpCode::SetStatic, 0, r_val, slot),
                    Some(span),
                );
                if let Some(dst_reg) = dst
                    && dst_reg != r_val
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::Mov, 0, dst_reg, r_val, 0),
                        Some(span),
                    );
                }
                self.allocator.free_temp(r_val);
                return Ok(());
            } else {
                let const_idx = self.pool.add_string(&name)?;
                let mut r_val = self.emit_expression(value, None)?;
                let value_type = self.query_type(value.span());
                let target_type = self.query_type(target.span());
                if target_type.is_floating_point()
                    && (value_type.is_signed_integer() || value_type.is_unsigned_integer())
                {
                    let converted_reg = self.allocator.alloc_temp()?;
                    self.emit_instruction(
                        Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, r_val, 0),
                        Some(span),
                    );
                    self.allocator.free_temp(r_val);
                    r_val = converted_reg;
                }
                self.emit_instruction(
                    Instruction::format_b(OpCode::SetStaticGlobal, 0, r_val, const_idx as u32),
                    Some(span),
                );
                if let Some(dst_reg) = dst
                    && dst_reg != r_val
                {
                    self.emit_instruction(
                        Instruction::format_a(OpCode::Mov, 0, dst_reg, r_val, 0),
                        Some(span),
                    );
                }
                self.allocator.free_temp(r_val);
                return Ok(());
            }
        } else if let Expression::MemberAccess {
            span: mem_span,
            object,
            member_span,
            is_scope_resolution: true,
        } = target
        {
            let obj_name = self.get_lexeme(object.span());
            let member_name = self.get_lexeme(*member_span);
            let full_sym = format!("{obj_name}::{member_name}");

            let mut r_val = self.emit_expression(value, None)?;
            if let Some(slot) = self
                .static_layout
                .and_then(|layout| layout.lookup_slot(&full_sym))
            {
                if !matches!(operator, TokenKind::OperatorAssign) {
                    let r_cur = self.allocator.alloc_temp()?;
                    self.emit_instruction(
                        Instruction::format_b(OpCode::GetStatic, 0, r_cur, slot),
                        Some(*mem_span),
                    );
                    let operand_type = self.query_type(target.span());
                    let op = self.select_compound_opcode(operator, &operand_type);
                    let r_calc = self.allocator.alloc_temp()?;
                    self.emit_instruction(
                        Instruction::format_a(op, 0, r_calc, r_cur, r_val),
                        Some(span),
                    );
                    self.allocator.free_temp(r_cur);
                    self.allocator.free_temp(r_val);
                    r_val = r_calc;
                }
                self.emit_instruction(
                    Instruction::format_b(OpCode::SetStatic, 0, r_val, slot),
                    Some(*mem_span),
                );
            } else {
                let const_idx = self.pool.add_string(&full_sym)?;
                self.emit_instruction(
                    Instruction::format_b(OpCode::SetStaticGlobal, 0, r_val, const_idx as u32),
                    Some(*mem_span),
                );
            }
            if let Some(dst_reg) = dst
                && dst_reg != r_val
            {
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, dst_reg, r_val, 0),
                    Some(span),
                );
            }
            self.allocator.free_temp(r_val);
        } else if let Expression::MemberAccess {
            span: mem_span,
            object,
            member_span,
            is_scope_resolution: false,
        } = target
        {
            let r_obj = self.emit_expression(object, None)?;
            let mut r_val = self.emit_expression(value, None)?;
            let value_type = self.query_type(value.span());
            let target_type = self.query_type(target.span());
            if target_type.is_floating_point()
                && (value_type.is_signed_integer() || value_type.is_unsigned_integer())
            {
                let converted_reg = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, r_val, 0),
                    Some(span),
                );
                self.allocator.free_temp(r_val);
                r_val = converted_reg;
            }
            let member_name = self.get_lexeme(*member_span);
            let offset = self.resolve_field_offset(object.span(), member_name);
            let field_tag = self.resolve_field_type_tag(object.span(), member_name);
            let is_weak = self.is_weak_field(object.span(), member_name);
            let set_flags = if field_tag > 0 {
                (field_tag << 1)
                    | (if is_weak {
                        solix_common::bytecode::instruction::FLAG_WEAK
                    } else {
                        0
                    })
            } else if is_weak {
                solix_common::bytecode::instruction::FLAG_WEAK
            } else {
                0
            };

            if !matches!(operator, TokenKind::OperatorAssign) {
                let r_cur = self.allocator.alloc_temp()?;
                let weak_flag = if is_weak {
                    solix_common::bytecode::instruction::FLAG_WEAK
                } else {
                    0
                };
                self.emit_instruction(
                    Instruction::format_a(
                        OpCode::GetField,
                        (field_tag << 1) | weak_flag,
                        r_cur,
                        r_obj,
                        offset,
                    ),
                    Some(*mem_span),
                );
                let operand_type = self.query_type(target.span());
                let op = self.select_compound_opcode(operator, &operand_type);
                let r_calc = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(op, 0, r_calc, r_cur, r_val),
                    Some(span),
                );
                self.allocator.free_temp(r_cur);
                self.allocator.free_temp(r_val);
                r_val = r_calc;
            }

            self.emit_instruction(
                Instruction::format_a(OpCode::SetField, set_flags, r_val, r_obj, offset),
                Some(*mem_span),
            );

            if self.query_type(value.span()).is_reference_type()
                && !self.is_borrowed_reference(value)
            {
                self.emit_instruction(
                    Instruction::format_a(OpCode::DecRef, 0, r_val, 0, 0),
                    Some(*mem_span),
                );
            }

            if let Some(dst_reg) = dst
                && dst_reg != r_val
            {
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, dst_reg, r_val, 0),
                    Some(span),
                );
            }
            self.allocator.free_temp(r_val);
            self.allocator.free_temp(r_obj);
        } else if let Expression::ArrayAccess {
            span: arr_span,
            array,
            index,
        } = target
        {
            let r_arr = self.emit_expression(array, None)?;
            let r_idx = self.emit_expression(index, None)?;
            let mut r_val = self.emit_expression(value, None)?;
            let value_type = self.query_type(value.span());
            let target_type = self.query_type(target.span());
            if target_type.is_floating_point()
                && (value_type.is_signed_integer() || value_type.is_unsigned_integer())
            {
                let converted_reg = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::ConvIToF, 0, converted_reg, r_val, 0),
                    Some(span),
                );
                self.allocator.free_temp(r_val);
                r_val = converted_reg;
            }

            if !matches!(operator, TokenKind::OperatorAssign) {
                let r_cur = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(OpCode::GetArray, 0, r_cur, r_arr, r_idx),
                    Some(*arr_span),
                );
                let operand_type = self.query_type(target.span());
                let op = self.select_compound_opcode(operator, &operand_type);
                let r_calc = self.allocator.alloc_temp()?;
                self.emit_instruction(
                    Instruction::format_a(op, 0, r_calc, r_cur, r_val),
                    Some(span),
                );
                self.allocator.free_temp(r_cur);
                self.allocator.free_temp(r_val);
                r_val = r_calc;
            }

            self.emit_instruction(
                Instruction::format_a(OpCode::SetArray, 0, r_arr, r_idx, r_val),
                Some(*arr_span),
            );

            if self.query_type(value.span()).is_reference_type()
                && !self.is_borrowed_reference(value)
            {
                self.emit_instruction(
                    Instruction::format_a(OpCode::DecRef, 0, r_val, 0, 0),
                    Some(*arr_span),
                );
            }

            if let Some(dst_reg) = dst
                && dst_reg != r_val
            {
                self.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, dst_reg, r_val, 0),
                    Some(span),
                );
            }
            self.allocator.free_temp(r_val);
            self.allocator.free_temp(r_idx);
            self.allocator.free_temp(r_arr);
        }
        Ok(())
    }

    fn emit_ternary(
        &mut self,
        dst: u16,
        condition: &Expression,
        then_branch: &Expression,
        else_branch: &Expression,
        span: Span,
    ) -> Result<(), EmissionError> {
        let condition_register = self.emit_expression(condition, None)?;
        let jump_false_pc = self.emit_instruction(
            Instruction::format_c(OpCode::JumpIfFalse, 0, condition_register, 0),
            Some(span),
        );
        self.allocator.free_temp(condition_register);

        let is_reference_result = self.query_type(span).is_reference_type();

        self.emit_expression(then_branch, Some(dst))?;
        if is_reference_result && self.is_borrowed_reference(then_branch) {
            self.emit_instruction(
                Instruction::format_a(OpCode::IncRef, 0, dst, 0, 0),
                Some(then_branch.span()),
            );
        }
        let jump_exit_pc =
            self.emit_instruction(Instruction::format_c(OpCode::Jump, 0, 0, 0), Some(span));

        let else_start_pc = self.function.instructions.len() as i32;
        let false_offset = else_start_pc - jump_false_pc as i32;
        self.function.instructions[jump_false_pc as usize] =
            Instruction::format_c(OpCode::JumpIfFalse, 0, condition_register, false_offset);

        self.emit_expression(else_branch, Some(dst))?;
        if is_reference_result && self.is_borrowed_reference(else_branch) {
            self.emit_instruction(
                Instruction::format_a(OpCode::IncRef, 0, dst, 0, 0),
                Some(else_branch.span()),
            );
        }

        let exit_pc = self.function.instructions.len() as i32;
        let exit_offset = exit_pc - jump_exit_pc as i32;
        self.function.instructions[jump_exit_pc as usize] =
            Instruction::format_c(OpCode::Jump, 0, 0, exit_offset);

        Ok(())
    }

    #[must_use]
    pub fn query_type(&self, span: Span) -> Type {
        if let Some(node_type) = self.node_types.and_then(|types| types.get(&span)) {
            return node_type.clone();
        }
        Type::Primitive(PrimitiveKind::Int32)
    }

    #[must_use]
    pub fn is_reference_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::TupleCreation { .. }
            | Expression::ArrayCreation { .. }
            | Expression::ArrayLiteral { .. }
            | Expression::NewInstance { .. }
            | Expression::Lambda { .. } => true,
            _ => {
                let ty = self.query_type(expr.span());
                ty.is_reference_type()
                    || matches!(ty, Type::GenericParameter { .. } | Type::Unresolved(_))
            }
        }
    }

    #[must_use]
    pub(crate) fn is_borrowed_reference(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier { span } => {
                let name = self.get_lexeme(*span);
                !self.function_ids.and_then(|ids| ids.get(name)).is_some()
            }
            Expression::MemberAccess {
                object,
                member_span,
                ..
            } => {
                let obj_name = self.get_lexeme(object.span());
                let member_name = self.get_lexeme(*member_span);
                let full_sym = format!("{obj_name}::{member_name}");
                !self
                    .function_ids
                    .and_then(|ids| ids.get(&full_sym))
                    .is_some()
            }
            Expression::ArrayAccess { .. } => true,
            Expression::Cast { expression, .. } => self.is_borrowed_reference(expression),
            _ => false,
        }
    }

    fn resolve_dst(&mut self, target_reg: Option<u16>) -> Result<u16, EmissionError> {
        match target_reg {
            Some(dst) => Ok(dst),
            None => self.allocator.alloc_temp(),
        }
    }

    fn resolve_field_offset(&self, object_span: Span, member_name: &str) -> u16 {
        let obj_name = self.get_lexeme(object_span);
        if obj_name == "this"
            && let Some(ref cls) = self.enclosing_class
        {
            let simple = cls.rsplit(['.', ':']).next().unwrap_or(cls);
            if let Some(layouts) = self.class_layouts
                && let Some(layout) = layouts.get(cls).or_else(|| layouts.get(simple))
                && let Some(&offset) = layout.field_offsets.get(member_name)
            {
                return offset as u16;
            }
        }

        let obj_type = self.query_type(object_span);
        if let Type::Class {
            symbol_name,
            type_arguments,
        } = obj_type
        {
            let generic_substitutions = self.get_enclosing_generic_substitutions();
            let resolved_arguments: Vec<String> = type_arguments
                .iter()
                .map(|type_argument| {
                    let argument_string = type_argument.to_string();
                    generic_substitutions
                        .get(&argument_string)
                        .cloned()
                        .unwrap_or(argument_string)
                })
                .collect();

            let mangled = if !resolved_arguments.is_empty() && !symbol_name.contains('<') {
                format!("{symbol_name}<{}>", resolved_arguments.join(", "))
            } else {
                symbol_name.clone()
            };
            let simple_base = symbol_name
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(&symbol_name);
            let simple_mangled = if !resolved_arguments.is_empty() && !simple_base.contains('<') {
                format!("{simple_base}<{}>", resolved_arguments.join(", "))
            } else {
                simple_base.to_string()
            };

            let offset_opt = self
                .class_layouts
                .and_then(|layouts| {
                    layouts
                        .get(&mangled)
                        .or_else(|| layouts.get(&simple_mangled))
                        .or_else(|| layouts.get(&symbol_name))
                        .or_else(|| layouts.get(simple_base))
                })
                .and_then(|layout| layout.field_offsets.get(member_name))
                .copied();
            if let Some(offset) = offset_opt {
                return offset as u16;
            }
        }
        8
    }

    fn is_weak_field(&self, object_span: Span, member_name: &str) -> bool {
        let obj_name = self.get_lexeme(object_span);
        if obj_name == "this"
            && let Some(ref cls) = self.enclosing_class
        {
            let base_cls = cls.split('<').next().unwrap_or(cls);
            let simple = base_cls.rsplit(['.', ':']).next().unwrap_or(base_cls);
            if let Some(layouts) = self.class_layouts
                && let Some(layout) = layouts
                    .get(cls)
                    .or_else(|| layouts.get(base_cls))
                    .or_else(|| layouts.get(simple))
                && let Some(&offset) = layout.field_offsets.get(member_name)
            {
                return layout.weak_field_offsets.contains(&offset);
            }
        }

        let obj_type = self.query_type(object_span);
        if let Type::Class {
            symbol_name,
            type_arguments,
        } = obj_type
        {
            let generic_substitutions = self.get_enclosing_generic_substitutions();
            let resolved_arguments: Vec<String> = type_arguments
                .iter()
                .map(|argument_type| {
                    let argument_string = argument_type.to_string();
                    generic_substitutions
                        .get(&argument_string)
                        .cloned()
                        .unwrap_or(argument_string)
                })
                .collect();

            let mangled = if !resolved_arguments.is_empty() && !symbol_name.contains('<') {
                format!("{symbol_name}<{}>", resolved_arguments.join(", "))
            } else {
                symbol_name.clone()
            };
            let simple_base = symbol_name
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(&symbol_name);
            let simple_mangled = if !resolved_arguments.is_empty() && !simple_base.contains('<') {
                format!("{simple_base}<{}>", resolved_arguments.join(", "))
            } else {
                simple_base.to_string()
            };
            if let Some(layouts) = self.class_layouts
                && let Some(layout) = layouts
                    .get(&mangled)
                    .or_else(|| layouts.get(&simple_mangled))
                    .or_else(|| layouts.get(&symbol_name))
                    .or_else(|| layouts.get(simple_base))
                && let Some(&offset) = layout.field_offsets.get(member_name)
            {
                return layout.weak_field_offsets.contains(&offset);
            }
        }
        false
    }

    fn resolve_field_type_tag(&self, object_span: Span, member_name: &str) -> u8 {
        let obj_name = self.get_lexeme(object_span);
        if obj_name == "this"
            && let Some(ref cls) = self.enclosing_class
        {
            let base_cls = cls.split('<').next().unwrap_or(cls);
            let simple = base_cls.rsplit(['.', ':']).next().unwrap_or(base_cls);
            if let Some(layouts) = self.class_layouts
                && let Some(layout) = layouts
                    .get(cls)
                    .or_else(|| layouts.get(base_cls))
                    .or_else(|| layouts.get(simple))
                && let Some(&offset) = layout.field_offsets.get(member_name)
                && let Some(&tag) = layout.field_types.get(&offset)
            {
                return tag;
            }
        }

        let obj_type = self.query_type(object_span);
        if let Type::Class {
            symbol_name,
            type_arguments,
        } = obj_type
        {
            let generic_substitutions = self.get_enclosing_generic_substitutions();
            let resolved_arguments: Vec<String> = type_arguments
                .iter()
                .map(|arg_type| {
                    let argument_string = arg_type.to_string();
                    generic_substitutions
                        .get(&argument_string)
                        .cloned()
                        .unwrap_or(argument_string)
                })
                .collect();

            let mangled = if !resolved_arguments.is_empty() && !symbol_name.contains('<') {
                format!("{symbol_name}<{}>", resolved_arguments.join(", "))
            } else {
                symbol_name.clone()
            };
            let simple_base = symbol_name
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(&symbol_name);
            let simple_mangled = if !resolved_arguments.is_empty() && !simple_base.contains('<') {
                format!("{simple_base}<{}>", resolved_arguments.join(", "))
            } else {
                simple_base.to_string()
            };

            let tag_opt = self
                .class_layouts
                .and_then(|layouts| {
                    layouts
                        .get(&mangled)
                        .or_else(|| layouts.get(&simple_mangled))
                        .or_else(|| layouts.get(&symbol_name))
                        .or_else(|| layouts.get(simple_base))
                })
                .and_then(|layout| {
                    layout
                        .field_offsets
                        .get(member_name)
                        .and_then(|&offset| layout.field_types.get(&offset).copied())
                });
            if let Some(tag) = tag_opt {
                return tag;
            }
        }
        0
    }

    fn resolve_type_byte_size(&self, type_name: &str) -> usize {
        match type_name {
            "int8" | "uint8" | "i8" | "u8" | "bool" | "char" => 1,
            "int16" | "uint16" | "i16" | "u16" => 2,
            "int32" | "uint32" | "i32" | "u32" | "f32" | "float32" => 4,
            "int64" | "uint64" | "i64" | "u64" | "f64" | "float64" => 8,
            _ => {
                if let Some(layouts) = self.class_layouts
                    && let Some(layout) = layouts.get(type_name)
                {
                    return layout.instance_size_bytes;
                }
                8
            }
        }
    }

    pub(crate) fn resolve_vtable_id(&self, type_name: &str) -> u32 {
        if let Some(layouts) = self.class_layouts {
            if let Some(layout) = layouts.get(type_name) {
                return layout.vtable_id;
            }
            let base_name = type_name.split('<').next().unwrap_or(type_name);
            let simple_base = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
            let generic_part = type_name
                .split_once('<')
                .map(|(_, args)| format!("<{args}"))
                .unwrap_or_default();
            let simple = format!("{simple_base}{generic_part}");
            if let Some(layout) = layouts.get(&simple) {
                return layout.vtable_id;
            }
            let enclosing_pkg = self.enclosing_class.as_deref().and_then(|cls| {
                let base = cls.split('<').next().unwrap_or(cls);
                base.rsplit_once("::").map(|(pkg, _)| pkg)
            });
            if let Some(pkg) = enclosing_pkg {
                let qualified_simple = format!("{pkg}::{simple}");
                if let Some(layout) = layouts.get(&qualified_simple) {
                    return layout.vtable_id;
                }
            }
        }
        0
    }

    fn resolve_vtable_id_for_type(&self, ty: &Type) -> u32 {
        match ty.unwrap_alias() {
            Type::Class {
                symbol_name,
                type_arguments,
            } => {
                if !type_arguments.is_empty() {
                    let mut specialized_name = if !symbol_name.contains('<') {
                        format!(
                            "{symbol_name}<{}>",
                            type_arguments
                                .iter()
                                .map(|arg| arg.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    } else {
                        symbol_name.clone()
                    };
                    let generic_substitutions = self.get_enclosing_generic_substitutions();
                    if !generic_substitutions.is_empty() {
                        if let Some(sub) = generic_substitutions.get(&specialized_name) {
                            specialized_name = sub.clone();
                        } else if specialized_name.contains('<')
                            && let Some((base_name, args_part)) = specialized_name.split_once('<')
                            && let Some(args_string) = args_part.strip_suffix('>')
                        {
                            let substituted_args: Vec<String> =
                                split_top_level_generic_arguments(args_string)
                                    .into_iter()
                                    .map(|arg| {
                                        generic_substitutions
                                            .get(arg)
                                            .cloned()
                                            .unwrap_or_else(|| arg.to_string())
                                    })
                                    .collect();
                            specialized_name =
                                format!("{base_name}<{}>", substituted_args.join(", "));
                        }
                    }
                    let specialized_vtable = self.resolve_vtable_id(&specialized_name);
                    if specialized_vtable > 0 {
                        return specialized_vtable;
                    }
                }
                self.resolve_vtable_id(symbol_name)
            }
            Type::Interface { symbol_name, .. } => self.resolve_vtable_id(symbol_name),
            Type::GenericParameter { name, .. } => {
                let substitutions = self.get_enclosing_generic_substitutions();
                if let Some(sub) = substitutions.get(name.as_str()) {
                    let vtable = self.resolve_vtable_id(sub);
                    if vtable > 0 {
                        return vtable;
                    }
                }
                0
            }
            Type::Unresolved(name) => {
                let substitutions = self.get_enclosing_generic_substitutions();
                if let Some(sub) = substitutions.get(name.as_str()) {
                    let vtable = self.resolve_vtable_id(sub);
                    if vtable > 0 {
                        return vtable;
                    }
                }
                self.resolve_vtable_id(name)
            }
            _ => 0,
        }
    }

    pub(crate) fn resolve_type_tag(&self, type_name: &str) -> u8 {
        match type_name {
            "int8" | "i8" => 1,
            "int16" | "i16" => 2,
            "int32" | "i32" | "int" => 3,
            "int64" | "i64" => 4,
            "uint8" | "u8" => 5,
            "uint16" | "u16" => 6,
            "uint32" | "u32" => 7,
            "uint64" | "u64" => 8,
            "float32" | "f32" => 9,
            "float64" | "f64" => 10,
            "bool" => 11,
            "char" => 12,
            "string" | "std.string" | "String" | "solix.core.String" | "solix::core::String" => 13,
            _ => {
                let vtable_id = self.resolve_vtable_id(type_name);
                if vtable_id > 0 {
                    return ((vtable_id as u8) & 0x7F) | 0x80;
                }
                0x80
            }
        }
    }

    fn get_enclosing_generic_substitutions(&self) -> HashMap<String, String> {
        let mut substitutions = HashMap::new();
        if let Some(enclosing_class_name) = self.enclosing_class.as_deref()
            && let Some((_base_name, args_part)) = enclosing_class_name.split_once('<')
            && let Some(args_string) = args_part.strip_suffix('>')
        {
            let concrete_arguments = split_top_level_generic_arguments(args_string);
            if concrete_arguments.len() == 1 {
                substitutions.insert("T".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("K".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("V".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("E".to_string(), concrete_arguments[0].to_string());
            } else if concrete_arguments.len() == 2 {
                substitutions.insert("K".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("V".to_string(), concrete_arguments[1].to_string());
                substitutions.insert("T".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("U".to_string(), concrete_arguments[1].to_string());
                substitutions.insert("E".to_string(), concrete_arguments[1].to_string());
            } else if concrete_arguments.len() >= 3 {
                substitutions.insert("T".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("U".to_string(), concrete_arguments[1].to_string());
                substitutions.insert("V".to_string(), concrete_arguments[2].to_string());
                substitutions.insert("K".to_string(), concrete_arguments[0].to_string());
                substitutions.insert("E".to_string(), concrete_arguments[2].to_string());
            }
        }
        substitutions
    }

    fn resolve_type_tag_for_type(&self, ty: &Type) -> u8 {
        match ty.unwrap_alias() {
            Type::Primitive(pk) => match pk {
                PrimitiveKind::Int8 => 1,
                PrimitiveKind::Int16 => 2,
                PrimitiveKind::Int32 => 3,
                PrimitiveKind::Int64 => 4,
                PrimitiveKind::UInt8 => 5,
                PrimitiveKind::UInt16 => 6,
                PrimitiveKind::UInt32 => 7,
                PrimitiveKind::UInt64 => 8,
                PrimitiveKind::Float32 => 9,
                PrimitiveKind::Float64 => 10,
                PrimitiveKind::Bool => 11,
                PrimitiveKind::Char => 12,
                _ => 3,
            },
            Type::Class {
                symbol_name,
                type_arguments,
            } => {
                if !type_arguments.is_empty() {
                    let mut specialized_name = if !symbol_name.contains('<') {
                        format!(
                            "{symbol_name}<{}>",
                            type_arguments
                                .iter()
                                .map(|arg| arg.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    } else {
                        symbol_name.clone()
                    };
                    let generic_substitutions = self.get_enclosing_generic_substitutions();
                    if !generic_substitutions.is_empty() {
                        if let Some(sub) = generic_substitutions.get(&specialized_name) {
                            specialized_name = sub.clone();
                        } else if specialized_name.contains('<')
                            && let Some((base_name, args_part)) = specialized_name.split_once('<')
                            && let Some(args_string) = args_part.strip_suffix('>')
                        {
                            let substituted_args: Vec<String> =
                                split_top_level_generic_arguments(args_string)
                                    .into_iter()
                                    .map(|arg| {
                                        generic_substitutions
                                            .get(arg)
                                            .cloned()
                                            .unwrap_or_else(|| arg.to_string())
                                    })
                                    .collect();
                            specialized_name =
                                format!("{base_name}<{}>", substituted_args.join(", "));
                        }
                    }
                    let specialized_tag = self.resolve_type_tag(&specialized_name);
                    if specialized_tag != 0x80 {
                        return specialized_tag;
                    }
                }
                self.resolve_type_tag(symbol_name)
            }
            Type::Interface { symbol_name, .. } => self.resolve_type_tag(symbol_name),
            Type::GenericParameter { name, .. } => {
                let substitutions = self.get_enclosing_generic_substitutions();
                if let Some(sub) = substitutions.get(name.as_str()) {
                    let tag = self.resolve_type_tag(sub);
                    if tag != 0x80 {
                        return tag;
                    }
                }
                if let Some(ref enclosing) = self.enclosing_class
                    && let Some(pos) = enclosing.find('<')
                {
                    let inner = &enclosing[pos + 1..enclosing.len().saturating_sub(1)];
                    return self.resolve_type_tag(inner.trim());
                }
                0x80
            }
            Type::Unresolved(name) => {
                let substitutions = self.get_enclosing_generic_substitutions();
                if let Some(sub) = substitutions.get(name.as_str()) {
                    let tag = self.resolve_type_tag(sub);
                    if tag != 0x80 {
                        return tag;
                    }
                }
                let direct_tag = self.resolve_type_tag(name);
                if direct_tag != 0x80 {
                    return direct_tag;
                }
                if let Some(ref enclosing) = self.enclosing_class
                    && let Some(pos) = enclosing.find('<')
                {
                    let inner = &enclosing[pos + 1..enclosing.len().saturating_sub(1)];
                    return self.resolve_type_tag(inner.trim());
                }
                0x80
            }
            Type::Array { .. } | Type::Tuple(_) => 0x80,
            _ => 0x80,
        }
    }

    fn resolve_method_dispatch(
        &self,
        object_span: Span,
        method_name: &str,
        arg_types: &[Type],
    ) -> (u16, bool) {
        let param_sig = arg_types
            .iter()
            .map(|arg_type| arg_type.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let mangled_name = format!("{method_name}({param_sig})");
        let simple_param_sig = arg_types
            .iter()
            .map(|argument_type| {
                let type_name = argument_type.to_string();
                type_name
                    .rsplit([':', '.'])
                    .next()
                    .unwrap_or(&type_name)
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(",");
        let simple_mangled_name = format!("{method_name}({simple_param_sig})");
        let arity_name = format!("{method_name}/{}", arg_types.len());

        let obj_name = self.get_lexeme(object_span);
        if obj_name == "this"
            && let Some(ref cls) = self.enclosing_class
        {
            let simple = cls.rsplit(['.', ':']).next().unwrap_or(cls);
            if let Some(layouts) = self.class_layouts
                && let Some(layout) = layouts.get(cls).or_else(|| layouts.get(simple))
                && let Some(&slot) = layout
                    .method_slots
                    .get(&mangled_name)
                    .or_else(|| layout.method_slots.get(&simple_mangled_name))
                    .or_else(|| layout.method_slots.get(&arity_name))
                    .or_else(|| layout.method_slots.get(method_name))
            {
                return (slot as u16, false);
            }
        }

        let obj_type = self.query_type(object_span);
        let is_interface = matches!(obj_type, Type::Interface { .. });
        if let Type::Class {
            ref symbol_name,
            ref type_arguments,
        }
        | Type::Interface {
            ref symbol_name,
            ref type_arguments,
        } = obj_type
        {
            let formatted_name = if !type_arguments.is_empty() && !symbol_name.contains('<') {
                format!(
                    "{symbol_name}<{}>",
                    type_arguments
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                symbol_name.clone()
            };
            let simple = formatted_name
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(&formatted_name);
            let raw_simple = symbol_name.rsplit(['.', ':']).next().unwrap_or(symbol_name);
            if let Some(layouts) = self.class_layouts
                && let Some(layout) = layouts
                    .get(&formatted_name)
                    .or_else(|| layouts.get(simple))
                    .or_else(|| layouts.get(symbol_name))
                    .or_else(|| layouts.get(raw_simple))
                && let Some(&slot) = layout
                    .method_slots
                    .get(&mangled_name)
                    .or_else(|| layout.method_slots.get(&simple_mangled_name))
                    .or_else(|| layout.method_slots.get(&arity_name))
                    .or_else(|| layout.method_slots.get(method_name))
            {
                if is_interface {
                    let target = ((layout.vtable_id as u16) << 8) | (slot as u16 & 0xFF);
                    return (target, true);
                } else {
                    return (slot as u16, false);
                }
            }
        }
        (0, is_interface)
    }

    fn resolve_receiver_vtable_id(&self, object_span: Span) -> Option<u32> {
        let layouts = self.class_layouts?;
        let obj_name = self.get_lexeme(object_span);
        if obj_name == "this"
            && let Some(ref cls) = self.enclosing_class
        {
            let simple = cls.rsplit(['.', ':']).next().unwrap_or(cls);
            if let Some(layout) = layouts.get(cls).or_else(|| layouts.get(simple)) {
                return Some(layout.vtable_id);
            }
        }

        let queried_type = self.query_type(object_span);
        let obj_type = queried_type.unwrap_alias();
        if let Type::Class {
            symbol_name,
            type_arguments,
        } = obj_type
        {
            let formatted_name = if !type_arguments.is_empty() && !symbol_name.contains('<') {
                format!(
                    "{symbol_name}<{}>",
                    type_arguments
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                symbol_name.clone()
            };
            let simple = formatted_name
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(&formatted_name);
            let raw_simple = symbol_name.rsplit(['.', ':']).next().unwrap_or(symbol_name);
            return layouts
                .get(&formatted_name)
                .or_else(|| layouts.get(simple))
                .or_else(|| layouts.get(symbol_name.as_str()))
                .or_else(|| layouts.get(raw_simple))
                .map(|layout| layout.vtable_id);
        }
        None
    }
}

/// Checks whether evaluating `expr` directly into a target register is safe (no short-circuit/assignment/control-flow side effects).
fn is_direct_eval_safe(expr: &Expression) -> bool {
    match expr {
        Expression::Literal { .. } | Expression::Identifier { .. } => true,
        Expression::Unary {
            operator, operand, ..
        } => {
            !matches!(
                operator,
                TokenKind::OperatorIncrement | TokenKind::OperatorDecrement
            ) && is_direct_eval_safe(operand)
        }
        Expression::Binary {
            operator,
            left,
            right,
            ..
        } => {
            !matches!(
                operator,
                TokenKind::OperatorLogicalAnd | TokenKind::OperatorLogicalOr
            ) && is_direct_eval_safe(left)
                && is_direct_eval_safe(right)
        }
        Expression::MemberAccess { object, .. } => is_direct_eval_safe(object),
        Expression::ArrayAccess { array, index, .. } => {
            is_direct_eval_safe(array) && is_direct_eval_safe(index)
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => is_direct_eval_safe(callee) && arguments.iter().all(is_direct_eval_safe),
        Expression::Cast { expression, .. } => is_direct_eval_safe(expression),
        _ => false,
    }
}

/// Returns `true` if `expr` inherently constructs or denotes a non-null heap reference.
fn is_static_non_null_literal_or_alloc(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::NewInstance { .. }
            | Expression::ArrayCreation { .. }
            | Expression::ArrayLiteral { .. }
            | Expression::Literal {
                value: LiteralValue::String(_),
                ..
            }
    )
}

/// Splits generic arguments at top-level commas, respecting nested angle brackets.
fn split_top_level_generic_arguments(arguments_string: &str) -> Vec<&str> {
    let mut split_arguments = Vec::new();
    let mut nesting_depth: usize = 0;
    let mut slice_start_index: usize = 0;
    for (char_index, character) in arguments_string.char_indices() {
        match character {
            '<' => nesting_depth += 1,
            '>' => {
                nesting_depth = nesting_depth.saturating_sub(1);
            }
            ',' if nesting_depth == 0 => {
                split_arguments.push(arguments_string[slice_start_index..char_index].trim());
                slice_start_index = char_index + 1;
            }
            _ => {}
        }
    }
    if slice_start_index < arguments_string.len() {
        split_arguments.push(arguments_string[slice_start_index..].trim());
    }
    split_arguments
}
