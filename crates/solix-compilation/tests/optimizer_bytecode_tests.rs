//! Comprehensive tests for [`BytecodeOptimizerPhase`].
//!
//! Validates:
//! 1. `LoadI32` + `AddI32` fusion into `AddI32Imm`.
//! 2. `LoadI32` + `SubI32` fusion into `SubI32Imm`.
//! 3. Integer comparisons + `JumpIfFalse` fusion into `JumpIf*I32`.
//! 4. Redundant `Mov` elimination.
//! 5. Loop-invariant constant hoisting.
//! 6. Exact exception handler PC ranges and execution correctness.

#![forbid(unsafe_code)]

use solix_common::bytecode::OpCode;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

/// Helper to compile source code and return the compilation context and VM execution result.
fn compile_and_execute(source_code: &str) -> (CompilationContext, Value) {
    let options = CompilationOptions::new()
        .with_source("test.slx", source_code)
        .with_inline_threshold(0); // Disable AST inliner to isolate bytecode optimization
    let mut context = CompilationContext::new(options);
    let compile_result = context.execute();
    assert!(
        compile_result.is_ok(),
        "Compilation failed: {compile_result:?}"
    );

    let artifacts = context
        .emission_artifacts
        .as_ref()
        .expect("Emission artifacts must be present");
    let runtime_options = RuntimeOptions::new().with_binary(artifacts.program.clone());
    let runtime_context =
        RuntimeContext::new(runtime_options).expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let execution_result = virtual_machine.run().expect("VM execution failed");
    (context, execution_result)
}

#[test]
fn test_fuse_load_i32_and_add_i32_into_add_i32_imm() {
    let source_code = r#"
        int32 main() {
            int32 base_val = 100;
            int32 sum_val = base_val + 42;
            return sum_val;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(142));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let has_add_imm = main_func
        .instructions
        .iter()
        .any(|inst| inst.raw_opcode() == OpCode::AddI32Imm as u8);
    assert!(
        has_add_imm,
        "Expected AddI32Imm superinstruction in main function bytecode"
    );
}

#[test]
fn test_fuse_load_i32_and_sub_i32_into_sub_i32_imm() {
    let source_code = r#"
        int32 main() {
            int32 base_val = 100;
            int32 diff_val = base_val - 25;
            return diff_val;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(75));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let has_sub_imm = main_func
        .instructions
        .iter()
        .any(|inst| inst.raw_opcode() == OpCode::SubI32Imm as u8);
    assert!(
        has_sub_imm,
        "Expected SubI32Imm superinstruction in main function bytecode"
    );
}

#[test]
fn test_fuse_comparison_and_jump_if_false_into_superinstruction() {
    let source_code = r#"
        int32 main() {
            int32 counter = 0;
            while (counter < 10) {
                counter = counter + 1;
            }
            return counter;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(10));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let has_super_jump = main_func.instructions.iter().any(|inst| {
        let opcode = inst.raw_opcode();
        opcode >= OpCode::JumpIfLtI32 as u8 && opcode <= OpCode::JumpIfNeqI32 as u8
    });
    assert!(
        has_super_jump,
        "Expected fused JumpIf*I32 superinstruction in while loop header"
    );
}

#[test]
fn test_eliminate_redundant_mov_instructions() {
    let source_code = r#"
        int32 calculate(int32 param_one, int32 param_two) {
            int32 local_result = param_one * param_two;
            return local_result;
        }
        int32 main() {
            return calculate(6, 7);
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(42));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let calc_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("calculate"))
        .expect("calculate function must exist");

    // Without optimization, MulI32 would target a temp register followed by Mov local_result, temp
    // With Mov elimination, MulI32 targets local_result directly
    let mov_count = calc_func
        .instructions
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::Mov as u8)
        .count();
    assert_eq!(
        mov_count, 0,
        "Expected redundant Mov instructions to be eliminated"
    );
}

#[test]
fn test_loop_invariant_constant_hoisting() {
    let source_code = r#"
        int32 main() {
            int32 accumulator = 0;
            for (int32 iteration = 0; iteration < 100; iteration = iteration + 1) {
                int32 constant_factor = 7;
                accumulator = accumulator + (iteration * constant_factor);
            }
            return accumulator > 0 ? 0 : 1;
        }
    "#;
    let (_context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_exception_handler_table_accuracy_after_optimization() {
    let source_code = r#"
        class CustomError {}

        int32 main() {
            int32 status_flag = 0;
            try {
                int32 computed_val = 10 + 20;
                if (computed_val == 30) {
                    throw new CustomError();
                }
                status_flag = 1;
            } catch (CustomError caught_err) {
                status_flag = 42;
            }
            return status_flag;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(42));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    assert!(
        !main_func.exception_handlers.is_empty(),
        "Exception handler entries must be preserved"
    );
    for handler in &main_func.exception_handlers {
        assert!(
            handler.try_start_pc < handler.try_end_pc,
            "try_start_pc must precede try_end_pc"
        );
        assert!(
            (handler.handler_pc as usize) < main_func.instructions.len(),
            "handler_pc must be within valid instruction range"
        );
    }
}

#[test]
fn test_positive_peephole_fuses_with_reused_scratch_register() {
    let source_code = r#"
        int32 sum_filtered(int32 limit) {
            int32 total = 0;
            for (int32 index = 0; index < limit; index = index + 1) {
                if (index < 50) {
                    total = total + 3;
                } else {
                    total = total + 7;
                }
            }
            return total;
        }
        int32 main() {
            return sum_filtered(100);
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(50 * 3 + 50 * 7));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let sum_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("sum_filtered"))
        .expect("sum_filtered function must exist");

    let add_imm_count = sum_func
        .instructions
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::AddI32Imm as u8)
        .count();
    let super_jump_count = sum_func
        .instructions
        .iter()
        .filter(|inst| {
            let raw = inst.raw_opcode();
            raw >= OpCode::JumpIfLtI32 as u8 && raw <= OpCode::JumpIfNeqI32 as u8
        })
        .count();

    assert!(
        add_imm_count >= 3,
        "Expected index + 1, total + 3, and total + 7 to all fuse into AddI32Imm, found {add_imm_count}"
    );
    assert!(
        super_jump_count >= 2,
        "Expected both loop header and inner branch comparisons to fuse into JumpIf*I32, found {super_jump_count}"
    );
}

#[test]
fn test_positive_loop_hoisting_moves_constant_loads_before_loop() {
    let source_code = r#"
        float64 accumulate_scaled(int32 iterations) {
            float64 acc = 0.0;
            for (int32 index = 0; index < iterations; index = index + 1) {
                acc = acc + 1.25;
            }
            return acc;
        }
        int32 main() {
            float64 result = accumulate_scaled(100);
            return result == 125.0 ? 0 : 1;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(0));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let loop_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("accumulate_scaled"))
        .expect("accumulate_scaled function must exist");

    let backedge = loop_func
        .instructions
        .iter()
        .enumerate()
        .find_map(|(pc_idx, inst)| {
            if inst.raw_opcode() == OpCode::Jump as u8 && inst.jump_offset() < 0 {
                let target_pc = (pc_idx as i32 + inst.jump_offset()) as usize;
                Some((target_pc, pc_idx))
            } else {
                None
            }
        })
        .expect("Expected backward Jump forming a loop");

    let loop_body_loads = loop_func.instructions[backedge.0..=backedge.1]
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::LoadF64 as u8)
        .count();
    assert_eq!(
        loop_body_loads, 0,
        "Expected constant LoadF64 inside loop [{}..={}] to be hoisted before loop header",
        backedge.0, backedge.1
    );
}

#[test]
fn test_negative_live_constant_used_multiple_times_preserved_correctly() {
    let source_code = r#"
        int32 compute_multi(int32 start_val, bool flag_val) {
            int32 offset_val = 9;
            int32 updated = start_val + offset_val;
            if (flag_val) {
                updated = updated + offset_val;
            }
            return updated + offset_val;
        }
        int32 main() {
            int32 first_case = compute_multi(10, true);
            int32 second_case = compute_multi(10, false);
            return first_case * 100 + second_case;
        }
    "#;
    let (_context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(37 * 100 + 28));
}

#[test]
fn test_positive_non_virtual_concrete_method_devirtualizes_to_call_direct() {
    let source_code = r#"
        class BoxedCounter {
            private int32 count_val;
            public BoxedCounter(int32 start_val) {
                this.count_val = start_val;
            }
            public int32 add_offset(int32 delta_val) {
                return this.count_val + delta_val;
            }
        }
        int32 main() {
            BoxedCounter holder = new BoxedCounter(30);
            return holder.add_offset(12);
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(42));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let has_virtual = main_func
        .instructions
        .iter()
        .any(|inst| inst.raw_opcode() == OpCode::CallVirtual as u8);
    let direct_count = main_func
        .instructions
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::CallDirect as u8)
        .count();

    assert!(
        !has_virtual,
        "Non-virtual concrete method on guaranteed non-null local should not emit CallVirtual"
    );
    assert!(
        direct_count >= 1,
        "Expected CallDirect emission for non-virtual concrete method call"
    );
}

#[test]
fn test_negative_nullable_receiver_retains_call_virtual_and_traps() {
    let source_code = r#"
        class SafeBox {
            public int32 fixed_num() {
                return 99;
            }
        }
        int32 main() {
            SafeBox maybe_null = null;
            return maybe_null.fixed_num();
        }
    "#;
    let options = CompilationOptions::new()
        .with_source("test_null_receiver.slx", source_code)
        .with_inline_threshold(0);
    let mut context = CompilationContext::new(options);
    context.execute().expect("compilation must succeed");
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");
    assert!(
        main_func
            .instructions
            .iter()
            .any(|inst| inst.raw_opcode() == OpCode::CallVirtual as u8),
        "Nullable receiver must retain CallVirtual null-check semantics"
    );

    let runtime_context = RuntimeContext::new(
        RuntimeOptions::new().with_binary(artifacts.program.clone()),
    )
    .expect("RuntimeContext creation failed");
    let mut virtual_machine = VirtualMachine::new(runtime_context);
    let runtime_err = virtual_machine
        .run()
        .expect_err("null receiver method call must trap cleanly");
    assert!(matches!(
        runtime_err,
        solix_runtime::RuntimeError::NullReferenceAccess
    ));
}

#[test]
fn test_positive_direct_assignment_eliminates_trailing_mov() {
    let source_code = r#"
        int32 accumulate(int32 limit) {
            int32 total = 0;
            for (int32 index = 0; index < limit; index = index + 1) {
                total = total + index;
            }
            return total;
        }
        int32 main() {
            return accumulate(10);
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(45));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let acc_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("accumulate"))
        .expect("accumulate function must exist");

    let mov_count = acc_func
        .instructions
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::Mov as u8)
        .count();
    assert_eq!(
        mov_count, 0,
        "Expected zero Mov instructions in assignment/loop update path, found {:?}",
        acc_func.instructions
    );
}

#[test]
fn test_positive_loop_sra_eliminates_pod_alloc_and_pure_getter_call() {
    let source_code = r#"
        class Box<T> {
            public T value;
            public Box(T initial_val) {
                this.value = initial_val;
            }
            public T get_val() {
                return this.value;
            }
        }
        int32 main() {
            int32 total = 0;
            for (int32 index = 0; index < 100; index = index + 1) {
                Box<int32> first_box = new Box<int32>(index);
                Box<int32> second_box = new Box<int32>(index * 2 + 1);
                total = total + first_box.get_val() + second_box.get_val();
            }
            return total;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");
    let expected: i32 = (0..100).map(|idx| idx + (idx * 2 + 1)).sum();
    assert_eq!(
        execution_result,
        Value::I32(expected),
        "instructions: {:#?}",
        main_func.instructions
    );

    let alloc_count = main_func
        .instructions
        .iter()
        .filter(|inst| inst.raw_opcode() == OpCode::AllocObject as u8)
        .count();
    let call_count = main_func
        .instructions
        .iter()
        .filter(|inst| {
            let raw = inst.raw_opcode();
            raw == OpCode::CallDirect as u8 || raw == OpCode::CallVirtual as u8
        })
        .count();

    assert_eq!(
        alloc_count, 0,
        "Loop-scoped SRA should eliminate all AllocObject instructions in main(), found {:?}",
        main_func.instructions
    );
    assert_eq!(
        call_count, 0,
        "Pure constructor and getter calls should be scalar-replaced in main(), found {:?}",
        main_func.instructions
    );
}

#[test]
fn test_negative_loop_sra_rejects_loop_carried_object_alias() {
    let source_code = r#"
        class Holder {
            public int32 val;
            public Holder(int32 initial_val) {
                this.val = initial_val;
            }
        }
        int32 main() {
            Holder saved = new Holder(5);
            int32 acc = 0;
            for (int32 index = 0; index < 4; index = index + 1) {
                acc = acc + saved.val;
                Holder fresh = new Holder(index + 10);
                saved = fresh;
            }
            return acc + saved.val;
        }
    "#;
    let (context, execution_result) = compile_and_execute(source_code);
    assert_eq!(execution_result, Value::I32(5 + 10 + 11 + 12 + 13));

    let artifacts = context.emission_artifacts.as_ref().unwrap();
    let main_func = artifacts
        .program
        .functions
        .iter()
        .find(|func| func.symbol.contains("main"))
        .expect("main function must exist");

    let has_alloc = main_func
        .instructions
        .iter()
        .any(|inst| inst.raw_opcode() == OpCode::AllocObject as u8);
    assert!(
        has_alloc,
        "Loop-carried object assigned to outer variable must preserve heap AllocObject"
    );
}
