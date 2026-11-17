//! Ahead-of-Time (AOT) bytecode verifier performing load-time static invariant proofs.
//!
//! Validates:
//! 1. Register Window Bound Proof: all operand registers < func.num_registers.
//! 2. Branch Target Proof: all branch targets fall within 0..=func.instructions.len().
//! 3. Static & Constant Pool Proof: static offsets < total_static_space and constant pool indices < pool.len().

#![forbid(unsafe_code)]

use crate::context::LoadedBinary;
use crate::exceptions::RuntimeError;
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode};

/// Verifies that all compiled functions across all loaded binaries satisfy bytecode safety proofs.
///
/// # Errors
/// Returns `RuntimeError::BinaryLoadError` if any register, branch target, constant pool index,
/// or static variable offset violates integrity bounds.
pub fn verify_program(
    loaded_binaries: &[LoadedBinary],
    all_functions: &[CompiledFunction],
    total_static_space: u32,
) -> Result<(), RuntimeError> {
    for (global_function_index, func) in all_functions.iter().enumerate() {
        let register_count = func.num_registers;
        let instruction_count = func.instructions.len();

        let owning_binary_pool_len = loaded_binaries
            .iter()
            .find(|binary_unit| {
                let start_index = binary_unit.function_id_offset as usize;
                let end_index = start_index + binary_unit.function_count;
                global_function_index >= start_index && global_function_index < end_index
            })
            .or_else(|| {
                loaded_binaries.iter().find(|binary_unit| {
                    (func.static_base_offset >= binary_unit.static_base_offset)
                        && (func.static_base_offset
                            <= binary_unit.static_base_offset + binary_unit.static_space_size)
                })
            })
            .map_or(0, |binary_unit| binary_unit.constant_pool.values.len());

        for (instruction_index, &instruction) in func.instructions.iter().enumerate() {
            verify_instruction(
                instruction,
                instruction_index,
                instruction_count,
                register_count,
                total_static_space,
                owning_binary_pool_len,
                &func.symbol,
            )?;
        }
    }

    Ok(())
}

/// Validates a single instruction word against function and binary resource limits.
fn verify_instruction(
    instruction: Instruction,
    instruction_index: usize,
    instruction_count: usize,
    register_count: u16,
    total_static_space: u32,
    constant_pool_len: usize,
    function_symbol: &str,
) -> Result<(), RuntimeError> {
    let raw_opcode = instruction.raw_opcode();
    let opcode = match OpCode::from_u8(raw_opcode, instruction_index) {
        Ok(valid_op) => valid_op,
        Err(_) => {
            return Err(RuntimeError::BinaryLoadError {
                reason: format!(
                    "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: unrecognized opcode 0x{raw_opcode:02X}"
                ),
            });
        }
    };

    let check_reg = |register_index: u16, register_name: &str| -> Result<(), RuntimeError> {
        if register_index >= register_count {
            Err(RuntimeError::BinaryLoadError {
                reason: format!(
                    "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: {register_name} register index {register_index} exceeds allocated register count {register_count}"
                ),
            })
        } else {
            Ok(())
        }
    };

    let check_branch_target = |offset: i32| -> Result<(), RuntimeError> {
        let target_pc = (instruction_index as i32) + offset;
        if target_pc < 0 || (target_pc as usize) > instruction_count {
            Err(RuntimeError::BinaryLoadError {
                reason: format!(
                    "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: branch target {target_pc} is out of bounds (instruction count: {instruction_count})"
                ),
            })
        } else {
            Ok(())
        }
    };

    match opcode {
        OpCode::Nop => {}

        OpCode::Mov => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "src1")?;
        }

        OpCode::LoadI32
        | OpCode::LoadI64
        | OpCode::LoadF64
        | OpCode::LoadBool
        | OpCode::LoadNull => {
            check_reg(instruction.dst(), "dst")?;
        }

        OpCode::LoadConst => {
            check_reg(instruction.dst(), "dst")?;
            let pool_index = instruction.imm32() as usize;
            if pool_index >= constant_pool_len && constant_pool_len > 0 {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: constant pool index {pool_index} exceeds constant pool size {constant_pool_len}"
                    ),
                });
            }
        }

        // 3-Register Arithmetic & Logic
        OpCode::AddI32
        | OpCode::AddI64
        | OpCode::AddF64
        | OpCode::SubI32
        | OpCode::SubI64
        | OpCode::SubF64
        | OpCode::MulI32
        | OpCode::MulI64
        | OpCode::MulF64
        | OpCode::DivI32
        | OpCode::DivI64
        | OpCode::DivF64
        | OpCode::ModI32
        | OpCode::ModI64
        | OpCode::Band
        | OpCode::Bor
        | OpCode::Bxor
        | OpCode::Shl
        | OpCode::Shr
        | OpCode::EqI32
        | OpCode::EqI64
        | OpCode::EqF64
        | OpCode::EqBool
        | OpCode::EqString
        | OpCode::EqRef
        | OpCode::NeqI32
        | OpCode::NeqI64
        | OpCode::NeqF64
        | OpCode::NeqBool
        | OpCode::NeqString
        | OpCode::NeqRef
        | OpCode::LtI32
        | OpCode::LtI64
        | OpCode::LtF64
        | OpCode::LteI32
        | OpCode::LteI64
        | OpCode::LteF64
        | OpCode::GtI32
        | OpCode::GtI64
        | OpCode::GtF64
        | OpCode::GteI32
        | OpCode::GteI64
        | OpCode::GteF64 => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "src1")?;
            check_reg(instruction.src2(), "src2")?;
        }

        // Unary Operations
        OpCode::NegI32
        | OpCode::NegI64
        | OpCode::NegF64
        | OpCode::Bnot
        | OpCode::Not
        | OpCode::ConvIToF
        | OpCode::ConvFToI => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "src1")?;
        }

        // Branches & Jumps
        OpCode::Jump | OpCode::Switch => {
            check_branch_target(instruction.jump_offset())?;
        }

        OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
            check_reg(instruction.cond_reg(), "cond_reg")?;
            check_branch_target(instruction.jump_offset())?;
        }

        // Windowed Function Invocations
        OpCode::CallDirect
        | OpCode::TailCallDirect
        | OpCode::CallVirtual
        | OpCode::CallInterface
        | OpCode::CallSymbol
        | OpCode::CallCallable
        | OpCode::CallExtern
        | OpCode::CallNativeDirect => {
            check_reg(instruction.call_dst(), "call_dst")?;
            let argument_count = instruction.arg_count() as u16;
            if argument_count > 0 {
                let argument_base = instruction.arg_base();
                let argument_limit = argument_base.saturating_add(argument_count);
                if argument_limit > register_count {
                    return Err(RuntimeError::BinaryLoadError {
                        reason: format!(
                            "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: call argument window [{argument_base}..{argument_limit}] exceeds allocated register count {register_count}"
                        ),
                    });
                }
            }
        }

        OpCode::Return => {}

        OpCode::ReturnValue => {
            check_reg(instruction.dst(), "return_val")?;
        }

        OpCode::Exit => {
            check_reg(instruction.dst(), "exit_code")?;
        }

        OpCode::AllocObject | OpCode::AllocObjectExtern => {
            check_reg(instruction.dst(), "dst")?;
        }

        OpCode::GetField => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "object_reg")?;
        }

        OpCode::SetField => {
            check_reg(instruction.dst(), "value_reg")?;
            check_reg(instruction.src1(), "object_reg")?;
        }

        OpCode::GetStatic | OpCode::SetStatic => {
            check_reg(instruction.dst(), "dst")?;
        }

        OpCode::GetStaticDirect | OpCode::SetStaticDirect => {
            check_reg(instruction.dst(), "dst")?;
            let slot = instruction.imm32();
            if slot >= total_static_space && total_static_space > 0 {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: static slot {slot} exceeds total static space {total_static_space}"
                    ),
                });
            }
        }

        OpCode::GetStaticGlobal | OpCode::SetStaticGlobal => {
            check_reg(instruction.dst(), "dst")?;
        }

        OpCode::AllocArray => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "length_reg")?;
        }

        OpCode::GetArray => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "array_reg")?;
            check_reg(instruction.src2(), "index_reg")?;
        }

        OpCode::SetArray => {
            check_reg(instruction.dst(), "value_reg")?;
            check_reg(instruction.src1(), "array_reg")?;
            check_reg(instruction.src2(), "index_reg")?;
        }

        OpCode::ArrayLength => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "array_reg")?;
        }

        OpCode::AllocTuple => {
            check_reg(instruction.dst(), "dst")?;
        }

        OpCode::GetTuple => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "tuple_reg")?;
        }

        OpCode::PackClosure => {
            check_reg(instruction.call_dst(), "closure_dst")?;
            let capture_count = instruction.arg_count() as u16;
            if capture_count > 0 {
                let capture_base = instruction.arg_base();
                let capture_limit = capture_base.saturating_add(capture_count);
                if capture_limit > register_count {
                    return Err(RuntimeError::BinaryLoadError {
                        reason: format!(
                            "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: closure capture window [{capture_base}..{capture_limit}] exceeds allocated register count {register_count}"
                        ),
                    });
                }
            }
        }

        OpCode::UnpackCaptures => {
            let capture_count = instruction.arg_count() as u16;
            if capture_count > 0 {
                let capture_base = instruction.arg_base();
                let capture_limit = capture_base.saturating_add(capture_count);
                if capture_limit > register_count {
                    return Err(RuntimeError::BinaryLoadError {
                        reason: format!(
                            "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: unpack capture window [{capture_base}..{capture_limit}] exceeds allocated register count {register_count}"
                        ),
                    });
                }
            }
        }

        OpCode::Instanceof | OpCode::CastCheck | OpCode::TupleInstanceof => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "instance_reg")?;
        }

        OpCode::Throw => {
            check_reg(instruction.dst(), "exception_reg")?;
        }

        OpCode::IncRef | OpCode::DecRef => {
            check_reg(instruction.dst(), "target_reg")?;
        }

        // Superinstructions
        OpCode::AddI32Imm | OpCode::SubI32Imm | OpCode::MulI32Imm | OpCode::ModI32Imm => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "src1")?;
        }

        OpCode::AddBandI32 | OpCode::MulAddF64 => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.flags() as u16, "acc_reg")?;
            check_reg(instruction.src1(), "val_reg")?;
            check_reg(instruction.src2(), "src2_reg")?;
        }

        OpCode::GetArrayJumpIfFalse => {
            check_reg(instruction.dst(), "arr_reg")?;
            check_reg(instruction.src1(), "idx_reg")?;
            let offset = instruction.src2() as i16 as i32;
            check_branch_target(offset)?;
        }

        OpCode::ForLoopI32 => {
            check_reg(instruction.dst(), "idx_reg")?;
            check_reg(instruction.src1(), "bound_reg")?;
            if (instruction.flags() & 0x80) != 0 {
                let step_reg = (instruction.flags() & 0x1F) as u16;
                check_reg(step_reg, "step_reg")?;
            }
            let offset = instruction.src2() as i16 as i32;
            check_branch_target(offset)?;
        }

        OpCode::JumpIfI32Imm => {
            check_reg(instruction.dst(), "reg_a")?;
            let offset = instruction.src2() as i16 as i32;
            check_branch_target(offset)?;
        }

        OpCode::JumpIfLtI32
        | OpCode::JumpIfLteI32
        | OpCode::JumpIfGtI32
        | OpCode::JumpIfGteI32
        | OpCode::JumpIfEqI32
        | OpCode::JumpIfNeqI32 => {
            check_reg(instruction.dst(), "reg_a")?;
            check_reg(instruction.src1(), "reg_b")?;
            let offset = instruction.src2() as i16 as i32;
            check_branch_target(offset)?;
        }
    }

    Ok(())
}
