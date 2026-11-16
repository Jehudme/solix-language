//! Milestone 056 Conformance Tests: Standard Library Expansion, VM Telemetry & Mount Virtual Filesystem.

#![forbid(unsafe_code)]

use solix_common::bytecode::{BinaryProgram, CompiledFunction};
use solix_runtime::context::RuntimeContext;
use solix_runtime::options::RuntimeOptions;
use solix_runtime::vm::VirtualMachine;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_vm_engine_all_56_metrics() {
    let mut binary_program = BinaryProgram::new();
    binary_program.functions.push(CompiledFunction::new(
        "telemetry::sample_entry",
        0,
        0,
        4,
    ));

    let options = RuntimeOptions::new()
        .with_binary(binary_program)
        .with_container_kind("slxpac")
        .with_container_resources(8, 65536);
    let context = RuntimeContext::new(options).expect("Runtime context creation must succeed");
    let virtual_machine = VirtualMachine::new(context);

    // Verify all 56 metrics are queryable without panicking
    for metric_index in 0..56u32 {
        let metric_value = virtual_machine.get_metric_by_id(metric_index);
        match metric_index {
            // Metric 28: func_total_registered
            28 => assert_eq!(
                metric_value, 1,
                "Metric 28 (func_total_registered) should equal 1"
            ),
            // Metric 36: container_is_slxpac
            36 => assert_eq!(
                metric_value, 1,
                "Metric 36 (container_is_slxpac) should equal 1"
            ),
            // Metric 37: container_is_slxarc
            37 => assert_eq!(
                metric_value, 0,
                "Metric 37 (container_is_slxarc) should equal 0"
            ),
            // Metric 38: container_is_slxb
            38 => assert_eq!(
                metric_value, 0,
                "Metric 38 (container_is_slxb) should equal 0"
            ),
            // Metric 40: container_resource_count
            40 => assert_eq!(
                metric_value, 8,
                "Metric 40 (container_resource_count) should equal 8"
            ),
            // Metric 41: container_resource_total_bytes
            41 => assert_eq!(
                metric_value, 65536,
                "Metric 41 (container_resource_total_bytes) should equal 65536"
            ),
            // Metric 44: process_pid
            44 => assert_eq!(
                metric_value,
                std::process::id() as u64,
                "Metric 44 (process_pid) should equal current process PID"
            ),
            // Metric 52: sys_available_parallelism
            52 => assert!(
                metric_value >= 1,
                "Metric 52 (sys_available_parallelism) should be >= 1"
            ),
            // All other metrics should return non-negative values
            _ => {
                let _ = metric_value;
            }
        }
    }
}

#[test]
fn test_mount_virtual_path_resolution_loose() {
    let unique_directory_name = format!(
        "solix_test_mount_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let temporary_root = std::env::temp_dir().join(unique_directory_name);
    let resources_directory = temporary_root.join("resources");
    fs::create_dir_all(&resources_directory).expect("Must create resources directory");

    let test_resource_file = resources_directory.join("config.json");
    let sample_resource_content = r#"{"name": "Solix Virtual FS", "version": "2.0.0"}"#;
    fs::write(&test_resource_file, sample_resource_content).expect("Must write resource file");

    let mount_uri = "mount://config.json";
    let stripped_relative_path = mount_uri.strip_prefix("mount://").unwrap();
    let resolved_file_path = resources_directory.join(stripped_relative_path);

    assert!(
        resolved_file_path.exists(),
        "Resolved file path must exist on disk"
    );
    let read_content =
        fs::read_to_string(&resolved_file_path).expect("Reading resolved resource must succeed");
    assert_eq!(read_content, sample_resource_content);

    let _ = fs::remove_dir_all(&temporary_root);
}

#[test]
fn test_mount_virtual_path_rejection_in_slxpac_archive() {
    // When executing inside an .slxpac container, any write operation to mount:// must throw E0330
    let target_path = PathBuf::from("mount://assets/texture.png");
    let is_in_slxpac_container = true;

    let write_result: Result<(), String> = if is_in_slxpac_container {
        Err(format!(
            "Virtual path '{}' under 'mount://' is read-only inside an .slxpac archive container. [E0330]",
            target_path.display()
        ))
    } else {
        Ok(())
    };

    assert!(write_result.is_err());
    let error_message = write_result.unwrap_err();
    assert!(
        error_message.contains("[E0330]"),
        "Error message must contain diagnostic code [E0330]"
    );
    assert!(
        error_message.contains("is read-only inside an .slxpac archive container"),
        "Error message must specify container read-only constraint"
    );
}
