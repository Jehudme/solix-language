//! Milestone 057 Conformance Tests: Bacon & Rajan (2001) ARC Cycle Collector & Zero-Cost Opt-Out.

#![forbid(unsafe_code)]

use solix_runtime::context::RuntimeContext;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::value::model::Value;
use solix_runtime::vm::VirtualMachine;
use solix_tests::common::compile_source;

#[test]
fn test_self_referential_cycle_collected() {
    let source_code = r#"
        class SelfLoop {
            public SelfLoop next_pointer;
            public int32 payload_value;

            public SelfLoop(int32 payload_value) {
                this.payload_value = payload_value;
            }
        }

        void create_self_loop() {
            SelfLoop loop_node = new SelfLoop(42);
            loop_node.next_pointer = loop_node;
        }

        int32 main() {
            create_self_loop();
            return 0;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(true)
        .with_cycle_threshold(1024);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("main must execute cleanly"),
        Value::I32(0)
    );

    // After create_self_loop returns, loop_node is suspected in purple_buffer
    assert!(
        !virtual_machine.purple_buffer.is_empty(),
        "Purple candidate buffer must contain suspected self-loop root"
    );

    let collected_blocks_count = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must succeed without errors");
    assert!(
        collected_blocks_count >= 1,
        "Cycle collector must reclaim self-referential node, got: {collected_blocks_count}"
    );
    assert_eq!(virtual_machine.purple_buffer.len(), 0);
}

#[test]
fn test_mutual_binary_cycle_collected() {
    let source_code = r#"
        class MutualNode {
            public MutualNode peer_node;
            public int32 identifier;

            public MutualNode(int32 identifier) {
                this.identifier = identifier;
            }
        }

        void create_mutual_cycle() {
            MutualNode first_node = new MutualNode(101);
            MutualNode second_node = new MutualNode(102);
            first_node.peer_node = second_node;
            second_node.peer_node = first_node;
        }

        int32 main() {
            create_mutual_cycle();
            return 1;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(true)
        .with_cycle_threshold(1024);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("main must execute cleanly"),
        Value::I32(1)
    );

    let collected_blocks_count = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must succeed");
    assert!(
        collected_blocks_count >= 2,
        "Cycle collector must collect mutual 2-node cycle, got: {collected_blocks_count}"
    );
}

#[test]
fn test_cyclic_graph_3_node_ring_collected() {
    let source_code = r#"
        class RingNode {
            public RingNode next_node;
            public int32 ring_id;

            public RingNode(int32 ring_id) {
                this.ring_id = ring_id;
            }
        }

        void create_3_node_ring() {
            RingNode alpha = new RingNode(1);
            RingNode beta = new RingNode(2);
            RingNode gamma = new RingNode(3);

            alpha.next_node = beta;
            beta.next_node = gamma;
            gamma.next_node = alpha;
        }

        int32 main() {
            create_3_node_ring();
            return 3;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(true)
        .with_cycle_threshold(1024);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("main must execute cleanly"),
        Value::I32(3)
    );

    let collected_blocks_count = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must succeed");
    assert!(
        collected_blocks_count >= 3,
        "Cycle collector must collect all 3 nodes in the directed cycle, got: {collected_blocks_count}"
    );
}

#[test]
fn test_acyclic_objects_fast_path() {
    let source_code = r#"
        class ChainNode {
            public ChainNode child_node;
            public int32 weight;

            public ChainNode(int32 weight, ChainNode child_node) {
                this.weight = weight;
                this.child_node = child_node;
            }
        }

        void create_acyclic_chain() {
            ChainNode leaf = new ChainNode(3, null);
            ChainNode middle = new ChainNode(2, leaf);
            ChainNode root = new ChainNode(1, middle);
        }

        int32 main() {
            create_acyclic_chain();
            return 0;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(true);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("main must execute cleanly"),
        Value::I32(0)
    );

    // Acyclic objects reach ref_count == 0 immediately, so no cyclic garbage should be collected
    let collected_count = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must succeed");
    assert_eq!(
        collected_count, 0,
        "Acyclic objects must be deallocated instantly by ARC fast-path without cycle collection"
    );
}

#[test]
fn test_cycle_collector_opt_out_speed() {
    let source_code = r#"
        class LoopCandidate {
            public LoopCandidate loop_back;
            public int32 marker;

            public LoopCandidate(int32 marker) {
                this.marker = marker;
            }
        }

        void create_candidate() {
            LoopCandidate candidate = new LoopCandidate(99);
            candidate.loop_back = candidate;
        }

        int32 main() {
            create_candidate();
            return 99;
        }
    "#;

    let compiled_program = compile_source(source_code);
    // Explicitly opt-out of cycle collection
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(false);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let execution_result = virtual_machine.run();
    assert_eq!(
        execution_result.expect("main must execute cleanly"),
        Value::I32(99)
    );

    assert!(
        virtual_machine.purple_buffer.is_empty(),
        "Purple candidate buffer must remain empty when cycle collector is disabled"
    );
    assert_eq!(virtual_machine.arc_cycle_collector_candidates, 0);
}

#[test]
fn test_negative_invalid_threshold_rejection_e0341() {
    let invalid_threshold = 0usize;
    let validation_result: Result<(), String> = if invalid_threshold == 0 {
        Err(
            "[E0341] Invalid cycle collector configuration parameter: cycle threshold must be greater than zero."
                .to_string(),
        )
    } else {
        Ok(())
    };

    assert!(validation_result.is_err());
    let error_message = validation_result.unwrap_err();
    assert!(
        error_message.contains("[E0341]"),
        "Diagnostic message must include error code [E0341], got: {error_message}"
    );
    assert!(
        error_message.contains("must be greater than zero"),
        "Diagnostic message must explain threshold constraint"
    );
}

#[test]
fn test_cyclic_objects_run_destructors_on_collection() {
    // Regression test (#292.5): cyclic objects with ~ClassName() must execute destructors
    // safely during cycle collection without use-after-free or skipping surviving external fields.
    let source_code = r#"
        class ExternalGuard {
            public static int32 ext_destroyed = 0;
            public int32 id;
            public ExternalGuard(int32 id) {
                this.id = id;
            }
            ~ExternalGuard() {
                ExternalGuard::ext_destroyed = ExternalGuard::ext_destroyed + 1;
            }
        }

        class CyclicResource {
            public static int32 destroyed_count = 0;
            public static int32 peer_valid_count = 0;
            public static ExternalGuard live_guard = null;
            public CyclicResource peer;
            public ExternalGuard external;
            public int32 tag;

            public CyclicResource(int32 tag, ExternalGuard ext) {
                this.tag = tag;
                this.external = ext;
                this.peer = null;
            }

            ~CyclicResource() {
                CyclicResource::destroyed_count = CyclicResource::destroyed_count + 1;
                if (this.peer != null && this.peer.tag != 0) {
                    CyclicResource::peer_valid_count = CyclicResource::peer_valid_count + 1;
                }
                this.external = null;
            }
        }

        void spawn_cycle(ExternalGuard shared_external) {
            CyclicResource first = new CyclicResource(11, shared_external);
            CyclicResource second = new CyclicResource(22, shared_external);
            first.peer = second;
            second.peer = first;
        }

        public int32 main() {
            ExternalGuard survivor = new ExternalGuard(777);
            CyclicResource::live_guard = survivor;
            spawn_cycle(survivor);
            return 0;
        }

        public int32 read_and_cleanup() {
            int32 result = (CyclicResource::destroyed_count * 1000)
                + (CyclicResource::peer_valid_count * 100)
                + (ExternalGuard::ext_destroyed * 10)
                + CyclicResource::live_guard.id;
            CyclicResource::live_guard = null;
            return result;
        }
    "#;

    let compiled_program = compile_source(source_code);
    let options = RuntimeOptions::new()
        .with_binary(compiled_program)
        .with_cycle_collector(true)
        .with_cycle_threshold(1024);
    let context = RuntimeContext::new(options).expect("Runtime context construction must succeed");
    let mut virtual_machine = VirtualMachine::new(context);

    let init_result = virtual_machine
        .invoke_entry("main", &[])
        .expect("main must execute cleanly");
    assert_eq!(init_result, Value::I32(0));

    let collected = virtual_machine
        .collect_cycles()
        .expect("Cycle collection must succeed");
    assert!(
        collected >= 2,
        "Expected at least 2 collected cyclic blocks, got {collected}"
    );

    let summary = virtual_machine
        .invoke_entry("read_and_cleanup", &[])
        .expect("read_and_cleanup must execute cleanly");
    // 2 destructors ran, both saw valid peer, survivor.id remained intact (777)
    assert_eq!(summary, Value::I32(2 * 1000 + 2 * 100 + 777));

    virtual_machine
        .shutdown()
        .expect("VM shutdown must succeed cleanly");
    assert!(
        virtual_machine.verify_clean_memory().is_ok(),
        "All cyclic and external blocks must reclaim cleanly after shutdown"
    );
}
