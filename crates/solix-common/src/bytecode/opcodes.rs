//! Register-based virtual machine opcodes for the Solix runtime.

#![forbid(unsafe_code)]

use crate::diagnostics::EmissionError;
use serde::{Deserialize, Serialize};

/// Strongly-typed enumeration of all 8-bit instruction opcodes supported by the virtual register VM.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // 0x00 .. 0x0F: Constants & Register Movement
    Nop = 0x00,
    Mov = 0x01,
    LoadI32 = 0x02,
    LoadI64 = 0x03,
    LoadF64 = 0x04,
    LoadBool = 0x05,
    LoadNull = 0x06,
    LoadConst = 0x07,

    // 0x10 .. 0x20: Arithmetic Operations
    AddI32 = 0x10,
    AddI64 = 0x11,
    AddF64 = 0x12,
    SubI32 = 0x13,
    SubI64 = 0x14,
    SubF64 = 0x15,
    MulI32 = 0x16,
    MulI64 = 0x17,
    MulF64 = 0x18,
    DivI32 = 0x19,
    DivI64 = 0x1A,
    DivF64 = 0x1B,
    ModI32 = 0x1C,
    ModI64 = 0x1D,
    NegI32 = 0x1E,
    NegI64 = 0x1F,
    NegF64 = 0x20,

    // 0x21 .. 0x2F: Bitwise Operations
    Band = 0x21,
    Bor = 0x22,
    Bxor = 0x23,
    Bnot = 0x24,
    Shl = 0x25,
    Shr = 0x26,

    // 0x30 .. 0x47: Comparisons
    EqI32 = 0x30,
    EqI64 = 0x31,
    EqF64 = 0x32,
    EqBool = 0x33,
    EqString = 0x34,
    EqRef = 0x35,
    NeqI32 = 0x36,
    NeqI64 = 0x37,
    NeqF64 = 0x38,
    NeqBool = 0x39,
    NeqString = 0x3A,
    NeqRef = 0x3B,
    LtI32 = 0x3C,
    LtI64 = 0x3D,
    LtF64 = 0x3E,
    LteI32 = 0x3F,
    LteI64 = 0x40,
    LteF64 = 0x41,
    GtI32 = 0x42,
    GtI64 = 0x43,
    GtF64 = 0x44,
    GteI32 = 0x45,
    GteI64 = 0x46,
    GteF64 = 0x47,

    // 0x48: Boolean Inversion
    Not = 0x48,

    // 0x50 .. 0x5F: Control Flow & Branching
    Jump = 0x50,
    JumpIfTrue = 0x51,
    JumpIfFalse = 0x52,
    Switch = 0x53,

    // 0x60 .. 0x6F: Function Invocations & Returns
    CallDirect = 0x60,
    CallVirtual = 0x61,
    CallInterface = 0x62,
    CallSymbol = 0x63,
    CallCallable = 0x64,
    Return = 0x65,
    ReturnValue = 0x66,
    Exit = 0x67,
    CallExtern = 0x68,

    // 0x70 .. 0x7F: Object Model, Instance Fields & Static Memory
    AllocObject = 0x70,
    GetField = 0x71,
    SetField = 0x72,
    GetStatic = 0x73,
    SetStatic = 0x74,
    GetStaticGlobal = 0x75,
    GetStaticDirect = 0x76,
    SetStaticGlobal = 0x77,
    SetStaticDirect = 0x78,
    AllocObjectExtern = 0x79,

    // 0x80 .. 0x8F: Arrays & Tuples
    AllocArray = 0x80,
    GetArray = 0x81,
    SetArray = 0x82,
    ArrayLength = 0x83,
    AllocTuple = 0x84,
    GetTuple = 0x85,

    // 0x90 .. 0x9F: Closures & Lambdas
    PackClosure = 0x90,
    UnpackCaptures = 0x91,

    // 0xA0 .. 0xAF: Type Reflection & Numeric Conversion
    Instanceof = 0xA0,
    CastCheck = 0xA1,
    ConvIToF = 0xA2,
    ConvFToI = 0xA3,
    TupleInstanceof = 0xA4,

    // 0xB0 .. 0xBF: Exception Handling
    Throw = 0xB0,

    // 0xC0 .. 0xCF: ARC Reference Counting
    IncRef = 0xC0,
    DecRef = 0xC1,
}

impl OpCode {
    /// Attempts to convert a raw 8-bit unsigned integer into a valid `OpCode`.
    ///
    /// # Errors
    /// Returns `EmissionError::InvalidOpcode` if `raw_byte` does not map to any recognized opcode.
    pub fn from_u8(raw_byte: u8, instruction_index: usize) -> Result<Self, EmissionError> {
        match raw_byte {
            0x00 => Ok(Self::Nop),
            0x01 => Ok(Self::Mov),
            0x02 => Ok(Self::LoadI32),
            0x03 => Ok(Self::LoadI64),
            0x04 => Ok(Self::LoadF64),
            0x05 => Ok(Self::LoadBool),
            0x06 => Ok(Self::LoadNull),
            0x07 => Ok(Self::LoadConst),

            0x10 => Ok(Self::AddI32),
            0x11 => Ok(Self::AddI64),
            0x12 => Ok(Self::AddF64),
            0x13 => Ok(Self::SubI32),
            0x14 => Ok(Self::SubI64),
            0x15 => Ok(Self::SubF64),
            0x16 => Ok(Self::MulI32),
            0x17 => Ok(Self::MulI64),
            0x18 => Ok(Self::MulF64),
            0x19 => Ok(Self::DivI32),
            0x1A => Ok(Self::DivI64),
            0x1B => Ok(Self::DivF64),
            0x1C => Ok(Self::ModI32),
            0x1D => Ok(Self::ModI64),
            0x1E => Ok(Self::NegI32),
            0x1F => Ok(Self::NegI64),
            0x20 => Ok(Self::NegF64),

            0x21 => Ok(Self::Band),
            0x22 => Ok(Self::Bor),
            0x23 => Ok(Self::Bxor),
            0x24 => Ok(Self::Bnot),
            0x25 => Ok(Self::Shl),
            0x26 => Ok(Self::Shr),

            0x30 => Ok(Self::EqI32),
            0x31 => Ok(Self::EqI64),
            0x32 => Ok(Self::EqF64),
            0x33 => Ok(Self::EqBool),
            0x34 => Ok(Self::EqString),
            0x35 => Ok(Self::EqRef),
            0x36 => Ok(Self::NeqI32),
            0x37 => Ok(Self::NeqI64),
            0x38 => Ok(Self::NeqF64),
            0x39 => Ok(Self::NeqBool),
            0x3A => Ok(Self::NeqString),
            0x3B => Ok(Self::NeqRef),
            0x3C => Ok(Self::LtI32),
            0x3D => Ok(Self::LtI64),
            0x3E => Ok(Self::LtF64),
            0x3F => Ok(Self::LteI32),
            0x40 => Ok(Self::LteI64),
            0x41 => Ok(Self::LteF64),
            0x42 => Ok(Self::GtI32),
            0x43 => Ok(Self::GtI64),
            0x44 => Ok(Self::GtF64),
            0x45 => Ok(Self::GteI32),
            0x46 => Ok(Self::GteI64),
            0x47 => Ok(Self::GteF64),

            0x48 => Ok(Self::Not),

            0x50 => Ok(Self::Jump),
            0x51 => Ok(Self::JumpIfTrue),
            0x52 => Ok(Self::JumpIfFalse),
            0x53 => Ok(Self::Switch),

            0x60 => Ok(Self::CallDirect),
            0x61 => Ok(Self::CallVirtual),
            0x62 => Ok(Self::CallInterface),
            0x63 => Ok(Self::CallSymbol),
            0x64 => Ok(Self::CallCallable),
            0x65 => Ok(Self::Return),
            0x66 => Ok(Self::ReturnValue),
            0x67 => Ok(Self::Exit),
            0x68 => Ok(Self::CallExtern),

            0x70 => Ok(Self::AllocObject),
            0x71 => Ok(Self::GetField),
            0x72 => Ok(Self::SetField),
            0x73 => Ok(Self::GetStatic),
            0x74 => Ok(Self::SetStatic),
            0x75 => Ok(Self::GetStaticGlobal),
            0x76 => Ok(Self::GetStaticDirect),
            0x77 => Ok(Self::SetStaticGlobal),
            0x78 => Ok(Self::SetStaticDirect),
            0x79 => Ok(Self::AllocObjectExtern),

            0x80 => Ok(Self::AllocArray),
            0x81 => Ok(Self::GetArray),
            0x82 => Ok(Self::SetArray),
            0x83 => Ok(Self::ArrayLength),
            0x84 => Ok(Self::AllocTuple),
            0x85 => Ok(Self::GetTuple),

            0x90 => Ok(Self::PackClosure),
            0x91 => Ok(Self::UnpackCaptures),

            0xA0 => Ok(Self::Instanceof),
            0xA1 => Ok(Self::CastCheck),
            0xA2 => Ok(Self::ConvIToF),
            0xA3 => Ok(Self::ConvFToI),
            0xA4 => Ok(Self::TupleInstanceof),

            0xB0 => Ok(Self::Throw),

            0xC0 => Ok(Self::IncRef),
            0xC1 => Ok(Self::DecRef),

            _ => Err(EmissionError::InvalidOpcode {
                raw_byte,
                instruction_index,
            }),
        }
    }

    /// Returns the human-readable string mnemonic of the opcode.
    #[must_use]
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Self::Nop => "NOP",
            Self::Mov => "MOV",
            Self::LoadI32 => "LOAD_I32",
            Self::LoadI64 => "LOAD_I64",
            Self::LoadF64 => "LOAD_F64",
            Self::LoadBool => "LOAD_BOOL",
            Self::LoadNull => "LOAD_NULL",
            Self::LoadConst => "LOAD_CONST",

            Self::AddI32 => "ADD_I32",
            Self::AddI64 => "ADD_I64",
            Self::AddF64 => "ADD_F64",
            Self::SubI32 => "SUB_I32",
            Self::SubI64 => "SUB_I64",
            Self::SubF64 => "SUB_F64",
            Self::MulI32 => "MUL_I32",
            Self::MulI64 => "MUL_I64",
            Self::MulF64 => "MUL_F64",
            Self::DivI32 => "DIV_I32",
            Self::DivI64 => "DIV_I64",
            Self::DivF64 => "DIV_F64",
            Self::ModI32 => "MOD_I32",
            Self::ModI64 => "MOD_I64",
            Self::NegI32 => "NEG_I32",
            Self::NegI64 => "NEG_I64",
            Self::NegF64 => "NEG_F64",

            Self::Band => "BAND",
            Self::Bor => "BOR",
            Self::Bxor => "BXOR",
            Self::Bnot => "BNOT",
            Self::Shl => "SHL",
            Self::Shr => "SHR",

            Self::EqI32 => "EQ_I32",
            Self::EqI64 => "EQ_I64",
            Self::EqF64 => "EQ_F64",
            Self::EqBool => "EQ_BOOL",
            Self::EqString => "EQ_STRING",
            Self::EqRef => "EQ_REF",
            Self::NeqI32 => "NEQ_I32",
            Self::NeqI64 => "NEQ_I64",
            Self::NeqF64 => "NEQ_F64",
            Self::NeqBool => "NEQ_BOOL",
            Self::NeqString => "NEQ_STRING",
            Self::NeqRef => "NEQ_REF",
            Self::LtI32 => "LT_I32",
            Self::LtI64 => "LT_I64",
            Self::LtF64 => "LT_F64",
            Self::LteI32 => "LTE_I32",
            Self::LteI64 => "LTE_I64",
            Self::LteF64 => "LTE_F64",
            Self::GtI32 => "GT_I32",
            Self::GtI64 => "GT_I64",
            Self::GtF64 => "GT_F64",
            Self::GteI32 => "GTE_I32",
            Self::GteI64 => "GTE_I64",
            Self::GteF64 => "GTE_F64",

            Self::Not => "NOT",

            Self::Jump => "JUMP",
            Self::JumpIfTrue => "JUMP_IF_TRUE",
            Self::JumpIfFalse => "JUMP_IF_FALSE",
            Self::Switch => "SWITCH",

            Self::CallDirect => "CALL_DIRECT",
            Self::CallVirtual => "CALL_VIRTUAL",
            Self::CallInterface => "CALL_INTERFACE",
            Self::CallSymbol => "CALL_SYMBOL",
            Self::CallCallable => "CALL_CALLABLE",
            Self::Return => "RETURN",
            Self::ReturnValue => "RETURN_VALUE",
            Self::Exit => "EXIT",
            Self::CallExtern => "CALL_EXTERN",

            Self::AllocObject => "ALLOC_OBJECT",
            Self::GetField => "GET_FIELD",
            Self::SetField => "SET_FIELD",
            Self::GetStatic => "GET_STATIC",
            Self::SetStatic => "SET_STATIC",
            Self::GetStaticGlobal => "GET_STATIC_GLOBAL",
            Self::GetStaticDirect => "GET_STATIC_DIRECT",
            Self::SetStaticGlobal => "SET_STATIC_GLOBAL",
            Self::SetStaticDirect => "SET_STATIC_DIRECT",
            Self::AllocObjectExtern => "ALLOC_OBJECT_EXTERN",

            Self::AllocArray => "ALLOC_ARRAY",
            Self::GetArray => "GET_ARRAY",
            Self::SetArray => "SET_ARRAY",
            Self::ArrayLength => "ARRAY_LENGTH",
            Self::AllocTuple => "ALLOC_TUPLE",
            Self::GetTuple => "GET_TUPLE",

            Self::PackClosure => "PACK_CLOSURE",
            Self::UnpackCaptures => "UNPACK_CAPTURES",

            Self::Instanceof => "INSTANCEOF",
            Self::CastCheck => "CAST_CHECK",
            Self::ConvIToF => "CONV_I_TO_F",
            Self::ConvFToI => "CONV_F_TO_I",
            Self::TupleInstanceof => "TUPLE_INSTANCEOF",

            Self::Throw => "THROW",

            Self::IncRef => "INC_REF",
            Self::DecRef => "DEC_REF",
        }
    }
}
