//! Ahead-of-Time (AOT) bytecode verifier performing load-time static invariant proofs.
//!
//! Validates:
//! 1. Register Window Bound Proof: all operand registers < func.num_registers.
//! 2. Branch Target Proof: all branch targets fall within 0..=func.instructions.len().
//! 3. Static & Constant Pool Proof: static offsets < total_static_space and constant pool indices < pool.len().

#![forbid(unsafe_code)]

use crate::context::LoadedBinary;
use crate::exceptions::RuntimeError;
use solix_common::bytecode::{CompiledClass, CompiledFunction, Instruction, OpCode};
use std::collections::HashMap;

struct ClassVerificationIndex {
    has_class_metadata: bool,
    class_sizes_by_vtable: HashMap<u32, usize>,
    max_instance_size: usize,
    max_vtable_len: usize,
    max_iface_slots: HashMap<u32, usize>,
}

impl ClassVerificationIndex {
    fn from_classes(all_classes: &[CompiledClass]) -> Self {
        let mut class_sizes_by_vtable = HashMap::with_capacity(all_classes.len());
        let mut max_instance_size = 0usize;
        let mut max_vtable_len = 0usize;
        let mut max_iface_slots: HashMap<u32, usize> = HashMap::new();

        for class_desc in all_classes {
            class_sizes_by_vtable.insert(class_desc.vtable_id, class_desc.instance_size);
            max_instance_size = max_instance_size.max(class_desc.instance_size);
            max_vtable_len = max_vtable_len.max(class_desc.vtable.len());
            for (iface_id, methods) in &class_desc.itable {
                max_iface_slots
                    .entry(*iface_id)
                    .and_modify(|cur| *cur = (*cur).max(methods.len()))
                    .or_insert(methods.len());
            }
        }

        Self {
            has_class_metadata: !all_classes.is_empty(),
            class_sizes_by_vtable,
            max_instance_size,
            max_vtable_len,
            max_iface_slots,
        }
    }
}

/// Verifies that all compiled functions across all loaded binaries satisfy bytecode safety proofs.
///
/// # Errors
/// Returns `RuntimeError::BinaryLoadError` if any register, branch target, constant pool index,
/// class/vtable/field operand, or static variable offset violates integrity bounds.
pub fn verify_program(
    loaded_binaries: &[LoadedBinary],
    all_functions: &[CompiledFunction],
    total_static_space: u32,
) -> Result<(), RuntimeError> {
    let flattened_classes: Vec<CompiledClass> = loaded_binaries
        .iter()
        .flat_map(|binary_unit| binary_unit.classes.iter().cloned())
        .collect();
    verify_program_with_classes(
        loaded_binaries,
        all_functions,
        &flattened_classes,
        total_static_space,
    )
}

/// Verifies that all compiled functions satisfy bytecode safety proofs using relocated class tables.
///
/// # Errors
/// Returns `RuntimeError::BinaryLoadError` if any register, branch target, constant pool index,
/// class/vtable/field operand, or static variable offset violates integrity bounds.
pub fn verify_program_with_classes(
    loaded_binaries: &[LoadedBinary],
    all_functions: &[CompiledFunction],
    all_classes: &[CompiledClass],
    total_static_space: u32,
) -> Result<(), RuntimeError> {
    let total_functions = all_functions.len();
    let class_index = ClassVerificationIndex::from_classes(all_classes);
    for (global_function_index, func) in all_functions.iter().enumerate() {
        let register_count = func.num_registers;
        let instruction_count = func.instructions.len();

        let owning_binary_pool_len = loaded_binaries
            .iter()
            .find(|binary_unit| {
                let start_index = binary_unit.function_id_offset as usize;
                let Some(end_index) = start_index.checked_add(binary_unit.function_count) else {
                    return false;
                };
                global_function_index >= start_index && global_function_index < end_index
            })
            .map_or(0, |binary_unit| binary_unit.constant_pool.values.len());

        for handler in &func.exception_handlers {
            let try_start = handler.try_start_pc as usize;
            let try_end = handler.try_end_pc as usize;
            if try_start > try_end || try_end > instruction_count {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{}': invalid exception range [{try_start}..{try_end}) for instruction count {instruction_count}",
                        func.symbol
                    ),
                });
            }
            if (handler.handler_pc as usize) >= instruction_count {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{}': exception handler_pc {} exceeds instruction count {instruction_count}",
                        func.symbol, handler.handler_pc
                    ),
                });
            }
            if let Some(finally_pc) = handler.finally_pc
                && (finally_pc as usize) >= instruction_count
            {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{}': exception finally_pc {finally_pc} exceeds instruction count {instruction_count}",
                        func.symbol
                    ),
                });
            }
            if handler.catch_reg >= register_count {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{}': exception catch_reg {} exceeds allocated register count {register_count}",
                        func.symbol, handler.catch_reg
                    ),
                });
            }
            for &try_ref_reg in &handler.try_ref_regs {
                if try_ref_reg >= register_count {
                    return Err(RuntimeError::BinaryLoadError {
                        reason: format!(
                            "AOT verifier failed in function '{}': exception try_ref_reg {try_ref_reg} exceeds allocated register count {register_count}",
                            func.symbol
                        ),
                    });
                }
            }
        }

        for (instruction_index, &instruction) in func.instructions.iter().enumerate() {
            verify_instruction(
                instruction,
                instruction_index,
                instruction_count,
                register_count,
                total_static_space,
                owning_binary_pool_len,
                total_functions,
                &class_index,
                &func.symbol,
            )?;
        }
    }

    Ok(())
}

/// Validates a single instruction word against function and binary resource limits.
#[allow(clippy::too_many_arguments)]
fn verify_instruction(
    instruction: Instruction,
    instruction_index: usize,
    instruction_count: usize,
    register_count: u16,
    total_static_space: u32,
    constant_pool_len: usize,
    total_functions: usize,
    class_index: &ClassVerificationIndex,
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

    let check_branch_target = |instruction_offset: i32| -> Result<(), RuntimeError> {
        let target_signed = i32::try_from(instruction_index)
            .ok()
            .and_then(|pc_i32| pc_i32.checked_add(instruction_offset));
        match target_signed {
            Some(target_program_counter)
                if target_program_counter >= 0
                    && (target_program_counter as usize) <= instruction_count =>
            {
                Ok(())
            }
            Some(target_program_counter) => Err(RuntimeError::BinaryLoadError {
                reason: format!(
                    "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: branch target {target_program_counter} is out of bounds (instruction count: {instruction_count})"
                ),
            }),
            None => {
                let wide_target = (instruction_index as i64) + i64::from(instruction_offset);
                Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: branch target {wide_target} is out of bounds (instruction count: {instruction_count})"
                    ),
                })
            }
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
            if pool_index >= constant_pool_len {
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
        | OpCode::DivU32
        | OpCode::DivU64
        | OpCode::ModU32
        | OpCode::ModU64
        | OpCode::Band
        | OpCode::Bor
        | OpCode::Bxor
        | OpCode::Shl
        | OpCode::Shr
        | OpCode::Lshr
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
        | OpCode::GteF64
        | OpCode::LtU32
        | OpCode::LteU32
        | OpCode::GtU32
        | OpCode::GteU32
        | OpCode::LtU64
        | OpCode::LteU64
        | OpCode::GtU64
        | OpCode::GteU64 => {
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
        OpCode::CallDirect | OpCode::TailCallDirect => {
            check_reg(instruction.call_dst(), "call_dst")?;
            let target_id = instruction.call_target() as usize;
            if target_id >= total_functions {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: call target function ID {target_id} exceeds total function count {total_functions}"
                    ),
                });
            }
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

        OpCode::CallNativeDirect => {
            check_reg(instruction.call_dst(), "call_dst")?;
            let target_id = instruction.call_target() as usize;
            if target_id >= total_functions {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: call target function ID {target_id} exceeds total function count {total_functions}"
                    ),
                });
            }
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

        OpCode::CallSymbol | OpCode::CallExtern => {
            check_reg(instruction.call_dst(), "call_dst")?;
            let pool_index = instruction.call_target() as usize;
            if pool_index >= constant_pool_len {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: constant pool index {pool_index} exceeds constant pool size {constant_pool_len}"
                    ),
                });
            }
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

        OpCode::CallCallable => {
            check_reg(instruction.call_dst(), "call_dst")?;
            check_reg(instruction.call_target(), "callee_reg")?;
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

        OpCode::CallVirtual | OpCode::CallInterface | OpCode::CallVirtualDirect => {
            check_reg(instruction.call_dst(), "call_dst")?;
            check_reg(instruction.arg_base(), "receiver_reg")?;
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
            if class_index.has_class_metadata {
                if opcode == OpCode::CallInterface {
                    let target = instruction.call_target();
                    let iface_id = (target >> 8) as u32;
                    let slot = (target & 0xFF) as usize;
                    if iface_id != 0
                        && let Some(&method_count) = class_index.max_iface_slots.get(&iface_id)
                        && slot >= method_count
                    {
                        return Err(RuntimeError::BinaryLoadError {
                            reason: format!(
                                "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: CallInterface target (interface {iface_id}, slot {slot}) exceeds loaded method count {method_count}"
                            ),
                        });
                    }
                } else {
                    let slot = (instruction.call_target() & 0x7FFF) as usize;
                    if slot >= class_index.max_vtable_len {
                        return Err(RuntimeError::BinaryLoadError {
                            reason: format!(
                                "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: virtual method slot {slot} exceeds max vtable length {}",
                                class_index.max_vtable_len
                            ),
                        });
                    }
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

        OpCode::AllocObject => {
            check_reg(instruction.dst(), "dst")?;
            if class_index.has_class_metadata {
                let vtable_id = instruction.src1() as u32;
                let alloc_size = instruction.src2() as usize;
                let Some(&expected_size) = class_index.class_sizes_by_vtable.get(&vtable_id) else {
                    return Err(RuntimeError::BinaryLoadError {
                        reason: format!(
                            "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: AllocObject vtable ID {vtable_id} does not match any loaded class"
                        ),
                    });
                };
                if alloc_size < 8 || alloc_size < expected_size {
                    return Err(RuntimeError::BinaryLoadError {
                        reason: format!(
                            "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: AllocObject size {alloc_size} violates class header/instance size requirement ({expected_size} bytes)"
                        ),
                    });
                }
            }
        }

        OpCode::AllocObjectExtern => {
            check_reg(instruction.dst(), "dst")?;
            let pool_index = instruction.imm32() as usize;
            if pool_index >= constant_pool_len {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: AllocObjectExtern constant pool index {pool_index} exceeds pool length {constant_pool_len}"
                    ),
                });
            }
        }

        OpCode::GetField | OpCode::GetFieldI32 | OpCode::GetFieldF64 => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "object_reg")?;
            check_field_bounds(
                class_index,
                opcode,
                instruction.flags(),
                instruction.src2() as usize,
                function_symbol,
                instruction_index,
            )?;
        }

        OpCode::SetField | OpCode::SetFieldI32 | OpCode::SetFieldF64 => {
            check_reg(instruction.dst(), "value_reg")?;
            check_reg(instruction.src1(), "object_reg")?;
            check_field_bounds(
                class_index,
                opcode,
                instruction.flags(),
                instruction.src2() as usize,
                function_symbol,
                instruction_index,
            )?;
        }

        OpCode::GetStatic
        | OpCode::SetStatic
        | OpCode::GetStaticDirect
        | OpCode::SetStaticDirect => {
            check_reg(instruction.dst(), "dst")?;
            let slot_index = instruction.imm32();
            if slot_index >= total_static_space {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: static slot {slot_index} exceeds total static space {total_static_space}"
                    ),
                });
            }
        }

        OpCode::GetStaticGlobal | OpCode::SetStaticGlobal => {
            check_reg(instruction.dst(), "dst")?;
            let pool_index = instruction.imm32() as usize;
            if pool_index >= constant_pool_len {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: global static constant pool index {pool_index} exceeds pool length {constant_pool_len}"
                    ),
                });
            }
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
            const MAX_TUPLE_ELEMENTS: u16 = 4096;
            let tuple_count = instruction.src1();
            if tuple_count > MAX_TUPLE_ELEMENTS {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: tuple element count {tuple_count} exceeds maximum {MAX_TUPLE_ELEMENTS}"
                    ),
                });
            }
        }

        OpCode::GetTuple => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "tuple_reg")?;
            const MAX_TUPLE_ELEMENTS: u16 = 4096;
            let tuple_slot = instruction.src2();
            if tuple_slot >= MAX_TUPLE_ELEMENTS {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: tuple slot index {tuple_slot} exceeds maximum {MAX_TUPLE_ELEMENTS}"
                    ),
                });
            }
        }

        OpCode::PackClosure => {
            check_reg(instruction.call_dst(), "closure_dst")?;
            let target_function_id = instruction.call_target() as usize;
            if target_function_id >= total_functions {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: closure target function ID {target_function_id} exceeds total function count {total_functions}"
                    ),
                });
            }
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

        OpCode::Instanceof | OpCode::CastCheck => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "instance_reg")?;
        }

        OpCode::TupleInstanceof => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "tuple_reg")?;
            check_reg(instruction.src2(), "index_reg")?;
        }

        OpCode::Throw => {
            check_reg(instruction.dst(), "exception_reg")?;
        }

        OpCode::IncRef | OpCode::DecRef => {
            check_reg(instruction.dst(), "target_reg")?;
        }

        // Superinstructions
        OpCode::AddI32Imm
        | OpCode::SubI32Imm
        | OpCode::MulI32Imm
        | OpCode::ModI32Imm
        | OpCode::AddI64Imm => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "src1")?;
        }

        OpCode::AddBandI32 | OpCode::MulAddF64 => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.flags() as u16, "acc_reg")?;
            check_reg(instruction.src1(), "val_reg")?;
            check_reg(instruction.src2(), "src2_reg")?;
        }

        OpCode::GetArrayJumpIfFalse | OpCode::GetArrayJumpIfTrue => {
            check_reg(instruction.dst(), "arr_reg")?;
            check_reg(instruction.src1(), "idx_reg")?;
            let offset = instruction.src2() as i16 as i32;
            check_branch_target(offset)?;
        }

        OpCode::AddF64Imm | OpCode::SubF64Imm | OpCode::MulF64Imm => {
            check_reg(instruction.dst(), "dst")?;
            check_reg(instruction.src1(), "src1")?;
            let pool_index = instruction.src2() as usize;
            if pool_index >= constant_pool_len {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: constant pool index {pool_index} exceeds constant pool size {constant_pool_len}"
                    ),
                });
            }
        }

        OpCode::ForLoopI32 => {
            check_reg(instruction.dst(), "idx_reg")?;
            check_reg(instruction.src1(), "bound_reg")?;
            let loop_flags = instruction.flags();
            if (loop_flags & 0x80) != 0 {
                let step_reg = (loop_flags & 0x1F) as u16;
                check_reg(step_reg, "step_reg")?;
            } else if (loop_flags & 0x0F) != 0 {
                return Err(RuntimeError::BinaryLoadError {
                    reason: format!(
                        "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: invalid ForLoopI32 reserved flags 0x{loop_flags:02X}"
                    ),
                });
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
        | OpCode::JumpIfNeqI32
        | OpCode::JumpIfLtF64
        | OpCode::JumpIfLteF64
        | OpCode::JumpIfLtI64
        | OpCode::JumpIfLteI64 => {
            check_reg(instruction.dst(), "reg_a")?;
            check_reg(instruction.src1(), "reg_b")?;
            let offset = instruction.src2() as i16 as i32;
            check_branch_target(offset)?;
        }
    }

    Ok(())
}

fn check_field_bounds(
    class_index: &ClassVerificationIndex,
    opcode: OpCode,
    flags: u8,
    field_offset: usize,
    function_symbol: &str,
    instruction_index: usize,
) -> Result<(), RuntimeError> {
    if !class_index.has_class_metadata {
        return Ok(());
    }
    let tag_bits = match opcode {
        OpCode::GetField if flags > 1 => flags >> 1,
        OpCode::SetField => (flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1,
        _ => 0,
    };
    let min_width = match opcode {
        OpCode::GetFieldF64 | OpCode::SetFieldF64 => 8usize,
        OpCode::GetFieldI32 | OpCode::SetFieldI32 => 4usize,
        _ => match tag_bits {
            7 | 8 | 10 | 12 => 8,
            5 | 6 | 9 => 4,
            3 | 4 => 2,
            _ => 1,
        },
    };
    if field_offset < 8
        || !field_offset.is_multiple_of(4)
        || field_offset
            .checked_add(min_width)
            .is_none_or(|end_offset| end_offset > class_index.max_instance_size)
    {
        return Err(RuntimeError::BinaryLoadError {
            reason: format!(
                "AOT verifier failed in function '{function_symbol}' at PC {instruction_index}: field offset {field_offset} (width {min_width}) violates object bounds/alignment (max instance size: {})",
                class_index.max_instance_size
            ),
        });
    }
    Ok(())
}
