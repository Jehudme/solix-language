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

    let Some(sig_info) = infer_signature_and_types(&compiled_fn) else {
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

    let mut clif_ctx = {
        let engine = vm.jit_engine.as_mut()?;
        engine.module.make_context()
    };
    clif_ctx.func.signature = clif_sig;

    let mut fb_ctx = FunctionBuilderContext::new();
    let build_ok = {
        let engine = vm.jit_engine.as_mut()?;
        translate_function(
            &compiled_fn,
            meta.binary_id as u32,
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

struct InferredFunctionTypes {
    reg_types: Vec<RegType>,
    func_sig: JitFuncSig,
}

fn reg_type_to_clif(reg_ty: RegType) -> types::Type {
    match reg_ty {
        RegType::I32 | RegType::Ref => types::I32,
        RegType::I64 => types::I64,
        RegType::F64 => types::F64,
    }
}

fn infer_signature_and_types(compiled_fn: &CompiledFunction) -> Option<InferredFunctionTypes> {
    let num_regs = (compiled_fn.num_registers as usize).max(compiled_fn.param_count as usize);
    let mut reg_types: Vec<Option<RegType>> = vec![None; num_regs];
    let mut return_type: Option<RegType> = None;
    let mut has_return = false;

    for &ref_reg in &compiled_fn.reference_registers {
        if (ref_reg as usize) < num_regs {
            reg_types[ref_reg as usize] = Some(RegType::Ref);
        }
    }

    let set_reg = |slots: &mut [Option<RegType>], reg: u16, ty: RegType| -> bool {
        let idx = reg as usize;
        if idx >= slots.len() {
            return false;
        }
        if let Some(existing) = slots[idx] {
            existing == ty
        } else {
            slots[idx] = Some(ty);
            true
        }
    };

    for _pass in 0..3 {
        for inst in &compiled_fn.instructions {
            let opcode = OpCode::from_raw_u8(inst.raw_opcode())?;
            match opcode {
                OpCode::Nop | OpCode::Jump => {}
                OpCode::Mov => {
                    let src_idx = inst.src1() as usize;
                    let dst_idx = inst.dst() as usize;
                    if src_idx >= num_regs || dst_idx >= num_regs {
                        return None;
                    }
                    if let Some(reg_ty) = reg_types[src_idx] {
                        if !set_reg(&mut reg_types, inst.dst(), reg_ty) {
                            return None;
                        }
                    } else if let Some(reg_ty) = reg_types[dst_idx] {
                        let _ = set_reg(&mut reg_types, inst.src1(), reg_ty);
                    }
                }
                OpCode::LoadI32 | OpCode::LoadBool => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32) {
                        return None;
                    }
                }
                OpCode::LoadI64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I64) {
                        return None;
                    }
                }
                OpCode::LoadF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64) {
                        return None;
                    }
                }
                OpCode::LoadNull => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::Ref) {
                        return None;
                    }
                }
                OpCode::LoadConst => {
                    if reg_types
                        .get(inst.dst() as usize)
                        .copied()
                        .flatten()
                        .is_none()
                        && !set_reg(&mut reg_types, inst.dst(), RegType::Ref)
                    {
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
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::NegI32 | OpCode::Bnot | OpCode::Not => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::AddI32Imm | OpCode::SubI32Imm | OpCode::MulI32Imm | OpCode::ModI32Imm => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::AddBandI32 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.flags() as u16, RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::AddI64 | OpCode::SubI64 | OpCode::MulI64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I64)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::I64)
                    {
                        return None;
                    }
                }
                OpCode::AddI64Imm => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I64)
                    {
                        return None;
                    }
                }
                OpCode::AddF64 | OpCode::SubF64 | OpCode::MulF64 | OpCode::DivF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::F64)
                    {
                        return None;
                    }
                }
                OpCode::NegF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::F64)
                    {
                        return None;
                    }
                }
                OpCode::AddF64Imm | OpCode::SubF64Imm | OpCode::MulF64Imm => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::F64)
                    {
                        return None;
                    }
                }
                OpCode::MulAddF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.flags() as u16, RegType::F64)
                    {
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
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::LtF64
                | OpCode::LteF64
                | OpCode::GtF64
                | OpCode::GteF64
                | OpCode::EqF64
                | OpCode::NeqF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::F64)
                    {
                        return None;
                    }
                }
                OpCode::JumpIfTrue | OpCode::JumpIfFalse | OpCode::JumpIfI32Imm => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32) {
                        return None;
                    }
                }
                OpCode::JumpIfLtI32
                | OpCode::JumpIfLteI32
                | OpCode::JumpIfGtI32
                | OpCode::JumpIfGteI32
                | OpCode::JumpIfEqI32
                | OpCode::JumpIfNeqI32 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::JumpIfLtF64 | OpCode::JumpIfLteF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::F64)
                    {
                        return None;
                    }
                }
                OpCode::ForLoopI32 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::ConvIToF => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64) {
                        return None;
                    }
                    if reg_types
                        .get(inst.src1() as usize)
                        .copied()
                        .flatten()
                        .is_none()
                    {
                        let _ = set_reg(&mut reg_types, inst.src1(), RegType::I32);
                    }
                }
                OpCode::ConvFToI => {
                    if !set_reg(&mut reg_types, inst.src1(), RegType::F64) {
                        return None;
                    }
                    if reg_types
                        .get(inst.dst() as usize)
                        .copied()
                        .flatten()
                        .is_none()
                    {
                        let _ = set_reg(&mut reg_types, inst.dst(), RegType::I32);
                    }
                }
                OpCode::AllocObject | OpCode::AllocArray => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::Ref) {
                        return None;
                    }
                }
                OpCode::ArrayLength => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::Ref)
                    {
                        return None;
                    }
                }
                OpCode::GetFieldI32 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::Ref)
                    {
                        return None;
                    }
                }
                OpCode::GetFieldF64 => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::Ref)
                    {
                        return None;
                    }
                }
                OpCode::SetFieldI32 => {
                    if !set_reg(&mut reg_types, inst.src1(), RegType::Ref)
                        || !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                    {
                        return None;
                    }
                }
                OpCode::SetFieldF64 => {
                    if !set_reg(&mut reg_types, inst.src1(), RegType::Ref)
                        || !set_reg(&mut reg_types, inst.dst(), RegType::F64)
                    {
                        return None;
                    }
                }
                OpCode::IncRef | OpCode::DecRef => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::Ref) {
                        return None;
                    }
                }
                OpCode::EqString | OpCode::NeqString | OpCode::EqRef | OpCode::NeqRef => {
                    if !set_reg(&mut reg_types, inst.dst(), RegType::I32)
                        || !set_reg(&mut reg_types, inst.src1(), RegType::Ref)
                        || !set_reg(&mut reg_types, inst.src2(), RegType::Ref)
                    {
                        return None;
                    }
                }
                OpCode::Return => {
                    has_return = true;
                }
                OpCode::ReturnValue => {
                    has_return = true;
                    if let Some(ty) = reg_types.get(inst.dst() as usize).copied().flatten() {
                        if let Some(prev) = return_type
                            && prev != ty
                        {
                            return None;
                        }
                        return_type = Some(ty);
                    }
                }
                OpCode::CallDirect | OpCode::TailCallDirect => {
                    // Leave return/arg registers flexible unless already constrained.
                }
                // Fall back to Tier 0 for opcodes not yet lowered in MVP.
                _ => return None,
            }
        }
    }

    if !has_return {
        return None;
    }

    let resolved_regs: Vec<RegType> = reg_types
        .into_iter()
        .map(|slot| slot.unwrap_or(RegType::I32))
        .collect();
    let param_count = compiled_fn.param_count as usize;
    let param_types = resolved_regs[..param_count].to_vec();

    Some(InferredFunctionTypes {
        reg_types: resolved_regs,
        func_sig: JitFuncSig {
            param_types,
            return_type,
        },
    })
}

fn translate_function(
    compiled_fn: &CompiledFunction,
    binary_id: u32,
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

    let entry_block = blocks_by_pc[&0];
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);

    let ctx_ptr = builder.block_params(entry_block)[0];
    let mut vars: Vec<Variable> = Vec::with_capacity(types_info.reg_types.len());
    for (reg_idx, &reg_ty) in types_info.reg_types.iter().enumerate() {
        let var = Variable::from_u32(reg_idx as u32);
        builder.declare_var(var, reg_type_to_clif(reg_ty));
        let zero_val = match reg_ty {
            RegType::I32 | RegType::Ref => builder.ins().iconst(types::I32, 0),
            RegType::I64 => builder.ins().iconst(types::I64, 0),
            RegType::F64 => builder.ins().f64const(0.0),
        };
        builder.def_var(var, zero_val);
        if reg_idx < types_info.func_sig.param_types.len() {
            let raw_arg = builder.block_params(entry_block)[1 + reg_idx];
            let typed_arg = match reg_ty {
                RegType::I32 | RegType::Ref => builder.ins().ireduce(types::I32, raw_arg),
                RegType::I64 => raw_arg,
                RegType::F64 => builder.ins().bitcast(types::F64, MemFlags::new(), raw_arg),
            };
            builder.def_var(var, typed_arg);
        }
        vars.push(var);
    }

    let self_func_ref = engine
        .module
        .declare_func_in_func(self_clif_id, builder.func);
    let rt_load_const_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_load_const, builder.func);
    let rt_alloc_obj_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_alloc_object, builder.func);
    let rt_inc_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_inc_ref, builder.func);
    let rt_dec_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_dec_ref, builder.func);
    let rt_call_slow_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_call_slow, builder.func);
    let rt_trap_null_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_trap_null, builder.func);
    let rt_trap_div_zero_ref = engine
        .module
        .declare_func_in_func(engine.rt_symbols.rt_trap_div_zero, builder.func);

    let mut block_terminated = false;

    for (pc, inst) in compiled_fn.instructions.iter().enumerate() {
        if pc > 0
            && let Some(&next_blk) = blocks_by_pc.get(&pc)
        {
            if !block_terminated {
                builder.ins().jump(next_blk, &[]);
            }
            builder.switch_to_block(next_blk);
            block_terminated = false;
        }
        if block_terminated {
            continue;
        }

        let opcode = OpCode::from_raw_u8(inst.raw_opcode()).ok_or(())?;
        match opcode {
            OpCode::Nop => {}
            OpCode::Mov => {
                let val = builder.use_var(vars[inst.src1() as usize]);
                builder.def_var(vars[inst.dst() as usize], val);
            }
            OpCode::LoadI32 => {
                let val = builder
                    .ins()
                    .iconst(types::I32, i64::from(inst.imm32_signed()));
                builder.def_var(vars[inst.dst() as usize], val);
            }
            OpCode::LoadI64 => {
                let val = builder
                    .ins()
                    .iconst(types::I64, i64::from(inst.imm32_signed()));
                builder.def_var(vars[inst.dst() as usize], val);
            }
            OpCode::LoadBool => {
                let val = builder
                    .ins()
                    .iconst(types::I32, if inst.imm32() != 0 { 1 } else { 0 });
                builder.def_var(vars[inst.dst() as usize], val);
            }
            OpCode::LoadNull => {
                let val = builder.ins().iconst(types::I32, 0);
                builder.def_var(vars[inst.dst() as usize], val);
            }
            OpCode::LoadF64 | OpCode::LoadConst => {
                let bin_arg = builder.ins().iconst(types::I32, i64::from(binary_id));
                let idx_arg = builder.ins().iconst(types::I32, i64::from(inst.imm32()));
                let call = builder
                    .ins()
                    .call(rt_load_const_ref, &[ctx_ptr, bin_arg, idx_arg]);
                let raw_word = builder.inst_results(call)[0];
                let dst_ty = types_info.reg_types[inst.dst() as usize];
                let typed = decode_raw_word(&mut builder, raw_word, dst_ty);
                builder.def_var(vars[inst.dst() as usize], typed);
            }
            OpCode::AddI32 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().iadd(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::SubI32 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().isub(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::MulI32 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().imul(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::DivI32 | OpCode::ModI32 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
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
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Band => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().band(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Bor => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().bor(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Bxor => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().bxor(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Shl => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().ishl(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Shr => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().sshr(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Lshr => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().ushr(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::NegI32 => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().ineg(src);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Bnot => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().bnot(src);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Not => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let is_zero = builder.ins().icmp_imm(IntCC::Equal, src, 0);
                let res = builder.ins().uextend(types::I32, is_zero);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::AddI32Imm => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().iadd_imm(src, i64::from(inst.src2() as i16));
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::SubI32Imm => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().iadd_imm(src, -i64::from(inst.src2() as i16));
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::MulI32Imm => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().imul_imm(src, i64::from(inst.src2() as i16));
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::ModI32Imm => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let divisor = i64::from(inst.src2() as i16);
                if divisor == 0 {
                    builder.ins().call(rt_trap_div_zero_ref, &[ctx_ptr]);
                    let zero_ret = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero_ret]);
                    block_terminated = true;
                } else {
                    let div_val = builder.ins().iconst(types::I32, divisor);
                    let res = builder.ins().srem(src, div_val);
                    builder.def_var(vars[inst.dst() as usize], res);
                }
            }
            OpCode::AddBandI32 => {
                let acc = builder.use_var(vars[inst.flags() as usize]);
                let val = builder.use_var(vars[inst.src1() as usize]);
                let mask = builder.use_var(vars[inst.src2() as usize]);
                let sum = builder.ins().iadd(acc, val);
                let masked = builder.ins().band(sum, mask);
                builder.def_var(vars[inst.dst() as usize], masked);
            }
            OpCode::AddI64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().iadd(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::SubI64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().isub(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::MulI64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().imul(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::AddI64Imm => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().iadd_imm(src, i64::from(inst.src2() as i16));
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::AddF64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().fadd(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::SubF64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().fsub(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::MulF64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().fmul(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::DivF64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let res = builder.ins().fdiv(lhs, rhs);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::NegF64 => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().fneg(src);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::AddF64Imm | OpCode::SubF64Imm | OpCode::MulF64Imm => {
                let bin_arg = builder.ins().iconst(types::I32, i64::from(binary_id));
                let idx_arg = builder.ins().iconst(types::I32, i64::from(inst.src2()));
                let call = builder
                    .ins()
                    .call(rt_load_const_ref, &[ctx_ptr, bin_arg, idx_arg]);
                let raw_word = builder.inst_results(call)[0];
                let const_f64 = builder.ins().bitcast(types::F64, MemFlags::new(), raw_word);
                let src_f64 = builder.use_var(vars[inst.src1() as usize]);
                let res = match opcode {
                    OpCode::AddF64Imm => builder.ins().fadd(src_f64, const_f64),
                    OpCode::SubF64Imm => builder.ins().fsub(src_f64, const_f64),
                    _ => builder.ins().fmul(src_f64, const_f64),
                };
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::MulAddF64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
                let acc = builder.use_var(vars[inst.flags() as usize]);
                let prod = builder.ins().fmul(lhs, rhs);
                let res = builder.ins().fadd(prod, acc);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::EqI32
            | OpCode::NeqI32
            | OpCode::LtI32
            | OpCode::LteI32
            | OpCode::GtI32
            | OpCode::GteI32
            | OpCode::EqBool
            | OpCode::NeqBool => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
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
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::EqF64
            | OpCode::NeqF64
            | OpCode::LtF64
            | OpCode::LteF64
            | OpCode::GtF64
            | OpCode::GteF64 => {
                let lhs = builder.use_var(vars[inst.src1() as usize]);
                let rhs = builder.use_var(vars[inst.src2() as usize]);
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
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::ConvIToF => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let res = builder.ins().fcvt_from_sint(types::F64, src);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::ConvFToI => {
                let src = builder.use_var(vars[inst.src1() as usize]);
                let dst_ty = types_info.reg_types[inst.dst() as usize];
                let res = builder
                    .ins()
                    .fcvt_to_sint_sat(reg_type_to_clif(dst_ty), src);
                builder.def_var(vars[inst.dst() as usize], res);
            }
            OpCode::Jump => {
                let target_pc = ((pc as i32) + inst.jump_offset()) as usize;
                let target_blk = blocks_by_pc[&target_pc];
                builder.ins().jump(target_blk, &[]);
                block_terminated = true;
            }
            OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
                let cond = builder.use_var(vars[inst.dst() as usize]);
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
                let val = builder.use_var(vars[inst.dst() as usize]);
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
                let lhs = builder.use_var(vars[inst.dst() as usize]);
                let rhs = builder.use_var(vars[inst.src1() as usize]);
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
                let lhs = builder.use_var(vars[inst.dst() as usize]);
                let rhs = builder.use_var(vars[inst.src1() as usize]);
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
                    builder.use_var(vars[(flags & 0x1F) as usize])
                };
                let cur_idx = builder.use_var(vars[inst.dst() as usize]);
                let next_idx = builder.ins().iadd(cur_idx, step);
                builder.def_var(vars[inst.dst() as usize], next_idx);
                let bound = builder.use_var(vars[inst.src1() as usize]);
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
                let vtable = builder.ins().iconst(types::I32, i64::from(inst.src1()));
                let size = builder.ins().iconst(types::I32, i64::from(inst.src2()));
                let is_pod = builder
                    .ins()
                    .iconst(types::I8, if inst.is_pod() { 1 } else { 0 });
                let call = builder
                    .ins()
                    .call(rt_alloc_obj_ref, &[ctx_ptr, vtable, size, is_pod]);
                let addr = builder.inst_results(call)[0];
                builder.def_var(vars[inst.dst() as usize], addr);
                emit_trap_return_check(&mut builder, ctx_ptr);
            }
            OpCode::GetFieldI32 | OpCode::GetFieldF64 => {
                let obj = builder.use_var(vars[inst.src1() as usize]);
                emit_null_check(&mut builder, ctx_ptr, obj, rt_trap_null_ref);
                let heap_base = builder.ins().load(ptr_ty, MemFlags::new(), ctx_ptr, 0);
                let obj_ext = builder.ins().uextend(ptr_ty, obj);
                let field_off = inst.src2() as i32;
                let ptr = builder.ins().iadd(heap_base, obj_ext);
                let read_ty = if opcode == OpCode::GetFieldI32 {
                    types::I32
                } else {
                    types::F64
                };
                let val = builder.ins().load(read_ty, MemFlags::new(), ptr, field_off);
                builder.def_var(vars[inst.dst() as usize], val);
            }
            OpCode::SetFieldI32 | OpCode::SetFieldF64 => {
                let obj = builder.use_var(vars[inst.src1() as usize]);
                emit_null_check(&mut builder, ctx_ptr, obj, rt_trap_null_ref);
                let heap_base = builder.ins().load(ptr_ty, MemFlags::new(), ctx_ptr, 0);
                let obj_ext = builder.ins().uextend(ptr_ty, obj);
                let field_off = inst.src2() as i32;
                let ptr = builder.ins().iadd(heap_base, obj_ext);
                let val = builder.use_var(vars[inst.dst() as usize]);
                builder.ins().store(MemFlags::new(), val, ptr, field_off);
            }
            OpCode::IncRef => {
                let addr = builder.use_var(vars[inst.dst() as usize]);
                builder.ins().call(rt_inc_ref, &[ctx_ptr, addr]);
            }
            OpCode::DecRef => {
                let addr = builder.use_var(vars[inst.dst() as usize]);
                builder.ins().call(rt_dec_ref, &[ctx_ptr, addr]);
            }
            OpCode::CallDirect | OpCode::TailCallDirect => {
                let target_id = inst.call_target() as u32;
                let arg_base = inst.arg_base() as usize;
                let arg_count = inst.arg_count() as usize;
                let call_dst = inst.call_dst() as usize;

                if let Some(target_clif_id) = engine
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
                        let reg_val = builder.use_var(vars[arg_base + arg_idx]);
                        let reg_ty = types_info.reg_types[arg_base + arg_idx];
                        call_args[1 + arg_idx] = encode_raw_word(&mut builder, reg_val, reg_ty);
                    }
                    let call = builder.ins().call(target_ref, &call_args);
                    let ret_raw = builder.inst_results(call)[0];
                    emit_trap_return_check(&mut builder, ctx_ptr);
                    if opcode == OpCode::TailCallDirect {
                        builder.ins().return_(&[ret_raw]);
                        block_terminated = true;
                    } else if call_dst < vars.len() {
                        let dst_ty = types_info.reg_types[call_dst];
                        let typed = decode_raw_word(&mut builder, ret_raw, dst_ty);
                        builder.def_var(vars[call_dst], typed);
                    }
                } else {
                    // Marshal arguments onto stack slot and call `rt_call_slow`.
                    let args_slot =
                        builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                            32,
                            3,
                        ));
                    let out_slot =
                        builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                            8,
                            3,
                        ));
                    let mut arg_types_mask: u32 = 0;
                    for arg_idx in 0..arg_count.min(4) {
                        let reg_val = builder.use_var(vars[arg_base + arg_idx]);
                        let reg_ty = types_info.reg_types[arg_base + arg_idx];
                        arg_types_mask |= reg_ty.to_code() << (arg_idx * 2);
                        let raw_val = encode_raw_word(&mut builder, reg_val, reg_ty);
                        builder
                            .ins()
                            .stack_store(raw_val, args_slot, (arg_idx * 8) as i32);
                    }
                    let ret_type_code = if call_dst < vars.len() {
                        types_info.reg_types[call_dst].to_code()
                    } else {
                        4
                    };
                    let args_addr = builder.ins().stack_addr(ptr_ty, args_slot, 0);
                    let out_addr = builder.ins().stack_addr(ptr_ty, out_slot, 0);
                    let fid_val = builder.ins().iconst(types::I32, i64::from(target_id));
                    let cnt_val = builder.ins().iconst(types::I32, arg_count as i64);
                    let mask_val = builder.ins().iconst(types::I32, i64::from(arg_types_mask));
                    let ret_ty_val = builder.ins().iconst(types::I32, i64::from(ret_type_code));
                    let call = builder.ins().call(
                        rt_call_slow_ref,
                        &[
                            ctx_ptr, fid_val, args_addr, cnt_val, out_addr, mask_val, ret_ty_val,
                        ],
                    );
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
                    if opcode == OpCode::TailCallDirect {
                        builder.ins().return_(&[ret_raw]);
                        block_terminated = true;
                    } else if call_dst < vars.len() {
                        let dst_ty = types_info.reg_types[call_dst];
                        let typed = decode_raw_word(&mut builder, ret_raw, dst_ty);
                        builder.def_var(vars[call_dst], typed);
                    }
                }
            }
            OpCode::Return => {
                let zero_ret = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero_ret]);
                block_terminated = true;
            }
            OpCode::ReturnValue => {
                let reg_idx = inst.dst() as usize;
                let reg_val = builder.use_var(vars[reg_idx]);
                let reg_ty = types_info.reg_types[reg_idx];
                let ret_raw = encode_raw_word(&mut builder, reg_val, reg_ty);
                builder.ins().return_(&[ret_raw]);
                block_terminated = true;
            }
            _ => return Err(()),
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

fn emit_trap_return_check(builder: &mut FunctionBuilder<'_>, ctx_ptr: ClifVal) {
    // has_trap is at offset 24 on 64-bit platforms (heap_ptr: 8, heap_len: 8, vm_ptr: 8)
    let has_trap = builder.ins().load(types::I8, MemFlags::new(), ctx_ptr, 24);
    let trapped = builder.ins().icmp_imm(IntCC::NotEqual, has_trap, 0);
    let trap_blk = builder.create_block();
    let ok_blk = builder.create_block();
    builder.ins().brif(trapped, trap_blk, &[], ok_blk, &[]);
    builder.switch_to_block(trap_blk);
    let zero_ret = builder.ins().iconst(types::I64, 0);
    builder.ins().return_(&[zero_ret]);
    builder.switch_to_block(ok_blk);
}
