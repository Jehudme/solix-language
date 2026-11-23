//! ArrayCreationExpression Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.2).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_3_1_zero_initialized_array() {
    let code = r#"
        int32 main() {
            int32[] data = new int32[5];
            return data[0];
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_reference_type_array_allocation() {
    let code = r#"
        class Item {
            public int32 id;
        }

        int32 main() {
            Item[] items = new Item[3];
            return items[0] == null ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_negative_size_at_runtime() {
    let code = r#"
        int32 main() {
            int32[] bad = new int32[-1];
            return 0;
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::OutOfMemory);
}

#[test]
fn test_case_4_2_array_creation_with_missing_size() {
    // E0105: Missing array size expression
    let code = r#"
        void test() {
            int32[] arr = new int32[]();
        }
    "#;
    assert_compile_error(code, "size");
}

#[test]
fn test_case_4_3_array_creation_with_non_integer_size() {
    let source_code = r#"
        void test_method() {
            int32[] invalid_array = new int32["five"];
        }
    "#;
    assert_compile_error(source_code, "E0250");
}
