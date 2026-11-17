//! Pass 3: Detailed AST statement and expression type checking, scoping, overload resolution, and generic instantiation.

#![forbid(unsafe_code)]

use crate::BindingError;
use crate::phases::binder::layout::ClassLayout;
use crate::phases::binder::monomorphizer::GenericBlueprintRegistry;
use crate::phases::binder::passes::pass1_symbol_discovery::{
    compute_effective_package, extract_namespace_path,
};
use crate::phases::binder::symbols::{LocalSymbolEntry, SymbolEntry, SymbolId, SymbolTable};
use crate::phases::binder::type_resolver::resolve_type_annotation;
use crate::phases::lexing::{Span, TokenKind};
use crate::phases::parser::statements::Statement;
use crate::phases::parser::statements::control_flow::*;
use crate::phases::parser::statements::declarations::*;
use crate::phases::parser::statements::expressions::*;
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::parser::visitor::AstVisitor;
use solix_common::{PrimitiveKind, Type};
use std::collections::HashMap;

/// Semantic type checking and expression evaluation visitor traversing the parsed AST.
pub struct TypeCheckingVisitor<'ctx> {
    pub source_id: u16,
    pub source_text: &'ctx str,
    pub symbol_table: &'ctx mut SymbolTable,
    pub blueprint_registry: &'ctx mut GenericBlueprintRegistry,
    pub layouts: &'ctx mut HashMap<SymbolId, ClassLayout>,
    pub node_types: &'ctx mut HashMap<Span, Type>,
    pub resolved_symbols: &'ctx mut HashMap<Span, SymbolId>,
    pub errors: &'ctx mut Vec<BindingError>,
    pub current_class: Option<SymbolId>,
    pub current_class_generic_names: std::collections::HashSet<String>,
    pub current_method_generic_names: std::collections::HashSet<String>,
    pub current_method_return_type: Option<Type>,
    pub is_static_method: bool,
    pub is_inside_destructor: bool,
    pub loop_depth: usize,
    pub switch_depth: usize,
    pub next_vtable_id: &'ctx mut u32,
    pub source_texts: &'ctx [String],
}

fn get_qualified_member_path(expression: &Expression, source_text: &str) -> Option<String> {
    match expression {
        Expression::Identifier { span } => {
            if span.end as usize <= source_text.len() {
                Some(source_text[span.start as usize..span.end as usize].to_string())
            } else {
                None
            }
        }
        Expression::MemberAccess {
            object,
            member_span,
            ..
        } => {
            let base_path = get_qualified_member_path(object, source_text)?;
            let member_name = if member_span.end as usize <= source_text.len() {
                &source_text[member_span.start as usize..member_span.end as usize]
            } else {
                return None;
            };
            Some(format!("{base_path}::{member_name}"))
        }
        _ => None,
    }
}

impl<'ctx> TypeCheckingVisitor<'ctx> {
    pub fn class_implements_equality_operator(&self, symbol_name: &str) -> bool {
        let simple_name = symbol_name.rsplit(['.', ':']).next().unwrap_or(symbol_name);
        for candidate in [symbol_name, simple_name] {
            if let Some(class_id) = self.symbol_table.resolve_symbol(self.source_id, candidate)
                && let Some(class_entry) = self.symbol_table.get_symbol(class_id)
            {
                for &child_id in &class_entry.members {
                    if let Some(child_entry) = self.symbol_table.get_symbol(child_id)
                        && (child_entry.name == "operator_==" || child_entry.name == "operator_!=")
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Evaluates the semantic [`Type`] of an arbitrary expression.
    pub fn evaluate_expression_type(&mut self, expression: &Expression) -> Type {
        let span = expression.span();
        let evaluated_type = self.evaluate_expression_type_internal(expression);
        self.node_types.insert(span, evaluated_type.clone());
        evaluated_type
    }

    /// Evaluates the semantic [`Type`] of an expression, coercing literals if an expected target type is provided.
    pub fn evaluate_expression_with_expected(
        &mut self,
        expression: &Expression,
        expected_type: Option<&Type>,
    ) -> Type {
        let span = expression.span();
        if let Some(target_type) = expected_type
            && coerce_literal_node_type(self.node_types, expression, target_type)
        {
            return target_type.clone();
        }
        let evaluated_type = self.evaluate_expression_type_internal(expression);
        self.node_types.insert(span, evaluated_type.clone());
        evaluated_type
    }

    /// Coerces a literal or negated literal expression to a compatible target type,
    /// recording the updated type in `self.node_types`.
    pub fn coerce_literal_node_type(
        &mut self,
        expression: &Expression,
        target_type: &Type,
    ) -> bool {
        coerce_literal_node_type(self.node_types, expression, target_type)
    }

    fn evaluate_expression_type_internal(&mut self, expression: &Expression) -> Type {
        let span = expression.span();
        match expression {
            Expression::Literal { value, .. } => match value {
                LiteralValue::Int(literal_val) => {
                    if *literal_val > i32::MAX as i64 || *literal_val < i32::MIN as i64 {
                        Type::Primitive(PrimitiveKind::Int64)
                    } else {
                        Type::Primitive(PrimitiveKind::Int32)
                    }
                }
                LiteralValue::UInt(literal_val) => {
                    if *literal_val > u32::MAX as u64 {
                        Type::Primitive(PrimitiveKind::UInt64)
                    } else {
                        Type::Primitive(PrimitiveKind::UInt32)
                    }
                }
                LiteralValue::Float(_) => Type::Primitive(PrimitiveKind::Float64),
                LiteralValue::String(_) => Type::Class {
                    symbol_name: "std.string".to_string(),
                    type_arguments: Vec::new(),
                },
                LiteralValue::Char(_) => Type::Primitive(PrimitiveKind::Char),
                LiteralValue::Bool(_) => Type::Primitive(PrimitiveKind::Bool),
                LiteralValue::Null => Type::Null,
            },
            Expression::Identifier { span } => {
                let identifier_name = if span.end as usize <= self.source_text.len() {
                    &self.source_text[span.start as usize..span.end as usize]
                } else {
                    ""
                };

                // Check "this"
                if identifier_name == "this" {
                    if self.is_static_method {
                        self.errors
                            .push(BindingError::ThisUsedOutsideInstanceMethod {
                                span: span.to_source_span(),
                            });
                        return Type::Error;
                    }
                    if let Some(class_id) = self.current_class
                        && let Some(class_entry) = self.symbol_table.get_symbol(class_id)
                    {
                        return class_entry.symbol_type.clone();
                    }
                    self.errors
                        .push(BindingError::ThisUsedOutsideInstanceMethod {
                            span: span.to_source_span(),
                        });
                    return Type::Error;
                }

                // Check "super"
                if identifier_name == "super" {
                    let Some(class_id) = self.current_class.filter(|_| !self.is_static_method)
                    else {
                        self.errors.push(BindingError::InvalidSuperCall {
                            span: span.to_source_span(),
                        });
                        return Type::Error;
                    };
                    let parent_class_id = self
                        .symbol_table
                        .get_symbol(class_id)
                        .and_then(|class_entry| class_entry.parent_symbol);
                    if let Some(base_id) = parent_class_id
                        && let Some(base_entry) = self.symbol_table.get_symbol(base_id)
                    {
                        return base_entry.symbol_type.clone();
                    } else {
                        self.errors.push(BindingError::InvalidSuperCall {
                            span: span.to_source_span(),
                        });
                        return Type::Error;
                    }
                }

                if let Some(symbol_id) = self
                    .symbol_table
                    .resolve_symbol(self.source_id, identifier_name)
                {
                    self.resolved_symbols.insert(*span, symbol_id);
                    if let Some(symbol_entry) = self.symbol_table.get_symbol(symbol_id) {
                        symbol_entry.symbol_type.clone()
                    } else {
                        Type::Error
                    }
                } else {
                    self.errors
                        .push(BindingError::undefined_symbol(identifier_name, *span));
                    Type::Error
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let mut left_type = self.evaluate_expression_type(left);
                let mut right_type = self.evaluate_expression_type(right);

                // If one side is a typed numeric variable/expr and the other side is a numeric literal:
                // Coerce the literal operand to the sibling operand's type if compatible!
                if left_type.is_numeric()
                    && !is_literal_or_negated_literal(left)
                    && is_literal_or_negated_literal(right)
                    && is_literal_compatible_with_type(right, &left_type)
                {
                    self.coerce_literal_node_type(right, &left_type);
                    right_type = left_type.clone();
                } else if right_type.is_numeric()
                    && !is_literal_or_negated_literal(right)
                    && is_literal_or_negated_literal(left)
                    && is_literal_compatible_with_type(left, &right_type)
                {
                    self.coerce_literal_node_type(left, &right_type);
                    left_type = right_type.clone();
                }

                match operator {
                    // Arithmetic operations
                    TokenKind::OperatorPlus
                    | TokenKind::OperatorMinus
                    | TokenKind::OperatorMultiply
                    | TokenKind::OperatorDivide
                    | TokenKind::OperatorModulo => {
                        // String concatenation
                        let is_string_left = matches!(
                            &left_type,
                            Type::Class { symbol_name, .. }
                                if symbol_name == "std.string"
                                    || symbol_name == "std::string"
                                    || symbol_name == "string"
                        );
                        let is_string_right = matches!(
                            &right_type,
                            Type::Class { symbol_name, .. }
                                if symbol_name == "std.string"
                                    || symbol_name == "std::string"
                                    || symbol_name == "string"
                        );
                        if *operator == TokenKind::OperatorPlus
                            && (is_string_left || is_string_right)
                        {
                            Type::Class {
                                symbol_name: "std.string".to_string(),
                                type_arguments: Vec::new(),
                            }
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type == Type::Primitive(PrimitiveKind::Float64)
                                || right_type == Type::Primitive(PrimitiveKind::Float64)
                            {
                                Type::Primitive(PrimitiveKind::Float64)
                            } else if left_type == Type::Primitive(PrimitiveKind::Float32)
                                || right_type == Type::Primitive(PrimitiveKind::Float32)
                            {
                                Type::Primitive(PrimitiveKind::Float32)
                            } else if left_type == Type::Primitive(PrimitiveKind::Int64)
                                || right_type == Type::Primitive(PrimitiveKind::Int64)
                            {
                                Type::Primitive(PrimitiveKind::Int64)
                            } else if left_type == Type::Primitive(PrimitiveKind::UInt64)
                                || right_type == Type::Primitive(PrimitiveKind::UInt64)
                            {
                                Type::Primitive(PrimitiveKind::UInt64)
                            } else {
                                Type::Primitive(PrimitiveKind::Int32)
                            }
                        } else if let Type::Class {
                            ref symbol_name, ..
                        } = left_type
                        {
                            let op_name = format!("operator_{}", operator.as_static_str());
                            if let Some(class_id) = self
                                .symbol_table
                                .resolve_symbol(self.source_id, symbol_name)
                                && let Some(class_entry) = self.symbol_table.get_symbol(class_id)
                            {
                                for &child_id in &class_entry.members {
                                    if let Some(child_entry) =
                                        self.symbol_table.get_symbol(child_id)
                                        && child_entry.name == op_name
                                        && let Type::FunctionPointer {
                                            ref return_type, ..
                                        } = child_entry.symbol_type
                                    {
                                        return *return_type.clone();
                                    }
                                }
                            }
                            self.errors.push(BindingError::InvalidOperatorOverload {
                                operator_name: op_name,
                                reason: format!(
                                    "Class '{symbol_name}' does not implement operator '{}'",
                                    operator.as_static_str()
                                ),
                                span: span.to_source_span(),
                            });
                            Type::Error
                        } else {
                            self.errors.push(BindingError::TypeMismatch {
                                expected: "numeric type".to_string(),
                                found: format!(
                                    "Cannot apply operator '{}' to types '{}' and '{}'",
                                    operator.as_static_str(),
                                    left_type,
                                    right_type
                                ),
                                span: span.to_source_span(),
                            });
                            Type::Error
                        }
                    }
                    // Relational comparisons
                    TokenKind::OperatorEqual
                    | TokenKind::OperatorNotEqual
                    | TokenKind::OperatorLessThan
                    | TokenKind::OperatorLessEqual
                    | TokenKind::OperatorGreaterThan
                    | TokenKind::OperatorGreaterEqual => {
                        let is_null_comparison = matches!(
                            operator,
                            TokenKind::OperatorEqual | TokenKind::OperatorNotEqual
                        ) && (left_type == Type::Null
                            || right_type == Type::Null
                            || matches!(
                                left.as_ref(),
                                Expression::Literal {
                                    value: LiteralValue::Null,
                                    ..
                                }
                            )
                            || matches!(
                                right.as_ref(),
                                Expression::Literal {
                                    value: LiteralValue::Null,
                                    ..
                                }
                            ));

                        // Check if left class implements operator == or != or < etc.
                        if !is_null_comparison
                            && let Type::Class {
                                ref symbol_name, ..
                            } = left_type
                        {
                            let op_name = format!("operator_{}", operator.as_static_str());
                            if let Some(class_id) = self
                                .symbol_table
                                .resolve_symbol(self.source_id, symbol_name)
                                && let Some(class_entry) = self.symbol_table.get_symbol(class_id)
                            {
                                for &child_id in &class_entry.members {
                                    if let Some(child_entry) =
                                        self.symbol_table.get_symbol(child_id)
                                        && child_entry.name == op_name
                                        && let Type::FunctionPointer {
                                            ref return_type, ..
                                        } = child_entry.symbol_type
                                    {
                                        return *return_type.clone();
                                    }
                                }
                            }
                        }

                        // Check enum comparison compatibility
                        if let (
                            Type::Enum {
                                symbol_name: left_enum_name,
                            },
                            Type::Enum {
                                symbol_name: right_enum_name,
                            },
                        ) = (&left_type, &right_type)
                            && left_enum_name != right_enum_name
                        {
                            self.errors.push(BindingError::TypeMismatch {
                                expected: left_enum_name.clone(),
                                found: right_enum_name.clone(),
                                span: right.span().to_source_span(),
                            });
                        }

                        Type::Primitive(PrimitiveKind::Bool)
                    }
                    // Logical operations
                    TokenKind::OperatorLogicalAnd | TokenKind::OperatorLogicalOr => {
                        Type::Primitive(PrimitiveKind::Bool)
                    }
                    // Bitwise operations
                    TokenKind::PunctuationAmpersand
                    | TokenKind::OperatorBitwiseOr
                    | TokenKind::OperatorBitwiseXor
                    | TokenKind::OperatorShiftLeft
                    | TokenKind::OperatorShiftRight => {
                        let mut has_error = false;
                        if !left_type.is_integer() && left_type != Type::Error {
                            self.errors.push(BindingError::BitwiseOperatorOnNonInteger {
                                operator: operator.as_static_str().to_string(),
                                found: left_type.to_string(),
                                span: left.span().to_source_span(),
                            });
                            has_error = true;
                        }
                        if !right_type.is_integer() && right_type != Type::Error {
                            self.errors.push(BindingError::BitwiseOperatorOnNonInteger {
                                operator: operator.as_static_str().to_string(),
                                found: right_type.to_string(),
                                span: right.span().to_source_span(),
                            });
                            has_error = true;
                        }
                        if has_error { Type::Error } else { left_type }
                    }
                    _ => left_type,
                }
            }
            Expression::Unary {
                operator,
                operand,
                span,
                ..
            } => {
                let operand_type = self.evaluate_expression_type(operand);
                match operator {
                    TokenKind::OperatorLogicalNot => {
                        if operand_type != Type::Primitive(PrimitiveKind::Bool)
                            && operand_type != Type::Error
                        {
                            self.errors.push(BindingError::TypeMismatch {
                                expected: "bool".to_string(),
                                found: operand_type.to_string(),
                                span: operand.span().to_source_span(),
                            });
                        }
                        Type::Primitive(PrimitiveKind::Bool)
                    }
                    TokenKind::OperatorMinus => {
                        if !operand_type.is_numeric() && operand_type != Type::Error {
                            self.errors.push(BindingError::TypeMismatch {
                                expected: "numeric type".to_string(),
                                found: format!(
                                    "Cannot apply unary operator '{}' to type '{}'",
                                    operator.as_static_str(),
                                    operand_type
                                ),
                                span: span.to_source_span(),
                            });
                            Type::Error
                        } else if let Expression::Literal {
                            value: LiteralValue::Int(int_val),
                            ..
                        } = &**operand
                        {
                            if *int_val == 2147483648 {
                                Type::Primitive(PrimitiveKind::Int32)
                            } else if *int_val > i32::MAX as i64 {
                                Type::Primitive(PrimitiveKind::Int64)
                            } else {
                                operand_type
                            }
                        } else if let Expression::Literal {
                            value: LiteralValue::UInt(uint_val),
                            ..
                        } = &**operand
                        {
                            if *uint_val == 2147483648 {
                                Type::Primitive(PrimitiveKind::Int32)
                            } else if *uint_val == 9223372036854775808 {
                                Type::Primitive(PrimitiveKind::Int64)
                            } else {
                                operand_type
                            }
                        } else if matches!(operand_type, Type::Primitive(PrimitiveKind::UInt64)) {
                            Type::Primitive(PrimitiveKind::Int64)
                        } else {
                            operand_type
                        }
                    }
                    TokenKind::OperatorBitwiseNot => {
                        if !operand_type.is_integer() && operand_type != Type::Error {
                            self.errors.push(BindingError::BitwiseOperatorOnNonInteger {
                                operator: "~".to_string(),
                                found: operand_type.to_string(),
                                span: operand.span().to_source_span(),
                            });
                            Type::Error
                        } else {
                            operand_type
                        }
                    }
                    TokenKind::OperatorPlus => {
                        if !operand_type.is_numeric() && operand_type != Type::Error {
                            self.errors.push(BindingError::TypeMismatch {
                                expected: "numeric type".to_string(),
                                found: format!(
                                    "Cannot apply unary operator '{}' to type '{}'",
                                    operator.as_static_str(),
                                    operand_type
                                ),
                                span: span.to_source_span(),
                            });
                            Type::Error
                        } else {
                            operand_type
                        }
                    }
                    TokenKind::OperatorIncrement | TokenKind::OperatorDecrement => operand_type,
                    _ => operand_type,
                }
            }
            Expression::ArrayAccess { array, index, .. } => {
                let target_type = self.evaluate_expression_type(array);
                let index_type = self.evaluate_expression_type(index);

                if !(index_type.is_signed_integer() || index_type.is_unsigned_integer())
                    && index_type != Type::Error
                {
                    self.errors.push(BindingError::TypeMismatch {
                        expected: "int32".to_string(),
                        found: format!("Array index must be int32, got '{index_type}'"),
                        span: index.span().to_source_span(),
                    });
                }

                match target_type {
                    Type::Array {
                        element_type,
                        depth,
                    } => {
                        if depth > 1 {
                            Type::Array {
                                element_type,
                                depth: depth - 1,
                            }
                        } else {
                            *element_type
                        }
                    }
                    Type::Tuple(elements) => {
                        let idx_opt = match &**index {
                            Expression::Literal {
                                value: LiteralValue::Int(int_val),
                                ..
                            } => Some(*int_val as usize),
                            Expression::Literal {
                                value: LiteralValue::UInt(uint_val),
                                ..
                            } => Some(*uint_val as usize),
                            _ => None,
                        };
                        match idx_opt {
                            Some(idx) if idx < elements.len() => elements[idx].clone(),
                            _ => elements
                                .first()
                                .cloned()
                                .unwrap_or(Type::Primitive(PrimitiveKind::Void)),
                        }
                    }
                    Type::Primitive(PrimitiveKind::TuplePrimitive) => {
                        Type::Primitive(PrimitiveKind::Int32)
                    }
                    Type::Class { symbol_name, .. }
                        if symbol_name == "std.string"
                            || symbol_name == "string"
                            || symbol_name == "std::string" =>
                    {
                        Type::Primitive(PrimitiveKind::Char)
                    }
                    _ => {
                        self.errors.push(BindingError::NonIndexableType {
                            found: target_type.to_string(),
                            span: span.to_source_span(),
                        });
                        Type::Error
                    }
                }
            }
            Expression::MemberAccess {
                span: member_access_span,
                object,
                member_span,
                is_scope_resolution,
            } => {
                let member_name = if member_span.end as usize <= self.source_text.len() {
                    &self.source_text[member_span.start as usize..member_span.end as usize]
                } else {
                    ""
                };

                // Check if this MemberAccess chain represents a package-qualified top-level function or variable
                if let Some(qualified_path) =
                    get_qualified_member_path(expression, self.source_text)
                {
                    let resolved = self
                        .symbol_table
                        .resolve_symbol(self.source_id, &qualified_path)
                        .or_else(|| {
                            if let Some((prefix, suffix)) = qualified_path.split_once("::") {
                                self.symbol_table.resolve_symbol(
                                    self.source_id,
                                    &format!("{prefix}::{prefix}::{suffix}"),
                                )
                            } else {
                                None
                            }
                        });
                    if let Some(symbol_id) = resolved
                        && let Some(symbol_entry) = self.symbol_table.get_symbol(symbol_id)
                        && (symbol_entry.parent_symbol.is_none()
                            || (*is_scope_resolution && symbol_entry.is_static)
                            || (symbol_entry.is_static && {
                                let parent_entry = symbol_entry
                                    .parent_symbol
                                    .and_then(|parent_id| self.symbol_table.get_symbol(parent_id));
                                parent_entry
                                    .map(|parent_symbol_entry| {
                                        let fqn_parts: Vec<&str> = parent_symbol_entry
                                            .fully_qualified_name
                                            .split("::")
                                            .collect();
                                        fqn_parts.len() <= 2
                                            && fqn_parts
                                                .first()
                                                .map(|package_name| {
                                                    package_name.eq_ignore_ascii_case(
                                                        &parent_symbol_entry.name,
                                                    )
                                                })
                                                .unwrap_or(false)
                                    })
                                    .unwrap_or(false)
                            }))
                    {
                        self.resolved_symbols.insert(*member_span, symbol_id);
                        self.resolved_symbols.insert(*member_access_span, symbol_id);
                        return symbol_entry.symbol_type.clone();
                    }
                }

                let target_type = self.evaluate_expression_type(object);

                // Array, tuple, and string .length property
                let is_string_target = matches!(
                    &target_type,
                    Type::Class { symbol_name, .. }
                        if symbol_name == "std.string"
                            || symbol_name == "std::string"
                            || symbol_name == "string"
                );
                if (matches!(target_type, Type::Array { .. })
                    || matches!(target_type, Type::Tuple(_))
                    || matches!(target_type, Type::Primitive(PrimitiveKind::TuplePrimitive))
                    || is_string_target)
                    && member_name == "length"
                {
                    if *is_scope_resolution {
                        self.errors
                            .push(BindingError::InstanceAccessViaScopeResolution {
                                member_name: member_name.to_string(),
                                owner: if matches!(target_type, Type::Array { .. }) {
                                    "array".to_string()
                                } else if is_string_target {
                                    "string".to_string()
                                } else {
                                    "tuple".to_string()
                                },
                                span: member_span.to_source_span(),
                            });
                    }
                    return Type::Primitive(PrimitiveKind::Int32);
                }

                // Check enum variants
                if let Type::Enum { ref symbol_name } = target_type
                    && let Some(enum_id) = self
                        .symbol_table
                        .resolve_symbol(self.source_id, symbol_name)
                    && let Some(enum_entry) = self.symbol_table.get_symbol(enum_id)
                {
                    for &child_id in &enum_entry.members {
                        if let Some(child_entry) = self.symbol_table.get_symbol(child_id)
                            && child_entry.name == member_name
                        {
                            self.resolved_symbols.insert(*member_span, child_id);
                            return child_entry.symbol_type.clone();
                        }
                    }
                }

                // Check class field or method with inheritance hierarchy traversal
                if let Type::Class {
                    ref symbol_name,
                    ref type_arguments,
                } = target_type
                {
                    let resolved_class_id = if !type_arguments.is_empty() {
                        let mangled = crate::phases::binder::monomorphizer::GenericBlueprintRegistry::mangle_name(
                            symbol_name,
                            type_arguments,
                        );
                        self.blueprint_registry
                            .instantiated_classes
                            .get(&mangled)
                            .copied()
                            .or_else(|| self.symbol_table.resolve_symbol(self.source_id, &mangled))
                            .or_else(|| {
                                self.symbol_table
                                    .resolve_symbol(self.source_id, symbol_name)
                            })
                    } else {
                        self.symbol_table
                            .resolve_symbol(self.source_id, symbol_name)
                    };

                    if let Some(mut current_class_id) = resolved_class_id {
                        while let Some(class_entry) = self.symbol_table.get_symbol(current_class_id)
                        {
                            for &child_id in &class_entry.members {
                                if let Some(child_entry) = self.symbol_table.get_symbol(child_id)
                                    && child_entry.name == member_name
                                {
                                    self.resolved_symbols.insert(*member_span, child_id);

                                    let is_method = matches!(
                                        child_entry.symbol_type,
                                        Type::FunctionPointer { .. }
                                    );
                                    if is_method && !child_entry.is_static && *is_scope_resolution {
                                        self.errors.push(
                                            BindingError::InstanceMethodAccessViaStatic {
                                                method_name: member_name.to_string(),
                                                class_name: symbol_name.clone(),
                                                span: member_span.to_source_span(),
                                            },
                                        );
                                    } else if !is_method
                                        && child_entry.is_static
                                        && !*is_scope_resolution
                                    {
                                        self.errors.push(BindingError::StaticAccessViaDot {
                                            member_name: member_name.to_string(),
                                            owner: symbol_name.clone(),
                                            span: member_span.to_source_span(),
                                        });
                                    } else if !is_method
                                        && !child_entry.is_static
                                        && !child_entry.is_const
                                        && *is_scope_resolution
                                    {
                                        self.errors.push(
                                            BindingError::InstanceAccessViaScopeResolution {
                                                member_name: member_name.to_string(),
                                                owner: symbol_name.clone(),
                                                span: member_span.to_source_span(),
                                            },
                                        );
                                    }

                                    // Access control check
                                    if child_entry.is_private()
                                        && self.current_class != Some(current_class_id)
                                    {
                                        self.errors.push(BindingError::PrivateAccessViolation {
                                            member_name: member_name.to_string(),
                                            owner_class: symbol_name.clone(),
                                            span: member_span.to_source_span(),
                                        });
                                    }

                                    if child_entry.is_protected() {
                                        let is_allowed = match self.current_class {
                                            Some(curr) => {
                                                self.is_subclass_or_same(curr, current_class_id)
                                            }
                                            None => false,
                                        };
                                        if !is_allowed {
                                            self.errors.push(
                                                BindingError::ProtectedAccessViolation {
                                                    member_name: member_name.to_string(),
                                                    owner_class: symbol_name.clone(),
                                                    span: member_span.to_source_span(),
                                                },
                                            );
                                        }
                                    }

                                    return child_entry.symbol_type.clone();
                                }
                            }

                            if let Some(super_id) = class_entry.parent_symbol {
                                current_class_id = super_id;
                            } else {
                                break;
                            }
                        }
                    }

                    self.errors.push(BindingError::UndefinedSymbol {
                        symbol_name: format!(
                            "Class '{symbol_name}' has no member named '{member_name}'"
                        ),
                        span: member_span.to_source_span(),
                    });
                    return Type::Error;
                }

                if let Type::Interface {
                    ref symbol_name, ..
                } = target_type
                {
                    if let Some(iface_id) = self
                        .symbol_table
                        .resolve_symbol(self.source_id, symbol_name)
                        && let Some(iface_entry) = self.symbol_table.get_symbol(iface_id)
                    {
                        for &child_id in &iface_entry.members {
                            if let Some(child_entry) = self.symbol_table.get_symbol(child_id)
                                && child_entry.name == member_name
                            {
                                self.resolved_symbols.insert(*member_span, child_id);
                                return child_entry.symbol_type.clone();
                            }
                        }
                    }
                    self.errors.push(BindingError::undefined_symbol(
                        format!("Interface '{symbol_name}' has no member named '{member_name}'"),
                        *member_span,
                    ));
                    return Type::Error;
                }

                if target_type == Type::Error {
                    return Type::Error;
                }

                if matches!(
                    target_type,
                    Type::Primitive(_)
                        | Type::Null
                        | Type::FunctionPointer { .. }
                        | Type::Array { .. }
                        | Type::Tuple(_)
                ) {
                    self.errors
                        .push(BindingError::MemberAccessOnNonCompositeType {
                            found: target_type.to_string(),
                            member_name: member_name.to_string(),
                            span: member_span.to_source_span(),
                        });
                    return Type::Error;
                }

                Type::Error
            }
            Expression::MethodCall {
                callee,
                generic_arguments,
                arguments,
                ..
            } => {
                let callee_type = self.evaluate_expression_type(callee);
                let evaluated_arg_types: Vec<Type> = arguments
                    .iter()
                    .map(|argument| self.evaluate_expression_type(argument))
                    .collect();

                // If callee is member access, check method return type and resolve exact overload
                if let Expression::MemberAccess {
                    object,
                    member_span,
                    ..
                } = &**callee
                {
                    let member_name = if member_span.end as usize <= self.source_text.len() {
                        &self.source_text[member_span.start as usize..member_span.end as usize]
                    } else {
                        ""
                    };
                    let obj_type = self
                        .node_types
                        .get(&object.span())
                        .cloned()
                        .unwrap_or(Type::Primitive(PrimitiveKind::Void));
                    if let Type::Class {
                        ref symbol_name,
                        ref type_arguments,
                    } = obj_type
                    {
                        let mut class_substitutions = HashMap::new();
                        if !type_arguments.is_empty() {
                            let clean_name = symbol_name.replace("::", ".");
                            if let Some(blueprint) = self
                                .blueprint_registry
                                .blueprints
                                .get(symbol_name)
                                .or_else(|| self.blueprint_registry.blueprints.get(&clean_name))
                                .or_else(|| {
                                    let short_name =
                                        symbol_name.rsplit("::").next().unwrap_or(symbol_name);
                                    self.blueprint_registry.blueprints.get(short_name)
                                })
                            {
                                for (param_name, arg_type) in
                                    blueprint.parameter_names.iter().zip(type_arguments)
                                {
                                    class_substitutions
                                        .insert(param_name.clone(), arg_type.clone());
                                }
                            }
                        }

                        let cls_id = if !type_arguments.is_empty() {
                            let mangled =
                                crate::phases::binder::monomorphizer::GenericBlueprintRegistry::mangle_name(
                                    symbol_name,
                                    type_arguments,
                                );
                            self.blueprint_registry
                                .instantiated_classes
                                .get(&mangled)
                                .copied()
                                .or_else(|| {
                                    self.symbol_table.resolve_symbol(self.source_id, &mangled)
                                })
                                .or_else(|| {
                                    self.symbol_table
                                        .resolve_symbol(self.source_id, symbol_name)
                                })
                        } else {
                            self.symbol_table
                                .resolve_symbol(self.source_id, symbol_name)
                        };

                        if let Some(mut current_class_id) = cls_id {
                            let mut found_method: Option<(SymbolId, Type)> = None;
                            while let Some(class_entry) =
                                self.symbol_table.get_symbol(current_class_id)
                            {
                                for &child_id in &class_entry.members {
                                    if let Some(child_entry) =
                                        self.symbol_table.get_symbol(child_id)
                                        && child_entry.name == member_name
                                        && let Type::FunctionPointer {
                                            ref parameter_types,
                                            ref return_type,
                                        } = child_entry.symbol_type
                                    {
                                        let effective_params: Vec<Type> =
                                            if class_substitutions.is_empty() {
                                                parameter_types.clone()
                                            } else {
                                                parameter_types
                                                    .iter()
                                                    .map(|param_type| {
                                                        Self::substitute_type(
                                                            param_type,
                                                            &class_substitutions,
                                                        )
                                                    })
                                                    .collect()
                                            };
                                        let effective_return = if class_substitutions.is_empty() {
                                            *return_type.clone()
                                        } else {
                                            Self::substitute_type(return_type, &class_substitutions)
                                        };
                                        if let Some(matched_return) = self.match_method_signature(
                                            &effective_params,
                                            &effective_return,
                                            generic_arguments,
                                            &evaluated_arg_types,
                                            arguments,
                                        ) {
                                            for (expected_param, argument_expr) in
                                                effective_params.iter().zip(arguments.iter())
                                            {
                                                if is_literal_compatible_with_type(
                                                    argument_expr,
                                                    expected_param,
                                                ) {
                                                    coerce_literal_node_type(
                                                        self.node_types,
                                                        argument_expr,
                                                        expected_param,
                                                    );
                                                }
                                            }
                                            found_method = Some((child_id, matched_return));
                                            break;
                                        }
                                    }
                                }
                                if found_method.is_some() {
                                    break;
                                }
                                current_class_id = match class_entry.parent_symbol {
                                    Some(parent) => parent,
                                    None => break,
                                };
                            }
                            if let Some((method_id, ret_type)) = found_method {
                                self.resolved_symbols.insert(*member_span, method_id);
                                return ret_type;
                            } else {
                                let arg_str = evaluated_arg_types
                                    .iter()
                                    .map(|arg_type| arg_type.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                self.errors.push(BindingError::NoMatchingOverload {
                                    method_name: format!("{symbol_name}.{member_name}"),
                                    argument_types: arg_str,
                                    span: member_span.to_source_span(),
                                });
                                return Type::Error;
                            }
                        }
                    }
                    if let Type::Interface {
                        ref symbol_name, ..
                    } = obj_type
                        && let Some(iface_id) = self
                            .symbol_table
                            .resolve_symbol(self.source_id, symbol_name)
                        && let Some(iface_entry) = self.symbol_table.get_symbol(iface_id)
                    {
                        let mut found_method: Option<(SymbolId, Type, Vec<Type>)> = None;
                        for &child_id in &iface_entry.members {
                            if let Some(child_entry) = self.symbol_table.get_symbol(child_id)
                                && child_entry.name == member_name
                                && let Type::FunctionPointer {
                                    ref parameter_types,
                                    ref return_type,
                                } = child_entry.symbol_type
                            {
                                found_method =
                                    Some((child_id, *return_type.clone(), parameter_types.clone()));
                                break;
                            }
                        }
                        if let Some((method_id, return_type, parameter_types)) = found_method {
                            self.resolved_symbols.insert(*member_span, method_id);
                            if evaluated_arg_types.len() != parameter_types.len() {
                                self.errors.push(BindingError::ArgumentCountMismatch {
                                    expected: parameter_types.len(),
                                    found: evaluated_arg_types.len(),
                                    span: member_span.to_source_span(),
                                });
                            } else {
                                for (param_index, (expected_param, actual_arg)) in
                                    parameter_types.iter().zip(&evaluated_arg_types).enumerate()
                                {
                                    let is_compatible =
                                        actual_arg.is_assignable_to(expected_param, &|src, tgt| {
                                            self.check_is_subtype(src, tgt)
                                        }) || is_literal_compatible_with_type(
                                            &arguments[param_index],
                                            expected_param,
                                        );
                                    if is_compatible {
                                        coerce_literal_node_type(
                                            self.node_types,
                                            &arguments[param_index],
                                            expected_param,
                                        );
                                    } else if !is_compatible
                                        && *expected_param != Type::Error
                                        && *actual_arg != Type::Error
                                    {
                                        self.errors.push(BindingError::ArgumentTypeMismatch {
                                            parameter_index: param_index,
                                            expected: expected_param.to_string(),
                                            found: actual_arg.to_string(),
                                            span: arguments[param_index].span().to_source_span(),
                                        });
                                    }
                                }
                            }
                            return return_type;
                        } else {
                            self.errors.push(BindingError::undefined_symbol(
                                format!(
                                    "Interface '{symbol_name}' has no method named '{member_name}'"
                                ),
                                *member_span,
                            ));
                            return Type::Error;
                        }
                    }
                }

                if let Type::FunctionPointer {
                    return_type,
                    parameter_types,
                } = callee_type
                {
                    if let Some(matched_return) = self.match_method_signature(
                        &parameter_types,
                        &return_type,
                        generic_arguments,
                        &evaluated_arg_types,
                        arguments,
                    ) {
                        for (expected_param, argument_expr) in
                            parameter_types.iter().zip(arguments.iter())
                        {
                            if is_literal_compatible_with_type(argument_expr, expected_param) {
                                coerce_literal_node_type(
                                    self.node_types,
                                    argument_expr,
                                    expected_param,
                                );
                            }
                        }
                        return matched_return;
                    }
                    if evaluated_arg_types.len() != parameter_types.len() {
                        self.errors.push(BindingError::ArgumentCountMismatch {
                            expected: parameter_types.len(),
                            found: evaluated_arg_types.len(),
                            span: callee.span().to_source_span(),
                        });
                    } else {
                        for (param_index, (expected_param, actual_arg)) in
                            parameter_types.iter().zip(&evaluated_arg_types).enumerate()
                        {
                            let is_compatible =
                                actual_arg.is_assignable_to(expected_param, &|src, tgt| {
                                    self.check_is_subtype(src, tgt)
                                }) || is_literal_compatible_with_type(
                                    &arguments[param_index],
                                    expected_param,
                                );
                            if is_compatible {
                                coerce_literal_node_type(
                                    self.node_types,
                                    &arguments[param_index],
                                    expected_param,
                                );
                            } else if !is_compatible
                                && *expected_param != Type::Error
                                && *actual_arg != Type::Error
                            {
                                self.errors.push(BindingError::ArgumentTypeMismatch {
                                    parameter_index: param_index,
                                    expected: expected_param.to_string(),
                                    found: actual_arg.to_string(),
                                    span: arguments[param_index].span().to_source_span(),
                                });
                            }
                        }
                    }
                    return *return_type;
                }

                // If callee is member access, check method return type
                if let Expression::MemberAccess { member_span, .. } = &**callee
                    && let Some(&method_symbol_id) = self.resolved_symbols.get(member_span)
                    && let Some(entry) = self.symbol_table.get_symbol(method_symbol_id)
                    && let Type::FunctionPointer { return_type, .. } = &entry.symbol_type
                {
                    return *return_type.clone();
                }

                if let Expression::Identifier { span } = &**callee {
                    let function_name = if span.end as usize <= self.source_text.len() {
                        &self.source_text[span.start as usize..span.end as usize]
                    } else {
                        ""
                    };
                    if function_name == "super" {
                        self.errors.push(BindingError::SuperConstructorMustBeFirst {
                            span: span.to_source_span(),
                        });
                        return Type::Primitive(PrimitiveKind::Void);
                    }
                    if let Some(function_id) = self
                        .symbol_table
                        .resolve_symbol(self.source_id, function_name)
                        && let Some(entry) = self.symbol_table.get_symbol(function_id)
                        && let Type::FunctionPointer {
                            return_type,
                            parameter_types,
                        } = &entry.symbol_type
                    {
                        if evaluated_arg_types.len() != parameter_types.len() {
                            self.errors.push(BindingError::ArgumentCountMismatch {
                                expected: parameter_types.len(),
                                found: evaluated_arg_types.len(),
                                span: span.to_source_span(),
                            });
                        } else {
                            for (param_index, (expected_param, actual_arg)) in
                                parameter_types.iter().zip(&evaluated_arg_types).enumerate()
                            {
                                let is_compatible =
                                    actual_arg.is_assignable_to(expected_param, &|src, tgt| {
                                        self.check_is_subtype(src, tgt)
                                    }) || is_literal_compatible_with_type(
                                        &arguments[param_index],
                                        expected_param,
                                    );
                                if is_compatible {
                                    coerce_literal_node_type(
                                        self.node_types,
                                        &arguments[param_index],
                                        expected_param,
                                    );
                                } else if !is_compatible
                                    && *expected_param != Type::Error
                                    && *actual_arg != Type::Error
                                {
                                    self.errors.push(BindingError::ArgumentTypeMismatch {
                                        parameter_index: param_index,
                                        expected: expected_param.to_string(),
                                        found: actual_arg.to_string(),
                                        span: arguments[param_index].span().to_source_span(),
                                    });
                                }
                            }
                        }
                        return *return_type.clone();
                    }
                }

                Type::Primitive(PrimitiveKind::Void)
            }
            Expression::NewInstance {
                instance_type,
                arguments,
                ..
            } => {
                let resolved_type = resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    instance_type,
                    self.symbol_table,
                );

                let evaluated_arg_types: Vec<Type> = arguments
                    .iter()
                    .map(|argument| self.evaluate_expression_type(argument))
                    .collect();

                if let Type::Class {
                    ref symbol_name, ..
                } = resolved_type
                    && let Some(class_id) = self
                        .symbol_table
                        .resolve_symbol(self.source_id, symbol_name)
                {
                    self.resolved_symbols.insert(instance_type.span(), class_id);
                }

                if let Type::Class {
                    ref symbol_name,
                    ref type_arguments,
                } = resolved_type
                    && let Some(class_id) = self
                        .symbol_table
                        .resolve_symbol(self.source_id, symbol_name)
                    && let Some(class_entry) = self.symbol_table.get_symbol(class_id)
                {
                    let ctors: Vec<SymbolId> = class_entry
                        .members
                        .iter()
                        .filter_map(|&mid| {
                            self.symbol_table.get_symbol(mid).and_then(|sym_entry| {
                                if sym_entry.name == "constructor" {
                                    Some(mid)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();

                    if !ctors.is_empty() {
                        let mut found_ctor = false;
                        for ctor_id in ctors {
                            if let Some(ctor_symbol) = self.symbol_table.get_symbol(ctor_id)
                                && let Type::FunctionPointer {
                                    ref parameter_types,
                                    ..
                                } = ctor_symbol.symbol_type
                                && parameter_types.len() == evaluated_arg_types.len()
                                && parameter_types
                                    .iter()
                                    .zip(arguments.iter())
                                    .zip(&evaluated_arg_types)
                                    .all(|((expected_param, argument_expr), actual_arg)| {
                                        actual_arg.is_assignable_to(expected_param, &|src, tgt| {
                                            self.check_is_subtype(src, tgt)
                                        }) || actual_arg == expected_param
                                            || (actual_arg.is_numeric()
                                                && expected_param.is_numeric())
                                            || is_literal_compatible_with_type(
                                                argument_expr,
                                                expected_param,
                                            )
                                            || !type_arguments.is_empty()
                                    })
                            {
                                for (expected_param, argument_expr) in
                                    parameter_types.iter().zip(arguments.iter())
                                {
                                    if is_literal_compatible_with_type(
                                        argument_expr,
                                        expected_param,
                                    ) {
                                        coerce_literal_node_type(
                                            self.node_types,
                                            argument_expr,
                                            expected_param,
                                        );
                                    }
                                }
                                found_ctor = true;
                                break;
                            }
                        }
                        if !found_ctor {
                            let arg_str = evaluated_arg_types
                                .iter()
                                .map(|arg_type| arg_type.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            self.errors.push(BindingError::NoMatchingOverload {
                                method_name: format!("{symbol_name}.constructor"),
                                argument_types: arg_str,
                                span: instance_type.span().to_source_span(),
                            });
                        }
                    } else if !evaluated_arg_types.is_empty() {
                        let arg_str = evaluated_arg_types
                            .iter()
                            .map(|arg_type| arg_type.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        self.errors.push(BindingError::NoMatchingOverload {
                            method_name: format!("{symbol_name}.constructor"),
                            argument_types: arg_str,
                            span: instance_type.span().to_source_span(),
                        });
                    }
                }

                if matches!(resolved_type, Type::Interface { .. }) {
                    self.errors
                        .push(BindingError::CannotInstantiateAbstractType {
                            type_name: resolved_type.to_string(),
                            span: instance_type.span().to_source_span(),
                        });
                } else if let Type::Class {
                    ref symbol_name, ..
                } = resolved_type
                    && let Some(class_id) = self
                        .symbol_table
                        .resolve_symbol(self.source_id, symbol_name)
                    && let Some(class_entry) = self.symbol_table.get_symbol(class_id)
                    && class_entry.modifiers.contains(&Modifier::Abstract)
                {
                    self.errors
                        .push(BindingError::CannotInstantiateAbstractType {
                            type_name: symbol_name.clone(),
                            span: instance_type.span().to_source_span(),
                        });
                }

                // If generic class instantiation, monomorphize on the fly
                if let Type::Class {
                    ref symbol_name,
                    ref type_arguments,
                } = resolved_type
                    && !type_arguments.is_empty()
                    && let Err(errors) = self.blueprint_registry.monomorphize_class(
                        symbol_name,
                        type_arguments,
                        self.symbol_table,
                        self.layouts,
                        self.source_texts,
                        self.next_vtable_id,
                    )
                {
                    self.errors.extend(errors);
                }

                resolved_type
            }
            Expression::ArrayCreation {
                element_type, size, ..
            } => {
                let resolved_element = resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    element_type,
                    self.symbol_table,
                );
                self.evaluate_expression_type(size);
                match resolved_element {
                    Type::Array {
                        element_type: inner_element,
                        depth,
                    } => Type::Array {
                        element_type: inner_element,
                        depth: depth + 1,
                    },
                    other => Type::Array {
                        element_type: Box::new(other),
                        depth: 1,
                    },
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                let mut element_type = if let Some(first_element) = elements.first() {
                    self.evaluate_expression_type(first_element)
                } else {
                    Type::Primitive(PrimitiveKind::Int32)
                };

                for element in elements {
                    let current_elem_type = self.evaluate_expression_type(element);
                    if current_elem_type == Type::Primitive(PrimitiveKind::Float64) {
                        element_type = Type::Primitive(PrimitiveKind::Float64);
                    } else if current_elem_type == Type::Primitive(PrimitiveKind::Float32)
                        && element_type != Type::Primitive(PrimitiveKind::Float64)
                    {
                        element_type = Type::Primitive(PrimitiveKind::Float32);
                    } else if current_elem_type == Type::Primitive(PrimitiveKind::Int64)
                        && !element_type.is_floating_point()
                    {
                        element_type = Type::Primitive(PrimitiveKind::Int64);
                    }
                }

                for element in elements {
                    let elem_type = self.evaluate_expression_type(element);
                    let is_compatible = elem_type.is_assignable_to(&element_type, &|src, tgt| {
                        self.check_is_subtype(src, tgt)
                    }) || element_type
                        .is_assignable_to(&elem_type, &|src, tgt| self.check_is_subtype(src, tgt))
                        || is_literal_compatible_with_type(element, &element_type);
                    if is_compatible {
                        self.coerce_literal_node_type(element, &element_type);
                    } else if elem_type != Type::Error && element_type != Type::Error {
                        self.errors.push(BindingError::TypeMismatch {
                            expected: element_type.to_string(),
                            found: format!(
                                "Incompatible types in array literal: expected '{element_type}', found '{elem_type}'"
                            ),
                            span: element.span().to_source_span(),
                        });
                    }
                }
                match element_type {
                    Type::Array {
                        element_type: inner_element,
                        depth,
                    } => Type::Array {
                        element_type: inner_element,
                        depth: depth + 1,
                    },
                    other => Type::Array {
                        element_type: Box::new(other),
                        depth: 1,
                    },
                }
            }
            Expression::TupleCreation { elements, .. } => {
                let element_types: Vec<Type> = elements
                    .iter()
                    .map(|element| self.evaluate_expression_type(element))
                    .collect();
                Type::Tuple(element_types)
            }
            Expression::Block { result, .. } => {
                if let Some(res) = result {
                    self.evaluate_expression_type(res)
                } else {
                    Type::Primitive(PrimitiveKind::Void)
                }
            }
            Expression::Cast {
                target_type,
                expression,
                span,
            } => {
                let src_type = self.evaluate_expression_type(expression);
                let tgt_type = resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    target_type,
                    self.symbol_table,
                );

                if let (
                    Type::Class {
                        symbol_name: src_cls,
                        ..
                    },
                    Type::Class {
                        symbol_name: tgt_cls,
                        ..
                    },
                ) = (&src_type, &tgt_type)
                    && !self.check_is_subtype(src_cls, tgt_cls)
                    && !self.check_is_subtype(tgt_cls, src_cls)
                {
                    self.errors.push(BindingError::InvalidCastType {
                        source_type: src_cls.clone(),
                        target_type: tgt_cls.clone(),
                        span: span.to_source_span(),
                    });
                }
                tgt_type
            }
            Expression::Instanceof {
                expression,
                target_type,
                span,
            } => {
                let src_type = self.evaluate_expression_type(expression);
                let tgt_type = resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    target_type,
                    self.symbol_table,
                );
                let is_tuple_access = if let Expression::ArrayAccess { array, .. } = &**expression {
                    matches!(
                        self.evaluate_expression_type(array),
                        Type::Tuple(_) | Type::Primitive(PrimitiveKind::TuplePrimitive)
                    )
                } else {
                    false
                };

                if !is_tuple_access && (src_type.is_primitive() || tgt_type.is_primitive()) {
                    self.errors.push(BindingError::InvalidTypeForInstanceOf {
                        found: if src_type.is_primitive() {
                            src_type.to_string()
                        } else {
                            tgt_type.to_string()
                        },
                        span: span.to_source_span(),
                    });
                }
                if let Type::Unresolved(ref name) = tgt_type {
                    self.errors.push(BindingError::UndefinedSymbol {
                        symbol_name: format!(
                            "Cannot resolve type '{name}' in instanceof expression"
                        ),
                        span: target_type.span().to_source_span(),
                    });
                }
                Type::Primitive(PrimitiveKind::Bool)
            }
            Expression::Ternary {
                span,
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.evaluate_expression_type(condition);
                if cond_type != Type::Primitive(PrimitiveKind::Bool) && cond_type != Type::Error {
                    self.errors.push(BindingError::TypeMismatch {
                        expected: "bool".to_string(),
                        found: format!(
                            "Ternary condition must be of type 'bool', got '{cond_type}'"
                        ),
                        span: condition.span().to_source_span(),
                    });
                }
                let then_type = self.evaluate_expression_type(then_branch);
                let else_type = self.evaluate_expression_type(else_branch);
                if !then_type
                    .is_assignable_to(&else_type, &|src, tgt| self.check_is_subtype(src, tgt))
                    && !else_type
                        .is_assignable_to(&then_type, &|src, tgt| self.check_is_subtype(src, tgt))
                    && then_type != Type::Error
                    && else_type != Type::Error
                {
                    self.errors.push(BindingError::TypeMismatch {
                        expected: then_type.to_string(),
                        found: format!("Ternary branches must have the same type, got '{then_type}' and '{else_type}'"),
                        span: span.to_source_span(),
                    });
                }
                then_type
            }
            Expression::Sizeof { target_type, .. } => {
                let resolved = resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    target_type,
                    self.symbol_table,
                );
                if matches!(resolved, Type::Error | Type::Unresolved(_)) {
                    let type_name = if target_type.span().end as usize <= self.source_text.len() {
                        &self.source_text
                            [target_type.span().start as usize..target_type.span().end as usize]
                    } else {
                        "unknown"
                    };
                    let local_sym = self.symbol_table.resolve_symbol(self.source_id, type_name);
                    if let Some(sym_id) = local_sym
                        && let Some(sym_entry) = self.symbol_table.get_symbol(sym_id)
                    {
                        self.node_types
                            .insert(target_type.span(), sym_entry.symbol_type.clone());
                    } else {
                        self.errors.push(BindingError::UndefinedSymbol {
                            symbol_name: type_name.to_string(),
                            span: target_type.span().to_source_span(),
                        });
                    }
                } else {
                    self.node_types.insert(target_type.span(), resolved);
                }
                Type::Primitive(PrimitiveKind::Int32)
            }
            Expression::Default { target_type, .. } => resolve_type_annotation(
                self.source_id,
                self.source_text,
                target_type,
                self.symbol_table,
            ),
            Expression::Lambda {
                captures,
                parameters,
                body,
                ..
            } => {
                self.validate_lambda_captures(captures);
                self.evaluate_lambda_with_expected(parameters, body, None)
            }
            Expression::Assignment { target, value, .. } => {
                let target_type = self.evaluate_expression_type(target);

                // Check const assignment
                match &**target {
                    Expression::Identifier { span: id_span } => {
                        let id_name = if id_span.end as usize <= self.source_text.len() {
                            &self.source_text[id_span.start as usize..id_span.end as usize]
                        } else {
                            ""
                        };
                        if let Some(&sym_id) = self.resolved_symbols.get(id_span)
                            && let Some(sym_entry) = self.symbol_table.get_symbol(sym_id)
                            && sym_entry.is_const
                        {
                            self.errors.push(BindingError::CannotAssignToConst {
                                variable_name: id_name.to_string(),
                                span: id_span.to_source_span(),
                            });
                        }
                    }
                    Expression::MemberAccess { member_span, .. } => {
                        let member_name = if member_span.end as usize <= self.source_text.len() {
                            &self.source_text[member_span.start as usize..member_span.end as usize]
                        } else {
                            ""
                        };
                        if let Some(&sym_id) = self.resolved_symbols.get(member_span)
                            && let Some(sym_entry) = self.symbol_table.get_symbol(sym_id)
                            && sym_entry.is_const
                        {
                            self.errors.push(BindingError::CannotAssignToConst {
                                variable_name: member_name.to_string(),
                                span: member_span.to_source_span(),
                            });
                        }
                    }
                    _ => {}
                }

                let value_type = match &**value {
                    Expression::Lambda {
                        captures,
                        parameters,
                        body,
                        ..
                    } => {
                        self.validate_lambda_captures(captures);
                        self.evaluate_lambda_with_expected(parameters, body, Some(&target_type))
                    }
                    _ => self.evaluate_expression_with_expected(value, Some(&target_type)),
                };

                let is_compatible = value_type
                    .is_assignable_to(&target_type, &|src, tgt| self.check_is_subtype(src, tgt))
                    || is_literal_compatible_with_type(value, &target_type);

                if is_compatible {
                    self.coerce_literal_node_type(value, &target_type);
                } else if target_type != Type::Error && value_type != Type::Error {
                    self.errors.push(BindingError::TypeMismatch {
                        expected: target_type.to_string(),
                        found: value_type.to_string(),
                        span: value.span().to_source_span(),
                    });
                }
                target_type
            }
        }
    }

    fn validate_lambda_captures(&mut self, captures: &[Span]) {
        for cap_span in captures {
            let cap_name = if cap_span.end as usize <= self.source_text.len() {
                &self.source_text[cap_span.start as usize..cap_span.end as usize]
            } else {
                ""
            };
            if cap_name == "this" {
                if self.is_static_method || self.current_class.is_none() {
                    self.errors.push(BindingError::undefined_symbol(
                        "Cannot capture 'this' outside of an instance method",
                        *cap_span,
                    ));
                }
            } else {
                let found = self
                    .symbol_table
                    .resolve_symbol(self.source_id, cap_name)
                    .is_some();
                if !found {
                    self.errors.push(BindingError::undefined_symbol(
                        format!("Undefined capture variable: {cap_name}"),
                        *cap_span,
                    ));
                }
            }
        }
    }

    /// Evaluates the semantic function pointer type of a lambda expression, optionally propagating expected parameter and return types.
    pub fn evaluate_lambda_with_expected(
        &mut self,
        parameters: &[Parameter],
        body: &LambdaBody,
        expected_type: Option<&Type>,
    ) -> Type {
        let (expected_return, expected_params) = match expected_type.map(|item| item.unwrap_alias())
        {
            Some(Type::FunctionPointer {
                return_type,
                parameter_types,
            }) => (
                Some((**return_type).clone()),
                Some(parameter_types.as_slice()),
            ),
            _ => (None, None),
        };

        self.symbol_table.enter_scope();
        let parameter_types: Vec<Type> = parameters
            .iter()
            .enumerate()
            .map(|(param_index, parameter)| {
                let param_name = if parameter.name_span.end as usize <= self.source_text.len() {
                    &self.source_text[parameter.name_span.start as usize
                        ..parameter.name_span.end as usize]
                } else {
                    "param"
                };

                let resolved_param = resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    &parameter.param_type,
                    self.symbol_table,
                );

                // If parameter was untyped in lambda syntax, its type resolves to Type::Unresolved matching param_name
                let param_type = if matches!(&resolved_param, Type::Unresolved(name) if name == param_name) {
                    if let Some(expected_list) = expected_params {
                        expected_list
                            .get(param_index)
                            .cloned()
                            .unwrap_or(Type::Primitive(PrimitiveKind::Int32))
                    } else {
                        Type::Primitive(PrimitiveKind::Int32)
                    }
                } else {
                    resolved_param
                };

                let param_symbol = SymbolEntry {
                    id: SymbolId(0),
                    name: param_name.to_string(),
                    fully_qualified_name: param_name.to_string(),
                    source_id: self.source_id,
                    span: parameter.span,
                    modifiers: Vec::new(),
                    symbol_type: param_type.clone(),
                    parent_symbol: None,
                    members: Vec::new(),
                    is_static: false,
                    is_const: false,
                    slot_index: Some(param_index as u32),
                    parameter_names: Vec::new(),
                };
                let param_id = self.symbol_table.insert_symbol(param_symbol);
                self.symbol_table.declare_local(param_name, param_id);
                param_type
            })
            .collect();

        let previous_return = self.current_method_return_type.take();
        if let Some(expected_ret) = &expected_return {
            self.current_method_return_type = Some(expected_ret.clone());
        }

        let return_type = match body {
            LambdaBody::Expression(expr) => {
                let expr_type = self.evaluate_expression_type(expr);
                if let Some(expected_ret) = &expected_return {
                    if expr_type == Type::Error || expr_type == Type::Primitive(PrimitiveKind::Void)
                    {
                        expected_ret.clone()
                    } else {
                        expr_type
                    }
                } else {
                    expr_type
                }
            }
            LambdaBody::Block(block) => {
                self.visit_block_statement(block);
                expected_return.unwrap_or(Type::Primitive(PrimitiveKind::Void))
            }
        };

        self.current_method_return_type = previous_return;
        self.symbol_table.exit_scope();

        Type::FunctionPointer {
            return_type: Box::new(return_type),
            parameter_types,
        }
    }

    fn is_subclass_or_same(&self, mut current: SymbolId, target: SymbolId) -> bool {
        if current == target {
            return true;
        }
        while let Some(entry) = self.symbol_table.get_symbol(current) {
            if let Some(parent) = entry.parent_symbol {
                if parent == target {
                    return true;
                }
                current = parent;
            } else {
                break;
            }
        }
        false
    }

    pub fn check_is_subtype(&self, src_name: &str, tgt_name: &str) -> bool {
        if src_name == tgt_name {
            return true;
        }
        let src_clean = src_name.rsplit(['.', ':']).next().unwrap_or(src_name);
        let tgt_clean = tgt_name.rsplit(['.', ':']).next().unwrap_or(tgt_name);
        if src_clean == tgt_clean {
            return true;
        }

        let src_id = match self
            .blueprint_registry
            .instantiated_classes
            .get(src_name)
            .copied()
            .or_else(|| {
                if let Some(base) = src_name.split('<').next() {
                    let base_clean = base.rsplit(['.', ':']).next().unwrap_or(base);
                    self.symbol_table
                        .resolve_symbol(self.source_id, base)
                        .or_else(|| self.symbol_table.resolve_symbol(self.source_id, base_clean))
                } else {
                    None
                }
            })
            .or_else(|| self.symbol_table.resolve_symbol(self.source_id, src_name))
            .or_else(|| self.symbol_table.resolve_symbol(self.source_id, src_clean))
        {
            Some(id) => id,
            None => return false,
        };
        let tgt_id = match self
            .symbol_table
            .resolve_symbol(self.source_id, tgt_name)
            .or_else(|| self.symbol_table.resolve_symbol(self.source_id, tgt_clean))
        {
            Some(id) => id,
            None => return false,
        };

        if self.is_subclass_or_same(src_id, tgt_id) {
            return true;
        }

        // Interface conformance check
        let mut curr_id = Some(src_id);
        while let Some(cid) = curr_id {
            if let Some(layout) = self.layouts.get(&cid)
                && (layout.itable_methods.contains_key(tgt_name)
                    || layout.itable_methods.contains_key(tgt_clean))
            {
                return true;
            }
            curr_id = self
                .symbol_table
                .get_symbol(cid)
                .and_then(|sym_entry| sym_entry.parent_symbol);
        }

        false
    }

    fn match_method_signature(
        &self,
        parameter_types: &[Type],
        return_type: &Type,
        generic_arguments: &[TypeAnnotation],
        evaluated_arg_types: &[Type],
        arguments: &[Expression],
    ) -> Option<Type> {
        if parameter_types.len() != evaluated_arg_types.len() {
            return None;
        }

        let mut generic_param_order = Vec::new();
        Self::collect_generic_params(return_type, &mut generic_param_order);
        for parameter_type in parameter_types {
            Self::collect_generic_params(parameter_type, &mut generic_param_order);
        }

        let mut generic_params_in_args = Vec::new();
        for parameter_type in parameter_types {
            Self::collect_generic_params(parameter_type, &mut generic_params_in_args);
        }

        if generic_arguments.is_empty() && generic_params_in_args.is_empty() {
            let is_matched = parameter_types
                .iter()
                .zip(evaluated_arg_types)
                .enumerate()
                .all(|(arg_index, (expected_type, actual_type))| {
                    actual_type.is_assignable_to(expected_type, &|src, tgt| {
                        self.check_is_subtype(src, tgt)
                    }) || actual_type == expected_type
                        || is_literal_compatible_with_type(&arguments[arg_index], expected_type)
                });
            if is_matched {
                Some(return_type.clone())
            } else {
                None
            }
        } else {
            let mut substitutions: HashMap<String, Type> = HashMap::new();
            if !generic_arguments.is_empty() {
                if generic_arguments.len() != generic_param_order.len() {
                    return None;
                }
                for (param_name, type_annotation) in
                    generic_param_order.iter().zip(generic_arguments)
                {
                    let resolved_type = resolve_type_annotation(
                        self.source_id,
                        self.source_text,
                        type_annotation,
                        self.symbol_table,
                    );
                    substitutions.insert(param_name.clone(), resolved_type);
                }
            } else {
                for (parameter_type, argument_type) in
                    parameter_types.iter().zip(evaluated_arg_types)
                {
                    if !Self::deduce_template_argument(
                        parameter_type,
                        argument_type,
                        &mut substitutions,
                    ) {
                        return None;
                    }
                }
                for param_name in &generic_params_in_args {
                    if !substitutions.contains_key(param_name) {
                        return None;
                    }
                }
            }

            let concrete_params: Vec<Type> = parameter_types
                .iter()
                .map(|param_type| Self::substitute_type(param_type, &substitutions))
                .collect();
            let concrete_return = Self::substitute_type(return_type, &substitutions);

            let is_matched = concrete_params
                .iter()
                .zip(evaluated_arg_types)
                .enumerate()
                .all(|(arg_index, (expected, actual))| {
                    actual.is_assignable_to(expected, &|src, tgt| self.check_is_subtype(src, tgt))
                        || actual == expected
                        || is_literal_compatible_with_type(&arguments[arg_index], expected)
                });
            if is_matched {
                Some(concrete_return)
            } else {
                None
            }
        }
    }

    fn collect_generic_params(target_type: &Type, generic_param_order: &mut Vec<String>) {
        match target_type {
            Type::Unresolved(name) => {
                if !generic_param_order.contains(name) {
                    generic_param_order.push(name.clone());
                }
            }
            Type::Array { element_type, .. } => {
                Self::collect_generic_params(element_type, generic_param_order);
            }
            Type::Tuple(element_types) => {
                for element_type in element_types {
                    Self::collect_generic_params(element_type, generic_param_order);
                }
            }
            Type::FunctionPointer {
                parameter_types,
                return_type,
            } => {
                Self::collect_generic_params(return_type, generic_param_order);
                for parameter_type in parameter_types {
                    Self::collect_generic_params(parameter_type, generic_param_order);
                }
            }
            Type::Class { type_arguments, .. } | Type::Interface { type_arguments, .. } => {
                for type_arg in type_arguments {
                    Self::collect_generic_params(type_arg, generic_param_order);
                }
            }
            _ => {}
        }
    }

    fn deduce_template_argument(
        pattern_type: &Type,
        concrete_type: &Type,
        substitutions: &mut HashMap<String, Type>,
    ) -> bool {
        match pattern_type {
            Type::Unresolved(param_name) => {
                if let Some(existing_type) = substitutions.get(param_name) {
                    existing_type == concrete_type
                } else {
                    substitutions.insert(param_name.clone(), concrete_type.clone());
                    true
                }
            }
            Type::Array {
                element_type: pattern_element,
                depth: pattern_depth,
            } => {
                if let Type::Array {
                    element_type: concrete_element,
                    depth: concrete_depth,
                } = concrete_type
                {
                    if pattern_depth != concrete_depth {
                        return false;
                    }
                    Self::deduce_template_argument(pattern_element, concrete_element, substitutions)
                } else {
                    false
                }
            }
            Type::Tuple(pattern_elements) => {
                if let Type::Tuple(concrete_elements) = concrete_type {
                    if pattern_elements.len() != concrete_elements.len() {
                        return false;
                    }
                    for (pattern_element, concrete_element) in
                        pattern_elements.iter().zip(concrete_elements)
                    {
                        if !Self::deduce_template_argument(
                            pattern_element,
                            concrete_element,
                            substitutions,
                        ) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Type::Class {
                symbol_name: pattern_symbol_name,
                type_arguments: pattern_type_arguments,
            } => {
                if let Type::Class {
                    symbol_name: concrete_symbol_name,
                    type_arguments: concrete_type_arguments,
                } = concrete_type
                {
                    if pattern_symbol_name.replace('.', "::")
                        != concrete_symbol_name.replace('.', "::")
                        || pattern_type_arguments.len() != concrete_type_arguments.len()
                    {
                        return false;
                    }
                    for (pattern_argument, concrete_argument) in
                        pattern_type_arguments.iter().zip(concrete_type_arguments)
                    {
                        if !Self::deduce_template_argument(
                            pattern_argument,
                            concrete_argument,
                            substitutions,
                        ) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Type::Interface {
                symbol_name: pattern_symbol_name,
                type_arguments: pattern_type_arguments,
            } => {
                if let Type::Interface {
                    symbol_name: concrete_symbol_name,
                    type_arguments: concrete_type_arguments,
                } = concrete_type
                {
                    if pattern_symbol_name.replace('.', "::")
                        != concrete_symbol_name.replace('.', "::")
                        || pattern_type_arguments.len() != concrete_type_arguments.len()
                    {
                        return false;
                    }
                    for (pattern_argument, concrete_argument) in
                        pattern_type_arguments.iter().zip(concrete_type_arguments)
                    {
                        if !Self::deduce_template_argument(
                            pattern_argument,
                            concrete_argument,
                            substitutions,
                        ) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Type::FunctionPointer {
                parameter_types: pattern_parameters,
                return_type: pattern_return,
            } => {
                if let Type::FunctionPointer {
                    parameter_types: concrete_parameters,
                    return_type: concrete_return,
                } = concrete_type
                {
                    if pattern_parameters.len() != concrete_parameters.len() {
                        return false;
                    }
                    if !Self::deduce_template_argument(
                        pattern_return,
                        concrete_return,
                        substitutions,
                    ) {
                        return false;
                    }
                    for (pattern_param, concrete_param) in
                        pattern_parameters.iter().zip(concrete_parameters)
                    {
                        if !Self::deduce_template_argument(
                            pattern_param,
                            concrete_param,
                            substitutions,
                        ) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            _ => pattern_type == concrete_type,
        }
    }

    fn substitute_type(target_type: &Type, substitutions: &HashMap<String, Type>) -> Type {
        match target_type {
            Type::Unresolved(name) => substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| target_type.clone()),
            Type::Array {
                element_type,
                depth,
            } => Type::Array {
                element_type: Box::new(Self::substitute_type(element_type, substitutions)),
                depth: *depth,
            },
            Type::Tuple(element_types) => Type::Tuple(
                element_types
                    .iter()
                    .map(|elem| Self::substitute_type(elem, substitutions))
                    .collect(),
            ),
            Type::FunctionPointer {
                parameter_types,
                return_type,
            } => Type::FunctionPointer {
                parameter_types: parameter_types
                    .iter()
                    .map(|param| Self::substitute_type(param, substitutions))
                    .collect(),
                return_type: Box::new(Self::substitute_type(return_type, substitutions)),
            },
            Type::Class {
                symbol_name,
                type_arguments,
            } => Type::Class {
                symbol_name: symbol_name.clone(),
                type_arguments: type_arguments
                    .iter()
                    .map(|arg| Self::substitute_type(arg, substitutions))
                    .collect(),
            },
            Type::Interface {
                symbol_name,
                type_arguments,
            } => Type::Interface {
                symbol_name: symbol_name.clone(),
                type_arguments: type_arguments
                    .iter()
                    .map(|arg| Self::substitute_type(arg, substitutions))
                    .collect(),
            },
            _ => target_type.clone(),
        }
    }
}

impl<'ctx> AstVisitor for TypeCheckingVisitor<'ctx> {
    fn visit_class_declaration(&mut self, node: &ClassDeclaration) {
        let class_name = if node.name_span.end as usize <= self.source_text.len() {
            &self.source_text[node.name_span.start as usize..node.name_span.end as usize]
        } else {
            "class"
        };

        let file_package = self
            .symbol_table
            .file_packages
            .get(self.source_id as usize)
            .cloned()
            .unwrap_or_default();
        let decl_namespace = extract_namespace_path(&node.namespace_path, self.source_text);
        let effective_package = compute_effective_package(&file_package, &decl_namespace);
        let fully_qualified_name = if effective_package.is_empty() {
            class_name.to_string()
        } else {
            format!("{effective_package}::{class_name}")
        };

        let previous_active_package = self.symbol_table.active_package.clone();
        if !effective_package.is_empty() {
            self.symbol_table.active_package = Some(effective_package);
        }

        let previous_class = self.current_class;
        let previous_generic_names = self.current_class_generic_names.clone();
        self.current_class = self
            .symbol_table
            .lookup_global(&fully_qualified_name)
            .or_else(|| self.symbol_table.resolve_symbol(self.source_id, class_name));
        self.current_class_generic_names = node
            .generic_params
            .iter()
            .filter_map(|param| {
                let span = param.name_span;
                if span.end as usize <= self.source_text.len() {
                    Some(self.source_text[span.start as usize..span.end as usize].to_string())
                } else {
                    None
                }
            })
            .collect();

        for member in &node.members {
            match member {
                ClassMember::Field(field) => self.visit_field_declaration(field),
                ClassMember::Method(method) => self.visit_method_declaration(method),
                ClassMember::Constructor(constructor) => {
                    self.visit_constructor_declaration(constructor)
                }
                ClassMember::Destructor(destructor) => {
                    self.visit_destructor_declaration(destructor)
                }
                ClassMember::Operator(operator) => self.visit_operator_declaration(operator),
                ClassMember::Enum(enum_decl) => self.visit_enum_declaration(enum_decl),
                ClassMember::Class(class_decl) => self.visit_class_declaration(class_decl),
                ClassMember::Interface(iface_decl) => self.visit_interface_declaration(iface_decl),
            }
        }

        self.symbol_table.active_package = previous_active_package;
        self.current_class = previous_class;
        self.current_class_generic_names = previous_generic_names;
    }

    fn visit_interface_declaration(&mut self, node: &InterfaceDeclaration) {
        let file_package = self
            .symbol_table
            .file_packages
            .get(self.source_id as usize)
            .cloned()
            .unwrap_or_default();
        let decl_namespace = extract_namespace_path(&node.namespace_path, self.source_text);
        let effective_package = compute_effective_package(&file_package, &decl_namespace);
        let previous_active_package = self.symbol_table.active_package.clone();
        if !effective_package.is_empty() {
            self.symbol_table.active_package = Some(effective_package);
        }

        let previous_generic_names = self.current_class_generic_names.clone();
        self.current_class_generic_names = node
            .generic_params
            .iter()
            .filter_map(|param| {
                let span = param.name_span;
                if span.end as usize <= self.source_text.len() {
                    Some(self.source_text[span.start as usize..span.end as usize].to_string())
                } else {
                    None
                }
            })
            .collect();

        for method in &node.methods {
            self.visit_method_declaration(method);
        }

        self.symbol_table.active_package = previous_active_package;
        self.current_class_generic_names = previous_generic_names;
    }

    fn visit_field_declaration(&mut self, node: &FieldDeclaration) {
        let field_name = if node.name_span.end as usize <= self.source_text.len() {
            &self.source_text[node.name_span.start as usize..node.name_span.end as usize]
        } else {
            "field"
        };
        let field_type = resolve_type_annotation(
            self.source_id,
            self.source_text,
            &node.field_type,
            self.symbol_table,
        );
        if let Some(initializer) = &node.initializer {
            let init_type = self.evaluate_expression_with_expected(initializer, Some(&field_type));
            let is_compatible = init_type
                .is_assignable_to(&field_type, &|src, tgt| self.check_is_subtype(src, tgt))
                || is_literal_compatible_with_type(initializer, &field_type);
            if is_compatible {
                self.coerce_literal_node_type(initializer, &field_type);
            } else if field_type != Type::Error && init_type != Type::Error {
                self.errors
                    .push(BindingError::FieldInitializerTypeMismatch {
                        field_name: field_name.to_string(),
                        expected: field_type.to_string(),
                        found: init_type.to_string(),
                        span: initializer.span().to_source_span(),
                    });
            }
        }
    }

    fn visit_method_declaration(&mut self, node: &MethodDeclaration) {
        let method_generic_names: std::collections::HashSet<String> = node
            .generic_params
            .iter()
            .filter_map(|param| {
                let span = param.name_span;
                if span.end as usize <= self.source_text.len() {
                    Some(self.source_text[span.start as usize..span.end as usize].to_string())
                } else {
                    None
                }
            })
            .collect();

        let previous_method_generic_names = std::mem::replace(
            &mut self.current_method_generic_names,
            method_generic_names.clone(),
        );

        let return_type = resolve_type_annotation(
            self.source_id,
            self.source_text,
            &node.return_type,
            self.symbol_table,
        );

        if let Some(unresolved_name) = return_type.first_unresolved_name()
            && !self.current_class_generic_names.contains(unresolved_name)
            && !method_generic_names.contains(unresolved_name)
        {
            self.errors.push(BindingError::UnknownTypeInDeclaration {
                type_name: unresolved_name.to_string(),
                span: node.return_type.span().to_source_span(),
            });
        }

        let previous_return = self.current_method_return_type.take();
        self.current_method_return_type = Some(return_type.clone());
        let previous_static = self.is_static_method;
        self.is_static_method = node.modifiers.contains(&Modifier::Static);

        self.symbol_table.enter_scope();

        for (slot_index, parameter) in node.parameters.iter().enumerate() {
            let param_name = if parameter.name_span.end as usize <= self.source_text.len() {
                &self.source_text
                    [parameter.name_span.start as usize..parameter.name_span.end as usize]
            } else {
                "arg"
            };

            let param_type = if parameter.is_variadic {
                Type::Primitive(PrimitiveKind::TuplePrimitive)
            } else {
                resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    &parameter.param_type,
                    self.symbol_table,
                )
            };

            if param_type == Type::Primitive(PrimitiveKind::Void) {
                self.errors.push(BindingError::VoidTypeNotAllowedHere {
                    span: parameter.span.to_source_span(),
                });
            }
            if let Some(unresolved_name) = param_type.first_unresolved_name()
                && !self.current_class_generic_names.contains(unresolved_name)
                && !method_generic_names.contains(unresolved_name)
            {
                self.errors.push(BindingError::UnknownTypeInDeclaration {
                    type_name: unresolved_name.to_string(),
                    span: parameter.param_type.span().to_source_span(),
                });
            }

            if let Some(default_val) = &parameter.default_value {
                let default_type =
                    self.evaluate_expression_with_expected(default_val, Some(&param_type));
                let is_compatible = default_type
                    .is_assignable_to(&param_type, &|src, tgt| self.check_is_subtype(src, tgt))
                    || is_literal_compatible_with_type(default_val, &param_type);
                if is_compatible {
                    self.coerce_literal_node_type(default_val, &param_type);
                } else if param_type != Type::Error && default_type != Type::Error {
                    self.errors
                        .push(BindingError::ParameterDefaultValueMismatch {
                            parameter_name: param_name.to_string(),
                            expected: param_type.to_string(),
                            found: default_type.to_string(),
                            span: default_val.span().to_source_span(),
                        });
                }
            }

            let param_symbol = SymbolEntry {
                id: SymbolId(0),
                name: param_name.to_string(),
                fully_qualified_name: String::new(),
                source_id: self.source_id,
                span: parameter.span,
                modifiers: Vec::new(),
                symbol_type: param_type.clone(),
                parent_symbol: None,
                members: Vec::new(),
                is_static: false,
                is_const: false,
                slot_index: Some(slot_index as u32),
                parameter_names: Vec::new(),
            };
            if let Some(innermost_scope) = self.symbol_table.scope_stack.last()
                && innermost_scope.contains_key(param_name)
            {
                self.errors.push(BindingError::DuplicateSymbol {
                    symbol_name: format!(
                        "Parameter '{param_name}' is already defined in this parameter list"
                    ),
                    span: parameter.name_span.to_source_span(),
                });
            }
            let param_id = self.symbol_table.insert_symbol(param_symbol);
            self.symbol_table.declare_local(param_name, param_id);
            self.symbol_table.record_local_symbol(LocalSymbolEntry {
                name: param_name.to_string(),
                symbol_type: param_type.clone(),
                decl_span: parameter.name_span,
                scope_span: node.span,
                source_id: self.source_id,
                is_const: false,
                symbol_id: param_id,
            });
            self.resolved_symbols.insert(parameter.name_span, param_id);
            self.node_types.insert(parameter.name_span, param_type);
        }

        if let Some(body) = &node.body {
            for statement in &body.statements {
                self.visit_statement(statement);
            }

            if !return_type.is_void()
                && !node.modifiers.contains(&Modifier::Abstract)
                && !node.modifiers.contains(&Modifier::Native)
                && !node.modifiers.contains(&Modifier::Extern)
                && !block_definitely_returns(body)
            {
                let method_name = if node.name_span.end as usize <= self.source_text.len() {
                    &self.source_text[node.name_span.start as usize..node.name_span.end as usize]
                } else {
                    "method"
                };
                self.errors.push(BindingError::MissingReturn {
                    method_name: method_name.to_string(),
                    expected_type: format!("{return_type:?}"),
                    span: node.span.to_source_span(),
                });
            }
        }

        self.symbol_table.exit_scope();
        self.current_method_return_type = previous_return;
        self.is_static_method = previous_static;
        self.current_method_generic_names = previous_method_generic_names;
    }

    fn visit_constructor_declaration(&mut self, node: &ConstructorDeclaration) {
        self.symbol_table.enter_scope();
        for (slot_index, parameter) in node.parameters.iter().enumerate() {
            let param_name = if parameter.name_span.end as usize <= self.source_text.len() {
                &self.source_text
                    [parameter.name_span.start as usize..parameter.name_span.end as usize]
            } else {
                "arg"
            };
            let param_type = if parameter.is_variadic {
                Type::Primitive(PrimitiveKind::TuplePrimitive)
            } else {
                resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    &parameter.param_type,
                    self.symbol_table,
                )
            };
            if param_type == Type::Primitive(PrimitiveKind::Void) {
                self.errors.push(BindingError::VoidTypeNotAllowedHere {
                    span: parameter.span.to_source_span(),
                });
            }
            if let Some(unresolved_name) = param_type.first_unresolved_name()
                && !self.current_class_generic_names.contains(unresolved_name)
            {
                self.errors.push(BindingError::UnknownTypeInDeclaration {
                    type_name: unresolved_name.to_string(),
                    span: parameter.param_type.span().to_source_span(),
                });
            }
            let param_symbol = SymbolEntry {
                id: SymbolId(0),
                name: param_name.to_string(),
                fully_qualified_name: String::new(),
                source_id: self.source_id,
                span: parameter.span,
                modifiers: Vec::new(),
                symbol_type: param_type.clone(),
                parent_symbol: None,
                members: Vec::new(),
                is_static: false,
                is_const: false,
                slot_index: Some(slot_index as u32),
                parameter_names: Vec::new(),
            };
            if let Some(innermost_scope) = self.symbol_table.scope_stack.last()
                && innermost_scope.contains_key(param_name)
            {
                self.errors.push(BindingError::DuplicateSymbol {
                    symbol_name: format!(
                        "Parameter '{param_name}' is already defined in this parameter list"
                    ),
                    span: parameter.name_span.to_source_span(),
                });
            }
            let param_id = self.symbol_table.insert_symbol(param_symbol);
            self.symbol_table.declare_local(param_name, param_id);
            self.symbol_table.record_local_symbol(LocalSymbolEntry {
                name: param_name.to_string(),
                symbol_type: param_type.clone(),
                decl_span: parameter.name_span,
                scope_span: node.span,
                source_id: self.source_id,
                is_const: false,
                symbol_id: param_id,
            });
            self.resolved_symbols.insert(parameter.name_span, param_id);
            self.node_types.insert(parameter.name_span, param_type);
        }

        if let Some(super_args) = &node.super_call_args {
            let parent_class_id = self
                .current_class
                .and_then(|class_id| self.symbol_table.get_symbol(class_id))
                .and_then(|class_entry| class_entry.parent_symbol);

            if let Some(base_class_id) = parent_class_id {
                let evaluated_super_arg_types: Vec<Type> = super_args
                    .iter()
                    .map(|arg| self.evaluate_expression_type(arg))
                    .collect();

                let mut matching_ctor = false;
                let mut expected_counts = Vec::new();

                if let Some(base_entry) = self.symbol_table.get_symbol(base_class_id) {
                    for &member_id in &base_entry.members {
                        if let Some(member_entry) = self.symbol_table.get_symbol(member_id)
                            && member_entry.name == "constructor"
                            && let Type::FunctionPointer {
                                ref parameter_types,
                                ..
                            } = member_entry.symbol_type
                        {
                            expected_counts.push(parameter_types.len());
                            if parameter_types.len() == evaluated_super_arg_types.len() {
                                let types_match = parameter_types
                                    .iter()
                                    .zip(super_args.iter())
                                    .zip(&evaluated_super_arg_types)
                                    .all(|((expected_param, argument_expr), actual_arg)| {
                                        actual_arg.is_assignable_to(expected_param, &|src, tgt| {
                                            self.check_is_subtype(src, tgt)
                                        }) || actual_arg == expected_param
                                            || is_literal_compatible_with_type(
                                                argument_expr,
                                                expected_param,
                                            )
                                    });
                                if types_match {
                                    for (expected_param, argument_expr) in
                                        parameter_types.iter().zip(super_args.iter())
                                    {
                                        if is_literal_compatible_with_type(
                                            argument_expr,
                                            expected_param,
                                        ) {
                                            coerce_literal_node_type(
                                                self.node_types,
                                                argument_expr,
                                                expected_param,
                                            );
                                        }
                                    }
                                    matching_ctor = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !matching_ctor {
                    let expected_count = expected_counts.first().copied().unwrap_or(0);
                    self.errors
                        .push(BindingError::SuperConstructorArgumentMismatch {
                            expected: expected_count,
                            found: evaluated_super_arg_types.len(),
                            span: node.span.to_source_span(),
                        });
                }
            } else {
                self.errors.push(BindingError::InvalidSuperCall {
                    span: node.span.to_source_span(),
                });
            }
        }

        for statement in &node.body.statements {
            self.visit_statement(statement);
        }
        self.symbol_table.exit_scope();
    }

    fn visit_destructor_declaration(&mut self, node: &DestructorDeclaration) {
        let previous_return = self.current_method_return_type.take();
        self.current_method_return_type = Some(Type::Primitive(PrimitiveKind::Void));
        let previous_static = self.is_static_method;
        self.is_static_method = false;
        let previous_destructor = self.is_inside_destructor;
        self.is_inside_destructor = true;

        self.symbol_table.enter_scope();
        for statement in &node.body.statements {
            self.visit_statement(statement);
        }
        self.symbol_table.exit_scope();

        self.is_inside_destructor = previous_destructor;
        self.current_method_return_type = previous_return;
        self.is_static_method = previous_static;
    }

    fn visit_operator_declaration(&mut self, node: &OperatorDeclaration) {
        let return_type = resolve_type_annotation(
            self.source_id,
            self.source_text,
            &node.return_type,
            self.symbol_table,
        );

        let previous_return = self.current_method_return_type.take();
        self.current_method_return_type = Some(return_type.clone());
        let previous_static = self.is_static_method;
        self.is_static_method = node.modifiers.contains(&Modifier::Static);

        self.symbol_table.enter_scope();

        for (slot_index, parameter) in node.parameters.iter().enumerate() {
            let param_name = if parameter.name_span.end as usize <= self.source_text.len() {
                &self.source_text
                    [parameter.name_span.start as usize..parameter.name_span.end as usize]
            } else {
                "param"
            };

            let param_type = if parameter.is_variadic {
                Type::Primitive(PrimitiveKind::TuplePrimitive)
            } else {
                resolve_type_annotation(
                    self.source_id,
                    self.source_text,
                    &parameter.param_type,
                    self.symbol_table,
                )
            };

            let param_symbol = SymbolEntry {
                id: SymbolId(0),
                name: param_name.to_string(),
                fully_qualified_name: String::new(),
                source_id: self.source_id,
                span: parameter.span,
                modifiers: Vec::new(),
                symbol_type: param_type.clone(),
                parent_symbol: self.current_class,
                members: Vec::new(),
                is_static: false,
                is_const: false,
                slot_index: Some(slot_index as u32),
                parameter_names: Vec::new(),
            };
            if let Some(innermost_scope) = self.symbol_table.scope_stack.last()
                && innermost_scope.contains_key(param_name)
            {
                self.errors.push(BindingError::DuplicateSymbol {
                    symbol_name: format!(
                        "Parameter '{param_name}' is already defined in this parameter list"
                    ),
                    span: parameter.name_span.to_source_span(),
                });
            }
            let param_id = self.symbol_table.insert_symbol(param_symbol);
            self.symbol_table.declare_local(param_name, param_id);
            self.symbol_table.record_local_symbol(LocalSymbolEntry {
                name: param_name.to_string(),
                symbol_type: param_type.clone(),
                decl_span: parameter.name_span,
                scope_span: node.span,
                source_id: self.source_id,
                is_const: false,
                symbol_id: param_id,
            });
            self.resolved_symbols.insert(parameter.name_span, param_id);
            self.node_types.insert(parameter.name_span, param_type);
        }

        for statement in &node.body.statements {
            self.visit_statement(statement);
        }

        self.symbol_table.exit_scope();
        self.current_method_return_type = previous_return;
        self.is_static_method = previous_static;
    }

    fn visit_block_statement(&mut self, node: &BlockStatement) {
        self.symbol_table.enter_scope();
        for statement in &node.statements {
            self.visit_statement(statement);
        }
        self.symbol_table.exit_scope();
    }

    fn visit_variable_declaration(&mut self, node: &VariableDeclarationStatement) {
        let var_name = if node.name_span.end as usize <= self.source_text.len() {
            &self.source_text[node.name_span.start as usize..node.name_span.end as usize]
        } else {
            "var"
        };

        let mut declared_type = resolve_type_annotation(
            self.source_id,
            self.source_text,
            &node.var_type,
            self.symbol_table,
        );

        if let Type::Class {
            ref symbol_name, ..
        } = declared_type
            && let Some(class_id) = self
                .symbol_table
                .resolve_symbol(self.source_id, symbol_name)
        {
            self.resolved_symbols.insert(node.var_type.span(), class_id);
        }

        if let TypeAnnotation::Named { path, .. } = &node.var_type {
            let segments: Vec<&str> = path
                .iter()
                .map(|span| {
                    let start = span.start as usize;
                    let end = span.end as usize;
                    if end <= self.source_text.len() {
                        &self.source_text[start..end]
                    } else {
                        ""
                    }
                })
                .collect();
            let name = segments.join(".");
            let wildcard_matches = self
                .symbol_table
                .find_wildcard_matches(self.source_id, &name);
            if wildcard_matches.len() > 1 {
                self.errors.push(BindingError::AmbiguousImport {
                    symbol_name: name,
                    candidates: wildcard_matches,
                    span: node.var_type.span().to_source_span(),
                });
                declared_type = Type::Error;
            }
        }

        if let Some(unresolved_name) = declared_type.first_unresolved_name()
            && !self.current_class_generic_names.contains(unresolved_name)
            && !self.current_method_generic_names.contains(unresolved_name)
        {
            self.errors.push(BindingError::undefined_symbol(
                unresolved_name,
                node.var_type.span(),
            ));
            declared_type = Type::Error;
        }

        if declared_type == Type::Primitive(PrimitiveKind::Void) {
            self.errors.push(BindingError::VoidTypeNotAllowedHere {
                span: node.var_type.span().to_source_span(),
            });
            return;
        }

        if let Some(innermost_scope) = self.symbol_table.scope_stack.last()
            && innermost_scope.contains_key(var_name)
        {
            self.errors.push(BindingError::DuplicateSymbol {
                symbol_name: format!("Variable '{var_name}' is already defined in this scope"),
                span: node.name_span.to_source_span(),
            });
            return;
        }

        if let Type::Class {
            ref symbol_name,
            ref type_arguments,
        } = declared_type
            && !type_arguments.is_empty()
            && let Err(errors) = self.blueprint_registry.monomorphize_class(
                symbol_name,
                type_arguments,
                self.symbol_table,
                self.layouts,
                self.source_texts,
                self.next_vtable_id,
            )
        {
            self.errors.extend(errors);
        }

        if let Some(initializer) = &node.initializer {
            let initializer_type = match initializer {
                Expression::Lambda {
                    captures,
                    parameters,
                    body,
                    ..
                } => {
                    self.validate_lambda_captures(captures);
                    self.evaluate_lambda_with_expected(parameters, body, Some(&declared_type))
                }
                _ => self.evaluate_expression_with_expected(initializer, Some(&declared_type)),
            };

            let is_compatible = initializer_type
                .is_assignable_to(&declared_type, &|src, tgt| self.check_is_subtype(src, tgt))
                || is_literal_compatible_with_type(initializer, &declared_type);

            if is_compatible {
                self.coerce_literal_node_type(initializer, &declared_type);
            } else if declared_type != Type::Error && initializer_type != Type::Error {
                self.errors.push(BindingError::TypeMismatch {
                    expected: declared_type.to_string(),
                    found: initializer_type.to_string(),
                    span: initializer.span().to_source_span(),
                });
            }
        }

        let is_const = node.is_const;
        let var_symbol = SymbolEntry {
            id: SymbolId(0),
            name: var_name.to_string(),
            fully_qualified_name: String::new(),
            source_id: self.source_id,
            span: node.span,
            modifiers: if is_const {
                vec![Modifier::Const]
            } else {
                Vec::new()
            },
            symbol_type: declared_type.clone(),
            parent_symbol: None,
            members: Vec::new(),
            is_static: false,
            is_const,
            slot_index: None,
            parameter_names: Vec::new(),
        };
        let var_id = self.symbol_table.insert_symbol(var_symbol);
        self.symbol_table.declare_local(var_name, var_id);
        self.symbol_table.record_local_symbol(LocalSymbolEntry {
            name: var_name.to_string(),
            symbol_type: declared_type.clone(),
            decl_span: node.name_span,
            scope_span: node.span,
            source_id: self.source_id,
            is_const,
            symbol_id: var_id,
        });
        self.resolved_symbols.insert(node.name_span, var_id);
        self.node_types
            .insert(node.name_span, declared_type.clone());
        self.node_types.insert(node.span, declared_type);
    }

    fn visit_return_statement(&mut self, node: &ReturnStatement) {
        if self.is_inside_destructor {
            if node.value.is_some() {
                self.errors.push(BindingError::ReturnOutsideFunction {
                    span: node.span.to_source_span(),
                });
            }
            return;
        }

        let expected_return_opt = self.current_method_return_type.clone();

        let returned_type = if let Some(expr) = &node.value {
            self.evaluate_expression_with_expected(expr, expected_return_opt.as_ref())
        } else {
            Type::Primitive(PrimitiveKind::Void)
        };

        if let Some(expected_return) = &expected_return_opt {
            let is_compatible = returned_type
                .is_assignable_to(expected_return, &|src, tgt| self.check_is_subtype(src, tgt))
                || node
                    .value
                    .as_ref()
                    .is_some_and(|expr| is_literal_compatible_with_type(expr, expected_return));

            if is_compatible {
                if let Some(expr) = &node.value {
                    coerce_literal_node_type(self.node_types, expr, expected_return);
                }
            } else if *expected_return != Type::Error && returned_type != Type::Error {
                self.errors.push(BindingError::ReturnTypeMismatch {
                    expected: expected_return.to_string(),
                    found: returned_type.to_string(),
                    span: node.span.to_source_span(),
                });
            }
        } else {
            self.errors.push(BindingError::ReturnOutsideFunction {
                span: node.span.to_source_span(),
            });
        }
    }

    fn visit_if_statement(&mut self, node: &IfStatement) {
        let cond_type = self.evaluate_expression_type(&node.condition);
        if cond_type != Type::Primitive(PrimitiveKind::Bool) {
            self.errors.push(BindingError::TypeMismatch {
                expected: "bool".to_string(),
                found: cond_type.to_string(),
                span: node.condition.span().to_source_span(),
            });
        }
        self.visit_statement(&node.then_branch);
        if let Some(else_branch) = &node.else_branch {
            self.visit_statement(else_branch);
        }
    }

    fn visit_while_statement(&mut self, node: &WhileStatement) {
        let cond_type = self.evaluate_expression_type(&node.condition);
        if cond_type != Type::Primitive(PrimitiveKind::Bool) {
            self.errors.push(BindingError::TypeMismatch {
                expected: "bool".to_string(),
                found: cond_type.to_string(),
                span: node.condition.span().to_source_span(),
            });
        }
        self.loop_depth += 1;
        self.visit_statement(&node.body);
        self.loop_depth -= 1;
    }

    fn visit_do_while_statement(&mut self, node: &DoWhileStatement) {
        self.loop_depth += 1;
        self.visit_statement(&node.body);
        self.loop_depth -= 1;
        let cond_type = self.evaluate_expression_type(&node.condition);
        if cond_type != Type::Primitive(PrimitiveKind::Bool) {
            self.errors.push(BindingError::TypeMismatch {
                expected: "bool".to_string(),
                found: cond_type.to_string(),
                span: node.condition.span().to_source_span(),
            });
        }
    }

    fn visit_for_statement(&mut self, node: &ForStatement) {
        self.symbol_table.enter_scope();
        if let Some(init) = &node.initializer {
            self.visit_statement(init);
        }
        if let Some(cond) = &node.condition {
            let cond_type = self.evaluate_expression_type(cond);
            if cond_type != Type::Primitive(PrimitiveKind::Bool) {
                self.errors.push(BindingError::TypeMismatch {
                    expected: "bool".to_string(),
                    found: cond_type.to_string(),
                    span: cond.span().to_source_span(),
                });
            }
        }
        if let Some(update) = &node.update {
            self.evaluate_expression_type(update);
        }
        self.loop_depth += 1;
        self.visit_statement(&node.body);
        self.loop_depth -= 1;
        self.symbol_table.exit_scope();
    }

    fn visit_switch_statement(&mut self, node: &SwitchStatement) {
        let discriminant_type = self.evaluate_expression_type(&node.discriminant);
        self.switch_depth += 1;
        let mut seen_cases = std::collections::HashSet::new();

        for case_clause in &node.cases {
            let case_source_span = case_clause.case_value.span().to_source_span();
            let case_span = case_clause.case_value.span();

            // 1. Check for duplicate case constant representation
            let case_repr = if (case_span.end as usize) <= self.source_text.len() {
                self.source_text[case_span.start as usize..case_span.end as usize]
                    .trim()
                    .to_string()
            } else {
                match &case_clause.case_value {
                    Expression::Literal { value, .. } => format!("{value:?}"),
                    _ => String::new(),
                }
            };

            if !case_repr.is_empty() && !seen_cases.insert(case_repr.clone()) {
                self.errors.push(BindingError::DuplicateSwitchCaseConstant {
                    case_value: case_repr,
                    span: case_source_span,
                });
            }

            // 2. Pattern and Type Checking
            match &case_clause.case_value {
                Expression::Instanceof { target_type, .. } => {
                    let resolved_target = resolve_type_annotation(
                        self.source_id,
                        self.source_text,
                        target_type,
                        self.symbol_table,
                    );
                    let is_valid_pattern = matches!(
                        resolved_target,
                        Type::Class { .. } | Type::Interface { .. } | Type::Tuple(..)
                    );
                    if !is_valid_pattern {
                        let found_description = resolved_target.to_string();
                        self.errors.push(BindingError::InvalidSwitchPatternTarget {
                            found: found_description,
                            span: target_type.span().to_source_span(),
                        });
                    }
                }
                Expression::TupleCreation { elements, .. } => {
                    if let Type::Tuple(ref disc_elements) = discriminant_type {
                        let case_arity = elements.len();
                        let disc_arity = disc_elements.len();
                        if case_arity != disc_arity {
                            self.errors.push(BindingError::TupleArityMismatchInCase {
                                case_arity,
                                disc_arity,
                                span: case_source_span,
                            });
                        } else {
                            for (element_index, element_expr) in elements.iter().enumerate() {
                                let element_type = self.evaluate_expression_type(element_expr);
                                let expected_type = &disc_elements[element_index];
                                let is_compatible = element_type
                                    .is_assignable_to(expected_type, &|source, target| {
                                        self.check_is_subtype(source, target)
                                    })
                                    || expected_type
                                        .is_assignable_to(&element_type, &|source, target| {
                                            self.check_is_subtype(source, target)
                                        })
                                    || (element_type.is_integer() && expected_type.is_integer());

                                if !is_compatible {
                                    self.errors.push(BindingError::IncompatibleSwitchCaseType {
                                        case_type: element_type.to_string(),
                                        discriminant_type: expected_type.to_string(),
                                        span: element_expr.span().to_source_span(),
                                    });
                                }
                            }
                        }
                    } else {
                        self.errors.push(BindingError::IncompatibleSwitchCaseType {
                            case_type: format!("tuple<{}>", elements.len()),
                            discriminant_type: discriminant_type.to_string(),
                            span: case_source_span,
                        });
                    }
                }
                Expression::Literal {
                    value: LiteralValue::Null,
                    ..
                } => {
                    if !discriminant_type.is_reference_type()
                        && !matches!(
                            discriminant_type,
                            Type::Class { .. }
                                | Type::Interface { .. }
                                | Type::Tuple(_)
                                | Type::Array { .. }
                        )
                    {
                        self.errors.push(BindingError::IncompatibleSwitchCaseType {
                            case_type: "null".to_string(),
                            discriminant_type: discriminant_type.to_string(),
                            span: case_source_span,
                        });
                    }
                }
                _ => {
                    if (discriminant_type.is_integer()
                        || discriminant_type.is_floating_point()
                        || discriminant_type == Type::Primitive(PrimitiveKind::Bool))
                        && matches!(&case_clause.case_value, Expression::Identifier { .. })
                    {
                        self.errors.push(BindingError::NonConstantCaseValue {
                            span: case_source_span,
                        });
                    }

                    let case_type = self.evaluate_expression_type(&case_clause.case_value);
                    if let Type::Class {
                        ref symbol_name, ..
                    } = discriminant_type
                    {
                        let is_string = symbol_name == "std::string"
                            || symbol_name == "std.string"
                            || symbol_name == "string";
                        if !is_string && !self.class_implements_equality_operator(symbol_name) {
                            self.errors
                                .push(BindingError::SwitchClassMissingEqualityOperator {
                                    class_name: symbol_name.clone(),
                                    span: case_source_span,
                                });
                        }
                    }

                    let are_both_integers =
                        case_type.is_integer() && discriminant_type.is_integer();
                    let are_both_floats =
                        case_type.is_floating_point() && discriminant_type.is_floating_point();
                    let is_assignable =
                        case_type.is_assignable_to(&discriminant_type, &|source, target| {
                            self.check_is_subtype(source, target)
                        }) || discriminant_type.is_assignable_to(&case_type, &|source, target| {
                            self.check_is_subtype(source, target)
                        });

                    if !are_both_integers
                        && !are_both_floats
                        && !is_assignable
                        && case_type != Type::Error
                        && discriminant_type != Type::Error
                    {
                        self.errors.push(BindingError::IncompatibleSwitchCaseType {
                            case_type: case_type.to_string(),
                            discriminant_type: discriminant_type.to_string(),
                            span: case_source_span,
                        });
                    }
                }
            }

            for statement in &case_clause.statements {
                self.visit_statement(statement);
            }
        }

        if let Some(default_statements) = &node.default_case {
            for statement in default_statements {
                self.visit_statement(statement);
            }
        }
        self.switch_depth -= 1;
    }

    fn visit_case_clause(&mut self, node: &CaseClause) {
        self.evaluate_expression_type(&node.case_value);
        for statement in &node.statements {
            self.visit_statement(statement);
        }
    }

    fn visit_try_statement(&mut self, node: &TryStatement) {
        self.visit_block_statement(&node.try_block);

        let mut caught_symbols: Vec<SymbolId> = Vec::new();
        for catch_clause in &node.catch_clauses {
            let catch_type = resolve_type_annotation(
                self.source_id,
                self.source_text,
                &catch_clause.exception_type,
                self.symbol_table,
            );

            let catch_symbol_id = match &catch_type {
                Type::Class { symbol_name, .. } => self
                    .symbol_table
                    .resolve_symbol(self.source_id, symbol_name),
                _ => None,
            };

            if catch_symbol_id.is_none() || !matches!(catch_type, Type::Class { .. }) {
                self.errors.push(BindingError::TypeMismatch {
                    expected: "catch clause exception class type".to_string(),
                    found: catch_type.to_string(),
                    span: catch_clause.exception_type.span().to_source_span(),
                });
            } else if let Some(sym_id) = catch_symbol_id {
                if caught_symbols.contains(&sym_id) {
                    self.errors.push(BindingError::DuplicateSymbol {
                        symbol_name: format!("duplicate catch clause for type '{}'", catch_type),
                        span: catch_clause.exception_type.span().to_source_span(),
                    });
                } else if caught_symbols
                    .iter()
                    .any(|&prev| self.is_subclass_or_same(sym_id, prev))
                {
                    self.errors.push(BindingError::TypeMismatch {
                        expected: "unreachable catch clause: superclass already caught".to_string(),
                        found: catch_type.to_string(),
                        span: catch_clause.exception_type.span().to_source_span(),
                    });
                }
                caught_symbols.push(sym_id);
            }

            self.visit_catch_clause(catch_clause);
        }

        if let Some(finally_block) = &node.finally_block {
            self.visit_block_statement(finally_block);
        }
    }

    fn visit_catch_clause(&mut self, node: &CatchClause) {
        let exception_name = if node.variable_name.end as usize <= self.source_text.len() {
            &self.source_text[node.variable_name.start as usize..node.variable_name.end as usize]
        } else {
            "ex"
        };
        let exception_type = resolve_type_annotation(
            self.source_id,
            self.source_text,
            &node.exception_type,
            self.symbol_table,
        );

        self.symbol_table.enter_scope();
        let exception_symbol = SymbolEntry {
            id: SymbolId(0),
            name: exception_name.to_string(),
            fully_qualified_name: exception_name.to_string(),
            source_id: self.source_id,
            span: node.variable_name,
            modifiers: Vec::new(),
            symbol_type: exception_type.clone(),
            parent_symbol: None,
            members: Vec::new(),
            is_static: false,
            is_const: false,
            slot_index: None,
            parameter_names: Vec::new(),
        };
        let exception_id = self.symbol_table.insert_symbol(exception_symbol);
        self.symbol_table
            .declare_local(exception_name, exception_id);
        self.symbol_table.record_local_symbol(LocalSymbolEntry {
            name: exception_name.to_string(),
            symbol_type: exception_type.clone(),
            decl_span: node.variable_name,
            scope_span: node.block.span,
            source_id: self.source_id,
            is_const: false,
            symbol_id: exception_id,
        });
        self.resolved_symbols
            .insert(node.variable_name, exception_id);
        self.node_types.insert(node.variable_name, exception_type);

        for statement in &node.block.statements {
            self.visit_statement(statement);
        }

        self.symbol_table.exit_scope();
    }

    fn visit_break_statement(&mut self, node: &BreakStatement) {
        if self.loop_depth == 0 && self.switch_depth == 0 {
            self.errors.push(BindingError::BreakOrContinueOutsideLoop {
                span: node.span.to_source_span(),
            });
        }
    }

    fn visit_continue_statement(&mut self, node: &ContinueStatement) {
        if self.loop_depth == 0 {
            self.errors.push(BindingError::BreakOrContinueOutsideLoop {
                span: node.span.to_source_span(),
            });
        }
    }

    fn visit_expression_statement(&mut self, node: &ExpressionStatement) {
        self.evaluate_expression_type(&node.expression);
    }

    fn visit_assert_statement(&mut self, node: &AssertStatement) {
        let cond_type = self.evaluate_expression_type(&node.condition);
        if cond_type != Type::Primitive(PrimitiveKind::Bool) && cond_type != Type::Error {
            self.errors.push(BindingError::TypeMismatch {
                expected: "bool".to_string(),
                found: cond_type.to_string(),
                span: node.condition.span().to_source_span(),
            });
        }
        if let Some(message) = &node.message {
            self.evaluate_expression_type(message);
        }
    }

    fn visit_exit_statement(&mut self, node: &ExitStatement) {
        let code_type = self.evaluate_expression_type(&node.code);
        if !code_type.is_signed_integer()
            && !code_type.is_unsigned_integer()
            && code_type != Type::Error
        {
            self.errors.push(BindingError::TypeMismatch {
                expected: "int32".to_string(),
                found: code_type.to_string(),
                span: node.code.span().to_source_span(),
            });
        }
    }

    fn visit_throw_statement(&mut self, node: &ThrowStatement) {
        let expr_type = self.evaluate_expression_type(&node.expression);
        if let Expression::Identifier { span } = &node.expression
            && let Some(&sym_id) = self.resolved_symbols.get(span)
            && let Some(sym_entry) = self.symbol_table.get_symbol(sym_id)
            && matches!(sym_entry.symbol_type, Type::Class { .. })
            && sym_entry.parent_symbol.is_none()
            && !self
                .symbol_table
                .scope_stack
                .iter()
                .any(|scope| scope.contains_key(&sym_entry.name))
        {
            self.errors.push(BindingError::TypeMismatch {
                expected: "throw target must be an Exception instance".to_string(),
                found: format!(
                    "Uninstantiated class identifier '{}' in throw",
                    sym_entry.name
                ),
                span: node.expression.span().to_source_span(),
            });
            return;
        }
        if expr_type.is_primitive() {
            self.errors.push(BindingError::TypeMismatch {
                expected: "throw target must be an Exception reference type".to_string(),
                found: format!("primitive {expr_type} in throw"),
                span: node.expression.span().to_source_span(),
            });
        }
    }
}

/// Coerces a literal or negated literal expression to a compatible target type,
/// recording the updated type in `node_types`.
pub fn coerce_literal_node_type(
    node_types: &mut HashMap<Span, Type>,
    expression: &Expression,
    target_type: &Type,
) -> bool {
    if !is_literal_compatible_with_type(expression, target_type) {
        return false;
    }
    let unwrapped_target = target_type.unwrap_alias();
    match expression {
        Expression::Literal { span, .. } => {
            node_types.insert(*span, unwrapped_target.clone());
            true
        }
        Expression::Unary {
            span,
            operator: TokenKind::OperatorMinus,
            operand,
            ..
        } => {
            node_types.insert(*span, unwrapped_target.clone());
            node_types.insert(operand.span(), unwrapped_target.clone());
            true
        }
        Expression::ArrayLiteral { span, elements } => {
            if let Type::Array {
                element_type,
                depth: 1,
            } = unwrapped_target
            {
                for element in elements {
                    coerce_literal_node_type(node_types, element, element_type);
                }
                node_types.insert(*span, unwrapped_target.clone());
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_literal_or_negated_literal(expression: &Expression) -> bool {
    match expression {
        Expression::Literal { .. } => true,
        Expression::Unary {
            operator: TokenKind::OperatorMinus,
            operand,
            ..
        } => matches!(**operand, Expression::Literal { .. }),
        _ => false,
    }
}

fn is_literal_compatible_with_type(expression: &Expression, target_type: &Type) -> bool {
    let unwrapped_target = target_type.unwrap_alias();
    match (expression, unwrapped_target) {
        (
            Expression::Literal {
                value: LiteralValue::Int(literal_val),
                ..
            },
            Type::Primitive(primitive_kind),
        ) => match primitive_kind {
            PrimitiveKind::Int8 => *literal_val >= -128 && *literal_val <= 127,
            PrimitiveKind::UInt8 => *literal_val >= 0 && *literal_val <= 255,
            PrimitiveKind::Int16 => *literal_val >= -32768 && *literal_val <= 32767,
            PrimitiveKind::UInt16 => *literal_val >= 0 && *literal_val <= 65535,
            PrimitiveKind::Int32 => {
                *literal_val >= i32::MIN as i64 && *literal_val <= i32::MAX as i64
            }
            PrimitiveKind::UInt32 => *literal_val >= 0 && *literal_val <= u32::MAX as i64,
            PrimitiveKind::Int64 => true,
            PrimitiveKind::UInt64 => *literal_val >= 0,
            PrimitiveKind::Float32 | PrimitiveKind::Float64 => true,
            _ => false,
        },
        (
            Expression::Literal {
                value: LiteralValue::UInt(literal_val),
                ..
            },
            Type::Primitive(primitive_kind),
        ) => match primitive_kind {
            PrimitiveKind::Int8 => *literal_val <= 127,
            PrimitiveKind::UInt8 => *literal_val <= 255,
            PrimitiveKind::Int16 => *literal_val <= 32767,
            PrimitiveKind::UInt16 => *literal_val <= 65535,
            PrimitiveKind::Int32 => *literal_val <= i32::MAX as u64,
            PrimitiveKind::UInt32 => *literal_val <= u32::MAX as u64,
            PrimitiveKind::Int64 => *literal_val <= i64::MAX as u64,
            PrimitiveKind::UInt64 => true,
            PrimitiveKind::Float32 | PrimitiveKind::Float64 => true,
            _ => false,
        },
        (
            Expression::Literal {
                value: LiteralValue::Float(_),
                ..
            },
            Type::Primitive(primitive_kind),
        ) => matches!(
            primitive_kind,
            PrimitiveKind::Float32 | PrimitiveKind::Float64
        ),
        (
            Expression::Literal {
                value: LiteralValue::String(_),
                ..
            },
            Type::Array {
                element_type,
                depth: 1,
            },
        ) if matches!(**element_type, Type::Primitive(PrimitiveKind::Char)) => true,
        (
            Expression::Unary {
                operator: TokenKind::OperatorMinus,
                operand,
                ..
            },
            Type::Primitive(primitive_kind),
        ) => match &**operand {
            Expression::Literal {
                value: LiteralValue::Int(literal_val),
                ..
            } => match primitive_kind {
                PrimitiveKind::Int8 => *literal_val <= 128 && *literal_val >= -127,
                PrimitiveKind::Int16 => *literal_val <= 32768 && *literal_val >= -32767,
                PrimitiveKind::Int32 => *literal_val <= 2147483648 && *literal_val >= -2147483647,
                PrimitiveKind::Int64 => true,
                PrimitiveKind::Float32 | PrimitiveKind::Float64 => true,
                _ => false,
            },
            Expression::Literal {
                value: LiteralValue::UInt(literal_val),
                ..
            } => match primitive_kind {
                PrimitiveKind::Int8 => *literal_val <= 128,
                PrimitiveKind::Int16 => *literal_val <= 32768,
                PrimitiveKind::Int32 => *literal_val <= 2147483648,
                PrimitiveKind::Int64 => *literal_val <= 9223372036854775808,
                PrimitiveKind::Float32 | PrimitiveKind::Float64 => true,
                _ => false,
            },
            Expression::Literal {
                value: LiteralValue::Float(_),
                ..
            } => matches!(
                primitive_kind,
                PrimitiveKind::Float32 | PrimitiveKind::Float64
            ),
            _ => false,
        },
        (
            Expression::ArrayLiteral { elements, .. },
            Type::Array {
                element_type,
                depth: 1,
            },
        ) => elements
            .iter()
            .all(|element_node| is_literal_compatible_with_type(element_node, element_type)),
        _ => false,
    }
}

fn block_definitely_returns(body: &BlockStatement) -> bool {
    for statement in &body.statements {
        if statement_definitely_returns(statement) {
            return true;
        }
    }
    false
}

fn statement_definitely_returns(statement: &Statement) -> bool {
    match statement {
        Statement::Return(_) | Statement::Throw(_) | Statement::Exit(_) => true,
        Statement::Block(block) => block_definitely_returns(block),
        Statement::If(if_stmt) => {
            if let Some(else_branch) = &if_stmt.else_branch {
                statement_definitely_returns(&if_stmt.then_branch)
                    && statement_definitely_returns(else_branch)
            } else {
                false
            }
        }
        Statement::Try(try_stmt) => {
            let try_returns = block_definitely_returns(&try_stmt.try_block);
            let finally_returns = try_stmt
                .finally_block
                .as_ref()
                .map(block_definitely_returns)
                .unwrap_or(false);
            if finally_returns {
                true
            } else if try_stmt.catch_clauses.is_empty() {
                try_returns
            } else {
                try_returns
                    && try_stmt
                        .catch_clauses
                        .iter()
                        .all(|clause| block_definitely_returns(&clause.block))
            }
        }
        Statement::Switch(switch_stmt) => {
            if let Some(default_stmts) = &switch_stmt.default_case {
                let default_returns = default_stmts.iter().any(statement_definitely_returns);
                let cases_return = switch_stmt
                    .cases
                    .iter()
                    .all(|clause| clause.statements.iter().any(statement_definitely_returns));
                default_returns && cases_return
            } else {
                false
            }
        }
        _ => false,
    }
}
