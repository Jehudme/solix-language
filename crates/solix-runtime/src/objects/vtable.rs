//! Virtual method table and interface dispatch structures.

#![forbid(unsafe_code)]

use solix_common::bytecode::CompiledClass;

/// Resolves method dispatch target function ID through an object class's virtual table.
#[must_use]
pub fn resolve_virtual_method(class: &CompiledClass, slot: usize) -> Option<u32> {
    class.vtable.get(slot).copied()
}

/// Resolves an interface method target function ID through the class's interface table.
#[must_use]
pub fn resolve_interface_method(class: &CompiledClass, iface_id: u32, slot: usize) -> Option<u32> {
    for (id, methods) in &class.itable {
        if *id == iface_id {
            return methods.get(slot).copied();
        }
    }
    // Fallback to virtual table index if itable is empty or identical
    class.vtable.get(slot).copied()
}
