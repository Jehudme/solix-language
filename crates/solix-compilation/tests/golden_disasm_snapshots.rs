//! Golden `.slxasm` disassembly and instruction-budget snapshot tests for `-O3` optimizations.
//!
//! Ensures critical optimization transformations preserve expected `.slxasm` instruction shapes,
//! mnemonic formatting, and strict maximum instruction-count ceilings.

#![forbid(unsafe_code)]

use solix_common::bytecode::CompiledFunction;
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_compilation::phases::emitter::disassemble_function;
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

fn compile_o3_and_find(
    source_name: &str,
    source_code: &str,
    target_symbol: &str,
) -> (CompilationContext, CompiledFunction, String, Value) {
    let options = CompilationOptions::new()
        .with_profile(BuildProfile::Release)
        .with_source(source_name, source_code);
    let mut context = CompilationContext::new(options);
    let result = context.execute();
    assert!(
        result.is_ok(),
        "Compilation failed for {source_name}: {result:?}"
    );

    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Emission artifacts must be present");
    let func = artifacts
        .program
        .functions
        .iter()
        .find(|candidate| candidate.symbol.contains(target_symbol))
        .unwrap_or_else(|| panic!("Function '{target_symbol}' not found in {source_name}"))
        .clone();
    let disasm = disassemble_function(&func, &artifacts.program.constant_pool);

    let runtime = RuntimeContext::new(RuntimeOptions::new().with_binary(artifacts.program.clone()))
        .expect("RuntimeContext build failed");
    let mut vm = VirtualMachine::new(runtime);
    let execution_value = vm.run().expect("VM execution failed");
    let _ = vm.collect_cycles();
    let telemetry = vm.heap.telemetry();
    assert_eq!(
        telemetry.active_live_objects, 0,
        "Expected zero active live objects for {source_name}, got {telemetry:?}"
    );
    assert_eq!(
        telemetry.total_allocations_count, telemetry.total_deallocations_count,
        "Expected balanced ARC alloc/dealloc counts for {source_name}, got {telemetry:?}"
    );

    (context, func, disasm, execution_value)
}

#[test]
fn golden_snapshot_binary_additive_self_recursion_accumulator() {
    let source = r#"
        int32 fib(int32 num) {
            if (num <= 1) {
                return num;
            }
            return fib(num - 1) + fib(num - 2);
        }
        int32 main() {
            return fib(10);
        }
    "#;
    let (_ctx, fib_fn, disasm, value) = compile_o3_and_find("golden_fib.slx", source, "fib");
    assert_eq!(value, Value::I32(55));
    assert!(
        fib_fn.instructions.len() <= 10,
        "Binary self-recursion accumulator regression in fib(): len={} > 10\n{disasm}",
        fib_fn.instructions.len()
    );
    let call_direct_count = disasm.matches("CALL_DIRECT").count();
    assert_eq!(
        call_direct_count, 1,
        "Expected hybrid tail-accumulator lowering with exactly 1 CALL_DIRECT in fib():\n{disasm}"
    );
    assert!(
        disasm.contains("JUMP_IF_I32_IMM") && disasm.contains("SUB_I32_IMM"),
        "Expected JUMP_IF_I32_IMM and SUB_I32_IMM in lowered fib() loop:\n{disasm}"
    );
}

#[test]
fn golden_snapshot_for_loop_i32_and_add_band_i32_fusion() {
    let source = r#"
        int32 main() {
            int32 total = 0;
            for (int32 idx = 0; idx < 100; idx = idx + 1) {
                total = (total + (idx ^ 3)) & 0x7FFFFFFF;
            }
            return total;
        }
    "#;
    let (_ctx, main_fn, disasm, value) =
        compile_o3_and_find("golden_for_addband.slx", source, "main");
    let expected: i32 = (0..100).fold(0i32, |acc, idx| (acc.wrapping_add(idx ^ 3)) & 0x7FFF_FFFF);
    assert_eq!(value, Value::I32(expected));
    assert!(
        main_fn.instructions.len() <= 10,
        "ForLoopI32 + AddBandI32 budget exceeded: len={} > 10\n{disasm}",
        main_fn.instructions.len()
    );
    assert!(
        disasm.contains("FOR_LOOP_I32") && disasm.contains("ADD_BAND_I32"),
        "Expected FOR_LOOP_I32 and ADD_BAND_I32 in disassembly:\n{disasm}"
    );
}

#[test]
fn golden_snapshot_mod_i32_imm_superinstruction() {
    let source = r#"
        int32 main() {
            int32 acc = 0;
            for (int32 idx = 1; idx < 50; idx = idx + 1) {
                acc = acc + (idx % 7);
            }
            return acc;
        }
    "#;
    let (_ctx, main_fn, disasm, value) = compile_o3_and_find("golden_mod_imm.slx", source, "main");
    let expected: i32 = (1..50).map(|idx| idx % 7).sum();
    assert_eq!(value, Value::I32(expected));
    assert!(
        main_fn.instructions.len() <= 8,
        "ModI32Imm loop instruction budget exceeded: len={} > 8\n{disasm}",
        main_fn.instructions.len()
    );
    assert!(
        disasm.contains("MOD_I32_IMM") && disasm.contains("FOR_LOOP_I32"),
        "Expected MOD_I32_IMM and FOR_LOOP_I32 superinstructions in disassembly:\n{disasm}"
    );
}

#[test]
fn golden_snapshot_borrowed_local_arc_elision() {
    let source = r#"
        class BoxNode {
            public int32 val;
            public BoxNode(int32 initial_val) {
                this.val = initial_val;
            }
        }
        int32 main() {
            BoxNode[] items = new BoxNode[3];
            items[0] = new BoxNode(10);
            items[1] = new BoxNode(20);
            items[2] = new BoxNode(30);
            int32 total = 0;
            for (int32 idx = 0; idx < 12; idx = idx + 1) {
                BoxNode node = items[idx % 3];
                total = total + node.val;
            }
            return total;
        }
    "#;
    let (_ctx, main_fn, disasm, value) =
        compile_o3_and_find("golden_arc_elision.slx", source, "main");
    assert_eq!(value, Value::I32(240));
    assert!(
        main_fn.instructions.len() <= 45,
        "Borrowed local loop instruction budget exceeded: len={}\n{disasm}",
        main_fn.instructions.len()
    );
    let backedge_idx = main_fn
        .instructions
        .iter()
        .rposition(|inst| {
            let line = format!("{inst:?}");
            line.contains("ForLoopI32")
        })
        .expect("ForLoopI32 loop backedge expected");
    let loop_start_idx =
        (backedge_idx as i32 + (main_fn.instructions[backedge_idx].src2() as i16 as i32)) as usize;
    let loop_arc_count = disasm
        .lines()
        .filter(|line| {
            line.trim_start()
                .starts_with(|ch: char| ch.is_ascii_digit())
        })
        .skip(loop_start_idx)
        .take(backedge_idx - loop_start_idx + 1)
        .filter(|line| line.contains("INC_REF") || line.contains("DEC_REF"))
        .count();
    assert_eq!(
        loop_arc_count, 0,
        "Expected borrowed local ARC elision inside loop body:\n{disasm}"
    );
}

#[test]
fn golden_snapshot_leaf_inlining_and_diamond_closure() {
    let source = r#"
        int32 combine(int32 lhs, int32 rhs) {
            return (lhs * 3) + rhs;
        }
        int32 add_one(int32 val) { return val + 1; }
        int32 mul_two(int32 val) { return val * 2; }
        int32 main() {
            int32(*)(int32) callback;
            int32 seed = 11;
            if ((seed & 1) == 1) {
                callback = add_one;
            } else {
                callback = mul_two;
            }
            return combine(callback(seed), 5);
        }
    "#;
    let (_ctx, main_fn, disasm, value) =
        compile_o3_and_find("golden_inline_closure.slx", source, "main");
    // (11 + 1) * 3 + 5 = 41
    assert_eq!(value, Value::I32(41));
    assert!(
        main_fn.instructions.len() <= 20,
        "Leaf inline + diamond closure budget exceeded: len={}\n{disasm}",
        main_fn.instructions.len()
    );
    assert!(
        !disasm.contains("CALL_DIRECT"),
        "Expected combine() leaf call to be inlined into main():\n{disasm}"
    );
}

#[test]
fn golden_snapshot_virtual_and_interface_dispatch() {
    let source = r#"
        interface Evaluator {
            int32 eval(int32 input_val);
        }
        class BaseEval implements Evaluator {
            public virtual int32 eval(int32 input_val) {
                return input_val + 5;
            }
        }
        class DerivedEval extends BaseEval {
            public override int32 eval(int32 input_val) {
                return input_val * 4;
            }
        }
        int32 run_poly(Evaluator target, int32 seed) {
            return target.eval(seed);
        }
        int32 run_virtual(BaseEval target, int32 seed) {
            return target.eval(seed);
        }
        int32 main() {
            BaseEval poly_ref = new DerivedEval();
            return run_poly(poly_ref, 6) + run_virtual(poly_ref, 7);
        }
    "#;
    let (ctx, _main_fn, _main_disasm, value) =
        compile_o3_and_find("golden_dispatch.slx", source, "main");
    // 6 * 4 + 7 * 4 = 24 + 28 = 52
    assert_eq!(value, Value::I32(52));

    let artifacts = ctx.emission_artifacts.as_ref().unwrap();
    let poly_fn = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("run_poly"))
        .expect("run_poly expected");
    let poly_disasm = disassemble_function(poly_fn, &artifacts.program.constant_pool);
    assert!(
        poly_fn.instructions.len() <= 6,
        "Interface dispatch run_poly budget exceeded: len={} > 6\n{poly_disasm}",
        poly_fn.instructions.len()
    );
    assert!(
        poly_disasm.contains("CALL_INTERFACE"),
        "Expected CALL_INTERFACE mnemonic in run_poly disassembly:\n{poly_disasm}"
    );

    let virt_fn = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("run_virtual"))
        .expect("run_virtual expected");
    let virt_disasm = disassemble_function(virt_fn, &artifacts.program.constant_pool);
    assert!(
        virt_fn.instructions.len() <= 6,
        "Virtual dispatch run_virtual budget exceeded: len={} > 6\n{virt_disasm}",
        virt_fn.instructions.len()
    );
    assert!(
        virt_disasm.contains("CALL_VIRTUAL"),
        "Expected CALL_VIRTUAL mnemonic in run_virtual disassembly:\n{virt_disasm}"
    );
}
