//! Suite: DynamicLinkingAndQuickening - Part V (TESTS.md)

#![forbid(unsafe_code)]

use solix_common::bytecode::{OpCode, extract_api_manifest};
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::manifest_to_source;
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[test]
fn test_case_57_1_multi_binary_vtable_and_itable_relocation() {
    // Binary 1: Library with Base and Derived classes
    let lib_code = r#"
        public interface Formatter {
            public int32 format(int32 x);
        }

        public class HexFormatter implements Formatter {
            public HexFormatter() {}
            public int32 format(int32 x) {
                return x * 16;
            }
        }
    "#;
    let lib_options = CompilationOptions::new()
        .with_source("lib.slx", lib_code)
        .with_package_metadata("lib", "1.0.0", "Formatters");
    let mut lib_ctx = CompilationContext::new(lib_options);
    lib_ctx.execute().unwrap();
    let lib_bin = lib_ctx.emission_artifacts.unwrap().program;

    // Binary 2: Consumer with its own independent class
    let app_code = r#"
        public class LocalPrinter {
            public int32 id;
            public LocalPrinter(int32 id) {
                this.id = id;
            }
            public int32 print_val(int32 val) {
                return this.id + val;
            }
        }

        class Main {
            public static int32 main() {
                LocalPrinter p = new LocalPrinter(100);
                return p.print_val(25);
            }
        }
    "#;
    let app_options = CompilationOptions::new().with_source("app.slx", app_code);
    let mut app_ctx = CompilationContext::new(app_options);
    app_ctx.execute().unwrap();
    let app_bin = app_ctx.emission_artifacts.unwrap().program;

    // Load both binaries
    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context init failed");

    // Verify vtable IDs were relocated and do not collide
    assert_eq!(context.all_classes.len(), 4);
    let vtable_ids: std::collections::HashSet<u32> =
        context.all_classes.iter().map(|c| c.vtable_id).collect();
    assert_eq!(vtable_ids.len(), context.all_classes.len());

    let mut vm = VirtualMachine::new(context);
    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(125));
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_case_57_2_in_place_instruction_quickening_for_dynamic_calls() {
    // 1. Library providing function
    let lib_code = r#"
        public int32 calculate_sum(int32 a, int32 b) {
            return a + b;
        }
    "#;
    let lib_options = CompilationOptions::new()
        .with_source("lib.slx", lib_code)
        .with_package_metadata("mathlib", "1.0.0", "Math Library");
    let mut lib_ctx = CompilationContext::new(lib_options);
    lib_ctx.execute().unwrap();
    let lib_bin = lib_ctx.emission_artifacts.unwrap().program;

    // 2. Extract API and synthesize header
    let manifest_json = extract_api_manifest(&lib_bin).unwrap();
    let (header_name, header_source) = manifest_to_source(&manifest_json).unwrap();

    // 3. Compile consumer against synthesized header
    let app_code = r#"
        import mathlib.*;

        class Main {
            public static int32 main() {
                int32 s = 0;
                int32 i = 0;
                while (i < 3) {
                    s = calculate_sum(s, 10);
                    i = i + 1;
                }
                return s;
            }
        }
    "#;
    let app_options = CompilationOptions::new()
        .with_source(&header_name, &header_source)
        .with_source("app.slx", app_code);
    let mut app_ctx = CompilationContext::new(app_options);
    app_ctx.execute().unwrap();
    let app_bin = app_ctx.emission_artifacts.unwrap().program;

    // Load both binaries
    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context init failed");

    // Locate Main::main function
    let main_id = *context.symbol_to_function_id.get("Main::main").unwrap();
    let main_func = context.get_function(main_id).unwrap();

    // Find the call instruction index
    let call_inst_idx = main_func
        .instructions
        .iter()
        .position(|inst| inst.opcode(0).ok() == Some(OpCode::CallExtern))
        .expect("CallExtern instruction must be present before execution");

    let mut vm = VirtualMachine::new(context);
    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(30));
    assert!(vm.verify_clean_memory().is_ok());

    // Verify in-place quickening to CallDirect
    let quickened_op = vm.context.get_function(main_id).unwrap().instructions[call_inst_idx]
        .opcode(0)
        .unwrap();
    assert_eq!(quickened_op, OpCode::CallDirect);
}

#[test]
fn test_case_57_3_in_place_instruction_quickening_for_static_variables() {
    let lib_code = r#"
        static int32 COUNTER = 50;
    "#;
    let lib_options = CompilationOptions::new()
        .with_source("lib.slx", lib_code)
        .with_package_metadata("counter_pkg", "1.0.0", "Counter");
    let mut lib_ctx = CompilationContext::new(lib_options);
    lib_ctx.execute().unwrap();
    let lib_bin = lib_ctx.emission_artifacts.unwrap().program;

    let manifest_json = extract_api_manifest(&lib_bin).unwrap();
    let (header_name, header_source) = manifest_to_source(&manifest_json).unwrap();

    let app_code = r#"
        class Main {
            public static int32 main() {
                COUNTER = COUNTER + 25;
                return COUNTER;
            }
        }
    "#;
    let app_options = CompilationOptions::new()
        .with_source(&header_name, &header_source)
        .with_source("app.slx", app_code);
    let mut app_ctx = CompilationContext::new(app_options);
    app_ctx.execute().unwrap();
    let app_bin = app_ctx.emission_artifacts.unwrap().program;

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context init failed");
    let main_id = *context.symbol_to_function_id.get("Main::main").unwrap();

    let mut vm = VirtualMachine::new(context);
    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(75));
    assert!(vm.verify_clean_memory().is_ok());

    // Verify SetStaticGlobal and GetStaticGlobal were quickened to SetStaticDirect / GetStaticDirect
    let instructions = &vm.context.get_function(main_id).unwrap().instructions;
    let has_get_direct = instructions
        .iter()
        .any(|i| i.opcode(0).ok() == Some(OpCode::GetStaticDirect));
    let has_set_direct = instructions
        .iter()
        .any(|i| i.opcode(0).ok() == Some(OpCode::SetStaticDirect));
    assert!(
        has_get_direct,
        "GetStaticGlobal must quicken to GetStaticDirect"
    );
    assert!(
        has_set_direct,
        "SetStaticGlobal must quicken to SetStaticDirect"
    );
}

#[test]
fn test_case_57_4_in_place_instruction_quickening_for_object_allocation() {
    let lib_code = r#"
        public class Point {
            public int32 x;
            public int32 y;
            public Point(int32 x, int32 y) {
                this.x = x;
                this.y = y;
            }
            public int32 sum() {
                return this.x + this.y;
            }
        }
    "#;
    let lib_options = CompilationOptions::new()
        .with_source("lib.slx", lib_code)
        .with_package_metadata("geom", "1.0.0", "Geometry");
    let mut lib_ctx = CompilationContext::new(lib_options);
    lib_ctx.execute().unwrap();
    let lib_bin = lib_ctx.emission_artifacts.unwrap().program;

    let manifest_json = extract_api_manifest(&lib_bin).unwrap();
    let (header_name, header_source) = manifest_to_source(&manifest_json).unwrap();

    let app_code = r#"
        import geom.*;

        class Main {
            public static int32 main() {
                Point pt = new Point(15, 35);
                return pt.sum();
            }
        }
    "#;
    let app_options = CompilationOptions::new()
        .with_source(&header_name, &header_source)
        .with_source("app.slx", app_code);
    let mut app_ctx = CompilationContext::new(app_options);
    app_ctx.execute().unwrap();
    let app_bin = app_ctx.emission_artifacts.unwrap().program;

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context init failed");
    let main_id = *context.symbol_to_function_id.get("Main::main").unwrap();

    // Verify AllocObjectExtern is present before execution
    let has_alloc_extern = context
        .get_function(main_id)
        .unwrap()
        .instructions
        .iter()
        .any(|i| i.opcode(0).ok() == Some(OpCode::AllocObjectExtern));
    assert!(
        has_alloc_extern,
        "AllocObjectExtern must be present before execution"
    );

    let mut vm = VirtualMachine::new(context);
    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(50));
    assert!(vm.verify_clean_memory().is_ok());

    // Verify AllocObjectExtern quickened to AllocObject
    let instructions = &vm.context.get_function(main_id).unwrap().instructions;
    let has_alloc_direct = instructions
        .iter()
        .any(|i| i.opcode(0).ok() == Some(OpCode::AllocObject));
    assert!(
        has_alloc_direct,
        "AllocObjectExtern must quicken to AllocObject"
    );
}

#[test]
fn test_case_57_5_cross_binary_generic_template_monomorphization() {
    let lib_code = r#"
        public class Container<T> {
            public T item;
            public Container(T item) {
                this.item = item;
            }
            public T get_item() {
                return this.item;
            }
        }
    "#;
    let lib_options = CompilationOptions::new()
        .with_source("lib.slx", lib_code)
        .with_package_metadata("generic_pkg", "1.0.0", "Generics");
    let mut lib_ctx = CompilationContext::new(lib_options);
    lib_ctx.execute().unwrap();
    let lib_bin = lib_ctx.emission_artifacts.unwrap().program;

    // Verify template was captured in manifest
    assert!(!lib_bin.export_manifest.generic_templates.is_empty());
    assert_eq!(
        lib_bin.export_manifest.generic_templates[0].name,
        "Container"
    );

    let manifest_json = extract_api_manifest(&lib_bin).unwrap();
    let (header_name, header_source) = manifest_to_source(&manifest_json).unwrap();

    // Consumer instantiates Container<int32>
    let app_code = r#"
        class Main {
            public static int32 main() {
                Container<int32> c = new Container<int32>(99);
                return c.get_item();
            }
        }
    "#;
    let app_options = CompilationOptions::new()
        .with_source(&header_name, &header_source)
        .with_source("app.slx", app_code);
    let mut app_ctx = CompilationContext::new(app_options);
    app_ctx.execute().unwrap();
    let app_bin = app_ctx.emission_artifacts.unwrap().program;

    let options = RuntimeOptions::new()
        .with_binary(lib_bin)
        .with_binary(app_bin);
    let context = RuntimeContext::new(options).expect("Context init failed");

    let mut vm = VirtualMachine::new(context);
    let res = vm.run().expect("Execution failed");
    assert_eq!(res, Value::I32(99));
    assert!(vm.verify_clean_memory().is_ok());
}
