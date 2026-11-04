//! Constant pool builder and deduplication for the bytecode emitter.

#![forbid(unsafe_code)]

use solix_common::bytecode::{ConstantPool, ConstantPoolValue};
use solix_common::diagnostics::EmissionError;

/// Manager responsible for deduplicating numeric and string constants during bytecode emission.
#[derive(Debug, Clone, Default)]
pub struct ConstantPoolBuilder {
    inner: ConstantPool,
}

impl ConstantPoolBuilder {
    /// Creates a new empty constant pool builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ConstantPool::new(),
        }
    }

    /// Deduplicates and registers a 64-bit integer, returning its pool index.
    pub fn add_i64(&mut self, value: i64) -> Result<u16, EmissionError> {
        self.inner.add_i64(value)
    }

    /// Deduplicates and registers a 64-bit float, returning its pool index.
    pub fn add_f64(&mut self, value: f64) -> Result<u16, EmissionError> {
        self.inner.add_f64(value)
    }

    /// Deduplicates and registers a string literal, returning its pool index.
    pub fn add_string(&mut self, value: &str) -> Result<u16, EmissionError> {
        self.inner.add_string(value)
    }

    /// Retrieves a reference to the constant value at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&ConstantPoolValue> {
        self.inner.get(index)
    }

    /// Returns the number of unique constants registered in the pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if no constants have been registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Finalizes the builder and extracts the compiled `ConstantPool`.
    #[must_use]
    pub fn finish(self) -> ConstantPool {
        self.inner
    }
}
