# Solix Benchmark Results (Milestone MS-061)

Canonical cross-language and release-over-release benchmark results comparing **Solix (`solix_fast`)** against **PUC-Rio Lua**, **LuaJIT**, **Python 3**, and **GCC C (`-O3`)**, plus progression from **MS-060** to **MS-061**.

> [!NOTE]
> **Measurement Methodology:** Executed via [`benchmarks/runner.py`](../benchmarks/runner.py) in `fast` mode with `5` warmup runs and `11` measured iterations pinned to an idle core (`taskset -c 0`) on Linux (`AMD FX(tm)-8150 Eight-Core Processor`) to eliminate `schedutil` cross-core frequency scaling jitter across all five runtimes. Source baselines: [`ms060_baseline.json`](../benchmarks/baselines/ms060_baseline.json) and [`ms061_baseline.json`](../benchmarks/baselines/ms061_baseline.json).

---

## 1. Executive Summary

- **MS-061 vs. MS-060 (Release Regression Gate):** **1.34x** geometric mean speedup across all 16 suites (`15` faster, `1` stable, **`0` regressions** beyond 5%), with **`-63.6 MB`** cumulative RSS delta across the sequential benchmark sweep.
- **Solix vs. PUC-Rio Lua (`lua`):** Wins **14 of 16** benchmarks with **1.52x** geometric mean speedup (up to **6.92x** faster on `11_generic_containers` and **4.02x** faster on `05_object_churn_pod`).
- **Solix vs. Python 3 (`python3`):** Wins **16 of 16** benchmarks with **8.69x** geometric mean speedup (up to **19.06x** faster on `11_generic_containers`).
- **Solix vs. LuaJIT (`luajit`):** Beats tracing-JIT native machine code on **2 of 16** workloads (`11_generic_containers`: **5.40x** faster; `14_exception_zero_cost`: **3.60x** faster) and remains within **0.92x** on `06_binary_trees_arc` (`9.82 ms` vs. `9.03 ms`), with **0.64x** geometric mean overall across a pure bytecode interpreter vs. JIT.
- **Solix vs. GCC C (`-O3`):** **5.20x** geometric mean ratio across full-process end-to-end execution including startup, bytecode loading, bounds checks, and deterministic ARC.

---

## 2. MS-060 to MS-061 Progression

| Workload | MS-060 (ms) | MS-061 (ms) | Speedup | Memory Diff | Status |
| :--- | ---: | ---: | :---: | ---: | :--- |
| `01_fib_recursive` | 40.22 ms | **35.68 ms** | +11.3% (**1.13x**) | -30.0 MB | Faster |
| `02_tight_loop_int` | 32.42 ms | **29.40 ms** | +9.3% (**1.10x**) | -5.9 MB | Faster |
| `03_nbody_float64` | 6.47 ms | **5.27 ms** | +18.5% (**1.23x**) | -5.2 MB | Faster |
| `04_ackermann_stack` | 9.40 ms | **7.56 ms** | +19.6% (**1.24x**) | -5.2 MB | Faster |
| `05_object_churn_pod` | 5.11 ms | **3.88 ms** | +24.1% (**1.32x**) | -5.2 MB | Faster |
| `06_binary_trees_arc` | 16.26 ms | **9.82 ms** | +39.6% (**1.66x**) | -5.2 MB | Faster |
| `07_virtual_dispatch` | 13.54 ms | **8.80 ms** | +35.0% (**1.54x**) | -1.7 MB | Faster |
| `08_interface_dispatch` | 11.83 ms | **9.49 ms** | +19.8% (**1.25x**) | -1.7 MB | Faster |
| `09_sieve_bool_array` | 18.40 ms | **4.97 ms** | +73.0% (**3.70x**) | -1.7 MB | Faster |
| `10_matrix_multiply_2d` | 3.60 ms | **3.42 ms** | +5.0% (**1.05x**) | -1.7 MB | Faster |
| `11_generic_containers` | 4.76 ms | **4.98 ms** | -4.6% (0.96x) | -0.0 MB | Stable |
| `12_closures_capture` | 4.14 ms | **3.16 ms** | +23.7% (**1.31x**) | -0.0 MB | Faster |
| `13_function_pointers` | 10.86 ms | **7.12 ms** | +34.4% (**1.53x**) | -0.0 MB | Faster |
| `14_exception_zero_cost` | 4.13 ms | **3.78 ms** | +8.5% (**1.09x**) | -0.0 MB | Faster |
| `15_string_processing` | 11.86 ms | **9.40 ms** | +20.7% (**1.26x**) | -0.0 MB | Faster |
| `16_multi_binary_link` | 5.05 ms | **3.90 ms** | +22.8% (**1.29x**) | -0.0 MB | Faster |
| **Geometric Mean / Total** | — | — | **1.34x** | **-63.6 MB** | **15 Faster, 1 Stable, 0 Regressed** |

---

## 3. Cross-Language Wall-Clock Comparison (`MS-061`)

Median wall-clock execution time in milliseconds across all five engines (lower is better; ratios show competitor time divided by Solix time, so `> 1.00x` indicates Solix is faster).

| Workload | Solix Fast (ms) | PUC-Rio Lua (ms) | Solix vs Lua | LuaJIT (ms) | Solix vs LuaJIT | Python 3 (ms) | Solix vs Python | GCC C `-O3` (ms) |
| :--- | ---: | ---: | :---: | ---: | :---: | ---: | :---: | ---: |
| `01_fib_recursive` | **35.68** | 41.14 | **1.15x** | 8.34 | 0.23x | 98.10 | **2.75x** | 2.19 |
| `02_tight_loop_int` | **29.40** | 32.01 | **1.09x** | 9.42 | 0.32x | 319.36 | **10.86x** | 1.68 |
| `03_nbody_float64` | 5.27 | 4.94 | 0.94x | 2.18 | 0.41x | 64.36 | **12.21x** | 1.68 |
| `04_ackermann_stack` | **7.56** | 7.64 | **1.01x** | 3.14 | 0.42x | 41.66 | **5.51x** | 1.20 |
| `05_object_churn_pod` | **3.88** | 15.60 | **4.02x** | 2.01 | 0.52x | 62.71 | **16.16x** | 1.01 |
| `06_binary_trees_arc` | **9.82** | 14.30 | **1.46x** | 9.03 | 0.92x | 40.30 | **4.10x** | 2.47 |
| `07_virtual_dispatch` | **8.80** | 12.18 | **1.38x** | 6.72 | 0.76x | 82.77 | **9.41x** | 1.42 |
| `08_interface_dispatch` | **9.49** | 12.02 | **1.27x** | 3.37 | 0.36x | 90.05 | **9.49x** | 1.51 |
| `09_sieve_bool_array` | **4.97** | 8.41 | **1.69x** | 3.73 | 0.75x | 59.47 | **11.97x** | 1.37 |
| `10_matrix_multiply_2d` | 3.42 | 2.04 | 0.60x | 1.89 | 0.55x | 24.88 | **7.27x** | 1.04 |
| `11_generic_containers` | **4.98** | 34.46 | **6.92x** | 26.88 | **5.40x** | 94.92 | **19.06x** | 0.82 |
| `12_closures_capture` | **3.16** | 4.04 | **1.28x** | 1.79 | 0.57x | 31.56 | **9.99x** | 0.89 |
| `13_function_pointers` | **7.12** | 9.81 | **1.38x** | 3.60 | 0.51x | 72.08 | **10.12x** | 1.28 |
| `14_exception_zero_cost` | **3.78** | 13.78 | **3.65x** | 13.61 | **3.60x** | 29.97 | **7.93x** | 0.89 |
| `15_string_processing` | **9.40** | 11.39 | **1.21x** | 3.76 | 0.40x | 66.33 | **7.06x** | 2.86 |
| `16_multi_binary_link` | **3.90** | 4.97 | **1.27x** | 2.14 | 0.55x | 37.08 | **9.51x** | 0.90 |
| **Summary** | **14/16 vs Lua** | **16/16 vs Py** | **1.52x geomean** | **2/16 vs LuaJIT** | **0.64x geomean** | — | **8.69x geomean** | **5.20x C ratio** |

---

## 4. Key Architectural Takeaways

1. **Why Solix Wins So Decisively on `11_generic_containers` (`5.40x` vs. LuaJIT) and `14_exception_zero_cost` (`3.60x` vs. LuaJIT):**
   - **Monomorphized Typed Arrays vs. Hash-Table Polymorphism (`11_generic_containers`):** Solix statically monomorphizes generic container types into contiguous packed storage with unboxed scalar operations, whereas Lua and LuaJIT must represent containers over dynamic tables with hash/array probes and polymorphic dispatch.
   - **Zero-Cost Static Handler Tables vs. `pcall` Frame Setup (`14_exception_zero_cost`):** Solix `try` blocks emit zero runtime instructions on the non-throwing fast path (`ExceptionHandlerEntry` PC ranges live strictly in static metadata), whereas Lua and LuaJIT must execute protected call wrappers (`pcall`) inside tight loops.
2. **Primary MS-061 Drivers:**
   - **`09_sieve_bool_array` (`3.70x` speedup, `18.40 ms -> 4.97 ms`):** Exact 1-byte `bool[]` `AllocArray` layout (`100 KB` vs `800 KB`), fused `GetArrayJumpIfFalse` (`0xDF`), bulk primitive array-fill loop detection (`ForLoopI32` `0x20`), and local CSE / loop rotation.
   - **`06_binary_trees_arc` (`1.66x` speedup, `16.26 ms -> 9.82 ms`):** `FLAG_MOVE` (`0x80`) `SetField` ownership transfers eliminate `IncRef`/`DecRef` churn and false-positive cycle-collector sweeps alongside lazy `purple_buffer` initialization.
   - **`07_virtual_dispatch` (`1.54x`) & `13_function_pointers` (`1.53x`):** Trivial 1–2 instruction leaf fast-call evaluation in `'call_dispatch`, branch-diamond 0-capture `PackClosure` specialization to `CallDirect`, leaf inlining, and borrowed alias pruning from `reference_registers`.
3. **Remaining Gap on `03_nbody_float64` & `10_matrix_multiply_2d`:**
   - These two short workloads (`~2–5 ms` total wall-clock) include ~1.6 ms of full ELF process startup and CLI initialization compared to the small `lua` C binary; pure VM execution already rivals or beats interpreted Lua on inner loops.
