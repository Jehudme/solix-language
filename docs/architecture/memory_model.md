# Byte-Addressed Memory Model & Alignment Architecture

## 1. Overview

The Solix Virtual Machine operates on a high-performance, byte-addressed virtual memory space (`VirtualHeap` implemented in [`crates/solix-runtime/src/memory/heap.rs`](../../crates/solix-runtime/src/memory/heap.rs)). Virtual memory is partitioned into:
1. **Static Address Space**: Contiguous byte segment at the base of virtual memory storing global and static class variables.
2. **Dynamic Heap Space**: Byte-addressed region governed by Automatic Reference Counting (ARC) and a segregated free-list manager (`FreeListManager`).

All heap management structures are implemented in safe Rust under `#![forbid(unsafe_code)]`, while the monomorphized VM dispatch loop (`crates/solix-runtime/src/vm/fast.rs`) executes pre-validated fast-path buffer accesses guarded by explicit bounds checks and `// SAFETY:` invariants. Allocation payload sizes are verified using overflow-safe checked arithmetic (`checked_mul`, `checked_add`) capped at `u32::MAX`, immediately failing with `RuntimeError::OutOfMemory` if arithmetic wrapping would occur. Tuple stores in `execute_set_tuple` validate that target indices are strictly within `tuple_len`, returning `RuntimeError::IndexOutOfBounds` rather than writing into adjacent heap memory. `VirtualHeap` allocates demand-paged backing storage starting at `32 KB` (`INITIAL_DYNAMIC_COMMIT`) and grows lazily up to configured `dynamic_capacity`, paired with a `1,024`-slot initial `register_stack` and a `512`-frame pre-reserved `CallFrame` stack (`32B` layout caching `inst_ptr`), while the cycle collector's `purple_buffer` initializes empty and grows only when potential cyclic objects are decremented.

---

## 2. Heap Block & 8-Byte Header Layout

Every dynamically allocated heap block (objects, primitive arrays, reference arrays, closures, strings, tuples) resides in the dynamic heap partition with an 8-byte packed header positioned immediately before the payload address:

```text
+------------------------------------+-------------------------------------------+
| Bytes 0..4 (32 bits)               | Bytes 4..8 (32 bits)                      |
+------------------------------------+-------------------------------------------+
| Payload Size / Count (u32)         | Tag (4b) | GcColor (2b) | RefCount (26b)  |
+------------------------------------+-------------------------------------------+
^ Address - 8                                                                    ^ Base Payload Address (Address + 0)
```

### 2.1. Header Packing Specification
The 8-byte header ([`crates/solix-runtime/src/memory/layout.rs`](../../crates/solix-runtime/src/memory/layout.rs#L138-L241)) packs:
- **`size` (Bytes 0..4)**: 32-bit unsigned integer holding the allocated byte size or array element count.
- **`tag_and_ref` (Bytes 4..8)**:
  - **Bits `[31:28]` (4 bits)**: Strongly typed `BlockTag` indicating the runtime representation.
  - **Bits `[27:26]` (2 bits)**: Tricolor `GcColor` state used by the Bacon & Rajan cycle collector.
  - **Bits `[25:0]` (26 bits)**: Active reference counter (`ref_count`). Maximum allowable reference count is `67,108,863` (`MAX_REF_COUNT = 0x03FF_FFFF`).

### 2.2. Block Tags (`BlockTag`)

| Tag Hex | Identifier | Description | Child Pointer Scanning |
|---|---|---|---|
| `0x00` | `Raw` | Raw byte buffer | None |
| `0x01` | `Object` | Class instance with virtual method table | Traverses reference fields via VTable metadata |
| `0x02` | `ArrayU8` | Homogeneous unsigned 8-bit byte array (`uint8[]`, `bool[]`) | None (primitive payload) |
| `0x03` | `ArrayI32` | Homogeneous signed 32-bit integer array (`int32[]`) | None (primitive payload) |
| `0x04` | `ArrayI64` | Homogeneous signed 64-bit integer array (`int64[]`) | None (primitive payload) |
| `0x05` | `ArrayF64` | Homogeneous signed 64-bit float array (`float64[]`) | None (primitive payload) |
| `0x06` | `ArrayRef` | Dynamic reference array holding object or collection pointers | Traverses each element pointer |
| `0x07` | `Tuple` | Heterogeneous tuple structure | Traverses reference fields |
| `0x08` | `Closure` | Closure environment holding function ID and captures | Traverses captured reference variables |
| `0x09` | `String` | Heap-allocated immutable UTF-8 string | None (character bytes) |
| `0x0A` | `ObjectPod` | Plain-Old-Data (POD) class instance without reference fields | None (fast single-pass deallocation) |

---

## 3. Natural Field Alignment in Class Instances

During class layout computation ([`crates/solix-compilation/src/phases/binder/layout.rs`](../../crates/solix-compilation/src/phases/binder/layout.rs)), fields are packed contiguously while adhering to standard natural alignment rules:

| Type Category | Primitive Types | Width in Bytes | Alignment Boundary |
|---|---|---|---|
| **1-byte** | `bool`, `char`, `int8`, `uint8` | 1 | 1-byte boundary |
| **2-byte** | `int16`, `uint16` | 2 | 2-byte boundary |
| **4-byte** | `int32`, `uint32`, `float32`, `enum` | 4 | 4-byte boundary |
| **8-byte** | `int64`, `uint64`, `float64`, Class references, `weak` references, Arrays | 8 | 8-byte boundary |

Field offsets start immediately after the 8-byte VTable ID slot at `address + 0`. Padding bytes are inserted whenever a field's current offset is not an integer multiple of its natural alignment requirement. Total instance allocation size is rounded up to the nearest multiple of 8 bytes (`ALIGNMENT = 8`).

```text
Class Instance Memory Structure:
+-------------------+--------------------+--------------------+--------------------+
| Address + 0 (8B)  | Address + 8 (4B)   | Address + 12 (4B)  | Address + 16 (8B)  |
+-------------------+--------------------+--------------------+--------------------+
| VTable ID (u64)   | Field 0 (int32)    | Field 1 (Padding)  | Field 2 (ObjectRef)|
+-------------------+--------------------+--------------------+--------------------+
```

---

## 4. Array Representation & Stride

Arrays in the byte-addressed heap have a standardized memory header:
- **`address + 0 .. address + 8`**: 64-bit unsigned integer recording element count (`length`).
- **`address + 8 .. end`**: Contiguous payload elements packed according to element size:
  - Stride 1: `uint8[]`, `int8[]`, `bool[]`, `char[]`
  - Stride 2: `uint16[]`, `int16[]`
  - Stride 4: `uint32[]`, `int32[]`, `float32[]`
  - Stride 8: `uint64[]`, `int64[]`, `float64[]`, object references (`ARRAY_REF`)

Opcodes accessing arrays compute offset via `effective_address = base_address + 8 + (element_index * stride)`. Boundary checks against `length` are enforced before every read or write operation, raising `RuntimeError::IndexOutOfBounds` on invalid indices. Primitive field writes in slow execution paths (`execute_set_field_slow`) strictly propagate heap bounds errors via `RuntimeError` traps instead of silent error drops. Cycle collection traces outgoing reference arrays across all object and reference tags (`tag == 0 || tag >= 13`) with 8-byte element size.

---

## 5. Segregated Free-List Allocation & Recycling

When an object's reference counter drops to zero, the runtime does not leak memory or require compaction sweeps. Instead:
1. Destructors and child reference decrements are triggered immediately.
2. The deallocated memory block is coalesced with adjacent free blocks in physical memory.
3. The coalesced block is returned to 32 power-of-two segregated size-class bins (`size_bins` indexed by a `u32` `non_empty_bins` bitmask) within `FreeListManager` ([`crates/solix-runtime/src/memory/deallocator.rs`](../../crates/solix-runtime/src/memory/deallocator.rs)), avoiding per-size `BTreeSet` heap allocations.
4. Subsequent allocations match against `quick_lists` (<= 256B), the monotonic bump allocator (skipping redundant zero-fills above `high_water_zeroed`), or `FreeListManager` via `trailing_zeros()` bitmask lookup for $O(1)$ memory reuse.

---

## 6. Heap Telemetry & Leak Detection

The runtime tracks memory operations via `HeapTelemetry`:
- `current_allocated_bytes`: Active dynamically allocated bytes.
- `peak_allocated_bytes`: Maximum recorded heap utilization.
- `total_allocations_count` & `total_deallocations_count`: Monotonically increasing operation counters.
- `active_live_objects`: Count of active dynamic allocations (internal VM telemetry helper `active_guest_objects()` excludes immortal `interned_string_addrs` constant-pool strings so `--metrics` and CLI leak detection accurately report `0` active guest leaks for clean programs).

Upon virtual machine termination, if guest allocations remain live (`active_guest_objects() > 0`), the runtime diagnostic layer can dump all active allocations and their backtraces to pinpoint leaks.

During DAP debugger inspection (`VirtualMachine::inspect_object_fields`), class reference fields are identified directly via `CompiledClass::reference_field_offsets` before scalar type-tag matching so nested object references (`player.inner.coord_x`) always resolve as `Value::Ref` / `Value::Null`.

