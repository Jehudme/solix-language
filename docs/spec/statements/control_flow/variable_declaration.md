# VariableDeclarationStatement

## 1. Overview & Purpose

A `VariableDeclarationStatement` introduces one or more named variables into the current scope. It binds an identifier to a static type, allocates a register in the function's activation frame, and optionally assigns an initial value.

In Solix:
- **Primitives (Value Types)**: Numbers, booleans, and characters (`int32`, `float64`, `bool`, `char`) hold raw bits directly in virtual registers and require no reference counting.
- **Reference Types**: Classes, interfaces, strings, and **all arrays** (including `int32[]`) hold heap addresses. Creating or copying a reference variable manages ownership via Automatic Reference Counting (`IncRef` and `DecRef`).

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### How It Compiles
1. The binder resolves the static type. If `array_depth > 0 || !is_primitive`, it flags the variable as a reference type.
2. The register allocator assigns a local virtual register (`r<target>`).
3. If an initializer expression is present:
   - Emits instructions computing the expression into register `r<src>`.
   - If copying an existing reference, emits `IncRef r<src>`.
   - Assigns `r<target> = r<src>` via `Mov` or direct register targeting.

### Bytecode Disassembly Example
```solix
// Solix Code
int32 count = 10;
String name = new String("Solix");
```

```bytecode
// Compiled Register VM Bytecode
LoadI32 r0, 10
Mov r1, r0                             // r1: count = 10 (value type, no ref counting)

LoadConst r2, 0                        // Constant pool index for "Solix"
CallDirect r3, string_new_fn_id, r2, 1 // r3: name = new String("Solix") (ref_count = 1)
```

---

## 3. Syntax & Grammar

```
VariableDeclarationStatement
    : Type Identifier ('=' Expression)? (',' Identifier ('=' Expression)?)* ';'
    ;
```

---

## 4. Semantic Rules & Scope

- **Type Consistency**: The initializer expression must evaluate to a type that is implicitly convertible or identical to the declared variable type.
- **Scope**: The variable is accessible from the point of its declaration until the end of its enclosing block.
- **No Type Inference**: Solix requires explicit types on all variable declarations.

---

## 5. ARC & Memory Management

Reference variables are registered in the frame's reference bitmask so that runtime stack unwinding, parameter passing, and scope exit automatically invoke `DecRef`.

---

## 6. Diagnostic Errors & Common Pitfalls

- **E0202 (Duplicate Variable)**: Attempting to declare multiple variables with the identical name in the same scope.
- **E0203 (Type Mismatch)**: Attempting to assign an incompatible type without an explicit cast.
