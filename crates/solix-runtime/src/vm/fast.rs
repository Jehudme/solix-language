//! High-performance inlined instruction dispatcher and register window caching.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::exceptions::RuntimeError;
use crate::frame::CallFrame;
use crate::memory::layout::BlockTag;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use solix_common::bytecode::{ConstantPoolValue, Instruction, OpCode};

#[cold]
#[inline(never)]
fn invalid_opcode_error(pc: u32, raw_opcode: u8) -> RuntimeError {
    RuntimeError::InvalidOpCode {
        pc,
        opcode: raw_opcode,
    }
}

#[cold]
#[inline(never)]
fn stack_overflow_error(current_depth: usize, max_depth: usize) -> RuntimeError {
    RuntimeError::StackOverflow {
        current_depth,
        max_depth,
    }
}

#[cold]
#[inline(never)]
fn function_not_found_error(function_id: u32) -> RuntimeError {
    RuntimeError::FunctionNotFound {
        symbol_or_id: format!("#{function_id}"),
    }
}

#[cold]
#[inline(never)]
fn invalid_static_offset_error(offset: u32, total_static: u32) -> RuntimeError {
    RuntimeError::InvalidStaticOffset {
        offset,
        total_static,
    }
}

#[cold]
#[inline(never)]
fn grow_register_stack(register_stack: &mut Vec<Value>, required: usize) {
    register_stack.resize(required + 4096, Value::Null);
}

impl VirtualMachine {

    /// Fast-path frame activation: copies argument registers within `register_stack` without allocating a heap vector.
    ///
    /// # Errors
    /// Returns `RuntimeError::StackOverflow` if recursion depth exceeds configured limit,
    /// or `RuntimeError::FunctionNotFound` if `function_id` is unrecognized.
    #[inline(always)]
    pub(crate) fn push_frame_fast(
        &mut self,
        function_id: u32,
        caller_dst: u16,
        caller_arg_start: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        if self.call_frames.len() >= self.context.options.max_call_stack_depth {
            return Err(stack_overflow_error(
                self.call_frames.len(),
                self.context.options.max_call_stack_depth,
            ));
        }

        let func = self
            .context
            .all_functions
            .get(function_id as usize)
            .ok_or_else(|| function_not_found_error(function_id))?;

        let meta = &self.context.function_meta[function_id as usize];
        let binary_id = meta.binary_id;
        let func_id_offset = meta.func_id_offset;

        let reg_base = if arg_count > 0 {
            caller_arg_start
        } else {
            self.call_frames.last().map_or(0, |last_frame| {
                last_frame.register_base + last_frame.register_count
            })
        };
        let num_regs = (func.num_registers as usize).max(arg_count);
        let required_regs = reg_base + num_regs;

        if required_regs > self.register_stack.len() {
            grow_register_stack(&mut self.register_stack, required_regs);
        }

        if meta.has_references {
            for &ref_reg in &func.reference_registers {
                let reg_index = ref_reg as usize;
                if reg_index >= arg_count && reg_index < num_regs {
                    // SAFETY: reg_base + reg_index < required_regs <= self.register_stack.len() was ensured above.
                    unsafe {
                        *self.register_stack.as_mut_ptr().add(reg_base + reg_index) = Value::Null;
                    }
                }
            }
        }

        let frame = CallFrame::new(
            function_id,
            binary_id,
            reg_base,
            num_regs,
            func.static_base_offset,
            func_id_offset,
            caller_dst,
        );

        self.call_frames.push(frame);
        if self.call_frames.len() > self.peak_call_stack_depth {
            self.peak_call_stack_depth = self.call_frames.len();
        }
        Ok(())
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
        if self.halted || (!SINGLE_STEP && self.call_frames.len() <= target_depth) {
            return Ok(());
        }
        if SINGLE_STEP && self.call_frames.is_empty() {
            return Ok(());
        }

        let mut inst_ptr: *const Instruction;
        let mut inst_len: usize;
        let mut pc: usize;
        let mut static_base: u32;
        let mut binary_id: usize;
        let mut reg_base: *mut Value;
        let mut local_inst_count: u64 = 0;

        macro_rules! reload_frame {
            () => {
                let curr = self
                    .call_frames
                    .last()
                    .ok_or_else(Self::null_reference_error)?;
                let func = self
                    .context
                    .all_functions
                    .get(curr.function_id as usize)
                    .ok_or_else(Self::null_reference_error)?;
                inst_ptr = func.instructions.as_ptr();
                inst_len = func.instructions.len();
                pc = curr.pc as usize;
                static_base = curr.static_base_offset;
                binary_id = curr.binary_id;
                // SAFETY: curr.register_base is guaranteed within register_stack bounds
                reg_base = unsafe { self.register_stack.as_mut_ptr().add(curr.register_base) };
            };
        }

        macro_rules! flush_telemetry {
            () => {
                if local_inst_count > 0 {
                    self.instructions_dispatched += local_inst_count;
                    local_inst_count = 0;
                }
            };
        }

        macro_rules! finish_instruction {
            () => {
                if SINGLE_STEP {
                    if let Some(frame) = self.call_frames.last_mut() {
                        frame.pc = pc as u32;
                    }
                    if local_inst_count > 0 {
                        self.instructions_dispatched += local_inst_count;
                    }
                    return Ok(());
                }
            };
        }

        reload_frame!();

        while !self.halted {
            if pc >= inst_len {
                let ret_val = Value::Null;
                let frame = self
                    .call_frames
                    .pop()
                    .ok_or_else(Self::null_reference_error)?;
                if self.call_frames.len() <= target_depth {
                    self.last_return_value = ret_val;
                    break;
                }
                let caller_frame = self
                    .call_frames
                    .last()
                    .ok_or_else(Self::null_reference_error)?;
                let caller_func = self
                    .context
                    .all_functions
                    .get(caller_frame.function_id as usize)
                    .ok_or_else(Self::null_reference_error)?;
                inst_ptr = caller_func.instructions.as_ptr();
                inst_len = caller_func.instructions.len();
                pc = caller_frame.pc as usize;
                static_base = caller_frame.static_base_offset;
                binary_id = caller_frame.binary_id;
                // SAFETY: caller_dst is within caller frame register window
                unsafe {
                    let caller_reg_base = self
                        .register_stack
                        .as_mut_ptr()
                        .add(caller_frame.register_base);
                    *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                    reg_base = caller_reg_base;
                }
                finish_instruction!();
                continue;
            }

            // SAFETY: pc < inst_len, so inst_ptr.add(pc) is in-bounds
            let inst = unsafe { *inst_ptr.add(pc) };
            pc += 1;
            local_inst_count += 1;

            let raw_opcode = inst.raw_opcode();
            let Some(opcode) = OpCode::from_raw_u8(raw_opcode) else {
                return Err(invalid_opcode_error(pc as u32, raw_opcode));
            };
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
                        *reg_base.add(inst.dst() as usize) = Value::I64(inst.imm32_signed() as i64);
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
                    let val = self.const_pool_cache[binary_id][idx];
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = val;
                    }
                }

                OpCode::LoadConst => {
                    let pool_idx = inst.imm32() as usize;
                    let const_val = self.const_pool_cache[binary_id][pool_idx];
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = const_val;
                    }
                }

                // =============================================================
                // 0x10 .. 0x20: Arithmetic Operations
                // =============================================================
                OpCode::AddI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(val_a.wrapping_add(val_b));
                    }
                }

                OpCode::AddI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(val_a.wrapping_add(val_b));
                    }
                }

                OpCode::AddF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (val_a, val_b) {
                        (Value::F64(left_val), Value::F64(right_val)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(left_val + right_val);
                        },
                        (left_val, right_val) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{left_val:?}, {right_val:?}"),
                            });
                        }
                    }
                }

                OpCode::SubI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(val_a.wrapping_sub(val_b));
                    }
                }

                OpCode::SubI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(val_a.wrapping_sub(val_b));
                    }
                }

                OpCode::SubF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (val_a, val_b) {
                        (Value::F64(left_val), Value::F64(right_val)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(left_val - right_val);
                        },
                        (left_val, right_val) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{left_val:?}, {right_val:?}"),
                            });
                        }
                    }
                }

                OpCode::MulI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(val_a.wrapping_mul(val_b));
                    }
                }

                OpCode::MulI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(val_a.wrapping_mul(val_b));
                    }
                }

                OpCode::MulF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (val_a, val_b) {
                        (Value::F64(left_val), Value::F64(right_val)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(left_val * right_val);
                        },
                        (left_val, right_val) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{left_val:?}, {right_val:?}"),
                            });
                        }
                    }
                }

                OpCode::DivI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    if val_b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(val_a.wrapping_div(val_b));
                    }
                }

                OpCode::DivI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    if val_b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(val_a.wrapping_div(val_b));
                    }
                }

                OpCode::DivF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (val_a, val_b) {
                        (Value::F64(left_val), Value::F64(right_val)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(left_val / right_val);
                        },
                        (left_val, right_val) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{left_val:?}, {right_val:?}"),
                            });
                        }
                    }
                }

                OpCode::ModI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    if val_b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(val_a.wrapping_rem(val_b));
                    }
                }

                OpCode::ModI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    if val_b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(val_a.wrapping_rem(val_b));
                    }
                }

                OpCode::NegI32 => {
                    let value = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(value.wrapping_neg());
                    }
                }

                OpCode::NegI64 => {
                    let value = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(value.wrapping_neg());
                    }
                }

                OpCode::NegF64 => {
                    let val = unsafe { *reg_base.add(inst.src1() as usize) };
                    match val {
                        Value::F64(float_val) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(-float_val);
                        },
                        other => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64".to_string(),
                                found: format!("{other:?}"),
                            });
                        }
                    }
                }

                // =============================================================
                // 0x21 .. 0x26: Bitwise Operations
                // =============================================================
                OpCode::Band => {
                    let raw_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let raw_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    let result = match (raw_a, raw_b) {
                        (Value::I32(lhs), Value::I32(rhs)) => Value::I32(lhs & rhs),
                        _ => Value::I64(raw_a.as_i64()? & raw_b.as_i64()?),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = result;
                    }
                }

                OpCode::Bor => {
                    let raw_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let raw_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    let result = match (raw_a, raw_b) {
                        (Value::I32(lhs), Value::I32(rhs)) => Value::I32(lhs | rhs),
                        _ => Value::I64(raw_a.as_i64()? | raw_b.as_i64()?),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = result;
                    }
                }

                OpCode::Bxor => {
                    let raw_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let raw_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    let result = match (raw_a, raw_b) {
                        (Value::I32(lhs), Value::I32(rhs)) => Value::I32(lhs ^ rhs),
                        _ => Value::I64(raw_a.as_i64()? ^ raw_b.as_i64()?),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = result;
                    }
                }

                OpCode::Bnot => {
                    let raw_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let result = match raw_a {
                        Value::I32(lhs) => Value::I32(!lhs),
                        _ => Value::I64(!raw_a.as_i64()?),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = result;
                    }
                }

                OpCode::Shl => {
                    let raw_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let raw_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    let result = match (raw_a, raw_b) {
                        (Value::I32(lhs), Value::I32(rhs)) => {
                            let shift = (rhs & 31) as u32;
                            Value::I32(lhs.wrapping_shl(shift))
                        }
                        _ => {
                            let shift = (raw_b.as_i64()? & 63) as u32;
                            Value::I64(raw_a.as_i64()?.wrapping_shl(shift))
                        }
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = result;
                    }
                }

                OpCode::Shr => {
                    let raw_a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let raw_b = unsafe { *reg_base.add(inst.src2() as usize) };
                    let result = match (raw_a, raw_b) {
                        (Value::I32(lhs), Value::I32(rhs)) => {
                            let shift = (rhs & 31) as u32;
                            Value::I32(lhs.wrapping_shr(shift))
                        }
                        _ => {
                            let shift = (raw_b.as_i64()? & 63) as u32;
                            Value::I64(raw_a.as_i64()?.wrapping_shr(shift))
                        }
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = result;
                    }
                }

                // =============================================================
                // 0x30 .. 0x47: Comparisons
                // =============================================================
                OpCode::EqI32 => {
                    let first_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let second_val = unsafe { *reg_base.add(inst.src2() as usize) };
                    let is_equal = match (first_val, second_val) {
                        (Value::I32(first_num), Value::I32(second_num)) => first_num == second_num,
                        _ => first_val.is_equal_to(&second_val),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(is_equal);
                    }
                }

                OpCode::EqI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a == val_b);
                    }
                }

                OpCode::EqF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a == val_b);
                    }
                }

                OpCode::EqBool => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_bool()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_bool()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a == val_b);
                    }
                }

                OpCode::EqString => {
                    let addr_left = unsafe { (*reg_base.add(inst.src1() as usize)).as_ref()? };
                    let addr_right = unsafe { (*reg_base.add(inst.src2() as usize)).as_ref()? };
                    let is_equal = if addr_left == addr_right {
                        true
                    } else if addr_left == 0 || addr_right == 0 {
                        false
                    } else {
                        let hdr_left = self.heap.read_header(addr_left)?;
                        let hdr_right = self.heap.read_header(addr_right)?;
                        if hdr_left.size != hdr_right.size {
                            false
                        } else {
                            let bytes_left =
                                self.heap.read_bytes(addr_left, hdr_left.size as usize)?;
                            let bytes_right =
                                self.heap.read_bytes(addr_right, hdr_right.size as usize)?;
                            bytes_left == bytes_right
                        }
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(is_equal);
                    }
                }

                OpCode::EqRef => {
                    let first_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let second_val = unsafe { *reg_base.add(inst.src2() as usize) };
                    let is_equal = match (first_val, second_val) {
                        (Value::Ref(first_addr), Value::Ref(second_addr)) => {
                            first_addr == second_addr
                        }
                        _ => first_val.is_equal_to(&second_val),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(is_equal);
                    }
                }

                OpCode::NeqI32 => {
                    let first_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let second_val = unsafe { *reg_base.add(inst.src2() as usize) };
                    let is_equal = match (first_val, second_val) {
                        (Value::I32(first_num), Value::I32(second_num)) => first_num == second_num,
                        _ => first_val.is_equal_to(&second_val),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(!is_equal);
                    }
                }

                OpCode::NeqI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a != val_b);
                    }
                }

                OpCode::NeqF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a != val_b);
                    }
                }

                OpCode::NeqBool => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_bool()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_bool()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a != val_b);
                    }
                }

                OpCode::NeqString => {
                    let addr_left = unsafe { (*reg_base.add(inst.src1() as usize)).as_ref()? };
                    let addr_right = unsafe { (*reg_base.add(inst.src2() as usize)).as_ref()? };
                    let is_equal = if addr_left == addr_right {
                        true
                    } else if addr_left == 0 || addr_right == 0 {
                        false
                    } else {
                        let hdr_left = self.heap.read_header(addr_left)?;
                        let hdr_right = self.heap.read_header(addr_right)?;
                        if hdr_left.size != hdr_right.size {
                            false
                        } else {
                            let bytes_left =
                                self.heap.read_bytes(addr_left, hdr_left.size as usize)?;
                            let bytes_right =
                                self.heap.read_bytes(addr_right, hdr_right.size as usize)?;
                            bytes_left == bytes_right
                        }
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(!is_equal);
                    }
                }

                OpCode::NeqRef => {
                    let first_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let second_val = unsafe { *reg_base.add(inst.src2() as usize) };
                    let is_equal = match (first_val, second_val) {
                        (Value::Ref(first_addr), Value::Ref(second_addr)) => {
                            first_addr == second_addr
                        }
                        _ => first_val.is_equal_to(&second_val),
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(!is_equal);
                    }
                }

                OpCode::LtI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                    }
                }

                OpCode::LtI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                    }
                }

                OpCode::LtF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a < val_b);
                    }
                }

                OpCode::LteI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                    }
                }

                OpCode::LteI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                    }
                }

                OpCode::LteF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a <= val_b);
                    }
                }

                OpCode::GtI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                    }
                }

                OpCode::GtI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                    }
                }

                OpCode::GtF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a > val_b);
                    }
                }

                OpCode::GteI32 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                    }
                }

                OpCode::GteI64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                    }
                }

                OpCode::GteF64 => {
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let val_b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(val_a >= val_b);
                    }
                }

                OpCode::Not => {
                    let cond = unsafe { *reg_base.add(inst.src1() as usize) }.is_truthy();
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(!cond);
                    }
                }

                OpCode::ConvIToF => {
                    let val_n = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::F64(val_n as f64);
                    }
                }

                OpCode::ConvFToI => {
                    let val_f = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
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
                    let cond = unsafe { *reg_base.add(inst.cond_reg() as usize) }.is_truthy();
                    if cond {
                        let offset = inst.jump_offset();
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                OpCode::JumpIfFalse => {
                    let cond = unsafe { *reg_base.add(inst.cond_reg() as usize) }.is_truthy();
                    if !cond {
                        let offset = inst.jump_offset();
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                // =============================================================
                // 0x60 .. 0x69: Function Invocations, Returns & Exit
                // =============================================================
                OpCode::CallDirect => {
                    let global_func_id = inst.call_target() as u32;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    let meta = &self.context.function_meta[global_func_id as usize];
                    if let Some(native_callback) = meta.native_fn {
                        self.record_native_call();
                        let call_args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        flush_telemetry!();
                        let call_result = native_callback.invoke(self, &call_args)?;
                        reload_frame!();
                        unsafe {
                            *reg_base.add(caller_dst as usize) = call_result;
                        }
                        finish_instruction!();
                        continue;
                    }

                    self.push_frame_fast(global_func_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
                    finish_instruction!();
                    continue;
                }

                OpCode::CallNativeDirect => {
                    let global_func_id = inst.call_target() as usize;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    let meta = &self.context.function_meta[global_func_id];
                    if let Some(native_callback) = meta.native_fn {
                        self.record_native_call();
                        let call_args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        flush_telemetry!();
                        let call_result = native_callback.invoke(self, &call_args)?;
                        reload_frame!();
                        unsafe {
                            *reg_base.add(caller_dst as usize) = call_result;
                        }
                    }
                    finish_instruction!();
                    continue;
                }

                OpCode::CallVirtual => {
                    let slot = inst.call_target() as usize;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    let receiver = unsafe { *reg_base.add(arg_base as usize) };
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let vtable_id = self.heap.read_u32(obj_addr)?;
                    let class_entry =
                        self.context
                            .get_class_by_vtable_id(vtable_id)
                            .ok_or_else(|| RuntimeError::FunctionNotFound {
                                symbol_or_id: format!("vtable #{vtable_id}"),
                            })?;
                    let global_func_id =
                        class_entry.vtable.get(slot).copied().ok_or_else(|| {
                            RuntimeError::FunctionNotFound {
                                symbol_or_id: format!(
                                    "vtable slot {slot} in class {}",
                                    class_entry.symbol
                                ),
                            }
                        })?;

                    let meta = &self.context.function_meta[global_func_id as usize];
                    if let Some(native_callback) = meta.native_fn {
                        self.record_native_call();
                        let call_args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        flush_telemetry!();
                        let call_result = native_callback.invoke(self, &call_args)?;
                        reload_frame!();
                        unsafe {
                            *reg_base.add(caller_dst as usize) = call_result;
                        }
                        finish_instruction!();
                        continue;
                    }

                    self.push_frame_fast(global_func_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
                    finish_instruction!();
                    continue;
                }

                OpCode::CallInterface => {
                    let target = inst.call_target();
                    let iface_id = (target >> 8) as u32;
                    let slot = (target & 0xFF) as usize;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    let receiver = unsafe { *reg_base.add(arg_base as usize) };
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let vtable_id = self.heap.read_u32(obj_addr)?;
                    let class_entry =
                        self.context
                            .get_class_by_vtable_id(vtable_id)
                            .ok_or_else(|| RuntimeError::FunctionNotFound {
                                symbol_or_id: format!("vtable #{vtable_id}"),
                            })?;
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
                    .ok_or_else(|| RuntimeError::FunctionNotFound {
                        symbol_or_id: format!(
                            "interface slot {slot} in class {}",
                            class_entry.symbol
                        ),
                    })?;

                    let meta = &self.context.function_meta[resolved_func_id as usize];
                    if let Some(native_callback) = meta.native_fn {
                        self.record_native_call();
                        let call_args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        flush_telemetry!();
                        let call_result = native_callback.invoke(self, &call_args)?;
                        reload_frame!();
                        unsafe {
                            *reg_base.add(caller_dst as usize) = call_result;
                        }
                        finish_instruction!();
                        continue;
                    }

                    self.push_frame_fast(
                        resolved_func_id,
                        caller_dst,
                        caller_arg_start,
                        arg_count,
                    )?;
                    reload_frame!();
                    finish_instruction!();
                    continue;
                }

                OpCode::CallSymbol => {
                    let const_idx = inst.call_target() as usize;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();
                    let caller_pc = pc.saturating_sub(1);

                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let pool_val =
                        pool.get(const_idx)
                            .ok_or_else(|| RuntimeError::TypeMismatch {
                                expected: "symbol string in constant pool".to_string(),
                                found: format!("index {const_idx}"),
                            })?;
                    let sym_str = match pool_val {
                        ConstantPoolValue::String(sym) => sym.clone(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{pool_val:?}"),
                            });
                        }
                    };

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_func_id = caller_frame.function_id;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    // Check native registry first
                    let clean_sym = sym_str.split('(').next().unwrap_or(&sym_str);
                    let mangled_sym = clean_sym.replace("::", "_");
                    let simple_sym = clean_sym.rsplit(['.', ':']).next().unwrap_or(clean_sym);
                    let native_candidate = self
                        .native_registry
                        .get(&sym_str)
                        .or_else(|| self.native_registry.get(clean_sym))
                        .or_else(|| self.native_registry.get(&mangled_sym))
                        .or_else(|| self.native_registry.get(simple_sym));

                    if let Some(&native_fn) = native_candidate {
                        self.record_native_call();
                        let call_args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        flush_telemetry!();
                        let call_result = native_fn.invoke(self, &call_args)?;
                        reload_frame!();
                        unsafe {
                            *reg_base.add(caller_dst as usize) = call_result;
                        }
                        if let Some(&global_id) = self.context.symbol_to_function_id.get(&sym_str) {
                            let quickened = Instruction::format_call(
                                OpCode::CallNativeDirect,
                                caller_dst,
                                global_id as u16,
                                arg_base,
                                arg_count as u8,
                            );
                            self.patch_instruction(caller_func_id, caller_pc, quickened);
                        }
                        finish_instruction!();
                        continue;
                    }

                    // Fallback to compiled guest function
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
                                    symbol_or_id: format!(
                                        "Call to unknown native function: {sym_str}"
                                    ),
                                }
                            }
                        })?;

                    if caller_func_id == global_id {
                        let mangled_unresolved = sym_str.replace("::", "_");
                        return Err(RuntimeError::FunctionNotFound {
                            symbol_or_id: format!(
                                "Call to unknown native function: {mangled_unresolved}"
                            ),
                        });
                    }

                    let quickened = Instruction::format_call(
                        OpCode::CallDirect,
                        caller_dst,
                        global_id as u16,
                        arg_base,
                        arg_count as u8,
                    );
                    self.patch_instruction(caller_func_id, caller_pc, quickened);

                    self.push_frame_fast(global_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
                    finish_instruction!();
                    continue;
                }

                OpCode::CallExtern => {
                    let const_idx = inst.call_target() as usize;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();
                    let caller_pc = pc.saturating_sub(1);

                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let pool_val =
                        pool.get(const_idx)
                            .ok_or_else(|| RuntimeError::TypeMismatch {
                                expected: "symbol string in constant pool".to_string(),
                                found: format!("index {const_idx}"),
                            })?;
                    let sym_str = match pool_val {
                        ConstantPoolValue::String(sym) => sym.clone(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{pool_val:?}"),
                            });
                        }
                    };

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_func_id = caller_frame.function_id;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    // Check native registry first
                    if let Some(&native_fn) = self.native_registry.get(&sym_str) {
                        self.record_native_call();
                        let call_args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        flush_telemetry!();
                        let call_result = native_fn.invoke(self, &call_args)?;
                        reload_frame!();
                        unsafe {
                            *reg_base.add(caller_dst as usize) = call_result;
                        }
                        finish_instruction!();
                        continue;
                    }

                    let global_id =
                        self.context
                            .resolve_function_symbol(&sym_str)
                            .ok_or_else(|| RuntimeError::FunctionNotFound {
                                symbol_or_id: sym_str.clone(),
                            })?;

                    // In-place quickening to CallDirect
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
                            self.record_native_call();
                            let call_args = self.register_stack
                                [caller_arg_start..caller_arg_start + arg_count]
                                .to_vec();
                            flush_telemetry!();
                            let call_result = native_fn.invoke(self, &call_args)?;
                            unsafe {
                                *reg_base.add(caller_dst as usize) = call_result;
                            }
                            finish_instruction!();
                            continue;
                        }
                    }

                    self.push_frame_fast(global_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
                    finish_instruction!();
                    continue;
                }

                OpCode::CallCallable => {
                    let r_callee = inst.call_target();
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    let callable_val = unsafe { *reg_base.add(r_callee as usize) };
                    match callable_val {
                        Value::Null => return Err(RuntimeError::NullReferenceAccess),
                        Value::FnPtr(global_func_id) => {
                            let meta = &self.context.function_meta[global_func_id as usize];
                            if let Some(native_callback) = meta.native_fn {
                                self.record_native_call();
                                let call_args = self.register_stack
                                    [caller_arg_start..caller_arg_start + arg_count]
                                    .to_vec();
                                flush_telemetry!();
                                let call_result = native_callback.invoke(self, &call_args)?;
                                reload_frame!();
                                unsafe {
                                    *reg_base.add(caller_dst as usize) = call_result;
                                }
                                finish_instruction!();
                                continue;
                            }
                            self.push_frame_fast(
                                global_func_id,
                                caller_dst,
                                caller_arg_start,
                                arg_count,
                            )?;
                            reload_frame!();
                            finish_instruction!();
                            continue;
                        }
                        Value::Ref(closure_addr) => {
                            if closure_addr == 0 {
                                return Err(RuntimeError::NullReferenceAccess);
                            }
                            let global_func_id = self.heap.read_u32(closure_addr)?;
                            let meta = &self.context.function_meta[global_func_id as usize];
                            if let Some(native_callback) = meta.native_fn {
                                self.record_native_call();
                                let call_args = self.register_stack
                                    [caller_arg_start..caller_arg_start + arg_count]
                                    .to_vec();
                                flush_telemetry!();
                                let call_result = native_callback.invoke(self, &call_args)?;
                                reload_frame!();
                                unsafe {
                                    *reg_base.add(caller_dst as usize) = call_result;
                                }
                                finish_instruction!();
                                continue;
                            }
                            self.push_frame_fast(
                                global_func_id,
                                caller_dst,
                                caller_arg_start,
                                arg_count,
                            )?;
                            if let Some(new_frame) = self.call_frames.last_mut() {
                                new_frame.active_closure = Some(closure_addr);
                            }
                            reload_frame!();
                            finish_instruction!();
                            continue;
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "callable".to_string(),
                                found: callable_val.type_name().to_string(),
                            });
                        }
                    }
                }

                OpCode::Return => {
                    let ret_val = Value::Null;
                    let frame = self
                        .call_frames
                        .pop()
                        .ok_or_else(Self::null_reference_error)?;
                    if self.call_frames.len() <= target_depth {
                        self.last_return_value = ret_val;
                        break;
                    }
                    let caller_frame = self
                        .call_frames
                        .last()
                        .ok_or_else(Self::null_reference_error)?;
                    let caller_func = self
                        .context
                        .all_functions
                        .get(caller_frame.function_id as usize)
                        .ok_or_else(Self::null_reference_error)?;
                    inst_ptr = caller_func.instructions.as_ptr();
                    inst_len = caller_func.instructions.len();
                    pc = caller_frame.pc as usize;
                    static_base = caller_frame.static_base_offset;
                    binary_id = caller_frame.binary_id;
                    // SAFETY: caller_dst is within caller frame register window
                    unsafe {
                        let caller_reg_base = self
                            .register_stack
                            .as_mut_ptr()
                            .add(caller_frame.register_base);
                        *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                        reg_base = caller_reg_base;
                    }
                    finish_instruction!();
                    continue;
                }

                OpCode::ReturnValue => {
                    // SAFETY: Return register is within frame window
                    let ret_val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let frame = self
                        .call_frames
                        .pop()
                        .ok_or_else(Self::null_reference_error)?;
                    if self.call_frames.len() <= target_depth {
                        self.last_return_value = ret_val;
                        break;
                    }
                    let caller_frame = self
                        .call_frames
                        .last()
                        .ok_or_else(Self::null_reference_error)?;
                    let caller_func = self
                        .context
                        .all_functions
                        .get(caller_frame.function_id as usize)
                        .ok_or_else(Self::null_reference_error)?;
                    inst_ptr = caller_func.instructions.as_ptr();
                    inst_len = caller_func.instructions.len();
                    pc = caller_frame.pc as usize;
                    static_base = caller_frame.static_base_offset;
                    binary_id = caller_frame.binary_id;
                    // SAFETY: caller_dst is within caller frame register window
                    unsafe {
                        let caller_reg_base = self
                            .register_stack
                            .as_mut_ptr()
                            .add(caller_frame.register_base);
                        *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                        reg_base = caller_reg_base;
                    }
                    finish_instruction!();
                    continue;
                }

                OpCode::Exit => {
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let code = val.as_i32()?;
                    self.exit_code = Some(code);
                    self.halted = true;
                    break;
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
                    let payload_addr = self.heap.allocate(size, tag)?;
                    self.heap.write_u32(payload_addr, vtable_id)?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                    }
                }

                OpCode::AllocObjectExtern => {
                    let caller_func_id = self.current_frame()?.function_id;
                    let caller_pc = pc.saturating_sub(1);
                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let pool_idx = inst.imm32() as usize;
                    let pool_val =
                        pool.get(pool_idx)
                            .ok_or_else(|| RuntimeError::TypeMismatch {
                                expected: "symbol string in constant pool".to_string(),
                                found: format!("index {pool_idx}"),
                            })?;
                    let sym_str = match pool_val {
                        ConstantPoolValue::String(sym) => sym.as_str(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{pool_val:?}"),
                            });
                        }
                    };
                    let (vtable_id, size) =
                        self.context.resolve_vtable_info(sym_str).ok_or_else(|| {
                            RuntimeError::FunctionNotFound {
                                symbol_or_id: format!("class {sym_str}"),
                            }
                        })?;
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
                    let payload_addr = self.heap.allocate(alloc_size, tag)?;
                    self.heap.write_u32(payload_addr, vtable_id)?;

                    // In-place quickening to AllocObject
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

                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                    }
                }

                OpCode::GetField => {
                    let receiver = unsafe { *reg_base.add(inst.src1() as usize) };
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(Self::null_reference_error());
                    }
                    let offset = inst.src2() as usize;
                    let field_addr = obj_addr + offset as u32;

                    let raw_flags = inst.flags();
                    let field_type_tag = if raw_flags > 1 {
                        Some(raw_flags >> 1)
                    } else {
                        let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
                        self.context
                            .get_class_by_vtable_id(vtable_id)
                            .and_then(|cls| cls.get_field_type(offset))
                    };

                    let val = match field_type_tag {
                        Some(1) => {
                            let byte_val = self.heap.read_u8(field_addr).unwrap_or(0);
                            Value::I32(byte_val as i8 as i32)
                        }
                        Some(2) => {
                            let byte_val = self.heap.read_u8(field_addr).unwrap_or(0);
                            Value::I32(byte_val as i32)
                        }
                        Some(3) => {
                            let word_val = self.heap.read_u16(field_addr).unwrap_or(0);
                            Value::I32(word_val as i16 as i32)
                        }
                        Some(4) => {
                            let word_val = self.heap.read_u16(field_addr).unwrap_or(0);
                            Value::I32(word_val as i32)
                        }
                        Some(5) | Some(6) => {
                            let raw = self.heap.read_u32(field_addr).unwrap_or(0);
                            Value::I32(raw as i32)
                        }
                        Some(7) | Some(8) => {
                            let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                            Value::I64(raw as i64)
                        }
                        Some(9) => {
                            let raw = self.heap.read_u32(field_addr).unwrap_or(0);
                            Value::F64(f32::from_bits(raw) as f64)
                        }
                        Some(10) => {
                            let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                            Value::F64(f64::from_bits(raw))
                        }
                        Some(11) => {
                            let raw = self.heap.read_u8(field_addr).unwrap_or(0);
                            Value::Bool(raw != 0)
                        }
                        Some(12) => {
                            let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                            if raw == 0 || raw == Value::TAG_NULL {
                                Value::Null
                            } else {
                                Value::from_raw_u64(raw)
                            }
                        }
                        _ => {
                            let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                            if raw == 0 {
                                let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
                                let maybe_class = self.context.get_class_by_vtable_id(vtable_id);
                                let is_ref = maybe_class.is_some_and(|cls| {
                                    cls.reference_field_offsets.contains(&offset)
                                        || cls.weak_field_offsets.contains(&offset)
                                });
                                if is_ref { Value::Null } else { Value::I32(0) }
                            } else {
                                Value::from_raw_u64(raw)
                            }
                        }
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = val;
                    }
                }

                OpCode::SetField => {
                    let receiver = unsafe { *reg_base.add(inst.src1() as usize) };
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let offset = inst.src2() as usize;
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let field_addr = obj_addr + offset as u32;

                    let raw_flags = inst.flags();
                    let is_weak = (raw_flags & solix_common::bytecode::instruction::FLAG_WEAK) != 0;
                    let tag_shifted = raw_flags >> 1;
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
                            let _ = self.heap.write_u8(field_addr, val.as_i32()? as u8);
                        }
                        Some(3) | Some(4) => {
                            let _ = self.heap.write_u16(field_addr, val.as_i32()? as u16);
                        }
                        Some(5) | Some(6) => {
                            let _ = self.heap.write_u32(field_addr, val.as_i32()? as u32);
                        }
                        Some(7) | Some(8) => {
                            let _ = self.heap.write_u64(field_addr, val.as_i64()? as u64);
                        }
                        Some(9) => {
                            let bits = (val.as_f64()? as f32).to_bits();
                            let _ = self.heap.write_u32(field_addr, bits);
                        }
                        Some(10) => {
                            let bits = val.as_f64()?.to_bits();
                            let _ = self.heap.write_u64(field_addr, bits);
                        }
                        Some(11) => {
                            let _ = self
                                .heap
                                .write_u8(field_addr, if val.as_bool()? { 1 } else { 0 });
                        }
                        _ => {
                            let old_raw = self.heap.read_u64(field_addr).unwrap_or(0);
                            let old_val = Value::from_raw_u64(old_raw);
                            self.heap.write_u64(field_addr, val.to_raw_u64())?;
                            if let Value::Ref(child_addr) = val
                                && !is_weak
                                && child_addr > 0
                            {
                                crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                            }
                            if let Value::Ref(old_child) = old_val
                                && !is_weak
                                && old_child > 0
                            {
                                let caller_frame = self.current_frame_mut()?;
                                caller_frame.pc = pc as u32;
                                self.release_reference(old_child)?;
                                reload_frame!();
                            }
                        }
                    }
                }

                OpCode::GetStaticDirect => {
                    let slot = inst.imm32() as usize;
                    if slot >= self.static_slots.len() {
                        return Err(RuntimeError::InvalidStaticOffset {
                            offset: slot as u32,
                            total_static: self.static_slots.len() as u32,
                        });
                    }
                    // SAFETY: Slot is verified in bounds
                    let static_val = unsafe { *self.static_slots.get_unchecked(slot) };
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = static_val;
                    }
                }

                OpCode::GetStatic => {
                    let slot = (static_base + inst.imm32()) as usize;
                    if slot >= self.static_slots.len() {
                        return Err(RuntimeError::InvalidStaticOffset {
                            offset: slot as u32,
                            total_static: self.static_slots.len() as u32,
                        });
                    }
                    // SAFETY: Slot is verified in bounds
                    let static_val = unsafe { *self.static_slots.get_unchecked(slot) };
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = static_val;
                    }
                }

                OpCode::SetStatic => {
                    let slot_idx = static_base + inst.imm32();
                    let val_to_set = unsafe { *reg_base.add(inst.dst() as usize) };
                    self.set_static(slot_idx, val_to_set)?;
                }

                OpCode::GetStaticGlobal => {
                    let caller_func_id = self.current_frame()?.function_id;
                    let caller_pc = pc.saturating_sub(1);
                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let pool_idx = inst.imm32() as usize;
                    let pool_val =
                        pool.get(pool_idx)
                            .ok_or_else(|| RuntimeError::TypeMismatch {
                                expected: "symbol string in constant pool".to_string(),
                                found: format!("index {pool_idx}"),
                            })?;
                    let sym_str = match pool_val {
                        ConstantPoolValue::String(sym) => sym.as_str(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{pool_val:?}"),
                            });
                        }
                    };
                    let resolved_slot = self.context.resolve_static_slot(sym_str).ok_or_else(|| {
                        invalid_static_offset_error(pool_idx as u32, self.static_slots.len() as u32)
                    })?;

                    // In-place quickening to GetStaticDirect
                    let quickened = Instruction::format_b(
                        OpCode::GetStaticDirect,
                        0,
                        inst.dst(),
                        resolved_slot,
                    );
                    self.patch_instruction(caller_func_id, caller_pc, quickened);

                    let static_val = self.get_static(resolved_slot)?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = static_val;
                    }
                }

                OpCode::SetStaticGlobal => {
                    let caller_func_id = self.current_frame()?.function_id;
                    let caller_pc = pc.saturating_sub(1);
                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let pool_idx = inst.imm32() as usize;
                    let pool_val =
                        pool.get(pool_idx)
                            .ok_or_else(|| RuntimeError::TypeMismatch {
                                expected: "symbol string in constant pool".to_string(),
                                found: format!("index {pool_idx}"),
                            })?;
                    let sym_str = match pool_val {
                        ConstantPoolValue::String(sym) => sym.as_str(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{pool_val:?}"),
                            });
                        }
                    };
                    let resolved_slot = self.context.resolve_static_slot(sym_str).ok_or_else(|| {
                        invalid_static_offset_error(pool_idx as u32, self.static_slots.len() as u32)
                    })?;

                    // In-place quickening to SetStaticDirect
                    let quickened = Instruction::format_b(
                        OpCode::SetStaticDirect,
                        0,
                        inst.dst(),
                        resolved_slot,
                    );
                    self.patch_instruction(caller_func_id, caller_pc, quickened);

                    let val_to_set = unsafe { *reg_base.add(inst.dst() as usize) };
                    self.set_static(resolved_slot, val_to_set)?;
                }

                OpCode::SetStaticDirect => {
                    let slot = inst.imm32();
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    self.set_static(slot, val)?;
                }

                // =============================================================
                // 0x80 .. 0x85 & 0xA4: Dynamic Arrays & Tuples
                // =============================================================
                OpCode::AllocArray => {
                    let raw_len = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    if raw_len < 0 {
                        return Err(RuntimeError::OutOfMemory {
                            requested: usize::MAX,
                            available: self.heap.dynamic_capacity(),
                        });
                    }
                    let array_len = raw_len as usize;
                    let elem_size = (inst.src2() as usize).max(1);
                    let payload_size = 8 + array_len * elem_size;
                    let payload_addr = self.heap.allocate(payload_size, BlockTag::ArrayRef)?;
                    self.heap.write_u32(payload_addr, array_len as u32)?;
                    self.heap.write_u8(payload_addr + 4, elem_size as u8)?;
                    self.heap.write_u8(payload_addr + 7, inst.flags())?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                    }
                }

                OpCode::ArrayLength => {
                    let arr_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let arr_addr = arr_val.as_ref()?;
                    if arr_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let header = self.heap.read_header(arr_addr)?;
                    let array_len = if header.tag == BlockTag::String {
                        header.size
                    } else {
                        self.heap.read_u32(arr_addr)?
                    };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(array_len as i32);
                    }
                }

                OpCode::GetArray => {
                    let arr_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let arr_addr = arr_val.as_ref()?;
                    if arr_addr == 0 {
                        return Err(Self::null_reference_error());
                    }
                    let index_i64 = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    let arr_offset = arr_addr as usize;
                    if arr_offset < 8 || arr_offset + 8 > self.heap.buffer.len() {
                        return Err(RuntimeError::NullReferenceAccess);
                    }

                    let tag_nibble = (self.heap.buffer[arr_offset - 1] >> 4) & 0x0F;
                    if tag_nibble == BlockTag::String as u8 {
                        let string_len = u32::from_le_bytes(
                            self.heap.buffer[arr_offset - 8..arr_offset - 4]
                                .try_into()
                                .unwrap_or([0; 4]),
                        ) as usize;
                        if (index_i64 as u64) >= (string_len as u64) {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: index_i64,
                                length: string_len,
                            });
                        }
                        let byte_val = self.heap.buffer[arr_offset + index_i64 as usize];
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(byte_val as i32);
                        }
                    } else if tag_nibble == BlockTag::Tuple as u8 {
                        let tuple_len = u32::from_le_bytes(
                            self.heap.buffer[arr_offset..arr_offset + 4]
                                .try_into()
                                .unwrap_or([0; 4]),
                        ) as usize;
                        if (index_i64 as u64) >= (tuple_len as u64) {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: index_i64,
                                length: tuple_len,
                            });
                        }
                        let index_usize = index_i64 as usize;
                        let elem_offset = arr_offset + 8 + (index_usize * 16);
                        if elem_offset + 16 > self.heap.buffer.len() {
                            return Err(RuntimeError::NullReferenceAccess);
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
                        unsafe {
                            *reg_base.add(inst.dst() as usize) =
                                Value::from_tagged_parts(tag_raw, payload_raw);
                        }
                    } else {
                        let array_len = u32::from_le_bytes(
                            self.heap.buffer[arr_offset..arr_offset + 4]
                                .try_into()
                                .unwrap_or([0; 4]),
                        ) as usize;
                        if (index_i64 as u64) >= (array_len as u64) {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: index_i64,
                                length: array_len,
                            });
                        }
                        let index_usize = index_i64 as usize;
                        let elem_size = (self.heap.buffer[arr_offset + 4] as usize).max(1);
                        let elem_offset = arr_offset + 8 + (index_usize * elem_size);
                        if elem_offset + elem_size > self.heap.buffer.len() {
                            return Err(RuntimeError::NullReferenceAccess);
                        }
                        let raw_word = match elem_size {
                            1 => self.heap.buffer[elem_offset] as u64,
                            2 => u16::from_le_bytes(
                                self.heap.buffer[elem_offset..elem_offset + 2]
                                    .try_into()
                                    .unwrap_or([0; 2]),
                            ) as u64,
                            4 => u32::from_le_bytes(
                                self.heap.buffer[elem_offset..elem_offset + 4]
                                    .try_into()
                                    .unwrap_or([0; 4]),
                            ) as u64,
                            _ => u64::from_le_bytes(
                                self.heap.buffer[elem_offset..elem_offset + 8]
                                    .try_into()
                                    .unwrap_or([0; 8]),
                            ),
                        };
                        let elem_type_tag = self.heap.buffer[arr_offset + 7];
                        let val = match elem_type_tag {
                            0 => Value::from_raw_u64(raw_word),
                            1 => Value::I32(raw_word as i8 as i32),
                            2 => Value::I32(raw_word as i16 as i32),
                            3 => Value::I32(raw_word as i32),
                            4 => Value::I64(raw_word as i64),
                            5 | 12 => Value::I32(raw_word as u8 as i32),
                            6 => Value::I32(raw_word as u16 as i32),
                            7 => Value::I32(raw_word as u32 as i32),
                            8 => Value::I64(raw_word as i64),
                            9 => {
                                if elem_size == 4 || (raw_word >> 32) == 0 {
                                    Value::F64(f32::from_bits(raw_word as u32) as f64)
                                } else {
                                    Value::F64(f64::from_bits(raw_word))
                                }
                            }
                            10 => Value::F64(f64::from_bits(raw_word)),
                            11 => Value::Bool(raw_word != 0),
                            13 | 0x80..=0xFF => {
                                if raw_word == 0 {
                                    Value::Null
                                } else {
                                    Value::Ref(raw_word as u32)
                                }
                            }
                            _ => Value::from_raw_u64(raw_word),
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = val;
                        }
                    }
                }

                OpCode::SetArray => {
                    let arr_val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let arr_addr = arr_val.as_ref()?;
                    if arr_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let index_i64 = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let arr_offset = arr_addr as usize;
                    if arr_offset < 8 || arr_offset + 8 > self.heap.buffer.len() {
                        return Err(RuntimeError::NullReferenceAccess);
                    }

                    let array_len = u32::from_le_bytes(
                        self.heap.buffer[arr_offset..arr_offset + 4]
                            .try_into()
                            .unwrap_or([0; 4]),
                    ) as usize;
                    if (index_i64 as u64) >= (array_len as u64) {
                        return Err(RuntimeError::IndexOutOfBounds {
                            index: index_i64,
                            length: array_len,
                        });
                    }
                    let index_usize = index_i64 as usize;
                    let val = unsafe { *reg_base.add(inst.src2() as usize) };
                    let tag_nibble = (self.heap.buffer[arr_offset - 1] >> 4) & 0x0F;
                    let is_tuple = tag_nibble == BlockTag::Tuple as u8;

                    if is_tuple {
                        let elem_offset = arr_offset + 8 + (index_usize * 16);
                        if elem_offset + 16 > self.heap.buffer.len() {
                            return Err(RuntimeError::NullReferenceAccess);
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
                        self.heap.buffer[elem_offset..elem_offset + 8]
                            .copy_from_slice(&new_tag.to_le_bytes());
                        self.heap.buffer[elem_offset + 8..elem_offset + 16]
                            .copy_from_slice(&new_payload.to_le_bytes());

                        if let Value::Ref(child_addr) = val
                            && child_addr > 0
                        {
                            crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                        }
                        if let Value::Ref(old_child) = old_val
                            && old_child > 0
                        {
                            let caller_frame = self.current_frame_mut()?;
                            caller_frame.pc = pc as u32;
                            self.release_reference(old_child)?;
                            reload_frame!();
                        }
                    } else {
                        let elem_size = (self.heap.buffer[arr_offset + 4] as usize).max(1);
                        let elem_offset = arr_offset + 8 + (index_usize * elem_size);
                        if elem_offset + elem_size > self.heap.buffer.len() {
                            return Err(RuntimeError::NullReferenceAccess);
                        }

                        let elem_type_tag = self.heap.buffer[arr_offset + 7];
                        let is_reference_element = elem_type_tag == 13 || elem_type_tag >= 0x80;

                        let old_child = if is_reference_element {
                            u32::from_le_bytes(
                                self.heap.buffer[elem_offset..elem_offset + 4]
                                    .try_into()
                                    .unwrap_or([0; 4]),
                            )
                        } else {
                            0
                        };

                        if elem_type_tag == 0 {
                            let encoded_word = val.to_raw_u64();
                            self.heap.buffer[elem_offset..elem_offset + 8]
                                .copy_from_slice(&encoded_word.to_le_bytes());
                        } else {
                            let primitive_val = val.to_primitive_u64();
                            match elem_size {
                                1 => {
                                    self.heap.buffer[elem_offset] = primitive_val as u8;
                                }
                                2 => {
                                    self.heap.buffer[elem_offset..elem_offset + 2]
                                        .copy_from_slice(&(primitive_val as u16).to_le_bytes());
                                }
                                4 => {
                                    self.heap.buffer[elem_offset..elem_offset + 4]
                                        .copy_from_slice(&(primitive_val as u32).to_le_bytes());
                                }
                                _ => {
                                    self.heap.buffer[elem_offset..elem_offset + 8]
                                        .copy_from_slice(&primitive_val.to_le_bytes());
                                }
                            }
                        }

                        if is_reference_element {
                            if let Value::Ref(child_addr) = val
                                && child_addr > 0
                            {
                                crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                            }
                            if old_child > 0 {
                                let caller_frame = self.current_frame_mut()?;
                                caller_frame.pc = pc as u32;
                                self.release_reference(old_child)?;
                                reload_frame!();
                            }
                        }
                    }
                }

                OpCode::AllocTuple => {
                    let tuple_count = inst.src1() as usize;
                    let payload_size = 8 + tuple_count * 16;
                    let payload_addr = self.heap.allocate(payload_size, BlockTag::Tuple)?;
                    self.heap.write_u32(payload_addr, tuple_count as u32)?;
                    self.heap.write_u32(payload_addr + 4, 16)?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                    }
                }

                OpCode::GetTuple => {
                    let tuple_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let tuple_addr = tuple_val.as_ref()?;
                    if tuple_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let slot_idx = inst.src2() as usize;
                    let tuple_len = self.heap.read_u32(tuple_addr)? as usize;
                    if slot_idx >= tuple_len {
                        return Err(RuntimeError::IndexOutOfBounds {
                            index: slot_idx as i64,
                            length: tuple_len,
                        });
                    }
                    let elem_addr = tuple_addr + 8 + (slot_idx * 16) as u32;
                    let tag_raw = self.heap.read_u64(elem_addr).unwrap_or(0);
                    let payload_raw = self.heap.read_u64(elem_addr + 8).unwrap_or(0);
                    unsafe {
                        *reg_base.add(inst.dst() as usize) =
                            Value::from_tagged_parts(tag_raw, payload_raw);
                    }
                }

                OpCode::TupleInstanceof => {
                    let tuple_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let tuple_addr = match tuple_val.as_ref() {
                        Ok(addr) if addr > 0 => addr,
                        _ => {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::Bool(false);
                            }
                            finish_instruction!();
                            continue;
                        }
                    };
                    let index_i64 = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    let tuple_len = self.heap.read_u32(tuple_addr)? as usize;
                    if index_i64 < 0 || (index_i64 as usize) >= tuple_len {
                        return Err(RuntimeError::IndexOutOfBounds {
                            index: index_i64,
                            length: tuple_len,
                        });
                    }
                    let elem_addr = tuple_addr + 8 + (index_i64 as usize * 16) as u32;
                    let tag_raw = self.heap.read_u64(elem_addr).unwrap_or(0);
                    let payload_raw = self.heap.read_u64(elem_addr + 8).unwrap_or(0);
                    let val = Value::from_tagged_parts(tag_raw, payload_raw);
                    let tag = inst.flags();
                    let is_match = match tag {
                        1..=4 => matches!(val, Value::I32(_) | Value::I64(_)),
                        5..=8 => matches!(val, Value::I32(_) | Value::I64(_)),
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

                    if cap_count == 0 {
                        // Stateless closure: encode directly as a tagged FnPtr with zero heap allocation.
                        unsafe {
                            *reg_base.add(caller_dst) = Value::FnPtr(target_lambda_id);
                        }
                    } else {
                        let payload_size = 8 + cap_count * 16;
                        let payload_addr = self.heap.allocate(payload_size, BlockTag::Closure)?;
                        self.heap.write_u32(payload_addr, target_lambda_id)?;
                        self.heap.write_u32(payload_addr + 4, cap_count as u32)?;

                        for index in 0..cap_count {
                            let cell_addr = payload_addr + 8 + (index * 16) as u32;
                            let cap_val = unsafe { *reg_base.add(cap_base + index) };
                            let (tag_raw, payload_raw) = cap_val.to_tagged_parts();
                            let _ = self.heap.write_u64(cell_addr, tag_raw);
                            let _ = self.heap.write_u64(cell_addr + 8, payload_raw);
                            if let Value::Ref(child_addr) = cap_val
                                && child_addr > 0
                            {
                                crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                            }
                        }

                        unsafe {
                            *reg_base.add(caller_dst) = Value::Ref(payload_addr);
                        }
                    }
                }

                OpCode::UnpackCaptures => {
                    let capture_base = inst.arg_base() as usize;
                    let capture_count = inst.arg_count() as usize;

                    if capture_count > 0 {
                        let closure_addr = self
                            .current_frame()?
                            .active_closure
                            .ok_or_else(Self::null_reference_error)?;

                        for index in 0..capture_count {
                            let cell_addr = closure_addr + 8 + (index * 16) as u32;
                            let tag_raw = self.heap.read_u64(cell_addr).unwrap_or(0);
                            let payload_raw = self.heap.read_u64(cell_addr + 8).unwrap_or(0);
                            unsafe {
                                *reg_base.add(capture_base + index) =
                                    Value::from_tagged_parts(tag_raw, payload_raw);
                            }
                        }
                    }
                }

                // =============================================================
                // 0xA0 .. 0xA1: Instanceof & Cast
                // =============================================================
                OpCode::Instanceof => {
                    let obj_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let obj_addr = match obj_val.as_ref() {
                        Ok(addr) if addr > 0 => addr,
                        _ => {
                            unsafe {
                                *reg_base.add(inst.dst() as usize) = Value::Bool(false);
                            }
                            finish_instruction!();
                            continue;
                        }
                    };
                    let target_vtable_id = inst.src2() as u32;
                    let header = self.heap.read_header(obj_addr)?;
                    let is_instance =
                        if header.tag == BlockTag::Object || header.tag == BlockTag::ObjectPod {
                            let obj_vtable_id = self.heap.read_u32(obj_addr)?;
                            self.is_instance_of(obj_vtable_id, target_vtable_id)
                        } else {
                            false
                        };
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(is_instance);
                    }
                }

                OpCode::CastCheck => {
                    let val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let tag = inst.flags();
                    if tag != 0 {
                        let is_match = match tag {
                            1..=4 => matches!(val, Value::I32(_) | Value::I64(_)),
                            5..=8 => matches!(val, Value::I32(_) | Value::I64(_)),
                            9..=10 => matches!(val, Value::F64(_)),
                            11 => matches!(val, Value::Bool(_)),
                            12 => matches!(val, Value::I32(_) | Value::I64(_)),
                            13 => {
                                if let Value::Ref(target_addr) = val {
                                    self.heap.read_header(target_addr).is_ok_and(|header_info| {
                                        header_info.tag == BlockTag::String
                                    })
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        };
                        if !is_match {
                            return Err(RuntimeError::InvalidCast {
                                from: format!("{val}"),
                                to: format!("type tag #{tag}"),
                            });
                        }
                        let converted_value = match (tag, val) {
                            (4 | 8, Value::I32(primitive_number)) => {
                                Value::I64(primitive_number as i64)
                            }
                            (1, Value::I32(primitive_number)) => {
                                Value::I32(primitive_number as i8 as i32)
                            }
                            (2, Value::I32(primitive_number)) => {
                                Value::I32(primitive_number as i16 as i32)
                            }
                            (5, Value::I32(primitive_number)) => {
                                Value::I32(primitive_number as u8 as i32)
                            }
                            (6, Value::I32(primitive_number)) => {
                                Value::I32(primitive_number as u16 as i32)
                            }
                            (1, Value::I64(primitive_number)) => {
                                Value::I32(primitive_number as i8 as i32)
                            }
                            (2, Value::I64(primitive_number)) => {
                                Value::I32(primitive_number as i16 as i32)
                            }
                            (5, Value::I64(primitive_number)) => {
                                Value::I32(primitive_number as u8 as i32)
                            }
                            (6, Value::I64(primitive_number)) => {
                                Value::I32(primitive_number as u16 as i32)
                            }
                            (3 | 7 | 12, Value::I64(primitive_number)) => {
                                Value::I32(primitive_number as i32)
                            }
                            _ => val,
                        };
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = converted_value;
                        }
                        finish_instruction!();
                        continue;
                    }

                    let obj_addr = val.as_ref()?;
                    if obj_addr == 0 {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Null;
                        }
                        finish_instruction!();
                        continue;
                    }
                    let target_vtable_id = inst.src2() as u32;
                    let header = self.heap.read_header(obj_addr)?;
                    let is_instance =
                        if header.tag == BlockTag::Object || header.tag == BlockTag::ObjectPod {
                            let obj_vtable_id = self.heap.read_u32(obj_addr)?;
                            self.is_instance_of(obj_vtable_id, target_vtable_id)
                        } else {
                            false
                        };
                    if !is_instance {
                        return Err(RuntimeError::InvalidCast {
                            from: format!("ref 0x{obj_addr:08X}"),
                            to: format!("vtable #{target_vtable_id}"),
                        });
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Ref(obj_addr);
                    }
                }

                // =============================================================
                // 0xC0 .. 0xC1: ARC Reference Counting
                // =============================================================
                OpCode::IncRef => {
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    if let Value::Ref(addr) = val
                        && addr > 0
                    {
                        if self.cycle_collector_enabled {
                            let mut header = self.heap.read_header(addr)?;
                            header.color = crate::memory::layout::GcColor::Black;
                            let _ = header.inc_ref()?;
                            self.heap.write_header(addr, header)?;
                        } else {
                            crate::memory::arc::inc_ref(&mut self.heap, addr)?;
                        }
                    }
                }

                OpCode::DecRef => {
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    if let Value::Ref(addr) = val
                        && addr > 0
                    {
                        let header = self.heap.read_header(addr)?;
                        if header.ref_count == crate::memory::layout::MAX_REF_COUNT {
                            // Immortal constant string or immortal object
                        } else if header.ref_count > 1 {
                            let mut updated_header = header;
                            updated_header.ref_count -= 1;
                            if self.cycle_collector_enabled
                                && crate::memory::layout::can_contain_references(header.tag)
                                && updated_header.color != crate::memory::layout::GcColor::Purple
                            {
                                updated_header.color = crate::memory::layout::GcColor::Purple;
                                self.purple_buffer.push(addr);
                                self.arc_cycle_collector_candidates += 1;
                            }
                            self.heap.write_header(addr, updated_header)?;
                            if self.cycle_collector_enabled
                                && self.purple_buffer.len() >= self.cycle_threshold
                            {
                                self.collect_cycles()?;
                            }
                        } else if header.tag == BlockTag::ObjectPod {
                            self.heap.deallocate_pod_fast(addr, header.size)?;
                        } else {
                            let caller_frame = self.current_frame_mut()?;
                            caller_frame.pc = pc as u32;
                            self.release_reference(addr)?;
                            reload_frame!();
                        }
                    }
                }

                // =============================================================
                // 0xD0 .. 0xD7: Superinstructions
                // =============================================================
                OpCode::AddI32Imm => {
                    let imm_val = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) =
                            Value::I32(val_a.wrapping_add(imm_val));
                    }
                }

                OpCode::SubI32Imm => {
                    let imm_val = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) =
                            Value::I32(val_a.wrapping_sub(imm_val));
                    }
                }

                OpCode::JumpIfLtI32 => {
                    let offset = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.dst() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    if val_a < val_b {
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                OpCode::JumpIfLteI32 => {
                    let offset = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.dst() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    if val_a <= val_b {
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                OpCode::JumpIfGtI32 => {
                    let offset = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.dst() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    if val_a > val_b {
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                OpCode::JumpIfGteI32 => {
                    let offset = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.dst() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    if val_a >= val_b {
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                OpCode::JumpIfEqI32 => {
                    let target_offset = inst.src2() as i16 as i32;
                    let first_val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let second_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let is_equal = match (first_val, second_val) {
                        (Value::I32(first_num), Value::I32(second_num)) => first_num == second_num,
                        _ => first_val.is_equal_to(&second_val),
                    };
                    if is_equal {
                        pc = ((pc as i32 - 1) + target_offset) as usize;
                    }
                }

                OpCode::JumpIfNeqI32 => {
                    let target_offset = inst.src2() as i16 as i32;
                    let first_val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let second_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let is_equal = match (first_val, second_val) {
                        (Value::I32(first_num), Value::I32(second_num)) => first_num == second_num,
                        _ => first_val.is_equal_to(&second_val),
                    };
                    if !is_equal {
                        pc = ((pc as i32 - 1) + target_offset) as usize;
                    }
                }

                // =============================================================
                // =============================================================
                // 0xB0 .. 0xBF: Exception Handling
                // =============================================================
                OpCode::Throw => {
                    let exc_val = unsafe { *reg_base.add(inst.dst() as usize) };
                    if exc_val.is_null() {
                        return Err(RuntimeError::NullReferenceAccess);
                    }

                    loop {
                        if self.call_frames.is_empty() {
                            if local_inst_count > 0 {
                                self.instructions_dispatched += local_inst_count;
                            }
                            return Err(RuntimeError::UserException {
                                message: format!("{exc_val}"),
                            });
                        }

                        let (func_id, cur_pc) = {
                            let frame = self.current_frame()?;
                            (frame.function_id, frame.pc.saturating_sub(1))
                        };

                        let func = self.context.get_function(func_id).ok_or_else(|| {
                            RuntimeError::FunctionNotFound {
                                symbol_or_id: format!("#{func_id}"),
                            }
                        })?;

                        // Look for matching exception handler
                        let mut matched_handler = None;
                        let mut fallback_handler = None;
                        for handler in &func.exception_handlers {
                            if cur_pc >= handler.try_start_pc && cur_pc < handler.try_end_pc {
                                match &handler.catch_type_symbol {
                                    None => {
                                        if fallback_handler.is_none() {
                                            fallback_handler = Some(handler.clone());
                                        }
                                    }
                                    Some(type_sym) => {
                                        let is_match = if let Value::Ref(obj_addr) = exc_val
                                            && obj_addr > 0
                                        {
                                            let obj_vtable_id =
                                                self.heap.read_u32(obj_addr).unwrap_or(0);
                                            if let Some(target_class) =
                                                self.context.all_classes.iter().find(
                                                    |candidate_class| {
                                                        candidate_class.symbol == *type_sym
                                                    },
                                                )
                                            {
                                                self.is_instance_of(
                                                    obj_vtable_id,
                                                    target_class.vtable_id,
                                                )
                                            } else {
                                                true
                                            }
                                        } else {
                                            true
                                        };
                                        if is_match {
                                            matched_handler = Some(handler.clone());
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if matched_handler.is_none() {
                            matched_handler = fallback_handler;
                        }

                        if let Some(handler) = matched_handler {
                            let exception_address = if let Value::Ref(target_addr) = exc_val {
                                target_addr
                            } else {
                                0
                            };
                            let curr_frame = self.current_frame()?;
                            let curr_reg_base = curr_frame.register_base;
                            for &register_index in &handler.try_ref_regs {
                                if register_index != handler.catch_reg {
                                    let reg_val = self.register_stack
                                        [curr_reg_base + register_index as usize];
                                    if let Value::Ref(object_addr) = reg_val
                                        && object_addr > 0
                                        && object_addr != exception_address
                                    {
                                        let _ = self.release_reference(object_addr);
                                        self.register_stack
                                            [curr_reg_base + register_index as usize] = Value::Null;
                                    }
                                }
                            }

                            self.register_stack[curr_reg_base + handler.catch_reg as usize] =
                                exc_val;
                            let frame = self.current_frame_mut()?;
                            frame.pc = handler.handler_pc;
                            flush_telemetry!();
                            reload_frame!();
                            finish_instruction!();
                            break;
                        }

                        // No handler in this frame: unwind and drop frame references
                        let curr_frame = self.current_frame()?;
                        let curr_reg_base = curr_frame.register_base;
                        let ref_regs = func.reference_registers.clone();
                        for ref_reg in ref_regs {
                            let reg_val = self.register_stack[curr_reg_base + ref_reg as usize];
                            if let Value::Ref(addr) = reg_val
                                && addr > 0
                            {
                                let _ = self.release_reference(addr);
                            }
                        }

                        let _frame = self
                            .call_frames
                            .pop()
                            .ok_or_else(Self::null_reference_error)?;

                        if self.call_frames.len() <= target_depth {
                            if local_inst_count > 0 {
                                self.instructions_dispatched += local_inst_count;
                            }
                            return Err(RuntimeError::UserException {
                                message: format!("{exc_val}"),
                            });
                        }
                    }
                    continue;
                }
            }
            finish_instruction!();
        }

        if local_inst_count > 0 {
            self.instructions_dispatched += local_inst_count;
        }

        // Final PC synchronization if still running
        if self.call_frames.len() > target_depth {
            let frame = self
                .call_frames
                .last_mut()
                .ok_or_else(Self::null_reference_error)?;
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
        let err = vm.run().expect_err("invalid raw opcode should trap cleanly");
        assert!(matches!(
            err,
            RuntimeError::InvalidOpCode {
                pc: 1,
                opcode: 0xFE
            }
        ));
    }
}
