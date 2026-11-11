# BlockStatement

## 1. Overview & Purpose

A `BlockStatement` (`{ ... }`) establishes a new lexical scope for variable declarations. It groups a sequence of zero or more statements into a single executable unit.

Variables declared within a block are only visible inside that block and its nested blocks. When execution exits the block—either normally or abruptly via `return`, `break`, `continue`, or `throw`—any active reference-counted variables declared in the block are immediately decremented via Automatic Reference Counting (ARC).

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### How It Compiles
When the compiler encounters `{ ... }`, it opens a new lexical scope. At the end of the block, it emits:
1. **Normal Scope Cleanup**: Emits `DecRef r<reg>` in reverse order of declaration for all active reference variables.
2. **Zero-Cost Unwinding**: Any exception thrown within the block uses the static function exception table (`ExceptionHandlerEntry`) to identify and decrement reference registers active in the block without requiring inline landing pads.

### Bytecode Disassembly Example
```solix
// Solix Code
{
    int32 x_coord = 42;
    String message = new String("hello");
}
```

```bytecode
// Compiled Register VM Bytecode
LoadI32 r0, 42
Mov r1, r0                 // r1 = 42 (x_coord)

LoadConst r2, 0            // Constant string pool index for "hello"
CallDirect r3, string_new_fn_id, r2, 1 // r3 = new String("hello") (ref_count = 1)

// --- Normal Scope Exit ---
DecRef r3                  // Decrement message (ref_count 1 -> 0, immediately freed!)
```

---

## 3. Syntax & Grammar

```
BlockStatement
    : '{' Statement* '}'
    ;
```

---

## 4. Semantic Rules & Scope

- **Shadowing**: A variable declared inside a block may not redeclare a variable with the same name in the same scope. Shadowing outer-scope variables is permitted.
- **Lifetime**: The lifetime of variables declared in the block ends at the closing brace `}`.

---

## 5. ARC & Memory Management

The compiler precisely tracks register allocation and reference lifetimes:
- Primitives (`int32`, `float64`, `bool`, etc.) require zero teardown.
- Reference types (`String`, class instances, arrays) have `DecRef` emitted in LIFO order upon exiting the block.

---

## 6. Diagnostic Errors & Common Pitfalls

- **E0202 (Duplicate Variable)**: Declaring two variables with the same name in the same block.
- **E0201 (Unresolved Identifier)**: Attempting to access a variable outside its declaring block.
