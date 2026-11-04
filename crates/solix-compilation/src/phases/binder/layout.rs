//! Class memory layout calculation, VTable construction, and ARC reference field tracing.

#![forbid(unsafe_code)]

use crate::phases::binder::symbols::SymbolId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime memory layout and polymorphic dispatch metadata calculated for a class.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ClassLayout {
    /// Symbol ID of the class type.
    pub class_symbol_id: SymbolId,
    /// Fully qualified name of the class.
    pub class_name: String,
    /// Total memory size in bytes occupied by an instance of this class on the dynamic heap.
    pub instance_size_bytes: usize,
    /// Unique sequential virtual table identifier assigned to this class.
    pub vtable_id: u32,
    /// Optional VTable identifier of the base superclass (for super calls and casting).
    pub base_vtable_id: Option<u32>,
    /// Byte offsets of all instance fields mapped by field identifier name.
    pub field_offsets: HashMap<String, usize>,
    /// Ordered list of virtual method symbol IDs populating the class's VTable slots.
    pub vtable_methods: Vec<SymbolId>,
    /// Interface method dispatch tables mapping interface name to implemented method symbol IDs.
    pub itable_methods: HashMap<String, Vec<SymbolId>>,
    /// Byte offsets of instance fields holding ARC-managed object references.
    pub reference_field_offsets: Vec<usize>,
    /// Byte offsets of non-owning weak reference instance fields.
    pub weak_field_offsets: Vec<usize>,
    /// Virtual method slot indices mapped by method identifier name.
    pub method_slots: HashMap<String, usize>,
    /// Explicit mapping of field byte offsets to their primitive or reference type tags.
    pub field_types: HashMap<usize, u8>,
    /// Whether this class is an external declaration imported from a library.
    pub is_extern: bool,
}

impl ClassLayout {
    /// Constructs a new class layout description.
    #[must_use]
    pub fn new(class_symbol_id: SymbolId, class_name: impl Into<String>, vtable_id: u32) -> Self {
        Self {
            class_symbol_id,
            class_name: class_name.into(),
            instance_size_bytes: 8, // Base header: 8 bytes for object metadata / vtable_id
            vtable_id,
            base_vtable_id: None,
            field_offsets: HashMap::new(),
            vtable_methods: Vec::new(),
            itable_methods: HashMap::new(),
            reference_field_offsets: Vec::new(),
            weak_field_offsets: Vec::new(),
            method_slots: HashMap::new(),
            field_types: HashMap::new(),
            is_extern: false,
        }
    }
}
