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
