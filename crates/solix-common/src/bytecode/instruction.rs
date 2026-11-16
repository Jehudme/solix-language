//! Fixed-width 64-bit aligned virtual machine instruction encoding.

#![forbid(unsafe_code)]

use crate::bytecode::opcodes::OpCode;
use crate::diagnostics::EmissionError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Instruction flag marking a non-owning weak reference assignment.
pub const FLAG_WEAK: u8 = 0x01;

/// Instruction flag marking an allocation of a Plain-Old-Data (POD) class instance.
pub const FLAG_POD: u8 = 0x02;

/// Strongly-typed 64-bit instruction word for the virtual register VM.
///
/// Encodes instructions into uniform 8-byte aligned words using four main formats:
/// - **Format A (3-Register Arithmetic & Logic):** `[Opcode:8 | Flags:8 | Dst:16 | Src1:16 | Src2:16]`
/// - **Format B (Register + 32-Bit Immediate / Constant):** `[Opcode:8 | Flags:8 | Dst:16 | Imm:32]`
/// - **Format C (Branch & Jump with Signed PC Offset):** `[Opcode:8 | Flags:8 | CondReg:16 | Offset:32]`
/// - **Format Call (Windowed Call):** `[Opcode:8 | Dst:16 | TargetId:16 | ArgBase:16 | ArgCount:8]`
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Instruction(pub u64);

impl Instruction {
    /// Constructs a Format A instruction (3 registers: dst, src1, src2).
    #[must_use]
    #[inline(always)]
    pub const fn format_a(opcode: OpCode, flags: u8, dst: u16, src1: u16, src2: u16) -> Self {
        let raw = (opcode as u8 as u64)
            | ((flags as u64) << 8)
            | ((dst as u64) << 16)
            | ((src1 as u64) << 32)
            | ((src2 as u64) << 48);
        Self(raw)
    }

    /// Constructs a Format B instruction (dst register + 32-bit unsigned immediate/index).
    #[must_use]
    #[inline(always)]
    pub const fn format_b(opcode: OpCode, flags: u8, dst: u16, imm: u32) -> Self {
        let raw = (opcode as u8 as u64)
            | ((flags as u64) << 8)
            | ((dst as u64) << 16)
            | ((imm as u64) << 32);
        Self(raw)
    }

    /// Returns a copy of this Format A or Format B instruction with the destination register modified.
    #[must_use]
    #[inline(always)]
    pub const fn with_dst(self, new_dst: u16) -> Self {
        let raw = (self.0 & !0x0000_0000_FFFF_0000) | ((new_dst as u64) << 16);
        Self(raw)
    }

    /// Constructs a Format B instruction with a 32-bit signed immediate.
    #[must_use]
    #[inline(always)]
    pub const fn format_b_signed(opcode: OpCode, flags: u8, dst: u16, imm: i32) -> Self {
        Self::format_b(opcode, flags, dst, imm as u32)
    }

    /// Constructs a Format C branch instruction with condition register and signed 32-bit offset.
    #[must_use]
    #[inline(always)]
    pub const fn format_c(opcode: OpCode, flags: u8, cond_reg: u16, offset: i32) -> Self {
        let raw = (opcode as u8 as u64)
            | ((flags as u64) << 8)
            | ((cond_reg as u64) << 16)
            | (((offset as u32) as u64) << 32);
        Self(raw)
    }

    /// Constructs a Format Call instruction with windowed register arguments.
    #[must_use]
    #[inline(always)]
    pub const fn format_call(
        opcode: OpCode,
        dst: u16,
        target_id: u16,
        arg_base: u16,
        arg_count: u8,
    ) -> Self {
        let raw = (opcode as u8 as u64)
            | ((dst as u64) << 8)
            | ((target_id as u64) << 24)
            | ((arg_base as u64) << 40)
            | ((arg_count as u64) << 56);
        Self(raw)
    }

    /// Extracts the raw 8-bit opcode byte.
    #[must_use]
    #[inline(always)]
    pub const fn raw_opcode(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// Decodes the typed `OpCode`.
    ///
    /// # Errors
    /// Returns `EmissionError::InvalidOpcode` if the opcode byte is unrecognized.
    #[inline(always)]
    pub fn opcode(&self, instruction_index: usize) -> Result<OpCode, EmissionError> {
        OpCode::from_u8(self.raw_opcode(), instruction_index)
    }

    /// Extracts the 8-bit instruction flags byte.
    #[must_use]
    #[inline(always)]
    pub const fn flags(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// Returns `true` if this instruction has the `FLAG_WEAK` bit set.
    #[must_use]
    #[inline(always)]
    pub const fn is_weak(&self) -> bool {
        (self.flags() & FLAG_WEAK) != 0
    }

    /// Returns `true` if this instruction has the `FLAG_POD` bit set.
    #[must_use]
    #[inline(always)]
    pub const fn is_pod(&self) -> bool {
        (self.flags() & FLAG_POD) != 0
    }

    /// Returns a copy of the instruction with updated flags.
    #[must_use]
    #[inline(always)]
    pub const fn with_flags(self, flags: u8) -> Self {
        let cleared = self.0 & !(0xFF << 8);
        Self(cleared | ((flags as u64) << 8))
    }

    /// Extracts the destination register `dst` for Format A and B.
    #[must_use]
    #[inline(always)]
    pub const fn dst(&self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    /// Extracts the first source register `src1` for Format A.
    #[must_use]
    #[inline(always)]
    pub const fn src1(&self) -> u16 {
        ((self.0 >> 32) & 0xFFFF) as u16
    }

    /// Extracts the second source register `src2` for Format A.
    #[must_use]
    #[inline(always)]
    pub const fn src2(&self) -> u16 {
        ((self.0 >> 48) & 0xFFFF) as u16
    }

    /// Extracts the 32-bit unsigned immediate value for Format B.
    #[must_use]
    #[inline(always)]
    pub const fn imm32(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Extracts the 32-bit signed immediate value for Format B.
    #[must_use]
    #[inline(always)]
    pub const fn imm32_signed(&self) -> i32 {
        (self.0 >> 32) as i32
    }

    /// Extracts the condition register for Format C.
    #[must_use]
    #[inline(always)]
    pub const fn cond_reg(&self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    /// Extracts the signed 32-bit relative PC jump offset for Format C.
    #[must_use]
    #[inline(always)]
    pub const fn jump_offset(&self) -> i32 {
        (self.0 >> 32) as i32
    }

    /// Extracts the destination register for Format Call.
    #[must_use]
    #[inline(always)]
    pub const fn call_dst(&self) -> u16 {
        ((self.0 >> 8) & 0xFFFF) as u16
    }

    /// Extracts the target function ID / vtable slot / const index for Format Call.
    #[must_use]
    #[inline(always)]
    pub const fn call_target(&self) -> u16 {
        ((self.0 >> 24) & 0xFFFF) as u16
    }

    /// Extracts the base register index of the argument window for Format Call.
    #[must_use]
    #[inline(always)]
    pub const fn arg_base(&self) -> u16 {
        ((self.0 >> 40) & 0xFFFF) as u16
    }

    /// Extracts the argument count for Format Call.
    #[must_use]
    #[inline(always)]
    pub const fn arg_count(&self) -> u8 {
        ((self.0 >> 56) & 0xFF) as u8
    }

    /// Serializes this instruction into 8 little-endian bytes.
    #[must_use]
    pub const fn to_le_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    /// Deserializes an instruction from 8 little-endian bytes.
    #[must_use]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_le_bytes(bytes))
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw_op = self.raw_opcode();
        if let Ok(opcode) = OpCode::from_u8(raw_op, 0) {
            write!(f, "Instruction({:?} [0x{:016x}])", opcode, self.0)
        } else {
            write!(
                f,
                "Instruction(Unknown({:#04x}) [0x{:016x}])",
                raw_op, self.0
            )
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw_op = self.raw_opcode();
        match OpCode::from_u8(raw_op, 0) {
            Ok(opcode) => match opcode {
                OpCode::CallDirect
                | OpCode::CallVirtual
                | OpCode::CallInterface
                | OpCode::CallSymbol
                | OpCode::CallCallable
                | OpCode::CallExtern
                | OpCode::CallNativeDirect => {
                    write!(
                        f,
                        "{:<14} r{}, target:{}, args:r{}..r{}",
                        opcode.mnemonic(),
                        self.call_dst(),
                        self.call_target(),
                        self.arg_base(),
                        self.arg_base() + self.arg_count() as u16
                    )
                }
                OpCode::Jump | OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                    write!(
                        f,
                        "{:<14} cond:r{}, offset:{:+}",
                        opcode.mnemonic(),
                        self.cond_reg(),
                        self.jump_offset()
                    )
                }
                OpCode::LoadI32
                | OpCode::LoadConst
                | OpCode::GetField
                | OpCode::SetField
                | OpCode::GetStaticGlobal
                | OpCode::GetStaticDirect
                | OpCode::SetStaticGlobal
                | OpCode::SetStaticDirect
                | OpCode::AllocObjectExtern => {
                    write!(
                        f,
                        "{:<14} r{}, imm:{}",
                        opcode.mnemonic(),
                        self.dst(),
                        self.imm32()
                    )
                }
                _ => {
                    write!(
                        f,
                        "{:<14} r{}, r{}, r{}",
                        opcode.mnemonic(),
                        self.dst(),
                        self.src1(),
                        self.src2()
                    )
                }
            },
            Err(_) => write!(f, "UNKNOWN_OPCODE({:#04x})", raw_op),
        }
    }
}
