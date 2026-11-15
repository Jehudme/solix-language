# Formal Specification: Zero-Cost Exception Handling & Unwinding

## Document Metadata

* **Specification ID:** SPEC-0008
* **Scope:** Exception Runtime, Unwinding & Try/Catch/Finally Tables (`crates/solix-runtime/src/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Table-Driven Exception Architecture

Solix implements a zero-cost exception handling model. When no exceptions are thrown, `try`/`catch` blocks incur zero runtime overhead—there is no runtime registration or stack push during normal control flow.

Each compiled function contains an immutable table of [`ExceptionHandlerEntry`](../../crates/solix-common/src/bytecode/function.rs) records:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionHandlerEntry {
    pub try_start_pc: u32,
    pub try_end_pc: u32,
    pub catch_type_symbol: Option<String>,
    pub catch_reg: u16,
    pub handler_pc: u32,
}
```

---

## 2. Unwinding State Machine

When a `throw exception_object;` instruction executes at runtime:

```text
                     Throw Instruction
                            │
                            ▼
           ┌──────────────────────────────────┐
           │ Search ExceptionHandler Table    │
           │ for [try_start_pc..try_end_pc]   │
           └────────────────┬─────────────────┘
                            │
               ┌────────────┴────────────┐
               │ Match?                  │ No Match
               ▼                         ▼
   ┌───────────────────────┐  ┌───────────────────────┐
   │ Check Exception Type  │  │ Unwind Frame          │
   │ via instanceof        │  │ • Release local refs  │
   └───────────┬───────────┘  │ • Pop call frame      │
               │              └───────────┬───────────┘
         ┌─────┴─────┐                    │
   Match │           │ No Match           ▼
         ▼           └────────────► Check Caller Frame
┌─────────────────────────┐               │
│ Bind catch_reg          │               ▼
│ Jump to handler_pc      │         Top of Stack?
│ Resume execution        │               │
└─────────────────────────┘               ▼
                                 Uncaught Exception Trap
                                 Emit Stack Trace & Exit
```

---

## 3. Finally Block Execution & RAII Interleaving

`finally` blocks execute deterministically regardless of whether the `try` block completed normally, executed an early `return`, or threw an unhandled exception:
1. **Normal Flow:** The compiler lowers `finally` blocks to execute immediately prior to leaving the block boundary.
2. **Unwinding Flow:** During exception unwinding, the runtime invokes intermediate `finally` handlers before searching higher frames.
3. **ARC Scope Cleanup:** Local references declared inside a `try` block are cleanly released prior to entering a `catch` or `finally` handler.
