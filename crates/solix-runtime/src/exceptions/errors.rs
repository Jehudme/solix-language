//! Runtime error types and diagnostic definitions.

#![forbid(unsafe_code)]

use thiserror::Error;

/// Enumeration of all runtime failure modes encountered during VM execution.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuntimeError {
    /// Out of memory during dynamic heap allocation.
    #[error("out of memory: failed to allocate {requested} bytes ({available} bytes available)")]
    OutOfMemory { requested: usize, available: usize },

    /// Call stack recursion limit exceeded.
    #[error(
        "call stack overflow: depth reached {current_depth} exceeding max limit of {max_depth}"
    )]
    StackOverflow {
        current_depth: usize,
        max_depth: usize,
    },

    /// Division or modulo by zero.
    #[error("division by zero")]
    DivideByZero,

    /// Array or tuple index out of bounds.
    #[error("index out of bounds: index {index} is invalid for collection of length {length}")]
    IndexOutOfBounds { index: i64, length: usize },

    /// Attempted dereference of a null reference pointer.
    #[error("null reference dereference")]
    NullReferenceAccess,

    /// Invalid dynamic type cast.
    #[error("invalid cast: cannot cast instance of '{from}' to '{to}'")]
    InvalidCast { from: String, to: String },

    /// Function identifier or symbol could not be resolved.
    #[error("function not found: '{symbol_or_id}'")]
    FunctionNotFound { symbol_or_id: String },

    /// Entry point could not be located or resolved.
    #[error("entry point '{symbol}' not found in loaded binaries")]
    EntryPointNotFound { symbol: String },

    /// Invalid instruction word or unrecognized opcode.
    #[error("invalid instruction at PC {pc}: raw value 0x{raw:016X}")]
    InvalidInstruction { pc: u32, raw: u64 },

    /// Unrecognized opcode byte encountered.
    #[error("unrecognized opcode byte 0x{opcode:02X} at PC {pc}")]
    InvalidOpCode { pc: u32, opcode: u8 },

    /// Invalid field byte offset within an object layout.
    #[error("invalid field offset {offset} for object of instance size {object_size}")]
    InvalidFieldOffset { offset: usize, object_size: usize },

    /// Static variable slot offset out of bounds.
    #[error("invalid static slot offset {offset} (total static partition size {total_static})")]
    InvalidStaticOffset { offset: u32, total_static: u32 },

    /// Binary parsing or loading failure.
    #[error("binary load error: {reason}")]
    BinaryLoadError { reason: String },

    /// Graceful process termination via Exit opcode.
    #[error("process exited with status code {code}")]
    Exit { code: i32 },

    /// Unhandled user exception thrown in Solix code.
    #[error("unhandled exception: {message}")]
    UserException { message: String },

    /// Host native callback execution failed.
    #[error("native call error in '{name}': {reason}")]
    NativeFunctionError { name: String, reason: String },

    /// Sandboxed execution security policy violation.
    #[error("security violation: {action} is prohibited in sandboxed mode")]
    SecurityViolation { action: String },

    /// Value type mismatch encountered during instruction execution.
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// Assertion failed via `assert` statement.
    #[error("assertion failed: {message}")]
    AssertionFailed { message: String },

    /// Unfreed memory leak detected upon shutdown audit.
    #[error("memory leak detected: {unallocated_count} live allocations remaining:\n{report}")]
    MemoryLeak {
        unallocated_count: usize,
        report: String,
    },

    /// Invalid entry point method signature.
    #[error(
        "invalid entry point signature: {symbol} has {param_count} parameters (expected 0 or 1)"
    )]
    InvalidEntryPointSignature { symbol: String, param_count: u16 },

    /// Native shared library loading or symbol resolution failure.
    #[error("SharedLibraryException: Failed to load library '{path}': {reason}")]
    SharedLibraryError { path: String, reason: String },
}

/// Canonical numerical error codes emitted on abnormal VM termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum RuntimeErrorCode {
    /// Normal successful execution.
    Success = 0,
    /// Out of memory during dynamic allocation.
    OutOfMemory = 1,
    /// Stack recursion limit exceeded.
    StackOverflow = 2,
    /// Division or modulo by zero.
    DivideByZero = 3,
    /// Array or tuple index out of bounds.
    IndexOutOfBounds = 4,
    /// Null reference access.
    NullReferenceAccess = 5,
    /// Invalid class/type cast.
    InvalidCast = 6,
    /// Function not found.
    FunctionNotFound = 7,
    /// Entry point symbol not found.
    EntryPointNotFound = 8,
    /// Invalid instruction word.
    InvalidInstruction = 9,
    /// Unrecognized opcode byte.
    InvalidOpCode = 10,
    /// Invalid field byte offset.
    InvalidFieldOffset = 11,
    /// Invalid static variable slot offset.
    InvalidStaticOffset = 12,
    /// Binary parsing or loading failure.
    BinaryLoadError = 13,
    /// Unhandled user exception.
    UserException = 14,
    /// Native host function error.
    NativeFunctionError = 15,
    /// Security violation in sandboxed mode.
    SecurityViolation = 16,
    /// Value type mismatch.
    TypeMismatch = 17,
    /// Assertion failed.
    AssertionFailed = 18,
    /// Memory leak upon shutdown.
    MemoryLeak = 19,
    /// Invalid entry point method signature.
    InvalidEntryPointSignature = 20,
    /// Explicit exit with non-zero code.
    ExplicitExit = 21,
    /// Native shared library load or invocation failure.
    SharedLibraryError = 22,
}

impl RuntimeErrorCode {
    /// Returns the signed 32-bit integer error code value.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl RuntimeError {
    /// Formats a concise error string.
    #[must_use]
    pub fn message(&self) -> String {
        format!("{self}")
    }

    /// Returns the canonical numerical `RuntimeErrorCode` associated with this runtime failure mode.
    #[must_use]
    pub const fn error_code(&self) -> RuntimeErrorCode {
        match self {
            Self::OutOfMemory { .. } => RuntimeErrorCode::OutOfMemory,
            Self::StackOverflow { .. } => RuntimeErrorCode::StackOverflow,
            Self::DivideByZero => RuntimeErrorCode::DivideByZero,
            Self::IndexOutOfBounds { .. } => RuntimeErrorCode::IndexOutOfBounds,
            Self::NullReferenceAccess => RuntimeErrorCode::NullReferenceAccess,
            Self::InvalidCast { .. } => RuntimeErrorCode::InvalidCast,
            Self::FunctionNotFound { .. } => RuntimeErrorCode::FunctionNotFound,
            Self::EntryPointNotFound { .. } => RuntimeErrorCode::EntryPointNotFound,
            Self::InvalidInstruction { .. } => RuntimeErrorCode::InvalidInstruction,
            Self::InvalidOpCode { .. } => RuntimeErrorCode::InvalidOpCode,
            Self::InvalidFieldOffset { .. } => RuntimeErrorCode::InvalidFieldOffset,
            Self::InvalidStaticOffset { .. } => RuntimeErrorCode::InvalidStaticOffset,
            Self::BinaryLoadError { .. } => RuntimeErrorCode::BinaryLoadError,
            Self::Exit { code } => {
                if *code == 0 {
                    RuntimeErrorCode::Success
                } else {
                    RuntimeErrorCode::ExplicitExit
                }
            }
            Self::UserException { .. } => RuntimeErrorCode::UserException,
            Self::NativeFunctionError { .. } => RuntimeErrorCode::NativeFunctionError,
            Self::SecurityViolation { .. } => RuntimeErrorCode::SecurityViolation,
            Self::TypeMismatch { .. } => RuntimeErrorCode::TypeMismatch,
            Self::AssertionFailed { .. } => RuntimeErrorCode::AssertionFailed,
            Self::MemoryLeak { .. } => RuntimeErrorCode::MemoryLeak,
            Self::InvalidEntryPointSignature { .. } => RuntimeErrorCode::InvalidEntryPointSignature,
            Self::SharedLibraryError { .. } => RuntimeErrorCode::SharedLibraryError,
        }
    }
}
