//! Process-level resource measurement and CLI telemetry reporting.

#![forbid(unsafe_code)]

/// Cross-platform process telemetry snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessTelemetry {
    /// Peak resident set size (RSS) in kilobytes.
    pub peak_rss_kb: u64,
    /// User-mode CPU execution time in microseconds.
    pub user_cpu_time_us: u64,
    /// Kernel-mode system CPU execution time in microseconds.
    pub system_cpu_time_us: u64,
}

/// Captures process-level telemetry in a safe, cross-platform manner.
#[must_use]
pub fn capture_process_telemetry() -> ProcessTelemetry {
    #[cfg(target_os = "linux")]
    {
        capture_linux_process_telemetry()
    }
    #[cfg(not(target_os = "linux"))]
    {
        ProcessTelemetry::default()
    }
}

#[cfg(target_os = "linux")]
fn capture_linux_process_telemetry() -> ProcessTelemetry {
    let mut peak_rss_kb = 0;
    if let Ok(status_text) = std::fs::read_to_string("/proc/self/status") {
        for line in status_text.lines() {
            if let Some(rest) = line.strip_prefix("VmHWM:")
                && let Some(token) = rest.split_whitespace().next()
                && let Ok(value) = token.parse::<u64>()
            {
                peak_rss_kb = value;
                break;
            } else if peak_rss_kb == 0
                && let Some(rest) = line.strip_prefix("VmRSS:")
                && let Some(token) = rest.split_whitespace().next()
                && let Ok(value) = token.parse::<u64>()
            {
                peak_rss_kb = value;
            }
        }
    }

    let (user_cpu_time_us, system_cpu_time_us) = read_linux_cpu_times();

    ProcessTelemetry {
        peak_rss_kb,
        user_cpu_time_us,
        system_cpu_time_us,
    }
}

#[cfg(target_os = "linux")]
fn read_linux_cpu_times() -> (u64, u64) {
    if let Ok(stat_text) = std::fs::read_to_string("/proc/self/stat") {
        // The process name is enclosed in parentheses, e.g. "1234 (process) S ..."
        if let Some(closing_paren_idx) = stat_text.rfind(')') {
            let after_comm = &stat_text[closing_paren_idx + 1..];
            let tokens: Vec<&str> = after_comm.split_whitespace().collect();
            // Tokens after comm:
            // index 0: state (field 3)
            // index 1: ppid (field 4)
            // ...
            // index 11: utime (field 14)
            // index 12: stime (field 15)
            if tokens.len() > 12 {
                let utime_ticks = tokens[11].parse::<u64>().unwrap_or(0);
                let stime_ticks = tokens[12].parse::<u64>().unwrap_or(0);
                // On Linux x86_64/ARM64 USER_HZ is typically 100 ticks/sec (10,000 µs per tick)
                let user_us = utime_ticks.saturating_mul(10_000);
                let system_us = stime_ticks.saturating_mul(10_000);
                return (user_us, system_us);
            }
        }
    }
    (0, 0)
}

/// Structured timing telemetry of VM execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimingsTelemetry {
    pub total_process_wall_time_us: u64,
    pub pure_vm_execution_time_us: u64,
    pub binary_load_time_us: u64,
    pub static_init_time_us: u64,
}

/// Structured memory telemetry of heap and process RSS.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryTelemetry {
    pub peak_virtual_heap_bytes: usize,
    pub peak_os_rss_kb: u64,
    pub heap_amplification_ratio: f64,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub active_leaks: usize,
}

/// Structured execution telemetry of VM opcode dispatch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionTelemetry {
    pub instructions_dispatched: u64,
    pub peak_stack_depth: usize,
    pub quickened_calls: u64,
    pub jit_compiled_functions: u64,
    pub jit_compile_time_ns: u64,
}

/// Structured execution telemetry report output for `solix run --metrics json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunMetricsReport {
    pub workload: String,
    pub status: String,
    pub exit_code: i32,
    pub timings: TimingsTelemetry,
    pub memory: MemoryTelemetry,
    pub execution: ExecutionTelemetry,
}

/// Structured compiler metrics report output for `solix compile --metrics json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompileMetricsReport {
    pub package_name: String,
    pub status: String,
    pub total_time_us: u64,
    pub throughput_mb_s: f64,
    pub source_bytes: usize,
    pub total_tokens: usize,
    pub total_instructions: usize,
    pub binary_size_bytes: usize,
    pub phases: Vec<solix_compilation::PhaseMetric>,
}
