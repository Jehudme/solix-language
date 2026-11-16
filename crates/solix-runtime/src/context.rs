//! Runtime execution context, module loader, and relocatable symbol registries.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::native::{LoadedSharedLibrary, NativeCallback, NativeRegistry};
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
    /// Whether this binary was compiled with the debug profile.
    pub is_debug_profile: bool,
    /// Canonical source files registered with this binary.
    pub source_files: Vec<String>,
}

/// Pre-computed runtime metadata for a compiled function to eliminate per-call lookups.
#[derive(Debug, Clone)]
pub struct FunctionRuntimeMeta {
    /// Index of the owning LoadedBinary in context.loaded_binaries.
    pub binary_id: usize,
    /// Direct function pointer if this function is a native host function / builtin.
    pub native_fn: Option<NativeCallback>,
    /// Function ID offset for the owning binary.
    pub func_id_offset: u32,
    /// Relocated static variable base slot offset for the owning module.
    pub static_base_offset: u32,
    /// Virtual register window size required by this function frame.
    pub num_registers: u16,
    /// Whether this function has any reference registers that require ARC tracking.
    pub has_references: bool,
    /// Whether this function has debug symbols (source map and/or local variables).
    pub has_debug_symbols: bool,
}

/// Central virtual machine execution context coordinating memory, loaded binaries, and symbols.
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// Active runtime configuration options.
    pub options: RuntimeOptions,
    /// Loaded dynamic shared libraries kept alive for runtime execution.
    pub shared_libraries: Vec<LoadedSharedLibrary>,
    /// Resolved host and C-ABI native functions repository.
    pub native_registry: NativeRegistry,
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
    /// Compact `O(1)` virtual method tables indexed by `[vtable_id][slot]`.
    pub flat_vtables: Vec<Box<[u32]>>,
    /// Compact `O(1)` interface method tables indexed by `[vtable_id][iface_id][slot]`.
    pub flat_itables: Vec<Box<[Box<[u32]>]>>,
    /// Pre-computed static acyclicity flag per `vtable_id` (skips cycle collector purple push when `true`).
    pub class_acyclic: Vec<bool>,
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

        // Verify SemVer compatibility across dependencies and auto-mount external modules
        verify_and_resolve_runtime_dependencies(&mut options)?;

        let mut loaded_binaries = Vec::with_capacity(options.binaries.len());
        let mut symbol_to_function_id = HashMap::new();
        let mut symbol_to_static_slot = HashMap::new();
        let mut symbol_to_vtable_id = HashMap::new();
        let mut all_functions = Vec::new();
        let mut all_classes = Vec::new();
        let mut init_function_ids = Vec::new();

        let mut defined_functions: HashMap<String, String> = HashMap::new();
        let mut defined_classes: HashMap<String, String> = HashMap::new();
        let mut defined_statics: HashMap<String, String> = HashMap::new();

        let generic_template_names: std::collections::HashSet<String> = options
            .binaries
            .iter()
            .flat_map(|binary_item| {
                binary_item
                    .export_manifest
                    .generic_templates
                    .iter()
                    .map(|template_item| template_item.name.clone())
            })
            .collect();

        let mut current_static_offset: u32 = 0;
        let mut current_function_offset: u32 = 0;
        let mut current_vtable_offset: u32 = 0;

        for (binary_idx, binary) in options.binaries.iter().enumerate() {
            let binary_label = if binary.package_name.is_empty() {
                format!("binary#{binary_idx}")
            } else {
                format!("{}@{}", binary.package_name, binary.package_version)
            };

            // Check duplicate static symbols across modules
            for (static_symbol, _) in &binary.static_symbols_map {
                if let Some(first_module) = defined_statics.get(static_symbol) {
                    if first_module != &binary_label {
                        return Err(RuntimeError::DuplicateSymbolDefinition {
                            kind: "static variable".to_string(),
                            symbol: static_symbol.clone(),
                            first_module: first_module.clone(),
                            second_module: binary_label.clone(),
                        });
                    }
                } else {
                    defined_statics.insert(static_symbol.clone(), binary_label.clone());
                }
            }

            // Check duplicate classes across modules
            for class in &binary.classes {
                let is_generic_instantiation =
                    class.symbol.contains('<') && class.symbol.contains('>');
                let is_generic_template = generic_template_names.iter().any(|template_name| {
                    &class.symbol == template_name
                        || class.symbol.ends_with(&format!("::{template_name}"))
                        || template_name.ends_with(&format!("::{}", class.symbol))
                });
                if !is_generic_instantiation && !is_generic_template {
                    if let Some(first_module) = defined_classes.get(&class.symbol) {
                        if first_module != &binary_label {
                            return Err(RuntimeError::DuplicateSymbolDefinition {
                                kind: "class".to_string(),
                                symbol: class.symbol.clone(),
                                first_module: first_module.clone(),
                                second_module: binary_label.clone(),
                            });
                        }
                    } else {
                        defined_classes.insert(class.symbol.clone(), binary_label.clone());
                    }
                }
            }

            // Check duplicate functions across modules
            for func in &binary.functions {
                let is_synthetic = func.symbol.starts_with('@')
                    || func.symbol.starts_with("__static_init")
                    || func.symbol.starts_with("<entry_")
                    || func.symbol.starts_with("<lambda_");
                let is_generic_instantiation =
                    func.symbol.contains('<') && func.symbol.contains('>');
                let is_generic_template = generic_template_names.contains(&func.symbol)
                    || generic_template_names
                        .iter()
                        .any(|template_name| func.symbol.starts_with(template_name));
                if !is_synthetic && !is_generic_instantiation && !is_generic_template {
                    if let Some(first_module) = defined_functions.get(&func.symbol) {
                        if first_module != &binary_label {
                            return Err(RuntimeError::DuplicateSymbolDefinition {
                                kind: "function".to_string(),
                                symbol: func.symbol.clone(),
                                first_module: first_module.clone(),
                                second_module: binary_label.clone(),
                            });
                        }
                    } else {
                        defined_functions.insert(func.symbol.clone(), binary_label.clone());
                    }
                }
            }

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
                is_debug_profile: binary.build_profile == "debug",
                source_files: binary.source_files.clone(),
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
                            } else if inst.raw_opcode() == OpCode::CastCheck as u8
                                && inst.flags() == 0
                            {
                                let old_target = inst.src2();
                                *inst = Instruction::format_a(
                                    OpCode::CastCheck,
                                    0,
                                    inst.dst(),
                                    inst.src1(),
                                    old_target + vtable_offset as u16,
                                );
                            } else if inst.raw_opcode() == OpCode::CallInterface as u8 {
                                let target = inst.call_target();
                                let old_iface_id = target >> 8;
                                if old_iface_id > 0 {
                                    let slot = target & 0xFF;
                                    let new_target =
                                        ((old_iface_id + vtable_offset as u16) << 8) | slot;
                                    *inst = Instruction::format_call(
                                        OpCode::CallInterface,
                                        inst.call_dst(),
                                        new_target,
                                        inst.arg_base(),
                                        inst.arg_count(),
                                    );
                                }
                            } else if inst.raw_opcode() == OpCode::TupleInstanceof as u8
                                && (inst.flags() & 0x80) != 0
                            {
                                let old_vtable_id = inst.flags() & 0x7F;
                                let new_flags =
                                    0x80 | ((old_vtable_id + vtable_offset as u8) & 0x7F);
                                *inst = Instruction::format_a(
                                    OpCode::TupleInstanceof,
                                    new_flags,
                                    inst.dst(),
                                    inst.src1(),
                                    inst.src2(),
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
        let max_iface_id = all_classes
            .iter()
            .flat_map(|class_entry| class_entry.itable.iter().map(|(iface_id, _)| *iface_id))
            .max()
            .unwrap_or(0)
            .max(max_relocated_vtable_id);

        let mut vtable_lookup = vec![None; (max_relocated_vtable_id + 1) as usize];
        let mut flat_vtables: Vec<Box<[u32]>> =
            vec![Box::default(); (max_relocated_vtable_id + 1) as usize];
        let mut flat_itables: Vec<Box<[Box<[u32]>]>> =
            vec![Box::default(); (max_relocated_vtable_id + 1) as usize];

        for class_entry in &all_classes {
            let class_idx = class_entry.vtable_id as usize;
            vtable_lookup[class_idx] = Some(class_entry.clone());
            flat_vtables[class_idx] = class_entry.vtable.clone().into_boxed_slice();
            if !class_entry.itable.is_empty() {
                let mut iface_rows = vec![Box::<[u32]>::default(); (max_iface_id + 1) as usize];
                for (iface_id, method_slots) in &class_entry.itable {
                    iface_rows[*iface_id as usize] = method_slots.clone().into_boxed_slice();
                }
                flat_itables[class_idx] = iface_rows.into_boxed_slice();
            }
        }
        let class_acyclic = compute_class_acyclicity(&vtable_lookup);

        let mut native_registry = NativeRegistry::with_builtins();
        for (native_name, native_callback) in &options.native_functions {
            native_registry.register(native_name, *native_callback);
        }

        let mut shared_libraries = Vec::new();
        for library_path in &options.native_libraries {
            if options.verify_checksums {
                let file_name = library_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                let expected_checksum = options.binaries.iter().find_map(|binary_program| {
                    binary_program
                        .native_library_checksums
                        .iter()
                        .find_map(|(path_key, hash)| {
                            if path_key.ends_with(file_name)
                                || library_path.to_string_lossy().ends_with(path_key)
                            {
                                Some(hash.clone())
                            } else {
                                None
                            }
                        })
                });

                if let Some(expected) = expected_checksum {
                    let actual =
                        solix_common::hash::sha256_file(library_path).map_err(|io_error| {
                            RuntimeError::SharedLibraryError {
                                path: library_path.display().to_string(),
                                reason: io_error.to_string(),
                            }
                        })?;
                    if actual != expected {
                        return Err(RuntimeError::NativeLibraryChecksumMismatch {
                            path: library_path.display().to_string(),
                            expected,
                            actual,
                        });
                    }
                }
            }

            let mut loader = crate::native::ffi::SharedLibraryLoader::new();
            loader.load_library(library_path)?;
            for (symbol, function_ptr) in loader.registered_functions {
                native_registry.register_c(symbol, function_ptr);
            }
            for library in loader.loaded_libraries {
                shared_libraries.push(LoadedSharedLibrary(std::sync::Arc::new(library)));
            }
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
            let mut native_fn = native_registry
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

            // If not found in native_registry, probe loaded shared libraries directly for fallback symbols
            if native_fn.is_none() && !shared_libraries.is_empty() {
                for loaded_library in &shared_libraries {
                    if let Some(c_func) = loaded_library.get_symbol(&func.symbol) {
                        native_fn = Some(NativeCallback::CAbi(c_func));
                        native_registry.register_c(&func.symbol, c_func);
                        break;
                    }
                }
            }

            let has_references = !func.reference_registers.is_empty();
            let is_debug_profile = loaded_binaries
                .get(binary_id)
                .is_some_and(|binary| binary.is_debug_profile);
            let has_debug_symbols = is_debug_profile && !func.source_map.is_empty();

            function_meta.push(FunctionRuntimeMeta {
                binary_id,
                native_fn,
                func_id_offset,
                static_base_offset: func.static_base_offset,
                num_registers: func.num_registers,
                has_references,
                has_debug_symbols,
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
            shared_libraries,
            native_registry,
            loaded_binaries,
            symbol_to_function_id,
            symbol_to_static_slot,
            symbol_to_vtable_id,
            all_functions,
            function_meta,
            all_classes,
            vtable_lookup,
            flat_vtables,
            flat_itables,
            class_acyclic,
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

    /// Returns `true` if instances of `vtable_id` cannot form ownership cycles on the reference graph.
    #[inline(always)]
    #[must_use]
    pub fn is_class_acyclic(&self, vtable_id: u32) -> bool {
        self.class_acyclic
            .get(vtable_id as usize)
            .copied()
            .unwrap_or(false)
    }

    /// Resolves a function's global ID for entry point invocation using exact, package-qualified,
    /// and unqualified fallback lookups.
    ///
    /// # Errors
    /// Returns `RuntimeError::EntryPointNotFound` if the symbol cannot be resolved.
    pub fn resolve_entry_function_id(&self, symbol: &str) -> Result<u32, RuntimeError> {
        // 1. Exact match in symbol_to_function_id
        if let Some(&func_id) = self.symbol_to_function_id.get(symbol) {
            return Ok(func_id);
        }

        // Clean symbol without arity/parameters
        let clean_symbol = symbol.split('(').next().unwrap_or(symbol);
        if let Some(&func_id) = self.symbol_to_function_id.get(clean_symbol) {
            return Ok(func_id);
        }

        let primary_pkg = self
            .options
            .binaries
            .first()
            .map(|primary_binary| primary_binary.package_name.as_str())
            .unwrap_or("");

        // 2. If symbol does not contain "::" (e.g. "main"), probe <pkg>::<symbol> and Main::<symbol>
        if !symbol.contains("::") {
            if !primary_pkg.is_empty() {
                let pkg_qualified = format!("{primary_pkg}::{clean_symbol}");
                if let Some(&func_id) = self.symbol_to_function_id.get(&pkg_qualified) {
                    return Ok(func_id);
                }
            }
            let main_class_qualified = format!("Main::{clean_symbol}");
            if let Some(&func_id) = self.symbol_to_function_id.get(&main_class_qualified) {
                return Ok(func_id);
            }
            if !primary_pkg.is_empty() {
                let pkg_main_qualified = format!("{primary_pkg}::Main::{clean_symbol}");
                if let Some(&func_id) = self.symbol_to_function_id.get(&pkg_main_qualified) {
                    return Ok(func_id);
                }
            }
        } else {
            // 3. If symbol is "<pkg>::<fn_name>" (e.g., "sample_workspace::main"), check if primary package is "<pkg>"
            if !primary_pkg.is_empty()
                && (symbol.starts_with(primary_pkg) || clean_symbol.starts_with(primary_pkg))
            {
                let suffix = clean_symbol
                    .strip_prefix(primary_pkg)
                    .unwrap_or(clean_symbol);
                let fn_name = suffix.strip_prefix("::").unwrap_or(suffix);
                if let Some(&func_id) = self.symbol_to_function_id.get(fn_name) {
                    return Ok(func_id);
                }
                let pkg_main_qualified = format!("{primary_pkg}::Main::{fn_name}");
                if let Some(&func_id) = self.symbol_to_function_id.get(&pkg_main_qualified) {
                    return Ok(func_id);
                }
            }
        }

        // Suffix scan fallback
        for (candidate_sym, &func_id) in &self.symbol_to_function_id {
            let candidate_clean = candidate_sym.split('(').next().unwrap_or(candidate_sym);
            if candidate_clean == symbol
                || candidate_clean == clean_symbol
                || candidate_clean.ends_with(&format!("::{clean_symbol}"))
                || clean_symbol.ends_with(&format!("::{candidate_clean}"))
            {
                return Ok(func_id);
            }
        }

        Err(RuntimeError::EntryPointNotFound {
            symbol: symbol.to_string(),
        })
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
            .map(|binary| &binary.constant_pool)
    }
}

fn verify_and_resolve_runtime_dependencies(
    options: &mut RuntimeOptions,
) -> Result<(), RuntimeError> {
    let mut work_queue: std::collections::VecDeque<(
        solix_common::solver::DependencyRequirement,
        String,
    )> = std::collections::VecDeque::new();
    let mut loaded_packages: std::collections::HashMap<
        String,
        (solix_common::solver::PackageVersion, usize),
    > = std::collections::HashMap::new();

    for (binary_index, binary) in options.binaries.iter().enumerate() {
        if !binary.package_name.is_empty()
            && let Ok(version) =
                solix_common::solver::PackageVersion::parse(&binary.package_version)
        {
            loaded_packages.insert(binary.package_name.clone(), (version, binary_index));
        }
        let requester_name = if !binary.package_name.is_empty() {
            binary.package_name.clone()
        } else {
            "root binary".to_string()
        };
        for requirement in &binary.export_manifest.dependencies {
            work_queue.push_back((requirement.clone(), requester_name.clone()));
        }
    }

    while let Some((requirement, requester_name)) = work_queue.pop_front() {
        if let Some((existing_version, existing_idx)) = loaded_packages.get(&requirement.name) {
            if !existing_version.is_compatible_with(&requirement.version) {
                return Err(RuntimeError::DependencyError {
                    reason: format!(
                        "Incompatible dependency '{}': package '{requester_name}' requires '{}', but loaded version is '{}'",
                        requirement.name, requirement.version, existing_version
                    ),
                });
            }
            if !requirement.sha256.is_empty()
                && let Some(existing_binary) = options.binaries.get(*existing_idx)
            {
                let actual_sha256 = existing_binary.compute_sha256();
                if requirement.sha256 != actual_sha256 {
                    if options.verify_checksums {
                        return Err(RuntimeError::DependencyError {
                            reason: format!(
                                "Dependency checksum mismatch for '{}@{}': expected {}, found {}",
                                requirement.name,
                                requirement.version,
                                requirement.sha256,
                                actual_sha256
                            ),
                        });
                    } else {
                        eprintln!(
                            "warning: checksum verification bypassed (--no-verify); loading unverified dependency '{}@{}'",
                            requirement.name, requirement.version
                        );
                    }
                }
            }
        } else if let Some(solver) = &options.solver {
            match solver.resolve_compatible(&requirement.name, requirement.version) {
                Ok(solved_package) => {
                    if !solved_package
                        .version
                        .is_compatible_with(&requirement.version)
                    {
                        return Err(RuntimeError::DependencyError {
                            reason: format!(
                                "Incompatible dependency '{}': solver returned version '{}' which does not satisfy required '{}' (requested by '{requester_name}')",
                                requirement.name, solved_package.version, requirement.version
                            ),
                        });
                    }

                    if !requirement.sha256.is_empty() {
                        let actual_sha256 = solved_package.program.compute_sha256();
                        if requirement.sha256 != actual_sha256 {
                            if options.verify_checksums {
                                return Err(RuntimeError::DependencyError {
                                    reason: format!(
                                        "Dependency checksum mismatch for '{}@{}': expected {}, found {}",
                                        requirement.name,
                                        requirement.version,
                                        requirement.sha256,
                                        actual_sha256
                                    ),
                                });
                            } else {
                                eprintln!(
                                    "warning: checksum verification bypassed (--no-verify); loading unverified dependency '{}@{}'",
                                    requirement.name, requirement.version
                                );
                            }
                        }
                    }

                    let new_index = options.binaries.len();
                    loaded_packages.insert(
                        requirement.name.clone(),
                        (solved_package.version, new_index),
                    );

                    for transitive in &solved_package.program.export_manifest.dependencies {
                        work_queue.push_back((transitive.clone(), requirement.name.clone()));
                    }

                    options.binaries.push(solved_package.program);
                    for secondary_program in solved_package.secondary_programs {
                        options.binaries.push(secondary_program);
                    }
                    for native_library_path in solved_package.native_libraries {
                        if !options.native_libraries.contains(&native_library_path) {
                            options.native_libraries.push(native_library_path);
                        }
                    }
                }
                Err(solver_error) => {
                    return Err(RuntimeError::DependencyError {
                        reason: format!(
                            "Missing required dependency '{}@{}' (requested by '{requester_name}'): {solver_error}",
                            requirement.name, requirement.version
                        ),
                    });
                }
            }
        } else {
            return Err(RuntimeError::DependencyError {
                reason: format!(
                    "Missing required dependency '{}@{}' (requested by '{requester_name}') and no module solver configured",
                    requirement.name, requirement.version
                ),
            });
        }
    }

    Ok(())
}

/// Computes static acyclicity per `vtable_id`.
///
/// A class is guaranteed acyclic if it has no strong reference fields or every strong reference
/// field strictly targets `String` (tag `13`) or statically acyclic concrete classes without
/// untyped/unknown edges.
fn compute_class_acyclicity(vtable_lookup: &[Option<CompiledClass>]) -> Vec<bool> {
    let class_count = vtable_lookup.len();
    let mut subtypes: Vec<Vec<usize>> = vec![Vec::new(); class_count];
    for (class_idx, class_opt) in vtable_lookup.iter().enumerate() {
        if let Some(class_entry) = class_opt
            && let Some(base_id) = class_entry.base_vtable_id
            && (base_id as usize) < class_count
        {
            subtypes[base_id as usize].push(class_idx);
        }
    }

    let mut edges: Vec<Option<Vec<usize>>> = vec![None; class_count];
    for (class_idx, class_opt) in vtable_lookup.iter().enumerate() {
        let Some(class_entry) = class_opt else {
            continue;
        };
        if class_entry.reference_field_offsets.is_empty() {
            edges[class_idx] = Some(Vec::new());
            continue;
        }
        let mut targets: Vec<usize> = Vec::new();
        let mut all_known = true;
        for &offset in &class_entry.reference_field_offsets {
            match class_entry.get_field_type(offset) {
                Some(13) => {}
                Some(tag_val) if (tag_val & 0x80) != 0 => {
                    let target_vid = (tag_val & 0x7F) as usize;
                    if target_vid < class_count && vtable_lookup[target_vid].is_some() {
                        let mut queue = vec![target_vid];
                        while let Some(subclass_id) = queue.pop() {
                            if !targets.contains(&subclass_id) {
                                targets.push(subclass_id);
                                queue.extend_from_slice(&subtypes[subclass_id]);
                            }
                        }
                    } else {
                        all_known = false;
                        break;
                    }
                }
                _ => {
                    all_known = false;
                    break;
                }
            }
        }
        if all_known {
            edges[class_idx] = Some(targets);
        }
    }

    fn check_acyclic(
        node_idx: usize,
        edges: &[Option<Vec<usize>>],
        memo: &mut [Option<bool>],
        visiting: &mut [bool],
    ) -> bool {
        if let Some(cached) = memo[node_idx] {
            return cached;
        }
        if visiting[node_idx] {
            return false;
        }
        let Some(ref target_list) = edges[node_idx] else {
            memo[node_idx] = Some(false);
            return false;
        };
        visiting[node_idx] = true;
        let mut is_clean = true;
        for &next_idx in target_list {
            if !check_acyclic(next_idx, edges, memo, visiting) {
                is_clean = false;
                break;
            }
        }
        visiting[node_idx] = false;
        memo[node_idx] = Some(is_clean);
        is_clean
    }

    let mut memo: Vec<Option<bool>> = vec![None; class_count];
    let mut visiting = vec![false; class_count];
    let mut acyclic = vec![false; class_count];
    for class_idx in 0..class_count {
        if vtable_lookup[class_idx].is_some() {
            acyclic[class_idx] = check_acyclic(class_idx, &edges, &mut memo, &mut visiting);
        }
    }
    acyclic
}
