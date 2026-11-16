//! Flow-sensitive Escape Analysis, Scalar Replacement of Aggregates (SRA),
//! and Automated Reference Counting (ARC) Elision.
//!
//! Conforms to Milestone 055 specifications.

#![forbid(unsafe_code)]

use solix_common::bytecode::{CompiledClass, CompiledFunction, Instruction, OpCode};
use std::collections::{HashMap, HashSet};

/// Lifetime and escape classification of a heap-allocated aggregate or reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeCategory {
    /// Object never escapes the local function frame.
    /// Safe for Scalar Replacement of Aggregates (SRA) and total ARC elimination.
    NoEscape,
    /// Object escapes to a callee via non-escaping or borrowed parameter.
    /// Safe for caller/callee ARC elision.
    ArgEscape,
    /// Object escapes to a heap field, array, static partition, return value, or throw.
    /// Must remain allocated on the heap.
    GlobalEscape,
}

/// Identifies whether an instruction causes the target register (or its aliases) to escape.
fn evaluate_instruction_escape(
    instruction: &Instruction,
    target_registers: &HashSet<u16>,
) -> Option<EscapeCategory> {
    let raw_opcode = instruction.raw_opcode();

    // 1. ReturnValue: variable returned from function -> GlobalEscape
    if raw_opcode == OpCode::ReturnValue as u8 && target_registers.contains(&instruction.dst()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 2. Throw: variable thrown as exception -> GlobalEscape
    if raw_opcode == OpCode::Throw as u8 && target_registers.contains(&instruction.dst()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 3. SetField: if candidate is the value being stored into a field -> GlobalEscape
    if raw_opcode == OpCode::SetField as u8 && target_registers.contains(&instruction.dst()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 4. SetArray: if candidate is stored into an array -> GlobalEscape
    if raw_opcode == OpCode::SetArray as u8 && target_registers.contains(&instruction.src2()) {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 5. SetStatic: if candidate is stored into static partition -> GlobalEscape
    if (raw_opcode == OpCode::SetStatic as u8
        || raw_opcode == OpCode::SetStaticDirect as u8
        || raw_opcode == OpCode::SetStaticGlobal as u8)
        && (target_registers.contains(&instruction.dst())
            || target_registers.contains(&instruction.src1()))
    {
        return Some(EscapeCategory::GlobalEscape);
    }

    // 6. Dynamic dispatch calls: CallVirtual, CallInterface -> GlobalEscape
    if raw_opcode == OpCode::CallVirtual as u8 || raw_opcode == OpCode::CallInterface as u8 {
        let argument_base = instruction.arg_base();
        let argument_count = instruction.arg_count() as u16;
        for reg_offset in 0..argument_count {
            if target_registers.contains(&(argument_base + reg_offset)) {
                return Some(EscapeCategory::GlobalEscape);
            }
        }
    }

    // 7. External or native calls: CallExtern, CallNativeDirect -> GlobalEscape
    if raw_opcode == OpCode::CallExtern as u8 || raw_opcode == OpCode::CallNativeDirect as u8 {
        let argument_base = instruction.arg_base();
        let argument_count = instruction.arg_count() as u16;
        for reg_offset in 0..argument_count {
            if target_registers.contains(&(argument_base + reg_offset)) {
                return Some(EscapeCategory::GlobalEscape);
            }
        }
    }

    // 8. Closure packing: PackClosure -> GlobalEscape
    if raw_opcode == OpCode::PackClosure as u8 {
        let argument_base = instruction.arg_base();
        let argument_count = instruction.arg_count() as u16;
        for reg_offset in 0..argument_count {
            if target_registers.contains(&(argument_base + reg_offset)) {
                return Some(EscapeCategory::GlobalEscape);
            }
        }
    }

    // 9. Polymorphic type checking: Instanceof, CastCheck -> GlobalEscape
    if (raw_opcode == OpCode::Instanceof as u8
        || raw_opcode == OpCode::CastCheck as u8
        || raw_opcode == OpCode::TupleInstanceof as u8)
        && (target_registers.contains(&instruction.dst())
            || target_registers.contains(&instruction.src1()))
    {
        return Some(EscapeCategory::GlobalEscape);
    }

    None
}

/// Analyzes a constructor function. If it is a pure field-forwarding constructor
/// (only `SetField` from parameters to `this`, followed by `Return`), returns a mapping
/// of `field_offset -> parameter_index`.
fn analyze_forwarding_constructor(
    constructor_function: &CompiledFunction,
) -> Option<HashMap<u16, u16>> {
    let mut field_to_param = HashMap::new();

    for instruction in &constructor_function.instructions {
        let raw_opcode = instruction.raw_opcode();
        if raw_opcode == OpCode::Nop as u8
            || raw_opcode == OpCode::Return as u8
            || raw_opcode == OpCode::ReturnValue as u8
            || raw_opcode == OpCode::Mov as u8
        {
            continue;
        }
        if raw_opcode == OpCode::SetField as u8 && instruction.src1() == 0 {
            let param_register = instruction.dst();
            let field_offset = instruction.src2();
            if param_register > 0 {
                field_to_param.insert(field_offset, param_register);
                continue;
            }
        }
        // Any other instruction means constructor has non-trivial logic
        return None;
    }

    if field_to_param.is_empty() {
        None
    } else {
        Some(field_to_param)
    }
}

/// Performs flow-sensitive escape analysis on a compiled function, classifying
/// candidate heap allocations into `NoEscape`, `ArgEscape`, or `GlobalEscape`.
pub fn classify_allocations(
    function: &CompiledFunction,
    classes: &[CompiledClass],
    all_functions: &[CompiledFunction],
) -> HashMap<usize, (u16, EscapeCategory)> {
    let mut allocation_categories = HashMap::new();

    for (instruction_index, instruction) in function.instructions.iter().enumerate() {
        if instruction.raw_opcode() == OpCode::AllocObject as u8 {
            let allocated_register = instruction.dst();
            let vtable_identifier = instruction.src1() as u32;

            // Find class descriptor
            let class_candidate = classes
                .iter()
                .find(|class_descriptor| class_descriptor.vtable_id == vtable_identifier);

            let Some(class_descriptor) = class_candidate else {
                allocation_categories.insert(
                    instruction_index,
                    (allocated_register, EscapeCategory::GlobalEscape),
                );
                continue;
            };

            // Non-aggregates (classes with destructors, inheritance, or reference fields) must escape
            if class_descriptor.destructor_id.is_some()
                || class_descriptor.base_vtable_id.is_some()
                || !class_descriptor.itable.is_empty()
                || !class_descriptor.reference_field_offsets.is_empty()
            {
                allocation_categories.insert(
                    instruction_index,
                    (allocated_register, EscapeCategory::GlobalEscape),
                );
                continue;
            }

            // Reject allocations enclosed in loops to avoid unhandled multi-iteration state
            let is_enclosed_in_loop = function.instructions.iter().enumerate().any(|(loop_index, loop_instruction)| {
                let loop_raw_opcode = loop_instruction.raw_opcode();
                if loop_raw_opcode == OpCode::Jump as u8
                    || loop_raw_opcode == OpCode::JumpIfTrue as u8
                    || loop_raw_opcode == OpCode::JumpIfFalse as u8
                {
                    let jump_target = (loop_index as i32) + loop_instruction.jump_offset();
                    jump_target >= 0 && (jump_target as usize) <= instruction_index && loop_index >= instruction_index
                } else if (OpCode::JumpIfLtI32 as u8..=OpCode::JumpIfNeqI32 as u8).contains(&loop_raw_opcode) {
                    let jump_target = (loop_index as i32) + (loop_instruction.src2() as i16 as i32);
                    jump_target >= 0 && (jump_target as usize) <= instruction_index && loop_index >= instruction_index
                } else {
                    false
                }
            });

            if is_enclosed_in_loop {
                allocation_categories.insert(
                    instruction_index,
                    (allocated_register, EscapeCategory::GlobalEscape),
                );
                continue;
            }

            let mut target_registers = HashSet::new();
            target_registers.insert(allocated_register);

            // Track constructor window base if CallDirect is called immediately
            let mut constructor_call_pc_index = None;
            let mut constructor_window_base = None;
            let mut constructor_target_id = None;

            for lookahead_index in instruction_index + 1..std::cmp::min(instruction_index + 6, function.instructions.len()) {
                let lookahead_instruction = &function.instructions[lookahead_index];
                if lookahead_instruction.raw_opcode() == OpCode::CallDirect as u8 {
                    let argument_base = lookahead_instruction.arg_base();
                    if function.instructions[instruction_index + 1..lookahead_index].iter().any(|previous_instruction| {
                        previous_instruction.raw_opcode() == OpCode::Mov as u8
                            && previous_instruction.src1() == allocated_register
                            && previous_instruction.dst() == argument_base
                    }) {
                        constructor_call_pc_index = Some(lookahead_index);
                        constructor_window_base = Some(argument_base);
                        constructor_target_id = Some(lookahead_instruction.call_target() as usize);
                    }
                    break;
                }
            }

            // If a constructor exists, it must be a pure field-forwarding constructor
            if let Some(target_func_index) = constructor_target_id {
                let is_pure_constructor = all_functions
                    .get(target_func_index)
                    .and_then(analyze_forwarding_constructor)
                    .is_some();

                if !is_pure_constructor {
                    allocation_categories.insert(
                        instruction_index,
                        (allocated_register, EscapeCategory::GlobalEscape),
                    );
                    continue;
                }
            }

            let mut overall_category = EscapeCategory::NoEscape;

            for (inspection_index, inspection_instruction) in
                function.instructions.iter().enumerate()
            {
                if inspection_index <= instruction_index {
                    continue;
                }

                let raw_opcode = inspection_instruction.raw_opcode();

                // Check for aliasing assignment: Mov alias, allocated_register
                if raw_opcode == OpCode::Mov as u8 {
                    let source_register = inspection_instruction.src1();
                    let destination_register = inspection_instruction.dst();
                    if target_registers.contains(&source_register) {
                        target_registers.insert(destination_register);
                    } else if target_registers.contains(&destination_register) {
                        target_registers.remove(&destination_register);
                    }
                }

                // Check CallDirect:
                if raw_opcode == OpCode::CallDirect as u8 {
                    let argument_base = inspection_instruction.arg_base();
                    let argument_count = inspection_instruction.arg_count() as u16;

                    let is_immediate_constructor = Some(inspection_index) == constructor_call_pc_index;

                    if is_immediate_constructor {
                        if let Some(window_base) = constructor_window_base {
                            target_registers.remove(&window_base);
                        }
                    } else {
                        // Non-constructor method or function call using target registers forces heap escape
                        for reg_offset in 0..argument_count {
                            if target_registers.contains(&(argument_base + reg_offset)) {
                                overall_category = EscapeCategory::GlobalEscape;
                                break;
                            }
                        }
                    }
                }

                if let Some(category) =
                    evaluate_instruction_escape(inspection_instruction, &target_registers)
                {
                    if category == EscapeCategory::GlobalEscape {
                        overall_category = EscapeCategory::GlobalEscape;
                        break;
                    } else if category == EscapeCategory::ArgEscape
                        && overall_category == EscapeCategory::NoEscape
                    {
                        overall_category = EscapeCategory::ArgEscape;
                    }
                }
            }

            allocation_categories.insert(
                instruction_index,
                (allocated_register, overall_category),
            );
        }
    }

    allocation_categories
}

/// Applies Scalar Replacement of Aggregates (SRA) on non-escaping objects.
///
/// Replaces `AllocObject` with `Nop`, forwards field writes directly to field
/// reads, and removes redundant `DecRef` calls on the scalar-replaced object.
pub fn apply_scalar_replacement(
    function: &mut CompiledFunction,
    classes: &[CompiledClass],
    all_functions: &[CompiledFunction],
) -> usize {
    let allocations = classify_allocations(function, classes, all_functions);
    let mut scalar_replaced_count = 0;

    for (allocation_pc, (allocated_register, category)) in allocations {
        if category != EscapeCategory::NoEscape {
            continue;
        }

        // Collect all field offsets read or written across allocated_register and its aliases
        let mut alias_registers = HashSet::new();
        alias_registers.insert(allocated_register);

        // Find constructor call
        let mut constructor_call_pc = None;
        let mut constructor_arg_base = None;
        let mut constructor_target_func_index = None;

        for check_index in allocation_pc + 1..std::cmp::min(allocation_pc + 8, function.instructions.len()) {
            let instruction_entry = &function.instructions[check_index];
            if instruction_entry.raw_opcode() == OpCode::CallDirect as u8 {
                let argument_base = instruction_entry.arg_base();
                if function.instructions[allocation_pc + 1..check_index].iter().any(|prev_inst| {
                    prev_inst.raw_opcode() == OpCode::Mov as u8
                        && prev_inst.src1() == allocated_register
                        && prev_inst.dst() == argument_base
                }) || argument_base == allocated_register {
                    constructor_call_pc = Some(check_index);
                    constructor_arg_base = Some(argument_base);
                    constructor_target_func_index = Some(instruction_entry.call_target() as usize);
                    break;
                }
            }
        }

        let mut field_offsets = Vec::new();
        let mut can_scalar_replace = true;

        for inspection_index in allocation_pc + 1..function.instructions.len() {
            let instruction = function.instructions[inspection_index];
            let raw_opcode = instruction.raw_opcode();

            if raw_opcode == OpCode::Mov as u8 && alias_registers.contains(&instruction.src1()) {
                alias_registers.insert(instruction.dst());
            }

            if (raw_opcode == OpCode::GetField as u8 || raw_opcode == OpCode::SetField as u8)
                && alias_registers.contains(&instruction.src1())
            {
                let offset = instruction.src2();
                if !field_offsets.contains(&offset) {
                    field_offsets.push(offset);
                }
            } else if raw_opcode == OpCode::CallDirect as u8 {
                let argument_base = instruction.arg_base();
                let is_constructor = Some(inspection_index) == constructor_call_pc;
                if !is_constructor && alias_registers.contains(&argument_base) {
                    can_scalar_replace = false;
                    break;
                }
            }
        }

        if !can_scalar_replace {
            continue;
        }

        field_offsets.sort_unstable();

        // Map constructor arguments to dedicated registers using analyzed constructor bytecode
        let mut field_values: HashMap<u16, u16> = HashMap::new();

        if let (Some(ctor_base), Some(target_function_index), Some(ctor_pc)) =
            (constructor_arg_base, constructor_target_func_index, constructor_call_pc)
            && let Some(target_func) = all_functions.get(target_function_index)
            && let Some(field_to_param) = analyze_forwarding_constructor(target_func)
        {
            let mut sorted_offsets: Vec<u16> = field_to_param.keys().copied().collect();
            sorted_offsets.sort_unstable();

                for &offset in &sorted_offsets {
                    let param_index = field_to_param[&offset];
                    let argument_register = ctor_base + param_index;

                    // Allocate dedicated register above function register count
                    let dedicated_register = function.num_registers;
                    function.num_registers += 1;
                    field_values.insert(offset, dedicated_register);

                    // Retarget instruction that wrote to argument_register directly to dedicated_register
                    let mut found_writer = false;
                    for search_pc in allocation_pc + 1..ctor_pc {
                        let candidate_instruction = function.instructions[search_pc];
                        let raw = candidate_instruction.raw_opcode();
                        if raw != OpCode::CallDirect as u8 && candidate_instruction.dst() == argument_register {
                            function.instructions[search_pc] = candidate_instruction.with_dst(dedicated_register);
                            found_writer = true;
                            break;
                        }
                    }

                    if !found_writer {
                        function.instructions[ctor_pc] = Instruction::format_a(
                            OpCode::Mov,
                            0,
                            dedicated_register,
                            argument_register,
                            0,
                        );
                    }
                }
            }

        // 1. Rewrite AllocObject to Nop
        function.instructions[allocation_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);

        // 2. Rewrite constructor CallDirect and constructor window Mov to Nop
        if let Some(ctor_pc) = constructor_call_pc {
            if function.instructions[ctor_pc].raw_opcode() == OpCode::CallDirect as u8 {
                function.instructions[ctor_pc] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            }
            for back_index in allocation_pc + 1..ctor_pc {
                let check_inst = &function.instructions[back_index];
                if check_inst.raw_opcode() == OpCode::Mov as u8
                    && alias_registers.contains(&check_inst.src1())
                {
                    function.instructions[back_index] =
                        Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                }
            }
        }

        // 3. Rewrite SetField, GetField, Mov, and DecRef/IncRef on allocated_register and aliases
        let mut rewrite_aliases = HashSet::new();
        rewrite_aliases.insert(allocated_register);

        for current_index in allocation_pc + 1..function.instructions.len() {
            let current_instruction = function.instructions[current_index];
            let raw_opcode = current_instruction.raw_opcode();

            if raw_opcode == OpCode::Mov as u8 && rewrite_aliases.contains(&current_instruction.src1()) {
                rewrite_aliases.insert(current_instruction.dst());
                function.instructions[current_index] =
                    Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            } else if raw_opcode == OpCode::SetField as u8 && rewrite_aliases.contains(&current_instruction.src1()) {
                let offset = current_instruction.src2();
                let value_register = current_instruction.dst();
                field_values.insert(offset, value_register);
                function.instructions[current_index] =
                    Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            } else if raw_opcode == OpCode::GetField as u8
                && rewrite_aliases.contains(&current_instruction.src1())
            {
                let offset = current_instruction.src2();
                let destination_register = current_instruction.dst();
                if let Some(&source_value_register) = field_values.get(&offset) {
                    function.instructions[current_index] = Instruction::format_a(
                        OpCode::Mov,
                        0,
                        destination_register,
                        source_value_register,
                        0,
                    );
                }
            } else if (raw_opcode == OpCode::DecRef as u8 || raw_opcode == OpCode::IncRef as u8)
                && rewrite_aliases.contains(&current_instruction.dst())
            {
                function.instructions[current_index] =
                    Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
            }
        }

        // 4. Remove all rewritten aliases from function.reference_registers
        for &alias_register in &rewrite_aliases {
            function
                .reference_registers
                .retain(|candidate_register| *candidate_register != alias_register);
        }

        scalar_replaced_count += 1;
    }

    scalar_replaced_count
}

/// Applies inter-procedural and local reference counting (ARC) elision.
///
/// Removes paired `IncRef` and `DecRef` instructions on local temporaries
/// and non-escaping parameters.
pub fn apply_arc_elision(function: &mut CompiledFunction) -> usize {
    let mut elided_count = 0;
    let instruction_count = function.instructions.len();

    for start_index in 0..instruction_count {
        if function.instructions[start_index].raw_opcode() == OpCode::IncRef as u8 {
            let target_register = function.instructions[start_index].dst();

            let mut matching_dec_index = None;
            for forward_index in start_index + 1..std::cmp::min(start_index + 32, instruction_count) {
                let forward_inst = function.instructions[forward_index];
                let forward_raw = forward_inst.raw_opcode();

                if forward_raw == OpCode::Jump as u8
                    || forward_raw == OpCode::JumpIfTrue as u8
                    || forward_raw == OpCode::JumpIfFalse as u8
                    || forward_raw == OpCode::Return as u8
                    || forward_raw == OpCode::ReturnValue as u8
                    || forward_raw == OpCode::Throw as u8
                {
                    break;
                }

                if forward_inst.dst() == target_register
                    && forward_raw != OpCode::IncRef as u8
                    && forward_raw != OpCode::DecRef as u8
                {
                    break;
                }

                if forward_raw == OpCode::DecRef as u8 && forward_inst.dst() == target_register {
                    matching_dec_index = Some(forward_index);
                    break;
                }
            }

            if let Some(dec_index) = matching_dec_index {
                function.instructions[start_index] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                function.instructions[dec_index] = Instruction::format_a(OpCode::Nop, 0, 0, 0, 0);
                elided_count += 2;
            }
        }
    }

    elided_count
}
