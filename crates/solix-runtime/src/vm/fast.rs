//! High-performance inlined instruction dispatcher and register window caching.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::exceptions::RuntimeError;
use crate::frame::CallFrame;
use crate::memory::layout::BlockTag;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use solix_common::bytecode::{Instruction, OpCode};

impl VirtualMachine {
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

        let meta = &self.context.function_meta[function_id as usize];
        let binary_id = meta.binary_id;
        let func_id_offset = meta.func_id_offset;

        let reg_base = self
            .call_frames
            .last()
            .map_or(0, |frame| frame.register_base + frame.register_count);
        let num_regs = (func.num_registers as usize).max(arg_count);

        if reg_base + num_regs > self.register_stack.len() {
            self.register_stack
                .resize(reg_base + num_regs + 4096, Value::Null);
        }

        if meta.has_references {
            self.register_stack[reg_base..reg_base + num_regs].fill(Value::Null);
        }

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
                    let idx = inst.imm32() as usize;
                    let val = self.const_pool_cache[binary_id][idx];
                    // SAFETY: Destination register is within frame window
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = val;
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
                        let args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        let result = native_callback(self, &args)?;
                        unsafe {
                            *reg_base.add(caller_dst as usize) = result;
                        }
                        continue;
                    }

                    self.push_frame_fast(global_func_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
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
                        let args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        let result = native_callback(self, &args)?;
                        unsafe {
                            *reg_base.add(caller_dst as usize) = result;
                        }
                    }
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
                        let args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        let result = native_callback(self, &args)?;
                        unsafe {
                            *reg_base.add(caller_dst as usize) = result;
                        }
                        continue;
                    }

                    self.push_frame_fast(global_func_id, caller_dst, caller_arg_start, arg_count)?;
                    reload_frame!();
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
                        let args = self.register_stack
                            [caller_arg_start..caller_arg_start + arg_count]
                            .to_vec();
                        let result = native_callback(self, &args)?;
                        unsafe {
                            *reg_base.add(caller_dst as usize) = result;
                        }
                        continue;
                    }

                    self.push_frame_fast(
                        resolved_func_id,
                        caller_dst,
                        caller_arg_start,
                        arg_count,
                    )?;
                    reload_frame!();
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
                                let args = self.register_stack
                                    [caller_arg_start..caller_arg_start + arg_count]
                                    .to_vec();
                                let result = native_callback(self, &args)?;
                                unsafe {
                                    *reg_base.add(caller_dst as usize) = result;
                                }
                                continue;
                            }
                            self.push_frame_fast(
                                global_func_id,
                                caller_dst,
                                caller_arg_start,
                                arg_count,
                            )?;
                            reload_frame!();
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
                                let args = self.register_stack
                                    [caller_arg_start..caller_arg_start + arg_count]
                                    .to_vec();
                                let result = native_callback(self, &args)?;
                                unsafe {
                                    *reg_base.add(caller_dst as usize) = result;
                                }
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
                        .ok_or(RuntimeError::NullReferenceAccess)?;
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
                // 0x70 .. 0x78: Object Model, Instance Fields & Static Memory
                // =============================================================
                OpCode::AllocObject => {
                    let vtable_id = inst.src1() as u32;
                    let size = inst.src2() as usize;
                    let payload_addr = self.heap.allocate(size, BlockTag::Object)?;
                    self.heap.write_u32(payload_addr, vtable_id)?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::Ref(payload_addr);
                    }
                }

                OpCode::GetField => {
                    let receiver = unsafe { *reg_base.add(inst.src1() as usize) };
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let offset = inst.src2() as usize;
                    let field_addr = obj_addr + offset as u32;
                    let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
                    let maybe_class = self.context.get_class_by_vtable_id(vtable_id);
                    let field_type_tag = maybe_class.and_then(|cls| cls.get_field_type(offset));

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
                            let child_addr = self.heap.read_u64(field_addr).unwrap_or(0) as u32;
                            if child_addr == 0 {
                                Value::Null
                            } else {
                                Value::Ref(child_addr)
                            }
                        }
                        _ => {
                            let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                            if raw == 0 {
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

                    let vtable_id = self.heap.read_u32(obj_addr).unwrap_or(0);
                    let maybe_class = self.context.get_class_by_vtable_id(vtable_id);
                    let field_type_tag = maybe_class.and_then(|cls| cls.get_field_type(offset));

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
                                && !inst.is_weak()
                                && child_addr > 0
                            {
                                crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                            }
                            if let Value::Ref(old_child) = old_val
                                && !inst.is_weak()
                                && old_child > 0
                            {
                                self.release_reference(old_child)?;
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
                    let array_len = self.heap.read_u32(arr_addr)?;
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::I32(array_len as i32);
                    }
                }

                OpCode::GetArray => {
                    let arr_val = unsafe { *reg_base.add(inst.src1() as usize) };
                    let arr_addr = arr_val.as_ref()?;
                    if arr_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let index_i64 = unsafe { *reg_base.add(inst.src2() as usize) }.as_i64()?;
                    let header = self.heap.read_header(arr_addr)?;
                    if header.tag == BlockTag::String {
                        let string_len = header.size as usize;
                        if index_i64 < 0 || (index_i64 as usize) >= string_len {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: index_i64,
                                length: string_len,
                            });
                        }
                        let byte_val = self.heap.read_u8(arr_addr + index_i64 as u32)?;
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::I32(byte_val as i32);
                        }
                    } else {
                        let array_len = self.heap.read_u32(arr_addr)? as usize;
                        if index_i64 < 0 || (index_i64 as usize) >= array_len {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: index_i64,
                                length: array_len,
                            });
                        }
                        let index_usize = index_i64 as usize;
                        let elem_size = self.heap.read_u8(arr_addr + 4)? as usize;
                        let elem_addr = arr_addr + 8 + (index_usize * elem_size) as u32;
                        let raw_word = self.heap.read_u64(elem_addr).unwrap_or(0);
                        let val = if raw_word == 0 {
                            let elem_type_tag = self.heap.read_u8(arr_addr + 7).unwrap_or(0);
                            match elem_type_tag {
                                13 | 0x80..=0xFF => Value::Null,
                                11 => Value::Bool(false),
                                9 | 10 => Value::F64(0.0),
                                _ => Value::I32(0),
                            }
                        } else {
                            Value::from_raw_u64(raw_word)
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
                    let array_len = self.heap.read_u32(arr_addr)? as usize;
                    if index_i64 < 0 || (index_i64 as usize) >= array_len {
                        return Err(RuntimeError::IndexOutOfBounds {
                            index: index_i64,
                            length: array_len,
                        });
                    }
                    let index_usize = index_i64 as usize;
                    let val = unsafe { *reg_base.add(inst.src2() as usize) };
                    let elem_size = self.heap.read_u8(arr_addr + 4)? as usize;
                    let elem_addr = arr_addr + 8 + (index_usize * elem_size) as u32;
                    let old_raw = self.heap.read_u64(elem_addr).unwrap_or(0);
                    let old_val = Value::from_raw_u64(old_raw);
                    self.heap.write_u64(elem_addr, val.to_raw_u64())?;
                    let header = self.heap.read_header(arr_addr)?;
                    let elem_type_tag = self.heap.read_u8(arr_addr + 7).unwrap_or(0);
                    let is_reference_element = header.tag == BlockTag::Tuple
                        || elem_type_tag == 13
                        || elem_type_tag >= 0x80;
                    if is_reference_element {
                        if let Value::Ref(child_addr) = val
                            && child_addr > 0
                        {
                            crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                        }
                        if let Value::Ref(old_child) = old_val
                            && old_child > 0
                        {
                            self.release_reference(old_child)?;
                        }
                    }
                }

                OpCode::AllocTuple => {
                    let tuple_count = inst.src1() as usize;
                    let payload_size = 8 + tuple_count * 8;
                    let payload_addr = self.heap.allocate(payload_size, BlockTag::Tuple)?;
                    self.heap.write_u32(payload_addr, tuple_count as u32)?;
                    self.heap.write_u32(payload_addr + 4, 8)?;
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
                    let elem_addr = tuple_addr + 8 + (slot_idx * 8) as u32;
                    let raw_word = self.heap.read_u64(elem_addr).unwrap_or(0);
                    unsafe {
                        *reg_base.add(inst.dst() as usize) = Value::from_raw_u64(raw_word);
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
                    let elem_addr = tuple_addr + 8 + (index_i64 as usize * 8) as u32;
                    let raw_word = self.heap.read_u64(elem_addr).unwrap_or(0);
                    let val = Value::from_raw_u64(raw_word);
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

                    let payload_size = 8 + cap_count * 8;
                    let payload_addr = self.heap.allocate(payload_size, BlockTag::Closure)?;
                    self.heap.write_u32(payload_addr, target_lambda_id)?;
                    self.heap.write_u32(payload_addr + 4, cap_count as u32)?;

                    for index in 0..cap_count {
                        let cell_addr = payload_addr + 8 + (index * 8) as u32;
                        let cap_val = unsafe { *reg_base.add(cap_base + index) };
                        let _ = self.heap.write_u64(cell_addr, cap_val.to_raw_u64());
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

                OpCode::UnpackCaptures => {
                    let capture_base = inst.arg_base() as usize;
                    let capture_count = inst.arg_count() as usize;

                    let closure_addr = self
                        .current_frame()?
                        .active_closure
                        .ok_or(RuntimeError::NullReferenceAccess)?;

                    for index in 0..capture_count {
                        let cell_addr = closure_addr + 8 + (index * 8) as u32;
                        let raw = self.heap.read_u64(cell_addr).unwrap_or(0);
                        unsafe {
                            *reg_base.add(capture_base + index) = Value::from_raw_u64(raw);
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
                            continue;
                        }
                    };
                    let target_vtable_id = inst.src2() as u32;
                    let header = self.heap.read_header(obj_addr)?;
                    let is_instance = if header.tag == BlockTag::Object {
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
                            12 => matches!(val, Value::I32(_)),
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
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = val;
                        }
                        continue;
                    }

                    let obj_addr = val.as_ref()?;
                    if obj_addr == 0 {
                        unsafe {
                            *reg_base.add(inst.dst() as usize) = Value::Null;
                        }
                        continue;
                    }
                    let target_vtable_id = inst.src2() as u32;
                    let header = self.heap.read_header(obj_addr)?;
                    let is_instance = if header.tag == BlockTag::Object {
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
                        crate::memory::arc::inc_ref(&mut self.heap, addr)?;
                    }
                }

                OpCode::DecRef => {
                    let val = unsafe { *reg_base.add(inst.dst() as usize) };
                    if let Value::Ref(addr) = val
                        && addr > 0
                    {
                        self.release_reference(addr)?;
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
                    let offset = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.dst() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    if val_a == val_b {
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
                }

                OpCode::JumpIfNeqI32 => {
                    let offset = inst.src2() as i16 as i32;
                    let val_a = unsafe { *reg_base.add(inst.dst() as usize) }.as_i32()?;
                    let val_b = unsafe { *reg_base.add(inst.src1() as usize) }.as_i32()?;
                    if val_a != val_b {
                        pc = ((pc as i32 - 1) + offset) as usize;
                    }
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
