//! Telemetry metrics and throughput models for compiler pipeline stages.

#![forbid(unsafe_code)]

/// Telemetry metrics recorded for a single compilation phase.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PhaseMetric {
    /// Canonical phase name.
    pub name: String,
    /// Execution duration in microseconds.
    pub duration_us: u64,
}

/// Comprehensive compiler throughput and stage latency telemetry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CompilerTelemetry {
    /// Package or library name.
    pub package_name: String,
    /// Total wall-clock compilation duration in microseconds.
    pub total_time_us: u64,
    /// Total size of all processed source code in bytes.
    pub source_bytes: usize,
    /// Total tokens emitted by lexical analysis.
    pub total_tokens: usize,
    /// Total instructions assembled by bytecode emitter.
    pub total_instructions: usize,
    /// Final binary container payload size in bytes.
    pub binary_size_bytes: usize,
    /// Megabytes scanned and compiled per second.
    pub throughput_mb_s: f64,
    /// Sequential phase duration breakdowns.
    pub phases: Vec<PhaseMetric>,
}
