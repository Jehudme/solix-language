//! Whole-program call-graph reachability analysis and dead-code elimination (DCE).
//!
//! Milestone 023 Pillar 3: Dual-Mode Linking (`--link dynamic` vs `--link static`).
//! In static mode, reachable functions and classes from dependency binaries are merged
//! directly into the output binary container, converting `CallExtern` to `CallDirect`
//! and pruning all unreachable code.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::FLAG_POD;
use solix_common::bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ConstantPoolValue, Instruction, OpCode,
};
use std::collections::{HashMap, HashSet, VecDeque};

fn relocate_constant(
    program: &mut BinaryProgram,
    source_program: &BinaryProgram,
    source_index: u16,
) -> u16 {
    match source_program
        .constant_pool
        .values
        .get(source_index as usize)
    {
        Some(ConstantPoolValue::String(string_value)) => program
            .constant_pool
            .add_string(string_value.as_str())
            .unwrap_or(source_index),
        Some(ConstantPoolValue::Int64(integer_value)) => program
            .constant_pool
            .add_i64(*integer_value)
            .unwrap_or(source_index),
        Some(ConstantPoolValue::Float64(float_bits)) => program
            .constant_pool
            .add_f64(f64::from_bits(*float_bits))
            .unwrap_or(source_index),
        None => source_index,
    }
}

fn relocate_constant_string(
    program: &mut BinaryProgram,
    source_program: &BinaryProgram,
    source_index: u16,
) -> u16 {
    match source_program
        .constant_pool
        .values
        .get(source_index as usize)
    {
        Some(ConstantPoolValue::String(string_value)) => program
            .constant_pool
            .add_string(string_value.as_str())
            .unwrap_or(source_index),
        _ => source_index,
    }
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.replace('.', "::")
}

fn symbol_matches(candidate_symbol: &str, target_symbol: &str) -> bool {
    let normalized_candidate = normalize_symbol(candidate_symbol);
    let normalized_target = normalize_symbol(target_symbol);
    if normalized_candidate == normalized_target {
        return true;
    }
    let candidate_bare = normalized_candidate
        .split('(')
        .next()
        .unwrap_or(&normalized_candidate)
        .split('/')
        .next()
        .unwrap_or(&normalized_candidate);
    let target_bare = normalized_target
        .split('(')
        .next()
        .unwrap_or(&normalized_target)
        .split('/')
        .next()
        .unwrap_or(&normalized_target);
    if candidate_bare == target_bare {
        return true;
    }
    let suffix_candidate = format!("::{candidate_bare}");
    let suffix_target = format!("::{target_bare}");
    normalized_candidate.ends_with(&suffix_target) || normalized_target.ends_with(&suffix_candidate)
}

/// Applies static whole-program reachability analysis and dead-code elimination.
///
/// In static mode, imports reachable functions from `dependency_programs`, rewrites
/// `OpCode::CallExtern` to `OpCode::CallDirect`, and prunes unreachable functions.
pub fn apply_reachability_and_static_linking(
    program: &mut BinaryProgram,
    dependency_programs: &[BinaryProgram],
    is_static_mode: bool,
) {
    if !is_static_mode {
        return;
    }

    let mut reachable_symbols: HashSet<String> = HashSet::new();
    let mut work_queue: VecDeque<String> = VecDeque::new();

    // 1. Seed reachability queue with entry point and module initializers
    if let Some(entry_symbol) = &program.entry_point_symbol {
        reachable_symbols.insert(entry_symbol.clone());
        work_queue.push_back(entry_symbol.clone());
    } else if let Some(entry_id) = program.entry_point_id
        && let Some(entry_function) = program.functions.get(entry_id as usize)
    {
        reachable_symbols.insert(entry_function.symbol.clone());
        work_queue.push_back(entry_function.symbol.clone());
    }

    for function in &program.functions {
        let is_root_symbol = function.symbol.contains("<init")
            || function.symbol.contains("<module_init>")
            || function.symbol == "main"
            || function.symbol.ends_with("::main");
        if is_root_symbol && reachable_symbols.insert(function.symbol.clone()) {
            work_queue.push_back(function.symbol.clone());
        }
    }

    // Index external dependency functions and classes
    let mut dependency_functions: Vec<(&CompiledFunction, &BinaryProgram)> = Vec::new();
    let mut dependency_classes: Vec<(&CompiledClass, &BinaryProgram)> = Vec::new();
    for dep_program in dependency_programs {
        for function in &dep_program.functions {
            dependency_functions.push((function, dep_program));
        }
        for class_item in &dep_program.classes {
            dependency_classes.push((class_item, dep_program));
        }
    }

    // Map to track imported dependency functions: (dep_func_ptr as usize) -> new_local_index
    let mut imported_functions: HashMap<String, usize> = HashMap::new();

    // 2. Discover transitively reachable functions
    while let Some(current_symbol) = work_queue.pop_front() {
        // Find current function in local program
        let current_function_opt = program
            .functions
            .iter()
            .find(|function| symbol_matches(&function.symbol, &current_symbol))
            .cloned();

        let current_function = match current_function_opt {
            Some(func) => func,
            None => {
                // Check if it exists in dependencies to import
                let dependency_match = dependency_functions.iter().find(|(dep_function, _)| {
                    symbol_matches(&dep_function.symbol, &current_symbol)
                });

                if let Some(&(dep_function, dep_program)) = dependency_match {
                    if let Some(&existing_index) = imported_functions.get(&dep_function.symbol) {
                        program.functions[existing_index].clone()
                    } else {
                        let mut cloned_function = dep_function.clone();
                        // Relocate constants for instructions
                        for instruction in &mut cloned_function.instructions {
                            let raw_op = instruction.raw_opcode();
                            if raw_op == OpCode::LoadConst as u8 {
                                let new_const = relocate_constant(
                                    program,
                                    dep_program,
                                    instruction.imm32() as u16,
                                );
                                *instruction = Instruction::format_b(
                                    OpCode::LoadConst,
                                    instruction.flags(),
                                    instruction.dst(),
                                    new_const as u32,
                                );
                            } else if raw_op == OpCode::CallExtern as u8 {
                                let new_const = relocate_constant_string(
                                    program,
                                    dep_program,
                                    instruction.call_target(),
                                );
                                *instruction = Instruction::format_call(
                                    OpCode::CallExtern,
                                    instruction.call_dst(),
                                    new_const,
                                    instruction.arg_base(),
                                    instruction.arg_count(),
                                );
                            } else if raw_op == OpCode::AllocObjectExtern as u8 {
                                let new_const = relocate_constant_string(
                                    program,
                                    dep_program,
                                    instruction.imm32() as u16,
                                );
                                *instruction = Instruction::format_b(
                                    OpCode::AllocObjectExtern,
                                    instruction.flags(),
                                    instruction.dst(),
                                    new_const as u32,
                                );
                            }
                        }
                        let new_index = program.functions.len();
                        imported_functions.insert(dep_function.symbol.clone(), new_index);
                        program.functions.push(cloned_function.clone());
                        cloned_function
                    }
                } else {
                    continue;
                }
            }
        };

        // Scan instructions of reachable function
        for instruction in &current_function.instructions {
            let raw_op = instruction.raw_opcode();
            if raw_op == OpCode::CallDirect as u8 {
                let target_id = instruction.call_target() as usize;
                if let Some(target_func) = program.functions.get(target_id)
                    && reachable_symbols.insert(target_func.symbol.clone())
                {
                    work_queue.push_back(target_func.symbol.clone());
                }
            } else if raw_op == OpCode::CallExtern as u8 {
                let string_index = instruction.call_target() as usize;
                if let Some(ConstantPoolValue::String(called_symbol)) =
                    program.constant_pool.values.get(string_index)
                {
                    let symbol_clone = called_symbol.clone();
                    if reachable_symbols.insert(symbol_clone.clone()) {
                        work_queue.push_back(symbol_clone);
                    }
                }
            } else if raw_op == OpCode::AllocObject as u8 {
                let vtable_id = instruction.src1() as u32;
                if let Some(compiled_class) = program
                    .classes
                    .iter()
                    .find(|candidate_class| candidate_class.vtable_id == vtable_id)
                {
                    for &vtable_func_id in &compiled_class.vtable {
                        if let Some(target_func) = program.functions.get(vtable_func_id as usize)
                            && reachable_symbols.insert(target_func.symbol.clone())
                        {
                            work_queue.push_back(target_func.symbol.clone());
                        }
                    }
                    if let Some(destructor_id) = compiled_class.destructor_id
                        && let Some(target_func) = program.functions.get(destructor_id as usize)
                        && reachable_symbols.insert(target_func.symbol.clone())
                    {
                        work_queue.push_back(target_func.symbol.clone());
                    }
                }
            } else if raw_op == OpCode::AllocObjectExtern as u8 {
                let string_index = instruction.imm32() as usize;
                if let Some(ConstantPoolValue::String(class_symbol)) =
                    program.constant_pool.values.get(string_index)
                {
                    let symbol_clone = class_symbol.clone();
                    // Relocate class from dependencies if needed
                    let class_match = dependency_classes
                        .iter()
                        .find(|(dep_class, _)| symbol_matches(&dep_class.symbol, &symbol_clone));
                    if let Some(&(dep_class, dep_program)) = class_match
                        && !program
                            .classes
                            .iter()
                            .any(|candidate| candidate.symbol == dep_class.symbol)
                    {
                        let mut cloned_class = dep_class.clone();
                        cloned_class.vtable_id = (program.classes.len() + 1) as u32;
                        // Enqueue all methods in vtable and destructor
                        for &method_index in &dep_class.vtable {
                            if let Some(dep_method) =
                                dep_program.functions.get(method_index as usize)
                                && reachable_symbols.insert(dep_method.symbol.clone())
                            {
                                work_queue.push_back(dep_method.symbol.clone());
                            }
                        }
                        if let Some(destructor_index) = dep_class.destructor_id
                            && let Some(dep_method) =
                                dep_program.functions.get(destructor_index as usize)
                            && reachable_symbols.insert(dep_method.symbol.clone())
                        {
                            work_queue.push_back(dep_method.symbol.clone());
                        }
                        program.classes.push(cloned_class);
                    }
                }
            }
        }
    }

    // 3. Rewrite CallExtern to CallDirect for now-imported functions
    let function_symbols: Vec<(usize, String)> = program
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| (index, function.symbol.clone()))
        .collect();

    for function in &mut program.functions {
        for instruction in &mut function.instructions {
            let raw_op = instruction.raw_opcode();
            if raw_op == OpCode::CallExtern as u8 {
                let string_index = instruction.call_target() as usize;
                if let Some(ConstantPoolValue::String(called_symbol)) =
                    program.constant_pool.values.get(string_index)
                {
                    let target_match = function_symbols
                        .iter()
                        .find(|(_, sym)| symbol_matches(sym, called_symbol));
                    if let Some(&(target_func_id, _)) = target_match {
                        *instruction = Instruction::format_call(
                            OpCode::CallDirect,
                            instruction.call_dst(),
                            target_func_id as u16,
                            instruction.arg_base(),
                            instruction.arg_count(),
                        );
                    }
                }
            } else if raw_op == OpCode::AllocObjectExtern as u8 {
                let string_index = instruction.imm32() as usize;
                if let Some(ConstantPoolValue::String(class_symbol)) =
                    program.constant_pool.values.get(string_index)
                {
                    let target_match = program
                        .classes
                        .iter()
                        .find(|candidate| symbol_matches(&candidate.symbol, class_symbol));
                    if let Some(target_class) = target_match {
                        let flags = if target_class.is_pod() { FLAG_POD } else { 0 };
                        *instruction = Instruction::format_a(
                            OpCode::AllocObject,
                            flags,
                            instruction.dst(),
                            target_class.vtable_id as u16,
                            target_class.instance_size as u16,
                        );
                    }
                }
            }
        }
    }

    // 4. Dead Code Elimination: Retain only reachable functions
    if !reachable_symbols.is_empty() {
        let mut old_to_new_index: HashMap<usize, usize> = HashMap::new();
        let mut pruned_functions: Vec<CompiledFunction> = Vec::new();

        for (old_index, function) in program.functions.drain(..).enumerate() {
            if reachable_symbols
                .iter()
                .any(|symbol| symbol_matches(&function.symbol, symbol))
            {
                let new_index = pruned_functions.len();
                old_to_new_index.insert(old_index, new_index);
                pruned_functions.push(function);
            }
        }

        // Remap CallDirect instructions to new function indices
        for function in &mut pruned_functions {
            for instruction in &mut function.instructions {
                if instruction.raw_opcode() == OpCode::CallDirect as u8 {
                    let old_id = instruction.call_target() as usize;
                    if let Some(&new_id) = old_to_new_index.get(&old_id) {
                        *instruction = Instruction::format_call(
                            OpCode::CallDirect,
                            instruction.call_dst(),
                            new_id as u16,
                            instruction.arg_base(),
                            instruction.arg_count(),
                        );
                    }
                }
            }
        }

        // Remap class vtables
        for compiled_class in &mut program.classes {
            for vtable_entry in &mut compiled_class.vtable {
                if let Some(&new_id) = old_to_new_index.get(&(*vtable_entry as usize)) {
                    *vtable_entry = new_id as u32;
                }
            }
            for (_iface_id, itable_entries) in &mut compiled_class.itable {
                for method_entry in itable_entries {
                    if let Some(&new_id) = old_to_new_index.get(&(*method_entry as usize)) {
                        *method_entry = new_id as u32;
                    }
                }
            }
            if let Some(destructor_id) = compiled_class.destructor_id
                && let Some(&new_id) = old_to_new_index.get(&(destructor_id as usize))
            {
                compiled_class.destructor_id = Some(new_id as u32);
            }
        }

        // Remap entry point ID
        if let Some(old_entry_id) = program.entry_point_id
            && let Some(&new_entry_id) = old_to_new_index.get(&(old_entry_id as usize))
        {
            program.entry_point_id = Some(new_entry_id as u32);
        }

        program.functions = pruned_functions;
    }
}
