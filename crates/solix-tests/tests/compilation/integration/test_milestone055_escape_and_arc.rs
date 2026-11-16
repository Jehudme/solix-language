//! Milestone 055 Conformance Tests: Escape Analysis, Scalar Replacement & ARC Elision.

#![forbid(unsafe_code)]

use solix_common::bytecode::OpCode;
use solix_runtime::value::model::Value;
use solix_tests::common::{compile_and_run, compile_source};

#[test]
fn test_sra_point_struct_registers() {
    let source_code = r#"
        class Point {
            public int32 x_coord;
            public int32 y_coord;

            public Point(int32 x_coord, int32 y_coord) {
                this.x_coord = x_coord;
                this.y_coord = y_coord;
            }
        }

        int32 main() {
            Point local_point = new Point(10, 20);
            int32 total_sum = local_point.x_coord + local_point.y_coord;
            return total_sum;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let main_function = compiled_program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    // SRA must eliminate AllocObject from main() because local_point does not escape!
    let contains_alloc_object = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::AllocObject as u8);
    assert!(
        !contains_alloc_object,
        "SRA should eliminate OpCode::AllocObject for non-escaping Point object in main()"
    );

    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(30));
}

#[test]
fn test_arc_elision_local_temporary() {
    let source_code = r#"
        class DataHolder {
            public int32 val;
            public DataHolder(int32 val) { this.val = val; }
        }

        int32 main() {
            DataHolder holder = new DataHolder(42);
            int32 extracted = holder.val;
            return extracted;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let main_function = compiled_program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let has_arc_instructions = main_function.instructions.iter().any(|instruction| {
        let opcode = instruction.raw_opcode();
        opcode == OpCode::IncRef as u8 || opcode == OpCode::DecRef as u8
    });

    assert!(
        !has_arc_instructions,
        "Local non-escaping object in main() should have zero IncRef/DecRef instructions after ARC elision"
    );

    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_escape_analysis_return_escape() {
    let source_code = r#"
        class Vector {
            public int32 magnitude;
            public Vector(int32 magnitude) { this.magnitude = magnitude; }
        }

        Vector create_vector(int32 magnitude) {
            Vector result_vector = new Vector(magnitude);
            return result_vector;
        }

        int32 main() {
            Vector created = create_vector(100);
            return created.magnitude;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let create_function = compiled_program
        .functions
        .iter()
        .find(|func| func.symbol.contains("create_vector"))
        .expect("create_vector must exist");

    // Returning result_vector must cause it to escape to GlobalEscape, preserving AllocObject
    let contains_alloc_object = create_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::AllocObject as u8);
    assert!(
        contains_alloc_object,
        "Returned object must escape, preserving OpCode::AllocObject on heap"
    );

    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(100));
}

#[test]
fn test_escape_analysis_array_escape() {
    let source_code = r#"
        class Item {
            public int32 identifier;
            public Item(int32 identifier) { this.identifier = identifier; }
        }

        int32 main() {
            Item[] item_array = new Item[2];
            Item target_item = new Item(777);
            item_array[0] = target_item;
            return item_array[0].identifier;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let main_function = compiled_program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main must exist");

    // Storing target_item into item_array causes it to escape to the heap array
    let contains_alloc_object = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::AllocObject as u8);
    assert!(
        contains_alloc_object,
        "Storing into array must preserve OpCode::AllocObject"
    );

    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(777));
}

#[test]
fn test_escape_analysis_aliasing_propagation() {
    let source_code = r#"
        class Wrapper {
            public int32 count;
            public Wrapper(int32 count) { this.count = count; }
        }

        Wrapper escape_target;

        int32 main() {
            Wrapper original = new Wrapper(55);
            Wrapper alias_var = original;
            escape_target = alias_var; // GlobalEscape propagates to original
            return escape_target.count;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let main_function = compiled_program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main must exist");

    let contains_alloc_object = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::AllocObject as u8);
    assert!(
        contains_alloc_object,
        "Aliasing an escaping variable must prevent scalar replacement"
    );

    let (execution_result, _) = compile_and_run(source_code);
    assert_eq!(execution_result, Value::I32(55));
}
