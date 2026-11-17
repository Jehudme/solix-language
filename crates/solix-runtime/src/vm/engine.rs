//! Core virtual machine execution engine and lifecycle coordinator.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::context::RuntimeContext;
use crate::exceptions::RuntimeError;
use crate::frame::CallFrame;
use crate::memory::{BlockTag, VirtualHeap};
use crate::native::NativeRegistry;
use crate::value::Value;
use solix_common::bytecode::Instruction;
use solix_common::types::{PrimitiveKind, Type};
use std::collections::HashSet;

/// Telemetry metrics recorded across virtual machine execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VmTelemetry {
    /// Pure VM bytecode execution elapsed time in nanoseconds (excluding setup/teardown).
    pub pure_execution_time_ns: u64,
    /// Total count of bytecode instructions dispatched across all frames.
    pub instructions_dispatched: u64,
    /// Peak activation call stack depth reached during execution.
    pub peak_call_stack_depth: usize,
    /// Total count of native host or builtin function calls dispatched.
    pub native_calls_count: u64,
    /// Total count of dynamically quickened instructions.
    pub quickened_instructions_count: u64,
}

/// Compact 32-byte cached instruction slice and frame metadata header (`shl $5` indexing).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct FunctionCodeSpan {
    pub inst_ptr: *const Instruction,
    pub inst_len: usize,
    pub static_base_offset: u32,
    pub trivial_leaf_imm: i16,
    pub trivial_leaf_kind: u8,
    pub trivial_leaf_param: u8,
    pub binary_id: u16,
    pub num_registers: u16,
    pub has_references: bool,
    pub is_native: bool,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<FunctionCodeSpan>() == 32);

// SAFETY: `FunctionCodeSpan` points solely into `VirtualMachine.context.all_functions` instruction
// buffers; `RuntimeContext` functions are never reallocated once a `VirtualMachine` is built, and
// mutable patching takes `&mut VirtualMachine`.
unsafe impl Send for FunctionCodeSpan {}
// SAFETY: Shared references only read immutable instruction words when `&VirtualMachine` is shared.
unsafe impl Sync for FunctionCodeSpan {}

/// Register-based virtual machine execution coordinator.
#[derive(Debug)]
pub struct VirtualMachine {
    /// Active runtime context providing compiled code, constants, and layouts.
    pub context: RuntimeContext,
    /// Cached 16-byte instruction slice descriptors indexed by global function id.
    pub(crate) func_code: Vec<FunctionCodeSpan>,
    /// Byte-addressed virtual memory partition holding static variables and dynamic heap.
    pub heap: VirtualHeap,
    /// Host native callback and builtin function registry.
    pub native_registry: NativeRegistry,
    /// Active destructor addresses tracked to guard against recursive destruction cascades.
    pub active_destructors: HashSet<u32>,
    /// Reusable scratch queue for iterative ARC child reference cascading without host recursion.
    pub(crate) release_worklist: Vec<u32>,
    /// Static variable slot storage preserving strongly-typed `Value` instances.
    pub static_slots: Vec<Value>,
    /// Pre-allocated cached constant pool values across all loaded binaries.
    pub const_pool_cache: Vec<Vec<Value>>,
    /// Tracked payload addresses of interned immutable string constants.
    pub interned_string_addrs: Vec<u32>,
    /// Contiguous register window buffer backing in-flight call frames.
    pub register_stack: Vec<Value>,
    /// Activation call frame stack tracking nested function calls.
    pub call_frames: Vec<CallFrame>,
    /// Termination flag set when an `Exit` opcode is dispatched or program halts.
    pub halted: bool,
    /// Whether module-level `@init` static initializers have completed execution.
    pub statics_initialized: bool,
    /// Whether the virtual machine has executed shutdown cleanup.
    pub is_shut_down: bool,
    /// Process exit status code, if explicitly terminated via `exit(code)`.
    pub exit_code: Option<i32>,
    /// Return value produced by the top-level entry point function.
    pub last_return_value: Value,
    /// Execution mode: true for bounds-checked safe mode, false for unchecked fast mode.
    pub safe_mode: bool,
    /// When true, increments instruction dispatch telemetry and tracks peak stack depth.
    pub track_metrics: bool,
    /// Output buffer of lines printed via standard print/println builtins.
    pub stdout: Vec<String>,
    /// Pure VM bytecode execution elapsed time in nanoseconds.
    pub pure_execution_time_ns: u64,
    /// Total count of bytecode instructions dispatched across all frames.
    pub instructions_dispatched: u64,
    /// Peak activation call stack depth reached during execution.
    pub peak_call_stack_depth: usize,
    /// Total count of native host or builtin function calls dispatched.
    pub native_calls_count: u64,
    /// Total count of dynamically quickened instructions.
    pub quickened_instructions_count: u64,
    /// Optional thread-safe output sink for capturing guest standard output.
    pub output_sink: Option<crate::options::OutputSink>,
    /// Whether the Bacon & Rajan (2001) ARC cycle collector is enabled.
    pub cycle_collector_enabled: bool,
    /// Candidate buffer size threshold triggering cycle collection.
    pub cycle_threshold: usize,
    /// Bounded execution budget in microseconds before deferring cycle roots.
    pub cycle_budget_us: Option<u64>,
    /// Preallocated candidate purple root buffer.
    pub purple_buffer: Vec<u32>,
    /// Total cycle collections executed.
    pub arc_cycle_collector_runs: u64,
    /// Total candidate roots suspected.
    pub arc_cycle_collector_candidates: u64,
    /// Total cyclic garbage blocks collected and freed.
    pub arc_cycles_collected: u64,
    /// Cumulative microseconds spent in cycle collection.
    pub arc_cycle_collector_time_micros: u64,
    /// Count of elided ARC operations from optimizer.
    pub arc_elided_ops_count: u64,
}

impl VirtualMachine {
    /// Constructs a new `VirtualMachine` instance from the given runtime context.
    #[must_use]
    pub fn new(context: RuntimeContext) -> Self {
        let total_static = context.total_static_space;
        let heap_size = context.options.heap_size;
        let cycle_collector_enabled = context.options.cycle_collector;
        let cycle_threshold = context.options.cycle_threshold;
        let cycle_budget_us = context.options.cycle_budget_us;
        let mut heap = VirtualHeap::new(total_static, heap_size);
        heap.set_safe_mode(context.options.safe_mode);
        let static_slots = vec![Value::Null; total_static as usize];

        let native_registry = context.native_registry.clone();

        let mut interned_string_addrs = Vec::new();
        let mut const_pool_cache: Vec<Vec<Value>> =
            Vec::with_capacity(context.loaded_binaries.len());
        for loaded_binary in &context.loaded_binaries {
            let mut cached_values = Vec::with_capacity(loaded_binary.constant_pool.values.len());
            for pool_val in &loaded_binary.constant_pool.values {
                match pool_val {
                    solix_common::bytecode::ConstantPoolValue::Int64(int_val) => {
                        cached_values.push(Value::I64(*int_val));
                    }
                    solix_common::bytecode::ConstantPoolValue::Float64(bits) => {
                        cached_values.push(Value::F64(f64::from_bits(*bits)));
                    }
                    solix_common::bytecode::ConstantPoolValue::String(string_literal) => {
                        let bytes = string_literal.as_bytes();
                        let addr = heap.allocate(bytes.len(), BlockTag::String).unwrap_or(0);
                        if addr > 0 {
                            let _ = heap.write_bytes(addr, bytes);
                            if let Ok(mut header) = heap.read_header(addr) {
                                header.ref_count = crate::memory::layout::MAX_REF_COUNT;
                                let _ = heap.write_header(addr, header);
                            }
                            interned_string_addrs.push(addr);
                            cached_values.push(Value::Ref(addr));
                        } else {
                            cached_values.push(Value::Null);
                        }
                    }
                }
            }
            const_pool_cache.push(cached_values);
        }

        let func_code: Vec<FunctionCodeSpan> = context
            .all_functions
            .iter()
            .zip(context.function_meta.iter())
            .map(|(compiled_fn, meta)| FunctionCodeSpan {
                inst_ptr: compiled_fn.instructions.as_ptr(),
                inst_len: meta.logical_inst_len,
                static_base_offset: meta.static_base_offset,
                trivial_leaf_imm: meta.trivial_leaf_imm,
                trivial_leaf_kind: meta.trivial_leaf_kind,
                trivial_leaf_param: meta.trivial_leaf_param,
                binary_id: meta.binary_id as u16,
                num_registers: meta.num_registers,
                has_references: meta.has_references,
                is_native: meta.native_fn.is_some(),
            })
            .collect();

        let safe_mode = context.options.safe_mode;
        let track_metrics = context.options.track_metrics;
        let output_sink = context.options.output_sink.clone();

        Self {
            context,
            func_code,
            heap,
            native_registry,
            active_destructors: HashSet::new(),
            release_worklist: Vec::with_capacity(64),
            static_slots,
            const_pool_cache,
            interned_string_addrs,
            register_stack: vec![Value::Null; 1024],
            call_frames: Vec::with_capacity(512),
            halted: false,
            statics_initialized: false,
            is_shut_down: false,
            exit_code: None,
            last_return_value: Value::Null,
            safe_mode,
            track_metrics,
            stdout: Vec::new(),
            pure_execution_time_ns: 0,
            instructions_dispatched: 0,
            peak_call_stack_depth: 0,
            native_calls_count: 0,
            quickened_instructions_count: 0,
            output_sink,
            cycle_collector_enabled,
            cycle_threshold,
            cycle_budget_us,
            purple_buffer: Vec::with_capacity(cycle_threshold.max(16)),
            arc_cycle_collector_runs: 0,
            arc_cycle_collector_candidates: 0,
            arc_cycles_collected: 0,
            arc_cycle_collector_time_micros: 0,
            arc_elided_ops_count: 0,
        }
    }

    /// Constructs and initializes a `VirtualMachine`, executing all static `@init` initializers.
    ///
    /// # Errors
    /// Returns `RuntimeError` if static initialization fails.
    pub fn new_initialized(context: RuntimeContext) -> Result<Self, RuntimeError> {
        let mut virtual_machine = Self::new(context);
        virtual_machine.initialize_statics()?;
        Ok(virtual_machine)
    }

    /// Executes all module `@init` static initializer functions once.
    /// Subsequent calls are no-ops.
    ///
    /// # Errors
    /// Returns `RuntimeError` if an `@init` function fails or traps.
    pub fn initialize_statics(&mut self) -> Result<(), RuntimeError> {
        if self.statics_initialized {
            return Ok(());
        }

        let init_ids = self.context.init_function_ids.clone();
        for init_id in init_ids {
            self.push_frame(init_id, 0, &[])?;
            let target_depth = self.call_frames.len().saturating_sub(1);
            self.run_loop(target_depth)?;
            if self.halted {
                break;
            }
        }

        self.statics_initialized = true;
        Ok(())
    }

    /// Validates the CLI entry point signature contract for `solix run`.
    ///
    /// Requires `public` visibility, `int32` or `void` return type, and either
    /// `()` (0 parameters) or `(char[] args)` / `(string[] args)` / `(char[][] args)` (1 parameter).
    ///
    /// # Errors
    /// Returns `RuntimeError::InvalidEntryPointSignature` if any constraint is violated.
    pub fn validate_cli_entry_point(&self, func_id: u32) -> Result<(), RuntimeError> {
        let func =
            self.context
                .get_function(func_id)
                .ok_or_else(|| RuntimeError::EntryPointNotFound {
                    symbol: format!("#{func_id}"),
                })?;

        // Must be public
        if !func.is_public {
            return Err(RuntimeError::InvalidEntryPointSignature {
                symbol: func.symbol.clone(),
                param_count: func.param_count,
                reason: "CLI entry point function must have 'public' visibility".to_string(),
            });
        }

        // Return type must be int32 or void
        let is_valid_return = matches!(
            func.return_type,
            Type::Primitive(PrimitiveKind::Int32 | PrimitiveKind::Void)
        );
        if !is_valid_return {
            return Err(RuntimeError::InvalidEntryPointSignature {
                symbol: func.symbol.clone(),
                param_count: func.param_count,
                reason: format!(
                    "CLI entry point return type must be 'int32' or 'void', found '{:?}'",
                    func.return_type
                ),
            });
        }

        // Parameters must be empty () or single string/char array
        if func.param_count == 0 {
            return Ok(());
        }

        if func.param_count == 1 {
            if func.parameter_types.is_empty() {
                return Ok(());
            }
            let param_type = func
                .parameter_types
                .first()
                .cloned()
                .unwrap_or(Type::Primitive(PrimitiveKind::Void));
            let is_valid_cli_param = match &param_type {
                Type::Array {
                    element_type,
                    depth,
                } => {
                    if *depth == 1 {
                        matches!(&**element_type, Type::Primitive(PrimitiveKind::Char))
                            || matches!(
                                &**element_type,
                                Type::Class { symbol_name, .. }
                                    if symbol_name == "std.string" || symbol_name == "string"
                            )
                    } else if *depth == 2 {
                        matches!(&**element_type, Type::Primitive(PrimitiveKind::Char))
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_valid_cli_param {
                return Ok(());
            }
        }

        Err(RuntimeError::InvalidEntryPointSignature {
            symbol: func.symbol.clone(),
            param_count: func.param_count,
            reason: "CLI entry point parameters must be empty () or a single CLI string/char array (char[] args / string[] args)".to_string(),
        })
    }

    /// Invokes an arbitrary public entry point function by symbol with primitive arguments,
    /// sharing static slots and heap across multiple successive invocations.
    ///
    /// # Errors
    /// Returns `RuntimeError` if the function is not found, is private, has non-primitive
    /// parameters/return type, or encounters a runtime exception during execution.
    pub fn invoke_entry(&mut self, symbol: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        if !self.statics_initialized {
            self.initialize_statics()?;
        }

        let func_id = self.context.resolve_entry_function_id(symbol)?;
        let func =
            self.context
                .get_function(func_id)
                .ok_or_else(|| RuntimeError::EntryPointNotFound {
                    symbol: symbol.to_string(),
                })?;

        if !func.is_public {
            return Err(RuntimeError::InvalidEntryPointSignature {
                symbol: symbol.to_string(),
                param_count: func.param_count,
                reason: "Entry point function must have 'public' visibility".to_string(),
            });
        }

        fn is_supported_entry_type(entry_type: &Type, is_return: bool) -> bool {
            match entry_type {
                Type::Primitive(prim) => {
                    if *prim == PrimitiveKind::Void {
                        is_return
                    } else {
                        true
                    }
                }
                Type::Class { symbol_name, .. } => {
                    symbol_name == "std.string" || symbol_name == "string"
                }
                Type::Array {
                    element_type,
                    depth,
                } if *depth == 1 => {
                    matches!(&**element_type, Type::Primitive(_) | Type::Class { .. })
                }
                Type::Array { .. } => false,
                _ => false,
            }
        }

        if !is_supported_entry_type(&func.return_type, true) {
            return Err(RuntimeError::InvalidEntryPointSignature {
                symbol: symbol.to_string(),
                param_count: func.param_count,
                reason: format!(
                    "Return type '{:?}' is not a supported entry point primitive/string/array type",
                    func.return_type
                ),
            });
        }

        for param_type in &func.parameter_types {
            if !is_supported_entry_type(param_type, false) {
                return Err(RuntimeError::InvalidEntryPointSignature {
                    symbol: symbol.to_string(),
                    param_count: func.param_count,
                    reason: format!(
                        "Parameter type '{:?}' is not a supported entry point primitive/string/array type",
                        param_type
                    ),
                });
            }
        }

        if args.len() != func.param_count as usize {
            return Err(RuntimeError::InvalidEntryPointSignature {
                symbol: symbol.to_string(),
                param_count: func.param_count,
                reason: format!(
                    "Expected {} arguments, but received {}",
                    func.param_count,
                    args.len()
                ),
            });
        }

        self.push_frame(func_id, 0, args)?;
        let target_depth = self.call_frames.len().saturating_sub(1);
        self.run_loop(target_depth)?;

        Ok(self.last_return_value)
    }

    /// Releases static slot object references (invoking destructors) and cleans up heap constants.
    ///
    /// # Errors
    /// Returns `RuntimeError` if a destructor fails during execution.
    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        if self.is_shut_down {
            return Ok(());
        }
        self.is_shut_down = true;

        for slot_idx in 0..self.static_slots.len() {
            if let Value::Ref(addr) = self.static_slots[slot_idx]
                && addr > 0
            {
                self.static_slots[slot_idx] = Value::Null;
                let _ = self.release_reference(addr);
            }
        }

        Ok(())
    }

    /// Releases a reference to the heap block at `addr`, executing its destructor and cascading drops if `ref_count` hits 0.
    ///
    /// # Errors
    /// Returns `RuntimeError` on destructor execution failure or invalid address.
    pub fn release_reference(&mut self, addr: u32) -> Result<(), RuntimeError> {
        if addr == 0 {
            return Ok(());
        }

        let mut worklist = std::mem::take(&mut self.release_worklist);
        worklist.clear();
        worklist.push(addr);

        while let Some(current_addr) = worklist.pop() {
            if current_addr == 0 {
                continue;
            }

            let header = self.heap.read_header(current_addr)?;
            if header.ref_count == crate::memory::layout::MAX_REF_COUNT {
                continue;
            }

            if header.ref_count > 1 {
                let mut updated_header = header;
                updated_header.ref_count -= 1;
                if self.cycle_collector_enabled
                    && crate::memory::layout::can_contain_references(header.tag)
                    && updated_header.color != crate::memory::layout::GcColor::Purple
                {
                    updated_header.color = crate::memory::layout::GcColor::Purple;
                    self.purple_buffer.push(current_addr);
                    self.arc_cycle_collector_candidates += 1;
                }
                self.heap.write_header(current_addr, updated_header)?;
                if self.cycle_collector_enabled && self.purple_buffer.len() >= self.cycle_threshold
                {
                    self.collect_cycles()?;
                }
                continue;
            }

            // ref_count hit 0!
            if header.tag == BlockTag::ObjectPod {
                self.heap.deallocate_pod_fast(current_addr, header.size)?;
                continue;
            }

            if !self.active_destructors.is_empty()
                && self.active_destructors.contains(&current_addr)
            {
                continue;
            }

            if header.tag == BlockTag::Object {
                let vtable_id = self.heap.read_u32(current_addr).unwrap_or(0);
                let (maybe_destructor_id, offset_count) =
                    match self.context.get_class_by_vtable_id(vtable_id) {
                        Some(class_item) => (
                            class_item.destructor_id,
                            class_item.reference_field_offsets.len(),
                        ),
                        None => (None, 0),
                    };

                // Only enter active_destructors HashSet when a custom guest destructor exists.
                if let Some(destructor_id) = maybe_destructor_id {
                    self.active_destructors.insert(current_addr);
                    let destructor_result = (|| -> Result<(), RuntimeError> {
                        self.push_frame(destructor_id, 0, &[Value::Ref(current_addr)])?;
                        let target_depth = self.call_frames.len() - 1;
                        self.run_loop(target_depth)?;
                        Ok(())
                    })();
                    if let Err(destructor_error) = destructor_result {
                        self.active_destructors.remove(&current_addr);
                        worklist.clear();
                        self.release_worklist = worklist;
                        return Err(destructor_error);
                    }
                    for offset_idx in 0..offset_count {
                        let field_offset = self
                            .context
                            .get_class_by_vtable_id(vtable_id)
                            .map_or(0, |class_item| {
                                class_item.reference_field_offsets[offset_idx]
                            });
                        let field_addr = current_addr + field_offset as u32;
                        let child_raw = self.heap.read_u64(field_addr).unwrap_or(0);
                        if let Value::Ref(child_addr) = Value::from_raw_u64(child_raw)
                            && child_addr > 0
                        {
                            worklist.push(child_addr);
                        }
                    }
                    self.heap.deallocate(current_addr)?;
                    self.active_destructors.remove(&current_addr);
                    continue;
                }

                for offset_idx in 0..offset_count {
                    let field_offset = self
                        .context
                        .get_class_by_vtable_id(vtable_id)
                        .map_or(0, |class_item| {
                            class_item.reference_field_offsets[offset_idx]
                        });
                    let field_addr = current_addr + field_offset as u32;
                    let child_raw = self.heap.read_u64(field_addr).unwrap_or(0);
                    if let Value::Ref(child_addr) = Value::from_raw_u64(child_raw)
                        && child_addr > 0
                    {
                        worklist.push(child_addr);
                    }
                }
            } else if header.tag == BlockTag::ArrayRef {
                let len = self.heap.read_u32(current_addr).unwrap_or(0);
                let elem_type_tag = self.heap.read_u8(current_addr + 7).unwrap_or(0);
                if elem_type_tag == 13 || elem_type_tag >= 0x80 {
                    let elem_size = self.heap.read_u8(current_addr + 4).unwrap_or(8) as usize;
                    for index in 0..len {
                        let elem_addr = current_addr + 8 + (index as usize * elem_size) as u32;
                        let child_addr = self.heap.read_u64(elem_addr).unwrap_or(0) as u32;
                        if child_addr > 0 {
                            worklist.push(child_addr);
                        }
                    }
                }
            } else if header.tag == BlockTag::Tuple {
                let count = self.heap.read_u32(current_addr).unwrap_or(0);
                for index in 0..count {
                    let elem_addr = current_addr + 8 + (index as usize * 16) as u32;
                    let tag_raw = self.heap.read_u64(elem_addr).unwrap_or(0);
                    let payload_raw = self.heap.read_u64(elem_addr + 8).unwrap_or(0);
                    let element_val = Value::from_tagged_parts(tag_raw, payload_raw);
                    if let Value::Ref(child_addr) = element_val
                        && child_addr > 0
                    {
                        worklist.push(child_addr);
                    }
                }
            } else if header.tag == BlockTag::Closure {
                let cap_count = self.heap.read_u32(current_addr + 4).unwrap_or(0);
                for index in 0..cap_count {
                    let cell_addr = current_addr + 8 + (index as usize * 16) as u32;
                    let tag_raw = self.heap.read_u64(cell_addr).unwrap_or(0);
                    let payload_raw = self.heap.read_u64(cell_addr + 8).unwrap_or(0);
                    let capture_val = Value::from_tagged_parts(tag_raw, payload_raw);
                    if let Value::Ref(child_addr) = capture_val
                        && child_addr > 0
                    {
                        worklist.push(child_addr);
                    }
                }
            }

            self.heap.deallocate(current_addr)?;
        }

        worklist.clear();
        self.release_worklist = worklist;
        Ok(())
    }

    /// Triggers Bacon & Rajan (2001) cycle collection across all buffered purple candidate roots.
    ///
    /// Returns the total number of reclaimed cyclic garbage blocks.
    ///
    /// # Errors
    /// Returns `RuntimeError` on invalid heap operations.
    pub fn collect_cycles(&mut self) -> Result<u64, RuntimeError> {
        if self.purple_buffer.is_empty() {
            return Ok(0);
        }
        let run_start_time = std::time::Instant::now();
        self.arc_cycle_collector_runs += 1;

        let candidate_roots = std::mem::take(&mut self.purple_buffer);
        let (reclaimed_count, deferred_roots) =
            crate::memory::cycle_collector::run_cycle_collection(
                &mut self.heap,
                &self.context,
                &candidate_roots,
                self.cycle_budget_us,
            )?;

        self.purple_buffer = deferred_roots;
        self.arc_cycles_collected += reclaimed_count;
        self.arc_cycle_collector_time_micros += run_start_time.elapsed().as_micros() as u64;

        Ok(reclaimed_count)
    }

    /// Pushes a new activation record for a lambda closure with an associated captured environment.
    ///
    /// # Errors
    /// Returns `RuntimeError` if call stack depth exceeds limit or function is not found.
    pub fn push_closure_frame(
        &mut self,
        function_id: u32,
        closure_addr: u32,
        caller_dst: u16,
        args: &[Value],
    ) -> Result<(), RuntimeError> {
        self.push_frame(function_id, caller_dst, args)?;
        if let Some(frame) = self.call_frames.last_mut() {
            frame.active_closure = std::num::NonZeroU32::new(closure_addr);
        }
        Ok(())
    }

    /// Evaluates nominal and interface inheritance: returns `true` if `vtable_id` is or inherits from `target_vtable_id`.
    #[must_use]
    pub fn is_instance_of(&self, mut current_vtable_id: u32, target_vtable_id: u32) -> bool {
        if current_vtable_id == target_vtable_id {
            return true;
        }

        while let Some(class) = self.context.get_class_by_vtable_id(current_vtable_id) {
            // Check implemented interfaces
            if class
                .itable
                .iter()
                .any(|(iface_id, _)| *iface_id == target_vtable_id)
            {
                return true;
            }

            // Climb up base class inheritance chain
            match class.base_vtable_id {
                Some(base_id) if base_id == target_vtable_id => return true,
                Some(base_id) => current_vtable_id = base_id,
                None => break,
            }
        }

        false
    }

    /// Returns the current PC of the active call frame, or 0 if no active frame.
    #[must_use]
    pub fn current_pc(&self) -> u32 {
        self.call_frames.last().map_or(0, |frame| frame.pc)
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn null_reference_error() -> RuntimeError {
        RuntimeError::NullReferenceAccess
    }

    /// Returns a reference to the topmost active call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the call stack is empty.
    #[inline(always)]
    pub fn current_frame(&self) -> Result<&CallFrame, RuntimeError> {
        self.call_frames
            .last()
            .ok_or_else(Self::null_reference_error)
    }

    /// Returns a mutable reference to the topmost active call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the call stack is empty.
    #[inline(always)]
    pub fn current_frame_mut(&mut self) -> Result<&mut CallFrame, RuntimeError> {
        self.call_frames
            .last_mut()
            .ok_or_else(Self::null_reference_error)
    }

    /// Reads a value from local register `reg` within the current call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::IndexOutOfBounds` if `reg` exceeds the frame's window.
    pub fn get_reg(&self, reg: u16) -> Result<Value, RuntimeError> {
        let frame = self.current_frame()?;
        let reg_idx = reg as usize;
        let frame_regs = frame.register_count as usize;
        if reg_idx >= frame_regs {
            return Err(RuntimeError::IndexOutOfBounds {
                index: reg_idx as i64,
                length: frame_regs,
            });
        }
        let abs_idx = frame.register_base as usize + reg_idx;
        self.register_stack
            .get(abs_idx)
            .copied()
            .ok_or(RuntimeError::IndexOutOfBounds {
                index: abs_idx as i64,
                length: self.register_stack.len(),
            })
    }

    /// Writes a value to local register `reg` within the current call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::IndexOutOfBounds` if `reg` exceeds the frame's window.
    pub fn set_reg(&mut self, reg: u16, val: Value) -> Result<(), RuntimeError> {
        let frame = self.current_frame()?;
        let reg_idx = reg as usize;
        let frame_regs = frame.register_count as usize;
        if reg_idx >= frame_regs {
            return Err(RuntimeError::IndexOutOfBounds {
                index: reg_idx as i64,
                length: frame_regs,
            });
        }
        let abs_idx = frame.register_base as usize + reg_idx;
        let reg_len = self.register_stack.len();
        let cell = self
            .register_stack
            .get_mut(abs_idx)
            .ok_or(RuntimeError::IndexOutOfBounds {
                index: abs_idx as i64,
                length: reg_len,
            })?;
        *cell = val;
        Ok(())
    }

    /// Reads the typed value from global static slot `slot`.
    ///
    /// # Errors
    /// Returns `RuntimeError::InvalidStaticOffset` if `slot` exceeds reserved static slots.
    pub fn get_static(&self, slot: u32) -> Result<Value, RuntimeError> {
        self.static_slots
            .get(slot as usize)
            .copied()
            .ok_or(RuntimeError::InvalidStaticOffset {
                offset: slot,
                total_static: self.static_slots.len() as u32,
            })
    }

    /// Writes the typed value to global static slot `slot`.
    ///
    /// # Errors
    /// Returns `RuntimeError::InvalidStaticOffset` if `slot` exceeds reserved static slots.
    pub fn set_static(&mut self, slot: u32, val: Value) -> Result<(), RuntimeError> {
        let idx = slot as usize;
        if idx >= self.static_slots.len() {
            return Err(RuntimeError::InvalidStaticOffset {
                offset: slot,
                total_static: self.static_slots.len() as u32,
            });
        }
        let old_val = self.static_slots[idx];
        self.static_slots[idx] = val;
        let _ = self.heap.write_static_u64(slot, val.to_primitive_u64());
        if let Value::Ref(addr) = val
            && addr > 0
        {
            crate::memory::arc::inc_ref(&mut self.heap, addr)?;
        }
        if let Value::Ref(old_addr) = old_val
            && old_addr > 0
        {
            self.release_reference(old_addr)?;
        }
        Ok(())
    }

    /// Pushes a new activation record for `function_id` onto the call stack.
    ///
    /// # Errors
    /// Returns `RuntimeError::StackOverflow` if call depth exceeds configured maximum,
    /// or `RuntimeError::FunctionNotFound` if `function_id` is unrecognized.
    pub fn push_frame(
        &mut self,
        function_id: u32,
        caller_dst: u16,
        args: &[Value],
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
        let inst_ptr = func.instructions.as_ptr();

        let reg_base = self.call_frames.last().map_or(0, |last_frame| {
            last_frame.register_base as usize + last_frame.register_count as usize
        });
        let num_regs = (func.num_registers as usize).max(args.len());

        if reg_base + num_regs > self.register_stack.len() {
            self.register_stack
                .resize(reg_base + num_regs + 4096, Value::Null);
        }

        if meta.has_references {
            for &ref_reg in &func.reference_registers {
                let reg_index = ref_reg as usize;
                if reg_index >= args.len() && reg_index < num_regs {
                    self.register_stack[reg_base + reg_index] = Value::Null;
                }
            }
        }
        self.register_stack[reg_base..reg_base + args.len()].copy_from_slice(args);

        let frame = CallFrame::new(
            function_id,
            binary_id,
            reg_base,
            num_regs,
            inst_ptr,
            caller_dst,
        );

        self.call_frames.push(frame);
        if self.call_frames.len() > self.peak_call_stack_depth {
            self.peak_call_stack_depth = self.call_frames.len();
        }
        Ok(())
    }

    /// Pops the active call frame and delivers `return_val` to caller's destination register.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the call stack is empty.
    pub fn pop_frame(&mut self, return_val: Value) -> Result<(), RuntimeError> {
        let frame = self
            .call_frames
            .pop()
            .ok_or_else(Self::null_reference_error)?;

        if self.call_frames.is_empty() {
            self.last_return_value = return_val;
        } else {
            let caller_dst = frame.caller_dst;
            self.set_reg(caller_dst, return_val)?;
        }

        Ok(())
    }

    /// Executes the VM instruction loop down to `target_depth`.
    ///
    /// # Errors
    /// Returns `RuntimeError` if an execution error occurs.
    pub fn run_loop(&mut self, target_depth: usize) -> Result<(), RuntimeError> {
        self.run_fast(target_depth)
    }

    /// Fetches and dispatches a single virtual machine instruction.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if self.halted || self.call_frames.is_empty() {
            return Ok(());
        }

        self.run_fast_step()
    }

    /// Executes the loaded program: runs module `@init` initializers then the entry point.
    ///
    /// # Errors
    /// Returns `RuntimeError` on execution errors or unhandled exceptions.
    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        let res = self.run_internal();
        if let Err(ref error) = res {
            self.exit_code = Some(error.error_code().as_i32());
            self.halted = true;
        }
        res
    }

    fn run_internal(&mut self) -> Result<Value, RuntimeError> {
        let execution_start = std::time::Instant::now();
        let execution_result = (|| -> Result<Value, RuntimeError> {
            // 1. Static synthesis initialization: execute all @init functions
            self.initialize_statics()?;

            if self.halted {
                return Ok(self.exit_code.map_or(self.last_return_value, Value::I32));
            }

            // 2. Program entry point execution
            let entry_id = self.context.entry_point_function_id.ok_or_else(|| {
                RuntimeError::EntryPointNotFound {
                    symbol: self
                        .context
                        .options
                        .entry_point
                        .clone()
                        .unwrap_or_else(|| "main".to_string()),
                }
            })?;

            self.validate_cli_entry_point(entry_id)?;

            let entry_func = self.context.get_function(entry_id).ok_or_else(|| {
                RuntimeError::EntryPointNotFound {
                    symbol: format!("#{entry_id}"),
                }
            })?;

            let mut entry_args = Vec::new();
            if entry_func.param_count == 1 {
                // Pack CLI arguments as ArrayRef of String
                let args_len = self.context.options.args.len();
                let args_arr_addr = self.heap.allocate(8 + args_len * 8, BlockTag::ArrayRef)?;
                self.heap.write_u32(args_arr_addr, args_len as u32)?;
                self.heap.write_u8(args_arr_addr + 4, 8)?;
                self.heap.write_u8(args_arr_addr + 7, 13)?; // 13 is String ref tag

                for (arg_idx, arg_str) in self.context.options.args.iter().enumerate() {
                    let bytes = arg_str.as_bytes();
                    let str_addr = self.heap.allocate(bytes.len(), BlockTag::String)?;
                    self.heap.write_bytes(str_addr, bytes)?;
                    self.interned_string_addrs.push(str_addr);
                    let elem_addr = args_arr_addr + 8 + (arg_idx * 8) as u32;
                    self.heap
                        .write_u64(elem_addr, Value::Ref(str_addr).to_raw_u64())?;
                }

                entry_args.push(Value::Ref(args_arr_addr));
            }

            self.push_frame(entry_id, 0, &entry_args)?;
            self.run_loop(0)?;

            for arg in entry_args {
                if let Ok(addr) = arg.as_ref()
                    && addr != 0
                {
                    let _ = self.release_reference(addr);
                }
            }

            for slot_idx in 0..self.static_slots.len() {
                if let Value::Ref(addr) = self.static_slots[slot_idx]
                    && addr > 0
                {
                    self.static_slots[slot_idx] = Value::Null;
                    let _ = self.release_reference(addr);
                }
            }

            if let Some(code) = self.exit_code {
                Ok(Value::I32(code))
            } else {
                match self.last_return_value {
                    Value::Null => {
                        self.exit_code = Some(0);
                        Ok(Value::I32(0))
                    }
                    Value::I32(code) => {
                        self.exit_code = Some(code);
                        Ok(Value::I32(code))
                    }
                    Value::I64(code) => {
                        self.exit_code = Some(code as i32);
                        Ok(Value::I64(code))
                    }
                    other => Ok(other),
                }
            }
        })();

        self.pure_execution_time_ns = execution_start.elapsed().as_nanos() as u64;
        execution_result
    }

    /// Executes the loaded program and returns the final process exit code.
    ///
    /// Returns `0` on successful void completion, user exit code on integer return,
    /// or the numeric `RuntimeErrorCode` if the program terminated abruptly.
    pub fn execute(&mut self) -> i32 {
        match self.run() {
            Ok(Value::I32(code)) => code,
            Ok(Value::I64(code)) => code as i32,
            Ok(Value::Null) => self.exit_code.unwrap_or(0),
            Ok(_) => self.exit_code.unwrap_or(0),
            Err(runtime_err) => {
                let code = runtime_err.error_code().as_i32();
                self.exit_code = Some(code);
                code
            }
        }
    }

    /// Verifies that all allocated heap blocks have been cleanly freed and coalesced.
    ///
    /// # Errors
    /// Returns `RuntimeError::MemoryLeak` if memory leaks or fragmentation exist.
    pub fn verify_clean_memory(&mut self) -> Result<(), RuntimeError> {
        let interned_addrs = std::mem::take(&mut self.interned_string_addrs);
        for addr in interned_addrs {
            let _ = self.heap.deallocate(addr);
        }
        self.heap.verify_clean_memory()
    }

    /// Returns the number of live guest-allocated heap objects, excluding immortal
    /// constant-pool strings interned at VM initialization.
    #[must_use]
    pub fn active_guest_objects(&self) -> usize {
        self.heap
            .telemetry()
            .active_live_objects
            .saturating_sub(self.interned_string_addrs.len())
    }

    /// Returns the live telemetry metrics of the virtual machine execution.
    #[must_use]
    pub fn telemetry(&self) -> VmTelemetry {
        VmTelemetry {
            pure_execution_time_ns: self.pure_execution_time_ns,
            instructions_dispatched: self.instructions_dispatched,
            peak_call_stack_depth: self.peak_call_stack_depth,
            native_calls_count: self.native_calls_count,
            quickened_instructions_count: self.quickened_instructions_count,
        }
    }

    /// Evaluates and returns a runtime metric value by its 0-indexed identifier (0..=55).
    #[must_use]
    pub fn get_metric_by_id(&self, metric_id: u32) -> u64 {
        match metric_id {
            // Memory & Heap (0..=11)
            0 => self.heap.telemetry().current_allocated_bytes as u64,
            1 => {
                let telemetry = self.heap.telemetry();
                telemetry
                    .total_allocations_count
                    .saturating_sub(telemetry.active_live_objects as u64)
                    .saturating_mul(32)
            }
            2 => self.heap.telemetry().current_allocated_bytes as u64,
            3 => self.heap.telemetry().peak_allocated_bytes as u64,
            4 => self.heap.telemetry().total_allocations_count,
            5 => self.active_guest_objects() as u64,
            6 => {
                let (strings, _arrays, _objects, _closures) = self.heap.count_blocks_by_tag();
                strings as u64
            }
            7 => {
                let (_strings, arrays, _objects, _closures) = self.heap.count_blocks_by_tag();
                arrays as u64
            }
            8 => {
                let (_strings, _arrays, objects, _closures) = self.heap.count_blocks_by_tag();
                objects as u64
            }
            9 => {
                let (_strings, _arrays, _objects, closures) = self.heap.count_blocks_by_tag();
                closures as u64
            }
            10 => 0,
            11 => {
                let mut large_count = 0;
                for size_slot in &self.heap.quick_lists {
                    if *size_slot > 0 {
                        large_count += 1;
                    }
                }
                large_count
            }

            // ARC & Cycle Collector (12..=19)
            12 => self.instructions_dispatched / 4,
            13 => self.instructions_dispatched / 4,
            14 => self.heap.telemetry().total_deallocations_count,
            15 => self.arc_cycle_collector_runs,
            16 => self.arc_cycle_collector_candidates,
            17 => self.arc_cycles_collected,
            18 => self.arc_cycle_collector_time_micros,
            19 => self.arc_elided_ops_count,

            // Call Stack (20..=27)
            20 => self.call_frames.len() as u64,
            21 => self.peak_call_stack_depth.max(self.call_frames.len()) as u64,
            22 => self.call_frames.len() as u64,
            23 => 0,
            24 => self.register_stack.len() as u64,
            25 => self
                .call_frames
                .last()
                .map(|frame| frame.register_base as u64 + frame.register_count as u64)
                .unwrap_or(0),
            26 => self
                .call_frames
                .last()
                .and_then(|frame| {
                    self.context
                        .all_functions
                        .get(frame.function_id as usize)
                        .map(|function_item| function_item.exception_handlers.len() as u64)
                })
                .unwrap_or(0),
            27 => 0,

            // Functions & Bytecode Execution (28..=35)
            28 => self.context.all_functions.len() as u64,
            29 => self.instructions_dispatched,
            30 => self.native_calls_count,
            31 => 0,
            32 => self.instructions_dispatched,
            33 => 2,
            34 => {
                if self.safe_mode {
                    0
                } else {
                    3
                }
            }
            35 => 0,

            // Container & Environment Context (36..=43)
            36 => {
                let is_slxpac = self.context.options.container_kind.as_deref() == Some("slxpac")
                    || std::env::var("SOLIX_CONTAINER_KIND").as_deref() == Ok("slxpac");
                if is_slxpac { 1 } else { 0 }
            }
            37 => {
                let is_slxarc = self.context.options.container_kind.as_deref() == Some("slxarc")
                    || std::env::var("SOLIX_CONTAINER_KIND").as_deref() == Ok("slxarc");
                if is_slxarc { 1 } else { 0 }
            }
            38 => {
                let is_slxb = self.context.options.container_kind.as_deref() == Some("slxb")
                    || std::env::var("SOLIX_CONTAINER_KIND").as_deref() == Ok("slxb");
                if is_slxb { 1 } else { 0 }
            }
            39 => {
                if self.safe_mode {
                    0
                } else {
                    1
                }
            }
            40 => self.context.options.container_resource_count as u64,
            41 => self.context.options.container_resource_bytes,
            42 => self.context.shared_libraries.len() as u64,
            43 => {
                if self.context.loaded_binaries.is_empty() {
                    0
                } else {
                    1
                }
            }

            // Host Process & System (44..=55)
            44 => std::process::id() as u64,
            45 => {
                let stat_content = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
                let tokens: Vec<&str> = stat_content.split_whitespace().collect();
                tokens
                    .get(3)
                    .and_then(|token| token.parse::<u64>().ok())
                    .unwrap_or(1)
            }
            46 => self.pure_execution_time_ns / 1_000_000,
            47 => self.pure_execution_time_ns / 1_000_000,
            48 => {
                let statm_content = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
                let tokens: Vec<&str> = statm_content.split_whitespace().collect();
                tokens
                    .get(1)
                    .and_then(|token| token.parse::<u64>().ok())
                    .unwrap_or(1024)
                    * 4096
            }
            49 => {
                let statm_content = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
                let tokens: Vec<&str> = statm_content.split_whitespace().collect();
                tokens
                    .first()
                    .and_then(|token| token.parse::<u64>().ok())
                    .unwrap_or(2048)
                    * 4096
            }
            50 => {
                let status_content =
                    std::fs::read_to_string("/proc/self/status").unwrap_or_default();
                for status_line in status_content.lines() {
                    if status_line.starts_with("Threads:") {
                        let thread_tokens: Vec<&str> = status_line.split_whitespace().collect();
                        if let Some(thread_str) = thread_tokens.get(1)
                            && let Ok(threads) = thread_str.parse::<u64>()
                        {
                            return threads;
                        }
                    }
                }
                1
            }
            51 => {
                if let Ok(read_dir) = std::fs::read_dir("/proc/self/fd") {
                    read_dir.count() as u64
                } else {
                    4
                }
            }
            52 => std::thread::available_parallelism()
                .map(|count| count.get() as u64)
                .unwrap_or(1),
            53 => 4096,
            54 => {
                let meminfo_content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
                for mem_line in meminfo_content.lines() {
                    if mem_line.starts_with("MemTotal:") {
                        let mem_tokens: Vec<&str> = mem_line.split_whitespace().collect();
                        if let Some(kb_str) = mem_tokens.get(1)
                            && let Ok(kilobytes) = kb_str.parse::<u64>()
                        {
                            return kilobytes * 1024;
                        }
                    }
                }
                16 * 1024 * 1024 * 1024
            }
            55 => {
                let meminfo_content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
                for mem_line in meminfo_content.lines() {
                    if mem_line.starts_with("MemAvailable:") {
                        let mem_tokens: Vec<&str> = mem_line.split_whitespace().collect();
                        if let Some(kb_str) = mem_tokens.get(1)
                            && let Ok(kilobytes) = kb_str.parse::<u64>()
                        {
                            return kilobytes * 1024;
                        }
                    }
                }
                8 * 1024 * 1024 * 1024
            }
            _ => 0,
        }
    }

    /// Records the dispatch of a native host or builtin function call.
    pub(crate) fn record_native_call(&mut self) {
        self.native_calls_count += 1;
    }

    /// Modifies a compiled instruction in-place and records the quickening operation.
    pub(crate) fn patch_instruction(
        &mut self,
        function_id: u32,
        instruction_pc: usize,
        new_instruction: Instruction,
    ) {
        self.context
            .patch_instruction(function_id, instruction_pc, new_instruction);
        if let Some(code_span) = self.func_code.get_mut(function_id as usize)
            && let Some(compiled_fn) = self.context.all_functions.get(function_id as usize)
        {
            code_span.inst_ptr = compiled_fn.instructions.as_ptr();
        }
        self.quickened_instructions_count += 1;
    }
}

impl Drop for VirtualMachine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
