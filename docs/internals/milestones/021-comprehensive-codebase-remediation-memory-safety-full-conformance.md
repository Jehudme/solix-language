# Milestone 021: Comprehensive Codebase Remediation, VM Memory Safety, Compiler Miscompilation Fixes & Full Test Conformance

## Document Metadata

* **Milestone ID:** MS-021
* **Target Version:** `v0.1.0-alpha.21`
* **Milestone Branch:** `milestone/021-comprehensive-codebase-remediation`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/codebase-forensic-audit-and-bug-report.md`
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/020-package-manager-solver-and-dependency-resolution.md` (MS-020)
  * `crates/solix-tests/TESTS.md`

---

## 1. Executive Summary & Scope

Following the forensic codebase audit (`docs/internals/codebase-forensic-audit-and-bug-report.md`), **46 distinct issues** were identified across `solix-runtime`, `solix-compilation`, `solix-common`, `solix-client`, and `solix-tests`.

**Milestone 021** is a dedicated stabilization, remediation, and conformance milestone that resolves every single confirmed bug and test gap across 6 structured phases:

1. **Phase 1: Critical Runtime & Memory Safety Remediation (`BUG-RT-01` – `BUG-RT-09`):**
   * Eliminates the Use-After-Free dangling `reg_base` pointer when `register_stack` reallocates during destructors or native callbacks.
   * Fixes `try { ... } finally { ... }` unwinding so uncaught exceptions automatically re-propagate after `finally` executes.
   * Replaces the lossy NaN-boxing of 64-bit integers (`Value::I64`) in arrays, tuples, and closures so all 64-bit signed/unsigned integers (`i64::MIN..=i64::MAX`) preserve their exact type and value.
   * Fixes `OpCode::ArrayLength` on `BlockTag::String` constants, narrow-stride array accesses, multi-binary relocation for `CastCheck`/`CallInterface`/`TupleInstanceof`, and destructor exception propagation.
2. **Phase 2: Compiler & Emitter Miscompilation Fixes (`BUG-COMP-01` – `BUG-COMP-12`):**
   * Fixes `%=` (`OperatorModuloAssign`) miscompiling as `+=`, and adds 64-bit `AddI64`/`SubI64`/`MulI64`/`DivI64`/`ModI64` compound assignments and `++`/`--` increments.
   * Implements call-site default parameter injection (`void foo(int32 value_a, int32 value_b = 10)`).
   * Fixes `BytecodeOptimizerPhase` register read/write analysis for `CallExtern`, `ConvIToF`, `ConvFToI`, `LoadI64`, `LoadF64`, `LoadBool`, and `AllocObjectExtern`.
   * Exports `Declaration::Interface`, `Declaration::Enum` (with explicit values), and `ClassMember::Operator` in `ExportManifest` and `manifest_to_source`.
   * Implements `super.method(...)` binding and emission, string concatenation (`+`), generic method body monomorphization (`default(T)` / `sizeof(T)`), and generic parameter bound checking (`<T : IPrintable>`).
3. **Phase 3: Lexer, Parser & Semantic Binder Fixes (`BUG-PARSE-01` – `BUG-BIND-01`):**
   * Strips `_` digit separators in `parse_numeric_literal` so `1_000`, `0xDEAD_BEEF`, `0b1111_0000`, and `3.141_592` parse without false `E0150` overflow errors.
   * Resolves transitive `extern import` statements using a worklist in `ParserPhase`.
   * Rejects all non-`const` modifiers on local variables (`static`, `virtual`, `override`, `abstract`, `native`, `inline`, `weak`) with `E0117`.
   * Aliases `pkg.member` when `package pkg;` contains `public class pkg` (or root-facade functions) even when the caller file declares its own `package`.
4. **Phase 4: Variadic Function Templates (`<args...>`) Implementation (`TESTS.md` Cases 55.5 & 55.6):**
   * Implements trailing variadic pack lowering (`fn<args...>(int32 first, args...)`) and automatic call-site `tuple(...)` argument bundling so `TESTS.md` Cases `55.5` and `55.6` transition from `[NOT IMPLEMENTED]` to `[IMPLEMENTED]`.
5. **Phase 5: Package Manager, CLI & Cross-Platform Cleanup (`BUG-CLI-01` – `BUG-CLI-03`):**
   * Replaces hardcoded `.so` extensions in `build_and_archive_tests.rs` with platform-aware helpers (`.dll` / `.dylib` / `.so`).
   * Cleans up ephemeral `temp_cache_dir` directories created by `solix run` and `solix compile` using RAII `tempfile::TempDir` guards.
   * Filters `lib/<os>-<arch>/` subdirectories in `SlxarcArchive::extract_and_load` by `std::env::consts::OS` and `std::env::consts::ARCH`.
6. **Phase 6: Test Suite Deduplication & New Regression Conformance Suite (`solix-tests`):**
   * Deduplicates the 11 redundant `unit_*.rs` files in `solix-runtime/tests/` and duplicate fixtures.
   * Replaces fake/weak tests (`test_case_3_5`, `test_case_1_5`, `test_case_4_6..4_8`) and loose substring assertions with exact error code assertions (`E0XXX`).
   * Adds **35+ new regression and conformance tests** covering every single bug fixed in this milestone.

---

## 2. Detailed Technical Specifications by Phase

### 2.1 Phase 1: Critical Runtime & Memory Safety Remediation (`solix-runtime`)

```
┌────────────────────────────────────────────────────────────────────────────┐
│            Phase 1: Runtime Memory & Execution Remediation                 │
├────────────────────────────────────────────────────────────────────────────┤
│ 1. Reload `reg_base` after `release_reference` and `native_callback.invoke`│
│ 2. Track `pending_exception: Option<Value>` on `CallFrame` for `finally`   │
│ 3. Store 1-byte type tag in Tuple/Array/Closure or preserve `I64` safely   │
│ 4. Handle `BlockTag::String` in `OpCode::ArrayLength` (`header.size`)      │
│ 5. Relocate `CastCheck`, `CallInterface`, and `TupleInstanceof` vtable IDs │
│ 6. Propagate errors thrown inside destructors (`~Class()`)                 │
└────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Fixing `BUG-RT-01` (Dangling `reg_base` Pointer in `crates/solix-runtime/src/vm/fast.rs`)
* **Root Cause:** `self.register_stack` can reallocate inside `native_callback.invoke(self, &call_args)` or inside `self.release_reference(...)` (when an object's `~destructor()` pushes a new `CallFrame`).
* **Implementation:**
  * In `OpCode::CallDirect`, `OpCode::CallNativeDirect`, `OpCode::CallVirtual`, `OpCode::CallInterface`, `OpCode::CallSymbol`, `OpCode::CallExtern`, and `OpCode::CallCallable`:
    Immediately after `let call_result = native_callback.invoke(self, &call_args)?;`, execute `reload_frame!();` **before** writing `*reg_base.add(caller_dst as usize) = call_result;`.
  * In `OpCode::DecRef`, `OpCode::SetField`, and `OpCode::SetArray`:
    Immediately after calling `self.release_reference(...)`, execute `reload_frame!();` so `reg_base`, `inst_ptr`, `inst_len`, and `pc` are synchronized with `self.register_stack`.

#### 2. Fixing `BUG-RT-02` (`try { ... } finally { ... }` Unwinding Rethrow in `exceptions.rs` & `fast.rs`)
* **Root Cause:** When `OpCode::Throw` unwinds into a `finally`-only handler (`handler.catch_type_symbol.is_none()` and `handler.finally_pc == Some(handler.handler_pc)`), the VM jumps to `finally_pc`, executes the `finally` block, and falls through to normal code instead of continuing to unwind the exception.
* **Implementation:**
  * In `crates/solix-compilation/src/phases/emitter/exceptions.rs`:
    1. Allocate a hidden local/scratch register `r_pending_exc` for the `try-finally` statement, initialized to `LoadNull` before `try_start_pc`.
    2. For the catch-all unwinding `ExceptionHandlerEntry` (`catch_type_symbol: None, handler_pc: finally_pc`), set `catch_reg: r_pending_exc`.
    3. In `fast.rs:1316`, always write `exc_val` into `self.register_stack[curr_reg_base + handler.catch_reg as usize] = exc_val;` (even when `handler.finally_pc.is_some()`!).
    4. At the end of emitting `finally_block` in `exceptions.rs:108`, emit a check: if `r_pending_exc` is not `null` (`JumpIfFalse` over `Throw r_pending_exc`), execute `OpCode::Throw r_pending_exc`!
    * **Result:** Normal execution and `catch`-handled execution leave `r_pending_exc == null` and continue past `finally`; uncaught exceptions store `exc_val` in `r_pending_exc`, run `finally`, and automatically re-execute `Throw r_pending_exc` to unwind to the caller frame!

#### 3. Fixing `BUG-RT-03` (`Value::I64` Corruption in `to_raw_u64` / `from_raw_u64`)
* **Root Cause:** A 64-bit word cannot simultaneously represent all $2^{64}$ `i64` bit patterns and all $2^{64}$ `f64` bit patterns via NaN-boxing alone without a tag or side-tag.
* **Implementation:**
  * **For `BlockTag::Tuple` (`AllocTuple`, `SetArray`, `GetTuple`, `TupleInstanceof`):**
    Allocate tuple slots as **16 bytes per element** (`[tag: u64 | payload: u64]`) or store a 1-byte `ValueTag` per element in the tuple payload (`8 + tuple_count * 9`), using `Value::to_tagged_bytes(&self) -> (u8, u64)` and `Value::from_tagged_bytes(tag: u8, raw: u64) -> Self`!
  * **For `BlockTag::Closure` (`PackClosure`, `UnpackCaptures`):**
    Store `(tag: u8, raw: u64)` (16-byte aligned cells: `8 + cap_count * 16`) in the closure heap block so captured `int64`, `float64`, `int32`, `bool`, `Ref`, and `FnPtr` values round-trip with **100% bit-exact fidelity** across all $2^{64}$ `i64` and `f64` values!
  * **For `BlockTag::ArrayRef` (`AllocArray`, `SetArray`, `GetArray`):**
    `AllocArray` already stores the element type tag in `self.heap.buffer[arr_offset + 7]` (`inst.flags()`)!
    In `GetArray` (`fast.rs:983`), inspect `elem_type_tag = self.heap.buffer[arr_offset + 7]`:
    * `1` (`int8`): `Value::I32(raw_word as i8 as i32)`
    * `2` (`int16`): `Value::I32(raw_word as i16 as i32)`
    * `3` (`int32`): `Value::I32(raw_word as i32)`
    * `4` (`int64`): `Value::I64(raw_word as i64)`
    * `5` (`uint8`) / `12` (`char`): `Value::I32(raw_word as u8 as i32)`
    * `6` (`uint16`): `Value::I32(raw_word as u16 as i32)`
    * `7` (`uint32`): `Value::I32(raw_word as u32 as i32)`
    * `8` (`uint64`): `Value::I64(raw_word as i64)`
    * `9` (`float32`) / `10` (`float64`): `Value::F64(f64::from_bits(raw_word))`
    * `11` (`bool`): `Value::Bool(raw_word != 0)`
    * `13` / `0x80..=0xFF` (Reference): `if raw_word == 0 { Value::Null } else { Value::Ref(raw_word as u32) }`
    And in `SetArray`, write `val.to_primitive_u64()`!

#### 4. Fixing `BUG-RT-04` (`OpCode::ArrayLength` on `BlockTag::String`)
* **Location:** `crates/solix-runtime/src/vm/fast.rs:924-934`
* **Implementation:**
  Check the block header tag at `arr_addr - 1`:
  ```rust
  let header = self.heap.read_header(arr_addr)?;
  let array_len = if header.tag == BlockTag::String {
      header.size
  } else {
      self.heap.read_u32(arr_addr)?
  };
  ```

#### 5. Fixing `BUG-RT-05` (`GetArray` and `SetArray` Element Stride Safety)
* **Location:** `crates/solix-runtime/src/vm/fast.rs:975-1025`
* **Implementation:**
  Dispatch read/write width based on `elem_size` (`1` $\to$ `read_u8`/`write_u8`, `2` $\to$ `read_u16`/`write_u16`, `4` $\to$ `read_u32`/`write_u32`, `8` $\to$ `read_u64`/`write_u64`).

#### 6. Fixing `BUG-RT-06` (`CastCheck`, `CallInterface`, and `TupleInstanceof` Multi-Binary VTable Relocation)
* **Location:** `crates/solix-runtime/src/context.rs:196-217`
* **Implementation:**
  When `vtable_offset > 0` in `RuntimeContext::new`:
  * Also relocate `OpCode::CastCheck` when `inst.flags() == 0` (`old_target + vtable_offset as u16`).
  * Also relocate `OpCode::CallInterface` when `(inst.call_target() >> 8) > 0`:
    ```rust
    let target = inst.call_target();
    let old_iface_id = target >> 8;
    let slot = target & 0xFF;
    let new_target = ((old_iface_id + vtable_offset as u16) << 8) | slot;
    ```
  * Also relocate `OpCode::TupleInstanceof` when `(inst.flags() & 0x80) != 0`:
    ```rust
    let old_vtable_id = inst.flags() & 0x7F;
    let new_flags = 0x80 | ((old_vtable_id + vtable_offset as u8) & 0x7F);
    ```

#### 7. Fixing `BUG-RT-07`, `BUG-RT-08`, and `BUG-RT-09`
* **`BUG-RT-07` (`engine.rs:151`):** Remove `eprintln!("DTOR ERROR: ...")` and propagate `destructor_result?` cleanly (cleaning up `self.active_destructors.remove(&addr)` before returning `Err`).
* **`BUG-RT-08` (`ffi.rs:321, 349`):** In `solix_vm_get_field_i32` / `solix_vm_set_field_i32`, if `offset < 8`, look up the object's `vtable_id` and use `class.field_types` sorted by offset to resolve the exact byte offset of the $N$-th field!
* **`BUG-RT-09` (`heap.rs:212`):** In `allocate_from_free_list`, update `header.size` or store the actual `allocated_block_size - HEADER_SIZE` so `verify_clean_memory` scans exact block boundaries.

---

### 2.2 Phase 2: Compiler & Emitter Miscompilation Fixes (`solix-compilation`)

#### 1. Fixing `BUG-COMP-01` & `BUG-COMP-02` (`%=` Modulo Assignment and 64-bit Compound Assignment / Increment)
* **Location:** `crates/solix-compilation/src/phases/emitter/emitter.rs:978-1450`
* **Implementation:**
  * Create a helper `select_compound_opcode(&self, operator: TokenKind, operand_type: &Type) -> OpCode` that maps:
    * `OperatorPlusAssign`: `AddF64` (float), `AddI64` (`int64`/`uint64`), `AddI32` (default)
    * `OperatorMinusAssign`: `SubF64`, `SubI64`, `SubI32`
    * `OperatorMultiplyAssign`: `MulF64`, `MulI64`, `MulI32`
    * `OperatorDivideAssign`: `DivF64`, `DivI64`, `DivI32`
    * `OperatorModuloAssign`: `ModI64` (`int64`/`uint64`), `ModI32` (default)
  * Use `select_compound_opcode` in all 4 targets of `emit_assignment` (local variable, static variable, object field, and array element), and only use `AddI32Imm`/`SubI32Imm` fast-path when `operand_type` is a 32-bit or narrower integer!
  * In `emit_unary` (`++` and `--`), check `self.query_type(operand.span())`:
    * If `int64`/`uint64`, emit `LoadI64 1` + `AddI64`/`SubI64`.
    * If `float32`/`float64`, emit `LoadF64 1.0` + `AddF64`/`SubF64`.
    * Also support `Expression::ArrayAccess` (`arr[i]++` and `++arr[i]`).

#### 2. Fixing `BUG-COMP-03` (Default Parameter Values at Call Sites)
* **Location:** `crates/solix-compilation/src/phases/binder/passes/pass3_type_checking.rs` & `emitter.rs`
* **Implementation:**
  * Store default parameter expressions (`Vec<Option<Expression>>`) on method/function metadata in `SymbolEntry` (or `CompilationContext`).
  * During `pass3_type_checking.rs` (or an AST normalization pass at the end of `Pass 3`), when a call site provides $K$ arguments to a function/method with $N$ parameters ($K < N$) and parameters $K \dots N-1$ all have `default_value: Some(default_expr)`, validate the $K$ provided arguments and automatically append the cloned `default_expr` nodes to the call site's `arguments` vector (and evaluate their node types)!
  * **Result:** Both the type checker and the bytecode emitter see the full $N$ arguments with zero special-casing needed in `Emitter`!

#### 3. Fixing `BUG-COMP-04` (`BytecodeOptimizerPhase` Instruction Analysis)
* **Location:** `crates/solix-compilation/src/phases/optimizer/bytecode_optimizer.rs:321-485`
* **Implementation:**
  * Add `OpCode::CallExtern` to `can_change_instruction_destination`, `rewrite_instruction_destination`, `instruction_reads_register`, and `count_register_writes`.
  * Add `OpCode::ConvIToF` and `OpCode::ConvFToI` to `can_change_instruction_destination` and `instruction_reads_register`.
  * Move `OpCode::LoadNull` to the Format B group in `can_change_instruction_destination` and `rewrite_instruction_destination`, and add `OpCode::LoadI64`, `OpCode::LoadF64`, `OpCode::LoadBool`, and `OpCode::AllocObjectExtern` to the Format B group.

#### 4. Fixing `BUG-COMP-05` (`ExportManifest` & `manifest_to_source` Completeness for Interfaces, Enums, and Operators)
* **Location:** `crates/solix-common/src/bytecode/manifest.rs`, `crates/solix-compilation/src/manifest.rs`, and `crates/solix-compilation/src/phases/emitter/pipeline.rs`
* **Implementation:**
  * Add `pub exported_enums: Vec<ExportedEnum>` (with `name`, `modifiers`, `variants: Vec<(String, Option<i64>)>`) to `ExportManifest` (with `#[serde(default)]`).
  * In `pipeline.rs`:
    * Match `Declaration::Interface(iface)` and push `ExportedClass { is_interface: true, ... }` into `exported_classes`.
    * Match `Declaration::Enum(enm)` and push `ExportedEnum` into `exported_enums`.
    * Match `ClassMember::Operator(op)` inside `Declaration::Class` and export operator methods.
  * In `manifest_to_source` (`manifest.rs`):
    * Synthesize `public enum <Name> { ... }` for each `ExportedEnum` and `public <ret> operator<op>(...)` for exported operators.

#### 5. Fixing `BUG-COMP-06` (`super.method(...)` Binding and Emission)
* **Location:** `pass3_type_checking.rs` & `emitter.rs`
* **Implementation:**
  * In `pass3_type_checking.rs` (`Expression::Identifier` and `Expression::MemberAccess`):
    * When `object` is `Expression::Identifier("super")`:
      * Verify `self.current_class` is `Some(cls_id)` and `!self.is_static_method`.
      * Verify the current class has a `parent_symbol` (`base_class_id`); if not, emit **`E0228` (`InvalidSuperCall`)** (`Keyword 'super' cannot be used in a class with no superclass`).
      * Resolve `member_name` starting from `base_class_id`, and record `base_class_id`'s `Type::Class` on `object.span()`.
  * In `emitter.rs` (`Expression::MethodCall`):
    * When `object` in `callee` (`MemberAccess`) is `"super"`, resolve the target method in the superclass (`BaseClass::method_name`) and emit `OpCode::CallDirect` passing `this` (`r0`) as argument `0`!

#### 6. Fixing `BUG-COMP-07` (`string + string` Concatenation)
* **Location:** `crates/solix-runtime/src/native/builtins.rs` & `crates/solix-compilation/src/phases/emitter/emitter.rs`
* **Implementation:**
  * Register a built-in native helper `__builtin_str_concat` in `register_all_builtins` (`builtins.rs`) that concatenates two `Value`s (using `format_vm_value(vm, &args[0]) + &format_vm_value(vm, &args[1])`) and allocates a new `BlockTag::String` on `vm.heap`.
  * In `emitter.rs` (`emit_binary`), when `operator == TokenKind::OperatorPlus` and either `left_type` or `right_type` is `std.string`, emit a 2-argument windowed `OpCode::CallSymbol` targeting `"__builtin_str_concat"`!

#### 7. Fixing `BUG-COMP-09`, `BUG-COMP-10`, `BUG-COMP-11`, and `BUG-COMP-12`
* **`BUG-COMP-09` (`control_flow.rs:369`):** In `emit_switch_statement`, select `EqString`, `EqI64`, or `EqI32` based on `self.query_type(switch_stmt.discriminant.span())`.
* **`BUG-COMP-10` (`ast_inliner.rs:665`):** Allocate synthetic inliner identifiers in a dedicated `<inliner_synthetic>` buffer at the end of `context.sources` instead of mutating `context.sources[0]`, and compute `source_bytes` in `pipeline.rs` excluding `<inliner_synthetic>`.
* **`BUG-COMP-11` (Generic Method Monomorphization for `default(T)` / `sizeof(T)`):** When a generic method `fn<T>(...)` is called with concrete type arguments (either explicit `Utils.convert<int32>(55)` or deduced `Deduce.identity(123)`), clone and specialize the method AST for `<T = int32>` (replacing `T` with `int32` in its body and rewriting the call site to target `Utils::convert<int32>`), so `default(T)` and `sizeof(T)` inside generic methods emit the exact specialized type!
* **`BUG-COMP-12` (Generic Type Bounds `<T : Bound>`):** In `monomorphize_class` and `match_method_signature`, verify that each concrete `type_arg` is assignable to every bound in `generic_param.bounds`; if not, emit **`E0264`** (`GenericTypeConstraintViolation`).

---

### 2.3 Phase 3: Lexer, Parser & Semantic Binder Fixes (`solix-compilation`)

1. **`BUG-PARSE-01` (Underscores in Numeric Literals — `parser.rs:1728`):**
   In `parse_numeric_literal`:
   ```rust
   let raw = self.get_lexeme(span);
   let stripped;
   let raw = if raw.contains('_') {
       stripped = raw.replace('_', "");
       stripped.as_str()
   } else {
       raw
   };
   ```
   This immediately fixes `1_000`, `0xDEAD_BEEF`, `0b1111_0000`, and `3.141_592` across all numeric radices!
2. **`BUG-PARSE-02` (Transitive `extern import` Worklist — `standard.rs:78`):**
   Use a `usize` cursor `let mut cursor = 0; while cursor < context.ast.len() { ... cursor += 1; }` in `ParserPhase::run` so any new source files added by `resolve_extern_import` are also scanned for transitive `extern import` statements.
3. **`BUG-PARSE-03` (All Non-`const` Modifiers on Local Variables — `parser.rs:943`):**
   In `parse_statement_in_block`, check for any non-`const` declaration modifier (`public`, `protected`, `private`, `internal`, `static`, `inline`, `native`, `virtual`, `override`, `weak`, `abstract`) before a local variable declaration, consume all consecutive modifiers, and emit `E0117` (`ModifierOnLocalVariable`).
4. **`BUG-BIND-01` (Root Facade Resolution When Caller File Has a `package` Statement — `symbols.rs` & `pass3_type_checking.rs`):**
   * In `pass1_symbol_discovery.rs`: when a class has the same name as its enclosing package (e.g. `package raylib; public class raylib` or `package math; public class Math` case-insensitively), or when `extern import pkg.*;` is used, allow `pkg.member` (`MemberAccess`) to resolve to either `pkg::member` (free function in package `pkg`) or `pkg::Class::member` (static method on class `pkg` inside package `pkg`).
   * In `symbols.rs:248`: allow step 6 (unqualified unique package fallback) to also check packages explicitly imported via `import pkg.*;` or `extern import pkg.*;` regardless of whether the caller file has its own `package` declaration!

---

### 2.4 Phase 4: Variadic Function Templates (`<args...>`) (`TESTS.md` Cases 55.5 & 55.6)

To achieve **100% implementation of all 55+ suites in `TESTS.md`**:
* **Syntax (Already Parsed by `parser.rs`):**
  ```solix
  int32 sum_all<args...>(int32 first, args...) {
      int32 total = first;
      for (int32 index = 0; i < args.length; i++) {
          total = total + (int32)args[i];
      }
      return total;
  }
  ```
* **Binder & Emitter Implementation:**
  1. **In `pass1_symbol_discovery.rs`, `pass2_type_hierarchy.rs`, and `pass3_type_checking.rs`:**
     * When a `MethodDeclaration` has a trailing parameter with `is_variadic == true` (or a generic parameter with `is_variadic == true`):
     * Bind the variadic parameter (`args`) inside the function scope with type `Type::Primitive(PrimitiveKind::TuplePrimitive)`.
     * Mark the function's `SymbolEntry` (or `MethodDeclaration`) as variadic with `fixed_param_count = parameters.len() - 1`.
  2. **At Call Sites (`pass3_type_checking.rs` & `emitter.rs`):**
     * When calling a variadic function/method with $M \ge \text{fixed\_param\_count}$ arguments:
       * Validate the first $\text{fixed\_param\_count}$ arguments against their declared parameter types.
       * Bundle the remaining $M - \text{fixed\_param\_count}$ trailing arguments into an `Expression::TupleCreation` (even if $0$ trailing arguments are passed, allocating an empty `tuple` of length `0`!).
     * Update `ParserState::parse_primary` (`parser.rs:1590`) so internal variadic bundling can construct a 0-element `AllocTuple(0)` while direct user `tuple()` syntax still reports `E0187` (`EmptyTupleExpression`).
  3. **Update `TESTS.md`:**
     * Transition **Case 55.5** (`Function Template Lowering with <args...>`) and **Case 55.6** (`Automatic Call-Site Argument Bundling`) in `crates/solix-tests/TESTS.md` from `[NOT IMPLEMENTED]` to `[IMPLEMENTED]` and add both tests to `test_tuple_primitive.rs`!

---

### 2.5 Phase 5: Package Manager, CLI & Cross-Platform Cleanup (`solix-client` & `solix-common`)

1. **`BUG-CLI-01` (`build_and_archive_tests.rs:281`):**
   Add a cross-platform helper `host_cdylib_filename("rust_bridge")` returning `"rust_bridge.dll"` on Windows, `"librust_bridge.dylib"` on macOS, and `"librust_bridge.so"` on Linux.
2. **`BUG-CLI-02` (`commands/run.rs` & `commands/compile.rs`):**
   Replace the manual `std::env::temp_dir().join(...)` directories in `execute_run` and `execute_compile` with `tempfile::TempDir::new()` held in a `Vec<TempDir>` for the lifetime of `execute_run` / `execute_compile`, guaranteeing automatic cleanup on exit!
3. **`BUG-CLI-03` (`project.rs:327`):**
   In `SlxarcArchive::extract_and_load`, when inspecting entries under `lib/`, if the entry is inside a platform subdirectory `lib/<subdir>/<filename>`, only extract it if `<subdir>` matches the current host OS and architecture (`format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)` or `host`).

---

### 2.6 Phase 6: Test Suite Deduplication & New Conformance Tests (`solix-tests`)

1. **Deduplicate `solix-runtime/tests/`:**
   Keep the unit-specific tests (`unit_dynamic_linking.rs`, `unit_telemetry.rs`, `unit_verifier.rs`, `unit_dependency_solver.rs`) in `crates/solix-runtime/tests/` and remove the 11 duplicated `unit_*.rs` files that are already in `crates/solix-tests/tests/runtime/test_*.rs`.
2. **Upgrade Fake/Weak Tests in `solix-tests`:**
   * Update `test_case_1_5_generic_function_returning_default` in `test_default_expression.rs` to test the actual generic static method `Helpers.getDefault<int32>()` and `Helpers.getDefault<bool>()` as specified in `TESTS.md`.
   * Tighten loose substring assertions in `control_flow/` and `expressions/` to assert exact diagnostic codes (`E01XX`, `E02XX`).
3. **Add New Regression Suite (`crates/solix-tests/tests/compilation/integration/test_audit_regressions.rs`):**
   Add end-to-end positive and negative tests verifying every bug fixed in Phases 1–5:
   * `test_regression_modulo_compound_assignment_i32_and_i64` (`BUG-COMP-01`)
   * `test_regression_int64_compound_assignments_and_increments` (`BUG-COMP-01`, `BUG-COMP-02`)
   * `test_regression_int64_large_positive_and_negative_in_tuples_arrays_closures` (`BUG-RT-03`)
   * `test_regression_try_finally_without_catch_propagates_exception_after_finally` (`BUG-RT-02`)
   * `test_regression_destructor_stack_growth_does_not_invalidate_reg_base` (`BUG-RT-01`)
   * `test_regression_string_length_and_string_concatenation_plus` (`BUG-RT-04`, `BUG-COMP-07`)
   * `test_regression_multi_binary_cast_check_and_interface_call_relocation` (`BUG-RT-06`)
   * `test_regression_default_parameter_arguments_injected_at_call_site` (`BUG-COMP-03`)
   * `test_regression_super_method_invocation_in_derived_class` (`BUG-COMP-06`)
   * `test_regression_numeric_literals_with_underscores_all_radices` (`BUG-PARSE-01`)
   * `test_regression_generic_type_bounds_enforced_positive_and_negative` (`BUG-COMP-12`)
   * `test_regression_export_manifest_interfaces_enums_and_operators_round_trip` (`BUG-COMP-05`)
   * `test_case_55_5_and_55_6_variadic_function_templates` (`Suite 55`)

---

## 3. Phased Execution Roadmap

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Runtime Memory Safety & Unwinding Fixes (`solix-runtime`)         │
│          └─► Fixes BUG-RT-01 .. BUG-RT-09                                  │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Compiler, Optimizer & Emitter Fixes (`solix-compilation`)         │
│          └─► Fixes BUG-COMP-01 .. BUG-COMP-12                              │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Parser, Lexer & Binder Fixes (`solix-compilation`)                │
│          └─► Fixes BUG-PARSE-01 .. BUG-PARSE-03 & BUG-BIND-01              │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Variadic Function Templates `<args...>` (`Suite 55.5 & 55.6`)     │
│          └─► 100% of TESTS.md marked [IMPLEMENTED]                         │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: CLI TempDir RAII & Cross-Platform Archive Fixes (`solix-client`)  │
│          └─► Fixes BUG-CLI-01 .. BUG-CLI-03                                │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Test Suite Deduplication & 35+ Audit Regression Conformance Tests │
│          └─► 100% pass across workspace + zero clippy warnings             │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Zero Dangling Pointers (`BUG-RT-01`)** | `reload_frame!()` after all destructor & native calls | `cargo test -p solix-tests` | [x] Verified |
| **Unwinding `try-finally` (`BUG-RT-02`)** | Uncaught exceptions run `finally` AND propagate to outer `catch` | `test_audit_regressions.rs` | [x] Verified |
| **Lossless 64-Bit `I64` (`BUG-RT-03`)** | `i64` values outside `i32` range preserved in tuples, arrays, closures | `test_audit_regressions.rs` | [x] Verified |
| **Compound `%=` & 64-Bit Ops (`BUG-COMP-01/02`)** | `%=` emits `ModI32`/`ModI64`; `int64` `+=`/`++` preserve 64-bit values | `test_audit_regressions.rs` | [x] Verified |
| **Default Parameters & `super.fn()` (`BUG-COMP-03/06`)** | Omitted default args injected; `super.method()` dispatches to base class | `test_audit_regressions.rs` | [x] Verified |
| **Underscore Literals (`BUG-PARSE-01`)** | `1_000`, `0xDEAD_BEEF`, `0b1111_0000` compile & execute cleanly | `test_audit_regressions.rs` | [x] Verified |
| **Variadic Templates (`Cases 55.5 & 55.6`)** | `sum_all<args...>` and `count_args<args...>` pass and marked `[IMPLEMENTED]` | `test_tuple_primitive.rs` | [x] Verified |
| **ExportManifest Interfaces & Enums (`BUG-COMP-05`)** | Interfaces, enums, and operators round-trip through `manifest_to_source` | `test_audit_regressions.rs` | [x] Verified |
| **Clippy & Formatting Clean** | Zero warnings across all workspace crates | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Verified |