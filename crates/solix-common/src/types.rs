//! Canonical type system representation and semantic type checking models.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Primitive scalar and built-in type kinds in Solix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveKind {
    /// Unit / void return type (`void`).
    Void,
    /// 1-bit boolean flag (`bool`).
    Bool,
    /// 8-bit unsigned ASCII character (`char`).
    Char,
    /// 8-bit signed two's complement integer (`int8`).
    Int8,
    /// 16-bit signed two's complement integer (`int16`).
    Int16,
    /// 32-bit signed two's complement integer (`int32`).
    Int32,
    /// 64-bit signed two's complement integer (`int64`).
    Int64,
    /// 8-bit unsigned integer / raw byte (`uint8`).
    UInt8,
    /// 16-bit unsigned integer (`uint16`).
    UInt16,
    /// 32-bit unsigned integer (`uint32`).
    UInt32,
    /// 64-bit unsigned integer (`uint64`).
    UInt64,
    /// 32-bit IEEE 754 single-precision float (`float32`).
    Float32,
    /// 64-bit IEEE 754 double-precision float (`float64`).
    Float64,
    /// First-class dynamic heterogeneous tuple primitive (`tuple`).
    TuplePrimitive,
}

impl PrimitiveKind {
    /// Returns the memory width in bytes occupied by this primitive type.
    #[must_use]
    pub const fn byte_size(self) -> usize {
        match self {
            Self::Void => 0,
            Self::Bool | Self::Char | Self::Int8 | Self::UInt8 => 1,
            Self::Int16 | Self::UInt16 => 2,
            Self::Int32 | Self::UInt32 | Self::Float32 => 4,
            Self::Int64 | Self::UInt64 | Self::Float64 | Self::TuplePrimitive => 8,
        }
    }

    /// Returns whether this primitive is a signed integer type.
    #[must_use]
    pub const fn is_signed_integer(self) -> bool {
        matches!(self, Self::Int8 | Self::Int16 | Self::Int32 | Self::Int64)
    }

    /// Returns whether this primitive is an unsigned integer type.
    #[must_use]
    pub const fn is_unsigned_integer(self) -> bool {
        matches!(
            self,
            Self::UInt8 | Self::UInt16 | Self::UInt32 | Self::UInt64
        )
    }

    /// Returns whether this primitive is any integer type (signed or unsigned).
    #[must_use]
    pub const fn is_integer(self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Returns whether this primitive is a floating-point type.
    #[must_use]
    pub const fn is_floating_point(self) -> bool {
        matches!(self, Self::Float32 | Self::Float64)
    }

    /// Returns whether this primitive is numeric (integer or floating-point).
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        self.is_integer() || self.is_floating_point()
    }

    /// Returns the canonical keyword name for this primitive.
    #[must_use]
    pub const fn keyword_name(self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Bool => "bool",
            Self::Char => "char",
            Self::Int8 => "int8",
            Self::Int16 => "int16",
            Self::Int32 => "int32",
            Self::Int64 => "int64",
            Self::UInt8 => "uint8",
            Self::UInt16 => "uint16",
            Self::UInt32 => "uint32",
            Self::UInt64 => "uint64",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::TuplePrimitive => "tuple",
        }
    }
}

impl Display for PrimitiveKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.keyword_name())
    }
}

/// Canonical semantic type representation across compilation and semantic analysis phases.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Primitive value type.
    Primitive(PrimitiveKind),
    /// User-defined class nominal reference type with optional concrete type arguments.
    Class {
        /// Fully-qualified class symbol name (e.g. `solix.core.String`, `solix.collections.List<int32>`).
        symbol_name: String,
        /// Concrete type arguments if this is an instantiated generic class.
        type_arguments: Vec<Type>,
    },
    /// Interface nominal reference type with optional concrete type arguments.
    Interface {
        /// Fully-qualified interface symbol name (e.g. `solix.core.IComparable`).
        symbol_name: String,
        /// Concrete type arguments if this is an instantiated generic interface.
        type_arguments: Vec<Type>,
    },
    /// Enumeration type with discrete named variants.
    Enum {
        /// Fully-qualified enum symbol name (e.g. `solix.tests.master.Color`).
        symbol_name: String,
    },
    /// Multidimensional or single-dimension array reference type (`T[]`).
    Array {
        /// Element type held within the array.
        element_type: Box<Type>,
        /// Array nesting depth (1 for `T[]`, 2 for `T[][]`).
        depth: usize,
    },
    /// Strongly-typed heterogeneous tuple value (`tuple<int32, string>`).
    Tuple(Vec<Type>),
    /// First-class function pointer callable signature (`RetType(*)(ParamType1, ParamType2)`).
    FunctionPointer {
        /// Callable return type.
        return_type: Box<Type>,
        /// Formal parameter types accepted by the callable.
        parameter_types: Vec<Type>,
    },
    /// Generic type parameter placeholder during generic definition analysis (`<T>`).
    GenericParameter {
        /// Identifier name of the generic parameter (e.g., `"T"`, `"V"`).
        name: String,
        /// 0-based parameter index.
        index: usize,
    },
    /// Type alias referencing an underlying resolved target type.
    Alias {
        /// Name of the alias.
        alias_name: String,
        /// The aliased underlying type.
        target_type: Box<Type>,
    },
    /// Literal null object reference, assignable to any reference type.
    Null,
    /// Unresolved nominal identifier before pass-2 type resolution completes.
    Unresolved(String),
    /// Error sentinel type used to suppress cascading diagnostic reporting.
    Error,
}

impl Default for Type {
    fn default() -> Self {
        Self::Primitive(PrimitiveKind::Void)
    }
}

impl Type {
    /// Returns whether this type represents an automatic reference counting (ARC) managed reference.
    #[must_use]
    pub fn is_reference_type(&self) -> bool {
        match self {
            Self::Class { .. }
            | Self::Interface { .. }
            | Self::Array { .. }
            | Self::Null
            | Self::Tuple(_)
            | Self::FunctionPointer { .. }
            | Self::Primitive(PrimitiveKind::TuplePrimitive) => true,
            Self::Alias { target_type, .. } => target_type.is_reference_type(),
            Self::Primitive(_)
            | Self::Enum { .. }
            | Self::GenericParameter { .. }
            | Self::Unresolved(_)
            | Self::Error => false,
        }
    }

    /// Returns whether this type is tracked by the GC reference-counting cascade.
    ///
    /// This differs from [`is_reference_type`] in that the primitive `string` type
    /// (`std.string` / `BlockTag::String`) is a raw byte buffer managed by its own
    /// internal allocator — it is **not** an ARC object and must **not** appear in
    /// `reference_field_offsets` or be passed to `release_reference`.
    #[must_use]
    pub fn is_gc_managed_reference_type(&self) -> bool {
        match self {
            Self::Class { symbol_name, .. }
                if symbol_name == "std.string"
                    || symbol_name == "std::string"
                    || symbol_name == "string" =>
            {
                false
            }
            Self::Alias { target_type, .. } => target_type.is_gc_managed_reference_type(),
            other => other.is_reference_type(),
        }
    }

    /// Returns whether this type is a primitive value type.
    #[must_use]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    /// Returns whether this type is a numeric scalar type.
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        match self {
            Self::Primitive(primitive) => primitive.is_numeric(),
            Self::Alias { target_type, .. } => target_type.is_numeric(),
            _ => false,
        }
    }

    /// Returns whether this type is a signed integer scalar type.
    #[must_use]
    pub fn is_signed_integer(&self) -> bool {
        match self {
            Self::Primitive(primitive) => primitive.is_signed_integer(),
            Self::Alias { target_type, .. } => target_type.is_signed_integer(),
            _ => false,
        }
    }

    /// Returns whether this type is an unsigned integer scalar type.
    #[must_use]
    pub fn is_unsigned_integer(&self) -> bool {
        match self {
            Self::Primitive(primitive) => primitive.is_unsigned_integer(),
            Self::Alias { target_type, .. } => target_type.is_unsigned_integer(),
            _ => false,
        }
    }

    /// Returns whether this type is any integer scalar type (signed or unsigned).
    #[must_use]
    pub fn is_integer(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Returns whether this type is a floating-point scalar type.
    #[must_use]
    pub fn is_floating_point(&self) -> bool {
        match self {
            Self::Primitive(primitive) => primitive.is_floating_point(),
            Self::Alias { target_type, .. } => target_type.is_floating_point(),
            _ => false,
        }
    }

    /// Returns whether this type is the unit `void` type.
    #[must_use]
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveKind::Void))
    }

    /// Unwraps any type aliases to access the underlying concrete type.
    #[must_use]
    pub fn unwrap_alias(&self) -> &Self {
        match self {
            Self::Alias { target_type, .. } => target_type.unwrap_alias(),
            other => other,
        }
    }

    /// Recursively inspects the type to locate the first unresolved nominal type reference, if any.
    #[must_use]
    pub fn first_unresolved_name(&self) -> Option<&str> {
        match self {
            Self::Unresolved(name) => Some(name.as_str()),
            Self::Array { element_type, .. } => element_type.first_unresolved_name(),
            Self::Tuple(elements) => elements
                .iter()
                .find_map(|element| element.first_unresolved_name()),
            Self::FunctionPointer {
                return_type,
                parameter_types,
            } => return_type.first_unresolved_name().or_else(|| {
                parameter_types
                    .iter()
                    .find_map(|parameter| parameter.first_unresolved_name())
            }),
            Self::Class { type_arguments, .. } | Self::Interface { type_arguments, .. } => {
                type_arguments
                    .iter()
                    .find_map(|argument| argument.first_unresolved_name())
            }
            Self::Alias { target_type, .. } => target_type.first_unresolved_name(),
            Self::Primitive(_)
            | Self::Enum { .. }
            | Self::GenericParameter { .. }
            | Self::Null
            | Self::Error => None,
        }
    }

    /// Evaluates if `self` can be implicitly widened to `target` without information loss.
    ///
    /// Chains defined in SPEC-0001 / `docs/internals/specs/language/type-system.md`:
    /// * `int8` -> `int16` -> `int32` -> `int64`
    /// * `uint8` -> `uint16` -> `uint32` -> `uint64`
    /// * `uint8` -> `int16` | `int32` | `int64`
    /// * `uint16` -> `int32` | `int64`
    /// * `uint32` -> `int64`
    /// * `float32` -> `float64`
    /// * `int8` | `uint8` | `int16` | `uint16` -> `float32` | `float64`
    /// * `int32` | `uint32` -> `float64`
    #[must_use]
    pub fn is_widening_convertible_to(&self, target: &Self) -> bool {
        let unwrapped_source = self.unwrap_alias();
        let unwrapped_target = target.unwrap_alias();

        if unwrapped_source == unwrapped_target {
            return true;
        }

        matches!(
            (unwrapped_source, unwrapped_target),
            (
                Self::Primitive(PrimitiveKind::Int8),
                Self::Primitive(
                    PrimitiveKind::Int16
                        | PrimitiveKind::Int32
                        | PrimitiveKind::Int64
                        | PrimitiveKind::Float32
                        | PrimitiveKind::Float64,
                ),
            ) | (
                Self::Primitive(PrimitiveKind::Int16),
                Self::Primitive(
                    PrimitiveKind::Int32
                        | PrimitiveKind::Int64
                        | PrimitiveKind::Float32
                        | PrimitiveKind::Float64,
                ),
            ) | (
                Self::Primitive(PrimitiveKind::Int32),
                Self::Primitive(PrimitiveKind::Int64 | PrimitiveKind::Float64),
            ) | (
                Self::Primitive(PrimitiveKind::UInt8),
                Self::Primitive(
                    PrimitiveKind::UInt16
                        | PrimitiveKind::UInt32
                        | PrimitiveKind::UInt64
                        | PrimitiveKind::Int16
                        | PrimitiveKind::Int32
                        | PrimitiveKind::Int64
                        | PrimitiveKind::Float32
                        | PrimitiveKind::Float64,
                ),
            ) | (
                Self::Primitive(PrimitiveKind::UInt16),
                Self::Primitive(
                    PrimitiveKind::UInt32
                        | PrimitiveKind::UInt64
                        | PrimitiveKind::Int32
                        | PrimitiveKind::Int64
                        | PrimitiveKind::Float32
                        | PrimitiveKind::Float64,
                ),
            ) | (
                Self::Primitive(PrimitiveKind::UInt32),
                Self::Primitive(
                    PrimitiveKind::UInt64 | PrimitiveKind::Int64 | PrimitiveKind::Float64,
                ),
            ) | (
                Self::Primitive(PrimitiveKind::Float32),
                Self::Primitive(PrimitiveKind::Float64),
            )
        )
    }

    /// Evaluates whether converting `self` to `target` is a narrowing conversion requiring an explicit cast.
    #[must_use]
    pub fn is_narrowing_convertible_to(&self, target: &Self) -> bool {
        let unwrapped_source = self.unwrap_alias();
        let unwrapped_target = target.unwrap_alias();

        if unwrapped_source.is_numeric() && unwrapped_target.is_numeric() {
            !unwrapped_source.is_widening_convertible_to(unwrapped_target)
        } else {
            false
        }
    }

    /// Determines whether an expression of type `self` can be assigned or passed to a location expecting `target`.
    ///
    /// Accepts a predicate closure `is_subtype(source_symbol, target_symbol)` to resolve nominal class/interface subtyping.
    pub fn is_assignable_to(&self, target: &Self, is_subtype: &dyn Fn(&str, &str) -> bool) -> bool {
        let unwrapped_source = self.unwrap_alias();
        let unwrapped_target = target.unwrap_alias();

        // Exact match
        if unwrapped_source == unwrapped_target {
            return true;
        }

        // Exact nominal match with normalized namespace separators
        match (unwrapped_source, unwrapped_target) {
            (
                Self::Class {
                    symbol_name: source_name,
                    type_arguments: source_args,
                },
                Self::Class {
                    symbol_name: target_name,
                    type_arguments: target_args,
                },
            ) if source_name.replace('.', "::") == target_name.replace('.', "::") => {
                if source_args.is_empty() || target_args.is_empty() {
                    return true;
                }
                return source_args == target_args;
            }
            (
                Self::Enum {
                    symbol_name: source_name,
                },
                Self::Enum {
                    symbol_name: target_name,
                },
            ) if source_name.replace('.', "::") == target_name.replace('.', "::") => {
                return true;
            }
            _ => {}
        }

        // Enum variants are convertible to integer types
        if matches!(unwrapped_source, Self::Enum { .. })
            && (unwrapped_target.is_signed_integer() || unwrapped_target.is_unsigned_integer())
        {
            return true;
        }

        // Null is assignable to any reference type
        if matches!(unwrapped_source, Self::Null) && unwrapped_target.is_reference_type() {
            return true;
        }

        // Numeric widening
        if unwrapped_source.is_widening_convertible_to(unwrapped_target) {
            return true;
        }

        // Nominal subtyping for classes and interfaces
        match (unwrapped_source, unwrapped_target) {
            (
                Self::Class {
                    symbol_name: source_name,
                    type_arguments: source_args,
                },
                Self::Class {
                    symbol_name: target_name,
                    type_arguments: target_args,
                },
            )
            | (
                Self::Class {
                    symbol_name: source_name,
                    type_arguments: source_args,
                },
                Self::Interface {
                    symbol_name: target_name,
                    type_arguments: target_args,
                },
            )
            | (
                Self::Interface {
                    symbol_name: source_name,
                    type_arguments: source_args,
                },
                Self::Interface {
                    symbol_name: target_name,
                    type_arguments: target_args,
                },
            ) => {
                if is_subtype(source_name, target_name) {
                    if source_args.is_empty() || target_args.is_empty() {
                        true
                    } else {
                        // Invariant generic parameters: arguments must match exactly
                        source_args == target_args
                    }
                } else {
                    false
                }
            }
            (
                Self::Array {
                    element_type: source_elem,
                    depth: source_depth,
                },
                Self::Array {
                    element_type: target_elem,
                    depth: target_depth,
                },
            ) => {
                if source_depth == target_depth {
                    // Array covariance for reference element types
                    source_elem.is_assignable_to(target_elem, is_subtype)
                } else {
                    false
                }
            }
            (Self::Tuple(_), Self::Primitive(PrimitiveKind::TuplePrimitive))
            | (Self::Primitive(PrimitiveKind::TuplePrimitive), Self::Tuple(_)) => true,
            (Self::Tuple(_), Self::Tuple(target_elements)) if target_elements.is_empty() => true,
            (Self::Tuple(source_elements), Self::Tuple(target_elements)) => {
                if source_elements.len() == target_elements.len() {
                    source_elements.iter().zip(target_elements.iter()).all(
                        |(source_item, target_item)| {
                            source_item.is_assignable_to(target_item, is_subtype)
                        },
                    )
                } else {
                    false
                }
            }
            (
                Self::FunctionPointer {
                    return_type: source_ret,
                    parameter_types: source_params,
                },
                Self::FunctionPointer {
                    return_type: target_ret,
                    parameter_types: target_params,
                },
            ) if source_params.len() == target_params.len() => {
                // Covariant return type, contravariant parameter types
                let return_compatible = source_ret.is_assignable_to(target_ret, is_subtype);
                let params_compatible = target_params.iter().zip(source_params.iter()).all(
                    |(target_param, source_param)| {
                        target_param.is_assignable_to(source_param, is_subtype)
                    },
                );
                return_compatible && params_compatible
            }
            _ => false,
        }
    }
}

impl Display for Type {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Primitive(primitive) => write!(formatter, "{primitive}"),
            Self::Class {
                symbol_name,
                type_arguments,
            }
            | Self::Interface {
                symbol_name,
                type_arguments,
            } => {
                // Normalize the string primitive aliases to the canonical `string` keyword.
                let normalized_name = if symbol_name == "std.string"
                    || symbol_name == "std::string"
                    || symbol_name == "string"
                {
                    "string".to_string()
                } else {
                    symbol_name.replace('.', "::")
                };
                write!(formatter, "{normalized_name}")?;
                if !type_arguments.is_empty() {
                    write!(formatter, "<")?;
                    for (index, argument) in type_arguments.iter().enumerate() {
                        if index > 0 {
                            write!(formatter, ", ")?;
                        }
                        // Format each argument and add a trailing space before `>` when the
                        // argument itself ends with `>` to avoid the parser seeing `>>` as
                        // the OperatorShiftRight token.
                        let argument_text = format!("{argument}");
                        write!(formatter, "{argument_text}")?;
                    }
                    // If the last argument's formatted text ends with `>`, emit a separating
                    // space so the closing `>` does not form a `>>` shift-right token.
                    let last_argument_text = format!(
                        "{}",
                        type_arguments.last().expect("checked non-empty above")
                    );
                    if last_argument_text.ends_with('>') {
                        write!(formatter, " ")?;
                    }
                    write!(formatter, ">")?;
                }
                Ok(())
            }
            Self::Enum { symbol_name } => {
                let normalized_name = symbol_name.replace('.', "::");
                write!(formatter, "{normalized_name}")
            }
            Self::Array {
                element_type,
                depth,
            } => {
                write!(formatter, "{element_type}")?;
                for _ in 0..*depth {
                    write!(formatter, "[]")?;
                }
                Ok(())
            }
            Self::Tuple(elements) => {
                write!(formatter, "tuple<")?;
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{element}")?;
                }
                write!(formatter, ">")
            }
            Self::FunctionPointer {
                return_type,
                parameter_types,
            } => {
                write!(formatter, "{return_type}(*)(")?;
                for (index, parameter) in parameter_types.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{parameter}")?;
                }
                write!(formatter, ")")
            }
            Self::GenericParameter { name, .. } => write!(formatter, "{name}"),
            Self::Alias { alias_name, .. } => write!(formatter, "{alias_name}"),
            Self::Null => write!(formatter, "null"),
            Self::Unresolved(name) => write!(formatter, "{name}"),
            Self::Error => write!(formatter, "<error-type>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_properties_and_byte_sizes() {
        assert_eq!(PrimitiveKind::Void.byte_size(), 0);
        assert_eq!(PrimitiveKind::Int8.byte_size(), 1);
        assert_eq!(PrimitiveKind::Int16.byte_size(), 2);
        assert_eq!(PrimitiveKind::Int32.byte_size(), 4);
        assert_eq!(PrimitiveKind::Int64.byte_size(), 8);
        assert_eq!(PrimitiveKind::Float32.byte_size(), 4);
        assert_eq!(PrimitiveKind::Float64.byte_size(), 8);

        assert!(PrimitiveKind::Int32.is_signed_integer());
        assert!(PrimitiveKind::UInt32.is_unsigned_integer());
        assert!(PrimitiveKind::Float64.is_floating_point());
        assert!(PrimitiveKind::Int32.is_numeric());
        assert!(!PrimitiveKind::Bool.is_numeric());
    }

    #[test]
    fn test_widening_conversions_chain() {
        let int8_type = Type::Primitive(PrimitiveKind::Int8);
        let int16_type = Type::Primitive(PrimitiveKind::Int16);
        let int32_type = Type::Primitive(PrimitiveKind::Int32);
        let int64_type = Type::Primitive(PrimitiveKind::Int64);
        let uint8_type = Type::Primitive(PrimitiveKind::UInt8);
        let uint16_type = Type::Primitive(PrimitiveKind::UInt16);
        let uint32_type = Type::Primitive(PrimitiveKind::UInt32);
        let uint64_type = Type::Primitive(PrimitiveKind::UInt64);
        let float64_type = Type::Primitive(PrimitiveKind::Float64);
        let float32_type = Type::Primitive(PrimitiveKind::Float32);

        // Signed chain
        assert!(int8_type.is_widening_convertible_to(&int16_type));
        assert!(int8_type.is_widening_convertible_to(&int32_type));
        assert!(int8_type.is_widening_convertible_to(&int64_type));
        assert!(int8_type.is_widening_convertible_to(&float32_type));
        assert!(int8_type.is_widening_convertible_to(&float64_type));
        assert!(int16_type.is_widening_convertible_to(&float32_type));
        assert!(int32_type.is_widening_convertible_to(&float64_type));
        assert!(float32_type.is_widening_convertible_to(&float64_type));

        // Unsigned chain
        assert!(uint8_type.is_widening_convertible_to(&uint16_type));
        assert!(uint8_type.is_widening_convertible_to(&uint32_type));
        assert!(uint8_type.is_widening_convertible_to(&uint64_type));
        assert!(uint8_type.is_widening_convertible_to(&int16_type));
        assert!(uint8_type.is_widening_convertible_to(&int32_type));
        assert!(uint8_type.is_widening_convertible_to(&int64_type));
        assert!(uint8_type.is_widening_convertible_to(&float32_type));
        assert!(uint8_type.is_widening_convertible_to(&float64_type));

        assert!(uint16_type.is_widening_convertible_to(&uint32_type));
        assert!(uint16_type.is_widening_convertible_to(&uint64_type));
        assert!(uint16_type.is_widening_convertible_to(&int32_type));
        assert!(uint16_type.is_widening_convertible_to(&int64_type));
        assert!(uint16_type.is_widening_convertible_to(&float32_type));
        assert!(uint16_type.is_widening_convertible_to(&float64_type));

        assert!(uint32_type.is_widening_convertible_to(&uint64_type));
        assert!(uint32_type.is_widening_convertible_to(&int64_type));
        assert!(uint32_type.is_widening_convertible_to(&float64_type));

        // Negative: narrowing conversions
        assert!(!int64_type.is_widening_convertible_to(&int32_type));
        assert!(int64_type.is_narrowing_convertible_to(&int32_type));
        assert!(!uint8_type.is_widening_convertible_to(&int8_type));
        assert!(!uint16_type.is_widening_convertible_to(&int16_type));
        assert!(!uint32_type.is_widening_convertible_to(&int32_type));
        assert!(!int64_type.is_widening_convertible_to(&float64_type));
        assert!(!float64_type.is_widening_convertible_to(&float32_type));
    }

    #[test]
    fn test_reference_type_classification() {
        let class_type = Type::Class {
            symbol_name: "std.collections.List".to_string(),
            type_arguments: vec![],
        };
        let array_type = Type::Array {
            element_type: Box::new(Type::Primitive(PrimitiveKind::Int32)),
            depth: 1,
        };
        let null_type = Type::Null;
        let int_type = Type::Primitive(PrimitiveKind::Int32);

        assert!(class_type.is_reference_type());
        assert!(array_type.is_reference_type());
        assert!(null_type.is_reference_type());
        assert!(!int_type.is_reference_type());
    }

    #[test]
    fn test_null_assignability_to_reference_types() {
        let null_type = Type::Null;
        let class_type = Type::Class {
            symbol_name: "Player".to_string(),
            type_arguments: vec![],
        };
        let int_type = Type::Primitive(PrimitiveKind::Int32);

        assert!(null_type.is_assignable_to(&class_type, &|_, _| false));
        assert!(!null_type.is_assignable_to(&int_type, &|_, _| false));
    }

    #[test]
    fn test_positive_first_unresolved_name_recursive() {
        let unresolved_direct = Type::Unresolved("MissingType".to_string());
        assert_eq!(
            unresolved_direct.first_unresolved_name(),
            Some("MissingType")
        );

        let unresolved_array = Type::Array {
            element_type: Box::new(Type::Unresolved("MissingElement".to_string())),
            depth: 2,
        };
        assert_eq!(
            unresolved_array.first_unresolved_name(),
            Some("MissingElement")
        );

        let unresolved_tuple = Type::Tuple(vec![
            Type::Primitive(PrimitiveKind::Int32),
            Type::Unresolved("MissingTupleItem".to_string()),
        ]);
        assert_eq!(
            unresolved_tuple.first_unresolved_name(),
            Some("MissingTupleItem")
        );

        let unresolved_fn_ptr = Type::FunctionPointer {
            return_type: Box::new(Type::Primitive(PrimitiveKind::Void)),
            parameter_types: vec![
                Type::Primitive(PrimitiveKind::Bool),
                Type::Unresolved("MissingParam".to_string()),
            ],
        };
        assert_eq!(
            unresolved_fn_ptr.first_unresolved_name(),
            Some("MissingParam")
        );

        let unresolved_class_arg = Type::Class {
            symbol_name: "std.collections.List".to_string(),
            type_arguments: vec![Type::Unresolved("MissingGenericArg".to_string())],
        };
        assert_eq!(
            unresolved_class_arg.first_unresolved_name(),
            Some("MissingGenericArg")
        );
    }

    #[test]
    fn test_negative_first_unresolved_name_resolved_types() {
        let primitive_type = Type::Primitive(PrimitiveKind::Int32);
        assert_eq!(primitive_type.first_unresolved_name(), None);

        let resolved_array = Type::Array {
            element_type: Box::new(Type::Primitive(PrimitiveKind::Float64)),
            depth: 1,
        };
        assert_eq!(resolved_array.first_unresolved_name(), None);

        let resolved_class = Type::Class {
            symbol_name: "solix.core.String".to_string(),
            type_arguments: vec![],
        };
        assert_eq!(resolved_class.first_unresolved_name(), None);
    }
}
