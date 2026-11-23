//! Flow-sensitive Escape Analysis, Scalar Replacement of Aggregates (SRA),
//! and Automated Reference Counting (ARC) Elision.
//!
//! Conforms to Milestone 055 specifications.

#![forbid(unsafe_code)]

use solix_common::bytecode::{CompiledClass, CompiledFunction, Instruction, OpCode};
use std::collections::{HashMap, HashSet};

/// Lifetime and escape classification of a heap-allocated aggregate or reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeCategory {
    /// Object never escapes the local function frame.
    /// Safe for Scalar Replacement of Aggregates (SRA) and total ARC elimination.
    NoEscape,
    /// Object escapes to a callee via non-escaping or borrowed parameter.
    /// Safe for caller/callee ARC elision.
    ArgEscape,
    /// Object escapes to a heap field, array, static partition, return value, or throw.
    /// Must remain allocated on the heap.
    GlobalEscape,
}

/// Identifies whether an instruction causes the target register (or its aliases) to escape.
#[must_use]
pub fn evaluate_instruction_escape(
    instruction: &Instruction,
    target_registers: &HashSet<u16>,
) -> Option<EscapeCategory> {
    let raw_opcode = instruction.raw_opcode();

    // 1. ReturnValue: variable returned from function -> GlobalEscape
    if raw_opcode == OpCode::ReturnValue as u8 && target_registers.contains(&instruction.dst()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 2. Throw: variable thrown as exception -> GlobalEscape
    if raw_opcode == OpCode::Throw as u8 && target_registers.contains(&instruction.dst()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 3. SetField: if candidate is the value being stored into a field -> GlobalEscape
    if (raw_opcode == OpCode::SetField as u8
        || raw_opcode == OpCode::SetFieldI32 as u8
        || raw_opcode == OpCode::SetFieldF64 as u8)
        && target_registers.contains(&instruction.dst())
    {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 4. SetArray: if candidate is stored into an array -> GlobalEscape
    if raw_opcode == OpCode::SetArray as u8 && target_registers.contains(&instruction.src2()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 5. SetStatic: if candidate is stored into static partition -> GlobalEscape
    if (raw_opcode == OpCode::SetStatic as u8
        || raw_opcode == OpCode::SetStaticDirect as u8
        || raw_opcode == OpCode::SetStaticGlobal as u8)
        && (target_registers.contains(&instruction.dst())
            || target_registers.contains(&instruction.src1()))
    {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 6. Dynamic dispatch calls: CallVirtual, CallInterface, CallVirtualDirect -> GlobalEscape
    if raw_opcode == OpCode::CallVirtual as u8
        || raw_opcode == OpCode::CallInterface as u8
        || raw_opcode == OpCode::CallVirtualDirect as u8
    {
        let argument_base = instruction.arg_base();
        let argument_count = instruction.arg_count() as u16;
        for reg_offset in 0..argument_count {
            if target_registers.contains(&(argument_base + reg_offset)) {
                return Some(EscapeCategory::GlobalEscape);
            }
        }
    }

    // 7. External or native calls: CallExtern, CallNativeDirect -> GlobalEscape
    if raw_opcode == OpCode::CallExtern as u8 || raw_opcode == OpCode::CallNativeDirect as u8 {
        let argument_base = instruction.arg_base();
        let argument_count = instruction.arg_count() as u16;
        for reg_offset in 0..argument_count {
            if target_registers.contains(&(argument_base + reg_offset)) {
                return Some(EscapeCategory::GlobalEscape);
            }
        }
    }

    // 8. Closure packing: PackClosure -> GlobalEscape
    if raw_opcode == OpCode::PackClosure as u8 {
        let argument_base = instruction.arg_base();
        let argument_count = instruction.arg_count() as u16;
        for reg_offset in 0..argument_count {
            if target_registers.contains(&(argument_base + reg_offset)) {
                return Some(EscapeCategory::GlobalEscape);
            }
        }
    }

    // 9. Polymorphic type checking: Instanceof, CastCheck -> GlobalEscape
    if (raw_opcode == OpCode::Instanceof as u8
        || raw_opcode == OpCode::CastCheck as u8
        || raw_opcode == OpCode::TupleInstanceof as u8)
        && (target_registers.contains(&instruction.dst())
            || target_registers.contains(&instruction.src1()))
    {
        return Some(EscapeCategory::GlobalEscape);
    }

    None
}

/// Analyzes a constructor function. If it is a pure field-forwarding constructor
/// (only `SetField` from parameters to `this`, followed by `Return`), returns a mapping
/// of `field_offset -> parameter_index`.
fn analyze_forwarding_constructor(
    constructor_function: &CompiledFunction,
) -> Option<HashMap<u16, u16>> {
    let mut field_to_param = HashMap::new();

    for instruction in &constructor_function.instructions {
        let raw_opcode = instruction.raw_opcode();
        if raw_opcode == OpCode::Nop as u8
            || raw_opcode == OpCode::Return as u8
            || raw_opcode == OpCode::ReturnValue as u8
            || raw_opcode == OpCode::Mov as u8
        {
            continue;
        }
        if raw_opcode == OpCode::SetField as u8 && instruction.src1() == 0 {
            let param_register = instruction.dst();
            let field_offset = instruction.src2();
            if param_register > 0 {
                field_to_param.insert(field_offset, param_register);
                continue;
            }
        }
        // Any other instruction means constructor has non-trivial logic
        return None;
    }

    if field_to_param.is_empty() {
        None
    } else {
        Some(field_to_param)
    }
}

/// Analyzes a pure single-field getter method (only `GetField` from `this` (`r0`), optional
/// `Mov` forwarding, and `ReturnValue`, with empty exception handlers). Returns `Some(field_offset)`
/// when safe to forward during scalar replacement.
fn analyze_forwarding_getter(getter_function: &CompiledFunction) -> Option<u16> {
    if !getter_function.exception_handlers.is_empty() {
        return None;
    }

    let mut tracked_reg: Option<u16> = None;
    let mut tracked_offset: Option<u16> = None;

    for instruction in &getter_function.instructions {
        let raw_opcode = instruction.raw_opcode();
        if raw_opcode == OpCode::Nop as u8 {
            continue;
        }
        if raw_opcode == OpCode::GetField as u8
            && instruction.src1() == 0
            && tracked_offset.is_none()
        {
            tracked_reg = Some(instruction.dst());
            tracked_offset = Some(instruction.src2());
            continue;
        }
        if raw_opcode == OpCode::IncRef as u8 && Some(instruction.dst()) == tracked_reg {
            // Safe when caller already verified the class has no reference fields
            continue;
        }
        if raw_opcode == OpCode::Mov as u8 && Some(instruction.src1()) == tracked_reg {
            tracked_reg = Some(instruction.dst());
            continue;
        }
        if raw_opcode == OpCode::ReturnValue as u8 && Some(instruction.dst()) == tracked_reg {
            return tracked_offset;
        }
        return None;
    }

    None
}

/// Decodes jump/branch target PC if `instruction` is any branch/jump within `[0..=inst_count]`.
fn decode_branch_target(
    instruction: &Instruction,
    pc_index: usize,
    inst_count: usize,
) -> Option<usize> {
    let raw_opcode = instruction.raw_opcode();
    if raw_opcode == OpCode::Jump as u8
        || raw_opcode == OpCode::JumpIfTrue as u8
        || raw_opcode == OpCode::JumpIfFalse as u8
    {
        let target_pc = (pc_index as i32) + instruction.jump_offset();
        if target_pc >= 0 && (target_pc as usize) <= inst_count {
            return Some(target_pc as usize);
        }
    } else if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode)
        || raw_opcode == OpCode::ForLoopI32 as u8
        || raw_opcode == OpCode::JumpIfI32Imm as u8
        || raw_opcode == OpCode::GetArrayJumpIfFalse as u8
        || raw_opcode == OpCode::GetArrayJumpIfTrue as u8
        || raw_opcode == OpCode::JumpIfLtF64 as u8
        || raw_opcode == OpCode::JumpIfLteF64 as u8
        || raw_opcode == OpCode::JumpIfLtI64 as u8
        || raw_opcode == OpCode::JumpIfLteI64 as u8
    {
        let target_pc = (pc_index as i32) + (instruction.src2() as i16 as i32);
        if target_pc >= 0 && (target_pc as usize) <= inst_count {
            return Some(target_pc as usize);
        }
    }
    None
}

#[inline(always)]
fn is_unsigned_comparison_opcode(raw_opcode_value: u8) -> bool {
    (raw_opcode_value >= OpCode::LtU32 as u8 && raw_opcode_value <= OpCode::GtU64 as u8)
        || raw_opcode_value == OpCode::GteU64 as u8
}

/// Returns `Some(reg)` if `instruction` overwrites a single destination register `reg`.
#[must_use]
pub fn instruction_write_dst(instruction: &Instruction) -> Option<u16> {
    let raw = instruction.raw_opcode();
    if (OpCode::Mov as u8..=OpCode::LoadConst as u8).contains(&raw)
        || (OpCode::AddI32 as u8..=OpCode::Not as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
        || is_unsigned_comparison_opcode(raw)
        || is_unsigned_comparison_opcode(raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::ForLoopI32 as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8
        || raw == OpCode::AllocObject as u8
        || raw == OpCode::AllocObjectExtern as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::GetStatic as u8
        || raw == OpCode::GetStaticDirect as u8
        || raw == OpCode::GetStaticGlobal as u8
        || raw == OpCode::AllocArray as u8
        || raw == OpCode::GetArray as u8
        || raw == OpCode::ArrayLength as u8
        || raw == OpCode::AllocTuple as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::Instanceof as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::TupleInstanceof as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::GetFieldI32 as u8
        || raw == OpCode::GetFieldF64 as u8
    {
        return Some(instruction.dst());
    }

    if raw == OpCode::CallDirect as u8
        || raw == OpCode::TailCallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
        || raw == OpCode::CallVirtualDirect as u8
    {
        return Some(instruction.call_dst());
    }

    None
}

/// Returns `true` if `instruction` writes to `target_reg` (including `UnpackCaptures` register windows).
#[must_use]
pub fn instruction_writes_reg(instruction: &Instruction, target_reg: u16) -> bool {
    if instruction.raw_opcode() == OpCode::UnpackCaptures as u8 {
        let base = instruction.arg_base();
        let count = instruction.arg_count() as u16;
        return target_reg >= base && target_reg < base + count;
    }
    instruction_write_dst(instruction) == Some(target_reg)
}

/// Returns `true` if `instruction` reads `target_reg`.
#[must_use]
pub fn instruction_reads_reg(instruction: &Instruction, target_reg: u16) -> bool {
    let raw = instruction.raw_opcode();
    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
        || raw == OpCode::JumpIfLtF64 as u8
        || raw == OpCode::JumpIfLteF64 as u8
        || raw == OpCode::JumpIfLtI64 as u8
        || raw == OpCode::JumpIfLteI64 as u8
    {
        return instruction.dst() == target_reg || instruction.src1() == target_reg;
    }
    if raw == OpCode::JumpIfI32Imm as u8 {
        return instruction.dst() == target_reg;
    }
    if raw == OpCode::ForLoopI32 as u8 {
        let reads_step =
            (instruction.flags() & 0x80) != 0 && (instruction.flags() & 0x1F) as u16 == target_reg;
        return instruction.dst() == target_reg || instruction.src1() == target_reg || reads_step;
    }
    if raw == OpCode::AddBandI32 as u8 || raw == OpCode::MulAddF64 as u8 {
        return (instruction.flags() as u16) == target_reg
            || instruction.src1() == target_reg
            || instruction.src2() == target_reg;
    }
    if raw == OpCode::GetArrayJumpIfFalse as u8 || raw == OpCode::GetArrayJumpIfTrue as u8 {
        return instruction.dst() == target_reg || instruction.src1() == target_reg;
    }
    if raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
        || raw == OpCode::AddI64Imm as u8
    {
        return instruction.src1() == target_reg;
    }
    if raw == OpCode::Mov as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::Not as u8
        || raw == OpCode::Bnot as u8
        || raw == OpCode::NegI32 as u8
        || raw == OpCode::NegI64 as u8
        || raw == OpCode::NegF64 as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::GetFieldI32 as u8
        || raw == OpCode::GetFieldF64 as u8
        || raw == OpCode::ArrayLength as u8
        || raw == OpCode::AllocArray as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::Instanceof as u8
    {
        return instruction.src1() == target_reg;
    }
    if raw == OpCode::TupleInstanceof as u8 {
        return instruction.src1() == target_reg || instruction.src2() == target_reg;
    }
    if raw == OpCode::DecRef as u8
        || raw == OpCode::IncRef as u8
        || raw == OpCode::Exit as u8
        || raw == OpCode::Throw as u8
        || raw == OpCode::ReturnValue as u8
    {
        return instruction.dst() == target_reg;
    }
    if raw == OpCode::SetField as u8
        || raw == OpCode::SetFieldI32 as u8
        || raw == OpCode::SetFieldF64 as u8
    {
        return instruction.dst() == target_reg || instruction.src1() == target_reg;
    }
    if raw == OpCode::GetArray as u8 {
        return instruction.src1() == target_reg || instruction.src2() == target_reg;
    }
    if raw == OpCode::SetArray as u8 {
        return instruction.dst() == target_reg
            || instruction.src1() == target_reg
            || instruction.src2() == target_reg;
    }
    if (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
    {
        return instruction.src1() == target_reg || instruction.src2() == target_reg;
    }
    if raw == OpCode::SetStatic as u8
        || raw == OpCode::SetStaticDirect as u8
        || raw == OpCode::SetStaticGlobal as u8
    {
        return instruction.dst() == target_reg;
    }
    if raw == OpCode::JumpIfTrue as u8
        || raw == OpCode::JumpIfFalse as u8
        || raw == OpCode::Switch as u8
    {
        return instruction.cond_reg() == target_reg;
    }
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::TailCallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
        || raw == OpCode::CallVirtualDirect as u8
    {
        if raw == OpCode::CallCallable as u8 && instruction.call_target() == target_reg {
            return true;
        }
        let base = instruction.arg_base();
        let count = instruction.arg_count() as u16;
        return target_reg >= base && target_reg < base + count;
    }
    false
}

/// Checks whether ANY execution path starting at `start_pc` can read `reg` (ignoring `DecRef`/`IncRef`
/// on `reg` only when `ignore_arc` is false? Wait: any read, including `GetField`/`SetField`/`ReturnValue`,
/// BEFORE `reg` is unconditionally overwritten).
fn is_reg_read_before_write_from(
    function: &CompiledFunction,
    jump_targets: &[Option<usize>],
    start_pc: usize,
    reg: u16,
) -> bool {
    let len = function.instructions.len();
    if start_pc >= len {
        return false;
    }

    let mut visited = vec![false; len];
    let mut worklist = vec![start_pc];

    while let Some(entry_pc) = worklist.pop() {
        for cur in entry_pc..len {
            if visited[cur] {
                break;
            }
            visited[cur] = true;

            let inst = &function.instructions[cur];
            let raw = inst.raw_opcode();

            if instruction_reads_reg(inst, reg) {
                return true;
            }

            if instruction_write_dst(inst) == Some(reg) {
                break;
            }

            if raw == OpCode::Return as u8
                || raw == OpCode::ReturnValue as u8
                || raw == OpCode::Exit as u8
                || raw == OpCode::Throw as u8
            {
                break;
            }

            if raw == OpCode::Jump as u8 {
                if let Some(target_pc) = jump_targets[cur] {
                    worklist.push(target_pc);
                } else {
                    return true;
                }
                break;
            }

            if raw == OpCode::JumpIfTrue as u8
                || raw == OpCode::JumpIfFalse as u8
                || (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
                || raw == OpCode::ForLoopI32 as u8
                || raw == OpCode::JumpIfI32Imm as u8
                || raw == OpCode::GetArrayJumpIfFalse as u8
                || raw == OpCode::GetArrayJumpIfTrue as u8
                || raw == OpCode::JumpIfLtF64 as u8
                || raw == OpCode::JumpIfLteF64 as u8
                || raw == OpCode::JumpIfLtI64 as u8
                || raw == OpCode::JumpIfLteI64 as u8
            {
                if let Some(target_pc) = jump_targets[cur] {
                    worklist.push(target_pc);
                } else {
                    return true;
                }
            } else if raw == OpCode::Switch as u8 {
                return true;
            }
        }
    }

    false
}

/// Rewrites the destination register of an instruction that writes a single destination (`Format A/B` or `Format Call`).
fn rewrite_any_write_dst(instruction: &Instruction, new_dst: u16) -> Option<Instruction> {
    let raw = instruction.raw_opcode();
    if (OpCode::Mov as u8..=OpCode::LoadConst as u8).contains(&raw)
        || (OpCode::AddI32 as u8..=OpCode::Not as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
        || is_unsigned_comparison_opcode(raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::GetArray as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::ArrayLength as u8
        || raw == OpCode::GetStatic as u8
        || raw == OpCode::GetStaticDirect as u8
        || raw == OpCode::GetStaticGlobal as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::Instanceof as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::TupleInstanceof as u8
    {
        return Some(instruction.with_dst(new_dst));
    }

    if raw == OpCode::CallDirect as u8
        || raw == OpCode::TailCallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
        || raw == OpCode::CallVirtualDirect as u8
    {
        let opcode = OpCode::from_raw_u8(raw)?;
        return Some(Instruction::format_call(
            opcode,
            new_dst,
            instruction.call_target(),
            instruction.arg_base(),
            instruction.arg_count(),
        ));
    }

    None
}

/// Finds the matching constructor `CallDirect` for `AllocObject` at `allocation_pc`.
fn find_constructor_call(
    function: &CompiledFunction,
    allocation_pc: usize,
    allocated_register: u16,
) -> Option<(usize, u16, usize)> {
    let limit = std::cmp::min(allocation_pc + 64, function.instructions.len());
    let mut mov_base_to_pc: HashMap<u16, usize> = HashMap::new();

    for (check_pc, inst) in function
        .instructions
        .iter()
        .enumerate()
        .take(limit)
        .skip(allocation_pc + 1)
    {
        let raw = inst.raw_opcode();

        if raw == OpCode::AllocObject as u8
            || raw == OpCode::AllocObjectExtern as u8
            || raw == OpCode::Jump as u8
            || raw == OpCode::JumpIfTrue as u8
            || raw == OpCode::JumpIfFalse as u8
            || (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
            || raw == OpCode::Return as u8
            || raw == OpCode::ReturnValue as u8
            || raw == OpCode::Throw as u8
            || raw == OpCode::Exit as u8
        {
            break;
        }

        if raw == OpCode::Mov as u8 && inst.src1() == allocated_register {
            mov_base_to_pc.insert(inst.dst(), check_pc);
            continue;
        }

        if raw == OpCode::CallDirect as u8 {
            let arg_base = inst.arg_base();
            let arg_count = inst.arg_count();
            if arg_count >= 1
                && (mov_base_to_pc.contains_key(&arg_base) || arg_base == allocated_register)
            {
                return Some((check_pc, arg_base, inst.call_target() as usize));
            }
        }

        if let Some(written) = instruction_write_dst(inst) {
            mov_base_to_pc.remove(&written);
        }
    }

    None
}

/// Performs flow-sensitive escape analysis on a compiled function, classifying
/// candidate heap allocations into `NoEscape`, `ArgEscape`, or `GlobalEscape`.
pub fn classify_allocations(
    function: &CompiledFunction,
    classes: &[CompiledClass],
    all_functions: &[CompiledFunction],
) -> HashMap<usize, (u16, EscapeCategory)> {
    let mut allocation_categories = HashMap::new();
    let inst_count = function.instructions.len();
    let mut jump_targets: Vec<Option<usize>> = Vec::with_capacity(inst_count);
    let mut backward_loops: Vec<(usize, usize)> = Vec::new();

    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        let target = decode_branch_target(instruction, pc_index, inst_count);
        jump_targets.push(target);
        if let Some(target_pc) = target
            && target_pc <= pc_index
        {
            backward_loops.push((target_pc, pc_index));
        }
    }

    for (instruction_index, instruction) in function.instructions.iter().enumerate() {
        if instruction.raw_opcode() == OpCode::AllocObject as u8 {
            let allocated_register = instruction.dst();
            let vtable_identifier = instruction.src1() as u32;

            // Find class descriptor
            let class_candidate = classes
                .iter()
                .find(|class_descriptor| class_descriptor.vtable_id == vtable_identifier);

            let Some(class_descriptor) = class_candidate else {
                allocation_categories.insert(
                    instruction_index,
                    (allocated_register, EscapeCategory::GlobalEscape),
                );
                continue;
            };

            // Non-aggregates (classes with destructors, inheritance, or reference fields) must escape
            if class_descriptor.destructor_id.is_some()
                || class_descriptor.base_vtable_id.is_some()
                || !class_descriptor.itable.is_empty()
                || !class_descriptor.reference_field_offsets.is_empty()
            {
                allocation_categories.insert(
                    instruction_index,
                    (allocated_register, EscapeCategory::GlobalEscape),
                );
                continue;
            }

            // Collect any backward-jump loops enclosing this allocation
            let mut enclosing_outer_hdr: Option<usize> = None;
            let mut enclosing_outer_back: Option<usize> = None;
            for &(hdr, back) in &backward_loops {
                if hdr <= instruction_index && instruction_index <= back {
                    enclosing_outer_hdr =
                        Some(enclosing_outer_hdr.map_or(hdr, |cur_hdr| cur_hdr.min(hdr)));
                    enclosing_outer_back =
                        Some(enclosing_outer_back.map_or(back, |cur_back| cur_back.max(back)));
                }
            }

            // Exception handlers crossing an enclosing allocation require conservative heap preservation
            if !function.exception_handlers.is_empty() && enclosing_outer_hdr.is_some() {
                allocation_categories.insert(
                    instruction_index,
                    (allocated_register, EscapeCategory::GlobalEscape),
                );
                continue;
            }

            // Track constructor window base if CallDirect is called immediately
            let ctor_info = find_constructor_call(function, instruction_index, allocated_register);
            let constructor_call_pc_index = ctor_info.map(|(call_pc, _, _)| call_pc);
            let constructor_window_base = ctor_info.map(|(_, arg_base, _)| arg_base);
            let constructor_target_id = ctor_info.map(|(_, _, func_id)| func_id);

            // If a constructor exists, it must be a pure field-forwarding constructor
            if let Some(target_func_index) = constructor_target_id {
                let is_pure_constructor = all_functions
                    .get(target_func_index)
                    .and_then(analyze_forwarding_constructor)
                    .is_some();

                if !is_pure_constructor {
                    allocation_categories.insert(
                        instruction_index,
                        (allocated_register, EscapeCategory::GlobalEscape),
                    );
                    continue;
                }
            }

            let scan_end = enclosing_outer_back.map_or(inst_count, |outer_back| outer_back + 1);
            let mut target_registers = HashSet::new();
            target_registers.insert(allocated_register);
            let mut ever_aliased_registers = target_registers.clone();
            let mut overall_category = EscapeCategory::NoEscape;

            for (inspection_index, inspection_instruction) in function
                .instructions
                .iter()
                .enumerate()
                .take(scan_end)
                .skip(instruction_index + 1)
            {
                let raw_opcode = inspection_instruction.raw_opcode();

                // Check for aliasing assignment or register overwrite
                if raw_opcode == OpCode::Mov as u8 {
                    let source_register = inspection_instruction.src1();
                    let destination_register = inspection_instruction.dst();
                    if target_registers.contains(&source_register) {
                        target_registers.insert(destination_register);
                        ever_aliased_registers.insert(destination_register);
                    } else {
                        target_registers.remove(&destination_register);
                    }
                    continue;
                }

                // Check CallDirect (constructor or pure getter vs escaping call):
                if raw_opcode == OpCode::CallDirect as u8 {
                    let argument_base = inspection_instruction.arg_base();
                    let argument_count = inspection_instruction.arg_count() as u16;
                    let call_dst = inspection_instruction.call_dst();

                    let is_immediate_constructor =
                        Some(inspection_index) == constructor_call_pc_index;

                    if is_immediate_constructor {
                        if let Some(window_base) = constructor_window_base {
                            target_registers.remove(&window_base);
                        }
                    } else if argument_count == 1 && target_registers.contains(&argument_base) {
                        let getter_offset = all_functions
                            .get(inspection_instruction.call_target() as usize)
                            .and_then(analyze_forwarding_getter);
                        if getter_offset.is_some() {
                            target_registers.remove(&call_dst);
                            continue;
                        }
                        overall_category = EscapeCategory::GlobalEscape;
                        break;
                    } else {
                        for reg_offset in 0..argument_count {
                            if target_registers.contains(&(argument_base + reg_offset)) {
                                overall_category = EscapeCategory::GlobalEscape;
                                break;
                            }
                        }
                        if overall_category == EscapeCategory::GlobalEscape {
                            break;
                        }
                    }
                    target_registers.remove(&call_dst);
                    continue;
                }

                if let Some(category) =
                    evaluate_instruction_escape(inspection_instruction, &target_registers)
                {
                    if category == EscapeCategory::GlobalEscape {
                        overall_category = EscapeCategory::GlobalEscape;
                        break;
                    } else if category == EscapeCategory::ArgEscape
                        && overall_category == EscapeCategory::NoEscape
                    {
                        overall_category = EscapeCategory::ArgEscape;
                    }
                }

                // Clear overwritten registers from live aliases
                if let Some(written_reg) = instruction_write_dst(inspection_instruction) {
                    target_registers.remove(&written_reg);
                }
            }

            // Check non-enclosing loops that both use and overwrite an alias of this allocation
            // (e.g. `Holder saved = new Holder(5); for (...) { sum += saved.val; saved = fresh; }`).
            if overall_category == EscapeCategory::NoEscape && enclosing_outer_hdr.is_none() {
                for &(hdr, back) in &backward_loops {
                    if instruction_index < hdr && back < inst_count {
                        let loop_slice = &function.instructions[hdr..=back];
                        let overwritten_in_loop: HashSet<u16> = loop_slice
                            .iter()
                            .filter_map(instruction_write_dst)
                            .filter(|reg| ever_aliased_registers.contains(reg))
                            .collect();
                        if !overwritten_in_loop.is_empty()
                            && loop_slice.iter().any(|loop_inst| {
                                overwritten_in_loop
                                    .iter()
                                    .any(|&reg| instruction_reads_reg(loop_inst, reg))
                            })
                        {
                            overall_category = EscapeCategory::GlobalEscape;
                            break;
                        }
                    }
                }
            }

            // Verify strict single-iteration confinement when enclosed in loop(s)
            if overall_category == EscapeCategory::NoEscape
                && let (Some(_outer_hdr), Some(outer_back)) =
                    (enclosing_outer_hdr, enclosing_outer_back)
            {
                // 1. No branch before allocation_pc may jump past allocation_pc and read an alias
                for pre_pc in 0..instruction_index {
                    if let Some(target_pc) = jump_targets[pre_pc]
                        && target_pc > instruction_index
                        && target_pc <= outer_back
                        && ever_aliased_registers.iter().any(|&reg| {
                            is_reg_read_before_write_from(function, &jump_targets, target_pc, reg)
                        })
                    {
                        overall_category = EscapeCategory::GlobalEscape;
                        break;
                    }
                }

                // 2. Back-edges must not carry live aliases into the next iteration before overwrite
                if overall_category == EscapeCategory::NoEscape {
                    let backedge_carries_alias = backward_loops.iter().any(|&(hdr, back)| {
                        hdr <= instruction_index
                            && instruction_index <= back
                            && ever_aliased_registers.iter().any(|&reg| {
                                is_reg_read_before_write_from(function, &jump_targets, hdr, reg)
                            })
                    });
                    if backedge_carries_alias {
                        overall_category = EscapeCategory::GlobalEscape;
                    }
                }

                // 3. Forward jumps leaving the enclosing loop must not expose aliases post-loop
                if overall_category == EscapeCategory::NoEscape {
                    for body_pc in instruction_index..=outer_back {
                        if let Some(target_pc) = jump_targets[body_pc]
                            && target_pc > outer_back
                            && ever_aliased_registers.iter().any(|&reg| {
                                is_reg_read_before_write_from(
                                    function,
                                    &jump_targets,
                                    target_pc,
                                    reg,
                                )
                            })
                        {
                            overall_category = EscapeCategory::GlobalEscape;
                            break;
                        }
                    }
                }

                // 4. Fallthrough after outer_back must not read any alias
                if overall_category == EscapeCategory::NoEscape
                    && (outer_back + 1) < inst_count
                    && function.instructions[outer_back].raw_opcode() != OpCode::Jump as u8
                    && ever_aliased_registers.iter().any(|&reg| {
                        is_reg_read_before_write_from(function, &jump_targets, outer_back + 1, reg)
                    })
                {
                    overall_category = EscapeCategory::GlobalEscape;
                }
            }

            allocation_categories.insert(instruction_index, (allocated_register, overall_category));
        }
    }

    allocation_categories
}

/// Applies Scalar Replacement of Aggregates (SRA) on non-escaping objects (including
/// iteration-local objects inside loops and calls to pure single-field getter methods).
///
/// Replaces `AllocObject` with `Nop`, forwards field writes and pure getter calls directly
/// to scalar field registers, and removes redundant `DecRef` calls.
pub fn apply_scalar_replacement(
    function: &mut CompiledFunction,
    classes: &[CompiledClass],
    all_functions: &[CompiledFunction],
) -> usize {
    let mut scalar_replaced_count = 0;

    loop {
        let allocations = classify_allocations(function, classes, all_functions);
        let mut sorted_allocs: Vec<(usize, u16)> = allocations
            .into_iter()
            .filter_map(|(pc, (reg, category))| {
                if category == EscapeCategory::NoEscape {
                    Some((pc, reg))
                } else {
                    None
                }
            })
            .collect();
        sorted_allocs.sort_by_key(|&(pc, _)| pc);

        let mut replaced_this_round = false;
        for (allocation_pc, allocated_register) in sorted_allocs {
            if try_scalar_replace_one(function, allocation_pc, allocated_register, all_functions) {
                scalar_replaced_count += 1;
                replaced_this_round = true;
                break;
            }
        }

        if !replaced_this_round {
            break;
        }
    }

    scalar_replaced_count
}

/// Attempts to scalar-replace a single verified `NoEscape` allocation at `allocation_pc`.
fn try_scalar_replace_one(
    function: &mut CompiledFunction,
    allocation_pc: usize,
    allocated_register: u16,
    all_functions: &[CompiledFunction],
) -> bool {
    let inst_count = function.instructions.len();
    let mut enclosing_outer_back: Option<usize> = None;
    for (pc_index, inst) in function.instructions.iter().enumerate() {
        if let Some(target_pc) = decode_branch_target(inst, pc_index, inst_count)
            && target_pc <= allocation_pc
            && allocation_pc <= pc_index
        {
            enclosing_outer_back =
                Some(enclosing_outer_back.map_or(pc_index, |cur| cur.max(pc_index)));
        }
    }
    let rewrite_limit = enclosing_outer_back.map_or(inst_count, |outer_back| outer_back + 1);

    let ctor_info = find_constructor_call(function, allocation_pc, allocated_register);
    let constructor_call_pc = ctor_info.map(|(call_pc, _, _)| call_pc);
    let constructor_arg_base = ctor_info.map(|(_, arg_base, _)| arg_base);
    let constructor_target_func_index = ctor_info.map(|(_, _, func_id)| func_id);

    // Phase A: Pre-validate all constructor args, getter reads, and field reads before mutating anything
    let mut sim_aliases = HashSet::new();
    sim_aliases.insert(allocated_register);
    let mut ever_aliases = sim_aliases.clone();
    let mut sim_known_fields: HashSet<u16> = HashSet::new();

    let ctor_field_to_param = if let (Some(target_func_idx), Some(_)) =
        (constructor_target_func_index, constructor_call_pc)
    {
        let Some(target_func) = all_functions.get(target_func_idx) else {
            return false;
        };
        let Some(field_to_param) = analyze_forwarding_constructor(target_func) else {
            return false;
        };
        for &offset in field_to_param.keys() {
            sim_known_fields.insert(offset);
        }
        Some(field_to_param)
    } else {
        None
    };

    // Validate constructor argument producers if present
    let mut ctor_plan: Vec<(u16, Option<(usize, Instruction)>)> = Vec::new();
    if let (Some(ctor_base), Some(field_to_param), Some(ctor_pc)) = (
        constructor_arg_base,
        &ctor_field_to_param,
        constructor_call_pc,
    ) {
        let mut sorted_offsets: Vec<u16> = field_to_param.keys().copied().collect();
        sorted_offsets.sort_unstable();
        let mut missing_writer_count = 0;

        for &offset in &sorted_offsets {
            let param_index = field_to_param[&offset];
            let argument_register = ctor_base + param_index;
            let mut found_writer: Option<(usize, Instruction)> = None;

            for search_pc in (allocation_pc + 1..ctor_pc).rev() {
                let cand = function.instructions[search_pc];
                if instruction_write_dst(&cand) == Some(argument_register) {
                    let read_after_writer = function.instructions[search_pc + 1..ctor_pc]
                        .iter()
                        .any(|later| instruction_reads_reg(later, argument_register));
                    if !read_after_writer {
                        found_writer = Some((search_pc, cand));
                    }
                    break;
                }
            }

            if found_writer.is_none() {
                missing_writer_count += 1;
            }
            ctor_plan.push((offset, found_writer));
        }

        if missing_writer_count > 1 {
            return false;
        }
    }

    let mut last_use_or_write_pc = allocation_pc;
    let mut sim_check_aliases = HashSet::new();
    sim_check_aliases.insert(allocated_register);
    let mut can_finish = true;
    for (inspection_index, &inst) in function
        .instructions
        .iter()
        .enumerate()
        .take(rewrite_limit)
        .skip(allocation_pc + 1)
    {
        let raw_opcode = inst.raw_opcode();

        if raw_opcode == OpCode::Mov as u8 {
            if sim_aliases.contains(&inst.src1()) {
                sim_aliases.insert(inst.dst());
                ever_aliases.insert(inst.dst());
                sim_check_aliases.insert(inst.dst());
            } else {
                sim_aliases.remove(&inst.dst());
            }
            continue;
        }

        if raw_opcode == OpCode::SetField as u8 && sim_aliases.contains(&inst.src1()) {
            sim_known_fields.insert(inst.src2());
            last_use_or_write_pc = inspection_index;
        } else if raw_opcode == OpCode::GetField as u8 && sim_aliases.contains(&inst.src1()) {
            if !sim_known_fields.contains(&inst.src2()) {
                return false;
            }
            last_use_or_write_pc = inspection_index;
        } else if raw_opcode == OpCode::CallDirect as u8 {
            let arg_base = inst.arg_base();
            let arg_count = inst.arg_count();
            let is_ctor = Some(inspection_index) == constructor_call_pc;
            if is_ctor {
                sim_aliases.remove(&arg_base);
                last_use_or_write_pc = inspection_index;
            } else if arg_count == 1 && sim_aliases.contains(&arg_base) {
                let Some(getter_func) = all_functions.get(inst.call_target() as usize) else {
                    return false;
                };
                let Some(getter_offset) = analyze_forwarding_getter(getter_func) else {
                    return false;
                };
                if !sim_known_fields.contains(&getter_offset) {
                    return false;
                }
                last_use_or_write_pc = inspection_index;
            } else {
                for offset in 0..arg_count as u16 {
                    if sim_aliases.contains(&(arg_base + offset)) {
                        can_finish = false;
                        break;
                    }
                }
                if !can_finish {
                    return false;
                }
            }
        }

        if let Some(written) = instruction_write_dst(&inst) {
            sim_aliases.remove(&written);
        }
    }

    let mut reuse_alloc_reg_as_single_field = false;
    if ctor_plan.len() == 1
        && sim_known_fields.len() == 1
        && let Some((_offset, Some((writer_pc, _)))) = ctor_plan.first()
    {
        let reads_alloc_in_ctor_setup = function.instructions[(allocation_pc + 1)..*writer_pc]
            .iter()
            .any(|setup_inst| {
                let setup_raw = setup_inst.raw_opcode();
                if setup_raw == OpCode::Mov as u8
                    && setup_inst.src1() == allocated_register
                    && constructor_arg_base == Some(setup_inst.dst())
                {
                    return false;
                }
                instruction_reads_reg(setup_inst, allocated_register)
            });
        if !reads_alloc_in_ctor_setup {
            reuse_alloc_reg_as_single_field = true;
        }
    }

    if !reuse_alloc_reg_as_single_field {
        let mut intervening_call_indices: Vec<usize> = Vec::new();
        for (check_pc, check_inst) in function
            .instructions
            .iter()
            .enumerate()
            .take(last_use_or_write_pc + 1)
            .skip(allocation_pc + 1)
        {
            let raw = check_inst.raw_opcode();
            if raw == OpCode::CallDirect as u8
                || raw == OpCode::CallVirtual as u8
                || raw == OpCode::CallInterface as u8
                || raw == OpCode::CallCallable as u8
                || raw == OpCode::CallSymbol as u8
                || raw == OpCode::CallExtern as u8
            {
                let is_this_ctor = Some(check_pc) == constructor_call_pc;
                let is_this_getter = raw == OpCode::CallDirect as u8
                    && check_inst.arg_count() == 1
                    && sim_check_aliases.contains(&check_inst.arg_base())
                    && all_functions
                        .get(check_inst.call_target() as usize)
                        .and_then(analyze_forwarding_getter)
                        .is_some();
                if !is_this_ctor && !is_this_getter {
                    intervening_call_indices.push(check_pc);
                }
            }
        }
        if !intervening_call_indices.is_empty() {
            return false;
        }
    }

    // Check register budget before allocating scalar field registers
    if (function.num_registers as usize) + ctor_plan.len() + 4 >= (u16::MAX as usize) {
        return false;
    }

    // Phase B: Apply scalar replacement mutations cleanly
    let mut field_values: HashMap<u16, u16> = HashMap::new();
    let mut ignored_ctor_writer_pcs: HashSet<usize> = HashSet::new();
    if let (Some(ctor_base), Some(field_to_param), Some(ctor_pc)) = (
        constructor_arg_base,
        &ctor_field_to_param,
        constructor_call_pc,
    ) {
        for (offset, found_writer) in ctor_plan {
            let dedicated_register = if reuse_alloc_reg_as_single_field {
                allocated_register
            } else {
                let fresh_reg = function.num_registers;
                function.num_registers += 1;
                fresh_reg
            };
            field_values.insert(offset, dedicated_register);

            if let Some((writer_pc, writer_inst)) = found_writer
                && let Some(retargeted) = rewrite_any_write_dst(&writer_inst, dedicated_register)
            {
                function.instructions[writer_pc] = retargeted;
                if reuse_alloc_reg_as_single_field {
                    ignored_ctor_writer_pcs.insert(writer_pc);
                }
            } else {
                let param_index = field_to_param[&offset];
                let argument_register = ctor_base + param_index;
                function.instructions[ctor_pc] =
                    Instruction::format_a(OpCode::Mov, 0, dedicated_register, argument_register, 0);
            }
        }
    }

    // 1. Rewrite AllocObject to Nop
    function.instructions[allocation_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);

    // 2. Rewrite constructor CallDirect and constructor window receiver Mov to Nop
    if let (Some(ctor_pc), Some(ctor_base)) = (constructor_call_pc, constructor_arg_base) {
        if function.instructions[ctor_pc].raw_opcode() == OpCode::CallDirect as u8 {
            function.instructions[ctor_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
        }
        for check_inst in function
            .instructions
            .iter_mut()
            .take(ctor_pc)
            .skip(allocation_pc + 1)
        {
            if check_inst.raw_opcode() == OpCode::Mov as u8
                && check_inst.dst() == ctor_base
                && ever_aliases.contains(&check_inst.src1())
            {
                *check_inst = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            }
        }
    }

    // 3. Rewrite Mov aliases, SetField, GetField, pure getter CallDirect, and DecRef/IncRef
    let mut rewrite_aliases = HashSet::new();
    rewrite_aliases.insert(allocated_register);

    for current_index in (allocation_pc + 1)..rewrite_limit {
        if ignored_ctor_writer_pcs.contains(&current_index) {
            continue;
        }
        let current_instruction = function.instructions[current_index];
        let raw_opcode = current_instruction.raw_opcode();

        if raw_opcode == OpCode::Mov as u8 {
            let src_reg = current_instruction.src1();
            let dst_reg = current_instruction.dst();
            if rewrite_aliases.contains(&src_reg) {
                rewrite_aliases.insert(dst_reg);
                function.instructions[current_index] =
                    Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            } else {
                rewrite_aliases.remove(&dst_reg);
            }
            continue;
        }

        if raw_opcode == OpCode::SetField as u8
            && rewrite_aliases.contains(&current_instruction.src1())
        {
            let offset = current_instruction.src2();
            let value_register = current_instruction.dst();
            let field_reg = *field_values.entry(offset).or_insert_with(|| {
                let fresh = function.num_registers;
                function.num_registers += 1;
                fresh
            });
            if field_reg != value_register {
                function.instructions[current_index] =
                    Instruction::format_a(OpCode::Mov, 0, field_reg, value_register, 0);
            } else {
                function.instructions[current_index] =
                    Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            }
            continue;
        }

        if raw_opcode == OpCode::GetField as u8
            && rewrite_aliases.contains(&current_instruction.src1())
        {
            let offset = current_instruction.src2();
            let destination_register = current_instruction.dst();
            if let Some(&source_value_register) = field_values.get(&offset) {
                function.instructions[current_index] = Instruction::format_a(
                    OpCode::Mov,
                    0,
                    destination_register,
                    source_value_register,
                    0,
                );
            }
            rewrite_aliases.remove(&destination_register);
            continue;
        }

        if raw_opcode == OpCode::CallDirect as u8 {
            let arg_base = current_instruction.arg_base();
            let arg_count = current_instruction.arg_count();
            let call_dst = current_instruction.call_dst();
            if arg_count == 1
                && rewrite_aliases.contains(&arg_base)
                && let Some(getter_offset) = all_functions
                    .get(current_instruction.call_target() as usize)
                    .and_then(analyze_forwarding_getter)
                && let Some(&source_value_register) = field_values.get(&getter_offset)
            {
                if call_dst == source_value_register {
                    function.instructions[current_index] =
                        Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                } else {
                    function.instructions[current_index] =
                        Instruction::format_a(OpCode::Mov, 0, call_dst, source_value_register, 0);
                }
            }
            rewrite_aliases.remove(&call_dst);
            continue;
        }

        if (raw_opcode == OpCode::DecRef as u8 || raw_opcode == OpCode::IncRef as u8)
            && rewrite_aliases.contains(&current_instruction.dst())
        {
            function.instructions[current_index] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            continue;
        }

        if current_instruction.raw_opcode() == OpCode::UnpackCaptures as u8 {
            let cap_base = current_instruction.arg_base();
            let cap_count = current_instruction.arg_count() as u16;
            for cap_reg in cap_base..cap_base + cap_count {
                rewrite_aliases.remove(&cap_reg);
            }
        } else if let Some(written) = instruction_write_dst(&current_instruction) {
            rewrite_aliases.remove(&written);
        }
    }

    // 4. Clean up reference_registers when no remaining instruction writes a potential reference to that register
    for &alias_register in &ever_aliases {
        let is_catch_reg = function
            .exception_handlers
            .iter()
            .any(|handler| handler.catch_reg == alias_register);
        let can_still_hold_reference = is_catch_reg
            || function.instructions.iter().any(|inst| {
                instruction_writes_reg(inst, alias_register)
                    && instruction_can_write_reference(inst.raw_opcode())
            });
        if !can_still_hold_reference {
            function
                .reference_registers
                .retain(|candidate_register| *candidate_register != alias_register);
        }
    }

    true
}

/// Returns `false` if an opcode strictly produces primitive/null values (never a heap reference).
fn instruction_can_write_reference(raw: u8) -> bool {
    !(raw == OpCode::LoadI32 as u8
        || raw == OpCode::LoadI64 as u8
        || raw == OpCode::LoadF64 as u8
        || raw == OpCode::LoadBool as u8
        || raw == OpCode::LoadNull as u8
        || (OpCode::AddI32 as u8..=OpCode::Not as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::ForLoopI32 as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::Instanceof as u8
        || raw == OpCode::ArrayLength as u8)
}

/// Applies inter-procedural and local reference counting (ARC) elision.
///
/// Removes paired `IncRef` and `DecRef` instructions on non-escaping local aliases,
/// including across post-dominated forward conditional branch diamonds.
pub fn apply_arc_elision(function: &mut CompiledFunction) -> usize {
    let mut elided_count = 0;
    let instruction_count = function.instructions.len();
    let critical_targets = collect_arc_branch_targets(function);

    for start_index in 0..instruction_count {
        if function.instructions[start_index].raw_opcode() == OpCode::IncRef as u8 {
            let target_register = function.instructions[start_index].dst();

            let mut matching_dec_index = None;
            let mut max_forward_target: Option<usize> = None;
            let limit = std::cmp::min(start_index + 48, instruction_count);
            for (forward_index, &forward_inst) in function
                .instructions
                .iter()
                .enumerate()
                .take(limit)
                .skip(start_index + 1)
            {
                // Reject external control-flow jumps entering the middle of a diamond before a branch saw it.
                let forward_raw = forward_inst.raw_opcode();

                if forward_raw == OpCode::DecRef as u8 && forward_inst.dst() == target_register {
                    let branch_span_ok =
                        max_forward_target.is_none_or(|max_target| max_target == forward_index);
                    if branch_span_ok {
                        matching_dec_index = Some(forward_index);
                    }
                    break;
                }

                // Any jump landing inside (start..=forward_index) except at the DecRef site
                // is fine only if it came from a branch after start_index; external jumps fail below.
                if critical_targets.contains(&forward_index)
                    && max_forward_target.is_none_or(|max_target| forward_index > max_target)
                {
                    break;
                }

                if forward_raw == OpCode::Return as u8
                    || forward_raw == OpCode::ReturnValue as u8
                    || forward_raw == OpCode::Exit as u8
                    || forward_raw == OpCode::Throw as u8
                    || forward_raw == OpCode::Switch as u8
                {
                    break;
                }

                if forward_raw == OpCode::Jump as u8
                    || forward_raw == OpCode::JumpIfTrue as u8
                    || forward_raw == OpCode::JumpIfFalse as u8
                    || (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8)
                        .contains(&forward_raw)
                    || forward_raw == OpCode::ForLoopI32 as u8
                    || forward_raw == OpCode::JumpIfI32Imm as u8
                    || forward_raw == OpCode::GetArrayJumpIfFalse as u8
                    || forward_raw == OpCode::GetArrayJumpIfTrue as u8
                    || forward_raw == OpCode::JumpIfLtF64 as u8
                    || forward_raw == OpCode::JumpIfLteF64 as u8
                    || forward_raw == OpCode::JumpIfLtI64 as u8
                    || forward_raw == OpCode::JumpIfLteI64 as u8
                {
                    let Some(target_pc) =
                        decode_branch_target(&forward_inst, forward_index, instruction_count)
                    else {
                        break;
                    };
                    if target_pc <= forward_index || target_pc >= limit {
                        break;
                    }
                    max_forward_target =
                        Some(max_forward_target.map_or(target_pc, |cur| cur.max(target_pc)));
                    continue;
                }

                // Prohibit calls, heap mutations, or escapes that could alias or release the source.
                if forward_raw == OpCode::CallDirect as u8
                    || forward_raw == OpCode::TailCallDirect as u8
                    || forward_raw == OpCode::CallVirtual as u8
                    || forward_raw == OpCode::CallInterface as u8
                    || forward_raw == OpCode::CallCallable as u8
                    || forward_raw == OpCode::CallNativeDirect as u8
                    || forward_raw == OpCode::CallSymbol as u8
                    || forward_raw == OpCode::CallExtern as u8
                    || forward_raw == OpCode::PackClosure as u8
                    || forward_raw == OpCode::SetField as u8
                    || forward_raw == OpCode::SetArray as u8
                    || forward_raw == OpCode::SetStatic as u8
                    || forward_raw == OpCode::SetStaticDirect as u8
                    || forward_raw == OpCode::SetStaticGlobal as u8
                    || forward_raw == OpCode::AllocTuple as u8
                    || forward_raw == OpCode::IncRef as u8
                    || forward_raw == OpCode::DecRef as u8
                {
                    break;
                }

                if instruction_write_dst(&forward_inst) == Some(target_register) {
                    break;
                }
            }

            if let Some(dec_index) = matching_dec_index {
                function.instructions[start_index] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                function.instructions[dec_index] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                elided_count += 2;
            }
        }
    }

    elided_count
}

fn collect_arc_branch_targets(function: &CompiledFunction) -> HashSet<usize> {
    let mut targets = HashSet::new();
    let inst_count = function.instructions.len();
    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        if let Some(target_pc) = decode_branch_target(inst, pc_idx, inst_count) {
            targets.insert(target_pc);
        }
    }
    for handler in &function.exception_handlers {
        targets.insert(handler.handler_pc as usize);
    }
    targets
}
