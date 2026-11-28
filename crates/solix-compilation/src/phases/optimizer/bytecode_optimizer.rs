//! Post-emission bytecode peephole and superinstruction optimizer phase.
//!
//! Applies redundant move elimination, superinstruction fusion (`AddI32Imm`, `SubI32Imm`,
//! `JumpIf*I32`), and exact relative branch and exception table PC recalculation.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::CompilationDiagnostic;
use crate::phases::phase::CompilationPhase;
use smallvec::{SmallVec, smallvec};
use solix_common::bytecode::{CompiledFunction, Instruction, OpCode, SourceMapEntry};
use solix_common::diagnostics::EmissionError;
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
        let allow_arith_inline = context.options.inline_threshold > 0;
        let pool_ref = Some(&emission_artifacts.program.constant_pool);

        // Optimize each compiled function individually first so leaf constructors and arithmetic
        // leaf helpers shrink to minimal bodies.
        for compiled_function in &mut emission_artifacts.program.functions {
            optimize_function(
                compiled_function,
                &class_descriptors,
                &function_descriptors,
                pool_ref,
            )
            .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?;
        }

        // Inline small leaf constructors and pure leaf arithmetic functions across callers
        // (multiple rounds handle inheritance chains), then re-converge caller passes.
        let mut has_call_direct = false;
        let mut has_inlinable_leaf = false;
        for candidate in &emission_artifacts.program.functions {
            if !has_call_direct
                && candidate
                    .instructions
                    .iter()
                    .any(|inst| inst.raw_opcode() == OpCode::CallDirect as u8)
            {
                has_call_direct = true;
            }
            if !has_inlinable_leaf
                && candidate.exception_handlers.is_empty()
                && candidate.param_count > 0
                && !candidate.instructions.is_empty()
                && candidate.instructions.len() <= 8
                && (candidate.symbol.contains("::constructor")
                    || (allow_arith_inline
                        && candidate.reference_registers.is_empty()
                        && !candidate.symbol.starts_with('@')
                        && candidate.symbol != "main"
                        && !candidate.symbol.ends_with("::main")))
            {
                has_inlinable_leaf = true;
            }
            if has_call_direct && has_inlinable_leaf {
                break;
            }
        }
        if has_call_direct && has_inlinable_leaf {
            for _inline_round in 0..3 {
                let optimized_snapshot = emission_artifacts.program.functions.clone();
                let mut round_changed = false;
                for (func_idx, compiled_function) in
                    emission_artifacts.program.functions.iter_mut().enumerate()
                {
                    if inline_leaf_functions(
                        compiled_function,
                        func_idx,
                        &optimized_snapshot,
                        allow_arith_inline,
                    )
                    .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?
                    {
                        round_changed = true;
                        optimize_function(
                            compiled_function,
                            &class_descriptors,
                            &optimized_snapshot,
                            pool_ref,
                        )
                        .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?;
                    }
                }
                if !round_changed {
                    break;
                }
            }
        }

        for (func_idx, compiled_function) in
            emission_artifacts.program.functions.iter_mut().enumerate()
        {
            rotate_bottom_tested_loops(compiled_function)
                .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?;
            optimize_peephole_passes(compiled_function, true, pool_ref)
                .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?;
            hoist_loop_invariant_constants(compiled_function)
                .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?;
            eliminate_common_subexpressions(compiled_function);
            elide_borrowed_local_arc_ops(compiled_function);
            optimize_peephole_passes(compiled_function, true, pool_ref)
                .map_err(|emission_err| vec![CompilationDiagnostic::from(emission_err)])?;
            fuse_post_licm_add_load_band(compiled_function);
            propagate_entry_constants(compiled_function, true);
            transform_binary_additive_self_recursion(compiled_function, func_idx);
        }

        let func_has_refs: Vec<bool> = emission_artifacts
            .program
            .functions
            .iter()
            .map(|func| !func.reference_registers.is_empty())
            .collect();
        for compiled_function in &mut emission_artifacts.program.functions {
            mark_array_fill_loops(compiled_function);
            lower_tail_calls(compiled_function, &func_has_refs);
            specialize_primitive_field_accesses(compiled_function);
        }

        // Keep functions_by_source synchronized with optimized functions when emitting build artifacts
        if context.options.emit_export_metadata {
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
        }

        Ok(())
    }
}

/// Dense register bitset avoiding SipHash `HashSet<u16>` allocations for common `<= 128`-register frames.
#[derive(Clone, Default)]
struct RegisterBitSet {
    inline_words: [u64; 2],
    overflow_words: Vec<u64>,
}

impl RegisterBitSet {
    fn new(num_registers: usize) -> Self {
        let total_words = num_registers.div_ceil(64);
        let overflow_words = if total_words > 2 {
            vec![0; total_words - 2]
        } else {
            Vec::new()
        };
        Self {
            inline_words: [0; 2],
            overflow_words,
        }
    }

    fn insert(&mut self, reg: u16) -> bool {
        let reg_index = reg as usize;
        let word_idx = reg_index / 64;
        let bit_mask = 1u64 << (reg_index % 64);
        if word_idx < 2 {
            let was_set = (self.inline_words[word_idx] & bit_mask) != 0;
            self.inline_words[word_idx] |= bit_mask;
            !was_set
        } else {
            let overflow_idx = word_idx - 2;
            if overflow_idx >= self.overflow_words.len() {
                self.overflow_words.resize(overflow_idx + 1, 0);
            }
            let was_set = (self.overflow_words[overflow_idx] & bit_mask) != 0;
            self.overflow_words[overflow_idx] |= bit_mask;
            !was_set
        }
    }

    fn remove(&mut self, reg: u16) -> bool {
        let reg_index = reg as usize;
        let word_idx = reg_index / 64;
        let bit_mask = 1u64 << (reg_index % 64);
        if word_idx < 2 {
            let was_set = (self.inline_words[word_idx] & bit_mask) != 0;
            self.inline_words[word_idx] &= !bit_mask;
            was_set
        } else {
            let overflow_idx = word_idx - 2;
            if let Some(word_slot) = self.overflow_words.get_mut(overflow_idx) {
                let was_set = (*word_slot & bit_mask) != 0;
                *word_slot &= !bit_mask;
                was_set
            } else {
                false
            }
        }
    }

    #[inline]
    fn contains(&self, reg: &u16) -> bool {
        let reg_index = *reg as usize;
        let word_idx = reg_index / 64;
        let bit_mask = 1u64 << (reg_index % 64);
        if word_idx < 2 {
            (self.inline_words[word_idx] & bit_mask) != 0
        } else {
            self.overflow_words
                .get(word_idx - 2)
                .is_some_and(|word_val| (*word_val & bit_mask) != 0)
        }
    }

    fn clear(&mut self) {
        self.inline_words = [0; 2];
        self.overflow_words.fill(0);
    }

    fn is_empty(&self) -> bool {
        self.inline_words == [0; 2] && self.overflow_words.iter().all(|word_val| *word_val == 0)
    }
}

/// Intermediate representation of an instruction during optimization passes.
#[derive(Clone)]
struct IntermediateInstruction {
    instruction: Instruction,
    original_pc: usize,
    covered_original_pcs: SmallVec<[usize; 2]>,
    jump_target_old_pc: Option<usize>,
}

/// Optimizes a single compiled function by running scalar replacement, ARC elision,
/// closure specialization, dead closure elimination, constant propagation, CSE,
/// redundant ARC/null field elimination, convergent peephole fusion with liveness analysis,
/// and loop-invariant constant/expression hoisting.
#[inline(always)]
fn is_unsigned_comparison_opcode(raw_opcode_value: u8) -> bool {
    (raw_opcode_value >= OpCode::LtU32 as u8 && raw_opcode_value <= OpCode::GtU64 as u8)
        || raw_opcode_value == OpCode::GteU64 as u8
}

/// Optimizes a single compiled function by running scalar replacement, ARC elision,
/// closure specialization, dead closure elimination, constant propagation, CSE,
/// redundant ARC/null field elimination, convergent peephole fusion with liveness analysis,
/// and loop-invariant constant/expression hoisting.
fn optimize_function(
    function: &mut CompiledFunction,
    classes: &[solix_common::bytecode::CompiledClass],
    all_functions: &[CompiledFunction],
    constant_pool: Option<&solix_common::bytecode::ConstantPool>,
) -> Result<(), EmissionError> {
    super::escape_analysis::apply_scalar_replacement(function, classes, all_functions);
    super::escape_analysis::apply_arc_elision(function);
    elide_redundant_arc_ops(function);
    optimize_peephole_passes(function, false, constant_pool)?;
    elide_borrowed_local_arc_ops(function);
    elide_fresh_null_set_fields(function);
    propagate_entry_constants(function, false);
    optimize_peephole_passes(function, false, constant_pool)?;
    hoist_loop_invariant_constants(function)?;
    eliminate_common_subexpressions(function);
    if eliminate_dead_pack_closures(function) {
        elide_redundant_arc_ops(function);
    }
    for _ in 0..4 {
        if !specialize_diamond_pack_closure_calls(function)? {
            break;
        }
    }
    optimize_peephole_passes(function, false, constant_pool)
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
            | OpCode::DivU32
            | OpCode::DivU64
            | OpCode::ModU32
            | OpCode::ModU64
            | OpCode::Lshr
            | OpCode::LtU32
            | OpCode::LteU32
            | OpCode::GtU32
            | OpCode::GteU32
            | OpCode::LtU64
            | OpCode::LteU64
            | OpCode::GtU64
            | OpCode::GteU64
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
            | OpCode::AddI64Imm
            | OpCode::AddF64Imm
            | OpCode::SubF64Imm
            | OpCode::MulF64Imm
            | OpCode::ForLoopI32
            | OpCode::AddBandI32
            | OpCode::MulAddF64
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
        if raw == OpCode::UnpackCaptures as u8 {
            let cap_base = inst.arg_base();
            let cap_count = inst.arg_count() as u16;
            for cap_reg in cap_base..cap_base + cap_count {
                let cap_idx = cap_reg as usize;
                if cap_idx < has_any_write.len() {
                    has_any_write[cap_idx] = true;
                    has_non_immortal_direct_write[cap_idx] = true;
                    scan_block_non_heap.remove(&cap_reg);
                }
            }
        } else if let Some(dst_reg) = instruction_write_dst(inst) {
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
        let is_catch_reg = function
            .exception_handlers
            .iter()
            .any(|handler| handler.catch_reg == reg);
        let idx = reg as usize;
        if !is_catch_reg
            && has_any_write[idx]
            && !has_non_immortal_direct_write[idx]
            && !mov_sources.contains_key(&reg)
        {
            always_immortal_regs.insert(reg);
        }
    }
    loop {
        let mut added = false;
        for reg in function.param_count..function.num_registers {
            let is_catch_reg = function
                .exception_handlers
                .iter()
                .any(|handler| handler.catch_reg == reg);
            let idx = reg as usize;
            if is_catch_reg
                || always_immortal_regs.contains(&reg)
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
    for (pc_idx, inst_slot) in function
        .instructions
        .iter_mut()
        .enumerate()
        .take(inst_count)
    {
        if critical_targets.contains(&pc_idx) {
            block_immortals.clear();
        }

        let inst = *inst_slot;
        let raw = inst.raw_opcode();

        if (raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8)
            && (block_immortals.contains(&inst.dst()) || always_immortal_regs.contains(&inst.dst()))
        {
            *inst_slot = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
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
        } else if raw == OpCode::UnpackCaptures as u8 {
            let cap_base = inst.arg_base();
            let cap_count = inst.arg_count() as u16;
            for cap_reg in cap_base..cap_base + cap_count {
                block_immortals.remove(&cap_reg);
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
            for scan_inst in function
                .instructions
                .iter()
                .take(inst_count)
                .skip(dec_pc + 1)
            {
                let scan_raw = scan_inst.raw_opcode();
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
        let mut borrowed_ref_writes = 0usize;
        let mut writes_ok = true;
        let mut escapes = false;
        let mut has_inc_or_dec = false;
        let mut used_owners: Vec<u16> = Vec::new();
        let mut max_non_epilogue_use: usize = 0;

        for (pc_idx, inst) in function.instructions.iter().enumerate() {
            let raw = inst.raw_opcode();
            if (raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8)
                && inst.dst() == candidate_reg
            {
                has_inc_or_dec = true;
            }
            if instruction_write_dst(inst) == Some(candidate_reg) {
                writes_count += 1;
                if (raw == OpCode::Mov as u8 && stable_owners.contains_key(&inst.src1()))
                    || (raw == OpCode::GetArray as u8
                        && immutable_local_arrays.contains(&inst.src1()))
                {
                    borrowed_ref_writes += 1;
                    used_owners.push(inst.src1());
                } else if is_guaranteed_non_heap_write(*inst, &immutable_local_arrays)
                    && !has_inc_or_dec
                {
                    // Safe primitive overwrite (e.g. reused register for integer literal after loop)
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
            && borrowed_ref_writes > 0
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
    let mut window_elided_regs: HashSet<u16> = HashSet::new();
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
            for (fwd_pc, &fwd_inst) in function
                .instructions
                .iter()
                .enumerate()
                .take(search_end)
                .skip(inc_pc + 1)
            {
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
                window_elided_regs.insert(target_reg);
            }
        }
    }

    let mut unowned_borrowed_regs = borrowed_aliases.clone();
    for candidate_reg in window_elided_regs {
        if candidate_reg < function.param_count
            || function
                .exception_handlers
                .iter()
                .any(|handler| handler.catch_reg == candidate_reg)
        {
            continue;
        }
        let still_owns_ref = function.instructions.iter().any(|inst| {
            let raw = inst.raw_opcode();
            if (raw == OpCode::IncRef as u8
                || raw == OpCode::DecRef as u8
                || raw == OpCode::ReturnValue as u8
                || raw == OpCode::Throw as u8)
                && inst.dst() == candidate_reg
            {
                return true;
            }
            if instruction_writes_register(inst, candidate_reg) {
                let is_borrowed_write = (raw == OpCode::GetArray as u8
                    && immutable_local_arrays.contains(&inst.src1()))
                    || (raw == OpCode::Mov as u8
                        && (stable_owners.contains_key(&inst.src1())
                            || borrowed_aliases.contains(&inst.src1())))
                    || is_guaranteed_non_heap_write(*inst, &immutable_local_arrays);
                return !is_borrowed_write;
            }
            false
        });
        if !still_owns_ref {
            unowned_borrowed_regs.insert(candidate_reg);
        }
    }

    if !unowned_borrowed_regs.is_empty() {
        function
            .reference_registers
            .retain(|reg| !unowned_borrowed_regs.contains(reg));
        for handler in &mut function.exception_handlers {
            handler
                .try_ref_regs
                .retain(|reg| !unowned_borrowed_regs.contains(reg));
        }
    }
}

/// Eliminates `PackClosure` allocations whose result register is never used for anything except
/// `IncRef` / `DecRef` (e.g. after closure call inlining in `AstInlinerPhase`).
fn eliminate_dead_pack_closures(function: &mut CompiledFunction) -> bool {
    let inst_count = function.instructions.len();
    if inst_count < 2 {
        return false;
    }

    let num_regs = function.num_registers as usize;
    let mut write_counts = vec![0u32; num_regs];
    let mut non_arc_read_counts = vec![0u32; num_regs];
    for inst in &function.instructions {
        let raw = inst.raw_opcode();
        if raw == OpCode::UnpackCaptures as u8 {
            let base = inst.arg_base();
            let count = inst.arg_count() as u16;
            for reg in base..base + count {
                if let Some(slot) = write_counts.get_mut(reg as usize) {
                    *slot += 1;
                }
            }
        } else if let Some(dst) = instruction_write_dst(inst)
            && let Some(slot) = write_counts.get_mut(dst as usize)
        {
            *slot += 1;
        }
        if raw != OpCode::IncRef as u8 && raw != OpCode::DecRef as u8 {
            for reg in 0..function.num_registers {
                if instruction_reads_register(inst, reg)
                    && let Some(slot) = non_arc_read_counts.get_mut(reg as usize)
                {
                    *slot += 1;
                }
            }
        }
    }

    let mut dead_closure_regs = RegisterBitSet::new(num_regs);
    for inst in &function.instructions {
        if inst.raw_opcode() == OpCode::PackClosure as u8 {
            let closure_reg = inst.call_dst();
            let is_catch_reg = function
                .exception_handlers
                .iter()
                .any(|handler| handler.catch_reg == closure_reg);
            if !is_catch_reg
                && closure_reg >= function.param_count
                && write_counts.get(closure_reg as usize).copied() == Some(1)
                && non_arc_read_counts
                    .get(closure_reg as usize)
                    .copied()
                    .unwrap_or(0)
                    == 0
            {
                dead_closure_regs.insert(closure_reg);
            }
        }
    }

    if dead_closure_regs.is_empty() {
        return false;
    }

    let mut removed_any = false;
    for inst in &mut function.instructions {
        let raw = inst.raw_opcode();
        if (raw == OpCode::PackClosure as u8 && dead_closure_regs.contains(&inst.call_dst()))
            || ((raw == OpCode::IncRef as u8 || raw == OpCode::DecRef as u8)
                && dead_closure_regs.contains(&inst.dst()))
        {
            *inst = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            removed_any = true;
        }
    }

    if removed_any {
        function
            .reference_registers
            .retain(|reg| !dead_closure_regs.contains(reg));
        for handler in &mut function.exception_handlers {
            handler
                .try_ref_regs
                .retain(|reg| !dead_closure_regs.contains(reg));
        }
    }
    removed_any
}

/// Returns `true` if `inst` is a pure register-only argument setup operation safe to duplicate
/// across arms of a zero-capture `PackClosure -> CallCallable` diamond.
fn is_pure_diamond_setup_inst(inst: Instruction) -> bool {
    let Some(opc) = OpCode::from_raw_u8(inst.raw_opcode()) else {
        return false;
    };
    match opc {
        OpCode::Nop
        | OpCode::Mov
        | OpCode::LoadI32
        | OpCode::LoadI64
        | OpCode::LoadF64
        | OpCode::LoadBool
        | OpCode::LoadNull
        | OpCode::AddI32
        | OpCode::AddI64
        | OpCode::AddF64
        | OpCode::SubI32
        | OpCode::SubI64
        | OpCode::SubF64
        | OpCode::MulI32
        | OpCode::MulI64
        | OpCode::MulF64
        | OpCode::Band
        | OpCode::Bor
        | OpCode::Bxor
        | OpCode::Bnot
        | OpCode::Shl
        | OpCode::Shr
        | OpCode::Lshr
        | OpCode::LtU32
        | OpCode::LteU32
        | OpCode::GtU32
        | OpCode::GteU32
        | OpCode::LtU64
        | OpCode::LteU64
        | OpCode::GtU64
        | OpCode::GteU64
        | OpCode::NegI32
        | OpCode::NegI64
        | OpCode::NegF64
        | OpCode::Not
        | OpCode::AddI32Imm
        | OpCode::SubI32Imm
        | OpCode::MulI32Imm
        | OpCode::AddI64Imm
        | OpCode::AddF64Imm
        | OpCode::SubF64Imm
        | OpCode::MulF64Imm
        | OpCode::ConvIToF
        | OpCode::ConvFToI => true,
        OpCode::ModI32Imm => (inst.src2() as i16) > 0,
        _ => false,
    }
}

/// Specializes branch-diamond zero-capture `PackClosure` assignments feeding a single
/// `CallCallable` at the diamond merge block into direct `CallDirect` calls inside each arm.
fn specialize_diamond_pack_closure_calls(
    function: &mut CompiledFunction,
) -> Result<bool, EmissionError> {
    if !function.exception_handlers.is_empty() {
        return Ok(false);
    }
    let inst_count = function.instructions.len();
    if inst_count < 5 {
        return Ok(false);
    }

    let critical_targets = collect_critical_targets(function);
    let mut jump_targets: Vec<Option<usize>> = Vec::with_capacity(inst_count);
    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        jump_targets.push(decode_jump_target(*inst, pc_idx, inst_count));
    }

    for (call_pc, call_inst) in function.instructions.iter().enumerate() {
        if call_inst.raw_opcode() != OpCode::CallCallable as u8 {
            continue;
        }
        let r_callable = call_inst.call_target();
        let arg_base = call_inst.arg_base();
        let arg_count = call_inst.arg_count();
        let call_dst = call_inst.call_dst();
        if r_callable >= arg_base && r_callable < arg_base + arg_count as u16 {
            continue;
        }

        // Require `r_callable` not to be read anywhere else in the function except at `call_pc`.
        let only_read_at_call = function
            .instructions
            .iter()
            .enumerate()
            .all(|(pc, inst)| pc == call_pc || !instruction_reads_register(inst, r_callable));
        if !only_read_at_call {
            continue;
        }

        // Scan backward at most 6 instructions to locate the diamond merge point.
        let min_merge = call_pc.saturating_sub(6);
        let mut merge_pc_opt = None;
        for candidate_merge in (min_merge..=call_pc).rev() {
            let valid_setup = (candidate_merge..call_pc).all(|pc_idx| {
                let inst = function.instructions[pc_idx];
                is_pure_diamond_setup_inst(inst)
                    && !instruction_writes_register(&inst, r_callable)
                    && (pc_idx == candidate_merge || !critical_targets.contains(&pc_idx))
            });
            if !valid_setup {
                break;
            }
            if critical_targets.contains(&candidate_merge) {
                merge_pc_opt = Some(candidate_merge);
                break;
            }
        }
        let Some(merge_pc) = merge_pc_opt else {
            continue;
        };
        if merge_pc == 0 {
            continue;
        }

        let fall_pack_pc = merge_pc - 1;
        let fall_pack = function.instructions[fall_pack_pc];
        if fall_pack.raw_opcode() != OpCode::PackClosure as u8
            || fall_pack.call_dst() != r_callable
            || fall_pack.arg_count() != 0
        {
            continue;
        }

        let incoming_jumps: Vec<usize> = jump_targets
            .iter()
            .enumerate()
            .filter_map(|(pc_idx, target_opt)| {
                if *target_opt == Some(merge_pc) {
                    Some(pc_idx)
                } else {
                    None
                }
            })
            .collect();
        if incoming_jumps.is_empty() {
            continue;
        }

        let mut jump_arm_map: HashMap<usize, u16> = HashMap::new();
        let mut all_arms_valid = true;
        for &jump_pc in &incoming_jumps {
            let jump_inst = function.instructions[jump_pc];
            if jump_inst.raw_opcode() != OpCode::Jump as u8
                || jump_pc == 0
                || critical_targets.contains(&jump_pc)
                || jump_pc + 1 == merge_pc
            {
                all_arms_valid = false;
                break;
            }
            let pack_pc = jump_pc - 1;
            let pack_inst = function.instructions[pack_pc];
            if pack_inst.raw_opcode() != OpCode::PackClosure as u8
                || pack_inst.call_dst() != r_callable
                || pack_inst.arg_count() != 0
            {
                all_arms_valid = false;
                break;
            }
            jump_arm_map.insert(pack_pc, pack_inst.call_target());
        }
        if !all_arms_valid {
            continue;
        }

        let setup_slice: Vec<Instruction> = function.instructions[merge_pc..call_pc].to_vec();
        let after_call_old_pc = call_pc + 1;
        let mut rebuilt: Vec<(Instruction, Option<usize>)> = Vec::with_capacity(inst_count + 16);
        let mut old_pc_to_new_pc = vec![u32::MAX; inst_count + 1];

        let mut cursor = 0;
        while cursor < inst_count {
            old_pc_to_new_pc[cursor] = rebuilt.len() as u32;
            if let Some(&fn_target) = jump_arm_map.get(&cursor) {
                for &setup_inst in &setup_slice {
                    rebuilt.push((setup_inst, None));
                }
                rebuilt.push((
                    Instruction::format_call(
                        OpCode::CallDirect,
                        call_dst,
                        fn_target,
                        arg_base,
                        arg_count,
                    ),
                    None,
                ));
                let jump_pc = cursor + 1;
                old_pc_to_new_pc[jump_pc] = rebuilt.len() as u32;
                rebuilt.push((
                    Instruction::format_c(OpCode::Jump, 0, 0, 0),
                    Some(after_call_old_pc),
                ));
                cursor += 2;
                continue;
            }
            if cursor == fall_pack_pc {
                for &setup_inst in &setup_slice {
                    rebuilt.push((setup_inst, None));
                }
                rebuilt.push((
                    Instruction::format_call(
                        OpCode::CallDirect,
                        call_dst,
                        fall_pack.call_target(),
                        arg_base,
                        arg_count,
                    ),
                    None,
                ));
                old_pc_to_new_pc[merge_pc..=call_pc].fill(rebuilt.len() as u32);
                cursor = after_call_old_pc;
                continue;
            }
            rebuilt.push((function.instructions[cursor], jump_targets[cursor]));
            cursor += 1;
        }
        old_pc_to_new_pc[inst_count] = rebuilt.len() as u32;

        let mut final_instructions = Vec::with_capacity(rebuilt.len());
        for (new_pc, (inst, target_opt)) in rebuilt.into_iter().enumerate() {
            let mut final_inst = inst;
            if let Some(target_old_pc) = target_opt {
                let target_new_pc = old_pc_to_new_pc
                    .get(target_old_pc)
                    .copied()
                    .unwrap_or(old_pc_to_new_pc[inst_count]);
                final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc)?;
            }
            final_instructions.push(final_inst);
        }
        apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
        return Ok(true);
    }

    Ok(false)
}

/// Propagates single-definition entry-block `LoadI32` constants into binary ALU instructions
/// (and remaining standalone `JumpIf*I32` comparisons when `fold_branches` is `true`).
fn propagate_entry_constants(function: &mut CompiledFunction, fold_branches: bool) -> bool {
    let inst_count = function.instructions.len();
    if inst_count < 2 {
        return false;
    }

    let critical_targets = collect_critical_targets(function);
    let mut write_counts = vec![0u32; function.num_registers as usize];
    for inst in &function.instructions {
        if inst.raw_opcode() == OpCode::UnpackCaptures as u8 {
            let base = inst.arg_base();
            let count = inst.arg_count() as u16;
            for reg in base..base + count {
                if let Some(slot) = write_counts.get_mut(reg as usize) {
                    *slot += 1;
                }
            }
        } else if let Some(dst) = instruction_write_dst(inst)
            && let Some(slot) = write_counts.get_mut(dst as usize)
        {
            *slot += 1;
        }
    }

    // Collect single-definition `LoadI32` constants in the entry basic block before any branch.
    let mut entry_consts: HashMap<u16, i16> = HashMap::new();
    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        if pc_idx > 0 && critical_targets.contains(&pc_idx) {
            break;
        }
        let raw = inst.raw_opcode();
        if is_control_flow_terminator(raw)
            || is_conditional_branch(raw)
            || raw == OpCode::Jump as u8
            || raw == OpCode::Switch as u8
        {
            break;
        }
        if raw == OpCode::LoadI32 as u8 {
            let dst = inst.dst();
            let imm = inst.imm32_signed();
            if dst >= function.param_count
                && write_counts.get(dst as usize).copied() == Some(1)
                && let Ok(imm16) = i16::try_from(imm)
            {
                entry_consts.insert(dst, imm16);
            }
        }
    }

    if entry_consts.is_empty() {
        return false;
    }

    let mut changed = false;
    for inst in &mut function.instructions {
        let raw = inst.raw_opcode();
        if raw == OpCode::AddI32 as u8 {
            let dst = inst.dst();
            let s1 = inst.src1();
            let s2 = inst.src2();
            if let Some(&imm16) = entry_consts.get(&s2)
                && s1 != s2
            {
                *inst = Instruction::format_a(OpCode::AddI32Imm, 0, dst, s1, imm16 as u16);
                changed = true;
            } else if let Some(&imm16) = entry_consts.get(&s1)
                && s1 != s2
            {
                *inst = Instruction::format_a(OpCode::AddI32Imm, 0, dst, s2, imm16 as u16);
                changed = true;
            }
        } else if raw == OpCode::SubI32 as u8 {
            let dst = inst.dst();
            let s1 = inst.src1();
            let s2 = inst.src2();
            if let Some(&imm16) = entry_consts.get(&s2)
                && s1 != s2
            {
                *inst = Instruction::format_a(OpCode::SubI32Imm, 0, dst, s1, imm16 as u16);
                changed = true;
            }
        } else if raw == OpCode::MulI32 as u8 {
            let dst = inst.dst();
            let s1 = inst.src1();
            let s2 = inst.src2();
            if let Some(&imm16) = entry_consts.get(&s2)
                && s1 != s2
            {
                *inst = Instruction::format_a(OpCode::MulI32Imm, 0, dst, s1, imm16 as u16);
                changed = true;
            } else if let Some(&imm16) = entry_consts.get(&s1)
                && s1 != s2
            {
                *inst = Instruction::format_a(OpCode::MulI32Imm, 0, dst, s2, imm16 as u16);
                changed = true;
            }
        } else if raw == OpCode::ModI32 as u8 {
            let dst = inst.dst();
            let s1 = inst.src1();
            let s2 = inst.src2();
            if let Some(&imm16) = entry_consts.get(&s2)
                && s1 != s2
                && imm16 > 0
            {
                *inst = Instruction::format_a(OpCode::ModI32Imm, 0, dst, s1, imm16 as u16);
                changed = true;
            }
        } else if fold_branches
            && (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
        {
            let reg_a = inst.dst();
            let reg_b = inst.src1();
            let offset = inst.src2();
            if let Some(&imm16) = entry_consts.get(&reg_b)
                && reg_a != reg_b
            {
                let flag = jump_i32_to_imm_flag(raw, false);
                *inst =
                    Instruction::format_a(OpCode::JumpIfI32Imm, flag, reg_a, imm16 as u16, offset);
                changed = true;
            } else if let Some(&imm16) = entry_consts.get(&reg_a)
                && reg_a != reg_b
            {
                let flag = jump_i32_to_imm_flag(raw, true);
                *inst =
                    Instruction::format_a(OpCode::JumpIfI32Imm, flag, reg_b, imm16 as u16, offset);
                changed = true;
            }
        }
    }

    changed
}

fn jump_i32_to_imm_flag(raw_opcode: u8, swapped: bool) -> u8 {
    match raw_opcode {
        raw_val if raw_val == OpCode::JumpIfEqI32 as u8 => 0,
        raw_val if raw_val == OpCode::JumpIfNeqI32 as u8 => 1,
        raw_val if raw_val == OpCode::JumpIfLtI32 as u8 => {
            if swapped {
                4
            } else {
                2
            }
        }
        raw_val if raw_val == OpCode::JumpIfLteI32 as u8 => {
            if swapped {
                5
            } else {
                3
            }
        }
        raw_val if raw_val == OpCode::JumpIfGtI32 as u8 => {
            if swapped {
                2
            } else {
                4
            }
        }
        _ => {
            if swapped {
                3
            } else {
                5
            }
        }
    }
}

/// Symbolic expression key used by extended-basic-block Local Common Subexpression Elimination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CseExprKey {
    BinaryComm { raw: u8, v_low: u32, v_high: u32 },
    BinaryOrd { raw: u8, v_lhs: u32, v_rhs: u32 },
    ImmOp { raw: u8, v_src: u32, imm: u16 },
}

fn cse_expr_for_instruction(
    inst: Instruction,
    reg_to_val: &mut HashMap<u16, u32>,
    val_holders: &mut HashMap<u32, Vec<u16>>,
    next_val: &mut u32,
) -> Option<CseExprKey> {
    let mut get_val = |reg: u16| -> u32 {
        if let Some(&vid) = reg_to_val.get(&reg) {
            vid
        } else {
            let vid = *next_val;
            *next_val += 1;
            reg_to_val.insert(reg, vid);
            val_holders.entry(vid).or_default().push(reg);
            vid
        }
    };

    let raw = inst.raw_opcode();
    if raw == OpCode::AddI32 as u8
        || raw == OpCode::MulI32 as u8
        || raw == OpCode::AddF64 as u8
        || raw == OpCode::MulF64 as u8
        || raw == OpCode::Band as u8
        || raw == OpCode::Bor as u8
        || raw == OpCode::Bxor as u8
    {
        let va = get_val(inst.src1());
        let vb = get_val(inst.src2());
        return Some(CseExprKey::BinaryComm {
            raw,
            v_low: va.min(vb),
            v_high: va.max(vb),
        });
    }
    if raw == OpCode::SubI32 as u8
        || raw == OpCode::SubF64 as u8
        || raw == OpCode::DivF64 as u8
        || raw == OpCode::Shl as u8
        || raw == OpCode::Shr as u8
    {
        let va = get_val(inst.src1());
        let vb = get_val(inst.src2());
        return Some(CseExprKey::BinaryOrd {
            raw,
            v_lhs: va,
            v_rhs: vb,
        });
    }
    if raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || (raw == OpCode::ModI32Imm as u8 && (inst.src2() as i16) > 0)
    {
        let va = get_val(inst.src1());
        return Some(CseExprKey::ImmOp {
            raw,
            v_src: va,
            imm: inst.src2(),
        });
    }
    None
}

fn invalidate_cse_reg(
    reg: u16,
    reg_to_val: &mut HashMap<u16, u32>,
    val_holders: &mut HashMap<u32, Vec<u16>>,
) {
    if let Some(old_vid) = reg_to_val.remove(&reg)
        && let Some(holders) = val_holders.get_mut(&old_vid)
    {
        holders.retain(|&holder_reg| holder_reg != reg);
    }
}

/// Eliminates redundant pure computations across Extended Basic Blocks (including fallthrough
/// segments past exit guards) while strictly invalidating call-clobbered windows (`>= arg_base`).
fn eliminate_common_subexpressions(function: &mut CompiledFunction) -> bool {
    let inst_count = function.instructions.len();
    if inst_count < 2 {
        return false;
    }

    let critical_targets = collect_critical_targets(function);
    let mut jump_targets: Vec<Option<usize>> = Vec::with_capacity(inst_count);
    for (pc_idx, inst) in function.instructions.iter().enumerate() {
        jump_targets.push(decode_jump_target(*inst, pc_idx, inst_count));
    }

    let mut reg_to_val: HashMap<u16, u32> = HashMap::new();
    let mut val_holders: HashMap<u32, Vec<u16>> = HashMap::new();
    let mut expr_to_val: HashMap<CseExprKey, u32> = HashMap::new();
    let mut next_val: u32 = 1;
    let mut duplicate_expr_pcs: Vec<(usize, u16)> = Vec::new();
    let mut any_changed = false;

    for (pc_idx, inst_slot) in function
        .instructions
        .iter_mut()
        .enumerate()
        .take(inst_count)
    {
        if critical_targets.contains(&pc_idx) {
            reg_to_val.clear();
            val_holders.clear();
            expr_to_val.clear();
        }

        let inst = *inst_slot;
        let raw = inst.raw_opcode();

        if raw == OpCode::CallDirect as u8
            || raw == OpCode::CallVirtual as u8
            || raw == OpCode::CallInterface as u8
            || raw == OpCode::CallCallable as u8
            || raw == OpCode::CallNativeDirect as u8
            || raw == OpCode::CallSymbol as u8
            || raw == OpCode::CallExtern as u8
        {
            invalidate_cse_reg(inst.call_dst(), &mut reg_to_val, &mut val_holders);
            if inst.arg_count() > 0 {
                let clobber_base = inst.arg_base();
                let clobbered: Vec<u16> = reg_to_val
                    .keys()
                    .copied()
                    .filter(|&reg| reg >= clobber_base)
                    .collect();
                for reg in clobbered {
                    invalidate_cse_reg(reg, &mut reg_to_val, &mut val_holders);
                }
            }
            continue;
        }

        if raw == OpCode::Mov as u8 {
            let dst = inst.dst();
            let src = inst.src1();
            let src_vid = if let Some(&vid) = reg_to_val.get(&src) {
                vid
            } else {
                let vid = next_val;
                next_val += 1;
                reg_to_val.insert(src, vid);
                val_holders.entry(vid).or_default().push(src);
                vid
            };
            if reg_to_val.get(&dst) == Some(&src_vid) {
                *inst_slot = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                any_changed = true;
            } else {
                invalidate_cse_reg(dst, &mut reg_to_val, &mut val_holders);
                reg_to_val.insert(dst, src_vid);
                val_holders.entry(src_vid).or_default().push(dst);
            }
            continue;
        }

        if let Some(expr_key) =
            cse_expr_for_instruction(inst, &mut reg_to_val, &mut val_holders, &mut next_val)
        {
            let dst = inst.dst();
            if let Some(&existing_vid) = expr_to_val.get(&expr_key) {
                let live_holder = val_holders.get(&existing_vid).and_then(|holders| {
                    holders
                        .iter()
                        .copied()
                        .find(|&candidate| reg_to_val.get(&candidate) == Some(&existing_vid))
                });
                if let Some(holder_reg) = live_holder {
                    if holder_reg == dst {
                        *inst_slot = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                    } else {
                        invalidate_cse_reg(dst, &mut reg_to_val, &mut val_holders);
                        reg_to_val.insert(dst, existing_vid);
                        val_holders.entry(existing_vid).or_default().push(dst);
                        *inst_slot = Instruction::format_a(OpCode::Mov, 0, dst, holder_reg, 0);
                    }
                    any_changed = true;
                } else {
                    invalidate_cse_reg(dst, &mut reg_to_val, &mut val_holders);
                    reg_to_val.insert(dst, existing_vid);
                    val_holders.entry(existing_vid).or_default().push(dst);
                    duplicate_expr_pcs.push((pc_idx, dst));
                }
            } else {
                invalidate_cse_reg(dst, &mut reg_to_val, &mut val_holders);
                let vid = next_val;
                next_val += 1;
                expr_to_val.insert(expr_key, vid);
                reg_to_val.insert(dst, vid);
                val_holders.entry(vid).or_default().push(dst);
            }
            continue;
        }

        if raw == OpCode::UnpackCaptures as u8 {
            let base = inst.arg_base();
            let count = inst.arg_count() as u16;
            for reg in base..base + count {
                invalidate_cse_reg(reg, &mut reg_to_val, &mut val_holders);
            }
        } else if let Some(dst) = instruction_write_dst(&inst) {
            invalidate_cse_reg(dst, &mut reg_to_val, &mut val_holders);
        }

        if is_control_flow_terminator(raw)
            || raw == OpCode::Jump as u8
            || raw == OpCode::Switch as u8
        {
            reg_to_val.clear();
            val_holders.clear();
            expr_to_val.clear();
        }
    }

    // Remove intermediate duplicate pure calculations whose downstream consumer became redundant.
    for (dup_pc, dst_reg) in duplicate_expr_pcs.into_iter().rev() {
        if is_dead_after_use_raw(function, &jump_targets, &critical_targets, dup_pc, dst_reg) {
            function.instructions[dup_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            any_changed = true;
        }
    }

    any_changed
}
/// Eliminates redundant `SetField null` on freshly `AllocObject`ed instances or at the start of leaf constructors.
fn elide_fresh_null_set_fields(function: &mut CompiledFunction) {
    let inst_count = function.instructions.len();
    if inst_count < 2 {
        return;
    }
    let critical_targets = collect_critical_targets(function);
    let is_ctor = function.symbol.contains("::constructor");
    let mut fresh_zero_regs = RegisterBitSet::new(function.num_registers as usize);
    let mut block_null_regs = RegisterBitSet::new(function.num_registers as usize);
    if is_ctor {
        fresh_zero_regs.insert(0);
    }

    for (pc_idx, inst_slot) in function
        .instructions
        .iter_mut()
        .enumerate()
        .take(inst_count)
    {
        if critical_targets.contains(&pc_idx) {
            fresh_zero_regs.clear();
            block_null_regs.clear();
        }
        let inst = *inst_slot;
        let raw = inst.raw_opcode();

        if raw == OpCode::SetField as u8
            && fresh_zero_regs.contains(&inst.src1())
            && block_null_regs.contains(&inst.dst())
        {
            *inst_slot = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            continue;
        }

        if raw == OpCode::AllocObject as u8 {
            fresh_zero_regs.insert(inst.dst());
            block_null_regs.remove(inst.dst());
            continue;
        }

        if raw == OpCode::LoadNull as u8 {
            block_null_regs.insert(inst.dst());
            fresh_zero_regs.remove(inst.dst());
            continue;
        }

        if let Some(written) = instruction_write_dst(&inst) {
            block_null_regs.remove(written);
            fresh_zero_regs.remove(written);
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

/// Checks if an instruction is an eligible pure non-trapping leaf body operation.
fn is_pure_leaf_arith_inst(inst: Instruction, param_count: u16) -> bool {
    let raw = inst.raw_opcode();
    let Some(opc) = OpCode::from_raw_u8(raw) else {
        return false;
    };
    let written_ok = instruction_write_dst(&inst).is_none_or(|dst| dst >= param_count);
    if !written_ok {
        return false;
    }
    matches!(
        opc,
        OpCode::Nop
            | OpCode::Mov
            | OpCode::LoadI32
            | OpCode::LoadI64
            | OpCode::LoadF64
            | OpCode::LoadBool
            | OpCode::LoadNull
            | OpCode::AddI32
            | OpCode::AddI64
            | OpCode::AddF64
            | OpCode::SubI32
            | OpCode::SubI64
            | OpCode::SubF64
            | OpCode::MulI32
            | OpCode::MulI64
            | OpCode::MulF64
            | OpCode::Band
            | OpCode::Bor
            | OpCode::Bxor
            | OpCode::Bnot
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::NegI32
            | OpCode::NegI64
            | OpCode::NegF64
            | OpCode::EqI32
            | OpCode::EqI64
            | OpCode::EqF64
            | OpCode::EqBool
            | OpCode::NeqI32
            | OpCode::NeqI64
            | OpCode::NeqF64
            | OpCode::NeqBool
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
            | OpCode::DivU32
            | OpCode::DivU64
            | OpCode::ModU32
            | OpCode::ModU64
            | OpCode::Lshr
            | OpCode::LtU32
            | OpCode::LteU32
            | OpCode::GtU32
            | OpCode::GteU32
            | OpCode::LtU64
            | OpCode::LteU64
            | OpCode::GtU64
            | OpCode::GteU64
            | OpCode::Not
            | OpCode::AddI32Imm
            | OpCode::SubI32Imm
            | OpCode::MulI32Imm
            | OpCode::AddI64Imm
            | OpCode::AddF64Imm
            | OpCode::SubF64Imm
            | OpCode::MulF64Imm
            | OpCode::AddBandI32
            | OpCode::MulAddF64
            | OpCode::ConvIToF
            | OpCode::ConvFToI
    )
}

/// Remaps registers in an inlined leaf constructor or pure arithmetic leaf function instruction.
fn remap_leaf_instruction(inst: Instruction, remap_reg: &dyn Fn(u16) -> u16) -> Instruction {
    let raw = inst.raw_opcode();
    if (raw == OpCode::SetField as u8
        || raw == OpCode::SetFieldI32 as u8
        || raw == OpCode::SetFieldF64 as u8)
        && let Some(opcode) = OpCode::from_raw_u8(raw)
    {
        return Instruction::format_a(
            opcode,
            inst.flags(),
            remap_reg(inst.dst()),
            remap_reg(inst.src1()),
            inst.src2(),
        );
    }
    if (raw == OpCode::Mov as u8
        || raw == OpCode::Not as u8
        || raw == OpCode::Bnot as u8
        || raw == OpCode::NegI32 as u8
        || raw == OpCode::NegI64 as u8
        || raw == OpCode::NegF64 as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8)
        && let Some(opcode) = OpCode::from_raw_u8(raw)
    {
        return Instruction::format_a(opcode, 0, remap_reg(inst.dst()), remap_reg(inst.src1()), 0);
    }
    if (raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8)
        && let Some(opcode) = OpCode::from_raw_u8(raw)
    {
        return Instruction::format_a(
            opcode,
            0,
            remap_reg(inst.dst()),
            remap_reg(inst.src1()),
            inst.src2(),
        );
    }
    if (raw == OpCode::AddBandI32 as u8 || raw == OpCode::MulAddF64 as u8)
        && let Some(opcode) = OpCode::from_raw_u8(raw)
    {
        return Instruction::format_a(
            opcode,
            remap_reg(inst.flags() as u16) as u8,
            remap_reg(inst.dst()),
            remap_reg(inst.src1()),
            remap_reg(inst.src2()),
        );
    }
    if ((OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw))
        && let Some(opcode) = OpCode::from_raw_u8(raw)
    {
        return Instruction::format_a(
            opcode,
            inst.flags(),
            remap_reg(inst.dst()),
            remap_reg(inst.src1()),
            remap_reg(inst.src2()),
        );
    }
    rewrite_instruction_destination(&inst, remap_reg(inst.dst()))
}

/// Inlines empty/pure field-initializing leaf constructors and small pure arithmetic leaf helpers
/// at `CallDirect` call sites.
fn inline_leaf_functions(
    caller: &mut CompiledFunction,
    caller_index: usize,
    all_functions: &[CompiledFunction],
    allow_arith_inline: bool,
) -> Result<bool, EmissionError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum LeafKind {
        None,
        Constructor,
        PureArith,
    }

    let mut leaf_kinds = vec![LeafKind::None; all_functions.len()];
    for (func_idx, candidate) in all_functions.iter().enumerate() {
        if func_idx == caller_index
            || !candidate.exception_handlers.is_empty()
            || candidate.param_count == 0
            || candidate.instructions.is_empty()
            || candidate.instructions.len() > 8
        {
            continue;
        }
        if candidate.symbol.contains("::constructor") {
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
                    (raw == OpCode::SetField as u8
                        || raw == OpCode::SetFieldI32 as u8
                        || raw == OpCode::SetFieldF64 as u8)
                        && inst.src1() == 0
                });
            if body_ok {
                leaf_kinds[func_idx] = LeafKind::Constructor;
            }
        } else if allow_arith_inline
            && candidate.reference_registers.is_empty()
            && !candidate.symbol.starts_with('@')
            && candidate.symbol != "main"
            && !candidate.symbol.ends_with("::main")
        {
            let body_ok = candidate
                .instructions
                .iter()
                .enumerate()
                .all(|(idx, inst)| {
                    if idx + 1 == candidate.instructions.len() {
                        let raw = inst.raw_opcode();
                        return raw == OpCode::Return as u8 || raw == OpCode::ReturnValue as u8;
                    }
                    is_pure_leaf_arith_inst(*inst, candidate.param_count)
                })
                && candidate.instructions.iter().all(|inst| {
                    (inst.raw_opcode() != OpCode::AddBandI32 as u8
                        && inst.raw_opcode() != OpCode::MulAddF64 as u8)
                        || candidate.num_registers <= 200
                });
            if body_ok {
                leaf_kinds[func_idx] = LeafKind::PureArith;
            }
        }
    }

    let inst_count = caller.instructions.len();
    let has_candidate = caller.instructions.iter().any(|inst| {
        inst.raw_opcode() == OpCode::CallDirect as u8
            && (inst.call_target() as usize) < leaf_kinds.len()
            && leaf_kinds[inst.call_target() as usize] != LeafKind::None
    });
    if !has_candidate {
        return Ok(false);
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
            let call_dst = inst.call_dst();
            let leaf_kind = leaf_kinds
                .get(target_idx)
                .copied()
                .unwrap_or(LeafKind::None);
            if leaf_kind != LeafKind::None && arg_count == all_functions[target_idx].param_count {
                let callee = &all_functions[target_idx];
                let scratch_needed = callee.num_registers.saturating_sub(arg_count);
                let limit_regs = if callee.instructions.iter().any(|callee_inst| {
                    let raw = callee_inst.raw_opcode();
                    raw == OpCode::AddBandI32 as u8 || raw == OpCode::MulAddF64 as u8
                }) {
                    255usize
                } else {
                    u16::MAX as usize
                };
                if (caller.num_registers as usize) + (scratch_needed as usize) < limit_regs {
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
                        let remapped = remap_leaf_instruction(ctor_inst, &remap_reg);
                        rebuilt.push((remapped, None));
                        pushed_any = true;
                    }
                    let tail_inst = callee.instructions[body_len];
                    if tail_inst.raw_opcode() == OpCode::ReturnValue as u8 {
                        let mapped_ret = remap_reg(tail_inst.dst());
                        if mapped_ret != call_dst {
                            rebuilt.push((
                                Instruction::format_a(OpCode::Mov, 0, call_dst, mapped_ret, 0),
                                None,
                            ));
                        }
                        pushed_any = true;
                    }
                    if !pushed_any && leaf_kind == LeafKind::Constructor {
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
        return Ok(false);
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
            final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc)?;
        }
        final_instructions.push(final_inst);
    }
    apply_pc_relocation(caller, final_instructions, &old_pc_to_new_pc);
    Ok(true)
}

/// Executes convergent multi-pass peephole fusion and redundant `Mov` elimination
/// backed by forward CFG liveness verification.
fn optimize_peephole_passes(
    function: &mut CompiledFunction,
    allow_imm_branch_fusion: bool,
    constant_pool: Option<&solix_common::bytecode::ConstantPool>,
) -> Result<(), EmissionError> {
    let original_count = function.instructions.len();
    if original_count == 0 {
        return Ok(());
    }

    let critical_targets = collect_critical_targets(function);
    let mut stream: Vec<IntermediateInstruction> = Vec::with_capacity(original_count);

    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        stream.push(IntermediateInstruction {
            instruction: *instruction,
            original_pc: pc_index,
            covered_original_pcs: smallvec![pc_index],
            jump_target_old_pc: decode_jump_target(*instruction, pc_index, original_count),
        });
    }

    let mut next_stream: Vec<IntermediateInstruction> = Vec::with_capacity(original_count);
    let mut pending_orphaned_pcs: SmallVec<[usize; 2]> = SmallVec::new();
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

        next_stream.clear();
        pending_orphaned_pcs.clear();
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

                let is_f64_compare_branch = (current_raw == OpCode::LtF64 as u8
                    || current_raw == OpCode::LteF64 as u8)
                    && next_raw == OpCode::JumpIfTrue as u8;

                let is_i64_compare_branch = (current_raw == OpCode::LtI64 as u8
                    || current_raw == OpCode::LteI64 as u8
                    || current_raw == OpCode::GtI64 as u8
                    || current_raw == OpCode::GteI64 as u8)
                    && is_branch;

                if !next_is_critical
                    && ((is_compare && is_branch) || is_f64_compare_branch || is_i64_compare_branch)
                {
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
                        let (fused_opcode, reg_first, reg_second) = if is_f64_compare_branch {
                            let opcode_val = if current_raw == OpCode::LtF64 as u8 {
                                OpCode::JumpIfLtF64
                            } else {
                                OpCode::JumpIfLteF64
                            };
                            (
                                opcode_val,
                                current_entry.instruction.src1(),
                                current_entry.instruction.src2(),
                            )
                        } else if is_i64_compare_branch {
                            let is_false_branch = next_raw == OpCode::JumpIfFalse as u8;
                            let src_one = current_entry.instruction.src1();
                            let src_two = current_entry.instruction.src2();
                            match (current_raw, is_false_branch) {
                                (raw_val, false) if raw_val == OpCode::LtI64 as u8 => {
                                    (OpCode::JumpIfLtI64, src_one, src_two)
                                }
                                (raw_val, true) if raw_val == OpCode::LtI64 as u8 => {
                                    (OpCode::JumpIfLteI64, src_two, src_one)
                                }
                                (raw_val, false) if raw_val == OpCode::LteI64 as u8 => {
                                    (OpCode::JumpIfLteI64, src_one, src_two)
                                }
                                (raw_val, true) if raw_val == OpCode::LteI64 as u8 => {
                                    (OpCode::JumpIfLtI64, src_two, src_one)
                                }
                                (raw_val, false) if raw_val == OpCode::GtI64 as u8 => {
                                    (OpCode::JumpIfLtI64, src_two, src_one)
                                }
                                (raw_val, true) if raw_val == OpCode::GtI64 as u8 => {
                                    (OpCode::JumpIfLteI64, src_one, src_two)
                                }
                                (raw_val, false) if raw_val == OpCode::GteI64 as u8 => {
                                    (OpCode::JumpIfLteI64, src_two, src_one)
                                }
                                (raw_val, true) if raw_val == OpCode::GteI64 as u8 => {
                                    (OpCode::JumpIfLtI64, src_one, src_two)
                                }
                                _ => (OpCode::JumpIfLtI64, src_one, src_two),
                            }
                        } else {
                            let opcode_val =
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
                            (
                                opcode_val,
                                current_entry.instruction.src1(),
                                current_entry.instruction.src2(),
                            )
                        };

                        let mut merged_pcs = {
                            let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                            combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                            combined_pcs
                        };
                        merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                        next_stream.push(IntermediateInstruction {
                            instruction: Instruction::format_a(
                                fused_opcode,
                                0,
                                reg_first,
                                reg_second,
                                0,
                            ),
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
                        } else if allow_imm_branch_fusion
                            && (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8)
                                .contains(&next_raw)
                        {
                            let reg_a = next_entry.instruction.dst();
                            let reg_b = next_entry.instruction.src1();
                            let cmp_info = if reg_b == r_imm && reg_a != r_imm {
                                Some((reg_a, jump_i32_to_imm_flag(next_raw, false)))
                            } else if reg_a == r_imm && reg_b != r_imm {
                                Some((reg_b, jump_i32_to_imm_flag(next_raw, true)))
                            } else {
                                None
                            };

                            if let Some((var_reg, cmp_flag)) = cmp_info
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
                                        OpCode::JumpIfI32Imm,
                                        cmp_flag,
                                        var_reg,
                                        imm_val as i16 as u16,
                                        0,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: next_entry.jump_target_old_pc,
                                });

                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        }
                    }
                }

                // ---------------------------------------------------------
                // Pattern 2-f64: LoadF64 + AddF64 / SubF64 / MulF64 Fusion -> AddF64Imm / SubF64Imm / MulF64Imm
                // ---------------------------------------------------------
                if !next_is_critical && current_raw == OpCode::LoadF64 as u8 {
                    let const_idx = current_entry.instruction.imm32();
                    if const_idx <= u16::MAX as u32 {
                        let r_const = current_entry.instruction.dst();
                        if next_raw == OpCode::AddF64 as u8 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();
                            let other_reg = if src2_reg == r_const && src1_reg != r_const {
                                Some(src1_reg)
                            } else if src1_reg == r_const && src2_reg != r_const {
                                Some(src2_reg)
                            } else {
                                None
                            };
                            if let Some(r_other) = other_reg
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_const,
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
                                        OpCode::AddF64Imm,
                                        0,
                                        dst_reg,
                                        r_other,
                                        const_idx as u16,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: None,
                                });
                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        } else if next_raw == OpCode::SubF64 as u8 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();
                            if src2_reg == r_const
                                && src1_reg != r_const
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_const,
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
                                        OpCode::SubF64Imm,
                                        0,
                                        dst_reg,
                                        src1_reg,
                                        const_idx as u16,
                                    ),
                                    original_pc: current_entry.original_pc,
                                    covered_original_pcs: merged_pcs,
                                    jump_target_old_pc: None,
                                });
                                changed = true;
                                cursor += 2;
                                continue;
                            }
                        } else if next_raw == OpCode::MulF64 as u8 {
                            let dst_reg = next_entry.instruction.dst();
                            let src1_reg = next_entry.instruction.src1();
                            let src2_reg = next_entry.instruction.src2();
                            let other_reg = if src2_reg == r_const && src1_reg != r_const {
                                Some(src1_reg)
                            } else if src1_reg == r_const && src2_reg != r_const {
                                Some(src2_reg)
                            } else {
                                None
                            };
                            if let Some(r_other) = other_reg
                                && is_dead_after_use(
                                    &stream,
                                    cursor + 1,
                                    r_const,
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
                                        OpCode::MulF64Imm,
                                        0,
                                        dst_reg,
                                        r_other,
                                        const_idx as u16,
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
                // Pattern 2-i64: LoadI64 / LoadI32 / LoadConst(Int64) + AddI64 Fusion -> AddI64Imm
                // ---------------------------------------------------------
                if !next_is_critical {
                    let maybe_imm_i64 = if current_raw == OpCode::LoadI64 as u8
                        || current_raw == OpCode::LoadI32 as u8
                    {
                        let imm_val = current_entry.instruction.imm32_signed();
                        if (-32768..=32767).contains(&imm_val) {
                            Some(imm_val)
                        } else {
                            None
                        }
                    } else if current_raw == OpCode::LoadConst as u8 {
                        let const_idx = current_entry.instruction.imm32() as usize;
                        constant_pool.and_then(|pool| match pool.values.get(const_idx) {
                            Some(solix_common::bytecode::ConstantPoolValue::Int64(num))
                                if (-32768..=32767).contains(num) =>
                            {
                                Some(*num as i32)
                            }
                            _ => None,
                        })
                    } else {
                        None
                    };

                    if let Some(imm_val) = maybe_imm_i64 {
                        let r_imm = current_entry.instruction.dst();
                        if next_raw == OpCode::AddI64 as u8 {
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
                                        OpCode::AddI64Imm,
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
                        }
                    }
                }

                // ---------------------------------------------------------
                // Pattern 2b: AddI32 + Band Fusion -> AddBandI32
                // ---------------------------------------------------------
                if !next_is_critical
                    && current_raw == OpCode::AddI32 as u8
                    && next_raw == OpCode::Band as u8
                {
                    let r_tmp = current_entry.instruction.dst();
                    let add_a = current_entry.instruction.src1();
                    let add_b = current_entry.instruction.src2();
                    let band_dst = next_entry.instruction.dst();
                    let band_a = next_entry.instruction.src1();
                    let band_b = next_entry.instruction.src2();

                    let mask_reg_opt = if band_a == r_tmp && band_b != r_tmp {
                        Some(band_b)
                    } else if band_b == r_tmp && band_a != r_tmp {
                        Some(band_a)
                    } else {
                        None
                    };

                    let acc_val_opt = if add_a <= 255 {
                        Some((add_a as u8, add_b))
                    } else if add_b <= 255 {
                        Some((add_b as u8, add_a))
                    } else {
                        None
                    };

                    if let (Some(r_mask), Some((r_acc, r_val))) = (mask_reg_opt, acc_val_opt)
                        && (r_tmp == band_dst
                            || is_dead_after_use(
                                &stream,
                                cursor + 1,
                                r_tmp,
                                function,
                                &old_pc_to_stream_idx,
                                &stream_critical,
                            ))
                    {
                        let mut merged_pcs = {
                            let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                            combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                            combined_pcs
                        };
                        merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                        next_stream.push(IntermediateInstruction {
                            instruction: Instruction::format_a(
                                OpCode::AddBandI32,
                                r_acc,
                                band_dst,
                                r_val,
                                r_mask,
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

                // ---------------------------------------------------------
                // Pattern 2b-2: MulF64 + AddF64 Fusion -> MulAddF64
                // ---------------------------------------------------------
                if allow_imm_branch_fusion
                    && !next_is_critical
                    && current_raw == OpCode::MulF64 as u8
                    && next_raw == OpCode::AddF64 as u8
                {
                    let r_tmp = current_entry.instruction.dst();
                    let mul_a = current_entry.instruction.src1();
                    let mul_b = current_entry.instruction.src2();
                    let add_dst = next_entry.instruction.dst();
                    let add_a = next_entry.instruction.src1();
                    let add_b = next_entry.instruction.src2();

                    let acc_reg_opt = if add_a == r_tmp && add_b != r_tmp {
                        Some(add_b)
                    } else if add_b == r_tmp && add_a != r_tmp {
                        Some(add_a)
                    } else {
                        None
                    };

                    if r_tmp != mul_a
                        && r_tmp != mul_b
                        && let Some(r_acc) = acc_reg_opt
                        && r_acc <= 255
                        && (r_tmp == add_dst
                            || is_dead_after_use(
                                &stream,
                                cursor + 1,
                                r_tmp,
                                function,
                                &old_pc_to_stream_idx,
                                &stream_critical,
                            ))
                    {
                        let mut merged_pcs = {
                            let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                            combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                            combined_pcs
                        };
                        merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                        next_stream.push(IntermediateInstruction {
                            instruction: Instruction::format_a(
                                OpCode::MulAddF64,
                                r_acc as u8,
                                add_dst,
                                mul_a,
                                mul_b,
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

                // ---------------------------------------------------------
                // Pattern 2b-3: GetArray + JumpIfFalse/JumpIfTrue -> GetArrayJumpIfFalse
                // ---------------------------------------------------------
                if allow_imm_branch_fusion
                    && !next_is_critical
                    && current_raw == OpCode::GetArray as u8
                    && (next_raw == OpCode::JumpIfFalse as u8
                        || next_raw == OpCode::JumpIfTrue as u8)
                {
                    let cond_reg = current_entry.instruction.dst();
                    let arr_reg = current_entry.instruction.src1();
                    let idx_reg = current_entry.instruction.src2();
                    if next_entry.instruction.cond_reg() == cond_reg
                        && arr_reg != cond_reg
                        && idx_reg != cond_reg
                        && is_dead_after_use(
                            &stream,
                            cursor + 1,
                            cond_reg,
                            function,
                            &old_pc_to_stream_idx,
                            &stream_critical,
                        )
                    {
                        let fused_opcode = if next_raw == OpCode::JumpIfTrue as u8 {
                            OpCode::GetArrayJumpIfTrue
                        } else {
                            OpCode::GetArrayJumpIfFalse
                        };
                        let mut merged_pcs = {
                            let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                            combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                            combined_pcs
                        };
                        merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                        next_stream.push(IntermediateInstruction {
                            instruction: Instruction::format_a(
                                fused_opcode,
                                0,
                                arr_reg,
                                idx_reg,
                                0,
                            ),
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
                // Pattern 2c: AddI32Imm(+1) / AddI32(step) + JumpIfLtI32/JumpIfLteI32 -> ForLoopI32
                // ---------------------------------------------------------
                if !next_is_critical
                    && (next_raw == OpCode::JumpIfLtI32 as u8
                        || next_raw == OpCode::JumpIfLteI32 as u8)
                {
                    let r_idx = next_entry.instruction.dst();
                    let r_bound = next_entry.instruction.src1();
                    if r_idx != r_bound {
                        let is_lte = if next_raw == OpCode::JumpIfLteI32 as u8 {
                            0x40u8
                        } else {
                            0u8
                        };
                        let loop_flags_opt = if current_raw == OpCode::AddI32Imm as u8
                            && current_entry.instruction.dst() == r_idx
                            && current_entry.instruction.src1() == r_idx
                            && (current_entry.instruction.src2() as i16) == 1
                        {
                            Some(is_lte)
                        } else if current_raw == OpCode::AddI32 as u8
                            && current_entry.instruction.dst() == r_idx
                        {
                            let s1 = current_entry.instruction.src1();
                            let s2 = current_entry.instruction.src2();
                            if s1 == r_idx && s2 < 32 && s2 != r_idx {
                                Some(0x80 | is_lte | (s2 as u8))
                            } else if s2 == r_idx && s1 < 32 && s1 != r_idx {
                                Some(0x80 | is_lte | (s1 as u8))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some(loop_flags) = loop_flags_opt {
                            let mut merged_pcs = {
                                let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                                combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                                combined_pcs
                            };
                            merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                            next_stream.push(IntermediateInstruction {
                                instruction: Instruction::format_a(
                                    OpCode::ForLoopI32,
                                    loop_flags,
                                    r_idx,
                                    r_bound,
                                    0,
                                ),
                                original_pc: current_entry.original_pc,
                                covered_original_pcs: merged_pcs,
                                jump_target_old_pc: next_entry.jump_target_old_pc,
                            });

                            changed = true;
                            cursor += 2;
                            continue;
                        }
                    }
                }

                // ---------------------------------------------------------
                // Pattern 2d: SetField / SetArray + DecRef -> FLAG_MOVE Store
                // ---------------------------------------------------------
                if !next_is_critical && next_raw == OpCode::DecRef as u8 {
                    let dec_reg = next_entry.instruction.dst();
                    if current_raw == OpCode::SetField as u8 {
                        let val_reg = current_entry.instruction.dst();
                        let obj_reg = current_entry.instruction.src1();
                        let flags = current_entry.instruction.flags();
                        let field_tag =
                            (flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
                        if dec_reg == val_reg
                            && val_reg != obj_reg
                            && (flags & solix_common::bytecode::instruction::FLAG_WEAK) == 0
                            && (flags & solix_common::bytecode::instruction::FLAG_MOVE) == 0
                            && (field_tag == 0 || field_tag == 12)
                            && is_dead_after_use(
                                &stream,
                                cursor + 1,
                                val_reg,
                                function,
                                &old_pc_to_stream_idx,
                                &stream_critical,
                            )
                        {
                            let mut merged_pcs = {
                                let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                                combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                                combined_pcs
                            };
                            merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                            next_stream.push(IntermediateInstruction {
                                instruction: Instruction::format_a(
                                    OpCode::SetField,
                                    flags | solix_common::bytecode::instruction::FLAG_MOVE,
                                    val_reg,
                                    obj_reg,
                                    current_entry.instruction.src2(),
                                ),
                                original_pc: current_entry.original_pc,
                                covered_original_pcs: merged_pcs,
                                jump_target_old_pc: None,
                            });

                            changed = true;
                            cursor += 2;
                            continue;
                        }
                    } else if current_raw == OpCode::SetArray as u8 {
                        let arr_reg = current_entry.instruction.dst();
                        let idx_reg = current_entry.instruction.src1();
                        let val_reg = current_entry.instruction.src2();
                        let flags = current_entry.instruction.flags();
                        if dec_reg == val_reg
                            && val_reg != arr_reg
                            && val_reg != idx_reg
                            && (flags & solix_common::bytecode::instruction::FLAG_MOVE) == 0
                            && is_dead_after_use(
                                &stream,
                                cursor + 1,
                                val_reg,
                                function,
                                &old_pc_to_stream_idx,
                                &stream_critical,
                            )
                        {
                            let mut merged_pcs = {
                                let mut combined_pcs = std::mem::take(&mut pending_orphaned_pcs);
                                combined_pcs.extend_from_slice(&current_entry.covered_original_pcs);
                                combined_pcs
                            };
                            merged_pcs.extend_from_slice(&next_entry.covered_original_pcs);

                            next_stream.push(IntermediateInstruction {
                                instruction: Instruction::format_a(
                                    OpCode::SetArray,
                                    flags | solix_common::bytecode::instruction::FLAG_MOVE,
                                    arr_reg,
                                    idx_reg,
                                    val_reg,
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

        std::mem::swap(&mut stream, &mut next_stream);
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
            final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc)?;
        }
        final_instructions.push(final_inst);
    }

    apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
    Ok(())
}

/// Checks whether `inst` is a hoistable pure arithmetic operation (`MulI32`, `AddI32`, `SubI32`,
/// `MulI32Imm`, `AddI32Imm`, `SubI32Imm`, `MulF64`, `AddF64`, `SubF64`).
fn is_hoistable_pure_arith(raw: u8) -> bool {
    raw == OpCode::AddI32 as u8
        || raw == OpCode::SubI32 as u8
        || raw == OpCode::MulI32 as u8
        || raw == OpCode::AddF64 as u8
        || raw == OpCode::SubF64 as u8
        || raw == OpCode::MulF64 as u8
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
}

/// Checks whether `inst` is a candidate operation that can be duplicated during loop rotation.
fn is_rotatable_guard_op(raw: u8) -> bool {
    is_hoistable_pure_arith(raw)
        || raw == OpCode::LtF64 as u8
        || raw == OpCode::LteF64 as u8
        || raw == OpCode::LtI64 as u8
        || raw == OpCode::LteI64 as u8
        || raw == OpCode::GetArray as u8
}

/// Specializes general `GetField` and `SetField` instructions accessing primitive int32/uint32
/// (tags 5, 6) or float64 (tag 10) fields into `GetFieldI32`/`SetFieldI32` or `GetFieldF64`/`SetFieldF64`
/// to eliminate runtime tag checking in tight dispatch loops.
fn specialize_primitive_field_accesses(function: &mut CompiledFunction) {
    for inst in &mut function.instructions {
        let raw = inst.raw_opcode();
        let raw_flags = inst.flags();
        let is_weak = (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) != 0;
        let is_move = (raw_flags & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
        let field_tag = (raw_flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
        if is_weak || is_move {
            continue;
        }
        if raw == OpCode::GetField as u8 {
            if field_tag == 5 || field_tag == 6 {
                *inst = Instruction::format_a(
                    OpCode::GetFieldI32,
                    0,
                    inst.dst(),
                    inst.src1(),
                    inst.src2(),
                );
            } else if field_tag == 10 {
                *inst = Instruction::format_a(
                    OpCode::GetFieldF64,
                    0,
                    inst.dst(),
                    inst.src1(),
                    inst.src2(),
                );
            }
        } else if raw == OpCode::SetField as u8 {
            if field_tag == 5 || field_tag == 6 {
                *inst = Instruction::format_a(
                    OpCode::SetFieldI32,
                    0,
                    inst.dst(),
                    inst.src1(),
                    inst.src2(),
                );
            } else if field_tag == 10 {
                *inst = Instruction::format_a(
                    OpCode::SetFieldF64,
                    0,
                    inst.dst(),
                    inst.src1(),
                    inst.src2(),
                );
            }
        }
    }
}

/// Checks whether all input operand registers read by a hoistable pure arithmetic instruction
/// remain completely unmodified throughout `[loop_body_start ..= loop_end_pc]`.
fn arith_inputs_loop_invariant(
    inst: Instruction,
    instructions: &[Instruction],
    loop_body_start: usize,
    loop_end_pc: usize,
) -> bool {
    let raw = inst.raw_opcode();
    let s1 = inst.src1();
    let check_s2 = raw == OpCode::AddI32 as u8
        || raw == OpCode::SubI32 as u8
        || raw == OpCode::MulI32 as u8
        || raw == OpCode::AddF64 as u8
        || raw == OpCode::SubF64 as u8
        || raw == OpCode::MulF64 as u8;
    let s2 = inst.src2();
    for check_inst in &instructions[loop_body_start..=loop_end_pc] {
        if instruction_writes_register(check_inst, s1) {
            return false;
        }
        if check_s2 && instruction_writes_register(check_inst, s2) {
            return false;
        }
    }
    true
}

/// Canonical deduplication key for hoisted constants and pure loop-invariant arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct HoistCacheKey {
    header_pc: usize,
    raw: u8,
    flags: u8,
    payload: u64,
}

/// Hoists loop-invariant constant loads (`LoadI32`, `LoadI64`, `LoadF64`, `LoadBool`,
/// `LoadConst`) and pure loop-invariant arithmetic above loop bodies into fresh registers and
/// rewrites in-loop consumers.
fn hoist_loop_invariant_constants(function: &mut CompiledFunction) -> Result<(), EmissionError> {
    let inst_count = function.instructions.len();
    if inst_count < 3 {
        return Ok(());
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
        return Ok(());
    }

    // Process innermost loops first so inner loop invariants can hoist out cleanly.
    let mut ordered_headers: Vec<(usize, usize)> = loops_by_header.into_iter().collect();
    ordered_headers.sort_by_key(|&(header_pc, loop_end_pc)| (loop_end_pc - header_pc, header_pc));

    // Build initial CFG liveness helper table
    let mut old_pc_targets: Vec<Option<usize>> = Vec::with_capacity(inst_count);
    for (pc_index, instruction) in function.instructions.iter().enumerate() {
        old_pc_targets.push(decode_jump_target(*instruction, pc_index, inst_count));
    }

    // For a rotated bottom-tested loop (`header_pc > 0`, `header_pc - 1` is an entry guard branching
    // to `loop_end + 1` and falling through to `header_pc`), placing the preheader right BEFORE
    // `header_pc` (with backedges targeting the first body instruction AFTER the preheader!) ensures
    // hoisted pure arithmetic never executes on 0-iteration loops!
    let mut is_rotated_entry_guard: HashMap<usize, bool> = HashMap::new();
    for &(header_pc, loop_end_pc) in &ordered_headers {
        let rotated_ok = header_pc > 0
            && old_pc_targets[header_pc - 1] == Some(loop_end_pc + 1)
            && is_conditional_branch(function.instructions[header_pc - 1].raw_opcode());
        // Only treat `header_pc - 1` as a dedicated entry guard when no forward jump outside the loop
        // targets `header_pc` directly (only backedge jumps inside `[header_pc..=loop_end_pc]` do).
        let only_backedges_target_header = old_pc_targets[..header_pc]
            .iter()
            .flatten()
            .all(|&target| target != header_pc)
            && old_pc_targets[(loop_end_pc + 1)..]
                .iter()
                .flatten()
                .all(|&target| target != header_pc);
        is_rotated_entry_guard.insert(
            header_pc,
            rotated_ok && only_backedges_target_header && !handler_pcs.contains(&header_pc),
        );
    }

    let mut preheaders: HashMap<usize, Vec<Instruction>> = HashMap::new();
    let mut hoisted_cache: HashMap<HoistCacheKey, u16> = HashMap::new();
    let mut hoisted_any = false;

    for (header_pc, loop_end_pc) in ordered_headers {
        let has_interior_external_entry = old_pc_targets[..header_pc]
            .iter()
            .flatten()
            .any(|&target_pc| target_pc > header_pc && target_pc <= loop_end_pc);
        if has_interior_external_entry {
            continue;
        }

        let allow_arith_here = is_rotated_entry_guard
            .get(&header_pc)
            .copied()
            .unwrap_or(false);
        let mut saw_branch_in_body = false;

        let mut cursor = header_pc;
        while cursor <= loop_end_pc && cursor < function.instructions.len() {
            let inst = function.instructions[cursor];
            let raw = inst.raw_opcode();
            if (cursor > header_pc && critical_targets.contains(&cursor))
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
                saw_branch_in_body = true;
            }

            let is_hoistable_load = raw == OpCode::LoadI32 as u8
                || raw == OpCode::LoadI64 as u8
                || raw == OpCode::LoadF64 as u8
                || raw == OpCode::LoadBool as u8
                || raw == OpCode::LoadConst as u8;

            let is_hoistable_arith = allow_arith_here
                && !saw_branch_in_body
                && is_hoistable_pure_arith(raw)
                && arith_inputs_loop_invariant(
                    inst,
                    &function.instructions,
                    header_pc,
                    loop_end_pc,
                );

            if !is_hoistable_load && !is_hoistable_arith {
                cursor += 1;
                continue;
            }

            let r_orig = inst.dst();
            let mut consumer_pcs: Vec<usize> = Vec::new();
            let mut last_use_pc: Option<usize> = None;
            let mut aborted_rewrite = false;
            let mut terminated_block = false;

            for (scan_pc, &scan_inst) in function
                .instructions
                .iter()
                .enumerate()
                .take(inst_count)
                .skip(cursor + 1)
            {
                if critical_targets.contains(&scan_pc) {
                    break;
                }
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

            let payload = if is_hoistable_load {
                inst.imm32() as u64
            } else {
                ((inst.src1() as u64) << 16) | (inst.src2() as u64)
            };
            let cache_key = HoistCacheKey {
                header_pc,
                raw,
                flags: inst.flags(),
                payload,
            };
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
        return Ok(());
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
            final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc)?;
        }
        final_instructions.push(final_inst);
    }

    apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
    Ok(())
}

/// Inverts a loop-exit conditional guard into a bottom-tested loop continuation guard.
fn invert_loop_guard(instruction: Instruction, back_offset: i32) -> Option<Instruction> {
    let raw = instruction.raw_opcode();
    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
        || raw == OpCode::GetArrayJumpIfFalse as u8
        || raw == OpCode::GetArrayJumpIfTrue as u8
        || raw == OpCode::JumpIfLtI64 as u8
        || raw == OpCode::JumpIfLteI64 as u8
    {
        let offset_i16 = i16::try_from(back_offset).ok()?;
        let (inverted, dst_reg, src_reg) = match raw {
            val if val == OpCode::JumpIfLtI32 as u8 => {
                (OpCode::JumpIfGteI32, instruction.dst(), instruction.src1())
            }
            val if val == OpCode::JumpIfLteI32 as u8 => {
                (OpCode::JumpIfGtI32, instruction.dst(), instruction.src1())
            }
            val if val == OpCode::JumpIfGtI32 as u8 => {
                (OpCode::JumpIfLteI32, instruction.dst(), instruction.src1())
            }
            val if val == OpCode::JumpIfGteI32 as u8 => {
                (OpCode::JumpIfLtI32, instruction.dst(), instruction.src1())
            }
            val if val == OpCode::JumpIfEqI32 as u8 => {
                (OpCode::JumpIfNeqI32, instruction.dst(), instruction.src1())
            }
            val if val == OpCode::GetArrayJumpIfFalse as u8 => (
                OpCode::GetArrayJumpIfTrue,
                instruction.dst(),
                instruction.src1(),
            ),
            val if val == OpCode::GetArrayJumpIfTrue as u8 => (
                OpCode::GetArrayJumpIfFalse,
                instruction.dst(),
                instruction.src1(),
            ),
            val if val == OpCode::JumpIfLtI64 as u8 => {
                (OpCode::JumpIfLteI64, instruction.src1(), instruction.dst())
            }
            val if val == OpCode::JumpIfLteI64 as u8 => {
                (OpCode::JumpIfLtI64, instruction.src1(), instruction.dst())
            }
            _ => (OpCode::JumpIfEqI32, instruction.dst(), instruction.src1()),
        };
        return Some(Instruction::format_a(
            inverted,
            0,
            dst_reg,
            src_reg,
            offset_i16 as u16,
        ));
    }
    if raw == OpCode::JumpIfFalse as u8 {
        return Some(Instruction::format_c(
            OpCode::JumpIfTrue,
            0,
            instruction.dst(),
            back_offset,
        ));
    }
    if raw == OpCode::JumpIfTrue as u8 {
        return Some(Instruction::format_c(
            OpCode::JumpIfFalse,
            0,
            instruction.dst(),
            back_offset,
        ));
    }
    None
}

/// Converts single-entry top-tested `JumpIf* -> exit; ...; Jump -> header` loops and
/// 2-instruction pure guard `(Alu + JumpIf* -> exit; ...; Jump -> header)` loops into bottom-tested
/// loops, eliminating one unconditional `Jump` dispatch per iteration. When no internal `continue`
/// jumps to `header`, preserves the top exit guard so `ForLoopI32` peephole fusion remains unblocked.
fn rotate_bottom_tested_loops(function: &mut CompiledFunction) -> Result<(), EmissionError> {
    let inst_count = function.instructions.len();
    if inst_count < 3
        || function
            .instructions
            .iter()
            .any(|inst| inst.raw_opcode() == OpCode::Switch as u8)
    {
        return Ok(());
    }

    let critical_targets = collect_critical_targets(function);
    let mut one_insn_candidates: Vec<(usize, usize)> = Vec::new();
    let mut two_insn_candidates: Vec<(usize, usize)> = Vec::new();
    for (loop_end, inst) in function.instructions.iter().enumerate() {
        if inst.raw_opcode() == OpCode::Jump as u8 && inst.jump_offset() < 0 {
            let header_signed = (loop_end as i32) + inst.jump_offset();
            if header_signed >= 0 && (header_signed as usize) < loop_end {
                let header_pc = header_signed as usize;
                if decode_jump_target(function.instructions[header_pc], header_pc, inst_count)
                    == Some(loop_end + 1)
                {
                    one_insn_candidates.push((header_pc, loop_end));
                } else if header_pc + 2 <= loop_end
                    && !critical_targets.contains(&(header_pc + 1))
                    && is_rotatable_guard_op(function.instructions[header_pc].raw_opcode())
                    && decode_jump_target(
                        function.instructions[header_pc + 1],
                        header_pc + 1,
                        inst_count,
                    ) == Some(loop_end + 1)
                {
                    two_insn_candidates.push((header_pc, loop_end));
                }
            }
        }
    }

    // First handle 1-instruction guard loops in-place (preserving PCs cleanly).
    if !one_insn_candidates.is_empty() {
        one_insn_candidates.sort_by_key(|&(header_pc, loop_end)| (loop_end - header_pc, header_pc));
        let mut used_endpoints = HashSet::new();
        for (header_pc, loop_end) in one_insn_candidates {
            if used_endpoints.contains(&header_pc) || used_endpoints.contains(&loop_end) {
                continue;
            }
            let guard_inst = function.instructions[header_pc];
            let body_pc = header_pc + 1;
            let back_offset = (body_pc as i32) - (loop_end as i32);
            let Some(inverted_guard) = invert_loop_guard(guard_inst, back_offset) else {
                continue;
            };

            let exception_blocks_rotation = function.exception_handlers.iter().any(|handler| {
                let start = handler.try_start_pc as usize;
                let end = handler.try_end_pc as usize;
                let catch_pc = handler.handler_pc as usize;
                let finally_pc = handler.finally_pc.map(|pc_val| pc_val as usize);
                if start == header_pc
                    || end == header_pc
                    || end == loop_end
                    || catch_pc == header_pc
                    || catch_pc == loop_end
                    || finally_pc == Some(header_pc)
                    || finally_pc == Some(loop_end)
                {
                    return true;
                }
                let catches_into_body = catch_pc > header_pc && catch_pc <= loop_end;
                let finally_into_body =
                    finally_pc.is_some_and(|pc_val| pc_val > header_pc && pc_val <= loop_end);
                (catches_into_body || finally_into_body) && (start <= header_pc || end > loop_end)
            });
            if exception_blocks_rotation {
                continue;
            }

            let has_external_entry =
                function
                    .instructions
                    .iter()
                    .enumerate()
                    .any(|(pc_idx, inst)| {
                        if pc_idx >= header_pc && pc_idx <= loop_end {
                            return false;
                        }
                        decode_jump_target(*inst, pc_idx, inst_count)
                            .is_some_and(|target| target > header_pc && target <= loop_end)
                    });
            if has_external_entry {
                continue;
            }

            let has_internal_continue = (body_pc..loop_end).any(|pc_idx| {
                decode_jump_target(function.instructions[pc_idx], pc_idx, inst_count)
                    == Some(header_pc)
            });

            if !has_internal_continue {
                // Keep top `header_pc` guard intact (exits directly to `loop_end + 1` on 0 iters);
                // place inverted back-edge guard at `loop_end` so `loop_end` is NOT a critical target
                // and `ADD_I32_IMM`/`ADD_I32` + `JUMP_IF_LT(E)_I32` fuses into `ForLoopI32`.
                function.instructions[loop_end] = inverted_guard;
            } else {
                let entry_offset_i64 = (loop_end as i64) - (header_pc as i64);
                let entry_offset = i32::try_from(entry_offset_i64).map_err(|_| {
                    EmissionError::JumpOffsetOverflow {
                        offset: entry_offset_i64,
                    }
                })?;
                let loop_end_u32 =
                    u32::try_from(loop_end).map_err(|_| EmissionError::JumpOffsetOverflow {
                        offset: i64::try_from(loop_end).unwrap_or(i64::MAX),
                    })?;
                let mut relocated_continues = Vec::new();
                for (pc_idx, inst) in function
                    .instructions
                    .iter()
                    .enumerate()
                    .take(loop_end)
                    .skip(body_pc)
                {
                    if decode_jump_target(*inst, pc_idx, inst_count) == Some(header_pc) {
                        let relocated = relocate_jump_instruction(*inst, pc_idx, loop_end_u32)?;
                        relocated_continues.push((pc_idx, relocated));
                    }
                }
                function.instructions[header_pc] =
                    Instruction::format_c(OpCode::Jump, 0, 0, entry_offset);
                function.instructions[loop_end] = inverted_guard;
                for (pc_idx, relocated) in relocated_continues {
                    function.instructions[pc_idx] = relocated;
                }
            }

            used_endpoints.insert(header_pc);
            used_endpoints.insert(loop_end);
        }
    }

    // Rotate 2-instruction pure guard loops (e.g. `MUL_I32 + JUMP_IF_GT_I32 -> exit`).
    for (header_pc, loop_end) in two_insn_candidates {
        let cur_len = function.instructions.len();
        if loop_end >= cur_len || header_pc + 2 > loop_end {
            continue;
        }
        let prep_inst = function.instructions[header_pc];
        let guard_pc = header_pc + 1;
        let guard_inst = function.instructions[guard_pc];
        let body_pc = header_pc + 2;
        if function.instructions[loop_end].raw_opcode() != OpCode::Jump as u8
            || decode_jump_target(function.instructions[loop_end], loop_end, cur_len)
                != Some(header_pc)
            || decode_jump_target(guard_inst, guard_pc, cur_len) != Some(loop_end + 1)
        {
            continue;
        }
        let Some(inverted_guard) = invert_loop_guard(guard_inst, 0) else {
            continue;
        };
        let exception_blocks = function.exception_handlers.iter().any(|handler| {
            let start = handler.try_start_pc as usize;
            let end = handler.try_end_pc as usize;
            let catch_pc = handler.handler_pc as usize;
            let finally_pc = handler.finally_pc.map(|pc_val| pc_val as usize);
            (start >= header_pc && start <= loop_end)
                || (end >= header_pc && end <= loop_end)
                || (catch_pc >= header_pc && catch_pc <= loop_end)
                || finally_pc.is_some_and(|fin_pc| fin_pc >= header_pc && fin_pc <= loop_end)
        });
        if exception_blocks {
            continue;
        }
        let has_external_entry = function
            .instructions
            .iter()
            .enumerate()
            .any(|(pc_idx, inst)| {
                if pc_idx >= header_pc && pc_idx <= loop_end {
                    return false;
                }
                decode_jump_target(*inst, pc_idx, cur_len)
                    .is_some_and(|target| target > header_pc && target <= loop_end)
            });
        if has_external_entry {
            continue;
        }

        let mut old_targets: Vec<Option<usize>> = Vec::with_capacity(cur_len);
        for (pc_idx, inst) in function.instructions.iter().enumerate() {
            old_targets.push(decode_jump_target(*inst, pc_idx, cur_len));
        }

        let mut rebuilt: Vec<(Instruction, Option<usize>)> = Vec::with_capacity(cur_len + 2);
        let mut old_pc_to_new_pc = vec![u32::MAX; cur_len + 1];
        for (pc_idx, &inst) in function.instructions.iter().enumerate() {
            old_pc_to_new_pc[pc_idx] = rebuilt.len() as u32;
            if pc_idx == loop_end {
                rebuilt.push((prep_inst, None));
                rebuilt.push((inverted_guard, Some(body_pc)));
            } else if pc_idx > header_pc
                && pc_idx < loop_end
                && old_targets[pc_idx] == Some(header_pc)
            {
                // Retarget internal continue edges to the bottom `prep_inst` copy.
                rebuilt.push((inst, Some(loop_end)));
            } else {
                rebuilt.push((inst, old_targets[pc_idx]));
            }
        }
        old_pc_to_new_pc[cur_len] = rebuilt.len() as u32;

        let mut final_instructions = Vec::with_capacity(rebuilt.len());
        for (new_pc, (inst, target_opt)) in rebuilt.into_iter().enumerate() {
            let mut final_inst = inst;
            if let Some(target_old_pc) = target_opt {
                let target_new_pc = old_pc_to_new_pc
                    .get(target_old_pc)
                    .copied()
                    .unwrap_or(old_pc_to_new_pc[cur_len]);
                final_inst = relocate_jump_instruction(final_inst, new_pc, target_new_pc)?;
            }
            final_instructions.push(final_inst);
        }
        apply_pc_relocation(function, final_instructions, &old_pc_to_new_pc);
        break;
    }

    Ok(())
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

/// Rewrites the signed PC jump offset of a branch/jump instruction cleanly.
///
/// # Errors
/// Returns `EmissionError::JumpOffsetOverflow` if the relocated branch offset exceeds the
/// signed integer range of the target instruction encoding (`i16` for Format-A, `i32` for Format-C).
pub fn relocate_jump_instruction(
    instruction: Instruction,
    new_pc: usize,
    target_new_pc: u32,
) -> Result<Instruction, EmissionError> {
    let new_pc_i64 = i64::try_from(new_pc)
        .map_err(|_| EmissionError::JumpOffsetOverflow { offset: i64::MAX })?;
    let new_offset = i64::from(target_new_pc) - new_pc_i64;
    let raw_opcode = instruction.raw_opcode();

    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw_opcode)
        || raw_opcode == OpCode::ForLoopI32 as u8
        || raw_opcode == OpCode::JumpIfI32Imm as u8
        || raw_opcode == OpCode::GetArrayJumpIfFalse as u8
        || raw_opcode == OpCode::GetArrayJumpIfTrue as u8
        || raw_opcode == OpCode::JumpIfLtF64 as u8
        || raw_opcode == OpCode::JumpIfLteF64 as u8
        || raw_opcode == OpCode::JumpIfLtI64 as u8
        || raw_opcode == OpCode::JumpIfLteI64 as u8
    {
        let offset_i16 = i16::try_from(new_offset)
            .map_err(|_| EmissionError::JumpOffsetOverflow { offset: new_offset })?;
        if let Some(opcode) = OpCode::from_raw_u8(raw_opcode) {
            return Ok(Instruction::format_a(
                opcode,
                instruction.flags(),
                instruction.dst(),
                instruction.src1(),
                offset_i16 as u16,
            ));
        }
    } else if (raw_opcode == OpCode::Jump as u8
        || raw_opcode == OpCode::JumpIfTrue as u8
        || raw_opcode == OpCode::JumpIfFalse as u8)
        && let Some(opcode) = OpCode::from_raw_u8(raw_opcode)
    {
        let offset_i32 = i32::try_from(new_offset)
            .map_err(|_| EmissionError::JumpOffsetOverflow { offset: new_offset })?;
        return Ok(Instruction::format_c(
            opcode,
            instruction.flags(),
            instruction.cond_reg(),
            offset_i32,
        ));
    }

    Ok(instruction)
}

/// Returns `true` if `opcode` strictly writes a primitive or null value (never a heap reference).
fn opcode_writes_only_non_reference(raw: u8) -> bool {
    raw == OpCode::LoadI32 as u8
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
        if scan_idx != at_idx && instruction_writes_register(inst, target_reg) {
            return opcode_writes_only_non_reference(inst.raw_opcode());
        }
        if scan_idx == at_idx && instruction_writes_register(inst, target_reg) {
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
        if instruction_writes_register(inst, target_reg) {
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
            if updated_raw == OpCode::UnpackCaptures as u8 {
                let cap_base = entry.instruction.arg_base();
                let cap_count = entry.instruction.arg_count() as u16;
                copy_map.retain(|&key_reg, &mut val_reg| {
                    !(key_reg >= cap_base && key_reg < cap_base + cap_count)
                        && !(val_reg >= cap_base && val_reg < cap_base + cap_count)
                });
            } else if let Some(written_reg) = instruction_write_dst(&entry.instruction) {
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
        if inst.raw_opcode() == OpCode::UnpackCaptures as u8 {
            let cap_base = inst.arg_base();
            let cap_count = inst.arg_count() as u16;
            for cap_reg in cap_base..cap_base + cap_count {
                *def_counts.entry(cap_reg).or_insert(0) += 1;
            }
        } else if let Some(written) = instruction_write_dst(&inst) {
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
            if instruction_writes_register(&stream[scan_idx].instruction, orig_reg) {
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
            .any(|idx| instruction_writes_register(&stream[idx].instruction, orig_reg));
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

            if instruction_writes_register(inst, reg) {
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

            if instruction_writes_register(inst, reg) {
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
        || raw == OpCode::ForLoopI32 as u8
        || raw == OpCode::JumpIfI32Imm as u8
        || raw == OpCode::GetArrayJumpIfFalse as u8
        || raw == OpCode::GetArrayJumpIfTrue as u8
        || raw == OpCode::JumpIfLtF64 as u8
        || raw == OpCode::JumpIfLteF64 as u8
        || raw == OpCode::JumpIfLtI64 as u8
        || raw == OpCode::JumpIfLteI64 as u8
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
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
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
        || is_unsigned_comparison_opcode(raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
        || raw == OpCode::ForLoopI32 as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8
        || raw == OpCode::AllocObject as u8
        || raw == OpCode::AllocObjectExtern as u8
        || raw == OpCode::GetField as u8
        || raw == OpCode::GetFieldI32 as u8
        || raw == OpCode::GetFieldF64 as u8
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
        || raw == OpCode::TailCallDirect as u8
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

/// Returns `true` if `instruction` writes to `target_reg` (including `UnpackCaptures` register windows).
fn instruction_writes_register(instruction: &Instruction, target_reg: u16) -> bool {
    if instruction.raw_opcode() == OpCode::UnpackCaptures as u8 {
        let base = instruction.arg_base();
        let count = instruction.arg_count() as u16;
        return target_reg >= base && target_reg < base + count;
    }
    instruction_write_dst(instruction) == Some(target_reg)
}

/// Checks whether a register read inside `instruction` can be cleanly rewritten from `from_reg` to `to_reg`.
fn can_rewrite_read_register(instruction: &Instruction, target_reg: u16) -> bool {
    let raw = instruction.raw_opcode();
    if raw == OpCode::CallDirect as u8
        || raw == OpCode::TailCallDirect as u8
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
    !(raw == OpCode::AllocTuple as u8
        || raw == OpCode::UnpackCaptures as u8
        || raw == OpCode::ForLoopI32 as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8)
}

/// Rewrites input register operands matching `from_reg` to `to_reg`, returning the updated instruction.
fn rewrite_instruction_read_register(
    instruction: &Instruction,
    from_reg: u16,
    to_reg: u16,
) -> Option<Instruction> {
    let raw = instruction.raw_opcode();
    let opcode = OpCode::from_raw_u8(raw)?;

    if raw == OpCode::JumpIfI32Imm as u8 {
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

    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
        || raw == OpCode::GetArrayJumpIfFalse as u8
        || raw == OpCode::GetArrayJumpIfTrue as u8
        || raw == OpCode::JumpIfLtF64 as u8
        || raw == OpCode::JumpIfLteF64 as u8
        || raw == OpCode::JumpIfLtI64 as u8
        || raw == OpCode::JumpIfLteI64 as u8
    {
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
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
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
        || raw == OpCode::GetFieldI32 as u8
        || raw == OpCode::GetFieldF64 as u8
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

    if raw == OpCode::SetField as u8
        || raw == OpCode::SetFieldI32 as u8
        || raw == OpCode::SetFieldF64 as u8
    {
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

    if raw == OpCode::GetArray as u8
        || (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
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
        || is_unsigned_comparison_opcode(raw)
        || raw == OpCode::Lshr as u8
        || raw == OpCode::Not as u8
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
        || raw == OpCode::GetFieldI32 as u8
        || raw == OpCode::GetFieldF64 as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8
        || raw == OpCode::ConvIToF as u8
        || raw == OpCode::ConvFToI as u8
        || (OpCode::LoadI32 as u8..=OpCode::LoadConst as u8).contains(&raw);

    if reads_new_dst && !pure_alu_or_load {
        return false;
    }

    // Format A instructions that write to dst
    if (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
        || raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
        || raw == OpCode::GetFieldI32 as u8
        || raw == OpCode::GetFieldF64 as u8
        || raw == OpCode::AddBandI32 as u8
        || raw == OpCode::MulAddF64 as u8
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
        || raw == OpCode::TailCallDirect as u8
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
        || raw == OpCode::TailCallDirect as u8
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
    if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&raw)
        || raw == OpCode::GetArrayJumpIfFalse as u8
        || raw == OpCode::GetArrayJumpIfTrue as u8
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

    // Format A: superinstructions AddI32Imm, SubI32Imm, MulI32Imm, and ModI32Imm read src1
    if raw == OpCode::AddI32Imm as u8
        || raw == OpCode::SubI32Imm as u8
        || raw == OpCode::MulI32Imm as u8
        || raw == OpCode::ModI32Imm as u8
        || raw == OpCode::AddI64Imm as u8
        || raw == OpCode::AddF64Imm as u8
        || raw == OpCode::SubF64Imm as u8
        || raw == OpCode::MulF64Imm as u8
    {
        return instruction.src1() == target_reg;
    }

    // Format A: Mov, Unary ops, GetField, ArrayLength, CastCheck, Instanceof read src1
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
    if raw == OpCode::SetField as u8
        || raw == OpCode::SetFieldI32 as u8
        || raw == OpCode::SetFieldF64 as u8
    {
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
    if (OpCode::AddI32 as u8..=OpCode::GteF64 as u8).contains(&raw)
        || is_unsigned_comparison_opcode(raw)
    {
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
        || raw == OpCode::TailCallDirect as u8
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

/// Lowers tail-position `CallDirect` + `ReturnValue` pairs into frame-reusing `TailCallDirect`
/// when the caller holds no owning ARC registers or exception handlers and the callee has no ARC cleanup.
fn lower_tail_calls(function: &mut CompiledFunction, func_has_refs: &[bool]) {
    if !function.reference_registers.is_empty()
        || !function.exception_handlers.is_empty()
        || function.instructions.len() < 2
    {
        return;
    }

    let critical_targets = collect_critical_targets(function);
    let last_idx = function.instructions.len() - 1;
    for pc_idx in 0..last_idx {
        let inst = function.instructions[pc_idx];
        let next_inst = function.instructions[pc_idx + 1];
        if inst.raw_opcode() == OpCode::CallDirect as u8
            && next_inst.raw_opcode() == OpCode::ReturnValue as u8
            && next_inst.dst() == inst.call_dst()
            && !critical_targets.contains(&(pc_idx + 1))
        {
            let target_id = inst.call_target() as usize;
            let callee_has_refs = func_has_refs.get(target_id).copied().unwrap_or(true);
            if !callee_has_refs {
                function.instructions[pc_idx] = Instruction::format_call(
                    OpCode::TailCallDirect,
                    inst.call_dst(),
                    inst.call_target(),
                    inst.arg_base(),
                    inst.arg_count(),
                );
            }
        }
    }
}

/// Flags tight 2-instruction `SetArray` + `ForLoopI32(offset = -1)` primitive fill/strided loops
/// with bit `0x20` on `ForLoopI32` for batch native slice execution at runtime.
fn mark_array_fill_loops(function: &mut CompiledFunction) {
    if function.instructions.len() < 2 {
        return;
    }

    let critical_targets = collect_critical_targets(function);
    for pc_idx in 1..function.instructions.len() {
        if critical_targets.contains(&pc_idx) {
            continue;
        }
        let for_inst = function.instructions[pc_idx];
        if for_inst.raw_opcode() != OpCode::ForLoopI32 as u8 {
            continue;
        }
        if (for_inst.src2() as i16) != -1 {
            continue;
        }
        let prev = function.instructions[pc_idx - 1];
        if prev.raw_opcode() != OpCode::SetArray as u8 {
            continue;
        }
        let idx_reg = for_inst.dst();
        let bound_reg = for_inst.src1();
        let flags = for_inst.flags();
        let step_aliases_idx = (flags & 0x80) != 0 && (flags & 0x1F) as u16 == idx_reg;
        if prev.src1() == idx_reg
            && prev.dst() != idx_reg
            && prev.src2() != idx_reg
            && idx_reg != bound_reg
            && !step_aliases_idx
        {
            function.instructions[pc_idx] = Instruction::format_a(
                OpCode::ForLoopI32,
                flags | 0x20,
                idx_reg,
                bound_reg,
                for_inst.src2(),
            );
        }
    }
}

/// Fuses `AddI32 + LoadI32(mask) + Band` into `LoadI32(mask) + AddBandI32` after LICM
/// so loop-invariant mask hoisting is never blocked during earlier peephole passes.
fn fuse_post_licm_add_load_band(function: &mut CompiledFunction) {
    let inst_count = function.instructions.len();
    if inst_count < 3 {
        return;
    }

    let critical_targets = collect_critical_targets(function);
    let mut jump_targets = Vec::with_capacity(inst_count);
    for (pc_idx, &inst) in function.instructions.iter().enumerate() {
        jump_targets.push(decode_jump_target(inst, pc_idx, inst_count));
    }

    let mut pc_idx = 0usize;
    while pc_idx + 2 < inst_count {
        if critical_targets.contains(&(pc_idx + 1)) || critical_targets.contains(&(pc_idx + 2)) {
            pc_idx += 1;
            continue;
        }
        let add_inst = function.instructions[pc_idx];
        let load_inst = function.instructions[pc_idx + 1];
        let band_inst = function.instructions[pc_idx + 2];
        if add_inst.raw_opcode() == OpCode::AddI32 as u8
            && load_inst.raw_opcode() == OpCode::LoadI32 as u8
            && band_inst.raw_opcode() == OpCode::Band as u8
        {
            let r_tmp = add_inst.dst();
            let add_a = add_inst.src1();
            let add_b = add_inst.src2();
            let r_mask = load_inst.dst();
            let band_dst = band_inst.dst();
            let band_a = band_inst.src1();
            let band_b = band_inst.src2();
            let mask_matches =
                (band_a == r_tmp && band_b == r_mask) || (band_b == r_tmp && band_a == r_mask);
            let acc_val_opt = if add_a <= 255 {
                Some((add_a as u8, add_b))
            } else if add_b <= 255 {
                Some((add_b as u8, add_a))
            } else {
                None
            };
            if mask_matches
                && r_mask != r_tmp
                && r_mask != add_a
                && r_mask != add_b
                && let Some((r_acc, r_val)) = acc_val_opt
                && (r_tmp == band_dst
                    || is_dead_after_use_raw(
                        function,
                        &jump_targets,
                        &critical_targets,
                        pc_idx + 2,
                        r_tmp,
                    ))
            {
                function.instructions[pc_idx] = load_inst;
                function.instructions[pc_idx + 1] =
                    Instruction::format_a(OpCode::AddBandI32, r_acc, band_dst, r_val, r_mask);
                function.instructions[pc_idx + 2] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                pc_idx += 3;
                continue;
            }
        }
        pc_idx += 1;
    }
}

/// Converts pure 1-parameter binary additive self-recursion
/// (`if n <= k { return n; } return self(n - d1) + self(n - d2);`)
/// into a hybrid accumulator loop (`acc += self(n - d1); n -= d2;`), halving total VM call
/// frames and keeping the recursive argument register at the top of the caller frame (`needs_arg_copy = false`).
fn transform_binary_additive_self_recursion(function: &mut CompiledFunction, self_func_idx: usize) {
    if function.param_count != 1
        || !function.reference_registers.is_empty()
        || !function.exception_handlers.is_empty()
        || function.instructions.len() != 8
    {
        return;
    }

    let insts = &function.instructions;
    let jump_guard = insts[0];
    let base_ret = insts[1];
    let sub1 = insts[2];
    let call1 = insts[3];
    let sub2 = insts[4];
    let call2 = insts[5];
    let add = insts[6];
    let rec_ret = insts[7];

    let guard_flags = jump_guard.flags();
    if jump_guard.raw_opcode() != OpCode::JumpIfI32Imm as u8
        || jump_guard.dst() != 0
        || (jump_guard.src2() as i16) != 2
        || !(guard_flags == 4 || guard_flags == 5)
        || base_ret.raw_opcode() != OpCode::ReturnValue as u8
        || base_ret.dst() != 0
    {
        return;
    }

    let step1 = sub1.src2() as i16;
    let step2 = sub2.src2() as i16;
    let arg1 = sub1.dst();
    let arg2 = sub2.dst();
    let res1 = call1.call_dst();
    let res2 = call2.call_dst();

    if sub1.raw_opcode() != OpCode::SubI32Imm as u8
        || sub1.src1() != 0
        || step1 <= 0
        || call1.raw_opcode() != OpCode::CallDirect as u8
        || call1.call_target() as usize != self_func_idx
        || call1.arg_base() != arg1
        || call1.arg_count() != 1
        || res1 == 0
        || sub2.raw_opcode() != OpCode::SubI32Imm as u8
        || sub2.src1() != 0
        || step2 <= 0
        || arg2 == res1
        || call2.raw_opcode() != OpCode::CallDirect as u8
        || call2.call_target() as usize != self_func_idx
        || call2.arg_base() != arg2
        || call2.arg_count() != 1
        || res2 == 0
        || add.raw_opcode() != OpCode::AddI32 as u8
        || !((add.src1() == res1 && add.src2() == res2)
            || (add.src1() == res2 && add.src2() == res1))
        || rec_ret.raw_opcode() != OpCode::ReturnValue as u8
        || rec_ret.dst() != add.dst()
    {
        return;
    }

    let Ok(target_u16) = u16::try_from(self_func_idx) else {
        return;
    };
    let rewritten = vec![
        Instruction::format_a(OpCode::JumpIfI32Imm, guard_flags, 0, jump_guard.src1(), 2),
        Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0),
        Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0),
        Instruction::format_a(OpCode::SubI32Imm, 0, 3, 0, step1 as u16),
        Instruction::format_call(OpCode::CallDirect, 2, target_u16, 3, 1),
        Instruction::format_a(OpCode::AddI32, 0, 1, 1, 2),
        Instruction::format_a(OpCode::SubI32Imm, 0, 0, 0, step2 as u16),
        Instruction::format_a(
            OpCode::JumpIfI32Imm,
            guard_flags,
            0,
            jump_guard.src1(),
            (-4i16) as u16,
        ),
        Instruction::format_a(OpCode::AddI32, 0, 1, 1, 0),
        Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0),
    ];

    function.instructions = rewritten;
    function.num_registers = 4;
    function.source_map.clear();
    for local in &mut function.local_variables {
        if local.end_pc > 10 {
            local.end_pc = 10;
        }
    }
}
