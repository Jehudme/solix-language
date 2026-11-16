//! Runtime configuration options and builder API.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::native::NativeFunction;
use solix_common::bytecode::BinaryProgram;
use solix_common::solver::ModuleSolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default dynamic heap size: 64 megabytes.
pub const DEFAULT_HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Default maximum call stack depth: 65,536 call frames.
pub const DEFAULT_MAX_CALL_STACK_DEPTH: usize = 65_536;

/// Configuration options governing virtual machine execution.
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    /// Optional symbol override for the entry point function (e.g. `"main"` or `"my_module::run"`).
    pub entry_point: Option<String>,
    /// In-memory binary programs loaded directly into the runtime context.
    pub binaries: Vec<BinaryProgram>,
    /// File paths of `.slxb` compiled binaries to load upon context creation.
    pub binary_paths: Vec<PathBuf>,
    /// File paths of native dynamic shared libraries (`.so`, `.dll`, `.dylib`) to load upon context creation.
    pub native_libraries: Vec<PathBuf>,
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
    /// Container packaging kind (e.g. "slxpac", "slxarc", "slxb", "source").
    pub container_kind: Option<String>,
    /// Root directory path holding extracted container resources.
    pub container_resource_root: Option<PathBuf>,
    /// Total count of bundled resources in the active container.
    pub container_resource_count: u32,
    /// Total byte length of bundled resources in the active container.
    pub container_resource_bytes: u64,
    /// Custom host native Rust functions registered for execution.
    pub native_functions: HashMap<String, NativeFunction>,
    /// Optional pluggable module solver for auto-fetching dependencies on PC B.
    pub solver: Option<Arc<dyn ModuleSolver>>,
    /// Whether to verify cryptographic SHA-256 checksums on dependencies and native libraries (default true; false when --no-verify).
    pub verify_checksums: bool,
    /// Optional thread-safe output sink for capturing guest standard output.
    pub output_sink: Option<OutputSink>,
}

/// Callback type for capturing guest standard output streams.
pub type OutputCallback = Arc<dyn Fn(&str, bool) + Send + Sync>;

/// Thread-safe sink for capturing guest standard output streams (e.g. for DAP events).
#[derive(Clone)]
pub struct OutputSink(pub OutputCallback);

impl std::fmt::Debug for OutputSink {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "OutputSink(<closure>)")
    }
}

impl PartialEq for OutputSink {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl PartialEq for RuntimeOptions {
    fn eq(&self, other: &Self) -> bool {
        self.entry_point == other.entry_point
            && self.binaries == other.binaries
            && self.binary_paths == other.binary_paths
            && self.native_libraries == other.native_libraries
            && self.heap_size == other.heap_size
            && self.max_call_stack_depth == other.max_call_stack_depth
            && self.args == other.args
            && self.safe_mode == other.safe_mode
            && self.sandboxed == other.sandboxed
            && self.native_functions == other.native_functions
            && self.verify_checksums == other.verify_checksums
            && match (&self.solver, &other.solver) {
                (Some(solver_a), Some(solver_b)) => Arc::ptr_eq(solver_a, solver_b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            entry_point: None,
            binaries: Vec::new(),
            binary_paths: Vec::new(),
            native_libraries: Vec::new(),
            heap_size: DEFAULT_HEAP_SIZE,
            max_call_stack_depth: DEFAULT_MAX_CALL_STACK_DEPTH,
            args: Vec::new(),
            safe_mode: false,
            sandboxed: false,
            container_kind: None,
            container_resource_root: None,
            container_resource_count: 0,
            container_resource_bytes: 0,
            native_functions: HashMap::new(),
            solver: None,
            verify_checksums: true,
            output_sink: None,
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

    /// Appends a native dynamic shared library path to load during context creation.
    #[must_use]
    pub fn with_native_library(mut self, path: impl AsRef<Path>) -> Self {
        self.native_libraries.push(path.as_ref().to_path_buf());
        self
    }

    /// Appends multiple native dynamic shared library paths to load during context creation.
    #[must_use]
    pub fn with_native_libraries(
        mut self,
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> Self {
        self.native_libraries
            .extend(paths.into_iter().map(|path| path.as_ref().to_path_buf()));
        self
    }

    /// Configures whether to enforce cryptographic checksum validation.
    #[must_use]
    pub fn with_verify_checksums(mut self, verify: bool) -> Self {
        self.verify_checksums = verify;
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

    /// Configures the pluggable module solver for runtime dependency resolution.
    #[must_use]
    pub fn with_solver(mut self, solver: Arc<dyn ModuleSolver>) -> Self {
        self.solver = Some(solver);
        self
    }

    /// Sets the container packaging kind (e.g. "slxpac", "slxarc", "slxb").
    #[must_use]
    pub fn with_container_kind(mut self, kind: impl Into<String>) -> Self {
        self.container_kind = Some(kind.into());
        self
    }

    /// Sets the root directory path holding extracted container resources.
    #[must_use]
    pub fn with_container_resource_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.container_resource_root = Some(path.into());
        self
    }

    /// Sets the container resource count and total bytes.
    #[must_use]
    pub fn with_container_resources(mut self, count: u32, total_bytes: u64) -> Self {
        self.container_resource_count = count;
        self.container_resource_bytes = total_bytes;
        self
    }

    /// Configures the thread-safe output sink for capturing guest standard output streams.
    #[must_use]
    pub fn with_output_sink(mut self, sink: OutputSink) -> Self {
        self.output_sink = Some(sink);
        self
    }
}
