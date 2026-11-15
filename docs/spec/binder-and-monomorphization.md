# Formal Specification: Binder, VTables, ITables & Monomorphization

## Document Metadata

* **Specification ID:** SPEC-0004
* **Scope:** Semantic Analysis, Symbol Resolution & Type Checking (`crates/solix-compilation/src/phases/binder/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-compilation/src/phases/binder/binder.rs`](../../crates/solix-compilation/src/phases/binder/binder.rs)

---

## 1. Three-Pass Semantic Binder Architecture

The semantic binder executes three sequential passes over all AST nodes in `CompilationContext`:

```text
┌────────────────────────────────────────────────────────┐
│ Pass 1: Global Symbol Declaration Collection          │
│   • Scans classes, interfaces, enums, and functions    │
│   • Builds TypeRegistry & nominal type handles        │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ Pass 2: Type Hierarchy & Memory Layout Computation     │
│   • Resolves extends and implements clauses            │
│   • Computes field offsets and memory alignments       │
│   • Generates virtual tables (VTables) and ITables     │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│ Pass 3: Method Bodies, Type Checking & Monomorphization│
│   • Resolves variable scopes and statement semantics   │
│   • Enforces scope resolution (::) for static members  │
│   • Instantiates generic templates and monomorphizes   │
└────────────────────────────────────────────────────────┘
```

---

## 2. Pass 1: Declaration Registration

* Registers all declared nominal types (`class`, `interface`, `enum`) into the global `TypeRegistry`.
* Detects duplicate type declarations within the same scope (`E0205`).
* Records function signatures (return type, parameter types, modifiers).

---

## 3. Pass 2: Hierarchy, VTables & ITables

### 3.1 Inheritance Cycle Detection
Traverses class inheritance chains to ensure acyclic relationships. Circular inheritance (`class A extends B` where `class B extends A`) emits diagnostic `E0207`.

### 3.2 VTable Construction
* Inherits the base class vtable entries in exact order.
* Overriding methods replace the base slot with the derived implementation.
* New virtual methods are appended with monotonically increasing slot IDs.
* Methods marked `override` without matching a base virtual method emit diagnostic `E0208`.

### 3.3 ITable Mapping
* For each implemented interface, builds an interface dispatch table mapping interface method slot IDs to concrete class method slot IDs.

---

## 4. Pass 3: Type Checking & Monomorphization

### 4.1 Scope Resolution Enforcement
* All static members and enum variants **MUST** use `::` (`Math::PI`, `Status::Active`).
* Accessing a static member via dot (`instance.static_member`) emits diagnostic `E0226`.

### 4.2 Generic Monomorphization
* Generic classes (`List<T>`) and generic methods (`Algorithms::sort<T>`) act as uncompiled blueprints during initial parsing.
* When referenced with concrete type arguments (`List<int32>`), the binder clones the AST template, substitutes type parameters with concrete types, and runs semantic type-checking.
* Monomorphized symbols are mangled deterministically (e.g. `List$int32`).
