//! High-performance inlined instruction dispatcher and register window caching.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::exceptions::RuntimeError;
use crate::frame::CallFrame;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use solix_common::bytecode::{Instruction, OpCode};

impl VirtualMachine {
    /// Checks whether a function is registered as a host native function or builtin callback.
    pub(crate) fn is_native_function(&self, global_func_id: u32) -> bool {
        if let Some(func) = self.context.get_function(global_func_id) {
            let clean = func.symbol.split('(').next().unwrap_or(&func.symbol);
            let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
            let has_qualifier = clean.contains("::");
            self.native_registry.get(&func.symbol).is_some()
                || self.native_registry.get(clean).is_some()
                || (!has_qualifier && self.native_registry.get(simple).is_some())
        } else {
            false
        }
    }

    /// Fast-path frame activation: copies argument registers within `register_stack` without allocating a heap vector.
    ///
    /// # Errors
    /// Returns `RuntimeError::StackOverflow` if recursion depth exceeds configured limit,
    /// or `RuntimeError::FunctionNotFound` if `function_id` is unrecognized.
    pub(crate) fn push_frame_fast(
        &mut self,
        function_id: u32,
        caller_dst: u16,
        caller_arg_start: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        if self.call_frames.len() >= self.context.options.max_call_stack_depth {
            return Err(RuntimeError::StackOverflow {
                current_depth: self.call_frames.len(),
                max_depth: self.context.options.max_call_stack_depth,
            });
        }

        let func = self.context.get_function(function_id).ok_or_else(|| {
            RuntimeError::FunctionNotFound {
                symbol_or_id: format!("#{function_id}"),
            }
        })?;

        let binary_id = self
            .context
            .loaded_binaries
            .iter()
            .position(|b| {
                function_id >= b.function_id_offset
                    && function_id < b.function_id_offset + b.function_count as u32
            })
            .unwrap_or(0);

        let func_id_offset = self
            .context
            .loaded_binaries
            .get(binary_id)
            .map_or(0, |b| b.function_id_offset);

        let reg_base = self.register_stack.len();
        let num_regs = (func.num_registers as usize).max(arg_count);

        self.register_stack.resize(reg_base + num_regs, Value::Null);
        if arg_count > 0 {
            self.register_stack
                .copy_within(caller_arg_start..caller_arg_start + arg_count, reg_base);
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
        if self.halted || self.call_frames.len() <= target_depth {
            return Ok(());
        }

        let mut inst_ptr: *const Instruction;
        let mut inst_len: usize;
        let mut pc: usize;
        let mut static_base: u32;
        let mut binary_id: usize;
        let mut reg_base: *mut Value;

        macro_rules! reload_frame {
            () => {
                let curr = self
                    .call_frames
                    .last()
                    .ok_or(RuntimeError::NullReferenceAccess)?;
                let func = &self.context.all_functions[curr.function_id as usize];
                inst_ptr = func.instructions.as_ptr();
                inst_len = func.instructions.len();
                pc = curr.pc as usize;
                static_base = curr.static_base_offset;
                binary_id = curr.binary_id;
                // SAFETY: curr.register_base is guaranteed within register_stack bounds
                reg_base = unsafe { self.register_stack.as_mut_ptr().add(curr.register_base) };
            };
        }

        reload_frame!();

        while !self.halted {
            if pc >= inst_len {
                let ret_val = Value::Null;
                let frame = self
                    .call_frames
                    .pop()
                    .ok_or(RuntimeError::NullReferenceAccess)?;
                self.register_stack.truncate(frame.register_base);
                if self.call_frames.len() <= target_depth {
                    self.last_return_value = ret_val;
                    break;
                }
                let caller_frame = self
                    .call_frames
                    .last()
                    .ok_or(RuntimeError::NullReferenceAccess)?;
                // SAFETY: caller_dst is within caller frame register window
                unsafe {
                    let caller_reg_base = self
                        .register_stack
                        .as_mut_ptr()
                        .add(caller_frame.register_base);
                    *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                }
                reload_frame!();
                continue;
            }

            // SAFETY: pc < inst_len, so inst_ptr.add(pc) is in-bounds
            let inst = unsafe { *inst_ptr.add(pc) };
            pc += 1;
            self.instructions_dispatched += 1;

            let opcode = inst.opcode(pc).map_err(|_| RuntimeError::InvalidOpCode {
                pc: pc as u32,
                opcode: inst.raw_opcode(),
            })?;
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
                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let idx = inst.imm32() as usize;
                    let c_val = pool.get(idx).ok_or_else(|| RuntimeError::TypeMismatch {
                        expected: "valid float constant pool index".to_string(),
                        found: format!("index {idx} out of pool length {}", pool.len()),
                    })?;
                    let f = c_val.as_f64().ok_or_else(|| RuntimeError::TypeMismatch {
                        expected: "Float64 constant".to_string(),
                        found: format!("{c_val:?}"),
                    })?;
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::F64(f);
                    }
                }

                // =============================================================
                // 0x10 .. 0x20: Arithmetic Operations
                // =============================================================
                OpCode::AddI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(a.wrapping_add(b));
                    }
                }

                OpCode::AddI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_add(b));
                    }
                }

                OpCode::AddF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (a, b) {
                        (Value::F64(x), Value::F64(y)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(x + y);
                        },
                        (x, y) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{x:?}, {y:?}"),
                            });
                        }
                    }
                }

                OpCode::SubI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(a.wrapping_sub(b));
                    }
                }

                OpCode::SubI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_sub(b));
                    }
                }

                OpCode::SubF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (a, b) {
                        (Value::F64(x), Value::F64(y)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(x - y);
                        },
                        (x, y) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{x:?}, {y:?}"),
                            });
                        }
                    }
                }

                OpCode::MulI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(a.wrapping_mul(b));
                    }
                }

                OpCode::MulI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_mul(b));
                    }
                }

                OpCode::MulF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (a, b) {
                        (Value::F64(x), Value::F64(y)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(x * y);
                        },
                        (x, y) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{x:?}, {y:?}"),
                            });
                        }
                    }
                }

                OpCode::DivI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    if b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(a.wrapping_div(b));
                    }
                }

                OpCode::DivI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    if b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_div(b));
                    }
                }

                OpCode::DivF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) };
                    let b = unsafe { *reg_base.add(inst.src2() as usize) };
                    match (a, b) {
                        (Value::F64(x), Value::F64(y)) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(x / y);
                        },
                        (x, y) => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "Float64 and Float64".to_string(),
                                found: format!("{x:?}, {y:?}"),
                            });
                        }
                    }
                }

                OpCode::ModI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    if b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(a.wrapping_rem(b));
                    }
                }

                OpCode::ModI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    if b == 0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_rem(b));
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
                        Value::F64(f) => unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::F64(-f);
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
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a & b);
                    }
                }

                OpCode::Bor => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a | b);
                    }
                }

                OpCode::Bxor => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a ^ b);
                    }
                }

                OpCode::Bnot => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(!a);
                    }
                }

                OpCode::Shl => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    let shift = (b & 63) as u32;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_shl(shift));
                    }
                }

                OpCode::Shr => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    let shift = (b & 63) as u32;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(a.wrapping_shr(shift));
                    }
                }

                // =============================================================
                // 0x30 .. 0x47: Comparisons
                // =============================================================
                OpCode::EqI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a == b);
                    }
                }

                OpCode::EqI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a == b);
                    }
                }

                OpCode::EqF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a == b);
                    }
                }

                OpCode::EqBool => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_bool()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_bool()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a == b);
                    }
                }

                OpCode::EqRef => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_ref()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_ref()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a == b);
                    }
                }

                OpCode::NeqI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a != b);
                    }
                }

                OpCode::NeqI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a != b);
                    }
                }

                OpCode::NeqF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a != b);
                    }
                }

                OpCode::NeqBool => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_bool()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_bool()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a != b);
                    }
                }

                OpCode::NeqRef => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_ref()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_ref()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a != b);
                    }
                }

                OpCode::LtI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a < b);
                    }
                }

                OpCode::LtI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a < b);
                    }
                }

                OpCode::LtF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a < b);
                    }
                }

                OpCode::LteI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a <= b);
                    }
                }

                OpCode::LteI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a <= b);
                    }
                }

                OpCode::LteF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a <= b);
                    }
                }

                OpCode::GtI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a > b);
                    }
                }

                OpCode::GtI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a > b);
                    }
                }

                OpCode::GtF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a > b);
                    }
                }

                OpCode::GteI32 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i32()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a >= b);
                    }
                }

                OpCode::GteI64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a >= b);
                    }
                }

                OpCode::GteF64 => {
                    let a = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    let b = unsafe { *reg_base.add(inst.src2() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(a >= b);
                    }
                }

                OpCode::Not => {
                    let cond = unsafe { *reg_base.add(inst.src1() as usize) }.is_truthy();
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Bool(!cond);
                    }
                }

                OpCode::ConvIToF => {
                    let n = unsafe { *reg_base.add(inst.src1() as usize) }.as_i64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::F64(n as f64);
                    }
                }

                OpCode::ConvFToI => {
                    let f = unsafe { *reg_base.add(inst.src1() as usize) }.as_f64()?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I64(f as i64);
                    }
                }

                // =============================================================
                // 0x50 .. 0x53: Control Flow & Branching
                // =============================================================
                OpCode::Jump | OpCode::Switch => {
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
                // 0x60 .. 0x67: Function Invocations, Returns & Exit
                // =============================================================
                OpCode::CallDirect => {
                    let global_func_id = inst.call_target() as u32;
                    let arg_base = inst.arg_base();
                    let arg_count = inst.arg_count() as usize;
                    let caller_dst = inst.call_dst();

                    if self.is_native_function(global_func_id) {
                        self.current_frame_mut()?.pc = pc as u32;
                        self.dispatch_instruction(inst)?;
                        if self.halted || self.call_frames.len() <= target_depth {
                            break;
                        }
                        reload_frame!();
                        continue;
                    }

                    let caller_frame = self.current_frame_mut()?;
                    caller_frame.pc = pc as u32;
                    let caller_arg_start = caller_frame.register_base + arg_base as usize;

                    self.push_frame_fast(global_func_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
                    continue;
                }

                OpCode::Return => {
                    let ret_val = Value::Null;
                    let frame = self
                        .call_frames
                        .pop()
                        .ok_or(RuntimeError::NullReferenceAccess)?;
                    self.register_stack.truncate(frame.register_base);
                    if self.call_frames.len() <= target_depth {
                        self.last_return_value = ret_val;
                        break;
                    }
                    let caller_frame = self
                        .call_frames
                        .last()
                        .ok_or(RuntimeError::NullReferenceAccess)?;
                    // SAFETY: caller_dst is within caller frame register window
                    unsafe {
                        let caller_reg_base = self
                            .register_stack
                            .as_mut_ptr()
                            .add(caller_frame.register_base);
                        *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                    }
                    reload_frame!();
                    continue;
                }

                OpCode::ReturnValue => {
                    // SAFETY: Return register is within frame window
                    let ret_val = unsafe { *reg_base.add(inst.dst() as usize) };
                    let frame = self
                        .call_frames
                        .pop()
                        .ok_or(RuntimeError::NullReferenceAccess)?;
                    self.register_stack.truncate(frame.register_base);
                    if self.call_frames.len() <= target_depth {
                        self.last_return_value = ret_val;
                        break;
                    }
                    let caller_frame = self
                        .call_frames
                        .last()
                        .ok_or(RuntimeError::NullReferenceAccess)?;
                    // SAFETY: caller_dst is within caller frame register window
                    unsafe {
                        let caller_reg_base = self
                            .register_stack
                            .as_mut_ptr()
                            .add(caller_frame.register_base);
                        *caller_reg_base.add(frame.caller_dst as usize) = ret_val;
                    }
                    reload_frame!();
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
                // 0x73 .. 0x78: Static Variables
                // =============================================================
                OpCode::GetStaticDirect => {
                    let slot = inst.imm32() as usize;
                    if slot >= self.static_slots.len() {
                        return Err(RuntimeError::InvalidStaticOffset {
                            offset: slot as u32,
                            total_static: self.static_slots.len() as u32,
                        });
                    }
                    // SAFETY: Slot is verified in bounds
                    let val = unsafe { *self.static_slots.get_unchecked(slot) };
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = val;
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
                    let val = unsafe { *self.static_slots.get_unchecked(slot) };
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = val;
                    }
                }

                OpCode::SetStaticDirect => {
                    let slot = inst.imm32();
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    self.set_static(slot, val)?;
                }

                // =============================================================
                // 0xC0 .. 0xC1: ARC Reference Counting
                // =============================================================
                OpCode::IncRef => {
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    if let Value::Ref(addr) = val
                        && addr > 0
                    {
                        crate::memory::arc::inc_ref(&mut self.heap, addr)?;
                    }
                }

                OpCode::DecRef => {
                    self.current_frame_mut()?.pc = pc as u32;
                    self.dispatch_instruction(inst)?;
                    if self.halted || self.call_frames.len() <= target_depth {
                        break;
                    }
                    reload_frame!();
                    continue;
                }

                // =============================================================
                // All other opcodes: synchronize frame PC and dispatch defensively
                // =============================================================
                _ => {
                    self.current_frame_mut()?.pc = pc as u32;
                    self.dispatch_instruction(inst)?;
                    if self.halted || self.call_frames.len() <= target_depth {
                        break;
                    }
                    reload_frame!();
                    continue;
                }
            }
        }

        // Final PC synchronization if still running
        if self.call_frames.len() > target_depth {
            let frame = self
                .call_frames
                .last_mut()
                .ok_or(RuntimeError::NullReferenceAccess)?;
            frame.pc = pc as u32;
        }

        Ok(())
    }
}
