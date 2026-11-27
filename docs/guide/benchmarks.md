# Solix Benchmarking & Performance Telemetry Architecture

This guide specifies the architecture, measurement methodology, hardware telemetry collection, and regression gate enforcement provided by the Solix Universal Benchmark Suite (`scripts/bench/solix_bench.py`).

---

## 1. Architectural Overview

Micro-benchmarks must isolate runtime virtual machine dispatch throughput from language compilation overhead. The Solix benchmark suite employs a two-tier evaluation architecture:

1. **Pre-Compilation Separation**: Prior to entering timed benchmark iterations, Solix source files (`*.slx`) are pre-compiled into a standalone bytecode container (`*.slxb`) via `solix compile <sources> -o <binary>.slxb --profile release`. The compiler phase duration is warmed up once and sampled across three passes (reporting the median as `Precompile (ms)`). Likewise, Lua comparative workloads (`*.lua`) are pre-compiled into bytecode (`*.luac`) via `luac -o <binary>.luac <source>.lua` (when `luac` is available) before timed execution on `lua`, matching C (`gcc -O3`), Java (`javac`), and Solix AOT parity.
2. **Pure Virtual Machine Dispatch**: Timed loop iterations execute directly against pre-compiled bytecode containers via `solix run <binary>.slxb` (or `solix run <binary>.slxb --safe`). This eliminates lexing, AST parsing, symbol binding, type inference, AST inlining, and bytecode emission from VM execution measurements.
3. **Comparative Language Parity**: Identical algorithms implemented in C (`gcc -O3 -lm`), Java (`javac` / `java`), Lua (`luac` + `lua`), LuaJIT, and Python 3 are verified for output checksum parity before comparative timings are recorded. Workloads are scaled to run for sustained durations so JIT engines (OpenJDK HotSpot C2 and LuaJIT) reach steady-state compiled execution.

---

## 2. Measurement Methodology

To deliver statistically robust performance metrics across varied host runners, the suite measures multiple dimensions:

| Metric Dimension | Description | Typical Use Case |
| :--- | :--- | :--- |
| `Precompile (ms)` | Wall-clock duration to compile `.slx` sources to `.slxb` | Compiler throughput analysis |
| `Median VM (ms)` | Median execution time across warmed iterations | Steady-state runtime dispatch speed |
| `Min / Max (ms)` | Best and worst observed sample durations | Jitter and operating system scheduling variance |
| `Peak RSS (KB)` | Maximum resident set size observed across runs | Memory footprint and allocation overhead |

---

## 3. Hardware Telemetry Extraction

Every benchmark run captures host machine hardware telemetry stored in the baseline JSON metadata and rendered in a `### Benchmark Environment` Markdown table at the top of release and comparison reports (and bannered in console output):

* **CPU Model**: Extracted from `/proc/cpuinfo` on Linux, `sysctl machdep.cpu.brand_string` on macOS, or `wmic cpu get name` on Windows.
* **CPU Logical Cores**: Number of available processing units detected by the operating system kernel.
* **Total Physical RAM**: Total system memory in megabytes detected via `/proc/meminfo`, `sysctl`, or `wmic`.
* **Kernel & Architecture**: Operating system platform name, release version, and processor architecture (`x86_64`, `aarch64`).

---

## 4. CLI Subcommands & Options

The benchmarking CLI provides three primary subcommands:

### 4.1. `verify` Subcommand

Executes every workload across all available language engines (Solix, C, Lua, LuaJIT, Python 3) and verifies that every engine emits matching trailing `RESULT:<value>` checksum lines.

```bash
python3 scripts/bench/solix_bench.py verify
```

### 4.2. `run` Subcommand

Executes benchmark suites and outputs formatted console tables, Markdown summaries, or structured JSON baselines.

```bash
# Run Solix benchmarks with warmed iterations
python3 scripts/bench/solix_bench.py run --warmup 3 --iterations 11

# Run with iteration scaling for sub-millisecond micro-benchmarks
python3 scripts/bench/solix_bench.py run --scale-iterations 10 --solix-only

# Target minimum sample duration per iteration
python3 scripts/bench/solix_bench.py run --benchmark-duration-ms 2000.0

# Save baseline for regression comparison
python3 scripts/bench/solix_bench.py run --save-baseline staging/release_baseline.json
```

Key options for `solix_bench run`:

* `--scale-iterations` / `--inner-loop`: Executes multiple passes per sample, averaging duration to eliminate timer quantization noise on fast workloads.
* `--benchmark-duration-ms`: Minimum target sample duration in milliseconds across iterations.
* `--no-precompile`: Executes Solix directly on sources without pre-compiling to `.slxb` (used for measuring full end-to-end compiler + runtime latency).
* `--solix-only`: Restricts benchmarks to Solix engines, skipping comparative language runtimes.
* `--pin-cpu`: Pins benchmark execution processes to CPU core 0 using `taskset` on Linux runners.
* `--format [table|markdown|json]`: Controls stdout output format.

### 4.3. `compare` Subcommand

Compares two benchmark baselines (or multi-baseline histories) to enforce automated performance regression gates in Continuous Integration.

```bash
# Compare current baseline against previous release
python3 scripts/bench/solix_bench.py compare \
  --previous baselines/previous_release.json \
  --current baselines/current_build.json \
  --threshold-pct 5.0 \
  --min-delta-ms 0.5 \
  --fail-on-regression
```

* `--threshold-pct`: Percentage regression threshold before flagging a workload as slower (default: `5.0%`).
* `--min-delta-ms`: Minimum absolute millisecond variance required before flagging a change, preventing sub-millisecond noise from failing CI runs (default: `0.5 ms`).
* `--fail-on-regression`: Exits with non-zero status code if any workload regresses beyond the configured threshold.

---

## 5. Markdown Report Schemas

1. **Self-Comparison Table (`compare`):** Reports explicit `Prev` and `Curr` columns plus deltas across VM execution (`Prev VM (ms)`, `Curr VM (ms)`, `VM Speedup`), precompilation (`Prev Compile (ms)`, `Curr Compile (ms)`, `Compile Diff`), peak memory (`Prev RSS (KB)`, `Curr RSS (KB)`, `RSS Diff`), and `Status`, followed by an explanatory `Column Glossary` callout.
2. **Cross-Language Table (`--cross-language` / `run --format markdown`):** Focuses on steady-state runtime throughput across engines: `Workload`, `Solix VM (ms)`, `Python 3 (ms)`, `Lua (ms)`, `LuaJIT (ms)`, `C -O3 (ms)`, and `Java (ms)`.
