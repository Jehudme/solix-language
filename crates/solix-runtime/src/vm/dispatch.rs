//! Virtual machine opcode instruction dispatch.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::exceptions::RuntimeError;
use crate::memory::BlockTag;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use solix_common::bytecode::{ConstantPoolValue, Instruction, OpCode};

impl VirtualMachine {
    /// Dispatches a single virtual machine instruction.
    pub fn dispatch_instruction(&mut self, inst: Instruction) -> Result<(), RuntimeError> {
        let opcode =
            inst.opcode(self.current_pc() as usize)
                .map_err(|_| RuntimeError::InvalidOpCode {
                    pc: self.current_pc(),
                    opcode: inst.raw_opcode(),
                })?;

        match opcode {
            // 0x00: Nop
            OpCode::Nop => Ok(()),

            // 0x01: Mov
            OpCode::Mov => {
                let val = self.get_reg(inst.src1())?;
                self.set_reg(inst.dst(), val)
            }

            // 0x02: LoadI32
            OpCode::LoadI32 => {
                let val = Value::I32(inst.imm32_signed());
                self.set_reg(inst.dst(), val)
            }

            // 0x03: LoadI64
            OpCode::LoadI64 => {
                let val = Value::I64(inst.imm32_signed() as i64);
                self.set_reg(inst.dst(), val)
            }

            // 0x04: LoadF64
            OpCode::LoadF64 => {
                let binary_id = self.current_frame()?.binary_id;
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
                self.set_reg(inst.dst(), Value::F64(f))
            }

            // 0x05: LoadBool
            OpCode::LoadBool => {
                let val = Value::Bool(inst.imm32() != 0);
                self.set_reg(inst.dst(), val)
            }

            // 0x06: LoadNull
            OpCode::LoadNull => self.set_reg(inst.dst(), Value::Null),

            // 0x07: LoadConst
            OpCode::LoadConst => {
                let binary_id = self.current_frame()?.binary_id;
                let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                    RuntimeError::BinaryLoadError {
                        reason: format!("constant pool missing for binary {binary_id}"),
                    }
                })?;
                let idx = inst.imm32() as usize;
                let c_val = pool.get(idx).ok_or_else(|| RuntimeError::TypeMismatch {
                    expected: "valid constant pool index".to_string(),
                    found: format!("index {idx} out of pool length {}", pool.len()),
                })?;

                let val = match c_val {
                    ConstantPoolValue::Int64(i) => Value::I64(*i),
                    ConstantPoolValue::Float64(bits) => Value::F64(f64::from_bits(*bits)),
                    ConstantPoolValue::String(s) => {
                        let bytes = s.as_bytes();
                        let addr = self.heap.allocate(bytes.len(), BlockTag::String)?;
                        self.heap.write_bytes(addr, bytes)?;
                        Value::Ref(addr)
                    }
                };
                self.set_reg(inst.dst(), val)
            }

            // =================================================================
            // 0x10 .. 0x20: Arithmetic Operations
            // =================================================================
            OpCode::AddI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::I32(a.wrapping_add(b)))
            }

            OpCode::AddI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a.wrapping_add(b)))
            }

            OpCode::AddF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::F64(a + b))
            }

            OpCode::SubI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::I32(a.wrapping_sub(b)))
            }

            OpCode::SubI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a.wrapping_sub(b)))
            }

            OpCode::SubF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::F64(a - b))
            }

            OpCode::MulI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::I32(a.wrapping_mul(b)))
            }

            OpCode::MulI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a.wrapping_mul(b)))
            }

            OpCode::MulF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::F64(a * b))
            }

            OpCode::DivI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                if b == 0 {
                    return Err(RuntimeError::DivideByZero);
                }
                self.set_reg(inst.dst(), Value::I32(a.wrapping_div(b)))
            }

            OpCode::DivI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                if b == 0 {
                    return Err(RuntimeError::DivideByZero);
                }
                self.set_reg(inst.dst(), Value::I64(a.wrapping_div(b)))
            }

            OpCode::DivF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::F64(a / b))
            }

            OpCode::ModI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                if b == 0 {
                    return Err(RuntimeError::DivideByZero);
                }
                self.set_reg(inst.dst(), Value::I32(a.wrapping_rem(b)))
            }

            OpCode::ModI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                if b == 0 {
                    return Err(RuntimeError::DivideByZero);
                }
                self.set_reg(inst.dst(), Value::I64(a.wrapping_rem(b)))
            }

            OpCode::NegI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                self.set_reg(inst.dst(), Value::I32(a.wrapping_neg()))
            }

            OpCode::NegI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a.wrapping_neg()))
            }

            OpCode::NegF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                self.set_reg(inst.dst(), Value::F64(-a))
            }

            // =================================================================
            // 0x21 .. 0x26: Bitwise Operations
            // =================================================================
            OpCode::Band => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a & b))
            }

            OpCode::Bor => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a | b))
            }

            OpCode::Bxor => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a ^ b))
            }

            OpCode::Bnot => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(!a))
            }

            OpCode::Shl => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a << (b & 63)))
            }

            OpCode::Shr => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::I64(a >> (b & 63)))
            }

            // =================================================================
            // 0x30 .. 0x47: Comparisons
            // =================================================================
            OpCode::EqI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::Bool(a == b))
            }

            OpCode::EqI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::Bool(a == b))
            }

            OpCode::EqF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::Bool(a == b))
            }

            OpCode::EqBool => {
                let a = self.get_reg(inst.src1())?.as_bool()?;
                let b = self.get_reg(inst.src2())?.as_bool()?;
                self.set_reg(inst.dst(), Value::Bool(a == b))
            }

            OpCode::EqString => {
                let addr1 = self.get_reg(inst.src1())?.as_ref()?;
                let addr2 = self.get_reg(inst.src2())?.as_ref()?;
                let is_eq = if addr1 == addr2 {
                    true
                } else if addr1 == 0 || addr2 == 0 {
                    false
                } else {
                    let h1 = self.heap.read_header(addr1)?;
                    let h2 = self.heap.read_header(addr2)?;
                    if h1.size != h2.size {
                        false
                    } else {
                        let b1 = self.heap.read_bytes(addr1, h1.size as usize)?;
                        let b2 = self.heap.read_bytes(addr2, h2.size as usize)?;
                        b1 == b2
                    }
                };
                self.set_reg(inst.dst(), Value::Bool(is_eq))
            }

            OpCode::EqRef => {
                let a = self.get_reg(inst.src1())?.as_ref()?;
                let b = self.get_reg(inst.src2())?.as_ref()?;
                self.set_reg(inst.dst(), Value::Bool(a == b))
            }

            OpCode::NeqI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::Bool(a != b))
            }

            OpCode::NeqI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::Bool(a != b))
            }

            OpCode::NeqF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::Bool(a != b))
            }

            OpCode::NeqBool => {
                let a = self.get_reg(inst.src1())?.as_bool()?;
                let b = self.get_reg(inst.src2())?.as_bool()?;
                self.set_reg(inst.dst(), Value::Bool(a != b))
            }

            OpCode::NeqString => {
                let addr1 = self.get_reg(inst.src1())?.as_ref()?;
                let addr2 = self.get_reg(inst.src2())?.as_ref()?;
                let is_eq = if addr1 == addr2 {
                    true
                } else if addr1 == 0 || addr2 == 0 {
                    false
                } else {
                    let h1 = self.heap.read_header(addr1)?;
                    let h2 = self.heap.read_header(addr2)?;
                    if h1.size != h2.size {
                        false
                    } else {
                        let b1 = self.heap.read_bytes(addr1, h1.size as usize)?;
                        let b2 = self.heap.read_bytes(addr2, h2.size as usize)?;
                        b1 == b2
                    }
                };
                self.set_reg(inst.dst(), Value::Bool(!is_eq))
            }

            OpCode::NeqRef => {
                let a = self.get_reg(inst.src1())?.as_ref()?;
                let b = self.get_reg(inst.src2())?.as_ref()?;
                self.set_reg(inst.dst(), Value::Bool(a != b))
            }

            OpCode::LtI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::Bool(a < b))
            }

            OpCode::LtI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::Bool(a < b))
            }

            OpCode::LtF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::Bool(a < b))
            }

            OpCode::LteI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::Bool(a <= b))
            }

            OpCode::LteI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::Bool(a <= b))
            }

            OpCode::LteF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::Bool(a <= b))
            }

            OpCode::GtI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::Bool(a > b))
            }

            OpCode::GtI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::Bool(a > b))
            }

            OpCode::GtF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::Bool(a > b))
            }

            OpCode::GteI32 => {
                let a = self.get_reg(inst.src1())?.as_i32()?;
                let b = self.get_reg(inst.src2())?.as_i32()?;
                self.set_reg(inst.dst(), Value::Bool(a >= b))
            }

            OpCode::GteI64 => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                let b = self.get_reg(inst.src2())?.as_i64()?;
                self.set_reg(inst.dst(), Value::Bool(a >= b))
            }

            OpCode::GteF64 => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                let b = self.get_reg(inst.src2())?.as_f64()?;
                self.set_reg(inst.dst(), Value::Bool(a >= b))
            }

            // =================================================================
            // 0x48: Boolean Inversion & Conversions
            // =================================================================
            OpCode::Not => {
                let a = self.get_reg(inst.src1())?;
                self.set_reg(inst.dst(), Value::Bool(!a.is_truthy()))
            }

            OpCode::ConvIToF => {
                let a = self.get_reg(inst.src1())?.as_i64()?;
                self.set_reg(inst.dst(), Value::F64(a as f64))
            }

            OpCode::ConvFToI => {
                let a = self.get_reg(inst.src1())?.as_f64()?;
                self.set_reg(inst.dst(), Value::I64(a as i64))
            }

            // 0x73: GetStatic
            OpCode::GetStatic => {
                let base = self.current_frame()?.static_base_offset;
                let slot = base + inst.imm32();
                let val = self.get_static(slot)?;
                self.set_reg(inst.dst(), val)
            }

            // 0x74: SetStatic
            OpCode::SetStatic => {
                let base = self.current_frame()?.static_base_offset;
                let slot = base + inst.imm32();
                let val = self.get_reg(inst.dst())?;
                self.set_static(slot, val)
            }

            // 0x75: GetStaticGlobal
            OpCode::GetStaticGlobal => {
                let (caller_func_id, caller_pc, binary_id) = {
                    let frame = self.current_frame()?;
                    (
                        frame.function_id,
                        frame.pc.saturating_sub(1) as usize,
                        frame.binary_id,
                    )
                };
                let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                    RuntimeError::BinaryLoadError {
                        reason: format!("constant pool missing for binary {binary_id}"),
                    }
                })?;
                let idx = inst.imm32() as usize;
                let c_val = pool.get(idx).ok_or_else(|| RuntimeError::TypeMismatch {
                    expected: "symbol string in constant pool".to_string(),
                    found: format!("index {idx}"),
                })?;
                let sym = match c_val {
                    ConstantPoolValue::String(s) => s.as_str(),
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "string symbol".to_string(),
                            found: format!("{c_val:?}"),
                        });
                    }
                };
                let slot = self.context.resolve_static_slot(sym).ok_or(
                    RuntimeError::InvalidStaticOffset {
                        offset: idx as u32,
                        total_static: self.static_slots.len() as u32,
                    },
                )?;

                // In-place quickening to GetStaticDirect
                let quickened = Instruction::format_b(OpCode::GetStaticDirect, 0, inst.dst(), slot);
                self.context
                    .patch_instruction(caller_func_id, caller_pc, quickened);

                let val = self.get_static(slot)?;
                self.set_reg(inst.dst(), val)
            }

            // 0x76: GetStaticDirect
            OpCode::GetStaticDirect => {
                let slot = inst.imm32();
                let val = self.get_static(slot)?;
                self.set_reg(inst.dst(), val)
            }

            // =================================================================
            // 0x50 .. 0x53: Control Flow & Branching
            // =================================================================
            OpCode::Jump => {
                let offset = inst.jump_offset();
                let frame = self.current_frame_mut()?;
                let target_pc = (frame.pc as i32 - 1) + offset;
                frame.pc = target_pc as u32;
                Ok(())
            }

            OpCode::JumpIfTrue => {
                let cond = self.get_reg(inst.cond_reg())?.is_truthy();
                if cond {
                    let offset = inst.jump_offset();
                    let frame = self.current_frame_mut()?;
                    let target_pc = (frame.pc as i32 - 1) + offset;
                    frame.pc = target_pc as u32;
                }
                Ok(())
            }

            OpCode::JumpIfFalse => {
                let cond = self.get_reg(inst.cond_reg())?.is_truthy();
                if !cond {
                    let offset = inst.jump_offset();
                    let frame = self.current_frame_mut()?;
                    let target_pc = (frame.pc as i32 - 1) + offset;
                    frame.pc = target_pc as u32;
                }
                Ok(())
            }

            OpCode::Switch => {
                let offset = inst.jump_offset();
                let frame = self.current_frame_mut()?;
                let target_pc = (frame.pc as i32 - 1) + offset;
                frame.pc = target_pc as u32;
                Ok(())
            }

            // =================================================================
            // 0x60 .. 0x64: Function Invocations & Calls
            // =================================================================
            OpCode::CallDirect => {
                let global_func_id = inst.call_target() as u32;
                let arg_base = inst.arg_base();
                let arg_count = inst.arg_count() as usize;
                let caller_dst = inst.call_dst();

                let mut args = Vec::with_capacity(arg_count);
                for i in 0..arg_count {
                    args.push(self.get_reg(arg_base + i as u16)?);
                }

                if let Some(func) = self.context.get_function(global_func_id) {
                    let clean = func.symbol.split('(').next().unwrap_or(&func.symbol);
                    let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
                    let has_qualifier = clean.contains("::");
                    if let Some(&native_fn) = self
                        .native_registry
                        .get(&func.symbol)
                        .or_else(|| self.native_registry.get(clean))
                        .or_else(|| {
                            if !has_qualifier {
                                self.native_registry.get(simple)
                            } else {
                                None
                            }
                        })
                    {
                        let result = native_fn(self, &args)?;
                        self.set_reg(caller_dst, result)?;
                        return Ok(());
                    }
                }

                self.push_frame(global_func_id, caller_dst, &args)
            }

            OpCode::CallSymbol => {
                let const_idx = inst.call_target() as usize;
                let arg_base = inst.arg_base();
                let arg_count = inst.arg_count() as usize;
                let caller_dst = inst.call_dst();

                let (sym, args) = {
                    let frame = self.current_frame()?;
                    let binary_id = frame.binary_id;
                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let c_val = pool
                        .get(const_idx)
                        .ok_or_else(|| RuntimeError::TypeMismatch {
                            expected: "symbol string in constant pool".to_string(),
                            found: format!("index {const_idx}"),
                        })?;
                    let s = match c_val {
                        ConstantPoolValue::String(s) => s.clone(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{c_val:?}"),
                            });
                        }
                    };

                    let mut args = Vec::with_capacity(arg_count);
                    for i in 0..arg_count {
                        args.push(self.get_reg(arg_base + i as u16)?);
                    }
                    (s, args)
                };

                // Check native registry first
                if let Some(&native_fn) = self.native_registry.get(&sym) {
                    let result = native_fn(self, &args)?;
                    self.set_reg(caller_dst, result)?;
                    return Ok(());
                }

                // Fallback to compiled guest function
                let global_id = self
                    .context
                    .symbol_to_function_id
                    .get(&sym)
                    .copied()
                    .ok_or(RuntimeError::FunctionNotFound { symbol_or_id: sym })?;

                self.push_frame(global_id, caller_dst, &args)
            }

            OpCode::CallCallable => {
                let r_callee = inst.call_target();
                let arg_base = inst.arg_base();
                let arg_count = inst.arg_count() as usize;
                let caller_dst = inst.call_dst();

                let callee_val = self.get_reg(r_callee)?;
                let mut args = Vec::with_capacity(arg_count);
                for i in 0..arg_count {
                    args.push(self.get_reg(arg_base + i as u16)?);
                }

                match callee_val {
                    Value::Null => Err(RuntimeError::NullReferenceAccess),
                    Value::FnPtr(func_id) => self.push_frame(func_id, caller_dst, &args),
                    Value::Ref(closure_addr) => {
                        if closure_addr == 0 {
                            return Err(RuntimeError::NullReferenceAccess);
                        }
                        let func_id = self.heap.read_u32(closure_addr)?;
                        self.push_closure_frame(func_id, closure_addr, caller_dst, &args)
                    }
                    _ => Err(RuntimeError::TypeMismatch {
                        expected: "callable (fn_ptr or closure ref)".to_string(),
                        found: callee_val.type_name().to_string(),
                    }),
                }
            }

            // =================================================================
            // 0x90 .. 0x91: Closures & Lambdas
            // =================================================================
            OpCode::PackClosure => {
                let target_lambda_id = inst.call_target() as u32;
                let cap_base = inst.arg_base();
                let cap_count = inst.arg_count() as usize;
                let dst = inst.call_dst();

                let mut captures = Vec::with_capacity(cap_count);
                for i in 0..cap_count {
                    captures.push(self.get_reg(cap_base + i as u16)?);
                }

                let payload_size = 8 + cap_count * 8;
                let payload_addr = self.heap.allocate(payload_size, BlockTag::Closure)?;
                self.heap.write_u32(payload_addr, target_lambda_id)?;
                self.heap.write_u32(payload_addr + 4, cap_count as u32)?;

                for (idx, cap_val) in captures.iter().enumerate() {
                    self.closure_captures.insert((payload_addr, idx), *cap_val);
                    let cell_addr = payload_addr + 8 + (idx * 8) as u32;
                    let _ = self.heap.write_u64(cell_addr, cap_val.to_raw_u64());
                    if let Value::Ref(child_addr) = cap_val
                        && *child_addr > 0
                    {
                        crate::memory::arc::inc_ref(&mut self.heap, *child_addr)?;
                    }
                }

                self.set_reg(dst, Value::Ref(payload_addr))
            }

            OpCode::UnpackCaptures => {
                let capture_base = inst.arg_base();
                let capture_count = inst.arg_count() as usize;

                let closure_addr = self
                    .current_frame()?
                    .active_closure
                    .ok_or(RuntimeError::NullReferenceAccess)?;

                for i in 0..capture_count {
                    let val = self
                        .closure_captures
                        .get(&(closure_addr, i))
                        .copied()
                        .unwrap_or(Value::Null);
                    self.set_reg(capture_base + i as u16, val)?;
                }
                Ok(())
            }

            OpCode::CallVirtual => {
                let slot = inst.call_target() as usize;
                let arg_base = inst.arg_base();
                let arg_count = inst.arg_count() as usize;
                let caller_dst = inst.call_dst();

                let (global_func_id, args) = {
                    let mut args = Vec::with_capacity(arg_count);
                    for i in 0..arg_count {
                        args.push(self.get_reg(arg_base + i as u16)?);
                    }
                    let receiver = args.first().ok_or(RuntimeError::NullReferenceAccess)?;
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let vtable_id = self.heap.read_u32(obj_addr)?;
                    let class = self
                        .context
                        .all_classes
                        .iter()
                        .find(|c| c.vtable_id == vtable_id)
                        .ok_or_else(|| RuntimeError::FunctionNotFound {
                            symbol_or_id: format!("vtable #{vtable_id}"),
                        })?;
                    let func_id = class.vtable.get(slot).copied().ok_or_else(|| {
                        RuntimeError::FunctionNotFound {
                            symbol_or_id: format!("vtable slot {slot} in class {}", class.symbol),
                        }
                    })?;
                    (func_id, args)
                };

                if let Some(func) = self.context.get_function(global_func_id) {
                    let clean = func.symbol.split('(').next().unwrap_or(&func.symbol);
                    let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
                    let has_qualifier = clean.contains("::");
                    if let Some(&native_fn) = self
                        .native_registry
                        .get(&func.symbol)
                        .or_else(|| self.native_registry.get(clean))
                        .or_else(|| {
                            if !has_qualifier {
                                self.native_registry.get(simple)
                            } else {
                                None
                            }
                        })
                    {
                        let result = native_fn(self, &args)?;
                        self.set_reg(caller_dst, result)?;
                        return Ok(());
                    }
                }

                self.push_frame(global_func_id, caller_dst, &args)
            }

            OpCode::CallInterface => {
                let target = inst.call_target();
                let iface_id = (target >> 8) as u32;
                let slot = (target & 0xFF) as usize;
                let arg_base = inst.arg_base();
                let arg_count = inst.arg_count() as usize;
                let caller_dst = inst.call_dst();

                let (global_func_id, args) = {
                    let mut args = Vec::with_capacity(arg_count);
                    for i in 0..arg_count {
                        args.push(self.get_reg(arg_base + i as u16)?);
                    }
                    let receiver = args.first().ok_or(RuntimeError::NullReferenceAccess)?;
                    let obj_addr = receiver.as_ref()?;
                    if obj_addr == 0 {
                        return Err(RuntimeError::NullReferenceAccess);
                    }
                    let vtable_id = self.heap.read_u32(obj_addr)?;
                    let class = self
                        .context
                        .all_classes
                        .iter()
                        .find(|c| c.vtable_id == vtable_id)
                        .ok_or_else(|| RuntimeError::FunctionNotFound {
                            symbol_or_id: format!("vtable #{vtable_id}"),
                        })?;

                    let func_id = if iface_id != 0 {
                        class
                            .itable
                            .iter()
                            .find(|(id, _)| *id == iface_id)
                            .and_then(|(_, methods)| methods.get(slot).copied())
                    } else {
                        None
                    }
                    .or_else(|| {
                        class
                            .itable
                            .iter()
                            .find_map(|(_, methods)| methods.get(slot).copied())
                            .or_else(|| class.vtable.get(slot).copied())
                    })
                    .ok_or_else(|| RuntimeError::FunctionNotFound {
                        symbol_or_id: format!("interface slot {slot} in class {}", class.symbol),
                    })?;

                    (func_id, args)
                };

                if let Some(func) = self.context.get_function(global_func_id) {
                    let clean = func.symbol.split('(').next().unwrap_or(&func.symbol);
                    let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
                    let has_qualifier = clean.contains("::");
                    if let Some(&native_fn) = self
                        .native_registry
                        .get(&func.symbol)
                        .or_else(|| self.native_registry.get(clean))
                        .or_else(|| {
                            if !has_qualifier {
                                self.native_registry.get(simple)
                            } else {
                                None
                            }
                        })
                    {
                        let result = native_fn(self, &args)?;
                        self.set_reg(caller_dst, result)?;
                        return Ok(());
                    }
                }

                self.push_frame(global_func_id, caller_dst, &args)
            }

            // =================================================================
            // 0x70 .. 0x72: Object Model & Instance Fields
            // =================================================================
            OpCode::AllocObject => {
                let vtable_id = inst.src1() as u32;
                let size = (inst.src2() as usize).max(8);
                let payload_addr = self.heap.allocate(size, BlockTag::Object)?;
                self.heap.write_u32(payload_addr, vtable_id)?;
                self.set_reg(inst.dst(), Value::Ref(payload_addr))
            }

            OpCode::GetField => {
                let obj_addr = self.get_reg(inst.src1())?.as_ref()?;
                if obj_addr == 0 {
                    return Err(RuntimeError::NullReferenceAccess);
                }
                let offset = inst.src2() as usize;
                let val = if let Some(&v) = self.object_fields.get(&(obj_addr, offset)) {
                    v
                } else {
                    let field_addr = obj_addr + offset as u32;
                    let raw = self.heap.read_u64(field_addr).unwrap_or(0);
                    Value::I64(raw as i64)
                };
                self.set_reg(inst.dst(), val)
            }

            OpCode::SetField => {
                let obj_addr = self.get_reg(inst.src1())?.as_ref()?;
                if obj_addr == 0 {
                    return Err(RuntimeError::NullReferenceAccess);
                }
                let offset = inst.src2() as usize;
                let val = self.get_reg(inst.dst())?;
                let old_val = self.object_fields.insert((obj_addr, offset), val);
                let field_addr = obj_addr + offset as u32;
                let _ = self.heap.write_u64(field_addr, val.to_raw_u64());
                if let Value::Ref(child_addr) = val
                    && !inst.is_weak()
                    && child_addr > 0
                {
                    crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                }
                if let Some(Value::Ref(old_child)) = old_val
                    && !inst.is_weak()
                    && old_child > 0
                {
                    self.release_reference(old_child)?;
                }
                Ok(())
            }

            // =================================================================
            // 0xA0 .. 0xA1: Nominal Inheritance & Type Casts
            // =================================================================
            OpCode::Instanceof => {
                let obj_addr = self.get_reg(inst.src1())?.as_ref()?;
                if obj_addr == 0 {
                    self.set_reg(inst.dst(), Value::Bool(false))?;
                    return Ok(());
                }
                let target_vtable_id = inst.src2() as u32;
                let header = self.heap.read_header(obj_addr)?;
                let is_inst = if header.tag == BlockTag::Object {
                    let obj_vtable_id = self.heap.read_u32(obj_addr)?;
                    self.is_instance_of(obj_vtable_id, target_vtable_id)
                } else {
                    false
                };
                self.set_reg(inst.dst(), Value::Bool(is_inst))
            }

            OpCode::CastCheck => {
                let obj_addr = self.get_reg(inst.src1())?.as_ref()?;
                if obj_addr == 0 {
                    self.set_reg(inst.dst(), Value::Null)?;
                    return Ok(());
                }
                let target_vtable_id = inst.src2() as u32;
                let header = self.heap.read_header(obj_addr)?;
                let is_inst = if header.tag == BlockTag::Object {
                    let obj_vtable_id = self.heap.read_u32(obj_addr)?;
                    self.is_instance_of(obj_vtable_id, target_vtable_id)
                } else {
                    false
                };
                if !is_inst {
                    return Err(RuntimeError::InvalidCast {
                        from: format!("ref 0x{obj_addr:08X}"),
                        to: format!("vtable #{target_vtable_id}"),
                    });
                }
                self.set_reg(inst.dst(), Value::Ref(obj_addr))
            }

            // =================================================================
            // 0x80 .. 0x85 & 0xA4: Dynamic Arrays & Tuples
            // =================================================================
            OpCode::AllocArray => {
                let raw_len = self.get_reg(inst.src1())?.as_i64()?;
                if raw_len < 0 {
                    return Err(RuntimeError::OutOfMemory {
                        requested: usize::MAX,
                        available: self.heap.dynamic_capacity(),
                    });
                }
                let len = raw_len as usize;
                let elem_size = (inst.src2() as usize).max(1);
                let payload_size = 8 + len * elem_size;
                let payload_addr = self.heap.allocate(payload_size, BlockTag::ArrayRef)?;
                self.heap.write_u32(payload_addr, len as u32)?;
                self.heap.write_u8(payload_addr + 4, elem_size as u8)?;
                self.heap.write_u8(payload_addr + 7, inst.flags())?;
                self.set_reg(inst.dst(), Value::Ref(payload_addr))
            }

            OpCode::GetArray => {
                let arr_addr = self.get_reg(inst.src1())?.as_ref()?;
                if arr_addr == 0 {
                    return Err(RuntimeError::NullReferenceAccess);
                }
                let idx = self.get_reg(inst.src2())?.as_i64()?;
                let header = self.heap.read_header(arr_addr)?;
                if header.tag == BlockTag::String {
                    let len = header.size as usize;
                    if idx < 0 || (idx as usize) >= len {
                        return Err(RuntimeError::IndexOutOfBounds {
                            index: idx,
                            length: len,
                        });
                    }
                    let byte = self.heap.read_u8(arr_addr + idx as u32)?;
                    return self.set_reg(inst.dst(), Value::I32(byte as i32));
                }
                let len = self.heap.read_u32(arr_addr)? as usize;
                if idx < 0 || (idx as usize) >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: idx,
                        length: len,
                    });
                }
                let idx_usize = idx as usize;
                let val = if let Some(&v) = self.array_elements.get(&(arr_addr, idx_usize)) {
                    v
                } else {
                    let tag = self.heap.read_u8(arr_addr + 7).unwrap_or(0);
                    match tag {
                        0 | 13 | 0x80..=0xFF => Value::Null,
                        11 => Value::Bool(false),
                        1 | 2 | 3 | 5 | 6 | 7 | 12 => Value::I32(0),
                        4 | 8 => Value::I64(0),
                        9 | 10 => Value::F64(0.0),
                        _ => Value::I32(0),
                    }
                };
                self.set_reg(inst.dst(), val)
            }

            OpCode::SetArray => {
                let arr_addr = self.get_reg(inst.dst())?.as_ref()?;
                if arr_addr == 0 {
                    return Err(RuntimeError::NullReferenceAccess);
                }
                let idx = self.get_reg(inst.src1())?.as_i64()?;
                let len = self.heap.read_u32(arr_addr)? as usize;
                if idx < 0 || (idx as usize) >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: idx,
                        length: len,
                    });
                }
                let idx_usize = idx as usize;
                let val = self.get_reg(inst.src2())?;
                let old_val = self.array_elements.insert((arr_addr, idx_usize), val);
                let elem_size = self.heap.read_u8(arr_addr + 4)? as usize;
                let elem_addr = arr_addr + 8 + (idx_usize * elem_size) as u32;
                let _ = self.heap.write_u64(elem_addr, val.to_raw_u64());
                if let Value::Ref(child_addr) = val
                    && child_addr > 0
                {
                    crate::memory::arc::inc_ref(&mut self.heap, child_addr)?;
                }
                if let Some(Value::Ref(old_child)) = old_val
                    && old_child > 0
                {
                    self.release_reference(old_child)?;
                }
                Ok(())
            }

            OpCode::ArrayLength => {
                let arr_addr = self.get_reg(inst.src1())?.as_ref()?;
                if arr_addr == 0 {
                    return Err(RuntimeError::NullReferenceAccess);
                }
                let len = self.heap.read_u32(arr_addr)?;
                self.set_reg(inst.dst(), Value::I32(len as i32))
            }

            OpCode::AllocTuple => {
                let count = inst.src1() as usize;
                let payload_size = 8 + count * 8;
                let payload_addr = self.heap.allocate(payload_size, BlockTag::Tuple)?;
                self.heap.write_u32(payload_addr, count as u32)?;
                self.heap.write_u32(payload_addr + 4, 8)?;
                self.set_reg(inst.dst(), Value::Ref(payload_addr))
            }

            OpCode::GetTuple => {
                let tuple_addr = self.get_reg(inst.src1())?.as_ref()?;
                if tuple_addr == 0 {
                    return Err(RuntimeError::NullReferenceAccess);
                }
                let slot_idx = inst.src2() as usize;
                let len = self.heap.read_u32(tuple_addr)? as usize;
                if slot_idx >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: slot_idx as i64,
                        length: len,
                    });
                }
                let val = if let Some(&v) = self.array_elements.get(&(tuple_addr, slot_idx)) {
                    v
                } else {
                    let elem_addr = tuple_addr + 8 + (slot_idx * 8) as u32;
                    let raw = self.heap.read_u64(elem_addr).unwrap_or(0);
                    Value::I64(raw as i64)
                };
                self.set_reg(inst.dst(), val)
            }

            OpCode::TupleInstanceof => {
                let tuple_addr = self.get_reg(inst.src1())?.as_ref()?;
                if tuple_addr == 0 {
                    self.set_reg(inst.dst(), Value::Bool(false))?;
                    return Ok(());
                }
                let idx = self.get_reg(inst.src2())?.as_i64()?;
                let len = self.heap.read_u32(tuple_addr)? as usize;
                if idx < 0 || (idx as usize) >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: idx,
                        length: len,
                    });
                }
                let val = self
                    .array_elements
                    .get(&(tuple_addr, idx as usize))
                    .copied()
                    .unwrap_or(Value::Null);
                let tag = inst.flags();
                let is_match = match tag {
                    1..=4 => matches!(val, Value::I32(_) | Value::I64(_)),
                    5..=8 => matches!(val, Value::I32(_) | Value::I64(_)),
                    9..=10 => matches!(val, Value::F64(_)),
                    11 => matches!(val, Value::Bool(_)),
                    13 => {
                        if let Value::Ref(a) = val {
                            self.heap
                                .read_header(a)
                                .is_ok_and(|h| h.tag == BlockTag::String)
                        } else {
                            false
                        }
                    }
                    _ if (tag & 0x80) != 0 => {
                        let target_vtable_id = (tag & 0x7F) as u32;
                        if let Value::Ref(a) = val {
                            let obj_vtable_id = self.heap.read_u32(a).unwrap_or(0);
                            self.is_instance_of(obj_vtable_id, target_vtable_id)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                self.set_reg(inst.dst(), Value::Bool(is_match))
            }

            // =================================================================
            // 0xB0: Exception Handling
            // =================================================================
            OpCode::Throw => {
                let exc_val = self.get_reg(inst.dst())?;
                if exc_val.is_null() {
                    return Err(RuntimeError::NullReferenceAccess);
                }

                loop {
                    if self.call_frames.is_empty() {
                        return Err(RuntimeError::UserException {
                            message: format!("{exc_val}"),
                        });
                    }

                    let (func_id, pc) = {
                        let frame = self.current_frame()?;
                        (frame.function_id, frame.pc.saturating_sub(1))
                    };

                    let func = self.context.get_function(func_id).ok_or_else(|| {
                        RuntimeError::FunctionNotFound {
                            symbol_or_id: format!("#{func_id}"),
                        }
                    })?;

                    // Look for matching exception handler
                    let mut matched = None;
                    let mut fallback = None;
                    for handler in &func.exception_handlers {
                        if pc >= handler.try_start_pc && pc < handler.try_end_pc {
                            match &handler.catch_type_symbol {
                                None => {
                                    if fallback.is_none() {
                                        fallback = Some(handler.clone());
                                    }
                                }
                                Some(type_sym) => {
                                    let is_match = if let Value::Ref(obj_addr) = exc_val
                                        && obj_addr > 0
                                    {
                                        let obj_vtable_id =
                                            self.heap.read_u32(obj_addr).unwrap_or(0);
                                        if let Some(target_class) = self
                                            .context
                                            .all_classes
                                            .iter()
                                            .find(|c| c.symbol == *type_sym)
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
                                        matched = Some(handler.clone());
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if matched.is_none() {
                        matched = fallback;
                    }

                    if let Some(handler) = matched {
                        if handler.finally_pc.is_none() {
                            self.set_reg(handler.catch_reg, exc_val)?;
                        }
                        let frame = self.current_frame_mut()?;
                        frame.pc = handler.handler_pc;
                        return Ok(());
                    }

                    // No handler in this frame: unwind and drop frame references
                    let ref_regs = func.reference_registers.clone();
                    for ref_reg in ref_regs {
                        if let Ok(val) = self.get_reg(ref_reg)
                            && let Value::Ref(addr) = val
                            && addr > 0
                        {
                            let _ = self.release_reference(addr);
                        }
                    }

                    let frame = self
                        .call_frames
                        .pop()
                        .ok_or(RuntimeError::NullReferenceAccess)?;
                    self.register_stack.truncate(frame.register_base);
                }
            }

            // =================================================================
            // 0xC0 .. 0xC1: ARC Reference Counting
            // =================================================================
            OpCode::IncRef => {
                let val = self.get_reg(inst.dst())?;
                if let Value::Ref(addr) = val
                    && addr > 0
                {
                    crate::memory::arc::inc_ref(&mut self.heap, addr)?;
                }
                Ok(())
            }

            OpCode::DecRef => {
                let val = self.get_reg(inst.dst())?;
                if let Value::Ref(addr) = val
                    && addr > 0
                {
                    self.release_reference(addr)?;
                }
                Ok(())
            }

            // =================================================================
            // 0x65 .. 0x67: Return & Exit
            // =================================================================
            OpCode::Return => self.pop_frame(Value::Null),

            // 0x66: ReturnValue
            OpCode::ReturnValue => {
                let val = self.get_reg(inst.dst())?;
                self.pop_frame(val)
            }

            // 0x67: Exit
            OpCode::Exit => {
                let code = self.get_reg(inst.dst())?.as_i32()?;
                self.exit_code = Some(code);
                self.halted = true;
                Ok(())
            }

            // 0x68: CallExtern
            OpCode::CallExtern => {
                let const_idx = inst.call_target() as usize;
                let arg_base = inst.arg_base();
                let arg_count = inst.arg_count() as usize;
                let caller_dst = inst.call_dst();

                let (caller_func_id, caller_pc, sym, args) = {
                    let frame = self.current_frame()?;
                    let binary_id = frame.binary_id;
                    let func_id = frame.function_id;
                    let pc = frame.pc.saturating_sub(1) as usize;
                    let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                        RuntimeError::BinaryLoadError {
                            reason: format!("constant pool missing for binary {binary_id}"),
                        }
                    })?;
                    let c_val = pool
                        .get(const_idx)
                        .ok_or_else(|| RuntimeError::TypeMismatch {
                            expected: "symbol string in constant pool".to_string(),
                            found: format!("index {const_idx}"),
                        })?;
                    let s = match c_val {
                        ConstantPoolValue::String(s) => s.clone(),
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string symbol".to_string(),
                                found: format!("{c_val:?}"),
                            });
                        }
                    };
                    let mut args = Vec::with_capacity(arg_count);
                    for i in 0..arg_count {
                        args.push(self.get_reg(arg_base + i as u16)?);
                    }
                    (func_id, pc, s, args)
                };

                // Check native registry first
                if let Some(&native_fn) = self.native_registry.get(&sym) {
                    let result = native_fn(self, &args)?;
                    self.set_reg(caller_dst, result)?;
                    return Ok(());
                }

                let global_id = self.context.resolve_function_symbol(&sym).ok_or_else(|| {
                    RuntimeError::FunctionNotFound {
                        symbol_or_id: sym.clone(),
                    }
                })?;

                // In-place quickening to CallDirect
                let quickened = Instruction::format_call(
                    OpCode::CallDirect,
                    caller_dst,
                    global_id as u16,
                    arg_base,
                    arg_count as u8,
                );
                self.context
                    .patch_instruction(caller_func_id, caller_pc, quickened);

                if let Some(func) = self.context.get_function(global_id) {
                    let clean = func.symbol.split('(').next().unwrap_or(&func.symbol);
                    let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
                    let has_qualifier = clean.contains("::");
                    if let Some(&native_fn) = self
                        .native_registry
                        .get(&func.symbol)
                        .or_else(|| self.native_registry.get(clean))
                        .or_else(|| {
                            if !has_qualifier {
                                self.native_registry.get(simple)
                            } else {
                                None
                            }
                        })
                    {
                        let result = native_fn(self, &args)?;
                        self.set_reg(caller_dst, result)?;
                        return Ok(());
                    }
                }

                self.push_frame(global_id, caller_dst, &args)
            }

            // 0x77: SetStaticGlobal
            OpCode::SetStaticGlobal => {
                let (caller_func_id, caller_pc, binary_id) = {
                    let frame = self.current_frame()?;
                    (
                        frame.function_id,
                        frame.pc.saturating_sub(1) as usize,
                        frame.binary_id,
                    )
                };
                let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                    RuntimeError::BinaryLoadError {
                        reason: format!("constant pool missing for binary {binary_id}"),
                    }
                })?;
                let idx = inst.imm32() as usize;
                let c_val = pool.get(idx).ok_or_else(|| RuntimeError::TypeMismatch {
                    expected: "symbol string in constant pool".to_string(),
                    found: format!("index {idx}"),
                })?;
                let sym = match c_val {
                    ConstantPoolValue::String(s) => s.as_str(),
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "string symbol".to_string(),
                            found: format!("{c_val:?}"),
                        });
                    }
                };
                let slot = self.context.resolve_static_slot(sym).ok_or(
                    RuntimeError::InvalidStaticOffset {
                        offset: idx as u32,
                        total_static: self.static_slots.len() as u32,
                    },
                )?;

                // In-place quickening to SetStaticDirect
                let quickened = Instruction::format_b(OpCode::SetStaticDirect, 0, inst.dst(), slot);
                self.context
                    .patch_instruction(caller_func_id, caller_pc, quickened);

                let val = self.get_reg(inst.dst())?;
                self.set_static(slot, val)
            }

            // 0x78: SetStaticDirect
            OpCode::SetStaticDirect => {
                let slot = inst.imm32();
                let val = self.get_reg(inst.dst())?;
                self.set_static(slot, val)
            }

            // 0x79: AllocObjectExtern
            OpCode::AllocObjectExtern => {
                let (caller_func_id, caller_pc, binary_id) = {
                    let frame = self.current_frame()?;
                    (
                        frame.function_id,
                        frame.pc.saturating_sub(1) as usize,
                        frame.binary_id,
                    )
                };
                let pool = self.context.get_constant_pool(binary_id).ok_or_else(|| {
                    RuntimeError::BinaryLoadError {
                        reason: format!("constant pool missing for binary {binary_id}"),
                    }
                })?;
                let idx = inst.imm32() as usize;
                let c_val = pool.get(idx).ok_or_else(|| RuntimeError::TypeMismatch {
                    expected: "symbol string in constant pool".to_string(),
                    found: format!("index {idx}"),
                })?;
                let sym = match c_val {
                    ConstantPoolValue::String(s) => s.as_str(),
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "string symbol".to_string(),
                            found: format!("{c_val:?}"),
                        });
                    }
                };
                let (vtable_id, size) = self.context.resolve_vtable_info(sym).ok_or_else(|| {
                    RuntimeError::FunctionNotFound {
                        symbol_or_id: format!("class {sym}"),
                    }
                })?;
                let size = size.max(8);
                let payload_addr = self.heap.allocate(size, BlockTag::Object)?;
                self.heap.write_u32(payload_addr, vtable_id)?;

                // In-place quickening to AllocObject
                let quickened = Instruction::format_a(
                    OpCode::AllocObject,
                    0,
                    inst.dst(),
                    vtable_id as u16,
                    size as u16,
                );
                self.context
                    .patch_instruction(caller_func_id, caller_pc, quickened);

                self.set_reg(inst.dst(), Value::Ref(payload_addr))
            }
        }
    }
}
