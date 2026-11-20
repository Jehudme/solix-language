# SwitchStatement

## 1. Overview & Purpose

A `SwitchStatement` evaluates a discriminant expression and branches to matching `case` clauses, falling back to an optional `default` branch when no candidate matches. In Solix `v2.0.0`, the `switch` statement supports universal primitives, strings, enums, class equality overloading (`operator ==` / `operator !=`), tuple value equality, and runtime `instanceof` pattern matching.

---

## 2. Supported Discriminant and Case Types

| Category | Discriminant Types | Case Label Syntax | Dispatch Opcode / Mechanism |
|---|---|---|---|
| **Integer Primitives** | `int8`, `uint8`, `int16`, `uint16`, `int32`, `uint32` | `case <int>:` | `OpCode::EqI32` |
| **64-bit Integers** | `int64`, `uint64` | `case <int64>:` | `OpCode::EqI64` |
| **Floating-Point** | `float32`, `float64` | `case <float>:` | `OpCode::EqF64` |
| **Booleans & Characters** | `bool`, `char` | `case true:`, `case 'c':` | `OpCode::EqBool` / `OpCode::EqI32` |
| **Strings** | `string` (`std::string`) | `case "literal":` | `OpCode::EqString` (interned pointer + byte compare) |
| **Enums** | Strongly-typed `enum` | `case Enum::Variant:`, `case Variant:` | `OpCode::EqI32` |
| **Nominal Classes** | Classes defining `operator ==` or `operator !=` | `case instance_var:` | Direct call to `operator_==` (or `operator_!=` with inversion) |
| **Tuples** | Tuple types `(T1, T2, ...)` | `case (val1, val2, ...):` | Element-wise `OpCode::GetTuple` and equality comparison |
| **Nominal Pattern Matching** | Reference types (`class`, `interface`) | `case instanceof TargetClass:` | `OpCode::Instanceof` via dynamic class vtable inspection |
| **Tuple Pattern Matching** | Tuple types `tuple` or `(T1, T2)` | `case instanceof (T1, T2):` | `OpCode::TupleInstanceof` per element type tag |
| **Null References** | Any nullable reference type | `case null:` | `OpCode::EqRef` pointer comparison against zero |

---

## 3. Class Equality Operator Dispatch

Classes used in equality switch cases must implement `operator ==` or `operator !=`. If neither operator overload is present on the class, the compiler rejects the switch with diagnostic `E0292`.

```solix
class ComplexNumber {
    public float64 real;
    public float64 imag;

    public ComplexNumber(float64 real, float64 imag) {
        this.real = real;
        this.imag = imag;
    }

    public bool operator ==(ComplexNumber other) {
        return this.real == other.real && this.imag == other.imag;
    }
}

int32 evaluate_state(ComplexNumber num) {
    ComplexNumber zero = new ComplexNumber(0.0, 0.0);
    ComplexNumber one = new ComplexNumber(1.0, 0.0);

    switch (num) {
        case zero: return 0;
        case one: return 1;
        default: return -1;
    }
}
```

---

## 4. Pattern Matching with `instanceof`

Cases may inspect runtime nominal types and tuple structure using `instanceof`:

```solix
class Node {}
class ElementNode extends Node { public string tag; }
class TextNode extends Node { public string text; }

string describe_node(Node node) {
    switch (node) {
        case instanceof ElementNode:
            return "element";
        case instanceof TextNode:
            return "text";
        case null:
            return "null";
        default:
            return "generic node";
    }
}
```

---

## 5. Diagnostic Codes

| Code | Name | Description |
|---|---|---|
| `E0292` | `SwitchClassMissingEqualityOperator` | Emitted when a class instance is compared in a switch case without implementing `operator ==` or `operator !=`. |
| `E0293` | `IncompatibleSwitchCaseType` | Emitted when a case value type cannot be assigned or compared to the discriminant type. |
| `E0294` | `DuplicateSwitchCaseConstant` | Emitted when duplicate case values or duplicate pattern targets are defined in the same switch. |
| `E0295` | `InvalidSwitchPatternTarget` | Emitted when `instanceof` targets a non-class, non-interface, non-tuple type. |
| `E0296` | `TupleArityMismatchInCase` | Emitted when the number of elements in a tuple case pattern mismatches the discriminant tuple arity. |
