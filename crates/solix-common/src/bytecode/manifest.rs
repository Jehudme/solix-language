//! Export manifest and API reflection models for dynamic linking and self-describing binaries.

#![forbid(unsafe_code)]

use crate::bytecode::binary::BinaryProgram;
use crate::types::Type;
use serde::{Deserialize, Serialize};

/// Declaration modifiers attached to exported classes, methods, fields, and static variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeclarationModifier {
    Public,
    Protected,
    Private,
    Static,
    Const,
    Virtual,
    Override,
    Abstract,
    Weak,
    Inline,
    Extern,
}

impl DeclarationModifier {
    /// Returns the canonical Solix keyword string for this modifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Protected => "protected",
            Self::Private => "private",
            Self::Static => "static",
            Self::Const => "const",
            Self::Virtual => "virtual",
            Self::Override => "override",
            Self::Abstract => "abstract",
            Self::Weak => "weak",
            Self::Inline => "inline",
            Self::Extern => "extern",
        }
    }
}

/// Metadata describing a formal parameter of an exported function or method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedParameter {
    /// Identifier name of the parameter.
    pub name: String,
    /// Canonical semantic type of the parameter.
    pub param_type: Type,
}

/// Metadata describing an exported function, method, constructor, or destructor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedFunction {
    /// Fully-qualified mangled symbol (e.g. `"Math::add/2"`, `"Point::constructor/2"`).
    pub symbol: String,
    /// Unmangled simple name (e.g. `"add"`, `"constructor"`, `"destructor"`).
    pub simple_name: String,
    /// Access control and behavior modifiers.
    pub modifiers: Vec<DeclarationModifier>,
    /// Formal parameters in declaration order.
    pub parameters: Vec<ExportedParameter>,
    /// Return type of the function or method.
    pub return_type: Type,
    /// Whether this function is a constructor.
    pub is_constructor: bool,
    /// Whether this function is a destructor.
    pub is_destructor: bool,
    /// Whether this function is an operator overload.
    #[serde(default)]
    pub is_operator: bool,
}

/// Metadata describing an exported static global variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedStatic {
    /// Fully qualified global symbol name (e.g. `"Math::PI"`).
    pub symbol: String,
    /// Unmangled identifier name.
    pub name: String,
    /// Semantic type of the static variable.
    pub var_type: Type,
    /// Modifiers applied to the variable declaration.
    pub modifiers: Vec<DeclarationModifier>,
    /// Module-local static memory slot index.
    pub slot: u32,
}

/// Metadata describing an exported instance field of a class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedField {
    /// Field identifier name.
    pub name: String,
    /// Semantic type of the field.
    pub field_type: Type,
    /// Modifiers applied to the field.
    pub modifiers: Vec<DeclarationModifier>,
    /// Byte offset within the class heap object layout.
    pub offset: usize,
}

/// Metadata describing an exported enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedEnum {
    /// Identifier name of the enum.
    pub name: String,
    /// Declaration modifiers applied to the enum.
    pub modifiers: Vec<DeclarationModifier>,
    /// Enumeration variants with optional explicit integer discriminator values.
    pub variants: Vec<(String, Option<i64>)>,
}

/// Metadata describing an exported nominal class or interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedClass {
    /// Fully qualified symbol name (e.g. `"std::collections::List"`).
    pub symbol: String,
    /// Unqualified class name.
    pub name: String,
    /// Class-level modifiers.
    pub modifiers: Vec<DeclarationModifier>,
    /// Whether this type is an interface.
    pub is_interface: bool,
    /// Fully qualified base class symbol if inherited.
    pub base_class: Option<String>,
    /// List of implemented interface symbols.
    pub interfaces: Vec<String>,
    /// Generic type parameter names (e.g. `["T", "U"]`).
    pub generic_parameters: Vec<String>,
    /// Exported public/protected instance fields.
    pub fields: Vec<ExportedField>,
    /// Exported member methods, constructors, and destructors.
    pub methods: Vec<ExportedFunction>,
}

/// Metadata containing uninstantiated generic template class or function source code.
/// This enables downstream consumers to monomorphize new concrete specializations (e.g., `Box<MyType>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedGenericTemplate {
    /// Template identifier name (e.g. `"Box"`).
    pub name: String,
    /// Generic type parameter names.
    pub parameter_names: Vec<String>,
    /// Raw Solix template source code for monomorphization.
    pub template_source: String,
}

/// Complete export manifest describing all public APIs and generic templates of a compiled binary.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ExportManifest {
    /// Package or library name.
    pub package_name: String,
    /// Package semantic version string.
    pub package_version: String,
    /// Human-readable package description.
    pub package_description: String,
    /// Explicit package dependencies recorded during compilation.
    #[serde(default)]
    pub dependencies: Vec<crate::solver::DependencyRequirement>,
    /// Exported nominal classes and interfaces.
    pub exported_classes: Vec<ExportedClass>,
    /// Exported nominal enumerations.
    #[serde(default)]
    pub exported_enums: Vec<ExportedEnum>,
    /// Exported top-level functions.
    pub exported_functions: Vec<ExportedFunction>,
    /// Exported static variables.
    pub exported_statics: Vec<ExportedStatic>,
    /// Uninstantiated generic templates for downstream monomorphization.
    pub generic_templates: Vec<ExportedGenericTemplate>,
}

/// Extracts the public API manifest from a compiled binary program as a formatted JSON string.
///
/// # Errors
/// Returns `serde_json::Error` if serialization fails.
pub fn extract_api_manifest(binary: &BinaryProgram) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&binary.export_manifest)
}
