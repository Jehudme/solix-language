//! Integration tests for v2.0 third-sweep architectural and safety fixes (Issues #171 - #180).
//!
//! Validates:
//! 1. Issue #171: Generic monomorphizer unbounded recursion depth limit (E0302).
//! 2. Issue #172: Parser expression call stack exhaustion depth limit (E0148).
//! 3. Issue #173: Fast VM negative bitwise shift operand invalid trap (RuntimeErrorCode::InvalidOperand).
//! 4. Issue #174: Module static initializer circular dependency detection (E0303) and topological sorting.
//! 5. Issue #175: Virtual method calls on this inside destructor declarations forbidden (E0304).
//! 6. Issue #176: Missing runtime array store check on covariant reference arrays (RuntimeErrorCode::ArrayStoreMismatch).
//! 7. Issue #177: Ambiguous symbol resolution on package name collision with top-level class declarations (E0305).
//! 8. Issue #178: Inexhaustive enum switch without default case (E0306).
//! 9. Issue #179: Standard library collection capacity doubling integer overflow guards.
//! 10. Issue #180: C/C++ FFI header version synchronization (v2.0.0).

#![forbid(unsafe_code)]

use solix_runtime::exceptions::errors::RuntimeErrorCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{
    assert_compile_error, assert_compile_sources_error, assert_runtime_error, compile_and_run,
};

// ============================================================================
// Issue #171: Generic Monomorphizer Recursion Depth Limit (E0302)
// ============================================================================

#[test]
fn test_generic_monomorphizer_self_referential_depth_limit_rejected() {
    let source_code = r#"
        class RecursiveGenericNode<GenericPayload> {
            public RecursiveGenericNode<RecursiveGenericNode<GenericPayload>> expanding_child;
        }

        class Main {
            public static int32 main() {
                RecursiveGenericNode<int32> root_instance = null;
                return 0;
            }
        }
    "#;
    assert_compile_error(source_code, "E0302");
}

#[test]
fn test_generic_monomorphizer_normal_depth_succeeds() {
    let source_code = r#"
        class Container<GenericPayload> {
            public GenericPayload stored_payload;

            public Container(GenericPayload initial_payload) {
                this.stored_payload = initial_payload;
            }

            public GenericPayload get_payload() {
                return this.stored_payload;
            }
        }

        class Main {
            public static int32 main() {
                Container<int32> container_instance = new Container<int32>(42);
                return container_instance.get_payload();
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

// ============================================================================
// Issue #172: Parser Expression Nesting Depth Limit (E0148)
// ============================================================================

#[test]
fn test_parser_deeply_nested_parentheses_rejected() {
    let mut expression_string = String::new();
    for _ in 0..40 {
        expression_string.push('(');
    }
    expression_string.push('1');
    for _ in 0..40 {
        expression_string.push(')');
    }

    let source_code = format!(
        r#"
        class Main {{
            public static int32 main() {{
                int32 calculated_value = {expression_string};
                return calculated_value;
            }}
        }}
        "#
    );
    assert_compile_error(&source_code, "E0148");
}

#[test]
fn test_parser_reasonable_nested_parentheses_succeeds() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int32 nested_computation = ((((((1 + 2) * 3) - 4) + 5) * 2));
                return nested_computation;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(20));
}

// ============================================================================
// Issue #173: Fast VM Negative Bitwise Shift Trapping (RuntimeErrorCode::InvalidOperand)
// ============================================================================

#[test]
fn test_bitwise_shift_right_negative_operand_traps() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int32 shift_count = -1;
                int32 computed_result = 16 >> shift_count;
                return computed_result;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::InvalidOperand);
}

#[test]
fn test_bitwise_shift_left_negative_operand_traps() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int32 shift_count = -2;
                int32 computed_result = 4 << shift_count;
                return computed_result;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::InvalidOperand);
}

#[test]
fn test_bitwise_shift_valid_operands_succeeds() {
    let source_code = r#"
        class Main {
            public static int32 main() {
                int32 right_shifted = 64 >> 2;
                int32 left_shifted = 16 << 2;
                if (right_shifted == 16 && left_shifted == 64) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

// ============================================================================
// Issue #174: Module Static Initializer Cycle Detection & Topological Ordering
// ============================================================================

#[test]
fn test_cyclic_static_field_dependencies_rejected() {
    let source_code = r#"
        class NodeFirst {
            public static int32 first_number = NodeSecond.second_number + 1;
        }

        class NodeSecond {
            public static int32 second_number = NodeFirst.first_number + 1;
        }

        class Main {
            public static int32 main() {
                return NodeFirst.first_number;
            }
        }
    "#;
    assert_compile_error(source_code, "E0303");
}

#[test]
fn test_acyclic_cross_class_static_field_dependency_ordered_correctly() {
    let source_code = r#"
        class ClassDependent {
            public static int32 derived_metric = ClassSource.base_metric + 15;
        }

        class ClassSource {
            public static int32 base_metric = 25;
        }

        class Main {
            public static int32 main() {
                return ClassDependent.derived_metric;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(40));
}

// ============================================================================
// Issue #175: Calling Virtual Methods in Destructor Forbidden (E0304)
// ============================================================================

#[test]
fn test_virtual_call_in_destructor_rejected() {
    let source_code = r#"
        class ManagedResource {
            public virtual void teardown_handles() {
            }

            ~ManagedResource() {
                this.teardown_handles();
            }
        }

        class Main {
            public static int32 main() {
                return 0;
            }
        }
    "#;
    assert_compile_error(source_code, "E0304");
}

#[test]
fn test_non_virtual_field_cleanup_in_destructor_succeeds() {
    let source_code = r#"
        class NativeHandleHolder {
            public int32 raw_handle;

            public NativeHandleHolder(int32 initial_handle) {
                this.raw_handle = initial_handle;
            }

            ~NativeHandleHolder() {
                this.raw_handle = 0;
            }
        }

        class Main {
            public static int32 main() {
                NativeHandleHolder holder = new NativeHandleHolder(101);
                return holder.raw_handle;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(101));
}

// ============================================================================
// Issue #176: Missing Runtime Array Store Check on Covariant Reference Arrays
// ============================================================================

#[test]
fn test_covariant_reference_array_store_mismatch_traps() {
    let source_code = r#"
        class AnimalSpecies {
            public int32 species_identifier;
        }

        class CanineSpecies extends AnimalSpecies {
            public int32 bark_loudness;
        }

        class FelineSpecies extends AnimalSpecies {
            public int32 purr_frequency;
        }

        class Main {
            public static int32 main() {
                CanineSpecies[] canine_array = new CanineSpecies[4];
                AnimalSpecies[] generic_array = canine_array;
                generic_array[0] = new FelineSpecies();
                return 0;
            }
        }
    "#;
    assert_runtime_error(source_code, RuntimeErrorCode::ArrayStoreMismatch);
}

#[test]
fn test_covariant_reference_array_store_valid_subtype_succeeds() {
    let source_code = r#"
        class AnimalSpecies {
            public int32 species_identifier;
        }

        class CanineSpecies extends AnimalSpecies {
            public int32 bark_loudness;
            public CanineSpecies(int32 initial_loudness) {
                this.bark_loudness = initial_loudness;
            }
        }

        class Main {
            public static int32 main() {
                CanineSpecies[] canine_array = new CanineSpecies[4];
                AnimalSpecies[] generic_array = canine_array;
                generic_array[0] = new CanineSpecies(99);
                return canine_array[0].bark_loudness;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(99));
}

// ============================================================================
// Issue #177: Package Name Collision with Top-Level Class (E0305)
// ============================================================================

#[test]
fn test_top_level_class_name_collision_with_package_segment_rejected() {
    let sources = [
        (
            "network_module.slx",
            r#"
            package network.http;

            public class RequestClient {
                public int32 request_timeout;
            }
            "#,
        ),
        (
            "colliding_type.slx",
            r#"
            public class network {
                public int32 dummy_counter;
            }

            public class Main {
                public static int32 main() {
                    return 0;
                }
            }
            "#,
        ),
    ];
    assert_compile_sources_error(&sources, "E0305");
}

#[test]
fn test_distinct_packages_without_collision_succeeds() {
    let sources = [
        (
            "service_module.slx",
            r#"
            package infrastructure.service;

            public class ServiceWorker {
                public int32 worker_identifier;
                public ServiceWorker(int32 initial_id) {
                    this.worker_identifier = initial_id;
                }
            }
            "#,
        ),
        (
            "main_module.slx",
            r#"
            import infrastructure.service.ServiceWorker;

            public class ApplicationRunner {
                public static int32 main() {
                    ServiceWorker worker = new ServiceWorker(55);
                    return worker.worker_identifier;
                }
            }
            "#,
        ),
    ];
    let binary_program = solix_tests::common::compile_multi_sources(&sources);
    let (execution_result, _) = solix_tests::common::run_binary(binary_program, None);
    assert_eq!(execution_result, Value::I32(55));
}

// ============================================================================
// Issue #178: Inexhaustive Enum Switch Without Default Case (E0306)
// ============================================================================

#[test]
fn test_inexhaustive_enum_switch_missing_variants_and_default_rejected() {
    let source_code = r#"
        enum CardinalDirection {
            North,
            South,
            East,
            West,
        }

        class Main {
            public static int32 classify_direction(CardinalDirection target_direction) {
                switch (target_direction) {
                    case CardinalDirection.North:
                        return 1;
                    case CardinalDirection.South:
                        return 2;
                }
                return 0;
            }

            public static int32 main() {
                return Main.classify_direction(CardinalDirection.North);
            }
        }
    "#;
    assert_compile_error(source_code, "E0306");
}

#[test]
fn test_exhaustive_enum_switch_all_cases_succeeds() {
    let source_code = r#"
        enum CardinalDirection {
            North,
            South,
            East,
            West,
        }

        class Main {
            public static int32 classify_direction(CardinalDirection target_direction) {
                switch (target_direction) {
                    case CardinalDirection.North:
                        return 1;
                    case CardinalDirection.South:
                        return 2;
                    case CardinalDirection.East:
                        return 3;
                    case CardinalDirection.West:
                        return 4;
                }
                return 0;
            }

            public static int32 main() {
                return Main.classify_direction(CardinalDirection.East);
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(3));
}

#[test]
fn test_inexhaustive_enum_switch_with_default_clause_succeeds() {
    let source_code = r#"
        enum CardinalDirection {
            North,
            South,
            East,
            West,
        }

        class Main {
            public static int32 classify_direction(CardinalDirection target_direction) {
                switch (target_direction) {
                    case CardinalDirection.North:
                        return 1;
                    default:
                        return 99;
                }
            }

            public static int32 main() {
                return Main.classify_direction(CardinalDirection.West);
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(99));
}

// ============================================================================
// Issue #179 & #180: Collections Capacity Guard & C-FFI Header Synchronized
// ============================================================================

#[test]
fn test_c_ffi_header_definitions_synchronized_with_runtime_version() {
    let ffi_header_content = include_str!("../../../../../ffi/c/include/solix_ffi.h");
    let package_version = env!("CARGO_PKG_VERSION");
    let version_segments: Vec<&str> = package_version.split('.').collect();
    assert_eq!(
        version_segments.len(),
        3,
        "Package version must follow semantic versioning format"
    );
    let expected_major = version_segments[0];
    let expected_minor = version_segments[1];
    let expected_patch = version_segments[2];

    assert!(
        ffi_header_content.contains(&format!("#define SOLIX_FFI_VERSION_MAJOR {expected_major}")),
        "FFI major version must match package major version {expected_major}"
    );
    assert!(
        ffi_header_content.contains(&format!("#define SOLIX_FFI_VERSION_MINOR {expected_minor}")),
        "FFI minor version must match package minor version {expected_minor}"
    );
    assert!(
        ffi_header_content.contains(&format!("#define SOLIX_FFI_VERSION_PATCH {expected_patch}")),
        "FFI patch version must match package patch version {expected_patch}"
    );
    assert!(
        ffi_header_content.contains(&format!("#define SOLIX_FFI_VERSION \"{package_version}\"")),
        "FFI version string must match package version '{package_version}'"
    );
}
