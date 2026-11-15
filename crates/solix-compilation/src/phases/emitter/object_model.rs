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

    let mut field_types: Vec<(usize, u8)> = layout
        .field_types
        .iter()
        .map(|(&field_offset, &tag)| (field_offset, tag))
        .collect();
    field_types.sort_by_key(|(field_offset, _)| *field_offset);
    compiled.field_types = field_types;

    let mut field_names: Vec<(usize, String, String)> = Vec::new();
    for (field_name, &field_offset) in &layout.field_offsets {
        let type_tag = layout.field_types.get(&field_offset).copied().unwrap_or(0);
        let type_name = match type_tag {
            1 => "int8",
            2 => "uint8",
            3 => "int16",
            4 => "uint16",
            5 => "int32",
            6 => "uint32",
            7 => "int64",
            8 => "uint64",
            9 => "f32",
            10 => "f64",
            11 => "bool",
            12 => "char",
            13 => "string",
            14 => "object",
            _ => "unknown",
        };
        field_names.push((field_offset, field_name.clone(), type_name.to_string()));
    }
    field_names.sort_by_key(|(field_offset, _, _)| *field_offset);
    compiled.field_names = field_names;

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
