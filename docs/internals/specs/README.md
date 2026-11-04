# Solix Formal Technical Specifications Index

## Document Metadata
* **Scope:** Universal Compiler, Virtual Machine ISA, Runtime Engine, and Tooling Specifications
* **Status:** Scaffolding Active
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## Specification Taxonomy

The formal specifications in this directory define the normative contract for the Solix language implementation:

* **Statements & Grammar (`statements/`):**
  * Control flow statements (`if`, `while`, `for`, `switch`, `return`, `break`, `continue`, `try_catch_finally`).
  * Declarations (`class`, `interface`, `enum`, `constructor`, `field`, `method`, `operator`).
  * Expressions (`binary`, `unary`, `assignment`, `call`, `member_access`, `literal`, `lambda`).
* **Virtual Machine Instruction Set Architecture (`vm_isa.md`):**
  * Bytecode container layout (`.slxbin`), 16-byte magic header, section table, opcode encoding.
* **Deterministic Runtime Engine (`runtime/`):**
  * ARC memory management, operand stack invariants, exception unwinding, native plugin C-FFI.
* **Command-Line Interface (`cli/`):**
  * Command specifications for `solix compile`, `run`, `check`, `build`, `pack`, `package`, `lsp`, and `dbg`.
