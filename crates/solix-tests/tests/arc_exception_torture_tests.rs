//! Torture test suite for ARC reference counting, cycle collector, exception unwinding, and FFI callbacks.
//!
//! Asserts `total_allocations_count == total_deallocations_count` and `active_live_objects == 0`
//! across deep object DAGs, cyclic graphs, capturing closures, early `return`/`break`/`continue`,
//! exceptions thrown mid-constructor and mid-array construction, and multi-frame native callbacks.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};

fn run_solix_and_assert_zero_leaks(
    source_name: &str,
    source_code: &str,
    profile: BuildProfile,
) -> Value {
    let options = CompilationOptions::new()
        .with_profile(profile)
        .with_source(source_name, source_code);
    let mut context = CompilationContext::new(options);
    let compile_res = context.execute();
    assert!(
        compile_res.is_ok(),
        "Compilation failed for {source_name} ({profile:?}): {compile_res:?}"
    );
    let binary = context
        .emission_artifacts
        .expect("Emission artifacts missing")
        .program;
    let disasm = solix_compilation::phases::emitter::disassemble_program(&binary);
    let runtime = RuntimeContext::new(RuntimeOptions::new().with_binary(binary))
        .expect("RuntimeContext creation failed");
    let mut vm = VirtualMachine::new(runtime);
    let result = vm.run().unwrap_or_else(|err| {
        panic!("VM execution failed for {source_name} ({profile:?}): {err:?}\n{disasm}");
    });
    let _ = vm.collect_cycles();

    let guest_live = vm.active_guest_objects();
    let telemetry = vm.heap.telemetry();
    assert_eq!(
        guest_live, 0,
        "Active live guest objects leaked in {source_name} ({profile:?}): {telemetry:?}, purple_remaining={:?}, cycles_collected={}\n{disasm}",
        vm.purple_buffer, vm.arc_cycles_collected
    );
    // For pure guest allocation programs without interned string literals, total allocs must match deallocs.
    let interned_count = vm.interned_string_addrs.len() as u64;
    assert_eq!(
        telemetry.total_allocations_count,
        telemetry.total_deallocations_count + interned_count,
        "ARC allocation/deallocation imbalance in {source_name} ({profile:?}): {telemetry:?}"
    );
    result
}

fn assert_both_profiles_zero_leaks(source_name: &str, source_code: &str, expected: Value) {
    let val_o0 = run_solix_and_assert_zero_leaks(source_name, source_code, BuildProfile::Debug);
    let val_o3 = run_solix_and_assert_zero_leaks(source_name, source_code, BuildProfile::Release);
    assert_eq!(
        val_o0, expected,
        "Unexpected result in -O0 for {source_name}"
    );
    assert_eq!(
        val_o3, expected,
        "Unexpected result in -O3 for {source_name}"
    );
}

#[test]
fn test_arc_deep_dag_and_linked_chain_reclamation() {
    let source = r#"
        class GraphNode {
            public int32 weight;
            public GraphNode left;
            public GraphNode right;
            public GraphNode(int32 weight_val, GraphNode left_child, GraphNode right_child) {
                this.weight = weight_val;
                this.left = left_child;
                this.right = right_child;
            }
        }

        GraphNode build_shared_dag(int32 depth) {
            if (depth <= 0) {
                return new GraphNode(1, null, null);
            }
            GraphNode shared = build_shared_dag(depth - 1);
            return new GraphNode(depth + shared.weight, shared, shared);
        }

        int32 main() {
            int32 total = 0;
            for (int32 round = 0; round < 5; round = round + 1) {
                GraphNode head = null;
                for (int32 step = 0; step < 120; step = step + 1) {
                    head = new GraphNode(step, head, null);
                }
                GraphNode dag = build_shared_dag(8);
                total = total + head.weight + dag.weight;
            }
            return total;
        }
    "#;
    // (119 + 37) * 5 = 156 * 5 = 780
    assert_both_profiles_zero_leaks("deep_dag.slx", source, Value::I32(780));
}

#[test]
fn test_cycle_collector_self_pair_triad_and_closure_cycles() {
    let source = r#"
        class RingNode {
            public int32 tag;
            public RingNode peer;
            public int32(*)() callback;
            public RingNode(int32 tag_val) {
                this.tag = tag_val;
                this.peer = null;
                this.callback = null;
            }
        }

        int32 spawn_cycles() {
            RingNode alpha = new RingNode(10);
            RingNode beta = new RingNode(20);
            RingNode gamma = new RingNode(30);
            alpha.peer = beta;
            beta.peer = gamma;
            gamma.peer = alpha;

            RingNode self_ref = new RingNode(7);
            self_ref.peer = self_ref;

            RingNode closure_holder = new RingNode(5);
            closure_holder.callback = () => closure_holder.tag + alpha.tag;

            return alpha.tag + beta.tag + gamma.tag + self_ref.tag + closure_holder.callback();
        }

        int32 main() {
            int32 sum = 0;
            for (int32 iter = 0; iter < 40; iter = iter + 1) {
                sum = sum + spawn_cycles();
            }
            return sum;
        }
    "#;
    // (10 + 20 + 30 + 7 + 15) * 40 = 82 * 40 = 3280
    assert_both_profiles_zero_leaks("cycle_torture.slx", source, Value::I32(3280));
}

#[test]
fn test_arc_early_return_break_continue_and_nested_scopes() {
    let source = r#"
        class Payload {
            public int32[] items;
            public Payload(int32 seed) {
                this.items = new int32[4];
                this.items[0] = seed;
                this.items[1] = seed + 1;
            }
        }

        int32 early_exit_helper(int32 limit) {
            Payload outer = new Payload(10);
            for (int32 outer_idx = 0; outer_idx < limit; outer_idx = outer_idx + 1) {
                Payload mid = new Payload(outer_idx);
                if (outer_idx == 2) {
                    continue;
                }
                for (int32 inner_idx = 0; inner_idx < 5; inner_idx = inner_idx + 1) {
                    Payload inner = new Payload(inner_idx);
                    if (inner_idx == 3 && outer_idx == 4) {
                        return outer.items[0] + mid.items[1] + inner.items[0];
                    }
                    if (inner_idx == 1) {
                        break;
                    }
                }
            }
            return outer.items[0];
        }

        int32 main() {
            return early_exit_helper(3) + early_exit_helper(6);
        }
    "#;
    assert_both_profiles_zero_leaks("early_exits.slx", source, Value::I32(20));
}

#[test]
fn test_arc_exception_unwinding_mid_ctor_mid_array_and_nested_rethrow() {
    let source = r#"
        class FaultError {
            public string detail;
            public FaultError(string detail_msg) {
                this.detail = detail_msg;
            }
        }

        class ResourceHolder {
            public int32[] buffer;
            public ResourceHolder(int32 size, bool fail_now) {
                this.buffer = new int32[size];
                this.buffer[0] = 99;
                if (fail_now) {
                    throw new FaultError("ctor_abort");
                }
            }
        }

        int32 nested_thrower(int32 depth) {
            ResourceHolder local_res = new ResourceHolder(8, false);
            int32[][] matrix = new int32[][3];
            matrix[0] = new int32[4];
            if (depth <= 0) {
                ResourceHolder boom = new ResourceHolder(4, true);
                return boom.buffer[0];
            }
            try {
                return nested_thrower(depth - 1) + local_res.buffer[0];
            } catch (FaultError inner_err) {
                ResourceHolder recovery = new ResourceHolder(2, false);
                if (depth == 1) {
                    throw inner_err;
                }
                return recovery.buffer[0] + 1;
            } finally {
                ResourceHolder cleanup = new ResourceHolder(2, false);
                int32 unused = cleanup.buffer[0];
            }
        }

        int32 main() {
            int32 acc = 0;
            for (int32 iter = 0; iter < 15; iter = iter + 1) {
                acc = acc + nested_thrower(3);
            }
            return acc;
        }
    "#;
    // depth 3 calls depth 2, calls depth 1 (rethrows), caught at depth 2 -> 99+1=100, +99=199 per iter -> 199*15=2985
    assert_both_profiles_zero_leaks("exception_unwind.slx", source, Value::I32(2985));
}

#[test]
fn test_multi_frame_native_ffi_callback_exception_unwinds_without_leaks() {
    fn host_alloc_and_invoke(
        vm: &mut VirtualMachine,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        let should_throw = matches!(args.first(), Some(Value::I32(1)));
        let cb_id = vm
            .context
            .symbol_to_function_id
            .get("solix_callback")
            .copied()
            .ok_or_else(|| RuntimeError::FunctionNotFound {
                symbol_or_id: "solix_callback".to_string(),
            })?;
        // Invoke back into Solix VM from inside the native callback.
        let nested_arg = if should_throw {
            Value::I32(1)
        } else {
            Value::I32(0)
        };
        let base_depth = vm.call_frames.len();
        vm.push_frame(cb_id, 0, &[nested_arg])?;
        vm.run_loop(base_depth)?;
        Ok(vm.last_return_value)
    }

    let mut binary = BinaryProgram::new();
    binary
        .constant_pool
        .values
        .push(solix_common::bytecode::ConstantPoolValue::String(
            "host_alloc_and_invoke".to_string(),
        ));

    // Function 0: solix_callback(flag: i32) -> i32 (allocates object array, optionally throws)
    let mut cb_fn = CompiledFunction::new("solix_callback", 0, 1, 4);
    cb_fn.reference_registers = vec![2];
    cb_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 8));
    cb_fn
        .instructions
        .push(Instruction::format_a(OpCode::AllocArray, 0, 2, 1, 0));
    cb_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 3, 1));
    cb_fn
        .instructions
        .push(Instruction::format_a(OpCode::JumpIfNeqI32, 0, 0, 3, 2));
    cb_fn
        .instructions
        .push(Instruction::format_a(OpCode::Throw, 0, 3, 0, 0));
    cb_fn
        .instructions
        .push(Instruction::format_a(OpCode::DecRef, 0, 2, 0, 0));
    cb_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 42));
    cb_fn
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    binary.functions.push(cb_fn);

    // Function 1: host_alloc_and_invoke(flag: i32) -> native stub
    let mut native_stub = CompiledFunction::new("host_alloc_and_invoke", 0, 1, 2);
    native_stub
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    binary.functions.push(native_stub);

    // Function 2: main() -> calls native stub twice (normal + throwing callback across FFI frame)
    let mut main_fn = CompiledFunction::new("main", 0, 0, 4);
    main_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 0));
    main_fn
        .instructions
        .push(Instruction::format_call(OpCode::CallSymbol, 0, 0, 1, 1));
    main_fn
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 2, 1));
    main_fn
        .instructions
        .push(Instruction::format_call(OpCode::CallSymbol, 0, 0, 2, 1));
    main_fn
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
    binary.functions.push(main_fn);
    binary.entry_point_id = Some(2);

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_function("host_alloc_and_invoke", host_alloc_and_invoke);
    let context = RuntimeContext::new(options).expect("RuntimeContext failed");
    let mut vm = VirtualMachine::new(context);
    let run_result = vm.run();
    assert!(
        matches!(run_result, Err(RuntimeError::UserException { .. })),
        "Expected uncaught exception across native callback boundary, got: {run_result:?}"
    );
    let _ = vm.collect_cycles();

    let telemetry = vm.heap.telemetry();
    let interned_count = vm.interned_string_addrs.len() as u64;
    assert_eq!(
        telemetry.total_allocations_count,
        telemetry.total_deallocations_count + interned_count
    );
    assert_eq!(vm.active_guest_objects(), 0);
}
