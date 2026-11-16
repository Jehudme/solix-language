# Solix Standard Library: VM Telemetry & Introspection (`solix.system.Vm`)

The `solix.system` package exposes deep runtime telemetry, heap inspection, and container metrics through `Vm` and `VmMetrics`.

## 1. Class Hierarchy & Types

* **`Vm`**: Static utility class providing direct access to 56 VM metric counters, cycle collection control, and execution tuning.
* **`VmMetrics`**: Structured snapshot container holding values for all 56 runtime metrics across 6 diagnostic domains:
  1. Memory & Heap (0..11)
  2. ARC & Cycle Collector (12..19)
  3. Call Stack (20..27)
  4. Functions & Execution (28..35)
  5. Container & Profile Context (36..43)
  6. Host Process & System (44..55)

## 2. API Reference

```solix
namespace solix.system;

public class Vm {
    public static int64 get_metric(int32 metric_id);
    public static VmMetrics metrics();
    public static int64 collect_cycles();
    public static void set_cycle_threshold(int32 threshold);
}
```

## 3. Usage Example

```solix
import solix.system.Vm;
import solix.system.VmMetrics;
import solix.io.Console;

int32 main() {
    VmMetrics snapshot = Vm::metrics();
    Console::print_line("Live Heap Blocks: " + snapshot.heap_live_blocks);
    Console::print_line("Instructions Run: " + snapshot.vm_instructions_executed);
    Console::print_line("Process PID:      " + snapshot.process_pid);
    return 0;
}
```
