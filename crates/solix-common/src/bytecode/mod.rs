//! Register-based bytecode instruction set, function descriptors, and SLXB binary format.

#![forbid(unsafe_code)]

pub mod binary;
pub mod class;
pub mod function;
pub mod instruction;
pub mod manifest;
pub mod opcodes;

pub use binary::{
    BinaryProgram, ConstantPool, ConstantPoolValue, MAGIC, VERSION_MAJOR, VERSION_MINOR,
};
pub use class::CompiledClass;
pub use function::{
    CompiledFunction, ExceptionHandlerEntry, LocalVariableDebugInfo, SourceMapEntry,
};
pub use instruction::Instruction;
pub use manifest::{
    DeclarationModifier, ExportManifest, ExportedClass, ExportedEnum, ExportedField,
    ExportedFunction, ExportedGenericTemplate, ExportedParameter, ExportedStatic,
    extract_api_manifest,
};
pub use opcodes::OpCode;
