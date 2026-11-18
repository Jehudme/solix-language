//! Differential `-O0` vs `-O3` equivalence fuzzing suite using `proptest`.
//!
//! Generates valid structured Solix programs spanning arithmetic, bitwise operations, loops
//! (`for`, `while`, `do-while`), nested conditionals, recursion, closures, objects/inheritance,
//! interfaces, arrays, tuples, and `try`/`catch`/`finally`, then compiles and executes every
//! program under both `-O0` (`BuildProfile::Debug`) and `-O3` (`BuildProfile::Release`) and
//! asserts bit-identical results, stdout, trap/exception behavior, and zero ARC leaks.

#![forbid(unsafe_code)]

use proptest::prelude::*;
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_runtime::{RuntimeContext, RuntimeOptions, Value, VirtualMachine};

#[derive(Debug, PartialEq)]
struct ExecutionSnapshot {
    outcome: Result<String, String>,
    stdout: Vec<String>,
    total_allocations: u64,
    total_deallocations: u64,
    active_live_objects: usize,
}

fn value_canonical_tag(val: &Value) -> String {
    match val {
        Value::I32(num) => format!("i32:{num}"),
        Value::I64(num) => format!("i64:{num}"),
        Value::F64(num) => format!("f64:{:016x}", num.to_bits()),
        Value::Bool(flag) => format!("bool:{flag}"),
        Value::Null => "null".to_string(),
        Value::Ref(addr) => format!("ref:{addr}"),
        Value::FnPtr(fid) => format!("fnptr:{fid}"),
    }
}

fn compile_and_snapshot(source_code: &str, profile: BuildProfile) -> ExecutionSnapshot {
    let options = CompilationOptions::new()
        .with_profile(profile)
        .with_source("diff_fuzz.slx", source_code);
    let mut context = CompilationContext::new(options);
    let compile_res = context.execute();
    assert!(
        compile_res.is_ok(),
        "Differential test source failed compilation under {profile:?}:\n{source_code}\nErr: {compile_res:?}"
    );
    let binary = context
        .emission_artifacts
        .expect("Emission artifacts expected")
        .program;
    let runtime = RuntimeContext::new(RuntimeOptions::new().with_binary(binary))
        .expect("RuntimeContext build failed");
    let mut vm = VirtualMachine::new(runtime);
    let outcome = match vm.run() {
        Ok(val) => Ok(value_canonical_tag(&val)),
        Err(err) => Err(format!("{err}")),
    };
    let _ = vm.collect_cycles();
    let stdout = vm.stdout.clone();
    let guest_leaks = vm.active_guest_objects();
    let telemetry = vm.heap.telemetry();
    ExecutionSnapshot {
        outcome,
        stdout,
        total_allocations: telemetry.total_allocations_count,
        total_deallocations: telemetry.total_deallocations_count,
        active_live_objects: guest_leaks,
    }
}

fn assert_o0_equals_o3(source_code: &str) {
    let o0 = compile_and_snapshot(source_code, BuildProfile::Debug);
    let o3 = compile_and_snapshot(source_code, BuildProfile::Release);

    if o0.outcome != o3.outcome || o0.stdout != o3.stdout {
        let mut ctx_o3 = CompilationContext::new(
            CompilationOptions::new()
                .with_profile(BuildProfile::Release)
                .with_source("diff_o3.slx", source_code),
        );
        let _ = ctx_o3.execute();
        let dis_o3 = ctx_o3
            .emission_artifacts
            .as_ref()
            .map(|art| solix_compilation::phases::emitter::disassemble_program(&art.program))
            .unwrap_or_default();
        assert_eq!(
            o0.outcome, o3.outcome,
            "Return/outcome divergence between -O0 and -O3!\nSource:\n{source_code}\n-O3 Disasm:\n{dis_o3}"
        );
    }
    assert_eq!(
        o0.stdout, o3.stdout,
        "Stdout divergence between -O0 and -O3!\nSource:\n{source_code}"
    );
    assert_eq!(
        o0.active_live_objects, 0,
        "ARC leak under -O0: {:?}\nSource:\n{source_code}",
        o0
    );
    assert_eq!(
        o3.active_live_objects, 0,
        "ARC leak under -O3: {:?}\nSource:\n{source_code}",
        o3
    );
}

fn build_structured_program(
    seed_a: i32,
    seed_b: i32,
    loop_bound: i32,
    op_mask: u8,
    use_try: bool,
    throw_branch: bool,
) -> String {
    let expr_op = match op_mask % 5 {
        0 => "((acc + idx) ^ (idx * 3)) & 0x7FFFFFFF",
        1 => "((acc - (idx << 1)) | (idx & 7))",
        2 => "((acc * 3) + (idx % 5)) & 0x00FFFFFF",
        3 => "((acc ^ (idx + bias)) >> 1)",
        _ => "((acc + (idx % 7)) - (bias & 3))",
    };

    let try_block = if use_try {
        format!(
            r#"
            int32 guarded = 0;
            try {{
                CounterBox tmp_box = new SubCounterBox(acc, bias);
                guarded = tmp_box.score(loop_limit);
                if ({throw_cond}) {{
                    throw new FuzzError("fuzz_exc");
                }}
                guarded = guarded + 1;
            }} catch (FuzzError err_exc) {{
                println(err_exc.detail);
                guarded = guarded + 100;
            }} finally {{
                println("fin");
                guarded = guarded + 10;
            }}
            acc = acc + guarded;
            "#,
            throw_cond = if throw_branch { "true" } else { "false" }
        )
    } else {
        "CounterBox tmp_box = new SubCounterBox(acc, bias); acc = acc + tmp_box.score(loop_limit);"
            .to_string()
    };

    format!(
        r#"
        native void println(string text);

        interface Scorable {{
            int32 score(int32 limit);
        }}

        class FuzzError {{
            public string detail;
            public FuzzError(string detail_msg) {{
                this.detail = detail_msg;
            }}
        }}

        class CounterBox implements Scorable {{
            public int32 base_val;
            public CounterBox() {{
                this.base_val = 0;
            }}
            public CounterBox(int32 initial_val) {{
                this.base_val = initial_val;
            }}
            public virtual int32 score(int32 limit) {{
                return this.base_val + limit;
            }}
        }}

        class SubCounterBox extends CounterBox {{
            public int32 offset;
            public SubCounterBox(int32 initial_val, int32 offset_val) {{
                this.base_val = initial_val;
                this.offset = offset_val;
            }}
            public override int32 score(int32 limit) {{
                return this.base_val + this.offset + limit;
            }}
        }}

        int32 rec_sum(int32 rem) {{
            if (rem <= 0) {{
                return 0;
            }}
            return rem + rec_sum(rem - 1);
        }}

        int32 main() {{
            int32 acc = {seed_a};
            int32 bias = {seed_b};
            int32 loop_limit = {loop_bound};
            int32[] data = new int32[4];
            data[0] = acc;
            data[1] = bias;
            data[2] = loop_limit;
            data[3] = rec_sum(loop_limit % 6);

            for (int32 idx = 0; idx < loop_limit; idx = idx + 1) {{
                if ((idx % 2) == 0) {{
                    acc = {expr_op};
                }} else {{
                    acc = acc + data[idx % 4];
                }}
            }}

            int32 w_idx = 0;
            while (w_idx < 3) {{
                acc = acc + (w_idx * 2);
                if (w_idx == 1 && (bias & 1) == 1) {{
                    w_idx = w_idx + 1;
                    continue;
                }}
                w_idx = w_idx + 1;
            }}

            int32 do_idx = 0;
            do {{
                acc = acc + do_idx;
                do_idx = do_idx + 1;
            }} while (do_idx < 2);

            (int32, int32) pair = (acc, data[3]);
            int32 captured = bias & 15;
            int32(*)(int32) adjust = (x) => (x + captured) - pair[0];
            acc = acc + adjust(pair[1]);

            {try_block}

            if (acc >= 0) {{
                println("nonneg");
            }} else {{
                println("neg");
            }}
            return acc;
        }}
        "#
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn proptest_differential_o0_vs_o3_equivalence(
        seed_a in -200i32..200i32,
        seed_b in -100i32..100i32,
        loop_bound in 1i32..14i32,
        op_mask in 0u8..10u8,
        use_try in any::<bool>(),
        throw_branch in any::<bool>(),
    ) {
        let source = build_structured_program(
            seed_a,
            seed_b,
            loop_bound,
            op_mask,
            use_try,
            throw_branch,
        );
        assert_o0_equals_o3(&source);
    }
}

#[test]
fn test_differential_o0_vs_o3_recursion_closures_inheritance_exceptions() {
    let source = r#"
        native void println(string text);

        interface Transformer {
            int32 apply(int32 val);
        }

        class MathError {
            public string reason;
            public MathError(string msg) {
                this.reason = msg;
            }
        }

        class LinearTransformer implements Transformer {
            public int32 scale;
            public LinearTransformer() {
                this.scale = 1;
            }
            public LinearTransformer(int32 scale_val) {
                this.scale = scale_val;
            }
            public virtual int32 apply(int32 val) {
                return val * this.scale;
            }
        }

        class OffsetTransformer extends LinearTransformer {
            public int32 bias;
            public OffsetTransformer(int32 scale_val, int32 bias_val) {
                this.scale = scale_val;
                this.bias = bias_val;
            }
            public override int32 apply(int32 val) {
                return (val * this.scale) + this.bias;
            }
        }

        int32 fib(int32 num) {
            if (num <= 1) {
                return num;
            }
            return fib(num - 1) + fib(num - 2);
        }

        int32 tail_acc(int32 count, int32 acc) {
            if (count <= 0) {
                return acc;
            }
            return tail_acc(count - 1, acc + count);
        }

        int32 risky_div(int32 numerator, int32 denom) {
            if (denom == 0) {
                throw new MathError("zero_denom");
            }
            return numerator / denom;
        }

        int32 main() {
            Transformer[] stages = new Transformer[2];
            stages[0] = new LinearTransformer(3);
            stages[1] = new OffsetTransformer(2, 7);

            int32 total = fib(8) + tail_acc(15, 0);
            for (int32 idx = 0; idx < 6; idx = idx + 1) {
                Transformer stage = stages[idx % 2];
                total = total + stage.apply(idx);
            }

            try {
                total = total + risky_div(40, 5);
                total = total + risky_div(10, 0);
            } catch (MathError err_obj) {
                println(err_obj.reason);
                total = total + 50;
            } finally {
                println("cleanup");
                total = total + 1;
            }

            int64 wide = (int64)total * 2;
            float64 flt = (float64)total * 0.5;
            if (wide > 0 && flt > 0.0) {
                println("ok_num");
            }
            return total;
        }
    "#;
    assert_o0_equals_o3(source);
}
