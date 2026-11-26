//! High-performance inlined instruction dispatcher and register window caching.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::exceptions::RuntimeError;
use crate::frame::CallFrame;
use crate::memory::layout::BlockTag;
use crate::value::Value;
use crate::vm::engine::{MonomorphicCallSite, VirtualMachine};
use solix_common::bytecode::{ConstantPoolValue, Instruction, OpCode};

/// Compact 8-byte boxed trap wrapper keeping error paths out of hot register allocation and stack frames.
#[repr(transparent)]
pub(crate) struct VmTrap(Box<RuntimeError>);

impl VmTrap {
    #[cold]
    #[inline(never)]
    pub(crate) fn from_error(error: RuntimeError) -> Self {
        Self(Box::new(error))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn into_inner(self) -> RuntimeError {
        *self.0
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn null_reference() -> Self {
        if std::env::var_os("SOLIX_TRACE_TRAPS").is_some() {
            eprintln!(
                "TRAP null_reference:\n{}",
                std::backtrace::Backtrace::force_capture()
            );
        }
        Self(Box::new(RuntimeError::NullReferenceAccess))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn divide_by_zero() -> Self {
        Self(Box::new(RuntimeError::DivideByZero))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn type_mismatch(expected: &'static str, found_value: &Value) -> Self {
        Self(Box::new(RuntimeError::TypeMismatch {
            expected: expected.to_string(),
            found: found_value.type_name().to_string(),
        }))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn invalid_static_offset(offset: u32, total_static: u32) -> Self {
        Self(Box::new(RuntimeError::InvalidStaticOffset {
            offset,
            total_static,
        }))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn index_out_of_bounds(index: i64, length: usize) -> Self {
        Self(Box::new(RuntimeError::IndexOutOfBounds { index, length }))
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(crate) fn out_of_memory(requested: usize, available: usize) -> Self {
        Self(Box::new(RuntimeError::OutOfMemory {
            requested,
            available,
        }))
    }
}

impl From<RuntimeError> for VmTrap {
    #[cold]
    #[inline(never)]
    fn from(error: RuntimeError) -> Self {
        Self(Box::new(error))
    }
}

#[cold]
#[inline(never)]
fn invalid_opcode_error(pc: u32, raw_opcode: u8) -> VmTrap {
    VmTrap::from_error(RuntimeError::InvalidOpCode {
        pc,
        opcode: raw_opcode,
    })
}

#[cold]
#[inline(never)]
fn stack_overflow_error(current_depth: usize, max_depth: usize) -> VmTrap {
    VmTrap::from_error(RuntimeError::StackOverflow {
        current_depth,
        max_depth,
    })
}

#[cold]
#[inline(never)]
fn function_not_found_error(function_id: u32) -> VmTrap {
    VmTrap::from_error(RuntimeError::FunctionNotFound {
        symbol_or_id: format!("#{function_id}"),
    })
}

#[cold]
#[inline(never)]
fn f64_pair_type_mismatch_error(left_val: &Value, right_val: &Value) -> VmTrap {
    VmTrap::from_error(RuntimeError::TypeMismatch {
        expected: "Float64 and Float64".to_string(),
        found: format!(
            "{} ({left_val}), {} ({right_val})",
            left_val.type_name(),
            right_val.type_name()
        ),
    })
}

#[cold]
#[inline(never)]
fn mul_add_f64_type_mismatch_cold(left_val: &Value, right_val: &Value, acc_val: &Value) -> VmTrap {
    VmTrap::from_error(RuntimeError::TypeMismatch {
        expected: "Float64, Float64, and Float64".to_string(),
        found: format!(
            "{} ({left_val}), {} ({right_val}), {} ({acc_val})",
            left_val.type_name(),
            right_val.type_name(),
            acc_val.type_name()
        ),
    })
}

#[cold]
#[inline(never)]
fn f64_type_mismatch_error(found_val: &Value) -> VmTrap {
    VmTrap::from_error(RuntimeError::TypeMismatch {
        expected: "Float64".to_string(),
        found: format!("{} ({found_val})", found_val.type_name()),
    })
}

#[cold]
#[inline(never)]
fn non_callable_type_mismatch_error(callable_val: &Value) -> VmTrap {
    VmTrap::from_error(RuntimeError::TypeMismatch {
        expected: "callable".to_string(),
        found: callable_val.type_name().to_string(),
    })
}

#[cold]
#[inline(never)]
fn vtable_not_found_error(vtable_id: u32) -> RuntimeError {
    RuntimeError::FunctionNotFound {
        symbol_or_id: format!("vtable #{vtable_id}"),
    }
}

#[cold]
#[inline(never)]
fn vtable_slot_not_found_error(slot: usize, class_symbol: &str) -> RuntimeError {
    RuntimeError::FunctionNotFound {
        symbol_or_id: format!("vtable slot {slot} in class {class_symbol}"),
    }
}

#[cold]
#[inline(never)]
fn interface_slot_not_found_error(slot: usize, class_symbol: &str) -> RuntimeError {
    RuntimeError::FunctionNotFound {
        symbol_or_id: format!("interface slot {slot} in class {class_symbol}"),
    }
}

#[cold]
#[inline(never)]
fn constant_pool_missing_error(binary_id: usize) -> RuntimeError {
    RuntimeError::BinaryLoadError {
        reason: format!("constant pool missing for binary {binary_id}"),
    }
}

#[cold]
#[inline(never)]
fn constant_pool_symbol_index_error(pool_idx: usize) -> RuntimeError {
    RuntimeError::TypeMismatch {
        expected: "symbol string in constant pool".to_string(),
        found: format!("index {pool_idx}"),
    }
}

#[cold]
#[inline(never)]
fn constant_pool_symbol_type_error(pool_val: &ConstantPoolValue) -> RuntimeError {
    RuntimeError::TypeMismatch {
        expected: "string symbol".to_string(),
        found: format!("{pool_val}"),
    }
}

#[cold]
#[inline(never)]
fn invalid_cast_tag_error(val: Value, tag: u8) -> RuntimeError {
    RuntimeError::InvalidCast {
        from: format!("{val}"),
        to: format!("type tag #{tag}"),
    }
}

#[cold]
#[inline(never)]
fn invalid_cast_vtable_error(obj_addr: u32, target_vtable_id: u32) -> RuntimeError {
    RuntimeError::InvalidCast {
        from: format!("ref 0x{obj_addr:08X}"),
        to: format!("vtable #{target_vtable_id}"),
    }
}

#[cold]
#[inline(never)]
fn grow_register_stack(register_stack: &mut Vec<Value>, required: usize) {
    register_stack.resize(required + 4096, Value::Null);
}

#[cold]
#[inline(never)]
fn read_i32_slow(val_ref: &Value) -> Result<i32, VmTrap> {
    if let Value::I64(num_val) = *val_ref {
        i32::try_from(num_val).map_err(|_| VmTrap::type_mismatch("int32", val_ref))
    } else {
        Err(VmTrap::type_mismatch("int32", val_ref))
    }
}

#[inline(always)]
fn read_i32_fast(val_ref: &Value) -> Result<i32, VmTrap> {
    if let Value::I32(num_val) = *val_ref {
        Ok(num_val)
    } else {
        read_i32_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_i64_slow(val_ref: &Value) -> Result<i64, VmTrap> {
    if let Value::I32(num_val) = *val_ref {
        Ok(num_val as i64)
    } else {
        Err(VmTrap::type_mismatch("int64", val_ref))
    }
}

#[inline(always)]
fn read_i64_fast(val_ref: &Value) -> Result<i64, VmTrap> {
    if let Value::I64(num_val) = *val_ref {
        Ok(num_val)
    } else {
        read_i64_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_i32_pair_slow(left_ref: &Value, right_ref: &Value) -> Result<(i32, i32), VmTrap> {
    Ok((read_i32_fast(left_ref)?, read_i32_fast(right_ref)?))
}

#[cold]
#[inline(never)]
fn read_i64_pair_slow(left_ref: &Value, right_ref: &Value) -> Result<(i64, i64), VmTrap> {
    Ok((read_i64_fast(left_ref)?, read_i64_fast(right_ref)?))
}

#[cold]
#[inline(never)]
fn read_array_index_slow(val_ref: &Value) -> Result<i64, VmTrap> {
    if let Value::I64(num_val) = *val_ref {
        Ok(num_val)
    } else {
        Err(VmTrap::type_mismatch("int64", val_ref))
    }
}

#[inline(always)]
fn read_array_index_fast(val_ref: &Value) -> Result<i64, VmTrap> {
    if let Value::I32(num_val) = *val_ref {
        Ok(num_val as i64)
    } else {
        read_array_index_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_f64_slow(val_ref: &Value) -> Result<f64, VmTrap> {
    match *val_ref {
        Value::I32(num_val) => Ok(num_val as f64),
        Value::I64(num_val) => Ok(num_val as f64),
        _ => Err(VmTrap::type_mismatch("float64", val_ref)),
    }
}

#[inline(always)]
fn read_f64_fast(val_ref: &Value) -> Result<f64, VmTrap> {
    if let Value::F64(float_val) = *val_ref {
        Ok(float_val)
    } else {
        read_f64_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_bool_slow(val_ref: &Value) -> Result<bool, VmTrap> {
    Err(VmTrap::type_mismatch("bool", val_ref))
}

#[inline(always)]
fn read_bool_fast(val_ref: &Value) -> Result<bool, VmTrap> {
    if let Value::Bool(bool_val) = *val_ref {
        Ok(bool_val)
    } else {
        read_bool_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_nonnull_ref_slow(val_ref: &Value) -> Result<u32, VmTrap> {
    match *val_ref {
        Value::Null | Value::Ref(0) => Err(VmTrap::null_reference()),
        _ => Err(VmTrap::type_mismatch("ref", val_ref)),
    }
}

#[inline(always)]
fn read_nonnull_ref_fast(val_ref: &Value) -> Result<u32, VmTrap> {
    if let Value::Ref(heap_addr) = *val_ref
        && heap_addr != 0
    {
        Ok(heap_addr)
    } else {
        read_nonnull_ref_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_ref_slow(val_ref: &Value) -> Result<u32, VmTrap> {
    match *val_ref {
        Value::Null => Ok(0),
        _ => Err(VmTrap::type_mismatch("ref", val_ref)),
    }
}

#[inline(always)]
fn read_ref_fast(val_ref: &Value) -> Result<u32, VmTrap> {
    if let Value::Ref(heap_addr) = *val_ref {
        Ok(heap_addr)
    } else {
        read_ref_slow(val_ref)
    }
}

#[cold]
#[inline(never)]
fn read_truthy_slow(val_ref: &Value) -> bool {
    val_ref.is_truthy()
}

#[inline(always)]
fn read_truthy_fast(val_ref: &Value) -> bool {
    match *val_ref {
        Value::Bool(flag) => flag,
        Value::I32(num_val) => num_val != 0,
        Value::Ref(addr) => addr != 0,
        Value::Null => false,
        _ => read_truthy_slow(val_ref),
    }
}

#[cold]
#[inline(never)]
fn values_equal_slow(left_ref: &Value, right_ref: &Value) -> bool {
    left_ref.is_equal_to(right_ref)
}

#[cold]
#[inline(never)]
fn band_slow(left_ref: &Value, right_ref: &Value) -> Result<Value, VmTrap> {
    Ok(Value::I64(
        read_i64_fast(left_ref)? & read_i64_fast(right_ref)?,
    ))
}

#[cold]
#[inline(never)]
fn bor_slow(left_ref: &Value, right_ref: &Value) -> Result<Value, VmTrap> {
    Ok(Value::I64(
        read_i64_fast(left_ref)? | read_i64_fast(right_ref)?,
    ))
}

#[cold]
#[inline(never)]
fn bxor_slow(left_ref: &Value, right_ref: &Value) -> Result<Value, VmTrap> {
    Ok(Value::I64(
        read_i64_fast(left_ref)? ^ read_i64_fast(right_ref)?,
    ))
}

#[cold]
#[inline(never)]
fn bnot_slow(val_ref: &Value) -> Result<Value, VmTrap> {
    Ok(Value::I64(!read_i64_fast(val_ref)?))
}

#[cold]
#[inline(never)]
fn shl_slow(left_ref: &Value, right_ref: &Value) -> Result<Value, VmTrap> {
    let right_signed = read_i64_fast(right_ref)?;
    if right_signed < 0 {
        return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
            operation: "bitwise shift left",
            reason: "negative shift count is invalid".to_string(),
        }));
    }
    let shift = (right_signed & 63) as u32;
    Ok(Value::I64(read_i64_fast(left_ref)?.wrapping_shl(shift)))
}

#[cold]
#[inline(never)]
fn to_primitive_u64_cold(val: &Value) -> u64 {
    val.to_primitive_u64()
}

#[inline(never)]
fn alloc_object_trap(
    heap: &mut crate::memory::VirtualHeap,
    payload_size: usize,
    tag: BlockTag,
    vtable_id: u32,
) -> Result<u32, VmTrap> {
    let payload_addr = heap
        .allocate(payload_size, tag)
        .map_err(VmTrap::from_error)?;
    write_heap_u32_fast(&mut heap.buffer, payload_addr, vtable_id);
    Ok(payload_addr)
}

#[inline(never)]
fn alloc_array_trap(
    heap: &mut crate::memory::VirtualHeap,
    array_len: usize,
    elem_size: usize,
    type_tag: u8,
    vtable_id: u16,
) -> Result<u32, VmTrap> {
    let payload_size = array_len
        .checked_mul(elem_size)
        .and_then(|bytes| bytes.checked_add(8))
        .filter(|&total_size| total_size <= u32::MAX as usize)
        .ok_or_else(|| VmTrap::out_of_memory(usize::MAX, heap.buffer.len()))?;
    let payload_addr = heap
        .allocate(payload_size, BlockTag::ArrayRef)
        .map_err(VmTrap::from_error)?;
    heap.write_u32(payload_addr, array_len as u32)
        .map_err(VmTrap::from_error)?;
    heap.write_u8(payload_addr + 4, elem_size as u8)
        .map_err(VmTrap::from_error)?;
    heap.write_u16(payload_addr + 5, vtable_id)
        .map_err(VmTrap::from_error)?;
    heap.write_u8(payload_addr + 7, type_tag)
        .map_err(VmTrap::from_error)?;
    Ok(payload_addr)
}

#[inline(never)]
fn array_length_trap(heap: &crate::memory::VirtualHeap, arr_addr: u32) -> Result<u32, VmTrap> {
    let header = heap.read_header(arr_addr).map_err(VmTrap::from_error)?;
    if header.tag == BlockTag::String {
        Ok(header.size)
    } else {
        heap.read_u32(arr_addr).map_err(VmTrap::from_error)
    }
}

#[cold]
#[inline(never)]
fn inc_ref_trap_cold(heap: &mut crate::memory::VirtualHeap, addr: u32) -> Result<(), VmTrap> {
    crate::memory::arc::inc_ref(heap, addr)
        .map(|_| ())
        .map_err(VmTrap::from_error)
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn try_complete_array_fill_loop(
    heap_buf: &mut [u8],
    reg_base: *mut Value,
    set_inst: Instruction,
    idx_reg: usize,
    next_idx: i32,
    bound: i32,
    step: i32,
    is_lte: bool,
) -> Option<u64> {
    if step <= 0 || next_idx < 0 || set_inst.flags() != 0 {
        return None;
    }
    let upper = if is_lte {
        bound as i64
    } else {
        (bound as i64) - 1
    };
    let next_i64 = next_idx as i64;
    if upper < next_i64 {
        return None;
    }
    let step_i64 = step as i64;
    let count_minus_1 = (upper - next_i64) / step_i64;
    let last_idx = next_i64 + count_minus_1 * step_i64;
    let exit_idx = last_idx + step_i64;
    if exit_idx > i32::MAX as i64 {
        return None;
    }

    // SAFETY: Caller verified `set_inst` registers are valid within the active frame.
    let arr_val = unsafe { *reg_base.add(set_inst.dst() as usize) };
    let Value::Ref(arr_addr) = arr_val else {
        return None;
    };
    if arr_addr == 0 {
        return None;
    }
    let arr_offset = arr_addr as usize;
    if arr_offset < 8 || arr_offset + 8 > heap_buf.len() {
        return None;
    }
    let tag_nibble = (heap_buf[arr_offset - 1] >> 4) & 0x0F;
    if tag_nibble != BlockTag::ArrayRef as u8 {
        return None;
    }
    let header_qword =
        u64::from_le(unsafe { std::ptr::read_unaligned(heap_buf.as_ptr().add(arr_offset).cast()) });
    let array_len = (header_qword as u32) as i64;
    if last_idx >= array_len {
        return None;
    }
    let meta_masked = ((header_qword >> 32) as u32) & 0xFF00_00FF;
    let fill_val = unsafe { *reg_base.add(set_inst.src2() as usize) };
    let next_usize = next_idx as usize;
    let last_usize = last_idx as usize;
    let step_usize = step as usize;
    let data_base = arr_offset + 8;

    const META_BOOL_1B: u32 = (11 << 24) | 1;
    const META_I32_4B: u32 = (3 << 24) | 4;
    const META_F64_8B: u32 = (10 << 24) | 8;

    match meta_masked {
        META_BOOL_1B => {
            let end_offset = data_base + last_usize + 1;
            if end_offset > heap_buf.len() {
                return None;
            }
            let byte_val = match fill_val {
                Value::Bool(flag) => u8::from(flag),
                Value::I32(num_val) => num_val as u8,
                _ => return None,
            };
            let slice = &mut heap_buf[data_base..end_offset];
            if step_usize == 1 {
                slice[next_usize..=last_usize].fill(byte_val);
            } else {
                let mut cursor = next_usize;
                while cursor <= last_usize {
                    slice[cursor] = byte_val;
                    cursor += step_usize;
                }
            }
        }
        META_I32_4B => {
            let end_offset = data_base + ((last_usize + 1) << 2);
            if end_offset > heap_buf.len() {
                return None;
            }
            let word_bytes = match fill_val {
                Value::I32(num_val) => (num_val as u32).to_le_bytes(),
                _ => return None,
            };
            let slice = &mut heap_buf[data_base..end_offset];
            let mut cursor = next_usize;
            while cursor <= last_usize {
                let byte_idx = cursor << 2;
                slice[byte_idx..byte_idx + 4].copy_from_slice(&word_bytes);
                cursor += step_usize;
            }
        }
        META_F64_8B => {
            let end_offset = data_base + ((last_usize + 1) << 3);
            if end_offset > heap_buf.len() {
                return None;
            }
            let float_bytes = match fill_val {
                Value::F64(float_val) => float_val.to_bits().to_le_bytes(),
                _ => return None,
            };
            let slice = &mut heap_buf[data_base..end_offset];
            let mut cursor = next_usize;
            while cursor <= last_usize {
                let byte_idx = cursor << 3;
                slice[byte_idx..byte_idx + 8].copy_from_slice(&float_bytes);
                cursor += step_usize;
            }
        }
        _ => return None,
    }

    // SAFETY: Caller verified `idx_reg` lies within active frame register window.
    unsafe {
        *reg_base.add(idx_reg) = Value::I32(exit_idx as i32);
    }
    Some(((count_minus_1 + 1) as u64) * 2)
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn try_complete_special_for_loop<const TRACK_METRICS: bool>(
    heap_buffer: &mut [u8],
    reg_base: *mut Value,
    inst_ptr: *const Instruction,
    pc: usize,
    inst: Instruction,
    next_idx: i32,
    bound: i32,
) -> Result<Option<u64>, VmTrap> {
    let flags = inst.flags();
    if (flags & 0x20) != 0 && pc >= 2 {
        let set_inst = unsafe { *inst_ptr.add(pc - 2) };
        if set_inst.raw_opcode() == OpCode::SetArray as u8 {
            let step = if (flags & 0x80) == 0 {
                1
            } else {
                read_i32_fast(unsafe { &*reg_base.add((flags & 0x1F) as usize) })?
            };
            if let Some(done_steps) = try_complete_array_fill_loop(
                heap_buffer,
                reg_base,
                set_inst,
                inst.dst() as usize,
                next_idx,
                bound,
                step,
                (flags & 0x40) != 0,
            ) {
                return Ok(Some(done_steps));
            }
        }
        return Ok(None);
    }

    if (flags & 0x90) == 0x10 {
        let offset = inst.src2() as i16 as i32;
        let loop_end_pc = pc - 1;
        let loop_start_pc = ((loop_end_pc as i32) + offset) as usize;
        let done_steps = try_complete_scalar_for_loop::<TRACK_METRICS>(
            reg_base,
            inst_ptr,
            loop_start_pc,
            loop_end_pc,
            inst.dst() as usize,
            inst.src1() as usize,
            next_idx,
            bound,
            (flags & 0x40) != 0,
        )?;
        return Ok(Some(done_steps));
    }

    Ok(None)
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn try_complete_scalar_for_loop<const TRACK_METRICS: bool>(
    reg_base: *mut Value,
    inst_ptr: *const Instruction,
    start_pc: usize,
    loop_end_pc: usize,
    idx_reg: usize,
    bound_reg: usize,
    initial_next_idx: i32,
    bound: i32,
    is_lte: bool,
) -> Result<u64, VmTrap> {
    let mut is_pure_i32 = idx_reg < 32 && bound_reg < 32;
    let mut used_mask: u32 = if is_pure_i32 {
        (1u32 << idx_reg) | (1u32 << bound_reg)
    } else {
        0
    };
    let mut written_mask: u32 = if is_pure_i32 { 1u32 << idx_reg } else { 0 };
    let mut defined_mask: u32 = used_mask;
    let mut live_in_mask: u32 = 0u32;
    let mut straight_line = true;

    const OP_MOV: u8 = OpCode::Mov as u8;
    const OP_LOAD_I32: u8 = OpCode::LoadI32 as u8;
    const OP_ADD_I32: u8 = OpCode::AddI32 as u8;
    const OP_SUB_I32: u8 = OpCode::SubI32 as u8;
    const OP_MUL_I32: u8 = OpCode::MulI32 as u8;
    const OP_NEG_I32: u8 = OpCode::NegI32 as u8;
    const OP_BAND: u8 = OpCode::Band as u8;
    const OP_BOR: u8 = OpCode::Bor as u8;
    const OP_BXOR: u8 = OpCode::Bxor as u8;
    const OP_BNOT: u8 = OpCode::Bnot as u8;
    const OP_SHL: u8 = OpCode::Shl as u8;
    const OP_SHR: u8 = OpCode::Shr as u8;
    const OP_ADD_I32_IMM: u8 = OpCode::AddI32Imm as u8;
    const OP_SUB_I32_IMM: u8 = OpCode::SubI32Imm as u8;
    const OP_MUL_I32_IMM: u8 = OpCode::MulI32Imm as u8;
    const OP_MOD_I32_IMM: u8 = OpCode::ModI32Imm as u8;
    const OP_ADD_BAND_I32: u8 = OpCode::AddBandI32 as u8;
    const OP_JUMP: u8 = OpCode::Jump as u8;
    const OP_JUMP_I32_IMM: u8 = OpCode::JumpIfI32Imm as u8;
    const OP_JUMP_LT_I32: u8 = OpCode::JumpIfLtI32 as u8;
    const OP_JUMP_LTE_I32: u8 = OpCode::JumpIfLteI32 as u8;
    const OP_JUMP_GT_I32: u8 = OpCode::JumpIfGtI32 as u8;
    const OP_JUMP_GTE_I32: u8 = OpCode::JumpIfGteI32 as u8;
    const OP_JUMP_EQ_I32: u8 = OpCode::JumpIfEqI32 as u8;
    const OP_JUMP_NEQ_I32: u8 = OpCode::JumpIfNeqI32 as u8;
    const OP_ADD_F64: u8 = OpCode::AddF64 as u8;
    const OP_SUB_F64: u8 = OpCode::SubF64 as u8;
    const OP_MUL_F64: u8 = OpCode::MulF64 as u8;
    const OP_DIV_F64: u8 = OpCode::DivF64 as u8;
    const OP_NEG_F64: u8 = OpCode::NegF64 as u8;
    const OP_MUL_ADD_F64: u8 = OpCode::MulAddF64 as u8;

    if is_pure_i32 {
        for check_pc in start_pc..loop_end_pc {
            // SAFETY: `check_pc < loop_end_pc` was verified by `context::mark_scalar_for_loops`.
            let inst = unsafe { *inst_ptr.add(check_pc) };
            let raw = inst.raw_opcode();
            let mark_read =
                |reg: u16, used: &mut u32, defined: u32, live_in: &mut u32, ok: &mut bool| {
                    if reg >= 32 {
                        *ok = false;
                        return;
                    }
                    let bit = 1u32 << reg;
                    *used |= bit;
                    if (defined & bit) == 0 {
                        *live_in |= bit;
                    }
                };

            match raw {
                OP_MOV | OP_NEG_I32 | OP_BNOT | OP_ADD_I32_IMM | OP_SUB_I32_IMM
                | OP_MUL_I32_IMM | OP_MOD_I32_IMM => {
                    mark_read(
                        inst.src1(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    let dst = inst.dst();
                    if dst >= 32 {
                        is_pure_i32 = false;
                    } else {
                        let bit = 1u32 << dst;
                        used_mask |= bit;
                        written_mask |= bit;
                        if straight_line {
                            defined_mask |= bit;
                        }
                    }
                }
                OP_LOAD_I32 => {
                    let dst = inst.dst();
                    if dst >= 32 {
                        is_pure_i32 = false;
                    } else {
                        let bit = 1u32 << dst;
                        used_mask |= bit;
                        written_mask |= bit;
                        if straight_line {
                            defined_mask |= bit;
                        }
                    }
                }
                OP_ADD_I32 | OP_SUB_I32 | OP_MUL_I32 | OP_BAND | OP_BOR | OP_BXOR | OP_SHL
                | OP_SHR => {
                    mark_read(
                        inst.src1(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    mark_read(
                        inst.src2(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    let dst = inst.dst();
                    if dst >= 32 {
                        is_pure_i32 = false;
                    } else {
                        let bit = 1u32 << dst;
                        used_mask |= bit;
                        written_mask |= bit;
                        if straight_line {
                            defined_mask |= bit;
                        }
                    }
                }
                OP_ADD_BAND_I32 => {
                    mark_read(
                        inst.flags() as u16,
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    mark_read(
                        inst.src1(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    mark_read(
                        inst.src2(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    let dst = inst.dst();
                    if dst >= 32 {
                        is_pure_i32 = false;
                    } else {
                        let bit = 1u32 << dst;
                        used_mask |= bit;
                        written_mask |= bit;
                        if straight_line {
                            defined_mask |= bit;
                        }
                    }
                }
                OP_JUMP => {
                    straight_line = false;
                }
                OP_JUMP_I32_IMM => {
                    straight_line = false;
                    mark_read(
                        inst.dst(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                }
                OP_JUMP_LT_I32..=OP_JUMP_NEQ_I32 => {
                    straight_line = false;
                    mark_read(
                        inst.dst(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                    mark_read(
                        inst.src1(),
                        &mut used_mask,
                        defined_mask,
                        &mut live_in_mask,
                        &mut is_pure_i32,
                    );
                }
                _ => {
                    is_pure_i32 = false;
                    break;
                }
            }
        }
    }

    if is_pure_i32 {
        let mut regs = [0i32; 32];
        let mut all_i32 = true;
        for (reg_idx, slot) in regs.iter_mut().enumerate() {
            let bit = 1u32 << reg_idx;
            if (used_mask & bit) != 0 {
                // SAFETY: `reg_idx` is verified within frame window by Tier 1 verifier.
                match unsafe { *reg_base.add(reg_idx) } {
                    Value::I32(val) => *slot = val,
                    _ if (live_in_mask & bit) == 0 => *slot = 0,
                    _ => {
                        all_i32 = false;
                        break;
                    }
                }
            }
        }
        if all_i32 {
            regs[idx_reg] = initial_next_idx;
            let mut steps: u64 = 0;
            let body_len = loop_end_pc.saturating_sub(start_pc);

            if straight_line && body_len > 0 && body_len <= 16 {
                #[derive(Clone, Copy)]
                enum ScalarFastOp {
                    ModPow2 {
                        dst: u8,
                        src1: u8,
                        mask: i32,
                    },
                    ModMagic {
                        dst: u8,
                        src1: u8,
                        imm: i32,
                        magic: u64,
                        shift: u32,
                    },
                    ModSlow {
                        dst: u8,
                        src1: u8,
                        imm: i32,
                    },
                    Bxor {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    AddBand {
                        dst: u8,
                        acc: u8,
                        src1: u8,
                        mask_reg: u8,
                    },
                    Add {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    Sub {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    Mul {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    Band {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    Bor {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    Shl {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    Shr {
                        dst: u8,
                        src1: u8,
                        src2: u8,
                    },
                    AddImm {
                        dst: u8,
                        src1: u8,
                        imm: i32,
                    },
                    SubImm {
                        dst: u8,
                        src1: u8,
                        imm: i32,
                    },
                    MulImm {
                        dst: u8,
                        src1: u8,
                        imm: i32,
                    },
                    Neg {
                        dst: u8,
                        src1: u8,
                    },
                    Bnot {
                        dst: u8,
                        src1: u8,
                    },
                    Mov {
                        dst: u8,
                        src1: u8,
                    },
                    LoadI32 {
                        dst: u8,
                        imm: i32,
                    },
                }

                let mut decoded = [ScalarFastOp::Mov { dst: 0, src1: 0 }; 16];
                for (slot_idx, slot) in decoded[..body_len].iter_mut().enumerate() {
                    let inst = unsafe { *inst_ptr.add(start_pc + slot_idx) };
                    let dst = inst.dst() as u8;
                    let src1 = inst.src1() as u8;
                    let src2 = inst.src2() as u8;
                    *slot = match inst.raw_opcode() {
                        OP_MOD_I32_IMM => {
                            let imm_val = inst.src2() as i16 as i32;
                            if imm_val > 0 && imm_val < 1024 {
                                if (imm_val & (imm_val - 1)) == 0 {
                                    ScalarFastOp::ModPow2 {
                                        dst,
                                        src1,
                                        mask: imm_val - 1,
                                    }
                                } else {
                                    let (magic, shift) = MOD_MAGIC_TABLE[imm_val as usize];
                                    ScalarFastOp::ModMagic {
                                        dst,
                                        src1,
                                        imm: imm_val,
                                        magic,
                                        shift,
                                    }
                                }
                            } else {
                                ScalarFastOp::ModSlow {
                                    dst,
                                    src1,
                                    imm: imm_val,
                                }
                            }
                        }
                        OP_BXOR => ScalarFastOp::Bxor { dst, src1, src2 },
                        OP_ADD_BAND_I32 => ScalarFastOp::AddBand {
                            dst,
                            acc: inst.flags(),
                            src1,
                            mask_reg: src2,
                        },
                        OP_ADD_I32 => ScalarFastOp::Add { dst, src1, src2 },
                        OP_SUB_I32 => ScalarFastOp::Sub { dst, src1, src2 },
                        OP_MUL_I32 => ScalarFastOp::Mul { dst, src1, src2 },
                        OP_BAND => ScalarFastOp::Band { dst, src1, src2 },
                        OP_BOR => ScalarFastOp::Bor { dst, src1, src2 },
                        OP_SHL => ScalarFastOp::Shl { dst, src1, src2 },
                        OP_SHR => ScalarFastOp::Shr { dst, src1, src2 },
                        OP_ADD_I32_IMM => ScalarFastOp::AddImm {
                            dst,
                            src1,
                            imm: inst.src2() as i16 as i32,
                        },
                        OP_SUB_I32_IMM => ScalarFastOp::SubImm {
                            dst,
                            src1,
                            imm: inst.src2() as i16 as i32,
                        },
                        OP_MUL_I32_IMM => ScalarFastOp::MulImm {
                            dst,
                            src1,
                            imm: inst.src2() as i16 as i32,
                        },
                        OP_NEG_I32 => ScalarFastOp::Neg { dst, src1 },
                        OP_BNOT => ScalarFastOp::Bnot { dst, src1 },
                        OP_MOV => ScalarFastOp::Mov { dst, src1 },
                        _ => ScalarFastOp::LoadI32 {
                            dst,
                            imm: inst.imm32_signed(),
                        },
                    };
                }

                let decoded_slice = &decoded[..body_len];
                loop {
                    for &scalar_op in decoded_slice {
                        match scalar_op {
                            ScalarFastOp::ModPow2 { dst, src1, mask } => {
                                let val_a = regs[(src1 as usize) & 31];
                                let rem_val = if val_a >= 0 {
                                    val_a & mask
                                } else {
                                    rem_i32_cold(val_a, mask + 1)?
                                };
                                regs[(dst as usize) & 31] = rem_val;
                            }
                            ScalarFastOp::ModMagic {
                                dst,
                                src1,
                                imm,
                                magic,
                                shift,
                            } => {
                                let val_a = regs[(src1 as usize) & 31];
                                let rem_val = if val_a >= 0 {
                                    let quotient =
                                        (((val_a as u64).wrapping_mul(magic)) >> shift) as i32;
                                    val_a - quotient.wrapping_mul(imm)
                                } else {
                                    rem_i32_cold(val_a, imm)?
                                };
                                regs[(dst as usize) & 31] = rem_val;
                            }
                            ScalarFastOp::ModSlow { dst, src1, imm } => {
                                regs[(dst as usize) & 31] =
                                    rem_i32_cold(regs[(src1 as usize) & 31], imm)?;
                            }
                            ScalarFastOp::Bxor { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31] ^ regs[(src2 as usize) & 31];
                            }
                            ScalarFastOp::AddBand {
                                dst,
                                acc,
                                src1,
                                mask_reg,
                            } => {
                                regs[(dst as usize) & 31] = regs[(acc as usize) & 31]
                                    .wrapping_add(regs[(src1 as usize) & 31])
                                    & regs[(mask_reg as usize) & 31];
                            }
                            ScalarFastOp::Add { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] = regs[(src1 as usize) & 31]
                                    .wrapping_add(regs[(src2 as usize) & 31]);
                            }
                            ScalarFastOp::Sub { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] = regs[(src1 as usize) & 31]
                                    .wrapping_sub(regs[(src2 as usize) & 31]);
                            }
                            ScalarFastOp::Mul { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] = regs[(src1 as usize) & 31]
                                    .wrapping_mul(regs[(src2 as usize) & 31]);
                            }
                            ScalarFastOp::Band { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31] & regs[(src2 as usize) & 31];
                            }
                            ScalarFastOp::Bor { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31] | regs[(src2 as usize) & 31];
                            }
                            ScalarFastOp::Shl { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] = regs[(src1 as usize) & 31]
                                    .wrapping_shl((regs[(src2 as usize) & 31] & 31) as u32);
                            }
                            ScalarFastOp::Shr { dst, src1, src2 } => {
                                regs[(dst as usize) & 31] = regs[(src1 as usize) & 31]
                                    .wrapping_shr((regs[(src2 as usize) & 31] & 31) as u32);
                            }
                            ScalarFastOp::AddImm { dst, src1, imm } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31].wrapping_add(imm);
                            }
                            ScalarFastOp::SubImm { dst, src1, imm } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31].wrapping_sub(imm);
                            }
                            ScalarFastOp::MulImm { dst, src1, imm } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31].wrapping_mul(imm);
                            }
                            ScalarFastOp::Neg { dst, src1 } => {
                                regs[(dst as usize) & 31] =
                                    regs[(src1 as usize) & 31].wrapping_neg();
                            }
                            ScalarFastOp::Bnot { dst, src1 } => {
                                regs[(dst as usize) & 31] = !regs[(src1 as usize) & 31];
                            }
                            ScalarFastOp::Mov { dst, src1 } => {
                                regs[(dst as usize) & 31] = regs[(src1 as usize) & 31];
                            }
                            ScalarFastOp::LoadI32 { dst, imm } => {
                                regs[(dst as usize) & 31] = imm;
                            }
                        }
                    }

                    if TRACK_METRICS {
                        steps += (body_len as u64) + 1;
                    }
                    let cur_idx = regs[idx_reg];
                    let next_idx = cur_idx.wrapping_add(1);
                    regs[idx_reg] = next_idx;
                    let keep_going = if is_lte {
                        cur_idx < bound
                    } else {
                        cur_idx
                            .checked_add(1)
                            .is_some_and(|next_val| next_val < bound)
                    };
                    if !keep_going {
                        break;
                    }
                }
            } else {
                loop {
                    let mut local_pc = start_pc;
                    while local_pc < loop_end_pc {
                        // SAFETY: `local_pc < loop_end_pc` verified in `context::mark_scalar_for_loops`.
                        let inst = unsafe { *inst_ptr.add(local_pc) };
                        local_pc += 1;
                        if TRACK_METRICS {
                            steps += 1;
                        }
                        match inst.raw_opcode() {
                            OP_MOD_I32_IMM => {
                                let imm_val = inst.src2() as i16 as i32;
                                let val_a = regs[inst.src1() as usize];
                                let rem_val = if val_a >= 0 && imm_val > 0 && imm_val < 1024 {
                                    if (imm_val & (imm_val - 1)) == 0 {
                                        val_a & (imm_val - 1)
                                    } else {
                                        let (magic, shift) = MOD_MAGIC_TABLE[imm_val as usize];
                                        let quotient =
                                            (((val_a as u64).wrapping_mul(magic)) >> shift) as i32;
                                        val_a - quotient.wrapping_mul(imm_val)
                                    }
                                } else {
                                    rem_i32_cold(val_a, imm_val)?
                                };
                                regs[inst.dst() as usize] = rem_val;
                            }
                            OP_BXOR => {
                                regs[inst.dst() as usize] =
                                    regs[inst.src1() as usize] ^ regs[inst.src2() as usize];
                            }
                            OP_ADD_BAND_I32 => {
                                regs[inst.dst() as usize] = regs[inst.flags() as usize]
                                    .wrapping_add(regs[inst.src1() as usize])
                                    & regs[inst.src2() as usize];
                            }
                            OP_ADD_I32 => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_add(regs[inst.src2() as usize]);
                            }
                            OP_SUB_I32 => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_sub(regs[inst.src2() as usize]);
                            }
                            OP_MUL_I32 => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_mul(regs[inst.src2() as usize]);
                            }
                            OP_ADD_I32_IMM => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_add(inst.src2() as i16 as i32);
                            }
                            OP_SUB_I32_IMM => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_sub(inst.src2() as i16 as i32);
                            }
                            OP_MUL_I32_IMM => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_mul(inst.src2() as i16 as i32);
                            }
                            OP_MOV => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize];
                            }
                            OP_LOAD_I32 => {
                                regs[inst.dst() as usize] = inst.imm32_signed();
                            }
                            OP_BAND => {
                                regs[inst.dst() as usize] =
                                    regs[inst.src1() as usize] & regs[inst.src2() as usize];
                            }
                            OP_BOR => {
                                regs[inst.dst() as usize] =
                                    regs[inst.src1() as usize] | regs[inst.src2() as usize];
                            }
                            OP_SHL => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_shl((regs[inst.src2() as usize] & 31) as u32);
                            }
                            OP_SHR => {
                                regs[inst.dst() as usize] = regs[inst.src1() as usize]
                                    .wrapping_shr((regs[inst.src2() as usize] & 31) as u32);
                            }
                            OP_NEG_I32 => {
                                regs[inst.dst() as usize] =
                                    regs[inst.src1() as usize].wrapping_neg();
                            }
                            OP_BNOT => {
                                regs[inst.dst() as usize] = !regs[inst.src1() as usize];
                            }
                            OP_JUMP => {
                                local_pc = ((local_pc as i32 - 1) + inst.jump_offset()) as usize;
                            }
                            OP_JUMP_I32_IMM => {
                                let val_a = regs[inst.dst() as usize];
                                let imm_val = inst.src1() as i16 as i32;
                                let take = match inst.flags() {
                                    0 => val_a == imm_val,
                                    1 => val_a != imm_val,
                                    2 => val_a < imm_val,
                                    3 => val_a <= imm_val,
                                    4 => val_a > imm_val,
                                    _ => val_a >= imm_val,
                                };
                                if take {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                            OP_JUMP_LT_I32 => {
                                if regs[inst.dst() as usize] < regs[inst.src1() as usize] {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                            OP_JUMP_LTE_I32 => {
                                if regs[inst.dst() as usize] <= regs[inst.src1() as usize] {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                            OP_JUMP_GT_I32 => {
                                if regs[inst.dst() as usize] > regs[inst.src1() as usize] {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                            OP_JUMP_GTE_I32 => {
                                if regs[inst.dst() as usize] >= regs[inst.src1() as usize] {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                            OP_JUMP_EQ_I32 => {
                                if regs[inst.dst() as usize] == regs[inst.src1() as usize] {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                            _ => {
                                if regs[inst.dst() as usize] != regs[inst.src1() as usize] {
                                    local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32))
                                        as usize;
                                }
                            }
                        }
                    }

                    let cur_idx = regs[idx_reg];
                    let next_idx = cur_idx.wrapping_add(1);
                    regs[idx_reg] = next_idx;
                    if TRACK_METRICS {
                        steps += 1;
                    }
                    let keep_going = if is_lte {
                        cur_idx < bound
                    } else {
                        cur_idx
                            .checked_add(1)
                            .is_some_and(|next_val| next_val < bound)
                    };
                    if !keep_going {
                        break;
                    }
                }
            }

            for (reg_idx, &val) in regs.iter().enumerate() {
                if (written_mask & (1u32 << reg_idx)) != 0 {
                    // SAFETY: `reg_idx < 32` written inside frame register window.
                    unsafe {
                        *reg_base.add(reg_idx) = Value::I32(val);
                    }
                }
            }
            return Ok(steps);
        }
    }

    // General tagged scalar loop fallback (handles f64 loops like 03_nbody_float64 and frames >= 32 regs).
    let mut cur_idx = initial_next_idx;
    let mut steps: u64 = 0;

    loop {
        let mut local_pc = start_pc;
        while local_pc < loop_end_pc {
            // SAFETY: `local_pc < loop_end_pc` was verified by `context::mark_scalar_for_loops`.
            let inst = unsafe { *inst_ptr.add(local_pc) };
            local_pc += 1;
            if TRACK_METRICS {
                steps += 1;
            }
            match inst.raw_opcode() {
                OP_MUL_ADD_F64 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let acc_ref = unsafe { &*reg_base.add(inst.flags() as usize) };
                    if let (Value::F64(lhs), Value::F64(rhs), Value::F64(acc)) =
                        (*left_ref, *right_ref, *acc_ref)
                    {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(lhs * rhs + acc);
                        }
                    } else {
                        return Err(mul_add_f64_type_mismatch_cold(left_ref, right_ref, acc_ref));
                    }
                }
                OP_ADD_F64 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    if let (Value::F64(lhs), Value::F64(rhs)) = (*left_ref, *right_ref) {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(lhs + rhs);
                        }
                    } else {
                        return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                    }
                }
                OP_SUB_F64 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    if let (Value::F64(lhs), Value::F64(rhs)) = (*left_ref, *right_ref) {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(lhs - rhs);
                        }
                    } else {
                        return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                    }
                }
                OP_MUL_F64 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    if let (Value::F64(lhs), Value::F64(rhs)) = (*left_ref, *right_ref) {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(lhs * rhs);
                        }
                    } else {
                        return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                    }
                }
                OP_DIV_F64 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    if let (Value::F64(lhs), Value::F64(rhs)) = (*left_ref, *right_ref) {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(lhs / rhs);
                        }
                    } else {
                        return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                    }
                }
                OP_NEG_F64 => {
                    let val_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    if let Value::F64(float_val) = *val_ref {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(-float_val);
                        }
                    } else {
                        return Err(f64_type_mismatch_error(val_ref));
                    }
                }
                OP_MOD_I32_IMM => {
                    let imm_val = inst.src2() as i16 as i32;
                    let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                    let rem_val = if val_a >= 0 && imm_val > 0 && imm_val < 1024 {
                        if (imm_val & (imm_val - 1)) == 0 {
                            val_a & (imm_val - 1)
                        } else {
                            let (magic, shift) = MOD_MAGIC_TABLE[imm_val as usize];
                            let quotient = (((val_a as u64).wrapping_mul(magic)) >> shift) as i32;
                            val_a - quotient.wrapping_mul(imm_val)
                        }
                    } else {
                        rem_i32_cold(val_a, imm_val)?
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(rem_val);
                    }
                }
                OP_ADD_BAND_I32 => {
                    let acc_ref = unsafe { &*reg_base.add(inst.flags() as usize) };
                    let val_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let mask_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let res = if let (Value::I32(acc), Value::I32(val), Value::I32(mask)) =
                        (*acc_ref, *val_ref, *mask_ref)
                    {
                        acc.wrapping_add(val) & mask
                    } else {
                        read_i32_fast(acc_ref)?.wrapping_add(read_i32_fast(val_ref)?)
                            & read_i32_fast(mask_ref)?
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(res);
                    }
                }
                OP_ADD_I32_IMM => {
                    let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) =
                            Value::I32(val_a.wrapping_add(inst.src2() as i16 as i32));
                    }
                }
                OP_SUB_I32_IMM => {
                    let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) =
                            Value::I32(val_a.wrapping_sub(inst.src2() as i16 as i32));
                    }
                }
                OP_MUL_I32_IMM => {
                    let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) =
                            Value::I32(val_a.wrapping_mul(inst.src2() as i16 as i32));
                    }
                }
                OP_ADD_I32 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let (lhs, rhs) =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            (val_a, val_b)
                        } else {
                            read_i32_pair_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(lhs.wrapping_add(rhs));
                    }
                }
                OP_SUB_I32 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let (lhs, rhs) =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            (val_a, val_b)
                        } else {
                            read_i32_pair_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(lhs.wrapping_sub(rhs));
                    }
                }
                OP_MUL_I32 => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let (lhs, rhs) =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            (val_a, val_b)
                        } else {
                            read_i32_pair_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(lhs.wrapping_mul(rhs));
                    }
                }
                OP_BAND => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let res =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            Value::I32(val_a & val_b)
                        } else {
                            band_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = res;
                    }
                }
                OP_BOR => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let res =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            Value::I32(val_a | val_b)
                        } else {
                            bor_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = res;
                    }
                }
                OP_BXOR => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let res =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            Value::I32(val_a ^ val_b)
                        } else {
                            bxor_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = res;
                    }
                }
                OP_SHL => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let res =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            if val_b < 0 {
                                return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
                                    operation: "bitwise shift left",
                                    reason: "negative shift count is invalid".to_string(),
                                }));
                            }
                            Value::I32(val_a.wrapping_shl((val_b & 31) as u32))
                        } else {
                            shl_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = res;
                    }
                }
                OP_SHR => {
                    let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                    let res =
                        if let (Value::I32(val_a), Value::I32(val_b)) = (*left_ref, *right_ref) {
                            if val_b < 0 {
                                return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
                                    operation: "bitwise shift right",
                                    reason: "negative shift count is invalid".to_string(),
                                }));
                            }
                            Value::I32(val_a.wrapping_shr((val_b & 31) as u32))
                        } else {
                            shr_slow(left_ref, right_ref)?
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = res;
                    }
                }
                OP_NEG_I32 => {
                    let val = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(val.wrapping_neg());
                    }
                }
                OP_BNOT => {
                    let val_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let res = if let Value::I32(int_val) = *val_ref {
                        Value::I32(!int_val)
                    } else {
                        bnot_slow(val_ref)?
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = res;
                    }
                }
                OP_MOV => unsafe {
                    let moved_val = *reg_base.add(inst.src1() as usize);
                    *reg_base.add(inst.dst() as usize) = moved_val;
                },
                OP_LOAD_I32 => unsafe {
                    *reg_base.add(inst.dst() as usize) = Value::I32(inst.imm32_signed());
                },
                OP_JUMP => {
                    local_pc = ((local_pc as i32 - 1) + inst.jump_offset()) as usize;
                }
                OP_JUMP_I32_IMM => {
                    let imm_val = inst.src1() as i16 as i32;
                    let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.dst() as usize) })?;
                    let take = match inst.flags() {
                        0 => val_a == imm_val,
                        1 => val_a != imm_val,
                        2 => val_a < imm_val,
                        3 => val_a <= imm_val,
                        4 => val_a > imm_val,
                        _ => val_a >= imm_val,
                    };
                    if take {
                        local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32)) as usize;
                    }
                }
                raw => {
                    let first_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                    let second_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                    let (val_a, val_b) =
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*first_ref, *second_ref) {
                            (lhs, rhs)
                        } else if raw == OP_JUMP_EQ_I32 || raw == OP_JUMP_NEQ_I32 {
                            let eq = values_equal_slow(first_ref, second_ref);
                            let take = if raw == OP_JUMP_EQ_I32 { eq } else { !eq };
                            if take {
                                local_pc =
                                    ((local_pc as i32 - 1) + (inst.src2() as i16 as i32)) as usize;
                            }
                            continue;
                        } else {
                            read_i32_pair_slow(first_ref, second_ref)?
                        };
                    let take = match raw {
                        OP_JUMP_LT_I32 => val_a < val_b,
                        OP_JUMP_LTE_I32 => val_a <= val_b,
                        OP_JUMP_GT_I32 => val_a > val_b,
                        OP_JUMP_GTE_I32 => val_a >= val_b,
                        OP_JUMP_EQ_I32 => val_a == val_b,
                        _ => val_a != val_b,
                    };
                    if take {
                        local_pc = ((local_pc as i32 - 1) + (inst.src2() as i16 as i32)) as usize;
                    }
                }
            }
        }

        let prev_idx = cur_idx;
        cur_idx = prev_idx.wrapping_add(1);
        unsafe {
            *reg_base.add(idx_reg) = Value::I32(cur_idx);
        }
        if TRACK_METRICS {
            steps += 1;
        }
        let keep_going = if is_lte {
            prev_idx < bound
        } else {
            prev_idx
                .checked_add(1)
                .is_some_and(|next_val| next_val < bound)
        };
        if !keep_going {
            break;
        }
    }

    Ok(steps)
}

#[inline(always)]
fn inc_ref_inline(
    heap: &mut crate::memory::VirtualHeap,
    addr: u32,
    cycle_collector_enabled: bool,
) -> Result<(), VmTrap> {
    let addr_usize = addr as usize;
    if addr_usize < 8 || addr_usize > heap.buffer.len() {
        return Err(VmTrap::null_reference());
    }
    // SAFETY: addr_usize >= 8 && addr_usize <= heap.buffer.len() verified above.
    let word_ptr = unsafe { heap.buffer.as_mut_ptr().add(addr_usize - 4).cast::<u32>() };
    let tag_and_ref = unsafe { u32::from_le(std::ptr::read_unaligned(word_ptr)) };
    let ref_count = tag_and_ref & crate::memory::layout::MAX_REF_COUNT;
    if ref_count == crate::memory::layout::MAX_REF_COUNT {
        return Ok(());
    }
    if ref_count == 0 || ref_count + 1 >= crate::memory::layout::MAX_REF_COUNT {
        return inc_ref_trap_cold(heap, addr);
    }
    let updated = if cycle_collector_enabled {
        // Clear bits 26..28 (GcColor::Black) and increment ref_count in place.
        (tag_and_ref & 0xF000_0000) | (ref_count + 1)
    } else {
        (tag_and_ref & !crate::memory::layout::MAX_REF_COUNT) | (ref_count + 1)
    };
    unsafe {
        std::ptr::write_unaligned(word_ptr, updated.to_le());
    }
    Ok(())
}

#[inline(never)]
fn get_array_variant_cold(
    heap: &crate::memory::VirtualHeap,
    arr_offset: usize,
    index_usize: usize,
    elem_size_raw: u8,
    elem_type_tag: u8,
) -> Result<Value, VmTrap> {
    let elem_size = (elem_size_raw as usize).max(1);
    let elem_offset = arr_offset + 8 + (index_usize * elem_size);
    if elem_offset + elem_size > heap.buffer.len() {
        return Err(VmTrap::null_reference());
    }
    // SAFETY: elem_offset + elem_size <= heap.buffer.len() verified above.
    let elem_ptr = unsafe { heap.buffer.as_ptr().add(elem_offset) };
    let raw_word = match elem_size {
        1 => unsafe { *elem_ptr as u64 },
        2 => unsafe { u16::from_le(std::ptr::read_unaligned(elem_ptr.cast::<u16>())) as u64 },
        4 => unsafe { u32::from_le(std::ptr::read_unaligned(elem_ptr.cast::<u32>())) as u64 },
        _ => unsafe { u64::from_le(std::ptr::read_unaligned(elem_ptr.cast::<u64>())) },
    };
    Ok(decode_array_element(elem_type_tag, elem_size, raw_word))
}

#[inline(never)]
fn decode_array_element_slow(elem_type_tag: u8, elem_size: usize, raw_word: u64) -> Value {
    match elem_type_tag {
        1 => Value::I32(raw_word as i8 as i32),
        2 => Value::I32(raw_word as i16 as i32),
        4 | 8 => Value::I64(raw_word as i64),
        5 | 12 => Value::I32(raw_word as u8 as i32),
        6 => Value::I32(raw_word as u16 as i32),
        7 => Value::I32(raw_word as u32 as i32),
        9 => {
            if elem_size == 4 || (raw_word >> 32) == 0 {
                Value::F64(f32::from_bits(raw_word as u32) as f64)
            } else {
                Value::F64(f64::from_bits(raw_word))
            }
        }
        _ => Value::from_raw_u64(raw_word),
    }
}

#[inline(always)]
fn decode_array_element(elem_type_tag: u8, elem_size: usize, raw_word: u64) -> Value {
    match elem_type_tag {
        11 => Value::Bool(raw_word != 0),
        10 => Value::F64(f64::from_bits(raw_word)),
        3 => Value::I32(raw_word as i32),
        0 => Value::from_raw_u64(raw_word),
        13 | 0x80..=0xFF => {
            if raw_word == 0 {
                Value::Null
            } else {
                Value::Ref(raw_word as u32)
            }
        }
        _ => decode_array_element_slow(elem_type_tag, elem_size, raw_word),
    }
}

const fn build_mod_magic_table() -> [(u64, u32); 1024] {
    let mut table = [(0u64, 0u32); 1024];
    let mut divisor = 1usize;
    while divisor < 1024 {
        let shift = 33 + divisor.ilog2();
        let numerator = 1u64 << shift;
        let div_u64 = divisor as u64;
        let magic = if numerator.is_multiple_of(div_u64) {
            numerator / div_u64
        } else {
            (numerator / div_u64) + 1
        };
        table[divisor] = (magic, shift);
        divisor += 1;
    }
    table
}

const MOD_MAGIC_TABLE: [(u64, u32); 1024] = build_mod_magic_table();

#[cold]
#[inline(never)]
fn div_i32_cold(val_a: i32, val_b: i32) -> Result<i32, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a.wrapping_div(val_b))
}

#[cold]
#[inline(never)]
fn rem_i32_cold(val_a: i32, val_b: i32) -> Result<i32, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a.wrapping_rem(val_b))
}

#[cold]
#[inline(never)]
fn div_i64_cold(val_a: i64, val_b: i64) -> Result<i64, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a.wrapping_div(val_b))
}

#[cold]
#[inline(never)]
fn rem_i64_cold(val_a: i64, val_b: i64) -> Result<i64, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a.wrapping_rem(val_b))
}

#[cold]
#[inline(never)]
fn div_u32_cold(val_a: u32, val_b: u32) -> Result<u32, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a / val_b)
}

#[cold]
#[inline(never)]
fn rem_u32_cold(val_a: u32, val_b: u32) -> Result<u32, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a % val_b)
}

#[cold]
#[inline(never)]
fn div_u64_cold(val_a: u64, val_b: u64) -> Result<u64, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a / val_b)
}

#[cold]
#[inline(never)]
fn rem_u64_cold(val_a: u64, val_b: u64) -> Result<u64, VmTrap> {
    if val_b == 0 {
        return Err(VmTrap::divide_by_zero());
    }
    Ok(val_a % val_b)
}

#[inline(always)]
fn read_u32_fast(operand: &Value) -> Result<u32, VmTrap> {
    match *operand {
        Value::I32(raw_integer) => Ok(raw_integer as u32),
        Value::I64(raw_integer) => u32::try_from(raw_integer)
            .map_err(|_| VmTrap::type_mismatch("u32 integer value", operand)),
        _ => Err(VmTrap::type_mismatch("u32 integer value", operand)),
    }
}

#[inline(always)]
fn read_u64_fast(operand: &Value) -> Result<u64, VmTrap> {
    match *operand {
        Value::I64(raw_integer) => Ok(raw_integer as u64),
        Value::I32(raw_integer) => Ok(raw_integer as u32 as u64),
        _ => Err(VmTrap::type_mismatch("u64 integer value", operand)),
    }
}

#[cold]
#[inline(never)]
fn shr_slow(left_ref: &Value, right_ref: &Value) -> Result<Value, VmTrap> {
    let right_signed = read_i64_fast(right_ref)?;
    if right_signed < 0 {
        return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
            operation: "bitwise shift right",
            reason: "negative shift count is invalid".to_string(),
        }));
    }
    let shift = (right_signed & 63) as u32;
    Ok(Value::I64(read_i64_fast(left_ref)?.wrapping_shr(shift)))
}

#[inline(always)]
fn read_heap_u32_fast(heap_buf: &[u8], addr: u32) -> Result<u32, VmTrap> {
    let idx = addr as usize;
    if idx + 4 > heap_buf.len() {
        return Err(VmTrap::null_reference());
    }
    // SAFETY: idx + 4 <= heap_buf.len() verified above.
    Ok(unsafe {
        u32::from_le(std::ptr::read_unaligned(
            heap_buf.as_ptr().add(idx).cast::<u32>(),
        ))
    })
}

#[inline(always)]
fn read_heap_u32_or_zero(heap_buf: &[u8], addr: u32) -> u32 {
    let idx = addr as usize;
    if idx + 4 > heap_buf.len() {
        return 0;
    }
    // SAFETY: idx + 4 <= heap_buf.len() verified above.
    unsafe {
        u32::from_le(std::ptr::read_unaligned(
            heap_buf.as_ptr().add(idx).cast::<u32>(),
        ))
    }
}

#[inline(always)]
fn read_heap_u64_or_zero(heap_buf: &[u8], addr: u32) -> u64 {
    let idx = addr as usize;
    if idx + 8 > heap_buf.len() {
        return 0;
    }
    // SAFETY: idx + 8 <= heap_buf.len() verified above.
    unsafe {
        u64::from_le(std::ptr::read_unaligned(
            heap_buf.as_ptr().add(idx).cast::<u64>(),
        ))
    }
}

#[inline(always)]
fn write_heap_u32_fast(heap_buf: &mut [u8], addr: u32, word_val: u32) {
    let idx = addr as usize;
    if idx + 4 <= heap_buf.len() {
        // SAFETY: idx + 4 <= heap_buf.len() verified above.
        unsafe {
            std::ptr::write_unaligned(
                heap_buf.as_mut_ptr().add(idx).cast::<u32>(),
                word_val.to_le(),
            );
        }
    }
}

#[inline(always)]
fn write_heap_u64_fast(heap_buf: &mut [u8], addr: u32, word_val: u64) {
    let idx = addr as usize;
    if idx + 8 <= heap_buf.len() {
        // SAFETY: idx + 8 <= heap_buf.len() verified above.
        unsafe {
            std::ptr::write_unaligned(
                heap_buf.as_mut_ptr().add(idx).cast::<u64>(),
                word_val.to_le(),
            );
        }
    }
}

impl VirtualMachine {
    #[inline(always)]
    fn current_frame_fast(&self) -> Result<&CallFrame, VmTrap> {
        self.call_frames.last().ok_or_else(VmTrap::null_reference)
    }

    #[inline(always)]
    fn current_frame_mut_fast(&mut self) -> Result<&mut CallFrame, VmTrap> {
        self.call_frames
            .last_mut()
            .ok_or_else(VmTrap::null_reference)
    }

    #[cold]
    #[inline(never)]
    fn resolve_constant_pool_symbol(
        &self,
        binary_id: usize,
        pool_idx: usize,
    ) -> Result<&str, VmTrap> {
        let pool = self
            .context
            .get_constant_pool(binary_id)
            .ok_or_else(|| VmTrap::from_error(constant_pool_missing_error(binary_id)))?;
        let pool_val = pool
            .get(pool_idx)
            .ok_or_else(|| VmTrap::from_error(constant_pool_symbol_index_error(pool_idx)))?;
        match pool_val {
            ConstantPoolValue::String(sym) => Ok(sym.as_str()),
            _ => Err(VmTrap::from_error(constant_pool_symbol_type_error(
                pool_val,
            ))),
        }
    }

    #[cold]
    #[inline(never)]
    fn invoke_native_callback_fast(
        &mut self,
        native_callback: crate::native::NativeCallback,
        caller_arg_start: usize,
        arg_count: usize,
    ) -> Result<Value, VmTrap> {
        self.record_native_call();
        let call_args =
            self.register_stack[caller_arg_start..caller_arg_start + arg_count].to_vec();
        native_callback
            .invoke(self, &call_args)
            .map_err(VmTrap::from_error)
    }

    #[inline(never)]
    fn compare_heap_strings(&self, addr_left: u32, addr_right: u32) -> Result<bool, VmTrap> {
        if addr_left == addr_right {
            return Ok(true);
        }
        if addr_left == 0 || addr_right == 0 {
            return Ok(false);
        }
        let left_idx = addr_left as usize;
        let right_idx = addr_right as usize;
        let heap_len = self.heap.buffer.len();
        if left_idx >= 8 && left_idx <= heap_len && right_idx >= 8 && right_idx <= heap_len {
            // SAFETY: Both header words are verified in-bounds above.
            let (len_left, len_right) = unsafe {
                let ptr = self.heap.buffer.as_ptr();
                (
                    u32::from_le(std::ptr::read_unaligned(
                        ptr.add(left_idx - 8).cast::<u32>(),
                    )) as usize,
                    u32::from_le(std::ptr::read_unaligned(
                        ptr.add(right_idx - 8).cast::<u32>(),
                    )) as usize,
                )
            };
            if len_left != len_right {
                return Ok(false);
            }
            if left_idx + len_left <= heap_len && right_idx + len_right <= heap_len {
                // SAFETY: Both payload byte slices are verified in-bounds above.
                unsafe {
                    let ptr = self.heap.buffer.as_ptr();
                    let left_slice = std::slice::from_raw_parts(ptr.add(left_idx), len_left);
                    let right_slice = std::slice::from_raw_parts(ptr.add(right_idx), len_right);
                    return Ok(left_slice == right_slice);
                }
            }
        }
        let hdr_left = self
            .heap
            .read_header(addr_left)
            .map_err(VmTrap::from_error)?;
        let hdr_right = self
            .heap
            .read_header(addr_right)
            .map_err(VmTrap::from_error)?;
        if hdr_left.size != hdr_right.size {
            return Ok(false);
        }
        let bytes_left = self
            .heap
            .read_bytes(addr_left, hdr_left.size as usize)
            .map_err(VmTrap::from_error)?;
        let bytes_right = self
            .heap
            .read_bytes(addr_right, hdr_right.size as usize)
            .map_err(VmTrap::from_error)?;
        Ok(bytes_left == bytes_right)
    }

    #[cold]
    #[inline(never)]
    fn release_reference_cold(&mut self, addr: u32) -> Result<(), VmTrap> {
        self.release_reference(addr).map_err(VmTrap::from_error)
    }

    #[cold]
    #[inline(never)]
    fn push_purple_candidate_cold(
        &mut self,
        addr: u32,
        tag_and_ref_word: u32,
    ) -> Result<(), VmTrap> {
        let updated_word = (tag_and_ref_word & !(3 << 26)) | (3 << 26);
        let addr_usize = addr as usize;
        // SAFETY: Caller verified addr_usize >= 8 && addr_usize <= heap.buffer.len().
        unsafe {
            std::ptr::write_unaligned(
                self.heap
                    .buffer
                    .as_mut_ptr()
                    .add(addr_usize - 4)
                    .cast::<u32>(),
                updated_word.to_le(),
            );
        }
        self.purple_buffer.push(addr);
        self.arc_cycle_collector_candidates += 1;
        if self.purple_buffer.len() >= self.cycle_threshold {
            self.collect_cycles()?;
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn maybe_push_cycle_candidate(
        &mut self,
        addr: u32,
        tag_and_ref: u32,
        tag_nibble: u8,
        decremented: u32,
    ) -> Result<(), VmTrap> {
        if (tag_nibble == BlockTag::Object as u8
            || tag_nibble == BlockTag::ArrayRef as u8
            || tag_nibble == BlockTag::Tuple as u8
            || tag_nibble == BlockTag::Closure as u8)
            && ((tag_and_ref >> 26) & 0x03) != 3
        {
            let needs_cycle_check = if tag_nibble == BlockTag::Object as u8 {
                let addr_usize = addr as usize;
                if addr_usize + 4 <= self.heap.buffer.len() {
                    // SAFETY: addr_usize + 4 <= self.heap.buffer.len() verified above.
                    let vtable_id = unsafe {
                        u32::from_le(std::ptr::read_unaligned(
                            self.heap.buffer.as_ptr().add(addr_usize).cast::<u32>(),
                        ))
                    };
                    !self.context.is_class_acyclic(vtable_id)
                } else {
                    true
                }
            } else {
                true
            };
            if needs_cycle_check {
                self.push_purple_candidate_cold(addr, decremented)?;
            }
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn resolve_virtual_slot_error(&self, vtable_id: u32, slot: usize) -> VmTrap {
        let Some(class_entry) = self.context.get_class_by_vtable_id(vtable_id) else {
            return VmTrap::from_error(vtable_not_found_error(vtable_id));
        };
        VmTrap::from_error(vtable_slot_not_found_error(slot, &class_entry.symbol))
    }

    #[cold]
    #[inline(never)]
    fn resolve_interface_slot_slow(
        &self,
        vtable_id: u32,
        iface_id: u32,
        slot: usize,
    ) -> Result<u32, VmTrap> {
        let class_entry = self
            .context
            .get_class_by_vtable_id(vtable_id)
            .ok_or_else(|| vtable_not_found_error(vtable_id))?;
        let resolved_func_id = if iface_id != 0 {
            class_entry
                .itable
                .iter()
                .find(|(interface_key, _)| *interface_key == iface_id)
                .and_then(|(_, method_slots)| method_slots.get(slot).copied())
        } else {
            None
        }
        .or_else(|| {
            class_entry
                .itable
                .iter()
                .find_map(|(_, method_slots)| method_slots.get(slot).copied())
                .or_else(|| class_entry.vtable.get(slot).copied())
        })
        .ok_or_else(|| interface_slot_not_found_error(slot, &class_entry.symbol))?;
        Ok(resolved_func_id)
    }

    #[cold]
    #[inline(never)]
    fn execute_get_array_special(
        &self,
        arr_offset: usize,
        index_i64: i64,
        tag_nibble: u8,
    ) -> Result<Value, VmTrap> {
        if tag_nibble == BlockTag::String as u8 {
            let string_len = u32::from_le_bytes(
                self.heap.buffer[arr_offset - 8..arr_offset - 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            ) as usize;
            if (index_i64 as u64) >= (string_len as u64) {
                return Err(VmTrap::index_out_of_bounds(index_i64, string_len));
            }
            let byte_offset = arr_offset + index_i64 as usize;
            if byte_offset >= self.heap.buffer.len() {
                return Err(VmTrap::null_reference());
            }
            return Ok(Value::I32(self.heap.buffer[byte_offset] as i32));
        }

        let tuple_len = u32::from_le_bytes(
            self.heap.buffer[arr_offset..arr_offset + 4]
                .try_into()
                .unwrap_or([0; 4]),
        ) as usize;
        if (index_i64 as u64) >= (tuple_len as u64) {
            return Err(VmTrap::index_out_of_bounds(index_i64, tuple_len));
        }
        let elem_offset = (index_i64 as usize)
            .checked_mul(16)
            .and_then(|offset| arr_offset.checked_add(8)?.checked_add(offset))
            .ok_or_else(VmTrap::null_reference)?;
        if elem_offset.checked_add(16).is_none() || elem_offset + 16 > self.heap.buffer.len() {
            return Err(VmTrap::null_reference());
        }
        let tag_raw = u64::from_le_bytes(
            self.heap.buffer[elem_offset..elem_offset + 8]
                .try_into()
                .unwrap_or([0; 8]),
        );
        let payload_raw = u64::from_le_bytes(
            self.heap.buffer[elem_offset + 8..elem_offset + 16]
                .try_into()
                .unwrap_or([0; 8]),
        );
        Ok(Value::from_tagged_parts(tag_raw, payload_raw))
    }

    #[cold]
    #[inline(never)]
    fn execute_set_tuple(
        &mut self,
        arr_offset: usize,
        index_usize: usize,
        val: Value,
        is_move: bool,
    ) -> Result<bool, VmTrap> {
        let tuple_len = self
            .heap
            .read_u32(arr_offset as u32)
            .map_err(VmTrap::from_error)? as usize;
        if index_usize >= tuple_len {
            return Err(VmTrap::index_out_of_bounds(index_usize as i64, tuple_len));
        }
        let elem_offset = arr_offset + 8 + (index_usize * 16);
        if elem_offset + 16 > self.heap.buffer.len() {
            return Err(VmTrap::null_reference());
        }
        let old_tag = u64::from_le_bytes(
            self.heap.buffer[elem_offset..elem_offset + 8]
                .try_into()
                .unwrap_or([0; 8]),
        );
        let old_payload = u64::from_le_bytes(
            self.heap.buffer[elem_offset + 8..elem_offset + 16]
                .try_into()
                .unwrap_or([0; 8]),
        );
        let old_val = Value::from_tagged_parts(old_tag, old_payload);

        let (new_tag, new_payload) = val.to_tagged_parts();
        self.heap.buffer[elem_offset..elem_offset + 8].copy_from_slice(&new_tag.to_le_bytes());
        self.heap.buffer[elem_offset + 8..elem_offset + 16]
            .copy_from_slice(&new_payload.to_le_bytes());

        if !is_move
            && let Value::Ref(child_addr) = val
            && child_addr > 0
        {
            crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
        }
        if let Value::Ref(old_child) = old_val
            && old_child > 0
        {
            self.release_reference_cold(old_child)?;
            return Ok(true);
        }
        Ok(false)
    }

    #[cold]
    #[inline(never)]
    fn execute_get_field_slow(
        &mut self,
        obj_addr: u32,
        offset: usize,
        raw_flags: u8,
    ) -> Result<Value, VmTrap> {
        let field_addr = obj_addr + offset as u32;
        let is_weak_flag = (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) != 0;
        let field_type_tag = if raw_flags > 1 {
            Some(raw_flags >> 1)
        } else {
            let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
            self.context
                .get_class_by_vtable_id(vtable_id)
                .and_then(|cls| cls.get_field_type(offset))
        };

        let val = match field_type_tag {
            Some(1) => Value::I32(self.heap.read_u8(field_addr).unwrap_or(0) as i8 as i32),
            Some(2) => Value::I32(self.heap.read_u8(field_addr).unwrap_or(0) as i32),
            Some(3) => Value::I32(self.heap.read_u16(field_addr).unwrap_or(0) as i16 as i32),
            Some(4) => Value::I32(self.heap.read_u16(field_addr).unwrap_or(0) as i32),
            Some(5) | Some(6) => Value::I32(self.heap.read_u32(field_addr).unwrap_or(0) as i32),
            Some(7) | Some(8) => Value::I64(self.heap.read_u64(field_addr).unwrap_or(0) as i64),
            Some(9) => {
                Value::F64(f32::from_bits(self.heap.read_u32(field_addr).unwrap_or(0)) as f64)
            }
            Some(10) => Value::F64(f64::from_bits(self.heap.read_u64(field_addr).unwrap_or(0))),
            Some(11) => Value::Bool(self.heap.read_u8(field_addr).unwrap_or(0) != 0),
            Some(12) => {
                let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                if raw == 0 || raw == Value::TAG_NULL {
                    Value::Null
                } else if (raw & Value::TAG_MASK) == Value::TAG_REF {
                    Value::Ref(raw as u32)
                } else if (raw & Value::TAG_MASK) == 0 {
                    Value::Ref((raw & 0xFFFF_FFFF) as u32)
                } else {
                    Value::from_raw_u64(raw)
                }
            }
            _ => {
                let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
                let maybe_class = self.context.get_class_by_vtable_id(vtable_id);
                let is_ref = maybe_class.is_some_and(|cls| {
                    cls.reference_field_offsets.contains(&offset)
                        || cls.weak_field_offsets.contains(&offset)
                });
                if raw == 0 || (is_ref && raw == Value::TAG_NULL) {
                    if is_ref { Value::Null } else { Value::I32(0) }
                } else if is_ref {
                    if (raw & Value::TAG_MASK) == Value::TAG_REF {
                        Value::Ref(raw as u32)
                    } else if (raw & Value::TAG_MASK) == 0 {
                        Value::Ref((raw & 0xFFFF_FFFF) as u32)
                    } else {
                        Value::from_raw_u64(raw)
                    }
                } else {
                    Value::from_raw_u64(raw)
                }
            }
        };

        if let Value::Ref(target_addr) = val {
            let is_weak_field = is_weak_flag || {
                let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
                self.context
                    .get_class_by_vtable_id(vtable_id)
                    .is_some_and(|cls| cls.weak_field_offsets.contains(&offset))
            };
            if is_weak_field
                && (target_addr == 0 || !self.heap.validate_weak_slot(field_addr, target_addr))
            {
                let _ = self.heap.write_u64(field_addr, Value::TAG_NULL);
                return Ok(Value::Null);
            }
        }
        Ok(val)
    }

    #[inline(never)]
    fn execute_set_field_slow(
        &mut self,
        obj_addr: u32,
        offset: usize,
        raw_flags: u8,
        val: Value,
        pc: usize,
    ) -> Result<bool, VmTrap> {
        let field_addr = obj_addr + offset as u32;
        let is_weak = (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) != 0;
        let is_move = (raw_flags & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
        let tag_shifted = (raw_flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
        let field_type_tag = if tag_shifted > 0 {
            Some(tag_shifted)
        } else {
            let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
            self.context
                .get_class_by_vtable_id(vtable_id)
                .and_then(|cls| cls.get_field_type(offset))
        };

        match field_type_tag {
            Some(1) | Some(2) => {
                self.heap.write_u8(field_addr, read_i32_fast(&val)? as u8)?;
            }
            Some(3) | Some(4) => {
                self.heap
                    .write_u16(field_addr, read_i32_fast(&val)? as u16)?;
            }
            Some(5) | Some(6) => {
                self.heap
                    .write_u32(field_addr, read_i32_fast(&val)? as u32)?;
            }
            Some(7) | Some(8) => {
                self.heap
                    .write_u64(field_addr, read_i64_fast(&val)? as u64)?;
            }
            Some(9) => {
                let bits = (read_f64_fast(&val)? as f32).to_bits();
                self.heap.write_u32(field_addr, bits)?;
            }
            Some(10) => {
                let bits = read_f64_fast(&val)?.to_bits();
                self.heap.write_u64(field_addr, bits)?;
            }
            Some(11) => {
                self.heap
                    .write_u8(field_addr, if read_bool_fast(&val)? { 1 } else { 0 })?;
            }
            _ => {
                let old_raw = self.heap.read_u64(field_addr).unwrap_or(0);
                let old_val = Value::from_raw_u64(old_raw);
                self.heap.write_u64(field_addr, val.to_raw_u64())?;
                if is_weak {
                    let target_addr = if let Value::Ref(addr) = val { addr } else { 0 };
                    self.heap
                        .record_weak_slot(obj_addr, field_addr, target_addr);
                }
                if let Value::Ref(child_addr) = val
                    && !is_weak
                    && !is_move
                    && child_addr > 0
                {
                    crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                }
                if let Value::Ref(old_child) = old_val
                    && !is_weak
                    && old_child > 0
                {
                    let caller_frame = self.current_frame_mut_fast()?;
                    caller_frame.pc = pc as u32;
                    self.release_reference_cold(old_child)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    #[inline(never)]
    fn execute_set_array_slow(
        &mut self,
        arr_offset: usize,
        index_usize: usize,
        val_reg: u16,
        raw_flags: u8,
        pc: usize,
    ) -> Result<bool, VmTrap> {
        let is_move = (raw_flags & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
        let val = {
            let caller_frame = self.current_frame_fast()?;
            let reg_idx = caller_frame.register_base as usize + val_reg as usize;
            let loaded = self.register_stack[reg_idx];
            if is_move {
                self.register_stack[reg_idx] = Value::Null;
            }
            loaded
        };
        let buf_ptr = self.heap.buffer.as_mut_ptr();
        let heap_len = self.heap.buffer.len();
        // SAFETY: Caller verified arr_offset >= 8 && arr_offset + 8 <= heap_len.
        let elem_size = unsafe { (*buf_ptr.add(arr_offset + 4) as usize).max(1) };
        let elem_type_tag = unsafe { *buf_ptr.add(arr_offset + 7) };
        let elem_offset = index_usize
            .checked_mul(elem_size)
            .and_then(|offset| arr_offset.checked_add(8)?.checked_add(offset))
            .ok_or_else(VmTrap::null_reference)?;
        if elem_offset.checked_add(elem_size).is_none() || elem_offset + elem_size > heap_len {
            return Err(VmTrap::null_reference());
        }

        let is_reference_element = elem_type_tag == 13 || elem_type_tag >= 0x80;
        if is_reference_element
            && (elem_type_tag & 0x80) != 0
            && let Value::Ref(assigned_address) = val
            && assigned_address > 0
        {
            let target_vtable_id = {
                let vtable_u16 = unsafe {
                    u16::from_le(std::ptr::read_unaligned(
                        buf_ptr.add(arr_offset + 5).cast::<u16>(),
                    ))
                } as u32;
                if vtable_u16 > 0 {
                    vtable_u16
                } else {
                    (elem_type_tag & 0x7F) as u32
                }
            };
            if target_vtable_id > 0 {
                let header = self
                    .heap
                    .read_header(assigned_address)
                    .map_err(VmTrap::from_error)?;
                if header.tag == BlockTag::Object || header.tag == BlockTag::ObjectPod {
                    let assigned_vtable_id = self
                        .heap
                        .read_u32(assigned_address)
                        .map_err(VmTrap::from_error)?;
                    if !self.is_instance_of(assigned_vtable_id, target_vtable_id) {
                        return Err(VmTrap::from_error(RuntimeError::ArrayStoreMismatch {
                            assigned_class: assigned_vtable_id,
                            array_element_class: target_vtable_id,
                        }));
                    }
                }
            }
        }
        // SAFETY: elem_offset + elem_size <= heap_len verified above.
        let elem_ptr = unsafe { buf_ptr.add(elem_offset) };
        let old_child = if is_reference_element {
            unsafe { u32::from_le(std::ptr::read_unaligned(elem_ptr.cast::<u32>())) }
        } else {
            0
        };

        if elem_type_tag == 0 {
            let encoded_word = val.to_raw_u64().to_le();
            unsafe {
                std::ptr::write_unaligned(elem_ptr.cast::<u64>(), encoded_word);
            }
        } else if elem_type_tag == 9 && elem_size == 4 {
            let bits = (read_f64_fast(&val)? as f32).to_bits();
            unsafe {
                std::ptr::write_unaligned(elem_ptr.cast::<u32>(), bits.to_le());
            }
        } else {
            let primitive_val = val.to_primitive_u64();
            match elem_size {
                1 => unsafe {
                    *elem_ptr = primitive_val as u8;
                },
                2 => unsafe {
                    std::ptr::write_unaligned(
                        elem_ptr.cast::<u16>(),
                        (primitive_val as u16).to_le(),
                    );
                },
                4 => unsafe {
                    std::ptr::write_unaligned(
                        elem_ptr.cast::<u32>(),
                        (primitive_val as u32).to_le(),
                    );
                },
                _ => unsafe {
                    std::ptr::write_unaligned(elem_ptr.cast::<u64>(), primitive_val.to_le());
                },
            }
        }

        if is_reference_element {
            if !is_move
                && let Value::Ref(child_addr) = val
                && child_addr > 0
            {
                crate::memory::arc::inc_ref(&mut self.heap, child_addr)
                    .map_err(VmTrap::from_error)?;
            }
            if old_child > 0 {
                let caller_frame = self.current_frame_mut_fast()?;
                caller_frame.pc = pc as u32;
                self.release_reference_cold(old_child)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cold]
    #[inline(never)]
    fn invoke_native_call_id(
        &mut self,
        global_func_id: u32,
        caller_dst: u16,
        caller_arg_start: usize,
        arg_count: usize,
    ) -> Result<(*const Instruction, usize, usize, *mut Value, usize, u32), VmTrap> {
        let Some(meta) = self.context.function_meta.get(global_func_id as usize) else {
            return Err(function_not_found_error(global_func_id));
        };
        let Some(native_callback) = meta.native_fn else {
            return self.push_frame_window(global_func_id, caller_dst, caller_arg_start, arg_count);
        };
        let call_result =
            self.invoke_native_callback_fast(native_callback, caller_arg_start, arg_count)?;
        let (caller_func_id, caller_binary_id, resume_pc, caller_reg_base_offset) = {
            let caller = self.current_frame_fast()?;
            (
                caller.function_id as usize,
                caller.binary_id as usize,
                caller.pc as usize,
                caller.register_base as usize,
            )
        };
        let code_span = self.func_code[caller_func_id];
        // SAFETY: caller_reg_base_offset is within register_stack and caller_dst within frame window.
        let caller_reg_base = unsafe {
            let ptr = self.register_stack.as_mut_ptr().add(caller_reg_base_offset);
            *ptr.add(caller_dst as usize) = call_result;
            ptr
        };
        Ok((
            code_span.inst_ptr,
            code_span.inst_len,
            resume_pc,
            caller_reg_base,
            caller_binary_id,
            code_span.static_base_offset,
        ))
    }

    #[cold]
    #[inline(never)]
    fn execute_call_symbol(&mut self, inst: Instruction, pc: usize) -> Result<(), VmTrap> {
        let const_idx = inst.call_target() as usize;
        let arg_base = inst.arg_base();
        let arg_count = inst.arg_count() as usize;
        let caller_dst = inst.call_dst();
        let caller_pc = pc.saturating_sub(1);
        let binary_id = self.current_frame_fast()?.binary_id as usize;

        let sym_str = self
            .resolve_constant_pool_symbol(binary_id, const_idx)?
            .to_owned();

        let caller_frame = self.current_frame_mut()?;
        caller_frame.pc = pc as u32;
        let caller_func_id = caller_frame.function_id;
        let caller_arg_start = caller_frame.register_base as usize + arg_base as usize;

        let clean_sym = sym_str.split('(').next().unwrap_or(&sym_str);
        let mangled_sym = clean_sym.replace("::", "_");
        let simple_sym = clean_sym.rsplit(['.', ':']).next().unwrap_or(clean_sym);
        let native_candidate = self
            .native_registry
            .get(&sym_str)
            .or_else(|| self.native_registry.get(clean_sym))
            .or_else(|| self.native_registry.get(&mangled_sym))
            .or_else(|| self.native_registry.get(simple_sym))
            .copied();

        if let Some(native_fn) = native_candidate {
            let call_result =
                self.invoke_native_callback_fast(native_fn, caller_arg_start, arg_count)?;
            let target_base = self.current_frame()?.register_base as usize;
            self.register_stack[target_base + caller_dst as usize] = call_result;
            if let Some(&global_id) = self.context.symbol_to_function_id.get(&sym_str)
                && self
                    .context
                    .function_meta
                    .get(global_id as usize)
                    .is_some_and(|meta| meta.native_fn.is_some())
            {
                let quickened = Instruction::format_call(
                    OpCode::CallNativeDirect,
                    caller_dst,
                    global_id as u16,
                    arg_base,
                    arg_count as u8,
                );
                self.patch_instruction(caller_func_id, caller_pc, quickened);
            }
            return Ok(());
        }

        let global_id = self
            .context
            .symbol_to_function_id
            .get(&sym_str)
            .copied()
            .ok_or_else(|| {
                if sym_str.contains("::") || sym_str.contains('_') {
                    let mangled_unresolved = sym_str.replace("::", "_");
                    RuntimeError::FunctionNotFound {
                        symbol_or_id: format!(
                            "Call to unknown native function: {mangled_unresolved}"
                        ),
                    }
                } else {
                    RuntimeError::FunctionNotFound {
                        symbol_or_id: format!("Call to unknown native function: {sym_str}"),
                    }
                }
            })?;

        if caller_func_id == global_id {
            let mangled_unresolved = sym_str.replace("::", "_");
            return Err(RuntimeError::FunctionNotFound {
                symbol_or_id: format!("Call to unknown native function: {mangled_unresolved}"),
            }
            .into());
        }

        let quickened = Instruction::format_call(
            OpCode::CallDirect,
            caller_dst,
            global_id as u16,
            arg_base,
            arg_count as u8,
        );
        self.patch_instruction(caller_func_id, caller_pc, quickened);

        self.push_frame_window(global_id, caller_dst, caller_arg_start, arg_count)
            .map(|_| ())
    }

    #[cold]
    #[inline(never)]
    fn execute_call_extern(&mut self, inst: Instruction, pc: usize) -> Result<(), VmTrap> {
        let const_idx = inst.call_target() as usize;
        let arg_base = inst.arg_base();
        let arg_count = inst.arg_count() as usize;
        let caller_dst = inst.call_dst();
        let caller_pc = pc.saturating_sub(1);
        let binary_id = self.current_frame_fast()?.binary_id as usize;

        let sym_str = self
            .resolve_constant_pool_symbol(binary_id, const_idx)?
            .to_owned();

        let caller_frame = self.current_frame_mut()?;
        caller_frame.pc = pc as u32;
        let caller_func_id = caller_frame.function_id;
        let caller_arg_start = caller_frame.register_base as usize + arg_base as usize;

        if let Some(&native_fn) = self.native_registry.get(&sym_str) {
            let call_result =
                self.invoke_native_callback_fast(native_fn, caller_arg_start, arg_count)?;
            let target_base = self.current_frame()?.register_base as usize;
            self.register_stack[target_base + caller_dst as usize] = call_result;
            return Ok(());
        }

        let global_id = self
            .context
            .resolve_function_symbol(&sym_str)
            .ok_or_else(|| RuntimeError::FunctionNotFound {
                symbol_or_id: sym_str.clone(),
            })?;

        let quickened = Instruction::format_call(
            OpCode::CallDirect,
            caller_dst,
            global_id as u16,
            arg_base,
            arg_count as u8,
        );
        self.patch_instruction(caller_func_id, caller_pc, quickened);

        if let Some(func) = self.context.get_function(global_id) {
            let clean_sym = func.symbol.split('(').next().unwrap_or(&func.symbol);
            let simple_sym = clean_sym.rsplit(['.', ':']).next().unwrap_or(clean_sym);
            let has_qualifier = clean_sym.contains("::");
            if let Some(&native_fn) = self
                .native_registry
                .get(&func.symbol)
                .or_else(|| self.native_registry.get(clean_sym))
                .or_else(|| {
                    if !has_qualifier {
                        self.native_registry.get(simple_sym)
                    } else {
                        None
                    }
                })
            {
                let call_result =
                    self.invoke_native_callback_fast(native_fn, caller_arg_start, arg_count)?;
                let target_base = self.current_frame()?.register_base as usize;
                self.register_stack[target_base + caller_dst as usize] = call_result;
                return Ok(());
            }
        }

        self.push_frame_window(global_id, caller_dst, caller_arg_start, arg_count)
            .map(|_| ())
    }

    #[inline(never)]
    fn set_static_trap(&mut self, slot: u32, val: Value) -> Result<(), VmTrap> {
        self.set_static(slot, val).map_err(VmTrap::from_error)
    }

    #[cold]
    #[inline(never)]
    fn execute_alloc_object_extern(
        &mut self,
        inst: Instruction,
        pc: usize,
    ) -> Result<Value, VmTrap> {
        let (caller_func_id, binary_id) = {
            let frame = self.current_frame_fast()?;
            (frame.function_id, frame.binary_id as usize)
        };
        let caller_pc = pc.saturating_sub(1);
        let pool_idx = inst.imm32() as usize;
        let (vtable_id, size) = {
            let sym_str = self.resolve_constant_pool_symbol(binary_id, pool_idx)?;
            self.context.resolve_vtable_info(sym_str).ok_or_else(|| {
                VmTrap::from_error(RuntimeError::FunctionNotFound {
                    symbol_or_id: format!("class {sym_str}"),
                })
            })?
        };
        let alloc_size = size.max(8);
        let is_pod = self
            .context
            .get_class_by_vtable_id(vtable_id)
            .is_some_and(|class_item| class_item.is_pod());
        let tag = if is_pod {
            BlockTag::ObjectPod
        } else {
            BlockTag::Object
        };
        let payload_addr = self
            .heap
            .allocate(alloc_size, tag)
            .map_err(VmTrap::from_error)?;
        self.heap
            .write_u32(payload_addr, vtable_id)
            .map_err(VmTrap::from_error)?;

        let flags = if is_pod {
            solix_common::bytecode::instruction::FLAG_POD
        } else {
            0
        };
        let quickened = Instruction::format_a(
            OpCode::AllocObject,
            flags,
            inst.dst(),
            vtable_id as u16,
            alloc_size as u16,
        );
        self.patch_instruction(caller_func_id, caller_pc, quickened);
        Ok(Value::Ref(payload_addr))
    }

    #[cold]
    #[inline(never)]
    fn execute_get_static_global(&mut self, inst: Instruction, pc: usize) -> Result<Value, VmTrap> {
        let (caller_func_id, binary_id) = {
            let frame = self.current_frame_fast()?;
            (frame.function_id, frame.binary_id as usize)
        };
        let caller_pc = pc.saturating_sub(1);
        let pool_idx = inst.imm32() as usize;
        let resolved_slot = {
            let sym_str = self.resolve_constant_pool_symbol(binary_id, pool_idx)?;
            self.context.resolve_static_slot(sym_str).ok_or_else(|| {
                VmTrap::invalid_static_offset(pool_idx as u32, self.static_slots.len() as u32)
            })?
        };
        let quickened =
            Instruction::format_b(OpCode::GetStaticDirect, 0, inst.dst(), resolved_slot);
        self.patch_instruction(caller_func_id, caller_pc, quickened);
        self.get_static(resolved_slot).map_err(VmTrap::from_error)
    }

    #[cold]
    #[inline(never)]
    fn execute_set_static_global(
        &mut self,
        inst: Instruction,
        pc: usize,
        val_to_set: Value,
    ) -> Result<(), VmTrap> {
        let (caller_func_id, binary_id) = {
            let frame = self.current_frame_fast()?;
            (frame.function_id, frame.binary_id as usize)
        };
        let caller_pc = pc.saturating_sub(1);
        let pool_idx = inst.imm32() as usize;
        let resolved_slot = {
            let sym_str = self.resolve_constant_pool_symbol(binary_id, pool_idx)?;
            self.context.resolve_static_slot(sym_str).ok_or_else(|| {
                VmTrap::invalid_static_offset(pool_idx as u32, self.static_slots.len() as u32)
            })?
        };
        let quickened =
            Instruction::format_b(OpCode::SetStaticDirect, 0, inst.dst(), resolved_slot);
        self.patch_instruction(caller_func_id, caller_pc, quickened);
        self.set_static(resolved_slot, val_to_set)
            .map_err(VmTrap::from_error)
    }

    #[inline(never)]
    fn execute_alloc_tuple(&mut self, tuple_count: usize) -> Result<Value, VmTrap> {
        let payload_size = tuple_count
            .checked_mul(16)
            .and_then(|bytes| bytes.checked_add(8))
            .filter(|&total_size| total_size <= u32::MAX as usize)
            .ok_or_else(|| VmTrap::out_of_memory(usize::MAX, self.heap.buffer.len()))?;
        let payload_addr = self
            .heap
            .allocate(payload_size, BlockTag::Tuple)
            .map_err(VmTrap::from_error)?;
        self.heap
            .write_u32(payload_addr, tuple_count as u32)
            .map_err(VmTrap::from_error)?;
        self.heap
            .write_u32(payload_addr + 4, 16)
            .map_err(VmTrap::from_error)?;
        Ok(Value::Ref(payload_addr))
    }

    #[inline(never)]
    fn execute_get_tuple(&self, tuple_val: Value, slot_idx: usize) -> Result<Value, VmTrap> {
        let tuple_addr = tuple_val.as_ref().map_err(VmTrap::from_error)?;
        if tuple_addr == 0 {
            return Err(VmTrap::null_reference());
        }
        let tuple_len = self.heap.read_u32(tuple_addr).map_err(VmTrap::from_error)? as usize;
        if slot_idx >= tuple_len {
            return Err(VmTrap::index_out_of_bounds(slot_idx as i64, tuple_len));
        }
        let elem_offset = (slot_idx as u32)
            .checked_mul(16)
            .ok_or_else(|| VmTrap::index_out_of_bounds(slot_idx as i64, tuple_len))?;
        let elem_addr = tuple_addr
            .checked_add(8)
            .and_then(|base_addr| base_addr.checked_add(elem_offset))
            .ok_or_else(VmTrap::null_reference)?;
        let tag_raw = self.heap.read_u64(elem_addr).unwrap_or(0);
        let payload_raw = self.heap.read_u64(elem_addr + 8).unwrap_or(0);
        Ok(Value::from_tagged_parts(tag_raw, payload_raw))
    }

    #[inline(never)]
    fn execute_tuple_instanceof(
        &self,
        tuple_val: Value,
        index_val: Value,
        tag: u8,
    ) -> Result<bool, VmTrap> {
        let tuple_addr = match tuple_val.as_ref() {
            Ok(addr) if addr > 0 => addr,
            _ => return Ok(false),
        };
        let index_i64 = index_val.as_i64().map_err(VmTrap::from_error)?;
        let tuple_len = self.heap.read_u32(tuple_addr).map_err(VmTrap::from_error)? as usize;
        if index_i64 < 0 || (index_i64 as usize) >= tuple_len {
            return Err(VmTrap::index_out_of_bounds(index_i64, tuple_len));
        }
        let slot_index_u32 = (index_i64 as usize) as u32;
        let elem_offset = slot_index_u32
            .checked_mul(16)
            .ok_or_else(|| VmTrap::index_out_of_bounds(index_i64, tuple_len))?;
        let elem_addr = tuple_addr
            .checked_add(8)
            .and_then(|base_addr| base_addr.checked_add(elem_offset))
            .ok_or_else(VmTrap::null_reference)?;
        let tag_raw = self.heap.read_u64(elem_addr).unwrap_or(0);
        let payload_raw = self.heap.read_u64(elem_addr + 8).unwrap_or(0);
        let val = Value::from_tagged_parts(tag_raw, payload_raw);
        let is_match = match tag {
            1..=8 => matches!(val, Value::I32(_) | Value::I64(_)),
            9..=10 => matches!(val, Value::F64(_)),
            11 => matches!(val, Value::Bool(_)),
            13 => {
                if let Value::Ref(target_addr) = val {
                    self.heap
                        .read_header(target_addr)
                        .is_ok_and(|header_info| header_info.tag == BlockTag::String)
                } else {
                    false
                }
            }
            _ if (tag & 0x80) != 0 => {
                let target_vtable_id = (tag & 0x7F) as u32;
                if let Value::Ref(target_addr) = val {
                    let obj_vtable_id = self.heap.read_u32(target_addr).unwrap_or(0);
                    self.is_instance_of(obj_vtable_id, target_vtable_id)
                } else {
                    false
                }
            }
            _ => false,
        };
        Ok(is_match)
    }

    #[inline(never)]
    fn execute_pack_closure_heap(
        &mut self,
        target_lambda_id: u32,
        cap_base: usize,
        cap_count: usize,
    ) -> Result<Value, VmTrap> {
        let frame_base = self.current_frame_fast()?.register_base as usize;
        let payload_size = 8 + cap_count * 16;
        let payload_addr = self
            .heap
            .allocate(payload_size, BlockTag::Closure)
            .map_err(VmTrap::from_error)?;
        self.heap
            .write_u32(payload_addr, target_lambda_id)
            .map_err(VmTrap::from_error)?;
        self.heap
            .write_u32(payload_addr + 4, cap_count as u32)
            .map_err(VmTrap::from_error)?;

        for index in 0..cap_count {
            let cell_addr = payload_addr + 8 + (index * 16) as u32;
            let cap_val = self.register_stack[frame_base + cap_base + index];
            let (tag_raw, payload_raw) = cap_val.to_tagged_parts();
            let _ = self.heap.write_u64(cell_addr, tag_raw);
            let _ = self.heap.write_u64(cell_addr + 8, payload_raw);
            if let Value::Ref(child_addr) = cap_val
                && child_addr > 0
            {
                crate::memory::arc::inc_ref(&mut self.heap, child_addr)
                    .map_err(VmTrap::from_error)?;
            }
        }
        Ok(Value::Ref(payload_addr))
    }

    #[inline(never)]
    fn execute_unpack_captures(
        &mut self,
        capture_base: usize,
        capture_count: usize,
    ) -> Result<(), VmTrap> {
        let frame = self.current_frame_fast()?;
        let frame_base = frame.register_base as usize;
        let closure_addr = frame
            .active_closure
            .map(std::num::NonZeroU32::get)
            .ok_or_else(VmTrap::null_reference)?;

        for index in 0..capture_count {
            let cell_addr = closure_addr + 8 + (index * 16) as u32;
            let tag_raw = self.heap.read_u64(cell_addr).unwrap_or(0);
            let payload_raw = self.heap.read_u64(cell_addr + 8).unwrap_or(0);
            self.register_stack[frame_base + capture_base + index] =
                Value::from_tagged_parts(tag_raw, payload_raw);
        }
        Ok(())
    }

    #[inline(never)]
    fn execute_instanceof(&self, obj_val: Value, target_vtable_id: u32) -> Result<bool, VmTrap> {
        let obj_addr = match obj_val.as_ref() {
            Ok(addr) if addr > 0 => addr,
            _ => return Ok(false),
        };
        let header = self
            .heap
            .read_header(obj_addr)
            .map_err(VmTrap::from_error)?;
        if header.tag == BlockTag::Object || header.tag == BlockTag::ObjectPod {
            let obj_vtable_id = self.heap.read_u32(obj_addr).map_err(VmTrap::from_error)?;
            Ok(self.is_instance_of(obj_vtable_id, target_vtable_id))
        } else {
            Ok(false)
        }
    }

    #[inline(never)]
    fn execute_cast_check(
        &self,
        val: Value,
        tag: u8,
        target_vtable_id: u32,
    ) -> Result<Value, VmTrap> {
        if tag != 0 {
            let is_match = match tag {
                1..=8 | 12 => matches!(val, Value::I32(_) | Value::I64(_)),
                9..=10 => matches!(val, Value::F64(_)),
                11 => matches!(val, Value::Bool(_)),
                13 => {
                    if let Value::Ref(target_addr) = val {
                        self.heap
                            .read_header(target_addr)
                            .is_ok_and(|header_info| header_info.tag == BlockTag::String)
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if !is_match {
                return Err(VmTrap::from_error(invalid_cast_tag_error(val, tag)));
            }
            let converted_value = match (tag, val) {
                (4 | 8, Value::I32(primitive_number)) => Value::I64(primitive_number as i64),
                (1, Value::I32(primitive_number)) => Value::I32(primitive_number as i8 as i32),
                (2, Value::I32(primitive_number)) => Value::I32(primitive_number as i16 as i32),
                (5, Value::I32(primitive_number)) => Value::I32(primitive_number as u8 as i32),
                (6, Value::I32(primitive_number)) => Value::I32(primitive_number as u16 as i32),
                (1, Value::I64(primitive_number)) => Value::I32(primitive_number as i8 as i32),
                (2, Value::I64(primitive_number)) => Value::I32(primitive_number as i16 as i32),
                (5, Value::I64(primitive_number)) => Value::I32(primitive_number as u8 as i32),
                (6, Value::I64(primitive_number)) => Value::I32(primitive_number as u16 as i32),
                (3 | 7 | 12, Value::I64(primitive_number)) => Value::I32(primitive_number as i32),
                _ => val,
            };
            return Ok(converted_value);
        }

        let obj_addr = val.as_ref().map_err(VmTrap::from_error)?;
        if obj_addr == 0 {
            return Ok(Value::Null);
        }
        let header = self
            .heap
            .read_header(obj_addr)
            .map_err(VmTrap::from_error)?;
        let is_instance = if header.tag == BlockTag::Object || header.tag == BlockTag::ObjectPod {
            let obj_vtable_id = self.heap.read_u32(obj_addr).map_err(VmTrap::from_error)?;
            self.is_instance_of(obj_vtable_id, target_vtable_id)
        } else {
            false
        };
        if !is_instance {
            return Err(VmTrap::from_error(invalid_cast_vtable_error(
                obj_addr,
                target_vtable_id,
            )));
        }
        Ok(Value::Ref(obj_addr))
    }

    /// Formats an uncaught exception value into a human-readable diagnostic message.
    pub fn format_exception_message(&self, exception_value: Value) -> String {
        match exception_value {
            Value::Null => "NullReferenceException".to_string(),
            Value::Ref(object_address) if object_address > 0 => {
                let header = match self.heap.read_header(object_address) {
                    Ok(header_val) => header_val,
                    Err(_) => return format!("ref({object_address})"),
                };
                match header.tag {
                    BlockTag::String => self
                        .heap
                        .read_string(object_address)
                        .unwrap_or_else(|_| format!("ref({object_address})")),
                    BlockTag::Object | BlockTag::ObjectPod => {
                        let header_vtable_id =
                            read_heap_u32_fast(&self.heap.buffer, object_address).unwrap_or(0);
                        let class_name = self
                            .context
                            .vtable_lookup
                            .get(header_vtable_id as usize)
                            .and_then(|class_option| class_option.as_ref())
                            .map(|compiled_class| compiled_class.symbol.clone())
                            .unwrap_or_else(|| {
                                self.context
                                    .all_classes
                                    .iter()
                                    .find(|candidate_class| {
                                        candidate_class.vtable_id == header_vtable_id
                                    })
                                    .map(|candidate_class| candidate_class.symbol.clone())
                                    .unwrap_or_else(|| format!("Object[vtable={header_vtable_id}]"))
                            });

                        if let Ok(fields) = self.inspect_object_fields(object_address) {
                            for (field_name, _field_type, field_value) in fields {
                                if (field_name == "message" || field_name == "error_message")
                                    && let Value::Ref(message_address) = field_value
                                    && message_address > 0
                                    && let Ok(message_string) =
                                        self.heap.read_string(message_address)
                                {
                                    return format!("{class_name}: {message_string}");
                                }
                            }
                        }
                        class_name
                    }
                    _ => format!("ref({object_address})"),
                }
            }
            other_value => format!("{other_value}"),
        }
    }

    #[cold]
    #[inline(never)]
    fn execute_throw(
        &mut self,
        exc_val: Value,
        exc_reg: Option<u16>,
        target_depth: usize,
    ) -> Result<(), VmTrap> {
        let mut exc_val = exc_val;
        if exc_val.is_null() {
            if let Some(target_class) = self.context.all_classes.iter().find(|candidate| {
                candidate.symbol == "NullReferenceException"
                    || candidate.symbol.ends_with(".NullReferenceException")
                    || candidate.symbol.ends_with("::NullReferenceException")
            }) {
                let payload_addr = alloc_object_trap(
                    &mut self.heap,
                    target_class.instance_size,
                    BlockTag::Object,
                    target_class.vtable_id,
                )?;
                exc_val = Value::Ref(payload_addr);
            } else {
                return Err(VmTrap::null_reference());
            }
        }

        // Determine whether the throwing frame already holds an independent owned reference for `exc_val`:
        // - Owned local register (`exc_reg` in `func.reference_registers`) or freshly allocated temporary
        //   (`exc_reg >= func.param_count`): transfer ownership out of `exc_reg` by clearing it to `Null`.
        // - Borrowed parameter (`exc_reg < func.param_count` and not in `reference_registers`) or unknown:
        //   retain an in-flight reference so caller/unwind cleanup cannot free `exc_val` while unwinding.
        let mut retained_flight_ref = false;
        if let Value::Ref(exc_addr) = exc_val
            && exc_addr > 0
        {
            let mut need_retain = true;
            if let (Some(reg_idx), Ok(frame)) = (exc_reg, self.current_frame_fast())
                && let Some(throwing_func) = self.context.get_function(frame.function_id)
            {
                let is_owned_local = throwing_func.reference_registers.contains(&reg_idx);
                let is_owned_temp = reg_idx >= throwing_func.param_count;
                if is_owned_local || is_owned_temp {
                    let abs_idx = frame.register_base as usize + reg_idx as usize;
                    if let Some(slot) = self.register_stack.get_mut(abs_idx) {
                        *slot = Value::Null;
                        need_retain = false;
                    }
                }
            }
            if need_retain {
                let _ = crate::memory::arc::inc_ref(&mut self.heap, exc_addr);
                retained_flight_ref = true;
            }
        }
        let exc_flight_addr = if retained_flight_ref {
            exc_val.as_ref().ok().filter(|addr| *addr > 0)
        } else {
            exc_val.as_ref().ok().filter(|addr| *addr > 0)
        };

        loop {
            if self.call_frames.is_empty() {
                let message = self.format_exception_message(exc_val);
                if let Some(exc_addr) = exc_flight_addr {
                    let _ = self.release_reference(exc_addr);
                }
                return Err(VmTrap::from_error(RuntimeError::UserException { message }));
            }

            let (func_id, cur_pc) = {
                let frame = self.current_frame_fast()?;
                (frame.function_id, frame.pc.saturating_sub(1))
            };

            let func = self
                .context
                .get_function(func_id)
                .ok_or_else(|| function_not_found_error(func_id))?;

            let mut matched_handler: Option<solix_common::bytecode::ExceptionHandlerEntry> = None;
            let mut matched_span_len: u32 = u32::MAX;
            let mut matched_is_typed = false;

            for handler in &func.exception_handlers {
                if cur_pc >= handler.try_start_pc && cur_pc < handler.try_end_pc {
                    let span_len = handler.try_end_pc.saturating_sub(handler.try_start_pc);
                    let (is_match, is_typed) = match &handler.catch_type_symbol {
                        None => (true, false),
                        Some(type_symbol) => {
                            let typed_match = if let Value::Ref(object_address) = exc_val
                                && object_address > 0
                            {
                                let object_header = self.heap.read_header(object_address).ok();
                                if let Some(header) = object_header
                                    && header.tag == BlockTag::String
                                {
                                    if let Ok(string_content) =
                                        self.heap.read_string(object_address)
                                    {
                                        if string_content.starts_with("AssertionError:") {
                                            type_symbol == "AssertionError"
                                                || type_symbol == "Exception"
                                                || type_symbol == "RuntimeException"
                                                || type_symbol.ends_with(".AssertionError")
                                                || type_symbol.ends_with(".Exception")
                                                || type_symbol.ends_with(".RuntimeException")
                                                || type_symbol == "string"
                                                || type_symbol == "std::string"
                                                || type_symbol == "std.string"
                                        } else {
                                            type_symbol == "string"
                                                || type_symbol == "std::string"
                                                || type_symbol == "std.string"
                                                || type_symbol == "solix.core.String"
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    let object_vtable_id =
                                        self.heap.read_u32(object_address).unwrap_or(0);
                                    if let Some(target_class) =
                                        self.context.all_classes.iter().find(|candidate| {
                                            candidate.symbol == *type_symbol
                                                || candidate
                                                    .symbol
                                                    .ends_with(&format!(".{type_symbol}"))
                                                || candidate
                                                    .symbol
                                                    .ends_with(&format!("::{type_symbol}"))
                                        })
                                    {
                                        self.is_instance_of(
                                            object_vtable_id,
                                            target_class.vtable_id,
                                        )
                                    } else {
                                        false
                                    }
                                }
                            } else {
                                false
                            };
                            (typed_match, true)
                        }
                    };
                    if is_match {
                        // Always pick the innermost (narrowest) enclosing handler scope first;
                        // within the same scope span, prefer an explicit typed `catch` over `<any>` finally.
                        let is_better = matched_handler.is_none()
                            || span_len < matched_span_len
                            || (span_len == matched_span_len && is_typed && !matched_is_typed);
                        if is_better {
                            matched_handler = Some(handler.clone());
                            matched_span_len = span_len;
                            matched_is_typed = is_typed;
                        }
                    }
                }
            }

            if let Some(handler) = matched_handler {
                let exception_address = if let Value::Ref(target_addr) = exc_val {
                    target_addr
                } else {
                    0
                };
                let curr_frame = self.current_frame_fast()?;
                let curr_reg_base = curr_frame.register_base as usize;
                for &register_index in &handler.try_ref_regs {
                    if register_index != handler.catch_reg {
                        let reg_val = self.register_stack[curr_reg_base + register_index as usize];
                        if let Value::Ref(object_addr) = reg_val
                            && object_addr > 0
                            && object_addr != exception_address
                        {
                            let _ = self.release_reference(object_addr);
                            self.register_stack[curr_reg_base + register_index as usize] =
                                Value::Null;
                        }
                    }
                }

                self.register_stack[curr_reg_base + handler.catch_reg as usize] = exc_val;
                let frame = self.current_frame_mut_fast()?;
                frame.pc = handler.handler_pc;
                return Ok(());
            }

            let curr_frame = self.current_frame_fast()?;
            let curr_reg_base = curr_frame.register_base as usize;
            let ref_regs = func.reference_registers.clone();
            for ref_reg in ref_regs {
                let reg_val = self.register_stack[curr_reg_base + ref_reg as usize];
                if let Value::Ref(addr) = reg_val
                    && addr > 0
                {
                    let _ = self.release_reference(addr);
                }
            }

            let _frame = self.call_frames.pop().ok_or_else(VmTrap::null_reference)?;

            if self.call_frames.len() <= target_depth {
                let message = self.format_exception_message(exc_val);
                if let Some(exc_addr) = exc_flight_addr {
                    let _ = self.release_reference(exc_addr);
                }
                return Err(VmTrap::from_error(RuntimeError::UserException { message }));
            }
        }
    }

    #[inline(never)]
    fn clear_frame_reference_registers(
        register_stack: &mut [Value],
        ref_regs: &[u16],
        reg_base: usize,
        arg_count: usize,
        num_regs: usize,
    ) {
        for &ref_reg in ref_regs {
            let reg_index = ref_reg as usize;
            if reg_index >= arg_count && reg_index < num_regs {
                // SAFETY: caller ensures reg_base + num_regs <= register_stack.len().
                unsafe {
                    *register_stack.as_mut_ptr().add(reg_base + reg_index) = Value::Null;
                }
            }
        }
    }

    /// Out-of-line frame activation handling stack growth, reference register clearing, and frame push.
    #[inline(never)]
    fn push_frame_window(
        &mut self,
        function_id: u32,
        caller_dst: u16,
        caller_arg_start: usize,
        arg_count: usize,
    ) -> Result<(*const Instruction, usize, usize, *mut Value, usize, u32), VmTrap> {
        if self.call_frames.len() >= self.context.options.max_call_stack_depth {
            return Err(stack_overflow_error(
                self.call_frames.len(),
                self.context.options.max_call_stack_depth,
            ));
        }

        let func_index = function_id as usize;
        let code_span = self
            .func_code
            .get(func_index)
            .copied()
            .ok_or_else(|| function_not_found_error(function_id))?;
        let binary_id = code_span.binary_id as usize;
        let has_references = code_span.has_references;
        let inst_ptr = code_span.inst_ptr;
        let inst_len = code_span.inst_len;
        let num_regs = (code_span.num_registers as usize).max(arg_count);

        let caller_frame_end = self.call_frames.last().map_or(0, |last_frame| {
            last_frame.register_base as usize + last_frame.register_count as usize
        });
        // Slide the register window in-place only when the argument slice sits at the top of
        // the caller's live register frame; otherwise allocate above caller_frame_end and copy.
        let (reg_base, needs_arg_copy) =
            if arg_count > 0 && caller_arg_start + arg_count >= caller_frame_end {
                (caller_arg_start, false)
            } else {
                (caller_frame_end, arg_count > 0)
            };
        let required_regs = reg_base + num_regs;

        if required_regs > self.register_stack.len() {
            grow_register_stack(&mut self.register_stack, required_regs);
        }

        if needs_arg_copy {
            // SAFETY: caller_arg_start..caller_arg_start + arg_count lies within caller frame,
            // reg_base == caller_frame_end >= caller_arg_start + arg_count (non-overlapping),
            // and reg_base + arg_count <= required_regs <= self.register_stack.len().
            unsafe {
                let stack_ptr = self.register_stack.as_mut_ptr();
                if arg_count == 1 {
                    *stack_ptr.add(reg_base) = *stack_ptr.add(caller_arg_start);
                } else if arg_count == 2 {
                    *stack_ptr.add(reg_base) = *stack_ptr.add(caller_arg_start);
                    *stack_ptr.add(reg_base + 1) = *stack_ptr.add(caller_arg_start + 1);
                } else {
                    std::ptr::copy_nonoverlapping(
                        stack_ptr.add(caller_arg_start),
                        stack_ptr.add(reg_base),
                        arg_count,
                    );
                }
            }
        }

        if has_references {
            let ref_regs = &self.context.all_functions[func_index].reference_registers;
            Self::clear_frame_reference_registers(
                &mut self.register_stack,
                ref_regs,
                reg_base,
                arg_count,
                num_regs,
            );
        }

        let frame = CallFrame::new(
            function_id,
            binary_id,
            reg_base,
            num_regs,
            inst_ptr,
            caller_dst,
        );

        self.call_frames.push(frame);
        if self.call_frames.len() > self.peak_call_stack_depth {
            self.peak_call_stack_depth = self.call_frames.len();
        }
        // SAFETY: reg_base + num_regs <= self.register_stack.len() was ensured above.
        let reg_base_ptr = unsafe { self.register_stack.as_mut_ptr().add(reg_base) };
        Ok((
            inst_ptr,
            inst_len,
            0,
            reg_base_ptr,
            binary_id,
            code_span.static_base_offset,
        ))
    }

    #[cold]
    #[inline(never)]
    fn dispatch_call_slow(
        &mut self,
        global_func_id: u32,
        caller_dst: u16,
        caller_arg_start: usize,
        arg_count: usize,
    ) -> Result<(*const Instruction, usize, usize, *mut Value, usize, u32), VmTrap> {
        let Some(span) = self.func_code.get(global_func_id as usize).copied() else {
            return Err(function_not_found_error(global_func_id));
        };
        if span.is_native {
            return self.invoke_native_call_id(
                global_func_id,
                caller_dst,
                caller_arg_start,
                arg_count,
            );
        }
        self.push_frame_window(global_func_id, caller_dst, caller_arg_start, arg_count)
    }

    /// High-performance instruction dispatch loop using raw-pointer caching and local register windows.
    ///
    /// # Errors
    /// Returns `RuntimeError` on guest traps, type mismatches, division by zero, or stack overflow.
    pub fn run_fast(&mut self, target_depth: usize) -> Result<(), RuntimeError> {
        self.run_fast_internal::<false>(target_depth)
    }

    /// Single-instruction step through the virtual machine using the unified engine loop.
    ///
    /// # Errors
    /// Returns `RuntimeError` on guest traps, type mismatches, division by zero, or stack overflow.
    pub fn run_fast_step(&mut self) -> Result<(), RuntimeError> {
        self.run_fast_internal::<true>(0)
    }

    /// High-performance instruction dispatch loop using raw-pointer caching and local register windows.
    ///
    /// # Errors
    /// Returns `RuntimeError` on guest traps, type mismatches, division by zero, or stack overflow.
    pub(crate) fn run_fast_internal<const SINGLE_STEP: bool>(
        &mut self,
        target_depth: usize,
    ) -> Result<(), RuntimeError> {
        if SINGLE_STEP {
            self.run_fast_trap::<true, true>(target_depth)
                .map_err(VmTrap::into_inner)
        } else if self.track_metrics {
            self.run_fast_trap::<false, true>(target_depth)
                .map_err(VmTrap::into_inner)
        } else {
            self.run_fast_trap::<false, false>(target_depth)
                .map_err(VmTrap::into_inner)
        }
    }

    fn run_fast_trap<const SINGLE_STEP: bool, const TRACK_METRICS: bool>(
        &mut self,
        target_depth: usize,
    ) -> Result<(), VmTrap> {
        if self.halted || (!SINGLE_STEP && self.call_frames.len() <= target_depth) {
            return Ok(());
        }
        if SINGLE_STEP && self.call_frames.is_empty() {
            return Ok(());
        }

        let mut inst_ptr: *const Instruction;
        let mut inst_len: usize = 0;
        let mut pc: usize;
        let mut reg_base: *mut Value;
        let mut current_binary_id: usize;
        let mut current_static_base: u32;
        let mut local_inst_count: u64 = 0;

        macro_rules! reload_frame {
            () => {
                let curr = self.current_frame_fast()?;
                inst_ptr = curr.inst_ptr;
                current_binary_id = curr.binary_id as usize;
                current_static_base = self.func_code[curr.function_id as usize].static_base_offset;
                if SINGLE_STEP {
                    inst_len = self.func_code[curr.function_id as usize].inst_len;
                }
                pc = curr.pc as usize;
                let curr_reg_base = curr.register_base as usize;
                // SAFETY: curr.register_base is guaranteed within register_stack bounds
                reg_base = unsafe { self.register_stack.as_mut_ptr().add(curr_reg_base) };
            };
        }

        macro_rules! finish_instruction {
            () => {
                if SINGLE_STEP {
                    if let Some(frame) = self.call_frames.last_mut() {
                        frame.pc = pc as u32;
                    }
                    if TRACK_METRICS && local_inst_count > 0 {
                        self.instructions_dispatched += local_inst_count;
                    }
                    return Ok(());
                }
            };
        }

        macro_rules! flush_inst_count {
            () => {
                if TRACK_METRICS && local_inst_count > 0 {
                    self.instructions_dispatched += local_inst_count;
                    local_inst_count = 0;
                }
            };
        }

        macro_rules! pop_and_return_value {
            ($ret_reg:expr, $loop_lbl:lifetime) => {
                let prev_reg_base = reg_base;
                let frame = self.call_frames.pop().ok_or_else(VmTrap::null_reference)?;
                if self.call_frames.len() <= target_depth {
                    self.last_return_value = match $ret_reg {
                        Some(src_idx) => unsafe { *prev_reg_base.add(src_idx) },
                        None => Value::Null,
                    };
                    break $loop_lbl;
                }
                let caller_frame = self.current_frame_fast()?;
                inst_ptr = caller_frame.inst_ptr;
                current_binary_id = caller_frame.binary_id as usize;
                current_static_base = self.func_code[caller_frame.function_id as usize].static_base_offset;
                if SINGLE_STEP {
                    inst_len = self.func_code[caller_frame.function_id as usize].inst_len;
                }
                pc = caller_frame.pc as usize;
                let caller_reg_offset = caller_frame.register_base as usize;
                // SAFETY: prev_reg_base + src_idx and caller_reg_base + caller_dst are within register_stack.
                unsafe {
                    let caller_reg_base = self.register_stack.as_mut_ptr().add(caller_reg_offset);
                    let ret_val = match $ret_reg {
                        Some(src_idx) => *prev_reg_base.add(src_idx),
                        None => Value::Null,
                    };
                    *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                    reg_base = caller_reg_base;
                }
                finish_instruction!();
                continue $loop_lbl;
            };
        }

        reload_frame!();

        'dispatch_loop: loop {
            if SINGLE_STEP && pc >= inst_len {
                pop_and_return_value!(None::<usize>, 'dispatch_loop);
            }

            // SAFETY: pc < verified length + sentinel Return, so inst_ptr.add(pc) is in-bounds
            let inst = unsafe { *inst_ptr.add(pc) };
            pc += 1;
            if TRACK_METRICS {
                local_inst_count += 1;
            }

            let raw_opcode = inst.raw_opcode();
            let Some(opcode) = OpCode::from_raw_u8(raw_opcode) else {
                return Err(invalid_opcode_error(pc as u32, raw_opcode));
            };
            let (call_func_id, call_closure_addr): (u32, u32) = 'call_dispatch: {
                match opcode {
                    // =============================================================
                    // 0x00 .. 0x07: Constants & Register Movement
                    // =============================================================
                    OpCode::Nop => {}

                    OpCode::Mov => {
                        // SAFETY: Register operands are verified by compiler within frame window
                        unsafe {
                            let val = *reg_base.add(inst.src1() as usize);
                            *reg_base.add(inst.dst() as usize) = val;
                        }
                    }

                    OpCode::LoadI32 => {
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(inst.imm32_signed());
                        }
                    }

                    OpCode::LoadI64 => {
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I64(inst.imm32_signed() as i64);
                        }
                    }

                    OpCode::LoadBool => {
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(inst.imm32() != 0);
                        }
                    }

                    OpCode::LoadNull => {
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Null;
                        }
                    }

                    OpCode::LoadF64 => {
                        let idx = inst.imm32() as usize;
                        let val = self.const_pool_cache[current_binary_id][idx];
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = val;
                        }
                    }

                    OpCode::LoadConst => {
                        let pool_idx = inst.imm32() as usize;
                        let const_val = self.const_pool_cache[current_binary_id][pool_idx];
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = const_val;
                        }
                    }

                    // =============================================================
                    // 0x10 .. 0x20: Arithmetic Operations
                    // =============================================================
                    OpCode::AddI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I32(val_a.wrapping_add(val_b));
                        }
                    }

                    OpCode::AddI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I64(val_a.wrapping_add(val_b));
                        }
                    }

                    OpCode::AddF64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::F64(left_val), Value::F64(right_val)) =
                            (*left_ref, *right_ref)
                        {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::F64(left_val + right_val);
                            }
                        } else {
                            return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                        }
                    }

                    OpCode::SubI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I32(val_a.wrapping_sub(val_b));
                        }
                    }

                    OpCode::SubI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I64(val_a.wrapping_sub(val_b));
                        }
                    }

                    OpCode::SubF64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::F64(left_val), Value::F64(right_val)) =
                            (*left_ref, *right_ref)
                        {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::F64(left_val - right_val);
                            }
                        } else {
                            return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                        }
                    }

                    OpCode::MulI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I32(val_a.wrapping_mul(val_b));
                        }
                    }

                    OpCode::MulI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I64(val_a.wrapping_mul(val_b));
                        }
                    }

                    OpCode::MulF64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::F64(left_val), Value::F64(right_val)) =
                            (*left_ref, *right_ref)
                        {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::F64(left_val * right_val);
                            }
                        } else {
                            return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                        }
                    }

                    OpCode::DivI32 => {
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_i32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let quotient = div_i32_cold(val_a, val_b)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(quotient);
                        }
                    }

                    OpCode::DivI64 => {
                        let val_a = read_i64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_i64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let quotient = div_i64_cold(val_a, val_b)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I64(quotient);
                        }
                    }

                    OpCode::DivF64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::F64(left_val), Value::F64(right_val)) =
                            (*left_ref, *right_ref)
                        {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::F64(left_val / right_val);
                            }
                        } else {
                            return Err(f64_pair_type_mismatch_error(left_ref, right_ref));
                        }
                    }

                    OpCode::ModI32 => {
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_i32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let rem_val = rem_i32_cold(val_a, val_b)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(rem_val);
                        }
                    }

                    OpCode::ModI64 => {
                        let val_a = read_i64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_i64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let rem_val = rem_i64_cold(val_a, val_b)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I64(rem_val);
                        }
                    }

                    OpCode::DivU32 => {
                        let dividend =
                            read_u32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let divisor =
                            read_u32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let quotient = div_u32_cold(dividend, divisor)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(quotient as i32);
                        }
                    }

                    OpCode::DivU64 => {
                        let dividend =
                            read_u64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let divisor =
                            read_u64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let quotient = div_u64_cold(dividend, divisor)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I64(quotient as i64);
                        }
                    }

                    OpCode::ModU32 => {
                        let dividend =
                            read_u32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let divisor =
                            read_u32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let remainder_value = rem_u32_cold(dividend, divisor)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(remainder_value as i32);
                        }
                    }

                    OpCode::ModU64 => {
                        let dividend =
                            read_u64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let divisor =
                            read_u64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let remainder_value = rem_u64_cold(dividend, divisor)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I64(remainder_value as i64);
                        }
                    }

                    OpCode::NegI32 => {
                        let value = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(value.wrapping_neg());
                        }
                    }

                    OpCode::NegI64 => {
                        let value = read_i64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I64(value.wrapping_neg());
                        }
                    }

                    OpCode::NegF64 => {
                        let val_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        if let Value::F64(float_val) = *val_ref {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::F64(-float_val);
                            }
                        } else {
                            return Err(f64_type_mismatch_error(val_ref));
                        }
                    }

                    // =============================================================
                    // 0x21 .. 0x26: Bitwise Operations
                    // =============================================================
                    OpCode::Band => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::I32(lhs & rhs);
                            }
                        } else {
                            let result = band_slow(left_ref, right_ref)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = result;
                            }
                        }
                    }

                    OpCode::Bor => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::I32(lhs | rhs);
                            }
                        } else {
                            let result = bor_slow(left_ref, right_ref)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = result;
                            }
                        }
                    }

                    OpCode::Bxor => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::I32(lhs ^ rhs);
                            }
                        } else {
                            let result = bxor_slow(left_ref, right_ref)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = result;
                            }
                        }
                    }

                    OpCode::Bnot => {
                        let val_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        if let Value::I32(lhs) = *val_ref {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::I32(!lhs);
                            }
                        } else {
                            let result = bnot_slow(val_ref)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = result;
                            }
                        }
                    }

                    OpCode::Shl => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                            if rhs < 0 {
                                return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
                                    operation: "bitwise shift left",
                                    reason: "negative shift count is invalid".to_string(),
                                }));
                            }
                            let shift = (rhs & 31) as u32;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::I32(lhs.wrapping_shl(shift));
                            }
                        } else {
                            let result = shl_slow(left_ref, right_ref)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = result;
                            }
                        }
                    }

                    OpCode::Shr => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                            if rhs < 0 {
                                return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
                                    operation: "bitwise shift right",
                                    reason: "negative shift count is invalid".to_string(),
                                }));
                            }
                            let shift = (rhs & 31) as u32;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::I32(lhs.wrapping_shr(shift));
                            }
                        } else {
                            let result = shr_slow(left_ref, right_ref)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = result;
                            }
                        }
                    }

                    OpCode::Lshr => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                            if rhs < 0 {
                                return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
                                    operation: "logical right shift",
                                    reason: "negative shift count is invalid".to_string(),
                                }));
                            }
                            let shift = (rhs & 31) as u32;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::I32(((lhs as u32).wrapping_shr(shift)) as i32);
                            }
                        } else {
                            let right_signed = read_i64_fast(right_ref)?;
                            if right_signed < 0 {
                                return Err(VmTrap::from_error(RuntimeError::InvalidOperand {
                                    operation: "logical right shift",
                                    reason: "negative shift count is invalid".to_string(),
                                }));
                            }
                            let val_a = read_u64_fast(left_ref)?;
                            let shift = (right_signed & 63) as u32;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::I64(val_a.wrapping_shr(shift) as i64);
                            }
                        }
                    }

                    // =============================================================
                    // 0x30 .. 0x47: Comparisons
                    // =============================================================
                    OpCode::EqI32 => {
                        let first_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let second_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let is_equal = if let (Value::I32(first_num), Value::I32(second_num)) =
                            (*first_ref, *second_ref)
                        {
                            first_num == second_num
                        } else {
                            values_equal_slow(first_ref, second_ref)
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(is_equal);
                        }
                    }

                    OpCode::EqI64 => {
                        let val_a = read_i64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_i64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a == val_b);
                        }
                    }

                    OpCode::EqF64 => {
                        let val_a = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_f64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a == val_b);
                        }
                    }

                    OpCode::EqBool => {
                        let val_a =
                            read_bool_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b =
                            read_bool_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a == val_b);
                        }
                    }

                    OpCode::EqString => {
                        let addr_left =
                            read_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let addr_right =
                            read_ref_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let is_equal = if addr_left == addr_right {
                            true
                        } else if addr_left == 0 || addr_right == 0 {
                            false
                        } else {
                            self.compare_heap_strings(addr_left, addr_right)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(is_equal);
                        }
                    }

                    OpCode::EqRef => {
                        let first_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let second_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let is_equal = match (*first_ref, *second_ref) {
                            (Value::Ref(first_addr), Value::Ref(second_addr)) => {
                                first_addr == second_addr
                            }
                            (Value::Null, Value::Null) => true,
                            (Value::Null, Value::Ref(addr)) | (Value::Ref(addr), Value::Null) => {
                                addr == 0
                            }
                            _ => values_equal_slow(first_ref, second_ref),
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(is_equal);
                        }
                    }

                    OpCode::NeqI32 => {
                        let first_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let second_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let is_equal = if let (Value::I32(first_num), Value::I32(second_num)) =
                            (*first_ref, *second_ref)
                        {
                            first_num == second_num
                        } else {
                            values_equal_slow(first_ref, second_ref)
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(!is_equal);
                        }
                    }

                    OpCode::NeqI64 => {
                        let val_a = read_i64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_i64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a != val_b);
                        }
                    }

                    OpCode::NeqF64 => {
                        let val_a = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_f64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a != val_b);
                        }
                    }

                    OpCode::NeqBool => {
                        let val_a =
                            read_bool_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b =
                            read_bool_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a != val_b);
                        }
                    }

                    OpCode::NeqString => {
                        let addr_left =
                            read_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let addr_right =
                            read_ref_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        let is_equal = if addr_left == addr_right {
                            true
                        } else if addr_left == 0 || addr_right == 0 {
                            false
                        } else {
                            self.compare_heap_strings(addr_left, addr_right)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(!is_equal);
                        }
                    }

                    OpCode::NeqRef => {
                        let first_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let second_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let is_equal = match (*first_ref, *second_ref) {
                            (Value::Ref(first_addr), Value::Ref(second_addr)) => {
                                first_addr == second_addr
                            }
                            (Value::Null, Value::Null) => true,
                            (Value::Null, Value::Ref(addr)) | (Value::Ref(addr), Value::Null) => {
                                addr == 0
                            }
                            _ => values_equal_slow(first_ref, second_ref),
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(!is_equal);
                        }
                    }

                    OpCode::LtI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                        }
                    }

                    OpCode::LtI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                        }
                    }

                    OpCode::LtF64 => {
                        let val_a = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_f64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                        }
                    }

                    OpCode::LteI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                        }
                    }

                    OpCode::LteI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                        }
                    }

                    OpCode::LteF64 => {
                        let val_a = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_f64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                        }
                    }

                    OpCode::GtI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                        }
                    }

                    OpCode::GtI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                        }
                    }

                    OpCode::GtF64 => {
                        let val_a = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_f64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                        }
                    }

                    OpCode::GteI32 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                        }
                    }

                    OpCode::GteI64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i64_pair_slow(left_ref, right_ref)?
                            };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                        }
                    }

                    OpCode::GteF64 => {
                        let val_a = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_f64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                        }
                    }

                    OpCode::LtU32 => {
                        let val_a = read_u32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                        }
                    }

                    OpCode::LteU32 => {
                        let val_a = read_u32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                        }
                    }

                    OpCode::GtU32 => {
                        let val_a = read_u32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                        }
                    }

                    OpCode::GteU32 => {
                        let val_a = read_u32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u32_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                        }
                    }

                    OpCode::LtU64 => {
                        let val_a = read_u64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                        }
                    }

                    OpCode::LteU64 => {
                        let val_a = read_u64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                        }
                    }

                    OpCode::GtU64 => {
                        let val_a = read_u64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                        }
                    }

                    OpCode::GteU64 => {
                        let val_a = read_u64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let val_b = read_u64_fast(unsafe { &*reg_base.add(inst.src2() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                        }
                    }

                    OpCode::Not => {
                        let cond =
                            read_truthy_fast(unsafe { &*reg_base.add(inst.src1() as usize) });
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(!cond);
                        }
                    }

                    OpCode::ConvIToF => {
                        let val_n = read_i64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(val_n as f64);
                        }
                    }

                    OpCode::ConvFToI => {
                        let val_f = read_f64_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I64(val_f as i64);
                        }
                    }

                    // =============================================================
                    // 0x50 .. 0x53: Control Flow & Branching
                    // =============================================================
                    OpCode::Jump => {
                        let offset = inst.jump_offset();
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }

                    OpCode::Switch => {
                        let offset = inst.jump_offset();
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }

                    OpCode::JumpIfTrue => {
                        let cond =
                            read_truthy_fast(unsafe { &*reg_base.add(inst.cond_reg() as usize) });
                        if cond {
                            let offset = inst.jump_offset();
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfFalse => {
                        let cond =
                            read_truthy_fast(unsafe { &*reg_base.add(inst.cond_reg() as usize) });
                        if !cond {
                            let offset = inst.jump_offset();
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    // =============================================================
                    // 0x60 .. 0x69: Function Invocations, Returns & Exit
                    // =============================================================
                    OpCode::CallDirect => {
                        break 'call_dispatch (inst.call_target() as u32, 0);
                    }

                    OpCode::TailCallDirect => {
                        let call_func_id = inst.call_target() as u32;
                        let call_arg_base = inst.arg_base() as usize;
                        let call_arg_count = inst.arg_count() as usize;
                        let span = *self
                            .func_code
                            .get(call_func_id as usize)
                            .ok_or_else(|| function_not_found_error(call_func_id))?;
                        if !SINGLE_STEP
                            && !span.is_native
                            && !span.has_references
                            && span.trivial_leaf_kind == 0
                        {
                            let reg_stack_len = self.register_stack.len();
                            let caller_frame = self.current_frame_mut_fast()?;
                            let caller_reg_base = caller_frame.register_base as usize;
                            let num_regs = (span.num_registers as usize).max(call_arg_count);
                            if caller_reg_base + num_regs <= reg_stack_len {
                                match call_arg_count {
                                    0 => {}
                                    1 => unsafe {
                                        *reg_base = *reg_base.add(call_arg_base);
                                    },
                                    2 => unsafe {
                                        let arg0 = *reg_base.add(call_arg_base);
                                        let arg1 = *reg_base.add(call_arg_base + 1);
                                        *reg_base = arg0;
                                        *reg_base.add(1) = arg1;
                                    },
                                    _ => unsafe {
                                        std::ptr::copy(
                                            reg_base.add(call_arg_base),
                                            reg_base,
                                            call_arg_count,
                                        );
                                    },
                                }
                                caller_frame.function_id = call_func_id;
                                caller_frame.binary_id = span.binary_id;
                                caller_frame.register_count = num_regs as u32;
                                caller_frame.inst_ptr = span.inst_ptr;
                                caller_frame.active_closure = None;
                                inst_ptr = span.inst_ptr;
                                current_binary_id = span.binary_id as usize;
                                current_static_base = span.static_base_offset;
                                pc = 0;
                                continue 'dispatch_loop;
                            }
                        }
                        break 'call_dispatch (call_func_id, 0);
                    }

                    OpCode::CallNativeDirect => {
                        let global_func_id = inst.call_target() as u32;
                        let arg_base = inst.arg_base() as usize;
                        let arg_count = inst.arg_count() as usize;
                        let caller_dst = inst.call_dst();

                        let caller_frame = self.current_frame_mut_fast()?;
                        caller_frame.pc = pc as u32;
                        let caller_arg_start = caller_frame.register_base as usize + arg_base;

                        flush_inst_count!();
                        (
                            inst_ptr,
                            inst_len,
                            pc,
                            reg_base,
                            current_binary_id,
                            current_static_base,
                        ) = self.invoke_native_call_id(
                            global_func_id,
                            caller_dst,
                            caller_arg_start,
                            arg_count,
                        )?;
                        finish_instruction!();
                        continue 'dispatch_loop;
                    }

                    OpCode::CallVirtual => {
                        let raw_target = inst.call_target();
                        let is_polymorphic = (raw_target & 0x8000) != 0;
                        let slot = (raw_target & 0x7FFF) as usize;
                        let arg_base = inst.arg_base() as usize;
                        let obj_addr = read_nonnull_ref_fast(unsafe { &*reg_base.add(arg_base) })?;
                        let vtable_id = read_heap_u32_fast(&self.heap.buffer, obj_addr)?;
                        let global_func_id = match self
                            .context
                            .flat_vtables
                            .get(vtable_id as usize)
                            .and_then(|slots| slots.get(slot))
                            .copied()
                        {
                            Some(func_id) => func_id,
                            None => {
                                if let Some(frame) = self.call_frames.last_mut() {
                                    frame.pc = pc as u32;
                                }
                                return Err(self.resolve_virtual_slot_error(vtable_id, slot));
                            }
                        };
                        if !is_polymorphic
                            && let Ok(target_func_u16) = u16::try_from(global_func_id)
                        {
                            let caller_func_id =
                                self.call_frames.last().map(|frame| frame.function_id);
                            if let Some(func_id) = caller_func_id {
                                let caller_pc = pc.saturating_sub(1);
                                self.monomorphic_call_caches.insert(
                                    (func_id, caller_pc as u32),
                                    MonomorphicCallSite {
                                        expected_vtable_id: vtable_id,
                                        original_slot: slot as u16,
                                        target_func_id: global_func_id,
                                    },
                                );
                                let quickened = Instruction::format_call(
                                    OpCode::CallVirtualDirect,
                                    inst.call_dst(),
                                    target_func_u16,
                                    inst.arg_base(),
                                    inst.arg_count(),
                                );
                                self.patch_instruction(func_id, caller_pc, quickened);
                            }
                        }
                        break 'call_dispatch (global_func_id, 0);
                    }

                    OpCode::CallVirtualDirect => {
                        let arg_base = inst.arg_base() as usize;
                        let obj_addr = read_nonnull_ref_fast(unsafe { &*reg_base.add(arg_base) })?;
                        let vtable_id = read_heap_u32_fast(&self.heap.buffer, obj_addr)?;
                        let caller_func_id = self.call_frames.last().map(|frame| frame.function_id);
                        let caller_pc = pc.saturating_sub(1);
                        let cached_site = caller_func_id.and_then(|func_id| {
                            self.monomorphic_call_caches
                                .get(&(func_id, caller_pc as u32))
                                .copied()
                        });

                        if let (Some(site), Some(func_id)) = (cached_site, caller_func_id) {
                            if site.expected_vtable_id == vtable_id {
                                break 'call_dispatch (site.target_func_id, 0);
                            }
                            let unquickened = Instruction::format_call(
                                OpCode::CallVirtual,
                                inst.call_dst(),
                                site.original_slot | 0x8000,
                                inst.arg_base(),
                                inst.arg_count(),
                            );
                            self.patch_instruction(func_id, caller_pc, unquickened);
                            let slot = site.original_slot as usize;
                            let global_func_id = match self
                                .context
                                .flat_vtables
                                .get(vtable_id as usize)
                                .and_then(|slots| slots.get(slot))
                                .copied()
                            {
                                Some(resolved_func_id) => resolved_func_id,
                                None => {
                                    if let Some(frame) = self.call_frames.last_mut() {
                                        frame.pc = pc as u32;
                                    }
                                    return Err(self.resolve_virtual_slot_error(vtable_id, slot));
                                }
                            };
                            break 'call_dispatch (global_func_id, 0);
                        } else {
                            let target_func_id = inst.call_target() as u32;
                            break 'call_dispatch (target_func_id, 0);
                        }
                    }

                    OpCode::CallInterface => {
                        let target = inst.call_target();
                        let iface_id = (target >> 8) as usize;
                        let slot = (target & 0xFF) as usize;
                        let arg_base = inst.arg_base() as usize;
                        let obj_addr = read_nonnull_ref_fast(unsafe { &*reg_base.add(arg_base) })?;
                        let vtable_id = read_heap_u32_fast(&self.heap.buffer, obj_addr)?;
                        let direct_func_id = if iface_id != 0 {
                            self.context
                                .flat_itables
                                .get(vtable_id as usize)
                                .and_then(|iface_rows| iface_rows.get(iface_id))
                                .and_then(|slots| slots.get(slot))
                                .copied()
                        } else {
                            None
                        };
                        let global_func_id = match direct_func_id {
                            Some(func_id) => func_id,
                            None => {
                                if let Some(frame) = self.call_frames.last_mut() {
                                    frame.pc = pc as u32;
                                }
                                self.resolve_interface_slot_slow(vtable_id, iface_id as u32, slot)?
                            }
                        };
                        break 'call_dispatch (global_func_id, 0);
                    }

                    OpCode::CallSymbol => {
                        flush_inst_count!();
                        self.execute_call_symbol(inst, pc)?;
                        reload_frame!();
                        finish_instruction!();
                        continue 'dispatch_loop;
                    }

                    OpCode::CallExtern => {
                        flush_inst_count!();
                        self.execute_call_extern(inst, pc)?;
                        reload_frame!();
                        finish_instruction!();
                        continue 'dispatch_loop;
                    }

                    OpCode::CallCallable => {
                        let r_callee = inst.call_target();
                        let callable_ref = unsafe { &*reg_base.add(r_callee as usize) };
                        match *callable_ref {
                            Value::Null => return Err(VmTrap::null_reference()),
                            Value::FnPtr(global_func_id) => {
                                break 'call_dispatch (global_func_id, 0);
                            }
                            Value::Ref(closure_addr) => {
                                if closure_addr == 0 {
                                    return Err(VmTrap::null_reference());
                                }
                                let global_func_id =
                                    read_heap_u32_fast(&self.heap.buffer, closure_addr)?;
                                break 'call_dispatch (global_func_id, closure_addr);
                            }
                            _ => {
                                return Err(non_callable_type_mismatch_error(callable_ref));
                            }
                        }
                    }

                    OpCode::Return => {
                        pop_and_return_value!(None::<usize>, 'dispatch_loop);
                    }

                    OpCode::ReturnValue => {
                        pop_and_return_value!(Some(inst.dst() as usize), 'dispatch_loop);
                    }

                    OpCode::Exit => {
                        let code = read_i32_fast(unsafe { &*reg_base.add(inst.dst() as usize) })?;
                        self.exit_code = Some(code);
                        self.halted = true;
                        break 'dispatch_loop;
                    }

                    // =============================================================
                    // 0x70 .. 0x78: Object Model, Instance Fields & Static Memory
                    // =============================================================
                    OpCode::AllocObject => {
                        let vtable_id = inst.src1() as u32;
                        let size = inst.src2() as usize;
                        let tag = if inst.is_pod() {
                            BlockTag::ObjectPod
                        } else {
                            BlockTag::Object
                        };
                        let payload_addr = alloc_object_trap(&mut self.heap, size, tag, vtable_id)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                        }
                    }

                    OpCode::AllocObjectExtern => {
                        let obj_ref = self.execute_alloc_object_extern(inst, pc)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = obj_ref;
                        }
                    }

                    OpCode::GetField => {
                        let obj_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let offset = inst.src2() as usize;
                        let field_addr = obj_addr + offset as u32;
                        let raw_flags = inst.flags();
                        let dst_idx = inst.dst() as usize;
                        if (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) == 0
                            && raw_flags > 1
                        {
                            match raw_flags >> 1 {
                                5 | 6 => unsafe {
                                    *reg_base.add(dst_idx) = Value::I32(read_heap_u32_or_zero(
                                        &self.heap.buffer,
                                        field_addr,
                                    )
                                        as i32);
                                },
                                7 | 8 => unsafe {
                                    *reg_base.add(dst_idx) = Value::I64(read_heap_u64_or_zero(
                                        &self.heap.buffer,
                                        field_addr,
                                    )
                                        as i64);
                                },
                                10 => unsafe {
                                    *reg_base.add(dst_idx) = Value::F64(f64::from_bits(
                                        read_heap_u64_or_zero(&self.heap.buffer, field_addr),
                                    ));
                                },
                                12 => {
                                    let raw = read_heap_u64_or_zero(&self.heap.buffer, field_addr);
                                    unsafe {
                                        if raw == 0 || raw == Value::TAG_NULL {
                                            *reg_base.add(dst_idx) = Value::Null;
                                        } else if (raw & Value::TAG_MASK) == Value::TAG_REF {
                                            *reg_base.add(dst_idx) = Value::Ref(raw as u32);
                                        } else if (raw & Value::TAG_MASK) == 0 {
                                            *reg_base.add(dst_idx) =
                                                Value::Ref((raw & 0xFFFF_FFFF) as u32);
                                        } else {
                                            *reg_base.add(dst_idx) = Value::from_raw_u64(raw);
                                        }
                                    }
                                }
                                _ => {
                                    let val =
                                        self.execute_get_field_slow(obj_addr, offset, raw_flags)?;
                                    unsafe {
                                        *reg_base.add(dst_idx) = val;
                                    }
                                }
                            }
                        } else {
                            let val = self.execute_get_field_slow(obj_addr, offset, raw_flags)?;
                            unsafe {
                                *reg_base.add(dst_idx) = val;
                            }
                        }
                    }

                    OpCode::SetField => {
                        let obj_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let offset = inst.src2() as usize;
                        let val_idx = inst.dst() as usize;
                        let val_ref = unsafe { &*reg_base.add(val_idx) };
                        let field_addr = obj_addr + offset as u32;
                        let raw_flags = inst.flags();
                        let is_move =
                            (raw_flags & solix_common::bytecode::instruction::FLAG_MOVE) != 0;
                        let field_tag =
                            (raw_flags & !solix_common::bytecode::instruction::FLAG_MOVE) >> 1;
                        if (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) == 0
                            && field_tag > 0
                        {
                            match field_tag {
                                5 | 6 => {
                                    write_heap_u32_fast(
                                        &mut self.heap.buffer,
                                        field_addr,
                                        read_i32_fast(val_ref)? as u32,
                                    );
                                }
                                7 | 8 => {
                                    write_heap_u64_fast(
                                        &mut self.heap.buffer,
                                        field_addr,
                                        read_i64_fast(val_ref)? as u64,
                                    );
                                }
                                10 => {
                                    let bits = read_f64_fast(val_ref)?.to_bits();
                                    write_heap_u64_fast(&mut self.heap.buffer, field_addr, bits);
                                }
                                12 => {
                                    let old_raw =
                                        read_heap_u64_or_zero(&self.heap.buffer, field_addr);
                                    if old_raw == 0 || old_raw == Value::TAG_NULL {
                                        let new_val = *val_ref;
                                        if is_move {
                                            unsafe {
                                                *reg_base.add(val_idx) = Value::Null;
                                            }
                                            write_heap_u64_fast(
                                                &mut self.heap.buffer,
                                                field_addr,
                                                new_val.to_raw_u64(),
                                            );
                                        } else {
                                            write_heap_u64_fast(
                                                &mut self.heap.buffer,
                                                field_addr,
                                                new_val.to_raw_u64(),
                                            );
                                            if let Value::Ref(child_addr) = new_val
                                                && child_addr > 0
                                            {
                                                inc_ref_inline(
                                                    &mut self.heap,
                                                    child_addr,
                                                    self.cycle_collector_enabled,
                                                )?;
                                            }
                                        }
                                    } else {
                                        let new_val = *val_ref;
                                        if is_move {
                                            unsafe {
                                                *reg_base.add(val_idx) = Value::Null;
                                            }
                                        }
                                        if self.execute_set_field_slow(
                                            obj_addr, offset, raw_flags, new_val, pc,
                                        )? {
                                            reload_frame!();
                                        }
                                    }
                                }
                                _ => {
                                    let new_val = *val_ref;
                                    if is_move {
                                        unsafe {
                                            *reg_base.add(val_idx) = Value::Null;
                                        }
                                    }
                                    if self.execute_set_field_slow(
                                        obj_addr, offset, raw_flags, new_val, pc,
                                    )? {
                                        reload_frame!();
                                    }
                                }
                            }
                        } else {
                            let new_val = *val_ref;
                            if is_move {
                                unsafe {
                                    *reg_base.add(val_idx) = Value::Null;
                                }
                            }
                            if self
                                .execute_set_field_slow(obj_addr, offset, raw_flags, new_val, pc)?
                            {
                                reload_frame!();
                            }
                        }
                    }

                    OpCode::GetStaticDirect => {
                        let slot = inst.imm32() as usize;
                        if slot >= self.static_slots.len() {
                            return Err(VmTrap::invalid_static_offset(
                                slot as u32,
                                self.static_slots.len() as u32,
                            ));
                        }
                        // SAFETY: Slot is verified in bounds
                        let static_val = unsafe { *self.static_slots.get_unchecked(slot) };
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = static_val;
                        }
                    }

                    OpCode::GetStatic => {
                        let slot = (current_static_base + inst.imm32()) as usize;
                        if slot >= self.static_slots.len() {
                            return Err(VmTrap::invalid_static_offset(
                                slot as u32,
                                self.static_slots.len() as u32,
                            ));
                        }
                        // SAFETY: Slot is verified in bounds
                        let static_val = unsafe { *self.static_slots.get_unchecked(slot) };
                        // SAFETY: Destination register is within frame window
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = static_val;
                        }
                    }

                    OpCode::SetStatic => {
                        let slot_idx = current_static_base + inst.imm32();
                        let val_to_set = unsafe { *reg_base.add(inst.dst() as usize) };
                        self.set_static_trap(slot_idx, val_to_set)?;
                    }

                    OpCode::GetStaticGlobal => {
                        let static_val = self.execute_get_static_global(inst, pc)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = static_val;
                        }
                    }

                    OpCode::SetStaticGlobal => {
                        let val_to_set = unsafe { *reg_base.add(inst.dst() as usize) };
                        self.execute_set_static_global(inst, pc, val_to_set)?;
                    }

                    OpCode::SetStaticDirect => {
                        let slot = inst.imm32();
                        let val = unsafe { *reg_base.add(inst.dst() as usize) };
                        self.set_static_trap(slot, val)?;
                    }

                    // =============================================================
                    // 0x80 .. 0x85 & 0xA4: Dynamic Arrays & Tuples
                    // =============================================================
                    OpCode::AllocArray => {
                        let raw_len =
                            read_array_index_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        if raw_len < 0 {
                            return Err(VmTrap::from_error(RuntimeError::IndexOutOfBounds {
                                index: raw_len,
                                length: 0,
                            }));
                        }
                        let array_len = raw_len as usize;
                        let type_tag = inst.flags();
                        let (elem_size, vtable_id) = if type_tag >= 0x80 {
                            (8, inst.src2())
                        } else {
                            ((inst.src2() as usize).max(1), 0)
                        };
                        let payload_addr = alloc_array_trap(
                            &mut self.heap,
                            array_len,
                            elem_size,
                            type_tag,
                            vtable_id,
                        )?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                        }
                    }

                    OpCode::ArrayLength => {
                        let arr_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let array_len = array_length_trap(&self.heap, arr_addr)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(array_len as i32);
                        }
                    }

                    OpCode::GetArray => {
                        let arr_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let arr_addr = if let Value::Ref(addr) = *arr_ref
                            && addr > 0
                        {
                            addr
                        } else {
                            read_nonnull_ref_slow(arr_ref)?
                        };
                        let idx_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let index_i64 = if let Value::I32(num_val) = *idx_ref {
                            num_val as i64
                        } else {
                            read_array_index_slow(idx_ref)?
                        };
                        let arr_offset = arr_addr as usize;
                        let heap_len = self.heap.buffer.len();
                        if arr_offset < 8 || arr_offset + 8 > heap_len {
                            return Err(VmTrap::null_reference());
                        }

                        // SAFETY: arr_offset >= 8 and arr_offset + 8 <= heap_len verified above.
                        let buf_ptr = self.heap.buffer.as_ptr();
                        let tag_nibble = unsafe { (*buf_ptr.add(arr_offset - 1) >> 4) & 0x0F };
                        let dst_idx = inst.dst() as usize;
                        if tag_nibble == BlockTag::ArrayRef as u8 {
                            let header_qword = unsafe {
                                u64::from_le(std::ptr::read_unaligned(
                                    buf_ptr.add(arr_offset).cast::<u64>(),
                                ))
                            };
                            let array_len = (header_qword as u32) as usize;
                            if (index_i64 as u64) >= (array_len as u64) {
                                return Err(VmTrap::index_out_of_bounds(index_i64, array_len));
                            }
                            let index_usize = index_i64 as usize;
                            let meta_masked = ((header_qword >> 32) as u32) & 0xFF00_00FF;
                            const META_BOOL_1B: u32 = (11 << 24) | 1;
                            const META_I32_4B: u32 = (3 << 24) | 4;
                            if meta_masked == META_BOOL_1B {
                                let elem_offset = arr_offset + 8 + index_usize;
                                if elem_offset + 1 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                let flag = unsafe { *buf_ptr.add(elem_offset) } != 0;
                                unsafe {
                                    *reg_base.add(dst_idx) = Value::Bool(flag);
                                }
                            } else if meta_masked == META_I32_4B {
                                let elem_offset = arr_offset + 8 + (index_usize << 2);
                                if elem_offset + 4 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                let raw = unsafe {
                                    u32::from_le(std::ptr::read_unaligned(
                                        buf_ptr.add(elem_offset).cast::<u32>(),
                                    ))
                                };
                                unsafe {
                                    *reg_base.add(dst_idx) = Value::I32(raw as i32);
                                }
                            } else {
                                let elem_size_raw = (meta_masked & 0xFF) as u8;
                                let elem_type_tag = (meta_masked >> 24) as u8;
                                if elem_size_raw == 8
                                    && (elem_type_tag >= 13
                                        || elem_type_tag == 10
                                        || elem_type_tag == 0)
                                {
                                    let elem_offset = arr_offset + 8 + (index_usize << 3);
                                    if elem_offset + 8 > heap_len {
                                        return Err(VmTrap::null_reference());
                                    }
                                    let raw = unsafe {
                                        u64::from_le(std::ptr::read_unaligned(
                                            buf_ptr.add(elem_offset).cast::<u64>(),
                                        ))
                                    };
                                    unsafe {
                                        if elem_type_tag >= 13 {
                                            if raw == 0 {
                                                *reg_base.add(dst_idx) = Value::Null;
                                            } else {
                                                *reg_base.add(dst_idx) = Value::Ref(raw as u32);
                                            }
                                        } else if elem_type_tag == 10 {
                                            *reg_base.add(dst_idx) =
                                                Value::F64(f64::from_bits(raw));
                                        } else {
                                            *reg_base.add(dst_idx) = Value::from_raw_u64(raw);
                                        }
                                    }
                                } else {
                                    let val = get_array_variant_cold(
                                        &self.heap,
                                        arr_offset,
                                        index_usize,
                                        elem_size_raw,
                                        elem_type_tag,
                                    )?;
                                    unsafe {
                                        *reg_base.add(dst_idx) = val;
                                    }
                                }
                            }
                        } else {
                            let val =
                                self.execute_get_array_special(arr_offset, index_i64, tag_nibble)?;
                            unsafe {
                                *reg_base.add(dst_idx) = val;
                            }
                        }
                    }

                    OpCode::SetArray => {
                        let arr_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let arr_addr = if let Value::Ref(addr) = *arr_ref
                            && addr > 0
                        {
                            addr
                        } else {
                            read_nonnull_ref_slow(arr_ref)?
                        };
                        let idx_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let index_i64 = if let Value::I32(num_val) = *idx_ref {
                            num_val as i64
                        } else {
                            read_array_index_slow(idx_ref)?
                        };
                        let arr_offset = arr_addr as usize;
                        let heap_len = self.heap.buffer.len();
                        if arr_offset < 8 || arr_offset + 8 > heap_len {
                            return Err(VmTrap::null_reference());
                        }

                        // SAFETY: arr_offset >= 8 and arr_offset + 8 <= heap_len verified above.
                        let buf_ptr = self.heap.buffer.as_mut_ptr();
                        let tag_nibble = unsafe { (*buf_ptr.add(arr_offset - 1) >> 4) & 0x0F };
                        let header_qword = unsafe {
                            u64::from_le(std::ptr::read_unaligned(
                                buf_ptr.add(arr_offset).cast::<u64>(),
                            ))
                        };
                        let array_len = (header_qword as u32) as usize;
                        if (index_i64 as u64) >= (array_len as u64) {
                            return Err(VmTrap::index_out_of_bounds(index_i64, array_len));
                        }
                        let index_usize = index_i64 as usize;

                        if tag_nibble == BlockTag::ArrayRef as u8 {
                            let meta_masked = ((header_qword >> 32) as u32) & 0xFF00_00FF;
                            let val_idx = inst.src2() as usize;
                            let val_ref = unsafe { &*reg_base.add(val_idx) };
                            const META_BOOL_1B: u32 = (11 << 24) | 1;
                            const META_I32_4B: u32 = (3 << 24) | 4;
                            const META_F64_8B: u32 = (10 << 24) | 8;
                            if meta_masked == META_BOOL_1B {
                                let elem_offset = arr_offset + 8 + index_usize;
                                if elem_offset + 1 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                let byte_val = match *val_ref {
                                    Value::Bool(flag) => u8::from(flag),
                                    Value::I32(num_val) => num_val as u8,
                                    _ => to_primitive_u64_cold(val_ref) as u8,
                                };
                                unsafe {
                                    *buf_ptr.add(elem_offset) = byte_val;
                                }
                            } else if meta_masked == META_I32_4B {
                                let elem_offset = arr_offset + 8 + (index_usize << 2);
                                if elem_offset + 4 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                let word_val = match *val_ref {
                                    Value::I32(num_val) => num_val as u32,
                                    _ => to_primitive_u64_cold(val_ref) as u32,
                                };
                                unsafe {
                                    std::ptr::write_unaligned(
                                        buf_ptr.add(elem_offset).cast::<u32>(),
                                        word_val.to_le(),
                                    );
                                }
                            } else if meta_masked == META_F64_8B {
                                let elem_offset = arr_offset + 8 + (index_usize << 3);
                                if elem_offset + 8 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                let bits = read_f64_fast(val_ref)?.to_bits();
                                unsafe {
                                    std::ptr::write_unaligned(
                                        buf_ptr.add(elem_offset).cast::<u64>(),
                                        bits.to_le(),
                                    );
                                }
                            } else {
                                let elem_size_raw = (meta_masked & 0xFF) as u8;
                                let elem_type_tag = (meta_masked >> 24) as u8;
                                if (elem_type_tag == 13 || elem_type_tag >= 0x80)
                                    && elem_size_raw == 8
                                    && (inst.flags()
                                        & solix_common::bytecode::instruction::FLAG_MOVE)
                                        != 0
                                {
                                    let elem_offset = arr_offset + 8 + (index_usize << 3);
                                    if elem_offset + 8 > heap_len {
                                        return Err(VmTrap::null_reference());
                                    }
                                    let old_child = unsafe {
                                        u32::from_le(std::ptr::read_unaligned(
                                            buf_ptr.add(elem_offset).cast::<u32>(),
                                        ))
                                    };
                                    let is_valid_store = if (elem_type_tag & 0x80) == 0 {
                                        true
                                    } else {
                                        let target_vtable_id = (elem_type_tag & 0x7F) as u32;
                                        if let Value::Ref(assigned_address) = *val_ref
                                            && assigned_address > 0
                                        {
                                            if let Ok(assigned_vtable_id) =
                                                self.heap.read_u32(assigned_address)
                                            {
                                                assigned_vtable_id == target_vtable_id
                                                    || self.is_instance_of(
                                                        assigned_vtable_id,
                                                        target_vtable_id,
                                                    )
                                            } else {
                                                false
                                            }
                                        } else {
                                            true
                                        }
                                    };
                                    if old_child == 0 && is_valid_store {
                                        let new_val = *val_ref;
                                        unsafe {
                                            *reg_base.add(val_idx) = Value::Null;
                                            std::ptr::write_unaligned(
                                                buf_ptr.add(elem_offset).cast::<u64>(),
                                                new_val.to_primitive_u64().to_le(),
                                            );
                                        }
                                    } else if self.execute_set_array_slow(
                                        arr_offset,
                                        index_usize,
                                        inst.src2(),
                                        inst.flags(),
                                        pc,
                                    )? {
                                        reload_frame!();
                                    }
                                } else if self.execute_set_array_slow(
                                    arr_offset,
                                    index_usize,
                                    inst.src2(),
                                    inst.flags(),
                                    pc,
                                )? {
                                    reload_frame!();
                                }
                            }
                        } else if tag_nibble == BlockTag::Tuple as u8 {
                            let val_idx = inst.src2() as usize;
                            let is_move = (inst.flags()
                                & solix_common::bytecode::instruction::FLAG_MOVE)
                                != 0;
                            let val = unsafe { *reg_base.add(val_idx) };
                            if is_move {
                                unsafe {
                                    *reg_base.add(val_idx) = Value::Null;
                                }
                            }
                            if self.execute_set_tuple(arr_offset, index_usize, val, is_move)? {
                                let caller_frame = self.current_frame_mut_fast()?;
                                caller_frame.pc = pc as u32;
                                reload_frame!();
                            }
                        } else if self.execute_set_array_slow(
                            arr_offset,
                            index_usize,
                            inst.src2(),
                            inst.flags(),
                            pc,
                        )? {
                            reload_frame!();
                        }
                    }

                    OpCode::AllocTuple => {
                        let tuple_ref = self.execute_alloc_tuple(inst.src1() as usize)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = tuple_ref;
                        }
                    }

                    OpCode::GetTuple => {
                        let tuple_val = unsafe { *reg_base.add(inst.src1() as usize) };
                        let elem_val = self.execute_get_tuple(tuple_val, inst.src2() as usize)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = elem_val;
                        }
                    }

                    OpCode::TupleInstanceof => {
                        let tuple_val = unsafe { *reg_base.add(inst.src1() as usize) };
                        let index_val = unsafe { *reg_base.add(inst.src2() as usize) };
                        let is_match =
                            self.execute_tuple_instanceof(tuple_val, index_val, inst.flags())?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(is_match);
                        }
                    }

                    // =============================================================
                    // 0x90 .. 0x91: Closures & Captures
                    // =============================================================
                    OpCode::PackClosure => {
                        let target_lambda_id = inst.call_target() as u32;
                        let cap_base = inst.arg_base() as usize;
                        let cap_count = inst.arg_count() as usize;
                        let caller_dst = inst.call_dst() as usize;

                        let closure_val = if cap_count == 0 {
                            // Stateless closure: encode directly as a tagged FnPtr with zero heap allocation.
                            Value::FnPtr(target_lambda_id)
                        } else {
                            self.execute_pack_closure_heap(target_lambda_id, cap_base, cap_count)?
                        };
                        unsafe {
                            *reg_base.add(caller_dst) = closure_val;
                        }
                    }

                    OpCode::UnpackCaptures => {
                        let capture_count = inst.arg_count() as usize;
                        if capture_count > 0 {
                            let capture_base = inst.arg_base() as usize;
                            self.execute_unpack_captures(capture_base, capture_count)?;
                        }
                    }

                    // =============================================================
                    // 0xA0 .. 0xA1: Instanceof & Cast
                    // =============================================================
                    OpCode::Instanceof => {
                        let obj_val = unsafe { *reg_base.add(inst.src1() as usize) };
                        let is_instance = self.execute_instanceof(obj_val, inst.src2() as u32)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Bool(is_instance);
                        }
                    }

                    OpCode::CastCheck => {
                        let val = unsafe { *reg_base.add(inst.src1() as usize) };
                        let converted =
                            self.execute_cast_check(val, inst.flags(), inst.src2() as u32)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = converted;
                        }
                    }

                    // =============================================================
                    // 0xC0 .. 0xC1: ARC Reference Counting
                    // =============================================================
                    OpCode::IncRef => {
                        if let Value::Ref(addr) = unsafe { *reg_base.add(inst.dst() as usize) }
                            && addr > 0
                        {
                            inc_ref_inline(&mut self.heap, addr, self.cycle_collector_enabled)?;
                        }
                    }

                    OpCode::DecRef => {
                        if let Value::Ref(addr) = unsafe { *reg_base.add(inst.dst() as usize) }
                            && addr > 0
                        {
                            let addr_usize = addr as usize;
                            if addr_usize < 8 || addr_usize > self.heap.buffer.len() {
                                return Err(VmTrap::null_reference());
                            }
                            // SAFETY: addr_usize >= 8 && addr_usize <= heap.buffer.len().
                            let word_ptr = unsafe {
                                self.heap
                                    .buffer
                                    .as_mut_ptr()
                                    .add(addr_usize - 4)
                                    .cast::<u32>()
                            };
                            let tag_and_ref =
                                unsafe { u32::from_le(std::ptr::read_unaligned(word_ptr)) };
                            let ref_count = tag_and_ref & crate::memory::layout::MAX_REF_COUNT;
                            let tag_nibble = ((tag_and_ref >> 28) & 0x0F) as u8;
                            if ref_count == crate::memory::layout::MAX_REF_COUNT {
                                // Immortal constant string or immortal object
                            } else if ref_count > 1 {
                                let decremented = (tag_and_ref
                                    & !crate::memory::layout::MAX_REF_COUNT)
                                    | (ref_count - 1);
                                unsafe {
                                    std::ptr::write_unaligned(word_ptr, decremented.to_le());
                                }
                                if self.cycle_collector_enabled {
                                    self.maybe_push_cycle_candidate(
                                        addr,
                                        tag_and_ref,
                                        tag_nibble,
                                        decremented,
                                    )?;
                                }
                            } else if tag_nibble == BlockTag::ObjectPod as u8 {
                                let payload_size = unsafe {
                                    u32::from_le(std::ptr::read_unaligned(
                                        self.heap.buffer.as_ptr().add(addr_usize - 8).cast::<u32>(),
                                    ))
                                };
                                self.heap.deallocate_pod_fast(addr, payload_size)?;
                            } else {
                                let caller_frame = self.current_frame_mut_fast()?;
                                caller_frame.pc = pc as u32;
                                self.release_reference_cold(addr)?;
                                reload_frame!();
                            }
                        }
                    }

                    // =============================================================
                    // 0xD0 .. 0xD9: Superinstructions
                    // =============================================================
                    OpCode::AddI32Imm => {
                        let imm_val = inst.src2() as i16 as i32;
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I32(val_a.wrapping_add(imm_val));
                        }
                    }

                    OpCode::SubI32Imm => {
                        let imm_val = inst.src2() as i16 as i32;
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I32(val_a.wrapping_sub(imm_val));
                        }
                    }

                    OpCode::MulI32Imm => {
                        let imm_val = inst.src2() as i16 as i32;
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I32(val_a.wrapping_mul(imm_val));
                        }
                    }

                    OpCode::ModI32Imm => {
                        let imm_val = inst.src2() as i16 as i32;
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        if val_a >= 0 && imm_val > 0 && imm_val < 1024 {
                            let rem_val = if (imm_val & (imm_val - 1)) == 0 {
                                val_a & (imm_val - 1)
                            } else {
                                let (magic, shift) = MOD_MAGIC_TABLE[imm_val as usize];
                                let quotient =
                                    (((val_a as u64).wrapping_mul(magic)) >> shift) as i32;
                                val_a - quotient.wrapping_mul(imm_val)
                            };
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::I32(rem_val);
                            }
                        } else {
                            let rem_val = rem_i32_cold(val_a, imm_val)?;
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::I32(rem_val);
                            }
                        }
                    }

                    OpCode::JumpIfLtI32 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        if val_a < val_b {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfLteI32 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        if val_a <= val_b {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfGtI32 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        if val_a > val_b {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfGteI32 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_a, val_b) =
                            if let (Value::I32(lhs), Value::I32(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                read_i32_pair_slow(left_ref, right_ref)?
                            };
                        if val_a >= val_b {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfEqI32 => {
                        let target_offset = inst.src2() as i16 as i32;
                        let first_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let second_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let is_equal = if let (Value::I32(first_num), Value::I32(second_num)) =
                            (*first_ref, *second_ref)
                        {
                            first_num == second_num
                        } else {
                            values_equal_slow(first_ref, second_ref)
                        };
                        if is_equal {
                            pc = ((pc as i32 - 1) + target_offset) as usize;
                        }
                    }

                    OpCode::JumpIfNeqI32 => {
                        let target_offset = inst.src2() as i16 as i32;
                        let first_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let second_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let is_equal = if let (Value::I32(first_num), Value::I32(second_num)) =
                            (*first_ref, *second_ref)
                        {
                            first_num == second_num
                        } else {
                            values_equal_slow(first_ref, second_ref)
                        };
                        if !is_equal {
                            pc = ((pc as i32 - 1) + target_offset) as usize;
                        }
                    }

                    OpCode::ForLoopI32 => {
                        let idx_slot = unsafe { &mut *reg_base.add(inst.dst() as usize) };
                        let bound_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let flags = inst.flags();
                        let step = if (flags & 0x80) == 0 {
                            1
                        } else {
                            read_i32_fast(unsafe { &*reg_base.add((flags & 0x1F) as usize) })?
                        };
                        let cur_idx = if let Value::I32(cur) = *idx_slot {
                            cur
                        } else {
                            read_i32_fast(idx_slot)?
                        };
                        let next_idx = cur_idx.wrapping_add(step);
                        *idx_slot = Value::I32(next_idx);
                        let bound = if let Value::I32(bound_val) = *bound_ref {
                            bound_val
                        } else {
                            read_i32_fast(bound_ref)?
                        };
                        let next_i64 = i64::from(cur_idx) + i64::from(step);
                        let bound_i64 = i64::from(bound);
                        let keep_going = if (flags & 0x40) == 0 {
                            next_i64 < bound_i64
                        } else {
                            next_i64 <= bound_i64
                        };
                        if keep_going {
                            if !SINGLE_STEP
                                && (flags & 0x30) != 0
                                && let Some(done_steps) =
                                    try_complete_special_for_loop::<TRACK_METRICS>(
                                        &mut self.heap.buffer,
                                        reg_base,
                                        inst_ptr,
                                        pc,
                                        inst,
                                        next_idx,
                                        bound,
                                    )?
                            {
                                if TRACK_METRICS {
                                    local_inst_count += done_steps;
                                }
                            } else {
                                let offset = inst.src2() as i16 as i32;
                                pc = ((pc as i32 - 1) + offset) as usize;
                            }
                        }
                    }

                    OpCode::AddBandI32 => {
                        let acc_ref = unsafe { &*reg_base.add(inst.flags() as usize) };
                        let val_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let mask_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let res = if let (Value::I32(acc), Value::I32(val), Value::I32(mask)) =
                            (*acc_ref, *val_ref, *mask_ref)
                        {
                            acc.wrapping_add(val) & mask
                        } else {
                            read_i32_fast(acc_ref)?.wrapping_add(read_i32_fast(val_ref)?)
                                & read_i32_fast(mask_ref)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(res);
                        }
                    }

                    OpCode::JumpIfI32Imm => {
                        let offset = inst.src2() as i16 as i32;
                        let imm_val = inst.src1() as i16 as i32;
                        let val_a = read_i32_fast(unsafe { &*reg_base.add(inst.dst() as usize) })?;
                        let take_branch = match inst.flags() {
                            0 => val_a == imm_val,
                            1 => val_a != imm_val,
                            2 => val_a < imm_val,
                            3 => val_a <= imm_val,
                            4 => val_a > imm_val,
                            _ => val_a >= imm_val,
                        };
                        if take_branch {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::MulAddF64 => {
                        let left_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src2() as usize) };
                        let acc_ref = unsafe { &*reg_base.add(inst.flags() as usize) };
                        if let (Value::F64(left_val), Value::F64(right_val), Value::F64(acc_val)) =
                            (*left_ref, *right_ref, *acc_ref)
                        {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) =
                                    Value::F64(left_val * right_val + acc_val);
                            }
                        } else {
                            return Err(mul_add_f64_type_mismatch_cold(
                                left_ref, right_ref, acc_ref,
                            ));
                        }
                    }

                    OpCode::GetArrayJumpIfFalse => {
                        let arr_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let arr_addr = if let Value::Ref(addr) = *arr_ref
                            && addr > 0
                        {
                            addr
                        } else {
                            read_nonnull_ref_slow(arr_ref)?
                        };
                        let idx_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let index_i64 = if let Value::I32(num_val) = *idx_ref {
                            num_val as i64
                        } else {
                            read_array_index_slow(idx_ref)?
                        };
                        let arr_offset = arr_addr as usize;
                        let heap_len = self.heap.buffer.len();
                        if arr_offset < 8 || arr_offset + 8 > heap_len {
                            return Err(VmTrap::null_reference());
                        }
                        let buf_ptr = self.heap.buffer.as_ptr();
                        let tag_nibble = unsafe { (*buf_ptr.add(arr_offset - 1) >> 4) & 0x0F };
                        let elem_truthy = if tag_nibble == BlockTag::ArrayRef as u8 {
                            let header_qword = unsafe {
                                u64::from_le(std::ptr::read_unaligned(
                                    buf_ptr.add(arr_offset).cast::<u64>(),
                                ))
                            };
                            let array_len = (header_qword as u32) as usize;
                            if (index_i64 as u64) >= (array_len as u64) {
                                return Err(VmTrap::index_out_of_bounds(index_i64, array_len));
                            }
                            let index_usize = index_i64 as usize;
                            let meta_masked = ((header_qword >> 32) as u32) & 0xFF00_00FF;
                            const META_BOOL_1B: u32 = (11 << 24) | 1;
                            if meta_masked == META_BOOL_1B {
                                let elem_offset = arr_offset + 8 + index_usize;
                                if elem_offset + 1 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                unsafe { *buf_ptr.add(elem_offset) != 0 }
                            } else {
                                let elem_size_raw = (meta_masked & 0xFF) as u8;
                                let elem_type_tag = (meta_masked >> 24) as u8;
                                let val = get_array_variant_cold(
                                    &self.heap,
                                    arr_offset,
                                    index_usize,
                                    elem_size_raw,
                                    elem_type_tag,
                                )?;
                                read_truthy_fast(&val)
                            }
                        } else {
                            let val =
                                self.execute_get_array_special(arr_offset, index_i64, tag_nibble)?;
                            read_truthy_fast(&val)
                        };
                        let take_branch = if inst.flags() == 0 {
                            !elem_truthy
                        } else {
                            elem_truthy
                        };
                        if take_branch {
                            let offset = inst.src2() as i16 as i32;
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::GetArrayJumpIfTrue => {
                        let arr_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let arr_addr = if let Value::Ref(addr) = *arr_ref
                            && addr > 0
                        {
                            addr
                        } else {
                            read_nonnull_ref_slow(arr_ref)?
                        };
                        let idx_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let index_i64 = if let Value::I32(num_val) = *idx_ref {
                            num_val as i64
                        } else {
                            read_array_index_slow(idx_ref)?
                        };
                        let arr_offset = arr_addr as usize;
                        let heap_len = self.heap.buffer.len();
                        if arr_offset < 8 || arr_offset + 8 > heap_len {
                            return Err(VmTrap::null_reference());
                        }
                        let buf_ptr = self.heap.buffer.as_ptr();
                        let tag_nibble = unsafe { (*buf_ptr.add(arr_offset - 1) >> 4) & 0x0F };
                        let elem_truthy = if tag_nibble == BlockTag::ArrayRef as u8 {
                            let header_qword = unsafe {
                                u64::from_le(std::ptr::read_unaligned(
                                    buf_ptr.add(arr_offset).cast::<u64>(),
                                ))
                            };
                            let array_len = (header_qword as u32) as usize;
                            if (index_i64 as u64) >= (array_len as u64) {
                                return Err(VmTrap::index_out_of_bounds(index_i64, array_len));
                            }
                            let index_usize = index_i64 as usize;
                            let meta_masked = ((header_qword >> 32) as u32) & 0xFF00_00FF;
                            const META_BOOL_1B: u32 = (11 << 24) | 1;
                            if meta_masked == META_BOOL_1B {
                                let elem_offset = arr_offset + 8 + index_usize;
                                if elem_offset + 1 > heap_len {
                                    return Err(VmTrap::null_reference());
                                }
                                unsafe { *buf_ptr.add(elem_offset) != 0 }
                            } else {
                                let elem_size_raw = (meta_masked & 0xFF) as u8;
                                let elem_type_tag = (meta_masked >> 24) as u8;
                                let val = get_array_variant_cold(
                                    &self.heap,
                                    arr_offset,
                                    index_usize,
                                    elem_size_raw,
                                    elem_type_tag,
                                )?;
                                read_truthy_fast(&val)
                            }
                        } else {
                            let val =
                                self.execute_get_array_special(arr_offset, index_i64, tag_nibble)?;
                            read_truthy_fast(&val)
                        };
                        let take_branch = if inst.flags() == 0 {
                            elem_truthy
                        } else {
                            !elem_truthy
                        };
                        if take_branch {
                            let offset = inst.src2() as i16 as i32;
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::AddF64Imm => {
                        let const_index = inst.src2() as usize;
                        let const_val = self.const_pool_cache[current_binary_id][const_index];
                        let const_num = match const_val {
                            Value::F64(num) => num,
                            _ => read_f64_fast(&const_val)?,
                        };
                        let src_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let src_num = if let Value::F64(num) = *src_ref {
                            num
                        } else {
                            read_f64_fast(src_ref)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(src_num + const_num);
                        }
                    }

                    OpCode::SubF64Imm => {
                        let const_index = inst.src2() as usize;
                        let const_val = self.const_pool_cache[current_binary_id][const_index];
                        let const_num = match const_val {
                            Value::F64(num) => num,
                            _ => read_f64_fast(&const_val)?,
                        };
                        let src_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let src_num = if let Value::F64(num) = *src_ref {
                            num
                        } else {
                            read_f64_fast(src_ref)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(src_num - const_num);
                        }
                    }

                    OpCode::MulF64Imm => {
                        let const_index = inst.src2() as usize;
                        let const_val = self.const_pool_cache[current_binary_id][const_index];
                        let const_num = match const_val {
                            Value::F64(num) => num,
                            _ => read_f64_fast(&const_val)?,
                        };
                        let src_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let src_num = if let Value::F64(num) = *src_ref {
                            num
                        } else {
                            read_f64_fast(src_ref)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(src_num * const_num);
                        }
                    }

                    OpCode::JumpIfLtF64 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_a, val_b) =
                            if let (Value::F64(lhs), Value::F64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                (read_f64_fast(left_ref)?, read_f64_fast(right_ref)?)
                            };
                        if val_a < val_b {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfLteF64 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_a, val_b) =
                            if let (Value::F64(lhs), Value::F64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                (read_f64_fast(left_ref)?, read_f64_fast(right_ref)?)
                            };
                        if val_a <= val_b {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::AddI64Imm => {
                        let imm_val = inst.src2() as i16 as i64;
                        let src_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let val_num = if let Value::I64(num) = *src_ref {
                            num
                        } else {
                            read_i64_fast(src_ref)?
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::I64(val_num.wrapping_add(imm_val));
                        }
                    }

                    OpCode::JumpIfLtI64 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_lhs, val_rhs) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                (read_i64_fast(left_ref)?, read_i64_fast(right_ref)?)
                            };
                        if val_lhs < val_rhs {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::JumpIfLteI64 => {
                        let offset = inst.src2() as i16 as i32;
                        let left_ref = unsafe { &*reg_base.add(inst.dst() as usize) };
                        let right_ref = unsafe { &*reg_base.add(inst.src1() as usize) };
                        let (val_lhs, val_rhs) =
                            if let (Value::I64(lhs), Value::I64(rhs)) = (*left_ref, *right_ref) {
                                (lhs, rhs)
                            } else {
                                (read_i64_fast(left_ref)?, read_i64_fast(right_ref)?)
                            };
                        if val_lhs <= val_rhs {
                            pc = ((pc as i32 - 1) + offset) as usize;
                        }
                    }

                    OpCode::GetFieldI32 => {
                        let obj_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let offset = inst.src2() as usize;
                        let field_addr = obj_addr + offset as u32;
                        let dst_idx = inst.dst() as usize;
                        unsafe {
                            *reg_base.add(dst_idx) =
                                Value::I32(
                                    read_heap_u32_or_zero(&self.heap.buffer, field_addr) as i32
                                );
                        }
                    }

                    OpCode::GetFieldF64 => {
                        let obj_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let offset = inst.src2() as usize;
                        let field_addr = obj_addr + offset as u32;
                        let dst_idx = inst.dst() as usize;
                        unsafe {
                            *reg_base.add(dst_idx) = Value::F64(f64::from_bits(
                                read_heap_u64_or_zero(&self.heap.buffer, field_addr),
                            ));
                        }
                    }

                    OpCode::SetFieldI32 => {
                        let obj_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let offset = inst.src2() as usize;
                        let val_idx = inst.dst() as usize;
                        let val_ref = unsafe { &*reg_base.add(val_idx) };
                        let field_addr = obj_addr + offset as u32;
                        write_heap_u32_fast(
                            &mut self.heap.buffer,
                            field_addr,
                            read_i32_fast(val_ref)? as u32,
                        );
                    }

                    OpCode::SetFieldF64 => {
                        let obj_addr =
                            read_nonnull_ref_fast(unsafe { &*reg_base.add(inst.src1() as usize) })?;
                        let offset = inst.src2() as usize;
                        let val_idx = inst.dst() as usize;
                        let val_ref = unsafe { &*reg_base.add(val_idx) };
                        let field_addr = obj_addr + offset as u32;
                        let val_bits = read_f64_fast(val_ref)?.to_bits();
                        write_heap_u64_fast(&mut self.heap.buffer, field_addr, val_bits);
                    }

                    // =============================================================
                    // 0xB0 .. 0xBF: Exception Handling
                    // =============================================================
                    OpCode::Throw => {
                        flush_inst_count!();
                        let caller_frame = self.current_frame_mut_fast()?;
                        caller_frame.pc = pc as u32;
                        let exc_reg = inst.dst();
                        let exc_val = unsafe { *reg_base.add(exc_reg as usize) };
                        self.execute_throw(exc_val, Some(exc_reg), target_depth)?;
                        reload_frame!();
                        finish_instruction!();
                        continue 'dispatch_loop;
                    }
                }
                finish_instruction!();
                continue 'dispatch_loop;
            };

            let call_dst = inst.call_dst();
            let call_arg_base = inst.arg_base() as usize;
            let call_arg_count = inst.arg_count() as usize;

            let span = *self
                .func_code
                .get(call_func_id as usize)
                .ok_or_else(|| function_not_found_error(call_func_id))?;

            if !SINGLE_STEP
                && span.trivial_leaf_kind != 0
                && (span.trivial_leaf_param as usize) < call_arg_count
                && self.call_frames.len() < self.context.options.max_call_stack_depth
            {
                let arg_val =
                    unsafe { *reg_base.add(call_arg_base + span.trivial_leaf_param as usize) };
                let leaf_done = match span.trivial_leaf_kind {
                    1 => {
                        unsafe {
                            *reg_base.add(call_dst as usize) = arg_val;
                        }
                        if TRACK_METRICS {
                            local_inst_count += 1;
                        }
                        true
                    }
                    2 => {
                        if let Value::I32(num_val) = arg_val {
                            unsafe {
                                *reg_base.add(call_dst as usize) =
                                    Value::I32(num_val.wrapping_add(span.trivial_leaf_imm as i32));
                            }
                            if TRACK_METRICS {
                                local_inst_count += 2;
                            }
                            true
                        } else {
                            false
                        }
                    }
                    3 => {
                        if let Value::I32(num_val) = arg_val {
                            unsafe {
                                *reg_base.add(call_dst as usize) =
                                    Value::I32(num_val.wrapping_sub(span.trivial_leaf_imm as i32));
                            }
                            if TRACK_METRICS {
                                local_inst_count += 2;
                            }
                            true
                        } else {
                            false
                        }
                    }
                    4 => {
                        if let Value::I32(num_val) = arg_val {
                            unsafe {
                                *reg_base.add(call_dst as usize) =
                                    Value::I32(num_val.wrapping_mul(span.trivial_leaf_imm as i32));
                            }
                            if TRACK_METRICS {
                                local_inst_count += 2;
                            }
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if leaf_done {
                    if TRACK_METRICS {
                        self.peak_call_stack_depth =
                            self.peak_call_stack_depth.max(self.call_frames.len() + 1);
                    }
                    continue 'dispatch_loop;
                }
            }

            let caller_frame = self.current_frame_mut_fast()?;
            caller_frame.pc = pc as u32;
            let caller_reg_base = caller_frame.register_base as usize;
            let caller_frame_end = caller_reg_base + caller_frame.register_count as usize;
            let caller_arg_start = caller_reg_base + call_arg_base;

            let num_regs = (span.num_registers as usize).max(call_arg_count);
            let (new_reg_base, needs_arg_copy) =
                if call_arg_count > 0 && caller_arg_start + call_arg_count >= caller_frame_end {
                    (caller_arg_start, false)
                } else {
                    (caller_frame_end, call_arg_count > 0)
                };
            let required_regs = new_reg_base + num_regs;
            let frame_len = self.call_frames.len();

            if !span.is_native
                && !span.has_references
                && required_regs <= self.register_stack.len()
                && call_arg_count <= 2
                && frame_len < self.call_frames.capacity()
                && frame_len < self.context.options.max_call_stack_depth
            {
                if needs_arg_copy {
                    // SAFETY: caller_arg_start..caller_arg_start + call_arg_count is in caller frame
                    // and new_reg_base + call_arg_count <= required_regs <= register_stack.len().
                    unsafe {
                        let stack_ptr = self.register_stack.as_mut_ptr();
                        *stack_ptr.add(new_reg_base) = *stack_ptr.add(caller_arg_start);
                        if call_arg_count == 2 {
                            *stack_ptr.add(new_reg_base + 1) = *stack_ptr.add(caller_arg_start + 1);
                        }
                    }
                }
                // SAFETY: new_reg_base + num_regs <= register_stack.len() and frame_len < call_frames.capacity() checked above.
                reg_base = unsafe { self.register_stack.as_mut_ptr().add(new_reg_base) };
                let mut new_frame = CallFrame::new(
                    call_func_id,
                    span.binary_id as usize,
                    new_reg_base,
                    num_regs,
                    span.inst_ptr,
                    call_dst,
                );
                new_frame.active_closure = std::num::NonZeroU32::new(call_closure_addr);
                let new_len = frame_len + 1;
                // SAFETY: frame_len < self.call_frames.capacity() verified above and CallFrame is Copy.
                unsafe {
                    self.call_frames
                        .as_mut_ptr()
                        .add(frame_len)
                        .write(new_frame);
                    self.call_frames.set_len(new_len);
                }
                if TRACK_METRICS {
                    self.peak_call_stack_depth = self.peak_call_stack_depth.max(new_len);
                }
                inst_ptr = span.inst_ptr;
                inst_len = span.inst_len;
                current_binary_id = span.binary_id as usize;
                current_static_base = span.static_base_offset;
                pc = 0;
            } else {
                if span.is_native {
                    flush_inst_count!();
                }
                (
                    inst_ptr,
                    inst_len,
                    pc,
                    reg_base,
                    current_binary_id,
                    current_static_base,
                ) = self.dispatch_call_slow(
                    call_func_id,
                    call_dst,
                    caller_arg_start,
                    call_arg_count,
                )?;
                if let Some(closure_nonnull) = std::num::NonZeroU32::new(call_closure_addr)
                    && !span.is_native
                    && let Some(new_frame) = self.call_frames.last_mut()
                {
                    new_frame.active_closure = Some(closure_nonnull);
                }
            }
            finish_instruction!();
        }

        if TRACK_METRICS && local_inst_count > 0 {
            self.instructions_dispatched += local_inst_count;
        }

        // Final PC synchronization if still running
        if self.call_frames.len() > target_depth {
            let frame = self.current_frame_mut_fast()?;
            frame.pc = pc as u32;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RuntimeContext;
    use crate::options::RuntimeOptions;
    use solix_common::bytecode::{BinaryProgram, CompiledFunction};

    #[test]
    fn test_positive_bitwise_ops_preserve_i32_variant() {
        let mut program = BinaryProgram::new();
        let mut main_fn = CompiledFunction::new("main", 0, 0, 8);
        main_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 0x0F0F));
        main_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0x00FF));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Band, 0, 2, 0, 1));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Bor, 0, 3, 2, 0));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Bxor, 0, 4, 3, 1));
        main_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 2));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Shl, 0, 6, 4, 5));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Shr, 0, 7, 6, 5));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Bnot, 0, 2, 7, 0));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Bnot, 0, 2, 2, 0));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
        program.functions.push(main_fn);
        program.entry_point_id = Some(0);

        let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
            .expect("context should build");
        let mut vm = VirtualMachine::new(context);
        let result = vm.run().expect("vm execution should succeed");
        assert_eq!(result, Value::I32(0x0FF0));
    }

    #[test]
    fn test_positive_stateless_pack_closure_uses_fnptr_without_heap_alloc() {
        let mut program = BinaryProgram::new();

        let mut lambda_fn = CompiledFunction::new("lambda_zero_cap", 0, 1, 2);
        lambda_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 10));
        lambda_fn
            .instructions
            .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 1));
        lambda_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
        program.functions.push(lambda_fn);

        let mut main_fn = CompiledFunction::new("main", 0, 0, 4);
        main_fn
            .instructions
            .push(Instruction::format_call(OpCode::PackClosure, 0, 0, 0, 0));
        main_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 32));
        main_fn
            .instructions
            .push(Instruction::format_call(OpCode::CallCallable, 3, 0, 2, 1));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 3, 0, 0));
        program.functions.push(main_fn);
        program.entry_point_id = Some(1);

        let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
            .expect("context should build");
        let mut vm = VirtualMachine::new(context);
        let result = vm.run().expect("vm execution should succeed");
        assert_eq!(result, Value::I32(42));
        assert_eq!(
            vm.heap.telemetry().total_allocations_count,
            0,
            "zero-capture PackClosure should not allocate on the heap"
        );
    }

    #[test]
    fn test_negative_invalid_opcode_in_fast_loop_returns_error() {
        let mut program = BinaryProgram::new();
        let mut main_fn = CompiledFunction::new("main", 0, 0, 1);
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
        program.functions.push(main_fn);
        program.entry_point_id = Some(0);

        let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
            .expect("context should build");
        let mut vm = VirtualMachine::new(context);
        vm.patch_instruction(0, 0, Instruction(0x0000_0000_0000_00FE));
        let err = vm
            .run()
            .expect_err("invalid raw opcode should trap cleanly");
        assert!(matches!(
            err,
            RuntimeError::InvalidOpCode {
                pc: 1,
                opcode: 0xFE
            }
        ));
    }

    #[test]
    fn test_positive_call_window_preserves_caller_high_registers() {
        let mut program = BinaryProgram::new();

        // Callee takes 1 arg (r0), uses r0..r4 (num_registers = 5) so an unsafe overlap
        // would overwrite caller r3..r6 if arg_base == 2 inside a 7-register caller frame.
        let mut callee_fn = CompiledFunction::new("callee", 0, 1, 5);
        callee_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 999));
        callee_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 888));
        callee_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 777));
        callee_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 4, 666));
        callee_fn
            .instructions
            .push(Instruction::format_a(OpCode::AddI32, 0, 1, 0, 1));
        callee_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 1, 0, 0));
        program.functions.push(callee_fn);

        // Caller keeps live constant r5 = 100 while calling callee(args: r2..r2).
        let mut main_fn = CompiledFunction::new("main", 0, 0, 7);
        main_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 5, 100));
        main_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 21));
        main_fn
            .instructions
            .push(Instruction::format_call(OpCode::CallDirect, 1, 0, 2, 1));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::AddI32, 0, 6, 1, 5));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 6, 0, 0));
        program.functions.push(main_fn);
        program.entry_point_id = Some(1);

        let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
            .expect("context should build");
        let mut vm = VirtualMachine::new(context);
        let result = vm.run().expect("vm execution should succeed");
        assert_eq!(result, Value::I32(1120));
    }

    #[test]
    fn test_positive_deep_linked_list_drop_without_stack_overflow() {
        let mut program = BinaryProgram::new();
        let mut node_class = solix_common::CompiledClass::new("ListNode", 0, None, 16);
        node_class.reference_field_offsets = vec![8];
        program.classes.push(node_class);

        let mut main_fn = CompiledFunction::new("main", 0, 0, 1);
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
        program.functions.push(main_fn);
        program.entry_point_id = Some(0);

        let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
            .expect("context should build");
        let mut vm = VirtualMachine::new(context);

        let mut head_addr: u32 = 0;
        for _ in 0..50_000 {
            let node_addr = vm
                .heap
                .allocate(16, BlockTag::Object)
                .expect("heap allocation should succeed");
            vm.heap
                .write_u32(node_addr, 0)
                .expect("vtable write should succeed");
            let next_val = if head_addr == 0 {
                Value::Null
            } else {
                Value::Ref(head_addr)
            };
            vm.heap
                .write_u64(node_addr + 8, next_val.to_raw_u64())
                .expect("field write should succeed");
            head_addr = node_addr;
        }

        vm.release_reference(head_addr)
            .expect("iterative release_reference must drop 50,000 nodes without stack overflow");
        assert_eq!(vm.heap.live_objects_count(), 0);
    }

    #[test]
    fn test_negative_divide_by_zero_and_type_mismatch_trap() {
        let mut div_program = BinaryProgram::new();
        let mut div_fn = CompiledFunction::new("main", 0, 0, 3);
        div_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
        div_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
        div_fn
            .instructions
            .push(Instruction::format_a(OpCode::DivI32, 0, 2, 0, 1));
        div_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
        div_program.functions.push(div_fn);
        div_program.entry_point_id = Some(0);

        let div_context = RuntimeContext::new(RuntimeOptions::new().with_binary(div_program))
            .expect("context should build");
        let mut div_vm = VirtualMachine::new(div_context);
        assert!(matches!(div_vm.run(), Err(RuntimeError::DivideByZero)));

        let mut mismatch_program = BinaryProgram::new();
        let mut mismatch_fn = CompiledFunction::new("main", 0, 0, 3);
        mismatch_fn
            .instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 10));
        mismatch_fn
            .instructions
            .push(Instruction::format_b(OpCode::LoadBool, 0, 1, 1));
        mismatch_fn
            .instructions
            .push(Instruction::format_a(OpCode::AddI32, 0, 2, 0, 1));
        mismatch_fn
            .instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
        mismatch_program.functions.push(mismatch_fn);
        mismatch_program.entry_point_id = Some(0);

        let mismatch_context =
            RuntimeContext::new(RuntimeOptions::new().with_binary(mismatch_program))
                .expect("context should build");
        let mut mismatch_vm = VirtualMachine::new(mismatch_context);
        assert!(matches!(
            mismatch_vm.run(),
            Err(RuntimeError::TypeMismatch { .. })
        ));
    }
}
