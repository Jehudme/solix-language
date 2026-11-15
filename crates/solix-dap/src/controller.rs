//! Execution controller and VmDebugHook implementing breakpoints, stepping, and exceptions.

#![forbid(unsafe_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use solix_common::bytecode::SourceMapEntry;
use solix_runtime::exceptions::RuntimeError;
use solix_runtime::value::Value;
use solix_runtime::vm::debug::{DebugSignal, VmDebugHook};
use solix_runtime::vm::engine::VirtualMachine;

use crate::evaluator::{
    check_hit_condition, evaluate_expression_in_frame, interpolate_log_template,
};
use crate::state::{BreakpointEntry, FunctionBreakpointEntry, SteppingState};

/// Type definition for logpoint output notifications sent to the DAP client.
pub type LogpointCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Interactive execution controller implementing `VmDebugHook`.
pub struct DapController {
    /// Active line breakpoints registered with the debugger.
    pub line_breakpoints: Vec<BreakpointEntry>,
    /// Active function breakpoints registered with the debugger.
    pub function_breakpoints: Vec<FunctionBreakpointEntry>,
    /// Current stepping mode.
    pub stepping_state: SteppingState,
    /// Pause on thrown exceptions filter.
    pub break_on_thrown_exceptions: bool,
    /// Pause on uncaught exceptions filter.
    pub break_on_uncaught_exceptions: bool,
    /// Asynchronous pause flag requested by the client.
    pub async_pause_requested: Arc<AtomicBool>,
    /// Optional logpoint output callback to emit console events without pausing.
    pub logpoint_output_callback: Option<LogpointCallback>,
    /// Human-readable reason for the most recent pause event.
    pub last_pause_reason: String,
    /// Location of the most recently hit breakpoint to avoid immediate re-triggering upon resumption.
    pub last_hit_breakpoint_location: Option<(u32, u32)>,
}

impl DapController {
    /// Constructs a new `DapController` instance.
    #[must_use]
    pub fn new(async_pause_requested: Arc<AtomicBool>) -> Self {
        Self {
            line_breakpoints: Vec::new(),
            function_breakpoints: Vec::new(),
            stepping_state: SteppingState::None,
            break_on_thrown_exceptions: false,
            break_on_uncaught_exceptions: true,
            async_pause_requested,
            logpoint_output_callback: None,
            last_pause_reason: "entry".to_string(),
            last_hit_breakpoint_location: None,
        }
    }

    /// Resolves the current source file path and 1-based source line for a given PC.
    #[must_use]
    pub fn resolve_source_location(
        virtual_machine: &VirtualMachine,
        function_id: u32,
        instruction_pc: u32,
    ) -> Option<(String, u32)> {
        let compiled_function = virtual_machine
            .context
            .all_functions
            .get(function_id as usize)?;
        let function_metadata = virtual_machine
            .context
            .function_meta
            .get(function_id as usize)?;
        let loaded_binary = virtual_machine
            .context
            .loaded_binaries
            .get(function_metadata.binary_id)?;

        let source_file_path = loaded_binary
            .source_files
            .get(compiled_function.source_id as usize)
            .cloned()
            .unwrap_or_default();

        let resolved_line = find_source_line_for_pc(&compiled_function.source_map, instruction_pc);
        Some((source_file_path, resolved_line))
    }
}

/// Finds the source line number for an instruction PC using binary search over the source map.
fn find_source_line_for_pc(source_map: &[SourceMapEntry], instruction_pc: u32) -> u32 {
    if source_map.is_empty() {
        return 0;
    }

    let mut current_best_line = source_map[0].line;
    for entry in source_map {
        if entry.instruction_pc <= instruction_pc {
            current_best_line = entry.line;
        } else {
            break;
        }
    }
    current_best_line
}

impl VmDebugHook for DapController {
    fn before_instruction(
        &mut self,
        virtual_machine: &VirtualMachine,
        function_id: u32,
        instruction_pc: u32,
    ) -> Result<DebugSignal, RuntimeError> {
        // 1. Check asynchronous client pause
        if self.async_pause_requested.swap(false, Ordering::SeqCst) {
            self.last_pause_reason = "pause".to_string();
            return Ok(DebugSignal::Pause);
        }

        // 2. Resolve source location
        let (current_file_path, current_line_number) =
            Self::resolve_source_location(virtual_machine, function_id, instruction_pc)
                .unwrap_or_default();

        let current_frame_depth = virtual_machine.call_frames.len();
        let active_frame_index = current_frame_depth.saturating_sub(1);

        let is_at_last_hit =
            self.last_hit_breakpoint_location == Some((function_id, instruction_pc));
        if !is_at_last_hit {
            self.last_hit_breakpoint_location = None;
        }

        // 3. Check line breakpoints
        if !is_at_last_hit {
            for line_breakpoint in &mut self.line_breakpoints {
                let Some(breakpoint_function_id) = line_breakpoint.function_id else {
                    continue;
                };
                if breakpoint_function_id != function_id {
                    continue;
                }

                let normalized_current = current_file_path.replace('\\', "/");
                let normalized_breakpoint = line_breakpoint.source_path.replace('\\', "/");
                let matches_file = normalized_current == normalized_breakpoint
                    || normalized_current.ends_with(&normalized_breakpoint)
                    || normalized_breakpoint.ends_with(&normalized_current);

                let matches_line = instruction_pc == line_breakpoint.instruction_pc;

                if matches_file && matches_line {
                    line_breakpoint.current_hit_count += 1;

                    // Test hit count condition if specified
                    if let Some(hit_condition) = &line_breakpoint.hit_condition_expression {
                        if !check_hit_condition(line_breakpoint.current_hit_count, hit_condition) {
                            continue;
                        }
                    }

                    // Test boolean guard expression if specified
                    if let Some(guard_expression) = &line_breakpoint.condition_expression {
                        match evaluate_expression_in_frame(
                            guard_expression,
                            virtual_machine,
                            active_frame_index,
                        ) {
                            Ok(evaluated_value) => {
                                if !evaluated_value.is_truthy() {
                                    continue;
                                }
                            }
                            Err(_) => continue,
                        }
                    }

                    // Check if this is a logpoint
                    if let Some(log_template) = &line_breakpoint.log_message_template {
                        let rendered_log_message = interpolate_log_template(
                            log_template,
                            virtual_machine,
                            active_frame_index,
                        );
                        if let Some(log_callback) = &self.logpoint_output_callback {
                            log_callback(rendered_log_message);
                        }
                        // Logpoints do not pause VM execution!
                        return Ok(DebugSignal::Continue);
                    }

                    self.last_pause_reason = "breakpoint".to_string();
                    self.last_hit_breakpoint_location = Some((function_id, instruction_pc));
                    return Ok(DebugSignal::Pause);
                }
            }

            // 4. Check function breakpoints on entry (instruction_pc == 0)
            if instruction_pc == 0 {
                if let Some(compiled_function) = virtual_machine
                    .context
                    .all_functions
                    .get(function_id as usize)
                {
                    let function_symbol = &compiled_function.symbol;
                    for function_breakpoint in &mut self.function_breakpoints {
                        let matches_name = function_symbol == &function_breakpoint.function_name
                            || function_symbol
                                .ends_with(&format!("::{}", function_breakpoint.function_name));

                        if matches_name {
                            function_breakpoint.current_hit_count += 1;

                            if let Some(hit_condition) =
                                &function_breakpoint.hit_condition_expression
                            {
                                if !check_hit_condition(
                                    function_breakpoint.current_hit_count,
                                    hit_condition,
                                ) {
                                    continue;
                                }
                            }

                            if let Some(guard_expression) =
                                &function_breakpoint.condition_expression
                            {
                                match evaluate_expression_in_frame(
                                    guard_expression,
                                    virtual_machine,
                                    active_frame_index,
                                ) {
                                    Ok(evaluated_value) => {
                                        if !evaluated_value.is_truthy() {
                                            continue;
                                        }
                                    }
                                    Err(_) => continue,
                                }
                            }

                            self.last_pause_reason = "function breakpoint".to_string();
                            self.last_hit_breakpoint_location = Some((function_id, instruction_pc));
                            return Ok(DebugSignal::Pause);
                        }
                    }
                }
            }
        }

        // 5. Evaluate active stepping state
        match &self.stepping_state {
            SteppingState::None => {}
            SteppingState::StepOver {
                start_call_depth,
                start_file_path,
                start_line_number,
            } => {
                if current_frame_depth <= *start_call_depth {
                    let line_changed = current_line_number != *start_line_number;
                    let file_changed = &current_file_path != start_file_path;
                    if (line_changed || file_changed) && current_line_number > 0 {
                        self.stepping_state = SteppingState::None;
                        self.last_pause_reason = "step".to_string();
                        return Ok(DebugSignal::Pause);
                    }
                }
            }
            SteppingState::StepIn {
                start_file_path,
                start_line_number,
            } => {
                let line_changed = current_line_number != *start_line_number;
                let file_changed = &current_file_path != start_file_path;
                if (line_changed || file_changed) && current_line_number > 0 {
                    self.stepping_state = SteppingState::None;
                    self.last_pause_reason = "step".to_string();
                    return Ok(DebugSignal::Pause);
                }
            }
            SteppingState::StepOut { target_call_depth } => {
                if current_frame_depth < *target_call_depth {
                    self.stepping_state = SteppingState::None;
                    self.last_pause_reason = "step".to_string();
                    return Ok(DebugSignal::Pause);
                }
            }
        }

        Ok(DebugSignal::Continue)
    }

    fn on_exception_thrown(
        &mut self,
        _virtual_machine: &VirtualMachine,
        _function_id: u32,
        _instruction_pc: u32,
        _exception_value: &Value,
    ) -> Result<DebugSignal, RuntimeError> {
        if self.break_on_thrown_exceptions {
            self.last_pause_reason = "exception".to_string();
            return Ok(DebugSignal::Pause);
        }
        Ok(DebugSignal::Continue)
    }

    fn on_uncaught_exception(
        &mut self,
        _virtual_machine: &VirtualMachine,
        _function_id: u32,
        _instruction_pc: u32,
        _runtime_error: &RuntimeError,
    ) -> Result<DebugSignal, RuntimeError> {
        if self.break_on_uncaught_exceptions {
            self.last_pause_reason = "exception".to_string();
            return Ok(DebugSignal::Pause);
        }
        Ok(DebugSignal::Continue)
    }
}
