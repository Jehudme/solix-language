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
    CallNativeDirect = 0x69,

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

    // 0xD0 .. 0xD7: Superinstructions
    AddI32Imm = 0xD0,
    SubI32Imm = 0xD1,
    JumpIfLtI32 = 0xD2,
    JumpIfLteI32 = 0xD3,
    JumpIfGtI32 = 0xD4,
    JumpIfGteI32 = 0xD5,
    JumpIfEqI32 = 0xD6,
    JumpIfNeqI32 = 0xD7,
}

impl OpCode {
    /// Attempts to convert a raw 8-bit unsigned integer into a valid `OpCode` without allocating diagnostics.
    #[must_use]
    #[inline(always)]
    pub const fn from_raw_u8(raw_byte: u8) -> Option<Self> {
        match raw_byte {
            0x00 => Some(Self::Nop),
            0x01 => Some(Self::Mov),
            0x02 => Some(Self::LoadI32),
            0x03 => Some(Self::LoadI64),
            0x04 => Some(Self::LoadF64),
            0x05 => Some(Self::LoadBool),
            0x06 => Some(Self::LoadNull),
            0x07 => Some(Self::LoadConst),

            0x10 => Some(Self::AddI32),
            0x11 => Some(Self::AddI64),
            0x12 => Some(Self::AddF64),
            0x13 => Some(Self::SubI32),
            0x14 => Some(Self::SubI64),
            0x15 => Some(Self::SubF64),
            0x16 => Some(Self::MulI32),
            0x17 => Some(Self::MulI64),
            0x18 => Some(Self::MulF64),
            0x19 => Some(Self::DivI32),
            0x1A => Some(Self::DivI64),
            0x1B => Some(Self::DivF64),
            0x1C => Some(Self::ModI32),
            0x1D => Some(Self::ModI64),
            0x1E => Some(Self::NegI32),
            0x1F => Some(Self::NegI64),
            0x20 => Some(Self::NegF64),

            0x21 => Some(Self::Band),
            0x22 => Some(Self::Bor),
            0x23 => Some(Self::Bxor),
            0x24 => Some(Self::Bnot),
            0x25 => Some(Self::Shl),
            0x26 => Some(Self::Shr),

            0x30 => Some(Self::EqI32),
            0x31 => Some(Self::EqI64),
            0x32 => Some(Self::EqF64),
            0x33 => Some(Self::EqBool),
            0x34 => Some(Self::EqString),
            0x35 => Some(Self::EqRef),
            0x36 => Some(Self::NeqI32),
            0x37 => Some(Self::NeqI64),
            0x38 => Some(Self::NeqF64),
            0x39 => Some(Self::NeqBool),
            0x3A => Some(Self::NeqString),
            0x3B => Some(Self::NeqRef),
            0x3C => Some(Self::LtI32),
            0x3D => Some(Self::LtI64),
            0x3E => Some(Self::LtF64),
            0x3F => Some(Self::LteI32),
            0x40 => Some(Self::LteI64),
            0x41 => Some(Self::LteF64),
            0x42 => Some(Self::GtI32),
            0x43 => Some(Self::GtI64),
            0x44 => Some(Self::GtF64),
            0x45 => Some(Self::GteI32),
            0x46 => Some(Self::GteI64),
            0x47 => Some(Self::GteF64),

            0x48 => Some(Self::Not),

            0x50 => Some(Self::Jump),
            0x51 => Some(Self::JumpIfTrue),
            0x52 => Some(Self::JumpIfFalse),
            0x53 => Some(Self::Switch),

            0x60 => Some(Self::CallDirect),
            0x61 => Some(Self::CallVirtual),
            0x62 => Some(Self::CallInterface),
            0x63 => Some(Self::CallSymbol),
            0x64 => Some(Self::CallCallable),
            0x65 => Some(Self::Return),
            0x66 => Some(Self::ReturnValue),
            0x67 => Some(Self::Exit),
            0x68 => Some(Self::CallExtern),
            0x69 => Some(Self::CallNativeDirect),

            0x70 => Some(Self::AllocObject),
            0x71 => Some(Self::GetField),
            0x72 => Some(Self::SetField),
            0x73 => Some(Self::GetStatic),
            0x74 => Some(Self::SetStatic),
            0x75 => Some(Self::GetStaticGlobal),
            0x76 => Some(Self::GetStaticDirect),
            0x77 => Some(Self::SetStaticGlobal),
            0x78 => Some(Self::SetStaticDirect),
            0x79 => Some(Self::AllocObjectExtern),

            0x80 => Some(Self::AllocArray),
            0x81 => Some(Self::GetArray),
            0x82 => Some(Self::SetArray),
            0x83 => Some(Self::ArrayLength),
            0x84 => Some(Self::AllocTuple),
            0x85 => Some(Self::GetTuple),

            0x90 => Some(Self::PackClosure),
            0x91 => Some(Self::UnpackCaptures),

            0xA0 => Some(Self::Instanceof),
            0xA1 => Some(Self::CastCheck),
            0xA2 => Some(Self::ConvIToF),
            0xA3 => Some(Self::ConvFToI),
            0xA4 => Some(Self::TupleInstanceof),

            0xB0 => Some(Self::Throw),

            0xC0 => Some(Self::IncRef),
            0xC1 => Some(Self::DecRef),

            0xD0 => Some(Self::AddI32Imm),
            0xD1 => Some(Self::SubI32Imm),
            0xD2 => Some(Self::JumpIfLtI32),
            0xD3 => Some(Self::JumpIfLteI32),
            0xD4 => Some(Self::JumpIfGtI32),
            0xD5 => Some(Self::JumpIfGteI32),
            0xD6 => Some(Self::JumpIfEqI32),
            0xD7 => Some(Self::JumpIfNeqI32),

            _ => None,
        }
    }

    /// Attempts to convert a raw 8-bit unsigned integer into a valid `OpCode`.
    ///
    /// # Errors
    /// Returns `EmissionError::InvalidOpcode` if `raw_byte` does not map to any recognized opcode.
    #[inline(always)]
    pub fn from_u8(raw_byte: u8, instruction_index: usize) -> Result<Self, EmissionError> {
        match Self::from_raw_u8(raw_byte) {
            Some(opcode) => Ok(opcode),
            None => Err(EmissionError::InvalidOpcode {
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
            Self::CallNativeDirect => "CALL_NATIVE_DIRECT",

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

            Self::AddI32Imm => "ADD_I32_IMM",
            Self::SubI32Imm => "SUB_I32_IMM",
            Self::JumpIfLtI32 => "JUMP_IF_LT_I32",
            Self::JumpIfLteI32 => "JUMP_IF_LTE_I32",
            Self::JumpIfGtI32 => "JUMP_IF_GT_I32",
            Self::JumpIfGteI32 => "JUMP_IF_GTE_I32",
            Self::JumpIfEqI32 => "JUMP_IF_EQ_I32",
            Self::JumpIfNeqI32 => "JUMP_IF_NEQ_I32",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpCode;
    use crate::diagnostics::EmissionError;

    #[test]
    fn test_positive_opcode_from_raw_u8_roundtrip() {
        let valid_bytes: [u8; 80] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23,
            0x24, 0x25, 0x26, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A,
            0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
            0x50, 0x51, 0x52, 0x53, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
        ];
        for raw_byte in valid_bytes {
            let decoded = OpCode::from_raw_u8(raw_byte);
            assert!(
                decoded.is_some(),
                "expected byte {raw_byte:#04x} to decode via from_raw_u8"
            );
            assert_eq!(
                decoded.map(|opcode| opcode as u8),
                Some(raw_byte),
                "round-tripped opcode byte mismatch"
            );
            assert_eq!(OpCode::from_u8(raw_byte, 0).ok(), decoded);
        }

        let extra_bytes: [u8; 25] = [
            0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x90, 0x91, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xB0,
            0xC0, 0xC1, 0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0x00,
        ];
        for raw_byte in extra_bytes {
            let decoded = OpCode::from_raw_u8(raw_byte);
            assert!(decoded.is_some());
            assert_eq!(Ok(decoded), OpCode::from_u8(raw_byte, 3).map(Some));
        }
    }

    #[test]
    fn test_negative_invalid_raw_opcode_returns_none() {
        let invalid_bytes: [u8; 5] = [0x08, 0x2F, 0x7A, 0x9F, 0xFF];
        for raw_byte in invalid_bytes {
            assert_eq!(OpCode::from_raw_u8(raw_byte), None);
            assert_eq!(
                OpCode::from_u8(raw_byte, 42),
                Err(EmissionError::InvalidOpcode {
                    raw_byte,
                    instruction_index: 42,
                })
            );
        }
    }
}
