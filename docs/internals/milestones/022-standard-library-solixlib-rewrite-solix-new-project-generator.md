# Milestone 022: Standard Library (`solixlib` v2.0) Complete Modernization, Rust Native FFI & `solix new` Client Project Generator

## Document Metadata

* **Milestone ID:** MS-022
* **Target Version:** `v0.1.0-alpha.22` (`solixlib` `v2.0.0`)
* **Milestone Branch:** `milestone/022-standard-library-and-project-generator`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/019-project-build-system-slxarc-and-native-loader.md` (MS-019)
  * `docs/internals/milestones/020-package-manager-solver-and-dependency-resolution.md` (MS-020)
  * `docs/internals/milestones/021-comprehensive-codebase-remediation-and-conformance.md` (MS-021)

---

## 1. Executive Summary & Architectural Vision

The legacy `solixlib` (`v1.2.1`) was written for the old C++ Solix engine (`manifest.json`, `.slxbin`, and C++ internal headers `#include "solix/runtime.hpp"`). It cannot compile or link against the modern Rust Solix toolchain (`manifest.toml`, `.slxarc`, and Rust `cdylib` native extensions), and its Solix code predates many of the best features now supported by the Solix language.

**Milestone 022** delivers two foundational pillars for the Solix ecosystem:

1. **Pillar 1: Client Project Generator Script & `solix new <path>` Subcommand:**
   * Just like `solix build` delegates to Python build scripts, `solix new <path>` invokes a dedicated client Python generator script (`scripts/init_project.py` / `solix_toolbox.py`) that **programmatically generates** a complete, ready-to-build Solix project (`manifest.toml`, `scripts/build.py`, `scripts/solix_toolbox.py`, `src/`, `tests/`, `resources/`, `lib/`, and optional Rust `cdylib` `natives/<name>_native/`) configured with the exact CLI arguments (`--name`, `--version`, `--description`, `--license`, `--author`, `--main`, `--test`, `--no-native`, `--lib`, `--deps`).
   * **Rust-Only Native Libraries:** Native subprojects in `natives/` are strictly **Rust `cdylib` crates** (`Cargo.toml` + `src/lib.rs`, matching `templates/untitled-rust`). CMake/C++ native project support is not used.
2. **Pillar 2: Complete Rewrite & Modernization of `solixlib` (`v2.0.0`) + Exhaustive Solix Test Suite in `solixlib/tests/`:**
   * Scaffolds `solixlib` as a standard Solix project using the new project generator (`name = "solix"`, `version = "2.0.0"`).
   * Replaces all legacy C++ `native/src/*.cpp` files with a single cross-platform **Rust `cdylib` (`natives/solixlib_native/`)** using the existing `SolixVmContext` and `SolixCValue` bridge in `crates/solix-runtime/src/native/ffi.rs`.
   * Rewrites every module (`solix.core`, `solix.collections`, `solix.io`, `solix.math`, `solix.time`, `solix.system`, `solix.crypto`) using modern Solix features:
     * Parameterized interfaces (`IComparable<T>`, `IEquatable<T>`, `ICloneable<T>`, `IHashable`).
     * Variadic generics (`<args...>`) & `tuple` (`Console::printf<args...>`, `String::format<args...>`, `List::of<T, args...>`, `Math::min_all<T, args...>`, `Path::join<args...>`).
     * Operator overloading (`operator[]`, `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&`, `|`, `^`, `~`).
     * Deterministic RAII destructors (`~FileStream()`, `~BufferedReader()`, `~StreamReader()`, `~StreamWriter()`, `~Process()`).
     * First-class `enum` types (`enum FileMode`, `enum SeekOrigin`, `enum ConsoleColor`).
     * Reorganized packages (`Console` moved into `solix.io`, `Exceptions` moved into `solix.core`).
   * **100% Function-by-Function Verification in `solixlib/tests/`:**
     * All unit and integration tests for `solixlib` are written **in Solix** inside `solixlib/tests/` (`test_core.slx`, `test_collections.slx`, `test_io.slx`, `test_math.slx`, `test_time.slx`, `test_system.slx`, `test_crypto.slx`, and `test_main.slx`), executed via `solix build solixlib --test` and `solix run --test solixlib/build/test/solix.slxarc` to verify that **every single function, method, and operator** works as intended with zero memory leaks!

---

## 2. Pillar 1: Client Project Generator Script (`scripts/init_project.py` & `solix new`)

### 2.1 How the Generator Script Works

Instead of copying a static template folder and doing fragile regex replacements afterward, the client provides a Python generator script (`scripts/init_project.py`, also exposed in `solix_toolbox.py` and invoked by `solix new <path>`) that **programmatically writes** each file in the new project using the user's exact parameters:

```
┌────────────────────────────────────────────────────────────────────────────┐
│                   `solix new <path> [options]` Workflow                    │
├────────────────────────────────────────────────────────────────────────────┤
│  User runs:                                                                │
│    solix new ./my_app --name my_app --version 0.2.0 --deps solix@2.0.0     │
│         │                                                                  │
│         ▼                                                                  │
│  `solix-client` (`crates/solix-client/src/commands/new.rs`) invokes        │
│  `python3 scripts/init_project.py <path> [flags...]`                       │
│         │                                                                  │
│         ▼                                                                  │
│  `init_project.py` programmatically generates:                             │
│    ├── .gitignore                                                          │
│    ├── manifest.toml          ([info], [entrypoints], [dependencies])      │
│    ├── scripts/                                                            │
│    │   ├── build.py           (Calls tb.auto_build(profile))               │
│    │   └── solix_toolbox.py   (Rust Cargo cdylib + Solix compile + slxarc) │
│    ├── src/                                                                │
│    │   └── main.slx           (Matches [entrypoints].main or --lib mode)   │
│    ├── tests/                                                              │
│    │   └── test_main.slx      (Matches [entrypoints].test)                 │
│    ├── natives/               (Rust cdylib crate if native enabled)        │
│    │   └── <name>_bridge/                                                  │
│    │       ├── Cargo.toml                                                  │
│    │       └── src/lib.rs                                                  │
│    ├── resources/.gitkeep                                                  │
│    └── lib/.gitkeep                                                        │
└────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 CLI Arguments for `scripts/init_project.py` & `solix new`

| Argument / Flag | Default | Description |
| :--- | :--- | :--- |
| `<path>` | *(Required)* | Target directory path where the project will be generated. |
| `--name <NAME>` | Directory name of `<path>` | Project/package name in `[info].name`. |
| `--version <VER>` | `"0.1.0"` | Initial SemVer version (`Major.Medium.Minor`) in `[info].version`. |
| `--description <TEXT>` | `"A Solix project"` | Package description in `[info].description`. |
| `--license <LIC>` | `"Apache-2.0"` | License identifier in `[info].license`. |
| `--author <NAME>` | `[]` (Repeatable) | Author(s) in `[info].authors`. |
| `--main <SYMBOL>` | `"Main::main"` | Main entrypoint symbol in `[entrypoints].main`. |
| `--test <SYMBOL>` | `"TestRunner::test"` | Test entrypoint symbol in `[entrypoints].test`. |
| `--lib` | `false` | Generate a library project (`src/lib.slx`, `package <name>;`) instead of an executable app. |
| `--no-native` | `false` | Do not generate a Rust crate inside `natives/` (creates an empty `natives/.gitkeep`). |
| `-d, --deps <PKG@VER>` | `[]` (Repeatable) | Adds `<pkg> = "<ver>"` entries to `[dependencies]` in `manifest.toml`. |
| `--force` | `false` | Allow generating into a directory that already contains a `manifest.toml`. |

---

## 3. Extending `SolixVmContext` in `crates/solix-runtime/src/native/ffi.rs`

In your current codebase, `crates/solix-runtime/src/native/ffi.rs` already defines the FFI bridge used by Rust `cdylib` native libraries (`Suite 49` and `templates/untitled-rust`):

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolixCValue {
    /// Tag: 0 = Null, 1 = Bool, 2 = I32, 3 = I64, 4 = F64, 5 = Ref, 6 = FnPtr
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

#[repr(C)]
pub struct SolixVmContext {
    pub magic: u64,
    pub vm_ptr: *mut c_void,
    pub get_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    pub set_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32, i32),
}
```

Right now, `SolixVmContext` only has `get_field_i32` and `set_field_i32`. For `solixlib`'s Rust native crate (`natives/solixlib_native/`) to read and return strings (`string` and `char[]`), byte arrays (`int8[]`), `int64[]`, and `float64[]` without any linker issues across Linux, macOS, and Windows, we simply **append these function pointers to `SolixVmContext`** in `crates/solix-runtime/src/native/ffi.rs`:

```rust
#[repr(C)]
pub struct SolixVmContext {
    pub magic: u64,
    pub vm_ptr: *mut c_void,
    pub get_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    pub set_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32, i32),

    // --- New Host Callbacks Appended for Standard Library & Native Crates ---
    /// Reads a guest `string`, `char[]`, or `int8[]` at `addr` into `out_buf` (up to `cap` bytes).
    /// Returns the total byte/element length of the string or array.
    pub read_bytes: unsafe extern "C" fn(vm_ptr: *mut c_void, addr: u32, out_buf: *mut u8, cap: u32) -> u32,

    /// Allocates a new `BlockTag::String` on the guest heap from `(ptr, len)` and returns its `u32` address.
    pub alloc_string: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,

    /// Allocates a new `char[]` (`BlockTag::ArrayRef`, elem_size=8, type_tag=12) from `(ptr, len)`
    /// so each ASCII/UTF-8 byte is stored as a `char` element, returning its `u32` address.
    pub alloc_chars: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,

    /// Allocates a new `int8[]` (`BlockTag::ArrayRef`, elem_size=8, type_tag=1) from `(ptr, len)`.
    pub alloc_byte_array: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,

    /// Allocates a new `ArrayRef` of `len` elements with `type_tag` (e.g. 4 = int64, 10 = float64, 13 = string).
    pub alloc_array: unsafe extern "C" fn(vm_ptr: *mut c_void, len: u32, type_tag: u8) -> u32,

    /// Gets or sets a 64-bit element (int64, float64 bits, or ref u32) at `index` in a guest `ArrayRef`.
    pub array_len: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32) -> u32,
    pub array_get_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32) -> u64,
    pub array_set_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32, raw_val: u64),
}
```

### Why Extending `SolixVmContext` Is So Clean:
1. **100% Backward Compatible with Suite 49 & `untitled-rust`:** Existing tests that only read the first 4 fields of `SolixVmContext` (`magic`, `vm_ptr`, `get_field_i32`, `set_field_i32`) continue to work untouched!
2. **Zero External Linker Symbols Needed:** Because `invoke_c_native` passes `&mut context_wrapper` as the first argument (`vm_context: *mut c_void`) to every native function, the Rust `cdylib` (`solixlib_native`) can call `(ctx.alloc_chars)(...)`, `(ctx.read_bytes)(...)`, `(ctx.alloc_string)(...)`, and `(ctx.array_set_u64)(...)` directly through the `vm_context` pointer!
3. **Important Array Stride Match (`elem_size = 8`):** Notice in `crates/solix-compilation/src/phases/emitter/emitter.rs:416` that Solix arrays (`char[]`, `int8[]`, `int32[]`, `int64[]`, `float64[]`) are allocated with `elem_size = 8` and `type_tag` in `arr_addr + 7` (`1` for `int8`, `3` for `int32`, `4` for `int64`, `10` for `float64`, `12` for `char`, `13` for `string`). By handling this inside `alloc_chars`, `alloc_byte_array`, and `read_bytes` in `ffi.rs`, `solixlib_native` is guaranteed 100% layout compatibility with the VM!

---

## 4. Forensic Audit of Legacy `solixlib` v1.2.1 (All 15 Solix Bugs Fixed in v2.0.0)

Every bug found in the legacy `solixlib` v1.2.1 code is documented below and fixed in v2.0.0:

| ID | Legacy File | Bug in Legacy v1.2.1 | Fix in Modern `solixlib` v2.0.0 |
| :--- | :--- | :--- | :--- |
| **BUG-01** | `core/ICloneable.slx` | `IStringable clone();` — non-generic, returned `IStringable`. | `public interface ICloneable<T> { T clone(); }` |
| **BUG-02** | `core/IComparable.slx` & `IEquatable.slx` | Non-generic `compare_to(IComparable)` & `equals(IEquatable)`. `String`, `Duration`, `Instant`, `DateTime` couldn't implement them! | `public interface IComparable<T> { int32 compare_to(T other); }` and `public interface IEquatable<T> { bool equals(T other); }`. Implemented by `String`, `Duration`, `Instant`, `DateTime`, `Path`! |
| **BUG-03** | `collections/HashMap.slx:70` | When `new HashMap<K, V>()` is called without a custom hasher, `_hasher` is `null` and `hash(key)` **always returns `0`**, turning `HashMap` into an $O(N)$ linked list! Also `if (h < 0) h = 0 - h;` stays negative for `MIN_INT` (`-2147483648`). | Provide a built-in generic/native fallback hash (`Primitives::hash_any<K>(key)`), and mask with `h & 0x7FFFFFFF` so `MIN_INT` never stays negative! |
| **BUG-04** | `collections/Interfaces.slx:27` | `IList<T>` declares `void set(int32 index, T item)` while `List<T>` and `LinkedList<T>` return `T set(int32 index, T item)` (fails with `E0207`!). | Standardize `IList<T>::set` to `T set(int32 index, T item);`. |
| **BUG-05** | `collections/Algorithms.slx:26` | Uses $O(N^2)$ Insertion Sort for `sort_int32` and `sort<T>`. | Upgrade to **$O(N \log N)$ MergeSort / QuickSort**! |
| **BUG-06** | `collections/BitSet.slx:7` | Stores `bool[] _bits` (1 bool per array slot) and loops 1 bit at a time. | Pack 64 bits per word into `int64[] _words` using bitwise `&`, `|`, `^`, `~`, and `Primitives::bit_count64`! |
| **BUG-07** | `collections/Deque.slx` & `Queue.slx` | Backed by `LinkedList<T>`, and `DequeIterator::next()` calls `_items.get(_idx)` which walks from the head every step ($O(N^2)$ iteration!). | Back `Deque<T>` and `Queue<T>` with a contiguous power-of-two **ring buffer (`T[]`)** with $O(1)$ indexing and $O(1)$ iterator access! |
| **BUG-08** | `collections/TreeMap.slx:112` | Unbalanced BST ($O(N)$ on sorted keys) and broken parent pointer update when deleting a node with two children (`succ != node.right`). | Implement a balanced **Red-Black Tree** (or AVL tree) with correct two-child node deletion! |
| **BUG-09** | `core/String.slx:148` | `String.format(fmt, String[] args)` only supports `{0}`..`{9}` (`'0' + i` breaks for `i >= 10`) and requires manual `String[]` array creation. | Add variadic `String::format<args...>(String fmt, args...)` supporting sequential `{}` and multi-digit positional `{0}`, `{10}`, etc., while keeping `format_array(String fmt, String[] args)`! |
| **BUG-10** | `math/Random.slx:25` | `if (low < 0) return -low;` returns `-2147483648` (negative!) when `low == MIN_INT`, and leaks a native C++ mutex handle on every `new Random()`. | Implement `xoshiro256**` / `SplitMix64` state directly in `Random` fields in pure Solix (seeded via `native_seed()`) and mask `(int32)(val & 0x7FFFFFFFL)`! |
| **BUG-11** | `io/FileStream.slx` & `system/Process.slx` | Missing `~FileStream()` and `~Process()` destructors, leaking OS file descriptors and child process handles when going out of scope. | Add `~FileStream()`, `~BufferedReader()`, `~StreamReader()`, `~StreamWriter()`, and `~Process()` RAII destructors! |
| **BUG-12** | `io/FileMode.slx` & `MemoryStream.slx` | `FileMode` was a class with `int32` fields instead of an `enum`, and `seek` used magic numbers `0, 1, 2`. | Use `public enum FileMode { READ = 0, WRITE = 1, APPEND = 2, READ_WRITE = 3 }` and `public enum SeekOrigin { BEGIN = 0, CURRENT = 1, END = 2 }`. |
| **BUG-13** | `system/Console.slx` | `Console` was in `solix.system` instead of `solix.io`, and `input_int()` didn't trim whitespace. | Move `Console` into `package solix.io;` (with a compatibility alias in `solix.system`), add `ConsoleColor` enum, variadic `printf<args...>`, and trim whitespace in `input_int()` / `input_double()`. |
| **BUG-14** | `collections/Collections.slx:21` | Calls `item.to_string()` on generic `T`, which fails when `T` is a primitive (`int32`, `bool`, `float64`). | Provide `String::from_any<T>(T val)` so collections of primitives and objects format cleanly! |
| **BUG-15** | `exceptions/Exceptions.slx` | Exceptions only accepted `char[]` in constructors, not `String` or `string`. | Move `Exceptions` into `solix.core` (re-exported in `solix.exceptions`) with overloaded constructors accepting `String`, `string`, `char[]`, and `()`. |

---

## 5. Pillar 2: Complete `solixlib` v2.0.0 Project Structure & Module Specifications

```
libraries/solixlib/
├── manifest.toml                          # name = "solix", version = "2.0.0"
├── scripts/
│   ├── build.py                           # Invoked by `solix build libraries/solixlib`
│   └── solix_toolbox.py                   # Compiles Rust cdylib + Solix sources + packs solix.slxarc
├── natives/
│   └── solixlib_native/                   # Cross-platform Rust cdylib (replaces C++ native/src/*.cpp)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                     # SolixCValue / SolixVmContext C-ABI entrypoints
│           ├── console_io.rs              # Console print/input/colors & FileStream handles
│           ├── fs_path.rs                 # Path, File, Directory, Resource
│           ├── string_prim.rs             # UTF-8/numeric conversions, Primitives bit/float ops, hashing
│           ├── math_time.rs               # Math transcendentals & DateTime/Instant/Stopwatch clocks
│           ├── sys_process.rs             # Environment vars/OS info & Process spawn/wait/kill
│           └── crypto.rs                  # SHA-256, SHA-1, MD5, Base64, Hex
├── src/
│   ├── core/                              # package solix.core;
│   │   ├── Interfaces.slx                 # IStringable, IHashable, IEquatable<T>, IComparable<T>, ICloneable<T>
│   │   ├── Primitives.slx                 # Aliases + 32/64-bit bit manipulation + float IEEE-754 checks
│   │   ├── String.slx                     # Operators +, ==, !=, <, <=, >, >=, []; variadic format<args...>()
│   │   ├── StringBuilder.slx              # Fluent append, append_line, insert, reverse, operator[]
│   │   ├── Optional.slx                   # Optional<T> with map, filter, or_else
│   │   ├── Result.slx                     # Result<T, E> with map, map_err, unwrap_or
│   │   └── Exceptions.slx                 # Exception hierarchy (with solix.exceptions alias)
│   ├── collections/                       # package solix.collections;
│   │   ├── Interfaces.slx                 # IIterator<T>, IIterable<T>, ICollection<T>, IList<T>, IDeque<T>, ISet<T>, IMap<K,V>
│   │   ├── List.slx                       # Dynamic array + operator[], List::of<T, args...>, filter/map/reduce/sort
│   │   ├── LinkedList.slx                 # Doubly-linked list implementing IList<T> & IDeque<T> + operator[]
│   │   ├── Deque.slx                      # O(1) Ring-buffer Deque<T> + operator[]
│   │   ├── Queue.slx                      # O(1) Ring-buffer FIFO Queue<T>
│   │   ├── Stack.slx                      # LIFO Stack<T>
│   │   ├── CircularBuffer.slx             # Fixed-capacity ring buffer + operator[]
│   │   ├── KeyValuePair.slx               # KeyValuePair<K, V>
│   │   ├── HashMap.slx                    # Hash table with working default hash + operator[K]
│   │   ├── HashSet.slx                    # HashSet<T> + set operators (&, |, ^, -)
│   │   ├── TreeMap.slx                    # Balanced Red-Black TreeMap<K, V> + operator[K]
│   │   ├── TreeSet.slx                    # Ordered TreeSet<T>
│   │   ├── PriorityQueue.slx              # Binary heap PriorityQueue<T>
│   │   ├── BitSet.slx                     # 64-bit word-packed BitSet + operators &, |, ^, ~, []
│   │   ├── Collections.slx                # Collection formatting & conversion utilities
│   │   └── Algorithms.slx                 # O(N log N) sort, binary_search, reverse, fill, shuffle, min, max
│   ├── io/                                # package solix.io;
│   │   ├── Enums.slx                      # enum FileMode, enum SeekOrigin, enum ConsoleColor
│   │   ├── Console.slx                    # Terminal I/O + variadic printf<args...>, print, println, colors
│   │   ├── IStream.slx                    # Stream interface
│   │   ├── MemoryStream.slx               # In-memory byte stream
│   │   ├── FileStream.slx                 # OS file stream with ~FileStream() destructor
│   │   ├── BufferedReader.slx             # Buffered reader with ~BufferedReader() destructor
│   │   ├── StreamReader.slx               # Text stream reader with ~StreamReader() destructor
│   │   ├── StreamWriter.slx               # Text stream writer with ~StreamWriter() destructor
│   │   ├── Path.slx                       # Path manipulation + operators /, ==, != + variadic Path::join<args...>
│   │   ├── File.slx                       # Static file helpers (exists, read_all_text, write_all_text, etc.)
│   │   ├── Directory.slx                  # Static directory helpers (exists, create, list, remove, etc.)
│   │   └── Resource.slx                   # Project resource loader
│   ├── math/                              # package solix.math;
│   │   ├── Math.slx                       # Generic abs/min/max/clamp + variadic min_all/max_all + trig/exp/pow
│   │   └── Random.slx                     # Pure-Solix xoshiro256** PRNG
│   ├── time/                              # package solix.time;
│   │   ├── Duration.slx                   # Duration + operators +, -, *, /, ==, !=, <, <=, >, >=
│   │   ├── Instant.slx                    # Monotonic Instant + operators +, -, ==, !=, <, <=, >, >=
│   │   ├── DateTime.slx                   # UTC/Local DateTime + operators +, -, ==, !=, <, <=, >, >=
│   │   └── Stopwatch.slx                  # High-resolution Stopwatch
│   ├── system/                            # package solix.system;
│   │   ├── Environment.slx                # Env vars, CLI args, OS/arch/CPU info, current/temp/home dir
│   │   ├── Process.slx                    # Child process management with ~Process() RAII destructor
│   │   └── ProcessResult.slx              # Process execution result
│   └── crypto/                            # package solix.crypto;
│       ├── Hash.slx                       # SHA-256, SHA-1, MD5 (raw bytes & hex String)
│       ├── Base64.slx                     # Base64 encode/decode
│       └── Hex.slx                        # Hex encode/decode
└── tests/                                 # Comprehensive Solix Test Suite (`solix build --test`)
    ├── test_main.slx                      # TestRunner::test() orchestrator running all 7 module suites
    ├── test_core.slx                      # Verifies every function in solix.core
    ├── test_collections.slx               # Verifies every collection, iterator, operator & algorithm
    ├── test_io.slx                        # Verifies Console, MemoryStream, FileStream, Readers/Writers, Path, File, Dir
    ├── test_math.slx                      # Verifies every Math function & Random distribution/bounds
    ├── test_time.slx                      # Verifies Duration, Instant, DateTime, Stopwatch & operators
    ├── test_system.slx                    # Verifies Environment & Process spawning/pipes/exit codes
    └── test_crypto.slx                    # Verifies SHA-256, SHA-1, MD5, Base64, Hex against RFC test vectors
```

---

## 6. Exhaustive Function-by-Function Test Verification Plan (`libraries/solixlib/tests/`)

As you requested, **all tests for `solixlib` must live inside `libraries/solixlib/tests/`** and verify that every single function, method, constructor, destructor, and operator works as intended:

### 6.1 `libraries/solixlib/tests/test_core.slx`
* **`Primitives`:** Tests `max_int8`..`max_uint64`, `count_leading_zeros32/64`, `count_trailing_zeros32/64`, `bit_count32/64`, `rotate_left32/64`, `rotate_right32/64`, `reverse_bytes32/64`, `highest_one_bit32/64`, `lowest_one_bit32/64`, `is_power_of_two32/64`, `next_power_of_two32/64`, `is_nan`, `is_infinite`, `is_finite`, `float_to_raw_bits`, `raw_bits_to_float`, `double_to_raw_bits`, `raw_bits_to_double`.
* **`String`:** Tests all constructors (`""`, `char[]`, `string`), `length()`, `is_empty()`, `is_blank()`, `char_at(i)`, `operator[](i)`, `substring`, `slice`, `to_upper()`, `to_lower()`, `trim()`, `trim_start()`, `trim_end()`, `starts_with`, `ends_with`, `contains`, `index_of`, `last_index_of`, `count`, `replace`, `replace_first`, `repeat`, `pad_left`, `pad_right`, `reverse()`, `split`, `lines()`, `join`, `format_array`, variadic `String::format<args...>()`, `from_int32`, `from_int64`, `from_float64`, `from_bool`, `to_int32`, `to_int64`, `to_float64`, `to_bool`, `hash_code()`, `equals`, `compare_to`, `clone()`, and operators `+`, `==`, `!=`, `<`, `<=`, `>`, `>=`.
* **`StringBuilder`:** Tests `append`, `append_char`, `append_int`, `append_long`, `append_double`, `append_bool`, `append_line`, `insert`, `delete_range`, `replace`, `reverse`, `clear`, `char_at`, `set_char_at`, `operator[]`, `length`, `capacity`, `ensure_capacity`, `to_string`.
* **`Optional<T>` & `Result<T, E>`:** Tests `of`, `empty`, `of_nullable`, `is_present`, `is_empty`, `get`, `or_else`, `or_else_get`, `if_present`, `filter`, `map`, and `Result::ok`, `Result::err`, `is_ok`, `is_err`, `unwrap`, `unwrap_err`, `unwrap_or`, `map`.
* **`Exceptions`:** Tests throwing and catching `Exception`, `RuntimeException`, `IndexOutOfBoundsException`, `NullPointerException`, `IllegalArgumentException`, `IllegalStateException`, `ArithmeticException`, `NumberFormatException`, `IOException`, `FileNotFoundException`, `InvalidCastException`, `UnsupportedOperationException`, `NoSuchElementException`, `TimeoutException`, `SecurityException`, `CryptoException`, `ProcessException`, `KeyNotFoundException`, `OutOfMemoryException` with both `String` and `char[]` messages.

### 6.2 `libraries/solixlib/tests/test_collections.slx`
* **`List<T>`:** Tests `new List<T>()`, `List::of<T, args...>()`, `add`, `insert`, `get`, `set`, `operator[]`, `remove_at`, `remove`, `contains`, `index_of`, `last_index_of`, `size`, `is_empty`, `clear`, `to_array`, `sub_list`, `reverse`, `sort`, `for_each`, `filter`, `map`, `reduce`, `any`, `all`, `count_if`, `find`, `iterator()`.
* **`LinkedList<T>`:** Tests `add_first`, `add_last`, `push_front`, `push_back`, `remove_first`, `remove_last`, `pop_front`, `pop_back`, `peek_first`, `peek_last`, `get`, `set`, `operator[]`, `remove`, `contains`, `index_of`, `reverse`, `filter`, `map`, `iterator()`.
* **`Deque<T>`, `Queue<T>`, `Stack<T>`, `CircularBuffer<T>`:** Tests ring-buffer growth, wrap-around, `push_front`/`push_back`/`pop_front`/`pop_back`, `enqueue`/`dequeue`/`peek`, `push`/`pop`/`peek`, `CircularBuffer` overwrite-on-full, `operator[]`, and $O(1)$ iterators.
* **`HashMap<K, V>` & `HashSet<T>`:** Tests default hashing (`new HashMap<String, int32>()` and `new HashMap<int32, String>()` without custom hash functions!), `put`, `get`, `operator[]`, `get_or_default`, `put_if_absent`, `remove`, `contains_key`, `contains_value`, `keys`, `values`, `entries`, resize/rehash under 500+ keys, and `HashSet<T>` union `|`, intersection `&`, symmetric difference `^`, and difference `-`.
* **`TreeMap<K, V>` & `TreeSet<T>`:** Tests Red-Black Tree balancing on sorted insertions (`1..200`), `put`, `get`, `operator[]`, `remove` (leaf, one-child, and two-child root/internal nodes), `first_key`, `last_key`, `floor_key`, `ceiling_key`, `lower_key`, `higher_key`, `keys` (verifying strictly ascending order!), and `TreeSet<T>` `first`, `last`, `subset`.
* **`PriorityQueue<T>` & `BitSet`:** Tests min-heap/max-heap `enqueue`, `dequeue`, `peek`, and 64-bit word `BitSet` `set`, `clear`, `flip`, `get`, `operator[]`, `cardinality`, `next_set_bit`, `next_clear_bit`, `and` (`&`), `or` (`|`), `xor` (`^`), `and_not`.
* **`Algorithms`:** Tests $O(N \log N)$ `sort_int32`, `sort_int64`, `sort_float64`, generic `sort<T>`, `binary_search`, `reverse`, `fill`, `copy`, `min`, `max`, `equals`, `shuffle`, `is_sorted`.

### 6.3 `libraries/solixlib/tests/test_io.slx`
* **`Console` & `Enums`:** Tests `FileMode`, `SeekOrigin`, `ConsoleColor`, `Sprint`/`Sprintln`/`String::format` integration, `MemoryStream` (`write_byte`, `read_byte`, `write`, `read`, `seek` with `SeekOrigin::BEGIN`/`CURRENT`/`END`, `position`, `length`, `to_array`, `clear`).
* **`FileStream`, `BufferedReader`, `StreamReader`, `StreamWriter`:** Creates temporary files, writes lines/bytes via `StreamWriter` and `FileStream`, reads them back via `StreamReader` (`read_line`, `read_lines`, `read_to_end`) and `BufferedReader`, and verifies `~FileStream()` / `~StreamWriter()` RAII destructors close handles automatically on scope exit.
* **`Path`, `File`, `Directory`:** Tests `Path::join<args...>`, `operator/`, `parent`, `file_name`, `stem`, `extension`, `with_extension`, `is_absolute`, `normalize`, `File::write_all_text`, `File::read_all_text`, `File::append_all_text`, `File::write_all_bytes`, `File::read_all_bytes`, `File::read_all_lines`, `File::exists`, `File::size`, `File::copy`, `File::rename`, `File::remove`, and `Directory::create`, `Directory::exists`, `Directory::list`, `Directory::remove`.

### 6.4 `libraries/solixlib/tests/test_math.slx`
* **`Math`:** Tests constants (`PI`, `E`, `TAU`, `SQRT2`), generic `abs`, `min`, `max`, `clamp`, `sign`, variadic `min_all<args...>`, `max_all<args...>`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `sqrt`, `cbrt`, `pow`, `exp`, `expm1`, `log`, `log10`, `log2`, `log1p`, `floor`, `ceil`, `round`, `trunc`, `fmod`, `hypot`, `lerp`, ` smoothstep`, `to_radians`, `to_degrees`, `gcd`, `lcm`, `is_prime`, `factorial`, `fibonacci`.
* **`Random`:** Tests seeded deterministic `Random(12345L)` and default `Random()`, verifying `next_int()` is always $\ge 0$, `next_int(max)` is in $[0, \text{max})$, `next_range(min, max)` is in $[\text{min}, \text{max})$, `next_int64()`, `next_double()` is in $[0.0, 1.0)$, `next_float()`, `next_bool()`, `next_bytes(buf)`, and `next_gaussian()`.

### 6.5 `libraries/solixlib/tests/test_time.slx`, `test_system.slx`, `test_crypto.slx`
* **`time`:** Tests `Duration` (`from_nanos`, `from_micros`, `from_millis`, `from_seconds`, `from_minutes`, `from_hours`, `from_days`, conversions, arithmetic `+`, `-`, `*`, `/`, comparisons `<`, `<=`, `>`, `>=`, `==`, `!=`), `Instant` (`now`, `elapsed`, `+`, `-`, comparisons), `DateTime` (`now_utc`, `now_local`, `from_unix_timestamp`, `of`, `year`, `month`, `day`, `hour`, `minute`, `second`, `day_of_week`, `is_leap_year`, `add_days`, `format_iso8601`, operators), and `Stopwatch` (`start_new`, `start`, `stop`, `reset`, `elapsed`).
* **`system`:** Tests `Environment` (`get_var`, `set_var`, `remove_var`, `has_var`, `os_name`, `arch`, `cpu_count`, `current_dir`, `temp_dir`, `home_dir`, `process_id`, `is_windows`, `is_linux`, `is_macos`) and `Process` (`exec`, `start`, `wait`, `is_running`, `exit_code`, `read_stdout`, `read_stderr`, `~Process()` destructor).
* **`crypto`:** Tests `Hex` (`encode`, `decode`, `is_valid`), `Base64` (`encode`, `encode_string`, `decode`, `decode_to_string`, `encode_url_safe`, `decode_url_safe`), and `Hash` (`sha256`, `sha256_hex`, `sha1`, `sha1_hex`, `md5`, `md5_hex`, `constant_time_equals`) against official NIST/RFC test vectors!

---

## 7. Phased Implementation Plan

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Client Generator Script (`scripts/init_project.py`) & `solix new` │
│          └─► `solix new <path>` delegates to Python generator script       │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Extend `SolixVmContext` in `crates/solix-runtime/src/native/ffi.rs`│
│          └─► Add string/char[]/array callbacks to `SolixVmContext`         │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Scaffold `libraries/solixlib` & Build `natives/solixlib_native/`  │
│          └─► Rust `cdylib` implementing all native functions               │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Implement `solix.core` & `solix.collections` (`src/`)             │
│          └─► Modern generics, operators, variadics, O(N log N) algorithms  │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Implement `solix.io`, `math`, `time`, `system`, `crypto` (`src/`) │
│          └─► `io.Console`, RAII destructors, pure-Solix RNG, RFC crypto    │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Write Exhaustive Solix Tests in `libraries/solixlib/tests/`       │
│          └─► Verify every function via `solix build --test` & `solix run`  │
└────────────────────────────────────────────────────────────────────────────┘
```

## 8. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Client Project Generator Script** | `scripts/init_project.py` + `solix new <path>` (Rust `cdylib` or pure Solix, no CMake) | `cargo test -p solix-client` | [ ] Pending |
| **`SolixVmContext` Heap Callbacks** | `read_bytes`, `alloc_string`, `alloc_chars`, `alloc_byte_array`, `alloc_array`, `array_get/set_u64` in `ffi.rs` | `cargo test -p solix-runtime` | [ ] Pending |
| **Rust Native Companion (`solixlib_native`)** | Compiles cleanly via Cargo into `solix.slxarc` without C++ | `solix build libraries/solixlib --debug` | [ ] Pending |
| **Complete `solixlib/tests/` Suite** | Every function in `core`, `collections`, `io`, `math`, `time`, `system`, `crypto` verified in `libraries/solixlib/tests/` | `solix build libraries/solixlib --test && solix run --test libraries/solixlib/build/test/solix.slxarc` | [ ] Pending |
| **Package Store Installation** | `solix install libraries/solixlib/build/release/solix.slxarc` + downstream `extern import solix.io.*;` | `cargo test -p solix-client` | [ ] Pending |
| **Clippy & Formatting Clean** | Zero warnings across all workspace crates | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |