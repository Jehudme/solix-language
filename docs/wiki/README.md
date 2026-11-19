# Solix Atomic Reference Wiki

The Solix Wiki serves as a fast, atomic dictionary and cheat sheet for software engineers, IDE tooling, and compiler developers.

Per [`GL-0012`](../guidelines/authoring-features-and-wiki.md), articles in the wiki are strictly modular, each providing direct root-cause explanations and minimal reproducible examples without deep architectural expositions.

---

## Wiki Reference Categories

### 1. [Diagnostic Error Code Encyclopedia (`errors/`)](errors/README.md)
Exhaustive reference for every diagnostic code emitted by the compiler and virtual machine:
* [**Lexer Errors (`E0000`–`E0015`)**](errors/lexer-errors.md): Unterminated string literals, invalid escape sequences, unrecognized tokens.
* [**Parser Errors (`E0101`–`E0200`)**](errors/parser-errors.md): Grammar violations, mismatched delimiters, missing semicolons, malformed signatures.
* [**Binder & Semantic Errors (`E0201`–`E0290`)**](errors/binder-errors.md): Type mismatches, undefined symbols, invalid static access (`.` vs `::`), vtable conflicts, missing interface methods.
* [**Emission Errors (`E0300`–`E0350`)**](errors/emission-errors.md): Register overflow, malformed container headers, constant pool exhaustion.
* [**Runtime Execution Errors (`R0001`–`R0023`)**](errors/runtime-errors.md): Stack overflow, null reference dereference, division by zero, invalid cast traps, entrypoint signature mismatches.

### 2. [Built-in Primitive Types (`primitives/`)](primitives/README.md)
All built-in primitive scalar and structural types, memory layouts, ranges, and conversion rules:
* [**Integer Types**](primitives/integers.md): Signed (`int8`, `int16`, `int32`, `int64`) and unsigned (`uint8`, `uint16`, `uint32`, `uint64`), plus friendly aliases (`byte`, `short`, `int`, `long`).
* [**Floating-Point Types**](primitives/floats.md): IEEE-754 single (`float32` / `float`) and double (`float64` / `double`).
* [**Booleans**](primitives/booleans.md): `bool` logical truth values.
* [**Characters**](primitives/chars.md): 32-bit Unicode scalar `char`.
* [**Tuples**](primitives/tuples.md): Structural anonymous composite values.
* [**Void**](primitives/void.md): Absence of return values.

### 3. [CLI Commands & Tooling (`commands/`)](commands/README.md)
Comprehensive flag descriptions, usage syntax, and output options:
* [**`compile` & `build`**](commands/compile-and-build.md): Compiling standalone files and orchestrating multi-language `manifest.yaml` projects into `.slxb` and `.slxarc`.
* [**`run` & `info`**](commands/run-and-info.md): Executing bytecode containers and inspecting binary sections, disassemblies, and metadata.
* [**Package & Scaffolding Commands**](commands/package-commands.md): `solix new`, `solix new native`, `solix install`, `solix uninstall`, and `solix list`.

### 4. [Language Keywords Dictionary (`keywords/`)](keywords/README.md)
Alphabetical reference for all 45+ reserved language keywords across control flow, declarations, modifiers, and expressions.

### 5. [Lexical Cheat Sheet](lexical.md)
Tokenization, numeric literals, escape sequences, operators, and comment rules.

### 6. [`manifest.yaml` Project Manifest Reference](manifest.md)
Declarative `manifest.yaml` schema, reusable `vars:` YAML anchors/merge keys, zero-inheritance profiles, OS-to-all-architectures `platforms` expansion, `pre_compile`/`post_compile` hooks, and auto-detected `natives`.

### 7. [Foreign Function Interface (FFI) SDKs (`ffi/`)](ffi/README.md)
Complete C-ABI, struct, macro, and dual-mode native library wiki:
* [**FFI Overview & ABI Types**](ffi/README.md): `SolixCValue`, type tags, array element tags, `SolixVmContext` magic, and dual-mode compilation flags.
* [**C & C++ SDK Reference (`ffi/c`, `solix::ffi`)**](ffi/c-cpp-sdk.md): `solix_ffi.h`, `solix_cvalue_*`, `solix_vm_*`, `solix_vm_set_error`, `SOLIX_REGISTER_MODULE`, and `SOLIX_EXPORT_NATIVES_*`.
* [**Rust SDK Reference (`ffi/rust`, `solix-ffi`)**](ffi/rust-sdk.md): `SolixCValue`, `SolixVmContext`, `NativeRegistry`, and `solix_export!`.
