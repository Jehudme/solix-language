# Exception Handling & Zero-Cost Stack Unwinding

## 1. Overview

Solix provides structured exception handling through `try-catch-finally` blocks and `throw` statements. The exception system guarantees that active reference variables are deterministically decremented (`DecRef`) and `finally` blocks execute reliably, even when an exception propagates across deeply nested call frames.

Solix employs a **zero-cost exception model**: normal execution through a `try` block executes zero setup or teardown instructions. Exception handling overhead is incurred only when an exception is actually thrown.

---

## 2. Zero-Cost Exception Metadata Table

Rather than executing dynamic registration instructions at runtime entry to a `try` block, the Solix emitter generates static metadata tables embedded in each compiled function:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionHandlerEntry {
    /// Starting bytecode instruction offset (inclusive).
    pub try_start_pc: u32,
    /// Ending bytecode instruction offset (exclusive).
    pub try_end_pc: u32,
    /// Expected exception class symbol (`None` matches any Exception type).
    pub catch_type_symbol: Option<String>,
    /// Virtual register allocated to receive the exception instance reference.
    pub catch_reg: u16,
    /// Target bytecode offset of the catch handler block.
    pub handler_pc: u32,
    /// Optional target bytecode offset of the `finally` block.
    pub finally_pc: Option<u32>,
    /// Reference-holding registers active inside the `try` scope for unwinding cleanup.
    pub try_ref_regs: Vec<u16>,
}
```

---

## 3. Unwinding & Dispatch Mechanics

When a `throw` statement executes:

```mermaid
flowchart TD
    ThrowInstruction["Throw (OpCode 0xB0)\nRead Exception Object Address"] --> SearchHandlers["Search Function Exception Table\nfind_matching_handler(handlers, current_pc)"]
    SearchHandlers --> CheckHandler{"Handler Found?"}
    CheckHandler -->|Yes| CheckType{"Exception instanceof catch_type_symbol?"}
    CheckType -->|Match| DispatchCatch["1. Decrement try_ref_regs in unwound scope\n2. Store exception into catch_reg\n3. Jump to handler_pc"]
    CheckType -->|Mismatch| NextHandler["Search Next Outermost Handler"]
    NextHandler --> CheckHandler
    CheckHandler -->|No| CheckFinally{"finally_pc Pending?"}
    CheckFinally -->|Yes| ExecFinally["Execute Finally Trampoline"]
    ExecFinally --> PopFrame["Pop Current Call Frame\nDecrement Frame Registers"]
    CheckFinally -->|No| PopFrame
    PopFrame --> SearchCaller["Search Caller Exception Table"]
    SearchCaller --> CheckHandler
```

### 3.1. Reverse Table Lookup
Exception handler search ([`crates/solix-runtime/src/exceptions/unwinder.rs`](../../crates/solix-runtime/src/exceptions/unwinder.rs)) searches the handler table in reverse order:
```rust
pub fn find_matching_handler(
    handlers: &[ExceptionHandlerEntry],
    program_counter: u32,
) -> Option<&ExceptionHandlerEntry> {
    handlers
        .iter()
        .rev()
        .find(|entry| program_counter >= entry.try_start_pc && program_counter < entry.try_end_pc)
}
```
Reverse lookup guarantees that innermost nested `try-catch` blocks take precedence over outer enclosing blocks.

### 3.2. Polymorphic Catch Matching
Catch blocks check type compatibility by traversing the class inheritance hierarchy via the object's virtual table ID. A handler matches if:
1. `catch_type_symbol` is `None` (untyped or catch-all handler).
2. The thrown instance's class symbol equals `catch_type_symbol`.
3. The thrown instance's class inherits from `catch_type_symbol`.

### 3.3. Deterministic ARC Cleanup During Unwinding
When propagating out of a guarded scope, the virtual machine tracks reference registers allocated within that scope (`try_ref_regs`). Before entering the `catch` block or popping the call frame, the runtime invokes `dec_ref()` on all active reference registers, ensuring zero leaks during exception propagation.

---

## 4. Guaranteed `finally` Execution

`finally` blocks are compiled as inline trampolines:
- **Normal Flow**: The `try` body completes, falls through into the `finally` block, and continues to subsequent statements.
- **Return in Try**: Any `return` inside `try` jumps to a synthesized trampoline that executes the `finally` code before returning.
- **Exception Flow**: If an exception occurs, the unwinder executes the `finally` trampoline before re-throwing or transferring control to outer handlers.
