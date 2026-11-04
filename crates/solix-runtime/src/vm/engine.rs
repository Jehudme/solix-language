//! Core virtual machine execution engine and lifecycle coordinator.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::context::RuntimeContext;
use crate::exceptions::RuntimeError;
use crate::frame::CallFrame;
use crate::memory::{BlockTag, VirtualHeap};
use crate::native::NativeRegistry;
use crate::value::Value;
use solix_common::bytecode::Instruction;
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

/// Register-based virtual machine execution coordinator.
#[derive(Debug)]
pub struct VirtualMachine {
    /// Active runtime context providing compiled code, constants, and layouts.
    pub context: RuntimeContext,
    /// Byte-addressed virtual memory partition holding static variables and dynamic heap.
    pub heap: VirtualHeap,
    /// Host native callback and builtin function registry.
    pub native_registry: NativeRegistry,
    /// Active destructor addresses tracked to guard against recursive destruction cascades.
    pub active_destructors: HashSet<u32>,
    /// Static variable slot storage preserving strongly-typed `Value` instances.
    pub static_slots: Vec<Value>,
    /// Pre-allocated cached constant pool values across all loaded binaries.
    pub const_pool_cache: Vec<Vec<Value>>,
    /// Tracked payload addresses of interned immutable string constants.
    pub interned_string_addrs: HashSet<u32>,
    /// Contiguous register window buffer backing in-flight call frames.
    pub register_stack: Vec<Value>,
    /// Activation call frame stack tracking nested function calls.
    pub call_frames: Vec<CallFrame>,
    /// Termination flag set when an `Exit` opcode is dispatched or program halts.
    pub halted: bool,
    /// Process exit status code, if explicitly terminated via `exit(code)`.
    pub exit_code: Option<i32>,
    /// Return value produced by the top-level entry point function.
    pub last_return_value: Value,
    /// Execution mode: true for bounds-checked safe mode, false for unchecked fast mode.
    pub safe_mode: bool,
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
}

impl VirtualMachine {
    /// Constructs a new `VirtualMachine` instance from the given runtime context.
    #[must_use]
    pub fn new(context: RuntimeContext) -> Self {
        let total_static = context.total_static_space;
        let heap_size = context.options.heap_size;
        let mut heap = VirtualHeap::new(total_static, heap_size);
        let static_slots = vec![Value::Null; total_static as usize];

        let mut native_registry = NativeRegistry::with_builtins();
        for (name, func) in &context.options.native_functions {
            native_registry.register(name, *func);
        }

        let mut interned_string_addrs = HashSet::new();
        let mut const_pool_cache: Vec<Vec<Value>> = Vec::with_capacity(context.loaded_binaries.len());
        for loaded_binary in &context.loaded_binaries {
            let mut cached_values =
                Vec::with_capacity(loaded_binary.constant_pool.values.len());
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
                            interned_string_addrs.insert(addr);
                            cached_values.push(Value::Ref(addr));
                        } else {
                            cached_values.push(Value::Null);
                        }
                    }
                }
            }
            const_pool_cache.push(cached_values);
        }

        let safe_mode = context.options.safe_mode;

        Self {
            context,
            heap,
            native_registry,
            active_destructors: HashSet::new(),
            static_slots,
            const_pool_cache,
            interned_string_addrs,
            register_stack: Vec::with_capacity(1024),
            call_frames: Vec::with_capacity(64),
            halted: false,
            exit_code: None,
            last_return_value: Value::Null,
            safe_mode,
            stdout: Vec::new(),
            pure_execution_time_ns: 0,
            instructions_dispatched: 0,
            peak_call_stack_depth: 0,
            native_calls_count: 0,
            quickened_instructions_count: 0,
        }
    }

    /// Releases a reference to the heap block at `addr`, executing its destructor and cascading drops if `ref_count` hits 0.
    ///
    /// # Errors
    /// Returns `RuntimeError` on destructor execution failure or invalid address.
    pub fn release_reference(&mut self, addr: u32) -> Result<(), RuntimeError> {
        if addr == 0 || self.interned_string_addrs.contains(&addr) {
            return Ok(());
        }

        let rc = crate::memory::arc::dec_ref(&mut self.heap, addr)?;
        if rc == 0 {
            if self.active_destructors.contains(&addr) {
                return Ok(());
            }
            self.active_destructors.insert(addr);

            let header = self.heap.read_header(addr)?;
            if header.tag == BlockTag::Object {
                let vtable_id = self.heap.read_u32(addr).unwrap_or(0);
                let (maybe_destructor_id, reference_offsets) = {
                    let maybe_class = self
                        .context
                        .all_classes
                        .iter()
                        .find(|cls| cls.vtable_id == vtable_id);
                    match maybe_class {
                        Some(class_item) => (
                            class_item.destructor_id,
                            class_item.reference_field_offsets.clone(),
                        ),
                        None => (None, Vec::new()),
                    }
                };

                // 1. Execute class destructor if defined
                if let Some(destructor_id) = maybe_destructor_id {
                    let destructor_result = (|| -> Result<(), RuntimeError> {
                        self.push_frame(destructor_id, 0, &[Value::Ref(addr)])?;
                        let target_depth = self.call_frames.len() - 1;
                        self.run_loop(target_depth)?;
                        Ok(())
                    })();
                    if let Err(ref error) = destructor_result {
                        eprintln!("DTOR ERROR: {error:?}");
                    }
                }

                // 2. Cascade drops to child reference fields directly from VirtualHeap in O(ref_fields)
                for field_offset in reference_offsets {
                    let field_addr = addr + field_offset as u32;
                    let child_addr = self.heap.read_u64(field_addr).unwrap_or(0) as u32;
                    if child_addr > 0 {
                        self.release_reference(child_addr)?;
                    }
                }
            } else if header.tag == BlockTag::ArrayRef {
                let len = self.heap.read_u32(addr).unwrap_or(0);
                let elem_type_tag = self.heap.read_u8(addr + 7).unwrap_or(0);
                if elem_type_tag == 13 || elem_type_tag >= 0x80 {
                    let elem_size = self.heap.read_u8(addr + 4).unwrap_or(8) as usize;
                    for index in 0..len {
                        let elem_addr = addr + 8 + (index as usize * elem_size) as u32;
                        let child_addr = self.heap.read_u64(elem_addr).unwrap_or(0) as u32;
                        if child_addr > 0 {
                            self.release_reference(child_addr)?;
                        }
                    }
                }
            } else if header.tag == BlockTag::Tuple {
                let count = self.heap.read_u32(addr).unwrap_or(0);
                for index in 0..count {
                    let elem_addr = addr + 8 + (index as usize * 8) as u32;
                    let val = Value::from_raw_u64(self.heap.read_u64(elem_addr).unwrap_or(0));
                    if let Value::Ref(child_addr) = val
                        && child_addr > 0
                    {
                        self.release_reference(child_addr)?;
                    }
                }
            } else if header.tag == BlockTag::Closure {
                let cap_count = self.heap.read_u32(addr + 4).unwrap_or(0);
                for index in 0..cap_count {
                    let cell_addr = addr + 8 + (index as usize * 8) as u32;
                    let val = Value::from_raw_u64(self.heap.read_u64(cell_addr).unwrap_or(0));
                    if let Value::Ref(child_addr) = val
                        && child_addr > 0
                    {
                        self.release_reference(child_addr)?;
                    }
                }
            }

            self.heap.deallocate(addr)?;
            self.active_destructors.remove(&addr);
        }

        Ok(())
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
            frame.active_closure = Some(closure_addr);
        }
        Ok(())
    }

    /// Evaluates nominal and interface inheritance: returns `true` if `vtable_id` is or inherits from `target_vtable_id`.
    #[must_use]
    pub fn is_instance_of(&self, mut current_vtable_id: u32, target_vtable_id: u32) -> bool {
        if current_vtable_id == target_vtable_id {
            return true;
        }

        while let Some(class) = self
            .context
            .all_classes
            .iter()
            .find(|c| c.vtable_id == current_vtable_id)
        {
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
        self.call_frames.last().map_or(0, |f| f.pc)
    }

    /// Returns a reference to the topmost active call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the call stack is empty.
    pub fn current_frame(&self) -> Result<&CallFrame, RuntimeError> {
        self.call_frames
            .last()
            .ok_or(RuntimeError::NullReferenceAccess)
    }

    /// Returns a mutable reference to the topmost active call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the call stack is empty.
    pub fn current_frame_mut(&mut self) -> Result<&mut CallFrame, RuntimeError> {
        self.call_frames
            .last_mut()
            .ok_or(RuntimeError::NullReferenceAccess)
    }

    /// Reads a value from local register `reg` within the current call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError::IndexOutOfBounds` if `reg` exceeds the frame's window.
    pub fn get_reg(&self, reg: u16) -> Result<Value, RuntimeError> {
        let frame = self.current_frame()?;
        let reg_idx = reg as usize;
        if reg_idx >= frame.register_count {
            return Err(RuntimeError::IndexOutOfBounds {
                index: reg_idx as i64,
                length: frame.register_count,
            });
        }
        let abs_idx = frame.register_base + reg_idx;
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
        if reg_idx >= frame.register_count {
            return Err(RuntimeError::IndexOutOfBounds {
                index: reg_idx as i64,
                length: frame.register_count,
            });
        }
        let abs_idx = frame.register_base + reg_idx;
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

        // Determine which binary owns this function
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
        let num_regs = (func.num_registers as usize).max(args.len());

        self.register_stack.resize(reg_base + num_regs, Value::Null);
        self.register_stack[reg_base..reg_base + args.len()].copy_from_slice(args);

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

    /// Pops the active call frame and delivers `return_val` to caller's destination register.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the call stack is empty.
    pub fn pop_frame(&mut self, return_val: Value) -> Result<(), RuntimeError> {
        let frame = self
            .call_frames
            .pop()
            .ok_or(RuntimeError::NullReferenceAccess)?;

        self.register_stack.truncate(frame.register_base);

        if self.call_frames.is_empty() {
            self.last_return_value = return_val;
        } else {
            let caller_dst = frame.caller_dst;
            self.set_reg(caller_dst, return_val)?;
        }

        Ok(())
    }

    /// Executes the VM instruction loop down to `target_depth` using either fast or safe mode.
    ///
    /// # Errors
    /// Returns `RuntimeError` if an execution error occurs.
    pub fn run_loop(&mut self, target_depth: usize) -> Result<(), RuntimeError> {
        if self.safe_mode {
            while !self.halted && self.call_frames.len() > target_depth {
                self.step()?;
            }
            Ok(())
        } else {
            self.run_fast(target_depth)
        }
    }

    /// Fetches and dispatches a single virtual machine instruction.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if self.halted || self.call_frames.is_empty() {
            return Ok(());
        }

        let (inst, _func_id) = {
            let (func_id, pc) = {
                let frame = self.current_frame()?;
                (frame.function_id, frame.pc)
            };
            let func = self.context.get_function(func_id).ok_or_else(|| {
                RuntimeError::FunctionNotFound {
                    symbol_or_id: format!("#{func_id}"),
                }
            })?;

            if (pc as usize) >= func.instructions.len() {
                return self.pop_frame(Value::Null);
            }

            let inst = func.instructions[pc as usize];
            let frame = self.current_frame_mut()?;
            frame.pc += 1;
            (inst, func_id)
        };

        self.instructions_dispatched += 1;
        self.dispatch_instruction(inst)
    }

    /// Executes the loaded program: runs module `@init` initializers then the entry point.
    ///
    /// # Errors
    /// Returns `RuntimeError` on execution errors or unhandled exceptions.
    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        let res = self.run_internal();
        if let Err(ref e) = res {
            self.exit_code = Some(e.error_code().as_i32());
            self.halted = true;
        }
        res
    }

    fn run_internal(&mut self) -> Result<Value, RuntimeError> {
        let execution_start = std::time::Instant::now();
        let execution_result = (|| -> Result<Value, RuntimeError> {
            // 1. Static synthesis initialization: execute all @init functions
            let init_ids = self.context.init_function_ids.clone();
            for init_id in init_ids {
                self.push_frame(init_id, 0, &[])?;
                let target_depth = self.call_frames.len() - 1;
                self.run_loop(target_depth)?;
                if self.halted {
                    break;
                }
            }

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

            let entry_func = self.context.get_function(entry_id).ok_or_else(|| {
                RuntimeError::EntryPointNotFound {
                    symbol: format!("#{entry_id}"),
                }
            })?;

            // Entry point signature validation: parameter count must be 0 or 1
            if entry_func.param_count > 1 {
                return Err(RuntimeError::InvalidEntryPointSignature {
                    symbol: entry_func.symbol.clone(),
                    param_count: entry_func.param_count,
                });
            }

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
            Err(e) => {
                let code = e.error_code().as_i32();
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
        let interned_addrs: Vec<u32> = self.interned_string_addrs.drain().collect();
        for addr in interned_addrs {
            let _ = self.heap.deallocate(addr);
        }
        self.heap.verify_clean_memory()
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
        self.quickened_instructions_count += 1;
    }
}
