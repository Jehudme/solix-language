# Milestone 047: Documentation Modernization, Alert Callouts & Strict Namespace Syntax

## Document Metadata
* **Milestone ID:** MS-047
* **Target Version:** `v1.1.4`
* **Milestone Branch:** `milestone/047-documentation-modernization-alert-callouts-and-strict-namespace-syntax`
* **Status:** COMPLETED
* **Lead Architect:** Core Systems & Documentation Architecture Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0010](../../internals/guidelines/authoring-docs.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Clear, precise, and professional documentation is essential for developer adoption. This milestone modernizes the complete documentation tree across `README.md`, `docs/guide/`, `docs/wiki/`, and `docs/spec/`:

1. **README Professionalization**:
   - Eliminate all emojis from `README.md` to establish a clean, production-grade presentation.
   - Expand the introduction using accessible, plain language to explain what Solix is, how deterministic Automatic Reference Counting (ARC) works without garbage collection pauses, how bytecode execution functions on the 64-bit register VM, and how the unified `solix` CLI toolchain simplifies development without external build systems.
2. **Strict Namespace Syntax (`::`) Alignment**:
   - Enforce the language invariant that namespaces and static symbols strictly use double colons `::` (e.g. `std::io::Console`, `solix::collections::List`, `Raylib::InitWindow`), while dot `.` is exclusively reserved for instance fields and methods.
   - Audit and rewrite all legacy documentation examples where `.` was mistakenly used for namespace member access.
3. **Wiki Primitives Restructuring**:
   - Retire `string` from the primitive types documentation (`string` in Solix is a reference-counted object type, not a scalar primitive).
   - Consolidate value primitives into dedicated, self-contained documentation pages with explicit type ranges, literal formats, and memory sizes.
4. **Comprehensive Guide Interconnection**:
   - Every guide in `docs/guide/` must include explicit "Next Steps" pointing to the subsequent guide and "Further Reading" links directing developers to in-depth feature specifications and wiki references.
   - Add inline links and keyword explanations for advanced keywords (e.g. `instanceof` in generic variadic code linking directly to `docs/wiki/keywords/expressions/instanceof.md`).
5. **Standardized Alert Callouts**:
   - Convert all informal note and warning blocks across the documentation into standardized GitHub-style alert callouts (`> [!NOTE]`, `> [!WARNING]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!CAUTION]`).

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS047-T01` | `docs/readme` | Rewrite `README.md`: remove emojis, expand introduction with plain language, document ARC, VM, and CLI toolchain. | Developer onboarding baseline |
| **P0** | `MS047-T02` | `docs/syntax` | Audit all documentation examples and replace namespace dot access with strict `::` scope resolution. | Syntactic correctness parity |
| **P1** | `MS047-T03` | `docs/wiki` | Restructure `docs/wiki/primitives/`: retire string from primitives, organize value primitive references. | Type system documentation accuracy |
| **P1** | `MS047-T04` | `docs/guide` | Add "Next Steps" and "Further Reading" sections to all guides; link unexplained keywords to wiki pages. | Guide navigation & readability |
| **P2** | `MS047-T05` | `docs/alerts` | Convert informal notes/warnings throughout `docs/` into GitHub-style alert callouts (`> [!NOTE]`, `> [!WARNING]`). | Visual consistency |
| **P2** | `MS047-T06` | `scripts` | Run and verify `scripts/verify_docs.py` and `scripts/doc_sync_check.py` with zero link breaks or GL-0011 defects. | Documentation CI gate |

---

## 3. Definition of Done (DoD)

1. `README.md` contains zero emojis and accessible introduction text explaining ARC, the 64-bit VM, and CLI tools.
2. Every code example in `docs/` accesses namespaces using `::` and instance members using `.`.
3. `string` is documented as an object type; value primitives have dedicated wiki pages.
4. All guides in `docs/guide/` feature sequential "Next Steps" and contextual "Further Reading" links.
5. GitHub alert callouts (`> [!NOTE]`, `> [!WARNING]`, etc.) are consistently formatted across all markdown documents.
6. `python3 scripts/verify_docs.py` and `python3 scripts/doc_sync_check.py` pass with 0 errors.
