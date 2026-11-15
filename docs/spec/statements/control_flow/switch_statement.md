# SwitchStatement

## 1. Overview & Purpose

A `SwitchStatement` evaluates an integral or enum selector expression and jumps to matching `case` constant labels, with an optional `default` fallback. Solix supports fall-through semantics unless interrupted by a `break`.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
switch (cmd) {
    case 1:
        run();
        break;
    default:
        stop();
        break;
}
```

```bytecode
// Compiled VM Bytecode
// r0 holds switch condition variable `command`
LoadI32 r1, 1                // Case constant 1
EqI32 r2, r0, r1             // Check match: command == 1
JumpIfTrue r2, <case_1_offset> // Jump to case 1 body
Jump <default_offset>        // Jump to default body

// Case 1 Body:
CallDirect r3, run_fn_id, r0, 0
Jump <switch_exit_offset>

// Default Body:
CallDirect r3, stop_fn_id, r0, 0
Jump <switch_exit_offset>

<switch_exit_ip>:
```
