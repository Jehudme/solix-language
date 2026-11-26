# Solix Diagnostic Error Code Encyclopedia

This encyclopedia catalogs every diagnostic error code emitted by the Solix compiler (`E0000`–`E0405`, `E9001`), runtime traps (`R0000`–`R0027`), and bundled toolchain/stdlib diagnostics.

Per [`GL-0012`](../../guidelines/authoring-features-and-wiki.md), entries specify the invariant violated, cause, remediation hint, and examples synchronized directly with [`crates/solix-common/src/diagnostics/`](../../../crates/solix-common/src/diagnostics/), [`crates/solix-compilation/src/diagnostics/`](../../../crates/solix-compilation/src/diagnostics/), and [`crates/solix-runtime/src/exceptions/errors.rs`](../../../crates/solix-runtime/src/exceptions/errors.rs).

---

## Diagnostic Categories

| Code Range | Subsystem | Description & Specification Link |
| :--- | :--- | :--- |
| [**`E0000`, `E0011` – `E0016`**](lexer-errors.md) | **Lexical Analyzer** | Unterminated literals, invalid number prefixes, unexpected characters. |
| [**`E0101` – `E0200`**](parser-errors.md) | **Recursive-Descent Parser** | Grammar syntax errors, mismatched delimiters, misplaced statements. |
| [**`E0201` – `E0299`, `E0315` – `E0323`**](binder-errors.md) | **Semantic Type Binder** | Type mismatches, undefined symbols, static `::` vs `.`, VTable/exception/switch checks. |
| [**`E0301` – `E0314`, `E0330` – `E0341`, `E0401` – `E0405`, `E9001`**](emission-errors.md) | **Bytecode Emitter, Manifest & Toolchain** | Register/pool limits, binary container corruption, manifest schema errors, stdlib/CLI errors, custom plugins. |
| [**`R0000` – `R0027`**](runtime-errors.md) | **Virtual Machine Runtime** | Runtime traps, null dereferences, invalid casts, process exit codes. |
