//! Expanded Switch Statement Conformance & Rejection Tests (Milestone 054).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_switch_all_primitives_numeric_and_string() {
    let source_code = r#"
        int32 test_string(string text) {
            switch (text) {
                case "apple": return 1;
                case "banana": return 2;
                case "cherry": return 3;
                default: return 0;
            }
        }

        int32 test_int64(int64 large_value) {
            switch (large_value) {
                case 10000000000: return 10;
                case 20000000000: return 20;
                default: return 0;
            }
        }

        int32 test_float64(float64 decimal_value) {
            switch (decimal_value) {
                case 3.14: return 314;
                case 2.718: return 271;
                default: return 0;
            }
        }

        int32 test_bool(bool flag) {
            switch (flag) {
                case true: return 100;
                case false: return 200;
                default: return 0;
            }
        }

        int32 main() {
            if (test_string("banana") != 2) return 1;
            if (test_string("unknown") != 0) return 2;
            if (test_int64(10000000000) != 10) return 3;
            if (test_float64(3.14) != 314) return 4;
            if (test_bool(true) != 100) return 5;
            if (test_bool(false) != 200) return 6;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_enum_variants_conformance() {
    let source_code = r#"
        enum TrafficLight { Red, Yellow, Green }

        int32 evaluate_action(TrafficLight signal) {
            switch (signal) {
                case TrafficLight.Red: return 1;
                case TrafficLight.Yellow: return 2;
                case TrafficLight.Green: return 3;
                default: return 0;
            }
        }

        int32 main() {
            if (evaluate_action(TrafficLight.Red) != 1) return 1;
            if (evaluate_action(TrafficLight.Yellow) != 2) return 2;
            if (evaluate_action(TrafficLight.Green) != 3) return 3;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_strings_hash_dispatch_sixteen_keys() {
    let source_code = r#"
        int32 classify_greek_letter(string key_name) {
            switch (key_name) {
                case "alpha": return 1;
                case "beta": return 2;
                case "gamma": return 3;
                case "delta": return 4;
                case "epsilon": return 5;
                case "zeta": return 6;
                case "eta": return 7;
                case "theta": return 8;
                case "iota": return 9;
                case "kappa": return 10;
                case "lambda": return 11;
                case "mu": return 12;
                case "nu": return 13;
                case "xi": return 14;
                case "omicron": return 15;
                case "pi": return 16;
                default: return 0;
            }
        }

        int32 main() {
            if (classify_greek_letter("alpha") != 1) return 1;
            if (classify_greek_letter("theta") != 8) return 2;
            if (classify_greek_letter("pi") != 16) return 3;
            if (classify_greek_letter("omega") != 0) return 4;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_class_overloaded_equality() {
    let source_code = r#"
        class Vector2D {
            public int32 x_coord;
            public int32 y_coord;

            public Vector2D(int32 x_coord, int32 y_coord) {
                this.x_coord = x_coord;
                this.y_coord = y_coord;
            }

            public bool operator ==(Vector2D other_vector) {
                return this.x_coord == other_vector.x_coord && this.y_coord == other_vector.y_coord;
            }
        }

        int32 match_vector(Vector2D target_vec) {
            Vector2D origin_vec = new Vector2D(0, 0);
            Vector2D unit_x_vec = new Vector2D(1, 0);
            Vector2D unit_y_vec = new Vector2D(0, 1);

            switch (target_vec) {
                case origin_vec: return 10;
                case unit_x_vec: return 20;
                case unit_y_vec: return 30;
                default: return 99;
            }
        }

        int32 main() {
            Vector2D test_point = new Vector2D(1, 0);
            if (match_vector(test_point) != 20) return 1;
            Vector2D outside_point = new Vector2D(5, 5);
            if (match_vector(outside_point) != 99) return 2;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_tuple_equality_dispatch() {
    let source_code = r#"
        int32 classify_command((int32, string) command_pair) {
            switch (command_pair) {
                case (1, "start"): return 100;
                case (2, "stop"): return 200;
                case (3, "pause"): return 300;
                default: return 0;
            }
        }

        int32 main() {
            (int32, string) first_pair = (1, "start");
            (int32, string) second_pair = (2, "stop");
            (int32, string) third_pair = (9, "unknown");

            if (classify_command(first_pair) != 100) return 1;
            if (classify_command(second_pair) != 200) return 2;
            if (classify_command(third_pair) != 0) return 3;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_instanceof_class_polymorphic() {
    let source_code = r#"
        class Shape {}
        class Circle extends Shape {
            public int32 radius;
            public Circle(int32 radius) { this.radius = radius; }
        }
        class Square extends Shape {
            public int32 side;
            public Square(int32 side) { this.side = side; }
        }

        int32 classify_shape(Shape target_shape) {
            switch (target_shape) {
                case instanceof Circle: return 1;
                case instanceof Square: return 2;
                default: return 0;
            }
        }

        int32 main() {
            Shape test_circle = new Circle(10);
            Shape test_square = new Square(20);
            Shape base_shape = new Shape();

            if (classify_shape(test_circle) != 1) return 1;
            if (classify_shape(test_square) != 2) return 2;
            if (classify_shape(base_shape) != 0) return 3;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_instanceof_tuple_pattern() {
    let source_code = r#"
        int32 classify_tuple((int32, string) tuple_arg) {
            switch (tuple_arg) {
                case instanceof (int32, string): return 42;
                default: return 0;
            }
        }

        int32 main() {
            (int32, string) my_pair = (1, "hello");
            if (classify_tuple(my_pair) != 42) return 1;
            return 0;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_switch_class_without_equality_rejected_e0292() {
    let source_code = r#"
        class PlainData {
            public int32 value_field;
        }

        void test_missing_operator(PlainData first_item, PlainData second_item) {
            switch (first_item) {
                case second_item: break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0292");
}

#[test]
fn test_switch_incompatible_case_type_rejected_e0293() {
    let source_code = r#"
        void test_type_incompatibility(int32 numeric_argument) {
            switch (numeric_argument) {
                case "invalid_string": break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0293");
}

#[test]
fn test_switch_duplicate_case_rejected_e0294() {
    let source_code = r#"
        void test_duplicate_case(int32 numeric_argument) {
            switch (numeric_argument) {
                case 42: break;
                case 42: break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0294");
}

#[test]
fn test_switch_invalid_instanceof_pattern_target_rejected_e0295() {
    let source_code = r#"
        class SampleObject {}

        void test_invalid_target(SampleObject candidate_object) {
            switch (candidate_object) {
                case instanceof int32: break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0295");
}

#[test]
fn test_switch_tuple_arity_mismatch_rejected_e0296() {
    let source_code = r#"
        void test_tuple_arity((int32, string) input_tuple) {
            switch (input_tuple) {
                case (1, "start", true): break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0296");
}
