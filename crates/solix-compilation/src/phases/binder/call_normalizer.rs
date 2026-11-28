//! Call-site normalization pass injecting default parameter expressions and bundling variadics.

#![forbid(unsafe_code)]

use crate::phases::binder::symbols::SymbolTable;
use crate::phases::lexing::Span;
use crate::phases::parser::statements::declarations::{
    ClassMember, Declaration, MethodDeclaration,
};
use crate::phases::parser::statements::expressions::{Expression, LambdaBody};
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::parser::statements::{BlockStatement, ProgramAst, Statement};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

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
    /// Optional fully qualified enclosing class or interface name.
    pub qualified_class_name: Option<String>,
    /// Optional class/interface return type name when returning an object reference.
    pub return_class_name: Option<String>,
    /// Formal parameter declarations.
    pub parameters: Vec<ParameterMetadata>,
}

#[derive(Default)]
struct CallableCatalog {
    callables: Vec<CallableMetadata>,
    known_classes: HashSet<String>,
    class_parents: HashMap<String, Vec<String>>,
    field_types: HashMap<(String, String), String>,
}

struct NormalizerScopes {
    frames: Vec<HashMap<String, Option<String>>>,
}

impl NormalizerScopes {
    fn new() -> Self {
        Self {
            frames: vec![HashMap::default()],
        }
    }

    fn push(&mut self) {
        self.frames.push(HashMap::default());
    }

    fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    fn insert(&mut self, name: String, class_name: Option<String>) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, class_name);
        }
    }

    fn lookup(&self, name: &str) -> Option<&Option<String>> {
        for frame in self.frames.iter().rev() {
            if let Some(found) = frame.get(name) {
                return Some(found);
            }
        }
        None
    }
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

fn extract_type_name(annotation: &TypeAnnotation, source_text: &str) -> Option<String> {
    match annotation {
        TypeAnnotation::Named { path, .. } => {
            let segments: Vec<&str> = path
                .iter()
                .map(|span| extract_identifier(source_text, *span))
                .filter(|seg| !seg.is_empty())
                .collect();
            if segments.is_empty() {
                None
            } else {
                Some(segments.join("::"))
            }
        }
        _ => None,
    }
}

fn extract_parameters(method: &MethodDeclaration) -> Vec<ParameterMetadata> {
    method
        .parameters
        .iter()
        .map(|parameter| ParameterMetadata {
            default_value: parameter.default_value.clone(),
            is_variadic: parameter.is_variadic,
        })
        .collect()
}

fn simple_type_name(name: &str) -> &str {
    let without_generics = name.split('<').next().unwrap_or(name);
    without_generics
        .rsplit(['.', ':'])
        .next()
        .unwrap_or(without_generics)
}

fn register_type_name(known_classes: &mut HashSet<String>, qualified: &str, simple: &str) {
    let normalized = qualified.replace('.', "::");
    if !normalized.is_empty() {
        known_classes.insert(normalized);
    }
    if !simple.is_empty() {
        known_classes.insert(simple.to_string());
    }
}

/// Collects callable metadata and class hierarchy/field types across all parsed ASTs.
fn collect_callable_catalog(
    ast_forest: &[Option<ProgramAst>],
    sources: &[String],
    symbol_table: Option<&SymbolTable>,
) -> CallableCatalog {
    let mut catalog = CallableCatalog::default();

    if let Some(table) = symbol_table {
        for symbol in &table.symbols {
            if matches!(
                symbol.symbol_type,
                solix_common::Type::Class { .. } | solix_common::Type::Interface { .. }
            ) {
                register_type_name(
                    &mut catalog.known_classes,
                    &symbol.fully_qualified_name,
                    &symbol.name,
                );
                if let Some(parent_id) = symbol.parent_symbol
                    && let Some(parent_sym) = table.get_symbol(parent_id)
                {
                    let parents = catalog
                        .class_parents
                        .entry(symbol.fully_qualified_name.replace('.', "::"))
                        .or_default();
                    parents.push(parent_sym.fully_qualified_name.replace('.', "::"));
                    let simple_parents = catalog
                        .class_parents
                        .entry(symbol.name.clone())
                        .or_default();
                    simple_parents.push(parent_sym.name.clone());
                }
            }
        }
    }

    for (source_index, program_option) in ast_forest.iter().enumerate() {
        let Some(program_ast) = program_option else {
            continue;
        };
        let source_text = sources
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");
        let package_prefix = program_ast
            .package
            .as_ref()
            .map(|pkg| {
                pkg.path
                    .iter()
                    .map(|span| extract_identifier(source_text, *span))
                    .filter(|seg| !seg.is_empty())
                    .collect::<Vec<_>>()
                    .join("::")
            })
            .unwrap_or_default();

        for declaration in &program_ast.declarations {
            match declaration {
                Declaration::Method(method_declaration) => {
                    let method_name = extract_identifier(source_text, method_declaration.name_span);
                    catalog.callables.push(CallableMetadata {
                        name: method_name.to_string(),
                        class_name: None,
                        qualified_class_name: None,
                        return_class_name: extract_type_name(
                            &method_declaration.return_type,
                            source_text,
                        ),
                        parameters: extract_parameters(method_declaration),
                    });
                }
                Declaration::Class(class_declaration) => {
                    let class_name = extract_identifier(source_text, class_declaration.name_span);
                    let qualified_class = if package_prefix.is_empty() {
                        class_name.to_string()
                    } else {
                        format!("{package_prefix}::{class_name}")
                    };
                    register_type_name(&mut catalog.known_classes, &qualified_class, class_name);

                    let mut parent_names = Vec::new();
                    if let Some(ref super_type) = class_declaration.super_class
                        && let Some(super_name) = extract_type_name(super_type, source_text)
                    {
                        parent_names.push(super_name);
                    }
                    for iface_type in &class_declaration.interfaces {
                        if let Some(iface_name) = extract_type_name(iface_type, source_text) {
                            parent_names.push(iface_name);
                        }
                    }
                    if !parent_names.is_empty() {
                        catalog
                            .class_parents
                            .entry(qualified_class.clone())
                            .or_default()
                            .extend(parent_names.iter().cloned());
                        catalog
                            .class_parents
                            .entry(class_name.to_string())
                            .or_default()
                            .extend(parent_names);
                    }

                    for member in &class_declaration.members {
                        match member {
                            ClassMember::Field(field_decl) => {
                                let field_name =
                                    extract_identifier(source_text, field_decl.name_span);
                                if let Some(field_cls) =
                                    extract_type_name(&field_decl.field_type, source_text)
                                {
                                    catalog.field_types.insert(
                                        (class_name.to_string(), field_name.to_string()),
                                        field_cls.clone(),
                                    );
                                    catalog.field_types.insert(
                                        (qualified_class.clone(), field_name.to_string()),
                                        field_cls,
                                    );
                                }
                            }
                            ClassMember::Method(method_declaration) => {
                                let method_name =
                                    extract_identifier(source_text, method_declaration.name_span);
                                catalog.callables.push(CallableMetadata {
                                    name: method_name.to_string(),
                                    class_name: Some(class_name.to_string()),
                                    qualified_class_name: Some(qualified_class.clone()),
                                    return_class_name: extract_type_name(
                                        &method_declaration.return_type,
                                        source_text,
                                    ),
                                    parameters: extract_parameters(method_declaration),
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
                                catalog.callables.push(CallableMetadata {
                                    name: "constructor".to_string(),
                                    class_name: Some(class_name.to_string()),
                                    qualified_class_name: Some(qualified_class.clone()),
                                    return_class_name: Some(qualified_class.clone()),
                                    parameters: parameters.clone(),
                                });
                                catalog.callables.push(CallableMetadata {
                                    name: class_name.to_string(),
                                    class_name: Some(class_name.to_string()),
                                    qualified_class_name: Some(qualified_class.clone()),
                                    return_class_name: Some(qualified_class.clone()),
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
                    let qualified_interface = if package_prefix.is_empty() {
                        interface_name.to_string()
                    } else {
                        format!("{package_prefix}::{interface_name}")
                    };
                    register_type_name(
                        &mut catalog.known_classes,
                        &qualified_interface,
                        interface_name,
                    );
                    for method_declaration in &interface_declaration.methods {
                        let method_name =
                            extract_identifier(source_text, method_declaration.name_span);
                        catalog.callables.push(CallableMetadata {
                            name: method_name.to_string(),
                            class_name: Some(interface_name.to_string()),
                            qualified_class_name: Some(qualified_interface.clone()),
                            return_class_name: extract_type_name(
                                &method_declaration.return_type,
                                source_text,
                            ),
                            parameters: extract_parameters(method_declaration),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    catalog
}

fn direct_class_matches(candidate: &CallableMetadata, target_class: &str) -> bool {
    let target_norm = target_class.replace('.', "::");
    let target_simple = simple_type_name(&target_norm);

    if let Some(ref qualified_cls) = candidate.qualified_class_name {
        let cand_norm = qualified_cls.replace('.', "::");
        if cand_norm == target_norm {
            return true;
        }
        // Only allow simple name comparison when caller didn't qualify a namespace path.
        if !target_norm.contains("::") && simple_type_name(&cand_norm) == target_simple {
            return true;
        }
    }

    if let Some(ref cand_cls) = candidate.class_name {
        let cand_norm = cand_cls.replace('.', "::");
        if cand_norm == target_norm {
            return true;
        }
        if !target_norm.contains("::") && simple_type_name(&cand_norm) == target_simple {
            return true;
        }
        if target_norm.contains("::")
            && cand_norm == target_simple
            && candidate
                .qualified_class_name
                .as_deref()
                .is_some_and(|qualified| qualified.replace('.', "::") == target_norm)
        {
            return true;
        }
    }

    false
}

fn class_or_ancestor_matches(
    catalog: &CallableCatalog,
    candidate: &CallableMetadata,
    target_class: &str,
) -> bool {
    if direct_class_matches(candidate, target_class) {
        return true;
    }

    let mut visited = HashSet::default();
    let mut worklist = vec![target_class.replace('.', "::")];

    while let Some(current) = worklist.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        let simple = simple_type_name(&current).to_string();
        for lookup_key in [&current, &simple] {
            if let Some(parents) = catalog.class_parents.get(lookup_key) {
                for parent in parents {
                    if direct_class_matches(candidate, parent) {
                        return true;
                    }
                    let parent_norm = parent.replace('.', "::");
                    if !visited.contains(&parent_norm) {
                        worklist.push(parent_norm);
                    }
                }
            }
        }
    }

    false
}

fn is_candidate_compatible_with_arity(candidate: &CallableMetadata, argument_count: usize) -> bool {
    let declared_count = candidate.parameters.len();
    let is_variadic = candidate
        .parameters
        .last()
        .map(|parameter| parameter.is_variadic)
        .unwrap_or(false);

    if is_variadic {
        let fixed_count = declared_count.saturating_sub(1);
        argument_count >= fixed_count
    } else {
        let default_count = candidate
            .parameters
            .iter()
            .rev()
            .take_while(|parameter| parameter.default_value.is_some())
            .count();
        let min_required = declared_count.saturating_sub(default_count);
        argument_count >= min_required && argument_count <= declared_count
    }
}

/// Finds the most compatible callable metadata entry matching the target class and name.
///
/// Never matches free functions when `target_class` is `Some(...)`, and never falls back
/// across unrelated classes.
fn find_matching_callable<'catalog>(
    catalog: &'catalog CallableCatalog,
    target_class: Option<&str>,
    target_name: &str,
    argument_count: usize,
) -> Option<&'catalog CallableMetadata> {
    if target_name.is_empty() {
        return None;
    }

    let mut exact_matches = Vec::new();
    let mut ancestor_matches = Vec::new();

    for candidate in &catalog.callables {
        if candidate.name != target_name {
            continue;
        }
        if !is_candidate_compatible_with_arity(candidate, argument_count) {
            continue;
        }

        match target_class {
            Some(target_class_name) => {
                if candidate.class_name.is_none() {
                    continue;
                }
                if direct_class_matches(candidate, target_class_name) {
                    exact_matches.push(candidate);
                } else if class_or_ancestor_matches(catalog, candidate, target_class_name) {
                    ancestor_matches.push(candidate);
                }
            }
            None => {
                if candidate.class_name.is_none() {
                    exact_matches.push(candidate);
                }
            }
        }
    }

    exact_matches
        .into_iter()
        .next()
        .or_else(|| ancestor_matches.into_iter().next())
}

fn lookup_field_type_in_hierarchy(
    catalog: &CallableCatalog,
    class_name: &str,
    field_name: &str,
) -> Option<String> {
    let mut visited = HashSet::default();
    let mut worklist = vec![class_name.replace('.', "::")];

    while let Some(current) = worklist.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        let simple = simple_type_name(&current).to_string();
        for lookup_class in [&current, &simple] {
            if let Some(found) = catalog
                .field_types
                .get(&(lookup_class.clone(), field_name.to_string()))
            {
                return Some(found.clone());
            }
            if let Some(parents) = catalog.class_parents.get(lookup_class) {
                for parent in parents {
                    let parent_norm = parent.replace('.', "::");
                    if !visited.contains(&parent_norm) {
                        worklist.push(parent_norm);
                    }
                }
            }
        }
    }

    None
}

fn extract_qualified_path(expression: &Expression, source_text: &str) -> Option<String> {
    match expression {
        Expression::Identifier { span } => {
            let ident = extract_identifier(source_text, *span);
            if ident.is_empty() {
                None
            } else {
                Some(ident.to_string())
            }
        }
        Expression::MemberAccess {
            object,
            member_span,
            ..
        } => {
            let prefix = extract_qualified_path(object, source_text)?;
            let member = extract_identifier(source_text, *member_span);
            if member.is_empty() {
                None
            } else {
                Some(format!("{prefix}::{member}"))
            }
        }
        _ => None,
    }
}

fn infer_receiver_class(
    expression: &Expression,
    catalog: &CallableCatalog,
    source_text: &str,
    enclosing_class: Option<&str>,
    scopes: &NormalizerScopes,
) -> Option<String> {
    match expression {
        Expression::Identifier { span } => {
            let ident = extract_identifier(source_text, *span);
            if ident == "this" {
                return enclosing_class.map(str::to_string);
            }
            if let Some(local_entry) = scopes.lookup(ident) {
                return local_entry.clone();
            }
            if let Some(cls) = enclosing_class
                && let Some(field_cls) = lookup_field_type_in_hierarchy(catalog, cls, ident)
            {
                return Some(field_cls);
            }
            if catalog.known_classes.contains(ident) {
                return Some(ident.to_string());
            }
            None
        }
        Expression::NewInstance { instance_type, .. } => {
            extract_type_name(instance_type, source_text)
        }
        Expression::Cast { target_type, .. } => extract_type_name(target_type, source_text),
        Expression::MemberAccess {
            object,
            member_span,
            ..
        } => {
            if let Some(qualified_path) = extract_qualified_path(expression, source_text)
                && catalog
                    .known_classes
                    .contains(&qualified_path.replace('.', "::"))
            {
                return Some(qualified_path);
            }
            let member_name = extract_identifier(source_text, *member_span);
            let owner_class =
                infer_receiver_class(object, catalog, source_text, enclosing_class, scopes)?;
            lookup_field_type_in_hierarchy(catalog, &owner_class, member_name)
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => resolve_call_candidate(
            callee,
            arguments.len(),
            catalog,
            source_text,
            enclosing_class,
            scopes,
        )
        .and_then(|candidate| candidate.return_class_name.clone()),
        _ => None,
    }
}

fn resolve_call_candidate<'catalog>(
    callee: &Expression,
    argument_count: usize,
    catalog: &'catalog CallableCatalog,
    source_text: &str,
    enclosing_class: Option<&str>,
    scopes: &NormalizerScopes,
) -> Option<&'catalog CallableMetadata> {
    match callee {
        Expression::Identifier {
            span: identifier_span,
        } => {
            let identifier_name = extract_identifier(source_text, *identifier_span);
            if let Some(cls) = enclosing_class
                && let Some(member_match) =
                    find_matching_callable(catalog, Some(cls), identifier_name, argument_count)
            {
                return Some(member_match);
            }
            find_matching_callable(catalog, None, identifier_name, argument_count)
        }
        Expression::MemberAccess {
            object,
            member_span,
            ..
        } => {
            let member_name = extract_identifier(source_text, *member_span);
            let receiver_class =
                infer_receiver_class(object, catalog, source_text, enclosing_class, scopes)?;
            find_matching_callable(catalog, Some(&receiver_class), member_name, argument_count)
        }
        _ => None,
    }
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
    catalog: &CallableCatalog,
    source_text: &str,
    enclosing_class: Option<&str>,
    scopes: &mut NormalizerScopes,
) {
    match expression {
        Expression::Binary { left, right, .. } => {
            normalize_expression(left, catalog, source_text, enclosing_class, scopes);
            normalize_expression(right, catalog, source_text, enclosing_class, scopes);
        }
        Expression::Unary { operand, .. } => {
            normalize_expression(operand, catalog, source_text, enclosing_class, scopes);
        }
        Expression::Assignment { target, value, .. } => {
            normalize_expression(target, catalog, source_text, enclosing_class, scopes);
            normalize_expression(value, catalog, source_text, enclosing_class, scopes);
        }
        Expression::MemberAccess { object, .. } => {
            normalize_expression(object, catalog, source_text, enclosing_class, scopes);
        }
        Expression::MethodCall {
            span,
            callee,
            arguments,
            ..
        } => {
            normalize_expression(callee, catalog, source_text, enclosing_class, scopes);
            for argument in arguments.iter_mut() {
                normalize_expression(argument, catalog, source_text, enclosing_class, scopes);
            }

            if let Some(candidate) = resolve_call_candidate(
                callee,
                arguments.len(),
                catalog,
                source_text,
                enclosing_class,
                scopes,
            ) {
                normalize_arguments_list(arguments, candidate, *span);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            normalize_expression(array, catalog, source_text, enclosing_class, scopes);
            normalize_expression(index, catalog, source_text, enclosing_class, scopes);
        }
        Expression::ArrayCreation { size, .. } => {
            normalize_expression(size, catalog, source_text, enclosing_class, scopes);
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for element in elements.iter_mut() {
                normalize_expression(element, catalog, source_text, enclosing_class, scopes);
            }
        }
        Expression::NewInstance {
            span,
            instance_type,
            arguments,
        } => {
            for argument in arguments.iter_mut() {
                normalize_expression(argument, catalog, source_text, enclosing_class, scopes);
            }

            if let Some(class_name) = extract_type_name(instance_type, source_text) {
                let simple_ctor = simple_type_name(&class_name);
                if let Some(candidate) =
                    find_matching_callable(catalog, Some(&class_name), simple_ctor, arguments.len())
                {
                    normalize_arguments_list(arguments, candidate, *span);
                }
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
            normalize_expression(
                inner_expression,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            normalize_expression(condition, catalog, source_text, enclosing_class, scopes);
            normalize_expression(then_branch, catalog, source_text, enclosing_class, scopes);
            normalize_expression(else_branch, catalog, source_text, enclosing_class, scopes);
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            for parameter in parameters.iter_mut() {
                if let Some(ref mut default_value) = parameter.default_value {
                    normalize_expression(
                        default_value,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
            }
            scopes.push();
            for parameter in parameters.iter() {
                let param_name = extract_identifier(source_text, parameter.name_span).to_string();
                let param_type = extract_type_name(&parameter.param_type, source_text);
                scopes.insert(param_name, param_type);
            }
            match body {
                LambdaBody::Expression(lambda_expression) => {
                    normalize_expression(
                        lambda_expression,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
                LambdaBody::Block(lambda_block) => {
                    normalize_block(lambda_block, catalog, source_text, enclosing_class, scopes);
                }
            }
            scopes.pop();
        }
        Expression::Block {
            statements, result, ..
        } => {
            scopes.push();
            for statement in statements.iter_mut() {
                normalize_statement(statement, catalog, source_text, enclosing_class, scopes);
            }
            if let Some(result_expression) = result {
                normalize_expression(
                    result_expression,
                    catalog,
                    source_text,
                    enclosing_class,
                    scopes,
                );
            }
            scopes.pop();
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
    catalog: &CallableCatalog,
    source_text: &str,
    enclosing_class: Option<&str>,
    scopes: &mut NormalizerScopes,
) {
    scopes.push();
    for statement in &mut block.statements {
        normalize_statement(statement, catalog, source_text, enclosing_class, scopes);
    }
    scopes.pop();
}

/// Normalizes an individual statement.
fn normalize_statement(
    statement: &mut Statement,
    catalog: &CallableCatalog,
    source_text: &str,
    enclosing_class: Option<&str>,
    scopes: &mut NormalizerScopes,
) {
    match statement {
        Statement::Block(block_statement) => {
            normalize_block(
                block_statement,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Statement::If(if_statement) => {
            normalize_expression(
                &mut if_statement.condition,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            normalize_statement(
                &mut if_statement.then_branch,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            if let Some(ref mut else_branch) = if_statement.else_branch {
                normalize_statement(else_branch, catalog, source_text, enclosing_class, scopes);
            }
        }
        Statement::While(while_statement) => {
            normalize_expression(
                &mut while_statement.condition,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            normalize_statement(
                &mut while_statement.body,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Statement::DoWhile(do_while_statement) => {
            normalize_statement(
                &mut do_while_statement.body,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            normalize_expression(
                &mut do_while_statement.condition,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Statement::For(for_statement) => {
            scopes.push();
            if let Some(ref mut initializer) = for_statement.initializer {
                normalize_statement(initializer, catalog, source_text, enclosing_class, scopes);
            }
            if let Some(ref mut condition) = for_statement.condition {
                normalize_expression(condition, catalog, source_text, enclosing_class, scopes);
            }
            if let Some(ref mut update) = for_statement.update {
                normalize_expression(update, catalog, source_text, enclosing_class, scopes);
            }
            normalize_statement(
                &mut for_statement.body,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            scopes.pop();
        }
        Statement::Switch(switch_statement) => {
            normalize_expression(
                &mut switch_statement.discriminant,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            for switch_case in &mut switch_statement.cases {
                normalize_expression(
                    &mut switch_case.case_value,
                    catalog,
                    source_text,
                    enclosing_class,
                    scopes,
                );
                scopes.push();
                for case_statement in &mut switch_case.statements {
                    normalize_statement(
                        case_statement,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
                scopes.pop();
            }
        }
        Statement::Try(try_statement) => {
            normalize_block(
                &mut try_statement.try_block,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            for catch_clause in &mut try_statement.catch_clauses {
                scopes.push();
                let catch_var =
                    extract_identifier(source_text, catch_clause.variable_name).to_string();
                let catch_cls = extract_type_name(&catch_clause.exception_type, source_text);
                scopes.insert(catch_var, catch_cls);
                normalize_block(
                    &mut catch_clause.block,
                    catalog,
                    source_text,
                    enclosing_class,
                    scopes,
                );
                scopes.pop();
            }
            if let Some(ref mut finally_block) = try_statement.finally_block {
                normalize_block(finally_block, catalog, source_text, enclosing_class, scopes);
            }
        }
        Statement::Throw(throw_statement) => {
            normalize_expression(
                &mut throw_statement.expression,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Statement::Return(return_statement) => {
            if let Some(ref mut value) = return_statement.value {
                normalize_expression(value, catalog, source_text, enclosing_class, scopes);
            }
        }
        Statement::VariableDeclaration(variable_statement) => {
            if let Some(ref mut initializer) = variable_statement.initializer {
                normalize_expression(initializer, catalog, source_text, enclosing_class, scopes);
            }
            let var_name =
                extract_identifier(source_text, variable_statement.name_span).to_string();
            let var_class = extract_type_name(&variable_statement.var_type, source_text);
            scopes.insert(var_name, var_class);
        }
        Statement::Expression(expression_statement) => {
            normalize_expression(
                &mut expression_statement.expression,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Statement::Assert(assert_statement) => {
            normalize_expression(
                &mut assert_statement.condition,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            if let Some(ref mut message) = assert_statement.message {
                normalize_expression(message, catalog, source_text, enclosing_class, scopes);
            }
        }
        Statement::Exit(exit_statement) => {
            normalize_expression(
                &mut exit_statement.code,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Statement::Declaration(declaration) => {
            normalize_declaration(declaration, catalog, source_text, enclosing_class, scopes);
        }
        Statement::Break(_) | Statement::Continue(_) => {}
    }
}

fn push_parameters_to_scope(
    parameters: &[crate::phases::parser::statements::declarations::Parameter],
    source_text: &str,
    scopes: &mut NormalizerScopes,
) {
    scopes.push();
    for parameter in parameters {
        let param_name = extract_identifier(source_text, parameter.name_span).to_string();
        let param_class = extract_type_name(&parameter.param_type, source_text);
        scopes.insert(param_name, param_class);
    }
}

fn normalize_declaration(
    declaration: &mut Declaration,
    catalog: &CallableCatalog,
    source_text: &str,
    enclosing_class: Option<&str>,
    scopes: &mut NormalizerScopes,
) {
    match declaration {
        Declaration::Method(method_declaration) => {
            for parameter in &mut method_declaration.parameters {
                if let Some(ref mut default_value) = parameter.default_value {
                    normalize_expression(
                        default_value,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
            }
            if let Some(ref mut body) = method_declaration.body {
                push_parameters_to_scope(&method_declaration.parameters, source_text, scopes);
                normalize_block(body, catalog, source_text, enclosing_class, scopes);
                scopes.pop();
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
                                    scopes,
                                );
                            }
                        }
                        if let Some(ref mut body) = method_declaration.body {
                            push_parameters_to_scope(
                                &method_declaration.parameters,
                                source_text,
                                scopes,
                            );
                            normalize_block(
                                body,
                                catalog,
                                source_text,
                                Some(&current_class),
                                scopes,
                            );
                            scopes.pop();
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
                                    scopes,
                                );
                            }
                        }
                        push_parameters_to_scope(
                            &constructor_declaration.parameters,
                            source_text,
                            scopes,
                        );
                        if let Some(ref mut super_arguments) =
                            constructor_declaration.super_call_args
                        {
                            for argument in super_arguments.iter_mut() {
                                normalize_expression(
                                    argument,
                                    catalog,
                                    source_text,
                                    Some(&current_class),
                                    scopes,
                                );
                            }
                        }
                        normalize_block(
                            &mut constructor_declaration.body,
                            catalog,
                            source_text,
                            Some(&current_class),
                            scopes,
                        );
                        scopes.pop();
                    }
                    ClassMember::Destructor(destructor_declaration) => {
                        normalize_block(
                            &mut destructor_declaration.body,
                            catalog,
                            source_text,
                            Some(&current_class),
                            scopes,
                        );
                    }
                    ClassMember::Field(field_declaration) => {
                        if let Some(ref mut initializer) = field_declaration.initializer {
                            normalize_expression(
                                initializer,
                                catalog,
                                source_text,
                                Some(&current_class),
                                scopes,
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
                                    scopes,
                                );
                            }
                        }
                        push_parameters_to_scope(
                            &operator_declaration.parameters,
                            source_text,
                            scopes,
                        );
                        normalize_block(
                            &mut operator_declaration.body,
                            catalog,
                            source_text,
                            Some(&current_class),
                            scopes,
                        );
                        scopes.pop();
                    }
                    _ => {}
                }
            }
        }
        Declaration::Field(field_declaration) => {
            if let Some(ref mut initializer) = field_declaration.initializer {
                normalize_expression(initializer, catalog, source_text, enclosing_class, scopes);
            }
        }
        Declaration::Constructor(constructor_declaration) => {
            for parameter in &mut constructor_declaration.parameters {
                if let Some(ref mut default_value) = parameter.default_value {
                    normalize_expression(
                        default_value,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
            }
            push_parameters_to_scope(&constructor_declaration.parameters, source_text, scopes);
            if let Some(ref mut super_arguments) = constructor_declaration.super_call_args {
                for argument in super_arguments.iter_mut() {
                    normalize_expression(argument, catalog, source_text, enclosing_class, scopes);
                }
            }
            normalize_block(
                &mut constructor_declaration.body,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            scopes.pop();
        }
        Declaration::Destructor(destructor_declaration) => {
            normalize_block(
                &mut destructor_declaration.body,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
        }
        Declaration::Operator(operator_declaration) => {
            for parameter in &mut operator_declaration.parameters {
                if let Some(ref mut default_value) = parameter.default_value {
                    normalize_expression(
                        default_value,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
            }
            push_parameters_to_scope(&operator_declaration.parameters, source_text, scopes);
            normalize_block(
                &mut operator_declaration.body,
                catalog,
                source_text,
                enclosing_class,
                scopes,
            );
            scopes.pop();
        }
        Declaration::Enum(enum_declaration) => {
            for variant in &mut enum_declaration.variants {
                if let Some(ref mut explicit_value) = variant.explicit_value {
                    normalize_expression(
                        explicit_value,
                        catalog,
                        source_text,
                        enclosing_class,
                        scopes,
                    );
                }
            }
        }
        Declaration::Interface(_) => {}
    }
}

/// Normalizes all call sites across all parsed program ASTs without external symbol table context.
pub fn normalize_call_sites(ast_forest: &mut [Option<ProgramAst>], sources: &[String]) {
    normalize_call_sites_with_symbols(ast_forest, sources, None);
}

/// Normalizes all call sites across all parsed program ASTs using discovered symbol table metadata.
pub fn normalize_call_sites_with_symbols(
    ast_forest: &mut [Option<ProgramAst>],
    sources: &[String],
    symbol_table: Option<&SymbolTable>,
) {
    let catalog = collect_callable_catalog(ast_forest, sources, symbol_table);

    for (source_index, program_option) in ast_forest.iter_mut().enumerate() {
        let Some(program_ast) = program_option else {
            continue;
        };
        let source_text = sources
            .get(source_index)
            .map(|text| text.as_str())
            .unwrap_or("");

        let mut scopes = NormalizerScopes::new();
        for declaration in &mut program_ast.declarations {
            normalize_declaration(declaration, &catalog, source_text, None, &mut scopes);
        }
    }
}
