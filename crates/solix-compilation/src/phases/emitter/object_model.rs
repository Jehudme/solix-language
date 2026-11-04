//! Object model translation, ClassLayout compilation, and dynamic dispatch wiring.

#![forbid(unsafe_code)]

use crate::phases::binder::layout::ClassLayout;
use crate::phases::binder::symbols::SymbolId;
use solix_common::bytecode::CompiledClass;
use std::collections::HashMap;

/// Compiles a semantically bound `ClassLayout` into a runtime `CompiledClass` descriptor.
#[must_use]
pub fn compile_class_layout(
    layout: &ClassLayout,
    symbol_to_func_id: &HashMap<SymbolId, u32>,
    iface_to_id: &HashMap<String, u32>,
    destructor_id: Option<u32>,
) -> CompiledClass {
    let mut compiled = CompiledClass::new(
        layout.class_name.clone(),
        layout.vtable_id,
        layout.base_vtable_id,
        layout.instance_size_bytes,
    );

    compiled.reference_field_offsets = layout.reference_field_offsets.clone();
    compiled.weak_field_offsets = layout.weak_field_offsets.clone();
    compiled.destructor_id = destructor_id;

    // Compile VTable slots
    compiled.vtable = layout
        .vtable_methods
        .iter()
        .map(|sym_id| symbol_to_func_id.get(sym_id).copied().unwrap_or(0))
        .collect();

    // Compile flat ITable caches
    for (iface_name, methods) in &layout.itable_methods {
        let iface_id = iface_to_id.get(iface_name).copied().unwrap_or(0);
        let method_vtable_slots = methods
            .iter()
            .map(|sym_id| symbol_to_func_id.get(sym_id).copied().unwrap_or(0))
            .collect();
        compiled.itable.push((iface_id, method_vtable_slots));
    }

    compiled
}
