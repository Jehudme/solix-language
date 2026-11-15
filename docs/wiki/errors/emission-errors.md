# Emission Error Codes (`E0300` – `E0350`)

This section documents bytecode emission diagnostics emitted by [`crates/solix-compilation/src/phases/emitter/`](../../../crates/solix-compilation/src/phases/emitter).

---

### `E0301`: Register Allocation Overflow
* **Cause:** A single function requires more than 65,535 simultaneous registers, exceeding the 16-bit register window index.
* **Fix:** Refactor deep expressions or split complex monolithic routines into smaller helper functions.

---

### `E0302`: Constant Pool Limit Exceeded
* **Cause:** The number of unique numeric and string constants in a single binary container exceeds the 32-bit constant pool capacity ($2^{32}-1$).

---

### `E0303`: Branch Target Out of Range
* **Cause:** A relative branch offset exceeds the 32-bit signed instruction jump limit.

---

### `E0304`: Corrupt Binary Container
* **Cause:** Deserialization of a compiled `.slxb` container failed due to malformed section boundaries, invalid magic bytes, or an unsupported container version.
