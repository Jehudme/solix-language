//! Integration tests for Milestone 023 Pillar 3: Dual-Mode Linking and Static DCE Reachability.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_compilation::CompilationContext::{
    BuildProfile, CompilationContext, CompilationOptions, LinkMode,
};
use solix_compilation::CompilationPipeline;
use solix_runtime::context::RuntimeContext;
use solix_runtime::value::Value;
use solix_runtime::{RuntimeOptions, VirtualMachine};

#[test]
fn test_static_linking_relocates_external_function_and_rewrites_to_call_direct() {
    // 1. Build a dependency binary with a reachable function and an unreachable dead function
    let mut dependency_binary = BinaryProgram::new();
    dependency_binary.package_name = "math_lib".to_string();

    // Reachable function: math_lib::add(a, b) -> a + b
    let mut add_function = CompiledFunction::new("math_lib::add", 2, 0, 4);
    // arg0: r[0], arg1: r[1], result: r[0] = r[0] + r[1]; return r[0]
    add_function
        .instructions
        .push(Instruction::format_a(OpCode::AddI32, 0, 0, 0, 1));
    add_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    dependency_binary.functions.push(add_function);

    // Unreachable function: math_lib::dead_function()
    let mut dead_function = CompiledFunction::new("math_lib::dead_function", 0, 0, 2);
    dead_function
        .instructions
        .push(Instruction::format_b(OpCode::LoadI32, 0, 0, 42));
    dead_function
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    dependency_binary.functions.push(dead_function);

    // 2. Build consumer code that calls math_lib::add
    let consumer_source = r#"
package app;

extern public int32 math_lib::add(int32 first, int32 second);

public int32 main() {
    return math_lib::add(15, 27);
}
"#;

    let mut compilation_options = CompilationOptions::new()
        .with_profile(BuildProfile::Debug)
        .with_link_mode(LinkMode::Static)
        .with_source("consumer.slx", consumer_source);
    compilation_options
        .dependency_binaries
        .push(dependency_binary);

    let mut context = CompilationContext::new(compilation_options);
    let mut pipeline = CompilationPipeline::standard();
    let compilation_result = pipeline.execute(&mut context);
    assert!(
        compilation_result.is_ok(),
        "Compilation failed: {:?}",
        compilation_result.err()
    );

    let artifacts = context.emit_context().expect("emit_context failed");

    let binary = &artifacts.program;

    // Verify unreachable function was pruned by Dead Code Elimination
    assert!(
        !binary
            .functions
            .iter()
            .any(|function| function.symbol.contains("dead_function")),
        "Dead function should have been eliminated by reachability DCE"
    );

    // Verify math_lib::add was merged into the binary
    assert!(
        binary
            .functions
            .iter()
            .any(|function| function.symbol.contains("math_lib::add")),
        "Reachable function math_lib::add must be statically linked into the binary"
    );

    // Verify main's call instruction was rewritten from CallExtern to CallDirect
    let main_function = binary
        .functions
        .iter()
        .find(|function| function.symbol == "main" || function.symbol.ends_with("::main"))
        .expect("main function must exist in binary");

    let has_call_direct = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallDirect as u8);

    assert!(
        has_call_direct,
        "Static linking must rewrite CallExtern to CallDirect"
    );

    let has_call_extern = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallExtern as u8);

    assert!(
        !has_call_extern,
        "Static binary must not retain CallExtern for resolved static dependency"
    );

    // Execute standalone without external dependency loaded into VM
    let runtime_options = RuntimeOptions::new().with_binary(binary.clone());
    let runtime_context = RuntimeContext::new(runtime_options)
        .expect("Failed to create runtime context for statically linked binary");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let execution_result = virtual_machine.run().expect("Execution failed");
    assert_eq!(
        execution_result,
        Value::I32(42),
        "Statically linked execution should return 15 + 27 = 42"
    );
}

#[test]
fn test_dynamic_linking_mode_retains_call_extern() {
    let consumer_source = r#"
package app;

extern public int32 external_service::compute(int32 input);

public int32 main() {
    return external_service::compute(10);
}
"#;

    let compilation_options = CompilationOptions::new()
        .with_profile(BuildProfile::Debug)
        .with_link_mode(LinkMode::Dynamic)
        .with_source("consumer_dyn.slx", consumer_source);

    let mut context = CompilationContext::new(compilation_options);
    let mut pipeline = CompilationPipeline::standard();
    let compilation_result = pipeline.execute(&mut context);
    assert!(
        compilation_result.is_ok(),
        "Compilation failed: {:?}",
        compilation_result.err()
    );

    let artifacts = context.emit_context().expect("emit_context failed");

    let binary = &artifacts.program;

    let main_function = binary
        .functions
        .iter()
        .find(|function| function.symbol == "main" || function.symbol.ends_with("::main"))
        .expect("main function must exist in binary");

    let has_call_extern = main_function
        .instructions
        .iter()
        .any(|instruction| instruction.raw_opcode() == OpCode::CallExtern as u8);

    assert!(
        has_call_extern,
        "Dynamic linking mode must preserve CallExtern for external symbols"
    );
}

#[test]
fn test_static_linking_relocates_call_symbol_with_high_constant_pool_index() {
    let mut dependency_binary = BinaryProgram::new();
    dependency_binary.package_name = "native_mod".to_string();

    // Pad dependency constant pool with unrelated constants so the symbol string sits at a
    // high index exceeding the consumer binary's constant pool length.
    for padding_idx in 0..64 {
        dependency_binary
            .constant_pool
            .add_string(format!("unused_constant_{padding_idx}"))
            .expect("add padding constant");
    }
    let sym_const_idx = dependency_binary
        .constant_pool
        .add_string("native_mod::bridge_add")
        .expect("add native symbol constant");
    assert!(sym_const_idx >= 64);

    let mut native_stub = CompiledFunction::new("native_mod::bridge_add", 0, 2, 3);
    native_stub.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        2,
        sym_const_idx,
        0,
        2,
    ));
    native_stub
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));
    dependency_binary.functions.push(native_stub);

    let consumer_source = r#"
package app;

extern public int32 native_mod::bridge_add(int32 left, int32 right);

public int32 main() {
    return native_mod::bridge_add(19, 23);
}
"#;

    let mut compilation_options = CompilationOptions::new()
        .with_profile(BuildProfile::Release)
        .with_link_mode(LinkMode::Static)
        .with_source("consumer_sym.slx", consumer_source);
    compilation_options
        .dependency_binaries
        .push(dependency_binary);

    let mut context = CompilationContext::new(compilation_options);
    let mut pipeline = CompilationPipeline::standard();
    pipeline
        .execute(&mut context)
        .expect("Static compilation with native CallSymbol dependency must succeed");

    let artifacts = context.emit_context().expect("emit_context failed");
    let binary = &artifacts.program;

    // Verify CallSymbol index was relocated within the merged binary's constant pool
    assert!(
        binary.constant_pool.len() < 64,
        "Consumer constant pool ({}) should be smaller than un-relocated index {sym_const_idx}",
        binary.constant_pool.len()
    );

    fn mock_bridge_add(
        _vm: &mut VirtualMachine,
        args: &[Value],
    ) -> Result<Value, solix_runtime::RuntimeError> {
        let left = args.first().and_then(|val| val.as_i32().ok()).unwrap_or(0);
        let right = args.get(1).and_then(|val| val.as_i32().ok()).unwrap_or(0);
        Ok(Value::I32(left + right))
    }

    let runtime_options = RuntimeOptions::new()
        .with_binary(binary.clone())
        .with_native_function("native_mod::bridge_add", mock_bridge_add);
    let runtime_context = RuntimeContext::new(runtime_options)
        .expect("Failed to initialize RuntimeContext with relocated CallSymbol");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let execution_result = virtual_machine
        .run()
        .expect("Executing statically linked native CallSymbol must succeed");
    assert_eq!(execution_result, Value::I32(42));
}

#[test]
fn test_static_linking_relocates_classes_vtables_overloads_constants_and_statics() {
    // 1. Compile rich dependency library with classes, virtual dispatch, constructor overloads,
    //    static globals (@init + GetStaticGlobal), float constants (LoadF64), closures, and dead code.
    let dep_source = r#"
package gfx_lib;

public class GfxConfig {
    public static int32 GLOBAL_SCALE = 3;
}

public class WidgetBase {
    public int32 width;

    public WidgetBase() {
        this.width = 5;
    }

    public virtual int32 compute(int32 input) {
        return this.width + input;
    }
}

public class FancyWidget extends WidgetBase {
    public int32 height;

    public FancyWidget() {
        this.width = 1;
        this.height = 1;
    }

    public FancyWidget(int32 init_width, int32 init_height) {
        this.width = init_width;
        this.height = init_height;
    }

    public override int32 compute(int32 input) {
        int32 area = this.width * this.height;
        int32 scaled_bias = FancyWidget::compute_bias(input);
        return area + scaled_bias + GfxConfig::GLOBAL_SCALE;
    }

    public static int32 compute_bias(int32 offset) {
        float64 factor = 2.5;
        int32 bonus = 0;
        if (factor > 2.0) {
            bonus = 6;
        }
        return offset + bonus;
    }
}

public int32 apply_with_lambda(int32 seed) {
    int32 captured_bias = 1;
    int32(*)(int32) doubler = [captured_bias](int32 value) => (value * 2) + captured_bias;
    return doubler(seed);
}

public int32 unreachable_dep_helper() {
    return 9999;
}
"#;

    let dep_options = CompilationOptions::new()
        .with_profile(BuildProfile::Release)
        .with_inline_threshold(0)
        .with_package_metadata("gfx_lib", "1.0.0", "Rich test dependency")
        .with_source("gfx_lib.slx", dep_source);
    let mut dep_context = CompilationContext::new(dep_options);
    let mut dep_pipeline = CompilationPipeline::standard();
    dep_pipeline
        .execute(&mut dep_context)
        .expect("Dependency compilation must succeed");
    let dep_artifacts = dep_context
        .emit_context()
        .expect("Dependency emission must succeed");
    let dep_binary = dep_artifacts.program.clone();

    // 2. Compile consumer with its own classes (high vtable_id sparse distribution) and
    //    static linking against `gfx_lib`.
    let consumer_source = r#"
package app;
import gfx_lib.*;

public class LocalAlpha {
    public int32 tag;
    public LocalAlpha(int32 tag_val) {
        this.tag = tag_val;
    }
    public virtual int32 read_tag() {
        return this.tag;
    }
}

public class LocalBeta {
    public int32 code;
    public LocalBeta(int32 code_val) {
        this.code = code_val;
    }
    public virtual int32 read_code() {
        return this.code + 5;
    }
}

public int32 main() {
    LocalBeta local_beta = new LocalBeta(4);
    WidgetBase widget = new FancyWidget(6, 5);
    int32 virtual_res = widget.compute(2);
    int32 doubled = gfx_lib::apply_with_lambda(local_beta.read_code());
    return virtual_res + doubled;
}
"#;

    let mut consumer_options = CompilationOptions::new()
        .with_profile(BuildProfile::Release)
        .with_link_mode(LinkMode::Static)
        .with_source("consumer_rich.slx", consumer_source);
    consumer_options.dependency_binaries.push(dep_binary);

    let mut consumer_context = CompilationContext::new(consumer_options);
    let mut consumer_pipeline = CompilationPipeline::standard();
    consumer_pipeline
        .execute(&mut consumer_context)
        .expect("Static consumer compilation must succeed");

    let consumer_artifacts = consumer_context
        .emit_context()
        .expect("Consumer emission must succeed");
    let binary = &consumer_artifacts.program;

    assert!(
        !binary
            .functions
            .iter()
            .any(|func| func.symbol.contains("unreachable_dep_helper")),
        "Dead dependency functions must be DCE-pruned"
    );
    assert!(
        binary.functions.iter().any(|func| {
            func.instructions
                .iter()
                .any(|inst| inst.raw_opcode() == OpCode::PackClosure as u8)
        }),
        "Static linking must preserve and remap PackClosure targets for reachable dependency lambdas"
    );
    assert!(
        binary.functions.iter().any(|func| {
            func.instructions
                .iter()
                .any(|inst| inst.raw_opcode() == OpCode::LoadF64 as u8)
        }),
        "Static linking must preserve and relocate LoadF64 constant pool references"
    );
    assert!(
        binary
            .functions
            .iter()
            .all(|func| !func.instructions.iter().any(|inst| {
                inst.raw_opcode() == OpCode::CallExtern as u8
                    || inst.raw_opcode() == OpCode::AllocObjectExtern as u8
            })),
        "All resolved external calls and allocations must be statically rewritten"
    );

    // 3. Execute completely standalone (no dependency binary attached to RuntimeOptions):
    //    local_beta.read_code() = 4 + 5 = 9
    //    doubled = 9 * 2 + 1 = 19
    //    widget = FancyWidget(6, 5) -> area = 30, compute_bias(2) = 2 + 6 = 8, GfxConfig::GLOBAL_SCALE = 3 -> virtual_res = 41
    //    total = 41 + 19 = 60
    let runtime_options = RuntimeOptions::new().with_binary(binary.clone());
    let runtime_context = RuntimeContext::new(runtime_options)
        .expect("Runtime initialization for rich static binary must succeed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let execution_result = virtual_machine.run().unwrap_or_else(|err| {
        let disasm = solix_compilation::phases::emitter::disassembler::disassemble_program(binary);
        panic!("Rich statically linked binary execution failed: {err:?}\nDISASM:\n{disasm}");
    });
    assert_eq!(execution_result, Value::I32(60));
}
