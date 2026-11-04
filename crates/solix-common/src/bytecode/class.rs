//! Compiled class descriptors, vtables, and flat interface dispatch caches.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Compiled object-oriented class descriptor used for memory allocation and dynamic dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledClass {
    /// Fully qualified class symbol name (e.g. `solix::models::User`).
    pub symbol: String,
    /// Unique virtual table index assigned to this class.
    pub vtable_id: u32,
    /// VTable index of the superclass, if this class inherits from a base class.
    pub base_vtable_id: Option<u32>,
    /// Heap allocation payload size in bytes required for instances of this class.
    pub instance_size: usize,
    /// Byte offsets of reference-counted fields within the object's instance memory.
    pub reference_field_offsets: Vec<usize>,
    /// Byte offsets of non-owning weak reference fields within the object's instance memory.
    pub weak_field_offsets: Vec<usize>,
    /// Virtual method table mapping virtual slot indices to function indices.
    pub vtable: Vec<u32>,
    /// Flat interface dispatch table: maps `interface_id` to the list of vtable method indices.
    pub itable: Vec<(u32, Vec<u32>)>,
    /// Optional function index of the user-defined destructor (`~Class()`).
    pub destructor_id: Option<u32>,
    /// Explicit mapping of field byte offsets to their primitive or reference type tags.
    pub field_types: Vec<(usize, u8)>,
}

impl CompiledClass {
    /// Constructs a new `CompiledClass` descriptor with initialized tables.
    #[must_use]
    pub fn new(
        symbol: impl Into<String>,
        vtable_id: u32,
        base_vtable_id: Option<u32>,
        instance_size: usize,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            vtable_id,
            base_vtable_id,
            instance_size,
            reference_field_offsets: Vec::new(),
            weak_field_offsets: Vec::new(),
            vtable: Vec::new(),
            itable: Vec::new(),
            destructor_id: None,
            field_types: Vec::new(),
        }
    }

    /// Returns `true` if this class has no destructor and holds no owning reference fields.
    #[must_use]
    pub fn is_pod(&self) -> bool {
        self.destructor_id.is_none() && self.reference_field_offsets.is_empty()
    }

    /// Looks up the type tag for the field at `offset`.
    #[must_use]
    pub fn get_field_type(&self, offset: usize) -> Option<u8> {
        self.field_types
            .iter()
            .find(|(field_offset, _)| *field_offset == offset)
            .map(|(_, tag)| *tag)
    }
}
