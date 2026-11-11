# Meta-Guideline: Authoring Engineering Guidelines

## Document Metadata
* **Guideline ID:** GL-0004
* **Scope:** Universal / Meta-Standard (`docs/internals/guidelines/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Runtime Maintainers, Documentation Stewards
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

Engineering guidelines define the architectural constitution of the Solix language project. If guideline documents are written haphazardly, inconsistently formatted, or lack normative precision, the engineering integrity of the entire compiler pipeline and virtual machine runtime collapses. Without a rigorous meta-standard governing the guidelines themselves, individual guidelines drift into subjective opinion pieces, omit critical failure mode analyses, or become ignored by contributors.

This meta-guideline establishes the non-negotiable structural requirements, review processes, lifecycle states, and normative standards governing the authoring, amendment, and retirement of all guideline documents within `docs/internals/guidelines/`. It enforces the canonical seven-section structure, RFC 2119 keyword rigor, and concrete Solix language contextualization across the entire documentation suite.

---

## 2. Normative Invariants & Core Principles

Every developer guideline within `docs/internals/guidelines/` **MUST** adhere to the following binding invariants:

1. **Mandatory Seven-Section Layout:** Every guideline file **MUST** contain exactly the seven canonical sections defined in Section 3 of this specification, in the exact sequence specified. No section **MAY** be omitted, combined, or renamed.
2. **Deterministic Identifier Assignment:** Every guideline **MUST** be assigned a unique, sequential identifier formatted as `GL-XXXX` (e.g., `GL-0001`, `GL-0012`). Once allocated, a guideline identifier **SHALL NOT** be reassigned to another topic, even if the original guideline is deprecated.
3. **Strict RFC 2119/8174 Keyword Usage:** Normative requirements **MUST** use uppercase RFC 2119 keywords (**MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, **OPTIONAL**). The document metadata **MUST** explicitly cite RFC 2119 / RFC 8174 conformance.
4. **Implementation-Agnostic Normative Core:** Guidelines **MUST** establish invariants and standards that transcend temporary implementation details. They specify *how* engineering must be performed, *what* invariants must be maintained, and *why* specific failure modes must be avoided, without pasting ephemeral compiler build logs or transient debug sessions.
5. **Mandatory Concrete Counter-Examples:** Section 6 ("Anti-Patterns & Counter-Examples") **MUST** present at least three concrete, technical counter-examples illustrating flawed practices, their exact catastrophic failure modes in the Solix compiler, VM runtime, or CLI, and their compliant corrections.
6. **Master Index Synchronization:** Any pull request adding, modifying, or deprecating a guideline **MUST** synchronously update the internal guidelines index within the same commit.

---

## 3. Structural Specification & Schema

### 3.1 Guideline Lifecycle State Machine

A guideline document progresses through four deterministic states:

```
    ┌──────────┐
    │  DRAFT   │
    └────┬─────┘
         │ (PR opened, initial authoring complete)
         ▼
    ┌──────────┐
    │  REVIEW  │ ◄──────────────────────────────┐
    └────┬─────┘                                │
         │ (Two Maintainer sign-offs)           │ (Substantive amendment proposed)
         ▼                                      │
    ┌─────────────────────────────────┐         │
    │  FINAL IMPLEMENTATION STANDARD  │ ────────┘
    └────────────────┬────────────────┘
                     │ (Superseded by newer standard)
                     ▼
             ┌──────────────┐
             │  DEPRECATED  │
             └──────────────┘
```

1. **`DRAFT`**: Document is undergoing active composition. Not yet normative.
2. **`REVIEW`**: Document is undergoing formal review in a GitHub pull request.
3. **`FINAL IMPLEMENTATION STANDARD`**: Fully approved and merged into `master`. Legally binding on all code and architecture.
4. **`DEPRECATED`**: Replaced by a newer standard or rendered obsolete by architectural evolution. Preserved with a deprecation notice.

### 3.2 Canonical Seven-Section Specification

Every guideline document **MUST** adhere to the following schema:

1. **Title & Document Metadata:**
   * Level-1 header (`# <Title>`) followed immediately by the `## Document Metadata` block containing: `Guideline ID`, `Scope`, `Status`, `Audience`, and `Normative Conformance`.
2. **`## 1. Executive Summary & Purpose`:**
   * Concise statement of why the standard exists, the engineering failure modes it prevents, and the foundational guarantees it maintains across the Solix language project.
3. **`## 2. Normative Invariants & Core Principles`:**
   * Numbered list of non-negotiable rules. Every rule **MUST** employ bold uppercase RFC 2119 keywords.
4. **`## 3. Structural Specification & Schema`:**
   * The formal definitions, state machines, grammars, or tabular schemas governing the domain.
5. **`## 4. Production-Ready Template`:**
   * A complete, copy-pasteable template with bracketed placeholders and guidance comments.
6. **`## 5. Concrete Production Example`:**
   * A realistic, comprehensive example specifically contextualized to real Solix components (`solix-compilation`, `solix-runtime`, `solix-common`, `solix-toolchain`, `solix-lsp`, `solix-dbg`, VM stack frames, ARC memory management, AST lowering, opcode dispatch).
7. **`## 6. Anti-Patterns & Counter-Examples (What NOT to Do)`:**
   * Itemized failure modes detailing flawed code/behavior, the specific operational catastrophe it causes, and the corrected pattern.
8. **`## 7. Verification & Compliance Checklist`:**
   * An actionable markdown task-list (`- [ ]`) specifying automated and manual review criteria.

---

## 4. Production-Ready Template

Engineers proposing a new guideline document **MUST** populate the following exact template:

```markdown
# [Title of Guideline]

## Document Metadata
* **Guideline ID:** GL-[XXXX]
* **Scope:** [Subsystems affected, or "Universal / Workspace-Wide"]
* **Status:** DRAFT
* **Audience:** [Target Engineering Roles]
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `SHOULD`, `MAY`)

---

## 1. Executive Summary & Purpose
<!-- 
Direct, uncompromising explanation of why this standard exists, the engineering 
failure modes it prevents, and the foundational guarantees it maintains across 
the Solix language ecosystem.
-->
[Provide executive summary here]

## 2. Normative Invariants & Core Principles
<!-- 
A numbered list of non-negotiable rules. Every rule MUST use RFC 2119 keywords 
in bold capital letters.
-->
1. **[Principle Name]:** Engineers **MUST** ...
2. **[Principle Name]:** Implementations **MUST NOT** ...
3. **[Principle Name]:** Subsystems **SHALL** ...

## 3. Structural Specification & Schema
<!-- 
The exact syntax, fields, format, or architectural layout that engineers must follow. 
Include tables, grammars, or state machines.
-->
[Provide structural specification and schema here]

## 4. Production-Ready Template
<!-- 
A fully usable, copy-pasteable Markdown/Text template containing bracketed placeholders, 
commented hints, and standard front-matter.
-->
[Provide production-ready template here]

## 5. Concrete Production Example
<!-- 
A realistic, comprehensive example specifically contextualized to Solix 
(referring to AST parsing, semantic analysis, opcode dispatch, stack frames, or ARC).
-->
[Provide concrete production example here]

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)
<!-- 
At least 3 realistic flawed patterns, explaining the exact technical failure 
mode or runtime catastrophe they cause, and providing the corrected form.
-->
### Anti-Pattern 1: [Flawed Pattern Name]
* **Flawed Practice:** [Description of mistake]
* **Failure Mode / Catastrophe:** [Technical catastrophe in compiler or runtime]
* **Corrected Form:** [Compliant implementation or specification]

### Anti-Pattern 2: [Flawed Pattern Name]
* **Flawed Practice:** [Description of mistake]
* **Failure Mode / Catastrophe:** [Technical catastrophe in compiler or runtime]
* **Corrected Form:** [Compliant implementation or specification]

### Anti-Pattern 3: [Flawed Pattern Name]
* **Flawed Practice:** [Description of mistake]
* **Failure Mode / Catastrophe:** [Technical catastrophe in compiler or runtime]
* **Corrected Form:** [Compliant implementation or specification]

## 7. Verification & Compliance Checklist
<!-- 
A markdown task-list (- [ ]) providing automated linting rules or an engineer 
sign-off audit before merging.
-->
- [ ] **Structural Completeness:** All 7 canonical sections are present and titled correctly.
- [ ] **RFC 2119 Conformance:** Normative keywords are capitalized and strictly conform to RFC 2119.
- [ ] **Contextual Realism:** Examples reference real Solix crates, language syntax, or VM ISA.
- [ ] **Master Index Updated:** Internal guidelines directory index includes this guideline.
```

---

## 5. Concrete Production Example

### Proposal for a New Guideline: GL-0012 (Virtual Machine Memory Safety & ARC Lifecycle)

Below is an excerpt demonstrating how a new guideline is authored adhering strictly to this meta-standard:

```markdown
# Virtual Machine Memory Safety & ARC Lifecycle Specification

## Document Metadata
* **Guideline ID:** GL-0012
* **Scope:** Virtual Machine & Memory Management (`crates/solix-runtime/`, `crates/solix-common/`)
* **Status:** DRAFT
* **Audience:** Core Systems Engineers, Runtime Architects
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `SHOULD`, `MAY`)

---

## 1. Executive Summary & Purpose
The Solix virtual machine (`solix-runtime`) executes untrusted compiled bytecode managing heap-allocated objects, closures, arrays, and classes via Automatic Reference Counting (ARC). Improper reference tracking, unhandled circular references, or missing stack frame barriers can cause heap exhaustion, dangling pointers, or use-after-free bugs. This specification defines the mandatory invariants for reference count modification, weak pointer handling, and deterministic destructor invocation during stack unwinding.

## 2. Normative Invariants & Core Principles
1. **Deterministic Reference Decrement:** When a local variable slot or operand stack element leaves scope, its associated reference count **MUST** be decremented immediately. If the strong count reaches zero, the object's destructor **MUST** execute before memory deallocation.
2. **Cycle Prevention via Weak References:** Back-references and parent pointers in object graphs **MUST** be represented using `WeakReference` tokens. Strong reference cycles **SHALL NOT** be created by the compiler code generator.
3. **No Unchecked Pointer Arithmetic:** The runtime virtual machine **MUST NOT** perform raw unchecked pointer arithmetic when reading operand values from bytecode buffers; all operand reads **MUST** validate buffer bounds.
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Section Omission or Renaming
* **Flawed Practice:** Creating a guideline that omits Section 6 ("Anti-Patterns & Counter-Examples") because the author believes "positive examples are sufficient."
* **Failure Mode:** Engineers repeatedly make the exact same architectural errors (e.g., unchecked operand stack popping, leaking circular references under ARC) because the boundary between acceptable and catastrophic failure modes was never documented.
* **Corrected Approach:** Always include Section 6 with at least three detailed, real-world failure analyses.

### Anti-Pattern 2: Colloquial or Uncapitalized RFC Keywords
* **Flawed Practice:** Writing: "It is recommended that the compiler should validate type compatibility."
* **Failure Mode:** Ambiguity during code review. Developers argue whether "should" in lowercase was a strict mandate or an optional suggestion, leading to unresolved type-safety holes in production.
* **Corrected Approach:** Use bold, capitalized keywords: "The semantic binder **MUST** validate type compatibility before emitting bytecode."

### Anti-Pattern 3: Abstract or Fictional Examples
* **Flawed Practice:** Using generic `FooBarService` and `ExampleWidget` in Section 5 instead of real Solix compiler and VM architecture.
* **Failure Mode:** Contributors struggle to apply abstract concepts to concrete crates (`solix-compilation`, `solix-runtime`, `solix-common`), leading to misinterpretations during implementation.
* **Corrected Approach:** Ground every example in actual Solix subsystems, AST nodes, opcodes, and interfaces.

---

## 7. Verification & Compliance Checklist

Authors and maintainers **MUST** verify the following checklist before approving any guideline PR:

- [ ] **Sequential ID Allocated:** Guideline ID follows `GL-XXXX` and is strictly sequential.
- [ ] **Metadata Block Present:** Document header contains all five required metadata fields.
- [ ] **Exact 7 Sections:** The document includes sections 1 through 7 with identical naming and ordering.
- [ ] **RFC 2119 Capitalization:** All normative rules utilize bold, capitalized RFC 2119 keywords.
- [ ] **Three Real Anti-Patterns:** Section 6 contains at least three concrete failure modes with catastrophe descriptions and corrected forms.
- [ ] **Copy-Paste Template Included:** Section 4 contains an unabbreviated, usable template.
- [ ] **Master Index Updated:** Internal guidelines index reflects the new guideline ID and details.
