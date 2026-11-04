//! AST type annotation to semantic type resolution engine.

#![forbid(unsafe_code)]

use crate::phases::binder::symbols::SymbolTable;
use crate::phases::lexing::TokenKind;
use crate::phases::parser::statements::types::TypeAnnotation;
use solix_common::{PrimitiveKind, Type};

/// Converts a syntactic AST [`TypeAnnotation`] into a canonical semantic [`Type`].
#[must_use]
pub fn resolve_type_annotation(
    source_id: u16,
    source_text: &str,
    annotation: &TypeAnnotation,
    symbol_table: &SymbolTable,
) -> Type {
    match annotation {
        TypeAnnotation::Primitive { kind, .. } => {
            let primitive_kind = match kind {
                TokenKind::PrimitiveVoid => PrimitiveKind::Void,
                TokenKind::PrimitiveBool => PrimitiveKind::Bool,
                TokenKind::PrimitiveChar => PrimitiveKind::Char,
                TokenKind::PrimitiveInt8 => PrimitiveKind::Int8,
                TokenKind::PrimitiveInt16 => PrimitiveKind::Int16,
                TokenKind::PrimitiveInt32 => PrimitiveKind::Int32,
                TokenKind::PrimitiveInt64 => PrimitiveKind::Int64,
                TokenKind::PrimitiveUint8 => PrimitiveKind::UInt8,
                TokenKind::PrimitiveUint16 => PrimitiveKind::UInt16,
                TokenKind::PrimitiveUint32 => PrimitiveKind::UInt32,
                TokenKind::PrimitiveUint64 => PrimitiveKind::UInt64,
                TokenKind::PrimitiveFloat32 => PrimitiveKind::Float32,
                TokenKind::PrimitiveFloat64 => PrimitiveKind::Float64,
                TokenKind::PrimitiveTuple => PrimitiveKind::TuplePrimitive,
                _ => PrimitiveKind::Int32,
            };
            Type::Primitive(primitive_kind)
        }
        TypeAnnotation::Named { path, .. } => {
            let segments: Vec<&str> = path
                .iter()
                .map(|span| {
                    let start = span.start as usize;
                    let end = span.end as usize;
                    if end <= source_text.len() {
                        &source_text[start..end]
                    } else {
                        ""
                    }
                })
                .collect();
            let qualified_name = segments.join(".");

            // First check if this matches a primitive alias or keyword directly
            match qualified_name.as_str() {
                "string" | "std.string" | "std::string" => {
                    return Type::Class {
                        symbol_name: "std.string".to_string(),
                        type_arguments: Vec::new(),
                    };
                }
                "void" => return Type::Primitive(PrimitiveKind::Void),
                "bool" => return Type::Primitive(PrimitiveKind::Bool),
                "char" => return Type::Primitive(PrimitiveKind::Char),
                "int8" | "byte" => return Type::Primitive(PrimitiveKind::Int8),
                "int16" | "short" => return Type::Primitive(PrimitiveKind::Int16),
                "int32" | "int" => return Type::Primitive(PrimitiveKind::Int32),
                "int64" | "long" => return Type::Primitive(PrimitiveKind::Int64),
                "uint8" | "ubyte" => return Type::Primitive(PrimitiveKind::UInt8),
                "uint16" | "ushort" => return Type::Primitive(PrimitiveKind::UInt16),
                "uint32" | "uint" => return Type::Primitive(PrimitiveKind::UInt32),
                "uint64" | "ulong" => return Type::Primitive(PrimitiveKind::UInt64),
                "float32" | "float" => return Type::Primitive(PrimitiveKind::Float32),
                "float64" | "double" => return Type::Primitive(PrimitiveKind::Float64),
                _ => {}
            }

            // Look up in symbol table
            if let Some(symbol_id) = symbol_table.resolve_symbol(source_id, &qualified_name) {
                if let Some(entry) = symbol_table.get_symbol(symbol_id) {
                    match entry.symbol_type {
                        Type::Class {
                            ref symbol_name, ..
                        } => Type::Class {
                            symbol_name: symbol_name.clone(),
                            type_arguments: Vec::new(),
                        },
                        Type::Interface {
                            ref symbol_name, ..
                        } => Type::Interface {
                            symbol_name: symbol_name.clone(),
                            type_arguments: Vec::new(),
                        },
                        Type::Enum { ref symbol_name } => Type::Enum {
                            symbol_name: symbol_name.clone(),
                        },
                        Type::Alias {
                            ref alias_name,
                            ref target_type,
                        } => Type::Alias {
                            alias_name: alias_name.clone(),
                            target_type: target_type.clone(),
                        },
                        _ => entry.symbol_type.clone(),
                    }
                } else {
                    Type::Unresolved(qualified_name)
                }
            } else {
                Type::Unresolved(qualified_name)
            }
        }
        TypeAnnotation::Generic {
            base, arguments, ..
        } => {
            let base_type = resolve_type_annotation(source_id, source_text, base, symbol_table);
            let resolved_args: Vec<Type> = arguments
                .iter()
                .map(|argument| {
                    resolve_type_annotation(source_id, source_text, argument, symbol_table)
                })
                .collect();

            match base_type {
                Type::Class { symbol_name, .. } => Type::Class {
                    symbol_name,
                    type_arguments: resolved_args,
                },
                Type::Interface { symbol_name, .. } => Type::Interface {
                    symbol_name,
                    type_arguments: resolved_args,
                },
                Type::Alias { alias_name, .. } => Type::Class {
                    symbol_name: alias_name,
                    type_arguments: resolved_args,
                },
                Type::Unresolved(name) => Type::Class {
                    symbol_name: name,
                    type_arguments: resolved_args,
                },
                other => other,
            }
        }
        TypeAnnotation::Array {
            element_type,
            depth,
            ..
        } => {
            let mut resolved_element =
                resolve_type_annotation(source_id, source_text, element_type, symbol_table);
            while let Type::Array {
                element_type: inner_element,
                ..
            } = resolved_element
            {
                resolved_element = *inner_element;
            }
            Type::Array {
                element_type: Box::new(resolved_element),
                depth: *depth,
            }
        }
        TypeAnnotation::Tuple { elements, .. } => {
            let resolved_elements: Vec<Type> = elements
                .iter()
                .map(|element| {
                    resolve_type_annotation(source_id, source_text, element, symbol_table)
                })
                .collect();
            Type::Tuple(resolved_elements)
        }
        TypeAnnotation::FunctionPointer {
            return_type,
            parameters,
            ..
        } => {
            let resolved_return =
                resolve_type_annotation(source_id, source_text, return_type, symbol_table);
            let resolved_parameters: Vec<Type> = parameters
                .iter()
                .map(|parameter| {
                    resolve_type_annotation(source_id, source_text, parameter, symbol_table)
                })
                .collect();
            Type::FunctionPointer {
                return_type: Box::new(resolved_return),
                parameter_types: resolved_parameters,
            }
        }
    }
}
