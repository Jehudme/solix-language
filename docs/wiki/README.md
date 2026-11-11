# Solix Atomic Reference Wiki

The Solix Wiki serves as a fast, atomic dictionary and cheat sheet for software engineers, IDE tooling, and compiler developers.

Per [`GL-0012`](file:///home/jehud/Projects/solix-language/docs/internals/guidelines/authoring-features-and-wiki.md), articles in the wiki are strictly modular, each providing direct root-cause explanations and minimal reproducible examples without deep architectural expositions.

---

## Wiki Reference Categories

### 1. [Diagnostic Error Code Encyclopedia (`errors/`)](file:///home/jehud/Projects/solix-language/docs/wiki/errors/README.md)
Exhaustive reference for every diagnostic code emitted by the compiler and virtual machine:
* [**Lexer Errors (`E0000`–`E0015`)**](file:///home/jehud/Projects/solix-language/docs/wiki/errors/lexer-errors.md): Unterminated string literals, invalid escape sequences, unrecognized tokens.
* [**Parser Errors (`E0101`–`E0200`)**](file:///home/jehud/Projects/solix-language/docs/wiki/errors/parser-errors.md): Grammar violations, mismatched delimiters, missing semicolons, malformed signatures.
* [**Binder & Semantic Errors (`E0201`–`E0290`)**](file:///home/jehud/Projects/solix-language/docs/wiki/errors/binder-errors.md): Type mismatches, undefined symbols, invalid static access (`.` vs `::`), vtable conflicts, missing interface methods.
* [**Emission Errors (`E0300`–`E0350`)**](file:///home/jehud/Projects/solix-language/docs/wiki/errors/emission-errors.md): Register overflow, malformed container headers, constant pool exhaustion.
* [**Runtime Execution Errors (`R0001`–`R0023`)**](file:///home/jehud/Projects/solix-language/docs/wiki/errors/runtime-errors.md): Stack overflow, null reference dereference, division by zero, invalid cast traps, entrypoint signature mismatches.

### 2. [Built-in Primitive Types (`primitives/`)](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/README.md)
All 14 built-in primitive types, memory layouts, ranges, and conversion rules:
* [**Integer Types**](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/integers.md): Signed (`int8`, `int16`, `int32`, `int64`) and unsigned (`uint8`, `uint16`, `uint32`, `uint64`), plus friendly aliases (`byte`, `short`, `int`, `long`).
* [**Floating-Point Types**](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/floats.md): IEEE-754 single (`float32` / `float`) and double (`float64` / `double`).
* [**Booleans & Characters**](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/booleans-and-chars.md): `bool` and 32-bit Unicode scalar `char`.
* [**Strings, Tuples & Void**](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/strings-tuples-and-void.md): Heap-allocated primitive `string`, structural anonymous `tuple`, and `void`.

### 3. [CLI Commands & Tooling (`commands/`)](file:///home/jehud/Projects/solix-language/docs/wiki/commands/README.md)
Comprehensive flag descriptions, usage syntax, and output options:
* [**`compile` & `build`**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/compile-and-build.md): Compiling standalone files and orchestrating multi-language projects into `.slxb` and `.slxarc`.
* [**`run` & `info`**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/run-and-info.md): Executing bytecode containers and inspecting binary sections, disassemblies, and metadata.
* [**Package Management Commands**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/package-commands.md): `solix new`, `solix install`, `solix uninstall`, and `solix list`.

### 4. [Language Keywords Dictionary (`keywords/`)](file:///home/jehud/Projects/solix-language/docs/wiki/keywords/README.md)
Alphabetical reference for all 45+ reserved language keywords across control flow, declarations, modifiers, and expressions.
