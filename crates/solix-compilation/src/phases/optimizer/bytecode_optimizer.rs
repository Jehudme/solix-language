//! Post-emission bytecode peephole and superinstruction optimizer phase.
//!
//! Applies redundant move elimination, superinstruction fusion (`AddI32Imm`, `SubI32Imm`,
//! `JumpIf*I32`), and exact relative branch and exception table PC recalculation.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::CompilationDiagnostic;
use crate::phases::phase::CompilationPhase;
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode, SourceMapEntry};
use std::collections::HashSet;

/// Post-emission bytecode optimizer implementing [`CompilationPhase`].
#[derive(Debug, Default, Clone, Copy)]
pub struct BytecodeOptimizerPhase;

impl BytecodeOptimizerPhase {
    /// Creates a new instance of the post-emission bytecode optimizer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for BytecodeOptimizerPhase {
    fn name(&self) -> &'static str {
        "Bytecode Peephole & Superinstruction Optimizer"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        if !context.options.bytecode_optimization {
            return Ok(());
        }

        let Some(emission_artifacts) = context.emission_artifacts.as_mut() else {
            return Ok(());
        };

        let class_descriptors = emission_artifacts.program.classes.clone();
        let function_descriptors = emission_artifacts.program.functions.clone();

        // Optimize each compiled function in the binary program
        for compiled_function in &mut emission_artifacts.program.functions {
            optimize_function(compiled_function, &class_descriptors, &function_descriptors);
        }

        // Keep functions_by_source synchronized with optimized functions
        for source_functions in &mut emission_artifacts.functions_by_source {
            for function in source_functions {
                if let Some(optimized) = emission_artifacts
                    .program
                    .functions
                    .iter()
                    .find(|func| func.symbol == function.symbol)
                {
                    *function = optimized.clone();
                }
            }
        }

        Ok(())
    }
}

/// Intermediate representation of an instruction during optimization passes.
#[derive(Clone)]
struct IntermediateInstruction {
    instruction: Instruction,
    original_pc: usize,
    second_original_pc: Option<usize>,
    jump_target_old_pc: Option<usize>,
}

/// Optimizes a single compiled function by running constant hoisting, peephole fusion,
/// move elimination, and PC relocation.
fn optimize_function(
    function: &mut CompiledFunction,
    classes: &[solix_common::bytecode::CompiledClass],
    all_functions: &[CompiledFunction],
) {
    super::escape_analysis::apply_scalar_replacement(function, classes, all_functions);
    super::escape_analysis::apply_arc_elision(function);

    let original_count = function.instructions.len();
    if original_count < 2 {
        return;
    }

    let critical_targets = collect_critical_targets(function);

    let mut intermediate_stream: Vec<IntermediateInstruction> = Vec::with_capacity(original_count);

    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        let raw_opcode = instruction.raw_opcode();
        let jump_target = if raw_opcode == OpCode::Jump as u8
            || raw_opcode == OpCode::JumpIfTrue as u8
            || raw_opcode == OpCode::JumpIfFalse as u8
        {
            let offset = instruction.jump_offset();
            let target_pc = (pc_index as i32) + offset;
            if target_pc >= 0 && (target_pc as usize) <= original_count {
                Some(target_pc as usize)
            } else {
                None
            }
        } else if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode) {
            let offset = instruction.src2() as i16 as i32;
            let target_pc = (pc_index as i32) + offset;
            if target_pc >= 0 && (target_pc as usize) <= original_count {
                Some(target_pc as usize)
            } else {
                None
            }
        } else {
            None
        };

        intermediate_stream.push(IntermediateInstruction {
            instruction: *instruction,
            original_pc: pc_index,
            second_original_pc: None,
            jump_target_old_pc: jump_target,
        });
    }

    // Step 1: Peephole fusion and redundant move elimination
    let mut fused_stream: Vec<IntermediateInstruction> =
        Vec::with_capacity(intermediate_stream.len());

    let mut cursor = 0;
    while cursor < intermediate_stream.len() {
        let current_entry = &intermediate_stream[cursor];
        let current_raw = current_entry.instruction.raw_opcode();

        if current_raw == OpCode::Nop as u8 {
            cursor += 1;
            continue;
        }

        if cursor + 1 < intermediate_stream.len() {
            let next_entry = &intermediate_stream[cursor + 1];
            let next_is_critical = critical_targets.contains(&next_entry.original_pc);

            let next_raw = next_entry.instruction.raw_opcode();

            // -------------------------------------------------------------
            // Pattern 1: Compare + Conditional Jump Fusion
            // LtI32/LteI32/GtI32/GteI32/EqI32/NeqI32 + JumpIfFalse / JumpIfTrue -> JumpIf*I32
            // -------------------------------------------------------------
            let is_compare = current_raw == OpCode::LtI32 as u8
                || current_raw == OpCode::LteI32 as u8
                || current_raw == OpCode::GtI32 as u8
                || current_raw == OpCode::GteI32 as u8
                || current_raw == OpCode::EqI32 as u8
                || current_raw == OpCode::NeqI32 as u8;

            let is_branch =
                next_raw == OpCode::JumpIfFalse as u8 || next_raw == OpCode::JumpIfTrue as u8;

            if !next_is_critical
                && is_compare
                && is_branch
                && current_entry.instruction.dst() == next_entry.instruction.cond_reg()
                && count_total_reads(&function.instructions, current_entry.instruction.dst()) == 1
            {
                let fused_opcode = match (current_raw, next_raw == OpCode::JumpIfFalse as u8) {
                    (r, true) if r == OpCode::LtI32 as u8 => OpCode::JumpIfGteI32,
                    (r, false) if r == OpCode::LtI32 as u8 => OpCode::JumpIfLtI32,
                    (r, true) if r == OpCode::LteI32 as u8 => OpCode::JumpIfGtI32,
                    (r, false) if r == OpCode::LteI32 as u8 => OpCode::JumpIfLteI32,
                    (r, true) if r == OpCode::GtI32 as u8 => OpCode::JumpIfLteI32,
                    (r, false) if r == OpCode::GtI32 as u8 => OpCode::JumpIfGtI32,
                    (r, true) if r == OpCode::GteI32 as u8 => OpCode::JumpIfLtI32,
                    (r, false) if r == OpCode::GteI32 as u8 => OpCode::JumpIfGteI32,
                    (r, true) if r == OpCode::EqI32 as u8 => OpCode::JumpIfNeqI32,
                    (r, false) if r == OpCode::EqI32 as u8 => OpCode::JumpIfEqI32,
                    (r, true) if r == OpCode::NeqI32 as u8 => OpCode::JumpIfEqI32,
                    _ => OpCode::JumpIfNeqI32,
                };

                let reg_a = current_entry.instruction.src1();
                let reg_b = current_entry.instruction.src2();

                fused_stream.push(IntermediateInstruction {
                    instruction: Instruction::format_a(fused_opcode, 0, reg_a, reg_b, 0),
                    original_pc: current_entry.original_pc,
                    second_original_pc: Some(next_entry.original_pc),
                    jump_target_old_pc: next_entry.jump_target_old_pc,
                });

                cursor += 2;
                continue;
            }

            // -------------------------------------------------------------
            // Pattern 2: LoadI32 + AddI32 / SubI32 Fusion
            // LoadI32 r_imm, imm + AddI32 dst, r_a, r_imm -> AddI32Imm dst, r_a, imm
            // -------------------------------------------------------------
            if !next_is_critical && current_raw == OpCode::LoadI32 as u8 {
                let imm_val = current_entry.instruction.imm32_signed();
                if (-32768..=32767).contains(&imm_val) {
                    let r_imm = current_entry.instruction.dst();
                    let total_reads = count_total_reads(&function.instructions, r_imm);

                    if total_reads == 1 && next_raw == OpCode::AddI32 as u8 {
                        let dst_reg = next_entry.instruction.dst();
                        let src1_reg = next_entry.instruction.src1();
                        let src2_reg = next_entry.instruction.src2();

                        let other_reg = if src2_reg == r_imm {
                            Some(src1_reg)
                        } else if src1_reg == r_imm {
                            Some(src2_reg)
                        } else {
                            None
                        };

                        if let Some(r_other) = other_reg {
                            fused_stream.push(IntermediateInstruction {
                                instruction: Instruction::format_a(
                                    OpCode::AddI32Imm,
                                    0,
                                    dst_reg,
                                    r_other,
                                    imm_val as i16 as u16,
                                ),
                                original_pc: current_entry.original_pc,
                                second_original_pc: Some(next_entry.original_pc),
                                jump_target_old_pc: None,
                            });

                            cursor += 2;
                            continue;
                        }
                    } else if total_reads == 1 && next_raw == OpCode::SubI32 as u8 {
                        let dst_reg = next_entry.instruction.dst();
                        let src1_reg = next_entry.instruction.src1();
                        let src2_reg = next_entry.instruction.src2();

                        // Only valid if subtracting immediate (r_a - imm)
                        if src2_reg == r_imm {
                            fused_stream.push(IntermediateInstruction {
                                instruction: Instruction::format_a(
                                    OpCode::SubI32Imm,
                                    0,
                                    dst_reg,
                                    src1_reg,
                                    imm_val as i16 as u16,
                                ),
                                original_pc: current_entry.original_pc,
                                second_original_pc: Some(next_entry.original_pc),
                                jump_target_old_pc: None,
                            });

                            cursor += 2;
                            continue;
                        }
                    }
                }
            }

            // -------------------------------------------------------------
            // Pattern 3: Peephole Mov Elimination
            // Op dst=r_tmp + Mov r_local, r_tmp -> Op dst=r_local
            // -------------------------------------------------------------
            if !next_is_critical
                && current_raw != OpCode::Mov as u8
                && next_raw == OpCode::Mov as u8
            {
                let r_tmp = next_entry.instruction.src1();
                let r_dest = next_entry.instruction.dst();

                if current_entry.instruction.dst() == r_tmp
                    && r_tmp != r_dest
                    && count_total_reads(&function.instructions, r_tmp) == 1
                    && can_change_instruction_destination(&current_entry.instruction, r_dest)
                {
                    let updated_instruction =
                        rewrite_instruction_destination(&current_entry.instruction, r_dest);

                    fused_stream.push(IntermediateInstruction {
                        instruction: updated_instruction,
                        original_pc: current_entry.original_pc,
                        second_original_pc: Some(next_entry.original_pc),
                        jump_target_old_pc: current_entry.jump_target_old_pc,
                    });

                    cursor += 2;
                    continue;
                }
            }
        }

        // Default: keep current instruction
        fused_stream.push(IntermediateInstruction {
            instruction: current_entry.instruction,
            original_pc: current_entry.original_pc,
            second_original_pc: None,
            jump_target_old_pc: current_entry.jump_target_old_pc,
        });

        cursor += 1;
    }

    // Step 2: Compute relocated instruction PCs and branch offsets
    let sentinel_pc = u32::MAX;
    let mut old_pc_to_new_pc = vec![sentinel_pc; original_count + 1];
    for (new_pc, entry) in fused_stream.iter().enumerate() {
        old_pc_to_new_pc[entry.original_pc] = new_pc as u32;
        if let Some(second_pc) = entry.second_original_pc {
            old_pc_to_new_pc[second_pc] = new_pc as u32;
        }
    }
    old_pc_to_new_pc[original_count] = fused_stream.len() as u32;
    for reverse_index in (0..original_count).rev() {
        if old_pc_to_new_pc[reverse_index] == sentinel_pc {
            old_pc_to_new_pc[reverse_index] = old_pc_to_new_pc[reverse_index + 1];
        }
    }

    // Step 3: Relocate jump targets
    let mut final_instructions = Vec::with_capacity(fused_stream.len());
    for (new_pc, intermediate) in fused_stream.iter().enumerate() {
        let mut final_inst = intermediate.instruction;
        if let Some(target_old_pc) = intermediate.jump_target_old_pc {
            let target_new_pc = if target_old_pc < old_pc_to_new_pc.len() {
                old_pc_to_new_pc[target_old_pc]
            } else {
                fused_stream.len() as u32
            };

            let new_offset = (target_new_pc as i32) - (new_pc as i32);
            let raw_opcode = final_inst.raw_opcode();

            if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode) {
                if let Ok(opcode) = final_inst.opcode(new_pc) {
                    final_inst = Instruction::format_a(
                        opcode,
                        0,
                        final_inst.dst(),
                        final_inst.src1(),
                        new_offset as i16 as u16,
                    );
                }
            } else if (raw_opcode == OpCode::Jump as u8
                || raw_opcode == OpCode::JumpIfTrue as u8
                || raw_opcode == OpCode::JumpIfFalse as u8)
                && let Ok(opcode) = final_inst.opcode(new_pc)
            {
                final_inst = Instruction::format_c(
                    opcode,
                    final_inst.flags(),
                    final_inst.cond_reg(),
                    new_offset,
                );
            }
        }

        final_instructions.push(final_inst);
    }

    // Apply relocated instructions and tables to function
    apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
}

/// Applies rebuilt instructions and translates exception handlers and source maps.
fn apply_pc_relocation(
    function: &mut CompiledFunction,
    new_instructions: Vec<Instruction>,
    old_pc_to_new_pc: &[u32],
) {
    function.instructions = new_instructions;

    // Translate ExceptionHandlerEntry PC ranges
    for handler in &mut function.exception_handlers {
        let start_idx = handler.try_start_pc as usize;
        let end_idx = handler.try_end_pc as usize;
        let handler_idx = handler.handler_pc as usize;

        if start_idx < old_pc_to_new_pc.len() {
            handler.try_start_pc = old_pc_to_new_pc[start_idx];
        }
        if end_idx < old_pc_to_new_pc.len() {
            handler.try_end_pc = old_pc_to_new_pc[end_idx];
        }
        if handler_idx < old_pc_to_new_pc.len() {
            handler.handler_pc = old_pc_to_new_pc[handler_idx];
        }
        if let Some(finally_idx) = handler.finally_pc
            && (finally_idx as usize) < old_pc_to_new_pc.len()
        {
            handler.finally_pc = Some(old_pc_to_new_pc[finally_idx as usize]);
        }
    }

    // Translate SourceMapEntry PCs
    let mut updated_source_map = Vec::with_capacity(function.source_map.len());
    let mut last_mapped_pc: Option<u32> = None;

    for entry in &function.source_map {
        let old_pc = entry.instruction_pc as usize;
        if old_pc < old_pc_to_new_pc.len() {
            let new_pc = old_pc_to_new_pc[old_pc];
            if last_mapped_pc != Some(new_pc) {
                updated_source_map.push(SourceMapEntry {
                    instruction_pc: new_pc,
                    span: entry.span,
                    line: entry.line,
                    column: entry.column,
                });
                last_mapped_pc = Some(new_pc);
            }
        }
    }

    function.source_map = updated_source_map;
}

/// Collects all critical branch targets and exception handler entry targets.
fn collect_critical_targets(function: &CompiledFunction) -> HashSet<usize> {
    let mut critical_targets = HashSet::new();
    let instruction_count = function.instructions.len();

    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        let raw_opcode = instruction.raw_opcode();
        if raw_opcode == OpCode::Jump as u8
            || raw_opcode == OpCode::JumpIfTrue as u8
            || raw_opcode == OpCode::JumpIfFalse as u8
        {
            let offset = instruction.jump_offset();
            let target_pc = (pc_index as i32) + offset;
            if target_pc >= 0 && (target_pc as usize) <= instruction_count {
                critical_targets.insert(target_pc as usize);
            }
        } else if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode) {
            let offset = instruction.src2() as i16 as i32;
            let target_pc = (pc_index as i32) + offset;
            if target_pc >= 0 && (target_pc as usize) <= instruction_count {
                critical_targets.insert(target_pc as usize);
            }
        }
    }

    for handler in &function.exception_handlers {
        critical_targets.insert(handler.try_start_pc as usize);
        critical_targets.insert(handler.try_end_pc as usize);
        critical_targets.insert(handler.handler_pc as usize);
        if let Some(finally_pc) = handler.finally_pc {
            critical_targets.insert(finally_pc as usize);
        }
    }

    critical_targets
}

/// Checks whether an instruction format allows rewriting its destination register.
fn can_change_instruction_destination(instruction: &Instruction, new_dst: u16) -> bool {
    let raw = instruction.raw_opcode();
    if instruction_reads_register(instruction, new_dst) {
        return false;
    }

    // Format A instructions that write to dst
    if (OpCode::AddI32 as u8..=OpCode::NeqString as u8).contains(&raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::GetArray as u8
        || raw == OpCode::AllocObject as u8
        || raw == OpCode::AllocArray as u8
        || raw == OpCode::AllocTuple as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::TupleInstanceof as u8
        || raw == OpCode::UnpackCaptures as u8
        || raw == OpCode::Not as u8
        || raw == OpCode::Bnot as u8
        || raw == OpCode::NegI32 as u8
        || raw == OpCode::NegI64 as u8
        || raw == OpCode::NegF64 as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::Instanceof as u8
        || raw == OpCode::ArrayLength as u8
    {
        return true;
    }

    // Format B instructions that write to dst
    if raw == OpCode::LoadI32 as u8
        || raw == OpCode::LoadI64 as u8
        || raw == OpCode::LoadF64 as u8
        || raw == OpCode::LoadBool as u8
        || raw == OpCode::LoadNull as u8
        || raw == OpCode::LoadConst as u8
        || raw == OpCode::GetStatic as u8
        || raw == OpCode::GetStaticDirect as u8
        || raw == OpCode::GetStaticGlobal as u8
        || raw == OpCode::AllocObjectExtern as u8
    {
        return true;
    }

    // Format Call instructions that write to call_dst
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
    {
        return true;
    }

    false
}

/// Rewrites an instruction's destination register to `new_dst`.
fn rewrite_instruction_destination(instruction: &Instruction, new_dst: u16) -> Instruction {
    let raw = instruction.raw_opcode();
    let Ok(opcode) = instruction.opcode(0) else {
        return *instruction;
    };

    // Format Call
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
    {
        return Instruction::format_call(
            opcode,
            new_dst,
            instruction.call_target(),
            instruction.arg_base(),
            instruction.arg_count(),
        );
    }

    // Format B
    if raw == OpCode::LoadI32 as u8
        || raw == OpCode::LoadI64 as u8
        || raw == OpCode::LoadF64 as u8
        || raw == OpCode::LoadBool as u8
        || raw == OpCode::LoadNull as u8
        || raw == OpCode::LoadConst as u8
        || raw == OpCode::GetStatic as u8
        || raw == OpCode::GetStaticDirect as u8
        || raw == OpCode::GetStaticGlobal as u8
        || raw == OpCode::AllocObjectExtern as u8
    {
        return Instruction::format_b(opcode, instruction.flags(), new_dst, instruction.imm32());
    }

    // Format A
    Instruction::format_a(
        opcode,
        instruction.flags(),
        new_dst,
        instruction.src1(),
        instruction.src2(),
    )
}

/// Checks whether an instruction reads `target_reg` as an input operand.
fn instruction_reads_register(instruction: &Instruction, target_reg: u16) -> bool {
    let raw = instruction.raw_opcode();

    // Format A: compare-and-jump instructions read dst and src1
    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw) {
        return instruction.dst() == target_reg || instruction.src1() == target_reg;
    }

    // Format A: superinstructions AddI32Imm and SubI32Imm read src1
    if raw == OpCode::AddI32Imm as u8 || raw == OpCode::SubI32Imm as u8 {
        return instruction.src1() == target_reg;
    }

    // Format A: Mov, Unary ops, GetField, ArrayLength, UnpackCaptures, CastCheck, Instanceof read src1
    if raw == OpCode::Mov as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::Not as u8
        || raw == OpCode::Bnot as u8
        || raw == OpCode::NegI32 as u8
        || raw == OpCode::NegI64 as u8
        || raw == OpCode::NegF64 as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::ArrayLength as u8
        || raw == OpCode::AllocArray as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::TupleInstanceof as u8
        || raw == OpCode::UnpackCaptures as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::Instanceof as u8
    {
        return instruction.src1() == target_reg;
    }

    // Format A: DecRef, IncRef, Exit, Throw, Return, ReturnValue read dst
    if raw == OpCode::DecRef as u8
        || raw == OpCode::IncRef as u8
        || raw == OpCode::Exit as u8
        || raw == OpCode::Throw as u8
        || raw == OpCode::Return as u8
        || raw == OpCode::ReturnValue as u8
    {
        return instruction.dst() == target_reg;
    }

    // Format A: SetField reads dst (value) and src1 (object)
    if raw == OpCode::SetField as u8 {
        return instruction.dst() == target_reg || instruction.src1() == target_reg;
    }

    // Format A: GetArray reads src1 (array) and src2 (index)
    if raw == OpCode::GetArray as u8 {
        return instruction.src1() == target_reg || instruction.src2() == target_reg;
    }

    // Format A: SetArray reads dst (array), src1 (index), and src2 (value)
    if raw == OpCode::SetArray as u8 {
        return instruction.dst() == target_reg
            || instruction.src1() == target_reg
            || instruction.src2() == target_reg;
    }

    // Format A: Binary operations read src1 and src2
    if (OpCode::AddI32 as u8..=OpCode::NeqString as u8).contains(&raw) {
        return instruction.src1() == target_reg || instruction.src2() == target_reg;
    }

    // Format B: SetStatic reads dst
    if raw == OpCode::SetStatic as u8
        || raw == OpCode::SetStaticDirect as u8
        || raw == OpCode::SetStaticGlobal as u8
    {
        return instruction.dst() == target_reg;
    }

    // Format C: JumpIfTrue and JumpIfFalse read cond_reg
    if raw == OpCode::JumpIfTrue as u8 || raw == OpCode::JumpIfFalse as u8 {
        return instruction.cond_reg() == target_reg;
    }

    // Format Call: windowed argument registers
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
    {
        let base = instruction.arg_base();
        let count = instruction.arg_count() as u16;
        return target_reg >= base && target_reg < base + count;
    }

    false
}

/// Counts total reads of `target_reg` across the entire function.
fn count_total_reads(instructions: &[Instruction], target_reg: u16) -> usize {
    let mut count = 0;
    for instruction in instructions {
        if instruction_reads_register(instruction, target_reg) {
            count += 1;
        }
    }
    count
}
