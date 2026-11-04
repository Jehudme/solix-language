//! Suite 53 & Suite 54: Function Declaration Scope Rules and Entry Point Resolution (TESTS.md)

#![forbid(unsafe_code)]

use solix_runtime::Value;
use solix_tests::common::{
    assert_compile_error, assert_compile_sources_error, compile_and_run, compile_and_run_multi,
};

#[test]
fn test_case_53_1_free_function_entry_point_without_static() {
    let source_code = r#"
        int32 helper(int32 value) {
            return value * 2;
        }

        int32 main() {
            return helper(21) == 42 ? 0 : 1;
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_53_2_rejection_of_static_on_free_function_outside_class() {
    let source_code = r#"
        static int32 compute() {
            return 10;
        }

        int32 main() {
            return compute();
        }
    "#;
    assert_compile_error(source_code, "E0174");
}

#[test]
fn test_case_54_1_single_unambiguous_entry_point_execution() {
    let service_source = r#"
        package app.service;

        public class Service {
            public static int32 run_service() {
                return 42;
            }
        }
    "#;
    let main_source = r#"
        package app;
        import app.service.*;

        int32 main() {
            return Service.run_service() == 42 ? 0 : 1;
        }
    "#;
    let sources = vec![("service.slx", service_source), ("main.slx", main_source)];
    let (execution_result, _) = compile_and_run_multi(&sources);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_54_2_ambiguous_entry_point_detection_across_modules() {
    let source_alpha = r#"
        package app.alpha;

        int32 main() {
            return 1;
        }
    "#;
    let source_beta = r#"
        package app.beta;

        int32 main() {
            return 2;
        }
    "#;
    let sources = vec![("mod_a.slx", source_alpha), ("mod_b.slx", source_beta)];
    assert_compile_sources_error(&sources, "E0244");
}
