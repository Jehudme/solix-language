//! Post-emission bytecode peephole and superinstruction optimizer phase.
//!
//! Applies redundant move elimination, superinstruction fusion (`AddI32Imm`, `SubI32Imm`,
//! `JumpIf*I32`), and exact relative branch and exception table PC recalculation.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::CompilationDiagnostic;
use crate::phases::phase::CompilationPhase;
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode, SourceMapEntry};
use std::collections::{HashMap, HashSet};

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

        // Optimize each compiled function individually first so leaf constructors shrink to minimal bodies.
        for compiled_function in &mut emission_artifacts.program.functions {
            optimize_function(compiled_function, &class_descriptors, &function_descriptors);
        }

        // Inline small leaf constructors across callers (multiple rounds handle inheritance chains),
        // then re-converge caller passes.
        for _inline_round in 0..3 {
            let optimized_snapshot = emission_artifacts.program.functions.clone();
            let mut round_changed = false;
            for (func_idx, compiled_function) in
                emission_artifacts.program.functions.iter_mut().enumerate()
            {
                if inline_leaf_constructors(compiled_function, func_idx, &optimized_snapshot) {
                    round_changed = true;
                    super::escape_analysis::apply_scalar_replacement(
                        compiled_function,
                        &class_descriptors,
                        &optimized_snapshot,
                    );
                    super::escape_analysis::apply_arc_elision(compiled_function);
                    elide_redundant_arc_ops(compiled_function);
                    optimize_peephole_passes(compiled_function);
                    elide_borrowed_local_arc_ops(compiled_function);
                    elide_fresh_null_set_fields(compiled_function);
                    optimize_peephole_passes(compiled_function);
                    hoist_loop_invariant_constants(compiled_function);
                }
            }
            if !round_changed {
                break;
            }
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
    covered_original_pcs: Vec<usize>,
    jump_target_old_pc: Option<usize>,
}

/// Optimizes a single compiled function by running scalar replacement, ARC elision,
/// redundant ARC/null field elimination, convergent peephole fusion with liveness analysis,
/// and loop-invariant constant hoisting.
fn optimize_function(
    function: &mut CompiledFunction,
    classes: &[solix_common::bytecode::CompiledClass],
    all_functions: &[CompiledFunction],
) {
    super::escape_analysis::apply_scalar_replacement(function, classes, all_functions);
    super::escape_analysis::apply_arc_elision(function);
    elide_redundant_arc_ops(function);
    optimize_peephole_passes(function);
    elide_borrowed_local_arc_ops(function);
    elide_fresh_null_set_fields(function);

    optimize_peephole_passes(function);
    hoist_loop_invariant_constants(function);
}

/// Returns `true` if `reg` holds a guaranteed immortal/null/FnPtr value throughout an immutable local array
/// or within a straight-line basic block (`LoadConst`, `LoadNull`, zero-capture `PackClosure`).
fn is_immortal_or_null_producer(inst: Instruction) -> bool {
    let raw = inst.raw_opcode();
    raw == OpCode::LoadConst as u8
        || raw == OpCode::LoadNull as u8
        || (raw == OpCode::PackClosure as u8 && inst.arg_count() == 0)
}

/// Finds local `AllocArray` registers populated exclusively by immortal/null values that never escape.
fn find_immortal_constant_arrays(
    function: &CompiledFunction,
    critical_targets: &HashSet<usize>,
) -> HashSet<u16> {
    let mut alloc_counts: HashMap<u16, usize> = HashMap::new();
    for inst in &function.instructions {
        if let Some(written) = instruction_write_dst(inst) {
            *alloc_counts.entry(written).or_insert(0) += 1;
        }
    }

    let mut candidates: HashSet<u16> = HashSet::new();
    for inst in &function.instructions {
        if inst.raw_opcode() == OpCode::AllocArray as u8 {
            let arr_reg = inst.dst();
            if alloc_counts.get(&arr_reg).copied() == Some(1) {
                candidates.insert(arr_reg);
            }
        }
    }

    if candidates.is_empty() {
        return candidates;
    }

    let mut disqualified: HashSet<u16> = HashSet::new();
    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        let raw = inst.raw_opcode();
        if raw == OpCode::SetArray as u8 {
            let arr_reg = inst.dst();
            if candidates.contains(&arr_reg) {
                let val_reg = inst.src2();
                let mut val_is_immortal = false;
                let mut back_pc = pc_idx;
                while back_pc > 0 {
                    if critical_targets.contains(&back_pc) {
                        break;
                    }
                    back_pc -= 1;
                    let prev = function.instructions[back_pc];
                    let prev_raw = prev.raw_opcode();
                    if is_control_flow_terminator(prev_raw)
                        || is_conditional_branch(prev_raw)
                        || prev_raw == OpCode::Jump as u8
                        || prev_raw == OpCode::Switch as u8
                    {
                        break;
                    }
                    if instruction_write_dst(&prev) == Some(val_reg) {
                        val_is_immortal = is_immortal_or_null_producer(prev);
                        break;
                    }
                }
                if !val_is_immortal {
                    disqualified.insert(arr_reg);
                }
            }
            let stored_val = inst.src2();
            if candidates.contains(&stored_val) {
                disqualified.insert(stored_val);
            }
            continue;
        }

        if raw == OpCode::GetArray as u8 || raw == OpCode::ArrayLength as u8 {
            if raw == OpCode::GetArray as u8 && candidates.contains(&inst.src2()) {
                disqualified.insert(inst.src2());
            }
            continue;
        }

        if raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8 {
            continue;
        }

        for candidate_reg in &candidates {
            if instruction_reads_register(inst, *candidate_reg) {
                disqualified.insert(*candidate_reg);
            }
        }
    }

    for bad_reg in disqualified {
        candidates.remove(&bad_reg);
    }
    candidates
}

/// Returns `true` if `inst` is guaranteed never to write a dynamically allocated mutable `Value::Ref`
/// (`Load*`, `PackClosure` with 0 captures, `GetArray` from an immortal constant array, or scalar ALU/compare opcodes).
fn is_guaranteed_non_heap_write(inst: Instruction, immortal_arrays: &HashSet<u16>) -> bool {
    let Some(opc) = OpCode::from_raw_u8(inst.raw_opcode()) else {
        return false;
    };
    matches!(
        opc,
        OpCode::LoadI32
            | OpCode::LoadI64
            | OpCode::LoadF64
            | OpCode::LoadBool
            | OpCode::LoadNull
            | OpCode::LoadConst
            | OpCode::AddI32
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
            | OpCode::NegI32
            | OpCode::NegI64
            | OpCode::NegF64
            | OpCode::Band
            | OpCode::Bor
            | OpCode::Bxor
            | OpCode::Bnot
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
            | OpCode::GteF64
            | OpCode::Not
            | OpCode::ArrayLength
            | OpCode::Instanceof
            | OpCode::ConvIToF
            | OpCode::ConvFToI
            | OpCode::TupleInstanceof
            | OpCode::AddI32Imm
            | OpCode::SubI32Imm
            | OpCode::MulI32Imm
            | OpCode::ModI32Imm
    ) || (opc == OpCode::PackClosure && inst.arg_count() == 0)
        || (opc == OpCode::GetArray && immortal_arrays.contains(&inst.src1()))
}

/// Eliminates redundant `IncRef` and `DecRef` on `LoadConst`, `LoadNull`, zero-capture `PackClosure`,
/// and elements loaded from immutable local constant arrays within basic blocks or across the function.
fn elide_redundant_arc_ops(function: &mut CompiledFunction) {
    let inst_count = function.instructions.len();
    if inst_count < 2 {
        return;
    }

    let critical_targets = collect_critical_targets(function);
    let immortal_arrays = find_immortal_constant_arrays(function, &critical_targets);

    // Find registers whose every definition across the entire function yields a non-heap-ref/immortal value.
    let mut always_immortal_regs: HashSet<u16> = HashSet::new();
    let mut has_any_write = vec![false; function.num_registers as usize];
    let mut has_non_immortal_direct_write = vec![false; function.num_registers as usize];
    let mut mov_sources: HashMap<u16, Vec<u16>> = HashMap::new();
    let mut scan_block_non_heap: HashSet<u16> = HashSet::new();
    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        if critical_targets.contains(&pc_idx) {
            scan_block_non_heap.clear();
        }
        let raw = inst.raw_opcode();
        if let Some(dst_reg) = instruction_write_dst(inst) {
            let idx = dst_reg as usize;
            if idx < has_any_write.len() {
                has_any_write[idx] = true;
                if is_guaranteed_non_heap_write(*inst, &immortal_arrays) {
                    scan_block_non_heap.insert(dst_reg);
                } else if raw == OpCode::Mov as u8 {
                    let src_reg = inst.src1();
                    if scan_block_non_heap.contains(&src_reg) {
                        scan_block_non_heap.insert(dst_reg);
                    } else {
                        scan_block_non_heap.remove(&dst_reg);
                        mov_sources.entry(dst_reg).or_default().push(src_reg);
                    }
                } else {
                    scan_block_non_heap.remove(&dst_reg);
                    has_non_immortal_direct_write[idx] = true;
                }
            }
        }
        if is_control_flow_terminator(raw)
            || is_conditional_branch(raw)
            || raw == OpCode::Jump as u8
            || raw == OpCode::Switch as u8
        {
            scan_block_non_heap.clear();
        }
    }
    for reg in function.param_count..function.num_registers {
        let idx = reg as usize;
        if has_any_write[idx]
            && !has_non_immortal_direct_write[idx]
            && !mov_sources.contains_key(&reg)
        {
            always_immortal_regs.insert(reg);
        }
    }
    loop {
        let mut added = false;
        for reg in function.param_count..function.num_registers {
            let idx = reg as usize;
            if always_immortal_regs.contains(&reg)
                || !has_any_write[idx]
                || has_non_immortal_direct_write[idx]
            {
                continue;
            }
            if let Some(srcs) = mov_sources.get(&reg)
                && srcs
                    .iter()
                    .all(|src_reg| always_immortal_regs.contains(src_reg))
            {
                always_immortal_regs.insert(reg);
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    if !always_immortal_regs.is_empty() {
        function
            .reference_registers
            .retain(|reg| !always_immortal_regs.contains(reg));
    }

    let mut block_immortals: HashSet<u16> = HashSet::new();
    for pc_idx in 0..inst_count {
        if critical_targets.contains(&pc_idx) {
            block_immortals.clear();
        }

        let inst = function.instructions[pc_idx];
        let raw = inst.raw_opcode();

        if (raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8)
            && (block_immortals.contains(&inst.dst()) || always_immortal_regs.contains(&inst.dst()))
        {
            function.instructions[pc_idx] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            continue;
        }

        if is_immortal_or_null_producer(inst) {
            if let Some(dst_reg) = instruction_write_dst(&inst) {
                block_immortals.insert(dst_reg);
            }
        } else if raw == OpCode::GetArray as u8 && immortal_arrays.contains(&inst.src1()) {
            block_immortals.insert(inst.dst());
        } else if raw == OpCode::Mov as u8 {
            let mov_dst = inst.dst();
            if block_immortals.contains(&inst.src1()) {
                block_immortals.insert(mov_dst);
            } else {
                block_immortals.remove(&mov_dst);
            }
        } else if let Some(written) = instruction_write_dst(&inst) {
            block_immortals.remove(&written);
        }

        if is_control_flow_terminator(raw)
            || is_conditional_branch(raw)
            || raw == OpCode::Jump as u8
            || raw == OpCode::Switch as u8
        {
            block_immortals.clear();
        }
    }
}

/// Elides ARC operations on non-escaping borrowed local alias registers whose owning source allocations
/// or immutable non-escaping local arrays remain live until the function epilogue cleanup block after all uses of the alias.
fn elide_borrowed_local_arc_ops(function: &mut CompiledFunction) {
    if !function.exception_handlers.is_empty() || function.instructions.len() < 4 {
        return;
    }
    let inst_count = function.instructions.len();
    let critical_targets = collect_critical_targets(function);
    let mut def_counts: HashMap<u16, usize> = HashMap::new();
    let mut def_opc: HashMap<u16, u8> = HashMap::new();
    let mut decref_pcs: HashMap<u16, Vec<usize>> = HashMap::new();
    let mut last_read_or_arc_pc: HashMap<u16, usize> = HashMap::new();

    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        let raw = inst.raw_opcode();
        if let Some(written) = instruction_write_dst(inst) {
            *def_counts.entry(written).or_insert(0) += 1;
            def_opc.insert(written, raw);
        }
        if raw == OpCode::DecRef as u8 {
            decref_pcs.entry(inst.dst()).or_default().push(pc_idx);
        }
        for reg in 0..function.num_registers {
            if instruction_reads_register(inst, reg) {
                last_read_or_arc_pc.insert(reg, pc_idx);
            }
        }
    }

    // Identify single-definition local owners (AllocObject / AllocArray) whose only DecRefs sit
    // in the straight-line epilogue cleanup right before Return/ReturnValue.
    let mut stable_owners: HashMap<u16, usize> = HashMap::new();
    for (&owner_reg, &count) in &def_counts {
        if count != 1 {
            continue;
        }
        let opc = def_opc.get(&owner_reg).copied().unwrap_or(0);
        if opc != OpCode::AllocObject as u8 && opc != OpCode::AllocArray as u8 {
            continue;
        }
        let Some(owner_decs) = decref_pcs.get(&owner_reg) else {
            continue;
        };
        if owner_decs.is_empty() {
            continue;
        }
        let all_in_epilogues = owner_decs.iter().all(|&dec_pc| {
            let mut epilogue_ok = false;
            for scan_pc in (dec_pc + 1)..inst_count {
                let scan_raw = function.instructions[scan_pc].raw_opcode();
                if scan_raw == OpCode::Return as u8 || scan_raw == OpCode::ReturnValue as u8 {
                    epilogue_ok = true;
                    break;
                }
                if scan_raw != OpCode::DecRef as u8 && scan_raw != OpCode::Nop as u8 {
                    break;
                }
            }
            epilogue_ok
        });
        if all_in_epilogues && let Some(&earliest_dec_pc) = owner_decs.iter().min() {
            stable_owners.insert(owner_reg, earliest_dec_pc);
        }
    }

    if stable_owners.is_empty() {
        return;
    }

    // Identify immutable post-initialization local AllocArray owners whose elements cannot be
    // overwritten or freed before epilogue DecRef (all SetArrays precede all GetArrays, non-escaping).
    let mut immutable_local_arrays: HashSet<u16> = HashSet::new();
    for &owner_reg in stable_owners.keys() {
        if def_opc.get(&owner_reg).copied() != Some(OpCode::AllocArray as u8) {
            continue;
        }
        let mut max_set_pc: Option<usize> = None;
        let mut min_get_pc: Option<usize> = None;
        let mut only_safe_array_uses = true;
        for (pc_idx, inst) in function.instructions.iter().enumerate() {
            let raw = inst.raw_opcode();
            if raw == OpCode::SetArray as u8 && inst.dst() == owner_reg {
                max_set_pc = Some(max_set_pc.map_or(pc_idx, |cur| cur.max(pc_idx)));
                continue;
            }
            if raw == OpCode::GetArray as u8 && inst.src1() == owner_reg {
                min_get_pc = Some(min_get_pc.map_or(pc_idx, |cur| cur.min(pc_idx)));
                continue;
            }
            if (raw == OpCode::ArrayLength as u8 && inst.src1() == owner_reg)
                || (raw == OpCode::DecRef as u8 && inst.dst() == owner_reg)
            {
                continue;
            }
            if instruction_reads_register(inst, owner_reg) {
                only_safe_array_uses = false;
                break;
            }
        }
        if !only_safe_array_uses {
            continue;
        }
        let phase_separated = match (max_set_pc, min_get_pc) {
            (Some(last_set), Some(first_get)) => {
                last_set < first_get
                    && !(function
                        .instructions
                        .iter()
                        .enumerate()
                        .any(|(pc_idx, inst)| {
                            decode_jump_target(*inst, pc_idx, inst_count)
                                .is_some_and(|target| target <= last_set && pc_idx >= first_get)
                        }))
            }
            (None, Some(_)) => true,
            _ => false,
        };
        if phase_separated {
            immutable_local_arrays.insert(owner_reg);
        }
    }

    // Find candidate alias registers written solely via Mov(alias, stable_owner) or
    // GetArray(alias, immutable_local_array, _) where owning allocations stay alive until epilogue.
    let mut borrowed_aliases: HashSet<u16> = HashSet::new();
    for candidate_reg in 0..function.num_registers {
        if (candidate_reg as usize) < (function.param_count as usize)
            || stable_owners.contains_key(&candidate_reg)
        {
            continue;
        }
        let mut writes_count = 0usize;
        let mut writes_ok = true;
        let mut escapes = false;
        let mut used_owners: Vec<u16> = Vec::new();
        let mut max_non_epilogue_use: usize = 0;

        for (pc_idx, inst) in function.instructions.iter().enumerate() {
            let raw = inst.raw_opcode();
            if instruction_write_dst(inst) == Some(candidate_reg) {
                writes_count += 1;
                if (raw == OpCode::Mov as u8 && stable_owners.contains_key(&inst.src1()))
                    || (raw == OpCode::GetArray as u8
                        && immutable_local_arrays.contains(&inst.src1()))
                {
                    used_owners.push(inst.src1());
                } else {
                    writes_ok = false;
                    break;
                }
            }
            if raw == OpCode::SetField as u8 && inst.dst() == candidate_reg
                || raw == OpCode::SetArray as u8 && inst.src2() == candidate_reg
                || (raw == OpCode::SetStatic as u8
                    || raw == OpCode::SetStaticDirect as u8
                    || raw == OpCode::SetStaticGlobal as u8)
                    && inst.dst() == candidate_reg
                || raw == OpCode::ReturnValue as u8 && inst.dst() == candidate_reg
                || raw == OpCode::Throw as u8 && inst.dst() == candidate_reg
                || raw == OpCode::PackClosure as u8
                    && candidate_reg >= inst.arg_base()
                    && candidate_reg < inst.arg_base() + inst.arg_count() as u16
                || raw == OpCode::AllocTuple as u8
                    && candidate_reg >= inst.src1()
                    && candidate_reg < inst.src1() + inst.src2()
            {
                escapes = true;
                break;
            }
            if instruction_reads_register(inst, candidate_reg) {
                let is_epilogue_decref = raw == OpCode::DecRef as u8
                    && function.instructions[pc_idx + 1..].iter().any(|trailing| {
                        trailing.raw_opcode() == OpCode::Return as u8
                            || trailing.raw_opcode() == OpCode::ReturnValue as u8
                    })
                    && function.instructions[pc_idx + 1..]
                        .iter()
                        .take_while(|trailing| {
                            trailing.raw_opcode() != OpCode::Return as u8
                                && trailing.raw_opcode() != OpCode::ReturnValue as u8
                        })
                        .all(|trailing| {
                            trailing.raw_opcode() == OpCode::DecRef as u8
                                || trailing.raw_opcode() == OpCode::Nop as u8
                        });
                if !is_epilogue_decref {
                    max_non_epilogue_use = max_non_epilogue_use.max(pc_idx);
                }
            }
        }

        if writes_ok
            && writes_count > 0
            && !escapes
            && used_owners
                .iter()
                .all(|owner| stable_owners[owner] > max_non_epilogue_use)
        {
            borrowed_aliases.insert(candidate_reg);
        }
    }

    if !borrowed_aliases.is_empty() {
        // Elide IncRef/DecRef on borrowed aliases and matched IncRef(owner) preceding Mov(alias, owner).
        for pc_idx in 0..inst_count {
            let inst = function.instructions[pc_idx];
            let raw = inst.raw_opcode();
            if (raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8)
                && borrowed_aliases.contains(&inst.dst())
            {
                function.instructions[pc_idx] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                continue;
            }
            if raw == OpCode::Mov as u8
                && borrowed_aliases.contains(&inst.dst())
                && stable_owners.contains_key(&inst.src1())
            {
                let owner_reg = inst.src1();
                // Look back up to 2 instructions in same basic block for `IncRef owner_reg`.
                let lookback_start = pc_idx.saturating_sub(2);
                for back_pc in (lookback_start..pc_idx).rev() {
                    if critical_targets.contains(&(back_pc + 1)) {
                        break;
                    }
                    let back_inst = function.instructions[back_pc];
                    if back_inst.raw_opcode() == OpCode::IncRef as u8
                        && back_inst.dst() == owner_reg
                    {
                        function.instructions[back_pc] =
                            Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                        break;
                    }
                }
            }
        }
    }

    // Also elide local `(GetArray reg, immut_arr -> IncRef reg ... DecRef reg)` windows
    // (including across closed forward branch diamonds and non-escaping calls)
    // when `reg` is reused elsewhere in the function or calls occur inside the window.
    if !immutable_local_arrays.is_empty() {
        let mut jump_bounds_to_target: HashMap<usize, (usize, usize)> = HashMap::new();
        for (src_pc, inst) in function.instructions.iter().enumerate() {
            if let Some(target_pc) = decode_jump_target(*inst, src_pc, inst_count) {
                jump_bounds_to_target
                    .entry(target_pc)
                    .and_modify(|bounds| {
                        bounds.0 = bounds.0.min(src_pc);
                        bounds.1 = bounds.1.max(src_pc);
                    })
                    .or_insert((src_pc, src_pc));
            }
        }
        for inc_pc in 1..inst_count {
            let inc_inst = function.instructions[inc_pc];
            if inc_inst.raw_opcode() != OpCode::IncRef as u8 {
                continue;
            }
            let target_reg = inc_inst.dst();
            let mut loaded_from_arr: Option<u16> = None;
            let mut scan_back = inc_pc;
            while scan_back > 0 && inc_pc - scan_back <= 4 {
                if critical_targets.contains(&scan_back) {
                    break;
                }
                scan_back -= 1;
                let back_inst = function.instructions[scan_back];
                let back_raw = back_inst.raw_opcode();
                if back_raw == OpCode::GetArray as u8
                    && back_inst.dst() == target_reg
                    && immutable_local_arrays.contains(&back_inst.src1())
                {
                    loaded_from_arr = Some(back_inst.src1());
                    break;
                }
                if instruction_write_dst(&back_inst) == Some(target_reg)
                    || is_control_flow_terminator(back_raw)
                    || is_conditional_branch(back_raw)
                    || back_raw == OpCode::Jump as u8
                    || back_raw == OpCode::Switch as u8
                {
                    break;
                }
            }
            let Some(arr_reg) = loaded_from_arr else {
                continue;
            };
            let epilogue_dec_pc = stable_owners[&arr_reg];
            let search_end = epilogue_dec_pc.min(inst_count).min(inc_pc + 48);
            let mut matched_dec_pc: Option<usize> = None;
            let mut max_forward_target: Option<usize> = None;
            for fwd_pc in (inc_pc + 1)..search_end {
                let fwd_inst = function.instructions[fwd_pc];
                let fwd_raw = fwd_inst.raw_opcode();
                if let Some(&(min_src, max_src)) = jump_bounds_to_target.get(&fwd_pc)
                    && (min_src <= inc_pc || max_src >= fwd_pc)
                {
                    break;
                }
                if fwd_raw == OpCode::DecRef as u8 && fwd_inst.dst() == target_reg {
                    if max_forward_target.is_none_or(|max_t| max_t <= fwd_pc) {
                        matched_dec_pc = Some(fwd_pc);
                    }
                    break;
                }
                if fwd_raw == OpCode::Jump as u8 || is_conditional_branch(fwd_raw) {
                    let Some(target_pc) = decode_jump_target(fwd_inst, fwd_pc, inst_count) else {
                        break;
                    };
                    if target_pc <= fwd_pc || target_pc >= search_end {
                        break;
                    }
                    max_forward_target =
                        Some(max_forward_target.map_or(target_pc, |cur| cur.max(target_pc)));
                    continue;
                }
                let escapes_reg = (fwd_raw == OpCode::SetField as u8
                    && fwd_inst.dst() == target_reg)
                    || (fwd_raw == OpCode::SetArray as u8 && fwd_inst.src2() == target_reg)
                    || ((fwd_raw == OpCode::SetStatic as u8
                        || fwd_raw == OpCode::SetStaticDirect as u8
                        || fwd_raw == OpCode::SetStaticGlobal as u8)
                        && fwd_inst.dst() == target_reg)
                    || (fwd_raw == OpCode::PackClosure as u8
                        && target_reg >= fwd_inst.arg_base()
                        && target_reg < fwd_inst.arg_base() + fwd_inst.arg_count() as u16)
                    || (fwd_raw == OpCode::AllocTuple as u8
                        && target_reg >= fwd_inst.src1()
                        && target_reg < fwd_inst.src1() + fwd_inst.src2());
                if escapes_reg
                    || instruction_write_dst(&fwd_inst) == Some(target_reg)
                    || is_control_flow_terminator(fwd_raw)
                    || fwd_raw == OpCode::Switch as u8
                    || ((fwd_raw == OpCode::IncRef as u8 || fwd_raw == OpCode::DecRef as u8)
                        && fwd_inst.dst() == arr_reg)
                {
                    break;
                }
            }
            if let Some(dec_pc) = matched_dec_pc {
                function.instructions[inc_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                function.instructions[dec_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            }
        }
    }
}

/// Eliminates redundant `SetField null` on freshly `AllocObject`ed instances or at the start of leaf constructors.
fn elide_fresh_null_set_fields(function: &mut CompiledFunction) {
    let inst_count = function.instructions.len();
    if inst_count < 2 {
        return;
    }
    let critical_targets = collect_critical_targets(function);
    let is_ctor = function.symbol.contains("::constructor");
    let mut fresh_zero_regs: HashSet<u16> = HashSet::new();
    let mut block_null_regs: HashSet<u16> = HashSet::new();
    if is_ctor {
        fresh_zero_regs.insert(0);
    }

    for pc_idx in 0..inst_count {
        if critical_targets.contains(&pc_idx) {
            fresh_zero_regs.clear();
            block_null_regs.clear();
        }
        let inst = function.instructions[pc_idx];
        let raw = inst.raw_opcode();

        if raw == OpCode::SetField as u8
            && fresh_zero_regs.contains(&inst.src1())
            && block_null_regs.contains(&inst.dst())
        {
            function.instructions[pc_idx] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            continue;
        }

        if raw == OpCode::AllocObject as u8 {
            fresh_zero_regs.insert(inst.dst());
            block_null_regs.remove(&inst.dst());
            continue;
        }

        if raw == OpCode::LoadNull as u8 {
            block_null_regs.insert(inst.dst());
            fresh_zero_regs.remove(&inst.dst());
            continue;
        }

        if let Some(written) = instruction_write_dst(&inst) {
            block_null_regs.remove(&written);
            fresh_zero_regs.remove(&written);
        }

        if is_control_flow_terminator(raw)
            || is_conditional_branch(raw)
            || raw == OpCode::Jump as u8
            || raw == OpCode::Switch as u8
            || raw == OpCode::CallDirect as u8
            || raw == OpCode::CallVirtual as u8
            || raw == OpCode::CallInterface as u8
            || raw == OpCode::CallCallable as u8
            || raw == OpCode::CallSymbol as u8
            || raw == OpCode::CallExtern as u8
        {
            fresh_zero_regs.clear();
            block_null_regs.clear();
        }
    }
}

/// Inlines empty or pure field-initializing leaf constructors at `CallDirect` call sites.
fn inline_leaf_constructors(
    caller: &mut CompiledFunction,
    caller_index: usize,
    all_functions: &[CompiledFunction],
) -> bool {
    let mut is_leaf_ctor = vec![false; all_functions.len()];
    for (func_idx, candidate) in all_functions.iter().enumerate() {
        if func_idx == caller_index
            || !candidate.symbol.contains("::constructor")
            || !candidate.exception_handlers.is_empty()
            || candidate.param_count == 0
            || candidate.instructions.is_empty()
            || candidate.instructions.len() > 8
        {
            continue;
        }
        let body_ok = candidate
            .instructions
            .iter()
            .enumerate()
            .all(|(idx, inst)| {
                let raw = inst.raw_opcode();
                if idx + 1 == candidate.instructions.len() {
                    return raw == OpCode::Return as u8;
                }
                if raw == OpCode::Nop as u8 || raw == OpCode::Mov as u8 {
                    return true;
                }
                if raw == OpCode::LoadI32 as u8
                    || raw == OpCode::LoadI64 as u8
                    || raw == OpCode::LoadF64 as u8
                    || raw == OpCode::LoadBool as u8
                    || raw == OpCode::LoadNull as u8
                    || raw == OpCode::LoadConst as u8
                {
                    return true;
                }
                raw == OpCode::SetField as u8 && inst.src1() == 0
            });
        is_leaf_ctor[func_idx] = body_ok;
    }

    let inst_count = caller.instructions.len();
    let has_candidate = caller.instructions.iter().any(|inst| {
        inst.raw_opcode() == OpCode::CallDirect as u8
            && (inst.call_target() as usize) < is_leaf_ctor.len()
            && is_leaf_ctor[inst.call_target() as usize]
    });
    if !has_candidate {
        return false;
    }

    let mut jump_targets: Vec<Option<usize>> = Vec::with_capacity(inst_count);
    let mut incoming_targets = vec![false; inst_count + 1];
    for (pc_idx, inst) in caller.instructions.iter().enumerate() {
        let target = decode_jump_target(*inst, pc_idx, inst_count);
        if let Some(target_pc) = target
            && target_pc < incoming_targets.len()
        {
            incoming_targets[target_pc] = true;
        }
        jump_targets.push(target);
    }
    for handler in &caller.exception_handlers {
        let handler_pc = handler.handler_pc as usize;
        if handler_pc < incoming_targets.len() {
            incoming_targets[handler_pc] = true;
        }
    }

    let mut rebuilt: Vec<(Instruction, Option<usize>)> = Vec::with_capacity(inst_count + 16);
    let mut old_pc_to_new_pc = vec![u32::MAX; inst_count + 1];
    let mut any_inlined = false;

    for (old_pc, &inst) in caller.instructions.iter().enumerate() {
        old_pc_to_new_pc[old_pc] = rebuilt.len() as u32;
        if inst.raw_opcode() == OpCode::CallDirect as u8 {
            let target_idx = inst.call_target() as usize;
            let arg_base = inst.arg_base();
            let arg_count = inst.arg_count() as u16;
            if target_idx < is_leaf_ctor.len()
                && is_leaf_ctor[target_idx]
                && arg_count == all_functions[target_idx].param_count
            {
                let callee = &all_functions[target_idx];
                let scratch_needed = callee.num_registers.saturating_sub(arg_count);
                if (caller.num_registers as usize) + (scratch_needed as usize) < u16::MAX as usize {
                    let scratch_base = caller.num_registers;
                    caller.num_registers += scratch_needed;
                    let remap_reg = |reg: u16| -> u16 {
                        if reg < arg_count {
                            arg_base + reg
                        } else {
                            scratch_base + (reg - arg_count)
                        }
                    };
                    let body_len = callee.instructions.len() - 1;
                    let mut pushed_any = false;
                    for &ctor_inst in &callee.instructions[..body_len] {
                        let c_raw = ctor_inst.raw_opcode();
                        if c_raw == OpCode::Nop as u8 {
                            continue;
                        }
                        let remapped = match c_raw {
                            raw if raw == OpCode::Mov as u8 => Instruction::format_a(
                                OpCode::Mov,
                                0,
                                remap_reg(ctor_inst.dst()),
                                remap_reg(ctor_inst.src1()),
                                0,
                            ),
                            raw if raw == OpCode::SetField as u8 => Instruction::format_a(
                                OpCode::SetField,
                                ctor_inst.flags(),
                                remap_reg(ctor_inst.dst()),
                                remap_reg(ctor_inst.src1()),
                                ctor_inst.src2(),
                            ),
                            _ => rewrite_instruction_destination(
                                &ctor_inst,
                                remap_reg(ctor_inst.dst()),
                            ),
                        };
                        rebuilt.push((remapped, None));
                        pushed_any = true;
                    }
                    if !pushed_any {
                        if let Some((prev_inst, None)) = rebuilt.last()
                            && !incoming_targets[old_pc]
                            && arg_count == 1
                            && prev_inst.raw_opcode() == OpCode::Mov as u8
                            && prev_inst.dst() == arg_base
                        {
                            let mut overwrites_before_read = false;
                            for (&scan_inst, &is_target) in caller.instructions[old_pc + 1..]
                                .iter()
                                .zip(&incoming_targets[old_pc + 1..inst_count])
                            {
                                if is_target {
                                    break;
                                }
                                let scan_raw = scan_inst.raw_opcode();
                                if instruction_reads_register(&scan_inst, arg_base) {
                                    break;
                                }
                                if instruction_write_dst(&scan_inst) == Some(arg_base) {
                                    overwrites_before_read = true;
                                    break;
                                }
                                if is_control_flow_terminator(scan_raw)
                                    || is_conditional_branch(scan_raw)
                                    || scan_raw == OpCode::Jump as u8
                                    || scan_raw == OpCode::Switch as u8
                                {
                                    break;
                                }
                            }
                            if overwrites_before_read
                                && let Some((slot_inst, _)) = rebuilt.last_mut()
                            {
                                *slot_inst = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                            }
                        }
                        rebuilt.push((Instruction::format_a(OpCode::Nop, 0, 0, 0, 0), None));
                    }
                    any_inlined = true;
                    continue;
                }
            }
        }
        rebuilt.push((inst, jump_targets[old_pc]));
    }

    if !any_inlined {
        return false;
    }

    old_pc_to_new_pc[inst_count] = rebuilt.len() as u32;
    let mut final_instructions = Vec::with_capacity(rebuilt.len());
    for (new_pc, (inst, target_opt)) in rebuilt.into_iter().enumerate() {
        let mut final_inst = inst;
        if let Some(target_old_pc) = target_opt {
            let target_new_pc = if target_old_pc < old_pc_to_new_pc.len() {
                old_pc_to_new_pc[target_old_pc]
            } else {
                old_pc_to_new_pc[inst_count]
            };
            final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc);
        }
        final_instructions.push(final_inst);
    }
    apply_pc_relocation(caller, final_instructions, &old_pc_to_new_pc);
    true
}

/// Executes convergent multi-pass peephole fusion and redundant `Mov` elimination
/// backed by forward CFG liveness verification.
fn optimize_peephole_passes(function: &mut CompiledFunction) {
    let original_count = function.instructions.len();
    if original_count == 0 {
        return;
    }

    let critical_targets = collect_critical_targets(function);
    let mut stream: Vec<IntermediateInstruction> = Vec::with_capacity(original_count);

    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        stream.push(IntermediateInstruction {
            instruction: *instruction,
            original_pc: pc_index,
            covered_original_pcs: vec![pc_index],
            jump_target_old_pc: decode_jump_target(*instruction, pc_index, original_count),
        });
    }

    let max_passes = 6;
    for _pass_index in 0..max_passes {
        let mut old_pc_to_stream_idx: Vec<Option<usize>> = vec![None; original_count + 1];
        for (stream_idx, entry) in stream.iter().enumerate() {
            for &covered_pc in &entry.covered_original_pcs {
                if covered_pc < old_pc_to_stream_idx.len() {
                    old_pc_to_stream_idx[covered_pc] = Some(stream_idx);
                }
            }
        }
        old_pc_to_stream_idx[original_count] = Some(stream.len());
        for reverse_pc in (0..original_count).rev() {
            if old_pc_to_stream_idx[reverse_pc].is_none() {
                old_pc_to_stream_idx[reverse_pc] = old_pc_to_stream_idx[reverse_pc + 1];
            }
        }
        let mut stream_critical = vec![false; stream.len() + 1];
        for &crit_pc in &critical_targets {
            if let Some(Some(target_idx)) = old_pc_to_stream_idx.get(crit_pc).copied()
                && target_idx < stream_critical.len()
            {
                stream_critical[target_idx] = true;
            }
        }

        let mut changed = propagate_block_copies(&mut stream, &stream_critical);
        changed |=
            propagate_single_def_copies(&mut stream, &old_pc_to_stream_idx, &stream_critical);

        let mut next_stream: Vec<IntermediateInstruction> = Vec::with_capacity(stream.len());
        let mut pending_orphaned_pcs: Vec<usize> = Vec::new();
        let mut cursor = 0;

        while cursor < stream.len() {
            let current_entry = &stream[cursor];
            let current_raw = current_entry.instruction.raw_opcode();

            if current_raw == OpCode::Nop as u8 {
                pending_orphaned_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                changed = true;
                cursor += 1;
                continue;
            }

            if current_raw == OpCode::Mov as u8 {
                let mov_dst = current_entry.instruction.dst();
                let mov_src = current_entry.instruction.src1();
                if mov_dst == mov_src {
                    pending_orphaned_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                    changed = true;
                    cursor += 1;
                    continue;
                }
                let src_is_non_ref =
                    is_reg_non_reference_at(&stream, cursor, mov_src, &stream_critical);
                if (!function.reference_registers.contains(&mov_dst) || src_is_non_ref)
                    && is_dead_after_use(
                        &stream,
                        cursor,
                        mov_dst,
                        function,
                        &old_pc_to_stream_idx,
                        &stream_critical,
                    )
                {
                    pending_orphaned_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                    changed = true;
                    cursor += 1;
                    continue;
                }
            }

            if (OpCode::LoadI32 as u8..=OpCode::LoadConst as u8).contains(&current_raw)
                && is_dead_after_use(
                    &stream,
                    cursor,
                    current_entry.instruction.dst(),
                    function,
                    &old_pc_to_stream_idx,
                    &stream_critical,
                )
            {
                pending_orphaned_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                changed = true;
                cursor += 1;
                continue;
            }

            if cursor + 1 < stream.len() {
                let next_entry = &stream[cursor + 1];
                let next_is_critical = stream_critical[cursor + 1];
                let next_raw = next_entry.instruction.raw_opcode();

                // ---------------------------------------------------------
                // Pattern 1: Compare + Conditional Jump Fusion
                // ---------------------------------------------------------
                let is_compare = current_raw == OpCode::LtI32 as u8
                    || current_raw == OpCode::LteI32 as u8
                    || current_raw == OpCode::GtI32 as u8
                    || current_raw == OpCode::GteI32 as u8
                    || current_raw == OpCode::EqI32 as u8
                    || current_raw == OpCode::NeqI32 as u8;

                let is_branch =
                    next_raw == OpCode::JumpIfFalse as u8 || next_raw == OpCode::JumpIfTrue as u8;

                if !next_is_critical && is_compare && is_branch {
                    let cmp_dst = current_entry.instruction.dst();
                    if cmp_dst == next_entry.instruction.cond_reg()
                        && is_dead_after_use(
                            &stream,
                            cursor + 1,
                            cmp_dst,
                            function,
                            &old_pc_to_stream_idx,
                            &stream_critical,
                        )
                    {
                        let fused_opcode =
                            match (current_raw, next_raw == OpCode::JumpIfFalse as u8) {
                                (raw_val, true) if raw_val == OpCode::LtI32 as u8 => {
                                    OpCode::JumpIfGteI32
                                }
                                (raw_val, false) if raw_val == OpCode::LtI32 as u8 => {
                                    OpCode::JumpIfLtI32
                                }
                                (raw_val, true) if raw_val == OpCode::LteI32 as u8 => {
                                    OpCode::JumpIfGtI32
                                }
                                (raw_val, false) if raw_val == OpCode::LteI32 as u8 => {
                                    OpCode::JumpIfLteI32
                                }
                                (raw_val, true) if raw_val == OpCode::GtI32 as u8 => {
                                    OpCode::JumpIfLteI32
                                }
                                (raw_val, false) if raw_val == OpCode::GtI32 as u8 => {
                                    OpCode::JumpIfGtI32
                                }
                                (raw_val, true) if raw_val == OpCode::GteI32 as u8 => {
                                    OpCode::JumpIfLtI32
                                }
                                (raw_val, false) if raw_val == OpCode::GteI32 as u8 => {
                                    OpCode::JumpIfGteI32
                                }
                                (raw_val, true) if raw_val == OpCode::EqI32 as u8 => {
                                    OpCode::JumpIfNeqI32
                                }
                                (raw_val, false) if raw_val == OpCode::EqI32 as u8 => {
                                    OpCode::JumpIfEqI32
                                }
                                (raw_val, true) if raw_val == OpCode::NeqI32 as u8 => {
                                    OpCode::JumpIfEqI32
                                }
                                _ => OpCode::JumpIfNeqI32,
                            };

                        let reg_a = current_entry.instruction.src1();
                        let reg_b = current_entry.instruction.src2();
                        let mut merged_pcs = {
                            let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                            combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                            combined_pcs
                        };
                        merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                        next_stream.push(IntermediateInstruction {
                            instruction: Instruction::format_a(fused_opcode, 0, reg_a, reg_b, 0),
                            original_pc: current_entry.original_pc,
                            covered_original_pcs: merged_pcs,
                            jump_target_old_pc: next_entry.jump_target_old_pc,
                        });

                        changed = true;
                        cursor += 2;
                        continue;
                    }
                }

                // ---------------------------------------------------------
                // Pattern 2: LoadI32 + AddI32 / SubI32 Fusion
                // ---------------------------------------------------------
                if !next_is_critical && current_raw == OpCode::LoadI32 as u8 {
                    let imm_val = current_entry.instruction.imm32_signed();
                    if (-32768..=32767).contains(&imm_val) {
                        let r_imm = current_entry.instruction.dst();
                        if next_raw == OpCode::AddI32 as u8 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();

                            let other_reg = if src2_reg == r_imm && src1_reg != r_imm {
                                Some(src1_reg)
                            } else if src1_reg == r_imm && src2_reg != r_imm {
                                Some(src2_reg)
                            } else {
                                None
                            };

                            if let Some(r_other) = other_reg
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_imm,
                                    function,
                                    &old_pc_to_stream_idx,
                                    &stream_critical,
                                )
                            {
                                let mut merged_pcs = {
                                    let mut combined_pcs =
                                        std::mem::take(&mut pending_orphaned_pcs);
                                    combined_pcs
                                        .extend_from_slice(&current_entry.covered_original_pcs);
                                    combined_pcs
                                };
                                merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                                next_stream.push(IntermediateInstruction {
                                    instruction: Instruction::format_a(
                                        OpCode::AddI32Imm,
                                        0,
                                        dst_reg,
                                        r_other,
                                        imm_val as i16 as u16,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: None,
                                });

                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        } else if next_raw == OpCode::SubI32 as u8 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();

                            if src2_reg == r_imm
                                && src1_reg != r_imm
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_imm,
                                    function,
                                    &old_pc_to_stream_idx,
                                    &stream_critical,
                                )
                            {
                                let mut merged_pcs = {
                                    let mut combined_pcs =
                                        std::mem::take(&mut pending_orphaned_pcs);
                                    combined_pcs
                                        .extend_from_slice(&current_entry.covered_original_pcs);
                                    combined_pcs
                                };
                                merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                                next_stream.push(IntermediateInstruction {
                                    instruction: Instruction::format_a(
                                        OpCode::SubI32Imm,
                                        0,
                                        dst_reg,
                                        src1_reg,
                                        imm_val as i16 as u16,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: None,
                                });

                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        } else if next_raw == OpCode::MulI32 as u8 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();

                            let other_reg = if src2_reg == r_imm && src1_reg != r_imm {
                                Some(src1_reg)
                            } else if src1_reg == r_imm && src2_reg != r_imm {
                                Some(src2_reg)
                            } else {
                                None
                            };

                            if let Some(r_other) = other_reg
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_imm,
                                    function,
                                    &old_pc_to_stream_idx,
                                    &stream_critical,
                                )
                            {
                                let mut merged_pcs = {
                                    let mut combined_pcs =
                                        std::mem::take(&mut pending_orphaned_pcs);
                                    combined_pcs
                                        .extend_from_slice(&current_entry.covered_original_pcs);
                                    combined_pcs
                                };
                                merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                                next_stream.push(IntermediateInstruction {
                                    instruction: Instruction::format_a(
                                        OpCode::MulI32Imm,
                                        0,
                                        dst_reg,
                                        r_other,
                                        imm_val as i16 as u16,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: None,
                                });

                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        } else if next_raw == OpCode::ModI32 as u8 && imm_val > 0 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();

                            if src2_reg == r_imm
                                && src1_reg != r_imm
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_imm,
                                    function,
                                    &old_pc_to_stream_idx,
                                    &stream_critical,
                                )
                            {
                                let mut merged_pcs = {
                                    let mut combined_pcs =
                                        std::mem::take(&mut pending_orphaned_pcs);
                                    combined_pcs
                                        .extend_from_slice(&current_entry.covered_original_pcs);
                                    combined_pcs
                                };
                                merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                                next_stream.push(IntermediateInstruction {
                                    instruction: Instruction::format_a(
                                        OpCode::ModI32Imm,
                                        0,
                                        dst_reg,
                                        src1_reg,
                                        imm_val as i16 as u16,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: None,
                                });

                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        }
                    }
                }

                // ---------------------------------------------------------
                // Pattern 3: Peephole Mov Elimination
                // ---------------------------------------------------------
                if !next_is_critical
                    && current_raw != OpCode::Mov as u8
                    && next_raw == OpCode::Mov as u8
                {
                    let r_tmp = next_entry.instruction.src1();
                    let r_dest = next_entry.instruction.dst();

                    if instruction_write_dst(&current_entry.instruction) == Some(r_tmp)
                        && r_tmp != r_dest
                        && can_change_instruction_destination(&current_entry.instruction, r_dest)
                        && is_dead_after_use(
                            &stream,
                            cursor + 1,
                            r_tmp,
                            function,
                            &old_pc_to_stream_idx,
                            &stream_critical,
                        )
                    {
                        let updated_instruction =
                            rewrite_instruction_destination(&current_entry.instruction, r_dest);
                        let mut merged_pcs = {
                            let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                            combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                            combined_pcs
                        };
                        merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                        next_stream.push(IntermediateInstruction {
                            instruction: updated_instruction,
                            original_pc: current_entry.original_pc,
                            covered_original_pcs: merged_pcs,
                            jump_target_old_pc: current_entry.jump_target_old_pc,
                        });

                        changed = true;
                        cursor += 2;
                        continue;
                    }
                }
            }

            let mut cloned_entry = current_entry.clone();
            if !pending_orphaned_pcs.is_empty() {
                let mut combined = std::mem::take(&mut pending_orphaned_pcs);
                combined.extend_from_slice(&cloned_entry.covered_original_pcs);
                cloned_entry.covered_original_pcs = combined;
            }
            next_stream.push(cloned_entry);
            cursor += 1;
        }
        if !pending_orphaned_pcs.is_empty()
            && let Some(last_entry) = next_stream.last_mut()
        {
            last_entry
                .covered_original_pcs
                .extend_from_slice(&pending_orphaned_pcs);
        }

        stream = next_stream;
        if !changed {
            break;
        }
    }

    // Relocate instruction PCs and branch offsets
    let sentinel_pc = u32::MAX;
    let mut old_pc_to_new_pc = vec![sentinel_pc; original_count + 1];
    for (new_pc, entry) in stream.iter().enumerate() {
        for &covered_pc in &entry.covered_original_pcs {
            if covered_pc < old_pc_to_new_pc.len() {
                old_pc_to_new_pc[covered_pc] = new_pc as u32;
            }
        }
    }
    old_pc_to_new_pc[original_count] = stream.len() as u32;
    for reverse_index in (0..original_count).rev() {
        if old_pc_to_new_pc[reverse_index] == sentinel_pc {
            old_pc_to_new_pc[reverse_index] = old_pc_to_new_pc[reverse_index + 1];
        }
    }

    let mut final_instructions = Vec::with_capacity(stream.len());
    for (new_pc, intermediate) in stream.iter().enumerate() {
        let mut final_inst = intermediate.instruction;
        if let Some(target_old_pc) = intermediate.jump_target_old_pc {
            let target_new_pc = if target_old_pc < old_pc_to_new_pc.len() {
                old_pc_to_new_pc[target_old_pc]
            } else {
                stream.len() as u32
            };
            final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc);
        }
        final_instructions.push(final_inst);
    }

    apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
}

/// Hoists loop-invariant constant loads (`LoadI32`, `LoadI64`, `LoadF64`, `LoadBool`,
/// `LoadConst`) above loop headers into fresh registers and rewrites in-loop consumers.
fn hoist_loop_invariant_constants(function: &mut CompiledFunction) {
    let inst_count = function.instructions.len();
    if inst_count < 3 {
        return;
    }

    let critical_targets = collect_critical_targets(function);
    let mut handler_pcs = HashSet::new();
    for handler in &function.exception_handlers {
        handler_pcs.insert(handler.try_start_pc as usize);
        handler_pcs.insert(handler.try_end_pc as usize);
        handler_pcs.insert(handler.handler_pc as usize);
        if let Some(finally_pc) = handler.finally_pc {
            handler_pcs.insert(finally_pc as usize);
        }
    }

    // Group backward jumps by header PC -> outermost enclosing back-edge end PC
    let mut loops_by_header: HashMap<usize, usize> = HashMap::new();
    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        if let Some(target_pc) = decode_jump_target(*instruction, pc_index, inst_count)
            && target_pc <= pc_index
            && !handler_pcs.contains(&target_pc)
        {
            loops_by_header
                .entry(target_pc)
                .and_modify(|existing_end| *existing_end = (*existing_end).max(pc_index))
                .or_insert(pc_index);
        }
    }

    if loops_by_header.is_empty() {
        return;
    }

    let mut ordered_headers: Vec<(usize, usize)> = loops_by_header.into_iter().collect();
    ordered_headers.sort_by_key(|&(header_pc, _)| header_pc);

    // Build initial CFG liveness helper table
    let mut old_pc_targets: Vec<Option<usize>> = Vec::with_capacity(inst_count);
    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        old_pc_targets.push(decode_jump_target(*instruction, pc_index, inst_count));
    }

    let mut preheaders: HashMap<usize, Vec<Instruction>> = HashMap::new();
    let mut hoisted_cache: HashMap<(usize, u8, u8, u32), u16> = HashMap::new();
    let mut hoisted_any = false;

    for (header_pc, loop_end_pc) in ordered_headers {
        let has_interior_external_entry = old_pc_targets[..header_pc]
            .iter()
            .flatten()
            .any(|&target_pc| target_pc > header_pc && target_pc <= loop_end_pc);
        if has_interior_external_entry {
            continue;
        }

        let mut cursor = header_pc;
        while cursor <= loop_end_pc && cursor < function.instructions.len() {
            let inst = function.instructions[cursor];
            let raw = inst.raw_opcode();
            let is_hoistable_load = raw == OpCode::LoadI32 as u8
                || raw == OpCode::LoadI64 as u8
                || raw == OpCode::LoadF64 as u8
                || raw == OpCode::LoadBool as u8
                || raw == OpCode::LoadConst as u8;

            if !is_hoistable_load {
                cursor += 1;
                continue;
            }

            let r_orig = inst.dst();
            let mut consumer_pcs: Vec<usize> = Vec::new();
            let mut last_use_pc: Option<usize> = None;
            let mut aborted_rewrite = false;
            let mut terminated_block = false;

            for scan_pc in (cursor + 1)..inst_count {
                if critical_targets.contains(&scan_pc) {
                    break;
                }
                let scan_inst = function.instructions[scan_pc];
                let scan_raw = scan_inst.raw_opcode();

                let reads_orig = instruction_reads_register(&scan_inst, r_orig);
                if reads_orig {
                    if can_rewrite_read_register(&scan_inst, r_orig) {
                        consumer_pcs.push(scan_pc);
                        last_use_pc = Some(scan_pc);
                    } else {
                        aborted_rewrite = true;
                        break;
                    }
                }

                if instruction_write_dst(&scan_inst) == Some(r_orig) {
                    terminated_block = true;
                    break;
                }

                if is_control_flow_terminator(scan_raw)
                    || is_conditional_branch(scan_raw)
                    || scan_raw == OpCode::Switch as u8
                {
                    break;
                }
            }

            if aborted_rewrite || consumer_pcs.is_empty() {
                cursor += 1;
                continue;
            }

            let Some(last_pc) = last_use_pc else {
                cursor += 1;
                continue;
            };

            let dead_after = if terminated_block
                && instruction_write_dst(&function.instructions[last_pc]) == Some(r_orig)
            {
                true
            } else {
                is_dead_after_use_raw(
                    function,
                    &old_pc_targets,
                    &critical_targets,
                    last_pc,
                    r_orig,
                )
            };

            if !dead_after {
                cursor += 1;
                continue;
            }

            let cache_key = (header_pc, raw, inst.flags(), inst.imm32());
            let hoisted_reg = if let Some(&existing_reg) = hoisted_cache.get(&cache_key) {
                existing_reg
            } else {
                if (function.num_registers as usize) >= (u16::MAX as usize) {
                    break;
                }
                let alloc_reg = function.num_registers;
                function.num_registers += 1;
                preheaders
                    .entry(header_pc)
                    .or_default()
                    .push(inst.with_dst(alloc_reg));
                hoisted_cache.insert(cache_key, alloc_reg);
                alloc_reg
            };

            for consumer_pc in consumer_pcs {
                if let Some(rewritten) = rewrite_instruction_read_register(
                    &function.instructions[consumer_pc],
                    r_orig,
                    hoisted_reg,
                ) {
                    function.instructions[consumer_pc] = rewritten;
                }
            }

            function.instructions[cursor] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            hoisted_any = true;
            cursor += 1;
        }
    }

    if !hoisted_any {
        return;
    }

    // Rebuild instruction list with hoisted preheaders and exact forward vs back-edge targets
    let mut rebuilt: Vec<(Instruction, Option<(usize, usize)>)> =
        Vec::with_capacity(inst_count + 16);
    let sentinel_pc = u32::MAX;
    let mut old_pc_to_new_pc = vec![sentinel_pc; inst_count + 1];
    let mut preheader_start_pc: HashMap<usize, u32> = HashMap::new();

    for (old_pc, instruction) in function.instructions.iter().enumerate() {
        if let Some(header_loads) = preheaders.get(&old_pc) {
            preheader_start_pc.insert(old_pc, rebuilt.len() as u32);
            for &hoisted_inst in header_loads {
                rebuilt.push((hoisted_inst, None));
            }
        }

        if instruction.raw_opcode() == OpCode::Nop as u8 {
            continue;
        }

        old_pc_to_new_pc[old_pc] = rebuilt.len() as u32;
        let jump_info = old_pc_targets[old_pc].map(|target| (old_pc, target));
        rebuilt.push((*instruction, jump_info));
    }

    old_pc_to_new_pc[inst_count] = rebuilt.len() as u32;
    for reverse_index in (0..inst_count).rev() {
        if old_pc_to_new_pc[reverse_index] == sentinel_pc {
            old_pc_to_new_pc[reverse_index] = old_pc_to_new_pc[reverse_index + 1];
        }
    }

    let mut final_instructions = Vec::with_capacity(rebuilt.len());
    for (new_pc, (inst, jump_info)) in rebuilt.into_iter().enumerate() {
        let mut final_inst = inst;
        if let Some((origin_old_pc, target_old_pc)) = jump_info {
            let target_new_pc = if origin_old_pc < target_old_pc {
                if let Some(&preheader_pc) = preheader_start_pc.get(&target_old_pc) {
                    preheader_pc
                } else if target_old_pc < old_pc_to_new_pc.len() {
                    old_pc_to_new_pc[target_old_pc]
                } else {
                    old_pc_to_new_pc[inst_count]
                }
            } else if target_old_pc < old_pc_to_new_pc.len() {
                old_pc_to_new_pc[target_old_pc]
            } else {
                old_pc_to_new_pc[inst_count]
            };
            final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc);
        }
        final_instructions.push(final_inst);
    }

    apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
}

/// Decodes a branch or jump instruction's absolute target PC within `[0..=inst_count]`.
fn decode_jump_target(
    instruction: Instruction,
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
    } else if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode) {
        let target_pc = (pc_index as i32) + (instruction.src2() as i16 as i32);
        if target_pc >= 0 && (target_pc as usize) <= inst_count {
            return Some(target_pc as usize);
        }
    }
    None
}

/// Rewrites the signed PC jump offset of a branch/jump instruction cleanly.
fn relocate_jump_instruction(
    instruction: Instruction,
    new_pc: usize,
    target_new_pc: u32,
) -> Instruction {
    let new_offset = (target_new_pc as i32) - (new_pc as i32);
    let raw_opcode = instruction.raw_opcode();

    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode) {
        if let Some(opcode) = OpCode::from_raw_u8(raw_opcode) {
            return Instruction::format_a(
                opcode,
                0,
                instruction.dst(),
                instruction.src1(),
                new_offset as i16 as u16,
            );
        }
    } else if (raw_opcode == OpCode::Jump as u8
        || raw_opcode == OpCode::JumpIfTrue as u8
        || raw_opcode == OpCode::JumpIfFalse as u8)
        && let Some(opcode) = OpCode::from_raw_u8(raw_opcode)
    {
        return Instruction::format_c(
            opcode,
            instruction.flags(),
            instruction.cond_reg(),
            new_offset,
        );
    }

    instruction
}

/// Returns `true` if `opcode` strictly writes a primitive or null value (never a heap reference).
fn opcode_writes_only_non_reference(raw: u8) -> bool {
    raw == OpCode::LoadI32 as u8
        || raw == OpCode::LoadI64 as u8
        || raw == OpCode::LoadF64 as u8
        || raw == OpCode::LoadBool as u8
        || raw == OpCode::LoadNull as u8
        || (OpCode::AddI32 as u8..=OpCode::Not as u8).contains(&raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::Instanceof as u8
        || raw == OpCode::ArrayLength as u8
}

/// Scans backward within the current basic block up to `at_idx` to prove `target_reg` holds a non-reference value.
fn is_reg_non_reference_at(
    stream: &[IntermediateInstruction],
    at_idx: usize,
    target_reg: u16,
    stream_critical: &[bool],
) -> bool {
    let mut scan_idx = at_idx;
    loop {
        let inst = &stream[scan_idx].instruction;
        if scan_idx != at_idx && instruction_write_dst(inst) == Some(target_reg) {
            return opcode_writes_only_non_reference(inst.raw_opcode());
        }
        if scan_idx == at_idx && instruction_write_dst(inst) == Some(target_reg) {
            let cur_raw = inst.raw_opcode();
            if opcode_writes_only_non_reference(cur_raw) {
                return true;
            }
            if cur_raw == OpCode::Mov as u8 {
                let source_reg = inst.src1();
                if source_reg != target_reg {
                    // Check if source_reg was defined right in this block as non-reference
                    return is_reg_non_reference_at(stream, scan_idx, source_reg, stream_critical);
                }
            }
            return false;
        }
        if stream_critical[scan_idx] || scan_idx == 0 {
            return false;
        }
        scan_idx -= 1;
        let prev_raw = stream[scan_idx].instruction.raw_opcode();
        if is_control_flow_terminator(prev_raw)
            || is_conditional_branch(prev_raw)
            || prev_raw == OpCode::Jump as u8
            || prev_raw == OpCode::Switch as u8
        {
            return false;
        }
    }
}

/// Scans backward in raw instructions up to `at_pc` to prove `target_reg` holds a non-reference value.
fn is_reg_non_reference_raw(
    function: &CompiledFunction,
    at_pc: usize,
    target_reg: u16,
    critical_targets: &HashSet<usize>,
) -> bool {
    let mut scan_pc = at_pc;
    loop {
        let inst = &function.instructions[scan_pc];
        if instruction_write_dst(inst) == Some(target_reg) {
            return opcode_writes_only_non_reference(inst.raw_opcode());
        }
        if scan_pc == 0 || critical_targets.contains(&scan_pc) {
            return false;
        }
        scan_pc -= 1;
        let prev_raw = function.instructions[scan_pc].raw_opcode();
        if is_control_flow_terminator(prev_raw)
            || is_conditional_branch(prev_raw)
            || prev_raw == OpCode::Jump as u8
            || prev_raw == OpCode::Switch as u8
        {
            return false;
        }
    }
}

/// Performs local basic-block copy propagation (`Mov dst, src` -> replace reads of `dst` with `src`).
fn propagate_block_copies(
    stream: &mut [IntermediateInstruction],
    stream_critical: &[bool],
) -> bool {
    let mut any_changed = false;
    let mut copy_map: HashMap<u16, u16> = HashMap::new();

    for (stream_idx, entry) in stream.iter_mut().enumerate() {
        if stream_critical[stream_idx] {
            copy_map.clear();
        }

        let raw = entry.instruction.raw_opcode();
        if !copy_map.is_empty() && raw != OpCode::IncRef as u8 && raw != OpCode::DecRef as u8 {
            let candidates: Vec<(u16, u16)> =
                copy_map.iter().map(|(&key, &val)| (key, val)).collect();
            for (alias_reg, orig_reg) in candidates {
                if instruction_reads_register(&entry.instruction, alias_reg)
                    && can_rewrite_read_register(&entry.instruction, alias_reg)
                    && let Some(rewritten) =
                        rewrite_instruction_read_register(&entry.instruction, alias_reg, orig_reg)
                {
                    entry.instruction = rewritten;
                    any_changed = true;
                }
            }
        }

        let updated_raw = entry.instruction.raw_opcode();
        if updated_raw == OpCode::Mov as u8 {
            let mov_dst = entry.instruction.dst();
            let mov_src = entry.instruction.src1();
            let canonical_src = copy_map.get(&mov_src).copied().unwrap_or(mov_src);
            copy_map.retain(|&key_reg, &mut val_reg| key_reg != mov_dst && val_reg != mov_dst);
            if mov_dst != canonical_src {
                copy_map.insert(mov_dst, canonical_src);
            }
        } else {
            if let Some(written_reg) = instruction_write_dst(&entry.instruction) {
                copy_map.retain(|&key_reg, &mut val_reg| {
                    key_reg != written_reg && val_reg != written_reg
                });
            }
            if is_control_flow_terminator(updated_raw)
                || is_conditional_branch(updated_raw)
                || updated_raw == OpCode::Jump as u8
                || updated_raw == OpCode::Switch as u8
            {
                copy_map.clear();
            }
        }
    }

    any_changed
}

/// Propagates single-definition `Mov alias, orig` copies across forward conditional diamonds
/// when `alias` has a single definition, `orig` is written in the same basic block before `alias`
/// and never modified across the interval `[orig_idx + 1..=max_read_idx]`, and all uses of `alias`
/// lie inside a closed forward CFG diamond.
fn propagate_single_def_copies(
    stream: &mut [IntermediateInstruction],
    old_pc_to_stream_idx: &[Option<usize>],
    stream_critical: &[bool],
) -> bool {
    let stream_len = stream.len();
    if stream_len < 3 {
        return false;
    }

    let mut def_counts: HashMap<u16, usize> = HashMap::new();
    let mut mov_candidates: Vec<(usize, u16, u16)> = Vec::new();
    let mut jump_edges: Vec<(usize, usize)> = Vec::new();

    for (idx, entry) in stream.iter().enumerate() {
        let inst = entry.instruction;
        if let Some(written) = instruction_write_dst(&inst) {
            *def_counts.entry(written).or_insert(0) += 1;
        }
        if inst.raw_opcode() == OpCode::Mov as u8 && inst.dst() != inst.src1() {
            mov_candidates.push((idx, inst.dst(), inst.src1()));
        }
        if let Some(target_old) = entry.jump_target_old_pc
            && target_old < old_pc_to_stream_idx.len()
            && let Some(target_idx) = old_pc_to_stream_idx[target_old]
        {
            jump_edges.push((idx, target_idx));
        }
    }

    let mut any_changed = false;
    for (mov_idx, alias_reg, orig_reg) in mov_candidates {
        if def_counts.get(&alias_reg).copied() != Some(1) {
            continue;
        }
        // Scan backward in the same basic block to locate the defining instruction for `orig_reg`.
        let mut orig_def_idx: Option<usize> = None;
        let mut scan_idx = mov_idx;
        while scan_idx > 0 {
            if stream_critical[scan_idx] {
                break;
            }
            scan_idx -= 1;
            let prev_raw = stream[scan_idx].instruction.raw_opcode();
            if is_control_flow_terminator(prev_raw)
                || is_conditional_branch(prev_raw)
                || prev_raw == OpCode::Jump as u8
                || prev_raw == OpCode::Switch as u8
            {
                break;
            }
            if instruction_write_dst(&stream[scan_idx].instruction) == Some(orig_reg) {
                orig_def_idx = Some(scan_idx);
                break;
            }
        }
        let Some(orig_idx) = orig_def_idx else {
            continue;
        };

        let mut max_read_idx = mov_idx;
        let mut reads_outside = false;
        let mut read_indices: Vec<usize> = Vec::new();
        for (scan_idx, entry) in stream.iter().enumerate() {
            let raw = entry.instruction.raw_opcode();
            if instruction_reads_register(&entry.instruction, alias_reg) {
                if raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8 {
                    reads_outside = true;
                    break;
                }
                if scan_idx <= mov_idx {
                    reads_outside = true;
                    break;
                }
                if can_rewrite_read_register(&entry.instruction, alias_reg) {
                    max_read_idx = max_read_idx.max(scan_idx);
                    read_indices.push(scan_idx);
                } else {
                    reads_outside = true;
                    break;
                }
            }
        }
        if reads_outside || read_indices.is_empty() {
            continue;
        }

        let orig_modified_in_span = ((orig_idx + 1)..=max_read_idx)
            .any(|idx| instruction_write_dst(&stream[idx].instruction) == Some(orig_reg));
        if orig_modified_in_span {
            continue;
        }

        let interval_closed = jump_edges.iter().all(|&(src_idx, dst_idx)| {
            let enters_from_outside = dst_idx > mov_idx
                && dst_idx <= max_read_idx
                && !(mov_idx..=max_read_idx).contains(&src_idx);
            let backward_edge_inside = dst_idx <= src_idx
                && (mov_idx..=max_read_idx).contains(&src_idx)
                && (mov_idx..=max_read_idx).contains(&dst_idx);
            let jumps_between_orig_and_mov = dst_idx > orig_idx && dst_idx <= mov_idx;
            !enters_from_outside && !backward_edge_inside && !jumps_between_orig_and_mov
        });
        if !interval_closed {
            continue;
        }

        for read_idx in read_indices {
            if let Some(rewritten) = rewrite_instruction_read_register(
                &stream[read_idx].instruction,
                alias_reg,
                orig_reg,
            ) {
                stream[read_idx].instruction = rewritten;
                any_changed = true;
            }
        }
    }

    any_changed
}

/// Forward CFG liveness check over `IntermediateInstruction` stream: returns `true` iff
/// `reg` is never read again before being overwritten (or exiting) along all paths after `use_idx`.
fn is_dead_after_use(
    stream: &[IntermediateInstruction],
    use_idx: usize,
    reg: u16,
    function: &CompiledFunction,
    old_pc_to_stream_idx: &[Option<usize>],
    stream_critical: &[bool],
) -> bool {
    let len = stream.len();
    if use_idx >= len {
        return true;
    }

    let reg_pinned_as_ref = function.reference_registers.contains(&reg)
        && !is_reg_non_reference_at(stream, use_idx, reg, stream_critical);

    let mut visited = vec![false; len];
    let mut worklist: Vec<usize> = Vec::with_capacity(8);

    let use_entry = &stream[use_idx];
    let use_raw = use_entry.instruction.raw_opcode();

    if !is_control_flow_terminator(use_raw) && use_raw != OpCode::Jump as u8 && use_idx + 1 < len {
        worklist.push(use_idx + 1);
    }
    if use_raw == OpCode::Jump as u8 || is_conditional_branch(use_raw) {
        let Some(target_old) = use_entry.jump_target_old_pc else {
            return false;
        };
        if target_old < old_pc_targets_len(old_pc_to_stream_idx)
            && let Some(target_stream_idx) = old_pc_to_stream_idx[target_old]
            && target_stream_idx < len
        {
            worklist.push(target_stream_idx);
        }
    } else if use_raw == OpCode::Switch as u8 {
        return false;
    }

    while let Some(start_idx) = worklist.pop() {
        for cur in start_idx..len {
            if visited[cur] {
                break;
            }
            visited[cur] = true;

            let entry = &stream[cur];
            let inst = &entry.instruction;
            let raw = inst.raw_opcode();

            if instruction_reads_register(inst, reg) {
                return false;
            }

            if can_instruction_throw(raw) && reg_pinned_as_ref {
                return false;
            }

            if !function.exception_handlers.is_empty() && can_instruction_throw(raw) {
                for handler in &function.exception_handlers {
                    let orig = entry.original_pc as u32;
                    if orig >= handler.try_start_pc && orig < handler.try_end_pc {
                        if handler.try_ref_regs.contains(&reg) {
                            return false;
                        }
                        let handler_pc = handler.handler_pc as usize;
                        if handler_pc < old_pc_to_stream_idx.len() {
                            if let Some(h_idx) = old_pc_to_stream_idx[handler_pc]
                                && h_idx < len
                            {
                                worklist.push(h_idx);
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }

            if instruction_write_dst(inst) == Some(reg) {
                break;
            }

            if is_control_flow_terminator(raw) {
                break;
            }

            if raw == OpCode::Jump as u8 {
                let Some(target_old) = entry.jump_target_old_pc else {
                    return false;
                };
                if target_old < old_pc_to_stream_idx.len() {
                    if let Some(target_idx) = old_pc_to_stream_idx[target_old] {
                        if target_idx < len {
                            worklist.push(target_idx);
                        }
                    } else {
                        return false;
                    }
                }
                break;
            }

            if is_conditional_branch(raw) {
                let Some(target_old) = entry.jump_target_old_pc else {
                    return false;
                };
                if target_old < old_pc_to_stream_idx.len() {
                    if let Some(target_idx) = old_pc_to_stream_idx[target_old] {
                        if target_idx < len {
                            worklist.push(target_idx);
                        }
                    } else {
                        return false;
                    }
                }
            } else if raw == OpCode::Switch as u8 {
                return false;
            }
        }
    }

    true
}

#[inline(always)]
fn old_pc_targets_len(slice: &[Option<usize>]) -> usize {
    slice.len()
}

/// Forward CFG liveness check directly over `function.instructions`.
fn is_dead_after_use_raw(
    function: &CompiledFunction,
    jump_targets: &[Option<usize>],
    critical_targets: &HashSet<usize>,
    use_pc: usize,
    reg: u16,
) -> bool {
    let len = function.instructions.len();
    if use_pc >= len {
        return true;
    }

    let reg_pinned_as_ref = function.reference_registers.contains(&reg)
        && !is_reg_non_reference_raw(function, use_pc, reg, critical_targets);

    let mut visited = vec![false; len];
    let mut worklist: Vec<usize> = Vec::with_capacity(8);

    let use_raw = function.instructions[use_pc].raw_opcode();
    if !is_control_flow_terminator(use_raw) && use_raw != OpCode::Jump as u8 && use_pc + 1 < len {
        worklist.push(use_pc + 1);
    }
    if use_raw == OpCode::Jump as u8 || is_conditional_branch(use_raw) {
        let Some(target_pc) = jump_targets[use_pc] else {
            return false;
        };
        if target_pc < len {
            worklist.push(target_pc);
        }
    } else if use_raw == OpCode::Switch as u8 {
        return false;
    }

    while let Some(start_pc) = worklist.pop() {
        for cur in start_pc..len {
            if visited[cur] {
                break;
            }
            visited[cur] = true;

            let inst = &function.instructions[cur];
            let raw = inst.raw_opcode();

            if instruction_reads_register(inst, reg) {
                return false;
            }

            if can_instruction_throw(raw) && reg_pinned_as_ref {
                return false;
            }

            if !function.exception_handlers.is_empty() && can_instruction_throw(raw) {
                let cur_u32 = cur as u32;
                for handler in &function.exception_handlers {
                    if cur_u32 >= handler.try_start_pc && cur_u32 < handler.try_end_pc {
                        if handler.try_ref_regs.contains(&reg) {
                            return false;
                        }
                        let handler_pc = handler.handler_pc as usize;
                        if handler_pc < len {
                            worklist.push(handler_pc);
                        }
                    }
                }
            }

            if instruction_write_dst(inst) == Some(reg) {
                break;
            }

            if is_control_flow_terminator(raw) {
                break;
            }

            if raw == OpCode::Jump as u8 {
                let Some(target_pc) = jump_targets[cur] else {
                    return false;
                };
                if target_pc < len {
                    worklist.push(target_pc);
                }
                break;
            }

            if is_conditional_branch(raw) {
                let Some(target_pc) = jump_targets[cur] else {
                    return false;
                };
                if target_pc < len {
                    worklist.push(target_pc);
                }
            } else if raw == OpCode::Switch as u8 {
                return false;
            }
        }
    }

    true
}

fn is_control_flow_terminator(raw: u8) -> bool {
    raw == OpCode::Return as u8
        || raw == OpCode::ReturnValue as u8
        || raw == OpCode::Exit as u8
        || raw == OpCode::Throw as u8
}

fn is_conditional_branch(raw: u8) -> bool {
    raw == OpCode::JumpIfTrue as u8
        || raw == OpCode::JumpIfFalse as u8
        || (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
}

fn can_instruction_throw(raw: u8) -> bool {
    !(raw == OpCode::Nop as u8
        || raw == OpCode::Mov as u8
        || raw == OpCode::LoadI32 as u8
        || raw == OpCode::LoadI64 as u8
        || raw == OpCode::LoadF64 as u8
        || raw == OpCode::LoadBool as u8
        || raw == OpCode::LoadNull as u8
        || raw == OpCode::LoadConst as u8
        || raw == OpCode::IncRef as u8
        || raw == OpCode::Jump as u8
        || raw == OpCode::JumpIfTrue as u8
        || raw == OpCode::JumpIfFalse as u8)
}

/// Returns `Some(reg)` if `instruction` writes to a single register `reg`.
fn instruction_write_dst(instruction: &Instruction) -> Option<u16> {
    let raw = instruction.raw_opcode();
    if (OpCode::Mov as u8..=OpCode::LoadConst as u8).contains(&raw)
        || (OpCode::AddI32 as u8..=OpCode::Not as u8).contains(&raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
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
    {
        return Some(instruction.dst());
    }

    if raw == OpCode::CallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
    {
        return Some(instruction.call_dst());
    }

    None
}

/// Checks whether a register read inside `instruction` can be cleanly rewritten from `from_reg` to `to_reg`.
fn can_rewrite_read_register(instruction: &Instruction, target_reg: u16) -> bool {
    let raw = instruction.raw_opcode();
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
    {
        return false;
    }
    if raw == OpCode::CallCallable as u8 {
        let base = instruction.arg_base();
        let count = instruction.arg_count() as u16;
        if target_reg >= base && target_reg < base + count {
            return false;
        }
        return instruction.call_target() == target_reg;
    }
    if raw == OpCode::AllocTuple as u8 || raw == OpCode::UnpackCaptures as u8 {
        return false;
    }
    true
}

/// Rewrites input register operands matching `from_reg` to `to_reg`, returning the updated instruction.
fn rewrite_instruction_read_register(
    instruction: &Instruction,
    from_reg: u16,
    to_reg: u16,
) -> Option<Instruction> {
    let raw = instruction.raw_opcode();
    let opcode = OpCode::from_raw_u8(raw)?;

    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw) {
        let new_dst = if instruction.dst() == from_reg {
            to_reg
        } else {
            instruction.dst()
        };
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            new_dst,
            new_src1,
            instruction.src2(),
        ));
    }

    if raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
    {
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            instruction.dst(),
            new_src1,
            instruction.src2(),
        ));
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
        || raw == OpCode::ArrayLength as u8
        || raw == OpCode::AllocArray as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::Instanceof as u8
    {
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            instruction.dst(),
            new_src1,
            instruction.src2(),
        ));
    }

    if raw == OpCode::TupleInstanceof as u8 {
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        let new_src2 = if instruction.src2() == from_reg {
            to_reg
        } else {
            instruction.src2()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            instruction.dst(),
            new_src1,
            new_src2,
        ));
    }

    if raw == OpCode::IncRef as u8
        || raw == OpCode::DecRef as u8
        || raw == OpCode::Exit as u8
        || raw == OpCode::Throw as u8
        || raw == OpCode::ReturnValue as u8
    {
        let new_dst = if instruction.dst() == from_reg {
            to_reg
        } else {
            instruction.dst()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            new_dst,
            instruction.src1(),
            instruction.src2(),
        ));
    }

    if raw == OpCode::SetStatic as u8
        || raw == OpCode::SetStaticDirect as u8
        || raw == OpCode::SetStaticGlobal as u8
    {
        let new_dst = if instruction.dst() == from_reg {
            to_reg
        } else {
            instruction.dst()
        };
        return Some(Instruction::format_b(
            opcode,
            instruction.flags(),
            new_dst,
            instruction.imm32(),
        ));
    }

    if raw == OpCode::SetField as u8 {
        let new_dst = if instruction.dst() == from_reg {
            to_reg
        } else {
            instruction.dst()
        };
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            new_dst,
            new_src1,
            instruction.src2(),
        ));
    }

    if raw == OpCode::GetArray as u8 || (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
    {
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        let new_src2 = if instruction.src2() == from_reg {
            to_reg
        } else {
            instruction.src2()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            instruction.dst(),
            new_src1,
            new_src2,
        ));
    }

    if raw == OpCode::SetArray as u8 {
        let new_dst = if instruction.dst() == from_reg {
            to_reg
        } else {
            instruction.dst()
        };
        let new_src1 = if instruction.src1() == from_reg {
            to_reg
        } else {
            instruction.src1()
        };
        let new_src2 = if instruction.src2() == from_reg {
            to_reg
        } else {
            instruction.src2()
        };
        return Some(Instruction::format_a(
            opcode,
            instruction.flags(),
            new_dst,
            new_src1,
            new_src2,
        ));
    }

    if raw == OpCode::JumpIfTrue as u8 || raw == OpCode::JumpIfFalse as u8 {
        let new_cond = if instruction.cond_reg() == from_reg {
            to_reg
        } else {
            instruction.cond_reg()
        };
        return Some(Instruction::format_c(
            opcode,
            instruction.flags(),
            new_cond,
            instruction.jump_offset(),
        ));
    }

    if raw == OpCode::CallCallable as u8 {
        let base = instruction.arg_base();
        let count = instruction.arg_count() as u16;
        if from_reg >= base && from_reg < base + count {
            return None;
        }
        let new_target = if instruction.call_target() == from_reg {
            to_reg
        } else {
            instruction.call_target()
        };
        return Some(Instruction::format_call(
            opcode,
            instruction.call_dst(),
            new_target,
            instruction.arg_base(),
            instruction.arg_count(),
        ));
    }

    None
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
    let reads_new_dst = instruction_reads_register(instruction, new_dst);
    let pure_alu_or_load = (OpCode::AddI32 as u8..=OpCode::MulF64 as u8).contains(&raw)
        || (OpCode::NegI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || raw == OpCode::Not as u8
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || (OpCode::LoadI32 as u8..=OpCode::LoadConst as u8).contains(&raw);

    if reads_new_dst && !pure_alu_or_load {
        return false;
    }

    // Format A instructions that write to dst
    if (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::GetArray as u8
        || raw == OpCode::AllocObject as u8
        || raw == OpCode::AllocArray as u8
        || raw == OpCode::AllocTuple as u8
        || raw == OpCode::GetTuple as u8
        || raw == OpCode::TupleInstanceof as u8
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

    // Format A: superinstructions AddI32Imm, SubI32Imm, MulI32Imm, and ModI32Imm read src1
    if raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
    {
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
        || raw == OpCode::UnpackCaptures as u8
        || raw == OpCode::CastCheck as u8
        || raw == OpCode::Instanceof as u8
    {
        return instruction.src1() == target_reg;
    }

    // Format A: DecRef, IncRef, Exit, Throw, ReturnValue read dst
    if raw == OpCode::DecRef as u8
        || raw == OpCode::IncRef as u8
        || raw == OpCode::Exit as u8
        || raw == OpCode::Throw as u8
        || raw == OpCode::ReturnValue as u8
    {
        return instruction.dst() == target_reg;
    }

    // Format A: SetField reads dst (value) and src1 (object)
    if raw == OpCode::SetField as u8 {
        return instruction.dst() == target_reg || instruction.src1() == target_reg;
    }

    // Format A: GetArray and TupleInstanceof read src1 and src2
    if raw == OpCode::GetArray as u8 || raw == OpCode::TupleInstanceof as u8 {
        return instruction.src1() == target_reg || instruction.src2() == target_reg;
    }

    // Format A: SetArray reads dst (array), src1 (index), and src2 (value)
    if raw == OpCode::SetArray as u8 {
        return instruction.dst() == target_reg
            || instruction.src1() == target_reg
            || instruction.src2() == target_reg;
    }

    // Format A: Binary operations read src1 and src2
    if (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw) {
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
    if raw == OpCode::JumpIfTrue as u8
        || raw == OpCode::JumpIfFalse as u8
        || raw == OpCode::Switch as u8
    {
        return instruction.cond_reg() == target_reg;
    }

    // Format Call: windowed argument registers (plus call_target for CallCallable)
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::CallVirtual as u8
        || raw == OpCode::CallInterface as u8
        || raw == OpCode::CallCallable as u8
        || raw == OpCode::CallNativeDirect as u8
        || raw == OpCode::CallSymbol as u8
        || raw == OpCode::CallExtern as u8
        || raw == OpCode::PackClosure as u8
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
