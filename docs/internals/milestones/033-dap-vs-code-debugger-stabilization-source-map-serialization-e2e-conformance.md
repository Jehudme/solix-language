# Milestone 033: DAP & VS Code Debugger Stabilization, Lossless Debug Metadata Serialization, Exact Breakpoint Resolution & Strict E2E Conformance

## Document Metadata

* **Milestone ID:** MS-033
* **Target Version:** `v0.1.0-alpha.33`
* **Milestone Branch:** `milestone/033-dap-and-vscode-debugger-stabilization`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/029-single-executable-consolidation-lsp-remediation-and-common-solver.md` (MS-029)
  * `docs/internals/milestones/030-industrial-debug-adapter-protocol-solix-dap-and-zero-overhead-vm-hooks.md` (MS-030)
  * `docs/internals/milestones/031-vscode-debugger-integration-release-run-debug-ux-and-automated-e2e-suite.md` (MS-031)

---

## 1. Executive Summary & Forensic Root-Cause Analysis

Interactive testing of the Solix VS Code Debugger (`solix dap --stdio` + `editors/vscode`) in `sample_workspace` revealed that placing breakpoints on lines 47–52 inside `main()` caused the debugger to pause unexpectedly inside `Point::distance_from_origin` at line `0:1`, jump the editor cursor to Line 1, Column 1 of `main.slx`, and display an empty `Locals & Parameters` scope.

A line-by-line forensic audit of `crates/solix-dap/src/`, `crates/solix-compilation/src/phases/emitter/`, `crates/solix-common/src/bytecode/`, `crates/solix-runtime/src/`, and `editors/vscode/src/` exposed **10 interlocking bugs** that caused this exact failure mode and allowed the automated test (`dap_rpc.test.mjs`) to pass falsely:

### 1.1 Forensic Root-Cause Table (All 10 Debugger Bugs)

| Bug ID | Exact Location | What the Code Currently Does (The Bug) | Catastrophic User-Visible Consequence |
| :--- | :--- | :--- | :--- |
| **BUG-DAP-01**<br>*(Unresolved Breakpoint Wildcard Trap at `pc = 0`)* | `solix-dap/src/server.rs:602-625`<br>`solix-dap/src/controller.rs:106-112` | 1. In `process_set_breakpoints` (`server.rs:602`), when `setBreakpoints` arrives before `launch` (which is standard in VS Code!) or when `resolve_breakpoint_location` returns `None`, it sets **`instruction_pc = 0`** and **`function_id = None`** with **`verified = true`**!<br>2. In `resolve_breakpoint_location` (`server.rs:549-585`), it searches `for (function_index, compiled_function) in all_functions` and picks the **FIRST function in the file** whose `source_map` has an entry (or fails to match), instead of checking that `requested_line` lies **within that function's start/end line range**!<br>3. In `controller.rs:107`: `None => true` matches **EVERY function in the program** at `instruction_pc == 0`! | Setting a breakpoint on line 47 of `main()` either remains `function_id: None, pc: 0` (trapping at `pc = 0` of the first function executed!) or snaps to the wrong function (`Point::distance_from_origin`)! |
| **BUG-DAP-02**<br>*(Missing or Unserialized `source_map`, `local_variables`, and `source_files` in `.slxb` / `.slxarc`)* | `solix-common/src/bytecode/binary.rs`<br>`solix-compilation/src/phases/emitter/` | When `solix build --debug` compiles `sample_workspace` into `build/debug/sample_workspace.slxarc` (`bin/sample_workspace.slxb`), `compiled_function.source_map` (1-based `line` numbers) and `compiled_function.local_variables` (`LocalVariableDebugInfo`) are either not populated for every statement/parameter/local or not serialized/deserialized by `BinaryProgram::to_bytes()` / `from_bytes()`! | `find_source_line_for_pc` returns **`0`** (`0:1` in Call Stack!), causing VS Code to jump to the top of `main.slx` (`Ln 1, Col 1`), and `Locals & Parameters` is completely empty! |
| **BUG-DAP-03**<br>*(Relative vs. Absolute Path Mismatch in `StackTrace` & `resolve_breakpoint_location`)* | `solix-dap/src/server.rs:212-246`<br>`solix-dap/src/controller.rs:50-65` | When `solix build --debug` compiles from `projectRoot`, `loaded_binary.source_files` stores relative paths (`"src/main.slx"`), while VS Code sends absolute paths (`"/home/.../sample_workspace/src/main.slx"`). Worse: `StackTrace` returns the raw relative string `"src/main.slx"` (or `""`) in `Source.path` and `line: 0` when `line_number == 0`! | Breakpoint path matching fails on Windows/Unix path separators (`\` vs `/`), and `StackTrace` returns `0:1` or relative paths that fail to highlight the active line in VS Code! |
| **BUG-DAP-04**<br>*(`Command::Scopes` Wipes Out All Variable References via `reset_transient_references()`)* | `solix-dap/src/server.rs:260` | `Command::Scopes` calls `reference_manager.reset_transient_references()` at the top of the handler! Whenever VS Code requests `Scopes` for Frame 0 and then Frame 1 (or `SolixInlineValuesProvider` queries scopes) before requesting `Variables`, the second `Scopes` request **deletes all `variablesReference` IDs** from the first `Scopes` request! | Clicking to expand `Locals & Parameters`, `Static & Global Variables`, or object fields in VS Code returns an empty list (`[]`) because the reference ID was wiped! |
| **BUG-DAP-05**<br>*(Guest `println` Writes Raw Text to `stdout`, Corrupting or Deadlocking the `--stdio` DAP Stream!)* | `solix-dap/src/server.rs:63-66, 520-525` | 1. `server.rs:64` holds `stdout_stream.lock()` inside `BufWriter` for the entire lifetime of the DAP server.<br>2. `setup_runtime_for_target` registers `builtin_println` / `builtin_print`, which write raw strings directly to `std::io::stdout()`! In `--stdio` mode, `stdout` IS the DAP JSON-RPC wire channel (`Content-Length: ...\r\n\r\n`)! | Any `println(...)` executed by the debugged program either deadlocks on `stdout.lock()` or injects raw text into the JSON-RPC stream, corrupting DAP communication with VS Code! |
| **BUG-DAP-06**<br>*(`readPackageName` in `debug.ts` and `commands.ts` Only Matches `[package]`, Ignoring `[info]`!)* | `editors/vscode/src/debug.ts:32`<br>`editors/vscode/src/commands.ts:39` | `manifestContent.match(/\[package\][^[]*?name\s*=\s*["']([^"']+)["']/s)` only looks for `[package]`, whereas Solix `manifest.toml` files use **`[info]`** (`[info]\nname = "sample_workspace"`)! | `readPackageName` fails to read `[info].name` from `manifest.toml`! |
| **BUG-DAP-07**<br>*(`SetVariable` Only Works on Raw Register Names `'r0'`, Ignoring Named Local Variables & Object Fields!)* | `solix-dap/src/server.rs:384-416` | In `Command::SetVariable`, `VariableScopeType::FrameLocals(frame_idx)` checks `if set_variable_arguments.name.starts_with('r')`! Local variables are named `origin_point`, `calculated_distance`, `static_result`—not `r0`! And `VariableScopeType::ObjectFields(obj_addr)` is not handled at all (`_ => {}`)! | Editing a local variable or object field in the VS Code Variables pane silently fails to update the VM state! |
| **BUG-DAP-08**<br>*(Array Elements & String Values Are Not Formatted or Expandable in `Variables`!)* | `solix-dap/src/server.rs:312-380, 631-642` | 1. `format_runtime_value` formats every `Value::Ref(addr)`—including `String` (`BlockTag::String`) and `ArrayRef`—as `"Object@0x000000D0"` instead of reading the string's UTF-8 text or showing `Array(len=N)`!<br>2. `VariableScopeType::ArrayElements(_)` at line 376 is an empty stub `{}`! | Strings display as opaque `"Object@0x..."` hex addresses instead of `"hello"`, and arrays cannot be expanded to inspect their elements! |
| **BUG-DAP-09**<br>*(`setup_runtime_for_target` on `.slx` Files Lacks `ClientModuleSolver` & Project Sibling Sources)* | `solix-dap/src/server.rs:514-567` | When launching a `.slx` file directly, `setup_runtime_for_target` creates `CompilationOptions::new()` without `.with_solver(Arc::new(ClientModuleSolver::new(...)))` and without loading sibling `src/*.slx` files or `manifest.toml` `[dependencies]`! | Debugging a standalone `.slx` file that uses `extern import solix.io.Console;` fails to compile in `solix dap`! |
| **BUG-DAP-10**<br>*(`dap_rpc.test.mjs` Did Not Assert `topFrame.line == 46` or Non-Empty `variables`!)* | `editors/vscode/test/dap_rpc.test.mjs:147-185` | `dap_rpc.test.mjs` set a breakpoint at line 46, waited for any `stopped` event, logged `Top frame: main at line 0` and `Locals variables: ` (empty), and only asserted `10 + 20 === 30`! | The test passed with a 100% broken breakpoint engine and empty variable table! |

---

## 2. Detailed Architectural Remediation Specification

### 2.1 Lossless Debug Metadata Emission & Binary Serialization (`solix-common`, `solix-compilation`, `solix-runtime`)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│            Complete Debug Metadata Pipeline (`.slx` -> `.slxb` -> DAP)           │
├──────────────────────────────────────────────────────────────────────────────────┤
│  1. EmitterPhase (`solix-compilation/src/phases/emitter/`):                      │
│     • For every compiled function/method/constructor/destructor:                 │
│       - Sets `func.source_id = source_id`.                                       │
│       - Converts AST `Span` byte offsets into 1-based `(line, column)` using     │
│         `context.sources[source_id]` and records `SourceMapEntry {               │
│         instruction_pc, span, line, column }` before every statement & call!     │
│       - Records `LocalVariableDebugInfo` for every parameter (`this` at `r0`,    │
│         params at `r1..rK`) and every local variable (`VariableDeclaration`,     │
│         `ForStatement` initializer, `CatchClause` variable) with its register!   │
│     • Populates `BinaryProgram.source_files` with canonical file paths!          │
│     • Populates `CompiledClass.field_names` with `(offset, name, type_name)`!    │
├──────────────────────────────────────────────────────────────────────────────────┤
│  2. Binary Serialization (`solix-common/src/bytecode/binary.rs`):                │
│     • Verifies `BinaryProgram::to_bytes()` and `from_bytes()` serialize and      │
│       deserialize `func.source_id`, `func.source_map` (with `line` & `column`),  │
│       `func.local_variables`, `class.field_names`, and `program.source_files`!   │
├──────────────────────────────────────────────────────────────────────────────────┤
│  3. VM Inspection (`solix-runtime/src/vm/engine.rs`):                            │
│     • `inspect_frame_locals(frame_index)`:                                       │
│       - Reads `func.local_variables` where `entry.start_pc <= frame.pc`          │
│         (always including parameters where `is_parameter == true`).              │
│       - Returns `(entry.name, entry.type_name, reg_value)` for each active var!  │
│     • `inspect_object_fields(heap_addr)`:                                        │
│       - Reads `vtable_id` at `heap_addr`, looks up `CompiledClass`, and reads    │
│         every field in `class.field_names` using its exact byte offset & tag!    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Exact Enclosing-Function Breakpoint Resolution (`crates/solix-dap/src/server.rs` & `controller.rs`)

1. **Function-Scoped Line Matching (`resolve_breakpoint_location`):**
   * When resolving `(source_path, requested_line)`:
     * Normalize both `source_path` and `function_source_file` (replacing `\` with `/` and comparing canonical suffixes or file names).
     * For each `compiled_function` in that file with a non-empty `source_map`:
       * Compute `func_min_line = compiled_function.source_map.iter().map(|e| e.line).min().unwrap_or(0)` and `func_max_line = compiled_function.source_map.iter().map(|e| e.line).max().unwrap_or(0)`.
       * **Crucial Rule:** A breakpoint at `requested_line` can ONLY match `compiled_function` if `requested_line >= func_min_line.saturating_sub(1) && requested_line <= func_max_line`!
       * Within that enclosing function, pick the first `SourceMapEntry` with `entry.line >= requested_line` (snapping to the exact `entry.instruction_pc` and `entry.line`).
2. **Never Wildcard-Trap on Unresolved Breakpoints (`controller.rs:106`):**
   * In `DapController::before_instruction`:
     * A `BreakpointEntry` **MUST** have `let Some(bp_func_id) = line_breakpoint.function_id` to trigger! If `function_id` is `None`, it **NEVER** triggers!
     * Furthermore, if `setBreakpoints` was called before `launch`, `Command::Launch` re-runs `resolve_breakpoint_location` on all pending breakpoints once `created_vm` is initialized, and emits `Event::Breakpoint` updates to VS Code.

### 2.3 Safe Guest `stdout` Redirection Over DAP (`Event::Output`)

1. **Eliminating `stdout.lock()` Contention & JSON-RPC Stream Corruption:**
   * Do **not** hold `io::stdout().lock()` across the entire session loop in `server.rs:64`; instead, wrap `io::Stdout` (without a permanent `.lock()`) in `BufWriter::new(io::stdout())`.
   * In `setup_runtime_for_target`, register custom DAP print callbacks (`dap_builtin_print`, `dap_builtin_println`) that append guest output strings into a shared `Arc<Mutex<Vec<String>>>` queue (`pending_console_outputs`).
   * Inside `run_until_stop` (and on every pause/termination), drain `pending_console_outputs` and send each message to VS Code via:
     ```rust
     server.send_event(Event::Output(OutputEventBody {
         category: Some(OutputEventCategory::Stdout),
         output: message,
         ..Default::default()
     }))?;
     ```
   * **Result:** `println(...)` and `Console::println(...)` never corrupt the `--stdio` JSON-RPC wire stream and appear live in the **VS Code Debug Console**!

### 2.4 Fixing `Scopes`, `Variables` (Strings, Arrays, Objects), `SetVariable`, and `StackTrace`

1. **`ReferenceManager` Lifecycle (`server.rs:260`):**
   * Remove `reference_manager.reset_transient_references()` from `Command::Scopes`!
   * Call `reference_manager.reset_transient_references()` **only** inside `run_until_stop` right before the VM resumes execution.
2. **Rich Value Formatting (`format_runtime_value` in `server.rs`):**
   * Pass `vm_ref: &VirtualMachine` to `format_runtime_value(vm_ref, value)`:
     * If `Value::Ref(addr)` points to `BlockTag::String`, read the UTF-8 string bytes from `vm_ref.heap` and format as `"\"Calculated distance from origin: 30\""`!
     * If `Value::Ref(addr)` points to `BlockTag::Object` or `BlockTag::ObjectPod`, read `vtable_id`, look up the class name (`Point`), and format as `"Point@0x000000D0"` (and make it expandable via `VariableScopeType::ObjectFields(addr)`!).
     * If `Value::Ref(addr)` points to an Array, read its length `len` and format as `"Array(len=N)@0x..."` (and make it expandable via `VariableScopeType::ArrayElements(addr)`!).
3. **Mutating Named Local Variables & Object Fields (`Command::SetVariable`):**
   * When `scope_type` is `VariableScopeType::FrameLocals(frame_idx)`:
     * Look up `set_variable_arguments.name` in `compiled_function.local_variables` to find its `register: u16` (or parse `r<N>` if it is a raw register name), and call `vm_ref.mutate_register(frame_idx, register, parsed_value)`.
   * When `scope_type` is `VariableScopeType::ObjectFields(object_addr)`:
     * Look up `set_variable_arguments.name` in the object's `CompiledClass.field_names` and write the new value to `object_addr + field_offset` in `vm_ref.heap`!
4. **Absolute Source Paths & 1-Based Lines in `StackTrace` (`server.rs:212-246`):**
   * Store `launch_project_root: Option<PathBuf>` in `handle_client_session`.
   * When constructing `StackFrame` in `Command::StackTrace`, if `file_path` is relative (e.g. `"src/main.slx"`), join it with `launch_project_root` and canonicalize it so VS Code always receives the exact absolute path (`/home/.../sample_workspace/src/main.slx`).
   * Ensure `line_number` is always $\ge 1$ (if `instruction_pc == 0` and `source_map` has entries, use `source_map[0].line`).

---

## 3. Phased Implementation Plan

### Phase 1: Audit & Fix Compiler Debug Emission & Binary Serialization (`solix-common`, `solix-compilation`, `solix-runtime`)
* [x] `[P0]` **Task 1.1:** Verify and fix `EmitterPhase` (`crates/solix-compilation/src/phases/emitter/`) so every `CompiledFunction` (including instance methods, static methods, constructors, destructors, and free functions like `main` and `test_addition`) records:
  - Accurate 1-based `(line, column)` on every `SourceMapEntry`.
  - Complete `local_variables: Vec<LocalVariableDebugInfo>` for `this` (`r0` on instance methods/constructors/destructors), all parameters, and all local variables (`origin_point`, `calculated_distance`, `static_result`, `computed_sum`).
  - Canonical `source_files: Vec<String>` on `BinaryProgram` and `field_names` on `CompiledClass`.
* [x] `[P0]` **Task 1.2:** Verify and fix `BinaryProgram::to_bytes()` and `BinaryProgram::from_bytes()` (`crates/solix-common/src/bytecode/binary.rs`) so `source_id`, `source_map` (including `line` and `column`), `local_variables`, `field_names`, `is_debug_profile`, and `source_files` round-trip through `.slxb` and `.slxarc` containers with zero data loss!
* [x] `[P0]` **Task 1.3:** Verify `VirtualMachine::inspect_frame_locals` and `VirtualMachine::inspect_object_fields` (`crates/solix-runtime/src/vm/engine.rs`) return all parameters (`this`, `operand_first`, `operand_second`), local variables (`origin_point`, `calculated_distance`), and object fields (`coordinate_x = 10`, `coordinate_y = 20`).

### Phase 2: Fix Breakpoint Resolution, `Scopes`, `Variables`, `SetVariable`, & `stdout` Capture (`crates/solix-dap`)
* [x] `[P0]` **Task 2.1:** Fix `resolve_breakpoint_location` and `process_set_breakpoints` in `crates/solix-dap/src/server.rs` and `before_instruction` in `crates/solix-dap/src/controller.rs`:
  - Match breakpoints only to the function whose `[func_min_line..=func_max_line]` range encloses `requested_line`.
  - Never trigger a breakpoint when `line_breakpoint.function_id.is_none()`.
  - Re-resolve all pre-launch breakpoints inside `Command::Launch` and normalize Windows/Unix path separators.
* [x] `[P0]` **Task 2.2:** Remove `reference_manager.reset_transient_references()` from `Command::Scopes` and place it in `run_until_stop` before VM execution resumes.
* [x] `[P0]` **Task 2.3:** Upgrade `format_runtime_value`, `Command::Variables`, and `Command::SetVariable` in `crates/solix-dap/src/server.rs` to display UTF-8 strings (`"..."`), class names (`Point@0x...`), expandable `ArrayElements`, and support mutating named local variables and object fields.
* [x] `[P0]` **Task 2.4:** Redirect guest `print` / `println` / `Console::println` during DAP sessions into DAP `Event::Output` notifications so guest stdout appears in the VS Code Debug Console without corrupting or deadlocking the `--stdio` JSON-RPC stream.
* [x] `[P0]` **Task 2.5:** Wire `ClientModuleSolver` (from `solix-common`) and project sibling file discovery into `setup_runtime_for_target` so standalone `.slx` files with `extern import` or sibling classes debug cleanly.

### Phase 3: Fix VS Code Extension Manifest Parsing & Debugger UX (`editors/vscode`)
* [x] `[P0]` **Task 3.1:** Fix `readPackageName` in `editors/vscode/src/debug.ts` and `editors/vscode/src/commands.ts` to parse both `[info]` and `[package]` sections in `manifest.toml`.
* [x] `[P0]` **Task 3.2:** Ensure `SolixDebugConfigurationProvider` passes `projectRoot` (`cwd`) in `debugConfiguration` so `solix dap` resolves relative `src/main.slx` paths to absolute filesystem paths on `stackTrace`.

### Phase 4: Strict End-to-End Automated Verification (`dap_tests.rs` & `dap_rpc.test.mjs`)
* [x] `[P0]` **Task 4.1:** Add end-to-end `.slxarc` and multi-function breakpoint tests in `crates/solix-dap/tests/dap_tests.rs`:
  - Compile `sample_workspace/src/main.slx` into a `.slxb` / `.slxarc` binary, load it from bytes, set breakpoints on lines 47, 48, 49, and assert:
    1. Execution does **NOT** stop inside `Point::distance_from_origin` or at line 0!
    2. Execution stops at **line 47** inside `main`!
    3. `inspect_frame_locals` returns `origin_point`, `calculated_distance = 30`, and `static_result = 100`!
    4. `inspect_object_fields` on `origin_point` returns `coordinate_x = 10` and `coordinate_y = 20`!
    5. Stepping (`Next`) advances from line 47 $\to$ line 48 $\to$ line 49!
* [x] `[P0]` **Task 4.2:** Upgrade `editors/vscode/test/dap_rpc.test.mjs` to test **both** launching `sample_workspace/src/main.slx` **and** launching `sample_workspace/build/debug/sample_workspace.slxarc`, with strict assertions on `topFrame.name === 'main'`, `topFrame.line === 46`, non-empty `Locals & Parameters` (`origin_point`, `calculated_distance`), object field expansion (`coordinate_x === '10'`, `coordinate_y === '20'`), `next` stepping, and `output` event capture!

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Zero Wildcard `pc = 0` Stops** | Breakpoints on lines 47–52 in `main()` never stop in `Point::distance_from_origin` or at `0:1` | `cargo test -p solix-dap` | [x] Signed Off |
| **Lossless `.slxb` / `.slxarc` Debug Info** | `source_map` lines, `local_variables`, and `field_names` survive `.slxb` serialization | `cargo test -p solix-dap -- test_exact_breakpoint_resolution_and_locals_inspection` | [x] Signed Off |
| **Populated `Locals & Parameters` Scope** | `origin_point`, `calculated_distance`, `this` visible and expandable in Variables pane | `cd editors/vscode && npm run test:dap-rpc` | [x] Signed Off |
| **Guest `println` in Debug Console** | `println` emits DAP `output` events without corrupting `--stdio` JSON-RPC | `cd editors/vscode && npm run test:dap-rpc` | [x] Signed Off |
| **Clippy, Rustfmt & VSIX Package Clean** | Zero warnings across Rust workspace and regenerated `solix-0.1.0.vsix` | `cargo clippy --workspace --all-targets -- -D warnings && cd editors/vscode && npm run package` | [x] Signed Off |