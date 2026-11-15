# Milestone 031: Industrial Debug Adapter Protocol (`solix-dap` / `solix dap`), Debug AST/Variable Symbols, Zero-Overhead VM Dispatch & Mixed-Profile Smart Stepping

## Document Metadata

* **Milestone ID:** MS-031
* **Target Version:** `v0.1.0-alpha.30`
* **Milestone Branch:** `milestone/030-solix-dap-and-vm-debugger`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/029-single-executable-consolidation-lsp-remediation-and-common-solver.md` (MS-029)

---

## 1. Executive Summary & Architectural Vision

To provide an industrial-grade interactive debugging experience in Visual Studio Code (and any DAP-compatible editor such as Neovim `nvim-dap`, Helix, or Zed), **Milestone 030** implements the complete **Solix Debug Adapter Protocol (`crates/solix-dap`)** exposed via **`solix dap [--stdio | --port <PORT>]`** in `solix-toolchain`.

Three non-negotiable engineering requirements govern this milestone:

1. **Zero Performance Impact in Release Mode (`0.0%` Overhead Guarantee):**
   * Normal execution (`solix run` / `BuildProfile::Release`) **MUST NOT** pay a single extra CPU cycle, branch check, or memory read for debugging support.
   * We achieve this via **Compile-Time Const-Generic Monomorphization (`run_fast_internal<const DEBUG_MODE: bool>`)** in `crates/solix-runtime/src/vm/fast.rs`:
     * When `DEBUG_MODE == false` (standard `vm.run()`), the Rust compiler completely eliminates all `if DEBUG_MODE { ... }` blocks at compile time, emitting the exact same hot loop machine code as Milestone 018!
     * When `DEBUG_MODE == true` (`vm.run_debug(&mut debug_controller)`), the debug-enabled loop checks breakpoints, step states, and exception traps.
2. **Full Private + Public Declaration AST & Local Variable Symbol Tables in Debug Builds:**
   * When compiling with `BuildProfile::Debug` (`solix compile --profile debug` / `solix build --debug`):
     * `ApiAndAstExportPhase` preserves **all declarations**—including `private` classes, `private` interfaces, `private` enums, `private` functions/methods, and `private static` fields—inside Section 7 (`ExportedAst`) of the `.slxb` binary container.
     * `EmitterPhase` populates `local_variables: Vec<LocalVariableDebugInfo>` on every `CompiledFunction`, mapping each virtual register `r0..rN` to its source identifier name, static `Type`, declaration `Span`, and live PC range `[start_pc, end_pc)`.
     * `EmitterPhase` emits fine-grained `SourceMapEntry` records (`pc`, `source_id`, `line`, `column`, `span`) for every statement and expression boundary, and stores `source_paths: Vec<String>` in `BinaryProgram` so `source_id` maps directly to its canonical filesystem path.
3. **Mixed Debug + Release Dependency Smart-Stepping ("Just-My-Debug-Code"):**
   * A project compiled in `debug` mode frequently depends on pre-installed packages (such as `solix@2.0.0.slxarc` or `raylib@1.0.0.slxarc`) that were compiled and installed in **`release` mode** (`build_profile == "release"`, where private symbols are stripped, functions are inlined, and local variable tables are empty).
   * **How `solix-dap` Handles Mixed Debug/Release Binaries Seamlessly:**
     * Every `LoadedBinary` and `FunctionRuntimeMeta` in `RuntimeContext` records `has_debug_symbols: bool` (`binary.build_profile == "debug" && !func.source_map.is_empty()`).
     * When the user presses **Step Into (`F11` / `stepIn`)** on a line that calls a `release`-compiled library function (e.g. `Console::println(...)` or `List::add(...)` where `has_debug_symbols == false`):
       * The VM **does NOT stop inside the stripped release function**! Instead, it automatically runs the release library function at full speed and only pauses if execution enters a `debug`-compiled function (for example, when `List::for_each` invokes a user lambda compiled in `debug` mode!) or when execution returns to the caller's `debug` frame!
     * If an uncaught runtime error or exception originates inside a `release` dependency frame, the Call Stack (`stackTrace`) still displays the release frame's function name (`[External Release Code] solix::collections::List::get`) while automatically highlighting and focusing the top-most user `debug` frame in the editor!

---

## 2. The Complete 24-Feature Modern DAP Matrix

Powered by the standard Rust **`dap`** crate (`dap = "0.4"`) for canonical DAP protocol types/framing, `crates/solix-dap` implements **24 modern debugger features**:

| # | DAP Request / Event / Capability | Feature Name | Solix Debugger Behavior |
| :-: | :--- | :--- | :--- |
| **1** | `initialize` & `initialized` | Capability Handshake | Advertises conditional breakpoints, hit-conditional breakpoints, logpoints, exception filters, `evaluateForHovers`, `setVariable`, `stepBack` readiness, and `terminateRequest`. |
| **2** | `launch` (`.slx`, `.slxb`, `.slxarc`) | Multi-Target Debug Launch | Launches standalone `.slx` scripts (auto-compiled in memory with `BuildProfile::Debug`), `.slxb` binaries, or `.slxarc` project archives; supports `stopOnEntry`, `args`, `cwd`, and `env`. |
| **3** | `attach` (TCP Socket) | Live VM Attach | Connects to a running Solix VM started with `solix dap --port <PORT>`. |
| **4** | `configurationDone` | Execution Kickoff | Begins VM execution after initial breakpoints and exception filters are configured. |
| **5** | `setBreakpoints` | Line & Column Breakpoints | Resolves `(file_path, line)` to the nearest executable instruction `pc` via `SourceMapEntry`, returning `verified: true` and the snapped line number (or `verified: false` with a reason if placed on a comment/blank line or inside a `release`-compiled file). |
| **6** | Conditional Breakpoints | Expression-Guarded Traps | Evaluates a boolean Solix condition expression (e.g. `coordinate_x > 50` or `index == 10`) in the active frame; pauses only when the condition evaluates to `true`. |
| **7** | Hit-Conditional Breakpoints | Hit-Count Traps | Tracks execution hit count per breakpoint and pauses when the hit condition (`== N`, `>= N`, `% N == 0`) is satisfied. |
| **8** | Logpoints (`logMessage`) | Non-Stopping Trace Logs | Interpolates `{variable_or_expr}` placeholders in `logMessage` and emits an `output` event (`category: "console"`) without pausing VM execution! |
| **9** | `setFunctionBreakpoints` | Function Name Breakpoints | Breaks on entry (`pc = 0`) of any function or method matching a qualified or unqualified name (e.g. `Point::distance_from_origin` or `main`). |
| **10** | `setExceptionBreakpoints` | `thrown` & `uncaught` Filters | Supports two filter IDs: **`uncaught`** (pauses immediately when an exception or runtime trap is about to unwind out of `main` unhandled) and **`thrown`** (pauses at the exact `OpCode::Throw` or runtime fault PC before stack unwinding!). |
| **11** | `continue` (`F5`) | Resume Execution | Resumes execution until the next breakpoint, logpoint, exception trap, pause request, or program exit. |
| **12** | `next` (`F10` — Step Over) | Statement-Level Step Over | Advances execution until `call_frames.len() <= start_depth` and `(current_source_id, current_line) != (start_source_id, start_line)` in a debug-enabled function. |
| **13** | `stepIn` (`F11` — Smart Step Into) | Smart Step Into | Steps into the called function if `callee.has_debug_symbols == true`; **automatically skips over `release`-compiled library frames** while still catching callbacks into `debug` closures! |
| **14** | `stepOut` (`Shift+F11` — Step Out) | Frame Return Step Out | Runs until the current call frame pops (`call_frames.len() < start_depth`) and pauses at the caller's return site. |
| **15** | `pause` | Asynchronous Pause | Interrupts a running loop at the next instruction boundary and emits `stopped` (`reason: "pause"`). |
| **16** | `threads` | VM Thread Reporting | Reports the active Solix VM execution thread (`id: 1, name: "Solix Main Thread"`). |
| **17** | `stackTrace` | Rich Mixed-Mode Call Stack | Returns `StackFrame` entries for all active `CallFrame`s with file path, 1-based line/column, parameter values, and `presentationHint: "subtle"` for external `release`-compiled dependency frames. |
| **18** | `scopes` | 3-Tier Frame Scopes | Returns three expandable scopes for any selected stack frame: **1) `Locals & Parameters`**, **2) `Static & Global Variables`**, and **3) `VM Registers & ARC Telemetry`**. |
| **19** | `variables` (Structured Heap Tree) | Recursive Heap & AST Inspector | Expands `Object` / `ObjectPod` references into their declared class fields (including `private` and inherited fields with types and values!), `ArrayRef` into indexed `[0..N]` elements (with `indexedVariables` paging), `Tuple` into `.0..N` slots, `Closure` into captured variables, and `String` into UTF-8 text + byte length! |
| **20** | `setVariable` | Live Variable & Field Mutation | Allows editing the value of a local variable, register, static variable, or object field directly in the VS Code Variables pane while paused! |
| **21** | `evaluate` (Watch, Hover & REPL) | Debug Console Expression Evaluator | Evaluates identifiers, field chains (`origin_point.coordinate_x`), array subscripts (`arr[2]`), tuple indexing (`t[0]`), binary arithmetic/comparisons (`a + b * 2`, `x == 10`), and `sizeof(Point)` in the context of any paused `frameId`! |
| **22** | `modules` | Loaded Binary & Profile Inspector | Lists all loaded `.slxb` / `.slxarc` modules in the VS Code Modules pane, showing package name, version, SHA-256, and whether symbols are loaded (`Debug Symbols Loaded` vs `Release Optimized (Skipped)`). |
| **23** | `disassemble` | Live 64-Bit ISA Disassembly View | Implements DAP `disassemble` so VS Code's "Open Disassembly View" shows the live 64-bit Register VM opcodes (`LoadI32`, `AddI32Imm`, `CallDirect`) alongside source lines! |
| **24** | Guest `stdout` / `stderr` Capture & `terminate` | Live Console Streaming | Streams guest `print`, `println`, and `Console::*` output directly to the VS Code Debug Console via DAP `output` events and verifies clean shutdown on `disconnect` / `terminate`. |

---

## 3. Deep Technical Design

### 3.1 Compiler & `.slxb` Container Debug Metadata (`solix-common` & `solix-compilation`)

#### 1. `LocalVariableDebugInfo` & Enhanced `SourceMapEntry` (`crates/solix-common/src/bytecode/function.rs`)
```rust
/// Maps a virtual register within a compiled function to its source variable name and type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalVariableDebugInfo {
    /// Virtual register index (`r0..rN`).
    pub register: u16,
    /// User-declared variable or parameter name.
    pub name: String,
    /// Static type name (e.g. `"int32"`, `"sample_workspace::Point"`).
    pub type_name: String,
    /// Static compiler `Type`.
    pub var_type: crate::types::Type,
    /// Inclusive starting bytecode PC where this variable enters scope.
    pub start_pc: u32,
    /// Exclusive ending bytecode PC where this variable exits scope.
    pub end_pc: u32,
    /// Whether this register is a function/method parameter (`r0` = `this` for instance methods).
    pub is_parameter: bool,
}
```
And on `SourceMapEntry`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub pc: u32,
    pub span: Span,
    /// 1-based source line number.
    #[serde(default)]
    pub line: u32,
    /// 1-based source column number.
    #[serde(default)]
    pub column: u32,
}
```
And on `BinaryProgram` (`crates/solix-common/src/bytecode/binary.rs`):
* Add `#[serde(default)] pub source_files: Vec<String>` (persisted in `ExportManifest`) mapping `source_id: u16` to the canonical file path (e.g. `"/path/to/project/src/main.slx"`).
* Add `#[serde(default)] pub field_names: Vec<(usize, String, String)>` (`(byte_offset, field_name, field_type_name)`) to `CompiledClass` (`crates/solix-common/src/bytecode/class.rs`) so the runtime and debugger can immediately format every field of any heap object (`Point { coordinate_x: 10, coordinate_y: 20 }`) by `vtable_id` in $O(1)$ time!

#### 2. Retaining Private Declarations in `ApiAndAstExportPhase` When `profile == BuildProfile::Debug`
* In `crates/solix-compilation/src/phases/export_api.rs`:
  * When `context.options.profile == BuildProfile::Debug`, `ApiAndAstExportPhase` includes **all** local classes, interfaces, enums, aliases, methods, constructors, static variables, and fields (both `public` and `private` / `internal`) in Section 7 (`ExportedAst`) of `.slxb`.
  * When `context.options.profile == BuildProfile::Release`, it continues to prune `private` classes, `private` methods, and `private` static fields so production `.slxb` containers remain minimal.

---

### 3.2 Zero-Overhead Const-Generic VM Dispatch & Mixed-Profile Smart Stepping (`solix-runtime`)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│          Zero-Overhead Const-Generic VM Loop (`solix-runtime/src/vm/fast.rs`)    │
├──────────────────────────────────────────────────────────────────────────────────┤
│  pub fn run(&mut self) -> Result<Value, RuntimeError> {                          │
│      self.run_fast_internal::<false>(None) // <-- 0.0% overhead in Release!      │
│  }                                                                               │
│                                                                                  │
│  pub fn run_debug(&mut self, hook: &mut dyn VmDebugHook) -> Result<...> {        │
│      self.run_fast_internal::<true>(Some(hook)) // <-- Full DAP instrumentation  │
│  }                                                                               │
└──────────────────────────────────────────────────────────────────────────────────┘
```

#### How `DEBUG_MODE = true` Works Without Slowing Down `Release` Dependencies:
1. Inside `run_fast_internal<const DEBUG_MODE: bool>`:
   ```rust
   while pc < inst_len {
       if DEBUG_MODE {
           // Only invoked when compiled with DEBUG_MODE == true!
           // Fast check: does current_func have debug symbols (build_profile == "debug")?
           if current_func_has_debug_symbols {
               if let Some(hook) = debug_hook.as_deref_mut() {
                   // Sync frame PC before calling hook
                   if let Some(frame) = self.call_frames.last_mut() {
                       frame.pc = pc as u32;
                   }
                   match hook.before_instruction(self, curr_func_id, pc as u32)? {
                       DebugSignal::Continue => {}
                       DebugSignal::Terminate => return Ok(Value::I32(0)),
                   }
                   reload_frame!();
               }
           }
       }
       let inst = unsafe { *inst_ptr.add(pc) };
       pc += 1;
       ...
   }
   ```
2. **Look at `if current_func_has_debug_symbols` above:**
   * `current_func_has_debug_symbols: bool` is a local register boolean loaded once when entering a frame (`meta.has_debug_symbols`).
   * If a user's `debug` program calls `Algorithms::sort` or `HashMap::put` in `solixlib` (which was compiled in `release` mode, so `meta.has_debug_symbols == false`), **the VM does not even call `hook.before_instruction` while executing inside the `release` library function!**
   * The `release` dependency runs at 100% native VM speed! And the moment `Algorithms::sort` calls the user's comparator closure (which was compiled in the user's `debug` binary, so `meta.has_debug_symbols == true`), `current_func_has_debug_symbols` becomes `true` and breakpoints/stepping immediately engage inside the user's closure!

---

### 3.3 Output Redirection for Guest `print` / `println` / `Console` (`solix-runtime`)
Currently, `builtins.rs` and `console_io.rs` use `print!` / `println!` directly to OS stdout. When running under `solix dap --stdio`, OS stdout is the JSON-RPC wire channel to VS Code! Writing raw guest `println("hello")` to OS stdout would corrupt the DAP `Content-Length` JSON-RPC stream!
* **The Fix:**
  * Add an optional thread-safe output sink `pub output_sink: Option<Arc<dyn Fn(&str, bool) + Send + Sync>>` to `RuntimeOptions` and `VirtualMachine` (and route `solixlib_native`'s `Console` print calls or `builtins.rs` through it when set).
  * When `solix dap --stdio` runs, it attaches an `output_sink` that forwards every guest `print` / `println` string as a proper DAP **`Event::Output(OutputEventBody { category: Some(OutputEventCategory::Stdout), output })`** to the VS Code Debug Console!

---

## 4. Phased Implementation Plan

### Phase 1: Compiler & `.slxb` Debug Metadata (`solix-common` & `solix-compilation`)
* [x] `[P0]` **Task 1.1:** Add `LocalVariableDebugInfo` and 1-based `(line, column)` on `SourceMapEntry` in `crates/solix-common/src/bytecode/function.rs`, `field_names: Vec<(usize, String, String)>` on `CompiledClass` (`class.rs`), and `source_files: Vec<String>` on `ExportManifest` / `BinaryProgram`.
* [x] `[P0]` **Task 1.2:** Update `EmitterPhase` (`crates/solix-compilation/src/phases/emitter/`) to:
  - Record `LocalVariableDebugInfo` entries for all parameters (`r0 = this`, `r1..rK`) and local variables (`VariableDeclarationStatement`, `ForStatement` init, `CatchClause` exception variable) with their `[start_pc, end_pc)` ranges.
  - Record `SourceMapEntry` at every statement and expression boundary with 1-based line and column coordinates.
  - Populate `CompiledClass::field_names` and `BinaryProgram::source_files`.
* [x] `[P0]` **Task 1.3:** Update `ApiAndAstExportPhase` (`crates/solix-compilation/src/phases/export_api.rs`) so `BuildProfile::Debug` retains all `private` and `internal` classes, interfaces, enums, functions, methods, and static fields in `ExportedAst`.

### Phase 2: Zero-Overhead Const-Generic VM Debug Hook & Heap Inspector (`solix-runtime`)
* [x] `[P0]` **Task 2.1:** Add `has_debug_symbols: bool` to `FunctionRuntimeMeta` in `crates/solix-runtime/src/context.rs` (`binary.build_profile == "debug" && !func.source_map.is_empty()`).
* [x] `[P0]` **Task 2.2:** Define `VmDebugHook` trait (`before_instruction`, `on_exception_thrown`, `on_uncaught_exception`) and `DebugSignal` in `crates/solix-runtime/src/vm/debug.rs`.
* [x] `[P0]` **Task 2.3:** Parameterize `run_fast_internal<const DEBUG_MODE: bool>` in `crates/solix-runtime/src/vm/fast.rs` so `vm.run()` uses `DEBUG_MODE = false` (zero overhead) and `vm.run_debug(hook)` uses `DEBUG_MODE = true` (calling `hook` on debug-enabled functions and exception sites).
* [x] `[P0]` **Task 2.4:** Add `output_sink` callback to `RuntimeOptions` and `VirtualMachine` so guest `print`/`println` outputs can be captured cleanly by DAP without polluting `stdout`, and add safe heap/frame inspection helpers (`inspect_frame_locals`, `inspect_object_fields`, `inspect_array_elements`, `inspect_tuple_elements`, `inspect_closure_captures`, `mutate_register_or_field`).

### Phase 3: `crates/solix-dap` Engine & `solix dap` CLI Command (`solix-dap` & `solix-toolchain`)
* [x] `[P0]` **Task 3.1:** Add `dap = "0.4"` to `[workspace.dependencies]` and `crates/solix-dap/Cargo.toml`.
* [x] `[P0]` **Task 3.2:** Implement the complete DAP state machine and debugger controller in `crates/solix-dap/src/`:
  - `breakpoints.rs`: Line-to-PC snapping via `SourceMapEntry`, function breakpoints, conditional breakpoint evaluator, hit counts, and logpoint `{expr}` interpolation.
  - `stepping.rs`: `StepIn` (with automatic skip of `release` dependency frames where `has_debug_symbols == false`, while pausing on debug callback re-entry!), `StepOver` (`next`), `StepOut`, `Continue`, and `Pause`.
  - `variables.rs`: Handle allocation (`variableReference` IDs) for `Locals & Parameters`, `Static & Global Variables`, `VM Registers & ARC Telemetry`, and recursive expansion of `Object` fields (including private fields), `ArrayRef` elements, `Tuple` elements, `Closure` captures, and `setVariable` live mutation.
  - `evaluator.rs`: Expression evaluator for Watch, Hover, Conditional Breakpoints, and Debug Console REPL (supporting variables, `this.field`, `obj.field`, `arr[idx]`, arithmetic/logical/comparison operators, and `sizeof`).
  - `server.rs`: Async/threaded DAP server loop handling all 24 DAP capabilities over `--stdio` or `--port <PORT>`, plus an in-memory `DapSession` test harness.
* [x] `[P0]` **Task 3.3:** Wire `solix dap [--stdio] [--port <PORT>]` in `crates/solix-toolchain/src/cli.rs` and `main.rs`.

### Phase 4: Exhaustive Automated DAP & Mixed-Profile Test Suite (`crates/solix-dap/tests/`)
* [x] `[P0]` **Task 4.1:** Write **25+ comprehensive automated tests** in `crates/solix-dap/tests/dap_tests.rs` verifying every single DAP feature:
  1. Handshake (`initialize` $\to$ `launch` $\to$ `configurationDone`).
  2. Line breakpoint snapping and pausing at the exact line.
  3. Conditional breakpoints (`index == 3`) and hit-count breakpoints (`hitCondition: "4"`).
  4. Logpoints (`"Value is {counter}"`) emitting `output` events without pausing.
  5. Function breakpoints (`Point::distance_from_origin`).
  6. `StepOver` (`next`), `StepIn`, and `StepOut` across multi-level function calls.
  7. **Mixed Debug + Release Dependency Smart-Stepping Test:** Compile `LibraryA` with `BuildProfile::Release` and `MainApp` with `BuildProfile::Debug`. Set a step-into (`stepIn`) on `LibraryA::compute()` and verify the debugger **skips over `LibraryA`'s internal release instructions** and lands directly on the next debug line of `MainApp` (AND test when `LibraryA::run_callback(cb)` invokes a `debug` closure `cb`, verifying `stepIn` pauses inside `cb`!).
  8. Inspecting `Locals`, `Parameters`, `Statics`, and expanding `Object` fields (including `private` fields!), `ArrayRef` elements, `Tuple` slots, and `Closure` captures.
  9. Mutating a local variable and an object field via `setVariable` and verifying the program computes with the new value!
  10. Evaluating complex expressions (`origin_point.coordinate_x + 10`, `arr[1] * 2`, `x > 5`) via `evaluate`.
  11. Exception breakpoints (`thrown` and `uncaught`) pausing on `throw new Exception("boom")` and divide-by-zero traps.
  12. `modules` request and `disassemble` request returning 64-bit Register VM opcodes.
  13. End-to-end stdio subprocess test (`solix dap --stdio`).

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **0.0% Release Overhead** | `run_fast_internal::<false>` has zero extra branches in release mode | `cargo test -p solix-runtime` | [x] Completed |
| **Full Debug Metadata in `.slxb`** | Private AST declarations, `LocalVariableDebugInfo`, `field_names`, and `source_files` emitted in `Debug` profile | `cargo test -p solix-compilation` | [x] Completed |
| **Mixed Debug/Release Stepping** | `stepIn` skips `release` dependencies & stops in `debug` callbacks | `cargo test -p solix-dap` | [x] Completed |
| **All 24 DAP Features Verified** | Breakpoints, Conditions, Logpoints, Watch/REPL, Heap Tree, `setVariable`, Exceptions, Disassembly | `cargo test -p solix-dap` (25+ tests) | [x] Completed |
| **Clippy & Rustfmt Clean** | Zero warnings, `#![forbid(unsafe_code)]` in `solix-dap`, zero single-letter vars (`GL-0011`) | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |