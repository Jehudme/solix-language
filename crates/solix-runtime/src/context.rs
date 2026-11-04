//! Runtime execution context, module loader, and relocatable symbol registries.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::native::{NativeFunction, NativeRegistry};
use crate::options::RuntimeOptions;
use solix_common::bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ConstantPool, Instruction, OpCode,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata for an individual binary loaded into the virtual machine.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedBinary {
    /// Zero-based sequence index of this binary within the loaded units list.
    pub binary_id: usize,
    /// Base slot offset in contiguous static memory reserved for this binary.
    pub static_base_offset: u32,
    /// Base offset added to local function table indices for intra-binary relocation.
    pub function_id_offset: u32,
    /// Total static memory slots required by this unit.
    pub static_space_size: u32,
    /// Default entry point function ID in this binary, if defined.
    pub entry_point_id: Option<u32>,
    /// Default entry point symbol in this binary, if defined.
    pub entry_point_symbol: Option<String>,
    /// Immutable constant pool associated with this binary.
    pub constant_pool: ConstantPool,
    /// Object class layouts and virtual method tables defined in this binary.
    pub classes: Vec<CompiledClass>,
    /// Number of compiled functions provided by this binary.
    pub function_count: usize,
}

/// Pre-computed runtime metadata for a compiled function to eliminate per-call lookups.
#[derive(Debug, Clone)]
pub struct FunctionRuntimeMeta {
    /// Index of the owning LoadedBinary in context.loaded_binaries.
    pub binary_id: usize,
    /// Function ID offset for the owning binary.
    pub func_id_offset: u32,
    /// Direct function pointer if this function is a native host function / builtin.
    pub native_fn: Option<NativeFunction>,
    /// Whether this function has any reference registers that require ARC tracking.
    pub has_references: bool,
}

/// Central virtual machine execution context coordinating memory, loaded binaries, and symbols.
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// Active runtime configuration options.
    pub options: RuntimeOptions,
    /// Metadata for each loaded binary unit.
    pub loaded_binaries: Vec<LoadedBinary>,
    /// Global lookup table mapping fully qualified symbols to global function indices.
    pub symbol_to_function_id: HashMap<String, u32>,
    /// Global lookup table mapping fully qualified static variable symbols to global static slots.
    pub symbol_to_static_slot: HashMap<String, u32>,
    /// Global lookup table mapping fully qualified class symbols to (vtable_id, instance_size).
    pub symbol_to_vtable_id: HashMap<String, (u32, usize)>,
    /// Flattened array of all compiled functions across all loaded binaries with relocated offsets.
    pub all_functions: Vec<CompiledFunction>,
    /// Flattened array of pre-computed runtime metadata for each compiled function.
    pub function_meta: Vec<FunctionRuntimeMeta>,
    /// Flattened array of all compiled class descriptors.
    pub all_classes: Vec<CompiledClass>,
    /// Fast direct lookup table for compiled classes indexed directly by `vtable_id`.
    pub vtable_lookup: Vec<Option<CompiledClass>>,
    /// Total static variable slots reserved across all loaded binaries.
    pub total_static_space: u32,
    /// Global function ID of the resolved program entry point, if available.
    pub entry_point_function_id: Option<u32>,
    /// List of static module `@init` function IDs to execute prior to the entry point.
    pub init_function_ids: Vec<u32>,
}

impl RuntimeContext {
    /// Constructs a new `RuntimeContext` from the provided configuration options.
    ///
    /// Reads any specified binary files, resolves multi-binary partition offsets,
    /// registers exported symbols, and locates the program entry point.
    ///
    /// # Errors
    /// Returns `RuntimeError::BinaryLoadError` if reading from disk or deserialization fails.
    pub fn new(mut options: RuntimeOptions) -> Result<Self, RuntimeError> {
        // Load file-based binaries from paths
        let paths = std::mem::take(&mut options.binary_paths);
        for path in paths {
            let binary = Self::load_binary_file(&path)?;
            options.binaries.push(binary);
        }

        let mut loaded_binaries = Vec::with_capacity(options.binaries.len());
        let mut symbol_to_function_id = HashMap::new();
        let mut symbol_to_static_slot = HashMap::new();
        let mut symbol_to_vtable_id = HashMap::new();
        let mut all_functions = Vec::new();
        let mut all_classes = Vec::new();
        let mut init_function_ids = Vec::new();

        let mut current_static_offset: u32 = 0;
        let mut current_function_offset: u32 = 0;
        let mut current_vtable_offset: u32 = 0;

        for (binary_idx, binary) in options.binaries.iter().enumerate() {
            let func_offset = current_function_offset;
            let stat_offset = current_static_offset;
            let vtable_offset = current_vtable_offset;
            let func_count = binary.functions.len();

            let loaded_unit = LoadedBinary {
                binary_id: binary_idx,
                static_base_offset: stat_offset,
                function_id_offset: func_offset,
                static_space_size: binary.static_space_size,
                entry_point_id: binary.entry_point_id,
                entry_point_symbol: binary.entry_point_symbol.clone(),
                constant_pool: binary.constant_pool.clone(),
                classes: binary.classes.clone(),
                function_count: func_count,
            };

            // Register static symbols
            for (symbol, local_slot) in &binary.static_symbols_map {
                let global_slot = stat_offset + *local_slot;
                symbol_to_static_slot.insert(symbol.clone(), global_slot);
                let simple = symbol.rsplit(['.', ':']).next().unwrap_or(symbol);
                if simple != symbol {
                    symbol_to_static_slot
                        .entry(simple.to_string())
                        .or_insert(global_slot);
                }
            }

            // Register and relocate classes
            let mut max_vtable_id = 0;
            for class in &binary.classes {
                if class.vtable_id >= max_vtable_id {
                    max_vtable_id = class.vtable_id + 1;
                }
                let mut relocated_class = class.clone();
                relocated_class.vtable_id += vtable_offset;
                if let Some(base_id) = relocated_class.base_vtable_id {
                    relocated_class.base_vtable_id = Some(base_id + vtable_offset);
                }
                for slot_func in &mut relocated_class.vtable {
                    *slot_func += func_offset;
                }
                for (_iface_id, iface_methods) in &mut relocated_class.itable {
                    for slot_func in iface_methods {
                        *slot_func += func_offset;
                    }
                }
                if let Some(dtor_id) = &mut relocated_class.destructor_id {
                    *dtor_id += func_offset;
                }

                symbol_to_vtable_id.insert(
                    relocated_class.symbol.clone(),
                    (relocated_class.vtable_id, relocated_class.instance_size),
                );
                let simple = relocated_class
                    .symbol
                    .rsplit(['.', ':'])
                    .next()
                    .unwrap_or(&relocated_class.symbol);
                if simple != relocated_class.symbol {
                    symbol_to_vtable_id
                        .entry(simple.to_string())
                        .or_insert((relocated_class.vtable_id, relocated_class.instance_size));
                }

                all_classes.push(relocated_class);
            }

            // Register functions and relocate static base offset + instructions
            for (local_idx, func) in binary.functions.iter().enumerate() {
                let global_func_id = func_offset + local_idx as u32;
                symbol_to_function_id.insert(func.symbol.clone(), global_func_id);
                let clean = func.symbol.split('(').next().unwrap_or(&func.symbol);
                if clean != func.symbol {
                    symbol_to_function_id.insert(clean.to_string(), global_func_id);
                }
                let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
                if simple != clean {
                    symbol_to_function_id
                        .entry(simple.to_string())
                        .or_insert(global_func_id);
                }

                let mut relocated_func = func.clone();
                relocated_func.static_base_offset = stat_offset;

                // Rewrite intra-module static instructions to direct global slots at load time
                if binary.static_space_size > 0 {
                    for inst in &mut relocated_func.instructions {
                        if inst.raw_opcode() == OpCode::GetStatic as u8
                            && inst.imm32() < binary.static_space_size
                        {
                            let absolute_slot = stat_offset + inst.imm32();
                            *inst = Instruction::format_b(
                                OpCode::GetStaticDirect,
                                0,
                                inst.dst(),
                                absolute_slot,
                            );
                        } else if inst.raw_opcode() == OpCode::SetStatic as u8
                            && inst.imm32() < binary.static_space_size
                        {
                            let absolute_slot = stat_offset + inst.imm32();
                            *inst = Instruction::format_b(
                                OpCode::SetStaticDirect,
                                0,
                                inst.dst(),
                                absolute_slot,
                            );
                        }
                    }
                }

                // Relocate intra-binary call targets and object vtable IDs if func_offset > 0 or vtable_offset > 0
                if func_offset > 0 || vtable_offset > 0 {
                    for inst in &mut relocated_func.instructions {
                        if func_offset > 0
                            && (inst.raw_opcode() == OpCode::CallDirect as u8
                                || inst.raw_opcode() == OpCode::PackClosure as u8)
                        {
                            let old_target = inst.call_target();
                            if let Ok(op) = inst.opcode(0) {
                                *inst = Instruction::format_call(
                                    op,
                                    inst.call_dst(),
                                    old_target + func_offset as u16,
                                    inst.arg_base(),
                                    inst.arg_count(),
                                );
                            }
                        }
                        if vtable_offset > 0 {
                            if inst.raw_opcode() == OpCode::AllocObject as u8 {
                                let old_vtable_id = inst.src1();
                                let size = inst.src2();
                                *inst = Instruction::format_a(
                                    OpCode::AllocObject,
                                    inst.flags(),
                                    inst.dst(),
                                    old_vtable_id + vtable_offset as u16,
                                    size,
                                );
                            } else if inst.raw_opcode() == OpCode::Instanceof as u8 {
                                let old_target = inst.src2();
                                *inst = Instruction::format_a(
                                    OpCode::Instanceof,
                                    inst.flags(),
                                    inst.dst(),
                                    inst.src1(),
                                    old_target + vtable_offset as u16,
                                );
                            }
                        }
                    }
                }

                // Detect @init functions
                if func.symbol.ends_with("@init") || func.symbol.ends_with("::@init") {
                    init_function_ids.push(global_func_id);
                }

                all_functions.push(relocated_func);
            }

            current_static_offset += binary.static_space_size;
            current_function_offset += func_count as u32;
            current_vtable_offset += max_vtable_id.max(binary.classes.len() as u32);
            loaded_binaries.push(loaded_unit);
        }

        // Resolve entry point
        let entry_point_function_id = Self::resolve_entry_point(
            &options.entry_point,
            &loaded_binaries,
            &symbol_to_function_id,
        );

        let max_relocated_vtable_id = all_classes
            .iter()
            .map(|class_entry| class_entry.vtable_id)
            .max()
            .unwrap_or(0);
        let mut vtable_lookup = vec![None; (max_relocated_vtable_id + 1) as usize];
        for class_entry in &all_classes {
            vtable_lookup[class_entry.vtable_id as usize] = Some(class_entry.clone());
        }

        let mut native_registry = NativeRegistry::with_builtins();
        for (native_name, native_callback) in &options.native_functions {
            native_registry.register(native_name, *native_callback);
        }

        let mut function_meta = Vec::with_capacity(all_functions.len());
        for (global_id, func) in all_functions.iter().enumerate() {
            let binary_id = loaded_binaries
                .iter()
                .position(|binary_unit| {
                    (global_id as u32) >= binary_unit.function_id_offset
                        && (global_id as u32)
                            < binary_unit.function_id_offset + binary_unit.function_count as u32
                })
                .unwrap_or(0);
            let func_id_offset = loaded_binaries
                .get(binary_id)
                .map_or(0, |binary_unit| binary_unit.function_id_offset);

            let clean_symbol = func.symbol.split('(').next().unwrap_or(&func.symbol);
            let simple_symbol = clean_symbol
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(clean_symbol);
            let has_qualifier = clean_symbol.contains("::");
            let native_fn = native_registry
                .get(&func.symbol)
                .copied()
                .or_else(|| native_registry.get(clean_symbol).copied())
                .or_else(|| {
                    if !has_qualifier {
                        native_registry.get(simple_symbol).copied()
                    } else {
                        None
                    }
                });

            let has_references = !func.reference_registers.is_empty();

            function_meta.push(FunctionRuntimeMeta {
                binary_id,
                func_id_offset,
                native_fn,
                has_references,
            });
        }

        // Annotate AllocObject instructions with FLAG_POD if the allocated class is POD
        for func in &mut all_functions {
            for inst in &mut func.instructions {
                if inst.raw_opcode() == OpCode::AllocObject as u8 {
                    let vtable_id = inst.src1() as usize;
                    if let Some(Some(class_item)) = vtable_lookup.get(vtable_id)
                        && class_item.is_pod()
                    {
                        *inst = inst.with_flags(
                            inst.flags() | solix_common::bytecode::instruction::FLAG_POD,
                        );
                    }
                }
            }
        }

        // Tier 1 AOT Bytecode Verifier: verify register windows, branch targets, and static space
        crate::verifier::verify_program(&loaded_binaries, &all_functions, current_static_offset)?;

        Ok(Self {
            options,
            loaded_binaries,
            symbol_to_function_id,
            symbol_to_static_slot,
            symbol_to_vtable_id,
            all_functions,
            function_meta,
            all_classes,
            vtable_lookup,
            total_static_space: current_static_offset,
            entry_point_function_id,
            init_function_ids,
        })
    }

    /// Returns a reference to the `CompiledClass` associated with `vtable_id`, if registered.
    #[must_use]
    pub fn get_class_by_vtable_id(&self, vtable_id: u32) -> Option<&CompiledClass> {
        self.vtable_lookup
            .get(vtable_id as usize)
            .and_then(Option::as_ref)
    }

    /// Resolves the global entry point function ID.
    fn resolve_entry_point(
        entry_override: &Option<String>,
        loaded_binaries: &[LoadedBinary],
        symbol_to_function_id: &HashMap<String, u32>,
    ) -> Option<u32> {
        // 1. Explicit override in RuntimeOptions
        if let Some(entry_sym) = entry_override {
            if let Some(id) = symbol_to_function_id.get(entry_sym) {
                return Some(*id);
            }
            for (sym, id) in symbol_to_function_id {
                let sym_base = sym.split('(').next().unwrap_or(sym);
                if sym_base == entry_sym
                    || sym_base.ends_with(entry_sym)
                        && (sym_base.len() == entry_sym.len()
                            || sym_base.as_bytes()[sym_base.len() - entry_sym.len() - 1] == b'.'
                            || sym_base.as_bytes()[sym_base.len() - entry_sym.len() - 1] == b':')
                {
                    return Some(*id);
                }
            }
            return None;
        }

        // 2. Primary binary's entry_point_id
        if let Some(primary) = loaded_binaries.first() {
            if let Some(local_id) = primary.entry_point_id {
                return Some(primary.function_id_offset + local_id);
            }

            // 3. Primary binary's entry_point_symbol
            if let Some(ref sym) = primary.entry_point_symbol
                && let Some(id) = symbol_to_function_id.get(sym)
            {
                return Some(*id);
            }
        }

        // 4. Fallback lookup for "main" symbol
        symbol_to_function_id.get("main").copied()
    }

    /// Reads and deserializes a `BinaryProgram` from a disk file path.
    fn load_binary_file(path: &Path) -> Result<BinaryProgram, RuntimeError> {
        let bytes = fs::read(path).map_err(|err| RuntimeError::BinaryLoadError {
            reason: format!("failed to read file '{path:?}': {err}"),
        })?;
        BinaryProgram::from_bytes(&bytes).map_err(|err| RuntimeError::BinaryLoadError {
            reason: format!("failed to deserialize binary from '{path:?}': {err:?}"),
        })
    }

    /// Retrieves a reference to a function by its global function ID.
    #[must_use]
    pub fn get_function(&self, id: u32) -> Option<&CompiledFunction> {
        self.all_functions.get(id as usize)
    }

    /// Retrieves a reference to a function by its fully qualified symbol.
    #[must_use]
    pub fn get_function_by_symbol(&self, symbol: &str) -> Option<&CompiledFunction> {
        let id = self.symbol_to_function_id.get(symbol)?;
        self.get_function(*id)
    }

    /// Resolves the global static memory slot for an exported static variable symbol.
    #[must_use]
    pub fn resolve_static_slot(&self, symbol: &str) -> Option<u32> {
        if let Some(&slot) = self.symbol_to_static_slot.get(symbol) {
            return Some(slot);
        }
        let simple = symbol.rsplit(['.', ':']).next().unwrap_or(symbol);
        if let Some(&slot) = self.symbol_to_static_slot.get(simple) {
            return Some(slot);
        }
        for (sym, slot) in &self.symbol_to_static_slot {
            if sym == symbol || sym.ends_with(symbol) || sym.ends_with(simple) {
                return Some(*slot);
            }
        }
        None
    }

    /// Resolves a function's global function ID by its fully qualified or simple symbol name.
    #[must_use]
    pub fn resolve_function_symbol(&self, symbol: &str) -> Option<u32> {
        if let Some(&id) = self.symbol_to_function_id.get(symbol) {
            return Some(id);
        }
        if let Some((base, _arity)) = symbol.rsplit_once('/') {
            if let Some(&id) = self.symbol_to_function_id.get(base) {
                return Some(id);
            }
            let simple = base.rsplit(['.', ':']).next().unwrap_or(base);
            if let Some(&id) = self.symbol_to_function_id.get(simple) {
                return Some(id);
            }
        }
        let clean = symbol.split(['(', '/']).next().unwrap_or(symbol);
        if let Some(&id) = self.symbol_to_function_id.get(clean) {
            return Some(id);
        }
        let simple = clean.rsplit(['.', ':']).next().unwrap_or(clean);
        if let Some(&id) = self.symbol_to_function_id.get(simple) {
            return Some(id);
        }
        for (sym, id) in &self.symbol_to_function_id {
            let sym_clean = sym.split(['(', '/']).next().unwrap_or(sym);
            if sym_clean == clean || sym_clean.ends_with(clean) {
                return Some(*id);
            }
            if clean.ends_with("::constructor") {
                let cls = clean.strip_suffix("::constructor").unwrap_or("");
                let ctor_alias = format!("{cls}::{cls}");
                if sym_clean == ctor_alias || sym_clean.ends_with(&ctor_alias) {
                    return Some(*id);
                }
            }
            if let Some((cls, method)) = clean.rsplit_once("::")
                && method == cls
            {
                let ctor_alias = format!("{cls}::constructor");
                if sym_clean == ctor_alias || sym_clean.ends_with(&ctor_alias) {
                    return Some(*id);
                }
            }
        }
        None
    }

    /// Resolves class virtual table ID and instance size in bytes by class symbol name.
    #[must_use]
    pub fn resolve_vtable_info(&self, symbol: &str) -> Option<(u32, usize)> {
        if let Some(&info) = self.symbol_to_vtable_id.get(symbol) {
            return Some(info);
        }
        let simple = symbol.rsplit(['.', ':']).next().unwrap_or(symbol);
        if let Some(&info) = self.symbol_to_vtable_id.get(simple) {
            return Some(info);
        }
        for (sym, info) in &self.symbol_to_vtable_id {
            if sym == symbol || sym.ends_with(symbol) || sym.ends_with(simple) {
                return Some(*info);
            }
        }
        None
    }

    /// Modifies a compiled instruction in-place for instruction quickening.
    pub fn patch_instruction(&mut self, func_id: u32, pc: usize, new_inst: Instruction) {
        if let Some(func) = self.all_functions.get_mut(func_id as usize)
            && let Some(inst) = func.instructions.get_mut(pc)
        {
            *inst = new_inst;
        }
    }

    /// Returns the constant pool associated with a given binary index.
    #[must_use]
    pub fn get_constant_pool(&self, binary_id: usize) -> Option<&ConstantPool> {
        self.loaded_binaries
            .get(binary_id)
            .map(|b| &b.constant_pool)
    }
}
