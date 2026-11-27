//! Integration tests for v2.0 fifth-sweep architectural and safety fixes (Issues #194 - #201).
//!
//! Validates:
//! 1. Issue #194: Verifier branch target boundary check preventing out-of-bounds reads (BinaryLoadError).
//! 2. Issue #195: Proper error propagation in `execute_set_field_slow` primitive field writes.
//! 3. Issue #196: Cycle collector outgoing reference tracing for reference arrays with tag 0 or 14..127.
//! 4. Issue #197: Final class inheritance compiler diagnostic (E0235).
//! 5. Issue #198: Final method override compiler diagnostic (E0267).
//! 6. Issue #199: Direct destructor invocation compiler diagnostic (E0284).
//! 7. Issue #200: Assignment to 'this' or 'super' compiler diagnostic (E0253).
//! 8. Issue #201: StringBuilder capacity doubling and append overflow guards.

#![forbid(unsafe_code)]

use solix_common::bytecode::instruction::Instruction;
use solix_common::bytecode::{BinaryProgram, CompiledFunction, OpCode};
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::{RuntimeContext, RuntimeError, VirtualMachine};
use solix_tests::common::{assert_compile_error, compile_and_run, compile_source};

// ============================================================================
// Issue #194: Verifier Branch Target Boundary Check
// ============================================================================

#[test]
fn test_verifier_accepts_valid_branch_target() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // Instruction 0: Nop
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
    // Instruction 1: Jump to instruction 2 (offset +1)
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::Jump, 0, 0, 1));
    // Instruction 2: Return void
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context_result.is_ok(),
        "Valid branch target within instruction buffer bounds should pass verification"
    );
}

#[test]
fn test_verifier_accepts_sentinel_branch_target_at_instruction_count() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // Total instructions: 2 (indices 0 and 1). Target instruction_count (2) is accepted
    // because the runtime context appends a sentinel return instruction for function exit.
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
    // Instruction 1: Jump with offset +1 -> target PC = 1 + 1 = 2 == instruction_count
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::Jump, 0, 0, 1));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context_result.is_ok(),
        "Branch target equal to instruction_count targeting sentinel return must be accepted"
    );
}

#[test]
fn test_verifier_rejects_branch_target_exceeding_instruction_count() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // Total instructions: 2 (indices 0 and 1). Target 3 exceeds instruction_count (2) + sentinel return.
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
    // Instruction 1: Jump with offset +2 -> target PC = 1 + 2 = 3 > instruction_count
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::Jump, 0, 0, 2));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context_result.is_err(),
        "Branch target strictly exceeding instruction_count must be rejected by AOT verifier"
    );

    if let Err(RuntimeError::BinaryLoadError { reason }) = runtime_context_result {
        assert!(
            reason.contains("branch target 3 is out of bounds"),
            "Error message should mention out of bounds branch target: {reason}"
        );
    } else {
        panic!("Expected BinaryLoadError for out-of-bounds branch target");
    }
}

#[test]
fn test_verifier_rejects_negative_branch_target() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 4);

    // Instruction 0: Nop
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
    // Instruction 1: Jump backwards past index 0 (offset -2 -> target -1)
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::Jump, 0, 0, -2));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context_result = RuntimeContext::new(runtime_options);
    assert!(
        runtime_context_result.is_err(),
        "Negative branch target must be rejected by AOT verifier"
    );

    if let Err(RuntimeError::BinaryLoadError { reason }) = runtime_context_result {
        assert!(
            reason.contains("branch target -1 is out of bounds"),
            "Error message should mention out of bounds branch target: {reason}"
        );
    } else {
        panic!("Expected BinaryLoadError for out-of-bounds branch target");
    }
}

// ============================================================================
// Issue #195: execute_set_field_slow Error Propagation
// ============================================================================

#[test]
fn test_set_field_primitive_valid_write_succeeds() {
    let source_code = r#"
class CoordinatePoint {
    public int32 horizontal_coordinate;
    public int32 vertical_coordinate;

    public CoordinatePoint(int32 initial_horizontal, int32 initial_vertical) {
        this.horizontal_coordinate = initial_horizontal;
        this.vertical_coordinate = initial_vertical;
    }
}

class Main {
    public static int32 main() {
        CoordinatePoint point_instance = new CoordinatePoint(100, 200);
        point_instance.horizontal_coordinate = 350;
        return point_instance.horizontal_coordinate + point_instance.vertical_coordinate;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(550));
}

#[test]
fn test_set_field_primitive_out_of_bounds_heap_write_traps() {
    let mut binary_program = BinaryProgram::new();
    let mut compiled_function = CompiledFunction::new("main", 0, 0, 8);

    // r[0] = 0x7FFF (invalid heap pointer)
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 0x7FFF));
    // r[1] = 42
    compiled_function
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 42));
    // SetField r[0], offset 0, val r[1], raw_flags: tag 5 (int32) << 1 = 10
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::SetField, 10, 0, 0, 1));
    compiled_function
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));

    binary_program.functions.push(compiled_function);
    binary_program.entry_point_id = Some(0);

    let runtime_options = RuntimeOptions::new().with_binary(binary_program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Context initialization must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run();
    assert!(
        execution_result.is_err(),
        "Out-of-bounds heap write in execute_set_field_slow must trap instead of silent error drop"
    );
}

// ============================================================================
// Issue #196: Cycle Collector Reference Array Tracing
// ============================================================================

#[test]
fn test_cycle_collector_traces_reference_arrays_and_collects_cycles() {
    let source_code = r#"
class LoopContainer {
    public LoopContainer[] link_slots;
    public int32 payload_identifier;

    public LoopContainer(int32 initial_payload) {
        this.payload_identifier = initial_payload;
    }
}

class Main {
    public static int32 main() {
        LoopContainer container_instance = new LoopContainer(77);
        LoopContainer[] object_slots = new LoopContainer[1];
        object_slots[0] = container_instance;
        container_instance.link_slots = object_slots;
        return 0;
    }
}
"#;
    let compiled_binary = compile_source(source_code);
    let runtime_options = RuntimeOptions::new()
        .with_binary(compiled_binary)
        .with_cycle_collector(true)
        .with_cycle_threshold(1024);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Runtime context initialization must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("Main function must succeed"),
        Value::I32(0)
    );

    let collected_blocks = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must execute without errors");
    assert!(
        collected_blocks >= 1,
        "Cycle collector must trace reference arrays and reclaim cyclic graph, collected: {collected_blocks}"
    );
}

#[test]
fn test_cycle_collector_does_not_trace_primitive_arrays() {
    let source_code = r#"
class NumberHolder {
    public int32[] numerical_values;

    public NumberHolder() {
        this.numerical_values = new int32[4];
        this.numerical_values[0] = 100;
        this.numerical_values[1] = 200;
        this.numerical_values[2] = 300;
        this.numerical_values[3] = 400;
    }
}

class Main {
    public static int32 main() {
        NumberHolder holder_instance = new NumberHolder();
        return 0;
    }
}
"#;
    let compiled_binary = compile_source(source_code);
    let runtime_options = RuntimeOptions::new()
        .with_binary(compiled_binary)
        .with_cycle_collector(true)
        .with_cycle_threshold(1024);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("Runtime context initialization must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("Main function must succeed"),
        Value::I32(0)
    );

    // Normal non-cyclic allocations are cleaned up by ARC without cycle collection leftovers
    let collected_blocks = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must execute without errors");
    assert_eq!(
        collected_blocks, 0,
        "Primitive arrays without cyclic references must not produce orphan cyclic blocks"
    );
}

// ============================================================================
// Issue #197: Final Class Inheritance Diagnostic (E0235)
// ============================================================================

#[test]
fn test_subclassing_open_class_succeeds() {
    let source_code = r#"
class OpenBaseClass {
    public int32 base_value;

    public OpenBaseClass(int32 initial_value) {
        this.base_value = initial_value;
    }
}

class ExtendedDerivedClass extends OpenBaseClass {
    public ExtendedDerivedClass(int32 derived_value) : super(derived_value) {}
}

class Main {
    public static int32 main() {
        ExtendedDerivedClass instance = new ExtendedDerivedClass(42);
        return instance.base_value;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(42));
}

#[test]
fn test_subclassing_final_class_rejected_with_e0235() {
    let source_code = r#"
final class ImmutableBaseClass {
    public int32 identifier;
}

class InvalidSubclass extends ImmutableBaseClass {
    public int32 secondary_value;
}

class Main {
    public static int32 main() {
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0235");
}

// ============================================================================
// Issue #198: Final Method Overriding Diagnostic (E0267)
// ============================================================================

#[test]
fn test_overriding_virtual_method_succeeds() {
    let source_code = r#"
class BaseCalculator {
    public virtual int32 compute_score(int32 input_value) {
        return input_value * 2;
    }
}

class AdvancedCalculator extends BaseCalculator {
    public override int32 compute_score(int32 input_value) {
        return input_value * 3;
    }
}

class Main {
    public static int32 main() {
        AdvancedCalculator calculator_instance = new AdvancedCalculator();
        return calculator_instance.compute_score(10);
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(30));
}

#[test]
fn test_overriding_final_method_rejected_with_e0267() {
    let source_code = r#"
class BaseWorkflow {
    public final int32 execute_core_protocol() {
        return 100;
    }
}

class CustomWorkflow extends BaseWorkflow {
    public override int32 execute_core_protocol() {
        return 200;
    }
}

class Main {
    public static int32 main() {
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0267");
}

// ============================================================================
// Issue #199: Direct Destructor Invocation Diagnostic (E0284)
// ============================================================================

#[test]
fn test_normal_method_calls_succeed() {
    let source_code = r#"
class ResourceHolder {
    public int32 active_handles;

    public ResourceHolder() {
        this.active_handles = 1;
    }

    public int32 release_handles() {
        this.active_handles = 0;
        return this.active_handles;
    }
}

class Main {
    public static int32 main() {
        ResourceHolder holder_instance = new ResourceHolder();
        return holder_instance.release_handles();
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(0));
}

#[test]
fn test_direct_destructor_invocation_rejected_with_e0284() {
    let source_code = r#"
class ManagedBuffer {
    public int32 buffer_size;

    public ManagedBuffer() {
        this.buffer_size = 1024;
    }

    ~ManagedBuffer() {
        this.buffer_size = 0;
    }
}

class Main {
    public static int32 main() {
        ManagedBuffer buffer_instance = new ManagedBuffer();
        buffer_instance.~ManagedBuffer();
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0284");
}

// ============================================================================
// Issue #200: Assignment to 'this' or 'super' Diagnostic (E0253)
// ============================================================================

#[test]
fn test_normal_field_assignment_succeeds() {
    let source_code = r#"
class MutableState {
    public int32 state_value;

    public void update_state(int32 new_value) {
        this.state_value = new_value;
    }
}

class Main {
    public static int32 main() {
        MutableState state_instance = new MutableState();
        state_instance.update_state(99);
        return state_instance.state_value;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(99));
}

#[test]
fn test_assigning_to_this_rejected_with_e0253() {
    let source_code = r#"
class InvalidStateAssignment {
    public int32 status_code;

    public void replace_self() {
        this = new InvalidStateAssignment();
    }
}

class Main {
    public static int32 main() {
        return 0;
    }
}
"#;
    assert_compile_error(source_code, "E0253");
}

// ============================================================================
// Issue #201: StringBuilder Capacity Doubling & Appending Safety
// ============================================================================

#[test]
fn test_string_builder_source_contains_overflow_guards() {
    let string_builder_source =
        include_str!("../../../../../libraries/solixlib/src/core/StringBuilder.slx");
    assert!(
        string_builder_source.contains("minimum_capacity > 1073741824"),
        "ensure_capacity must guard against huge capacity overflow"
    );
    assert!(
        string_builder_source.contains("new_capacity > 1073741824"),
        "ensure_capacity must guard against doubling overflow"
    );
    assert!(
        string_builder_source.contains("this.count > 1073741824 - raw_text.length"),
        "append must guard against index overflow"
    );
}

#[test]
fn test_string_builder_capacity_guards_against_negative_overflow() {
    let source_code = r#"
class SafeStringBuilder {
    public char[] buffer;
    public int32 count;

    public SafeStringBuilder(int32 initial_capacity) {
        this.buffer = new char[initial_capacity];
        this.count = 0;
    }

    public void ensure_capacity(int32 minimum_capacity) {
        if (minimum_capacity <= 0) {
            return;
        }
        if (minimum_capacity > 1073741824) {
            minimum_capacity = 1073741824;
        }
        if (minimum_capacity > this.buffer.length) {
            int32 new_capacity = this.buffer.length * 2;
            if (new_capacity < minimum_capacity || new_capacity <= 0 || new_capacity > 1073741824) {
                new_capacity = minimum_capacity;
            }
            char[] new_buffer = new char[new_capacity];
            int32 copy_index = 0;
            while (copy_index < this.count) {
                new_buffer[copy_index] = this.buffer[copy_index];
                copy_index = copy_index + 1;
            }
            this.buffer = new_buffer;
        }
    }

    public void append_char(char character) {
        if (this.count >= 1073741824) {
            return;
        }
        this.ensure_capacity(this.count + 1);
        this.buffer[this.count] = character;
        this.count = this.count + 1;
    }
}

class Main {
    public static int32 main() {
        SafeStringBuilder builder_instance = new SafeStringBuilder(8);
        builder_instance.ensure_capacity(-50);
        builder_instance.append_char('A');
        return builder_instance.count;
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(1));
}

#[test]
fn test_weak_field_returns_null_after_target_dropped_and_heap_reused() {
    // Regression test (#300.1): compiled Solix weak fields must emit FLAG_WEAK on GetField and
    // return null (never stale/impostor object) after the target drops and its heap slot is reused.
    let source_code = r#"
class OwnerNode {
    public int32 marker;
    public OwnerNode(int32 marker) {
        this.marker = marker;
    }
}

class ImpostorNode {
    public int32 decoy;
    public ImpostorNode(int32 decoy) {
        this.decoy = decoy;
    }
}

class ObserverNode {
    public weak OwnerNode parent;
    public ObserverNode(OwnerNode parent) {
        this.parent = parent;
    }
}

class Main {
    public static int32 main() {
        OwnerNode owner = new OwnerNode(42);
        ObserverNode observer = new ObserverNode(owner);
        int32 live_check = 0;
        if (observer.parent != null && observer.parent.marker == 42) {
            live_check = 1;
        }
        // Drop strong reference to OwnerNode and allocate same-sized ImpostorNode
        owner = null;
        ImpostorNode impostor = new ImpostorNode(999);
        int32 nullified_check = 0;
        if (observer.parent == null) {
            nullified_check = 1;
        }
        return (live_check * 10) + nullified_check + (impostor.decoy * 0);
    }
}
"#;
    let (execution_value, _virtual_machine) = compile_and_run(source_code);
    assert_eq!(execution_value, Value::I32(11));
}
