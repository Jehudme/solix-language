# Emission, Manifest & Toolchain Error Codes (`E0301` – `E0314`, `E0330` – `E0341`, `E0401` – `E0405`, `E9001`)

This section documents bytecode emission diagnostics ([`EmissionError`](../../../crates/solix-common/src/diagnostics/emission.rs)), manifest validation diagnostics ([`ManifestError`](../../../crates/solix-common/src/diagnostics/manifest.rs)), standard library/CLI error codes, and custom pipeline diagnostics ([`CompilationDiagnostic::Custom`](../../../crates/solix-compilation/src/diagnostics/compilation.rs)).

---

## Bytecode Emission & Binary Container (`EmissionError`)

---

### `E0301`: Constant Pool Overflow
* **Diagnostic Message:** `Constant pool limit exceeded: <count> entries exceeds u16 maximum (65,535)`
* **Variant:** `EmissionError::ConstantPoolOverflow`
* **Resolution:** Reduce the number of unique string/float/int64 constants in a single compilation unit or split large data tables into external resource files (`res://`). Note: `EmissionError::Manifest` delegates directly to `ManifestError::error_code()` (`E0401`–`E0405`).

---

### `E0302`: Register Overflow
* **Diagnostic Message:** `Register limit exceeded: register index <register_index> exceeds maximum register ceiling of <max>`
* **Variant:** `EmissionError::RegisterOverflow`
* **Resolution:** Refactor complex expressions or deep local variable scopes into smaller functions.

---

### `E0303`: Argument Count Overflow
* **Diagnostic Message:** `Argument count limit exceeded: function call has <argument_count> arguments (maximum allowed is <max>)`
* **Variant:** `EmissionError::ArgumentCountOverflow`
* **Resolution:** Reduce the parameter list or pass complex parameters inside an object or struct.

---

### `E0304`: Jump Offset Overflow
* **Diagnostic Message:** `Jump offset out of bounds: offset <offset> exceeds signed 32-bit branch range`
* **Variant:** `EmissionError::JumpOffsetOverflow`
* **Resolution:** Split very large function bodies into smaller helper routines.

---

### `E0305`: Invalid Magic
* **Diagnostic Message:** `Invalid binary magic header: expected 'SLXB', found <value>`
* **Variant:** `EmissionError::InvalidMagic`
* **Resolution:** Ensure the input file is a valid Solix compiled binary (*.slxb).

---

### `E0306`: Unsupported Version
* **Diagnostic Message:** `Unsupported binary container version: expected version <expected_major>.<expected_minor>, found <found_major>.<found_minor>`
* **Variant:** `EmissionError::UnsupportedVersion`
* **Resolution:** Recompile the source files using the current compiler version.

---

### `E0307`: Unexpected Eof
* **Diagnostic Message:** `Unexpected end of binary stream while decoding section '<section>': required <required> bytes, available <available>`
* **Variant:** `EmissionError::UnexpectedEof`
* **Resolution:** The compiled binary file appears truncated or corrupted.

---

### `E0308`: Invalid Opcode
* **Diagnostic Message:** `Invalid opcode byte <raw_byte> at instruction offset <instruction_index>`
* **Variant:** `EmissionError::InvalidOpcode`
* **Resolution:** The binary file contains an invalid or unrecognized virtual machine opcode.

---

### `E0309`: Corrupt Binary Container
* **Diagnostic Message:** `Corrupt binary container in section '<section>': <details>`
* **Variant:** `EmissionError::CorruptBinaryContainer`
* **Resolution:** The binary file does not conform to the SLXB format specification.

---

### `E0310`: Missing Entry Point
* **Diagnostic Message:** `Program entry point '<entry_symbol>' could not be resolved`
* **Variant:** `EmissionError::MissingEntryPoint`
* **Resolution:** Ensure a valid entry point function is declared or configured via 'profiles.<profile>.entrypoint' or '--entry'.

---

### `E0311`: Duplicate Symbol
* **Diagnostic Message:** `Duplicate function symbol '<symbol>' detected in binary module`
* **Variant:** `EmissionError::DuplicateSymbol`
* **Resolution:** Rename one of the conflicting function symbols.

---

### `E0312`: Symbol Not Found
* **Diagnostic Message:** `Symbol '<symbol>' not found in binary unit symbol table`
* **Variant:** `EmissionError::SymbolNotFound`
* **Resolution:** Verify that the referenced function or static variable is exported.

---

### `E0313`: Malformed Export Manifest
* **Diagnostic Message:** `Malformed export manifest in binary container: <reason>`
* **Variant:** `EmissionError::MalformedExportManifest`
* **Resolution:** Ensure the binary container has a valid serialized export manifest.

---

### `E0314`: Container Serialization Failed
* **Diagnostic Message:** `Project archive or container error: <reason>`
* **Variant:** `EmissionError::ContainerSerializationFailed`
* **Resolution:** Check project structure, manifest.yaml, or archive container integrity.

---

## Standard Library & Toolchain CLI (`E0330` – `E0341`)

---

### `E0330`: Read-Only Container Virtual Path or I/O Failure
* **Cause:** Attempted to write to a read-only `mount://` virtual archive path inside a `.slxp` container, or failed filesystem I/O in `solix.io.File`.
* **Resolution:** Write mutable files to a host filesystem path instead of `mount://` archive resources.

---

### `E0331`: Invalid UTF-8 Encoding in Native String Bridge
* **Cause:** A native string buffer or guest byte slice contained malformed UTF-8 sequences.
* **Resolution:** Ensure external buffers passed to string decoding routines contain valid UTF-8 data.

---

### `E0332`: JSON Parsing or Pointer Evaluation Error
* **Cause:** `JsonValue::parse` or JSON Pointer navigation encountered invalid JSON syntax or a missing pointer segment.
* **Resolution:** Validate input JSON syntax and verify RFC 6901 pointer paths before indexing.

---

### `E0333`: Network Socket or Stream Failure
* **Cause:** `TcpListener`, `TcpStream`, or TLS socket operations failed to bind, connect, read, or write.
* **Resolution:** Verify host address/port availability and catch network exceptions around socket operations.

---

### `E0334`: Compression Codec Failure
* **Cause:** Deflate/Gzip/Zlib compression or decompression failed due to corrupt input data.
* **Resolution:** Verify the compressed byte stream matches the selected compression format.

---

### `E0341`: Invalid Cycle Collector Threshold Configuration
* **Cause:** `--cycle-threshold` was passed as `0` on `solix run`, which would trigger cycle collection on every object allocation.
* **Resolution:** Provide a positive integer `--cycle-threshold` greater than zero (default: `1024`).

---

## Project Manifest Validation (`ManifestError`)

---

### `E0401`: Parse Failed
* **Diagnostic Message:** `Failed to parse <filename>: <message>\n<snippet>`
* **Variant:** `ManifestError::ParseFailed`
* **Resolution:** Ensure the manifest follows the valid Solix manifest.yaml schema.

---

### `E0402`: Deprecated Inherits
* **Diagnostic Message:** `Failed to parse <filename>: profile '<profile_name>' uses 'inherits', which has been removed; configure each profile explicitly or use YAML anchors (& / *)\n<snippet>`
* **Variant:** `ManifestError::DeprecatedInherits`
* **Resolution:** Configure each profile explicitly or use YAML anchors (& / *).

---

### `E0403`: Missing Release Profile
* **Diagnostic Message:** `Failed to parse <filename>: manifest must define mandatory profiles.release profile\n<snippet>`
* **Variant:** `ManifestError::MissingReleaseProfile`
* **Resolution:** Define at least a 'release' profile under 'profiles' in manifest.yaml.

---

### `E0404`: Unknown Root Field
* **Diagnostic Message:** `Failed to parse <filename>: unknown field `<field_name>` at root level, expected one of <expected>\n<snippet>`
* **Variant:** `ManifestError::UnknownRootField`
* **Resolution:** Remove unrecognized keys or prefix custom vendor keys with 'x-'.

---

### `E0405`: Io Error
* **Diagnostic Message:** `Failed to read manifest file '<path>': <reason>`
* **Variant:** `ManifestError::IoError`
* **Resolution:** Check that the manifest file exists and has read permissions.

---

### `E9001`: Custom Compiler Plugin Diagnostic
* **Cause:** Emitted by user or pipeline extension phases invoking `CompilationDiagnostic::custom("E9001", ...)`.
* **Resolution:** Inspect the custom diagnostic message and accompanying source span.
