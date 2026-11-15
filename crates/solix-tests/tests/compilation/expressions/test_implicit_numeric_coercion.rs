//! Implicit Numeric Literal Coercion & Safe Widening Conformance Tests (MS-037).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_ms037_local_variable_declaration_coercion() {
    let source_code = r#"
        int32 main() {
            uint8 unsigned_byte = 255;
            int8 signed_byte = -128;
            uint16 unsigned_short = 65535;
            int16 signed_short = -32768;
            int64 signed_large = 100;
            float32 single_float = 3.5;
            float64 double_float = 42;

            if (unsigned_byte != 255) return 1;
            if (signed_byte != -128) return 2;
            if (unsigned_short != 65535) return 3;
            if (signed_short != -32768) return 4;
            if (signed_large != 100) return 5;
            if (single_float < 3.4 || single_float > 3.6) return 6;
            if (double_float < 41.9 || double_float > 42.1) return 7;

            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_ms037_constructor_and_method_argument_coercion() {
    let source_code = r#"
        class ColorRgba {
            public uint8 red_channel;
            public uint8 green_channel;
            public uint8 blue_channel;
            public uint8 alpha_channel;

            public ColorRgba(uint8 red_val, uint8 green_val, uint8 blue_val, uint8 alpha_val) {
                this.red_channel = red_val;
                this.green_channel = green_val;
                this.blue_channel = blue_val;
                this.alpha_channel = alpha_val;
            }

            public bool matches(uint8 expected_red, uint8 expected_alpha) {
                return (this.red_channel == expected_red && this.alpha_channel == expected_alpha);
            }
        }

        int32 main() {
            ColorRgba primary_color = new ColorRgba(18, 20, 30, 255);
            if (!primary_color.matches(18, 255)) {
                return 1;
            }
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_ms037_method_return_value_coercion() {
    let source_code = r#"
        class Calculator {
            public static float32 compute_ratio() {
                return 10.5;
            }

            public static int64 compute_zero() {
                return 0;
            }
        }

        int32 main() {
            float32 ratio_val = Calculator::compute_ratio();
            int64 zero_val = Calculator::compute_zero();
            if (ratio_val < 10.4 || ratio_val > 10.6) return 1;
            if (zero_val != 0) return 2;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_ms037_assignment_expression_coercion() {
    let source_code = r#"
        int32 main() {
            float32 float_slot = 0.0;
            uint8 byte_slot = 0;

            float_slot = 50.25;
            byte_slot = 200;

            if (float_slot < 50.2 || float_slot > 50.3) return 1;
            if (byte_slot != 200) return 2;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_ms037_array_literal_coercion() {
    let source_code = r#"
        int32 main() {
            float32[] float_list = [1.0, 2.5, 3.0];
            int64[] integer_list = [10, 20, 30];

            if (float_list[0] < 0.9 || float_list[0] > 1.1) return 1;
            if (float_list[1] < 2.4 || float_list[1] > 2.6) return 2;
            if (integer_list[2] != 30) return 3;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_ms037_binary_arithmetic_literal_coercion() {
    let source_code = r#"
        int32 main() {
            float32 base_position = 100.0;
            float32 offset_position = base_position + 24.5;
            float32 doubled_position = base_position * 2.0;

            if (offset_position < 124.4 || offset_position > 124.6) return 1;
            if (doubled_position < 199.9 || doubled_position > 200.1) return 2;

            uint8 byte_level = 100;
            if (byte_level != 100) return 3;

            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_ms037_out_of_bounds_integer_literal_rejection() {
    let code_unsigned_overflow = r#"
        void test_method() {
            uint8 invalid_byte = 256;
        }
    "#;
    assert_compile_error(code_unsigned_overflow, "E0203");

    let code_signed_overflow = r#"
        void test_method() {
            int8 invalid_byte = 128;
        }
    "#;
    assert_compile_error(code_signed_overflow, "E0203");

    let code_negative_unsigned = r#"
        void test_method() {
            uint8 invalid_byte = -1;
        }
    "#;
    assert_compile_error(code_negative_unsigned, "E0203");
}

#[test]
fn test_case_ms037_narrowing_variable_rejection_without_explicit_cast() {
    let code_narrowing = r#"
        void test_method() {
            int32 source_int = 42;
            uint8 destination_byte = source_int;
        }
    "#;
    assert_compile_error(code_narrowing, "E0203");
}
