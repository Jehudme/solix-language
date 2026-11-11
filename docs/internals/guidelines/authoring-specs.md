# Language & Subsystem Specification Invariants & Schema

## Document Metadata
* **Guideline ID:** GL-0005
* **Scope:** Architectural, Language & Subsystem Specifications (`docs/internals/specs/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, VM Runtime Engineers, Tooling Authors
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

Subsystem specifications in `docs/internals/specs/` serve as the primary architectural blueprints and behavioral contracts for the Solix language, compiler, virtual machine, and toolchain. A recurring defect in systems engineering is the conflation of *specifications* with *code implementations*. When engineers paste raw Rust implementation functions into specifications, the document quickly rots, drifts from the compiler, and obscures core architectural guarantees with transient code details.

This specification formalizes how all core dimensions of the Solix language platform **MUST** be specified:
1. **Language Statements, Keywords & Grammar:** Formal BNF syntax, AST representation, semantic validation, and scoping rules for all language statements (control flow, declarations, expressions, modules).
2. **VM Instruction Set Architecture (ISA) & Opcodes:** Complete binary encoding, mnemonic naming, operand byte layouts, stack effects, and trap conditions for every virtual machine opcode.
3. **VM Runtime & Opcode Execution Management:** The exact mechanics of how the virtual machine manages opcode dispatch, instruction pointers, call frames, local variables, the operand stack, Automatic Reference Counting (ARC) heap memory, deterministic destructor execution, exception unwinding tables, and instruction budgeting.
4. **CLI Commands & Tooling Options:** Detailed specifications of all command-line commands, flags, arguments, environment variables, exit codes, and output formatting.

---

## 2. Normative Invariants & Core Principles

All specifications within `docs/internals/specs/` **MUST** strictly adhere to the following normative invariants:

1. **The Separation Invariant (Zero Multi-Line Implementation Code):** Specifications **MUST NOT** contain raw multi-line Rust implementation code. Implementation logic belongs in source crates (`crates/*`). Specifications govern *what* language guarantees and binary formats exist, while the compiler and runtime govern *how* they are executed.
2. **Exhaustive Language Statement Coverage:** Every keyword, statement, declaration, and expression supported by the Solix language **MUST** be documented in `docs/internals/specs/statements/` with:
   * Formal Backus-Naur Form (BNF) grammar.
   * Abstract Syntax Tree (AST) node structure.
   * Semantic analysis, type checking, and scope resolution rules.
   * Lowering strategy to bytecode opcodes.
3. **Deterministic Opcode Specification:** Every opcode in `docs/internals/specs/vm_isa.md` **MUST** be defined with:
   * Hexadecimal byte value (`0x00` - `0xFF`).
   * Explicit operand layout (byte width, endianness, signedness).
   * Formal stack effect formula: `[stack_before] -> [stack_after]`.
   * Trap conditions that halt execution or trigger runtime exceptions.
4. **Explicit Runtime Management Invariants:** Specifications for runtime opcode execution **MUST** document:
   * **Call Frame Lifecycle:** Activation record layout, return address preservation, and local variable frame offsets.
   * **Memory & ARC Management:** Object header layout, strong/weak reference tracking, and deterministic destructor invocation upon reference drop.
   * **Exception Handling & Unwinding:** Exception table search algorithm, catch block jump resolution, and finally block execution invariants.
   * **Instruction Quota & Traps:** Quota decrements, deterministic budget exhaustion traps, and division-by-zero protection.
5. **Deterministic CLI Option Specification:** Every CLI command in `docs/internals/specs/cli/` **MUST** specify all options, short flags, data types, defaults, environment variables, standard exit codes, and `clap` derive schemas.
6. **Normative Precedence:** Subsystem specifications outrank code implementations, developer guides, and discussions. Code that contradicts an approved specification is non-compliant and **MUST** be corrected.

---

## 3. Structural Specification & Schema

### 3.1 Taxonomy of Subsystem Specifications

The `docs/internals/specs/` directory is partitioned into four core domains:

```text
docs/internals/specs/
├── statements/                 # Language syntax, keywords, AST nodes, and statements
│   ├── control_flow/           # if, while, for, do_while, switch, return, break, continue, throw, try
│   ├── declarations/           # class, interface, enum, constructor, destructor, method, field, generic
│   ├── expressions/            # binary, unary, call, member_access, new, array, cast, lambda
│   └── modules/                # package, import, alias
├── vm_isa.md                   # Complete VM Instruction Set Architecture (Opcodes & Bytecode format)
├── runtime/                    # Virtual machine execution, memory model, and runtime mechanics
│   ├── execution_loop.md       # Opcode dispatch loop, instruction pointer, and stack frames
│   ├── arc_memory_model.md     # Reference counting, object headers, and destructor invocation
│   └── exception_unwinding.md  # Unwinding tables, catch search, and finally execution
└── tools/cli/                  # CLI toolchain commands and arguments
    ├── new.md                  # 'solix new' project scaffolding
    ├── compile.md              # 'solix compile' command and options
    ├── build.md                # 'solix build' command and options
    ├── run.md                  # 'solix run' command and options
    ├── info.md                 # 'solix info' command and options
    ├── install.md              # 'solix install' command and options
    ├── uninstall.md            # 'solix uninstall' command and options
    ├── list.md                 # 'solix list' command and options
    └── manifest.md             # 'manifest.toml' specification
```

### 3.2 Opcode Specification Schema

Every opcode documented in `docs/internals/specs/vm_isa.md` **MUST** follow this structured tabular schema:

```markdown
### `OP_<NAME>` (`0x<HEX>`)

* **Mnemonic:** `OP_<NAME>`
* **Opcode Value:** `0x<HEX>` (`u8`)
* **Operands:** `<Operand 1>` (`<type>`), `<Operand 2>` (`<type>`)
* **Stack Transition:** `[operand_a, operand_b] -> [result]`
* **Description:** [Formal semantics and VM behavior]
* **Runtime Traps:**
  * `Trap::StackUnderflow` if fewer than $N$ items exist on the operand stack.
  * `Trap::<SpecificError>` under specific invalid conditions.
```

### 3.3 Statement Specification Schema

Every statement documented in `docs/internals/specs/statements/` **MUST** follow this schema:

1. **Syntax & BNF Grammar:** Formal grammar defining tokens and production rules.
2. **AST Node Structure:** Abstract representation of the statement node.
3. **Scoping & Semantic Rules:** Variable visibility, shadowing, and type constraints.
4. **Bytecode Lowering:** Sequence of emitted opcodes and jump labels.
5. **Diagnostics & Error Conditions:** Compiler error codes (`E0XXX`) emitted for syntax or semantic violations.

---

## 4. Production-Ready Templates

### 4.1 Language Statement Specification Template

```markdown
# [Statement Name] Statement Specification

## Document Metadata

* **Specification ID:** SPEC-STMT-[XXX]
* **Category:** Control Flow / Declaration / Expression / Module
* **Status:** Final Implementation Standard
* **Affected Crates:** `solix-compilation`, `solix-common`

---

## 1. Syntax & BNF Grammar

```bnf
<statement-name> ::= "keyword" <expression> "{" <statement-list> "}"
```

## 2. Abstract Syntax Tree (AST) Node

| AST Field | Rust Type | Description |
| :--- | :--- | :--- |
| `condition` | `Box<ExpressionNode>` | Evaluated conditional expression |
| `body` | `BlockStatementNode` | Enclosed body statement block |
| `span` | `SourceSpan` | Precise source location for diagnostics |

## 3. Semantic Analysis & Type Rules
1. The `condition` expression **MUST** resolve to type `bool`.
2. Any variable declared within `body` **MUST NOT** be visible outside the statement scope.
3. Breaking out of the statement via `break` **MUST** be validated against loop depth.

## 4. Bytecode Lowering & Opcode Sequence

```text
LOOP_START:
    <evaluate condition>
    OP_JUMP_IF_FALSE -> LOOP_END
    <evaluate body>
    OP_JUMP -> LOOP_START
LOOP_END:
```

## 5. Diagnostics & Error Codes

| Error Code | Condition | Diagnostic Message |
| :--- | :--- | :--- |
| `E0201` | Condition is not boolean | `mismatched types: expected bool, found {type}` |
| `E0202` | Unreachable statement | `unreachable code detected after return` |
```

### 4.2 VM Opcode Specification Template

```markdown
### `OP_<NAME>` (`0x<HEX>`)

* **Mnemonic:** `OP_<NAME>`
* **Byte Value:** `0x<HEX>`
* **Operands:**
  | Operand | Width | Type | Description |
  | :--- | :--- | :--- | :--- |
  | `operand_1` | 2 bytes | `u16` (Little Endian) | Jump offset or constant pool index |
* **Operand Stack Transition:**
  $$\text{Stack: } [\dots, \text{val}_1, \text{val}_2] \longrightarrow [\dots, \text{result}]$$
* **VM Execution Semantics:**
  1. Pop $\text{val}_2$ from the top of the operand stack.
  2. Pop $\text{val}_1$ from the top of the operand stack.
  3. Compute $\text{result} = \text{val}_1 \odot \text{val}_2$.
  4. Push $\text{result}$ onto the operand stack.
* **Traps & Fault Conditions:**
  * `Trap::StackUnderflow`: Operand stack depth $< 2$.
  * `Trap::IntegerOverflow`: Arithmetic overflow with overflow checking enabled.
```

---

## 5. Concrete Production Examples

### Example 1: Language Statement Spec (`while_statement.md`)

```markdown
# While Statement Specification

## 1. Syntax & BNF Grammar

```bnf
<while-statement> ::= "while" <expression> <block-statement>
```

## 2. AST Representation

| Field | Type | Description |
| :--- | :--- | :--- |
| `condition` | `ExpressionNode` | Boolean guard expression |
| `body` | `BlockStatementNode` | Statements executed while condition is true |
| `span` | `SourceSpan` | Byte offset span for error reporting |

## 3. Bytecode Lowering Pattern

```text
.loop_header:
    [evaluate condition]
    OP_JUMP_IF_FALSE .loop_exit
    [evaluate body statements]
    OP_JUMP .loop_header
.loop_exit:
```

## 4. Diagnostic Invariants
* If `condition` evaluates to any type other than `bool`, the compiler **MUST** emit diagnostic `E0140: ConditionExpressionMustBeBool`.
```

### Example 2: VM Opcode Spec (`OP_CALL_METHOD`)

```markdown
### `OP_CALL_METHOD` (`0x42`)

* **Mnemonic:** `OP_CALL_METHOD`
* **Byte Value:** `0x42`
* **Operands:**
  | Operand | Width | Type | Description |
  | :--- | :--- | :--- | :--- |
  | `method_index` | 2 bytes | `u16` | Index of method descriptor in constant table |
  | `argument_count`| 1 byte  | `u8`  | Number of arguments passed on stack |
* **Stack Transition:**
  $$[\dots, \text{receiver\_instance}, \text{arg}_1, \dots, \text{arg}_N] \longrightarrow [\dots, \text{return\_value}]$$
* **Runtime Execution Management:**
  1. Verify $\text{receiver\_instance}$ is non-null. If null, trap with `Trap::NullPointerDereference`.
  2. Allocate a new `CallFrame` with base pointer pointing to arguments.
  3. Increment ARC reference counts on all argument objects.
  4. Set instruction pointer (`ip`) to method entry point address.
* **Traps:**
  * `Trap::NullPointerDereference`: When receiver instance is null.
  * `Trap::StackOverflow`: When call frame depth exceeds `MAX_CALL_DEPTH` ($1,024$).
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Pasting Rust Match Arms Instead of Specifying Opcodes
* **Flawed Practice:** Pasting a 60-line `match opcode { ... }` function from `crates/solix-runtime/src/vm.rs` directly into `vm_isa.md`.
* **Failure Mode:** When the VM refactors its internal registers or switches dispatch strategies, the specification becomes out of date. Contributors can no longer determine whether the code or the spec is authoritative.
* **Corrected Approach:** Document the opcode value, operands, stack effect formula, and operational semantics abstractly.

### Anti-Pattern 2: Unspecified Operand Stack Effects
* **Flawed Practice:** Writing: "`OP_ADD` adds numbers together" without documenting what it pops, what it pushes, or stack underflow conditions.
* **Failure Mode:** The compiler emits code leaving extra operands on the stack. The VM stack grows unboundedly across function calls until the process crashes.
* **Corrected Approach:** Always specify the exact stack transition formula: `[val1, val2] -> [result]`.

### Anti-Pattern 3: Omitting Traps and Failure Modes
* **Flawed Practice:** Documenting `OP_DIV` without specifying behavior when dividing by zero.
* **Failure Mode:** One developer implements division-by-zero by returning zero; another panics with Rust `panic!()`; another returns NaN. The runtime becomes non-deterministic and crashes host processes.
* **Corrected Approach:** Explicitly state: "If denominator is zero, the VM **MUST** trap with `Trap::DivisionByZero`."

---

## 7. Verification & Compliance Checklist

Authors and maintainers **MUST** verify the following criteria for every specification PR:

- [ ] **Zero Implementation Code:** No raw multi-line Rust source code exists in the specification.
- [ ] **BNF Grammar Defined:** Every language statement includes formal BNF production grammar.
- [ ] **Stack Transition Formulated:** Every opcode specifies exact stack effects using `[before] -> [after]`.
- [ ] **Runtime Traps Tabulated:** All failure conditions, panics, and traps are explicitly identified.
- [ ] **CLI Options Tabulated:** All CLI commands specify types, defaults, and exit codes.
- [ ] **Diagnostic Codes Documented:** Compiler error codes associated with the syntax are registered.
