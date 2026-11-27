//! Interactive virtual machine debug hooks, state controller, and inspection APIs.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;

/// Signal returned by `VmDebugHook::before_instruction` to control VM execution flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSignal {
    /// Continue execution normally until the next breakpoint, trap, or pause.
    Continue,
    /// Pause execution immediately and return control to the debugger.
    Pause,
    /// Terminate execution immediately.
    Terminate,
}

/// Interface for interactive debugger control and inspection.
pub trait VmDebugHook: Send {
    /// Invoked before dispatching an instruction within a function with debug symbols.
    fn before_instruction(
        &mut self,
        vm: &VirtualMachine,
        function_id: u32,
        instruction_pc: u32,
    ) -> Result<DebugSignal, RuntimeError>;

    /// Invoked when a guest exception is thrown (`OpCode::Throw`).
    fn on_exception_thrown(
        &mut self,
        _vm: &VirtualMachine,
        _function_id: u32,
        _instruction_pc: u32,
        _exception_value: &Value,
    ) -> Result<DebugSignal, RuntimeError> {
        Ok(DebugSignal::Continue)
    }

    /// Invoked when an exception or trap is uncaught before unwinding out of `main`.
    fn on_uncaught_exception(
        &mut self,
        _vm: &VirtualMachine,
        _function_id: u32,
        _instruction_pc: u32,
        _runtime_error: &RuntimeError,
    ) -> Result<DebugSignal, RuntimeError> {
        Ok(DebugSignal::Continue)
    }
}

impl VirtualMachine {
    /// Executes the virtual machine under debugger supervision with zero overhead in release mode.
    ///
    /// # Errors
    /// Returns `RuntimeError` on unhandled guest faults or memory violations.
    pub fn run_debug(&mut self, hook: &mut dyn VmDebugHook) -> Result<DebugSignal, RuntimeError> {
        while !self.halted && !self.call_frames.is_empty() {
            let current_frame = match self.call_frames.last() {
                Some(active_frame) => active_frame,
                None => break,
            };

            let function_id = current_frame.function_id;
            let current_pc = current_frame.pc;

            let has_debug_symbols = self
                .context
                .function_meta
                .get(function_id as usize)
                .is_some_and(|metadata| metadata.has_debug_symbols);

            if has_debug_symbols {
                let signal = hook.before_instruction(self, function_id, current_pc)?;
                match signal {
                    DebugSignal::Pause => return Ok(DebugSignal::Pause),
                    DebugSignal::Terminate => {
                        self.halted = true;
                        return Ok(DebugSignal::Terminate);
                    }
                    DebugSignal::Continue => {}
                }
            }

            self.run_fast_step()?;
        }

        Ok(DebugSignal::Continue)
    }

    /// Inspects in-scope local variables and parameters for an active call frame.
    #[must_use]
    pub fn inspect_frame_locals(&self, frame_index: usize) -> Vec<(String, String, Value)> {
        let mut results = Vec::new();
        if frame_index >= self.call_frames.len() {
            return results;
        }

        let target_frame = &self.call_frames[frame_index];
        let compiled_function = match self
            .context
            .all_functions
            .get(target_frame.function_id as usize)
        {
            Some(func) => func,
            None => return results,
        };

        for local_entry in &compiled_function.local_variables {
            if target_frame.pc >= local_entry.start_pc && target_frame.pc <= local_entry.end_pc {
                let absolute_register_index =
                    target_frame.register_base as usize + local_entry.register as usize;
                let register_value = self
                    .register_stack
                    .get(absolute_register_index)
                    .copied()
                    .unwrap_or(Value::Null);

                results.push((
                    local_entry.name.clone(),
                    local_entry.type_name.clone(),
                    register_value,
                ));
            }
        }

        results
    }

    /// Inspects the declared fields of an object allocated on the heap.
    ///
    /// # Errors
    /// Returns `RuntimeError` if the address does not point to a valid heap object.
    pub fn inspect_object_fields(
        &self,
        object_address: u32,
    ) -> Result<Vec<(String, String, Value)>, RuntimeError> {
        let mut field_results = Vec::new();
        if object_address == 0 {
            return Ok(field_results);
        }

        // Read VTable ID stored at header offset
        let header_vtable_id = self.heap.read_u32(object_address)?;
        let compiled_class = self
            .context
            .vtable_lookup
            .get(header_vtable_id as usize)
            .and_then(|class_opt| class_opt.as_ref())
            .ok_or(RuntimeError::InvalidVTableAccess {
                vtable_id: header_vtable_id,
            })?;

        for (byte_offset, field_name, field_type_name) in &compiled_class.field_names {
            let field_address = object_address + *byte_offset as u32;
            let field_value = if compiled_class.reference_field_offsets.contains(byte_offset) {
                let heap_ref = self.heap.read_u32(field_address)?;
                if heap_ref == 0 {
                    Value::Null
                } else {
                    Value::Ref(heap_ref)
                }
            } else {
                match field_type_name.as_str() {
                    "int8" => Value::I32(i32::from(self.heap.read_u8(field_address)? as i8)),
                    "uint8" => Value::I32(i32::from(self.heap.read_u8(field_address)?)),
                    "int16" => Value::I32(i32::from(self.heap.read_u16(field_address)? as i16)),
                    "uint16" => Value::I32(i32::from(self.heap.read_u16(field_address)?)),
                    "int32" => Value::I32(self.heap.read_u32(field_address)? as i32),
                    "uint32" => Value::I64(i64::from(self.heap.read_u32(field_address)?)),
                    "int64" => Value::I64(self.heap.read_u64(field_address)? as i64),
                    "uint64" => Value::I64(self.heap.read_u64(field_address)? as i64),
                    "f32" | "float32" => Value::F64(f64::from(f32::from_bits(
                        self.heap.read_u32(field_address)?,
                    ))),
                    "f64" | "float64" => {
                        Value::F64(f64::from_bits(self.heap.read_u64(field_address)?))
                    }
                    "bool" => Value::Bool(self.heap.read_u8(field_address)? != 0),
                    "char" => Value::I32(self.heap.read_u32(field_address)? as i32),
                    "string" | "object" => {
                        let heap_ref = self.heap.read_u32(field_address)?;
                        if heap_ref == 0 {
                            Value::Null
                        } else {
                            Value::Ref(heap_ref)
                        }
                    }
                    _ => Value::I32(self.heap.read_u32(field_address)? as i32),
                }
            };

            field_results.push((field_name.clone(), field_type_name.clone(), field_value));
        }

        Ok(field_results)
    }

    /// Inspects the elements of an array allocated on the heap.
    ///
    /// # Errors
    /// Returns `RuntimeError` if the address does not point to a valid heap array.
    pub fn inspect_array_elements(
        &self,
        array_address: u32,
    ) -> Result<Vec<(String, String, Value)>, RuntimeError> {
        let mut results = Vec::new();
        if array_address == 0 {
            return Ok(results);
        }
        let header = self.heap.read_header(array_address)?;
        if header.tag != crate::memory::BlockTag::ArrayRef {
            return Ok(results);
        }
        let element_count = self.heap.read_u32(array_address)? as usize;
        let element_type_tag = self.heap.read_u32(array_address + 4)?;
        for element_index in 0..element_count.min(256) {
            let element_offset = array_address + 8 + (element_index * 8) as u32;
            let element_value = match element_type_tag {
                1..=5 => Value::I32(self.heap.read_u32(element_offset)? as i32),
                6 | 7 => Value::I64(self.heap.read_u64(element_offset)? as i64),
                8 | 9 => Value::F64(f64::from_bits(self.heap.read_u64(element_offset)?)),
                10 => Value::Bool(self.heap.read_u8(element_offset)? != 0),
                _ => {
                    let heap_reference = self.heap.read_u32(element_offset)?;
                    if heap_reference == 0 {
                        Value::Null
                    } else {
                        Value::Ref(heap_reference)
                    }
                }
            };
            results.push((
                format!("[{element_index}]"),
                element_value.type_name().to_string(),
                element_value,
            ));
        }
        Ok(results)
    }

    /// Mutates the value of a register within an active call frame.
    ///
    /// # Errors
    /// Returns `RuntimeError` if the frame index is invalid.
    pub fn mutate_register(
        &mut self,
        frame_index: usize,
        register: u16,
        new_value: Value,
    ) -> Result<(), RuntimeError> {
        if frame_index >= self.call_frames.len() {
            return Err(RuntimeError::IndexOutOfBounds {
                index: frame_index as i64,
                length: self.call_frames.len(),
            });
        }

        let target_frame = &self.call_frames[frame_index];
        let absolute_register_index = target_frame.register_base as usize + register as usize;
        if absolute_register_index < self.register_stack.len() {
            self.register_stack[absolute_register_index] = new_value;
        }

        Ok(())
    }

    /// Mutates the field value of an object allocated on the heap.
    ///
    /// # Errors
    /// Returns `RuntimeError` on invalid address or out-of-bounds offset.
    pub fn mutate_object_field(
        &mut self,
        object_address: u32,
        byte_offset: usize,
        new_value: Value,
    ) -> Result<(), RuntimeError> {
        let field_address = object_address + byte_offset as u32;
        match new_value {
            Value::I32(int_value) => self.heap.write_u32(field_address, int_value as u32)?,
            Value::I64(long_value) => self.heap.write_u64(field_address, long_value as u64)?,
            Value::F64(double_value) => {
                self.heap.write_u64(field_address, double_value.to_bits())?
            }
            Value::Bool(bool_value) => self
                .heap
                .write_u8(field_address, if bool_value { 1 } else { 0 })?,
            Value::Ref(ref_address) => self.heap.write_u32(field_address, ref_address)?,
            Value::FnPtr(fn_ptr) => self.heap.write_u32(field_address, fn_ptr)?,
            Value::Null => self.heap.write_u32(field_address, 0)?,
        }
        Ok(())
    }
}
