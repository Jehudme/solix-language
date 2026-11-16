# Formal Specification: Deterministic Memory Model, VirtualHeap & ARC

## Document Metadata

* **Specification ID:** SPEC-0007
* **Scope:** Memory Allocator, Automatic Reference Counting & Object Lifecycle (`crates/solix-runtime/src/memory/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-runtime/src/memory/heap.rs`](../../crates/solix-runtime/src/memory/heap.rs)

---

## 1. VirtualHeap Allocator Architecture

The Solix runtime manages object allocations via the [`VirtualHeap`](../../crates/solix-runtime/src/memory/heap.rs), a segregated free-list memory allocator:

```text
┌────────────────────────────────────────────────────────┐
│ VirtualHeap Segregated Size Classes                    │
├────────────────────────────────────────────────────────┤
│ • Small Classes:  32, 64, 128, 256, 512, 1024 bytes    │
│ • Medium Classes: 2048, 4096, 8192 bytes               │
│ • Large Blocks:   Direct contiguous chunk allocations  │
│ • Memory Boundary Retraction & Coalescing              │
└────────────────────────────────────────────────────────┘
```

---

## 2. BlockHeader Layout (24 Bytes)

Every allocated heap object begins with an immutable 24-byte header:

```text
┌─────────────────────────┬─────────────────────────┐
│ Strong Ref Count (u32)  │ Weak Ref Count (u32)    │ 0..7
├─────────────────────────┼─────────────────────────┤
│ Class ID (u32)          │ Total Size Bytes (u32)  │ 8..15
├─────────────────────────┼─────────────────────────┤
│ Flags (u32)             │ Destructor Function(u32)│ 16..23
└─────────────────────────┴─────────────────────────┘
```

* **`strong_ref_count`**: Number of active strong references keeping the object alive.
* **`weak_ref_count`**: Number of weak references. Keeps the header alive after payload deallocation to prevent dangling pointer traps.
* **`class_id`**: Identifies class metadata and layout for field tracing and dynamic type inspection.
* **`destructor_fn_id`**: Slot ID of the user-defined `~ClassName()` destructor (or `0xFFFFFFFF` if absent).

---

## 3. ARC Object Lifecycle & Deterministic RAII

```text
                     AllocObject
                          │
                          ▼
             ┌─────────────────────────┐
             │ ALIVE                   │
             │ strong_refs >= 1        │
             │ payload fully accessible│
             └────────────┬────────────┘
                          │
                          │ strong_refs == 0
                          ▼
             ┌─────────────────────────┐
             │ EXECUTING DESTRUCTOR    │
             │ Invokes ~ClassName()    │
             │ Releases child refs     │
             └────────────┬────────────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
  weak_refs == 0                 weak_refs > 0
            │                           │
            ▼                           ▼
┌────────────────────────┐  ┌────────────────────────┐
│ FREED                  │  │ ZOMBIE                 │
│ Block returned to heap │  │ Payload zeroed         │
│ Segregated free list   │  │ Header awaits weak=0   │
└────────────────────────┘  └────────────────────────┘
```

1. **Deterministic Scope Exit (RAII):**
   * When execution leaves a local scope (via normal fall-through, `return`, `break`, `continue`, or `throw`), the runtime automatically emits `ReleaseRef` instructions for all active reference registers.
2. **Recursive Child Cleanup:**
   * When an object dies, its reference field offsets are read from its `CompiledClass` descriptor. All child object references are released recursively, freeing entire object graphs deterministically without garbage collection pauses.
3. **Leak-Free Invariant:**
   * Clean execution guarantees zero remaining heap allocations: `vm.verify_clean_memory() == Ok(())`.

---

## 4. Bacon & Rajan (2001) Synchronous Cycle Collector

To collect circular reference graphs while retaining deterministic ARC deallocation for acyclic objects, Solix integrates the Bacon & Rajan (2001) four-color synchronous cycle collection algorithm:

### Four-Color State Machine

* **`Black` (0):** Normal in-use or free heap block.
* **`Gray` (1):** Candidate member of a suspected cyclic subgraph undergoing simulated trial deletion.
* **`White` (2):** Confirmed unreachable garbage cycle candidate scheduled for sweeping.
* **`Purple` (3):** Suspected root of a potential cycle, buffered when an object with outgoing references is decremented to a non-zero count.

### Algorithm Phases

1. **Trial Deletion (`MarkGray`):** Traverses candidate subgraphs from purple roots, decrementing internal reference counts on outgoing child pointers to isolate internal versus external references.
2. **Cycle Confirmation (`Scan`):** Evaluates reference counts. If an object's reference count is $> 0$, external references exist: it and all reachable children are repainted `Black` and counts restored via `ScanBlack`. If reference count is $0$, the node is painted `White`.
3. **Reclamation (`CollectWhite`):** Traverses confirmed `White` nodes, repainting them `Black` and returning heap memory to the segregated free list or quick lists via `force_deallocate`.

### Zero-Cost Opt-Out & Tuning

* **CLI Flags:**
  * `--cycle-collector <on|off>`: Enables or completely bypasses purple root buffering. When disabled, execution runs pure ARC with zero candidate tracking overhead.
  * `--cycle-threshold <N>`: Candidate buffer size triggering automatic collection (default: 1024). Specifying `0` rejects with diagnostic `[E0341]`.
  * `--cycle-budget-us <N>`: Bounded execution budget in microseconds. If exceeded, deferred roots trigger warning `[W0340]`.
* **Guest API (`solix.system.Vm`):**
  * `Vm::collect_cycles() -> int64`: Explicitly triggers synchronous cycle collection.
  * `Vm::set_cycle_threshold(int32 threshold)`: Dynamically configures the collection threshold.

