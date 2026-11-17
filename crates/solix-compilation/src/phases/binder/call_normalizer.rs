//! Call-site normalization pass injecting default parameter expressions and bundling variadics.

#![forbid(unsafe_code)]

use crate::phases::lexing::Span;
use crate::phases::parser::statements::declarations::{ClassMember, Declaration};
use crate::phases::parser::statements::expressions::{Expression, LambdaBody};
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::parser::statements::{BlockStatement, ProgramAst, Statement};

/// Metadata for a single formal parameter.
#[derive(Clone, Debug)]
pub struct ParameterMetadata {
    /// Optional default argument expression.
    pub default_value: Option<Expression>,
    /// Whether this is a variadic parameter pack (`...`).
    pub is_variadic: bool,
}

/// Metadata for a callable function, method, or constructor.
#[derive(Clone, Debug)]
pub struct CallableMetadata {
    /// Unqualified callable symbol name.
    pub name: String,
    /// Optional enclosing class or interface name.
    pub class_name: Option<String>,
    /// Formal parameter declarations.
    pub parameters: Vec<ParameterMetadata>,
}

/// Helper function to extract a string slice from source text given a coordinate span.
fn extract_identifier(source_text: &str, span: Span) -> &str {
    let start_offset = span.start as usize;
    let end_offset = span.end as usize;
    if end_offset <= source_text.len() && start_offset <= end_offset {
        &source_text[start_offset..end_offset]
    } else {
        ""
    }
}

/// Collects callable metadata for all declared functions, methods, and constructors.
fn collect_callable_catalog(
    ast_forest: &[Option<ProgramAst>],
    sources: &[String],
) -> Vec<CallableMetadata> {
    let mut catalog = Vec::new();

    for (source_index, program_option) in ast_forest.iter().enumerate() {
        let Some(program_ast) = program_option else {
            continue;
        };
        let source_text = sources
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");

        for declaration in &program_ast.declarations {
            match declaration {
                Declaration::Method(method_declaration) => {
                    let method_name = extract_identifier(source_text, method_declaration.name_span);
                    let parameters = method_declaration
                        .parameters
                        .iter()
                        .map(|parameter| ParameterMetadata {
                            default_value: parameter.default_value.clone(),
                            is_variadic: parameter.is_variadic,
                        })
                        .collect();
                    catalog.push(CallableMetadata {
                        name: method_name.to_string(),
                        class_name: None,
                        parameters,
                    });
                }
                Declaration::Class(class_declaration) => {
                    let class_name = extract_identifier(source_text, class_declaration.name_span);
                    for member in &class_declaration.members {
                        match member {
                            ClassMember::Method(method_declaration) => {
                                let method_name =
                                    extract_identifier(source_text, method_declaration.name_span);
                                let parameters = method_declaration
                                    .parameters
                                    .iter()
                                    .map(|parameter| ParameterMetadata {
                                        default_value: parameter.default_value.clone(),
                                        is_variadic: parameter.is_variadic,
                                    })
                                    .collect();
                                catalog.push(CallableMetadata {
                                    name: method_name.to_string(),
                                    class_name: Some(class_name.to_string()),
                                    parameters,
                                });
                            }
                            ClassMember::Constructor(constructor_declaration) => {
                                let parameters: Vec<ParameterMetadata> = constructor_declaration
                                    .parameters
                                    .iter()
                                    .map(|parameter| ParameterMetadata {
                                        default_value: parameter.default_value.clone(),
                                        is_variadic: parameter.is_variadic,
                                    })
                                    .collect();
                                catalog.push(CallableMetadata {
                                    name: "constructor".to_string(),
                                    class_name: Some(class_name.to_string()),
                                    parameters: parameters.clone(),
                                });
                                catalog.push(CallableMetadata {
                                    name: class_name.to_string(),
                                    class_name: Some(class_name.to_string()),
                                    parameters,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                Declaration::Interface(interface_declaration) => {
                    let interface_name =
                        extract_identifier(source_text, interface_declaration.name_span);
                    for method_declaration in &interface_declaration.methods {
                        let method_name =
                            extract_identifier(source_text, method_declaration.name_span);
                        let parameters = method_declaration
                            .parameters
                            .iter()
                            .map(|parameter| ParameterMetadata {
                                default_value: parameter.default_value.clone(),
                                is_variadic: parameter.is_variadic,
                            })
                            .collect();
                        catalog.push(CallableMetadata {
                            name: method_name.to_string(),
                            class_name: Some(interface_name.to_string()),
                            parameters,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    catalog
}

/// Finds the most compatible callable metadata entry matching the target invocation.
fn find_matching_callable<'catalog>(
    catalog: &'catalog [CallableMetadata],
    target_class: Option<&str>,
    target_name: &str,
    argument_count: usize,
) -> Option<&'catalog CallableMetadata> {
    if target_name.is_empty() {
        return None;
    }

    // Step 1: Filter candidates by name and optional class
    let mut candidates: Vec<&'catalog CallableMetadata> = catalog
        .iter()
        .filter(|candidate| {
            if candidate.name != target_name {
                return false;
            }
            if let Some(target_class_name) = target_class {
                if let Some(ref candidate_class_name) = candidate.class_name {
                    candidate_class_name == target_class_name
                } else {
                    true
                }
            } else {
                true
            }
        })
        .collect();

    if candidates.is_empty() && target_class.is_some() {
        // Fallback: match by name regardless of class
        candidates = catalog
            .iter()
            .filter(|candidate| candidate.name == target_name)
            .collect();
    }

    // Step 2: Select candidate compatible with argument_count
    for candidate in &candidates {
        let declared_count = candidate.parameters.len();
        let is_variadic = candidate
            .parameters
            .last()
            .map(|parameter| parameter.is_variadic)
            .unwrap_or(false);

        if is_variadic {
            let fixed_count = declared_count.saturating_sub(1);
            if argument_count >= fixed_count {
                return Some(candidate);
            }
        } else {
            let default_count = candidate
                .parameters
                .iter()
                .rev()
                .take_while(|parameter| parameter.default_value.is_some())
                .count();
            let min_required = declared_count.saturating_sub(default_count);
            if argument_count >= min_required && argument_count <= declared_count {
                return Some(candidate);
            }
        }
    }

    None
}

/// Normalizes call arguments by appending default argument expressions or bundling variadics.
fn normalize_arguments_list(
    arguments: &mut Vec<Expression>,
    candidate: &CallableMetadata,
    call_span: Span,
) {
    let declared_count = candidate.parameters.len();
    let is_variadic = candidate
        .parameters
        .last()
        .map(|parameter| parameter.is_variadic)
        .unwrap_or(false);

    if is_variadic {
        let fixed_count = declared_count.saturating_sub(1);
        if arguments.len() >= fixed_count {
            let trailing_arguments: Vec<Expression> = arguments.drain(fixed_count..).collect();
            let bundle_span = if let (Some(first_argument), Some(last_argument)) =
                (trailing_arguments.first(), trailing_arguments.last())
            {
                first_argument.span().combine(last_argument.span())
            } else {
                call_span
            };
            arguments.push(Expression::TupleCreation {
                span: bundle_span,
                elements: trailing_arguments,
            });
        }
    } else if arguments.len() < declared_count {
        let start_index = arguments.len();
        for parameter in candidate
            .parameters
            .iter()
            .take(declared_count)
            .skip(start_index)
        {
            if let Some(ref default_expression) = parameter.default_value {
                arguments.push(default_expression.clone());
            }
        }
    }
}

/// Normalizes all expressions recursively in an AST expression node.
fn normalize_expression(
    expression: &mut Expression,
    catalog: &[CallableMetadata],
    source_text: &str,
    enclosing_class: Option<&str>,
) {
    match expression {
        Expression::Binary { left, right, .. } => {
            normalize_expression(left, catalog, source_text, enclosing_class);
            normalize_expression(right, catalog, source_text, enclosing_class);
        }
        Expression::Unary { operand, .. } => {
            normalize_expression(operand, catalog, source_text, enclosing_class);
        }
        Expression::Assignment { target, value, .. } => {
            normalize_expression(target, catalog, source_text, enclosing_class);
            normalize_expression(value, catalog, source_text, enclosing_class);
        }
        Expression::MemberAccess { object, .. } => {
            normalize_expression(object, catalog, source_text, enclosing_class);
        }
        Expression::MethodCall {
            span,
            callee,
            arguments,
            ..
        } => {
            normalize_expression(callee, catalog, source_text, enclosing_class);
            for argument in arguments.iter_mut() {
                normalize_expression(argument, catalog, source_text, enclosing_class);
            }

            let (target_class, target_name) = match &**callee {
                Expression::Identifier {
                    span: identifier_span,
                } => {
                    let identifier_name = extract_identifier(source_text, *identifier_span);
                    (enclosing_class, identifier_name)
                }
                Expression::MemberAccess {
                    object,
                    member_span,
                    ..
                } => {
                    let member_name = extract_identifier(source_text, *member_span);
                    let object_name = match &**object {
                        Expression::Identifier { span: object_span } => {
                            let extracted = extract_identifier(source_text, *object_span);
                            if extracted == "this" {
                                enclosing_class
                            } else {
                                Some(extracted)
                            }
                        }
                        _ => None,
                    };
                    (object_name, member_name)
                }
                _ => (None, ""),
            };

            if let Some(candidate) =
                find_matching_callable(catalog, target_class, target_name, arguments.len())
            {
                normalize_arguments_list(arguments, candidate, *span);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            normalize_expression(array, catalog, source_text, enclosing_class);
            normalize_expression(index, catalog, source_text, enclosing_class);
        }
        Expression::ArrayCreation { size, .. } => {
            normalize_expression(size, catalog, source_text, enclosing_class);
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for element in elements.iter_mut() {
                normalize_expression(element, catalog, source_text, enclosing_class);
            }
        }
        Expression::NewInstance {
            span,
            instance_type,
            arguments,
        } => {
            for argument in arguments.iter_mut() {
                normalize_expression(argument, catalog, source_text, enclosing_class);
            }

            let class_name = match instance_type {
                TypeAnnotation::Named { path, .. } => path
                    .last()
                    .map(|segment| extract_identifier(source_text, *segment))
                    .unwrap_or(""),
                _ => "",
            };

            if let Some(candidate) =
                find_matching_callable(catalog, Some(class_name), class_name, arguments.len())
            {
                normalize_arguments_list(arguments, candidate, *span);
            }
        }
        Expression::Cast {
            expression: inner_expression,
            ..
        }
        | Expression::Instanceof {
            expression: inner_expression,
            ..
        } => {
            normalize_expression(inner_expression, catalog, source_text, enclosing_class);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            normalize_expression(condition, catalog, source_text, enclosing_class);
            normalize_expression(then_branch, catalog, source_text, enclosing_class);
            normalize_expression(else_branch, catalog, source_text, enclosing_class);
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            for parameter in parameters.iter_mut() {
                if let Some(ref mut default_value) = parameter.default_value {
                    normalize_expression(default_value, catalog, source_text, enclosing_class);
                }
            }
            match body {
                LambdaBody::Expression(lambda_expression) => {
                    normalize_expression(lambda_expression, catalog, source_text, enclosing_class);
                }
                LambdaBody::Block(lambda_block) => {
                    normalize_block(lambda_block, catalog, source_text, enclosing_class);
                }
            }
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements.iter_mut() {
                normalize_statement(statement, catalog, source_text, enclosing_class);
            }
            if let Some(result_expression) = result {
                normalize_expression(result_expression, catalog, source_text, enclosing_class);
            }
        }
        Expression::Literal { .. }
        | Expression::Identifier { .. }
        | Expression::Sizeof { .. }
        | Expression::Default { .. } => {}
    }
}

/// Normalizes all statements in a block statement.
fn normalize_block(
    block: &mut BlockStatement,
    catalog: &[CallableMetadata],
    source_text: &str,
    enclosing_class: Option<&str>,
) {
    for statement in &mut block.statements {
        normalize_statement(statement, catalog, source_text, enclosing_class);
    }
}

/// Normalizes an individual statement.
fn normalize_statement(
    statement: &mut Statement,
    catalog: &[CallableMetadata],
    source_text: &str,
    enclosing_class: Option<&str>,
) {
    match statement {
        Statement::Block(block_statement) => {
            normalize_block(block_statement, catalog, source_text, enclosing_class);
        }
        Statement::If(if_statement) => {
            normalize_expression(
                &mut if_statement.condition,
                catalog,
                source_text,
                enclosing_class,
            );
            normalize_statement(
                &mut if_statement.then_branch,
                catalog,
                source_text,
                enclosing_class,
            );
            if let Some(ref mut else_branch) = if_statement.else_branch {
                normalize_statement(else_branch, catalog, source_text, enclosing_class);
            }
        }
        Statement::While(while_statement) => {
            normalize_expression(
                &mut while_statement.condition,
                catalog,
                source_text,
                enclosing_class,
            );
            normalize_statement(
                &mut while_statement.body,
                catalog,
                source_text,
                enclosing_class,
            );
        }
        Statement::DoWhile(do_while_statement) => {
            normalize_statement(
                &mut do_while_statement.body,
                catalog,
                source_text,
                enclosing_class,
            );
            normalize_expression(
                &mut do_while_statement.condition,
                catalog,
                source_text,
                enclosing_class,
            );
        }
        Statement::For(for_statement) => {
            if let Some(ref mut initializer) = for_statement.initializer {
                normalize_statement(initializer, catalog, source_text, enclosing_class);
            }
            if let Some(ref mut condition) = for_statement.condition {
                normalize_expression(condition, catalog, source_text, enclosing_class);
            }
            if let Some(ref mut update) = for_statement.update {
                normalize_expression(update, catalog, source_text, enclosing_class);
            }
            normalize_statement(
                &mut for_statement.body,
                catalog,
                source_text,
                enclosing_class,
            );
        }
        Statement::Switch(switch_statement) => {
            normalize_expression(
                &mut switch_statement.discriminant,
                catalog,
                source_text,
                enclosing_class,
            );
            for switch_case in &mut switch_statement.cases {
                normalize_expression(
                    &mut switch_case.case_value,
                    catalog,
                    source_text,
                    enclosing_class,
                );
                for case_statement in &mut switch_case.statements {
                    normalize_statement(case_statement, catalog, source_text, enclosing_class);
                }
            }
        }
        Statement::Try(try_statement) => {
            normalize_block(
                &mut try_statement.try_block,
                catalog,
                source_text,
                enclosing_class,
            );
            for catch_clause in &mut try_statement.catch_clauses {
                normalize_block(
                    &mut catch_clause.block,
                    catalog,
                    source_text,
                    enclosing_class,
                );
            }
            if let Some(ref mut finally_block) = try_statement.finally_block {
                normalize_block(finally_block, catalog, source_text, enclosing_class);
            }
        }
        Statement::Throw(throw_statement) => {
            normalize_expression(
                &mut throw_statement.expression,
                catalog,
                source_text,
                enclosing_class,
            );
        }
        Statement::Return(return_statement) => {
            if let Some(ref mut value) = return_statement.value {
                normalize_expression(value, catalog, source_text, enclosing_class);
            }
        }
        Statement::VariableDeclaration(variable_statement) => {
            if let Some(ref mut initializer) = variable_statement.initializer {
                normalize_expression(initializer, catalog, source_text, enclosing_class);
            }
        }
        Statement::Expression(expression_statement) => {
            normalize_expression(
                &mut expression_statement.expression,
                catalog,
                source_text,
                enclosing_class,
            );
        }
        Statement::Assert(assert_statement) => {
            normalize_expression(
                &mut assert_statement.condition,
                catalog,
                source_text,
                enclosing_class,
            );
            if let Some(ref mut message) = assert_statement.message {
                normalize_expression(message, catalog, source_text, enclosing_class);
            }
        }
        Statement::Exit(exit_statement) => {
            normalize_expression(
                &mut exit_statement.code,
                catalog,
                source_text,
                enclosing_class,
            );
        }
        Statement::Declaration(declaration) => match declaration {
            Declaration::Method(method_declaration) => {
                for parameter in &mut method_declaration.parameters {
                    if let Some(ref mut default_value) = parameter.default_value {
                        normalize_expression(default_value, catalog, source_text, enclosing_class);
                    }
                }
                if let Some(ref mut body) = method_declaration.body {
                    normalize_block(body, catalog, source_text, enclosing_class);
                }
            }
            Declaration::Class(class_declaration) => {
                let current_class =
                    extract_identifier(source_text, class_declaration.name_span).to_string();
                for member in &mut class_declaration.members {
                    match member {
                        ClassMember::Method(method_declaration) => {
                            for parameter in &mut method_declaration.parameters {
                                if let Some(ref mut default_value) = parameter.default_value {
                                    normalize_expression(
                                        default_value,
                                        catalog,
                                        source_text,
                                        Some(&current_class),
                                    );
                                }
                            }
                            if let Some(ref mut body) = method_declaration.body {
                                normalize_block(body, catalog, source_text, Some(&current_class));
                            }
                        }
                        ClassMember::Constructor(constructor_declaration) => {
                            for parameter in &mut constructor_declaration.parameters {
                                if let Some(ref mut default_value) = parameter.default_value {
                                    normalize_expression(
                                        default_value,
                                        catalog,
                                        source_text,
                                        Some(&current_class),
                                    );
                                }
                            }
                            if let Some(ref mut super_arguments) =
                                constructor_declaration.super_call_args
                            {
                                for argument in super_arguments.iter_mut() {
                                    normalize_expression(
                                        argument,
                                        catalog,
                                        source_text,
                                        Some(&current_class),
                                    );
                                }
                            }
                            normalize_block(
                                &mut constructor_declaration.body,
                                catalog,
                                source_text,
                                Some(&current_class),
                            );
                        }
                        ClassMember::Destructor(destructor_declaration) => {
                            normalize_block(
                                &mut destructor_declaration.body,
                                catalog,
                                source_text,
                                Some(&current_class),
                            );
                        }
                        ClassMember::Field(field_declaration) => {
                            if let Some(ref mut initializer) = field_declaration.initializer {
                                normalize_expression(
                                    initializer,
                                    catalog,
                                    source_text,
                                    Some(&current_class),
                                );
                            }
                        }
                        ClassMember::Operator(operator_declaration) => {
                            for parameter in &mut operator_declaration.parameters {
                                if let Some(ref mut default_value) = parameter.default_value {
                                    normalize_expression(
                                        default_value,
                                        catalog,
                                        source_text,
                                        Some(&current_class),
                                    );
                                }
                            }
                            normalize_block(
                                &mut operator_declaration.body,
                                catalog,
                                source_text,
                                Some(&current_class),
                            );
                        }
                        _ => {}
                    }
                }
            }
            Declaration::Field(field_declaration) => {
                if let Some(ref mut initializer) = field_declaration.initializer {
                    normalize_expression(initializer, catalog, source_text, enclosing_class);
                }
            }
            Declaration::Constructor(constructor_declaration) => {
                for parameter in &mut constructor_declaration.parameters {
                    if let Some(ref mut default_value) = parameter.default_value {
                        normalize_expression(default_value, catalog, source_text, enclosing_class);
                    }
                }
                if let Some(ref mut super_arguments) = constructor_declaration.super_call_args {
                    for argument in super_arguments.iter_mut() {
                        normalize_expression(argument, catalog, source_text, enclosing_class);
                    }
                }
                normalize_block(
                    &mut constructor_declaration.body,
                    catalog,
                    source_text,
                    enclosing_class,
                );
            }
            Declaration::Destructor(destructor_declaration) => {
                normalize_block(
                    &mut destructor_declaration.body,
                    catalog,
                    source_text,
                    enclosing_class,
                );
            }
            Declaration::Operator(operator_declaration) => {
                for parameter in &mut operator_declaration.parameters {
                    if let Some(ref mut default_value) = parameter.default_value {
                        normalize_expression(default_value, catalog, source_text, enclosing_class);
                    }
                }
                normalize_block(
                    &mut operator_declaration.body,
                    catalog,
                    source_text,
                    enclosing_class,
                );
            }
            Declaration::Enum(enum_declaration) => {
                for variant in &mut enum_declaration.variants {
                    if let Some(ref mut explicit_value) = variant.explicit_value {
                        normalize_expression(explicit_value, catalog, source_text, enclosing_class);
                    }
                }
            }
            Declaration::Interface(_) => {}
        },
        Statement::Break(_) | Statement::Continue(_) => {}
    }
}

/// Normalizes all call sites across all parsed program ASTs in the compilation context.
pub fn normalize_call_sites(ast_forest: &mut [Option<ProgramAst>], sources: &[String]) {
    let catalog = collect_callable_catalog(ast_forest, sources);

    for (source_index, program_option) in ast_forest.iter_mut().enumerate() {
        let Some(program_ast) = program_option else {
            continue;
        };
        let source_text = sources
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");

        for declaration in &mut program_ast.declarations {
            match declaration {
                Declaration::Method(method_declaration) => {
                    for parameter in &mut method_declaration.parameters {
                        if let Some(ref mut default_value) = parameter.default_value {
                            normalize_expression(default_value, &catalog, source_text, None);
                        }
                    }
                    if let Some(ref mut body) = method_declaration.body {
                        normalize_block(body, &catalog, source_text, None);
                    }
                }
                Declaration::Class(class_declaration) => {
                    let current_class =
                        extract_identifier(source_text, class_declaration.name_span).to_string();
                    for member in &mut class_declaration.members {
                        match member {
                            ClassMember::Method(method_declaration) => {
                                for parameter in &mut method_declaration.parameters {
                                    if let Some(ref mut default_value) = parameter.default_value {
                                        normalize_expression(
                                            default_value,
                                            &catalog,
                                            source_text,
                                            Some(&current_class),
                                        );
                                    }
                                }
                                if let Some(ref mut body) = method_declaration.body {
                                    normalize_block(
                                        body,
                                        &catalog,
                                        source_text,
                                        Some(&current_class),
                                    );
                                }
                            }
                            ClassMember::Constructor(constructor_declaration) => {
                                for parameter in &mut constructor_declaration.parameters {
                                    if let Some(ref mut default_value) = parameter.default_value {
                                        normalize_expression(
                                            default_value,
                                            &catalog,
                                            source_text,
                                            Some(&current_class),
                                        );
                                    }
                                }
                                if let Some(ref mut super_arguments) =
                                    constructor_declaration.super_call_args
                                {
                                    for argument in super_arguments.iter_mut() {
                                        normalize_expression(
                                            argument,
                                            &catalog,
                                            source_text,
                                            Some(&current_class),
                                        );
                                    }
                                }
                                normalize_block(
                                    &mut constructor_declaration.body,
                                    &catalog,
                                    source_text,
                                    Some(&current_class),
                                );
                            }
                            ClassMember::Destructor(destructor_declaration) => {
                                normalize_block(
                                    &mut destructor_declaration.body,
                                    &catalog,
                                    source_text,
                                    Some(&current_class),
                                );
                            }
                            ClassMember::Field(field_declaration) => {
                                if let Some(ref mut initializer) = field_declaration.initializer {
                                    normalize_expression(
                                        initializer,
                                        &catalog,
                                        source_text,
                                        Some(&current_class),
                                    );
                                }
                            }
                            ClassMember::Operator(operator_declaration) => {
                                for parameter in &mut operator_declaration.parameters {
                                    if let Some(ref mut default_value) = parameter.default_value {
                                        normalize_expression(
                                            default_value,
                                            &catalog,
                                            source_text,
                                            Some(&current_class),
                                        );
                                    }
                                }
                                normalize_block(
                                    &mut operator_declaration.body,
                                    &catalog,
                                    source_text,
                                    Some(&current_class),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                Declaration::Field(field_declaration) => {
                    if let Some(ref mut initializer) = field_declaration.initializer {
                        normalize_expression(initializer, &catalog, source_text, None);
                    }
                }
                Declaration::Constructor(constructor_declaration) => {
                    for parameter in &mut constructor_declaration.parameters {
                        if let Some(ref mut default_value) = parameter.default_value {
                            normalize_expression(default_value, &catalog, source_text, None);
                        }
                    }
                    if let Some(ref mut super_arguments) = constructor_declaration.super_call_args {
                        for argument in super_arguments.iter_mut() {
                            normalize_expression(argument, &catalog, source_text, None);
                        }
                    }
                    normalize_block(
                        &mut constructor_declaration.body,
                        &catalog,
                        source_text,
                        None,
                    );
                }
                Declaration::Destructor(destructor_declaration) => {
                    normalize_block(
                        &mut destructor_declaration.body,
                        &catalog,
                        source_text,
                        None,
                    );
                }
                Declaration::Operator(operator_declaration) => {
                    for parameter in &mut operator_declaration.parameters {
                        if let Some(ref mut default_value) = parameter.default_value {
                            normalize_expression(default_value, &catalog, source_text, None);
                        }
                    }
                    normalize_block(&mut operator_declaration.body, &catalog, source_text, None);
                }
                Declaration::Enum(enum_declaration) => {
                    for variant in &mut enum_declaration.variants {
                        if let Some(ref mut explicit_value) = variant.explicit_value {
                            normalize_expression(explicit_value, &catalog, source_text, None);
                        }
                    }
                }
                Declaration::Interface(_) => {}
            }
        }
    }
}
