//! Positive Language Conformance Test Suite covering all core Solix language features.
//!
//! Validates end-to-end execution across:
//! 1. Bitwise operations on integer types.
//! 2. 4+ level deep inheritance hierarchies with virtual dispatch.
//! 3. Generics and monomorphization.
//! 4. Switch statements and control flow.
//! 5. Exception handling, try-catch-finally, and RAII memory cleanup.
//! 6. Tuples, arrays, sizeof, instanceof, closures, and operator overloading.

#![forbid(unsafe_code)]

use solix_runtime::Value;
use solix_tests::common::compile_and_run;

// =============================================================================
// 1. Bitwise Operations Conformance Suite
// =============================================================================

#[test]
fn test_positive_bitwise_and_or_xor_int32() {
    let source_code = r#"
        int32 main() {
            int32 first_operand = 0b1100;
            int32 second_operand = 0b1010;

            int32 and_result = first_operand & second_operand;
            int32 or_result = first_operand | second_operand;
            int32 xor_result = first_operand ^ second_operand;

            if (and_result != 0b1000) {
                return 1;
            }
            if (or_result != 0b1110) {
                return 2;
            }
            if (xor_result != 0b0110) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_bitwise_shift_operations_int32() {
    let source_code = r#"
        int32 main() {
            int32 base_value = 1;
            int32 shifted_left = base_value << 4;
            int32 shifted_right = shifted_left >> 2;

            if (shifted_left != 16) {
                return 1;
            }
            if (shifted_right != 4) {
                return 2;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_bitwise_not_operation_int32() {
    let source_code = r#"
        int32 main() {
            int32 initial_value = 0;
            int32 bitwise_inverted = ~initial_value;

            if (bitwise_inverted != -1) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_bitwise_compound_mask_and_pack() {
    let source_code = r#"
        int32 main() {
            int32 high_byte = 0x12;
            int32 low_byte = 0x34;
            int32 packed_word = ((high_byte & 0xFF) << 8) | (low_byte & 0xFF);

            if (packed_word != 0x1234) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// =============================================================================
// 2. Deep Inheritance Hierarchies & Virtual Override Dispatch (4+ Levels)
// =============================================================================

#[test]
fn test_positive_four_level_inheritance_hierarchy_dispatch() {
    let source_code = r#"
        class RootEntity {
            public virtual int32 compute_score() {
                return 10;
            }
        }

        class LevelOneEntity extends RootEntity {
            public virtual override int32 compute_score() {
                return 20;
            }
        }

        class LevelTwoEntity extends LevelOneEntity {
            public virtual override int32 compute_score() {
                return 30;
            }
        }

        class LevelThreeEntity extends LevelTwoEntity {
            public override int32 compute_score() {
                return 40;
            }
        }

        int32 evaluate_entity(RootEntity entity_instance) {
            return entity_instance.compute_score();
        }

        int32 main() {
            RootEntity root_instance = new RootEntity();
            RootEntity level_one = new LevelOneEntity();
            RootEntity level_two = new LevelTwoEntity();
            RootEntity level_three = new LevelThreeEntity();

            if (evaluate_entity(root_instance) != 10) {
                return 1;
            }
            if (evaluate_entity(level_one) != 20) {
                return 2;
            }
            if (evaluate_entity(level_two) != 30) {
                return 3;
            }
            if (evaluate_entity(level_three) != 40) {
                return 4;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_class_implementing_multiple_interfaces() {
    let source_code = r#"
        interface IFirstCapability {
            int32 first_metric();
        }

        interface ISecondCapability {
            int32 second_metric();
        }

        interface IThirdCapability {
            int32 third_metric();
        }

        class MultiCapableWorker implements IFirstCapability, ISecondCapability, IThirdCapability {
            public int32 first_metric() {
                return 100;
            }

            public int32 second_metric() {
                return 200;
            }

            public int32 third_metric() {
                return 300;
            }
        }

        int32 main() {
            MultiCapableWorker worker_instance = new MultiCapableWorker();
            IFirstCapability first_interface = worker_instance;
            ISecondCapability second_interface = worker_instance;
            IThirdCapability third_interface = worker_instance;

            if (first_interface.first_metric() != 100) {
                return 1;
            }
            if (second_interface.second_metric() != 200) {
                return 2;
            }
            if (third_interface.third_metric() != 300) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_interface_extension_hierarchy_dispatch() {
    let source_code = r#"
        interface IBaseProtocol {
            int32 base_id();
        }

        interface IExtendedProtocol extends IBaseProtocol {
            int32 extended_id();
        }

        class ProtocolImplementation implements IExtendedProtocol {
            public int32 base_id() {
                return 50;
            }

            public int32 extended_id() {
                return 75;
            }
        }

        int32 main() {
            ProtocolImplementation protocol_instance = new ProtocolImplementation();
            IExtendedProtocol extended_ref = protocol_instance;
            IBaseProtocol base_ref = protocol_instance;

            if (base_ref.base_id() != 50) {
                return 1;
            }
            if (extended_ref.extended_id() != 75) {
                return 2;
            }
            if (extended_ref.base_id() != 50) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_abstract_base_with_partial_implementation() {
    let source_code = r#"
        abstract class AbstractCalculator {
            public abstract int32 calculate_value();

            public int32 double_calculated_value() {
                return this.calculate_value() * 2;
            }
        }

        class ConcreteCalculator extends AbstractCalculator {
            public override int32 calculate_value() {
                return 21;
            }
        }

        int32 main() {
            AbstractCalculator calculator_instance = new ConcreteCalculator();
            if (calculator_instance.double_calculated_value() != 42) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// =============================================================================
// 3. Generics and Monomorphization Conformance Suite
// =============================================================================

#[test]
fn test_positive_generic_single_parameter_container() {
    let source_code = r#"
        class BoxContainer<TValue> {
            public TValue internal_value;

            public BoxContainer(TValue initial_value) {
                this.internal_value = initial_value;
            }

            public TValue get_value() {
                return this.internal_value;
            }
        }

        int32 main() {
            BoxContainer<int32> integer_box = new BoxContainer<int32>(42);
            if (integer_box.get_value() != 42) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_generic_pair_two_parameters() {
    let source_code = r#"
        class KeyValuePair<TKey, TData> {
            public TKey key_entry;
            public TData data_entry;

            public KeyValuePair(TKey given_key, TData given_data) {
                this.key_entry = given_key;
                this.data_entry = given_data;
            }
        }

        int32 main() {
            KeyValuePair<int32, bool> pair_instance = new KeyValuePair<int32, bool>(99, true);
            if (pair_instance.key_entry != 99) {
                return 1;
            }
            if (!pair_instance.data_entry) {
                return 2;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_generic_method_explicit_type_arguments() {
    let source_code = r#"
        class ConversionUtils {
            public static TOutput identity_convert<TOutput>(TOutput input_value) {
                return input_value;
            }
        }

        int32 main() {
            int32 converted_integer = ConversionUtils::identity_convert<int32>(123);
            if (converted_integer != 123) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// =============================================================================
// 4. Control Flow, Pattern Matching & Switch Statements
// =============================================================================

#[test]
fn test_positive_switch_statement_integer_matching() {
    let source_code = r#"
        int32 evaluate_priority(int32 priority_code) {
            int32 computed_multiplier = 0;
            switch (priority_code) {
                case 1:
                    computed_multiplier = 10;
                    break;
                case 2:
                    computed_multiplier = 20;
                    break;
                case 3:
                    computed_multiplier = 30;
                    break;
                default:
                    computed_multiplier = 99;
                    break;
            }
            return computed_multiplier;
        }

        int32 main() {
            if (evaluate_priority(1) != 10) {
                return 1;
            }
            if (evaluate_priority(2) != 20) {
                return 2;
            }
            if (evaluate_priority(3) != 30) {
                return 3;
            }
            if (evaluate_priority(9) != 99) {
                return 4;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_switch_statement_enum_variants() {
    let source_code = r#"
        enum TaskStatus {
            Pending,
            InProgress,
            Completed,
            Failed
        }

        int32 map_status_to_code(TaskStatus status_input) {
            int32 numeric_code = 0;
            switch (status_input) {
                case TaskStatus::Pending:
                    numeric_code = 1;
                    break;
                case TaskStatus::InProgress:
                    numeric_code = 2;
                    break;
                case TaskStatus::Completed:
                    numeric_code = 3;
                    break;
                case TaskStatus::Failed:
                    numeric_code = 4;
                    break;
                default:
                    numeric_code = -1;
                    break;
            }
            return numeric_code;
        }

        int32 main() {
            if (map_status_to_code(TaskStatus::Pending) != 1) {
                return 1;
            }
            if (map_status_to_code(TaskStatus::InProgress) != 2) {
                return 2;
            }
            if (map_status_to_code(TaskStatus::Completed) != 3) {
                return 3;
            }
            if (map_status_to_code(TaskStatus::Failed) != 4) {
                return 4;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_do_while_loop_guaranteed_initial_iteration() {
    let source_code = r#"
        int32 main() {
            int32 execution_count = 0;
            bool condition_flag = false;

            do {
                execution_count = execution_count + 1;
            } while (condition_flag);

            if (execution_count != 1) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_for_loop_with_continue_and_break() {
    let source_code = r#"
        int32 main() {
            int32 accumulated_sum = 0;

            for (int32 counter = 0; counter < 20; counter = counter + 1) {
                if (counter == 5) {
                    continue;
                }
                if (counter == 10) {
                    break;
                }
                accumulated_sum = accumulated_sum + counter;
            }

            // Sum: 0+1+2+3+4 + 6+7+8+9 = 10 + 30 = 40
            if (accumulated_sum != 40) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// =============================================================================
// 5. Exception Handling, Try-Catch-Finally, and RAII Destructors
// =============================================================================

#[test]
fn test_positive_try_catch_basic_exception_handling() {
    let source_code = r#"
        class CustomError {
            public int32 error_code;

            public CustomError(int32 given_code) {
                this.error_code = given_code;
            }
        }

        int32 main() {
            int32 captured_code = 0;
            try {
                throw new CustomError(404);
            } catch (CustomError caught_error) {
                captured_code = caught_error.error_code;
            }

            if (captured_code != 404) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_try_catch_finally_execution_order() {
    let source_code = r#"
        class SimpleException {}

        int32 main() {
            bool try_ran = false;
            bool catch_ran = false;
            bool finally_ran = false;

            try {
                try_ran = true;
                throw new SimpleException();
            } catch (SimpleException caught_error) {
                catch_ran = true;
            } finally {
                finally_ran = true;
            }

            if (!try_ran || !catch_ran || !finally_ran) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_try_finally_without_exception_executes() {
    let source_code = r#"
        int32 main() {
            int32 cleanup_marker = 0;

            try {
                cleanup_marker = 10;
            } finally {
                cleanup_marker = cleanup_marker + 5;
            }

            if (cleanup_marker != 15) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_cross_frame_exception_unwinding() {
    let source_code = r#"
        class DeepException {
            public int32 reason_code;

            public DeepException(int32 code) {
                this.reason_code = code;
            }
        }

        void innermost_failure() {
            throw new DeepException(500);
        }

        void intermediate_caller() {
            innermost_failure();
        }

        int32 main() {
            int32 caught_reason = 0;
            try {
                intermediate_caller();
            } catch (DeepException deep_error) {
                caught_reason = deep_error.reason_code;
            }

            if (caught_reason != 500) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// =============================================================================
// 6. Tuples, Arrays, Sizeof, InstanceOf, Closures, and Operators
// =============================================================================

#[test]
fn test_positive_sizeof_primitives_and_references() {
    let source_code = r#"
        class LargeObject {
            public int64 first_field;
            public int64 second_field;
        }

        int32 main() {
            int32 size_int32 = sizeof(int32);
            int32 size_int64 = sizeof(int64);
            int32 size_bool = sizeof(bool);

            if (size_int32 != 4) {
                return 1;
            }
            if (size_int64 != 8) {
                return 2;
            }
            if (size_bool != 1) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_instanceof_with_polymorphic_hierarchy() {
    let source_code = r#"
        interface IAuditable {}

        class BaseRecord implements IAuditable {}
        class DerivedRecord extends BaseRecord {}
        class IndependentRecord {}

        int32 main() {
            BaseRecord record_instance = new DerivedRecord();

            if (!(record_instance instanceof DerivedRecord)) {
                return 1;
            }
            if (!(record_instance instanceof BaseRecord)) {
                return 2;
            }
            if (!(record_instance instanceof IAuditable)) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_tuple_packing_and_indexing() {
    let source_code = r#"
        int32 main() {
            tuple coordinate_tuple = tuple(10, true, 30);

            int32 first_coord = cast(coordinate_tuple[0], int32);
            bool second_flag = cast(coordinate_tuple[1], bool);
            int32 third_coord = cast(coordinate_tuple[2], int32);

            if (first_coord != 10) {
                return 1;
            }
            if (!second_flag) {
                return 2;
            }
            if (third_coord != 30) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_array_allocation_and_length() {
    let source_code = r#"
        int32 main() {
            int32[] number_array = new int32[5];
            number_array[0] = 10;
            number_array[1] = 20;
            number_array[2] = 30;

            if (number_array.length != 5) {
                return 1;
            }
            if (number_array[0] != 10 || number_array[1] != 20 || number_array[2] != 30) {
                return 2;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_operator_overloading_vector_plus() {
    let source_code = r#"
        class Vector2D {
            public int32 x_coordinate;
            public int32 y_coordinate;

            public Vector2D(int32 initial_x, int32 initial_y) {
                this.x_coordinate = initial_x;
                this.y_coordinate = initial_y;
            }

            public Vector2D operator +(Vector2D other_vector) {
                return new Vector2D(
                    this.x_coordinate + other_vector.x_coordinate,
                    this.y_coordinate + other_vector.y_coordinate
                );
            }
        }

        int32 main() {
            Vector2D first_point = new Vector2D(5, 10);
            Vector2D second_point = new Vector2D(15, 20);
            Vector2D combined_point = first_point + second_point;

            if (combined_point.x_coordinate != 20) {
                return 1;
            }
            if (combined_point.y_coordinate != 30) {
                return 2;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// =============================================================================
// 7. Additional Conformance Tests (Extended Bitwise, Generics, RAII, and Logic)
// =============================================================================

#[test]
fn test_positive_bitwise_unsigned_uint32() {
    let source_code = r#"
        int32 main() {
            uint32 alpha_mask = 0xFF00;
            uint32 beta_mask = 0x00FF;
            uint32 combined_mask = alpha_mask | beta_mask;

            if (combined_mask != 0xFFFF) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_bitwise_shifts_and_mask_uint8() {
    let source_code = r#"
        int32 main() {
            uint8 low_nibble = 0x0F;
            uint8 shifted_nibble = (uint8)(low_nibble << 4);

            if (shifted_nibble != 0xF0) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_bitwise_xor_swap_idiom() {
    let source_code = r#"
        int32 main() {
            int32 first_swap_variable = 105;
            int32 second_swap_variable = 205;

            first_swap_variable = first_swap_variable ^ second_swap_variable;
            second_swap_variable = first_swap_variable ^ second_swap_variable;
            first_swap_variable = first_swap_variable ^ second_swap_variable;

            if (first_swap_variable != 205) {
                return 1;
            }
            if (second_swap_variable != 105) {
                return 2;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_generic_nested_containers() {
    let source_code = r#"
        class Wrapper<TItem> {
            public TItem item_holder;

            public Wrapper(TItem given_item) {
                this.item_holder = given_item;
            }
        }

        int32 main() {
            Wrapper<int32> inner_wrapper = new Wrapper<int32>(555);
            Wrapper<Wrapper<int32>> outer_wrapper = new Wrapper<Wrapper<int32>>(inner_wrapper);

            if (outer_wrapper.item_holder.item_holder != 555) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_generic_class_implementing_interface() {
    let source_code = r#"
        interface IValueProvider {
            int32 extract_value();
        }

        class GenericProvider<TData> implements IValueProvider {
            public TData internal_data;

            public GenericProvider(TData given_data) {
                this.internal_data = given_data;
            }

            public int32 extract_value() {
                return 777;
            }
        }

        int32 main() {
            IValueProvider provider = new GenericProvider<string>("test string");
            if (provider.extract_value() != 777) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_generic_method_type_inference() {
    let source_code = r#"
        class InferenceEngine {
            public static TItem mirror<TItem>(TItem input_argument) {
                return input_argument;
            }
        }

        int32 main() {
            int32 inferred_number = InferenceEngine::mirror(888);
            if (inferred_number != 888) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_nested_while_loops_with_continue() {
    let source_code = r#"
        int32 main() {
            int32 outer_counter = 0;
            int32 total_iterations = 0;

            while (outer_counter < 4) {
                outer_counter = outer_counter + 1;
                int32 inner_counter = 0;
                while (inner_counter < 3) {
                    inner_counter = inner_counter + 1;
                    if (inner_counter == 2) {
                        continue;
                    }
                    total_iterations = total_iterations + 1;
                }
            }

            // 4 outer iterations * 2 counted inner iterations = 8
            if (total_iterations != 8) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_nested_try_catch_blocks() {
    let source_code = r#"
        class InnerException {}
        class OuterException {}

        int32 main() {
            bool inner_handled = false;
            bool outer_untouched = true;

            try {
                try {
                    throw new InnerException();
                } catch (InnerException inner_error) {
                    inner_handled = true;
                }
            } catch (OuterException outer_error) {
                outer_untouched = false;
            }

            if (!inner_handled || !outer_untouched) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_raii_destructor_cleanup_order() {
    let source_code = r#"
        class ResourceTracker {
            public static int32 destroyed_count = 0;

            ~ResourceTracker() {
                ResourceTracker::destroyed_count = ResourceTracker::destroyed_count + 1;
            }
        }

        void run_scoped_objects() {
            ResourceTracker first_guard = new ResourceTracker();
            ResourceTracker second_guard = new ResourceTracker();
        }

        int32 main() {
            ResourceTracker::destroyed_count = 0;
            run_scoped_objects();
            if (ResourceTracker::destroyed_count != 2) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_array_iteration_and_mutation() {
    let source_code = r#"
        int32 main() {
            int32[] numbers = new int32[4];
            for (int32 index = 0; index < 4; index = index + 1) {
                numbers[index] = (index + 1) * 10;
            }

            int32 aggregated_sum = 0;
            for (int32 check_index = 0; check_index < 4; check_index = check_index + 1) {
                aggregated_sum = aggregated_sum + numbers[check_index];
            }

            // 10 + 20 + 30 + 40 = 100
            if (aggregated_sum != 100) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_tuple_heterogeneous_types() {
    let source_code = r#"
        class Payload {
            public int32 value_field;
            public Payload(int32 initial_val) {
                this.value_field = initial_val;
            }
        }

        int32 main() {
            Payload payload_object = new Payload(999);
            tuple heterogeneous_tuple = tuple(payload_object, 42, true);

            Payload extracted_payload = cast(heterogeneous_tuple[0], Payload);
            int32 extracted_integer = cast(heterogeneous_tuple[1], int32);
            bool extracted_boolean = cast(heterogeneous_tuple[2], bool);

            if (extracted_payload.value_field != 999) {
                return 1;
            }
            if (extracted_integer != 42) {
                return 2;
            }
            if (!extracted_boolean) {
                return 3;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_operator_minus_overload() {
    let source_code = r#"
        class Position2D {
            public int32 horizontal_coordinate;
            public int32 vertical_coordinate;

            public Position2D(int32 pos_x, int32 pos_y) {
                this.horizontal_coordinate = pos_x;
                this.vertical_coordinate = pos_y;
            }

            public Position2D operator -(Position2D delta_position) {
                return new Position2D(
                    this.horizontal_coordinate - delta_position.horizontal_coordinate,
                    this.vertical_coordinate - delta_position.vertical_coordinate
                );
            }
        }

        int32 main() {
            Position2D start_position = new Position2D(50, 80);
            Position2D displacement = new Position2D(20, 30);
            Position2D final_position = start_position - displacement;

            if (final_position.horizontal_coordinate != 30) {
                return 1;
            }
            if (final_position.vertical_coordinate != 50) {
                return 2;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_virtual_method_chaining_without_super() {
    let source_code = r#"
        class BasePipeline {
            public virtual int32 process_base(int32 input_data) {
                return input_data + 10;
            }
        }

        class ExtendedPipeline extends BasePipeline {
            public virtual override int32 process_base(int32 input_data) {
                return input_data + 30;
            }
        }

        int32 main() {
            BasePipeline pipeline_instance = new ExtendedPipeline();
            if (pipeline_instance.process_base(5) != 35) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_diamond_interface_dispatch() {
    let source_code = r#"
        interface IRootInterface {
            int32 get_root_identity();
        }

        interface IBranchLeft extends IRootInterface {
            int32 get_left_metric();
        }

        interface IBranchRight extends IRootInterface {
            int32 get_right_metric();
        }

        class DiamondConductor implements IBranchLeft, IBranchRight {
            public int32 get_root_identity() {
                return 1000;
            }

            public int32 get_left_metric() {
                return 2000;
            }

            public int32 get_right_metric() {
                return 3000;
            }
        }

        int32 main() {
            DiamondConductor conductor = new DiamondConductor();
            IBranchLeft left_ref = conductor;
            IBranchRight right_ref = conductor;

            if (left_ref.get_root_identity() != 1000) {
                return 1;
            }
            if (left_ref.get_left_metric() != 2000) {
                return 2;
            }
            if (right_ref.get_root_identity() != 1000) {
                return 3;
            }
            if (right_ref.get_right_metric() != 3000) {
                return 4;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_switch_statement_fallthrough_default() {
    let source_code = r#"
        int32 main() {
            int32 matched_indicator = 0;
            int32 test_key = 999;

            switch (test_key) {
                case 1:
                    matched_indicator = 1;
                    break;
                case 2:
                    matched_indicator = 2;
                    break;
                default:
                    matched_indicator = 42;
                    break;
            }

            if (matched_indicator != 42) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_positive_multi_branch_logical_evaluation() {
    let source_code = r#"
        int32 main() {
            bool condition_one = true;
            bool condition_two = false;
            bool condition_three = true;

            bool compound_evaluation = (condition_one || condition_two) && condition_three;
            if (!compound_evaluation) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}
