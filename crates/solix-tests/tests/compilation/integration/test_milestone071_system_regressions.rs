//! Milestone 071 System-Wide Regression Fortress Tests.
//!
//! Validates:
//! 1. ARC Optimization: zero leaks and zero double-frees (`active_guest_objects == 0`) across
//!    conditional branch diamonds, loop breaks, early returns, and thrown exception unwinding.
//! 2. Cross-Package Generic Monomorphization: distinct generic class instantiations where
//!    identical class names exist in distinct external packages.
//! 3. Cross-Namespace Method Inlining: inlining small methods across distinct imported namespaces
//!    without symbol collisions.

#![forbid(unsafe_code)]

use solix_common::bytecode::extract_api_manifest;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::manifest_to_source;
use solix_runtime::context::RuntimeContext;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::vm::VirtualMachine;

#[test]
fn test_arc_regression_branch_diamonds_loop_breaks_and_exception_unwinding() {
    let source_code = r#"
        class PayloadNode {
            public int32 payload_value;
            public PayloadNode next_node;

            public PayloadNode(int32 payload_value) {
                this.payload_value = payload_value;
                this.next_node = null;
            }
        }

        class UnwindException {}

        class ArcTortureRunner {
            public static void frame_three(PayloadNode root_node, bool should_throw) {
                PayloadNode third_node = new PayloadNode(300);
                root_node.next_node = third_node;
                if (should_throw) {
                    throw new UnwindException();
                }
            }

            public static void frame_two(PayloadNode root_node, bool should_throw) {
                PayloadNode second_node = new PayloadNode(200);
                second_node.next_node = root_node;
                ArcTortureRunner::frame_three(second_node, should_throw);
            }

            public static int32 run_diamond_and_loops(int32 selector) {
                // 1. Conditional branch diamond
                PayloadNode candidate_first = new PayloadNode(10);
                PayloadNode candidate_second = new PayloadNode(20);
                PayloadNode active_node = null;

                if (selector > 5) {
                    active_node = candidate_first;
                } else {
                    active_node = candidate_second;
                }

                // 2. Loop breaks with temporary object creation
                int32 loop_counter = 0;
                while (loop_counter < 10) {
                    PayloadNode iteration_node = new PayloadNode(loop_counter);
                    if (loop_counter == 5) {
                        break;
                    }
                    loop_counter = loop_counter + 1;
                }

                // 3. Exception unwinding across 3 call frames
                int32 caught_status = 0;
                try {
                    PayloadNode root_node = new PayloadNode(100);
                    ArcTortureRunner::frame_two(root_node, true);
                } catch (UnwindException caught_error) {
                    caught_status = 77;
                }

                // 4. Early return inside nested block
                if (selector == 999) {
                    PayloadNode early_node = new PayloadNode(999);
                    return early_node.payload_value;
                }

                return active_node.payload_value + caught_status;
            }
        }

        class Main {
            public static int32 main() {
                int32 result_first = ArcTortureRunner::run_diamond_and_loops(10); // 10 + 77 = 87
                int32 result_second = ArcTortureRunner::run_diamond_and_loops(2);  // 20 + 77 = 97
                int32 result_third = ArcTortureRunner::run_diamond_and_loops(999); // 999

                if (result_first == 87 && result_second == 97 && result_third == 999) {
                    return 0;
                }
                return 1;
            }
        }
    "#;

    let options = CompilationOptions::new().with_source("arc_torture.slx", source_code);
    let mut context = CompilationContext::new(options);
    context
        .execute()
        .expect("Compilation of arc torture suite failed");

    let artifacts = context
        .emission_artifacts
        .expect("Artifacts must be present");
    let runtime_options = RuntimeOptions::new().with_binary(artifacts.program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run().expect("VM execution failed");
    assert_eq!(execution_result, Value::I32(0));
    assert_eq!(
        virtual_machine.active_guest_objects(),
        0,
        "Active guest objects must be exactly 0 after execution"
    );
    assert!(
        virtual_machine.verify_clean_memory().is_ok(),
        "Memory verification must report clean heap with zero leaks"
    );
}

#[test]
fn test_cross_package_generic_monomorphization_identical_class_names() {
    // 1. Library A defines package collections with generic Container<T>
    let collections_library_source = r#"
        package collections;

        public class Container<T> {
            public T container_value;

            public Container(T initial_value) {
                this.container_value = initial_value;
            }

            public T get_container_value() {
                return this.container_value;
            }
        }
    "#;

    let collections_options = CompilationOptions::new()
        .with_source("collections.slx", collections_library_source)
        .with_package_metadata("collections", "1.0.0", "Collections Package");
    let mut collections_context = CompilationContext::new(collections_options);
    collections_context
        .execute()
        .expect("Compilation of collections library failed");
    let collections_binary = collections_context.emission_artifacts.unwrap().program;

    let collections_manifest_json =
        extract_api_manifest(&collections_binary).expect("Manifest extraction failed");
    let (collections_header_name, collections_header_source) =
        manifest_to_source(&collections_manifest_json).expect("Header synthesis failed");

    // 2. Library B defines package storage with generic Container<T> (identical class name!)
    let storage_library_source = r#"
        package storage;

        public class Container<T> {
            public T stored_value;

            public Container(T initial_value) {
                this.stored_value = initial_value;
            }

            public T get_stored_value() {
                return this.stored_value;
            }
        }
    "#;

    let storage_options = CompilationOptions::new()
        .with_source("storage.slx", storage_library_source)
        .with_package_metadata("storage", "1.0.0", "Storage Package");
    let mut storage_context = CompilationContext::new(storage_options);
    storage_context
        .execute()
        .expect("Compilation of storage library failed");
    let storage_binary = storage_context.emission_artifacts.unwrap().program;

    let storage_manifest_json =
        extract_api_manifest(&storage_binary).expect("Manifest extraction failed");
    let (storage_header_name, storage_header_source) =
        manifest_to_source(&storage_manifest_json).expect("Header synthesis failed");

    // 3. Application consumer imports both and instantiates both with int32
    let consumer_application_source = r#"
        import collections.Container as CollectionsContainer;
        import storage.Container as StorageContainer;

        class Main {
            public static int32 main() {
                CollectionsContainer<int32> first_instance = new CollectionsContainer<int32>(40);
                StorageContainer<int32> second_instance = new StorageContainer<int32>(2);

                int32 combined_total = first_instance.get_container_value() + second_instance.get_stored_value();
                if (combined_total == 42) {
                    return 0;
                }
                return 1;
            }
        }
    "#;

    let application_options = CompilationOptions::new()
        .with_source(&collections_header_name, &collections_header_source)
        .with_source(&storage_header_name, &storage_header_source)
        .with_source("app.slx", consumer_application_source);
    let mut application_context = CompilationContext::new(application_options);
    application_context
        .execute()
        .expect("Compilation of application failed");
    let application_binary = application_context.emission_artifacts.unwrap().program;

    let runtime_options = RuntimeOptions::new()
        .with_binary(collections_binary)
        .with_binary(storage_binary)
        .with_binary(application_binary);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run().expect("VM execution failed");
    assert_eq!(execution_result, Value::I32(0));
    assert_eq!(
        virtual_machine.active_guest_objects(),
        0,
        "Active guest objects must be 0 after generic execution"
    );
    assert!(
        virtual_machine.verify_clean_memory().is_ok(),
        "Memory must be cleanly released with zero leaks"
    );
}

#[test]
fn test_cross_namespace_method_inlining_across_multiple_imported_namespaces() {
    let math_source = r#"
        package math_domain;

        class ScalingEngine {
            public static int32 scale_value(int32 operand_value) {
                return operand_value * 3;
            }
        }
    "#;

    let graphics_source = r#"
        package graphics_domain;

        class ScalingEngine {
            public static int32 scale_value(int32 operand_value) {
                return operand_value * 7;
            }
        }
    "#;

    let application_source = r#"
        import math_domain.ScalingEngine as MathEngine;
        import graphics_domain.ScalingEngine as GraphicsEngine;

        class Main {
            public static int32 main() {
                int32 math_scaled = MathEngine::scale_value(10);
                int32 graphics_scaled = GraphicsEngine::scale_value(10);

                if (math_scaled == 30 && graphics_scaled == 70) {
                    return 0;
                }
                return 1;
            }
        }
    "#;

    let compilation_options = CompilationOptions::new()
        .with_source("math.slx", math_source)
        .with_source("graphics.slx", graphics_source)
        .with_source("app.slx", application_source)
        .with_inline_threshold(16); // Enable inliner for small leaf methods

    let mut context = CompilationContext::new(compilation_options);
    context
        .execute()
        .expect("Compilation with inlining enabled failed");

    let artifacts = context
        .emission_artifacts
        .expect("Emission artifacts must exist");
    let main_function = artifacts
        .program
        .functions
        .iter()
        .find(|function| function.symbol.contains("Main::main"))
        .expect("Main::main must exist");

    // Both method calls should be inlined directly into main
    let has_direct_calls = main_function.instructions.iter().any(|instruction| {
        let opcode = instruction.raw_opcode();
        opcode == solix_common::bytecode::OpCode::CallDirect as u8
    });
    assert!(
        !has_direct_calls,
        "Expected both cross-namespace scaling methods to be inlined into Main::main"
    );

    let runtime_options = RuntimeOptions::new().with_binary(artifacts.program);
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);

    let execution_result = virtual_machine.run().expect("VM execution failed");
    assert_eq!(execution_result, Value::I32(0));
    assert!(
        virtual_machine.verify_clean_memory().is_ok(),
        "Inlined execution must be cleanly verified with zero leaks"
    );
}
