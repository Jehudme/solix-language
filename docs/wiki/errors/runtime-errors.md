# Runtime Execution Errors (`R0001` – `R0027`)

This section documents runtime trap codes and execution faults emitted by the Solix Virtual Machine ([`crates/solix-runtime/src/exceptions/errors.rs`](../../../crates/solix-runtime/src/exceptions/errors.rs)).

---

| Trap Code | Numeric Exit | Error Identifier | Description & Trigger Cause |
| :---: | :---: | :--- | :--- |
| **`R0001`** | 1 | `OutOfMemory` | `VirtualHeap` exhausted available memory during dynamic allocation |
| **`R0002`** | 2 | `StackOverflow` | Call frame depth exceeded maximum recursion ceiling (default: 10,000 frames) |
| **`R0003`** | 3 | `DivideByZero` | Integer division or modulo by zero (`numerator / 0` or `numerator % 0`) |
| **`R0004`** | 4 | `IndexOutOfBounds` | Array, string, or collection indexed with negative or out-of-range index |
| **`R0005`** | 5 | `NullReferenceAccess`| Attempted field read, write, method invocation on null, or throw of a null reference |
| **`R0006`** | 6 | `InvalidCast` | Object reference downcast failed dynamic type check at runtime |
| **`R0007`** | 7 | `FunctionNotFound` | Dynamic linker failed to resolve target function index |
| **`R0008`** | 8 | `EntryPointNotFound` | Compiled `.slxb` container does not define a valid entry point (`main()`) |
| **`R0009`** | 9 | `InvalidInstruction`| Executed 64-bit instruction word failed format decoding |
| **`R0010`** | 10 | `InvalidOpCode` | VM instruction dispatcher encountered unmapped opcode byte |
| **`R0011`** | 11 | `InvalidFieldOffset`| Field access byte offset exceeds physical object allocation boundary |
| **`R0012`** | 12 | `InvalidStaticOffset`| Static slot index exceeds allocated global static memory space |
| **`R0013`** | 13 | `BinaryLoadError` | Malformed `.slxb` header, magic mismatch, or version incompatibility |
| **`R0014`** | 14 | `UserException` | Unhandled user exception unwound through `main()` without a matching catch |
| **`R0015`** | 15 | `NativeFunctionError`| C-FFI host plugin returned an error or crashed |
| **`R0016`** | 16 | `SecurityViolation` | Sandboxed execution violated security or filesystem policy |
| **`R0017`** | 17 | `TypeMismatch` | Register value bit-tag incompatible with instruction type requirement |
| **`R0018`** | 18 | `AssertionFailed` | Runtime `assert(false)` statement failed |
| **`R0019`** | 19 | `MemoryLeak` | Unfreed allocations detected on shutdown (`verify_clean_memory`) |
| **`R0020`** | 20 | `InvalidEntryPointSignature` | `main` function does not match `int32 main()` or `void main()` |
| **`R0021`** | 21 | `ExplicitExit` | Process halted via explicit `exit(code)` invocation |
| **`R0022`** | 22 | `SharedLibraryError`| `dlopen` or `LoadLibrary` failed to link native extension in `natives/` |
| **`R0023`** | 23 | `DependencyError` | Runtime dependency resolution or pinned SHA-256 verification failure |
| **`R0024`** | 24 | `DuplicateSymbolDefinition` | Duplicate function, class, or static variable symbol collision across loaded modules |
| **`R0025`** | 25 | `NativeLibraryChecksumMismatch` | SHA-256 verification of native shared library failed |
| **`R0026`** | 26 | `InvalidOperand` | Invalid operand for operation (e.g., negative bitwise shift count) |
| **`R0027`** | 27 | `ArrayStoreMismatch` | Attempted to store an incompatible object instance into a covariant reference array |
