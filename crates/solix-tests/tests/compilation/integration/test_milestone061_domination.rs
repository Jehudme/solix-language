//! Milestone 061 Conformance Tests: Superinstructions, Typed Array Packing,
//! FLAG_MOVE Ownership Transfer, Multi-Binary Relocation, Escape/Unpack Soundness & Telemetry.

#![forbid(unsafe_code)]

use solix_common::bytecode::OpCode;
use solix_common::bytecode::instruction::FLAG_MOVE;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_runtime::context::RuntimeContext;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::vm::VirtualMachine;
use solix_tests::common::compile_source;

#[test]
fn test_positive_typed_array_compact_element_sizes() {
    let source_code = r#"
        int32 main() {
            bool[] flags = new bool[100001];
            int32[] counts = new int32[3600];
            flags[0] = true;
            flags[100000] = true;
            counts[0] = 40;
            counts[3599] = 2;
            int32 total = counts[0] + counts[3599];
            if (flags[0] && flags[100000]) {
                return total;
            }
            return 0;
        }
    "#;

    let options = CompilationOptions::new().with_source("typed_arrays.slx", source_code);
    let mut context = CompilationContext::new(options);
    context.execute().expect("Compilation must succeed");
    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Artifacts must exist");

    let main_fn = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol == "main" || func.symbol.ends_with("::main"))
        .expect("main function must exist");
    let elem_sizes: Vec<(u8, u16)> = main_fn
        .instructions
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::AllocArray as u8)
        .map(|inst| (inst.flags(), inst.src2()))
        .collect();
    assert!(
        elem_sizes.iter().any(|&(tag, sz)| tag == 11 && sz == 1),
        "bool[] must emit elem_size == 1, got {elem_sizes:?}"
    );
    assert!(
        elem_sizes.iter().any(|&(tag, sz)| tag == 3 && sz == 4),
        "int32[] must emit elem_size == 4, got {elem_sizes:?}"
    );

    let runtime_context =
        RuntimeContext::new(RuntimeOptions::new().with_binary(artifacts.program.clone()))
            .expect("Runtime context must build");
    let mut vm = VirtualMachine::new(runtime_context);
    let result = vm.run().expect("Execution must succeed");
    assert_eq!(result, Value::I32(42));
    assert!(
        vm.heap.telemetry().peak_allocated_bytes < 160_000,
        "Compact bool[100001] (100KB) + int32[3600] (14.4KB) must stay under 160KB, got {}",
        vm.heap.telemetry().peak_allocated_bytes
    );
    assert_eq!(vm.active_guest_objects(), 0);
}

#[test]
fn test_positive_set_field_move_elides_cycle_collector_candidates() {
    let source_code = r#"
        class TreeNode {
            public int32 val;
            public TreeNode left;
            public TreeNode right;

            public TreeNode(int32 val) {
                this.val = val;
            }
        }

        TreeNode build_tree(int32 depth, int32 item) {
            TreeNode node = new TreeNode(item);
            if (depth > 0) {
                node.left = build_tree(depth - 1, item * 2);
                node.right = build_tree(depth - 1, item * 2 + 1);
            }
            return node;
        }

        int32 sum_tree(TreeNode node) {
            if (node == null) {
                return 0;
            }
            return node.val + sum_tree(node.left) + sum_tree(node.right);
        }

        int32 main() {
            TreeNode root = build_tree(4, 1);
            return sum_tree(root);
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("tree_move_store.slx", source_code)
        .with_inline_threshold(0);
    let mut context = CompilationContext::new(options);
    context.execute().expect("Compilation must succeed");
    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Artifacts must exist");

    let build_fn = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("build_tree"))
        .expect("build_tree function must exist");
    assert!(
        build_fn
            .instructions
            .iter()
            .any(|inst| inst.raw_opcode() == OpCode::SetField as u8
                && (inst.flags() & FLAG_MOVE) != 0),
        "Expected SetField with FLAG_MOVE in build_tree: {:?}",
        build_fn.instructions
    );

    let runtime_options = RuntimeOptions::new()
        .with_binary(artifacts.program.clone())
        .with_cycle_collector(true)
        .with_cycle_threshold(16);
    let runtime_context = RuntimeContext::new(runtime_options).expect("RuntimeContext must build");
    let mut vm = VirtualMachine::new(runtime_context);
    let res = vm.run().expect("Execution must succeed");
    assert_eq!(res, Value::I32(496));
    assert_eq!(vm.arc_cycle_collector_runs, 0);
    assert_eq!(vm.arc_cycle_collector_candidates, 0);
    assert_eq!(vm.active_guest_objects(), 0);
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_positive_multi_binary_interface_dispatch_and_array_cast() {
    let primary_source = r#"
        class PrimaryPlaceholder {
            public int32 unused;
            public PrimaryPlaceholder() {
                this.unused = 1;
            }
        }
        int32 primary_entry() {
            PrimaryPlaceholder placeholder = new PrimaryPlaceholder();
            return placeholder.unused;
        }
    "#;

    let secondary_source = r#"
        interface ICalc {
            int32 eval(int32 input);
        }

        class Multiplier implements ICalc {
            public int32 factor;
            public Multiplier(int32 factor) {
                this.factor = factor;
            }
            public int32 eval(int32 input) {
                return input * this.factor;
            }
        }

        int32 run_secondary() {
            Multiplier[] items = new Multiplier[2];
            items[0] = new Multiplier(3);
            items[1] = new Multiplier(4);
            ICalc first = items[0];
            ICalc second = items[1];
            int32 acc = first.eval(5) + second.eval(7);
            if (items[0] instanceof ICalc) {
                return acc;
            }
            return -1;
        }
    "#;

    let primary_bin = compile_source(primary_source);
    let secondary_bin = compile_source(secondary_source);

    let options = RuntimeOptions::new()
        .with_binary(primary_bin)
        .with_binary(secondary_bin)
        .with_entry_point("run_secondary");
    let runtime_context = RuntimeContext::new(options).expect("RuntimeContext must build");
    let mut vm = VirtualMachine::new(runtime_context);
    let result = vm.run().expect("Secondary interface dispatch must succeed");
    assert_eq!(result, Value::I32(3 * 5 + 4 * 7));
    assert_eq!(vm.active_guest_objects(), 0);
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_negative_borrowed_alias_exception_unwind_no_double_free() {
    let source_code = r#"
        class UnwindTestError {
            public int32 code;
            public UnwindTestError(int32 code) {
                this.code = code;
            }
        }

        class Worker {
            public virtual int32 compute(int32 val) {
                return val + 1;
            }
        }

        int32 main() {
            Worker[] workers = new Worker[2];
            workers[0] = new Worker();
            workers[1] = new Worker();
            int32 sum = 0;
            for (int32 idx = 0; idx < 4; idx = idx + 1) {
                Worker borrowed = workers[idx % 2];
                try {
                    sum = sum + borrowed.compute(idx);
                    if (idx == 2) {
                        throw new UnwindTestError(99);
                    }
                } catch (UnwindTestError caught) {
                    sum = sum + caught.code;
                }
            }
            return sum;
        }
    "#;

    let program = compile_source(source_code);
    let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
        .expect("RuntimeContext must build");
    let mut vm = VirtualMachine::new(context);
    let res = vm
        .run()
        .expect("Unwind with borrowed alias must not double-free");
    // (0+1) + (1+1) + (2+1) + 99 + (3+1) = 1 + 2 + 3 + 99 + 4 = 109
    assert_eq!(res, Value::I32(109));
    assert_eq!(vm.active_guest_objects(), 0);
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_negative_unpack_captures_reassignment_preserves_decref() {
    let source_code = r#"
        class BoxedPayload {
            public int32 amount;
            public BoxedPayload(int32 amount) {
                this.amount = amount;
            }
        }

        int32 run_capture_drop(bool reset_flag) {
            BoxedPayload captured = new BoxedPayload(25);
            int32(*)(bool) lambda_fn = (flag) => {
                if (flag) {
                    int32 out = captured.amount + 17;
                    captured = null;
                    return out;
                }
                return captured.amount;
            };
            return lambda_fn(reset_flag);
        }

        int32 main() {
            return run_capture_drop(true);
        }
    "#;

    let options = CompilationOptions::new()
        .with_source("unpack_capture_drop.slx", source_code)
        .with_inline_threshold(0);
    let mut context = CompilationContext::new(options);
    context.execute().expect("Compilation must succeed");
    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Artifacts must exist");

    let runtime_context =
        RuntimeContext::new(RuntimeOptions::new().with_binary(artifacts.program.clone()))
            .expect("RuntimeContext must build");
    let mut vm = VirtualMachine::new(runtime_context);
    let res = vm.run().expect("Execution must succeed");
    assert_eq!(res, Value::I32(42));
    assert_eq!(vm.active_guest_objects(), 0);
    assert!(vm.verify_clean_memory().is_ok());
}

#[test]
fn test_positive_active_guest_objects_excludes_interned_strings() {
    let source_code = r#"
        int32 main() {
            string first_lit = "RESULT: ok";
            string second_lit = "unused literal";
            if (first_lit != second_lit) {
                return 1;
            }
            return 0;
        }
    "#;
    let program = compile_source(source_code);
    let context = RuntimeContext::new(RuntimeOptions::new().with_binary(program))
        .expect("RuntimeContext must build");
    let mut vm = VirtualMachine::new(context);
    let res = vm.run().expect("Execution must succeed");
    assert_eq!(res, Value::I32(1));
    assert_eq!(vm.active_guest_objects(), 0);
    assert_eq!(vm.get_metric_by_id(5), 0);
}
