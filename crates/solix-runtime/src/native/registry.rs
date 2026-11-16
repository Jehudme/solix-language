//! Native foreign function interface and host Rust callback registry.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::exceptions::RuntimeError;
use crate::native::ffi::NativeCFunctionPtr;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use std::collections::HashMap;

/// Type signature of a host native Rust callback invokable by the virtual machine.
pub type NativeFunction = fn(&mut VirtualMachine, &[Value]) -> Result<Value, RuntimeError>;

/// Unified native callback representation encompassing host Rust function pointers and loaded C-ABI function pointers.
#[derive(Clone, Copy, Debug)]
pub enum NativeCallback {
    /// Host Rust function pointer callback.
    Rust(NativeFunction),
    /// Dynamic C-ABI function pointer callback.
    CAbi(NativeCFunctionPtr),
}

impl NativeCallback {
    /// Invokes the native callback with virtual machine context and arguments.
    pub fn invoke(
        &self,
        vm: &mut VirtualMachine,
        arguments: &[Value],
    ) -> Result<Value, RuntimeError> {
        match self {
            Self::Rust(rust_function) => rust_function(vm, arguments),
            Self::CAbi(c_function) => crate::native::ffi::invoke_c_native(*c_function, vm, arguments),
        }
    }
}

impl From<NativeFunction> for NativeCallback {
    fn from(rust_function: NativeFunction) -> Self {
        Self::Rust(rust_function)
    }
}

impl From<NativeCFunctionPtr> for NativeCallback {
    fn from(c_function: NativeCFunctionPtr) -> Self {
        Self::CAbi(c_function)
    }
}

/// Central repository of host native callbacks and core language builtins.
#[derive(Default, Clone, Debug)]
pub struct NativeRegistry {
    /// Maps canonical native symbol name to its host Rust or dynamic C callback.
    pub functions: HashMap<String, NativeCallback>,
}

impl NativeRegistry {
    /// Creates a new empty `NativeRegistry`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Registers a custom host native Rust callback.
    pub fn register(&mut self, symbol: impl Into<String>, callback: NativeFunction) {
        self.functions
            .insert(symbol.into(), NativeCallback::Rust(callback));
    }

    /// Registers a dynamic C-ABI callback.
    pub fn register_c(&mut self, symbol: impl Into<String>, callback: NativeCFunctionPtr) {
        self.functions
            .insert(symbol.into(), NativeCallback::CAbi(callback));
    }

    /// Registers a generic native callback.
    pub fn register_callback(&mut self, symbol: impl Into<String>, callback: NativeCallback) {
        self.functions.insert(symbol.into(), callback);
    }

    /// Looks up a registered host callback by symbol.
    #[must_use]
    pub fn get(&self, symbol: &str) -> Option<&NativeCallback> {
        self.functions.get(symbol)
    }

    /// Creates a registry pre-populated with standard built-in functions.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        crate::native::builtins::register_all_builtins(&mut registry);
        registry
    }
}
