//! Free variable capture analysis and closure synthesis for first-class lambdas and function pointers.

#![forbid(unsafe_code)]

use crate::phases::binder::ClassLayout;
use crate::phases::emitter::constant_pool::ConstantPoolBuilder;
use crate::phases::emitter::control_flow::LoopContext;
use crate::phases::emitter::emitter::Emitter;
use crate::phases::emitter::register_allocator::RegisterAllocator;
use crate::phases::emitter::static_layout::StaticLayout;
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::control_flow::BlockStatement;
use crate::phases::parser::statements::expressions::{Expression, LambdaBody};
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode};
use solix_common::diagnostics::EmissionError;
use solix_common::span::Span;
use solix_common::types::Type;
use std::collections::{HashMap, HashSet};

/// Analyzes an AST expression tree to detect free variables originating from outer enclosing scopes.
#[must_use]
pub fn find_free_variables(
    expr: &Expression,
    outer_allocator: &RegisterAllocator,
    sources: Option<&[String]>,
) -> Vec<String> {
    let mut captured = Vec::new();
    let mut visited = HashSet::new();
    let mut local_scope = HashSet::new();

    if let Expression::Lambda { captures, .. } = expr {
        for cap_span in captures {
            if let Some(srcs) = sources {
                let id = cap_span.id as usize;
                if id < srcs.len() {
                    let name = &srcs[id][cap_span.start as usize..cap_span.end as usize];
                    if outer_allocator.lookup_local(name).is_some()
                        && visited.insert(name.to_string())
                    {
                        captured.push(name.to_string());
                    }
                }
            }
        }
    }

    if let Expression::Lambda { parameters, .. } = expr {
        for param in parameters {
            if let Some(srcs) = sources {
                let id = param.name_span.id as usize;
                if id < srcs.len() {
                    let name =
                        &srcs[id][param.name_span.start as usize..param.name_span.end as usize];
                    local_scope.insert(name.to_string());
                }
            }
        }
    }

    if let Expression::Lambda { body, .. } = expr {
        match body {
            LambdaBody::Expression(e) => {
                collect_identifiers_expr(
                    e,
                    &local_scope,
                    outer_allocator,
                    sources,
                    &mut captured,
                    &mut visited,
                );
            }
            LambdaBody::Block(b) => {
                collect_identifiers_block(
                    b,
                    &mut local_scope,
                    outer_allocator,
                    sources,
                    &mut captured,
                    &mut visited,
                );
            }
        }
    }

    captured.sort();
    captured.dedup();
    captured
}

fn collect_identifiers_expr(
    expr: &Expression,
    local_scope: &HashSet<String>,
    outer_allocator: &RegisterAllocator,
    sources: Option<&[String]>,
    captured: &mut Vec<String>,
    visited: &mut HashSet<String>,
) {
    match expr {
        Expression::Identifier { span } => {
            if let Some(srcs) = sources {
                let id = span.id as usize;
                if id < srcs.len() {
                    let name = &srcs[id][span.start as usize..span.end as usize];
                    if !local_scope.contains(name)
                        && outer_allocator.lookup_local(name).is_some()
                        && visited.insert(name.to_string())
                    {
                        captured.push(name.to_string());
                    }
                }
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_identifiers_expr(
                left,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
            collect_identifiers_expr(
                right,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::Unary { operand, .. } => {
            collect_identifiers_expr(
                operand,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::Assignment { target, value, .. } => {
            collect_identifiers_expr(
                target,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
            collect_identifiers_expr(
                value,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::MemberAccess { object, .. } => {
            collect_identifiers_expr(
                object,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            collect_identifiers_expr(
                callee,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
            for arg in arguments {
                collect_identifiers_expr(
                    arg,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            collect_identifiers_expr(
                array,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
            collect_identifiers_expr(
                index,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::ArrayCreation { size, .. } => {
            collect_identifiers_expr(
                size,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for elem in elements {
                collect_identifiers_expr(
                    elem,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
            }
        }
        Expression::NewInstance { arguments, .. } => {
            for arg in arguments {
                collect_identifiers_expr(
                    arg,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
            }
        }
        Expression::Cast { expression, .. } | Expression::Instanceof { expression, .. } => {
            collect_identifiers_expr(
                expression,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_identifiers_expr(
                condition,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
            collect_identifiers_expr(
                then_branch,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
            collect_identifiers_expr(
                else_branch,
                local_scope,
                outer_allocator,
                sources,
                captured,
                visited,
            );
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            let mut inner_scope = local_scope.clone();
            for param in parameters {
                if let Some(srcs) = sources {
                    let id = param.name_span.id as usize;
                    if id < srcs.len() {
                        let name =
                            &srcs[id][param.name_span.start as usize..param.name_span.end as usize];
                        inner_scope.insert(name.to_string());
                    }
                }
            }
            match body {
                LambdaBody::Expression(e) => {
                    collect_identifiers_expr(
                        e,
                        &inner_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
                LambdaBody::Block(b) => {
                    collect_identifiers_block(
                        b,
                        &mut inner_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
            }
        }
        _ => {}
    }
}

fn collect_identifiers_block(
    block: &BlockStatement,
    local_scope: &mut HashSet<String>,
    outer_allocator: &RegisterAllocator,
    sources: Option<&[String]>,
    captured: &mut Vec<String>,
    visited: &mut HashSet<String>,
) {
    for stmt in &block.statements {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                if let Some(init) = &var_decl.initializer {
                    collect_identifiers_expr(
                        init,
                        local_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
                if let Some(srcs) = sources {
                    let id = var_decl.name_span.id as usize;
                    if id < srcs.len() {
                        let name = &srcs[id]
                            [var_decl.name_span.start as usize..var_decl.name_span.end as usize];
                        local_scope.insert(name.to_string());
                    }
                }
            }
            Statement::Expression(es) => {
                collect_identifiers_expr(
                    &es.expression,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
            }
            Statement::If(if_stmt) => {
                collect_identifiers_expr(
                    &if_stmt.condition,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
                if let Statement::Block(b) = if_stmt.then_branch.as_ref() {
                    let mut then_scope = local_scope.clone();
                    collect_identifiers_block(
                        b,
                        &mut then_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
                if let Some(Statement::Block(b)) = if_stmt.else_branch.as_deref() {
                    let mut else_scope = local_scope.clone();
                    collect_identifiers_block(
                        b,
                        &mut else_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
            }
            Statement::While(while_stmt) => {
                collect_identifiers_expr(
                    &while_stmt.condition,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
                if let Statement::Block(b) = while_stmt.body.as_ref() {
                    let mut body_scope = local_scope.clone();
                    collect_identifiers_block(
                        b,
                        &mut body_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
            }
            Statement::DoWhile(do_while) => {
                collect_identifiers_expr(
                    &do_while.condition,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
                if let Statement::Block(b) = do_while.body.as_ref() {
                    let mut body_scope = local_scope.clone();
                    collect_identifiers_block(
                        b,
                        &mut body_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
            }
            Statement::For(for_stmt) => {
                let mut for_scope = local_scope.clone();
                if let Some(Statement::VariableDeclaration(var_decl)) =
                    for_stmt.initializer.as_deref()
                {
                    if let Some(ie) = &var_decl.initializer {
                        collect_identifiers_expr(
                            ie,
                            &for_scope,
                            outer_allocator,
                            sources,
                            captured,
                            visited,
                        );
                    }
                    if let Some(srcs) = sources {
                        let id = var_decl.name_span.id as usize;
                        if id < srcs.len() {
                            let name = &srcs[id][var_decl.name_span.start as usize
                                ..var_decl.name_span.end as usize];
                            for_scope.insert(name.to_string());
                        }
                    }
                }
                if let Some(cond) = &for_stmt.condition {
                    collect_identifiers_expr(
                        cond,
                        &for_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
                if let Some(upd) = &for_stmt.update {
                    collect_identifiers_expr(
                        upd,
                        &for_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
                if let Statement::Block(b) = for_stmt.body.as_ref() {
                    collect_identifiers_block(
                        b,
                        &mut for_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
            }
            Statement::Return(ret) => {
                if let Some(val) = &ret.value {
                    collect_identifiers_expr(
                        val,
                        local_scope,
                        outer_allocator,
                        sources,
                        captured,
                        visited,
                    );
                }
            }
            Statement::Throw(throw_stmt) => {
                collect_identifiers_expr(
                    &throw_stmt.expression,
                    local_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
            }
            Statement::Block(b) => {
                let mut inner_scope = local_scope.clone();
                collect_identifiers_block(
                    b,
                    &mut inner_scope,
                    outer_allocator,
                    sources,
                    captured,
                    visited,
                );
            }
            _ => {}
        }
    }
}

/// Synthesizes a standalone `CompiledFunction` representing a lambda closure body.
///
/// # Errors
/// Returns `EmissionError` if body compilation fails.
#[allow(clippy::too_many_arguments)]
pub fn synthesize_lambda_function(
    lambda_id: u32,
    lambda: &Expression,
    captured_vars: &[String],
    pool: &mut ConstantPoolBuilder,
    node_types: Option<&HashMap<Span, Type>>,
    sources: Option<&[String]>,
    class_layouts: Option<&HashMap<String, ClassLayout>>,
    function_ids: Option<&HashMap<String, u32>>,
    static_layout: Option<&StaticLayout>,
    synthesized_functions: Option<&mut Vec<CompiledFunction>>,
) -> Result<CompiledFunction, EmissionError> {
    if let Expression::Lambda {
        span,
        parameters,
        body,
        ..
    } = lambda
    {
        let symbol = format!("<lambda_{lambda_id}>");
        let param_count = parameters.len() as u16;
        let mut func = CompiledFunction::new(symbol, span.id, param_count, 0);
        let mut allocator = RegisterAllocator::new(param_count);

        // Bind incoming lambda parameters
        for param in parameters {
            let name = if let Some(srcs) = sources {
                let id = param.name_span.id as usize;
                if id < srcs.len() {
                    srcs[id][param.name_span.start as usize..param.name_span.end as usize]
                        .to_string()
                } else {
                    "arg".to_string()
                }
            } else {
                "arg".to_string()
            };
            allocator.alloc_param(name)?;
        }

        allocator.enter_scope();

        // Bind captured variables unpacked into registers at lambda entry
        if !captured_vars.is_empty() {
            let capture_base = allocator.alloc_window(captured_vars.len())?;
            func.emit(
                Instruction::format_call(
                    OpCode::UnpackCaptures,
                    0,
                    0,
                    capture_base,
                    captured_vars.len() as u8,
                ),
                Some(*span),
            );
            for (idx, var_name) in captured_vars.iter().enumerate() {
                let reg = capture_base + idx as u16;
                allocator.bind_local(var_name.clone(), reg);
            }
        }

        {
            let mut emitter = Emitter::new(&mut func, &mut allocator, pool);
            if let Some(types) = node_types {
                emitter = emitter.with_node_types(types);
            }
            if let Some(srcs) = sources {
                emitter = emitter.with_sources(srcs);
            }
            if let Some(layouts) = class_layouts {
                emitter = emitter.with_class_layouts(layouts);
            }
            if let Some(fids) = function_ids {
                emitter = emitter.with_function_ids(fids);
            }
            if let Some(slayout) = static_layout {
                emitter = emitter.with_static_layout(slayout);
            }
            if let Some(synth) = synthesized_functions {
                emitter = emitter.with_synthesized_functions(synth);
            }

            match body {
                LambdaBody::Expression(expr) => {
                    let r_res = emitter.emit_expression(expr, None)?;
                    emitter.emit_instruction(
                        Instruction::format_a(OpCode::ReturnValue, 0, r_res, 0, 0),
                        Some(expr.span()),
                    );
                    emitter.allocator.free_temp(r_res);
                }
                LambdaBody::Block(block) => {
                    let mut loop_stack = Vec::<LoopContext>::new();
                    emitter.emit_block_statement(block, &mut loop_stack)?;
                    let has_terminator = emitter
                        .function
                        .instructions
                        .last()
                        .and_then(|inst| inst.opcode(0).ok())
                        .is_some_and(|op| matches!(op, OpCode::Return | OpCode::ReturnValue));
                    if !has_terminator {
                        emitter.emit_instruction(
                            Instruction::format_a(OpCode::Return, 0, 0, 0, 0),
                            Some(block.span),
                        );
                    }
                }
            }
        }

        func.num_registers = allocator.peak_registers();
        Ok(func)
    } else {
        Err(EmissionError::CorruptBinaryContainer {
            section: "Lambda".to_string(),
            details: "Expression is not a lambda".to_string(),
        })
    }
}
