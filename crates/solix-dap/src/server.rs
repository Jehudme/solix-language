//! DAP JSON-RPC server loop over stdio or TCP socket.

#![forbid(unsafe_code)]

use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dap::events::{Event, ExitedEventBody, StoppedEventBody};
use dap::requests::{
    Command, LaunchRequestArguments, SetBreakpointsArguments,
};
use dap::responses::{
    ContinueResponse, DisassembleResponse, EvaluateResponse, ModulesResponse, ResponseBody,
    ScopesResponse, SetBreakpointsResponse, SetExceptionBreakpointsResponse,
    SetFunctionBreakpointsResponse, SetVariableResponse, StackTraceResponse, ThreadsResponse,
    VariablesResponse,
};
use dap::server::Server;
use dap::types::{
    Breakpoint, Capabilities, DisassembledInstruction, Module, Scope,
    Source, StackFrame, StoppedEventReason, Thread, Variable,
};

use solix_common::bytecode::BinaryProgram;
use solix_common::project::SlxarcArchive;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::pipeline::CompilationPipeline;
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::value::Value;
use solix_runtime::vm::debug::DebugSignal;
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};

use crate::controller::DapController;
use crate::evaluator::evaluate_expression_in_frame;
use crate::state::{BreakpointEntry, FunctionBreakpointEntry, SteppingState, VariableReferenceManager, VariableScopeType};

/// Runs the Solix DAP server on stdio or over a TCP port.
///
/// # Errors
/// Returns an error if socket binding, standard stream access, or execution fails.
pub fn run_dap_server_blocking(
    port_number: Option<u16>,
    _project_directory: Option<PathBuf>,
) -> anyhow::Result<()> {
    if let Some(port) = port_number {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        tracing::info!("Solix DAP listening on 127.0.0.1:{}", port);
        for stream_result in listener.incoming() {
            let socket_stream = stream_result?;
            let reader = BufReader::new(socket_stream.try_clone()?);
            let writer = BufWriter::new(socket_stream);
            let mut dap_server = Server::new(reader, writer);
            handle_client_session(&mut dap_server)?;
        }
    } else {
        let stdin_stream = io::stdin();
        let stdout_stream = io::stdout();
        let reader = BufReader::new(stdin_stream.lock());
        let writer = BufWriter::new(stdout_stream.lock());
        let mut dap_server = Server::new(reader, writer);
        handle_client_session(&mut dap_server)?;
    }

    Ok(())
}

/// Handles a complete DAP client session.
fn handle_client_session<ReaderType: io::Read, WriterType: io::Write>(
    server: &mut Server<ReaderType, WriterType>,
) -> anyhow::Result<()> {
    let async_pause_requested = Arc::new(AtomicBool::new(false));
    let mut controller = DapController::new(async_pause_requested.clone());
    let mut reference_manager = VariableReferenceManager::new();

    let mut virtual_machine: Option<VirtualMachine> = None;
    let mut stop_on_entry_flag = false;
    let mut execution_launched = false;

    while let Some(request) = server.poll_request()? {
        match request.command {
            Command::Initialize(_) => {
                let capabilities = Capabilities {
                    supports_configuration_done_request: Some(true),
                    supports_function_breakpoints: Some(true),
                    supports_conditional_breakpoints: Some(true),
                    supports_hit_conditional_breakpoints: Some(true),
                    supports_log_points: Some(true),
                    supports_evaluate_for_hovers: Some(true),
                    supports_set_variable: Some(true),
                    supports_modules_request: Some(true),
                    supports_disassemble_request: Some(true),
                    supports_terminate_request: Some(true),
                    ..Default::default()
                };

                server.respond(request.success(ResponseBody::Initialize(capabilities)))?;
                server.send_event(Event::Initialized)?;
            }

            Command::Launch(ref launch_arguments) => {
                let parsed_program_path = extract_launch_string(launch_arguments, "program");
                let stop_on_entry = extract_launch_boolean(launch_arguments, "stopOnEntry").unwrap_or(false);
                stop_on_entry_flag = stop_on_entry;

                if let Some(target_program) = parsed_program_path {
                    let target_path = PathBuf::from(&target_program);
                    let runtime_options = match setup_runtime_for_target(&target_path) {
                        Ok(options) => options,
                        Err(setup_error) => {
                            server.respond(request.error(&setup_error.to_string()))?;
                            continue;
                        }
                    };

                    let runtime_context = match RuntimeContext::new(runtime_options) {
                        Ok(context) => context,
                        Err(context_error) => {
                            server.respond(request.error(&context_error.to_string()))?;
                            continue;
                        }
                    };
                    let mut created_vm = VirtualMachine::new(runtime_context);

                    // Execute static initializers
                    for init_func_id in created_vm.context.init_function_ids.clone() {
                        let _ = created_vm.push_frame(init_func_id, 0, &[]);
                        let target_depth = created_vm.call_frames.len().saturating_sub(1);
                        let _ = created_vm.run_loop(target_depth);
                    }

                    // Prepare entry point
                    if let Some(entry_id) = created_vm.context.entry_point_function_id {
                        let _ = created_vm.push_frame(entry_id, 0, &[]);
                    }

                    // Re-resolve any breakpoints registered before launch
                    for line_breakpoint in &mut controller.line_breakpoints {
                        if line_breakpoint.function_id.is_none() {
                            if let Some((snapped_line, snapped_pc, candidate_function_id)) =
                                resolve_breakpoint_location(&line_breakpoint.source_path, line_breakpoint.requested_line, &created_vm)
                            {
                                line_breakpoint.verified_line = snapped_line;
                                line_breakpoint.instruction_pc = snapped_pc;
                                line_breakpoint.function_id = Some(candidate_function_id);
                            }
                        }
                    }

                    virtual_machine = Some(created_vm);
                    execution_launched = true;
                }

                server.respond(request.success(ResponseBody::Launch))?;
            }

            Command::ConfigurationDone => {
                server.respond(request.success(ResponseBody::ConfigurationDone))?;

                if execution_launched {
                    if stop_on_entry_flag {
                        server.send_event(Event::Stopped(StoppedEventBody {
                            reason: StoppedEventReason::Entry,
                            description: Some("Paused on program entry point".to_string()),
                            thread_id: Some(1),
                            preserve_focus_hint: Some(false),
                            text: None,
                            all_threads_stopped: Some(true),
                            hit_breakpoint_ids: None,
                        }))?;
                    } else if let Some(vm_ref) = virtual_machine.as_mut() {
                        run_until_stop(vm_ref, &mut controller, server)?;
                    }
                }
            }

            Command::SetBreakpoints(ref breakpoints_arguments) => {
                let (source_path, registered_breakpoints) = process_set_breakpoints(
                    breakpoints_arguments,
                    virtual_machine.as_ref(),
                );

                controller.line_breakpoints.retain(|bp| bp.source_path != source_path);
                controller.line_breakpoints.extend(registered_breakpoints.clone());

                let response_breakpoints: Vec<Breakpoint> = registered_breakpoints
                    .iter()
                    .map(|entry| Breakpoint {
                        id: None,
                        verified: true,
                        message: None,
                        source: Some(Source {
                            name: Some(Path::new(&entry.source_path).file_name().unwrap_or_default().to_string_lossy().to_string()),
                            path: Some(entry.source_path.clone()),
                            source_reference: None,
                            presentation_hint: None,
                            origin: None,
                            sources: None,
                            adapter_data: None,
                            checksums: None,
                        }),
                        line: Some(entry.verified_line as i64),
                        column: Some(1),
                        end_line: None,
                        end_column: None,
                        instruction_reference: None,
                        offset: None,
                    })
                    .collect();

                server.respond(request.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse {
                    breakpoints: response_breakpoints,
                })))?;
            }

            Command::SetFunctionBreakpoints(ref func_breakpoints_args) => {
                let mut registered_func_breakpoints = Vec::new();
                for func_bp in &func_breakpoints_args.breakpoints {
                    registered_func_breakpoints.push(FunctionBreakpointEntry {
                        function_name: func_bp.name.clone(),
                        condition_expression: func_bp.condition.clone(),
                        hit_condition_expression: func_bp.hit_condition.clone(),
                        current_hit_count: 0,
                    });
                }
                controller.function_breakpoints = registered_func_breakpoints.clone();

                let verified_breakpoints: Vec<Breakpoint> = registered_func_breakpoints
                    .iter()
                    .map(|func_entry| Breakpoint {
                        id: None,
                        verified: true,
                        message: Some(format!("Function '{}'", func_entry.function_name)),
                        source: None,
                        line: None,
                        column: None,
                        end_line: None,
                        end_column: None,
                        instruction_reference: None,
                        offset: None,
                    })
                    .collect();

                server.respond(request.success(ResponseBody::SetFunctionBreakpoints(SetFunctionBreakpointsResponse {
                    breakpoints: verified_breakpoints,
                })))?;
            }

            Command::SetExceptionBreakpoints(ref exception_breakpoints_args) => {
                controller.break_on_thrown_exceptions = exception_breakpoints_args.filters.contains(&"thrown".to_string());
                controller.break_on_uncaught_exceptions = exception_breakpoints_args.filters.contains(&"uncaught".to_string());
                server.respond(request.success(ResponseBody::SetExceptionBreakpoints(SetExceptionBreakpointsResponse {
                    breakpoints: None,
                })))?;
            }

            Command::Threads => {
                let threads_list = vec![Thread {
                    id: 1,
                    name: "Solix Main Thread".to_string(),
                }];
                server.respond(request.success(ResponseBody::Threads(ThreadsResponse {
                    threads: threads_list,
                })))?;
            }

            Command::StackTrace(_) => {
                let mut stack_frames = Vec::new();
                if let Some(vm_ref) = virtual_machine.as_ref() {
                    for (frame_index, active_frame) in vm_ref.call_frames.iter().enumerate().rev() {
                        let function_id = active_frame.function_id;
                        let function_symbol = vm_ref
                            .context
                            .all_functions
                            .get(function_id as usize)
                            .map(|func| func.symbol.clone())
                            .unwrap_or_else(|| format!("func_{function_id}"));

                        let (file_path, line_number) = DapController::resolve_source_location(
                            vm_ref,
                            function_id,
                            active_frame.pc,
                        ).unwrap_or((String::new(), 1));

                        let file_name = Path::new(&file_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        stack_frames.push(StackFrame {
                            id: frame_index as i64,
                            name: function_symbol,
                            source: Some(Source {
                                name: Some(file_name),
                                path: Some(file_path),
                                source_reference: None,
                                presentation_hint: None,
                                origin: None,
                                sources: None,
                                adapter_data: None,
                                checksums: None,
                            }),
                            line: line_number as i64,
                            column: 1,
                            end_line: None,
                            end_column: None,
                            can_restart: Some(false),
                            instruction_pointer_reference: Some(format!("0x{:X}", active_frame.pc)),
                            module_id: None,
                            presentation_hint: None,
                        });
                    }
                }

                let total_frames = stack_frames.len() as i64;
                server.respond(request.success(ResponseBody::StackTrace(StackTraceResponse {
                    stack_frames,
                    total_frames: Some(total_frames),
                })))?;
            }

            Command::Scopes(ref scopes_arguments) => {
                reference_manager.reset_transient_references();
                let frame_index = scopes_arguments.frame_id as usize;

                let locals_ref = reference_manager.allocate_reference(VariableScopeType::FrameLocals(frame_index));
                let statics_ref = reference_manager.allocate_reference(VariableScopeType::GlobalStatics);
                let registers_ref = reference_manager.allocate_reference(VariableScopeType::VmRegisters(frame_index));

                let scopes_list = vec![
                    Scope {
                        name: "Locals & Parameters".to_string(),
                        presentation_hint: None,
                        variables_reference: locals_ref,
                        named_variables: None,
                        indexed_variables: None,
                        expensive: false,
                        source: None,
                        line: None,
                        column: None,
                        end_line: None,
                        end_column: None,
                    },
                    Scope {
                        name: "Static & Global Variables".to_string(),
                        presentation_hint: None,
                        variables_reference: statics_ref,
                        named_variables: None,
                        indexed_variables: None,
                        expensive: false,
                        source: None,
                        line: None,
                        column: None,
                        end_line: None,
                        end_column: None,
                    },
                    Scope {
                        name: "VM Registers & Telemetry".to_string(),
                        presentation_hint: None,
                        variables_reference: registers_ref,
                        named_variables: None,
                        indexed_variables: None,
                        expensive: false,
                        source: None,
                        line: None,
                        column: None,
                        end_line: None,
                        end_column: None,
                    },
                ];

                server.respond(request.success(ResponseBody::Scopes(ScopesResponse {
                    scopes: scopes_list,
                })))?;
            }

            Command::Variables(ref variables_arguments) => {
                let mut variables_list = Vec::new();
                if let Some(vm_ref) = virtual_machine.as_ref() {
                    let requested_scope = reference_manager.resolve_reference(variables_arguments.variables_reference).cloned();
                    if let Some(scope_type) = requested_scope {
                        match scope_type {
                            VariableScopeType::FrameLocals(frame_idx) => {
                                let locals = vm_ref.inspect_frame_locals(frame_idx);
                                for (var_name, var_type, var_val) in locals {
                                    let child_reference = match var_val {
                                        Value::Ref(object_address) if object_address > 0 => {
                                            reference_manager.allocate_reference(VariableScopeType::ObjectFields(object_address))
                                        }
                                        _ => 0,
                                    };

                                    variables_list.push(Variable {
                                        name: var_name,
                                        value: format_runtime_value(&var_val),
                                        type_field: Some(var_type),
                                        presentation_hint: None,
                                        evaluate_name: None,
                                        variables_reference: child_reference,
                                        named_variables: None,
                                        indexed_variables: None,
                                        memory_reference: None,
                                    });
                                }
                            }
                            VariableScopeType::GlobalStatics => {
                                for (slot_index, slot_val) in vm_ref.static_slots.iter().enumerate() {
                                    let child_reference = match slot_val {
                                        Value::Ref(object_address) if *object_address > 0 => {
                                            reference_manager.allocate_reference(VariableScopeType::ObjectFields(*object_address))
                                        }
                                        _ => 0,
                                    };

                                    variables_list.push(Variable {
                                        name: format!("static_{slot_index}"),
                                        value: format_runtime_value(slot_val),
                                        type_field: Some(slot_val.type_name().to_string()),
                                        presentation_hint: None,
                                        evaluate_name: None,
                                        variables_reference: child_reference,
                                        named_variables: None,
                                        indexed_variables: None,
                                        memory_reference: None,
                                    });
                                }
                            }
                            VariableScopeType::VmRegisters(frame_idx) => {
                                if let Some(frame) = vm_ref.call_frames.get(frame_idx) {
                                    for register_idx in 0..16 {
                                        let absolute_idx = frame.register_base + register_idx;
                                        if let Some(register_val) = vm_ref.register_stack.get(absolute_idx) {
                                            variables_list.push(Variable {
                                                name: format!("r{register_idx}"),
                                                value: format_runtime_value(register_val),
                                                type_field: Some(register_val.type_name().to_string()),
                                                presentation_hint: None,
                                                evaluate_name: None,
                                                variables_reference: 0,
                                                named_variables: None,
                                                indexed_variables: None,
                                                memory_reference: None,
                                            });
                                        }
                                    }
                                }
                            }
                            VariableScopeType::ObjectFields(object_addr) => {
                                if let Ok(fields) = vm_ref.inspect_object_fields(object_addr) {
                                    for (field_name, field_type, field_val) in fields {
                                        let child_reference = match field_val {
                                            Value::Ref(nested_addr) if nested_addr > 0 => {
                                                reference_manager.allocate_reference(VariableScopeType::ObjectFields(nested_addr))
                                            }
                                            _ => 0,
                                        };

                                        variables_list.push(Variable {
                                            name: field_name,
                                            value: format_runtime_value(&field_val),
                                            type_field: Some(field_type),
                                            presentation_hint: None,
                                            evaluate_name: None,
                                            variables_reference: child_reference,
                                            named_variables: None,
                                            indexed_variables: None,
                                            memory_reference: None,
                                        });
                                    }
                                }
                            }
                            VariableScopeType::ArrayElements(_) => {}
                        }
                    }
                }

                server.respond(request.success(ResponseBody::Variables(VariablesResponse {
                    variables: variables_list,
                })))?;
            }

            Command::SetVariable(ref set_variable_arguments) => {
                let updated_value_string = set_variable_arguments.value.clone();
                let parsed_value = parse_literal_to_value(&updated_value_string);

                if let Some(vm_ref) = virtual_machine.as_mut() {
                    let requested_scope = reference_manager.resolve_reference(set_variable_arguments.variables_reference).cloned();
                    if let Some(scope_type) = requested_scope {
                        match scope_type {
                            VariableScopeType::FrameLocals(frame_idx) => {
                                if set_variable_arguments.name.starts_with('r') {
                                    if let Ok(reg_num) = set_variable_arguments.name[1..].parse::<u16>() {
                                        let _ = vm_ref.mutate_register(frame_idx, reg_num, parsed_value);
                                    }
                                }
                            }
                            VariableScopeType::VmRegisters(frame_idx) => {
                                if set_variable_arguments.name.starts_with('r') {
                                    if let Ok(reg_num) = set_variable_arguments.name[1..].parse::<u16>() {
                                        let _ = vm_ref.mutate_register(frame_idx, reg_num, parsed_value);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                server.respond(request.success(ResponseBody::SetVariable(SetVariableResponse {
                    value: updated_value_string,
                    type_field: Some(parsed_value.type_name().to_string()),
                    variables_reference: None,
                    named_variables: None,
                    indexed_variables: None,
                })))?;
            }

            Command::Evaluate(ref evaluate_arguments) => {
                let frame_index = evaluate_arguments.frame_id.unwrap_or(0) as usize;
                let evaluation_result = if let Some(vm_ref) = virtual_machine.as_ref() {
                    evaluate_expression_in_frame(&evaluate_arguments.expression, vm_ref, frame_index)
                } else {
                    Err("Virtual machine not running".to_string())
                };

                match evaluation_result {
                    Ok(evaluated_value) => {
                        let child_reference = match evaluated_value {
                            Value::Ref(object_address) if object_address > 0 => {
                                reference_manager.allocate_reference(VariableScopeType::ObjectFields(object_address))
                            }
                            _ => 0,
                        };

                        server.respond(request.success(ResponseBody::Evaluate(EvaluateResponse {
                            result: format_runtime_value(&evaluated_value),
                            type_field: Some(evaluated_value.type_name().to_string()),
                            presentation_hint: None,
                            variables_reference: child_reference,
                            named_variables: None,
                            indexed_variables: None,
                            memory_reference: None,
                        })))?;
                    }
                    Err(error_string) => {
                        server.respond(request.error(&error_string))?;
                    }
                }
            }

            Command::Continue(_) => {
                server.respond(request.success(ResponseBody::Continue(ContinueResponse {
                    all_threads_continued: Some(true),
                })))?;

                if let Some(vm_ref) = virtual_machine.as_mut() {
                    controller.stepping_state = SteppingState::None;
                    run_until_stop(vm_ref, &mut controller, server)?;
                }
            }

            Command::Next(_) => {
                server.respond(request.success(ResponseBody::Next))?;

                if let Some(vm_ref) = virtual_machine.as_mut() {
                    let current_depth = vm_ref.call_frames.len();
                    let (current_file, current_line) = vm_ref
                        .call_frames
                        .last()
                        .and_then(|frame| DapController::resolve_source_location(vm_ref, frame.function_id, frame.pc))
                        .unwrap_or((String::new(), 0));

                    controller.stepping_state = SteppingState::StepOver {
                        start_call_depth: current_depth,
                        start_file_path: current_file,
                        start_line_number: current_line,
                    };
                    run_until_stop(vm_ref, &mut controller, server)?;
                }
            }

            Command::StepIn(_) => {
                server.respond(request.success(ResponseBody::StepIn))?;

                if let Some(vm_ref) = virtual_machine.as_mut() {
                    let (current_file, current_line) = vm_ref
                        .call_frames
                        .last()
                        .and_then(|frame| DapController::resolve_source_location(vm_ref, frame.function_id, frame.pc))
                        .unwrap_or((String::new(), 0));

                    controller.stepping_state = SteppingState::StepIn {
                        start_file_path: current_file,
                        start_line_number: current_line,
                    };
                    run_until_stop(vm_ref, &mut controller, server)?;
                }
            }

            Command::StepOut(_) => {
                server.respond(request.success(ResponseBody::StepOut))?;

                if let Some(vm_ref) = virtual_machine.as_mut() {
                    let current_depth = vm_ref.call_frames.len();
                    controller.stepping_state = SteppingState::StepOut {
                        target_call_depth: current_depth,
                    };
                    run_until_stop(vm_ref, &mut controller, server)?;
                }
            }

            Command::Pause(_) => {
                async_pause_requested.store(true, Ordering::SeqCst);
                server.respond(request.success(ResponseBody::Pause))?;
            }

            Command::Modules(_) => {
                let mut modules_list = Vec::new();
                if let Some(vm_ref) = virtual_machine.as_ref() {
                    for (binary_idx, loaded_binary) in vm_ref.context.loaded_binaries.iter().enumerate() {
                        let module_name = loaded_binary.source_files.first().cloned().unwrap_or_else(|| format!("binary_{binary_idx}"));
                        modules_list.push(Module {
                            id: dap::types::ModuleId::String(format!("mod_{binary_idx}")),
                            name: module_name,
                            path: loaded_binary.source_files.first().cloned(),
                            is_optimized: Some(!loaded_binary.is_debug_profile),
                            is_user_code: Some(loaded_binary.is_debug_profile),
                            version: Some("2.0".to_string()),
                            symbol_status: Some(if loaded_binary.is_debug_profile {
                                "Debug Symbols Loaded".to_string()
                            } else {
                                "Release Optimized (Skipped)".to_string()
                            }),
                            symbol_file_path: None,
                            date_time_stamp: None,
                            address_range: None,
                        });
                    }
                }

                server.respond(request.success(ResponseBody::Modules(ModulesResponse {
                    modules: modules_list,
                    total_modules: None,
                })))?;
            }

            Command::Disassemble(_) => {
                let mut instructions = Vec::new();
                if let Some(vm_ref) = virtual_machine.as_ref() {
                    if let Some(active_frame) = vm_ref.call_frames.last() {
                        if let Some(compiled_func) = vm_ref.context.all_functions.get(active_frame.function_id as usize) {
                            for (pc_index, instruction) in compiled_func.instructions.iter().enumerate() {
                                instructions.push(DisassembledInstruction {
                                    address: format!("0x{:X}", pc_index),
                                    instruction_bytes: None,
                                    instruction: format!("{instruction:?}"),
                                    symbol: None,
                                    location: None,
                                    line: None,
                                    column: None,
                                    end_line: None,
                                    end_column: None,
                                });
                            }
                        }
                    }
                }

                server.respond(request.success(ResponseBody::Disassemble(DisassembleResponse {
                    instructions,
                })))?;
            }

            Command::Disconnect(_) => {
                server.respond(request.success(ResponseBody::Disconnect))?;
                break;
            }

            Command::Terminate(_) => {
                server.respond(request.success(ResponseBody::Terminate))?;
                server.send_event(Event::Terminated(None))?;
                server.send_event(Event::Exited(ExitedEventBody { exit_code: 0 }))?;
                break;
            }

            _ => {
                server.respond(request.error("Unsupported DAP command"))?;
            }
        }
    }

    Ok(())
}

/// Runs the VM step loop under controller supervision until a pause signal or program halt.
fn run_until_stop<ReaderType: io::Read, WriterType: io::Write>(
    virtual_machine: &mut VirtualMachine,
    controller: &mut DapController,
    server: &mut Server<ReaderType, WriterType>,
) -> anyhow::Result<()> {
    let debug_signal = virtual_machine
        .run_debug(controller)
        .map_err(|runtime_error| anyhow::anyhow!("{runtime_error}"))?;

    match debug_signal {
        DebugSignal::Pause => {
            let pause_reason = match controller.last_pause_reason.as_str() {
                "breakpoint" => StoppedEventReason::Breakpoint,
                "step" => StoppedEventReason::Step,
                "pause" => StoppedEventReason::Pause,
                "exception" => StoppedEventReason::Exception,
                _ => StoppedEventReason::Step,
            };

            server.send_event(Event::Stopped(StoppedEventBody {
                reason: pause_reason,
                description: Some(format!("Execution paused ({})", controller.last_pause_reason)),
                thread_id: Some(1),
                preserve_focus_hint: Some(false),
                text: None,
                all_threads_stopped: Some(true),
                hit_breakpoint_ids: None,
            }))?;
        }
        DebugSignal::Continue => {
            if virtual_machine.halted || virtual_machine.call_frames.is_empty() {
                let exit_code = virtual_machine.exit_code.unwrap_or(0);
                server.send_event(Event::Terminated(None))?;
                server.send_event(Event::Exited(ExitedEventBody {
                    exit_code: i64::from(exit_code),
                }))?;
            }
        }
        DebugSignal::Terminate => {
            server.send_event(Event::Terminated(None))?;
            server.send_event(Event::Exited(ExitedEventBody { exit_code: 0 }))?;
        }
    }

    Ok(())
}

/// Sets up binary programs and runtime options for the given launch target.
fn setup_runtime_for_target(
    target_path: &Path,
) -> anyhow::Result<RuntimeOptions> {
    let mut runtime_options = RuntimeOptions::new()
        .with_heap_size(32 * 1024 * 1024)
        .with_max_call_stack_depth(1024)
        .with_native_function("println", builtin_println)
        .with_native_function("print", builtin_print)
        .with_native_function("blt::io::println", builtin_println)
        .with_native_function("blt::io::print", builtin_print)
        .with_native_function("Console::println", builtin_println)
        .with_native_function("Console::print", builtin_print);

    let extension = target_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if extension.eq_ignore_ascii_case("slx") {
        let source_content = fs::read_to_string(target_path)?;
        let file_stem = target_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("app");

        let compilation_options = CompilationOptions::new()
            .with_profile(solix_compilation::BuildProfile::Debug)
            .with_package_metadata(file_stem, "0.1.0", "")
            .with_builtins(true)
            .with_source(target_path.display().to_string(), source_content);

        let mut pipeline = CompilationPipeline::standard();
        let mut compilation_context = CompilationContext::new(compilation_options);
        pipeline.execute(&mut compilation_context).map_err(|diagnostics| {
            let error_details: Vec<String> = diagnostics
                .iter()
                .map(|diag| format!("{diag:?}"))
                .collect();
            anyhow::anyhow!("Compilation failed with {} diagnostics: {}", diagnostics.len(), error_details.join(", "))
        })?;

        let artifacts = compilation_context
            .emission_artifacts
            .take()
            .ok_or_else(|| anyhow::anyhow!("Bytecode emission phase produced no artifacts"))?;
        runtime_options = runtime_options.with_binary(artifacts.program);
    } else if extension.eq_ignore_ascii_case("slxarc") {
        let temp_dir = std::env::temp_dir();
        let loaded_archive = SlxarcArchive::extract_and_load(target_path, &temp_dir)?;
        for native_lib in loaded_archive.native_libraries {
            runtime_options = runtime_options.with_native_library(native_lib);
        }
        for binary_prog in loaded_archive.binaries {
            runtime_options = runtime_options.with_binary(binary_prog);
        }
        for secondary_prog in loaded_archive.secondary_binaries {
            runtime_options = runtime_options.with_binary(secondary_prog);
        }
    } else if extension.eq_ignore_ascii_case("slxb") {
        let binary_bytes = fs::read(target_path)?;
        let program = BinaryProgram::from_bytes(&binary_bytes)?;
        runtime_options = runtime_options.with_binary(program);
    }

    Ok(runtime_options)
}

/// Snaps a requested source coordinate to the nearest compiled function, instruction PC, and line.
fn resolve_breakpoint_location(
    source_path: &str,
    requested_line: u32,
    virtual_machine: &VirtualMachine,
) -> Option<(u32, u32, u32)> {
    let mut best_candidate: Option<(u32, u32, u32)> = None;

    for (function_index, compiled_function) in virtual_machine.context.all_functions.iter().enumerate() {
        let function_metadata = match virtual_machine.context.function_meta.get(function_index) {
            Some(metadata) => metadata,
            None => continue,
        };
        let loaded_binary = match virtual_machine.context.loaded_binaries.get(function_metadata.binary_id) {
            Some(binary) => binary,
            None => continue,
        };
        let function_source_file = match loaded_binary.source_files.get(compiled_function.source_id as usize) {
            Some(file_name) => file_name,
            None => continue,
        };

        let matches_file = function_source_file.ends_with(source_path)
            || source_path.ends_with(function_source_file);
        if !matches_file {
            continue;
        }

        for source_map_entry in &compiled_function.source_map {
            if source_map_entry.line >= requested_line {
                match best_candidate {
                    None => {
                        best_candidate = Some((
                            source_map_entry.line,
                            source_map_entry.instruction_pc,
                            function_index as u32,
                        ));
                    }
                    Some((best_line, _, _)) => {
                        if source_map_entry.line < best_line {
                            best_candidate = Some((
                                source_map_entry.line,
                                source_map_entry.instruction_pc,
                                function_index as u32,
                            ));
                        }
                    }
                }
                break;
            }
        }
    }

    best_candidate
}

/// Processes breakpoint registrations for a source document.
fn process_set_breakpoints(
    arguments: &SetBreakpointsArguments,
    virtual_machine: Option<&VirtualMachine>,
) -> (String, Vec<BreakpointEntry>) {
    let source_path = arguments
        .source
        .path
        .clone()
        .unwrap_or_else(|| arguments.source.name.clone().unwrap_or_default());

    let mut registered_breakpoints = Vec::new();
    if let Some(requested_breakpoints) = &arguments.breakpoints {
        for requested_breakpoint in requested_breakpoints {
            let requested_line = requested_breakpoint.line as u32;
            let mut snapped_pc = 0;
            let mut snapped_line = requested_line;
            let mut matched_function_id = None;

            if let Some(virtual_machine_ref) = virtual_machine {
                if let Some((candidate_line, candidate_pc, candidate_function_id)) =
                    resolve_breakpoint_location(&source_path, requested_line, virtual_machine_ref)
                {
                    snapped_line = candidate_line;
                    snapped_pc = candidate_pc;
                    matched_function_id = Some(candidate_function_id);
                }
            }

            registered_breakpoints.push(BreakpointEntry {
                source_path: source_path.clone(),
                requested_line,
                verified_line: snapped_line,
                instruction_pc: snapped_pc,
                function_id: matched_function_id,
                condition_expression: requested_breakpoint.condition.clone(),
                hit_condition_expression: requested_breakpoint.hit_condition.clone(),
                log_message_template: requested_breakpoint.log_message.clone(),
                current_hit_count: 0,
            });
        }
    }

    (source_path, registered_breakpoints)
}

/// Formats a runtime value for display in the DAP Variables and Watch panes.
fn format_runtime_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(bool_val) => bool_val.to_string(),
        Value::I32(int_val) => int_val.to_string(),
        Value::I64(long_val) => long_val.to_string(),
        Value::F64(float_val) => float_val.to_string(),
        Value::Ref(heap_addr) => format!("Object@0x{heap_addr:08X}"),
        Value::FnPtr(fn_idx) => format!("fn#{fn_idx}"),
    }
}

/// Parses a string literal into a typed runtime `Value`.
fn parse_literal_to_value(literal_text: &str) -> Value {
    let trimmed = literal_text.trim();
    if trimmed == "true" {
        Value::Bool(true)
    } else if trimmed == "false" {
        Value::Bool(false)
    } else if trimmed == "null" {
        Value::Null
    } else if let Ok(int_val) = trimmed.parse::<i32>() {
        Value::I32(int_val)
    } else if let Ok(long_val) = trimmed.parse::<i64>() {
        Value::I64(long_val)
    } else if let Ok(float_val) = trimmed.parse::<f64>() {
        Value::F64(float_val)
    } else {
        Value::I32(0)
    }
}

/// Extracts a string parameter from LaunchRequestArguments additional data.
fn extract_launch_string(arguments: &LaunchRequestArguments, key_name: &str) -> Option<String> {
    arguments
        .additional_data
        .as_ref()
        .and_then(|json_val| json_val.get(key_name))
        .and_then(|field_val| field_val.as_str())
        .map(|str_slice| str_slice.to_string())
}

/// Extracts a boolean parameter from LaunchRequestArguments additional data.
fn extract_launch_boolean(arguments: &LaunchRequestArguments, key_name: &str) -> Option<bool> {
    arguments
        .additional_data
        .as_ref()
        .and_then(|json_val| json_val.get(key_name))
        .and_then(|field_val| field_val.as_bool())
}
