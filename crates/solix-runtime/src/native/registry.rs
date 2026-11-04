//! Native foreign function interface and host Rust callback registry.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use std::collections::HashMap;

/// Type signature of a host native Rust callback invokable by the virtual machine.
pub type NativeFunction = fn(&mut VirtualMachine, &[Value]) -> Result<Value, RuntimeError>;

/// Central repository of host native callbacks and core language builtins.
#[derive(Default, Clone, Debug)]
pub struct NativeRegistry {
    /// Maps canonical native symbol name to its host Rust callback pointer.
    pub functions: HashMap<String, NativeFunction>,
}

impl NativeRegistry {
    /// Creates a new empty `NativeRegistry`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Registers a custom host native callback.
    pub fn register(&mut self, symbol: impl Into<String>, callback: NativeFunction) {
        self.functions.insert(symbol.into(), callback);
    }

    /// Looks up a registered host callback by symbol.
    #[must_use]
    pub fn get(&self, symbol: &str) -> Option<&NativeFunction> {
        self.functions.get(symbol)
    }

    /// Creates a registry pre-populated with standard built-in functions.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        crate::native::builtins::register_all_builtins(&mut reg);
        reg
    }
}
