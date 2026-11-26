//! Pre-emission AST inliner phase automatically inlining small functions and methods (<= 8 statements).

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::CompilationDiagnostic;
use crate::phases::lexing::Span;
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::control_flow::VariableDeclarationStatement;
use crate::phases::parser::statements::declarations::{
    ClassMember, Declaration, MethodDeclaration, Modifier,
};
use crate::phases::parser::statements::expressions::{Expression, LambdaBody};
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::phase::CompilationPhase;
use solix_common::Type;
use std::collections::{HashMap, HashSet};

/// Maximum recursion depth allowed during AST inlining expansion to prevent code blowup.
const MAX_INLINE_DEPTH: usize = 4;

/// Metadata describing an inlinable function or method.
#[derive(Debug, Clone)]
struct InlinableFunctionInfo {
    parameter_names: Vec<String>,
    parameter_types: Vec<TypeAnnotation>,
    parameter_semantic_types: Vec<Option<Type>>,
    return_type_annotation: TypeAnnotation,
    resolved_return_type: Option<Type>,
    body_statements: Vec<Statement>,
    has_this_receiver: bool,
}

#[derive(Clone, Copy)]
struct EnclosingClassContext<'class> {
    qualified_name: &'class str,
    simple_fallback: Option<&'class str>,
    name_span: Span,
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

        let mut free_func_counts: HashMap<&str, usize> = HashMap::new();
        let mut class_name_counts: HashMap<String, usize> = HashMap::new();
        for (source_index, ast_option) in context.ast.iter().enumerate() {
            let Ok(source_id) = u16::try_from(source_index) else {
                break;
            };
            if context.is_external_source(source_id) {
                continue;
            }
            let ast = match ast_option {
                Some(ast_ref) => ast_ref,
                None => continue,
            };
            for declaration in &ast.declarations {
                if let Declaration::Method(method) = declaration
                    && source_index < context.sources.len()
                {
                    let name = &context.sources[source_index]
                        [method.name_span.start as usize..method.name_span.end as usize];
                    *free_func_counts.entry(name).or_insert(0) += 1;
                }
                if let Declaration::Class(class_decl) = declaration
                    && source_index < context.sources.len()
                {
                    let class_name = &context.sources[source_index]
                        [class_decl.name_span.start as usize..class_decl.name_span.end as usize];
                    *class_name_counts.entry(class_name.to_string()).or_insert(0) += 1;
                }
            }
        }

        let symbol_table = context
            .binding_artifacts
            .as_ref()
            .map(|artifacts| &artifacts.symbol_table);

        for (source_index, ast_option) in context.ast.iter().enumerate() {
            let Ok(source_id) = u16::try_from(source_index) else {
                break;
            };
            if context.is_external_source(source_id) {
                continue;
            }
            let ast = match ast_option {
                Some(ast_ref) => ast_ref,
                None => continue,
            };
            let source_code = &context.sources[source_index];
            for declaration in &ast.declarations {
                match declaration {
                    Declaration::Method(method) => {
                        let method_name = if source_index < context.sources.len() {
                            &source_code
                                [method.name_span.start as usize..method.name_span.end as usize]
                        } else {
                            ""
                        };
                        if free_func_counts.get(method_name).copied().unwrap_or(0) > 1 {
                            continue;
                        }
                        try_register_inlinable_method(
                            method,
                            None,
                            inline_threshold,
                            source_id,
                            &context.sources,
                            symbol_table,
                            &mut inlinable_functions,
                        );
                    }
                    Declaration::Class(class_decl) => {
                        let is_generic_class = !class_decl.generic_params.is_empty();
                        let class_name = if source_index < context.sources.len() {
                            &source_code[class_decl.name_span.start as usize
                                ..class_decl.name_span.end as usize]
                        } else {
                            ""
                        };

                        let fully_qualified_class = symbol_table
                            .and_then(|table| {
                                table
                                    .symbols
                                    .iter()
                                    .find(|entry| {
                                        entry.span == class_decl.span
                                            || (entry.source_id == source_id
                                                && entry.name == class_name)
                                    })
                                    .map(|entry| entry.fully_qualified_name.clone())
                            })
                            .unwrap_or_else(|| {
                                let file_package = ast
                                    .package
                                    .as_ref()
                                    .map(|package_decl| {
                                        package_decl
                                            .path
                                            .iter()
                                            .map(|span| {
                                                &source_code[span.start as usize..span.end as usize]
                                            })
                                            .filter(|segment| !segment.is_empty())
                                            .collect::<Vec<_>>()
                                            .join("::")
                                    })
                                    .unwrap_or_default();
                                let decl_namespace = class_decl
                                    .namespace_path
                                    .iter()
                                    .map(|span| {
                                        &source_code[span.start as usize..span.end as usize]
                                    })
                                    .filter(|segment| !segment.is_empty())
                                    .collect::<Vec<_>>()
                                    .join("::");
                                let effective_package = if file_package.is_empty() {
                                    decl_namespace
                                } else if decl_namespace.is_empty() {
                                    file_package
                                } else {
                                    format!("{file_package}::{decl_namespace}")
                                };
                                if effective_package.is_empty() {
                                    class_name.to_string()
                                } else {
                                    format!("{effective_package}::{class_name}")
                                }
                            });

                        let is_unique_class_name =
                            class_name_counts.get(class_name).copied().unwrap_or(0) <= 1;
                        let simple_fallback = if is_unique_class_name {
                            Some(class_name)
                        } else {
                            None
                        };

                        let mut method_counts: HashMap<&str, usize> = HashMap::new();
                        for member in &class_decl.members {
                            if let ClassMember::Method(method) = member
                                && source_index < context.sources.len()
                            {
                                let name = &source_code[method.name_span.start as usize
                                    ..method.name_span.end as usize];
                                *method_counts.entry(name).or_insert(0) += 1;
                            }
                        }

                        let class_ctx = EnclosingClassContext {
                            qualified_name: &fully_qualified_class,
                            simple_fallback,
                            name_span: class_decl.name_span,
                        };

                        for member in &class_decl.members {
                            if let ClassMember::Method(method) = member {
                                let method_name = if source_index < context.sources.len() {
                                    &source_code[method.name_span.start as usize
                                        ..method.name_span.end as usize]
                                } else {
                                    ""
                                };
                                let is_overloaded =
                                    method_counts.get(method_name).copied().unwrap_or(0) > 1;
                                if is_overloaded {
                                    continue;
                                }
                                if !method.modifiers.contains(&Modifier::Static) && is_generic_class
                                {
                                    continue;
                                }
                                try_register_inlinable_method(
                                    method,
                                    Some(class_ctx),
                                    inline_threshold,
                                    source_id,
                                    &context.sources,
                                    symbol_table,
                                    &mut inlinable_functions,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if inlinable_functions.is_empty() && inline_threshold == 0 {
            return Ok(());
        }

        // 2. Traverse and inline call sites in root statements and function/method bodies
        let mut call_counter: usize = 0;

        let ast_count = context.ast.len();
        for ast_index in 0..ast_count {
            let Ok(source_id) = u16::try_from(ast_index) else {
                break;
            };
            if context.is_external_source(source_id) {
                continue;
            }
            if let Some(mut current_ast) = context.ast[ast_index].take() {
                // Inline calls in root statements
                transform_statement_list(
                    &mut current_ast.root_statements,
                    &inlinable_functions,
                    context,
                    &mut call_counter,
                    0,
                    inline_threshold,
                );

                // Inline calls in declaration bodies
                for declaration in &mut current_ast.declarations {
                    match declaration {
                        Declaration::Method(method) => {
                            if let Some(body) = &mut method.body {
                                transform_statement_list(
                                    &mut body.statements,
                                    &inlinable_functions,
                                    context,
                                    &mut call_counter,
                                    0,
                                    inline_threshold,
                                );
                            }
                        }
                        Declaration::Class(class_decl) => {
                            for member in &mut class_decl.members {
                                match member {
                                    ClassMember::Method(method) => {
                                        if let Some(body) = &mut method.body {
                                            transform_statement_list(
                                                &mut body.statements,
                                                &inlinable_functions,
                                                context,
                                                &mut call_counter,
                                                0,
                                                inline_threshold,
                                            );
                                        }
                                    }
                                    ClassMember::Constructor(constructor) => {
                                        transform_statement_list(
                                            &mut constructor.body.statements,
                                            &inlinable_functions,
                                            context,
                                            &mut call_counter,
                                            0,
                                            inline_threshold,
                                        );
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

/// Attempts to flatten a sequence of early guard returns into a nested ternary expression.
fn try_flatten_guard_returns(statements: &[Statement]) -> Option<Expression> {
    if statements.len() < 2 {
        return None;
    }
    let last_return_expr = match statements.last()? {
        Statement::Return(return_stmt) => return_stmt.value.clone()?,
        _ => return None,
    };

    let mut guard_pairs = Vec::new();
    for statement in &statements[..statements.len() - 1] {
        match statement {
            Statement::If(if_stmt) if if_stmt.else_branch.is_none() => {
                let guard_ret = match &*if_stmt.then_branch {
                    Statement::Return(ret) => ret.value.clone()?,
                    Statement::Block(block) if block.statements.len() == 1 => {
                        match &block.statements[0] {
                            Statement::Return(ret) => ret.value.clone()?,
                            _ => return None,
                        }
                    }
                    _ => return None,
                };
                guard_pairs.push((if_stmt.span, if_stmt.condition.clone(), guard_ret));
            }
            _ => return None,
        }
    }

    let mut current_expr = last_return_expr;
    for (guard_span, guard_cond, guard_ret) in guard_pairs.into_iter().rev() {
        current_expr = Expression::Ternary {
            span: guard_span,
            condition: Box::new(guard_cond),
            then_branch: Box::new(guard_ret),
            else_branch: Box::new(current_expr),
        };
    }

    Some(current_expr)
}

/// Evaluates whether a method is eligible for automatic inlining and registers its metadata.
fn try_register_inlinable_method(
    method: &MethodDeclaration,
    enclosing_class: Option<EnclosingClassContext<'_>>,
    inline_threshold: usize,
    source_id: u16,
    sources: &[String],
    symbol_table: Option<&crate::phases::binder::symbols::SymbolTable>,
    inlinable_functions: &mut HashMap<String, InlinableFunctionInfo>,
) {
    // Prohibit virtual, override, native, abstract, extern, or generic methods
    if method.modifiers.contains(&Modifier::Virtual)
        || method.modifiers.contains(&Modifier::Override)
        || method.modifiers.contains(&Modifier::Native)
        || method.modifiers.contains(&Modifier::Abstract)
        || method.modifiers.contains(&Modifier::Extern)
        || !method.generic_params.is_empty()
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

    let effective_statements = if is_void {
        // Void methods must not have ANY return statements
        if body.statements.iter().any(statement_has_return) {
            return;
        }
        body.statements.clone()
    } else {
        // Value-returning methods must end with a top-level Return statement
        let Some(Statement::Return(_)) = body.statements.last() else {
            return;
        };
        // Mid-block returns allowed only if flattenable into ternary guards
        if statement_count > 1
            && body.statements[..statement_count - 1]
                .iter()
                .any(statement_has_return)
        {
            if let Some(ternary_expr) = try_flatten_guard_returns(&body.statements) {
                vec![Statement::Return(
                    crate::phases::parser::statements::control_flow::ReturnStatement {
                        span: method.span,
                        value: Some(ternary_expr),
                    },
                )]
            } else {
                return;
            }
        } else {
            body.statements.clone()
        }
    };

    // Prohibit inlining bodies containing lambdas (to prevent closure capture corruption)
    if effective_statements.iter().any(statement_has_lambda) {
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
    if effective_statements
        .iter()
        .any(|statement| statement_calls_symbol(statement, sources, method_name))
    {
        return;
    }

    let source_text = sources
        .get(source_id as usize)
        .map(String::as_str)
        .unwrap_or("");
    let resolved_return_type = symbol_table.map(|table| {
        crate::phases::binder::type_resolver::resolve_type_annotation(
            source_id,
            source_text,
            &method.return_type,
            table,
        )
    });

    // Collect parameter names, AST types, and optional semantic types
    let mut parameter_names = Vec::with_capacity(method.parameters.len());
    let mut parameter_types = Vec::with_capacity(method.parameters.len());
    let mut parameter_semantic_types = Vec::with_capacity(method.parameters.len());

    for parameter in &method.parameters {
        let param_source_index = parameter.name_span.id as usize;
        if param_source_index >= sources.len() {
            return;
        }
        let param_name = &sources[param_source_index]
            [parameter.name_span.start as usize..parameter.name_span.end as usize];
        parameter_names.push(param_name.to_string());
        parameter_types.push(parameter.param_type.clone());
        let param_semantic = symbol_table.map(|table| {
            crate::phases::binder::type_resolver::resolve_type_annotation(
                source_id,
                source_text,
                &parameter.param_type,
                table,
            )
        });
        parameter_semantic_types.push(param_semantic);
    }

    let info = InlinableFunctionInfo {
        parameter_names,
        parameter_types,
        parameter_semantic_types,
        return_type_annotation: method.return_type.clone(),
        resolved_return_type,
        body_statements: effective_statements,
        has_this_receiver: false,
    };

    if let Some(class_ctx) = enclosing_class {
        let class_name = class_ctx.qualified_name;
        let qualified_colon = format!("{class_name}::{method_name}");
        let qualified_dot = qualified_colon.replace("::", ".");
        if method.modifiers.contains(&Modifier::Static) {
            inlinable_functions.insert(qualified_colon.clone(), info.clone());
            inlinable_functions.insert(qualified_dot, info.clone());
            if let Some(simple_class) = class_ctx.simple_fallback {
                let simple_qualified = format!("{simple_class}::{method_name}");
                inlinable_functions.insert(simple_qualified, info);
            }
        } else {
            // Non-virtual non-override instance method: bind "this" as parameter 0
            // Only inline if return type is primitive non-void and body accesses "this"
            let is_primitive_return = match &method.return_type {
                TypeAnnotation::Primitive {
                    span: type_span, ..
                } => {
                    let type_source_index = type_span.id as usize;
                    type_source_index < sources.len()
                        && &sources[type_source_index]
                            [type_span.start as usize..type_span.end as usize]
                            != "void"
                }
                _ => false,
            };
            let accesses_this = info
                .body_statements
                .iter()
                .any(|stmt| statement_references_ident(stmt, sources, "this"));

            if is_primitive_return && accesses_this {
                let this_type_annotation = TypeAnnotation::Named {
                    span: class_ctx.name_span,
                    path: vec![class_ctx.name_span],
                };
                let this_semantic_type = Type::Class {
                    symbol_name: class_ctx.qualified_name.to_string(),
                    type_arguments: Vec::new(),
                };
                let mut instance_info = info;
                instance_info.has_this_receiver = true;
                instance_info.parameter_names.insert(0, "this".to_string());
                instance_info
                    .parameter_types
                    .insert(0, this_type_annotation);
                instance_info
                    .parameter_semantic_types
                    .insert(0, Some(this_semantic_type));
                inlinable_functions.insert(qualified_colon.clone(), instance_info.clone());
                inlinable_functions.insert(qualified_dot, instance_info.clone());
                if let Some(simple_class) = class_ctx.simple_fallback {
                    let simple_qualified = format!("{simple_class}::{method_name}");
                    inlinable_functions.insert(simple_qualified, instance_info);
                }
            }
        }
    } else {
        // Free / top-level functions
        inlinable_functions.insert(method_name.to_string(), info);
    }
}

/// Recursively inspects expressions to see if an identifier is referenced.
fn expression_references_ident(
    expression: &Expression,
    sources: &[String],
    target_name: &str,
) -> bool {
    match expression {
        Expression::Identifier { span } => {
            let source_index = span.id as usize;
            if source_index < sources.len() {
                &sources[source_index][span.start as usize..span.end as usize] == target_name
            } else {
                false
            }
        }
        Expression::MemberAccess { object, .. } => {
            expression_references_ident(object, sources, target_name)
        }
        Expression::Binary { left, right, .. } => {
            expression_references_ident(left, sources, target_name)
                || expression_references_ident(right, sources, target_name)
        }
        Expression::Unary { operand, .. } => {
            expression_references_ident(operand, sources, target_name)
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            expression_references_ident(condition, sources, target_name)
                || expression_references_ident(then_branch, sources, target_name)
                || expression_references_ident(else_branch, sources, target_name)
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            expression_references_ident(callee, sources, target_name)
                || arguments
                    .iter()
                    .any(|arg| expression_references_ident(arg, sources, target_name))
        }
        Expression::ArrayAccess { array, index, .. } => {
            expression_references_ident(array, sources, target_name)
                || expression_references_ident(index, sources, target_name)
        }
        Expression::Block {
            statements, result, ..
        } => {
            statements
                .iter()
                .any(|stmt| statement_references_ident(stmt, sources, target_name))
                || result
                    .as_ref()
                    .is_some_and(|res| expression_references_ident(res, sources, target_name))
        }
        _ => false,
    }
}

/// Recursively inspects statements to see if an identifier is referenced.
fn statement_references_ident(
    statement: &Statement,
    sources: &[String],
    target_name: &str,
) -> bool {
    match statement {
        Statement::Return(return_stmt) => return_stmt.value.as_ref().is_some_and(|return_val| {
            expression_references_ident(return_val, sources, target_name)
        }),
        Statement::Expression(expr_stmt) => {
            expression_references_ident(&expr_stmt.expression, sources, target_name)
        }
        Statement::VariableDeclaration(var_decl) => var_decl
            .initializer
            .as_ref()
            .is_some_and(|init| expression_references_ident(init, sources, target_name)),
        Statement::Block(block) => block
            .statements
            .iter()
            .any(|stmt| statement_references_ident(stmt, sources, target_name)),
        Statement::If(if_stmt) => {
            expression_references_ident(&if_stmt.condition, sources, target_name)
                || statement_references_ident(&if_stmt.then_branch, sources, target_name)
                || if_stmt
                    .else_branch
                    .as_deref()
                    .is_some_and(|stmt| statement_references_ident(stmt, sources, target_name))
        }
        _ => false,
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
            .any(|inner| statement_calls_symbol(inner, sources, target_name)),
        Statement::If(if_stmt) => {
            expression_calls_symbol(&if_stmt.condition, sources, target_name)
                || statement_calls_symbol(&if_stmt.then_branch, sources, target_name)
                || if_stmt.else_branch.as_deref().is_some_and(|else_stmt| {
                    statement_calls_symbol(else_stmt, sources, target_name)
                })
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
                .is_some_and(|init_stmt| statement_calls_symbol(init_stmt, sources, target_name))
                || for_stmt.condition.as_ref().is_some_and(|cond_expr| {
                    expression_calls_symbol(cond_expr, sources, target_name)
                })
                || for_stmt
                    .update
                    .as_ref()
                    .is_some_and(|upd_expr| expression_calls_symbol(upd_expr, sources, target_name))
                || statement_calls_symbol(&for_stmt.body, sources, target_name)
        }
        Statement::Switch(switch_stmt) => {
            expression_calls_symbol(&switch_stmt.discriminant, sources, target_name)
                || switch_stmt.cases.iter().any(|case| {
                    expression_calls_symbol(&case.case_value, sources, target_name)
                        || case.statements.iter().any(|case_stmt| {
                            statement_calls_symbol(case_stmt, sources, target_name)
                        })
                })
                || switch_stmt.default_case.as_deref().is_some_and(|stmts| {
                    stmts.iter().any(|default_stmt| {
                        statement_calls_symbol(default_stmt, sources, target_name)
                    })
                })
        }
        Statement::Return(return_stmt) => return_stmt
            .value
            .as_ref()
            .is_some_and(|ret_expr| expression_calls_symbol(ret_expr, sources, target_name)),
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
                    .is_some_and(|msg_expr| expression_calls_symbol(msg_expr, sources, target_name))
        }
        Statement::Exit(exit_stmt) => {
            expression_calls_symbol(&exit_stmt.code, sources, target_name)
        }
        Statement::Try(try_stmt) => {
            try_stmt
                .try_block
                .statements
                .iter()
                .any(|nested_stmt| statement_calls_symbol(nested_stmt, sources, target_name))
                || try_stmt.catch_clauses.iter().any(|catch_clause| {
                    catch_clause.block.statements.iter().any(|nested_stmt| {
                        statement_calls_symbol(nested_stmt, sources, target_name)
                    })
                })
                || try_stmt
                    .finally_block
                    .as_ref()
                    .is_some_and(|finally_block| {
                        finally_block.statements.iter().any(|nested_stmt| {
                            statement_calls_symbol(nested_stmt, sources, target_name)
                        })
                    })
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
                .any(|stmt| statement_calls_symbol(stmt, sources, target_name))
                || result
                    .as_deref()
                    .is_some_and(|ret_expr| expression_calls_symbol(ret_expr, sources, target_name))
        }
        _ => false,
    }
}

/// Checks if a local closure variable is safe to inline across `statements`.
fn try_collect_local_lambda(
    var_decl: &VariableDeclarationStatement,
    statements: &[Statement],
    inline_threshold: usize,
    sources: &[String],
) -> Option<(String, InlinableFunctionInfo)> {
    if inline_threshold == 0 {
        return None;
    }
    let name_src_idx = var_decl.name_span.id as usize;
    if name_src_idx >= sources.len() {
        return None;
    }
    let var_name =
        &sources[name_src_idx][var_decl.name_span.start as usize..var_decl.name_span.end as usize];
    if var_name.is_empty() {
        return None;
    }

    let Some(Expression::Lambda {
        parameters, body, ..
    }) = &var_decl.initializer
    else {
        return None;
    };

    let (raw_statements, body_expr_only) = match body {
        LambdaBody::Expression(expr) => (
            vec![Statement::Return(
                crate::phases::parser::statements::control_flow::ReturnStatement {
                    span: var_decl.span,
                    value: Some((**expr).clone()),
                },
            )],
            true,
        ),
        LambdaBody::Block(block) => (block.statements.clone(), false),
    };

    if raw_statements.is_empty() || raw_statements.len() > inline_threshold {
        return None;
    }
    if raw_statements.iter().any(statement_has_lambda) {
        return None;
    }

    let return_type_annotation = match &var_decl.var_type {
        TypeAnnotation::FunctionPointer { return_type, .. } => (**return_type).clone(),
        other => other.clone(),
    };
    let is_void = matches!(
        &return_type_annotation,
        TypeAnnotation::Primitive { span: type_span, .. }
            if (type_span.id as usize) < sources.len()
                && &sources[type_span.id as usize]
                    [type_span.start as usize..type_span.end as usize]
                    == "void"
    );

    let effective_statements = if body_expr_only {
        raw_statements
    } else if is_void {
        if raw_statements.iter().any(statement_has_return) {
            return None;
        }
        raw_statements
    } else {
        let Some(Statement::Return(_)) = raw_statements.last() else {
            return None;
        };
        let count = raw_statements.len();
        if count > 1 && raw_statements[..count - 1].iter().any(statement_has_return) {
            let flattened = try_flatten_guard_returns(&raw_statements)?;
            vec![Statement::Return(
                crate::phases::parser::statements::control_flow::ReturnStatement {
                    span: var_decl.span,
                    value: Some(flattened),
                },
            )]
        } else {
            raw_statements
        }
    };

    if effective_statements
        .iter()
        .any(|stmt| statement_calls_symbol(stmt, sources, var_name))
    {
        return None;
    }

    let fp_param_types: Option<&[TypeAnnotation]> = match &var_decl.var_type {
        TypeAnnotation::FunctionPointer { parameters, .. } => Some(parameters.as_slice()),
        _ => None,
    };

    let mut parameter_names = Vec::with_capacity(parameters.len());
    let mut parameter_types = Vec::with_capacity(parameters.len());
    let mut param_name_set = HashSet::new();
    for (param_idx, param) in parameters.iter().enumerate() {
        let param_src = param.name_span.id as usize;
        if param_src >= sources.len() {
            return None;
        }
        let param_name =
            &sources[param_src][param.name_span.start as usize..param.name_span.end as usize];
        let param_type = fp_param_types
            .and_then(|fp_list| fp_list.get(param_idx))
            .unwrap_or(&param.param_type)
            .clone();
        param_name_set.insert(param_name.to_string());
        parameter_names.push(param_name.to_string());
        parameter_types.push(param_type);
    }

    let mut local_decl_names = Vec::new();
    collect_local_variable_names(&effective_statements, sources, &mut local_decl_names);
    let mut bound_names = param_name_set;
    for local_name in local_decl_names {
        bound_names.insert(local_name);
    }

    let mut mutated_idents = HashSet::new();
    let mut escaped_idents = HashSet::new();
    collect_mutated_and_non_callee_idents(
        statements,
        sources,
        &mut mutated_idents,
        &mut escaped_idents,
    );
    if mutated_idents.contains(var_name) || escaped_idents.contains(var_name) {
        return None;
    }

    let mut body_idents = HashSet::new();
    for stmt in &effective_statements {
        collect_all_idents_in_statement(stmt, sources, &mut body_idents);
    }
    for body_ident in &body_idents {
        if !bound_names.contains(body_ident) && mutated_idents.contains(body_ident) {
            return None;
        }
    }

    let param_count = parameter_types.len();
    Some((
        var_name.to_string(),
        InlinableFunctionInfo {
            parameter_names,
            parameter_types,
            parameter_semantic_types: vec![None; param_count],
            return_type_annotation,
            resolved_return_type: None,
            body_statements: effective_statements,
            has_this_receiver: false,
        },
    ))
}

fn collect_mutated_and_non_callee_idents(
    statements: &[Statement],
    sources: &[String],
    mutated: &mut HashSet<String>,
    non_callee_uses: &mut HashSet<String>,
) {
    for stmt in statements {
        collect_stmt_mutated_and_non_callee(stmt, sources, mutated, non_callee_uses);
    }
}

fn collect_stmt_mutated_and_non_callee(
    stmt: &Statement,
    sources: &[String],
    mutated: &mut HashSet<String>,
    non_callee_uses: &mut HashSet<String>,
) {
    match stmt {
        Statement::Block(block) => {
            collect_mutated_and_non_callee_idents(
                &block.statements,
                sources,
                mutated,
                non_callee_uses,
            );
        }
        Statement::If(if_stmt) => {
            collect_expr_mutated_and_non_callee(
                &if_stmt.condition,
                sources,
                mutated,
                non_callee_uses,
            );
            collect_stmt_mutated_and_non_callee(
                &if_stmt.then_branch,
                sources,
                mutated,
                non_callee_uses,
            );
            if let Some(else_branch) = &if_stmt.else_branch {
                collect_stmt_mutated_and_non_callee(else_branch, sources, mutated, non_callee_uses);
            }
        }
        Statement::While(while_stmt) => {
            collect_expr_mutated_and_non_callee(
                &while_stmt.condition,
                sources,
                mutated,
                non_callee_uses,
            );
            collect_stmt_mutated_and_non_callee(
                &while_stmt.body,
                sources,
                mutated,
                non_callee_uses,
            );
        }
        Statement::DoWhile(do_while) => {
            collect_stmt_mutated_and_non_callee(&do_while.body, sources, mutated, non_callee_uses);
            collect_expr_mutated_and_non_callee(
                &do_while.condition,
                sources,
                mutated,
                non_callee_uses,
            );
        }
        Statement::For(for_stmt) => {
            if let Some(initializer) = &for_stmt.initializer {
                collect_stmt_mutated_and_non_callee(initializer, sources, mutated, non_callee_uses);
            }
            if let Some(condition) = &for_stmt.condition {
                collect_expr_mutated_and_non_callee(condition, sources, mutated, non_callee_uses);
            }
            if let Some(update) = &for_stmt.update {
                collect_expr_mutated_and_non_callee(update, sources, mutated, non_callee_uses);
            }
            collect_stmt_mutated_and_non_callee(&for_stmt.body, sources, mutated, non_callee_uses);
        }
        Statement::Switch(switch_stmt) => {
            collect_expr_mutated_and_non_callee(
                &switch_stmt.discriminant,
                sources,
                mutated,
                non_callee_uses,
            );
            for case in &switch_stmt.cases {
                collect_expr_mutated_and_non_callee(
                    &case.case_value,
                    sources,
                    mutated,
                    non_callee_uses,
                );
                collect_mutated_and_non_callee_idents(
                    &case.statements,
                    sources,
                    mutated,
                    non_callee_uses,
                );
            }
            if let Some(default_case) = &switch_stmt.default_case {
                collect_mutated_and_non_callee_idents(
                    default_case,
                    sources,
                    mutated,
                    non_callee_uses,
                );
            }
        }
        Statement::Return(ret) => {
            if let Some(value) = &ret.value {
                collect_expr_mutated_and_non_callee(value, sources, mutated, non_callee_uses);
            }
        }
        Statement::Throw(throw_stmt) => {
            collect_expr_mutated_and_non_callee(
                &throw_stmt.expression,
                sources,
                mutated,
                non_callee_uses,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            if let Some(initializer) = &var_decl.initializer
                && !matches!(initializer, Expression::Lambda { .. })
            {
                collect_expr_mutated_and_non_callee(initializer, sources, mutated, non_callee_uses);
            }
        }
        Statement::Expression(expr_stmt) => {
            collect_expr_mutated_and_non_callee(
                &expr_stmt.expression,
                sources,
                mutated,
                non_callee_uses,
            );
        }
        Statement::Assert(assert_stmt) => {
            collect_expr_mutated_and_non_callee(
                &assert_stmt.condition,
                sources,
                mutated,
                non_callee_uses,
            );
            if let Some(message) = &assert_stmt.message {
                collect_expr_mutated_and_non_callee(message, sources, mutated, non_callee_uses);
            }
        }
        Statement::Exit(exit_stmt) => {
            collect_expr_mutated_and_non_callee(&exit_stmt.code, sources, mutated, non_callee_uses);
        }
        Statement::Try(try_stmt) => {
            collect_mutated_and_non_callee_idents(
                &try_stmt.try_block.statements,
                sources,
                mutated,
                non_callee_uses,
            );
            for catch_clause in &try_stmt.catch_clauses {
                collect_mutated_and_non_callee_idents(
                    &catch_clause.block.statements,
                    sources,
                    mutated,
                    non_callee_uses,
                );
            }
            if let Some(finally_block) = &try_stmt.finally_block {
                collect_mutated_and_non_callee_idents(
                    &finally_block.statements,
                    sources,
                    mutated,
                    non_callee_uses,
                );
            }
        }
        _ => {}
    }
}

fn collect_expr_mutated_and_non_callee(
    expr: &Expression,
    sources: &[String],
    mutated: &mut HashSet<String>,
    non_callee_uses: &mut HashSet<String>,
) {
    match expr {
        Expression::Identifier { span } => {
            let source_index = span.id as usize;
            if source_index < sources.len() {
                let ident_text = &sources[source_index][span.start as usize..span.end as usize];
                non_callee_uses.insert(ident_text.to_string());
            }
        }
        Expression::Assignment { target, value, .. } => {
            if let Expression::Identifier { span } = target.as_ref() {
                let source_index = span.id as usize;
                if source_index < sources.len() {
                    let ident_text = &sources[source_index][span.start as usize..span.end as usize];
                    mutated.insert(ident_text.to_string());
                }
            } else {
                collect_expr_mutated_and_non_callee(target, sources, mutated, non_callee_uses);
            }
            collect_expr_mutated_and_non_callee(value, sources, mutated, non_callee_uses);
        }
        Expression::Unary {
            operator, operand, ..
        } => {
            if matches!(
                operator,
                crate::phases::lexing::TokenKind::OperatorIncrement
                    | crate::phases::lexing::TokenKind::OperatorDecrement
            ) && let Expression::Identifier { span } = operand.as_ref()
            {
                let source_index = span.id as usize;
                if source_index < sources.len() {
                    let ident_text = &sources[source_index][span.start as usize..span.end as usize];
                    mutated.insert(ident_text.to_string());
                }
            }
            collect_expr_mutated_and_non_callee(operand, sources, mutated, non_callee_uses);
        }
        Expression::Binary { left, right, .. } => {
            collect_expr_mutated_and_non_callee(left, sources, mutated, non_callee_uses);
            collect_expr_mutated_and_non_callee(right, sources, mutated, non_callee_uses);
        }
        Expression::MemberAccess { object, .. } => {
            collect_expr_mutated_and_non_callee(object, sources, mutated, non_callee_uses);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            if !matches!(callee.as_ref(), Expression::Identifier { .. }) {
                collect_expr_mutated_and_non_callee(callee, sources, mutated, non_callee_uses);
            }
            for arg in arguments {
                collect_expr_mutated_and_non_callee(arg, sources, mutated, non_callee_uses);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            collect_expr_mutated_and_non_callee(array, sources, mutated, non_callee_uses);
            collect_expr_mutated_and_non_callee(index, sources, mutated, non_callee_uses);
        }
        Expression::ArrayCreation { size, .. } => {
            collect_expr_mutated_and_non_callee(size, sources, mutated, non_callee_uses);
        }
        Expression::ArrayLiteral { elements, .. }
        | Expression::TupleCreation { elements, .. }
        | Expression::NewInstance {
            arguments: elements,
            ..
        } => {
            for elem in elements {
                collect_expr_mutated_and_non_callee(elem, sources, mutated, non_callee_uses);
            }
        }
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => {
            collect_expr_mutated_and_non_callee(inner, sources, mutated, non_callee_uses);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_mutated_and_non_callee(condition, sources, mutated, non_callee_uses);
            collect_expr_mutated_and_non_callee(then_branch, sources, mutated, non_callee_uses);
            collect_expr_mutated_and_non_callee(else_branch, sources, mutated, non_callee_uses);
        }
        Expression::Lambda { body, .. } => match body {
            LambdaBody::Expression(body_expr) => {
                collect_expr_mutated_and_non_callee(body_expr, sources, mutated, non_callee_uses);
            }
            LambdaBody::Block(block) => {
                collect_mutated_and_non_callee_idents(
                    &block.statements,
                    sources,
                    mutated,
                    non_callee_uses,
                );
            }
        },
        Expression::Block {
            statements, result, ..
        } => {
            collect_mutated_and_non_callee_idents(statements, sources, mutated, non_callee_uses);
            if let Some(res) = result {
                collect_expr_mutated_and_non_callee(res, sources, mutated, non_callee_uses);
            }
        }
        _ => {}
    }
}

fn collect_all_idents_in_statement(
    stmt: &Statement,
    sources: &[String],
    idents: &mut HashSet<String>,
) {
    match stmt {
        Statement::Block(block) => {
            for inner in &block.statements {
                collect_all_idents_in_statement(inner, sources, idents);
            }
        }
        Statement::If(if_stmt) => {
            collect_all_idents_in_expr(&if_stmt.condition, sources, idents);
            collect_all_idents_in_statement(&if_stmt.then_branch, sources, idents);
            if let Some(else_branch) = &if_stmt.else_branch {
                collect_all_idents_in_statement(else_branch, sources, idents);
            }
        }
        Statement::While(while_stmt) => {
            collect_all_idents_in_expr(&while_stmt.condition, sources, idents);
            collect_all_idents_in_statement(&while_stmt.body, sources, idents);
        }
        Statement::DoWhile(do_while) => {
            collect_all_idents_in_statement(&do_while.body, sources, idents);
            collect_all_idents_in_expr(&do_while.condition, sources, idents);
        }
        Statement::For(for_stmt) => {
            if let Some(init) = &for_stmt.initializer {
                collect_all_idents_in_statement(init, sources, idents);
            }
            if let Some(cond) = &for_stmt.condition {
                collect_all_idents_in_expr(cond, sources, idents);
            }
            if let Some(upd) = &for_stmt.update {
                collect_all_idents_in_expr(upd, sources, idents);
            }
            collect_all_idents_in_statement(&for_stmt.body, sources, idents);
        }
        Statement::Return(ret) => {
            if let Some(val) = &ret.value {
                collect_all_idents_in_expr(val, sources, idents);
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            if let Some(init) = &var_decl.initializer {
                collect_all_idents_in_expr(init, sources, idents);
            }
        }
        Statement::Expression(expr_stmt) => {
            collect_all_idents_in_expr(&expr_stmt.expression, sources, idents);
        }
        _ => {}
    }
}

fn collect_all_idents_in_expr(expr: &Expression, sources: &[String], idents: &mut HashSet<String>) {
    match expr {
        Expression::Identifier { span } => {
            let source_index = span.id as usize;
            if source_index < sources.len() {
                idents.insert(
                    sources[source_index][span.start as usize..span.end as usize].to_string(),
                );
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_all_idents_in_expr(left, sources, idents);
            collect_all_idents_in_expr(right, sources, idents);
        }
        Expression::Unary { operand, .. } => collect_all_idents_in_expr(operand, sources, idents),
        Expression::Assignment { target, value, .. } => {
            collect_all_idents_in_expr(target, sources, idents);
            collect_all_idents_in_expr(value, sources, idents);
        }
        Expression::MemberAccess { object, .. } => {
            collect_all_idents_in_expr(object, sources, idents)
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            collect_all_idents_in_expr(callee, sources, idents);
            for arg in arguments {
                collect_all_idents_in_expr(arg, sources, idents);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            collect_all_idents_in_expr(array, sources, idents);
            collect_all_idents_in_expr(index, sources, idents);
        }
        Expression::ArrayCreation { size, .. } => collect_all_idents_in_expr(size, sources, idents),
        Expression::ArrayLiteral { elements, .. }
        | Expression::TupleCreation { elements, .. }
        | Expression::NewInstance {
            arguments: elements,
            ..
        } => {
            for elem in elements {
                collect_all_idents_in_expr(elem, sources, idents);
            }
        }
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => {
            collect_all_idents_in_expr(inner, sources, idents);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_all_idents_in_expr(condition, sources, idents);
            collect_all_idents_in_expr(then_branch, sources, idents);
            collect_all_idents_in_expr(else_branch, sources, idents);
        }
        Expression::Block {
            statements, result, ..
        } => {
            for inner in statements {
                collect_all_idents_in_statement(inner, sources, idents);
            }
            if let Some(res) = result {
                collect_all_idents_in_expr(res, sources, idents);
            }
        }
        _ => {}
    }
}

/// Recursively transforms a statement slice, registering eligible local closures into a scoped inline map.
fn transform_statement_list(
    statements: &mut [Statement],
    inlinable_functions: &HashMap<String, InlinableFunctionInfo>,
    context: &mut CompilationContext,
    call_counter: &mut usize,
    depth: usize,
    inline_threshold: usize,
) {
    let mut local_lambdas: Vec<(String, InlinableFunctionInfo)> = Vec::new();
    for statement in statements.iter() {
        if let Statement::VariableDeclaration(var_decl) = statement
            && let Some(entry) =
                try_collect_local_lambda(var_decl, statements, inline_threshold, &context.sources)
        {
            local_lambdas.push(entry);
        }
    }

    if local_lambdas.is_empty() {
        for statement in statements.iter_mut() {
            transform_statement(statement, inlinable_functions, context, call_counter, depth);
        }
    } else {
        let mut scoped_map = inlinable_functions.clone();
        for (lambda_name, lambda_info) in local_lambdas {
            scoped_map.insert(lambda_name, lambda_info);
        }
        for statement in statements.iter_mut() {
            transform_statement(statement, &scoped_map, context, call_counter, depth);
        }
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
            transform_statement_list(
                &mut block.statements,
                inlinable_functions,
                context,
                call_counter,
                depth,
                context.options.inline_threshold,
            );
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
        Statement::Try(try_stmt) => {
            for try_statement in &mut try_stmt.try_block.statements {
                transform_statement(
                    try_statement,
                    inlinable_functions,
                    context,
                    call_counter,
                    depth,
                );
            }
            for catch_clause in &mut try_stmt.catch_clauses {
                for catch_statement in &mut catch_clause.block.statements {
                    transform_statement(
                        catch_statement,
                        inlinable_functions,
                        context,
                        call_counter,
                        depth,
                    );
                }
            }
            if let Some(finally_block) = &mut try_stmt.finally_block {
                for finally_statement in &mut finally_block.statements {
                    transform_statement(
                        finally_statement,
                        inlinable_functions,
                        context,
                        call_counter,
                        depth,
                    );
                }
            }
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
        let (target_symbol, this_receiver_arg) = match callee.as_ref() {
            Expression::Identifier { span: id_span } => {
                if let Some(artifacts) = context.binding_artifacts.as_ref() {
                    if let Some(&symbol_id) = artifacts
                        .resolved_symbols
                        .get(id_span)
                        .or_else(|| artifacts.resolved_symbols.get(&callee.span()))
                        && let Some(symbol_entry) = artifacts.symbol_table.get_symbol(symbol_id)
                    {
                        if context.is_external_source(symbol_entry.source_id) {
                            (None, None)
                        } else if let Some(parent_id) = symbol_entry.parent_symbol {
                            if let Some(parent_class) = artifacts.symbol_table.get_symbol(parent_id)
                            {
                                let fully_qualified = format!(
                                    "{}::{}",
                                    parent_class.fully_qualified_name, symbol_entry.name
                                );
                                let colon_key = fully_qualified.replace('.', "::");
                                let dot_key = fully_qualified.replace("::", ".");
                                let matched_key = if inlinable_functions.contains_key(&colon_key) {
                                    Some(colon_key)
                                } else if inlinable_functions.contains_key(&dot_key) {
                                    Some(dot_key)
                                } else if inlinable_functions.contains_key(&fully_qualified) {
                                    Some(fully_qualified)
                                } else {
                                    None
                                };
                                if let Some(key) = matched_key
                                    && inlinable_functions
                                        .get(&key)
                                        .is_some_and(|info| !info.has_this_receiver)
                                {
                                    (Some(key), None)
                                } else {
                                    (None, None)
                                }
                            } else {
                                (None, None)
                            }
                        } else if inlinable_functions
                            .get(&symbol_entry.name)
                            .is_some_and(|info| !info.has_this_receiver)
                        {
                            (Some(symbol_entry.name.clone()), None)
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    let source_index = id_span.id as usize;
                    if source_index < context.sources.len() {
                        let source = &context.sources[source_index];
                        let name = &source[id_span.start as usize..id_span.end as usize];
                        if inlinable_functions
                            .get(name)
                            .is_some_and(|info| !info.has_this_receiver)
                        {
                            (Some(name.to_string()), None)
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                }
            }
            Expression::MemberAccess {
                object,
                member_span,
                ..
            } => {
                let member_source_index = member_span.id as usize;
                if member_source_index < context.sources.len() {
                    let member_name = &context.sources[member_source_index]
                        [member_span.start as usize..member_span.end as usize];

                    // Priority 1: Check resolved_symbols from binder
                    let resolved_target =
                        context.binding_artifacts.as_ref().and_then(|artifacts| {
                            artifacts
                                .resolved_symbols
                                .get(member_span)
                                .or_else(|| artifacts.resolved_symbols.get(&callee.span()))
                                .and_then(|&symbol_id| artifacts.symbol_table.get_symbol(symbol_id))
                                .and_then(|method_symbol| {
                                    let parent_id = method_symbol.parent_symbol?;
                                    let parent_class =
                                        artifacts.symbol_table.get_symbol(parent_id)?;
                                    let fully_qualified = format!(
                                        "{}::{}",
                                        parent_class.fully_qualified_name, method_symbol.name
                                    );
                                    let colon_target = fully_qualified.replace('.', "::");
                                    let dot_target = fully_qualified.replace("::", ".");
                                    if inlinable_functions.contains_key(&colon_target) {
                                        Some(colon_target)
                                    } else if inlinable_functions.contains_key(&dot_target) {
                                        Some(dot_target)
                                    } else if inlinable_functions.contains_key(&fully_qualified) {
                                        Some(fully_qualified)
                                    } else {
                                        None
                                    }
                                })
                        });

                    if let Some(target_key) = resolved_target {
                        let has_receiver = inlinable_functions
                            .get(&target_key)
                            .map(|info| info.has_this_receiver)
                            .unwrap_or(false);
                        if has_receiver {
                            (Some(target_key), Some((**object).clone()))
                        } else {
                            (Some(target_key), None)
                        }
                    } else {
                        let obj_source_index = object.span().id as usize;
                        let obj_name = if obj_source_index < context.sources.len() {
                            &context.sources[obj_source_index]
                                [object.span().start as usize..object.span().end as usize]
                        } else {
                            ""
                        };

                        let static_qualified = format!("{obj_name}::{member_name}");
                        let colon_static = static_qualified.replace('.', "::");
                        if let Some(target_info) = inlinable_functions
                            .get(&colon_static)
                            .or_else(|| inlinable_functions.get(&static_qualified))
                            && !target_info.has_this_receiver
                        {
                            let matched_key = if inlinable_functions.contains_key(&colon_static) {
                                colon_static
                            } else {
                                static_qualified
                            };
                            (Some(matched_key), None)
                        } else {
                            // Check if object is an instance with a known Class type
                            let obj_type = context
                                .binding_artifacts
                                .as_ref()
                                .and_then(|artifacts| artifacts.node_types.get(&object.span()))
                                .cloned();

                            if let Some(Type::Class { symbol_name, .. }) = obj_type {
                                let base_class =
                                    symbol_name.split('<').next().unwrap_or(&symbol_name);
                                let colon_base = base_class.replace('.', "::");
                                let simple_class =
                                    base_class.rsplit(['.', ':']).next().unwrap_or(base_class);

                                let colon_instance = format!("{colon_base}::{member_name}");
                                let fqn_instance = format!("{base_class}::{member_name}");
                                let simple_instance = format!("{simple_class}::{member_name}");

                                if let Some(target_info) = inlinable_functions
                                    .get(&colon_instance)
                                    .or_else(|| inlinable_functions.get(&fqn_instance))
                                    .or_else(|| inlinable_functions.get(&simple_instance))
                                    && target_info.has_this_receiver
                                {
                                    let matched_key =
                                        if inlinable_functions.contains_key(&colon_instance) {
                                            colon_instance
                                        } else if inlinable_functions.contains_key(&fqn_instance) {
                                            fqn_instance
                                        } else {
                                            simple_instance
                                        };
                                    (Some(matched_key), Some((**object).clone()))
                                } else {
                                    (None, None)
                                }
                            } else {
                                (None, None)
                            }
                        }
                    }
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };

        if let Some(symbol_key) = target_symbol
            && let Some(target_info) = inlinable_functions.get(&symbol_key).cloned()
        {
            let mut call_args = Vec::new();
            if let Some(receiver_expr) = this_receiver_arg {
                call_args.push(receiver_expr);
            }
            call_args.extend(std::mem::take(arguments));

            if target_info.parameter_names.len() == call_args.len() {
                let call_span = *span;
                let inlined_block = inline_call_into_block(
                    call_span,
                    &target_info,
                    call_args,
                    context,
                    call_counter,
                );
                *expression = inlined_block;
            }
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

        // Record argument's type for the synthetic renamed_span so Emitter knows its type
        if let Some(artifacts) = context.binding_artifacts.as_mut() {
            let inferred_param_type =
                artifacts
                    .node_types
                    .get(&arg_expr.span())
                    .cloned()
                    .or_else(|| {
                        target_info
                            .parameter_semantic_types
                            .get(param_index)
                            .and_then(|semantic_type| semantic_type.clone())
                    });
            if let Some(evaluated_type) = inferred_param_type {
                artifacts.node_types.insert(renamed_span, evaluated_type);
            }
        }

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
                let node_types = context
                    .binding_artifacts
                    .as_mut()
                    .map(|artifacts| &mut artifacts.node_types);
                rename_expression(
                    &mut return_value,
                    &renamed_identifiers,
                    &context.sources,
                    node_types,
                );
                result_expression = Some(Box::new(return_value));
            }
            break;
        }

        let mut cloned_statement = body_statement.clone();
        let node_types = context
            .binding_artifacts
            .as_mut()
            .map(|artifacts| &mut artifacts.node_types);
        rename_statement(
            &mut cloned_statement,
            &renamed_identifiers,
            &context.sources,
            node_types,
        );
        block_statements.push(cloned_statement);
    }

    // 4. Update semantic binding expression_types so query_type works on the inlined block
    if let Some(artifacts) = context.binding_artifacts.as_mut()
        && !artifacts.node_types.contains_key(&call_span)
    {
        let resolved_return_type = target_info.resolved_return_type.clone().unwrap_or_else(|| {
            match &target_info.return_type_annotation {
                TypeAnnotation::Primitive {
                    span: type_span, ..
                } => {
                    let src_id = type_span.id as usize;
                    if src_id < context.sources.len() {
                        let text = &context.sources[src_id]
                            [type_span.start as usize..type_span.end as usize];
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
                other_annotation => crate::phases::binder::type_resolver::resolve_type_annotation(
                    call_span.id,
                    context
                        .sources
                        .get(call_span.id as usize)
                        .map(String::as_str)
                        .unwrap_or(""),
                    other_annotation,
                    &artifacts.symbol_table,
                ),
            }
        });
        artifacts.node_types.insert(call_span, resolved_return_type);
    }

    Expression::Block {
        span: call_span,
        statements: block_statements,
        result: result_expression,
    }
}

/// Allocates a valid source span representing `name` by appending to the synthetic inliner source buffer.
fn allocate_synthetic_identifier(context: &mut CompilationContext, name: &str) -> Span {
    let synthetic_source_index = if let Some(found_index) = context
        .source_names
        .iter()
        .position(|source_name| source_name == "<inliner_synthetic>")
    {
        found_index
    } else {
        if u16::try_from(context.sources.len()).is_err() {
            return Span::default();
        }
        context.sources.push(String::new());
        context.source_names.push("<inliner_synthetic>".to_string());
        if context.ast.len() < context.sources.len() {
            context.ast.resize_with(context.sources.len(), || None);
        }
        context.sources.len() - 1
    };

    let Ok(synthetic_source_id) = u16::try_from(synthetic_source_index) else {
        return Span::default();
    };
    let start_offset = context.sources[synthetic_source_index].len() as u32;
    context.sources[synthetic_source_index].push_str(name);
    let end_offset = context.sources[synthetic_source_index].len() as u32;
    Span::new(synthetic_source_id, start_offset, end_offset)
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

/// Substitutes identifiers matching `renamed_identifiers` with their hygienic synthetic spans
/// while preserving semantic `node_types` for renamed spans so virtual dispatch stays accurate.
fn rename_expression(
    expression: &mut Expression,
    renamed_identifiers: &HashMap<String, Span>,
    sources: &[String],
    mut node_types: Option<&mut HashMap<Span, Type>>,
) {
    match expression {
        Expression::Identifier { span } => {
            let source_index = span.id as usize;
            if source_index < sources.len() {
                let name = &sources[source_index][span.start as usize..span.end as usize];
                if let Some(&new_span) = renamed_identifiers.get(name) {
                    if let Some(types_map) = node_types.as_deref_mut()
                        && !types_map.contains_key(&new_span)
                        && let Some(resolved_type) = types_map.get(span).cloned()
                    {
                        types_map.insert(new_span, resolved_type);
                    }
                    *span = new_span;
                }
            }
        }
        Expression::Binary { left, right, .. } => {
            rename_expression(
                left,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_expression(right, renamed_identifiers, sources, node_types);
        }
        Expression::Unary { operand, .. } => {
            rename_expression(operand, renamed_identifiers, sources, node_types);
        }
        Expression::Assignment { target, value, .. } => {
            rename_expression(
                target,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_expression(value, renamed_identifiers, sources, node_types);
        }
        Expression::MemberAccess { object, .. } => {
            rename_expression(object, renamed_identifiers, sources, node_types);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            rename_expression(
                callee,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            for argument in arguments {
                rename_expression(
                    argument,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            rename_expression(
                array,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_expression(index, renamed_identifiers, sources, node_types);
        }
        Expression::ArrayCreation { size, .. } => {
            rename_expression(size, renamed_identifiers, sources, node_types);
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for element in elements {
                rename_expression(
                    element,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
        }
        Expression::NewInstance { arguments, .. } => {
            for argument in arguments {
                rename_expression(
                    argument,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
        }
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => {
            rename_expression(inner, renamed_identifiers, sources, node_types);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            rename_expression(
                condition,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_expression(
                then_branch,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_expression(else_branch, renamed_identifiers, sources, node_types);
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements {
                rename_statement(
                    statement,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
            if let Some(res) = result {
                rename_expression(res, renamed_identifiers, sources, node_types);
            }
        }
        _ => {}
    }
}

/// Substitutes identifiers inside statements with their hygienic synthetic spans
/// while preserving semantic `node_types` for renamed declarations and references.
fn rename_statement(
    statement: &mut Statement,
    renamed_identifiers: &HashMap<String, Span>,
    sources: &[String],
    mut node_types: Option<&mut HashMap<Span, Type>>,
) {
    match statement {
        Statement::Block(block) => {
            for nested_statement in &mut block.statements {
                rename_statement(
                    nested_statement,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
        }
        Statement::If(if_stmt) => {
            rename_expression(
                &mut if_stmt.condition,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_statement(
                &mut if_stmt.then_branch,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            if let Some(else_branch) = &mut if_stmt.else_branch {
                rename_statement(else_branch, renamed_identifiers, sources, node_types);
            }
        }
        Statement::While(while_stmt) => {
            rename_expression(
                &mut while_stmt.condition,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_statement(
                &mut while_stmt.body,
                renamed_identifiers,
                sources,
                node_types,
            );
        }
        Statement::DoWhile(do_while) => {
            rename_statement(
                &mut do_while.body,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            rename_expression(
                &mut do_while.condition,
                renamed_identifiers,
                sources,
                node_types,
            );
        }
        Statement::For(for_stmt) => {
            if let Some(initializer) = &mut for_stmt.initializer {
                rename_statement(
                    initializer,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
            if let Some(condition) = &mut for_stmt.condition {
                rename_expression(
                    condition,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
            if let Some(update) = &mut for_stmt.update {
                rename_expression(
                    update,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
            rename_statement(&mut for_stmt.body, renamed_identifiers, sources, node_types);
        }
        Statement::Switch(switch_stmt) => {
            rename_expression(
                &mut switch_stmt.discriminant,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            for case in &mut switch_stmt.cases {
                rename_expression(
                    &mut case.case_value,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
                for case_statement in &mut case.statements {
                    rename_statement(
                        case_statement,
                        renamed_identifiers,
                        sources,
                        node_types.as_deref_mut(),
                    );
                }
            }
            if let Some(default_statements) = &mut switch_stmt.default_case {
                for default_statement in default_statements {
                    rename_statement(
                        default_statement,
                        renamed_identifiers,
                        sources,
                        node_types.as_deref_mut(),
                    );
                }
            }
        }
        Statement::Return(return_stmt) => {
            if let Some(value) = &mut return_stmt.value {
                rename_expression(value, renamed_identifiers, sources, node_types);
            }
        }
        Statement::Throw(throw_stmt) => {
            rename_expression(
                &mut throw_stmt.expression,
                renamed_identifiers,
                sources,
                node_types,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            let source_index = var_decl.name_span.id as usize;
            if source_index < sources.len() {
                let name = &sources[source_index]
                    [var_decl.name_span.start as usize..var_decl.name_span.end as usize];
                if let Some(&new_span) = renamed_identifiers.get(name) {
                    if let Some(types_map) = node_types.as_deref_mut()
                        && !types_map.contains_key(&new_span)
                        && let Some(resolved_type) =
                            types_map.get(&var_decl.name_span).cloned().or_else(|| {
                                var_decl
                                    .initializer
                                    .as_ref()
                                    .and_then(|init| types_map.get(&init.span()).cloned())
                            })
                    {
                        types_map.insert(new_span, resolved_type);
                    }
                    var_decl.name_span = new_span;
                }
            }
            if let Some(initializer) = &mut var_decl.initializer {
                rename_expression(initializer, renamed_identifiers, sources, node_types);
            }
        }
        Statement::Expression(expr_stmt) => {
            rename_expression(
                &mut expr_stmt.expression,
                renamed_identifiers,
                sources,
                node_types,
            );
        }
        Statement::Assert(assert_stmt) => {
            rename_expression(
                &mut assert_stmt.condition,
                renamed_identifiers,
                sources,
                node_types.as_deref_mut(),
            );
            if let Some(message) = &mut assert_stmt.message {
                rename_expression(message, renamed_identifiers, sources, node_types);
            }
        }
        Statement::Exit(exit_stmt) => {
            rename_expression(
                &mut exit_stmt.code,
                renamed_identifiers,
                sources,
                node_types,
            );
        }
        Statement::Try(try_stmt) => {
            for try_statement in &mut try_stmt.try_block.statements {
                rename_statement(
                    try_statement,
                    renamed_identifiers,
                    sources,
                    node_types.as_deref_mut(),
                );
            }
            for catch_clause in &mut try_stmt.catch_clauses {
                for catch_statement in &mut catch_clause.block.statements {
                    rename_statement(
                        catch_statement,
                        renamed_identifiers,
                        sources,
                        node_types.as_deref_mut(),
                    );
                }
            }
            if let Some(finally_block) = &mut try_stmt.finally_block {
                for finally_statement in &mut finally_block.statements {
                    rename_statement(
                        finally_statement,
                        renamed_identifiers,
                        sources,
                        node_types.as_deref_mut(),
                    );
                }
            }
        }
        _ => {}
    }
}
