//! Semantic analysis (binder) phase module.

#![forbid(unsafe_code)]

#[allow(clippy::module_inception)]
pub mod binder;
pub mod layout;
pub mod monomorphizer;
pub mod passes;
pub mod symbols;
pub mod type_resolver;

pub use binder::{BindingArtifacts, BindingMetaData, bind, bind_context, bind_incremental};
pub use layout::ClassLayout;
pub use monomorphizer::{GenericBlueprint, GenericBlueprintRegistry};
pub use symbols::{ImportEntry, SymbolEntry, SymbolId, SymbolKind, SymbolTable};
pub use type_resolver::resolve_type_annotation;
