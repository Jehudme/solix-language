# Embedding Solix Compilation and Runtime in Rust

## Document Metadata

* **Scope:** Developer Guide for Host Applications, Game Engines, and Embedding Pipelines
* **Target Crates:** `solix-compilation`, `solix-runtime`, `solix-common`
* **Status:** Current Implementation Standard

---

## 1. Overview & Architecture

Solix provides modular Rust crates that allow host applications to compile Solix source code directly in memory and execute the resulting bytecode through the register-based virtual machine or native Cranelift JIT engine without spawning subprocesses.

```mermaid
flowchart LR
    Source["Solix Source Code (.slx)"] --> Pipeline["solix-compilation (CompilationPipeline)"]
    Pipeline --> Binary["BinaryProgram (.slxb bytes)"]
    Binary --> Context["solix-runtime (RuntimeContext)"]
    Context --> VM["VirtualMachine (Interpreter / Cranelift JIT)"]
    VM --> Output["Execution Result (Value)"]
```

* **`solix-common`**: Shared bytecode data structures (`BinaryProgram`, `Instruction`, `OpCode`), constants, and resolver abstractions.
* **`solix-compilation`**: Multi-phase compilation pipeline: dependency resolution, lexing, parsing, semantic binding, AST inlining, and bytecode emission.
* **`solix-runtime`**: Register-based virtual machine, heap memory manager (`VirtualHeap`), ARC memory lifecycle, cycle collector, and native Cranelift JIT compiler backend.

---

## 2. Cargo Dependency Configuration

Add the required Solix crates to the host application's `Cargo.toml`. When embedding the virtual machine on 64-bit desktop/server platforms (`x86_64` and `aarch64`), enable the `jit` feature to leverage Cranelift native machine-code compilation:

```toml
[dependencies]
solix-common = { path = "../solix-language/crates/solix-common" }
solix-compilation = { path = "../solix-language/crates/solix-compilation" }
solix-runtime = { path = "../solix-language/crates/solix-runtime", features = ["jit"] }
```

> [!NOTE]
> The `jit` feature requires a supported 64-bit architecture (`x86_64` or `aarch64`) and non-WebAssembly targets. On 32-bit platforms or WebAssembly embeddings, omit the `jit` feature to run purely on the high-performance interpreter loop.

---

## 3. Compilation Pipeline Options Reference (`CompilationOptions`)

The `solix_compilation::CompilationContext::CompilationOptions` structure controls source inputs, optimization thresholds, and binary packaging:

| Option Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `profile` | `BuildProfile` | `BuildProfile::Release` | Optimization profile: `Debug` (unoptimized, exact lines), `Release` (inlining + peephole optimizations), or `Test` (test harness). |
| `codes` | `HashMap<String, String>` | Empty | In-memory source code files mapped by path or filename (e.g. `"main.slx"` -> source text). |
| `package_name` | `String` | Empty | Name of the compiled package or library. |
| `package_version` | `String` | Empty | Semantic version string of the package. |
| `package_description` | `String` | Empty | Human-readable package description. |
| `inline_threshold` | `usize` | `8` (`Release`) / `0` (`Debug`) | Maximum statement count for automatic AST function inlining. Set to `0` to disable automatic inlining. |
| `bytecode_optimization` | `bool` | `true` (`Release`) / `false` (`Debug`) | Whether to execute post-emission bytecode peephole optimizations. |
| `dependency_binaries` | `Vec<BinaryProgram>` | Empty | Pre-compiled dependencies linked into the compilation context. |
| `dependency_specs` | `Vec<DependencyRequirement>` | Empty | Explicit external package requirements resolved via package solver. |
| `solver` | `Option<Arc<dyn ModuleSolver>>` | `None` | Pluggable module solver for retrieving transitive dependencies. |
| `entry_point` | `Option<String>` | `None` | Optional explicit entry point function symbol (defaults to `"main"`). |
| `link_mode` | `LinkMode` | `LinkMode::Dynamic` | Linking strategy: `LinkMode::Dynamic` (external linking) or `LinkMode::Static` (monolithic binary). |
| `verify_checksums` | `bool` | `true` | Enforces cryptographic SHA-256 checksum validation on external modules. |
| `embed_dependency_hashes` | `bool` | `false` | Embeds dependency cryptographic hashes into the emitted binary header. |
| `emit_export_metadata` | `bool` | `true` | Serializes public API AST headers and disassembly maps for external consumption. |

### Builder Methods for `CompilationOptions`

`CompilationOptions` provides a fluent builder interface:

```rust
use solix_compilation::{BuildProfile, CompilationOptions, LinkMode};

let options = CompilationOptions::new()
    .with_source("main.slx", "fn main() -> int32 { return 42; }")
    .with_profile(BuildProfile::Release)
    .with_link_mode(LinkMode::SingleBinary)
    .with_inline_threshold(12)
    .with_bytecode_optimization(true)
    .with_emit_export_metadata(false);
```

---

## 4. Runtime Options Reference (`RuntimeOptions`)

The `solix_runtime::RuntimeOptions` structure configures the virtual memory capacity, execution safety checks, JIT mode, and host integration:

| Option Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `heap_size` | `usize` | `64 * 1024 * 1024` (64 MB) | Capacity of the byte-addressed dynamic virtual heap in bytes. |
| `max_call_stack_depth` | `usize` | `65_536` | Maximum call recursion depth before triggering a stack overflow error (`R0003`). |
| `jit_mode` | `JitMode` | `JitMode::Tiered` | Cranelift JIT mode: `JitMode::Off` (interpreter only), `JitMode::Tiered` (hot functions + loops), or `JitMode::Eager` (all functions on 1st call). |
| `safe_mode` | `bool` | `false` | When `true`, enforces strict defensive bounds checking; when `false`, runs on the fast dispatch engine. |
| `track_metrics` | `bool` | `true` | Records instruction counts, JIT compile timings, and heap allocations into `VmTelemetry`. |
| `sandboxed` | `bool` | `false` | Restricts guest program access to native host libraries and system APIs. |
| `cycle_collector` | `bool` | `true` | Enables the Bacon-Rajan ARC cycle collector to detect cyclic references. |
| `cycle_threshold` | `usize` | `1024` | Count of purple deallocation candidates buffering before triggering collection. |
| `cycle_budget_us` | `Option<u64>` | `None` | Optional execution time budget in microseconds for incremental collection. |
| `args` | `Vec<String>` | Empty | Command-line arguments made accessible to the guest program via standard library APIs. |
| `binaries` | `Vec<BinaryProgram>` | Empty | In-memory binary programs loaded directly into the runtime context. |
| `binary_paths` | `Vec<PathBuf>` | Empty | File paths of `.slxb` binaries to load on startup. |
| `native_libraries` | `Vec<PathBuf>` | Empty | Paths of shared native dynamic libraries (`.so`, `.dll`, `.dylib`) loaded on startup. |
| `native_functions` | `HashMap<String, NativeFunction>` | Empty | Custom Rust host functions exposed directly to the guest runtime. |
| `output_sink` | `Option<OutputSink>` | `None` | Thread-safe writer for intercepting guest standard output and error streams. |

### Builder Methods for `RuntimeOptions`

```rust
use solix_runtime::{JitMode, RuntimeOptions};

let options = RuntimeOptions::builder()
    .with_heap_size(32 * 1024 * 1024)
    .with_max_call_stack_depth(4096)
    .with_jit_mode(JitMode::Tiered)
    .with_safe_mode(false)
    .with_cycle_collector(true)
    .build();
```

---

## 5. End-to-End Embedding Guide

Below is a complete, working example demonstrating how to compile Solix source code in memory and execute it inside the virtual machine.

```rust
use solix_compilation::{
    BuildProfile, CompilationContext::CompilationContext, CompilationOptions,
    pipeline::CompilationPipeline,
};
use solix_runtime::{JitMode, RuntimeContext, RuntimeOptions, Value, VirtualMachine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Define Solix source code
    let source_code = r#"
        class Calculator {
            static fn add(first_val: int32, second_val: int32) -> int32 {
                return first_val + second_val;
            }
        }

        fn main() -> int32 {
            let result_val: int32 = Calculator::add(15, 27);
            return result_val;
        }
    "#;

    // Step 2: Configure compilation options
    let compilation_options = CompilationOptions::new()
        .with_source("main.slx", source_code)
        .with_profile(BuildProfile::Release)
        .with_emit_export_metadata(false);

    let mut context = CompilationContext::new(compilation_options);

    // Step 3: Run the compilation pipeline
    let mut pipeline = CompilationPipeline::standard();
    if let Err(diagnostics) = pipeline.execute(&mut context) {
        for diagnostic in diagnostics {
            eprintln!("Compilation error: {}", diagnostic.message());
        }
        return Err("Compilation failed".into());
    }

    // Step 4: Extract the emitted binary program
    let binary_program = context
        .binary_program
        .take()
        .ok_or("No binary program produced by compilation pipeline")?;

    // Step 5: Configure runtime options and initialize runtime context
    let runtime_options = RuntimeOptions::builder()
        .with_heap_size(16 * 1024 * 1024)
        .with_jit_mode(JitMode::Tiered)
        .build();

    let mut runtime_context = RuntimeContext::new(runtime_options);
    let binary_id = runtime_context.load_binary(binary_program)?;

    // Step 6: Create VirtualMachine and execute the entry point
    let mut vm = VirtualMachine::new(&runtime_context);
    let execution_result = vm.execute(binary_id)?;

    match execution_result {
        Value::I32(return_int) => {
            println!("Program returned successfully with: {}", return_int);
            assert_eq!(return_int, 42);
        }
        other_value => {
            println!("Program returned unexpected value: {:?}", other_value);
        }
    }

    Ok(())
}
```

---

## 6. Calling Specific Solix Functions from Rust

In host integrations such as game engines, scripts often expose handler methods rather than a single `main()` entry point. The host can look up function symbols and invoke them directly:

```rust
use solix_runtime::{RuntimeContext, Value, VirtualMachine};

fn invoke_host_handler(
    runtime_context: &RuntimeContext,
    vm: &mut VirtualMachine,
    target_symbol: &str,
    arguments: &[Value],
) -> Result<Value, Box<dyn std::error::Error>> {
    let function_id = runtime_context
        .resolve_function_symbol(target_symbol)
        .ok_or_else(|| format!("Function symbol not found: {}", target_symbol))?;

    let result_value = vm.invoke(function_id, arguments)?;
    Ok(result_value)
}
```

---

## 7. Exposing Custom Rust Native Functions to Solix

The host application can expose custom Rust functions to the Solix runtime using `RuntimeOptionsBuilder::with_native_function`:

```rust
use solix_runtime::{
    NativeFunction, RuntimeContext, RuntimeOptions, RuntimeError, Value, VirtualMachine,
};

fn native_log_message(
    _context: &mut solix_runtime::vm::VirtualMachine,
    arguments: &[Value],
) -> Result<Value, RuntimeError> {
    if let Some(first_arg) = arguments.first() {
        println!("[HOST LOG] Received argument: {:?}", first_arg);
    }
    Ok(Value::Null)
}

fn create_engine_with_host_callbacks() -> RuntimeContext {
    let options = RuntimeOptions::builder()
        .with_native_function(
            "Host::log(any)".to_string(),
            NativeFunction::Dynamic(native_log_message),
        )
        .build();

    RuntimeContext::new(options)
}
```
