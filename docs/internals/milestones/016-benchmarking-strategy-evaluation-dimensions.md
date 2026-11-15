# Solix Multi-Dimensional Benchmarking & Telemetry Strategy

## Document Metadata
* **Scope:** CLI Direct Source Execution (`solix run <files.slx>`), Cross-Platform Telemetry, Memory Architecture, Compiler Throughput & Multi-Language Parity
* **Status:** Active Implementation Standard
* **Target Milestone:** `docs/internals/milestones/016-benchmarking-strategy-evaluation-dimensions.md` (MS-016)
* **Governing Guidelines:** `GL-0001` (Commits), `GL-0002` (Branching), `GL-0008` (Milestones), `GL-0009` (Testing Matrices), `GL-0010` (Documentation), `GL-0011` (Code Quality)

---

## 1. Executive Summary & Purpose

A major risk when optimizing an interpreter or compiler is relying on a single, coarse metric—such as total process runtime on 3 or 4 basic scripts. Measuring only total wall-clock time conflates operating system process initialization, dynamic library linking, file I/O, static `@init` evaluation, and memory teardown with pure bytecode loop execution. Furthermore, execution time reveals nothing about **memory footprint, cache pressure, compiler throughput, or allocation thrashing**.

Before modifying the VM or implementing a JIT, Solix requires an uncompromising, automated benchmarking and telemetry framework, backed by streamlined developer ergonomics. This strategy establishes:

1. **Direct Source Execution Ergonomics:** Enhancing `solix run` so that it automatically compiles `.slx` source files on-the-fly and loads the resulting binary container directly into runtime options in memory. Users and benchmark harnesses can execute `.slx` source code directly without requiring a separate pre-compilation step.
2. **Self-Comparative Evaluation (Axis I):** Evaluating Solix against its own past commits, comparing `--safe` vs. fast mode, dissecting compiler phase latencies, tracking instruction elimination, and detecting memory leaks or heap amplification.
3. **Cross-Language Comparative Evaluation (Axis II):** Fair, apples-to-apples benchmark workloads matching identical algorithms and data structures across **Solix**, **Legacy C++ Solix (v2.1.0)**, **PUC-Rio Lua 5.5**, **LuaJIT 2.1**, **Python 3.15**, and **Go**.
4. **Universal Cross-Platform Architecture:** Ensuring all benchmark operations, memory RSS tracking, and telemetry recording function uniformly across **Linux**, **macOS (Darwin)**, and **Windows (NT)** without platform-exclusive restrictions.

---

## 2. Prerequisite Tooling: Direct Source Execution in `solix run`

To enable frictionless benchmark coordination and rapid developer turnaround, `solix run` must support executing `.slx` source files directly, seamlessly compiling them in memory before execution.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        solix run Target Resolution                     │
├──────────────────────────────┬─────────────────────────────────────────┤
│    Inputs: *.slx Source      │ • In-memory CompilationPipeline         │
│          Files               │ • Full standard phase execution         │
│                              │ • Rich miette diagnostic rendering      │
│                              │ • Direct in-memory BinaryProgram mount  │
├──────────────────────────────┼─────────────────────────────────────────┤
│    Inputs: *.slxb Binary     │ • Fast deserialization from bytes       │
│         Containers           │ • Peer module dynamic linking           │
├──────────────────────────────┼─────────────────────────────────────────┤
│    Hybrid / Multi-Source     │ • Compile *.slx sources together        │
│          Workloads           │ • Mount *.slxb libraries via --lib      │
│                              │ • Run VM with unified RuntimeOptions    │
└──────────────────────────────┴─────────────────────────────────────────┘
```

### 2.1 CLI Interface Specification
`solix run` accepts both `.slxb` precompiled binaries and `.slx` source files:
```bash
# Execute precompiled binary
solix run app.slxb

# Compile on-the-fly and execute single source
solix run main.slx

# Compile multi-source project on-the-fly and execute
solix run helper.slx main.slx

# Run multiple sources and dependency libraries directly as positional arguments
solix run source1.slx source2.slx lib1.slxb lib2.slxb

# Safe mode execution of source
solix run main.slx --safe --verbose
```

> [!NOTE]
> **Positional Arguments Invariant:** `solix run` accepts any arbitrary mixture of `.slx` source files and `.slxb` binary containers directly as positional arguments. Flags `--lib` / `-l` are reserved for native platform shared libraries (`.so`, `.dll`, `.dylib`) rather than Solix bytecode containers.

### 2.2 Compilation Pipeline & Error Handling
1. If `.slx` files are supplied, `execute_run` initializes a `CompilationContext` with the specified source files and builtins enabled (unless `--no-builtins` is specified).
2. The compilation pipeline executes: Lexing $\to$ Parsing $\to$ Semantic Binding $\to$ Bytecode Emission.
3. If compilation fails, errors are rendered to `stderr` with full source span highlighting via `render_diagnostics`, and execution halts with status code 1.
4. On compilation success, the generated `BinaryProgram` is appended to `RuntimeOptions` alongside any explicitly loaded `.slxb` peer containers.

---

## 3. Axis I: Evaluating Solix Against Itself (Internal Telemetry)

Internal benchmarks provide the diagnostic clarity needed during development. By instrumenting the compiler pipeline and VM runtime, we measure internal invariants that external test harnesses cannot see.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Solix Internal Telemetry                        │
├──────────────────────────────┬─────────────────────────────────────────┤
│    Compiler Pipeline Pass    │ • Lexer throughput (MB/s & tokens/ms)   │
│         Throughput           │ • Parser AST rate & peak memory         │
│                              │ • Binder pass breakdown & monomorphize  │
│                              │ • Emitter instruction assembly latency  │
├──────────────────────────────┼─────────────────────────────────────────┤
│    Runtime Execution &       │ • Pure VM execution (excluding startup) │
│      Dispatch Health         │ • Fast mode vs. --safe checked mode     │
│                              │ • Total dynamic instructions dispatched │
│                              │ • Static & call relocation quickening   │
├──────────────────────────────┼─────────────────────────────────────────┤
│   Memory Health &            │ • VirtualHeap peak bytes vs. OS RSS     │
│      Amplification           │ • Heap Amplification Ratio              │
│                              │ • Freelist fragmentation & splits       │
│                              │ • ARC IncRef/DecRef ops & cascade drops │
└──────────────────────────────┴─────────────────────────────────────────┘
```

### 3.1 Compiler Pipeline Stage Breakdown (`solix compile --metrics`)
Instead of treating compilation as a black box, instrument each phase in microseconds:
* **Lexer Throughput (`LexerPhase`):** Megabytes scanned per second ($\text{MB/s}$) and tokens emitted per millisecond.
* **Parser Throughput (`ParserPhase`):** AST nodes constructed per millisecond and peak memory allocated by `ProgramAst`.
* **Semantic Analysis & Type Resolution (`BinderPhase`):**
  - **Pass 1 (Symbol Discovery):** Symbol table indexing and import outline resolution latency.
  - **Pass 2 (Hierarchy & Layouts):** VTable generation, field offset alignment, and cycle detection.
  - **Pass 3 (Type Checking):** Expression type evaluation and local slot assignment.
  - **Generic Monomorphization:** Time spent specializing templates (e.g. 1 specialization vs. 100 nested instantiations).
  - **Incremental Rebind:** Measuring `bind_incremental(source_id)` on single-file mutations (target: $< 5\text{ ms}$).
* **Bytecode Emission & Container Assembly (`EmitterPhase`):** Instructions emitted per millisecond, constant deduplication, jump backpatching, and `.slxb` serialization speed.

### 3.2 Execution Mode & Dispatch Comparison
Every benchmark workload is evaluated across internal configurations:
1. **Default Fast Mode (`solix run`):** Inlined dispatch loop using raw pointer register indexing (`run_fast`).
2. **Safe Checked Mode (`solix run --safe`):** Bounds-checked registers, defensive opcode guards, and complete shutdown verification (`verify_clean_memory()`).
3. **In-Process Warm VM Execution:** Executing pre-loaded `.slxb` containers in-memory using Criterion benches, stripping out OS `fork`/`exec` process creation and dynamic linker startup latency.
4. **Historical Regression Tracking:** Comparing against saved baselines (`ms015_baseline.json`) to assert that new pull requests never degrade throughput by more than $2\%$.

### 3.3 Memory Footprint & Universal Cross-Platform RSS Tracking
Exposed via a machine-readable `--metrics-json` flag on `solix run`:
* **VirtualHeap High-Water Mark:** Peak bytes occupied inside `VirtualHeap` buffer.
* **Host Process RSS (Resident Set Size):** Physical RAM queried using cross-platform telemetry:
  - **Linux:** `/proc/self/status` (`VmHWM`) or `getrusage(RUSAGE_CHILDREN)` in KB.
  - **macOS:** `getrusage(RUSAGE_CHILDREN)` scaled from bytes to KB (`ru_maxrss / 1024`), verified via `mach_task_basic_info`.
  - **Windows:** Win32 API `GetProcessMemoryInfo` (`PeakWorkingSetSize` in KB) via `psutil` or native Win32 bindings.
* **Memory Amplification Ratio:**
  $$\text{Amplification Ratio} = \frac{\text{Host Peak RSS} - \text{Idle Baseline RSS}}{\text{Guest VirtualHeap High-Water Mark}}$$
* **Allocator & ARC Telemetry:**
  - Dynamic block allocation count (`allocate` calls).
  - Dynamic deallocation count (`deallocate` calls).
  - Segregated free-list splits, coalesces, and fragment count.
  - Total ARC increments (`inc_ref`) and decrements (`dec_ref`).
  - Total destructor invocations (`~Class()`) and maximum cascade release depth.

### 3.4 Bytecode Quality & Dispatched Instruction Budgeting
* **Static Bytecode Footprint:** Total instructions emitted per function and container file size.
* **Dynamic Instruction Count:** The total number of instructions the VM actually executed during the run.
* **Peak Register Pressure:** Peak register window depth per function frame (`num_registers`).

---

## 4. Axis II: Evaluating Solix Against Other Languages

To compare Solix fairly against **PUC-Rio Lua 5.5**, **LuaJIT 2.1**, **Python 3.15**, **Go**, and **C/Rust**, workloads isolate specific language runtime characteristics across all supported operating systems.

### 4.1 The 6 Cross-Language Evaluation Dimensions

| Dimension | Primary Focus | Metrics Recorded | Why It Is Critical |
| :--- | :--- | :--- | :--- |
| **1. Cold Startup Latency** | Minimal executable turnaround | Minimum wall time to load `hello_world` and exit | Critical for CLI tools, serverless scripts, and developer feedback loops. |
| **2. Pure Algorithm Compute** | Interpreter dispatch loop efficiency | Internal high-resolution monotonic time (`clock()`) | Separates VM opcode dispatch speed from process startup. |
| **3. Physical Memory (Peak RSS)** | Cross-platform memory compactness | Peak Resident Set Size (`ru_maxrss` / `PeakWorkingSetSize` in KB) | Detects runtime bloat, boxed variable overhead, and memory amplification. |
| **4. Allocation & Reclamation Churn** | Object lifecycle throughput | Objects created & freed per second; tail latency | Evaluates deterministic **ARC** vs. Lua/Python/Go tracing **Garbage Collection**. |
| **5. Hardware CPU Efficiency** | Micro-architectural performance | IPC, branch-miss %, cache misses (Linux `perf stat`, Windows ETW, macOS sample) | Explains *why* an interpreter is fast (branch prediction vs. cache thrashing). |
| **6. Edit-to-Run Turnaround** | Full developer feedback cycle | Wall-clock time for `solix run script.slx` vs. interpreted run | Ensures Solix retains the rapid feedback feel of a dynamic language. |

---

### 4.2 The 16 Canonical Benchmark Workloads

```
workloads/
├── Category A: Control Flow, Stack & Numeric Computation
│   ├── 01_fib_recursive.slx        # Deep call stack, recursion & frame push/pop
│   ├── 02_tight_loop_int.slx       # 50M iterations: modulo, bitwise arithmetic, loop branching
│   ├── 03_nbody_float64.slx        # 1M steps: IEEE-754 double-precision arithmetic & math
│   └── 04_ackermann_stack.slx      # Deep stack frames & multi-argument function churn
│
├── Category B: Object Model, Polymorphism & Dynamic Dispatch
│   ├── 05_object_churn_pod.slx     # 1M short-lived Point instances: allocation, field get/set, ARC drop
│   ├── 06_binary_trees_arc.slx     # Depth 14 tree: deep object graphs, traversal & cascade release
│   ├── 07_virtual_dispatch.slx     # 10M calls: polymorphic VTable dynamic dispatch (Shape.area)
│   └── 08_interface_dispatch.slx   # 10M calls: interface method resolution via flat ITables
│
├── Category C: Arrays, Tuples & Memory Locality
│   ├── 09_sieve_bool_array.slx     # 1M primes: flat byte array allocation & strided indexing
│   ├── 10_matrix_multiply_2d.slx   # 200x200 int32[][] matrix multiplication: nested loops & cache locality
│   └── 11_generic_containers.slx   # 5M operations: monomorphized Box<int32> vs. tuple<int32, bool>
│
├── Category D: First-Class Callables & Exception Handling
│   ├── 12_closures_capture.slx     # 5M calls: higher-order functions & environment captures
│   ├── 13_function_pointers.slx    # 10M calls: stateless indirect function pointer invocations
│   └── 14_exception_zero_cost.slx  # 10M iterations inside try without throw vs. 100k thrown & caught
│
└── Category E: Strings, Globals & Dynamic Linking
    ├── 15_string_processing.slx    # String instantiation, concatenation & equality comparisons
    └── 16_multi_binary_link.slx    # Cross-binary calls & instruction quickening across .slxb units
```

#### Detailed Workload Invariants:
* **`01_fib_recursive` (`fib(35)`):** Evaluates pure function call and return overhead, register frame allocation, and integer additions across 14.9 million calls.
* **`02_tight_loop_int` (50M iterations):** Evaluates integer arithmetic (`+`, `-`, `%`, `&`, `|`, `^`, `<<`), register reuse, and loop jump backpatching.
* **`03_nbody_float64` (1M steps):** Simulates gravitational interactions of celestial bodies. Evaluates IEEE-754 64-bit float instructions (`AddF64`, `SubF64`, `MulF64`, `DivF64`, `NegF64`) and register pressure.
* **`06_binary_trees_arc` (Depth 14):** Creates a large balanced tree, tests tree walking, and drops the root node. In garbage-collected languages (Lua, Python, Go), this forces full garbage collection cycles. In Solix, it tests deterministic **ARC recursive cascade release** and memory coalescing.
* **`07_virtual_dispatch` (10M calls):** Creates an array of derived classes (`Circle`, `Square`, `Triangle`) inheriting from `Shape` and calls virtual `get_area()`. Compares Solix's static VTable against Lua's metatable index and Python's `__getattribute__`.
* **`08_interface_dispatch` (10M calls):** Calls methods through interface variables (`IWorker`). Evaluates Solix's $O(1)$ flat `itable` search against Go interfaces and dynamic dispatch.
* **`09_sieve_bool_array` (1M primes):** Evaluates dense array allocation and non-sequential reads/writes. Compares Solix flat byte slices against Lua hash/array hybrid tables and Python lists.
* **`14_exception_zero_cost`:**
  - *Part A (Zero-Cost Invariant):* 10M loop iterations wrapped in `try { ... }` that never throws. Proves that Solix incurs **0% CPU penalty** compared to Lua `pcall` or Python `try/except`.
  - *Part B (Unwinding Invariant):* 100k thrown and caught exceptions traversing 3 call frames with active destructors (`~Resource()`). Tests unwinding search and RAII safety.
* **`16_multi_binary_link` (5M calls):** Executes cross-module function calls across two `.slxb` containers, demonstrating that **In-Place Instruction Quickening** (`CallExtern` $\to$ `CallDirect`) eliminates dynamic symbol resolution after the first pass.

---

## 5. Architectural Telemetry Implementation Plan

To enable programmatic recording of these metrics, Solix crates expose telemetry instrumentation:

### 5.1 Memory Subsystem Telemetry (`VirtualHeap`)
Inside `crates/solix-runtime/src/memory/heap.rs`:
```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeapTelemetry {
    pub current_allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub total_allocations_count: u64,
    pub total_deallocations_count: u64,
    pub active_live_objects: usize,
}

impl VirtualHeap {
    pub fn telemetry(&self) -> HeapTelemetry { ... }
}
```

### 5.2 Virtual Machine Telemetry (`VirtualMachine`)
Inside `crates/solix-runtime/src/vm/engine.rs`:
```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VmTelemetry {
    pub pure_execution_time_ns: u64,
    pub instructions_dispatched: u64,
    pub peak_call_stack_depth: usize,
    pub native_calls_count: u64,
    pub quickened_instructions_count: u64,
}
```

### 5.3 Cross-Platform Process Telemetry (`ProcessTelemetry`)
Inside `crates/solix-client/src/telemetry.rs` (or runtime platform layer):
```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcessTelemetry {
    pub peak_rss_kb: u64,
    pub user_cpu_time_us: u64,
    pub system_cpu_time_us: u64,
}

pub fn capture_process_telemetry() -> ProcessTelemetry {
    // Linux: /proc/self/status or getrusage
    // macOS: getrusage(RUSAGE_SELF) with byte/KB normalization
    // Windows: K32GetProcessMemoryInfo / GetProcessMemoryInfo PeakWorkingSetSize
}
```

### 5.4 CLI Integration (`solix run --metrics-json` & `solix compile --metrics-json`)
When invoked with `--metrics-json`:
```json
{
  "workload": "01_fib_recursive",
  "status": "success",
  "exit_code": 0,
  "timings": {
    "total_process_wall_time_us": 142100,
    "pure_vm_execution_time_us": 138500,
    "binary_load_time_us": 1200,
    "static_init_time_us": 450
  },
  "memory": {
    "peak_virtual_heap_bytes": 24576,
    "peak_os_rss_kb": 4120,
    "heap_amplification_ratio": 1.05,
    "total_allocations": 1,
    "total_deallocations": 1,
    "active_leaks": 0
  },
  "execution": {
    "instructions_dispatched": 29850124,
    "peak_stack_depth": 36,
    "quickened_calls": 0
  }
}
```

---

## 6. Universal Cross-Platform Benchmark Harness (`runner.py`)

The upgraded benchmark harness orchestrates compilation, execution, output verification, memory recording, and markdown generation across Windows, macOS, and Linux:

```
benchmarks/
├── runner.py                   # Automated cross-platform test coordinator (Python 3.10+)
├── Cargo.toml                  # Rust Criterion benchmarks for in-process VM loops
├── config.json                 # Cross-platform compiler/interpreter paths and flags
├── baselines/                  # Saved JSON telemetry baselines (e.g. ms015.json)
└── workloads/                  # 16 benchmark cases implemented across all languages
    ├── 01_fib_recursive/
    │   ├── fib.slx
    │   ├── fib.lua
    │   ├── fib.py
    │   ├── fib.go
    │   └── fib.c
    ...
```

### 6.1 Verification and Correctness Gate
The runner **MUST** verify functional correctness before recording timings:
1. Every workload returns a deterministic integer result or prints a cryptographic/mathematical checksum.
2. The runner asserts that Solix, Lua, LuaJIT, Python, and Go produce **identical results**. If results differ, the benchmark is flagged as `FAIL` and excluded from timing reports.

### 6.2 Cross-Platform Measurement Protocol
* **Warmup:** 2 untimed runs to warm OS filesystem buffers and CPU frequency governors.
* **Timed Iterations:** 5 timed executions per engine.
* **Statistical Processing:** Computes **Median**, **Minimum**, **Maximum**, and **Standard Deviation ($\sigma$)**.
* **Universal Memory Tracking:**
  - Uses `psutil.Process().memory_info().rss` when available.
  - Fallback on Linux/macOS: `resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss` (with automatic `darwin` byte-to-KB scaling).
  - Fallback on Windows: `ctypes.windll.psapi.GetProcessMemoryInfo`.
* **Hardware Performance Profiling:**
  - Linux: Optional `perf stat` capturing `cycles`, `instructions`, `branch-misses`, and `L1-dcache-load-misses`.
  - Windows / macOS: Graceful degradation with clear informational notice, recording wall-clock and RSS metrics without failing.

### 6.3 Automated Baseline Comparison & Diffing
The runner supports automated commit-over-commit diffing:
```bash
# Save current state as baseline
python3 benchmarks/runner.py --save-baseline baselines/ms015.json

# Run against modified compiler and print differential comparison
python3 benchmarks/runner.py --compare baselines/ms015.json
```
The output displays speedup and memory delta percentages ($\Delta\%$):
```text
>>> Comparing with 'ms015.json':
  01_fib_recursive :   1582.3 ms ->   178.5 ms (-88.7% time, 8.86x faster) [PASSED]
  02_tight_loop_int:   2381.1 ms ->   512.4 ms (-78.5% time, 4.65x faster) [PASSED]
  05_object_churn  :   1815.7 ms ->   125.0 ms (-93.1% time, 14.5x faster) [PASSED]
  06_binary_trees  :   940.2 MB  ->    24.1 MB (-97.4% RAM, 39.0x leaner) [PASSED]
```

---

## 7. Phased Implementation Roadmap & Priorities

In accordance with `GL-0008`, Milestone 016 is executed across four disciplined phases:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   Milestone 016 Delivery Phases                        │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 1: CLI Direct Source Execution (solix run <files.slx>)           │
│          • In-memory compilation, positional sources & libs, tests     │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Core Telemetry Subsystems in Runtime & Compiler               │
│          • HeapTelemetry, VmTelemetry, ProcessTelemetry, --metrics-json│
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Universal Cross-Platform Benchmark Suite & Runner             │
│          • runner.py, 16 workloads (.slx, .lua, .py, .go, .c)          │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Conformance, Parity, Regression Verification & Documentation  │
│          • Correctness gate, safe vs fast parity, BENCHMARK_RESULTS.md │
└────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: CLI Direct Source Execution (`solix run <files.slx>`)
* **Task 1.1 (`P0`):** Enhance `RunArgs` in [`crates/solix-client/src/cli.rs`](../../../crates/solix-client/src/cli.rs) to accept an arbitrary positional mixture of `.slx` source files and `.slxb` binaries (with `--no-builtins` option, deliberately omitting `--lib` to reserve it for native shared libraries).
* **Task 1.2 (`P0`):** Implement automatic on-the-fly compilation in [`crates/solix-client/src/commands/run.rs`](../../../crates/solix-client/src/commands/run.rs) using `CompilationPipeline::standard()`, rendering compiler errors with `render_diagnostics`, and mounting the resulting `BinaryProgram` directly into `RuntimeOptions`.
* **Task 1.3 (`P1`):** Add exhaustive CLI test coverage in [`crates/solix-client/tests/cli_tests.rs`](../../../crates/solix-client/tests/cli_tests.rs) verifying single-source execution, multi-source execution, compilation error diagnostics, `--no-builtins`, and mixed positional source/binary executions.

### Phase 2: Core Telemetry Subsystems in Runtime & Compiler
* **Task 2.1 (`P0`):** Implement `HeapTelemetry` in [`crates/solix-runtime/src/memory/heap.rs`](../../../crates/solix-runtime/src/memory/heap.rs) tracking live objects, peak allocated bytes, and allocation/deallocation counters.
* **Task 2.2 (`P0`):** Implement `VmTelemetry` in [`crates/solix-runtime/src/vm/engine.rs`](../../../crates/solix-runtime/src/vm/engine.rs) recording pure VM execution duration, total dispatched instructions, peak call stack depth, and quickened instruction counts.
* **Task 2.3 (`P1`):** Implement cross-platform process RSS telemetry querying in `crates/solix-client/src/telemetry.rs` supporting Linux, macOS, and Windows.
* **Task 2.4 (`P1`):** Wire `--metrics` and `--metrics-json` flags into `solix run` and `solix compile`, reporting structured timing, memory, and dispatch statistics.

### Phase 3: Universal Cross-Platform Benchmark Suite & Runner
* **Task 3.1 (`P0`):** Implement `benchmarks/runner.py` with multi-OS support (Linux, macOS, Windows), `psutil` integration, native fallback memory tracking, and monotonic timing.
* **Task 3.2 (`P1`):** Author all 16 canonical benchmark workloads across Category A to Category E in Solix (`.slx`).
* **Task 3.3 (`P1`):** Author reference benchmark implementations in Lua, Python, Go, and C with matching mathematical/cryptographic output checksums.

### Phase 4: Conformance, Parity, Regression Verification & Documentation
* **Task 4.1 (`P0`):** Run the automated verification gate (`python3 benchmarks/runner.py --verify`) asserting identical computation results across Solix and comparative engines.
* **Task 4.2 (`P1`):** Validate safe mode (`--safe`) vs. fast mode parity and record the performance delta.
* **Task 4.3 (`P1`):** Generate baseline records and produce the comprehensive multi-dimensional report in `BENCHMARK_RESULTS.md`.

---

## 8. Definition of Done (DoD) Sign-Off Matrix

| Requirement | Target Criteria | Verification Tool | Priority | Sign-off Status |
| :--- | :--- | :--- | :--- | :--- |
| **Direct `.slx` Execution** | `solix run script.slx` compiles and runs without intermediate disk artifact | `cargo test -p solix-client -- test_cli_run_direct_source` | `P0` | [x] Complete |
| **Multi-Source `.slx` Run** | `solix run a.slx b.slx` compiles multi-file programs on-the-fly | `cargo test -p solix-client -- test_cli_run_multi_source` | `P0` | [x] Complete |
| **Compiler Phase Telemetry** | Microsecond phase breakdowns and MB/s throughput reported | `solix compile --metrics-json` | `P1` | [x] Complete |
| **Runtime & Memory Telemetry** | Pure VM time, Peak VirtualHeap, Alloc/Dealloc counts & Peak RSS | `solix run --metrics-json` | `P1` | [x] Complete |
| **Cross-Platform Memory RSS** | Accurate peak RSS recording on Windows, macOS, and Linux | `benchmarks/runner.py` | `P1` | [x] Complete |
| **16 Multi-Language Workloads** | All 16 workloads implemented in `.slx`, `.lua`, `.py`, `.c` with verified checksums | `python3 benchmarks/runner.py --verify` | `P1` | [x] Complete |
| **Safe vs. Fast Parity** | Verifies zero behavioral divergence between `--safe` and fast mode | `python3 benchmarks/runner.py --mode parity` | `P1` | [x] Complete |
| **Zero Workspace Regressions** | Full workspace test suite passes with 0 failures, 0 clippy warnings | `cargo test --workspace && cargo clippy` | `P0` | [x] Complete |
| **Automated Markdown Reporter** | Produces structured tabular reports with comparative speedup ratios | `BENCHMARK_RESULTS.md` | `P1` | [x] Complete |
