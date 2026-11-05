//! Bytecode emission and binary container diagnostics.

#![forbid(unsafe_code)]

use miette::Diagnostic;

/// Structured diagnostic errors emitted during bytecode emission and binary container serialization/deserialization.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Diagnostic)]
pub enum EmissionError {
    #[error("Constant pool limit exceeded: {count} exceeds maximum of {max} constants")]
    #[diagnostic(
        code(solix::emitter::constant_pool_overflow),
        help("Reduce the number of distinct constants in the compilation unit.")
    )]
    ConstantPoolOverflow { count: usize, max: usize },

    #[error(
        "Register limit exceeded: register index {register_index} exceeds maximum register ceiling of {max}"
    )]
    #[diagnostic(
        code(solix::emitter::register_overflow),
        help("Refactor complex expressions or deep local variable scopes into smaller functions.")
    )]
    RegisterOverflow { register_index: usize, max: usize },

    #[error(
        "Argument count limit exceeded: function call has {argument_count} arguments (maximum allowed is {max})"
    )]
    #[diagnostic(
        code(solix::emitter::argument_count_overflow),
        help("Reduce the parameter list or pass complex parameters inside an object or struct.")
    )]
    ArgumentCountOverflow { argument_count: usize, max: usize },

    #[error("Jump offset out of bounds: offset {offset} exceeds signed 32-bit branch range")]
    #[diagnostic(
        code(solix::emitter::jump_offset_overflow),
        help("Split very large function bodies into smaller helper routines.")
    )]
    JumpOffsetOverflow { offset: i64 },

    #[error("Invalid binary magic header: expected 'SLXB', found {found:?}")]
    #[diagnostic(
        code(solix::emitter::invalid_magic),
        help("Ensure the input file is a valid Solix compiled binary (*.slxb).")
    )]
    InvalidMagic { found: [u8; 4] },

    #[error(
        "Unsupported binary container version: expected version {expected_major}.{expected_minor}, found {found_major}.{found_minor}"
    )]
    #[diagnostic(
        code(solix::emitter::unsupported_version),
        help("Recompile the source files using the current compiler version.")
    )]
    UnsupportedVersion {
        expected_major: u16,
        expected_minor: u16,
        found_major: u16,
        found_minor: u16,
    },

    #[error(
        "Unexpected end of binary stream while decoding section '{section}': required {required} bytes, available {available}"
    )]
    #[diagnostic(
        code(solix::emitter::unexpected_eof),
        help("The compiled binary file appears truncated or corrupted.")
    )]
    UnexpectedEof {
        section: String,
        required: usize,
        available: usize,
    },

    #[error("Invalid opcode byte {raw_byte:#04x} at instruction offset {instruction_index}")]
    #[diagnostic(
        code(solix::emitter::invalid_opcode),
        help("The binary file contains an invalid or unrecognized virtual machine opcode.")
    )]
    InvalidOpcode {
        raw_byte: u8,
        instruction_index: usize,
    },

    #[error("Corrupt binary container in section '{section}': {details}")]
    #[diagnostic(
        code(solix::emitter::corrupt_binary),
        help("The binary file does not conform to the SLXB format specification.")
    )]
    CorruptBinaryContainer { section: String, details: String },

    #[error("Program entry point '{entry_symbol}' could not be resolved")]
    #[diagnostic(
        code(solix::emitter::missing_entry_point),
        help("Ensure a valid 'main' function is declared in the source program.")
    )]
    MissingEntryPoint { entry_symbol: String },

    #[error("Duplicate function symbol '{symbol}' detected in binary module")]
    #[diagnostic(
        code(solix::emitter::duplicate_symbol),
        help("Rename one of the conflicting function symbols.")
    )]
    DuplicateSymbol { symbol: String },

    #[error("Symbol '{symbol}' not found in binary unit symbol table")]
    #[diagnostic(
        code(solix::emitter::symbol_not_found),
        help("Verify that the referenced function or static variable is exported.")
    )]
    SymbolNotFound { symbol: String },

    #[error("Malformed export manifest in binary container: {reason}")]
    #[diagnostic(
        code(solix::emitter::malformed_export_manifest),
        help("Ensure the binary container has a valid serialized export manifest.")
    )]
    MalformedExportManifest { reason: String },

    #[error("Project archive or container error: {reason}")]
    #[diagnostic(
        code(solix::emitter::container_serialization_failed),
        help("Check project structure, manifest.toml, or archive container integrity.")
    )]
    ContainerSerializationFailed { reason: String },
}

impl EmissionError {
    /// Returns the standardized compiler error code string (e.g. `"E0301"`).
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::ConstantPoolOverflow { .. } => "E0301",
            Self::RegisterOverflow { .. } => "E0302",
            Self::ArgumentCountOverflow { .. } => "E0303",
            Self::JumpOffsetOverflow { .. } => "E0304",
            Self::InvalidMagic { .. } => "E0305",
            Self::UnsupportedVersion { .. } => "E0306",
            Self::UnexpectedEof { .. } => "E0307",
            Self::InvalidOpcode { .. } => "E0308",
            Self::CorruptBinaryContainer { .. } => "E0309",
            Self::MissingEntryPoint { .. } => "E0310",
            Self::DuplicateSymbol { .. } => "E0311",
            Self::SymbolNotFound { .. } => "E0312",
            Self::MalformedExportManifest { .. } => "E0313",
            Self::ContainerSerializationFailed { .. } => "E0314",
        }
    }
}
