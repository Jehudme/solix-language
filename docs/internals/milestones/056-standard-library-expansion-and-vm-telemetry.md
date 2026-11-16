# Milestone 056: Standard Library Expansion, C++ std::format & VM Telemetry

## Document Metadata
* **Milestone ID:** MS-056
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/056-standard-library-expansion-and-vm-telemetry`
* **Status:** DRAFT
* **Lead Architect:** Standard Library & Runtime Architecture Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 056 delivers high-impact standard library expansions (`solixlib`), adds advanced C++ `std::format`-style formatting, creates a deep 56-metric VM telemetry and introspection sublibrary (`solix.system.Vm`), and establishes `mount://` virtual filesystem redirection for `.slxpac` archive containers.

Key objectives:
1. **Advanced C++ `std::format`-Style Formatting Engine**:
   - Enhance `String::format` and `Console::printf` with full specifier grammar: `{[index][:[fill]align][sign][#][0][width][.precision][type]}`.
   - Support hex (`{:x}`, `{:X}`), binary (`{:b}`), octal (`{:o}`), float precision (`{:.2f}`, `{:e}`), zero-padding (`{:08x}`), and alignment (`{:<10}`, `{:>10}`, `{:^10}`).
2. **Standard Library Additions**:
   - `solix.json`: 1-to-1 parity with `serde_json` via native bridge (`JsonValue`, `JsonObject`, `JsonArray`, pointer queries, pretty printing).
   - `solix.net`: Cross-platform networking via `std::net` (`TcpListener`, `TcpStream`, `UdpSocket`, `HttpClient`).
   - `solix.compress`: Buffer and streaming compression via `flate2` (`Gzip`, `Zlib`, `Deflate`, `CompressedStream`).
3. **VM Telemetry & Introspection Sublibrary (`solix.system.Vm`)**:
   - Expose 56 comprehensive runtime metrics across Heap Memory, ARC/Cycle Collector, Call Stack, Functions/Metadata, Container/Profile Context, and OS Process stats.
4. **Filesystem `mount://<path>` Redirection**:
   - Intercept I/O on `mount://<path>` paths with static first-use caching.
   - Redirect to `resources/<path>` inside `.slxpac` ZIP containers (read-only; writes throw `E0330`).
   - Redirect to `./resources/<path>` when running loose source/debug builds.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS056-T01` | `solixlib/core` | Implement C++ `std::format`-style format parser and radix conversions in `String.slx` and `solixlib_native`. | Text formatting |
| **P0** | `MS056-T02` | `solixlib/json` | Implement `solix.json` 1-to-1 `serde_json` bridge (`JsonValue`, `JsonObject`, `JsonArray`). | Data serialization |
| **P1** | `MS056-T03` | `solixlib/net` | Implement `solix.net` TCP/UDP sockets and `HttpClient`. | Networking |
| **P1** | `MS056-T04` | `solixlib/compress` | Implement `solix.compress` Gzip, Zlib, and Deflate wrappers. | Compression |
| **P0** | `MS056-T05` | `solixlib/vm` | Implement `solix.system.Vm` and `VmMetrics` exposing 56 native runtime metrics. | Runtime introspection |
| **P0** | `MS056-T06` | `solixlib/io` | Implement `mount://` filesystem redirection with static cached container state. | Resource virtualization |
| **P2** | `MS056-T07` | `docs` | Document all standard library modules and add tutorials. | Documentation sync |

---

## 3. New Diagnostics & Error Codes

| Error Code | Subsystem | Name | Description & Template |
|:---|:---|:---|:---|
| **`E0330`** | Standard Library (`solix.io`) | `MountPathReadOnly` | `Virtual path '{path}' under 'mount://' is read-only inside an .slxpac archive container.` |
| **`E0331`** | Standard Library (`solix.core`) | `InvalidFormatString` | `Invalid format specifier in template string: {details}` |
| **`E0332`** | Standard Library (`solix.json`) | `JsonParseError` | `Failed to parse JSON string at line {line}, column {col}: {reason}` |
| **`E0333`** | Standard Library (`solix.net`) | `NetworkSocketError` | `Network operation failed on socket: {reason}` |
| **`E0334`** | Standard Library (`solix.compress`) | `DecompressionError` | `Failed to decompress buffer using algorithm '{algorithm}': {reason}` |

---

## 4. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_format_hex_binary_octal`: `String::format("{:#08x} {:b}", 255, 10)` returns `"0x0000ff 1010"`.
2. `test_format_alignment_and_padding`: Right, left, and center alignment with custom fill characters.
3. `test_json_parse_and_stringify`: Round-trip parsing and pretty-printing of complex JSON objects.
4. `test_json_pointer_navigation`: Fetching nested values via JSON pointer `/store/book/0/title`.
5. `test_net_tcp_echo`: Client connecting to local `TcpListener` sends and receives echo payload.
6. `test_compress_gzip_roundtrip`: Compressing and decompressing string payload matches original byte-for-byte.
7. `test_vm_metrics_56_fields`: Querying `Vm::metrics()` returns non-null values for all 56 metrics.
8. `test_mount_read_resource_loose`: `File::read_all_text("mount://data.txt")` correctly reads from local `./resources/data.txt`.

### B. Negative Rejection Tests
1. `test_mount_write_in_archive_throws`: Attempting `File::write_all_text("mount://data.txt", ...)` inside `.slxpac` throws `E0330`.
2. `test_format_malformed_specifier_throws`: Invalid specifier throws `E0331`.
3. `test_json_syntax_error_throws`: Malformed JSON string throws `E0332` with line and column information.

---

## 5. Definition of Done (DoD)
1. `solixlib` compiles with zero warnings; native bridge tests pass 100%.
2. All 56 metrics verified across Linux, macOS, and Windows.
3. Full documentation updated in `docs/stdlib/`.
