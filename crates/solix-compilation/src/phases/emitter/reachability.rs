//! Whole-program call-graph reachability analysis and dead-code elimination (DCE).
//!
//! Milestone 023 Pillar 3: Dual-Mode Linking (`--link dynamic` vs `--link static`).
//! In static mode, reachable functions, classes, static slots, and constant pool entries
//! from dependency binaries are relocated and merged directly into the output binary container,
//! converting `CallExtern` / `AllocObjectExtern` into direct equivalents and pruning
//! all unreachable functions.

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

fn split_call_symbol_arity(symbol: &str) -> (&str, Option<u16>) {
    if let Some((base_part, arity_part)) = symbol.rsplit_once('/')
        && let Ok(parsed_arity) = arity_part.parse::<u16>()
    {
        return (base_part, Some(parsed_arity));
    }
    (symbol, None)
}

fn bare_normalized_symbol(symbol: &str) -> String {
    let normalized = normalize_symbol(symbol);
    let without_sig = normalized.split('(').next().unwrap_or(&normalized);
    let (without_arity, _) = split_call_symbol_arity(without_sig);
    without_arity.to_string()
}

fn class_symbol_matches(candidate_symbol: &str, target_symbol: &str) -> bool {
    let normalized_candidate = normalize_symbol(candidate_symbol);
    let normalized_target = normalize_symbol(target_symbol);
    if normalized_candidate == normalized_target {
        return true;
    }
    let suffix_candidate = format!("::{normalized_candidate}");
    let suffix_target = format!("::{normalized_target}");
    normalized_candidate.ends_with(&suffix_target) || normalized_target.ends_with(&suffix_candidate)
}

fn is_initializer_or_entry_root(symbol: &str) -> bool {
    symbol == "@init"
        || symbol.ends_with("::@init")
        || symbol.contains("<init")
        || symbol.contains("<module_init>")
        || symbol == "main"
        || symbol.ends_with("::main")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedFunctionTarget {
    Local(usize),
    Dependency(usize, usize),
}

struct StaticLinker<'program> {
    program: &'program mut BinaryProgram,
    dependency_programs: &'program [BinaryProgram],
    local_function_count: usize,
    reachable_indices: HashSet<usize>,
    work_queue: VecDeque<usize>,
    imported_functions: HashMap<(usize, usize), usize>,
    imported_classes: HashMap<(usize, u32), u32>,
    dep_static_bases: HashMap<usize, u32>,
    dep_source_bases: HashMap<usize, u16>,
    next_vtable_id: u32,
    call_extern_rewrites: HashMap<(usize, usize), usize>,
    alloc_extern_rewrites: HashMap<(usize, usize), u32>,
    call_interface_ifaces: HashMap<(usize, usize), u32>,
    tuple_instanceof_vts: HashMap<(usize, usize), u32>,
}

impl<'program> StaticLinker<'program> {
    fn new(
        program: &'program mut BinaryProgram,
        dependency_programs: &'program [BinaryProgram],
    ) -> Self {
        let local_function_count = program.functions.len();
        let max_vtable_id = program
            .classes
            .iter()
            .map(|class_item| class_item.vtable_id)
            .max()
            .unwrap_or(0);
        let max_iface_id = program
            .classes
            .iter()
            .flat_map(|class_item| class_item.itable.iter().map(|(iface_id, _)| *iface_id))
            .max()
            .unwrap_or(0);
        let next_vtable_id = max_vtable_id.max(max_iface_id).saturating_add(1);

        Self {
            program,
            dependency_programs,
            local_function_count,
            reachable_indices: HashSet::new(),
            work_queue: VecDeque::new(),
            imported_functions: HashMap::new(),
            imported_classes: HashMap::new(),
            dep_static_bases: HashMap::new(),
            dep_source_bases: HashMap::new(),
            next_vtable_id,
            call_extern_rewrites: HashMap::new(),
            alloc_extern_rewrites: HashMap::new(),
            call_interface_ifaces: HashMap::new(),
            tuple_instanceof_vts: HashMap::new(),
        }
    }

    fn has_executable_entry(&self) -> bool {
        if self.program.entry_point_id.is_some() {
            return true;
        }
        if let Some(entry_symbol) = &self.program.entry_point_symbol
            && self
                .program
                .functions
                .iter()
                .take(self.local_function_count)
                .any(|function| function.symbol == *entry_symbol)
        {
            return true;
        }
        self.program
            .functions
            .iter()
            .take(self.local_function_count)
            .any(|function| function.symbol == "main" || function.symbol.ends_with("::main"))
    }

    fn seed_roots(&mut self) {
        if !self.has_executable_entry() {
            // Library builds have no executable entrypoint: keep all local functions and classes
            // reachable while still importing any statically linked dependency targets they use.
            for func_index in 0..self.local_function_count {
                self.enqueue_function_index(func_index);
            }
            return;
        }

        if let Some(entry_id) = self.program.entry_point_id
            && (entry_id as usize) < self.local_function_count
        {
            self.enqueue_function_index(entry_id as usize);
        } else if let Some(entry_symbol) = self.program.entry_point_symbol.clone() {
            for func_index in 0..self.local_function_count {
                if self.program.functions[func_index].symbol == entry_symbol {
                    self.enqueue_function_index(func_index);
                }
            }
        }

        for func_index in 0..self.local_function_count {
            if is_initializer_or_entry_root(&self.program.functions[func_index].symbol) {
                self.enqueue_function_index(func_index);
            }
        }

        // Statically imported dependencies may have companion native FFI libraries (e.g. Raylib)
        // that construct or unpack class instances by name at runtime via `solix_vm_get_class_vtable_id`.
        // Ensure all dependency class descriptors are imported into the target program.
        self.import_dependency_classes();
    }

    fn import_dependency_classes(&mut self) {
        for dep_idx in 0..self.dependency_programs.len() {
            let dep_vtables: Vec<u32> = self.dependency_programs[dep_idx]
                .classes
                .iter()
                .map(|candidate_class| candidate_class.vtable_id)
                .collect();
            for dep_vt in dep_vtables {
                self.import_dep_class(dep_idx, dep_vt);
            }
        }
    }

    fn enqueue_function_index(&mut self, func_index: usize) {
        if func_index < self.program.functions.len() && self.reachable_indices.insert(func_index) {
            self.work_queue.push_back(func_index);
        }
    }

    fn ensure_dep_source_offset(&mut self, dep_idx: usize) -> u16 {
        if let Some(&base_offset) = self.dep_source_bases.get(&dep_idx) {
            return base_offset;
        }
        let base_offset = self.program.source_files.len() as u16;
        if let Some(dep_program) = self.dependency_programs.get(dep_idx) {
            self.program
                .source_files
                .extend(dep_program.source_files.iter().cloned());
        }
        self.dep_source_bases.insert(dep_idx, base_offset);
        base_offset
    }

    fn ensure_dep_static_space(&mut self, dep_idx: usize) -> u32 {
        if let Some(&base_slot) = self.dep_static_bases.get(&dep_idx) {
            return base_slot;
        }
        let base_slot = self.program.static_space_size;
        let dep_program = &self.dependency_programs[dep_idx];
        self.program.static_space_size += dep_program.static_space_size;

        for (symbol, local_slot) in &dep_program.static_symbols_map {
            let relocated_slot = base_slot + *local_slot;
            if !self
                .program
                .static_symbols_map
                .iter()
                .any(|(existing_sym, _)| existing_sym == symbol)
            {
                self.program
                    .static_symbols_map
                    .push((symbol.clone(), relocated_slot));
            }
        }
        self.dep_static_bases.insert(dep_idx, base_slot);

        // Import dependency initializer functions so relocated static globals are initialized.
        for (dep_func_idx, dep_func) in dep_program.functions.iter().enumerate() {
            if dep_func.symbol == "@init"
                || dep_func.symbol.ends_with("::@init")
                || dep_func.symbol.contains("<module_init>")
                || dep_func.symbol.contains("<init>")
            {
                let _ = self.import_dep_function(dep_idx, dep_func_idx);
            }
        }

        base_slot
    }

    fn reserve_dep_vtable_id(&mut self, dep_idx: usize, dep_vtable_id: u32) -> u32 {
        if dep_vtable_id == 0 {
            return 0;
        }
        if let Some(&mapped_id) = self.imported_classes.get(&(dep_idx, dep_vtable_id)) {
            return mapped_id;
        }
        if let Some(imported_id) = self.import_dep_class(dep_idx, dep_vtable_id) {
            return imported_id;
        }
        let allocated_id = self.next_vtable_id;
        self.next_vtable_id = self.next_vtable_id.saturating_add(1);
        self.imported_classes
            .insert((dep_idx, dep_vtable_id), allocated_id);
        allocated_id
    }

    fn import_dep_class(&mut self, dep_idx: usize, dep_vtable_id: u32) -> Option<u32> {
        if dep_vtable_id == 0 {
            return None;
        }
        if let Some(&existing_id) = self.imported_classes.get(&(dep_idx, dep_vtable_id)) {
            return Some(existing_id);
        }

        let dep_program = self.dependency_programs.get(dep_idx)?;
        let dep_class = dep_program
            .classes
            .iter()
            .find(|candidate| candidate.vtable_id == dep_vtable_id)?;

        // Reuse already-present class if normalized symbol matches cleanly (e.g. shared diamond dep).
        let normalized_dep_sym = normalize_symbol(&dep_class.symbol);
        if let Some(existing_class) = self
            .program
            .classes
            .iter()
            .find(|candidate| normalize_symbol(&candidate.symbol) == normalized_dep_sym)
        {
            let existing_id = existing_class.vtable_id;
            self.imported_classes
                .insert((dep_idx, dep_vtable_id), existing_id);
            return Some(existing_id);
        }

        let new_vtable_id = self.next_vtable_id;
        self.next_vtable_id = self.next_vtable_id.saturating_add(1);
        self.imported_classes
            .insert((dep_idx, dep_vtable_id), new_vtable_id);

        let mut cloned_class: CompiledClass = dep_class.clone();
        cloned_class.vtable_id = new_vtable_id;

        if let Some(base_id) = dep_class.base_vtable_id {
            cloned_class.base_vtable_id = Some(self.reserve_dep_vtable_id(dep_idx, base_id));
        }

        for slot_func_id in &mut cloned_class.vtable {
            if let Some(relocated_id) = self.import_dep_function(dep_idx, *slot_func_id as usize) {
                *slot_func_id = relocated_id as u32;
            }
        }

        for (iface_id, iface_methods) in &mut cloned_class.itable {
            if *iface_id > 0 {
                *iface_id = self.reserve_dep_vtable_id(dep_idx, *iface_id);
            }
            for method_func_id in iface_methods {
                if let Some(relocated_id) =
                    self.import_dep_function(dep_idx, *method_func_id as usize)
                {
                    *method_func_id = relocated_id as u32;
                }
            }
        }

        if let Some(dtor_func_id) = dep_class.destructor_id {
            cloned_class.destructor_id = self
                .import_dep_function(dep_idx, dtor_func_id as usize)
                .map(|relocated_id| relocated_id as u32);
        }

        self.program.classes.push(cloned_class);
        Some(new_vtable_id)
    }

    fn import_dep_function(&mut self, dep_idx: usize, dep_func_idx: usize) -> Option<usize> {
        if let Some(&existing_index) = self.imported_functions.get(&(dep_idx, dep_func_idx)) {
            self.enqueue_function_index(existing_index);
            return Some(existing_index);
        }

        let dep_program = self.dependency_programs.get(dep_idx)?;
        let dep_function = dep_program.functions.get(dep_func_idx)?;

        let new_index = self.program.functions.len();
        self.imported_functions
            .insert((dep_idx, dep_func_idx), new_index);

        let source_offset = self.ensure_dep_source_offset(dep_idx);
        let mut cloned_function = dep_function.clone();
        cloned_function.source_id = source_offset.saturating_add(dep_function.source_id);

        if cloned_function.symbol == "@init" || cloned_function.symbol == "<module_init>" {
            let pkg_prefix = if dep_program.package_name.is_empty() {
                format!("__dep_{dep_idx}")
            } else {
                format!("__dep_{dep_idx}_{}", dep_program.package_name)
            };
            cloned_function.symbol = format!("{pkg_prefix}::{}", cloned_function.symbol);
        }

        // Push placeholder before relocating instructions so mutually recursive dependency calls resolve.
        self.program.functions.push(cloned_function.clone());
        self.enqueue_function_index(new_index);

        let has_dep_statics = dep_program.static_space_size > 0;
        let dep_static_base = if has_dep_statics {
            let uses_statics = cloned_function.instructions.iter().any(|instruction| {
                let raw_op = instruction.raw_opcode();
                raw_op == OpCode::GetStatic as u8 || raw_op == OpCode::SetStatic as u8
            });
            if uses_statics {
                Some(self.ensure_dep_static_space(dep_idx))
            } else {
                None
            }
        } else {
            None
        };

        for (inst_pc, instruction) in cloned_function.instructions.iter_mut().enumerate() {
            let raw_op = instruction.raw_opcode();
            if raw_op == OpCode::LoadConst as u8 || raw_op == OpCode::LoadF64 as u8 {
                if let Ok(opcode) = instruction.opcode(0) {
                    let new_const =
                        relocate_constant(self.program, dep_program, instruction.imm32() as u16);
                    *instruction = Instruction::format_b(
                        opcode,
                        instruction.flags(),
                        instruction.dst(),
                        u32::from(new_const),
                    );
                }
            } else if raw_op == OpCode::CallExtern as u8 || raw_op == OpCode::CallSymbol as u8 {
                if let Ok(opcode) = instruction.opcode(0) {
                    let new_const = relocate_constant_string(
                        self.program,
                        dep_program,
                        instruction.call_target(),
                    );
                    *instruction = Instruction::format_call(
                        opcode,
                        instruction.call_dst(),
                        new_const,
                        instruction.arg_base(),
                        instruction.arg_count(),
                    );
                }
            } else if raw_op == OpCode::AllocObjectExtern as u8
                || raw_op == OpCode::GetStaticGlobal as u8
                || raw_op == OpCode::SetStaticGlobal as u8
            {
                if let Ok(opcode) = instruction.opcode(0) {
                    let new_const = relocate_constant_string(
                        self.program,
                        dep_program,
                        instruction.imm32() as u16,
                    );
                    *instruction = Instruction::format_b(
                        opcode,
                        instruction.flags(),
                        instruction.dst(),
                        u32::from(new_const),
                    );
                }
            } else if (raw_op == OpCode::CallDirect as u8
                || raw_op == OpCode::TailCallDirect as u8
                || raw_op == OpCode::PackClosure as u8)
                && let Ok(opcode) = instruction.opcode(0)
            {
                let dep_target_idx = instruction.call_target() as usize;
                if let Some(relocated_target) = self.import_dep_function(dep_idx, dep_target_idx) {
                    *instruction = Instruction::format_call(
                        opcode,
                        instruction.call_dst(),
                        relocated_target as u16,
                        instruction.arg_base(),
                        instruction.arg_count(),
                    );
                }
            } else if raw_op == OpCode::AllocObject as u8 {
                let dep_vt = u32::from(instruction.src1());
                if let Some(new_vt) = self.import_dep_class(dep_idx, dep_vt) {
                    *instruction = Instruction::format_a(
                        OpCode::AllocObject,
                        instruction.flags(),
                        instruction.dst(),
                        new_vt as u16,
                        instruction.src2(),
                    );
                }
            } else if (raw_op == OpCode::Instanceof as u8
                || (raw_op == OpCode::CastCheck as u8 && instruction.flags() == 0))
                && instruction.src2() > 0
                && let Ok(opcode) = instruction.opcode(0)
            {
                let new_vt = self.reserve_dep_vtable_id(dep_idx, u32::from(instruction.src2()));
                *instruction = Instruction::format_a(
                    opcode,
                    instruction.flags(),
                    instruction.dst(),
                    instruction.src1(),
                    new_vt as u16,
                );
            } else if raw_op == OpCode::CallInterface as u8 {
                let target = instruction.call_target();
                let old_iface_id = u32::from(target >> 8);
                if old_iface_id > 0 {
                    let slot = target & 0xFF;
                    let new_iface_id = self.reserve_dep_vtable_id(dep_idx, old_iface_id);
                    self.call_interface_ifaces
                        .insert((new_index, inst_pc), new_iface_id);
                    let new_target = ((new_iface_id as u16) << 8) | slot;
                    *instruction = Instruction::format_call(
                        OpCode::CallInterface,
                        instruction.call_dst(),
                        new_target,
                        instruction.arg_base(),
                        instruction.arg_count(),
                    );
                }
            } else if (raw_op == OpCode::TupleInstanceof as u8
                || raw_op == OpCode::AllocArray as u8)
                && (instruction.flags() & 0x80) != 0
                && let Ok(opcode) = instruction.opcode(0)
            {
                let old_vt = if opcode == OpCode::AllocArray && instruction.src2() > 0 {
                    u32::from(instruction.src2())
                } else {
                    u32::from(instruction.flags() & 0x7F)
                };
                if old_vt > 0 {
                    let new_vt = self.reserve_dep_vtable_id(dep_idx, old_vt);
                    if opcode == OpCode::TupleInstanceof {
                        self.tuple_instanceof_vts
                            .insert((new_index, inst_pc), new_vt);
                    }
                    let new_flags = 0x80 | ((new_vt as u8) & 0x7F);
                    let new_src2 = if opcode == OpCode::AllocArray {
                        new_vt as u16
                    } else {
                        instruction.src2()
                    };
                    *instruction = Instruction::format_a(
                        opcode,
                        new_flags,
                        instruction.dst(),
                        instruction.src1(),
                        new_src2,
                    );
                }
            } else if (raw_op == OpCode::GetStatic as u8 || raw_op == OpCode::SetStatic as u8)
                && let Some(stat_base) = dep_static_base
                && instruction.imm32() < dep_program.static_space_size
                && let Ok(opcode) = instruction.opcode(0)
            {
                *instruction = Instruction::format_b(
                    opcode,
                    instruction.flags(),
                    instruction.dst(),
                    stat_base + instruction.imm32(),
                );
            }
        }

        self.program.functions[new_index] = cloned_function;
        Some(new_index)
    }

    fn enqueue_class_methods(&mut self, vtable_id: u32) {
        let mut visited_vtables = HashSet::new();
        let mut pending_vtable = Some(vtable_id);

        while let Some(active_vt) = pending_vtable {
            if !visited_vtables.insert(active_vt) {
                break;
            }
            let Some(class_snapshot) = self
                .program
                .classes
                .iter()
                .find(|candidate| candidate.vtable_id == active_vt)
                .cloned()
            else {
                break;
            };

            for func_id in class_snapshot.vtable {
                self.enqueue_function_index(func_id as usize);
            }
            for (_, method_ids) in class_snapshot.itable {
                for func_id in method_ids {
                    self.enqueue_function_index(func_id as usize);
                }
            }
            if let Some(dtor_id) = class_snapshot.destructor_id {
                self.enqueue_function_index(dtor_id as usize);
            }

            pending_vtable = class_snapshot.base_vtable_id;
        }
    }

    fn resolve_call_target(
        &self,
        called_symbol: &str,
        call_arg_count: u16,
    ) -> Option<ResolvedFunctionTarget> {
        let normalized_full = normalize_symbol(called_symbol);
        let (raw_base, _) = split_call_symbol_arity(called_symbol);
        let target_bare = bare_normalized_symbol(raw_base);
        let target_suffix = format!("::{target_bare}");

        let candidate_programs =
            std::iter::once((None, &self.program.functions[..self.local_function_count])).chain(
                self.dependency_programs
                    .iter()
                    .enumerate()
                    .map(|(dep_idx, dep_prog)| (Some(dep_idx), &dep_prog.functions[..])),
            );

        // Pass 1: Exact normalized full symbol (including signature if present)
        for (dep_opt, functions) in candidate_programs.clone() {
            for (func_idx, function) in functions.iter().enumerate() {
                if normalize_symbol(&function.symbol) == normalized_full {
                    return Some(match dep_opt {
                        None => ResolvedFunctionTarget::Local(func_idx),
                        Some(dep_idx) => ResolvedFunctionTarget::Dependency(dep_idx, func_idx),
                    });
                }
            }
        }

        // Pass 2: Arity-matching bare normalized symbol
        for (dep_opt, functions) in candidate_programs.clone() {
            for (func_idx, function) in functions.iter().enumerate() {
                if function.param_count == call_arg_count
                    && bare_normalized_symbol(&function.symbol) == target_bare
                {
                    return Some(match dep_opt {
                        None => ResolvedFunctionTarget::Local(func_idx),
                        Some(dep_idx) => ResolvedFunctionTarget::Dependency(dep_idx, func_idx),
                    });
                }
            }
        }

        // Pass 3: Arity-matching qualified suffix (e.g. `Raylib::InitWindow` -> `raylib::Raylib::InitWindow`)
        for (dep_opt, functions) in candidate_programs.clone() {
            for (func_idx, function) in functions.iter().enumerate() {
                if function.param_count != call_arg_count {
                    continue;
                }
                let candidate_bare = bare_normalized_symbol(&function.symbol);
                let candidate_suffix = format!("::{candidate_bare}");
                if candidate_bare.ends_with(&target_suffix)
                    || target_bare.ends_with(&candidate_suffix)
                {
                    return Some(match dep_opt {
                        None => ResolvedFunctionTarget::Local(func_idx),
                        Some(dep_idx) => ResolvedFunctionTarget::Dependency(dep_idx, func_idx),
                    });
                }
            }
        }

        // Pass 4: Fallback without arity check — exact bare first, then qualified suffix
        for (dep_opt, functions) in candidate_programs.clone() {
            for (func_idx, function) in functions.iter().enumerate() {
                if bare_normalized_symbol(&function.symbol) == target_bare {
                    return Some(match dep_opt {
                        None => ResolvedFunctionTarget::Local(func_idx),
                        Some(dep_idx) => ResolvedFunctionTarget::Dependency(dep_idx, func_idx),
                    });
                }
            }
        }

        for (dep_opt, functions) in candidate_programs {
            for (func_idx, function) in functions.iter().enumerate() {
                let candidate_bare = bare_normalized_symbol(&function.symbol);
                let candidate_suffix = format!("::{candidate_bare}");
                if candidate_bare.ends_with(&target_suffix)
                    || target_bare.ends_with(&candidate_suffix)
                {
                    return Some(match dep_opt {
                        None => ResolvedFunctionTarget::Local(func_idx),
                        Some(dep_idx) => ResolvedFunctionTarget::Dependency(dep_idx, func_idx),
                    });
                }
            }
        }

        None
    }

    fn find_dep_class(&self, class_symbol: &str) -> Option<(usize, u32)> {
        let normalized_target = normalize_symbol(class_symbol);
        for (dep_idx, dep_program) in self.dependency_programs.iter().enumerate() {
            for dep_class in &dep_program.classes {
                if normalize_symbol(&dep_class.symbol) == normalized_target {
                    return Some((dep_idx, dep_class.vtable_id));
                }
            }
        }
        for (dep_idx, dep_program) in self.dependency_programs.iter().enumerate() {
            for dep_class in &dep_program.classes {
                if class_symbol_matches(&dep_class.symbol, class_symbol) {
                    return Some((dep_idx, dep_class.vtable_id));
                }
            }
        }
        None
    }

    fn find_dep_static_owner(&self, global_symbol: &str) -> Option<usize> {
        let normalized_target = normalize_symbol(global_symbol);
        let simple_target = normalized_target
            .rsplit("::")
            .next()
            .unwrap_or(&normalized_target);

        for (dep_idx, dep_program) in self.dependency_programs.iter().enumerate() {
            for (dep_global, _) in &dep_program.static_symbols_map {
                let normalized_candidate = normalize_symbol(dep_global);
                if normalized_candidate == normalized_target
                    || class_symbol_matches(dep_global, global_symbol)
                    || normalized_candidate.rsplit("::").next() == Some(simple_target)
                {
                    return Some(dep_idx);
                }
            }
        }
        None
    }

    fn discover_reachable(&mut self) {
        while let Some(func_idx) = self.work_queue.pop_front() {
            let instructions = self.program.functions[func_idx].instructions.clone();
            for (inst_pc, instruction) in instructions.iter().enumerate() {
                let raw_op = instruction.raw_opcode();
                if raw_op == OpCode::CallDirect as u8
                    || raw_op == OpCode::TailCallDirect as u8
                    || raw_op == OpCode::PackClosure as u8
                {
                    self.enqueue_function_index(instruction.call_target() as usize);
                } else if raw_op == OpCode::CallExtern as u8 {
                    let string_index = instruction.call_target() as usize;
                    let called_symbol = match self.program.constant_pool.values.get(string_index) {
                        Some(ConstantPoolValue::String(sym)) => sym.clone(),
                        _ => continue,
                    };
                    if let Some(target) =
                        self.resolve_call_target(&called_symbol, u16::from(instruction.arg_count()))
                    {
                        let resolved_idx = match target {
                            ResolvedFunctionTarget::Local(local_idx) => {
                                self.enqueue_function_index(local_idx);
                                Some(local_idx)
                            }
                            ResolvedFunctionTarget::Dependency(dep_idx, dep_func_idx) => {
                                self.import_dep_function(dep_idx, dep_func_idx)
                            }
                        };
                        if let Some(target_idx) = resolved_idx {
                            self.call_extern_rewrites
                                .insert((func_idx, inst_pc), target_idx);
                        }
                    }
                } else if raw_op == OpCode::AllocObject as u8 {
                    self.enqueue_class_methods(u32::from(instruction.src1()));
                } else if raw_op == OpCode::Instanceof as u8
                    || (raw_op == OpCode::CastCheck as u8 && instruction.flags() == 0)
                {
                    if instruction.src2() > 0 {
                        self.enqueue_class_methods(u32::from(instruction.src2()));
                    }
                } else if (raw_op == OpCode::TupleInstanceof as u8
                    || raw_op == OpCode::AllocArray as u8)
                    && (instruction.flags() & 0x80) != 0
                {
                    let vtable_id = u32::from(instruction.flags() & 0x7F);
                    if vtable_id > 0 {
                        self.enqueue_class_methods(vtable_id);
                    }
                } else if raw_op == OpCode::CallInterface as u8 {
                    let iface_id = u32::from(instruction.call_target() >> 8);
                    if iface_id > 0 {
                        self.enqueue_class_methods(iface_id);
                    }
                } else if raw_op == OpCode::AllocObjectExtern as u8 {
                    let string_index = instruction.imm32() as usize;
                    let class_symbol = match self.program.constant_pool.values.get(string_index) {
                        Some(ConstantPoolValue::String(sym)) => sym.clone(),
                        _ => continue,
                    };
                    let existing_vt = self
                        .program
                        .classes
                        .iter()
                        .find(|candidate| class_symbol_matches(&candidate.symbol, &class_symbol))
                        .map(|candidate| candidate.vtable_id);
                    let target_vt = if let Some(vt) = existing_vt {
                        Some(vt)
                    } else if let Some((dep_idx, dep_vt)) = self.find_dep_class(&class_symbol) {
                        self.import_dep_class(dep_idx, dep_vt)
                    } else {
                        None
                    };
                    if let Some(resolved_vt) = target_vt {
                        self.alloc_extern_rewrites
                            .insert((func_idx, inst_pc), resolved_vt);
                        self.enqueue_class_methods(resolved_vt);
                    }
                } else if (raw_op == OpCode::GetStaticGlobal as u8
                    || raw_op == OpCode::SetStaticGlobal as u8)
                    && let Some(ConstantPoolValue::String(global_symbol)) = self
                        .program
                        .constant_pool
                        .values
                        .get(instruction.imm32() as usize)
                {
                    let global_sym_cloned = global_symbol.clone();
                    let already_present =
                        self.program
                            .static_symbols_map
                            .iter()
                            .any(|(existing_sym, _)| {
                                class_symbol_matches(existing_sym, &global_sym_cloned)
                            });
                    if !already_present
                        && let Some(dep_idx) = self.find_dep_static_owner(&global_sym_cloned)
                    {
                        let _ = self.ensure_dep_static_space(dep_idx);
                    }
                }
            }
        }
    }

    fn rewrite_external_instructions(&mut self) {
        if self.call_extern_rewrites.is_empty() && self.alloc_extern_rewrites.is_empty() {
            return;
        }
        let mut class_info_by_vtable: HashMap<u32, (usize, bool)> = HashMap::new();
        for class_item in &self.program.classes {
            class_info_by_vtable.insert(
                class_item.vtable_id,
                (class_item.instance_size, class_item.is_pod()),
            );
        }

        for (&(func_idx, inst_pc), &target_func_id) in &self.call_extern_rewrites {
            if let Some(function) = self.program.functions.get_mut(func_idx)
                && let Some(instruction) = function.instructions.get_mut(inst_pc)
            {
                *instruction = Instruction::format_call(
                    OpCode::CallDirect,
                    instruction.call_dst(),
                    target_func_id as u16,
                    instruction.arg_base(),
                    instruction.arg_count(),
                );
            }
        }

        for (&(func_idx, inst_pc), &target_vt) in &self.alloc_extern_rewrites {
            if let Some(&(instance_size, is_pod)) = class_info_by_vtable.get(&target_vt)
                && let Some(function) = self.program.functions.get_mut(func_idx)
                && let Some(instruction) = function.instructions.get_mut(inst_pc)
            {
                let flags = if is_pod { FLAG_POD } else { 0 };
                *instruction = Instruction::format_a(
                    OpCode::AllocObject,
                    flags,
                    instruction.dst(),
                    target_vt as u16,
                    instance_size as u16,
                );
            }
        }
    }

    fn prune_and_remap(&mut self) {
        if self.reachable_indices.is_empty()
            || self.reachable_indices.len() == self.program.functions.len()
        {
            return;
        }

        let mut old_to_new_index: HashMap<usize, usize> = HashMap::new();
        let mut pruned_functions: Vec<CompiledFunction> = Vec::new();

        for (old_index, function) in self.program.functions.drain(..).enumerate() {
            if self.reachable_indices.contains(&old_index) {
                let new_index = pruned_functions.len();
                old_to_new_index.insert(old_index, new_index);
                pruned_functions.push(function);
            }
        }

        for function in &mut pruned_functions {
            for instruction in &mut function.instructions {
                let raw_op = instruction.raw_opcode();
                if (raw_op == OpCode::CallDirect as u8
                    || raw_op == OpCode::TailCallDirect as u8
                    || raw_op == OpCode::PackClosure as u8)
                    && let Ok(call_op) = instruction.opcode(0)
                {
                    let old_id = instruction.call_target() as usize;
                    if let Some(&new_id) = old_to_new_index.get(&old_id) {
                        *instruction = Instruction::format_call(
                            call_op,
                            instruction.call_dst(),
                            new_id as u16,
                            instruction.arg_base(),
                            instruction.arg_count(),
                        );
                    }
                }
            }
        }

        for compiled_class in &mut self.program.classes {
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

        if let Some(old_entry_id) = self.program.entry_point_id
            && let Some(&new_entry_id) = old_to_new_index.get(&(old_entry_id as usize))
        {
            self.program.entry_point_id = Some(new_entry_id as u32);
        }

        self.call_interface_ifaces = std::mem::take(&mut self.call_interface_ifaces)
            .into_iter()
            .filter_map(|((old_idx, inst_pc), iface_id)| {
                old_to_new_index
                    .get(&old_idx)
                    .map(|&new_idx| ((new_idx, inst_pc), iface_id))
            })
            .collect();
        self.tuple_instanceof_vts = std::mem::take(&mut self.tuple_instanceof_vts)
            .into_iter()
            .filter_map(|((old_idx, inst_pc), vt_id)| {
                old_to_new_index
                    .get(&old_idx)
                    .map(|&new_idx| ((new_idx, inst_pc), vt_id))
            })
            .collect();

        self.program.functions = pruned_functions;
    }
}

/// Compacts `vtable_id`s across `program` so interfaces and `TupleInstanceof` targets receive the
/// lowest dense IDs (`1..`), preventing `CallInterface` (8-bit `iface_id`) and
/// `TupleInstanceof` (7-bit `vtable_id`) instruction-encoding truncation.
fn compact_program_vtables(
    program: &mut BinaryProgram,
    call_interface_ifaces: &HashMap<(usize, usize), u32>,
    tuple_instanceof_vts: &HashMap<(usize, usize), u32>,
) -> Result<(), solix_common::diagnostics::EmissionError> {
    if program.classes.is_empty()
        && call_interface_ifaces.is_empty()
        && tuple_instanceof_vts.is_empty()
    {
        return Ok(());
    }

    let mut tuple_priority_ids: HashSet<u32> = HashSet::new();
    let mut interface_priority_ids: HashSet<u32> = HashSet::new();
    let mut all_ids: HashSet<u32> = HashSet::new();

    for class_desc in &program.classes {
        if class_desc.vtable_id > 0 {
            all_ids.insert(class_desc.vtable_id);
        }
        if let Some(base_id) = class_desc.base_vtable_id
            && base_id > 0
        {
            all_ids.insert(base_id);
        }
        for (iface_id, _) in &class_desc.itable {
            if *iface_id > 0 {
                interface_priority_ids.insert(*iface_id);
                all_ids.insert(*iface_id);
            }
        }
    }

    for (func_idx, func) in program.functions.iter().enumerate() {
        for (inst_pc, instruction) in func.instructions.iter().enumerate() {
            let raw_op = instruction.raw_opcode();
            if raw_op == OpCode::AllocObject as u8 {
                let vt = u32::from(instruction.src1());
                if vt > 0 {
                    all_ids.insert(vt);
                }
            } else if raw_op == OpCode::Instanceof as u8
                || (raw_op == OpCode::CastCheck as u8 && instruction.flags() == 0)
            {
                let vt = u32::from(instruction.src2());
                if vt > 0 {
                    all_ids.insert(vt);
                }
            } else if raw_op == OpCode::CallInterface as u8 {
                let iface_id = call_interface_ifaces
                    .get(&(func_idx, inst_pc))
                    .copied()
                    .unwrap_or_else(|| u32::from(instruction.call_target() >> 8));
                if iface_id > 0 {
                    interface_priority_ids.insert(iface_id);
                    all_ids.insert(iface_id);
                }
            } else if raw_op == OpCode::TupleInstanceof as u8 && (instruction.flags() & 0x80) != 0 {
                let vt = tuple_instanceof_vts
                    .get(&(func_idx, inst_pc))
                    .copied()
                    .unwrap_or_else(|| u32::from(instruction.flags() & 0x7F));
                if vt > 0 {
                    tuple_priority_ids.insert(vt);
                    all_ids.insert(vt);
                }
            } else if raw_op == OpCode::AllocArray as u8 && instruction.flags() >= 0x80 {
                let vt = if instruction.src2() > 0 {
                    u32::from(instruction.src2())
                } else {
                    u32::from(instruction.flags() & 0x7F)
                };
                if vt > 0 {
                    all_ids.insert(vt);
                }
            }
        }
    }

    let mut tuple_list: Vec<u32> = tuple_priority_ids.iter().copied().collect();
    tuple_list.sort_unstable();

    let mut iface_list: Vec<u32> = interface_priority_ids
        .iter()
        .copied()
        .filter(|id| !tuple_priority_ids.contains(id))
        .collect();
    iface_list.sort_unstable();

    let mut other_list: Vec<u32> = all_ids
        .iter()
        .copied()
        .filter(|id| !tuple_priority_ids.contains(id) && !interface_priority_ids.contains(id))
        .collect();
    other_list.sort_unstable();

    let mut id_remap: HashMap<u32, u32> = HashMap::with_capacity(all_ids.len());
    let mut next_compact_id = 1u32;
    for old_id in tuple_list.into_iter().chain(iface_list).chain(other_list) {
        id_remap.insert(old_id, next_compact_id);
        next_compact_id = next_compact_id.saturating_add(1);
    }

    for class_desc in &mut program.classes {
        if class_desc.vtable_id > 0
            && let Some(&mapped_id) = id_remap.get(&class_desc.vtable_id)
        {
            class_desc.vtable_id = mapped_id;
        }
        if let Some(base_id) = class_desc.base_vtable_id
            && base_id > 0
            && let Some(&mapped_base) = id_remap.get(&base_id)
        {
            class_desc.base_vtable_id = Some(mapped_base);
        }
        for (iface_id, _) in &mut class_desc.itable {
            if *iface_id > 0
                && let Some(&mapped_iface) = id_remap.get(iface_id)
            {
                *iface_id = mapped_iface;
            }
        }
    }

    for (func_idx, func) in program.functions.iter_mut().enumerate() {
        for (inst_pc, instruction) in func.instructions.iter_mut().enumerate() {
            let raw_op = instruction.raw_opcode();
            if raw_op == OpCode::AllocObject as u8 {
                let old_vt = u32::from(instruction.src1());
                if old_vt > 0
                    && let Some(&new_vt) = id_remap.get(&old_vt)
                {
                    let new_vt_u16 = u16::try_from(new_vt).map_err(|_| {
                        solix_common::diagnostics::EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "Compacted AllocObject vtable ID {new_vt} exceeds 16-bit limit"
                            ),
                        }
                    })?;
                    *instruction = Instruction::format_a(
                        OpCode::AllocObject,
                        instruction.flags(),
                        instruction.dst(),
                        new_vt_u16,
                        instruction.src2(),
                    );
                }
            } else if raw_op == OpCode::Instanceof as u8
                || (raw_op == OpCode::CastCheck as u8 && instruction.flags() == 0)
            {
                let old_vt = u32::from(instruction.src2());
                if old_vt > 0
                    && let Some(&new_vt) = id_remap.get(&old_vt)
                    && let Ok(opcode) = instruction.opcode(inst_pc)
                {
                    let new_vt_u16 = u16::try_from(new_vt).map_err(|_| {
                        solix_common::diagnostics::EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "Compacted type-check vtable ID {new_vt} exceeds 16-bit limit"
                            ),
                        }
                    })?;
                    *instruction = Instruction::format_a(
                        opcode,
                        instruction.flags(),
                        instruction.dst(),
                        instruction.src1(),
                        new_vt_u16,
                    );
                }
            } else if raw_op == OpCode::CallInterface as u8 {
                let old_iface = call_interface_ifaces
                    .get(&(func_idx, inst_pc))
                    .copied()
                    .unwrap_or_else(|| u32::from(instruction.call_target() >> 8));
                if old_iface > 0
                    && let Some(&new_iface) = id_remap.get(&old_iface)
                {
                    let new_iface_u8 = u8::try_from(new_iface).map_err(|_| {
                        solix_common::diagnostics::EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "Interface ID {new_iface} exceeds CallInterface 8-bit opcode encoding limit (255)"
                            ),
                        }
                    })?;
                    let slot = instruction.call_target() & 0xFF;
                    let new_target = (u16::from(new_iface_u8) << 8) | slot;
                    *instruction = Instruction::format_call(
                        OpCode::CallInterface,
                        instruction.call_dst(),
                        new_target,
                        instruction.arg_base(),
                        instruction.arg_count(),
                    );
                }
            } else if raw_op == OpCode::TupleInstanceof as u8 && (instruction.flags() & 0x80) != 0 {
                let old_vt = tuple_instanceof_vts
                    .get(&(func_idx, inst_pc))
                    .copied()
                    .unwrap_or_else(|| u32::from(instruction.flags() & 0x7F));
                if old_vt > 0
                    && let Some(&new_vt) = id_remap.get(&old_vt)
                {
                    if new_vt > 0x7F {
                        return Err(
                            solix_common::diagnostics::EmissionError::ContainerSerializationFailed {
                                reason: format!(
                                    "TupleInstanceof class ID {new_vt} exceeds 7-bit type-tag limit (127)"
                                ),
                            },
                        );
                    }
                    let new_flags = 0x80 | ((new_vt as u8) & 0x7F);
                    *instruction = instruction.with_flags(new_flags);
                }
            } else if raw_op == OpCode::AllocArray as u8 && instruction.flags() >= 0x80 {
                let old_vt = if instruction.src2() > 0 {
                    u32::from(instruction.src2())
                } else {
                    u32::from(instruction.flags() & 0x7F)
                };
                if old_vt > 0
                    && let Some(&new_vt) = id_remap.get(&old_vt)
                {
                    let new_vt_u16 = u16::try_from(new_vt).map_err(|_| {
                        solix_common::diagnostics::EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "Compacted AllocArray class ID {new_vt} exceeds 16-bit limit"
                            ),
                        }
                    })?;
                    let new_flags = 0x80 | ((new_vt as u8) & 0x7F);
                    *instruction = Instruction::format_a(
                        OpCode::AllocArray,
                        new_flags,
                        instruction.dst(),
                        instruction.src1(),
                        new_vt_u16,
                    );
                }
            }
        }
    }

    Ok(())
}

/// Applies static whole-program reachability analysis and dead-code elimination.
///
/// In static mode, imports reachable functions, classes, and static globals from
/// `dependency_programs`, rewrites `OpCode::CallExtern` / `OpCode::AllocObjectExtern` to
/// `OpCode::CallDirect` / `OpCode::AllocObject`, and prunes unreachable functions.
///
/// # Errors
/// Returns `EmissionError` if compacted `vtable_id`s exceed opcode encoding bounds.
pub fn apply_reachability_and_static_linking(
    program: &mut BinaryProgram,
    dependency_programs: &[BinaryProgram],
    is_static_mode: bool,
) -> Result<(), solix_common::diagnostics::EmissionError> {
    if !is_static_mode {
        return Ok(());
    }

    let mut linker = StaticLinker::new(program, dependency_programs);
    linker.seed_roots();
    linker.discover_reachable();
    linker.rewrite_external_instructions();
    linker.prune_and_remap();
    compact_program_vtables(
        linker.program,
        &linker.call_interface_ifaces,
        &linker.tuple_instanceof_vts,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_reachability_and_static_linking;
    use solix_common::bytecode::{
        BinaryProgram, CompiledClass, CompiledFunction, Instruction, OpCode,
    };

    #[test]
    fn test_static_linker_compacts_interfaces_below_u8_limit_when_many_classes_exist() {
        let mut dep = BinaryProgram::new();
        dep.package_name = "many_classes_dep".to_string();
        let mut dep_impl = CompiledClass::new("DepImpl", 9, None, 8);
        dep_impl.itable.push((10, vec![1]));
        dep.classes.push(dep_impl);
        dep.classes
            .push(CompiledClass::new("DepIface", 10, None, 8));

        let mut dep_entry = CompiledFunction::new("dep_call", 0, 1, 2);
        dep_entry.instructions.push(Instruction::format_call(
            OpCode::CallInterface,
            0,
            10 << 8,
            0,
            1,
        ));
        dep_entry
            .instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

        let mut dep_method = CompiledFunction::new("DepImpl::ping", 0, 1, 1);
        dep_method
            .instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

        let mut dep_helper = CompiledFunction::new("dep_alloc_all", 0, 1, 2);
        dep_helper
            .instructions
            .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 9, 8));
        dep_helper
            .instructions
            .push(Instruction::format_call(OpCode::CallDirect, 0, 0, 0, 0));
        dep_helper
            .instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

        dep.functions.push(dep_entry);
        dep.functions.push(dep_method);
        dep.functions.push(dep_helper);

        let mut program = BinaryProgram::new();
        for class_idx in 1..=260u32 {
            program.classes.push(CompiledClass::new(
                format!("LocalClass{class_idx}"),
                class_idx,
                None,
                8,
            ));
        }
        let ext_idx = program
            .constant_pool
            .add_string("dep_alloc_all")
            .expect("add constant pool string");
        let mut main_fn = CompiledFunction::new("Main::main", 0, 0, 2);
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::AllocObject, 0, 0, 260, 8));
        main_fn.instructions.push(Instruction::format_call(
            OpCode::CallExtern,
            0,
            ext_idx,
            0,
            0,
        ));
        main_fn
            .instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
        program.functions.push(main_fn);
        program.entry_point = Some("Main::main".to_string());
        program.entry_point_id = Some(0);

        apply_reachability_and_static_linking(&mut program, &[dep], true)
            .expect("compacting interfaces below 255 must succeed");

        let compacted_iface = program
            .classes
            .iter()
            .find(|cls| cls.symbol == "DepIface")
            .expect("DepIface must be imported");
        assert!(
            compacted_iface.vtable_id >= 1 && compacted_iface.vtable_id <= 255,
            "DepIface must receive compact low ID <= 255, got {}",
            compacted_iface.vtable_id
        );
        let call_iface_inst = program
            .functions
            .iter()
            .flat_map(|func| func.instructions.iter())
            .find(|inst| inst.raw_opcode() == OpCode::CallInterface as u8)
            .copied()
            .expect("CallInterface instruction must exist");
        assert_eq!(
            u32::from(call_iface_inst.call_target() >> 8),
            compacted_iface.vtable_id,
            "CallInterface target high byte must match compacted interface ID"
        );
    }
}
