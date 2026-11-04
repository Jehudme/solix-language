//! Pre-emission AST inliner phase automatically inlining small functions and methods (<= 8 statements).

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::phases::lexing::Span;
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::control_flow::VariableDeclarationStatement;
use crate::phases::parser::statements::declarations::{
    ClassMember, Declaration, MethodDeclaration, Modifier,
};
use crate::phases::parser::statements::expressions::Expression;
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::phase::CompilationPhase;
use solix_common::{CompilationDiagnostic, Type};
use std::collections::HashMap;

/// Maximum recursion depth allowed during AST inlining expansion to prevent code blowup.
const MAX_INLINE_DEPTH: usize = 4;

/// Metadata describing an inlinable function or method.
#[derive(Debug, Clone)]
struct InlinableFunctionInfo {
    parameter_names: Vec<String>,
    parameter_types: Vec<TypeAnnotation>,
    return_type_annotation: TypeAnnotation,
    body_statements: Vec<Statement>,
}

/// AST inliner compilation phase.
#[derive(Debug, Default, Clone)]
pub struct AstInlinerPhase;

impl AstInlinerPhase {
    /// Constructs a new [`AstInlinerPhase`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for AstInlinerPhase {
    fn name(&self) -> &'static str {
        "AST Function & Method Inliner"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        let inline_threshold = context.options.inline_threshold;

        // 1. Collect inlinable candidate functions across all ASTs
        let mut inlinable_functions: HashMap<String, InlinableFunctionInfo> = HashMap::new();

        for ast in context.ast.iter().flatten() {
            for declaration in &ast.declarations {
                match declaration {
                    Declaration::Method(method) => {
                        try_register_inlinable_method(
                            method,
                            None,
                            inline_threshold,
                            &context.sources,
                            &mut inlinable_functions,
                        );
                    }
                    Declaration::Class(class_decl) => {
                        let class_name =
                            if (class_decl.name_span.id as usize) < context.sources.len() {
                                let source = &context.sources[class_decl.name_span.id as usize];
                                &source[class_decl.name_span.start as usize
                                    ..class_decl.name_span.end as usize]
                            } else {
                                ""
                            };
                        for member in &class_decl.members {
                            if let ClassMember::Method(method) = member {
                                try_register_inlinable_method(
                                    method,
                                    Some(class_name),
                                    inline_threshold,
                                    &context.sources,
                                    &mut inlinable_functions,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if inlinable_functions.is_empty() {
            return Ok(());
        }

        // 2. Traverse and inline call sites in root statements and function/method bodies
        let mut call_counter: usize = 0;

        let ast_count = context.ast.len();
        for ast_index in 0..ast_count {
            if let Some(mut current_ast) = context.ast[ast_index].take() {
                // Inline calls in root statements
                for statement in &mut current_ast.root_statements {
                    transform_statement(
                        statement,
                        &inlinable_functions,
                        context,
                        &mut call_counter,
                        0,
                    );
                }

                // Inline calls in declaration bodies
                for declaration in &mut current_ast.declarations {
                    match declaration {
                        Declaration::Method(method) => {
                            if let Some(body) = &mut method.body {
                                for statement in &mut body.statements {
                                    transform_statement(
                                        statement,
                                        &inlinable_functions,
                                        context,
                                        &mut call_counter,
                                        0,
                                    );
                                }
                            }
                        }
                        Declaration::Class(class_decl) => {
                            for member in &mut class_decl.members {
                                match member {
                                    ClassMember::Method(method) => {
                                        if let Some(body) = &mut method.body {
                                            for statement in &mut body.statements {
                                                transform_statement(
                                                    statement,
                                                    &inlinable_functions,
                                                    context,
                                                    &mut call_counter,
                                                    0,
                                                );
                                            }
                                        }
                                    }
                                    ClassMember::Constructor(constructor) => {
                                        for statement in &mut constructor.body.statements {
                                            transform_statement(
                                                statement,
                                                &inlinable_functions,
                                                context,
                                                &mut call_counter,
                                                0,
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                context.ast[ast_index] = Some(current_ast);
            }
        }

        Ok(())
    }
}

/// Evaluates whether a method is eligible for automatic inlining and registers its metadata.
fn try_register_inlinable_method(
    method: &MethodDeclaration,
    enclosing_class: Option<&str>,
    inline_threshold: usize,
    sources: &[String],
    inlinable_functions: &mut HashMap<String, InlinableFunctionInfo>,
) {
    // Prohibit virtual, override, native, abstract, or extern methods
    if method.modifiers.contains(&Modifier::Virtual)
        || method.modifiers.contains(&Modifier::Override)
        || method.modifiers.contains(&Modifier::Native)
        || method.modifiers.contains(&Modifier::Abstract)
        || method.modifiers.contains(&Modifier::Extern)
    {
        return;
    }

    let Some(body) = &method.body else {
        return;
    };

    let statement_count = body.statements.len();
    if statement_count == 0 {
        return;
    }

    let is_explicit = method.modifiers.contains(&Modifier::Inline);
    if !is_explicit && (inline_threshold == 0 || statement_count > inline_threshold) {
        return;
    }

    // Check if return type is void
    let is_void = match &method.return_type {
        TypeAnnotation::Primitive {
            span: type_span, ..
        } => {
            let type_source_index = type_span.id as usize;
            type_source_index < sources.len()
                && &sources[type_source_index][type_span.start as usize..type_span.end as usize]
                    == "void"
        }
        _ => false,
    };

    if is_void {
        // Void methods must not have ANY return statements
        if body.statements.iter().any(statement_has_return) {
            return;
        }
    } else {
        // Value-returning methods must end with a top-level Return statement
        let Some(Statement::Return(_)) = body.statements.last() else {
            return;
        };
        // And no mid-block returns allowed
        if statement_count > 1
            && body.statements[..statement_count - 1]
                .iter()
                .any(statement_has_return)
        {
            return;
        }
    }

    // Prohibit inlining bodies containing lambdas (to prevent closure capture corruption)
    if body.statements.iter().any(statement_has_lambda) {
        return;
    }

    // Extract function name
    let source_index = method.name_span.id as usize;
    if source_index >= sources.len() {
        return;
    }
    let method_name =
        &sources[source_index][method.name_span.start as usize..method.name_span.end as usize];

    // Recursion check: body cannot call method_name
    if body
        .statements
        .iter()
        .any(|statement| statement_calls_symbol(statement, sources, method_name))
    {
        return;
    }

    // Collect parameter names and types
    let mut parameter_names = Vec::with_capacity(method.parameters.len());
    let mut parameter_types = Vec::with_capacity(method.parameters.len());

    for parameter in &method.parameters {
        let param_source_index = parameter.name_span.id as usize;
        if param_source_index >= sources.len() {
            return;
        }
        let param_name = &sources[param_source_index]
            [parameter.name_span.start as usize..parameter.name_span.end as usize];
        parameter_names.push(param_name.to_string());
        parameter_types.push(parameter.param_type.clone());
    }

    let info = InlinableFunctionInfo {
        parameter_names,
        parameter_types,
        return_type_annotation: method.return_type.clone(),
        body_statements: body.statements.clone(),
    };

    if let Some(class_name) = enclosing_class {
        // Class methods: only static methods can be inlined as qualified static calls
        if method.modifiers.contains(&Modifier::Static) {
            let qualified_name = format!("{class_name}::{method_name}");
            inlinable_functions.insert(qualified_name, info);
        }
    } else {
        // Free / top-level functions
        inlinable_functions.insert(method_name.to_string(), info);
    }
}

/// Recursively inspects statements for presence of a `Return` statement.
fn statement_has_return(statement: &Statement) -> bool {
    match statement {
        Statement::Return(_) => true,
        Statement::Block(block) => block.statements.iter().any(statement_has_return),
        Statement::If(if_stmt) => {
            statement_has_return(&if_stmt.then_branch)
                || if_stmt
                    .else_branch
                    .as_deref()
                    .is_some_and(statement_has_return)
        }
        Statement::While(while_stmt) => statement_has_return(&while_stmt.body),
        Statement::DoWhile(do_while) => statement_has_return(&do_while.body),
        Statement::For(for_stmt) => statement_has_return(&for_stmt.body),
        Statement::Switch(switch_stmt) => {
            switch_stmt
                .cases
                .iter()
                .any(|case| case.statements.iter().any(statement_has_return))
                || switch_stmt
                    .default_case
                    .as_deref()
                    .is_some_and(|stmts| stmts.iter().any(statement_has_return))
        }
        Statement::Try(try_stmt) => {
            try_stmt
                .try_block
                .statements
                .iter()
                .any(statement_has_return)
                || try_stmt.catch_clauses.iter().any(|catch_clause| {
                    catch_clause
                        .block
                        .statements
                        .iter()
                        .any(statement_has_return)
                })
                || try_stmt
                    .finally_block
                    .as_ref()
                    .is_some_and(|finally_block| {
                        finally_block.statements.iter().any(statement_has_return)
                    })
        }
        _ => false,
    }
}

/// Recursively inspects statements for presence of a `Lambda` expression.
fn statement_has_lambda(statement: &Statement) -> bool {
    match statement {
        Statement::Block(block) => block.statements.iter().any(statement_has_lambda),
        Statement::If(if_stmt) => {
            expression_has_lambda(&if_stmt.condition)
                || statement_has_lambda(&if_stmt.then_branch)
                || if_stmt
                    .else_branch
                    .as_deref()
                    .is_some_and(statement_has_lambda)
        }
        Statement::While(while_stmt) => {
            expression_has_lambda(&while_stmt.condition) || statement_has_lambda(&while_stmt.body)
        }
        Statement::DoWhile(do_while) => {
            statement_has_lambda(&do_while.body) || expression_has_lambda(&do_while.condition)
        }
        Statement::For(for_stmt) => {
            for_stmt
                .initializer
                .as_deref()
                .is_some_and(statement_has_lambda)
                || for_stmt
                    .condition
                    .as_ref()
                    .is_some_and(expression_has_lambda)
                || for_stmt.update.as_ref().is_some_and(expression_has_lambda)
                || statement_has_lambda(&for_stmt.body)
        }
        Statement::Switch(switch_stmt) => {
            expression_has_lambda(&switch_stmt.discriminant)
                || switch_stmt.cases.iter().any(|case| {
                    expression_has_lambda(&case.case_value)
                        || case.statements.iter().any(statement_has_lambda)
                })
                || switch_stmt
                    .default_case
                    .as_deref()
                    .is_some_and(|stmts| stmts.iter().any(statement_has_lambda))
        }
        Statement::Return(return_stmt) => return_stmt
            .value
            .as_ref()
            .is_some_and(expression_has_lambda),
        Statement::VariableDeclaration(var_decl) => var_decl
            .initializer
            .as_ref()
            .is_some_and(expression_has_lambda),
        Statement::Expression(expr_stmt) => expression_has_lambda(&expr_stmt.expression),
        Statement::Try(try_stmt) => {
            try_stmt
                .try_block
                .statements
                .iter()
                .any(statement_has_lambda)
                || try_stmt.catch_clauses.iter().any(|catch_clause| {
                    catch_clause
                        .block
                        .statements
                        .iter()
                        .any(statement_has_lambda)
                })
                || try_stmt
                    .finally_block
                    .as_ref()
                    .is_some_and(|finally_block| {
                        finally_block.statements.iter().any(statement_has_lambda)
                    })
        }
        Statement::Throw(throw_stmt) => expression_has_lambda(&throw_stmt.expression),
        Statement::Assert(assert_stmt) => {
            expression_has_lambda(&assert_stmt.condition)
                || assert_stmt
                    .message
                    .as_ref()
                    .is_some_and(expression_has_lambda)
        }
        Statement::Exit(exit_stmt) => expression_has_lambda(&exit_stmt.code),
        _ => false,
    }
}

/// Recursively inspects expressions for presence of a `Lambda` expression.
fn expression_has_lambda(expression: &Expression) -> bool {
    match expression {
        Expression::Lambda { .. } => true,
        Expression::Binary { left, right, .. } => {
            expression_has_lambda(left) || expression_has_lambda(right)
        }
        Expression::Unary { operand, .. } => expression_has_lambda(operand),
        Expression::Assignment { target, value, .. } => {
            expression_has_lambda(target) || expression_has_lambda(value)
        }
        Expression::MemberAccess { object, .. } => expression_has_lambda(object),
        Expression::MethodCall {
            callee, arguments, ..
        } => expression_has_lambda(callee) || arguments.iter().any(expression_has_lambda),
        Expression::ArrayAccess { array, index, .. } => {
            expression_has_lambda(array) || expression_has_lambda(index)
        }
        Expression::ArrayCreation { size, .. } => expression_has_lambda(size),
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            elements.iter().any(expression_has_lambda)
        }
        Expression::NewInstance { arguments, .. } => arguments.iter().any(expression_has_lambda),
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => expression_has_lambda(inner),
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            expression_has_lambda(condition)
                || expression_has_lambda(then_branch)
                || expression_has_lambda(else_branch)
        }
        Expression::Block {
            statements, result, ..
        } => {
            statements.iter().any(statement_has_lambda)
                || result.as_deref().is_some_and(expression_has_lambda)
        }
        _ => false,
    }
}

/// Recursively inspects statements for calls to `target_name`.
fn statement_calls_symbol(statement: &Statement, sources: &[String], target_name: &str) -> bool {
    match statement {
        Statement::Block(block) => block
            .statements
            .iter()
            .any(|s| statement_calls_symbol(s, sources, target_name)),
        Statement::If(if_stmt) => {
            expression_calls_symbol(&if_stmt.condition, sources, target_name)
                || statement_calls_symbol(&if_stmt.then_branch, sources, target_name)
                || if_stmt
                    .else_branch
                    .as_deref()
                    .is_some_and(|s| statement_calls_symbol(s, sources, target_name))
        }
        Statement::While(while_stmt) => {
            expression_calls_symbol(&while_stmt.condition, sources, target_name)
                || statement_calls_symbol(&while_stmt.body, sources, target_name)
        }
        Statement::DoWhile(do_while) => {
            statement_calls_symbol(&do_while.body, sources, target_name)
                || expression_calls_symbol(&do_while.condition, sources, target_name)
        }
        Statement::For(for_stmt) => {
            for_stmt
                .initializer
                .as_deref()
                .is_some_and(|s| statement_calls_symbol(s, sources, target_name))
                || for_stmt
                    .condition
                    .as_ref()
                    .is_some_and(|e| expression_calls_symbol(e, sources, target_name))
                || for_stmt
                    .update
                    .as_ref()
                    .is_some_and(|e| expression_calls_symbol(e, sources, target_name))
                || statement_calls_symbol(&for_stmt.body, sources, target_name)
        }
        Statement::Switch(switch_stmt) => {
            expression_calls_symbol(&switch_stmt.discriminant, sources, target_name)
                || switch_stmt.cases.iter().any(|case| {
                    expression_calls_symbol(&case.case_value, sources, target_name)
                        || case
                            .statements
                            .iter()
                            .any(|s| statement_calls_symbol(s, sources, target_name))
                })
                || switch_stmt.default_case.as_deref().is_some_and(|stmts| {
                    stmts
                        .iter()
                        .any(|s| statement_calls_symbol(s, sources, target_name))
                })
        }
        Statement::Return(return_stmt) => return_stmt
            .value
            .as_ref()
            .is_some_and(|v| expression_calls_symbol(v, sources, target_name)),
        Statement::Throw(throw_stmt) => {
            expression_calls_symbol(&throw_stmt.expression, sources, target_name)
        }
        Statement::VariableDeclaration(var_decl) => var_decl
            .initializer
            .as_ref()
            .is_some_and(|init| expression_calls_symbol(init, sources, target_name)),
        Statement::Expression(expr_stmt) => {
            expression_calls_symbol(&expr_stmt.expression, sources, target_name)
        }
        Statement::Assert(assert_stmt) => {
            expression_calls_symbol(&assert_stmt.condition, sources, target_name)
                || assert_stmt
                    .message
                    .as_ref()
                    .is_some_and(|m| expression_calls_symbol(m, sources, target_name))
        }
        Statement::Exit(exit_stmt) => {
            expression_calls_symbol(&exit_stmt.code, sources, target_name)
        }
        _ => false,
    }
}

/// Recursively inspects expressions for calls to `target_name`.
fn expression_calls_symbol(expression: &Expression, sources: &[String], target_name: &str) -> bool {
    match expression {
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            let matches_callee = match callee.as_ref() {
                Expression::Identifier { span } => {
                    let source_index = span.id as usize;
                    if source_index < sources.len() {
                        let name = &sources[source_index][span.start as usize..span.end as usize];
                        name == target_name
                    } else {
                        false
                    }
                }
                Expression::MemberAccess { member_span, .. } => {
                    let source_index = member_span.id as usize;
                    if source_index < sources.len() {
                        let name = &sources[source_index]
                            [member_span.start as usize..member_span.end as usize];
                        name == target_name
                    } else {
                        false
                    }
                }
                _ => false,
            };
            matches_callee
                || expression_calls_symbol(callee, sources, target_name)
                || arguments
                    .iter()
                    .any(|arg| expression_calls_symbol(arg, sources, target_name))
        }
        Expression::Binary { left, right, .. } => {
            expression_calls_symbol(left, sources, target_name)
                || expression_calls_symbol(right, sources, target_name)
        }
        Expression::Unary { operand, .. } => expression_calls_symbol(operand, sources, target_name),
        Expression::Assignment { target, value, .. } => {
            expression_calls_symbol(target, sources, target_name)
                || expression_calls_symbol(value, sources, target_name)
        }
        Expression::MemberAccess { object, .. } => {
            expression_calls_symbol(object, sources, target_name)
        }
        Expression::ArrayAccess { array, index, .. } => {
            expression_calls_symbol(array, sources, target_name)
                || expression_calls_symbol(index, sources, target_name)
        }
        Expression::ArrayCreation { size, .. } => {
            expression_calls_symbol(size, sources, target_name)
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            elements
                .iter()
                .any(|elem| expression_calls_symbol(elem, sources, target_name))
        }
        Expression::NewInstance { arguments, .. } => arguments
            .iter()
            .any(|arg| expression_calls_symbol(arg, sources, target_name)),
        Expression::Cast { expression, .. } | Expression::Instanceof { expression, .. } => {
            expression_calls_symbol(expression, sources, target_name)
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            expression_calls_symbol(condition, sources, target_name)
                || expression_calls_symbol(then_branch, sources, target_name)
                || expression_calls_symbol(else_branch, sources, target_name)
        }
        Expression::Block {
            statements, result, ..
        } => {
            statements
                .iter()
                .any(|s| statement_calls_symbol(s, sources, target_name))
                || result
                    .as_deref()
                    .is_some_and(|r| expression_calls_symbol(r, sources, target_name))
        }
        _ => false,
    }
}

/// Recursively transforms statements, inlining candidate calls.
fn transform_statement(
    statement: &mut Statement,
    inlinable_functions: &HashMap<String, InlinableFunctionInfo>,
    context: &mut CompilationContext,
    call_counter: &mut usize,
    depth: usize,
) {
    if depth >= MAX_INLINE_DEPTH {
        return;
    }

    match statement {
        Statement::Block(block) => {
            for nested_statement in &mut block.statements {
                transform_statement(
                    nested_statement,
                    inlinable_functions,
                    context,
                    call_counter,
                    depth,
                );
            }
        }
        Statement::If(if_stmt) => {
            transform_expression(
                &mut if_stmt.condition,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            transform_statement(
                &mut if_stmt.then_branch,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            if let Some(else_branch) = &mut if_stmt.else_branch {
                transform_statement(
                    else_branch,
                    inlinable_functions,
                    context,
                    call_counter,
                    depth,
                );
            }
        }
        Statement::While(while_stmt) => {
            transform_expression(
                &mut while_stmt.condition,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            transform_statement(
                &mut while_stmt.body,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        Statement::DoWhile(do_while) => {
            transform_statement(
                &mut do_while.body,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            transform_expression(
                &mut do_while.condition,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        Statement::For(for_stmt) => {
            if let Some(initializer) = &mut for_stmt.initializer {
                transform_statement(
                    initializer,
                    inlinable_functions,
                    context,
                    call_counter,
                    depth,
                );
            }
            if let Some(condition) = &mut for_stmt.condition {
                transform_expression(condition, inlinable_functions, context, call_counter, depth);
            }
            if let Some(update) = &mut for_stmt.update {
                transform_expression(update, inlinable_functions, context, call_counter, depth);
            }
            transform_statement(
                &mut for_stmt.body,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        Statement::Switch(switch_stmt) => {
            transform_expression(
                &mut switch_stmt.discriminant,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            for case in &mut switch_stmt.cases {
                transform_expression(
                    &mut case.case_value,
                    inlinable_functions,
                    context,
                    call_counter,
                    depth,
                );
                for case_statement in &mut case.statements {
                    transform_statement(
                        case_statement,
                        inlinable_functions,
                        context,
                        call_counter,
                        depth,
                    );
                }
            }
            if let Some(default_statements) = &mut switch_stmt.default_case {
                for default_statement in default_statements {
                    transform_statement(
                        default_statement,
                        inlinable_functions,
                        context,
                        call_counter,
                        depth,
                    );
                }
            }
        }
        Statement::Return(return_stmt) => {
            if let Some(value) = &mut return_stmt.value {
                transform_expression(value, inlinable_functions, context, call_counter, depth);
            }
        }
        Statement::Throw(throw_stmt) => {
            transform_expression(
                &mut throw_stmt.expression,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            if let Some(initializer) = &mut var_decl.initializer {
                transform_expression(
                    initializer,
                    inlinable_functions,
                    context,
                    call_counter,
                    depth,
                );
            }
        }
        Statement::Expression(expr_stmt) => {
            transform_expression(
                &mut expr_stmt.expression,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        Statement::Assert(assert_stmt) => {
            transform_expression(
                &mut assert_stmt.condition,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            if let Some(message) = &mut assert_stmt.message {
                transform_expression(message, inlinable_functions, context, call_counter, depth);
            }
        }
        Statement::Exit(exit_stmt) => {
            transform_expression(
                &mut exit_stmt.code,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        _ => {}
    }
}

/// Recursively transforms expressions and inlines candidate method calls.
fn transform_expression(
    expression: &mut Expression,
    inlinable_functions: &HashMap<String, InlinableFunctionInfo>,
    context: &mut CompilationContext,
    call_counter: &mut usize,
    depth: usize,
) {
    if depth >= MAX_INLINE_DEPTH {
        return;
    }

    // First transform subexpressions
    match expression {
        Expression::Binary { left, right, .. } => {
            transform_expression(left, inlinable_functions, context, call_counter, depth);
            transform_expression(right, inlinable_functions, context, call_counter, depth);
        }
        Expression::Unary { operand, .. } => {
            transform_expression(operand, inlinable_functions, context, call_counter, depth);
        }
        Expression::Assignment { target, value, .. } => {
            transform_expression(target, inlinable_functions, context, call_counter, depth);
            transform_expression(value, inlinable_functions, context, call_counter, depth);
        }
        Expression::MemberAccess { object, .. } => {
            transform_expression(object, inlinable_functions, context, call_counter, depth);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            transform_expression(callee, inlinable_functions, context, call_counter, depth);
            for argument in arguments.iter_mut() {
                transform_expression(argument, inlinable_functions, context, call_counter, depth);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            transform_expression(array, inlinable_functions, context, call_counter, depth);
            transform_expression(index, inlinable_functions, context, call_counter, depth);
        }
        Expression::ArrayCreation { size, .. } => {
            transform_expression(size, inlinable_functions, context, call_counter, depth);
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for element in elements.iter_mut() {
                transform_expression(element, inlinable_functions, context, call_counter, depth);
            }
        }
        Expression::NewInstance { arguments, .. } => {
            for argument in arguments.iter_mut() {
                transform_expression(argument, inlinable_functions, context, call_counter, depth);
            }
        }
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => {
            transform_expression(inner, inlinable_functions, context, call_counter, depth);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            transform_expression(condition, inlinable_functions, context, call_counter, depth);
            transform_expression(
                then_branch,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
            transform_expression(
                else_branch,
                inlinable_functions,
                context,
                call_counter,
                depth,
            );
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements.iter_mut() {
                transform_statement(statement, inlinable_functions, context, call_counter, depth);
            }
            if let Some(res) = result {
                transform_expression(res, inlinable_functions, context, call_counter, depth);
            }
        }
        _ => {}
    }

    // Check if this expression is a MethodCall eligible for inlining
    if let Expression::MethodCall {
        span,
        callee,
        arguments,
        ..
    } = expression
    {
        let target_symbol = match callee.as_ref() {
            Expression::Identifier { span: id_span } => {
                let source_index = id_span.id as usize;
                if source_index < context.sources.len() {
                    let source = &context.sources[source_index];
                    Some(source[id_span.start as usize..id_span.end as usize].to_string())
                } else {
                    None
                }
            }
            Expression::MemberAccess {
                object,
                member_span,
                ..
            } => {
                let obj_source_index = object.span().id as usize;
                let member_source_index = member_span.id as usize;
                if obj_source_index < context.sources.len()
                    && member_source_index < context.sources.len()
                {
                    let obj_name = &context.sources[obj_source_index]
                        [object.span().start as usize..object.span().end as usize];
                    let member_name = &context.sources[member_source_index]
                        [member_span.start as usize..member_span.end as usize];
                    let qualified = format!("{obj_name}::{member_name}");
                    if inlinable_functions.contains_key(&qualified) {
                        Some(qualified)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(symbol_key) = target_symbol
            && let Some(target_info) = inlinable_functions.get(&symbol_key).cloned()
            && target_info.parameter_names.len() == arguments.len()
        {
            let call_span = *span;
            let inlined_block = inline_call_into_block(
                call_span,
                &target_info,
                std::mem::take(arguments),
                context,
                call_counter,
            );
            *expression = inlined_block;
        }
    }
}

/// Synthesizes an [`Expression::Block`] representing the inlined invocation.
fn inline_call_into_block(
    call_span: Span,
    target_info: &InlinableFunctionInfo,
    arguments: Vec<Expression>,
    context: &mut CompilationContext,
    call_counter: &mut usize,
) -> Expression {
    *call_counter += 1;
    let call_id = *call_counter;

    let mut renamed_identifiers: HashMap<String, Span> = HashMap::new();
    let mut block_statements: Vec<Statement> = Vec::new();

    // 1. Create let-bindings for each parameter: let __inl_{call_id}_{param} = arg;
    for (param_index, parameter_name) in target_info.parameter_names.iter().enumerate() {
        let unique_name = format!("__inl_{call_id}_{parameter_name}");
        let renamed_span = allocate_synthetic_identifier(context, &unique_name);
        renamed_identifiers.insert(parameter_name.clone(), renamed_span);

        let arg_expr = arguments[param_index].clone();
        let param_type = target_info.parameter_types[param_index].clone();

        let param_decl = VariableDeclarationStatement {
            span: renamed_span,
            name_span: renamed_span,
            is_const: false,
            var_type: param_type,
            initializer: Some(arg_expr),
        };
        block_statements.push(Statement::VariableDeclaration(param_decl));
    }

    // 2. Identify local variables inside callee body and allocate renamed spans
    let mut local_names = Vec::new();
    collect_local_variable_names(
        &target_info.body_statements,
        &context.sources,
        &mut local_names,
    );
    for local_name in local_names {
        if let std::collections::hash_map::Entry::Vacant(vacant_entry) =
            renamed_identifiers.entry(local_name)
        {
            let key = vacant_entry.key();
            let unique_name = format!("__inl_{call_id}_{key}");
            let renamed_span = allocate_synthetic_identifier(context, &unique_name);
            vacant_entry.insert(renamed_span);
        }
    }

    // 3. Deep-clone and substitute statements in callee body
    let statement_count = target_info.body_statements.len();
    let mut result_expression: Option<Box<Expression>> = None;

    for (statement_index, body_statement) in target_info.body_statements.iter().enumerate() {
        let is_last_statement = statement_index + 1 == statement_count;
        if is_last_statement && let Statement::Return(return_stmt) = body_statement {
            if let Some(mut return_value) = return_stmt.value.clone() {
                rename_expression(&mut return_value, &renamed_identifiers, &context.sources);
                result_expression = Some(Box::new(return_value));
            }
            break;
        }

        let mut cloned_statement = body_statement.clone();
        rename_statement(
            &mut cloned_statement,
            &renamed_identifiers,
            &context.sources,
        );
        block_statements.push(cloned_statement);
    }

    // 4. Update semantic binding expression_types so query_type works on the inlined block
    if let Some(artifacts) = context.binding_artifacts.as_mut()
        && let std::collections::hash_map::Entry::Vacant(vacant_entry) =
            artifacts.node_types.entry(call_span)
    {
        let resolved_return_type = match &target_info.return_type_annotation {
            TypeAnnotation::Primitive {
                span: type_span, ..
            } => {
                let src_id = type_span.id as usize;
                if src_id < context.sources.len() {
                    let text =
                        &context.sources[src_id][type_span.start as usize..type_span.end as usize];
                    match text {
                        "int8" => Type::Primitive(solix_common::PrimitiveKind::Int8),
                        "int16" => Type::Primitive(solix_common::PrimitiveKind::Int16),
                        "int32" => Type::Primitive(solix_common::PrimitiveKind::Int32),
                        "int64" => Type::Primitive(solix_common::PrimitiveKind::Int64),
                        "uint8" => Type::Primitive(solix_common::PrimitiveKind::UInt8),
                        "uint16" => Type::Primitive(solix_common::PrimitiveKind::UInt16),
                        "uint32" => Type::Primitive(solix_common::PrimitiveKind::UInt32),
                        "uint64" => Type::Primitive(solix_common::PrimitiveKind::UInt64),
                        "float32" => Type::Primitive(solix_common::PrimitiveKind::Float32),
                        "float64" => Type::Primitive(solix_common::PrimitiveKind::Float64),
                        "bool" => Type::Primitive(solix_common::PrimitiveKind::Bool),
                        "char" => Type::Primitive(solix_common::PrimitiveKind::Char),
                        _ => Type::Primitive(solix_common::PrimitiveKind::Void),
                    }
                } else {
                    Type::Primitive(solix_common::PrimitiveKind::Void)
                }
            }
            _ => Type::Primitive(solix_common::PrimitiveKind::Int32),
        };
        vacant_entry.insert(resolved_return_type);
    }

    Expression::Block {
        span: call_span,
        statements: block_statements,
        result: result_expression,
    }
}

/// Allocates a valid source span representing `name` by appending to the primary source buffer.
fn allocate_synthetic_identifier(context: &mut CompilationContext, name: &str) -> Span {
    if context.sources.is_empty() {
        context.sources.push(String::new());
        context.source_names.push("<synthetic>".to_string());
    }
    let start = context.sources[0].len() as u32;
    context.sources[0].push_str(name);
    let end = context.sources[0].len() as u32;
    Span::new(0, start, end)
}

/// Recursively collects local variable declaration names inside statement blocks.
fn collect_local_variable_names(
    statements: &[Statement],
    sources: &[String],
    local_names: &mut Vec<String>,
) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(var_decl) => {
                let source_index = var_decl.name_span.id as usize;
                if source_index < sources.len() {
                    let name = &sources[source_index]
                        [var_decl.name_span.start as usize..var_decl.name_span.end as usize];
                    local_names.push(name.to_string());
                }
            }
            Statement::Block(block) => {
                collect_local_variable_names(&block.statements, sources, local_names);
            }
            Statement::If(if_stmt) => {
                collect_local_variable_names(
                    std::slice::from_ref(&if_stmt.then_branch),
                    sources,
                    local_names,
                );
                if let Some(else_branch) = &if_stmt.else_branch {
                    collect_local_variable_names(
                        std::slice::from_ref(else_branch),
                        sources,
                        local_names,
                    );
                }
            }
            Statement::While(while_stmt) => {
                collect_local_variable_names(
                    std::slice::from_ref(&while_stmt.body),
                    sources,
                    local_names,
                );
            }
            Statement::DoWhile(do_while) => {
                collect_local_variable_names(
                    std::slice::from_ref(&do_while.body),
                    sources,
                    local_names,
                );
            }
            Statement::For(for_stmt) => {
                if let Some(initializer) = &for_stmt.initializer {
                    collect_local_variable_names(
                        std::slice::from_ref(initializer),
                        sources,
                        local_names,
                    );
                }
                collect_local_variable_names(
                    std::slice::from_ref(&for_stmt.body),
                    sources,
                    local_names,
                );
            }
            _ => {}
        }
    }
}

/// Substitutes identifiers matching `renamed_identifiers` with their hygienic synthetic spans.
fn rename_expression(
    expression: &mut Expression,
    renamed_identifiers: &HashMap<String, Span>,
    sources: &[String],
) {
    match expression {
        Expression::Identifier { span } => {
            let source_index = span.id as usize;
            if source_index < sources.len() {
                let name = &sources[source_index][span.start as usize..span.end as usize];
                if let Some(&new_span) = renamed_identifiers.get(name) {
                    *span = new_span;
                }
            }
        }
        Expression::Binary { left, right, .. } => {
            rename_expression(left, renamed_identifiers, sources);
            rename_expression(right, renamed_identifiers, sources);
        }
        Expression::Unary { operand, .. } => {
            rename_expression(operand, renamed_identifiers, sources);
        }
        Expression::Assignment { target, value, .. } => {
            rename_expression(target, renamed_identifiers, sources);
            rename_expression(value, renamed_identifiers, sources);
        }
        Expression::MemberAccess { object, .. } => {
            rename_expression(object, renamed_identifiers, sources);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            rename_expression(callee, renamed_identifiers, sources);
            for argument in arguments {
                rename_expression(argument, renamed_identifiers, sources);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            rename_expression(array, renamed_identifiers, sources);
            rename_expression(index, renamed_identifiers, sources);
        }
        Expression::ArrayCreation { size, .. } => {
            rename_expression(size, renamed_identifiers, sources);
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for element in elements {
                rename_expression(element, renamed_identifiers, sources);
            }
        }
        Expression::NewInstance { arguments, .. } => {
            for argument in arguments {
                rename_expression(argument, renamed_identifiers, sources);
            }
        }
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => {
            rename_expression(inner, renamed_identifiers, sources);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            rename_expression(condition, renamed_identifiers, sources);
            rename_expression(then_branch, renamed_identifiers, sources);
            rename_expression(else_branch, renamed_identifiers, sources);
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements {
                rename_statement(statement, renamed_identifiers, sources);
            }
            if let Some(res) = result {
                rename_expression(res, renamed_identifiers, sources);
            }
        }
        _ => {}
    }
}

/// Substitutes identifiers inside statements with their hygienic synthetic spans.
fn rename_statement(
    statement: &mut Statement,
    renamed_identifiers: &HashMap<String, Span>,
    sources: &[String],
) {
    match statement {
        Statement::Block(block) => {
            for nested_statement in &mut block.statements {
                rename_statement(nested_statement, renamed_identifiers, sources);
            }
        }
        Statement::If(if_stmt) => {
            rename_expression(&mut if_stmt.condition, renamed_identifiers, sources);
            rename_statement(&mut if_stmt.then_branch, renamed_identifiers, sources);
            if let Some(else_branch) = &mut if_stmt.else_branch {
                rename_statement(else_branch, renamed_identifiers, sources);
            }
        }
        Statement::While(while_stmt) => {
            rename_expression(&mut while_stmt.condition, renamed_identifiers, sources);
            rename_statement(&mut while_stmt.body, renamed_identifiers, sources);
        }
        Statement::DoWhile(do_while) => {
            rename_statement(&mut do_while.body, renamed_identifiers, sources);
            rename_expression(&mut do_while.condition, renamed_identifiers, sources);
        }
        Statement::For(for_stmt) => {
            if let Some(initializer) = &mut for_stmt.initializer {
                rename_statement(initializer, renamed_identifiers, sources);
            }
            if let Some(condition) = &mut for_stmt.condition {
                rename_expression(condition, renamed_identifiers, sources);
            }
            if let Some(update) = &mut for_stmt.update {
                rename_expression(update, renamed_identifiers, sources);
            }
            rename_statement(&mut for_stmt.body, renamed_identifiers, sources);
        }
        Statement::Switch(switch_stmt) => {
            rename_expression(&mut switch_stmt.discriminant, renamed_identifiers, sources);
            for case in &mut switch_stmt.cases {
                rename_expression(&mut case.case_value, renamed_identifiers, sources);
                for case_statement in &mut case.statements {
                    rename_statement(case_statement, renamed_identifiers, sources);
                }
            }
            if let Some(default_statements) = &mut switch_stmt.default_case {
                for default_statement in default_statements {
                    rename_statement(default_statement, renamed_identifiers, sources);
                }
            }
        }
        Statement::Return(return_stmt) => {
            if let Some(value) = &mut return_stmt.value {
                rename_expression(value, renamed_identifiers, sources);
            }
        }
        Statement::Throw(throw_stmt) => {
            rename_expression(&mut throw_stmt.expression, renamed_identifiers, sources);
        }
        Statement::VariableDeclaration(var_decl) => {
            let source_index = var_decl.name_span.id as usize;
            if source_index < sources.len() {
                let name = &sources[source_index]
                    [var_decl.name_span.start as usize..var_decl.name_span.end as usize];
                if let Some(&new_span) = renamed_identifiers.get(name) {
                    var_decl.name_span = new_span;
                }
            }
            if let Some(initializer) = &mut var_decl.initializer {
                rename_expression(initializer, renamed_identifiers, sources);
            }
        }
        Statement::Expression(expr_stmt) => {
            rename_expression(&mut expr_stmt.expression, renamed_identifiers, sources);
        }
        Statement::Assert(assert_stmt) => {
            rename_expression(&mut assert_stmt.condition, renamed_identifiers, sources);
            if let Some(message) = &mut assert_stmt.message {
                rename_expression(message, renamed_identifiers, sources);
            }
        }
        Statement::Exit(exit_stmt) => {
            rename_expression(&mut exit_stmt.code, renamed_identifiers, sources);
        }
        _ => {}
    }
}
