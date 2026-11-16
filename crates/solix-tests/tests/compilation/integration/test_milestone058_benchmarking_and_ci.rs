//! Milestone 058 Conformance Tests: Dynamic Release Benchmarking, Comparison Tables & CI/CD Gate.

#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent of solix-tests crate must exist")
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

#[test]
fn test_compare_releases_table_generation() {
    let workspace_root = get_workspace_root();
    let compare_script = workspace_root.join("benchmarks/compare_releases.py");
    let temporary_directory =
        tempfile::tempdir().expect("Temporary directory creation must succeed");
    let current_baseline_path = temporary_directory.path().join("current_baseline.json");
    let previous_baseline_path = temporary_directory.path().join("previous_baseline.json");
    let markdown_output_path = temporary_directory.path().join("comparison_table.md");
    let json_output_path = temporary_directory.path().join("comparison_results.json");

    let current_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "01_fib_recursive": {
                "solix_fast": {"median_ms": 145.5, "rss_kb": 20480}
            },
            "02_tight_loop_int": {
                "solix_fast": {"median_ms": 230.0, "rss_kb": 40960}
            }
        }
    }"#;

    let previous_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "01_fib_recursive": {
                "solix_fast": {"median_ms": 155.0, "rss_kb": 21504}
            },
            "02_tight_loop_int": {
                "solix_fast": {"median_ms": 231.0, "rss_kb": 40960}
            }
        }
    }"#;

    fs::write(&current_baseline_path, current_json_content)
        .expect("Writing current baseline JSON must succeed");
    fs::write(&previous_baseline_path, previous_json_content)
        .expect("Writing previous baseline JSON must succeed");

    let command_status = Command::new("python3")
        .arg(&compare_script)
        .arg("--current")
        .arg(&current_baseline_path)
        .arg("--previous")
        .arg(&previous_baseline_path)
        .arg("--previous-tag")
        .arg("v1.2.0")
        .arg("--current-tag")
        .arg("v2.0.0")
        .arg("--markdown-output")
        .arg(&markdown_output_path)
        .arg("--json-output")
        .arg(&json_output_path)
        .current_dir(&workspace_root)
        .status()
        .expect("Executing compare_releases.py must succeed");

    assert!(
        command_status.success(),
        "compare_releases.py execution should succeed with exit code 0"
    );

    let rendered_markdown = fs::read_to_string(&markdown_output_path)
        .expect("Reading generated markdown output must succeed");

    assert!(
        rendered_markdown.contains("### Performance Benchmarks vs v1.2.0 (`v2.0.0`)"),
        "Markdown must include the target heading"
    );
    assert!(
        rendered_markdown.contains(
            "| Workload | Previous (ms) | Current (ms) | Speedup | Memory Diff | Status |"
        ),
        "Markdown must include table header"
    );
    assert!(
        rendered_markdown.contains("01_fib_recursive"),
        "Markdown must include 01_fib_recursive workload row"
    );
    assert!(
        rendered_markdown.contains("🚀 FASTER"),
        "01_fib_recursive should be flagged as FASTER"
    );
    assert!(
        rendered_markdown.contains("⚖️ STABLE"),
        "02_tight_loop_int should be flagged as STABLE"
    );

    let rendered_json =
        fs::read_to_string(&json_output_path).expect("Reading generated JSON output must succeed");
    assert!(rendered_json.contains("\"faster_count\": 1"));
    assert!(rendered_json.contains("\"stable_count\": 1"));
    assert!(rendered_json.contains("\"has_regression\": false"));
}

#[test]
fn test_compare_releases_regression_detection() {
    let workspace_root = get_workspace_root();
    let compare_script = workspace_root.join("benchmarks/compare_releases.py");
    let temporary_directory =
        tempfile::tempdir().expect("Temporary directory creation must succeed");
    let current_baseline_path = temporary_directory.path().join("regressed_current.json");
    let previous_baseline_path = temporary_directory.path().join("baseline_previous.json");

    // Current is significantly slower (200.0 ms vs 100.0 ms = 50% regression)
    let current_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "regressed_workload": {
                "solix_fast": {"median_ms": 200.0, "rss_kb": 20480}
            }
        }
    }"#;

    let previous_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "regressed_workload": {
                "solix_fast": {"median_ms": 100.0, "rss_kb": 20480}
            }
        }
    }"#;

    fs::write(&current_baseline_path, current_json_content)
        .expect("Writing current baseline JSON must succeed");
    fs::write(&previous_baseline_path, previous_json_content)
        .expect("Writing previous baseline JSON must succeed");

    let command_status = Command::new("python3")
        .arg(&compare_script)
        .arg("--current")
        .arg(&current_baseline_path)
        .arg("--previous")
        .arg(&previous_baseline_path)
        .arg("--threshold-pct")
        .arg("5.0")
        .arg("--fail-on-regression")
        .arg("--quiet")
        .current_dir(&workspace_root)
        .status()
        .expect("Executing compare_releases.py must succeed");

    assert!(
        !command_status.success(),
        "compare_releases.py must return non-zero exit code when regression exceeds threshold"
    );
}

#[test]
fn test_compare_releases_inject_notes() {
    let workspace_root = get_workspace_root();
    let compare_script = workspace_root.join("benchmarks/compare_releases.py");
    let temporary_directory =
        tempfile::tempdir().expect("Temporary directory creation must succeed");
    let current_baseline_path = temporary_directory.path().join("current.json");
    let previous_baseline_path = temporary_directory.path().join("previous.json");
    let release_notes_path = temporary_directory.path().join("RELEASE_NOTES.md");

    let current_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "01_fib_recursive": {
                "solix_fast": {"median_ms": 145.5, "rss_kb": 20480}
            }
        }
    }"#;

    let previous_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "01_fib_recursive": {
                "solix_fast": {"median_ms": 150.0, "rss_kb": 20480}
            }
        }
    }"#;

    let initial_notes = "# Solix Release v2.0.0\n\nWelcome to Solix v2.0.0!\n";
    fs::write(&release_notes_path, initial_notes)
        .expect("Writing initial release notes must succeed");
    fs::write(&current_baseline_path, current_json_content)
        .expect("Writing current baseline must succeed");
    fs::write(&previous_baseline_path, previous_json_content)
        .expect("Writing previous baseline must succeed");

    let command_status = Command::new("python3")
        .arg(&compare_script)
        .arg("--current")
        .arg(&current_baseline_path)
        .arg("--previous")
        .arg(&previous_baseline_path)
        .arg("--inject-notes")
        .arg(&release_notes_path)
        .arg("--quiet")
        .current_dir(&workspace_root)
        .status()
        .expect("Executing compare_releases.py must succeed");

    assert!(command_status.success());

    let updated_notes = fs::read_to_string(&release_notes_path)
        .expect("Reading updated release notes must succeed");

    assert!(updated_notes.starts_with("# Solix Release v2.0.0"));
    assert!(updated_notes.contains("### Performance Benchmarks vs Previous Release"));
    assert!(updated_notes.contains("01_fib_recursive"));
}

#[test]
fn test_compare_releases_initial_baseline_mode() {
    let workspace_root = get_workspace_root();
    let compare_script = workspace_root.join("benchmarks/compare_releases.py");
    let temporary_directory =
        tempfile::tempdir().expect("Temporary directory creation must succeed");
    let current_baseline_path = temporary_directory.path().join("current.json");
    let empty_baselines_dir = temporary_directory.path().join("empty_baselines");
    fs::create_dir_all(&empty_baselines_dir).expect("Creating empty directory must succeed");
    let markdown_output_path = temporary_directory.path().join("initial_table.md");

    let current_json_content = r#"{
        "metadata": {"mode": "fast"},
        "workloads": {
            "01_fib_recursive": {
                "solix_fast": {"median_ms": 145.5, "rss_kb": 20480}
            }
        }
    }"#;

    fs::write(&current_baseline_path, current_json_content)
        .expect("Writing current baseline must succeed");

    let command_status = Command::new("python3")
        .arg(&compare_script)
        .arg("--current")
        .arg(&current_baseline_path)
        .arg("--current-tag")
        .arg("v2.0.0")
        .arg("--no-fallback")
        .arg("--markdown-output")
        .arg(&markdown_output_path)
        .arg("--quiet")
        .current_dir(&empty_baselines_dir)
        .status()
        .expect("Executing compare_releases.py must succeed");

    assert!(command_status.success());

    let rendered_markdown = fs::read_to_string(&markdown_output_path)
        .expect("Reading initial markdown output must succeed");

    assert!(rendered_markdown.contains("### Performance Baseline (v2.0.0)"));
    assert!(rendered_markdown.contains("📌 BASELINE"));
}

#[test]
fn test_benchmark_runner_verify_command() {
    let workspace_root = get_workspace_root();
    let runner_script = workspace_root.join("benchmarks/runner.py");

    let command_output = Command::new("python3")
        .arg(&runner_script)
        .arg("--verify")
        .current_dir(&workspace_root)
        .output()
        .expect("Executing benchmarks/runner.py --verify must succeed");

    assert!(
        command_output.status.success(),
        "runner.py --verify must complete with success exit code"
    );

    let standard_output = String::from_utf8_lossy(&command_output.stdout);
    assert!(
        standard_output.contains(
            "Verification gate passed: all workloads produced identical results across engines"
        ),
        "runner.py --verify output must confirm cross-engine verification pass"
    );
}
