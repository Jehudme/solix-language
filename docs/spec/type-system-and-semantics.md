# Formal Specification: Type System, Hierarchy & Semantics

## Document Metadata

* **Specification ID:** SPEC-0002
* **Scope:** Type Checking & Semantic Type Hierarchy (`crates/solix-compilation/src/phases/binder/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-common/src/types.rs`](file:///home/jehud/Projects/solix-language/crates/solix-common/src/types.rs)

---

## 1. Type System Taxonomy

Solix possesses a static, nominal type system with structural anonymous tuples and first-class function pointer signatures:

```text
Type
├── Primitive Types
│   ├── Signed Integers:   int8, int16, int32, int64
│   ├── Unsigned Integers: uint8, uint16, uint32, uint64
│   ├── Floating Point:    float32, float64
│   ├── Logical:           bool
│   ├── Character:         char (32-bit Unicode scalar)
│   ├── String:            string (primitive UTF-8 heap string)
│   └── Bottom / Unit:     void
├── Composite & Structural Types
│   ├── Arrays:            T[]
│   ├── Tuples:            (T1, T2, ..., Tn)
│   └── Function Pointers: RetType(*)(Param1, Param2, ..., ParamN)
└── Reference & Nominal Types
    ├── Classes:           Nominal reference objects (single inheritance)
    ├── Interfaces:        Abstract contracts (multiple implementation)
    ├── Enums:             32-bit integer-backed nominal enumerations
    └── Generics:          Parameterized blueprints instantiated at monomorphization
```

---

## 2. Primitive Types & Memory Layouts

| Primitive Type | Bit Width | Memory Alignment | Value Range | Default Value | Friendly Aliases |
| :--- | :---: | :---: | :--- | :--- | :--- |
| `int8` | 8 | 1 byte | $-128$ to $127$ | `0` | `sbyte` |
| `int16` | 16 | 2 bytes | $-32,768$ to $32,767$ | `0` | `short` |
| `int32` | 32 | 4 bytes | $-2^{31}$ to $2^{31}-1$ | `0` | `int` |
| `int64` | 64 | 8 bytes | $-2^{63}$ to $2^{63}-1$ | `0` | `long` |
| `uint8` | 8 | 1 byte | $0$ to $255$ | `0` | `byte` |
| `uint16` | 16 | 2 bytes | $0$ to $65,535$ | `0` | `ushort` |
| `uint32` | 32 | 4 bytes | $0$ to $2^{32}-1$ | `0` | `uint` |
| `uint64` | 64 | 8 bytes | $0$ to $2^{64}-1$ | `0` | `ulong` |
| `float32` | 32 | 4 bytes | IEEE-754 Single | `0.0f` | `float` |
| `float64` | 64 | 8 bytes | IEEE-754 Double | `0.0` | `double` |
| `bool` | 8 | 1 byte | `true` or `false` | `false` | `boolean` |
| `char` | 32 | 4 bytes | Unicode U+0000 to U+10FFFF | `'\0'` | — |
| `string` | 64 | 8 bytes (Ref) | UTF-8 byte slice on VirtualHeap | `""` | `String` |
| `void` | 0 | 0 bytes | Absence of value | — | — |

---

## 3. Reference Types & Object Model

### 3.1 Classes
Classes are nominal reference types managed deterministically by Automatic Reference Counting (ARC) on the `VirtualHeap`:
* **Single Inheritance:** Every class inherits from at most one direct superclass. If no superclass is specified, it descends from the root object contract.
* **Scope Resolution for Static Members:** All static methods and static fields **MUST** be accessed through `::` (e.g. `Math::max(first_value, second_value)`). Attempting to access static members via dot (`.`) emits diagnostic `E0226`.

### 3.2 Interfaces
Interfaces define pure contract method signatures. A class can implement multiple interfaces (`implements ISerializable, IComparable<T>`).
* Interface method dispatch is resolved via dynamic itables (`itable_id`) with sliding offset quickening during execution.

---

## 4. Subtyping & Conversions

### 4.1 Implicit Conversions
Implicit conversions are allowed only where no information loss can occur (widening numeric promotions):
* `int8` $\to$ `int16` $\to$ `int32` $\to$ `int64` $\to$ `float64`.
* `uint8` $\to$ `uint16` $\to$ `uint32` $\to$ `uint64` $\to$ `float64`.
* Subclass reference $\to$ Superclass reference $\to$ Implemented Interface reference.

### 4.2 Explicit Casts
Narrowing numeric conversions or downcasting require explicit syntax:
1. **C-Style Cast:** `(int32)long_value`
2. **Intrinsic Cast:** `cast(long_value, int32)`
3. **Type Inspection:** `instanceof(object_reference, TargetClass)` evaluates to `bool`.
4. **Size Inspection:** `sizeof(Type)` returns byte width as `uint32`.
5. **Default Construction:** `default(Type)` initializes zero-state memory.
