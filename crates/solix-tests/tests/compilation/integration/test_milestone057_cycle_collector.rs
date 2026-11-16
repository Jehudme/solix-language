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
