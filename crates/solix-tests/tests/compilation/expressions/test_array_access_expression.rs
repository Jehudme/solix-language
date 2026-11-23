//! ArrayAccessExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.1).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_3_1_in_bounds_read_and_write() {
    let code = r#"
        int32 main() {
            int32[] buffer = new int32[3];
            buffer[0] = 100;
            buffer[1] = 200;
            int32 sum = buffer[0] + buffer[1];
            return sum;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(300));
}

#[test]
fn test_case_3_2_multi_dimensional_array_access() {
    let code = r#"
        int32 main() {
            int32[][] grid = new int32[][2];
            grid[0] = new int32[2];
            grid[0][1] = 42;
            return grid[0][1] == 42 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_array_slot_mutation_arc_deallocation() {
    let code = r#"
        class Item {
            public int32 id;
            public Item(int32 id) { this.id = id; }
        }

        int32 main() {
            Item[] arr = new Item[1];
            for (int32 i = 0; i < 5; i++) {
                arr[0] = new Item(i);
            }
            Item it = arr[0];
            return it.id == 4 ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_index_out_of_bounds_runtime_fault() {
    let code = r#"
        int32 main() {
            int32[] data = new int32[2];
            int32 fail = data[5];
            return fail;
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::IndexOutOfBounds);
}

#[test]
fn test_case_4_2_non_integer_array_subscript() {
    // E0203: Array index must be int32
    let code = r#"
        void test(int32[] arr) {
            int32 v = arr["key"];
        }
    "#;
    assert_compile_error(code, "Array index must be int32");
}

#[test]
fn test_case_4_4_subscript_access_on_null_reference_at_runtime() {
    let code = r#"
        int32 main() {
            int32[] arr = null;
            int32 v = arr[0];
            return 0;
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::NullReferenceAccess);
}

#[test]
fn test_case_4_5_tuple_index_out_of_bounds_compile_time_error() {
    let source_code = r#"
        void test_method() {
            (int32, string) tuple_value = (10, "test");
            int32 invalid_element = tuple_value[5];
        }
    "#;
    assert_compile_error(source_code, "E0252");
}
