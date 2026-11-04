//! Multi-pass semantic analysis submodules.

#![forbid(unsafe_code)]

pub mod pass1_symbol_discovery;
pub mod pass2_type_hierarchy;
pub mod pass3_type_checking;

pub use pass1_symbol_discovery::execute_pass1_symbol_discovery;
pub use pass2_type_hierarchy::execute_pass2_type_hierarchy;
pub use pass3_type_checking::TypeCheckingVisitor;
