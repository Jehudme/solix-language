# Formal Specification: Modules, Imports & Qualified Declarations

## Document Metadata

* **Specification ID:** SPEC-0012
* **Scope:** Module System, Namespace Scopes & Qualified Declarations (`crates/solix-compilation/src/phases/parser/statements/modules.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Syntax & EBNF Grammar

```ebnf
PackageStatement ::= "package" QualifiedIdentifier ";"
ImportStatement  ::= ["extern"] "import" QualifiedIdentifier [ ".*" ] ";"
AliasStatement   ::= "alias" Identifier "=" QualifiedIdentifier ";"
QualifiedIdentifier ::= Identifier ( "::" Identifier | "." Identifier )*
```

---

## 2. Package Declarations & Qualified Symbols

Solix v2.0 supports two complementary namespace declaration topologies:
1. **File-Level Package Header:**
   ```solix
   package solix.io;

   public class Console { ... }
   ```
2. **Inline Qualified Declarations:**
   ```solix
   public class graphics::rendering::Renderer {
       public static void initialize() { ... }
   }
   ```
   Symbols declared with inline qualifiers are placed directly into the specified namespace without requiring a file-level package header.

---

## 3. External Dependencies & `extern import`

The `extern import` directive instructs the compiler to resolve an external library package through the [`ModuleSolver`](file:///home/jehud/Projects/solix-language/crates/solix-common/src/solver.rs):

```solix
extern import solix.io.Console;
extern import raylib.*;
```

* The first identifier segment (`solix` or `raylib`) identifies the root package name.
* In Phase 0 or Phase 2, the compiler queries the package solver to fetch the latest or compatible `.slxb`/`.slxarc` package.
* Pre-parsed ASTs are injected into `CompilationContext.ast`, and external call sites emit `CallExtern` instructions.

---

## 4. Scope Resolution Operator (`::`)

* **Normative Law:** All static methods, static fields, and enum variants **MUST** be accessed using `::`.
* **Example:** `Console::println("text")`, `Math::sqrt(value)`.
* **Prohibition:** Accessing static members using the instance member access operator (`.`) emits diagnostic `E0226`.
