//! Conformance Audit Regression Test Suite for Milestone 021.
//!
//! Validates end-to-end fixes for runtime memory safety, compiler code generation,
//! binder type checking, parser grammar extensions, and variadics.

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_source, run_binary};

#[test]
fn test_regression_modulo_compound_assignment_i32_and_i64() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int32 val_i32 = 17;
                val_i32 %= 5; // 2
                val_i32 += 8; // 10
                val_i32 -= 3; // 7
                val_i32 *= 4; // 28
                val_i32 /= 2; // 14

                int64 val_i64 = 55L;
                val_i64 %= 10L; // 5L
                val_i64 += 20L; // 25L
                val_i64 -= 5L;  // 20L
                val_i64 *= 2L;  // 40L
                val_i64 /= 4L;  // 10L

                if (val_i32 == 14 && val_i64 == 10L) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_int64_compound_assignments_and_increments() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int64 accumulator = 100L;
                int64 pre_inc = ++accumulator;
                int64 post_inc = accumulator++;
                int64 pre_dec = --accumulator;
                int64 post_dec = accumulator--;

                int64[] array_values = new int64[3];
                array_values[1] = 50L;
                array_values[1]++;
                ++array_values[1];

                if (pre_inc == 101L && post_inc == 101L && accumulator == 100L
                    && pre_dec == 101L && post_dec == 101L && array_values[1] == 52L) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_int64_large_positive_and_negative_in_tuples_arrays_closures() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int64 max_val = 0x7FFF_FFFF_FFFF_0000L;
                int64 min_val = -0x7FFF_FFFF_FFFF_0000L;

                int64[] numbers = new int64[2];
                numbers[0] = max_val;
                numbers[1] = min_val;

                tuple tuple_pair = tuple(max_val, min_val);
                int64 val_zero = cast(tuple_pair[0], int64);
                int64 val_one = cast(tuple_pair[1], int64);

                int64 captured = max_val;
                int64(*)() closure_fn = [captured]() => captured;
                int64 from_closure = closure_fn();

                if (numbers[0] == max_val && numbers[1] == min_val
                    && val_zero == max_val && val_one == min_val
                    && from_closure == max_val) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_try_finally_without_catch_propagates_exception_after_finally() {
    let source_code = r#"
        class Exception {}
        class CustomError extends Exception {}

        class Main {
            public static int32 main() {
                int32 side_effect = 0;
                int32 caught_error = 0;
                try {
                    try {
                        throw new CustomError();
                    } finally {
                        side_effect = 100;
                    }
                } catch (CustomError error_obj) {
                    caught_error = 42;
                }

                if (side_effect == 100 && caught_error == 42) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_destructor_stack_growth_does_not_invalidate_reg_base() {
    let source_code = r#"
        class Resource {
            public int32 payload;
            public Resource(int32 initial_payload) {
                this.payload = initial_payload;
            }
            ~Resource() {
                this.cleanup_helper(this.payload);
            }
            public int32 cleanup_helper(int32 value) {
                return value * 2;
            }
        }

        class Main {
            public static int32 main() {
                int32 local_preserve = 777;
                for (int32 iteration = 0; iteration < 20; iteration++) {
                    Resource resource_instance = new Resource(iteration);
                }
                if (local_preserve == 777) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_string_length_and_string_concatenation_plus() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                string hello = "Hello";
                string world = " World!";
                string combined = hello + world;
                int32 combined_len = combined.length;

                string mutable_str = "Start";
                mutable_str += "End";

                if (combined_len == 12 && mutable_str.length == 8) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_default_parameter_arguments_injected_at_call_site() {
    let source_code = r#"
        class Calculator {
            public static int32 compute(int32 base_val, int32 multiplier = 10, int32 offset = 5) {
                return base_val * multiplier + offset;
            }
        }

        class Main {
            public static int32 main() {
                int32 result_default_all = Calculator.compute(4); // 4 * 10 + 5 = 45
                int32 result_default_one = Calculator.compute(4, 3); // 4 * 3 + 5 = 17
                int32 result_explicit = Calculator.compute(4, 3, 2); // 4 * 3 + 2 = 14

                if (result_default_all == 45 && result_default_one == 17 && result_explicit == 14) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_super_method_invocation_in_derived_class() {
    let source_code = r#"
        class BaseEntity {
            public int32 base_multiplier;
            public BaseEntity(int32 multiplier) {
                this.base_multiplier = multiplier;
            }
            public virtual int32 calculate(int32 factor) {
                return this.base_multiplier * factor;
            }
        }

        class DerivedEntity extends BaseEntity {
            public int32 bonus;
            public DerivedEntity(int32 multiplier, int32 bonus_val) : super(multiplier) {
                this.bonus = bonus_val;
            }
            public override int32 calculate(int32 factor) {
                return super.calculate(factor) + this.bonus;
            }
        }

        class Main {
            public static int32 main() {
                DerivedEntity entity = new DerivedEntity(5, 3);
                int32 computation_result = entity.calculate(4); // 5 * 4 + 3 = 23
                if (computation_result == 23) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_numeric_literals_with_underscores_all_radices() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int32 decimal_val = 1_000_000;
                int32 hex_val = 0xAA_BB;
                int32 binary_val = 0b1010_1011;
                float64 float_val = 1_234.567_890;

                if (decimal_val == 1000000 && hex_val == 43707 && binary_val == 171 && float_val > 1234.56) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_generic_type_bounds_enforced_positive_and_negative() {
    let valid_code = r#"
        interface IPrintable {
            void print();
        }

        class Document implements IPrintable {
            public void print() {}
        }

        class Printer<T: IPrintable> {
            public void process(T item) {
                item.print();
            }
        }

        class Main {
            public static int32 main() {
                Printer<Document> printer_instance = new Printer<Document>();
                return 0;
            }
        }
    "#;
    let program = compile_source(valid_code);
    let (result, mut vm) = run_binary(program, Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());

    let invalid_code = r#"
        interface IPrintable {
            void print();
        }

        class NonPrintable {}

        class Printer<T: IPrintable> {}

        class Main {
            public static int32 main() {
                Printer<NonPrintable> printer_instance = new Printer<NonPrintable>();
                return 0;
            }
        }
    "#;
    assert_compile_error(invalid_code, "E0297");
}

#[test]
fn test_regression_export_manifest_interfaces_enums_and_operators_round_trip() {
    let source_code = r#"
        public interface IHandler {
            void handle();
        }

        public enum HttpStatus {
            Ok = 200,
            NotFound = 404
        }

        public class Vector2D {
            public int32 x_coord;
            public int32 y_coord;

            public Vector2D(int32 x_coord, int32 y_coord) {
                this.x_coord = x_coord;
                this.y_coord = y_coord;
            }

            public Vector2D operator+(Vector2D other) {
                return new Vector2D(this.x_coord + other.x_coord, this.y_coord + other.y_coord);
            }
        }

        public class Main {
            public static int32 main() {
                Vector2D first_vec = new Vector2D(1, 2);
                Vector2D second_vec = new Vector2D(3, 4);
                Vector2D sum_vec = first_vec + second_vec;
                if (sum_vec.x_coord == 4 && sum_vec.y_coord == 6) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program.clone(), Some("Main::main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());

    let manifest = program.export_manifest;
    assert!(
        manifest
            .exported_classes
            .iter()
            .any(|class_item| class_item.name == "IHandler" && class_item.is_interface)
    );
    assert!(
        manifest
            .exported_enums
            .iter()
            .any(|enum_item| enum_item.name == "HttpStatus")
    );
    let vector_class = manifest
        .exported_classes
        .iter()
        .find(|class_item| class_item.name == "Vector2D")
        .expect("Vector2D class must be exported");
    assert!(
        vector_class
            .methods
            .iter()
            .any(|method_item| method_item.is_operator)
    );
}

#[test]
fn test_case_55_5_and_55_6_variadic_function_templates() {
    let source_code = r#"
        int32 sum_all<args...>(int32 first, args...) {
            int32 total = first;
            for (int32 index = 0; index < args.length; index++) {
                total = total + cast(args[index], int32);
            }
            return total;
        }

        int32 count_args<args...>(int32 label, args...) {
            return args.length;
        }

        int32 main() {
            int32 calculated_sum = sum_all(10, 20, 30, 40);
            int32 item_count = count_args(3, 1, "text", true);

            if (calculated_sum == 100 && item_count == 3) {
                return 0;
            }
            return 1;
        }
    "#;
    let program = compile_source(source_code);
    let (result, mut vm) = run_binary(program, Some("main"));
    assert_eq!(result, Value::I32(0));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_regression_multi_binary_cast_check_and_interface_call_relocation() {
    let source_first = r#"
        class FirstModuleEntry {
            public int32 dummy_field;
            public FirstModuleEntry(int32 initial_val) {
                this.dummy_field = initial_val;
            }
        }
    "#;
    let binary_first = compile_source(source_first);

    let source_second = r#"
        interface IAuditable {
            int32 audit();
        }

        class ServiceAccount implements IAuditable {
            public int32 account_id;
            public ServiceAccount(int32 id_val) {
                this.account_id = id_val;
            }
            public int32 audit() {
                return this.account_id * 2;
            }
        }

        class SecondModuleMain {
            public static int32 main() {
                ServiceAccount account = new ServiceAccount(21);
                bool is_account = account instanceof ServiceAccount;
                ServiceAccount checked_account = cast(account, ServiceAccount);
                int32 audit_result = checked_account.audit();

                if (is_account && audit_result == 42) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let binary_second = compile_source(source_second);

    let options = solix_runtime::RuntimeOptions::new()
        .with_binary(binary_first)
        .with_binary(binary_second)
        .with_entry_point("SecondModuleMain::main");
    let context =
        solix_runtime::RuntimeContext::new(options).expect("RuntimeContext creation failed");
    let mut virtual_machine = solix_runtime::VirtualMachine::new(context);
    let execution_result = virtual_machine.run().expect("VM execution failed");
    assert_eq!(execution_result, Value::I32(0));
    assert!(virtual_machine.verify_clean_memory().is_ok());
}
