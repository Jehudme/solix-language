//! Diagnostic bytecode disassembler and pretty-printer for functions and SLXB binaries.

#![forbid(unsafe_code)]

use solix_common::bytecode::{
    BinaryProgram, CompiledFunction, ConstantPool, ConstantPoolValue, Instruction, OpCode,
};
use std::fmt::Write;

/// Disassembles an individual instruction into a human-readable assembly string.
#[must_use]
pub fn disassemble_instruction(
    instruction: &Instruction,
    pc: usize,
    pool: &ConstantPool,
) -> String {
    let raw_op = instruction.raw_opcode();
    let opcode = match OpCode::from_u8(raw_op, pc) {
        Ok(op) => op,
        Err(_) => return format!("{pc:04}  UNKNOWN_OPCODE({raw_op:#04x})"),
    };

    match opcode {
        OpCode::CallDirect
        | OpCode::CallVirtual
        | OpCode::CallInterface
        | OpCode::CallSymbol
        | OpCode::CallCallable
        | OpCode::CallExtern
        | OpCode::CallNativeDirect => {
            let dst = instruction.call_dst();
            let target = instruction.call_target();
            let base = instruction.arg_base();
            let count = instruction.arg_count();
            if count == 0 {
                format!(
                    "{pc:04}  {:<16} r{}, target:{target}, args:[]",
                    opcode.mnemonic(),
                    dst
                )
            } else {
                format!(
                    "{pc:04}  {:<16} r{}, target:{target}, args:r{base}..r{}",
                    opcode.mnemonic(),
                    dst,
                    base + count as u16 - 1
                )
            }
        }

        OpCode::Jump => {
            let offset = instruction.jump_offset();
            let target_pc = pc as i32 + offset;
            format!(
                "{pc:04}  {:<16} @{target_pc:04} ({offset:+})",
                opcode.mnemonic()
            )
        }

        OpCode::JumpIfTrue | OpCode::JumpIfFalse => {
            let cond = instruction.cond_reg();
            let offset = instruction.jump_offset();
            let target_pc = pc as i32 + offset;
            format!(
                "{pc:04}  {:<16} r{cond}, @{target_pc:04} ({offset:+})",
                opcode.mnemonic()
            )
        }

        OpCode::JumpIfLtI32
        | OpCode::JumpIfLteI32
        | OpCode::JumpIfGtI32
        | OpCode::JumpIfGteI32
        | OpCode::JumpIfEqI32
        | OpCode::JumpIfNeqI32 => {
            let offset = instruction.src2() as i16 as i32;
            let target_pc = pc as i32 + offset;
            format!(
                "{pc:04}  {:<16} r{}, r{}, @{target_pc:04} ({offset:+})",
                opcode.mnemonic(),
                instruction.dst(),
                instruction.src1()
            )
        }

        OpCode::AddI32Imm | OpCode::SubI32Imm | OpCode::MulI32Imm | OpCode::ModI32Imm => {
            let imm = instruction.src2() as i16;
            format!(
                "{pc:04}  {:<16} r{}, r{}, {imm:+}",
                opcode.mnemonic(),
                instruction.dst(),
                instruction.src1()
            )
        }

        OpCode::LoadConst => {
            let dst = instruction.dst();
            let const_idx = instruction.imm32() as usize;
            let repr = match pool.get(const_idx) {
                Some(ConstantPoolValue::String(text)) => format!("\"{text}\""),
                Some(ConstantPoolValue::Int64(int_val)) => format!("{int_val} (i64)"),
                Some(ConstantPoolValue::Float64(bits)) => {
                    format!("{} (f64)", f64::from_bits(*bits))
                }
                None => format!("<invalid const #{const_idx}>"),
            };
            format!(
                "{pc:04}  {:<16} r{dst}, $c{const_idx}               ; {repr}",
                opcode.mnemonic()
            )
        }

        OpCode::LoadI32 => {
            format!(
                "{pc:04}  {:<16} r{}, {}",
                opcode.mnemonic(),
                instruction.dst(),
                instruction.imm32_signed()
            )
        }

        OpCode::LoadBool => {
            format!(
                "{pc:04}  {:<16} r{}, {}",
                opcode.mnemonic(),
                instruction.dst(),
                if instruction.imm32() == 1 {
                    "true"
                } else {
                    "false"
                }
            )
        }

        OpCode::LoadNull => {
            format!(
                "{pc:04}  {:<16} r{}, null",
                opcode.mnemonic(),
                instruction.dst()
            )
        }

        OpCode::Return => format!("{pc:04}  {}", opcode.mnemonic()),

        OpCode::ReturnValue | OpCode::IncRef | OpCode::DecRef | OpCode::Throw | OpCode::Exit => {
            format!("{pc:04}  {:<16} r{}", opcode.mnemonic(), instruction.dst())
        }

        OpCode::Mov => {
            format!(
                "{pc:04}  {:<16} r{}, r{}",
                opcode.mnemonic(),
                instruction.dst(),
                instruction.src1()
            )
        }

        OpCode::Not | OpCode::Bnot | OpCode::NegI32 | OpCode::NegI64 | OpCode::NegF64 => {
            format!(
                "{pc:04}  {:<16} r{}, r{}",
                opcode.mnemonic(),
                instruction.dst(),
                instruction.src1()
            )
        }

        _ => {
            format!(
                "{pc:04}  {:<16} r{}, r{}, r{}",
                opcode.mnemonic(),
                instruction.dst(),
                instruction.src1(),
                instruction.src2()
            )
        }
    }
}

/// Disassembles an entire compiled function into clean, indented assembly text.
#[must_use]
pub fn disassemble_function(func: &CompiledFunction, pool: &ConstantPool) -> String {
    let mut output = String::new();

    let _ = writeln!(
        output,
        ".function {}(params: {}) -> frame_size: {}, refs: {:?}",
        func.symbol, func.param_count, func.num_registers, func.reference_registers
    );
    let _ = writeln!(
        output,
        "  .source_id: {}, static_base: {}",
        func.source_id, func.static_base_offset
    );

    if !func.exception_handlers.is_empty() {
        let _ = writeln!(output, "  .handlers:");
        for handler in &func.exception_handlers {
            let catch_type = handler.catch_type_symbol.as_deref().unwrap_or("<any>");
            let finally_str = match handler.finally_pc {
                Some(finally_target) => format!(", finally: @{finally_target:04}"),
                None => String::new(),
            };
            let _ = writeln!(
                output,
                "    [TRY: {:04}..{:04} -> CATCH '{}' -> @{:04} (reg: r{}){}]",
                handler.try_start_pc,
                handler.try_end_pc,
                catch_type,
                handler.handler_pc,
                handler.catch_reg,
                finally_str
            );
        }
    }

    let _ = writeln!(output, "  .code:");
    for (pc, inst) in func.instructions.iter().enumerate() {
        let line = disassemble_instruction(inst, pc, pool);
        let _ = writeln!(output, "    {line}");
    }

    output
}

/// Disassembles a complete `BinaryProgram` container into an annotated assembly listing.
#[must_use]
pub fn disassemble_program(program: &BinaryProgram) -> String {
    let mut output = String::new();

    let _ = writeln!(
        output,
        "; ================================================================"
    );
    let _ = writeln!(output, "; Solix SLXB Binary Disassembly");
    let _ = writeln!(
        output,
        "; Version: {}.{}, Flags: {:#010x}",
        program.version_major, program.version_minor, program.flags
    );
    let _ = writeln!(
        output,
        "; Entry Point: {:?} (id: {:?})",
        program.entry_point_symbol, program.entry_point_id
    );
    let _ = writeln!(
        output,
        "; Static Reservation: {} slots",
        program.static_space_size
    );
    let _ = writeln!(
        output,
        "; ================================================================\n"
    );

    if !program.constant_pool.is_empty() {
        let _ = writeln!(output, ".constant_pool:");
        for (idx, val) in program.constant_pool.values.iter().enumerate() {
            match val {
                ConstantPoolValue::Int64(int_val) => {
                    let _ = writeln!(output, "  $c{idx:02}: i64 = {int_val}");
                }
                ConstantPoolValue::Float64(bits) => {
                    let _ = writeln!(output, "  $c{idx:02}: f64 = {}", f64::from_bits(*bits));
                }
                ConstantPoolValue::String(text) => {
                    let _ = writeln!(output, "  $c{idx:02}: string = \"{text}\"");
                }
            }
        }
        let _ = writeln!(output);
    }

    if !program.classes.is_empty() {
        let _ = writeln!(output, ".classes:");
        for cls in &program.classes {
            let _ = writeln!(
                output,
                "  .class {} (vtable_id: {}, size: {}B, destructor: {:?})",
                cls.symbol, cls.vtable_id, cls.instance_size, cls.destructor_id
            );
            let _ = writeln!(output, "    .vtable: {:?}", cls.vtable);
            if !cls.itable.is_empty() {
                let _ = writeln!(output, "    .itable: {:?}", cls.itable);
            }
        }
        let _ = writeln!(output);
    }

    let _ = writeln!(output, ".functions ({} total):", program.functions.len());
    for (function_index, func) in program.functions.iter().enumerate() {
        let _ = writeln!(output, "; Function #{function_index}");
        let disassembled = disassemble_function(func, &program.constant_pool);
        let _ = writeln!(output, "{disassembled}");
    }

    output
}
