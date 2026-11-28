//! Bytecode-to-Cranelift IR translator for Solix `CompiledFunction`s.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::vm::engine::VirtualMachine;
use crate::vm::jit::{JitCompileState, JitFuncSig, RegType};
use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{
    AbiParam, Block, InstBuilder, MemFlags, Signature, Value as ClifVal, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use solix_common::bytecode::{CompiledFunction, OpCode};
use std::collections::HashMap;
use std::time::Instant;

/// Ensures `func_id` is compiled (or marked `Unsupported` if ineligible) and returns its pointer & signature.
pub(crate) fn ensure_compiled(
    vm: &mut VirtualMachine,
    func_id: u32,
) -> Option<(*const u8, JitFuncSig)> {
    let idx = func_id as usize;
    let state = vm.jit_engine.as_ref()?.slots.get(idx)?.state;
    match state {
        JitCompileState::Compiled => {
            let slot = &vm.jit_engine.as_ref()?.slots[idx];
            return Some((slot.ptr, slot.sig.clone()?));
        }
        JitCompileState::Unsupported => return None,
        JitCompileState::Pending => {}
    }

    let start_time = Instant::now();
    let compiled_fn = vm.context.all_functions.get(idx)?.clone();
    let meta = vm.context.function_meta.get(idx)?.clone();
    if meta.native_fn.is_some()
        || compiled_fn.param_count as usize > 4
        || !compiled_fn.exception_handlers.is_empty()
    {
        if let Some(engine) = vm.jit_engine.as_mut() {
            engine.slots[idx].state = JitCompileState::Unsupported;
        }
        return None;
    }

    let Some(sig_info) = infer_signature_and_types(
        func_id,
        &compiled_fn,
        vm.context.get_constant_pool(meta.binary_id),
    ) else {
        if let Some(engine) = vm.jit_engine.as_mut() {
            engine.slots[idx].state = JitCompileState::Unsupported;
        }
        return None;
    };

    let clif_sig = {
        let engine = vm.jit_engine.as_ref()?;
        let ptr_ty = engine.module.target_config().pointer_type();
        let call_conv = engine.module.isa().default_call_conv();
        let mut clif_sig = Signature::new(call_conv);
        clif_sig.params.push(AbiParam::new(ptr_ty)); // ctx_ptr
        for _ in 0..4 {
            clif_sig.params.push(AbiParam::new(types::I64));
        }
        clif_sig.returns.push(AbiParam::new(types::I64));
        clif_sig
    };

    let clif_func_id = {
        let engine = vm.jit_engine.as_mut()?;
        let fn_name = format!("solix_jit_fn_{func_id}");
        let id = engine
            .module
            .declare_function(&fn_name, Linkage::Local, &clif_sig)
            .ok()?;
        engine.slots[idx].clif_id = Some(id);
        engine.slots[idx].sig = Some(sig_info.func_sig.clone());
        id
    };

    let mut direct_callees: Vec<u32> = Vec::new();
    for inst in &compiled_fn.instructions {
        if matches!(
            OpCode::from_raw_u8(inst.raw_opcode()),
            Some(OpCode::CallDirect | OpCode::TailCallDirect)
        ) {
            let target_id = u32::from(inst.call_target());
            if target_id != func_id && !direct_callees.contains(&target_id) {
                direct_callees.push(target_id);
            }
        }
    }
    if let Some(mono_sites) = vm.monomorphic_call_caches.get(func_id as usize) {
        for site in mono_sites.iter().flatten() {
            if site.expected_vtable_id != u32::MAX {
                let target_id = site.target_func_id;
                if target_id != func_id && !direct_callees.contains(&target_id) {
                    direct_callees.push(target_id);
                }
            }
        }
    }
    for target_id in direct_callees {
        let should_precompile = vm
            .jit_engine
            .as_ref()
            .and_then(|engine| engine.slots.get(target_id as usize))
            .is_some_and(|slot| slot.state == JitCompileState::Pending && slot.clif_id.is_none());
        if should_precompile {
            let _ = ensure_compiled(vm, target_id);
        }
    }

    let mut clif_ctx = {
        let engine = vm.jit_engine.as_mut()?;
        engine.module.make_context()
    };
    clif_ctx.func.signature = clif_sig;

    let const_pool = vm.context.get_constant_pool(meta.binary_id);
    let const_pool_cache = vm.const_pool_cache.get(meta.binary_id).map(Vec::as_slice);
    let monomorphic_caches = vm
        .monomorphic_call_caches
        .get(func_id as usize)
        .map(Vec::as_slice);
    let mut fb_ctx = FunctionBuilderContext::new();
    let build_ok = {
        let engine = vm.jit_engine.as_mut()?;
        translate_function(
            &compiled_fn,
            meta.binary_id as u32,
            const_pool,
            const_pool_cache,
            monomorphic_caches,
            &sig_info,
            clif_func_id,
            &mut clif_ctx.func,
            &mut fb_ctx,
            engine,
        )
    };

    if build_ok.is_err() {
        let engine = vm.jit_engine.as_mut()?;
        engine.slots[idx].state = JitCompileState::Unsupported;
        engine.slots[idx].clif_id = None;
        return None;
    }

    let compiled_ptr = {
        let engine = vm.jit_engine.as_mut()?;
        if engine
            .module
            .define_function(clif_func_id, &mut clif_ctx)
            .is_err()
        {
            engine.slots[idx].state = JitCompileState::Unsupported;
            return None;
        }
        engine.module.clear_context(&mut clif_ctx);
        if engine.module.finalize_definitions().is_err() {
            engine.slots[idx].state = JitCompileState::Unsupported;
            return None;
        }
        let code_ptr = engine.module.get_finalized_function(clif_func_id);
        engine.slots[idx].ptr = code_ptr;
        engine.slots[idx].state = JitCompileState::Compiled;
        engine.compiled_count += 1;
        engine.compile_time_ns += start_time.elapsed().as_nanos() as u64;
        code_ptr
    };

    Some((compiled_ptr, sig_info.func_sig))
}

#[derive(Clone, Copy, Default)]
struct InstTypeInfo {
    dst_ty: Option<RegType>,
    src_ty: Option<RegType>,
    call_args: [Option<RegType>; 4],
}

struct InferredFunctionTypes {
    used_masks: Vec<u8>,
    inst_types: Vec<InstTypeInfo>,
    func_sig: JitFuncSig,
}

struct VarTable {
    slots: Vec<[Option<Variable>; 4]>,
}

impl VarTable {
    fn get(&self, reg: u16, reg_ty: RegType) -> Variable {
        // Every `(reg, reg_ty)` read/written by a lowered instruction is marked in `used_masks`.
        self.slots[reg as usize][reg_ty.to_code() as usize].unwrap_or_else(|| Variable::from_u32(0))
    }
}

fn reg_type_to_clif(reg_ty: RegType) -> types::Type {
    match reg_ty {
        RegType::I32 | RegType::Ref => types::I32,
        RegType::I64 => types::I64,
        RegType::F64 => types::F64,
    }
}

fn infer_signature_and_types(
    func_id: u32,
    compiled_fn: &CompiledFunction,
    const_pool: Option<&solix_common::bytecode::ConstantPool>,
) -> Option<InferredFunctionTypes> {
    let num_regs = (compiled_fn.num_registers as usize).max(compiled_fn.param_count as usize);
    let param_count = compiled_fn.param_count as usize;
    let inst_count = compiled_fn.instructions.len();
    let mut used_masks: Vec<u8> = vec![0; num_regs];
    let mut param_types: Vec<Option<RegType>> = vec![None; param_count];
    let mut inst_types: Vec<InstTypeInfo> = vec![InstTypeInfo::default(); inst_count];
    let mut return_type: Option<RegType> = None;
    let mut has_return = false;

    for &ref_reg in &compiled_fn.reference_registers {
        let idx = ref_reg as usize;
        if idx < num_regs {
            used_masks[idx] |= 1 << RegType::Ref.to_code();
            if idx < param_count {
                param_types[idx] = Some(RegType::Ref);
            }
        }
    }

    let mut is_block_start = vec![false; inst_count];
    if inst_count > 0 {
        is_block_start[0] = true;
    }
    for (pc, inst) in compiled_fn.instructions.iter().enumerate() {
        let opcode = OpCode::from_raw_u8(inst.raw_opcode())?;
        match opcode {
            OpCode::Jump => {
                let target = ((pc as i32) + inst.jump_offset()) as usize;
                if target >= inst_count {
                    return None;
                }
                is_block_start[target] = true;
            }
            OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                let target = ((pc as i32) + inst.jump_offset()) as usize;
                if target >= inst_count || pc + 1 >= inst_count {
                    return None;
                }
                is_block_start[target] = true;
                is_block_start[pc + 1] = true;
            }
            OpCode::JumpIfI32Imm
            | OpCode::JumpIfLtI32
            | OpCode::JumpIfLteI32
            | OpCode::JumpIfGtI32
            | OpCode::JumpIfGteI32
            | OpCode::JumpIfEqI32
            | OpCode::JumpIfNeqI32
            | OpCode::JumpIfLtF64
            | OpCode::JumpIfLteF64
            | OpCode::ForLoopI32
            | OpCode::GetArrayJumpIfFalse
            | OpCode::GetArrayJumpIfTrue => {
                let target = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                if target >= inst_count || pc + 1 >= inst_count {
                    return None;
                }
                is_block_start[target] = true;
                is_block_start[pc + 1] = true;
            }
            _ => {}
        }
    }

    let mark_read = |reg: u16,
                     ty: RegType,
                     used: &mut [u8],
                     cur: &mut [Option<RegType>],
                     params: &mut [Option<RegType>],
                     written: &[bool]|
     -> bool {
        let idx = reg as usize;
        if idx >= used.len() {
            return false;
        }
        used[idx] |= 1 << ty.to_code();
        if idx < params.len() && !written[idx] && params[idx].is_none() {
            params[idx] = Some(ty);
        }
        if cur[idx].is_none() {
            cur[idx] = Some(ty);
        }
        true
    };

    let mark_write = |reg: u16,
                      ty: RegType,
                      used: &mut [u8],
                      cur: &mut [Option<RegType>],
                      written: &mut [bool]|
     -> bool {
        let idx = reg as usize;
        if idx >= used.len() {
            return false;
        }
        used[idx] |= 1 << ty.to_code();
        cur[idx] = Some(ty);
        if idx < written.len() {
            written[idx] = true;
        }
        true
    };

    let mut reachable_inst = vec![false; inst_count];
    for _iter in 0..3 {
        let mut block_in: Vec<Option<Vec<Option<RegType>>>> = vec![None; inst_count];
        let mut block_written: Vec<Option<Vec<bool>>> = vec![None; inst_count];
        if inst_count > 0 {
            let mut entry_ty = vec![None; num_regs];
            for &ref_reg in &compiled_fn.reference_registers {
                let idx = ref_reg as usize;
                if idx < num_regs {
                    entry_ty[idx] = Some(RegType::Ref);
                }
            }
            for (idx, &pty) in param_types.iter().enumerate() {
                if pty.is_some() {
                    entry_ty[idx] = pty;
                }
            }
            block_in[0] = Some(entry_ty);
            block_written[0] = Some(vec![false; param_count]);
        }
        let mut worklist: Vec<usize> = if inst_count > 0 { vec![0] } else { Vec::new() };
        let mut in_queue = vec![false; inst_count];
        if inst_count > 0 {
            in_queue[0] = true;
        }

        let join_block = |target: usize,
                          cur: &[Option<RegType>],
                          written: &[bool],
                          b_in: &mut [Option<Vec<Option<RegType>>>],
                          b_written: &mut [Option<Vec<bool>>],
                          wl: &mut Vec<usize>,
                          inq: &mut [bool]| {
            let mut changed = false;
            match (&mut b_in[target], &mut b_written[target]) {
                (None, _) => {
                    b_in[target] = Some(cur.to_vec());
                    b_written[target] = Some(written.to_vec());
                    changed = true;
                }
                (Some(ex_ty), Some(ex_wr)) => {
                    for (slot, &inc) in ex_ty.iter_mut().zip(cur.iter()) {
                        if slot.is_some() && *slot != inc {
                            *slot = None;
                            changed = true;
                        }
                    }
                    for (slot, &inc) in ex_wr.iter_mut().zip(written.iter()) {
                        if !*slot && inc {
                            *slot = true;
                            changed = true;
                        }
                    }
                }
                _ => {}
            }
            if changed && !inq[target] {
                inq[target] = true;
                wl.push(target);
            }
        };

        while let Some(block_pc) = worklist.pop() {
            in_queue[block_pc] = false;
            let Some(mut current_ty) = block_in[block_pc].clone() else {
                continue;
            };
            let Some(mut written_param) = block_written[block_pc].clone() else {
                continue;
            };
            for pc in block_pc..inst_count {
                if pc > block_pc && is_block_start[pc] {
                    join_block(
                        pc,
                        &current_ty,
                        &written_param,
                        &mut block_in,
                        &mut block_written,
                        &mut worklist,
                        &mut in_queue,
                    );
                    break;
                }
                reachable_inst[pc] = true;
                let inst = compiled_fn.instructions[pc];
                let opcode = OpCode::from_raw_u8(inst.raw_opcode())?;
                match opcode {
                    OpCode::Nop | OpCode::Jump => {}
                    OpCode::Mov => {
                        let src_idx = inst.src1() as usize;
                        let dst_idx = inst.dst() as usize;
                        if src_idx >= num_regs || dst_idx >= num_regs {
                            return None;
                        }
                        let chosen = current_ty[src_idx]
                            .or(inst_types[pc].dst_ty)
                            .or(current_ty[dst_idx]);
                        if let Some(reg_ty) = chosen {
                            inst_types[pc].src_ty = Some(reg_ty);
                            inst_types[pc].dst_ty = Some(reg_ty);
                            mark_read(
                                inst.src1(),
                                reg_ty,
                                &mut used_masks,
                                &mut current_ty,
                                &mut param_types,
                                &written_param,
                            );
                            mark_write(
                                inst.dst(),
                                reg_ty,
                                &mut used_masks,
                                &mut current_ty,
                                &mut written_param,
                            );
                        }
                    }
                    OpCode::LoadI32 | OpCode::LoadBool => {
                        if !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::LoadI64 => {
                        if !mark_write(
                            inst.dst(),
                            RegType::I64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::LoadF64 => {
                        inst_types[pc].dst_ty = Some(RegType::F64);
                        if !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::LoadNull => {
                        if !mark_write(
                            inst.dst(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::LoadConst => {
                        use solix_common::bytecode::ConstantPoolValue;
                        let const_ty =
                            match const_pool.and_then(|pool| pool.get(inst.imm32() as usize)) {
                                Some(ConstantPoolValue::Int64(_)) => RegType::I64,
                                Some(ConstantPoolValue::Float64(_)) => RegType::F64,
                                _ => RegType::Ref,
                            };
                        inst_types[pc].dst_ty = Some(const_ty);
                        if !mark_write(
                            inst.dst(),
                            const_ty,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddI32
                    | OpCode::SubI32
                    | OpCode::MulI32
                    | OpCode::DivI32
                    | OpCode::ModI32
                    | OpCode::Band
                    | OpCode::Bor
                    | OpCode::Bxor
                    | OpCode::Shl
                    | OpCode::Shr
                    | OpCode::Lshr => {
                        if !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::NegI32 | OpCode::Bnot | OpCode::Not => {
                        if !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddI32Imm
                    | OpCode::SubI32Imm
                    | OpCode::MulI32Imm
                    | OpCode::ModI32Imm => {
                        if !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddBandI32 => {
                        if !mark_read(
                            u16::from(inst.flags()),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddI64 | OpCode::SubI64 | OpCode::MulI64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::I64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::I64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddI64Imm => {
                        if !mark_read(
                            inst.src1(),
                            RegType::I64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddF64 | OpCode::SubF64 | OpCode::MulF64 | OpCode::DivF64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::NegF64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AddF64Imm | OpCode::SubF64Imm | OpCode::MulF64Imm => {
                        if !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::MulAddF64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            u16::from(inst.flags()),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::EqI32
                    | OpCode::NeqI32
                    | OpCode::LtI32
                    | OpCode::LteI32
                    | OpCode::GtI32
                    | OpCode::GteI32
                    | OpCode::EqBool
                    | OpCode::NeqBool => {
                        if !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::LtF64
                    | OpCode::LteF64
                    | OpCode::GtF64
                    | OpCode::GteF64
                    | OpCode::EqF64
                    | OpCode::NeqF64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                        let cond_ty = current_ty
                            .get(inst.dst() as usize)
                            .copied()
                            .flatten()
                            .unwrap_or(RegType::I32);
                        inst_types[pc].src_ty = Some(cond_ty);
                        if !mark_read(
                            inst.dst(),
                            cond_ty,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::JumpIfI32Imm => {
                        if !mark_read(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::JumpIfLtI32
                    | OpCode::JumpIfLteI32
                    | OpCode::JumpIfGtI32
                    | OpCode::JumpIfGteI32
                    | OpCode::JumpIfEqI32
                    | OpCode::JumpIfNeqI32 => {
                        if !mark_read(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::JumpIfLtF64 | OpCode::JumpIfLteF64 => {
                        if !mark_read(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::ForLoopI32 => {
                        if !mark_read(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                        if (inst.flags() & 0x80) != 0
                            && !mark_read(
                                u16::from(inst.flags() & 0x1F),
                                RegType::I32,
                                &mut used_masks,
                                &mut current_ty,
                                &mut param_types,
                                &written_param,
                            )
                        {
                            return None;
                        }
                        mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        );
                    }
                    OpCode::ConvIToF => {
                        let src_ty = current_ty
                            .get(inst.src1() as usize)
                            .copied()
                            .flatten()
                            .unwrap_or(RegType::I32);
                        inst_types[pc].src_ty = Some(src_ty);
                        if !mark_read(
                            inst.src1(),
                            src_ty,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::ConvFToI => {
                        let dst_ty = inst_types[pc].dst_ty.unwrap_or(RegType::I32);
                        inst_types[pc].dst_ty = Some(dst_ty);
                        if !mark_read(
                            inst.src1(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            dst_ty,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::AllocObject | OpCode::AllocArray => {
                        if opcode == OpCode::AllocArray
                            && !mark_read(
                                inst.src1(),
                                RegType::I32,
                                &mut used_masks,
                                &mut current_ty,
                                &mut param_types,
                                &written_param,
                            )
                        {
                            return None;
                        }
                        if !mark_write(
                            inst.dst(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::ArrayLength => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::GetFieldI32 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::GetFieldF64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::SetFieldI32 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::SetFieldF64 => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.dst(),
                            RegType::F64,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::IncRef | OpCode::DecRef => {
                        if !mark_read(
                            inst.dst(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::EqRef | OpCode::NeqRef | OpCode::EqString | OpCode::NeqString => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::GetField => {
                        if (inst.flags() & 1) != 0 {
                            return None;
                        }
                        let field_ty = match inst.flags() {
                            10 | 12 => RegType::I32,
                            14 | 16 => RegType::I64,
                            20 => RegType::F64,
                            24 => RegType::Ref,
                            _ => return None,
                        };
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_write(
                            inst.dst(),
                            field_ty,
                            &mut used_masks,
                            &mut current_ty,
                            &mut written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::SetField => {
                        let raw_flags = inst.flags();
                        if (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) != 0 {
                            return None;
                        }
                        let field_tag =
                            (raw_flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
                        let field_ty = match field_tag {
                            5 | 6 => RegType::I32,
                            7 | 8 => RegType::I64,
                            10 => RegType::F64,
                            12 => RegType::Ref,
                            _ => return None,
                        };
                        inst_types[pc].src_ty = Some(field_ty);
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.dst(),
                            field_ty,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::GetArray => {
                        if !mark_read(
                            inst.src1(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src2(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                        let dst_ty =
                            infer_next_consumer_type(&compiled_fn.instructions, pc, inst.dst())
                                .or(inst_types[pc].dst_ty);
                        if let Some(ty) = dst_ty {
                            inst_types[pc].dst_ty = Some(ty);
                            mark_write(
                                inst.dst(),
                                ty,
                                &mut used_masks,
                                &mut current_ty,
                                &mut written_param,
                            );
                        } else if let Some(slot) = current_ty.get_mut(inst.dst() as usize) {
                            *slot = None;
                        }
                    }
                    OpCode::SetArray => {
                        if !mark_read(
                            inst.dst(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                        if let Some(val_ty) =
                            current_ty.get(inst.src2() as usize).copied().flatten()
                        {
                            inst_types[pc].src_ty = Some(val_ty);
                            mark_read(
                                inst.src2(),
                                val_ty,
                                &mut used_masks,
                                &mut current_ty,
                                &mut param_types,
                                &written_param,
                            );
                        }
                    }
                    OpCode::GetArrayJumpIfFalse | OpCode::GetArrayJumpIfTrue => {
                        if !mark_read(
                            inst.dst(),
                            RegType::Ref,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) || !mark_read(
                            inst.src1(),
                            RegType::I32,
                            &mut used_masks,
                            &mut current_ty,
                            &mut param_types,
                            &written_param,
                        ) {
                            return None;
                        }
                    }
                    OpCode::CallDirect
                    | OpCode::TailCallDirect
                    | OpCode::CallNativeDirect
                    | OpCode::CallSymbol
                    | OpCode::CallVirtual
                    | OpCode::CallVirtualDirect
                    | OpCode::CallInterface => {
                        let is_method_call = matches!(
                            opcode,
                            OpCode::CallVirtual | OpCode::CallVirtualDirect | OpCode::CallInterface
                        );
                        let is_self_call =
                            matches!(opcode, OpCode::CallDirect | OpCode::TailCallDirect)
                                && u32::from(inst.call_target()) == func_id;
                        let arg_base = inst.arg_base() as usize;
                        let arg_count = inst.arg_count() as usize;
                        if arg_count > 4
                            || arg_base + arg_count > num_regs
                            || (is_method_call && arg_count == 0)
                        {
                            return None;
                        }
                        for arg_idx in 0..arg_count.min(4) {
                            let reg = (arg_base + arg_idx) as u16;
                            let inferred = if is_method_call && arg_idx == 0 {
                                Some(RegType::Ref)
                            } else {
                                current_ty[arg_base + arg_idx].or_else(|| {
                                    if is_self_call {
                                        param_types.get(arg_idx).copied().flatten()
                                    } else {
                                        None
                                    }
                                })
                            };
                            if let Some(ty) = inferred {
                                inst_types[pc].call_args[arg_idx] = Some(ty);
                                mark_read(
                                    reg,
                                    ty,
                                    &mut used_masks,
                                    &mut current_ty,
                                    &mut param_types,
                                    &written_param,
                                );
                            }
                        }
                        let call_dst = inst.call_dst() as usize;
                        if opcode != OpCode::TailCallDirect && call_dst < num_regs {
                            let dst_ty = infer_next_consumer_type(
                                &compiled_fn.instructions,
                                pc,
                                inst.call_dst(),
                            )
                            .or(inst_types[pc].dst_ty)
                            .or(if is_self_call {
                                return_type
                            } else {
                                None
                            });
                            if let Some(ty) = dst_ty {
                                inst_types[pc].dst_ty = Some(ty);
                                mark_write(
                                    inst.call_dst(),
                                    ty,
                                    &mut used_masks,
                                    &mut current_ty,
                                    &mut written_param,
                                );
                            } else {
                                current_ty[call_dst] = None;
                                if call_dst < written_param.len() {
                                    written_param[call_dst] = true;
                                }
                            }
                        }
                    }
                    OpCode::Return => {
                        has_return = true;
                    }
                    OpCode::ReturnValue => {
                        has_return = true;
                        let chosen = current_ty
                            .get(inst.dst() as usize)
                            .copied()
                            .flatten()
                            .or(return_type);
                        if let Some(ty) = chosen {
                            if let Some(prev) = return_type
                                && prev != ty
                            {
                                return None;
                            }
                            return_type = Some(ty);
                            inst_types[pc].src_ty = Some(ty);
                            mark_read(
                                inst.dst(),
                                ty,
                                &mut used_masks,
                                &mut current_ty,
                                &mut param_types,
                                &written_param,
                            );
                        }
                    }
                    _ => return None,
                }
                match opcode {
                    OpCode::Jump => {
                        let target = ((pc as i32) + inst.jump_offset()) as usize;
                        join_block(
                            target,
                            &current_ty,
                            &written_param,
                            &mut block_in,
                            &mut block_written,
                            &mut worklist,
                            &mut in_queue,
                        );
                        break;
                    }
                    OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                        let target = ((pc as i32) + inst.jump_offset()) as usize;
                        join_block(
                            target,
                            &current_ty,
                            &written_param,
                            &mut block_in,
                            &mut block_written,
                            &mut worklist,
                            &mut in_queue,
                        );
                        join_block(
                            pc + 1,
                            &current_ty,
                            &written_param,
                            &mut block_in,
                            &mut block_written,
                            &mut worklist,
                            &mut in_queue,
                        );
                        break;
                    }
                    OpCode::JumpIfI32Imm
                    | OpCode::JumpIfLtI32
                    | OpCode::JumpIfLteI32
                    | OpCode::JumpIfGtI32
                    | OpCode::JumpIfGteI32
                    | OpCode::JumpIfEqI32
                    | OpCode::JumpIfNeqI32
                    | OpCode::JumpIfLtF64
                    | OpCode::JumpIfLteF64
                    | OpCode::ForLoopI32
                    | OpCode::GetArrayJumpIfFalse
                    | OpCode::GetArrayJumpIfTrue => {
                        let target = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                        join_block(
                            target,
                            &current_ty,
                            &written_param,
                            &mut block_in,
                            &mut block_written,
                            &mut worklist,
                            &mut in_queue,
                        );
                        join_block(
                            pc + 1,
                            &current_ty,
                            &written_param,
                            &mut block_in,
                            &mut block_written,
                            &mut worklist,
                            &mut in_queue,
                        );
                        break;
                    }
                    OpCode::Return | OpCode::ReturnValue | OpCode::TailCallDirect => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    if !has_return {
        return None;
    }

    let resolved_params = param_types;

    for (pc, inst) in compiled_fn.instructions.iter().enumerate() {
        if !reachable_inst[pc] {
            continue;
        }
        let opcode = OpCode::from_raw_u8(inst.raw_opcode())?;
        match opcode {
            OpCode::Mov => {
                inst_types[pc].src_ty?;
                inst_types[pc].dst_ty?;
            }
            OpCode::GetArray => {
                inst_types[pc].dst_ty?;
            }
            OpCode::SetArray | OpCode::SetField | OpCode::ReturnValue => {
                inst_types[pc].src_ty?;
            }
            OpCode::CallDirect
            | OpCode::TailCallDirect
            | OpCode::CallNativeDirect
            | OpCode::CallSymbol
            | OpCode::CallVirtual
            | OpCode::CallVirtualDirect
            | OpCode::CallInterface => {
                let arg_count = inst.arg_count() as usize;
                for arg_ty in inst_types[pc].call_args.iter().take(arg_count.min(4)) {
                    (*arg_ty)?;
                }
                if opcode != OpCode::TailCallDirect
                    && inst_types[pc].dst_ty.is_none()
                    && is_reg_used_after(&compiled_fn.instructions, pc, inst.call_dst())
                {
                    return None;
                }
            }
            _ => {}
        }
    }

    Some(InferredFunctionTypes {
        used_masks,
        inst_types,
        func_sig: JitFuncSig {
            param_types: resolved_params,
            return_type,
        },
    })
}

fn infer_next_consumer_type(
    instructions: &[solix_common::bytecode::Instruction],
    def_pc: usize,
    mut reg: u16,
) -> Option<RegType> {
    for inst in instructions.iter().skip(def_pc + 1) {
        let opcode = OpCode::from_raw_u8(inst.raw_opcode())?;
        if opcode == OpCode::Mov && inst.src1() == reg {
            reg = inst.dst();
            continue;
        }
        match opcode {
            OpCode::AddI32
            | OpCode::SubI32
            | OpCode::MulI32
            | OpCode::DivI32
            | OpCode::ModI32
            | OpCode::Band
            | OpCode::Bor
            | OpCode::Bxor
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::Lshr
            | OpCode::EqI32
            | OpCode::NeqI32
            | OpCode::LtI32
            | OpCode::LteI32
            | OpCode::GtI32
            | OpCode::GteI32
            | OpCode::EqBool
            | OpCode::NeqBool
                if inst.src1() == reg || inst.src2() == reg =>
            {
                return Some(RegType::I32);
            }
            OpCode::AddI32Imm
            | OpCode::SubI32Imm
            | OpCode::MulI32Imm
            | OpCode::ModI32Imm
            | OpCode::NegI32
            | OpCode::Bnot
            | OpCode::Not
            | OpCode::AllocArray
                if inst.src1() == reg =>
            {
                return Some(RegType::I32);
            }
            OpCode::AddBandI32
                if inst.src1() == reg || inst.src2() == reg || u16::from(inst.flags()) == reg =>
            {
                return Some(RegType::I32);
            }
            OpCode::AddI64 | OpCode::SubI64 | OpCode::MulI64
                if inst.src1() == reg || inst.src2() == reg =>
            {
                return Some(RegType::I64);
            }
            OpCode::AddF64
            | OpCode::SubF64
            | OpCode::MulF64
            | OpCode::DivF64
            | OpCode::EqF64
            | OpCode::NeqF64
            | OpCode::LtF64
            | OpCode::LteF64
            | OpCode::GtF64
            | OpCode::GteF64
                if inst.src1() == reg || inst.src2() == reg =>
            {
                return Some(RegType::F64);
            }
            OpCode::AddF64Imm
            | OpCode::SubF64Imm
            | OpCode::MulF64Imm
            | OpCode::NegF64
            | OpCode::ConvFToI
                if inst.src1() == reg =>
            {
                return Some(RegType::F64);
            }
            OpCode::MulAddF64
                if inst.src1() == reg || inst.src2() == reg || u16::from(inst.flags()) == reg =>
            {
                return Some(RegType::F64);
            }
            OpCode::GetFieldI32
            | OpCode::GetFieldF64
            | OpCode::GetField
            | OpCode::ArrayLength
            | OpCode::SetFieldI32
            | OpCode::SetFieldF64
            | OpCode::SetField
            | OpCode::GetArray
                if inst.src1() == reg =>
            {
                return Some(RegType::Ref);
            }
            OpCode::GetArray if inst.src2() == reg => return Some(RegType::I32),
            OpCode::SetArray | OpCode::GetArrayJumpIfFalse | OpCode::GetArrayJumpIfTrue
                if inst.dst() == reg =>
            {
                return Some(RegType::Ref);
            }
            OpCode::SetArray | OpCode::GetArrayJumpIfFalse | OpCode::GetArrayJumpIfTrue
                if inst.src1() == reg =>
            {
                return Some(RegType::I32);
            }
            OpCode::IncRef | OpCode::DecRef if inst.dst() == reg => return Some(RegType::Ref),
            OpCode::SetFieldI32 if inst.dst() == reg => return Some(RegType::I32),
            OpCode::SetFieldF64 if inst.dst() == reg => return Some(RegType::F64),
            OpCode::SetField if inst.dst() == reg => {
                let tag = (inst.flags() & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
                return match tag {
                    5 | 6 => Some(RegType::I32),
                    7 | 8 => Some(RegType::I64),
                    10 => Some(RegType::F64),
                    12 => Some(RegType::Ref),
                    _ => None,
                };
            }
            OpCode::ForLoopI32 if inst.dst() == reg || inst.src1() == reg => {
                return Some(RegType::I32);
            }
            OpCode::JumpIfLtI32
            | OpCode::JumpIfLteI32
            | OpCode::JumpIfGtI32
            | OpCode::JumpIfGteI32
            | OpCode::JumpIfEqI32
            | OpCode::JumpIfNeqI32
                if inst.dst() == reg || inst.src1() == reg =>
            {
                return Some(RegType::I32);
            }
            OpCode::JumpIfI32Imm if inst.dst() == reg => return Some(RegType::I32),
            OpCode::JumpIfLtF64 | OpCode::JumpIfLteF64
                if inst.dst() == reg || inst.src1() == reg =>
            {
                return Some(RegType::F64);
            }
            OpCode::EqRef | OpCode::NeqRef | OpCode::EqString | OpCode::NeqString
                if inst.src1() == reg || inst.src2() == reg =>
            {
                return Some(RegType::Ref);
            }
            OpCode::CallVirtual | OpCode::CallVirtualDirect | OpCode::CallInterface
                if inst.arg_base() == reg && inst.arg_count() >= 1 =>
            {
                return Some(RegType::Ref);
            }
            OpCode::Jump | OpCode::Return | OpCode::ReturnValue | OpCode::TailCallDirect => {
                return None;
            }
            OpCode::Mov
            | OpCode::LoadI32
            | OpCode::LoadI64
            | OpCode::LoadBool
            | OpCode::LoadNull
            | OpCode::LoadF64
            | OpCode::LoadConst
            | OpCode::AddI32
            | OpCode::SubI32
            | OpCode::MulI32
            | OpCode::DivI32
            | OpCode::ModI32
            | OpCode::Band
            | OpCode::Bor
            | OpCode::Bxor
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::Lshr
            | OpCode::NegI32
            | OpCode::Bnot
            | OpCode::Not
            | OpCode::AddI32Imm
            | OpCode::SubI32Imm
            | OpCode::MulI32Imm
            | OpCode::ModI32Imm
            | OpCode::AddBandI32
            | OpCode::AddI64
            | OpCode::SubI64
            | OpCode::MulI64
            | OpCode::AddI64Imm
            | OpCode::AddF64
            | OpCode::SubF64
            | OpCode::MulF64
            | OpCode::DivF64
            | OpCode::NegF64
            | OpCode::AddF64Imm
            | OpCode::SubF64Imm
            | OpCode::MulF64Imm
            | OpCode::MulAddF64
            | OpCode::EqI32
            | OpCode::NeqI32
            | OpCode::LtI32
            | OpCode::LteI32
            | OpCode::GtI32
            | OpCode::GteI32
            | OpCode::EqBool
            | OpCode::NeqBool
            | OpCode::EqF64
            | OpCode::NeqF64
            | OpCode::LtF64
            | OpCode::LteF64
            | OpCode::GtF64
            | OpCode::GteF64
            | OpCode::ConvIToF
            | OpCode::ConvFToI
            | OpCode::AllocObject
            | OpCode::AllocArray
            | OpCode::GetFieldI32
            | OpCode::GetFieldF64
            | OpCode::GetField
            | OpCode::EqRef
            | OpCode::NeqRef
            | OpCode::EqString
            | OpCode::NeqString
            | OpCode::ArrayLength
            | OpCode::GetArray
                if inst.dst() == reg =>
            {
                return None;
            }
            _ => {}
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn translate_function(
    compiled_fn: &CompiledFunction,
    binary_id: u32,
    const_pool: Option<&solix_common::bytecode::ConstantPool>,
    const_pool_cache: Option<&[crate::Value]>,
    monomorphic_caches: Option<&[Option<crate::vm::engine::MonomorphicCallSite>]>,
    types_info: &InferredFunctionTypes,
    self_clif_id: FuncId,
    clif_func: &mut cranelift_codegen::ir::Function,
    fb_ctx: &mut FunctionBuilderContext,
    engine: &mut crate::vm::jit::JitEngine,
) -> Result<(), ()> {
    let ptr_ty = engine.module.target_config().pointer_type();
    let inst_count = compiled_fn.instructions.len();
    if inst_count == 0 {
        return Err(());
    }

    let mut block_pcs: Vec<usize> = vec![0];
    for (pc, inst) in compiled_fn.instructions.iter().enumerate() {
        let Some(opcode) = OpCode::from_raw_u8(inst.raw_opcode()) else {
            return Err(());
        };
        match opcode {
            OpCode::Jump => {
                let target = ((pc as i32) + inst.jump_offset()) as usize;
                block_pcs.push(target);
                if pc + 1 < inst_count {
                    block_pcs.push(pc + 1);
                }
            }
            OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                let target = ((pc as i32) + inst.jump_offset()) as usize;
                block_pcs.push(target);
                if pc + 1 < inst_count {
                    block_pcs.push(pc + 1);
                }
            }
            OpCode::JumpIfI32Imm
            | OpCode::JumpIfLtI32
            | OpCode::JumpIfLteI32
            | OpCode::JumpIfGtI32
            | OpCode::JumpIfGteI32
            | OpCode::JumpIfEqI32
            | OpCode::JumpIfNeqI32
            | OpCode::JumpIfLtF64
            | OpCode::JumpIfLteF64
            | OpCode::GetArrayJumpIfFalse
            | OpCode::GetArrayJumpIfTrue
            | OpCode::ForLoopI32 => {
                let target = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                block_pcs.push(target);
                if pc + 1 < inst_count {
                    block_pcs.push(pc + 1);
                }
            }
            OpCode::Return | OpCode::ReturnValue if pc + 1 < inst_count => {
                block_pcs.push(pc + 1);
            }
            _ => {}
        }
    }
    block_pcs.sort_unstable();
    block_pcs.dedup();

    let mut builder = FunctionBuilder::new(clif_func, fb_ctx);
    let mut blocks_by_pc: HashMap<usize, Block> = HashMap::new();
    for &b_pc in &block_pcs {
        if b_pc < inst_count {
            blocks_by_pc.insert(b_pc, builder.create_block());
        }
    }

    let fn_entry = builder.create_block();
    builder.append_block_params_for_function_params(fn_entry);
    builder.switch_to_block(fn_entry);

    let ctx_ptr = builder.block_params(fn_entry)[0];
    let mut vars = VarTable {
        slots: vec![[None; 4]; types_info.used_masks.len()],
    };
    let all_reg_types = [RegType::I32, RegType::I64, RegType::F64, RegType::Ref];
    for (reg_idx, &mask) in types_info.used_masks.iter().enumerate() {
        let param_ty = types_info
            .func_sig
            .param_types
            .get(reg_idx)
            .copied()
            .flatten();
        let effective_mask = if let Some(pty) = param_ty {
            mask | (1u8 << pty.to_code())
        } else {
            mask
        };
        for reg_ty in all_reg_types {
            let code = reg_ty.to_code() as usize;
            if (effective_mask & (1u8 << code)) == 0 {
                continue;
            }
            let var = Variable::from_u32((reg_idx * 4 + code) as u32);
            builder.declare_var(var, reg_type_to_clif(reg_ty));
            if param_ty == Some(reg_ty) {
                let raw_arg = builder.block_params(fn_entry)[1 + reg_idx];
                let typed_arg = match reg_ty {
                    RegType::I32 | RegType::Ref => builder.ins().ireduce(types::I32, raw_arg),
                    RegType::I64 => raw_arg,
                    RegType::F64 => builder.ins().bitcast(types::F64, MemFlags::new(), raw_arg),
                };
                builder.def_var(var, typed_arg);
            } else {
                let zero_val = match reg_ty {
                    RegType::I32 | RegType::Ref => builder.ins().iconst(types::I32, 0),
                    RegType::I64 => builder.ins().iconst(types::I64, 0),
                    RegType::F64 => builder.ins().f64const(0.0),
                };
                builder.def_var(var, zero_val);
            }
            vars.slots[reg_idx][code] = Some(var);
        }
    }
    let body_entry_block = blocks_by_pc[&0];
    builder.ins().jump(body_entry_block, &[]);
    builder.switch_to_block(body_entry_block);

    let self_func_ref = engine
        .module
        .declare_func_in_func(self_clif_id, builder.func);
    let rt_load_const_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_load_const, builder.func);
    let rt_alloc_obj_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_alloc_object, builder.func);
    let rt_alloc_arr_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_alloc_array, builder.func);
    let rt_inc_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_inc_ref, builder.func);
    let rt_dec_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_dec_ref, builder.func);
    let rt_call_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_call_slow, builder.func);
    let rt_call_symbol_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_call_symbol_slow, builder.func);
    let rt_call_virtual_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_call_virtual_slow, builder.func);
    let rt_call_interface_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_call_interface_slow, builder.func);
    let rt_resolve_jit_virtual_fn_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_resolve_jit_virtual_fn, builder.func);
    let rt_resolve_jit_interface_fn_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_resolve_jit_interface_fn, builder.func);
    let jit_call_sig_ref = {
        let mut sig = Signature::new(engine.module.isa().default_call_conv());
        sig.params.push(AbiParam::new(ptr_ty));
        for _ in 0..4 {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        builder.import_signature(sig)
    };
    let rt_set_field_ref_fn = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_set_field_ref, builder.func);
    let rt_get_array_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_get_array_slow, builder.func);
    let rt_set_array_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_set_array_slow, builder.func);
    let rt_string_eq_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_string_eq, builder.func);
    let rt_trap_null_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_trap_null, builder.func);
    let rt_trap_div_zero_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_trap_div_zero, builder.func);
    let rt_trap_oob_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_trap_oob, builder.func);

    let mut block_terminated = false;
    let can_realloc = function_can_reallocate_heap(&compiled_fn.instructions, types_info);
    let hoisted_heap_base = if can_realloc {
        None
    } else {
        Some(builder.ins().load(ptr_ty, MemFlags::new(), ctx_ptr, 0))
    };
    let mut opt_state = BlockOptState::new(hoisted_heap_base);

    for (pc, inst) in compiled_fn.instructions.iter().enumerate() {
        if pc > 0
            && let Some(&next_blk) = blocks_by_pc.get(&pc)
        {
            if !block_terminated {
                builder.ins().jump(next_blk, &[]);
            }
            builder.switch_to_block(next_blk);
            block_terminated = false;
            opt_state.reset_block();
        }
        if block_terminated {
            continue;
        }

        let opcode = OpCode::from_raw_u8(inst.raw_opcode()).ok_or(())?;
        let inst_ty = types_info.inst_types[pc];
        match opcode {
            OpCode::Nop => {}
            OpCode::Mov => {
                let mov_ty = inst_ty.dst_ty.unwrap_or(RegType::I32);
                let val = builder.use_var(vars.get(inst.src1(), mov_ty));
                builder.def_var(vars.get(inst.dst(), mov_ty), val);
                if mov_ty == RegType::Ref {
                    let src_non_null = opt_state.is_non_null(inst.src1());
                    opt_state.set_non_null(inst.dst(), src_non_null);
                } else {
                    opt_state.clear_non_null(inst.dst());
                }
            }
            OpCode::LoadI32 => {
                let val = builder
                    .ins()
                    .iconst(types::I32, i64::from(inst.imm32_signed()));
                builder.def_var(vars.get(inst.dst(), RegType::I32), val);
            }
            OpCode::LoadI64 => {
                let val = builder
                    .ins()
                    .iconst(types::I64, i64::from(inst.imm32_signed()));
                builder.def_var(vars.get(inst.dst(), RegType::I64), val);
            }
            OpCode::LoadBool => {
                let val = builder
                    .ins()
                    .iconst(types::I32, if inst.imm32() != 0 { 1 } else { 0 });
                builder.def_var(vars.get(inst.dst(), RegType::I32), val);
            }
            OpCode::LoadNull => {
                let val = builder.ins().iconst(types::I32, 0);
                builder.def_var(vars.get(inst.dst(), RegType::Ref), val);
            }
            OpCode::LoadF64 => {
                use solix_common::bytecode::ConstantPoolValue;
                if let Some(ConstantPoolValue::Float64(bits)) =
                    const_pool.and_then(|pool| pool.get(inst.imm32() as usize))
                {
                    let val = builder.ins().f64const(f64::from_bits(*bits));
                    builder.def_var(vars.get(inst.dst(), RegType::F64), val);
                } else {
                    let bin_arg = builder.ins().iconst(types::I32, i64::from(binary_id));
                    let idx_arg = builder.ins().iconst(types::I32, i64::from(inst.imm32()));
                    let call = builder
                        .ins()
                        .call(rt_load_const_ref, &[ctx_ptr, bin_arg, idx_arg]);
                    let raw_word = builder.inst_results(call)[0];
                    let typed = decode_raw_word(&mut builder, raw_word, RegType::F64);
                    builder.def_var(vars.get(inst.dst(), RegType::F64), typed);
                }
            }
            OpCode::LoadConst => {
                use solix_common::bytecode::ConstantPoolValue;
                let dst_ty = inst_ty.dst_ty.unwrap_or(RegType::I64);
                if dst_ty == RegType::Ref
                    && let Some(crate::Value::Ref(addr)) =
                        const_pool_cache.and_then(|cache| cache.get(inst.imm32() as usize))
                {
                    let val = builder.ins().iconst(types::I32, i64::from(*addr));
                    builder.def_var(vars.get(inst.dst(), RegType::Ref), val);
                } else {
                    match const_pool.and_then(|pool| pool.get(inst.imm32() as usize)) {
                        Some(ConstantPoolValue::Int64(num)) if dst_ty == RegType::I64 => {
                            let val = builder.ins().iconst(types::I64, *num);
                            builder.def_var(vars.get(inst.dst(), RegType::I64), val);
                        }
                        Some(ConstantPoolValue::Float64(bits)) if dst_ty == RegType::F64 => {
                            let val = builder.ins().f64const(f64::from_bits(*bits));
                            builder.def_var(vars.get(inst.dst(), RegType::F64), val);
                        }
                        _ => {
                            let bin_arg = builder.ins().iconst(types::I32, i64::from(binary_id));
                            let idx_arg = builder.ins().iconst(types::I32, i64::from(inst.imm32()));
                            let call = builder
                                .ins()
                                .call(rt_load_const_ref, &[ctx_ptr, bin_arg, idx_arg]);
                            let raw_word = builder.inst_results(call)[0];
                            let typed = decode_raw_word(&mut builder, raw_word, dst_ty);
                            builder.def_var(vars.get(inst.dst(), dst_ty), typed);
                        }
                    }
                }
            }
            OpCode::AddI32 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().iadd(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::SubI32 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().isub(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::MulI32 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().imul(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::DivI32 | OpCode::ModI32 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let is_zero = builder.ins().icmp_imm(IntCC::Equal, rhs, 0);
                let trap_blk = builder.create_block();
                let ok_blk = builder.create_block();
                builder.ins().brif(is_zero, trap_blk, &[], ok_blk, &[]);
                builder.switch_to_block(trap_blk);
                builder.ins().call(rt_trap_div_zero_ref, &[ctx_ptr]);
                let zero_ret = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero_ret]);
                builder.switch_to_block(ok_blk);
                let res = if opcode == OpCode::DivI32 {
                    builder.ins().sdiv(lhs, rhs)
                } else {
                    builder.ins().srem(lhs, rhs)
                };
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Band => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().band(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Bor => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().bor(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Bxor => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().bxor(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Shl => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().ishl(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Shr => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().sshr(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Lshr => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let res = builder.ins().ushr(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::NegI32 => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let res = builder.ins().ineg(src);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Bnot => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let res = builder.ins().bnot(src);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::Not => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let is_zero = builder.ins().icmp_imm(IntCC::Equal, src, 0);
                let res = builder.ins().uextend(types::I32, is_zero);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::AddI32Imm => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let res = builder.ins().iadd_imm(src, i64::from(inst.src2() as i16));
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::SubI32Imm => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let res = builder.ins().iadd_imm(src, -i64::from(inst.src2() as i16));
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::MulI32Imm => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let res = builder.ins().imul_imm(src, i64::from(inst.src2() as i16));
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::ModI32Imm => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let divisor = inst.src2() as i16 as i32;
                if divisor == 0 {
                    builder.ins().call(rt_trap_div_zero_ref, &[ctx_ptr]);
                    let zero_ret = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero_ret]);
                    block_terminated = true;
                } else if divisor == 1 || divisor == -1 {
                    let zero = builder.ins().iconst(types::I32, 0);
                    builder.def_var(vars.get(inst.dst(), RegType::I32), zero);
                } else if divisor > 0 && (divisor as u32).is_power_of_two() {
                    let shift = i64::from((divisor as u32).trailing_zeros());
                    let mask = i64::from(divisor - 1);
                    let sign = builder.ins().sshr_imm(src, 31);
                    let adjust = builder.ins().ushr_imm(sign, 32 - shift);
                    let biased = builder.ins().iadd(src, adjust);
                    let masked = builder.ins().band_imm(biased, mask);
                    let res = builder.ins().isub(masked, adjust);
                    builder.def_var(vars.get(inst.dst(), RegType::I32), res);
                } else {
                    let (magic, shift) = signed_div_magic_i32(divisor);
                    let magic_val = builder.ins().iconst(types::I32, i64::from(magic));
                    let mut quot = builder.ins().smulhi(src, magic_val);
                    if divisor > 0 && magic < 0 {
                        quot = builder.ins().iadd(quot, src);
                    } else if divisor < 0 && magic > 0 {
                        quot = builder.ins().isub(quot, src);
                    }
                    if shift > 0 {
                        quot = builder.ins().sshr_imm(quot, i64::from(shift));
                    }
                    let sign_bit = builder.ins().ushr_imm(quot, 31);
                    let trunc_quot = builder.ins().iadd(quot, sign_bit);
                    let prod = builder.ins().imul_imm(trunc_quot, i64::from(divisor));
                    let res = builder.ins().isub(src, prod);
                    builder.def_var(vars.get(inst.dst(), RegType::I32), res);
                }
            }
            OpCode::AddBandI32 => {
                let acc = builder.use_var(vars.get(u16::from(inst.flags()), RegType::I32));
                let val = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let mask = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let sum = builder.ins().iadd(acc, val);
                let masked = builder.ins().band(sum, mask);
                builder.def_var(vars.get(inst.dst(), RegType::I32), masked);
            }
            OpCode::AddI64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I64));
                let res = builder.ins().iadd(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I64), res);
            }
            OpCode::SubI64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I64));
                let res = builder.ins().isub(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I64), res);
            }
            OpCode::MulI64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I64));
                let res = builder.ins().imul(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::I64), res);
            }
            OpCode::AddI64Imm => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::I64));
                let res = builder.ins().iadd_imm(src, i64::from(inst.src2() as i16));
                builder.def_var(vars.get(inst.dst(), RegType::I64), res);
            }
            OpCode::AddF64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::F64));
                let res = builder.ins().fadd(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::SubF64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::F64));
                let res = builder.ins().fsub(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::MulF64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::F64));
                let res = builder.ins().fmul(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::DivF64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::F64));
                let res = builder.ins().fdiv(lhs, rhs);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::NegF64 => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let res = builder.ins().fneg(src);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::AddF64Imm | OpCode::SubF64Imm | OpCode::MulF64Imm => {
                use solix_common::bytecode::ConstantPoolValue;
                let const_f64 = if let Some(ConstantPoolValue::Float64(bits)) =
                    const_pool.and_then(|pool| pool.get(inst.src2() as usize))
                {
                    builder.ins().f64const(f64::from_bits(*bits))
                } else {
                    let bin_arg = builder.ins().iconst(types::I32, i64::from(binary_id));
                    let idx_arg = builder.ins().iconst(types::I32, i64::from(inst.src2()));
                    let call = builder
                        .ins()
                        .call(rt_load_const_ref, &[ctx_ptr, bin_arg, idx_arg]);
                    let raw_word = builder.inst_results(call)[0];
                    builder.ins().bitcast(types::F64, MemFlags::new(), raw_word)
                };
                let src_f64 = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let res = match opcode {
                    OpCode::AddF64Imm => builder.ins().fadd(src_f64, const_f64),
                    OpCode::SubF64Imm => builder.ins().fsub(src_f64, const_f64),
                    _ => builder.ins().fmul(src_f64, const_f64),
                };
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::MulAddF64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::F64));
                let acc = builder.use_var(vars.get(u16::from(inst.flags()), RegType::F64));
                let prod = builder.ins().fmul(lhs, rhs);
                let res = builder.ins().fadd(prod, acc);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::EqI32
            | OpCode::NeqI32
            | OpCode::LtI32
            | OpCode::LteI32
            | OpCode::GtI32
            | OpCode::GteI32
            | OpCode::EqBool
            | OpCode::NeqBool => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let cc = match opcode {
                    OpCode::EqI32 | OpCode::EqBool => IntCC::Equal,
                    OpCode::NeqI32 | OpCode::NeqBool => IntCC::NotEqual,
                    OpCode::LtI32 => IntCC::SignedLessThan,
                    OpCode::LteI32 => IntCC::SignedLessThanOrEqual,
                    OpCode::GtI32 => IntCC::SignedGreaterThan,
                    _ => IntCC::SignedGreaterThanOrEqual,
                };
                let cmp = builder.ins().icmp(cc, lhs, rhs);
                let res = builder.ins().uextend(types::I32, cmp);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::EqF64
            | OpCode::NeqF64
            | OpCode::LtF64
            | OpCode::LteF64
            | OpCode::GtF64
            | OpCode::GteF64 => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::F64));
                let cc = match opcode {
                    OpCode::EqF64 => FloatCC::Equal,
                    OpCode::NeqF64 => FloatCC::NotEqual,
                    OpCode::LtF64 => FloatCC::LessThan,
                    OpCode::LteF64 => FloatCC::LessThanOrEqual,
                    OpCode::GtF64 => FloatCC::GreaterThan,
                    _ => FloatCC::GreaterThanOrEqual,
                };
                let cmp = builder.ins().fcmp(cc, lhs, rhs);
                let res = builder.ins().uextend(types::I32, cmp);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::ConvIToF => {
                let src_ty = inst_ty.src_ty.unwrap_or(RegType::I32);
                let src = builder.use_var(vars.get(inst.src1(), src_ty));
                let res = builder.ins().fcvt_from_sint(types::F64, src);
                builder.def_var(vars.get(inst.dst(), RegType::F64), res);
            }
            OpCode::ConvFToI => {
                let src = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let dst_ty = inst_ty.dst_ty.unwrap_or(RegType::I32);
                let res = builder
                    .ins()
                    .fcvt_to_sint_sat(reg_type_to_clif(dst_ty), src);
                builder.def_var(vars.get(inst.dst(), dst_ty), res);
            }
            OpCode::Jump => {
                let target_pc = ((pc as i32) + inst.jump_offset()) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                builder.ins().jump(target_blk, &[]);
                block_terminated = true;
            }
            OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                let cond_ty = inst_ty.src_ty.unwrap_or(RegType::I32);
                let cond = builder.use_var(vars.get(inst.dst(), cond_ty));
                let cc = if opcode == OpCode::JumpIfTrue {
                    IntCC::NotEqual
                } else {
                    IntCC::Equal
                };
                let taken = builder.ins().icmp_imm(cc, cond, 0);
                let target_pc = ((pc as i32) + inst.jump_offset()) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                let next_blk = blocks_by_pc[&(pc + 1)];
                builder.ins().brif(taken, target_blk, &[], next_blk, &[]);
                block_terminated = true;
            }
            OpCode::JumpIfI32Imm => {
                let val = builder.use_var(vars.get(inst.dst(), RegType::I32));
                let imm = i64::from(inst.src1() as i16);
                let cc = match inst.flags() {
                    0 => IntCC::Equal,
                    1 => IntCC::NotEqual,
                    2 => IntCC::SignedLessThan,
                    3 => IntCC::SignedLessThanOrEqual,
                    4 => IntCC::SignedGreaterThan,
                    _ => IntCC::SignedGreaterThanOrEqual,
                };
                let taken = builder.ins().icmp_imm(cc, val, imm);
                let target_pc = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                let next_blk = blocks_by_pc[&(pc + 1)];
                builder.ins().brif(taken, target_blk, &[], next_blk, &[]);
                block_terminated = true;
            }
            OpCode::JumpIfLtI32
            | OpCode::JumpIfLteI32
            | OpCode::JumpIfGtI32
            | OpCode::JumpIfGteI32
            | OpCode::JumpIfEqI32
            | OpCode::JumpIfNeqI32 => {
                let lhs = builder.use_var(vars.get(inst.dst(), RegType::I32));
                let rhs = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let cc = match opcode {
                    OpCode::JumpIfLtI32 => IntCC::SignedLessThan,
                    OpCode::JumpIfLteI32 => IntCC::SignedLessThanOrEqual,
                    OpCode::JumpIfGtI32 => IntCC::SignedGreaterThan,
                    OpCode::JumpIfGteI32 => IntCC::SignedGreaterThanOrEqual,
                    OpCode::JumpIfEqI32 => IntCC::Equal,
                    _ => IntCC::NotEqual,
                };
                let taken = builder.ins().icmp(cc, lhs, rhs);
                let target_pc = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                let next_blk = blocks_by_pc[&(pc + 1)];
                builder.ins().brif(taken, target_blk, &[], next_blk, &[]);
                block_terminated = true;
            }
            OpCode::JumpIfLtF64 | OpCode::JumpIfLteF64 => {
                let lhs = builder.use_var(vars.get(inst.dst(), RegType::F64));
                let rhs = builder.use_var(vars.get(inst.src1(), RegType::F64));
                let cc = if opcode == OpCode::JumpIfLtF64 {
                    FloatCC::LessThan
                } else {
                    FloatCC::LessThanOrEqual
                };
                let taken = builder.ins().fcmp(cc, lhs, rhs);
                let target_pc = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                let next_blk = blocks_by_pc[&(pc + 1)];
                builder.ins().brif(taken, target_blk, &[], next_blk, &[]);
                block_terminated = true;
            }
            OpCode::ForLoopI32 => {
                let flags = inst.flags();
                let step = if (flags & 0x80) == 0 {
                    builder.ins().iconst(types::I32, 1)
                } else {
                    builder.use_var(vars.get(u16::from(flags & 0x1F), RegType::I32))
                };
                let cur_idx = builder.use_var(vars.get(inst.dst(), RegType::I32));
                let next_idx = builder.ins().iadd(cur_idx, step);
                builder.def_var(vars.get(inst.dst(), RegType::I32), next_idx);
                let bound = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let cur_i64 = builder.ins().sextend(types::I64, cur_idx);
                let step_i64 = builder.ins().sextend(types::I64, step);
                let next_i64 = builder.ins().iadd(cur_i64, step_i64);
                let bound_i64 = builder.ins().sextend(types::I64, bound);
                let cc = if (flags & 0x40) == 0 {
                    IntCC::SignedLessThan
                } else {
                    IntCC::SignedLessThanOrEqual
                };
                let keep_going = builder.ins().icmp(cc, next_i64, bound_i64);
                let target_pc = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                let next_blk = blocks_by_pc[&(pc + 1)];
                builder
                    .ins()
                    .brif(keep_going, target_blk, &[], next_blk, &[]);
                block_terminated = true;
            }
            OpCode::AllocObject => {
                use crate::memory::{BlockHeader, BlockTag};
                use crate::vm::jit::JitContext;

                let payload_size = inst.src2() as usize;
                let aligned_payload = ((payload_size + 7) & !7).max(8);
                let required_block = 8 + aligned_payload;
                let tag = if inst.is_pod() {
                    BlockTag::ObjectPod
                } else {
                    BlockTag::Object
                };
                let header_u64 =
                    u64::from_le_bytes(BlockHeader::new(payload_size as u32, tag, 1).to_bytes());

                let cursor_off = std::mem::offset_of!(JitContext, bump_cursor) as i32;
                let limit_off = std::mem::offset_of!(JitContext, bump_limit) as i32;
                let static_off = std::mem::offset_of!(JitContext, static_size) as i32;
                let heap_off = std::mem::offset_of!(JitContext, heap_ptr) as i32;
                let count_off = std::mem::offset_of!(JitContext, jit_alloc_count) as i32;
                let bytes_off = std::mem::offset_of!(JitContext, jit_alloc_bytes) as i32;

                let cursor = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, cursor_off);
                let limit = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, limit_off);
                let new_cursor = builder.ins().iadd_imm(cursor, required_block as i64);
                let fits = builder
                    .ins()
                    .icmp(IntCC::UnsignedLessThanOrEqual, new_cursor, limit);

                let fast_blk = builder.create_block();
                let slow_blk = builder.create_block();
                let merge_blk = builder.create_block();
                builder.append_block_param(merge_blk, types::I32);
                builder.ins().brif(fits, fast_blk, &[], slow_blk, &[]);

                builder.switch_to_block(fast_blk);
                builder
                    .ins()
                    .store(MemFlags::new(), new_cursor, ctx_ptr, cursor_off);
                let cur_cnt = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, count_off);
                let new_cnt = builder.ins().iadd_imm(cur_cnt, 1);
                builder
                    .ins()
                    .store(MemFlags::new(), new_cnt, ctx_ptr, count_off);
                let cur_bytes = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, bytes_off);
                let new_bytes = builder.ins().iadd_imm(cur_bytes, required_block as i64);
                builder
                    .ins()
                    .store(MemFlags::new(), new_bytes, ctx_ptr, bytes_off);

                let static_sz =
                    builder
                        .ins()
                        .load(types::I64, MemFlags::new(), ctx_ptr, static_off);
                let block_abs = builder.ins().iadd(static_sz, cursor);
                let heap_base = builder
                    .ins()
                    .load(ptr_ty, MemFlags::new(), ctx_ptr, heap_off);
                let block_ptr = builder.ins().iadd(heap_base, block_abs);
                let hdr_const = builder.ins().iconst(types::I64, header_u64 as i64);
                builder
                    .ins()
                    .store(MemFlags::new(), hdr_const, block_ptr, 0);
                let vtable_const = builder.ins().iconst(types::I32, i64::from(inst.src1()));
                builder
                    .ins()
                    .store(MemFlags::new(), vtable_const, block_ptr, 8);
                let payload_abs = builder.ins().iadd_imm(block_abs, 8);
                let payload_i32 = builder.ins().ireduce(types::I32, payload_abs);
                builder.ins().jump(merge_blk, &[payload_i32]);

                builder.switch_to_block(slow_blk);
                let vtable = builder.ins().iconst(types::I32, i64::from(inst.src1()));
                let size = builder.ins().iconst(types::I32, i64::from(inst.src2()));
                let is_pod = builder
                    .ins()
                    .iconst(types::I8, if inst.is_pod() { 1 } else { 0 });
                let call = builder
                    .ins()
                    .call(rt_alloc_obj_ref, &[ctx_ptr, vtable, size, is_pod]);
                let slow_addr = builder.inst_results(call)[0];
                emit_trap_return_check(&mut builder, ctx_ptr);
                builder.ins().jump(merge_blk, &[slow_addr]);

                builder.switch_to_block(merge_blk);
                let merged_addr = builder.block_params(merge_blk)[0];
                builder.def_var(vars.get(inst.dst(), RegType::Ref), merged_addr);
                opt_state.invalidate_heap();
                opt_state.set_non_null(inst.dst(), true);
            }
            OpCode::AllocArray => {
                use crate::memory::{BlockTag, GcColor};
                use crate::vm::jit::JitContext;

                let raw_len = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let type_tag = inst.flags();
                let (elem_size, vtable_id) = if type_tag >= 0x80 {
                    (8usize, inst.src2())
                } else {
                    ((inst.src2() as usize).max(1), 0u16)
                };

                let non_neg = builder
                    .ins()
                    .icmp_imm(IntCC::SignedGreaterThanOrEqual, raw_len, 0);
                let len_i64 = builder.ins().uextend(types::I64, raw_len);
                let elem_bytes = builder.ins().imul_imm(len_i64, elem_size as i64);
                let payload_size = builder.ins().iadd_imm(elem_bytes, 8);
                let aligned_plus = builder.ins().iadd_imm(payload_size, 7);
                let aligned_payload = builder.ins().band_imm(aligned_plus, -8);
                let required_block = builder.ins().iadd_imm(aligned_payload, 8);

                let cursor_off = std::mem::offset_of!(JitContext, bump_cursor) as i32;
                let limit_off = std::mem::offset_of!(JitContext, bump_limit) as i32;
                let static_off = std::mem::offset_of!(JitContext, static_size) as i32;
                let heap_off = std::mem::offset_of!(JitContext, heap_ptr) as i32;
                let count_off = std::mem::offset_of!(JitContext, jit_alloc_count) as i32;
                let bytes_off = std::mem::offset_of!(JitContext, jit_alloc_bytes) as i32;

                let cursor = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, cursor_off);
                let limit = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, limit_off);
                let new_cursor = builder.ins().iadd(cursor, required_block);
                let fits_bump =
                    builder
                        .ins()
                        .icmp(IntCC::UnsignedLessThanOrEqual, new_cursor, limit);
                let fits_len = builder.ins().icmp_imm(
                    IntCC::UnsignedLessThanOrEqual,
                    payload_size,
                    i64::from(u32::MAX),
                );
                let can_fast_step1 = builder.ins().band(non_neg, fits_bump);
                let can_fast = builder.ins().band(can_fast_step1, fits_len);

                let fast_blk = builder.create_block();
                let slow_blk = builder.create_block();
                let merge_blk = builder.create_block();
                builder.append_block_param(merge_blk, types::I32);
                builder.ins().brif(can_fast, fast_blk, &[], slow_blk, &[]);

                builder.switch_to_block(fast_blk);
                builder
                    .ins()
                    .store(MemFlags::new(), new_cursor, ctx_ptr, cursor_off);
                let cur_cnt = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, count_off);
                let new_cnt = builder.ins().iadd_imm(cur_cnt, 1);
                builder
                    .ins()
                    .store(MemFlags::new(), new_cnt, ctx_ptr, count_off);
                let cur_bytes = builder
                    .ins()
                    .load(types::I64, MemFlags::new(), ctx_ptr, bytes_off);
                let new_bytes = builder.ins().iadd(cur_bytes, required_block);
                builder
                    .ins()
                    .store(MemFlags::new(), new_bytes, ctx_ptr, bytes_off);

                let static_sz =
                    builder
                        .ins()
                        .load(types::I64, MemFlags::new(), ctx_ptr, static_off);
                let block_abs = builder.ins().iadd(static_sz, cursor);
                let heap_base = builder
                    .ins()
                    .load(ptr_ty, MemFlags::new(), ctx_ptr, heap_off);
                let block_ptr = builder.ins().iadd(heap_base, block_abs);

                let payload_size_i32 = builder.ins().ireduce(types::I32, payload_size);
                builder
                    .ins()
                    .store(MemFlags::new(), payload_size_i32, block_ptr, 0);
                let tag_and_ref_u32: u32 = ((u32::from(BlockTag::ArrayRef.to_u8()) & 0x0F) << 28)
                    | ((u32::from(GcColor::Black.to_u8()) & 0x03) << 26)
                    | 1;
                let tag_and_ref = builder.ins().iconst(types::I32, i64::from(tag_and_ref_u32));
                builder
                    .ins()
                    .store(MemFlags::new(), tag_and_ref, block_ptr, 4);
                builder.ins().store(MemFlags::new(), raw_len, block_ptr, 8);
                let meta2_u32: u32 =
                    (elem_size as u32) | (u32::from(vtable_id) << 8) | (u32::from(type_tag) << 24);
                let meta2 = builder.ins().iconst(types::I32, i64::from(meta2_u32));
                builder.ins().store(MemFlags::new(), meta2, block_ptr, 12);

                let payload_abs = builder.ins().iadd_imm(block_abs, 8);
                let payload_i32 = builder.ins().ireduce(types::I32, payload_abs);
                builder.ins().jump(merge_blk, &[payload_i32]);

                builder.switch_to_block(slow_blk);
                let elem_sz_val = builder.ins().iconst(types::I32, elem_size as i64);
                let type_tag_val = builder.ins().iconst(types::I32, i64::from(type_tag));
                let vtable_val = builder.ins().iconst(types::I32, i64::from(vtable_id));
                let call = builder.ins().call(
                    rt_alloc_arr_ref,
                    &[ctx_ptr, raw_len, elem_sz_val, type_tag_val, vtable_val],
                );
                let slow_addr = builder.inst_results(call)[0];
                emit_trap_return_check(&mut builder, ctx_ptr);
                builder.ins().jump(merge_blk, &[slow_addr]);

                builder.switch_to_block(merge_blk);
                let merged_addr = builder.block_params(merge_blk)[0];
                builder.def_var(vars.get(inst.dst(), RegType::Ref), merged_addr);
                opt_state.invalidate_heap();
                opt_state.set_non_null(inst.dst(), true);
            }
            OpCode::GetFieldI32 | OpCode::GetFieldF64 => {
                let obj = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                opt_state.ensure_non_null(
                    &mut builder,
                    ctx_ptr,
                    inst.src1(),
                    obj,
                    rt_trap_null_ref,
                );
                let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let obj_ext = builder.ins().uextend(ptr_ty, obj);
                let field_off = inst.src2() as i32;
                let ptr = builder.ins().iadd(heap_base, obj_ext);
                let (read_ty, dst_ty) = if opcode == OpCode::GetFieldI32 {
                    (types::I32, RegType::I32)
                } else {
                    (types::F64, RegType::F64)
                };
                let val = builder.ins().load(read_ty, MemFlags::new(), ptr, field_off);
                builder.def_var(vars.get(inst.dst(), dst_ty), val);
                opt_state.clear_non_null(inst.dst());
            }
            OpCode::SetFieldI32 | OpCode::SetFieldF64 => {
                let obj = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                opt_state.ensure_non_null(
                    &mut builder,
                    ctx_ptr,
                    inst.src1(),
                    obj,
                    rt_trap_null_ref,
                );
                let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let obj_ext = builder.ins().uextend(ptr_ty, obj);
                let field_off = inst.src2() as i32;
                let ptr = builder.ins().iadd(heap_base, obj_ext);
                let val_ty = if opcode == OpCode::SetFieldI32 {
                    RegType::I32
                } else {
                    RegType::F64
                };
                let val = builder.use_var(vars.get(inst.dst(), val_ty));
                builder.ins().store(MemFlags::new(), val, ptr, field_off);
            }
            OpCode::GetField => {
                let obj = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                opt_state.ensure_non_null(
                    &mut builder,
                    ctx_ptr,
                    inst.src1(),
                    obj,
                    rt_trap_null_ref,
                );
                let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let obj_ext = builder.ins().uextend(ptr_ty, obj);
                let field_off = inst.src2() as i32;
                let ptr = builder.ins().iadd(heap_base, obj_ext);
                match inst.flags() {
                    10 | 12 => {
                        let val = builder
                            .ins()
                            .load(types::I32, MemFlags::new(), ptr, field_off);
                        builder.def_var(vars.get(inst.dst(), RegType::I32), val);
                    }
                    14 | 16 => {
                        let val = builder
                            .ins()
                            .load(types::I64, MemFlags::new(), ptr, field_off);
                        builder.def_var(vars.get(inst.dst(), RegType::I64), val);
                    }
                    20 => {
                        let val = builder
                            .ins()
                            .load(types::F64, MemFlags::new(), ptr, field_off);
                        builder.def_var(vars.get(inst.dst(), RegType::F64), val);
                    }
                    24 => {
                        let raw = builder
                            .ins()
                            .load(types::I64, MemFlags::new(), ptr, field_off);
                        let addr = builder.ins().ireduce(types::I32, raw);
                        builder.def_var(vars.get(inst.dst(), RegType::Ref), addr);
                    }
                    _ => return Err(()),
                }
                opt_state.clear_non_null(inst.dst());
            }
            OpCode::SetField => {
                let raw_flags = inst.flags();
                let is_move = (raw_flags & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
                let field_tag = (raw_flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
                let obj = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                opt_state.ensure_non_null(
                    &mut builder,
                    ctx_ptr,
                    inst.src1(),
                    obj,
                    rt_trap_null_ref,
                );
                let val_ty = inst_ty.src_ty.unwrap_or(RegType::I32);
                let val = builder.use_var(vars.get(inst.dst(), val_ty));
                if field_tag == 12 {
                    let off_val = builder.ins().iconst(types::I32, i64::from(inst.src2()));
                    let move_val = builder
                        .ins()
                        .iconst(types::I8, i64::from(u8::from(is_move)));
                    builder
                        .ins()
                        .call(rt_set_field_ref_fn, &[ctx_ptr, obj, off_val, val, move_val]);
                    emit_trap_return_check(&mut builder, ctx_ptr);
                    opt_state.invalidate_heap();
                } else {
                    let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                    let obj_ext = builder.ins().uextend(ptr_ty, obj);
                    let field_off = inst.src2() as i32;
                    let ptr = builder.ins().iadd(heap_base, obj_ext);
                    builder.ins().store(MemFlags::new(), val, ptr, field_off);
                }
            }
            OpCode::EqRef | OpCode::NeqRef => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::Ref));
                let cc = if opcode == OpCode::EqRef {
                    IntCC::Equal
                } else {
                    IntCC::NotEqual
                };
                let cmp = builder.ins().icmp(cc, lhs, rhs);
                let res = builder.ins().uextend(types::I32, cmp);
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::EqString | OpCode::NeqString => {
                let lhs = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                let rhs = builder.use_var(vars.get(inst.src2(), RegType::Ref));
                let same_ptr = builder.ins().icmp(IntCC::Equal, lhs, rhs);

                let same_blk = builder.create_block();
                let diff_ptr_blk = builder.create_block();
                let non_null_blk = builder.create_block();
                let slow_eq_blk = builder.create_block();
                let neq_blk = builder.create_block();
                let merge_blk = builder.create_block();
                builder.append_block_param(merge_blk, types::I32);

                builder
                    .ins()
                    .brif(same_ptr, same_blk, &[], diff_ptr_blk, &[]);

                builder.switch_to_block(same_blk);
                let one_val = builder.ins().iconst(types::I32, 1);
                builder.ins().jump(merge_blk, &[one_val]);

                builder.switch_to_block(diff_ptr_blk);
                let lhs_zero = builder.ins().icmp_imm(IntCC::Equal, lhs, 0);
                let rhs_zero = builder.ins().icmp_imm(IntCC::Equal, rhs, 0);
                let any_zero = builder.ins().bor(lhs_zero, rhs_zero);
                builder
                    .ins()
                    .brif(any_zero, neq_blk, &[], non_null_blk, &[]);

                builder.switch_to_block(non_null_blk);
                let heap_base = opt_state.peek_or_load_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let lhs_ext = builder.ins().uextend(ptr_ty, lhs);
                let rhs_ext = builder.ins().uextend(ptr_ty, rhs);
                let lhs_ptr = builder.ins().iadd(heap_base, lhs_ext);
                let rhs_ptr = builder.ins().iadd(heap_base, rhs_ext);
                let lhs_len = builder.ins().load(types::I32, MemFlags::new(), lhs_ptr, -8);
                let rhs_len = builder.ins().load(types::I32, MemFlags::new(), rhs_ptr, -8);
                let len_eq = builder.ins().icmp(IntCC::Equal, lhs_len, rhs_len);
                builder.ins().brif(len_eq, slow_eq_blk, &[], neq_blk, &[]);

                builder.switch_to_block(neq_blk);
                let zero_val = builder.ins().iconst(types::I32, 0);
                builder.ins().jump(merge_blk, &[zero_val]);

                builder.switch_to_block(slow_eq_blk);
                let call = builder.ins().call(rt_string_eq_ref, &[ctx_ptr, lhs, rhs]);
                let eq_byte = builder.inst_results(call)[0];
                let eq_i32 = builder.ins().uextend(types::I32, eq_byte);
                builder.ins().jump(merge_blk, &[eq_i32]);

                builder.switch_to_block(merge_blk);
                let eq_res = builder.block_params(merge_blk)[0];
                let res = if opcode == OpCode::EqString {
                    eq_res
                } else {
                    builder.ins().bxor_imm(eq_res, 1)
                };
                builder.def_var(vars.get(inst.dst(), RegType::I32), res);
            }
            OpCode::ArrayLength => {
                use crate::memory::BlockTag;

                let arr = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                opt_state.ensure_non_null(
                    &mut builder,
                    ctx_ptr,
                    inst.src1(),
                    arr,
                    rt_trap_null_ref,
                );
                let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let arr_ext = builder.ins().uextend(ptr_ty, arr);
                let arr_ptr = builder.ins().iadd(heap_base, arr_ext);
                let tag_byte = builder.ins().load(types::I8, MemFlags::new(), arr_ptr, -1);
                let tag_shift = builder.ins().ushr_imm(tag_byte, 4);
                let tag_nibble = builder.ins().band_imm(tag_shift, 0x0F);
                let is_str = builder.ins().icmp_imm(
                    IntCC::Equal,
                    tag_nibble,
                    i64::from(BlockTag::String.to_u8()),
                );
                let str_len = builder.ins().load(types::I32, MemFlags::new(), arr_ptr, -8);
                let arr_len = builder.ins().load(types::I32, MemFlags::new(), arr_ptr, 0);
                let final_len = builder.ins().select(is_str, str_len, arr_len);
                builder.def_var(vars.get(inst.dst(), RegType::I32), final_len);
                opt_state.clear_non_null(inst.dst());
            }
            OpCode::GetArray => {
                use crate::memory::BlockTag;

                let arr = builder.use_var(vars.get(inst.src1(), RegType::Ref));
                opt_state.ensure_non_null(
                    &mut builder,
                    ctx_ptr,
                    inst.src1(),
                    arr,
                    rt_trap_null_ref,
                );
                let idx_i32 = builder.use_var(vars.get(inst.src2(), RegType::I32));
                let dst_ty = inst_ty.dst_ty.unwrap_or(RegType::I32);

                let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let arr_ext = builder.ins().uextend(ptr_ty, arr);
                let arr_ptr = builder.ins().iadd(heap_base, arr_ext);
                let tag_byte = builder.ins().load(types::I8, MemFlags::new(), arr_ptr, -1);
                let tag_shift = builder.ins().ushr_imm(tag_byte, 4);
                let tag_nibble = builder.ins().band_imm(tag_shift, 0x0F);
                let is_arr = builder.ins().icmp_imm(
                    IntCC::Equal,
                    tag_nibble,
                    i64::from(BlockTag::ArrayRef.to_u8()),
                );

                let arr_blk = builder.create_block();
                let bounds_ok_blk = builder.create_block();
                let oob_blk = builder.create_block();
                let fast_load_blk = builder.create_block();
                let fast_1b_blk = builder.create_block();
                let slow_blk = builder.create_block();
                let merge_blk = builder.create_block();
                builder.append_block_param(merge_blk, reg_type_to_clif(dst_ty));

                let elem_sz = builder.ins().load(types::I8, MemFlags::new(), arr_ptr, 4);
                let arr_len = builder.ins().load(types::I32, MemFlags::new(), arr_ptr, 0);
                let in_bounds = builder
                    .ins()
                    .icmp(IntCC::UnsignedLessThan, idx_i32, arr_len);

                match dst_ty {
                    RegType::I64 | RegType::F64 | RegType::Ref => {
                        let is_8b = builder.ins().icmp_imm(IntCC::Equal, elem_sz, 8);
                        let fast_hdr = builder.ins().band(is_arr, is_8b);
                        builder.ins().brif(fast_hdr, arr_blk, &[], slow_blk, &[]);

                        builder.switch_to_block(arr_blk);
                        builder
                            .ins()
                            .brif(in_bounds, fast_load_blk, &[], oob_blk, &[]);

                        builder.switch_to_block(bounds_ok_blk);
                        builder.ins().jump(slow_blk, &[]);
                        builder.switch_to_block(fast_1b_blk);
                        builder.ins().jump(slow_blk, &[]);

                        builder.switch_to_block(fast_load_blk);
                        let idx_ext = builder.ins().uextend(ptr_ty, idx_i32);
                        let off8 = builder.ins().ishl_imm(idx_ext, 3);
                        let ptr8 = builder.ins().iadd(arr_ptr, off8);
                        let loaded = if dst_ty == RegType::F64 {
                            builder.ins().load(types::F64, MemFlags::new(), ptr8, 8)
                        } else if dst_ty == RegType::I64 {
                            builder.ins().load(types::I64, MemFlags::new(), ptr8, 8)
                        } else {
                            let raw = builder.ins().load(types::I64, MemFlags::new(), ptr8, 8);
                            builder.ins().ireduce(types::I32, raw)
                        };
                        builder.ins().jump(merge_blk, &[loaded]);
                    }
                    RegType::I32 => {
                        builder.ins().brif(is_arr, arr_blk, &[], slow_blk, &[]);

                        builder.switch_to_block(arr_blk);
                        builder
                            .ins()
                            .brif(in_bounds, bounds_ok_blk, &[], oob_blk, &[]);

                        builder.switch_to_block(bounds_ok_blk);
                        let is_4b = builder.ins().icmp_imm(IntCC::Equal, elem_sz, 4);
                        let check_1b_blk = builder.create_block();
                        builder
                            .ins()
                            .brif(is_4b, fast_load_blk, &[], check_1b_blk, &[]);
                        builder.switch_to_block(check_1b_blk);
                        let is_1b = builder.ins().icmp_imm(IntCC::Equal, elem_sz, 1);
                        builder.ins().brif(is_1b, fast_1b_blk, &[], slow_blk, &[]);

                        builder.switch_to_block(fast_load_blk);
                        let idx_ext4 = builder.ins().uextend(ptr_ty, idx_i32);
                        let off4 = builder.ins().ishl_imm(idx_ext4, 2);
                        let ptr4 = builder.ins().iadd(arr_ptr, off4);
                        let val4 = builder.ins().load(types::I32, MemFlags::new(), ptr4, 8);
                        builder.ins().jump(merge_blk, &[val4]);

                        builder.switch_to_block(fast_1b_blk);
                        let idx_ext1 = builder.ins().uextend(ptr_ty, idx_i32);
                        let ptr1 = builder.ins().iadd(arr_ptr, idx_ext1);
                        let val1 = builder.ins().load(types::I8, MemFlags::new(), ptr1, 8);
                        let val1_i32 = builder.ins().uextend(types::I32, val1);
                        builder.ins().jump(merge_blk, &[val1_i32]);
                    }
                }

                builder.switch_to_block(oob_blk);
                let idx_i64_oob = builder.ins().sextend(types::I64, idx_i32);
                let len_i64_oob = builder.ins().uextend(types::I64, arr_len);
                builder
                    .ins()
                    .call(rt_trap_oob_ref, &[ctx_ptr, idx_i64_oob, len_i64_oob]);
                let zero_ret = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero_ret]);

                builder.switch_to_block(slow_blk);
                let idx_i64_slow = builder.ins().sextend(types::I64, idx_i32);
                let ty_code_val = builder
                    .ins()
                    .iconst(types::I32, i64::from(dst_ty.to_code()));
                let call = builder.ins().call(
                    rt_get_array_slow_ref,
                    &[ctx_ptr, arr, idx_i64_slow, ty_code_val],
                );
                let raw_word = builder.inst_results(call)[0];
                emit_trap_return_check(&mut builder, ctx_ptr);
                let typed = decode_raw_word(&mut builder, raw_word, dst_ty);
                builder.ins().jump(merge_blk, &[typed]);

                builder.switch_to_block(merge_blk);
                let merged_val = builder.block_params(merge_blk)[0];
                builder.def_var(vars.get(inst.dst(), dst_ty), merged_val);
                opt_state.clear_non_null(inst.dst());
            }
            OpCode::SetArray => {
                use crate::memory::BlockTag;

                let arr = builder.use_var(vars.get(inst.dst(), RegType::Ref));
                opt_state.ensure_non_null(&mut builder, ctx_ptr, inst.dst(), arr, rt_trap_null_ref);
                let idx_i32 = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let val_ty = inst_ty.src_ty.unwrap_or(RegType::I32);
                let val = builder.use_var(vars.get(inst.src2(), val_ty));

                if val_ty == RegType::Ref {
                    let idx_i64 = builder.ins().sextend(types::I64, idx_i32);
                    let raw_val = encode_raw_word(&mut builder, val, val_ty);
                    let ty_code_val = builder
                        .ins()
                        .iconst(types::I32, i64::from(val_ty.to_code()));
                    let is_move =
                        (inst.flags() & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
                    let move_val = builder
                        .ins()
                        .iconst(types::I8, i64::from(u8::from(is_move)));
                    builder.ins().call(
                        rt_set_array_slow_ref,
                        &[ctx_ptr, arr, idx_i64, raw_val, ty_code_val, move_val],
                    );
                    emit_trap_return_check(&mut builder, ctx_ptr);
                    opt_state.invalidate_heap();
                } else {
                    let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                    let arr_ext = builder.ins().uextend(ptr_ty, arr);
                    let arr_ptr = builder.ins().iadd(heap_base, arr_ext);
                    let tag_byte = builder.ins().load(types::I8, MemFlags::new(), arr_ptr, -1);
                    let tag_shift = builder.ins().ushr_imm(tag_byte, 4);
                    let tag_nibble = builder.ins().band_imm(tag_shift, 0x0F);
                    let is_arr = builder.ins().icmp_imm(
                        IntCC::Equal,
                        tag_nibble,
                        i64::from(BlockTag::ArrayRef.to_u8()),
                    );

                    let arr_blk = builder.create_block();
                    let bounds_ok_blk = builder.create_block();
                    let oob_blk = builder.create_block();
                    let fast_store_blk = builder.create_block();
                    let fast_1b_blk = builder.create_block();
                    let slow_blk = builder.create_block();
                    let done_blk = builder.create_block();

                    let elem_sz = builder.ins().load(types::I8, MemFlags::new(), arr_ptr, 4);
                    let arr_len = builder.ins().load(types::I32, MemFlags::new(), arr_ptr, 0);
                    let in_bounds = builder
                        .ins()
                        .icmp(IntCC::UnsignedLessThan, idx_i32, arr_len);

                    match val_ty {
                        RegType::I32 => {
                            builder.ins().brif(is_arr, arr_blk, &[], slow_blk, &[]);

                            builder.switch_to_block(arr_blk);
                            builder
                                .ins()
                                .brif(in_bounds, bounds_ok_blk, &[], oob_blk, &[]);

                            builder.switch_to_block(bounds_ok_blk);
                            let is_4b = builder.ins().icmp_imm(IntCC::Equal, elem_sz, 4);
                            let check_1b_blk = builder.create_block();
                            builder
                                .ins()
                                .brif(is_4b, fast_store_blk, &[], check_1b_blk, &[]);
                            builder.switch_to_block(check_1b_blk);
                            let is_1b = builder.ins().icmp_imm(IntCC::Equal, elem_sz, 1);
                            builder.ins().brif(is_1b, fast_1b_blk, &[], slow_blk, &[]);

                            builder.switch_to_block(fast_store_blk);
                            let idx_ext4 = builder.ins().uextend(ptr_ty, idx_i32);
                            let off4 = builder.ins().ishl_imm(idx_ext4, 2);
                            let ptr4 = builder.ins().iadd(arr_ptr, off4);
                            builder.ins().store(MemFlags::new(), val, ptr4, 8);
                            builder.ins().jump(done_blk, &[]);

                            builder.switch_to_block(fast_1b_blk);
                            let idx_ext1 = builder.ins().uextend(ptr_ty, idx_i32);
                            let ptr1 = builder.ins().iadd(arr_ptr, idx_ext1);
                            let byte_val = builder.ins().ireduce(types::I8, val);
                            builder.ins().store(MemFlags::new(), byte_val, ptr1, 8);
                            builder.ins().jump(done_blk, &[]);
                        }
                        RegType::I64 | RegType::F64 => {
                            let is_8b = builder.ins().icmp_imm(IntCC::Equal, elem_sz, 8);
                            let fast_hdr = builder.ins().band(is_arr, is_8b);
                            builder.ins().brif(fast_hdr, arr_blk, &[], slow_blk, &[]);

                            builder.switch_to_block(arr_blk);
                            builder
                                .ins()
                                .brif(in_bounds, fast_store_blk, &[], oob_blk, &[]);

                            builder.switch_to_block(bounds_ok_blk);
                            builder.ins().jump(slow_blk, &[]);
                            builder.switch_to_block(fast_1b_blk);
                            builder.ins().jump(slow_blk, &[]);

                            builder.switch_to_block(fast_store_blk);
                            let idx_ext8 = builder.ins().uextend(ptr_ty, idx_i32);
                            let off8 = builder.ins().ishl_imm(idx_ext8, 3);
                            let ptr8 = builder.ins().iadd(arr_ptr, off8);
                            builder.ins().store(MemFlags::new(), val, ptr8, 8);
                            builder.ins().jump(done_blk, &[]);
                        }
                        RegType::Ref => {}
                    }

                    builder.switch_to_block(oob_blk);
                    let idx_i64_oob = builder.ins().sextend(types::I64, idx_i32);
                    let len_i64_oob = builder.ins().uextend(types::I64, arr_len);
                    builder
                        .ins()
                        .call(rt_trap_oob_ref, &[ctx_ptr, idx_i64_oob, len_i64_oob]);
                    let zero_ret = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero_ret]);

                    builder.switch_to_block(slow_blk);
                    let idx_i64_slow = builder.ins().sextend(types::I64, idx_i32);
                    let raw_val = encode_raw_word(&mut builder, val, val_ty);
                    let ty_code_val = builder
                        .ins()
                        .iconst(types::I32, i64::from(val_ty.to_code()));
                    let is_move =
                        (inst.flags() & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
                    let move_val = builder
                        .ins()
                        .iconst(types::I8, i64::from(u8::from(is_move)));
                    builder.ins().call(
                        rt_set_array_slow_ref,
                        &[ctx_ptr, arr, idx_i64_slow, raw_val, ty_code_val, move_val],
                    );
                    emit_trap_return_check(&mut builder, ctx_ptr);
                    builder.ins().jump(done_blk, &[]);

                    builder.switch_to_block(done_blk);
                }
            }
            OpCode::GetArrayJumpIfFalse | OpCode::GetArrayJumpIfTrue => {
                let arr = builder.use_var(vars.get(inst.dst(), RegType::Ref));
                opt_state.ensure_non_null(&mut builder, ctx_ptr, inst.dst(), arr, rt_trap_null_ref);
                let idx_i32 = builder.use_var(vars.get(inst.src1(), RegType::I32));
                let heap_base = opt_state.get_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let arr_ext = builder.ins().uextend(ptr_ty, arr);
                let arr_ptr = builder.ins().iadd(heap_base, arr_ext);
                let arr_len = builder.ins().load(types::I32, MemFlags::new(), arr_ptr, 0);
                let in_bounds = builder
                    .ins()
                    .icmp(IntCC::UnsignedLessThan, idx_i32, arr_len);
                let oob_blk = builder.create_block();
                let ok_blk = builder.create_block();
                builder.ins().brif(in_bounds, ok_blk, &[], oob_blk, &[]);
                builder.switch_to_block(oob_blk);
                let idx_i64_oob = builder.ins().sextend(types::I64, idx_i32);
                let len_i64_oob = builder.ins().uextend(types::I64, arr_len);
                builder
                    .ins()
                    .call(rt_trap_oob_ref, &[ctx_ptr, idx_i64_oob, len_i64_oob]);
                let zero_ret = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero_ret]);
                builder.switch_to_block(ok_blk);

                let idx_ext = builder.ins().uextend(ptr_ty, idx_i32);
                let elem_ptr = builder.ins().iadd(arr_ptr, idx_ext);
                let elem_byte = builder.ins().load(types::I8, MemFlags::new(), elem_ptr, 8);
                let cc = if opcode == OpCode::GetArrayJumpIfFalse {
                    IntCC::Equal
                } else {
                    IntCC::NotEqual
                };
                let taken = builder.ins().icmp_imm(cc, elem_byte, 0);
                let target_pc = ((pc as i32) + (inst.src2() as i16 as i32)) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                let next_blk = blocks_by_pc[&(pc + 1)];
                builder.ins().brif(taken, target_blk, &[], next_blk, &[]);
                block_terminated = true;
            }
            OpCode::IncRef => {
                use crate::memory::MAX_REF_COUNT;

                let addr = builder.use_var(vars.get(inst.dst(), RegType::Ref));
                let is_null = builder.ins().icmp_imm(IntCC::Equal, addr, 0);
                let non_null_blk = builder.create_block();
                let fast_blk = builder.create_block();
                let check_immortal_blk = builder.create_block();
                let slow_blk = builder.create_block();
                let done_blk = builder.create_block();

                builder
                    .ins()
                    .brif(is_null, done_blk, &[], non_null_blk, &[]);

                builder.switch_to_block(non_null_blk);
                let heap_base = opt_state.peek_or_load_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let addr_ext = builder.ins().uextend(ptr_ty, addr);
                let payload_ptr = builder.ins().iadd(heap_base, addr_ext);
                let word = builder
                    .ins()
                    .load(types::I32, MemFlags::new(), payload_ptr, -4);
                let rc = builder.ins().band_imm(word, i64::from(MAX_REF_COUNT));
                let rc_sub1 = builder.ins().iadd_imm(rc, -1);
                let can_fast = builder.ins().icmp_imm(
                    IntCC::UnsignedLessThan,
                    rc_sub1,
                    i64::from(MAX_REF_COUNT - 2),
                );
                builder
                    .ins()
                    .brif(can_fast, fast_blk, &[], check_immortal_blk, &[]);

                builder.switch_to_block(fast_blk);
                let cleared_color = builder.ins().band_imm(word, i64::from(0xF3FF_FFFF_u32));
                let updated = builder.ins().iadd_imm(cleared_color, 1);
                builder
                    .ins()
                    .store(MemFlags::new(), updated, payload_ptr, -4);
                builder.ins().jump(done_blk, &[]);

                builder.switch_to_block(check_immortal_blk);
                let is_immortal =
                    builder
                        .ins()
                        .icmp_imm(IntCC::Equal, rc, i64::from(MAX_REF_COUNT));
                builder
                    .ins()
                    .brif(is_immortal, done_blk, &[], slow_blk, &[]);

                builder.switch_to_block(slow_blk);
                builder.ins().call(rt_inc_ref, &[ctx_ptr, addr]);
                emit_trap_return_check(&mut builder, ctx_ptr);
                builder.ins().jump(done_blk, &[]);

                builder.switch_to_block(done_blk);
            }
            OpCode::DecRef => {
                use crate::memory::MAX_REF_COUNT;
                use crate::vm::jit::JitContext;

                let addr = builder.use_var(vars.get(inst.dst(), RegType::Ref));
                let is_null = builder.ins().icmp_imm(IntCC::Equal, addr, 0);
                let non_null_blk = builder.create_block();
                let check_cc_blk = builder.create_block();
                let fast_blk = builder.create_block();
                let slow_blk = builder.create_block();
                let done_blk = builder.create_block();

                builder
                    .ins()
                    .brif(is_null, done_blk, &[], non_null_blk, &[]);

                builder.switch_to_block(non_null_blk);
                let heap_base = opt_state.peek_or_load_heap_base(&mut builder, ctx_ptr, ptr_ty);
                let addr_ext = builder.ins().uextend(ptr_ty, addr);
                let payload_ptr = builder.ins().iadd(heap_base, addr_ext);
                let word = builder
                    .ins()
                    .load(types::I32, MemFlags::new(), payload_ptr, -4);
                let rc = builder.ins().band_imm(word, i64::from(MAX_REF_COUNT));
                let rc_sub2 = builder.ins().iadd_imm(rc, -2);
                let can_fast_rc = builder.ins().icmp_imm(
                    IntCC::UnsignedLessThan,
                    rc_sub2,
                    i64::from(MAX_REF_COUNT - 2),
                );
                builder
                    .ins()
                    .brif(can_fast_rc, check_cc_blk, &[], slow_blk, &[]);

                builder.switch_to_block(check_cc_blk);
                let cc_off = std::mem::offset_of!(JitContext, cycle_collector_enabled) as i32;
                let cc_enabled = builder
                    .ins()
                    .load(types::I8, MemFlags::new(), ctx_ptr, cc_off);
                let cc_off_flag = builder.ins().icmp_imm(IntCC::Equal, cc_enabled, 0);
                builder
                    .ins()
                    .brif(cc_off_flag, fast_blk, &[], slow_blk, &[]);

                builder.switch_to_block(fast_blk);
                let updated = builder.ins().iadd_imm(word, -1);
                builder
                    .ins()
                    .store(MemFlags::new(), updated, payload_ptr, -4);
                builder.ins().jump(done_blk, &[]);

                builder.switch_to_block(slow_blk);
                builder.ins().call(rt_dec_ref, &[ctx_ptr, addr]);
                builder.ins().jump(done_blk, &[]);

                builder.switch_to_block(done_blk);
                opt_state.invalidate_heap();
            }
            OpCode::CallDirect | OpCode::TailCallDirect => {
                let target_id = inst.call_target() as u32;
                let arg_base = inst.arg_base() as usize;
                let arg_count = inst.arg_count() as usize;
                let call_dst = inst.call_dst() as usize;

                if opcode == OpCode::TailCallDirect
                    && engine
                        .slots
                        .get(target_id as usize)
                        .and_then(|slot| slot.clif_id)
                        == Some(self_clif_id)
                    && arg_count == types_info.func_sig.param_types.len()
                    && arg_count <= 4
                {
                    let mut next_params: [Option<(Variable, ClifVal)>; 4] = [None; 4];
                    let mut all_match = true;
                    for (arg_idx, &param_ty_opt) in
                        types_info.func_sig.param_types.iter().enumerate()
                    {
                        let Some(param_ty) = param_ty_opt else {
                            continue;
                        };
                        let reg_ty = inst_ty.call_args[arg_idx].unwrap_or(RegType::I32);
                        if reg_ty != param_ty {
                            all_match = false;
                            break;
                        }
                        let arg_val =
                            builder.use_var(vars.get((arg_base + arg_idx) as u16, reg_ty));
                        next_params[arg_idx] = Some((vars.get(arg_idx as u16, param_ty), arg_val));
                    }
                    if all_match {
                        for (param_var, next_val) in next_params.into_iter().flatten() {
                            builder.def_var(param_var, next_val);
                        }
                        builder.ins().jump(body_entry_block, &[]);
                        block_terminated = true;
                    } else {
                        let mut call_args = [builder.ins().iconst(types::I64, 0); 5];
                        call_args[0] = ctx_ptr;
                        for arg_idx in 0..arg_count.min(4) {
                            let reg_ty = inst_ty.call_args[arg_idx].unwrap_or(RegType::I32);
                            let reg_val =
                                builder.use_var(vars.get((arg_base + arg_idx) as u16, reg_ty));
                            call_args[1 + arg_idx] = encode_raw_word(&mut builder, reg_val, reg_ty);
                        }
                        let call = builder.ins().call(self_func_ref, &call_args);
                        let ret_raw = builder.inst_results(call)[0];
                        emit_trap_return_check(&mut builder, ctx_ptr);
                        builder.ins().return_(&[ret_raw]);
                        block_terminated = true;
                    }
                } else if let Some(target_clif_id) = engine
                    .slots
                    .get(target_id as usize)
                    .and_then(|slot| slot.clif_id)
                {
                    let target_ref = if target_clif_id == self_clif_id {
                        self_func_ref
                    } else {
                        engine
                            .module
                            .declare_func_in_func(target_clif_id, builder.func)
                    };
                    let mut call_args = [builder.ins().iconst(types::I64, 0); 5];
                    call_args[0] = ctx_ptr;
                    for arg_idx in 0..arg_count.min(4) {
                        let reg_ty = inst_ty.call_args[arg_idx].unwrap_or(RegType::I32);
                        let reg_val =
                            builder.use_var(vars.get((arg_base + arg_idx) as u16, reg_ty));
                        call_args[1 + arg_idx] = encode_raw_word(&mut builder, reg_val, reg_ty);
                    }
                    let call = builder.ins().call(target_ref, &call_args);
                    let ret_raw = builder.inst_results(call)[0];
                    emit_trap_return_check(&mut builder, ctx_ptr);
                    if opcode == OpCode::TailCallDirect {
                        builder.ins().return_(&[ret_raw]);
                        block_terminated = true;
                    } else if let Some(dst_ty) = inst_ty.dst_ty {
                        let typed = decode_raw_word(&mut builder, ret_raw, dst_ty);
                        builder.def_var(vars.get(call_dst as u16, dst_ty), typed);
                    }
                } else {
                    let is_dst_used =
                        is_reg_used_after(&compiled_fn.instructions, pc, call_dst as u16);
                    emit_slow_call_dispatch(
                        &mut builder,
                        rt_call_slow_ref,
                        ctx_ptr,
                        ptr_ty,
                        None,
                        target_id,
                        arg_base,
                        arg_count,
                        call_dst,
                        is_dst_used,
                        opcode == OpCode::TailCallDirect,
                        &vars,
                        inst_ty,
                    );
                    if opcode == OpCode::TailCallDirect {
                        block_terminated = true;
                    }
                }
            }
            OpCode::CallNativeDirect => {
                let target_id = inst.call_target() as u32;
                let arg_base = inst.arg_base() as usize;
                let arg_count = inst.arg_count() as usize;
                let call_dst = inst.call_dst() as usize;
                let is_dst_used = is_reg_used_after(&compiled_fn.instructions, pc, call_dst as u16);
                emit_slow_call_dispatch(
                    &mut builder,
                    rt_call_slow_ref,
                    ctx_ptr,
                    ptr_ty,
                    None,
                    target_id,
                    arg_base,
                    arg_count,
                    call_dst,
                    is_dst_used,
                    false,
                    &vars,
                    inst_ty,
                );
            }
            OpCode::CallSymbol => {
                let sym_idx = inst.call_target() as u32;
                let arg_base = inst.arg_base() as usize;
                let arg_count = inst.arg_count() as usize;
                let call_dst = inst.call_dst() as usize;
                let sym_name = match const_pool.and_then(|pool| pool.get(sym_idx as usize)) {
                    Some(solix_common::bytecode::ConstantPoolValue::String(text)) => text.as_str(),
                    _ => "",
                };
                if (sym_name == "Math::sqrt" || sym_name == "sqrt")
                    && arg_count == 1
                    && inst_ty.call_args[0] == Some(RegType::F64)
                    && inst_ty.dst_ty == Some(RegType::F64)
                {
                    let arg_val = builder.use_var(vars.get(arg_base as u16, RegType::F64));
                    let res = builder.ins().sqrt(arg_val);
                    builder.def_var(vars.get(call_dst as u16, RegType::F64), res);
                } else {
                    let is_dst_used =
                        is_reg_used_after(&compiled_fn.instructions, pc, call_dst as u16);
                    emit_slow_call_dispatch(
                        &mut builder,
                        rt_call_symbol_slow_ref,
                        ctx_ptr,
                        ptr_ty,
                        Some(binary_id),
                        sym_idx,
                        arg_base,
                        arg_count,
                        call_dst,
                        is_dst_used,
                        false,
                        &vars,
                        inst_ty,
                    );
                }
            }
            OpCode::CallVirtual | OpCode::CallVirtualDirect => {
                let raw_target = inst.call_target() as u32;
                let arg_base = inst.arg_base() as usize;
                let arg_count = inst.arg_count() as usize;
                let call_dst = inst.call_dst() as usize;
                let is_dst_used = is_reg_used_after(&compiled_fn.instructions, pc, call_dst as u16);
                let mono_site = monomorphic_caches
                    .and_then(|sites| sites.get(pc).copied().flatten())
                    .filter(|site| site.expected_vtable_id != u32::MAX);
                let effective_target =
                    mono_site.map_or(raw_target, |site| u32::from(site.original_slot) | 0x8000);
                let mono_clif = mono_site.and_then(|site| {
                    engine
                        .slots
                        .get(site.target_func_id as usize)
                        .and_then(|slot| slot.clif_id)
                        .map(|clif_id| (site.expected_vtable_id, clif_id))
                });
                let mono_target_ref = mono_clif.map(|(expected_vt, target_clif_id)| {
                    let func_ref = if target_clif_id == self_clif_id {
                        self_func_ref
                    } else {
                        engine
                            .module
                            .declare_func_in_func(target_clif_id, builder.func)
                    };
                    (expected_vt, func_ref)
                });
                emit_virtual_or_interface_call(
                    &mut builder,
                    rt_resolve_jit_virtual_fn_ref,
                    rt_call_virtual_slow_ref,
                    jit_call_sig_ref,
                    rt_trap_null_ref,
                    ctx_ptr,
                    ptr_ty,
                    effective_target,
                    arg_base,
                    arg_count,
                    call_dst,
                    is_dst_used,
                    mono_target_ref,
                    &vars,
                    inst_ty,
                    &mut opt_state,
                );
            }
            OpCode::CallInterface => {
                let target = inst.call_target() as u32;
                let arg_base = inst.arg_base() as usize;
                let arg_count = inst.arg_count() as usize;
                let call_dst = inst.call_dst() as usize;
                let is_dst_used = is_reg_used_after(&compiled_fn.instructions, pc, call_dst as u16);
                emit_virtual_or_interface_call(
                    &mut builder,
                    rt_resolve_jit_interface_fn_ref,
                    rt_call_interface_slow_ref,
                    jit_call_sig_ref,
                    rt_trap_null_ref,
                    ctx_ptr,
                    ptr_ty,
                    target,
                    arg_base,
                    arg_count,
                    call_dst,
                    is_dst_used,
                    None,
                    &vars,
                    inst_ty,
                    &mut opt_state,
                );
            }
            OpCode::Return => {
                let zero_ret = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero_ret]);
                block_terminated = true;
            }
            OpCode::ReturnValue => {
                let reg_ty = inst_ty.src_ty.unwrap_or(RegType::I32);
                let reg_val = builder.use_var(vars.get(inst.dst(), reg_ty));
                let ret_raw = encode_raw_word(&mut builder, reg_val, reg_ty);
                builder.ins().return_(&[ret_raw]);
                block_terminated = true;
            }
            _ => return Err(()),
        }

        match opcode {
            OpCode::CallDirect
            | OpCode::TailCallDirect
            | OpCode::CallNativeDirect
            | OpCode::CallSymbol => {
                opt_state.invalidate_heap();
                opt_state.clear_non_null(inst.call_dst());
            }
            OpCode::LoadNull | OpCode::LoadConst => {
                opt_state.clear_non_null(inst.dst());
            }
            _ => {}
        }
    }

    if !block_terminated {
        let zero_ret = builder.ins().iconst(types::I64, 0);
        builder.ins().return_(&[zero_ret]);
    }

    builder.seal_all_blocks();
    builder.finalize();
    Ok(())
}

fn encode_raw_word(builder: &mut FunctionBuilder<'_>, val: ClifVal, reg_ty: RegType) -> ClifVal {
    match reg_ty {
        RegType::I32 | RegType::Ref => builder.ins().uextend(types::I64, val),
        RegType::I64 => val,
        RegType::F64 => builder.ins().bitcast(types::I64, MemFlags::new(), val),
    }
}

fn decode_raw_word(builder: &mut FunctionBuilder<'_>, raw: ClifVal, reg_ty: RegType) -> ClifVal {
    match reg_ty {
        RegType::I32 | RegType::Ref => builder.ins().ireduce(types::I32, raw),
        RegType::I64 => raw,
        RegType::F64 => builder.ins().bitcast(types::F64, MemFlags::new(), raw),
    }
}

struct BlockOptState {
    hoisted_heap_base: Option<ClifVal>,
    block_heap_base: Option<ClifVal>,
    non_null_mask: u128,
}

impl BlockOptState {
    fn new(hoisted_heap_base: Option<ClifVal>) -> Self {
        Self {
            hoisted_heap_base,
            block_heap_base: hoisted_heap_base,
            non_null_mask: 0,
        }
    }

    fn reset_block(&mut self) {
        self.block_heap_base = self.hoisted_heap_base;
        self.non_null_mask = 0;
    }

    fn invalidate_heap(&mut self) {
        self.block_heap_base = self.hoisted_heap_base;
    }

    fn is_non_null(&self, reg: u16) -> bool {
        let idx = usize::from(reg);
        idx < 128 && (self.non_null_mask & (1u128 << idx)) != 0
    }

    fn set_non_null(&mut self, reg: u16, non_null: bool) {
        let idx = usize::from(reg);
        if idx >= 128 {
            return;
        }
        if non_null {
            self.non_null_mask |= 1u128 << idx;
        } else {
            self.non_null_mask &= !(1u128 << idx);
        }
    }

    fn clear_non_null(&mut self, reg: u16) {
        self.set_non_null(reg, false);
    }

    fn ensure_non_null(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        ctx_ptr: ClifVal,
        reg: u16,
        obj: ClifVal,
        rt_trap_null_ref: cranelift_codegen::ir::FuncRef,
    ) {
        if self.is_non_null(reg) {
            return;
        }
        emit_null_check(builder, ctx_ptr, obj, rt_trap_null_ref);
        self.set_non_null(reg, true);
    }

    fn get_heap_base(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        ctx_ptr: ClifVal,
        ptr_ty: types::Type,
    ) -> ClifVal {
        if let Some(base) = self.block_heap_base {
            return base;
        }
        let base = builder.ins().load(ptr_ty, MemFlags::new(), ctx_ptr, 0);
        self.block_heap_base = Some(base);
        base
    }

    fn peek_or_load_heap_base(
        &self,
        builder: &mut FunctionBuilder<'_>,
        ctx_ptr: ClifVal,
        ptr_ty: types::Type,
    ) -> ClifVal {
        if let Some(base) = self.block_heap_base {
            return base;
        }
        builder.ins().load(ptr_ty, MemFlags::new(), ctx_ptr, 0)
    }
}

fn function_can_reallocate_heap(
    instructions: &[solix_common::bytecode::Instruction],
    types_info: &InferredFunctionTypes,
) -> bool {
    for (pc, inst) in instructions.iter().enumerate() {
        let Some(opcode) = OpCode::from_raw_u8(inst.raw_opcode()) else {
            return true;
        };
        match opcode {
            OpCode::AllocObject
            | OpCode::AllocArray
            | OpCode::DecRef
            | OpCode::CallDirect
            | OpCode::TailCallDirect
            | OpCode::CallNativeDirect
            | OpCode::CallSymbol
            | OpCode::CallVirtual
            | OpCode::CallVirtualDirect
            | OpCode::CallInterface
            | OpCode::Throw => return true,
            OpCode::SetField => {
                let raw_flags = inst.flags();
                let field_tag = (raw_flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
                if field_tag == 12 {
                    return true;
                }
            }
            OpCode::SetArray if types_info.inst_types[pc].src_ty == Some(RegType::Ref) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn emit_null_check(
    builder: &mut FunctionBuilder<'_>,
    ctx_ptr: ClifVal,
    obj: ClifVal,
    rt_trap_null_ref: cranelift_codegen::ir::FuncRef,
) {
    let is_null = builder.ins().icmp_imm(IntCC::Equal, obj, 0);
    let trap_blk = builder.create_block();
    let ok_blk = builder.create_block();
    builder.ins().brif(is_null, trap_blk, &[], ok_blk, &[]);
    builder.switch_to_block(trap_blk);
    builder.ins().call(rt_trap_null_ref, &[ctx_ptr]);
    let zero_ret = builder.ins().iconst(types::I64, 0);
    builder.ins().return_(&[zero_ret]);
    builder.switch_to_block(ok_blk);
}

fn is_reg_used_after(
    instructions: &[solix_common::bytecode::Instruction],
    call_pc: usize,
    reg: u16,
) -> bool {
    let mut saw_branch = false;
    for (offset, inst) in instructions.iter().skip(call_pc + 1).enumerate() {
        let cur_pc = call_pc + 1 + offset;
        let Some(opcode) = OpCode::from_raw_u8(inst.raw_opcode()) else {
            return true;
        };
        match opcode {
            OpCode::AddI32
            | OpCode::SubI32
            | OpCode::MulI32
            | OpCode::DivI32
            | OpCode::ModI32
            | OpCode::Band
            | OpCode::Bor
            | OpCode::Bxor
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::Lshr
            | OpCode::AddI64
            | OpCode::SubI64
            | OpCode::MulI64
            | OpCode::AddF64
            | OpCode::SubF64
            | OpCode::MulF64
            | OpCode::DivF64
            | OpCode::EqI32
            | OpCode::NeqI32
            | OpCode::LtI32
            | OpCode::LteI32
            | OpCode::GtI32
            | OpCode::GteI32
            | OpCode::EqBool
            | OpCode::NeqBool
            | OpCode::EqF64
            | OpCode::NeqF64
            | OpCode::LtF64
            | OpCode::LteF64
            | OpCode::GtF64
            | OpCode::GteF64
            | OpCode::EqRef
            | OpCode::NeqRef
            | OpCode::EqString
            | OpCode::NeqString
            | OpCode::GetArray
                if inst.src1() == reg || inst.src2() == reg =>
            {
                return true;
            }
            OpCode::Mov
            | OpCode::NegI32
            | OpCode::Bnot
            | OpCode::Not
            | OpCode::AddI32Imm
            | OpCode::SubI32Imm
            | OpCode::MulI32Imm
            | OpCode::ModI32Imm
            | OpCode::AddI64Imm
            | OpCode::NegF64
            | OpCode::AddF64Imm
            | OpCode::SubF64Imm
            | OpCode::MulF64Imm
            | OpCode::ConvIToF
            | OpCode::ConvFToI
            | OpCode::AllocArray
            | OpCode::GetFieldI32
            | OpCode::GetFieldF64
            | OpCode::GetField
            | OpCode::ArrayLength
                if inst.src1() == reg =>
            {
                return true;
            }
            OpCode::ReturnValue
            | OpCode::JumpIfTrue
            | OpCode::JumpIfFalse
            | OpCode::JumpIfI32Imm
            | OpCode::IncRef
            | OpCode::DecRef
                if inst.dst() == reg =>
            {
                return true;
            }
            OpCode::JumpIfLtI32
            | OpCode::JumpIfLteI32
            | OpCode::JumpIfGtI32
            | OpCode::JumpIfGteI32
            | OpCode::JumpIfEqI32
            | OpCode::JumpIfNeqI32
            | OpCode::JumpIfLtF64
            | OpCode::JumpIfLteF64
            | OpCode::SetField
            | OpCode::SetFieldI32
            | OpCode::SetFieldF64
            | OpCode::GetArrayJumpIfFalse
            | OpCode::GetArrayJumpIfTrue
                if inst.dst() == reg || inst.src1() == reg =>
            {
                return true;
            }
            OpCode::ForLoopI32
                if inst.dst() == reg
                    || inst.src1() == reg
                    || ((inst.flags() & 0x80) != 0 && u16::from(inst.flags() & 0x1F) == reg) =>
            {
                return true;
            }
            OpCode::SetArray if inst.dst() == reg || inst.src1() == reg || inst.src2() == reg => {
                return true;
            }
            OpCode::MulAddF64 | OpCode::AddBandI32
                if inst.src1() == reg || inst.src2() == reg || u16::from(inst.flags()) == reg =>
            {
                return true;
            }
            OpCode::CallDirect
            | OpCode::TailCallDirect
            | OpCode::CallNativeDirect
            | OpCode::CallSymbol
            | OpCode::CallVirtual
            | OpCode::CallVirtualDirect
            | OpCode::CallInterface => {
                let base = inst.arg_base();
                let count = u16::from(inst.arg_count());
                if reg >= base && reg < base.saturating_add(count) {
                    return true;
                }
                if !saw_branch && (opcode == OpCode::TailCallDirect || inst.call_dst() == reg) {
                    return false;
                }
            }
            OpCode::Jump | OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                let target = (cur_pc as i32) + inst.jump_offset();
                if target <= cur_pc as i32 {
                    return true;
                }
                saw_branch = true;
            }
            OpCode::JumpIfI32Imm
            | OpCode::JumpIfLtI32
            | OpCode::JumpIfLteI32
            | OpCode::JumpIfGtI32
            | OpCode::JumpIfGteI32
            | OpCode::JumpIfEqI32
            | OpCode::JumpIfNeqI32
            | OpCode::JumpIfLtF64
            | OpCode::JumpIfLteF64
            | OpCode::GetArrayJumpIfFalse
            | OpCode::GetArrayJumpIfTrue
            | OpCode::ForLoopI32 => {
                let target = (cur_pc as i32) + i32::from(inst.src2() as i16);
                if target <= cur_pc as i32 {
                    return true;
                }
                saw_branch = true;
            }
            OpCode::Return | OpCode::ReturnValue if !saw_branch => {
                return false;
            }
            _ if !saw_branch && inst.dst() == reg => {
                return false;
            }
            _ => {}
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn emit_virtual_or_interface_call(
    builder: &mut FunctionBuilder<'_>,
    resolve_jit_ref: cranelift_codegen::ir::FuncRef,
    slow_callee_ref: cranelift_codegen::ir::FuncRef,
    jit_call_sig_ref: cranelift_codegen::ir::SigRef,
    rt_trap_null_ref: cranelift_codegen::ir::FuncRef,
    ctx_ptr: ClifVal,
    ptr_ty: types::Type,
    target_id: u32,
    arg_base: usize,
    arg_count: usize,
    call_dst: usize,
    is_dst_used: bool,
    mono_target: Option<(u32, cranelift_codegen::ir::FuncRef)>,
    vars: &VarTable,
    inst_ty: InstTypeInfo,
    opt_state: &mut BlockOptState,
) {
    let obj = builder.use_var(vars.get(arg_base as u16, RegType::Ref));
    opt_state.ensure_non_null(builder, ctx_ptr, arg_base as u16, obj, rt_trap_null_ref);

    let mut call_args = [builder.ins().iconst(types::I64, 0); 5];
    call_args[0] = ctx_ptr;
    for (arg_idx, arg_ty_opt) in inst_ty.call_args.iter().enumerate().take(arg_count.min(4)) {
        let reg_ty = arg_ty_opt.unwrap_or(RegType::I32);
        let reg_val = builder.use_var(vars.get((arg_base + arg_idx) as u16, reg_ty));
        call_args[1 + arg_idx] = encode_raw_word(builder, reg_val, reg_ty);
    }

    let merge_blk = builder.create_block();
    builder.append_block_param(merge_blk, types::I64);
    let resolve_blk = builder.create_block();

    if let Some((expected_vtable_id, direct_ref)) = mono_target {
        let heap_base = opt_state.get_heap_base(builder, ctx_ptr, ptr_ty);
        let obj_ext = builder.ins().uextend(ptr_ty, obj);
        let obj_ptr = builder.ins().iadd(heap_base, obj_ext);
        let actual_vt = builder.ins().load(types::I32, MemFlags::new(), obj_ptr, 0);
        let is_hit = builder
            .ins()
            .icmp_imm(IntCC::Equal, actual_vt, i64::from(expected_vtable_id));
        let mono_blk = builder.create_block();
        builder.ins().brif(is_hit, mono_blk, &[], resolve_blk, &[]);

        builder.switch_to_block(mono_blk);
        let direct_call = builder.ins().call(direct_ref, &call_args);
        let ret_raw = builder.inst_results(direct_call)[0];
        emit_trap_return_check(builder, ctx_ptr);
        builder.ins().jump(merge_blk, &[ret_raw]);
    } else {
        builder.ins().jump(resolve_blk, &[]);
    }

    builder.switch_to_block(resolve_blk);
    let target_val = builder.ins().iconst(types::I32, i64::from(target_id));
    let cnt_val = builder.ins().iconst(types::I32, arg_count as i64);
    let resolve_call = builder
        .ins()
        .call(resolve_jit_ref, &[ctx_ptr, obj, target_val, cnt_val]);
    let fn_ptr = builder.inst_results(resolve_call)[0];
    let is_jit = builder.ins().icmp_imm(IntCC::NotEqual, fn_ptr, 0);
    let indirect_blk = builder.create_block();
    let slow_blk = builder.create_block();
    builder.ins().brif(is_jit, indirect_blk, &[], slow_blk, &[]);

    builder.switch_to_block(indirect_blk);
    let ind_call = builder
        .ins()
        .call_indirect(jit_call_sig_ref, fn_ptr, &call_args);
    let ind_ret = builder.inst_results(ind_call)[0];
    emit_trap_return_check(builder, ctx_ptr);
    builder.ins().jump(merge_blk, &[ind_ret]);

    builder.switch_to_block(slow_blk);
    emit_trap_return_check(builder, ctx_ptr);
    emit_slow_call_dispatch(
        builder,
        slow_callee_ref,
        ctx_ptr,
        ptr_ty,
        None,
        target_id,
        arg_base,
        arg_count,
        call_dst,
        is_dst_used,
        false,
        vars,
        inst_ty,
    );
    let slow_ret = if let Some(dst_ty) = inst_ty.dst_ty {
        let val = builder.use_var(vars.get(call_dst as u16, dst_ty));
        encode_raw_word(builder, val, dst_ty)
    } else {
        builder.ins().iconst(types::I64, 0)
    };
    builder.ins().jump(merge_blk, &[slow_ret]);

    builder.switch_to_block(merge_blk);
    let merged_raw = builder.block_params(merge_blk)[0];
    if let Some(dst_ty) = inst_ty.dst_ty {
        let typed = decode_raw_word(builder, merged_raw, dst_ty);
        builder.def_var(vars.get(call_dst as u16, dst_ty), typed);
    }
    opt_state.invalidate_heap();
    opt_state.clear_non_null(call_dst as u16);
}

#[allow(clippy::too_many_arguments)]
fn emit_slow_call_dispatch(
    builder: &mut FunctionBuilder<'_>,
    callee_ref: cranelift_codegen::ir::FuncRef,
    ctx_ptr: ClifVal,
    ptr_ty: types::Type,
    binary_id: Option<u32>,
    target_id: u32,
    arg_base: usize,
    arg_count: usize,
    call_dst: usize,
    is_dst_used: bool,
    is_tail: bool,
    vars: &VarTable,
    inst_ty: InstTypeInfo,
) {
    let args_slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
        cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
        32,
        3,
    ));
    let out_slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
        cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
        8,
        3,
    ));
    let mut arg_types_mask: u32 = 0;
    for (arg_idx, arg_ty_opt) in inst_ty.call_args.iter().enumerate().take(arg_count.min(4)) {
        let reg_ty = arg_ty_opt.unwrap_or(RegType::I32);
        let reg_val = builder.use_var(vars.get((arg_base + arg_idx) as u16, reg_ty));
        arg_types_mask |= reg_ty.to_code() << (arg_idx * 2);
        let raw_val = encode_raw_word(builder, reg_val, reg_ty);
        builder
            .ins()
            .stack_store(raw_val, args_slot, (arg_idx * 8) as i32);
    }
    let ret_type_code = if is_dst_used {
        inst_ty.dst_ty.map_or(4, RegType::to_code)
    } else {
        4
    };
    let args_addr = builder.ins().stack_addr(ptr_ty, args_slot, 0);
    let out_addr = builder.ins().stack_addr(ptr_ty, out_slot, 0);
    let target_val = builder.ins().iconst(types::I32, i64::from(target_id));
    let cnt_val = builder.ins().iconst(types::I32, arg_count as i64);
    let mask_val = builder.ins().iconst(types::I32, i64::from(arg_types_mask));
    let ret_ty_val = builder.ins().iconst(types::I32, i64::from(ret_type_code));
    let call = if let Some(bin_id) = binary_id {
        let bin_val = builder.ins().iconst(types::I32, i64::from(bin_id));
        builder.ins().call(
            callee_ref,
            &[
                ctx_ptr, bin_val, target_val, args_addr, cnt_val, out_addr, mask_val, ret_ty_val,
            ],
        )
    } else {
        builder.ins().call(
            callee_ref,
            &[
                ctx_ptr, target_val, args_addr, cnt_val, out_addr, mask_val, ret_ty_val,
            ],
        )
    };
    let trapped = builder.inst_results(call)[0];
    let is_trap = builder.ins().icmp_imm(IntCC::NotEqual, trapped, 0);
    let trap_blk = builder.create_block();
    let ok_blk = builder.create_block();
    builder.ins().brif(is_trap, trap_blk, &[], ok_blk, &[]);
    builder.switch_to_block(trap_blk);
    let zero_ret = builder.ins().iconst(types::I64, 0);
    builder.ins().return_(&[zero_ret]);
    builder.switch_to_block(ok_blk);
    let ret_raw = builder.ins().stack_load(types::I64, out_slot, 0);
    if is_tail {
        builder.ins().return_(&[ret_raw]);
    } else if let Some(dst_ty) = inst_ty.dst_ty {
        let typed = decode_raw_word(builder, ret_raw, dst_ty);
        builder.def_var(vars.get(call_dst as u16, dst_ty), typed);
    }
}

fn emit_trap_return_check(builder: &mut FunctionBuilder<'_>, ctx_ptr: ClifVal) {
    let trap_off = std::mem::offset_of!(crate::vm::jit::JitContext, has_trap) as i32;
    let has_trap = builder
        .ins()
        .load(types::I8, MemFlags::new(), ctx_ptr, trap_off);
    let trapped = builder.ins().icmp_imm(IntCC::NotEqual, has_trap, 0);
    let trap_blk = builder.create_block();
    let ok_blk = builder.create_block();
    builder.ins().brif(trapped, trap_blk, &[], ok_blk, &[]);
    builder.switch_to_block(trap_blk);
    let zero_ret = builder.ins().iconst(types::I64, 0);
    builder.ins().return_(&[zero_ret]);
    builder.switch_to_block(ok_blk);
}

fn signed_div_magic_i32(divisor: i32) -> (i32, u32) {
    let abs_div = divisor.unsigned_abs();
    let base_t = 0x8000_0000u32.wrapping_add((divisor as u32) >> 31);
    let anc = base_t - 1 - (base_t % abs_div);
    let mut shift_p: u32 = 31;
    let mut quot1 = 0x8000_0000u32 / anc;
    let mut rem1 = 0x8000_0000u32 - quot1 * anc;
    let mut quot2 = 0x8000_0000u32 / abs_div;
    let mut rem2 = 0x8000_0000u32 - quot2 * abs_div;
    loop {
        shift_p += 1;
        quot1 <<= 1;
        rem1 <<= 1;
        if rem1 >= anc {
            quot1 += 1;
            rem1 -= anc;
        }
        quot2 <<= 1;
        rem2 <<= 1;
        if rem2 >= abs_div {
            quot2 += 1;
            rem2 -= abs_div;
        }
        let delta = abs_div - rem2;
        if quot1 < delta || (quot1 == delta && rem1 == 0) {
            continue;
        }
        break;
    }
    let mut magic_u32 = quot2 + 1;
    if divisor < 0 {
        magic_u32 = magic_u32.wrapping_neg();
    }
    (magic_u32 as i32, shift_p - 32)
}

#[cfg(test)]
mod tests {
    use super::signed_div_magic_i32;

    #[test]
    fn test_signed_div_magic_i32_exactness() {
        let divisors = [-20, -10, -3, 2, 3, 4, 5, 7, 10, 16, 20, 100];
        let samples = [
            i32::MIN,
            i32::MIN + 1,
            -1000,
            -100,
            -21,
            -20,
            -19,
            -3,
            -1,
            0,
            1,
            3,
            19,
            20,
            21,
            100,
            1000,
            i32::MAX - 1,
            i32::MAX,
        ];
        for divisor in divisors {
            let (magic, shift) = signed_div_magic_i32(divisor);
            for sample in samples {
                let prod = (i64::from(sample) * i64::from(magic)) >> 32;
                let mut quot = prod as i32;
                if divisor > 0 && magic < 0 {
                    quot = quot.wrapping_add(sample);
                } else if divisor < 0 && magic > 0 {
                    quot = quot.wrapping_sub(sample);
                }
                if shift > 0 {
                    quot >>= shift;
                }
                let sign_bit = ((quot as u32) >> 31) as i32;
                let trunc_quot = quot.wrapping_add(sign_bit);
                let rem = sample.wrapping_sub(trunc_quot.wrapping_mul(divisor));
                assert_eq!(rem, sample % divisor, "sample={sample}, divisor={divisor}");
            }
        }
    }
}
