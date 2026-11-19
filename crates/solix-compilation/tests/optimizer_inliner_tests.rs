//! Comprehensive test suite for Phase 4: AST Function & Method Inliner.

#![forbid(unsafe_code)]

use solix_common::bytecode::OpCode;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

/// Helper compiling source code through the standard pipeline and returning the binary program.
fn compile_source(
    source_name: &str,
    source_text: &str,
    inline_threshold: usize,
) -> CompilationContext {
    let options = CompilationOptions::new()
        .with_source(source_name, source_text)
        .with_inline_threshold(inline_threshold);
    let mut context = CompilationContext::new(options);
    let compilation_result = context.execute();
    assert!(
        compilation_result.is_ok(),
        "Compilation failed for {source_name}: {compilation_result:?}"
    );
    context
}

/// Helper running a compiled binary program in the virtual machine and returning the exit code/value.
fn run_program(context: &CompilationContext) -> Value {
    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Emission artifacts must be present");
    let runtime_options = RuntimeOptions::new().with_binary(artifacts.program.clone());
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    virtual_machine.run().expect("VM execution failed")
}

#[test]
fn test_inliner_single_statement_function() {
    let source_code = r#"
        int32 add(int32 first_val, int32 second_val) {
            return first_val + second_val;
        }

        int32 main() {
            int32 left_operand = 15;
            int32 right_operand = 27;
            return add(left_operand, right_operand);
        }
    "#;

    let context = compile_source("test_single_stmt.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    // Locate the main function
    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    // Inlined call to add must NOT contain OpCode::CallDirect
    let contains_direct_call = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);
    assert!(
        !contains_direct_call,
        "Single statement function 'add' should have been inlined into 'main'"
    );

    // Verify VM produces exact result (15 + 27 = 42)
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(42));
}

#[test]
fn test_inliner_four_statement_function() {
    let source_code = r#"
        int32 compute_quad(int32 initial_val, int32 step_val) {
            int32 intermediate_one = initial_val * 2;
            int32 intermediate_two = step_val * 3;
            int32 intermediate_sum = intermediate_one + intermediate_two;
            return intermediate_sum + 10;
        }

        int32 main() {
            int32 base_val = 5;
            int32 stride_val = 4;
            return compute_quad(base_val, stride_val);
        }
    "#;

    let context = compile_source("test_four_stmt.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_direct_call = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);
    assert!(
        !contains_direct_call,
        "4-statement function 'compute_quad' should have been inlined into 'main'"
    );

    // 5 * 2 + 4 * 3 + 10 = 10 + 12 + 10 = 32
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(32));
}

#[test]
fn test_inliner_eight_statement_function() {
    let source_code = r#"
        int32 accumulate_eight(int32 initial_input) {
            int32 level_one = initial_input + 1;
            int32 level_two = level_one + 2;
            int32 level_three = level_two + 3;
            int32 level_four = level_three + 4;
            int32 level_five = level_four + 5;
            int32 level_six = level_five + 6;
            int32 level_seven = level_six + 7;
            return level_seven;
        }

        int32 main() {
            return accumulate_eight(10);
        }
    "#;

    let context = compile_source("test_eight_stmt.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_direct_call = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);
    assert!(
        !contains_direct_call,
        "8-statement function 'accumulate_eight' should have been inlined into 'main'"
    );

    // 10 + 1 + 2 + 3 + 4 + 5 + 6 + 7 = 38
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(38));
}

#[test]
fn test_non_inlining_nine_statement_function() {
    let source_code = r#"
        int32 accumulate_nine(int32 initial_input) {
            int32 level_one = initial_input + 1;
            int32 level_two = level_one + 2;
            int32 level_three = level_two + 3;
            int32 level_four = level_three + 4;
            int32 level_five = level_four + 5;
            int32 level_six = level_five + 6;
            int32 level_seven = level_six + 7;
            int32 level_eight = level_seven + 8;
            return level_eight;
        }

        int32 main() {
            return accumulate_nine(10);
        }
    "#;

    let context = compile_source("test_nine_stmt.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_direct_call = main_function.instructions.iter().any(|instruction| {
        let raw = instruction.raw_opcode();
        raw == OpCode::CallDirect as u8 || raw == OpCode::TailCallDirect as u8
    });
    assert!(
        contains_direct_call,
        "9-statement function 'accumulate_nine' should NOT be inlined when threshold is 8"
    );

    // 10 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 = 46
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(46));
}

#[test]
fn test_explicit_inline_modifier_overrides_threshold() {
    let source_code = r#"
        inline int32 explicit_inlined_nine(int32 initial_input) {
            int32 level_one = initial_input + 1;
            int32 level_two = level_one + 2;
            int32 level_three = level_two + 3;
            int32 level_four = level_three + 4;
            int32 level_five = level_four + 5;
            int32 level_six = level_five + 6;
            int32 level_seven = level_six + 7;
            int32 level_eight = level_seven + 8;
            return level_eight;
        }

        int32 main() {
            return explicit_inlined_nine(10);
        }
    "#;

    // Threshold is set to 4 (smaller than 9), but function is explicitly marked 'inline'
    let context = compile_source("test_explicit_inline.slx", source_code, 4);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_direct_call = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);
    assert!(
        !contains_direct_call,
        "Function marked with 'inline' modifier must be inlined regardless of threshold"
    );

    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(46));
}

#[test]
fn test_non_inlining_recursive_functions() {
    let source_code = r#"
        int32 fibonacci(int32 count) {
            if (count <= 1) {
                return count;
            }
            return fibonacci(count - 1) + fibonacci(count - 2);
        }

        int32 main() {
            return fibonacci(6);
        }
    "#;

    let context = compile_source("test_recursive.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let fib_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("fibonacci"))
        .expect("fibonacci function must exist");

    let recursive_call_present = fib_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);
    assert!(
        recursive_call_present,
        "Recursive function 'fibonacci' must preserve CallDirect and NOT be self-inlined"
    );

    // fib(6) = 8
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(8));
}

#[test]
fn test_non_inlining_virtual_and_override_methods() {
    let source_code = r#"
        class BaseEntity {
            public virtual int32 calculate_score() {
                return 100;
            }
        }

        class PlayerEntity extends BaseEntity {
            public override int32 calculate_score() {
                return 250;
            }
        }

        int32 main() {
            PlayerEntity entity = new PlayerEntity();
            return entity.calculate_score();
        }
    "#;

    let context = compile_source("test_virtual_non_inline.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_virtual_call = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallVirtual as u8);
    assert!(
        contains_virtual_call,
        "Virtual method call must preserve CallVirtual and NOT be inlined"
    );

    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(250));
}

#[test]
fn test_hygienic_variable_renaming_avoids_shadowing() {
    let source_code = r#"
        int32 multiply_add(int32 primary_factor, int32 secondary_factor) {
            int32 temp_buffer = primary_factor * 10;
            return temp_buffer + secondary_factor;
        }

        int32 main() {
            int32 temp_buffer = 999;
            int32 primary_factor = 5;
            int32 inlined_result = multiply_add(primary_factor, 7);
            // Verify caller's temp_buffer was not clobbered by callee's temp_buffer
            return temp_buffer + inlined_result;
        }
    "#;

    let context = compile_source("test_hygienic_renaming.slx", source_code, 8);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_direct_call = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);
    assert!(
        !contains_direct_call,
        "'multiply_add' should have been inlined into 'main'"
    );

    // inlined_result = 5 * 10 + 7 = 57.
    // caller's temp_buffer = 999.
    // Total = 999 + 57 = 1056.
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(1056));
}

#[test]
fn test_threshold_zero_disables_automatic_inlining() {
    let source_code = r#"
        int32 simple_inc(int32 initial_val) {
            return initial_val + 1;
        }

        int32 main() {
            return simple_inc(41);
        }
    "#;

    // Threshold = 0 disables automatic inlining
    let context = compile_source("test_threshold_zero.slx", source_code, 0);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    let main_function = program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let contains_direct_call = main_function.instructions.iter().any(|instruction| {
        let raw = instruction.raw_opcode();
        raw == OpCode::CallDirect as u8 || raw == OpCode::TailCallDirect as u8
    });
    assert!(
        contains_direct_call,
        "With threshold 0, simple_inc should NOT be inlined and must preserve a direct call"
    );

    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(42));
}

#[test]
fn test_inlined_method_preserves_local_virtual_dispatch() {
    let source_code = r#"
        class TextBox {
            public int32 code;

            public TextBox(int32 initial_code) {
                this.code = initial_code;
            }

            public virtual int32 slot_zero() {
                return 999;
            }

            public virtual int32 compute_weighted() {
                return this.code * 3;
            }
        }

        class Wrapper {
            public TextBox inner;

            public Wrapper(int32 raw_code) {
                this.inner = new TextBox(raw_code);
            }

            public TextBox unwrap() {
                return this.inner;
            }

            public int32 evaluate() {
                TextBox local_box = this.unwrap();
                return local_box.compute_weighted();
            }
        }

        int32 main() {
            Wrapper holder = new Wrapper(14);
            return holder.evaluate();
        }
    "#;

    let context = compile_source("test_inlined_local_vcall.slx", source_code, 8);
    let execution_value = run_program(&context);
    assert_eq!(execution_value, Value::I32(42));
}

#[test]
fn test_ast_inliner_method_resolution_across_namespaces_no_collision() {
    let source_graphics = r#"
        package graphics;

        public class Renderer {
            public int32 bias;

            public Renderer(int32 initial_bias) {
                this.bias = initial_bias;
            }

            public int32 render() {
                return this.bias + 10;
            }

            public static int32 compute_speed(int32 input_speed) {
                return input_speed * 2;
            }
        }
    "#;

    let source_audio = r#"
        package audio;

        public class Renderer {
            public int32 bias;

            public Renderer(int32 initial_bias) {
                this.bias = initial_bias;
            }

            public int32 render() {
                return this.bias + 100;
            }

            public static int32 compute_speed(int32 input_speed) {
                return input_speed * 10;
            }
        }
    "#;

    let source_main = r#"
        import graphics.*;
        import audio.*;

        public class MainApp {
            public static int32 main() {
                graphics::Renderer graphics_renderer = new graphics::Renderer(5);
                audio::Renderer audio_renderer = new audio::Renderer(7);

                int32 graphics_value = graphics_renderer.render();
                int32 audio_value = audio_renderer.render();

                int32 graphics_static = graphics::Renderer::compute_speed(3);
                int32 audio_static = audio::Renderer::compute_speed(3);

                // graphics_value = 5 + 10 = 15
                // audio_value = 7 + 100 = 107
                // graphics_static = 3 * 2 = 6
                // audio_static = 3 * 10 = 30
                // Total = 15 + 107 + 6 + 30 = 158
                return graphics_value + audio_value + graphics_static + audio_static;
            }
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("graphics_renderer.slx", source_graphics)
        .with_source("audio_renderer.slx", source_audio)
        .with_source("main.slx", source_main)
        .with_entry_point(Some("MainApp::main".to_string()))
        .with_inline_threshold(8);

    let mut context = CompilationContext::new(options);
    let compilation_result = context.execute();
    assert!(
        compilation_result.is_ok(),
        "Multi-namespace compilation failed: {compilation_result:?}"
    );

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let program = &artifacts.program;

    // Locate the main function and ensure both render calls were inlined
    let main_function = program
        .functions
        .iter()
        .find(|function| function.symbol.contains("main"))
        .expect("main function must exist");

    let contains_render_call = main_function.instructions.iter().any(|instruction| {
        let raw_opcode = instruction.raw_opcode();
        (raw_opcode == OpCode::CallDirect as u8
            || raw_opcode == OpCode::CallVirtual as u8
            || raw_opcode == OpCode::CallExtern as u8)
            && instruction.imm32() > 0
    });
    assert!(
        !contains_render_call,
        "Calls to Renderer methods must be inlined into main"
    );

    let execution_value = run_program(&context);
    assert_eq!(
        execution_value,
        Value::I32(158),
        "Each namespace's method body must be independently inlined without cross-namespace collision"
    );
}
