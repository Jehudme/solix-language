# Formal Specification: Optimizer, AST Inliner & Superinstructions

## Document Metadata

* **Specification ID:** SPEC-0005
* **Scope:** AST Inlining & Bytecode Peephole Optimization (`crates/solix-compilation/src/phases/optimizer/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/guidelines/authoring-specs.md` (GL-0005)

---

## 1. AST Inliner Phase (`AstInlinerPhase`)

The AST inliner runs immediately prior to bytecode emission:

### 1.1 Inlining Eligibility Criteria
A call site is eligible for direct AST substitution if:
1. **Explicit Annotation:** The callee is annotated with the `inline` modifier.
2. **Heuristic Candidate:** The callee is a leaf function (contains no nested function calls or recursion), contains no loops (`while`, `for`), and has an AST statement count within the profile threshold (default `8` in `debug`/`test`, `16` in `release`).
3. **No Dynamic Dispatch:** Virtual methods invoked via VTable or interface methods cannot be inlined at compile time.

### 1.2 Inlining Procedure & Variable Hygiene
1. **Parameter Binding:** Argument expressions are evaluated into fresh synthetic local variables to preserve left-to-right evaluation order and prevent multiple evaluations of side-effecting arguments.
2. **Alpha-Renaming:** All local variables declared within the inlined body receive unique synthetic names (`__inlined_var_0`, `__inlined_var_1`) to avoid variable shadowing or variable capture in the caller's scope.
3. **Return Substitution:** The `return expr;` statement is replaced with an assignment to a temporary variable representing the call site's value.

---

## 2. Bytecode Peephole Optimizer (`BytecodeOptimizerPhase`)

The peephole optimizer slides a multi-instruction inspection window over emitted 64-bit instructions:

### 2.1 Redundant Instruction Elimination
* **No-Op Move:** `Mov(R_dst, R_src)` where `R_dst == R_src` is removed.
* **Dead Stores:** A store to a register that is immediately overwritten without an intervening read is pruned.

### 2.2 Branch Simplification
* Direct jumps to the next sequential instruction (`Jump(PC + 1)`) are stripped.
* Inverted conditional branches: A conditional branch over an unconditional jump is inverted to branch directly to the ultimate destination.

---

## 3. Superinstruction Synthesis

Superinstructions combine multiple operations into a single 64-bit instruction to minimize dispatch overhead in the VM:

| Pattern | Normal Micro-Ops | Fused Superinstruction | Gain |
| :--- | :--- | :--- | :--- |
| **Immediate Addition** | `LoadI32(R1, imm)` + `AddI32(R0, R0, R1)` | `AddI32Imm(R0, imm16)` | Eliminates 1 dispatch & 1 register allocation |
| **Immediate Comparison** | `LoadI32(R1, imm)` + `EqI32(R0, R0, R1)` | `JumpIfI32Imm(R0, imm, offset)` | Eliminates constant pool load and comparison |
| **Compare & Branch** | `EqI32(R0, R1, R2)` + `JumpIfFalse(R0, offset)` | `JumpIfNeqI32(R1, R2, offset)` | Merges comparison and branch into 1 opcode |
