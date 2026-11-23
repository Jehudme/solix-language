//! Verification and integration tests for Solix DAP features.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use solix_common::bytecode::BinaryProgram;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::pipeline::CompilationPipeline;
use solix_dap::controller::DapController;
use solix_dap::evaluator::{
    check_hit_condition, evaluate_expression_in_frame, interpolate_log_template,
};
use solix_dap::state::{
    BreakpointEntry, SteppingState, VariableReferenceManager, VariableScopeType,
};
use solix_runtime::native::builtins::{builtin_print, builtin_println};
use solix_runtime::value::Value;
use solix_runtime::vm::debug::DebugSignal;
use solix_runtime::{RuntimeContext, RuntimeOptions, VirtualMachine};

#[test]
fn test_variable_reference_manager_lifecycle() {
    let mut reference_manager = VariableReferenceManager::new();

    let first_ref = reference_manager.allocate_reference(VariableScopeType::FrameLocals(0));
    let second_ref = reference_manager.allocate_reference(VariableScopeType::GlobalStatics);
    let third_ref = reference_manager.allocate_reference(VariableScopeType::ObjectFields(42));

    assert_eq!(first_ref, 1000);
    assert_eq!(second_ref, 1001);
    assert_eq!(third_ref, 1002);

    assert_eq!(
        reference_manager.resolve_reference(first_ref),
        Some(&VariableScopeType::FrameLocals(0))
    );
    assert_eq!(
        reference_manager.resolve_reference(second_ref),
        Some(&VariableScopeType::GlobalStatics)
    );
    assert_eq!(
        reference_manager.resolve_reference(third_ref),
        Some(&VariableScopeType::ObjectFields(42))
    );

    reference_manager.reset_transient_references();
    assert_eq!(reference_manager.resolve_reference(first_ref), None);

    let reused_first_ref = reference_manager.allocate_reference(VariableScopeType::FrameLocals(1));
    assert_eq!(reused_first_ref, 1000);
}

#[test]
fn test_evaluator_literals_and_arithmetic() {
    let runtime_options = RuntimeOptions::new();
    let runtime_context = RuntimeContext::new(runtime_options).expect("RuntimeContext creation");
    let virtual_machine = VirtualMachine::new(runtime_context);

    // Literal evaluations
    let bool_true_result =
        evaluate_expression_in_frame("true", &virtual_machine, 0).expect("Evaluate true");
    assert_eq!(bool_true_result, Value::Bool(true));

    let bool_false_result =
        evaluate_expression_in_frame("false", &virtual_machine, 0).expect("Evaluate false");
    assert_eq!(bool_false_result, Value::Bool(false));

    let int_result = evaluate_expression_in_frame("42", &virtual_machine, 0).expect("Evaluate 42");
    assert_eq!(int_result, Value::I32(42));

    // Binary arithmetic
    let addition_result =
        evaluate_expression_in_frame("10 + 20", &virtual_machine, 0).expect("Evaluate 10 + 20");
    assert_eq!(addition_result, Value::I32(30));

    let multiplication_result =
        evaluate_expression_in_frame("5 * 8", &virtual_machine, 0).expect("Evaluate 5 * 8");
    assert_eq!(multiplication_result, Value::I32(40));

    // Binary comparison
    let comparison_equal_result =
        evaluate_expression_in_frame("10 == 10", &virtual_machine, 0).expect("Evaluate 10 == 10");
    assert_eq!(comparison_equal_result, Value::Bool(true));

    let comparison_less_result =
        evaluate_expression_in_frame("10 < 5", &virtual_machine, 0).expect("Evaluate 10 < 5");
    assert_eq!(comparison_less_result, Value::Bool(false));
}

#[test]
fn test_evaluator_float_and_mixed_arithmetic() {
    let runtime_options = RuntimeOptions::new();
    let runtime_context = RuntimeContext::new(runtime_options).expect("RuntimeContext creation");
    let virtual_machine = VirtualMachine::new(runtime_context);

    // Floating-point arithmetic
    let float_add_result =
        evaluate_expression_in_frame("1.5 + 2.5", &virtual_machine, 0).expect("Evaluate 1.5 + 2.5");
    assert_eq!(float_add_result, Value::F64(4.0));

    let float_mul_result = evaluate_expression_in_frame("10.0 * 0.5", &virtual_machine, 0)
        .expect("Evaluate 10.0 * 0.5");
    assert_eq!(float_mul_result, Value::F64(5.0));

    // Mixed numeric arithmetic
    let mixed_long_int_result =
        evaluate_expression_in_frame("10L + 5", &virtual_machine, 0).expect("Evaluate 10L + 5");
    assert_eq!(mixed_long_int_result, Value::I64(15));

    let mixed_float_int_result =
        evaluate_expression_in_frame("1.5 * 2", &virtual_machine, 0).expect("Evaluate 1.5 * 2");
    assert_eq!(mixed_float_int_result, Value::F64(3.0));

    // Mixed comparisons
    let mixed_compare_less =
        evaluate_expression_in_frame("10 < 20L", &virtual_machine, 0).expect("Evaluate 10 < 20L");
    assert_eq!(mixed_compare_less, Value::Bool(true));

    let mixed_compare_greater =
        evaluate_expression_in_frame("15.5 > 10", &virtual_machine, 0).expect("Evaluate 15.5 > 10");
    assert_eq!(mixed_compare_greater, Value::Bool(true));
}

#[test]
fn test_evaluator_logical_operators_and_compound_conditions() {
    let runtime_options = RuntimeOptions::new();
    let runtime_context = RuntimeContext::new(runtime_options).expect("RuntimeContext creation");
    let virtual_machine = VirtualMachine::new(runtime_context);

    // Direct logical conjunction (&&)
    let and_true_result = evaluate_expression_in_frame("true && true", &virtual_machine, 0)
        .expect("Evaluate true && true");
    assert_eq!(and_true_result, Value::Bool(true));

    let and_false_result = evaluate_expression_in_frame("true && false", &virtual_machine, 0)
        .expect("Evaluate true && false");
    assert_eq!(and_false_result, Value::Bool(false));

    // Direct logical disjunction (||)
    let or_true_result = evaluate_expression_in_frame("true || false", &virtual_machine, 0)
        .expect("Evaluate true || false");
    assert_eq!(or_true_result, Value::Bool(true));

    let or_false_result = evaluate_expression_in_frame("false || false", &virtual_machine, 0)
        .expect("Evaluate false || false");
    assert_eq!(or_false_result, Value::Bool(false));

    // Compound conditions (comparisons with logical operators)
    let compound_and_result =
        evaluate_expression_in_frame("10 > 5 && 20 < 30", &virtual_machine, 0)
            .expect("Evaluate 10 > 5 && 20 < 30");
    assert_eq!(compound_and_result, Value::Bool(true));

    let compound_or_result = evaluate_expression_in_frame("10 < 5 || 20 < 30", &virtual_machine, 0)
        .expect("Evaluate 10 < 5 || 20 < 30");
    assert_eq!(compound_or_result, Value::Bool(true));

    // Negative tests: non-boolean operands to logical operators
    let invalid_and_result = evaluate_expression_in_frame("10 && 20", &virtual_machine, 0);
    assert!(invalid_and_result.is_err());
    assert!(
        invalid_and_result
            .unwrap_err()
            .contains("requires boolean operands")
    );

    let invalid_or_result = evaluate_expression_in_frame("true || 5", &virtual_machine, 0);
    assert!(invalid_or_result.is_err());
    assert!(
        invalid_or_result
            .unwrap_err()
            .contains("requires boolean operands")
    );
}

#[test]
fn test_hit_condition_parser() {
    // Direct threshold
    assert!(check_hit_condition(5, "5"));
    assert!(check_hit_condition(10, "5"));
    assert!(!check_hit_condition(4, "5"));

    // Greater than or equal
    assert!(check_hit_condition(10, ">= 10"));
    assert!(check_hit_condition(15, ">= 10"));
    assert!(!check_hit_condition(9, ">= 10"));

    // Exact hit
    assert!(check_hit_condition(7, "== 7"));
    assert!(!check_hit_condition(8, "== 7"));

    // Modulo hits
    assert!(check_hit_condition(10, "% 5 == 0"));
    assert!(check_hit_condition(15, "% 5 == 0"));
    assert!(!check_hit_condition(12, "% 5 == 0"));
}

#[test]
fn test_logpoint_interpolation() {
    let runtime_options = RuntimeOptions::new();
    let runtime_context = RuntimeContext::new(runtime_options).expect("RuntimeContext creation");
    let virtual_machine = VirtualMachine::new(runtime_context);

    let template_text = "Iteration {10 + 20} reached with status {true}";
    let rendered_text = interpolate_log_template(template_text, &virtual_machine, 0);
    assert_eq!(
        rendered_text,
        "Iteration I32(30) reached with status Bool(true)"
    );
}

#[test]
fn test_controller_stepping_state_transitions() {
    let async_pause_flag = Arc::new(AtomicBool::new(false));
    let mut controller = DapController::new(async_pause_flag);

    assert_eq!(controller.stepping_state, SteppingState::None);

    controller.stepping_state = SteppingState::StepOver {
        start_call_depth: 2,
        start_file_path: "/src/main.slx".to_string(),
        start_line_number: 10,
    };

    match &controller.stepping_state {
        SteppingState::StepOver {
            start_call_depth,
            start_line_number,
            ..
        } => {
            assert_eq!(*start_call_depth, 2);
            assert_eq!(*start_line_number, 10);
        }
        _ => panic!("Expected StepOver state"),
    }
}

#[test]
fn test_exact_breakpoint_resolution_and_locals_inspection() {
    let source_path = PathBuf::from("tests/fixtures/dap_main.slx");
    let source_content = r#"package sample_workspace;

native void println(string msg);


/// Represents a 2D coordinate point with reference-counted heap allocation.
class Point {
    public int32 coordinate_x;
    public int32 coordinate_y;

    public Point(int32 initial_x, int32 initial_y) {
        this.coordinate_x = initial_x;
        this.coordinate_y = initial_y;
    }

    /// Returns the Manhattan distance from origin.
    public int32 distance_from_origin() {
        return this.coordinate_x + this.coordinate_y;
    }

    /// Deterministic destructor invoked upon scope exit.
    ~Point() {
        // Resource cleanup upon destruction
    }
}

/// Static mathematical utilities class.
class MathHelper {
    /// Computes the sum of two integers.
    public static int32 add(int32 operand_first, int32 operand_second) {
        return operand_first + operand_second;
    }
}

/// Unit test recognized by the Run Test CodeLens button.
void test_addition() {
    int32 computed_sum = MathHelper::add(15, 25);
    assert(computed_sum == 40, "Sum calculation mismatch");
}

/// Application entry point recognized by the Run Main CodeLens button.
void main() {
    Point origin_point = new Point(10, 20);
    int32 calculated_distance = origin_point.distance_from_origin();
    println("Calculated distance from origin: 30");

    int32 static_result = MathHelper::add(50, 50);
    println("Static addition result: 100");
}"#
    .to_string();

    let compilation_options = CompilationOptions::new()
        .with_profile(solix_compilation::BuildProfile::Debug)
        .with_package_metadata("sample_workspace", "0.1.0", "")
        .with_source(source_path.display().to_string(), source_content);

    let mut pipeline = CompilationPipeline::standard();
    let mut compilation_context = CompilationContext::new(compilation_options);
    pipeline
        .execute(&mut compilation_context)
        .expect("Compile main.slx");

    let artifacts = compilation_context
        .emission_artifacts
        .take()
        .expect("Emission artifacts");

    // Lossless serialization roundtrip through SLXB binary format
    let serialized_bytes = artifacts.program.to_bytes();
    let deserialized_program =
        BinaryProgram::from_bytes(&serialized_bytes).expect("Deserialize BinaryProgram");

    // Verify metadata survived binary serialization
    assert_eq!(deserialized_program.build_profile, "debug");
    assert!(!deserialized_program.classes.is_empty());

    let point_class = deserialized_program
        .classes
        .iter()
        .find(|candidate_class| candidate_class.symbol == "Point")
        .expect("Find Point class");
    assert_eq!(point_class.field_names.len(), 2);
    assert_eq!(point_class.field_names[0].1, "coordinate_x");
    assert_eq!(point_class.field_names[1].1, "coordinate_y");

    let target_instruction_pc = {
        let main_function = deserialized_program
            .functions
            .iter()
            .find(|candidate_func| {
                candidate_func.symbol == "main" || candidate_func.symbol.ends_with("::main")
            })
            .expect("Find main function");
        assert!(!main_function.source_map.is_empty());
        assert!(!main_function.local_variables.is_empty());
        main_function
            .source_map
            .iter()
            .find(|map_entry| map_entry.line == 47)
            .expect("Find line 47 in main function source map")
            .instruction_pc
    };

    // Configure runtime
    let runtime_options = RuntimeOptions::new()
        .with_heap_size(16 * 1024 * 1024)
        .with_native_function("println", builtin_println)
        .with_native_function("print", builtin_print)
        .with_binary(deserialized_program);

    let runtime_context = RuntimeContext::new(runtime_options).expect("RuntimeContext creation");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    // Prepare main entry frame
    let main_function_id = virtual_machine
        .context
        .entry_point_function_id
        .expect("Entry point function ID");
    let _ = virtual_machine.push_frame(main_function_id, 0, &[]);

    // Configure DAP controller with exact breakpoint on line 47 inside main()
    let async_pause_flag = Arc::new(AtomicBool::new(false));
    let mut controller = DapController::new(async_pause_flag);

    controller.line_breakpoints.push(BreakpointEntry {
        source_path: source_path.display().to_string(),
        requested_line: 47,
        verified_line: 47,
        instruction_pc: target_instruction_pc,
        function_id: Some(main_function_id),
        condition_expression: None,
        hit_condition_expression: None,
        log_message_template: None,
        current_hit_count: 0,
    });

    // Run until breakpoint
    let execution_signal = virtual_machine
        .run_debug(&mut controller)
        .expect("Run debug until breakpoint");

    assert_eq!(execution_signal, DebugSignal::Pause);
    assert_eq!(controller.last_pause_reason, "breakpoint");

    // Verify it paused inside main(), NOT in Point::distance_from_origin
    let active_frame = virtual_machine
        .call_frames
        .last()
        .expect("Active call frame");
    assert_eq!(active_frame.function_id, main_function_id);

    let (stopped_file, stopped_line) = DapController::resolve_source_location(
        &virtual_machine,
        active_frame.function_id,
        active_frame.pc,
    )
    .expect("Resolve source location");
    assert!(stopped_file.ends_with("main.slx"));
    assert_eq!(stopped_line, 47);

    // Inspect locals in frame 0: origin_point and calculated_distance must be visible
    let frame_locals = virtual_machine.inspect_frame_locals(0);
    assert!(!frame_locals.is_empty(), "Frame locals must not be empty");

    let origin_point_entry = frame_locals
        .iter()
        .find(|(var_name, _, _)| var_name == "origin_point")
        .expect("Locals must contain origin_point");
    let calculated_distance_entry = frame_locals
        .iter()
        .find(|(var_name, _, _)| var_name == "calculated_distance")
        .expect("Locals must contain calculated_distance");

    assert_eq!(calculated_distance_entry.2, Value::I32(30));

    // Inspect fields of origin_point: coordinate_x = 10, coordinate_y = 20
    if let Value::Ref(point_heap_address) = origin_point_entry.2 {
        assert!(point_heap_address > 0);
        let object_fields = virtual_machine
            .inspect_object_fields(point_heap_address)
            .expect("Inspect object fields");

        let coord_x_field = object_fields
            .iter()
            .find(|(field_name, _, _)| field_name == "coordinate_x")
            .expect("Find coordinate_x");
        let coord_y_field = object_fields
            .iter()
            .find(|(field_name, _, _)| field_name == "coordinate_y")
            .expect("Find coordinate_y");

        assert_eq!(coord_x_field.2, Value::I32(10));
        assert_eq!(coord_y_field.2, Value::I32(20));
    } else {
        panic!("origin_point is not a Value::Ref");
    }

    // Step over to line 48
    controller.stepping_state = SteppingState::StepOver {
        start_call_depth: virtual_machine.call_frames.len(),
        start_file_path: stopped_file,
        start_line_number: stopped_line,
    };

    let step_signal = virtual_machine
        .run_debug(&mut controller)
        .expect("Step over to line 48");
    assert_eq!(step_signal, DebugSignal::Pause);
    assert_eq!(controller.last_pause_reason, "step");

    let stepped_frame = virtual_machine
        .call_frames
        .last()
        .expect("Stepped call frame");
    let (_, stepped_line) = DapController::resolve_source_location(
        &virtual_machine,
        stepped_frame.function_id,
        stepped_frame.pc,
    )
    .expect("Resolve source location after step");
    assert_eq!(stepped_line, 48);

    // After step over, static_result should now be visible and equal to 100
    let updated_locals = virtual_machine.inspect_frame_locals(0);
    let static_result_entry = updated_locals
        .iter()
        .find(|(var_name, _, _)| var_name == "static_result")
        .expect("Locals must contain static_result after stepping line 47");
    assert_eq!(static_result_entry.2, Value::I32(100));
}
