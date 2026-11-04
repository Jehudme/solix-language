//! Runtime configuration options and builder API.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::native::NativeFunction;
use solix_common::bytecode::BinaryProgram;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Default dynamic heap size: 64 megabytes.
pub const DEFAULT_HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Default maximum call stack depth: 65,536 call frames.
pub const DEFAULT_MAX_CALL_STACK_DEPTH: usize = 65_536;

/// Configuration options governing virtual machine execution.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeOptions {
    /// Optional symbol override for the entry point function (e.g. `"main"` or `"my_module::run"`).
    pub entry_point: Option<String>,
    /// In-memory binary programs loaded directly into the runtime context.
    pub binaries: Vec<BinaryProgram>,
    /// File paths of `.slxb` compiled binaries to load upon context creation.
    pub binary_paths: Vec<PathBuf>,
    /// Dynamic heap memory capacity in bytes.
    pub heap_size: usize,
    /// Maximum allowable call frame recursion depth before triggering a stack overflow.
    pub max_call_stack_depth: usize,
    /// Command line arguments forwarded into the guest VM program.
    pub args: Vec<String>,
    /// Execution mode: when `true`, bounds-checking and defensive invariant validations are enforced;
    /// when `false` (default), the high-performance raw-pointer inlined engine is used.
    pub safe_mode: bool,
    /// Sandboxing flag disabling non-sandboxed host native operations and filesystem access.
    pub sandboxed: bool,
    /// Custom host native Rust functions registered for execution.
    pub native_functions: HashMap<String, NativeFunction>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            entry_point: None,
            binaries: Vec::new(),
            binary_paths: Vec::new(),
            heap_size: DEFAULT_HEAP_SIZE,
            max_call_stack_depth: DEFAULT_MAX_CALL_STACK_DEPTH,
            args: Vec::new(),
            safe_mode: false,
            sandboxed: false,
            native_functions: HashMap::new(),
        }
    }
}

impl RuntimeOptions {
    /// Creates a new default configuration with 64MB heap and 65,536 frame call stack depth.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a custom host native function.
    #[must_use]
    pub fn with_native_function(mut self, name: impl Into<String>, func: NativeFunction) -> Self {
        self.native_functions.insert(name.into(), func);
        self
    }

    /// Sets the custom entry point symbol override.
    #[must_use]
    pub fn with_entry_point(mut self, symbol: impl Into<String>) -> Self {
        self.entry_point = Some(symbol.into());
        self
    }

    /// Sets an optional entry point symbol.
    #[must_use]
    pub fn with_optional_entry_point(mut self, symbol: Option<String>) -> Self {
        self.entry_point = symbol;
        self
    }

    /// Appends a compiled `BinaryProgram` directly to the execution unit list.
    #[must_use]
    pub fn with_binary(mut self, binary: BinaryProgram) -> Self {
        self.binaries.push(binary);
        self
    }

    /// Appends multiple compiled `BinaryProgram` instances.
    #[must_use]
    pub fn with_binaries(mut self, binaries: impl IntoIterator<Item = BinaryProgram>) -> Self {
        self.binaries.extend(binaries);
        self
    }

    /// Deserializes a binary from raw `SLXB` bytes and adds it to the unit list.
    ///
    /// # Errors
    /// Returns `RuntimeError::BinaryLoadError` if deserialization fails.
    pub fn with_binary_bytes(mut self, bytes: &[u8]) -> Result<Self, RuntimeError> {
        let binary =
            BinaryProgram::from_bytes(bytes).map_err(|err| RuntimeError::BinaryLoadError {
                reason: format!("{err:?}"),
            })?;
        self.binaries.push(binary);
        Ok(self)
    }

    /// Appends a binary file path to load during context creation.
    #[must_use]
    pub fn with_binary_path(mut self, path: impl AsRef<Path>) -> Self {
        self.binary_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Configures the dynamic heap memory capacity in bytes.
    #[must_use]
    pub fn with_heap_size(mut self, heap_size: usize) -> Self {
        self.heap_size = heap_size;
        self
    }

    /// Configures the maximum allowable call frame stack depth.
    #[must_use]
    pub fn with_max_call_stack_depth(mut self, max_depth: usize) -> Self {
        self.max_call_stack_depth = max_depth;
        self
    }

    /// Configures the command line arguments forwarded into the guest VM.
    #[must_use]
    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Sets whether safe mode (bounds-checked execution) is enabled.
    #[must_use]
    pub fn with_safe_mode(mut self, safe_mode: bool) -> Self {
        self.safe_mode = safe_mode;
        self
    }

    /// Sets whether the runtime executes in sandboxed mode.
    #[must_use]
    pub fn with_sandboxed(mut self, sandboxed: bool) -> Self {
        self.sandboxed = sandboxed;
        self
    }
}
