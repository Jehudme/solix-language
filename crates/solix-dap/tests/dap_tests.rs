//! Verification and integration tests for Solix DAP features.

#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use solix_dap::controller::DapController;
use solix_dap::evaluator::{
    check_hit_condition, evaluate_expression_in_frame, interpolate_log_template,
};
use solix_dap::state::{
    SteppingState, VariableReferenceManager, VariableScopeType,
};
use solix_runtime::value::Value;
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

    let reused_first_ref =
        reference_manager.allocate_reference(VariableScopeType::FrameLocals(1));
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

    let int_result =
        evaluate_expression_in_frame("42", &virtual_machine, 0).expect("Evaluate 42");
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
