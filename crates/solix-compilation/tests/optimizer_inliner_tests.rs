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

#[test]
fn test_ast_inliner_preserves_this_class_type_not_return_type() {
    use solix_compilation::phases::optimizer::AstInlinerPhase;
    use solix_compilation::phases::parser::statements::Statement;
    use solix_compilation::phases::parser::statements::control_flow::ReturnStatement;
    use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
    use solix_compilation::phases::parser::statements::expressions::Expression;
    use solix_compilation::phases::parser::statements::types::TypeAnnotation;
    use solix_compilation::phases::{BinderPhase, LexerPhase, ParserPhase};
    use solix_compilation::pipeline::CompilationPipeline;

    let source = r#"
        public class Point {
            public int32 coord_x;
            public int32 coord_y;
            public float64 scale;

            public Point(int32 init_x, int32 init_y, float64 init_scale) {
                this.coord_x = init_x;
                this.coord_y = init_y;
                this.scale = init_scale;
            }

            public int32 sum_coords() {
                return this.coord_x + this.coord_y;
            }

            public float64 get_scaled_x() {
                return cast(this.coord_x, float64) * this.scale;
            }
        }

        public class MainApp {
            public static int32 main() {
                Point pt = new Point(3, 4, 2.0);
                int32 total = pt.sum_coords();
                float64 scaled = pt.get_scaled_x();
                return total + cast(scaled, int32);
            }
        }
    "#;

    let mut pipeline = CompilationPipeline::empty()
        .add_phase(LexerPhase::new())
        .add_phase(ParserPhase::new())
        .add_phase(BinderPhase::new())
        .add_phase(AstInlinerPhase::new());

    let options = CompilationOptions::new()
        .with_source("main.slx", source)
        .with_entry_point(Some("MainApp::main".to_string()))
        .with_inline_threshold(8);
    let mut context = CompilationContext::new(options);
    context
        .execute_pipeline(&mut pipeline)
        .expect("Frontend + AstInliner pipeline must succeed");

    let artifacts = context
        .binding_artifacts
        .as_ref()
        .expect("Binding artifacts present");
    let ast = context.ast[0].as_ref().expect("AST present");

    let mut found_synthetic_this_count = 0usize;
    for decl in &ast.declarations {
        if let Declaration::Class(class_decl) = decl {
            for member in &class_decl.members {
                if let ClassMember::Method(method) = member
                    && let Some(body) = &method.body
                {
                    for stmt in &body.statements {
                        let init_expr = match stmt {
                            Statement::VariableDeclaration(var_decl) => &var_decl.initializer,
                            _ => &None,
                        };
                        if let Some(Expression::Block { statements, .. }) = init_expr {
                            for inner_stmt in statements {
                                if let Statement::VariableDeclaration(inner_var) = inner_stmt {
                                    let var_name = context.get_lexeme(inner_var.name_span);
                                    if var_name.ends_with("_this") {
                                        found_synthetic_this_count += 1;
                                        assert!(
                                            matches!(
                                                inner_var.var_type,
                                                TypeAnnotation::Named { .. }
                                            ),
                                            "Synthetic '{var_name}' var_type must be TypeAnnotation::Named (Point), got {:?}",
                                            inner_var.var_type
                                        );
                                        let node_type = artifacts
                                            .node_types
                                            .get(&inner_var.name_span)
                                            .expect("Synthetic 'this' node_type must exist");
                                        assert!(
                                            matches!(node_type, solix_common::Type::Class { symbol_name, .. } if symbol_name.contains("Point")),
                                            "Expected Type::Class(Point) for '{var_name}', got {node_type:?}"
                                        );
                                    }
                                }
                            }
                        }
                        if let Statement::Return(ReturnStatement { .. }) = stmt {}
                    }
                }
            }
        }
    }
    assert_eq!(
        found_synthetic_this_count, 2,
        "Expected both pt.sum_coords() and pt.get_scaled_x() to synthesize 'this' bindings"
    );

    // Also run end-to-end execution to verify emitted bytecode behavior
    let full_options = CompilationOptions::new()
        .with_source("main.slx", source)
        .with_entry_point(Some("MainApp::main".to_string()))
        .with_inline_threshold(8);
    let mut full_context = CompilationContext::new(full_options);
    full_context
        .execute()
        .expect("Full compilation with inlined instance methods must succeed");
    assert_eq!(run_program(&full_context), Value::I32(13));
}

#[test]
fn test_ast_inliner_unqualified_call_respects_resolved_symbols_and_return_types() {
    use solix_compilation::phases::optimizer::AstInlinerPhase;
    use solix_compilation::phases::parser::statements::Statement;
    use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};
    use solix_compilation::phases::{BinderPhase, LexerPhase, ParserPhase};
    use solix_compilation::pipeline::CompilationPipeline;

    // Case 1: External symbol with same name as a local free function must NOT be rewritten
    // to inline the unrelated local free function body.
    let ext_source = r#"
        package extpkg;

        public int32 shared_helper() {
            return 10;
        }
    "#;
    let local_source = r#"
        package app;
        import extpkg.*;

        public class Point {
            public int32 val;
            public Point(int32 v) { this.val = v; }
        }

        Point make_point() {
            return new Point(42);
        }

        int32[] make_array() {
            return new int32[3];
        }

        public class MainApp {
            public static int32 main() {
                Point made = make_point();
                int32[] arr = make_array();
                return shared_helper() + made.val + arr.length;
            }
        }
    "#;
    let helper_source = r#"
        package other;
        int32 shared_helper() {
            return 9999;
        }
    "#;

    let mut pipeline = CompilationPipeline::empty()
        .add_phase(LexerPhase::new())
        .add_phase(ParserPhase::new())
        .add_phase(BinderPhase::new());

    let options = CompilationOptions::new()
        .with_source("ext.slx", ext_source)
        .with_source("local.slx", local_source)
        .with_source("other.slx", helper_source)
        .with_entry_point(Some("app::MainApp::main".to_string()))
        .with_inline_threshold(8);
    let mut context = CompilationContext::new(options);
    context.mark_external_source(0);
    context
        .execute_pipeline(&mut pipeline)
        .expect("Frontend pipeline must succeed");

    // Clear node_types for make_point() and make_array() call spans to exercise vacant return resolution
    let ast = context.ast[1].as_ref().expect("local AST");
    let mut call_spans = Vec::new();
    for decl in &ast.declarations {
        if let Declaration::Class(class_decl) = decl {
            for member in &class_decl.members {
                if let ClassMember::Method(method) = member
                    && let Some(body) = &method.body
                {
                    for stmt in &body.statements {
                        if let Statement::VariableDeclaration(var_decl) = stmt
                            && let Some(init) = &var_decl.initializer
                        {
                            call_spans.push((
                                context.get_lexeme(var_decl.name_span).to_string(),
                                init.span(),
                            ));
                        }
                    }
                }
            }
        }
    }
    if let Some(artifacts) = context.binding_artifacts.as_mut() {
        for (_, span) in &call_spans {
            artifacts.node_types.remove(span);
        }
    }

    let mut inliner = AstInlinerPhase::new();
    use solix_compilation::phases::phase::CompilationPhase;
    inliner
        .run(&mut context)
        .expect("AstInlinerPhase must succeed");

    let artifacts = context
        .binding_artifacts
        .as_ref()
        .expect("Binding artifacts");
    for (name, span) in &call_spans {
        let recorded = artifacts
            .node_types
            .get(span)
            .expect("Inlined call span must recover return type into node_types");
        if name == "made" {
            assert!(
                matches!(recorded, solix_common::Type::Class { symbol_name, .. } if symbol_name.contains("Point")),
                "make_point() return type must be Type::Class(Point), got {recorded:?}"
            );
        } else if name == "arr" {
            assert!(
                matches!(recorded, solix_common::Type::Array { .. }),
                "make_array() return type must be Type::Array, got {recorded:?}"
            );
        }
    }

    // Confirm shared_helper() call inside MainApp::main was NOT inlined with 9999 from 'other'
    let main_ast = context.ast[1].as_ref().expect("local AST after inliner");
    let return_expr = main_ast.declarations.iter().find_map(|decl| {
        if let Declaration::Class(class_decl) = decl {
            for member in &class_decl.members {
                if let ClassMember::Method(method) = member
                    && let Some(body) = &method.body
                {
                    for stmt in &body.statements {
                        if let Statement::Return(ret) = stmt {
                            return ret.value.as_ref();
                        }
                    }
                }
            }
        }
        None
    });
    let serialized_ret = format!("{return_expr:?}");
    assert!(
        !serialized_ret.contains("9999"),
        "External shared_helper() must never inline unrelated other::shared_helper (9999): {serialized_ret}"
    );
}
