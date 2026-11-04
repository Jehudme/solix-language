# Milestone 015: Compiler & Runtime Stabilization, Diagnostic Precision & Exhaustive Conformance

## Document Metadata
* **Milestone ID:** MS-015
* **Target Version:** `v0.1.0-alpha.15`
* **Milestone Branch:** `milestone/015-stabilization-and-diagnostic-precision`
* **Status:** ACTIVE
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-15
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `crates/solix-tests/TESTS.md`

---

## 1. Executive Summary & Forensic Codebase Audit

Before optimizing the virtual machine for raw execution speed (now deferred to **Milestone 016**), an exhaustive, line-by-line forensic audit of all **104 `TokenKind` variants**, **57 keywords**, and **13 modifiers** across `solix-common`, `solix-compilation`, `solix-runtime`, and `solix-tests` revealed **24 systemic compiler bypasses, modifier validation holes, parser diagnostic anti-patterns, and missing test suite implementations**.

Most shockingly, invalid programs such as:
```solix
void main(tretertertreter[] args) {
    println(args);
    fn(args);
}
void fn(fdsfds args) {
    println("wow it really run");
}
```
or:
```solix
native weak const override class BrokenClass {
    virtual abstract inline int32 bad_field = 10;
    static virtual abstract BrokenClass() {}
}
```
currently compile with **zero errors**!

### 1.1 Forensic Root-Cause Analysis: All 24 Systemic Compiler & Runtime Bypasses

| Bug ID | Exact File & Line Location | What the Code Currently Does (The Bug) | Catastrophic Consequence |
| :--- | :--- | :--- | :--- |
| **BUG-01** | `pass3_type_checking.rs:761-790`<br>`pass1_symbol_discovery.rs:365-404`<br>`pass2_type_hierarchy.rs:288-330` | `visit_method_declaration`, `visit_constructor_declaration`, and `visit_operator_declaration` call `resolve_type_annotation` on return types and parameters, which returns `Type::Unresolved("fdsfds")` or `Type::Array { element_type: Unresolved("tretertertreter") }`, **and never checks if the resolved type contains `Type::Unresolved`!** | Functions, methods, constructors, and operators can declare completely fake return types and parameter types (`void fn(fdsfds args)`) and compile cleanly! |
| **BUG-02** | `pass2_type_hierarchy.rs:242-286`<br>`pass1_symbol_discovery.rs:413-444` | Class fields and top-level global variables resolve `field_type` via `resolve_type_annotation`, get `Type::Unresolved(...)`, and store it in `SymbolEntry` **without checking for `Type::Unresolved`!** | Class fields (`class A { fdsfds x; }`) and global variables with fake types compile without error! |
| **BUG-03** | `pass3_type_checking.rs:845`<br>`types.rs:106` | `Type::Array`, `Type::Tuple`, `Type::FunctionPointer`, and `Type::Class { type_arguments }` wrap inner `Type::Unresolved` inside composite types, and `visit_variable_declaration` only checks `if let Type::Unresolved(name) = &declared_type` at the top level! | Local variables like `tretertertreter[] x;`, `tuple<fdsfds, int32> t;`, or `void(*)(fake_type) fn;` bypass `E0201` completely! |
| **BUG-04** | `pass3_type_checking.rs:424-441` | In `Expression::MethodCall`, when `callee` is a top-level free function (`Expression::Identifier`), Pass 3 looks up `function_id`, extracts `return_type`, **and NEVER validates `arguments.len()` or `evaluated_arg_types` against `parameter_types`!** | You can pass `tretertertreter[]` into `fn(fdsfds args)`, or pass 5 strings into `fn(int32 a)`, and the compiler accepts it silently! |
| **BUG-05** | `pass3_type_checking.rs:412` | When `Expression::MethodCall` is called on a `Type::Interface`, `Type::FunctionPointer`, or closure variable, Pass 3 returns `*return_type` **without checking argument count or argument types against `parameter_types`!** | Calling an interface method, function pointer, or lambda with wrong argument types or wrong arity compiles with zero errors! |
| **BUG-06** | `pass2_type_hierarchy.rs:451`<br>`pass3_type_checking.rs:485` | Interfaces (`implements Foo`) and generic type arguments (`Box<FakeType>`) are resolved via `resolve_type_annotation`, and if `Foo` is `Type::Unresolved` or a `Class`, `if let Type::Interface` simply skips it! | `class A implements NonExistentInterface {}` or `class A implements ConcreteClass {}` compiles with zero errors! |
| **BUG-07** | `parser.rs:947`<br>(`parse_ternary`) | When parsing `println?(args);`, `parse_ternary` consumes `?`, parses `(args)` as `then_branch`, and then checks `if self.cursor.match_token(TokenKind::PunctuationColon).is_none()` -> **returns `ParsingError::ExpectedClosingDelimiter`!** | Any stray `?` or incomplete ternary `a ? b` reports `"unclosed delimiter — Add matching ')', '}', or ']'"` instead of `"Expected ':' in ternary conditional expression"`! |
| **BUG-08** | `crates/solix-common/src/diagnostics/*.rs` | Every single `#[error("... at {span:?}")]` in `LexingError`, `ParsingError`, and `BindingError` formats `miette::SourceSpan`'s raw Rust `Debug` struct (`at SourceSpan { offset: SourceOffset(144), length: 1 }`) into the user-facing headline message! | Every compiler error prints ugly internal `SourceSpan { offset: SourceOffset(...), length: ... }` noise on top of `miette`'s graphical source snippet! |
| **BUG-09** | `parser.rs` (27 call sites!) | `ParsingError::ExpectedClosingDelimiter` (`E0104`) is used as a lazy catch-all in **27 different places** in `parser.rs`—including when expecting **opening** `(`, expecting `:`, expecting `>`, expecting `)`, expecting `]`, and expecting `}`! | Whenever the parser gets confused, it blindly outputs `E0104: Expected closing delimiter` with a generic message! |
| **BUG-10** | `pass3_type_checking.rs:804-820` | In `visit_constructor_declaration`, `super(args)` checks `has_super`, evaluates `arg`, **and never checks `super_args` count or types against the superclass constructors!** | Subclasses can call `super("wrong", 1, 2, 3)` with arbitrary bogus arguments and compile cleanly! |
| **BUG-11** | `pass3_type_checking.rs` (Destructor) | `visit_destructor_declaration` is not overridden in `TypeCheckingVisitor`, so `current_method_return_type` is `None`. Void `return;` inside a destructor fails with `ReturnOutsideFunction`, and `this` member access is not properly validated. | Destructor body type-checking is broken for early `return;`. |
| **BUG-12** | `solix-compilation` & `solix-runtime` Entry Point Validation | Neither the compiler nor `VirtualMachine::run_internal` validates that `main` has a canonical parameter type (`char[][]` or `string[]`) and return type (`void` or `int32`), nor does the compiler detect **ambiguous `main` entry points across multiple files** (`TESTS.md` Suite 54.2) or reject `static` on free functions (`TESTS.md` Suite 53.2)! | `void main(tretertertreter[] args)` runs and receives CLI arguments as `<ref 0x00000008>`! |
| **BUG-13** | `parser.rs:285-315` (`parse_modifiers`) | `parse_modifiers` collects any sequence of the 13 modifiers (`public`, `protected`, `private`, `internal`, `static`, `inline`, `native`, `const`, `virtual`, `override`, `weak`, `abstract`, `extern`) and **never validates which modifiers are legal on classes vs. interfaces vs. enums vs. fields vs. methods vs. constructors vs. destructors vs. operators!** | `native class A {}`, `weak class B {}`, `virtual int32 x;`, `abstract int32 y;`, `static ~A() {}`, `virtual A() {}`, `extern enum E { X }`, `native int32 z;` all compile without error! |
| **BUG-14** | `pass2_type_hierarchy.rs:334-366` | `Modifier::Override` and `Modifier::Virtual` are never enforced! A derived class can mark a method `override` when **no superclass method with that name exists**, can override a **non-virtual** method, or can change the return type of an overridden method without triggering `E0207`! | `override` and `virtual` keywords are currently decorative no-ops in Pass 2! |
| **BUG-15** | `pass2_type_hierarchy.rs:288` | Declaring an `abstract` method inside a **concrete (non-abstract) class** (`class Concrete { public abstract void foo(); }`) is accepted during parsing and Pass 2, and then emits a confusing `UnimplementedInterfaceMethod` error on the class itself instead of `AbstractMethodInConcreteClass`! | Misleading error message when placing `abstract` methods in non-abstract classes. |
| **BUG-16** | `pass1_symbol_discovery.rs` & `pass3_type_checking.rs` | `native` functions/methods are never validated for FFI signature legality (e.g., `native` with generic type parameters `<T>`, `native` on constructors/fields/classes/enums, or `native` combined with `abstract`, `virtual`, `override`, `inline`, `extern`). | Illegal `native` declarations compile silently and fail only at runtime or produce broken bytecode! |
| **BUG-17** | `pass1_symbol_discovery.rs` & `parser.rs` | `extern` is checked for body (`E0137`), initializer (`E0154`), local scope (`E0155`), and `native`/`inline`/`abstract` (`E0156`), **but is NOT rejected on `enum` declarations, `~destructor` declarations, `operator` declarations, or combined with `private` / `virtual` / `override` / `weak`!** | `extern private void foo();`, `extern enum E { A }`, or `extern virtual void bar();` compile silently! |
| **BUG-18** | `parser.rs:845` (`parse_switch_statement`) | A `switch` statement allows **multiple `default:` branches**, silently overwriting the first `default_case` with the second one! | Duplicate `default:` labels in `switch` are silently ignored and dead-coded! |
| **BUG-19** | `pass3_type_checking.rs:285` (`MemberAccess`) & `type_resolver.rs:30` | In `Expression::MemberAccess`, if `target_type` is not a Class, Enum, or Array/Tuple `.length` (for example, `(42).foo` or `true.bar` or `my_interface.unknown_field`), the function falls through to line 342 and **returns `Type::Primitive(PrimitiveKind::Int32)` with ZERO errors!** | Accessing `.anything` on an `int32`, `bool`, `float64`, or `interface` field silently succeeds and returns `int32`! |
| **BUG-20** | `pass3_type_checking.rs:344` (`MethodCall` on `Interface`) | Calling a method on an `Interface` variable (`IWorker w; w.nonExistentMethod(1, 2, 3);`) **is not checked in `Expression::MethodCall` at all!** Line 356 only checks `if let Type::Class { .. } = obj_type`, so calling any fake method or passing wrong arguments on a `Type::Interface` variable falls through to line 442 and returns `void` with **zero errors**! | You can call `w.completelyFakeMethod("abc", 123)` on any interface variable and it compiles with 0 errors! |
| **BUG-21** | `tokens.rs` & `lexer.rs` | `OpCode` defines `Bor` (`\|`), `Bxor` (`^`), `Shl` (`<<`), and `Shr` (`>>`), and `emitter.rs` references bitwise operations, **but `lexer.rs` has no tokens for `\|`, `^`, `<<`, or `>>`** (treating single `\|` and `^` as `E0000: UnknownToken` and `<<`/`>>` as two `<` or `>` tokens!). | Bitwise OR, XOR, and shifts cannot even be lexed or parsed! |
| **BUG-22** | `pass2_type_hierarchy.rs:413` (`EnumVariant` explicit values) | In `enum Color { RED = "hello", GREEN = 10, BLUE = 10 }`, the binder **never type-checks `variant.explicit_value`** (allowing strings/booleans as enum values!), **never evaluates explicit integer values** (Emitter always uses `0, 1, 2` index instead of `10, 20`!), and **never checks for duplicate enum values**! | `Color::GREEN = 10` in `master_all_statements.slx` actually compiles to `1` instead of `10`, and `enum E { A = "str" }` compiles without error! |
| **BUG-23** | `pass3_type_checking.rs` (`Parameter::default_value`) | Parameter default values (`void foo(int32 x = "wrong")`) are parsed into `Parameter.default_value`, **but Pass 3 never type-checks `default_value` against `param_type`, and Emitter never emits default arguments when omitted at call sites!** | Default parameter values can have completely wrong types (`int32 x = "hello"`) and compile with zero errors! |
| **BUG-24** | `pass2_type_hierarchy.rs` & `pass3_type_checking.rs` (`GenericParameter::bounds`) | Generic type bounds (`class Box<T : IPrintable>`) are parsed into `GenericParameter.bounds`, **but are never resolved or checked during Pass 1/2/3 or `monomorphize_class`!** | You can write `class Box<T : FakeInterface>` or instantiate `Box<int32>` violating `T : IPrintable` with zero errors! |

---

## 2. Complete 104-Token & 13-Modifier Audit Matrix

To guarantee **zero overlooked keywords or tokens**, every `Modifier` and `TokenKind` has been audited against every declaration and statement context.

### 2.1 The 13 Modifiers Legality Matrix

Every declaration must validate its `modifiers: Vec<Modifier>` against this exact matrix before binding:

| Modifier | `class` | `interface` | `enum` | Class `field` | Global `field` | Class `method` | Free `fn` | `constructor` | `~destructor` | `operator` | Local `var` |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`public`** | YES | YES | YES | YES | **NO (`E0175`)** | YES | YES | YES | YES (optional) | YES | **NO (`E0117`)** |
| **`protected`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | YES | **NO (`E0175`)** | YES | **NO (`E0199`)** | YES | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`private`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | YES | **NO (`E0175`)** | YES | **NO (`E0199`)** | YES | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`internal`** | YES | YES | YES | YES | **NO (`E0175`)** | YES | YES | YES | **NO (`E0130`)** | YES | **NO (`E0117`)** |
| **`static`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | YES | YES (implicit) | YES | **NO (`E0174`)** | **NO (`E0273`)** | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`inline`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | **NO (`E0274`)** | **NO (`E0274`)** | YES | YES | YES | **NO (`E0130`)** | YES | **NO (`E0117`)** |
| **`native`** | **NO (`E0275`)** | **NO (`E0275`)** | **NO (`E0275`)** | **NO (`E0275`)** | **NO (`E0275`)** | YES (no body) | YES (no body) | **NO (`E0275`)** | **NO (`E0275`)** | **NO (`E0275`)** | **NO (`E0117`)** |
| **`const`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | YES | YES | **NO (`E0276`)** | **NO (`E0276`)** | **NO (`E0273`)** | **NO (`E0130`)** | **NO (`E0200`)** | YES |
| **`virtual`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | **NO (`E0274`)** | **NO (`E0274`)** | YES (non-static) | **NO (`E0266`)** | **NO (`E0273`)** | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`override`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | **NO (`E0274`)** | **NO (`E0274`)** | YES (non-static) | **NO (`E0266`)** | **NO (`E0273`)** | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`weak`** | **NO (`E0196`)** | **NO (`E0197`)** | **NO (`E0198`)** | YES (ref only) | **NO (`E0274`)** | **NO (`E0276`)** | **NO (`E0276`)** | **NO (`E0273`)** | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`abstract`** | YES | **NO (`E0197`)** | **NO (`E0198`)** | **NO (`E0274`)** | **NO (`E0274`)** | YES (in abstract cls) | **NO (`E0199`)** | **NO (`E0273`)** | **NO (`E0130`)** | **NO (`E0200`)** | **NO (`E0117`)** |
| **`extern`** | YES | YES | **NO (`E0277`)** | YES (no init) | YES (no init) | YES (no body) | YES (no body) | YES (no body) | **NO (`E0277`)** | **NO (`E0277`)** | **NO (`E0155`)** |

#### Forbidden Modifier Combinations on the Same Declaration:
1. **Duplicate Modifiers (`E0172`):** Any modifier repeated more than once (`public public`, `static static`, `const const`).
2. **Conflicting Visibility (`E0173`):** More than one of `{public, protected, private, internal}`.
3. **`extern` Conflicts (`E0156`):** `extern` combined with `{native, inline, abstract, private, virtual, override, weak}`.
4. **`native` Conflicts (`E0278`):** `native` combined with `{abstract, virtual, override, inline, extern, const, weak}` or generic type parameters `<T>` (`E0279`).
5. **`abstract` Conflicts (`E0280`):** `abstract` combined with `{static, private, inline, native, extern, virtual, final}`.
6. **`static` Conflicts (`E0266`):** `static` combined with `{virtual, override, abstract, weak}`.
7. **Interface Method Modifiers (`E0281`):** Interface methods cannot be `{private, protected, static, native, inline, virtual, override, const, weak}`.

---

## 3. Exhaustive Diagnostic Error Code Specification (`E0000` – `E0295`)

### 3.1 Lexer Diagnostic Codes (`solix-common::diagnostics::lexing`)

| Error Code | Variant Name | Diagnostic Code | Trigger Condition & Remediation |
| :--- | :--- | :--- | :--- |
| **`E0000`** | `UnknownToken` | `solix::lexer::unknown_token` | Unrecognized byte (e.g. `@`, `#`, `$`, `` ` ``). |
| **`E0011`** | `UnterminatedStringLiteral` | `solix::lexer::unterminated_string` | String literal `"` reaches newline or EOF without closing quote. |
| **`E0012`** | `UnterminatedCharacterLiteral` | `solix::lexer::unterminated_char` | Character literal `'` reaches newline or EOF without closing quote. |
| **`E0013`** | `UnterminatedBlockComment` | `solix::lexer::unterminated_comment` | Multi-line comment `/*` reaches EOF without `*/`. |
| **`E0014`** | `MalformedHexLiteral` | `solix::lexer::malformed_hex` | Hexadecimal prefix `0x` not followed by valid hex digits. |
| **`E0015`** | `MalformedBinaryLiteral` | `solix::lexer::malformed_binary` | Binary prefix `0b` not followed by valid binary digits. |

### 3.2 Parser Diagnostic Codes (`solix-common::diagnostics::parsing`)

| Error Code | Variant Name | Diagnostic Message (Clean, Zero `SourceSpan` Debug Noise) |
| :--- | :--- | :--- |
| **`E0101`** | `ExpectedSemicolon` | Expected semicolon ';' after statement |
| **`E0102`** | `ExpectedIdentifier` | Expected identifier name |
| **`E0103`** | `ExpectedExpression` | Expected expression |
| **`E0104`** | `ExpectedCloseParen` | Expected closing parenthesis ')' |
| **`E0105`** | `ExpectedTypeName` | Expected type name |
| **`E0106`** | `InvalidAssignmentTarget` | Invalid assignment target |
| **`E0107`** | `InvalidIncrementTarget` | Invalid operand for increment or decrement operator |
| **`E0108`** | `ArrayAllocationRequiresSize` | Array allocation requires size expression |
| **`E0109`** | `ExpectedLambdaArrow` | Expected '=>' arrow in lambda expression |
| **`E0110`** | `ExpectedTypeInCast` | Expected type name in cast expression |
| **`E0111`** | `PackageStatementNotFirst` | 'package' statement must be the very first statement in the file |
| **`E0112`** | `DuplicatePackageStatement` | Only one 'package' statement is allowed per file |
| **`E0113`** | `ImportAfterDeclaration` | Import statements must appear before class, interface, or enum declarations |
| **`E0114`** | `ClassInLocalScope` | Classes cannot be declared inside a function or method body |
| **`E0115`** | `EnumInLocalScope` | Enums cannot be declared inside a function or method body |
| **`E0116`** | `InterfaceInLocalScope` | Interfaces cannot be declared inside a function or method body |
| **`E0117`** | `ModifierOnLocalVariable` | Access or member modifiers are not allowed on local variables |
| **`E0118`** | `OperatorOutsideClass` | Operator overloads can only be declared inside a class body |
| **`E0119`** | `ConstructorOutsideClass` | Constructors can only be declared inside a class body |
| **`E0120`** | `SolitaryDeclarationInBranchOrLoop` | Variable declarations are not allowed as immediate solitary branch or loop statements without a block |
| **`E0121`** | `CaseLabelNotConstant` | Case label must be a constant literal or enum variant |
| **`E0122`** | `StatementBeforeCaseInSwitch` | Statement without a preceding 'case' or 'default' label in switch |
| **`E0123`** | `MethodInsideMethod` | Methods cannot be declared inside another method |
| **`E0124`** | `ReturnOutsideFunction` | 'return' statement outside of function or method body |
| **`E0125`** | `InterfaceDestructor` | Interfaces cannot declare destructors |
| **`E0126`** | `InterfaceField` | Interfaces must not declare instance fields |
| **`E0127`** | `InterfaceMethodWithBody` | Interface methods cannot have a body |
| **`E0128`** | `StatementDirectlyInClass` | Executable statements or bare blocks are not allowed directly in class body |
| **`E0129`** | `MultipleDestructorsInClass` | Class cannot declare multiple destructors |
| **`E0130`** | `InvalidDestructorModifier` | Destructors cannot have static, virtual, override, abstract, or visibility modifiers |
| **`E0131`** | `DestructorNameMismatch` | Destructor name does not match enclosing class name |
| **`E0132`** | `DestructorParameters` | Destructors cannot accept parameters |
| **`E0133`** | `ConstructorNameMismatch` | Constructor name does not match enclosing class name |
| **`E0134`** | `ConstructorReturnType` | Constructors must not specify a return type |
| **`E0135`** | `InvalidOperatorOverload` | Operator cannot be overloaded |
| **`E0136`** | `VariadicParameterNotLast` | Variadic parameter must be the last parameter in parameter list |
| **`E0137`** | `ExternWithBody` | Extern function or method cannot have a body implementation |
| **`E0138`** | `AbstractOrNativeMethodWithBody` | Abstract or native method cannot have a body |
| **`E0139`** | `ExpectedBlock` | Expected block enclosed in braces '{ ... }' |
| **`E0140`** | `TryMissingCatchOrFinally` | 'try' statement must have at least one 'catch' or 'finally' clause |
| **`E0141`** | `MalformedCatchParameter` | Malformed catch parameter clause |
| **`E0142`** | `MalformedSwitch` | Malformed switch statement condition |
| **`E0143`** | `MalformedForLoop` | Malformed for-loop statement |
| **`E0144`** | `MalformedWhileCondition` | Malformed while-loop condition |
| **`E0145`** | `MalformedDoWhile` | Malformed do-while loop statement |
| **`E0146`** | `MalformedAlias` | Malformed alias statement |
| **`E0147`** | `UnexpectedToken` | Unexpected token |
| **`E0150`** | `IntegerLiteralOverflow` | Integer literal exceeds 64-bit capacity |
| **`E0151`** | `InvalidEscapeSequence` | Invalid escape sequence in literal |
| **`E0152`** | `EmptyCharacterLiteral` | Empty character literal |
| **`E0153`** | `MultiCharacterLiteral` | Character literal contains multiple characters |
| **`E0154`** | `ExternWithInitializer` | Extern variable declaration cannot have an initializer |
| **`E0155`** | `ExternInsideFunction` | Extern modifier is not allowed inside a local function or block |
| **`E0156`** | `InvalidExternModifier` | Extern modifier cannot be combined with '{modifier}' |
| **`E0160`** | `ExpectedOpenParen` | Expected opening parenthesis '(' |
| **`E0161`** | `ExpectedCloseBrace` | Expected closing curly brace '}' |
| **`E0162`** | `ExpectedOpenBrace` | Expected opening curly brace '{' |
| **`E0163`** | `ExpectedCloseBracket` | Expected closing square bracket ']' |
| **`E0164`** | `ExpectedOpenBracket` | Expected opening square bracket '[' |
| **`E0165`** | `ExpectedCloseAngle` | Expected closing angle bracket '>' in generic parameter list |
| **`E0166`** | `ExpectedColonInTernary` | Expected ':' in ternary conditional expression |
| **`E0167`** | `ExpectedColonInCase` | Expected ':' after switch 'case' or 'default' label |
| **`E0168`** | `ExpectedColonAfterSuper` | Expected 'super(...)' after ':' in constructor header |
| **`E0169`** | `ExpectedCommaInCast` | Expected comma ',' separating expression and target type in 'cast(expr, Type)' |
| **`E0170`** | `ExpectedConstructorCallOrSize` | Expected constructor arguments '(...)' or array size '[...]' after 'new Type' |
| **`E0171`** | `UnexpectedQuestionOperator` | Unexpected '?' operator; incomplete ternary expression |
| **`E0172`** | `DuplicateModifierOnDeclaration` | Duplicate modifier '{modifier}' on declaration |
| **`E0173`** | `ConflictingVisibilityModifiers` | Conflicting visibility modifiers '{first}' and '{second}' |
| **`E0174`** | `StaticOnFreeFunction` | Modifier 'static' is not allowed on top-level free functions |
| **`E0175`** | `StandaloneFieldWithAccessModifier` | Top-level global variables cannot declare class access modifiers ('{modifier}') |
| **`E0176`** | `MalformedFunctionPointerType` | Malformed function pointer type signature |
| **`E0177`** | `ExpectedCatchOrFinallyBlock` | Expected block '{ ... }' after catch or finally |
| **`E0178`** | `ExpectedIdentifierInParameter` | Expected parameter name identifier in parameter list |
| **`E0179`** | `ExpectedParameterList` | Expected parameter list enclosed in '(' ... ')' |
| **`E0180`** | `ExpectedInterfaceList` | Expected interface type name after 'implements' |
| **`E0181`** | `ExpectedSuperclassList` | Expected base class type name after 'extends' |
| **`E0182`** | `ExpectedEnumBrace` | Expected '{' after enum name |
| **`E0183`** | `ExpectedSwitchBrace` | Expected '{' after switch expression |
| **`E0184`** | `DuplicateDefaultInSwitch` | Switch statement cannot contain multiple 'default' branches |
| **`E0185`** | `ModifierOnModuleStatement` | Modifiers are not allowed on 'package', 'import', or 'alias' statements |
| **`E0186`** | `ExpectedExpressionInIndex` | Expected array index expression between brackets '[...]' |
| **`E0187`** | `EmptyTupleExpression` | Tuple expressions must contain at least one element |
| **`E0188`** | `MalformedTypeBound` | Malformed generic constraint bound after ':' |
| **`E0189`** | `ExpectedConditionInIf` | Expected conditional expression inside 'if (...)' |
| **`E0190`** | `ExpectedConditionInWhile` | Expected conditional expression inside 'while (...)' |
| **`E0191`** | `EmptyEnumDeclaration` | Enum declaration must contain at least one variant |
| **`E0192`** | `UnexpectedSemicolon` | Unexpected ';' |
| **`E0193`** | `MissingOperatorInExpression` | Missing binary operator between expressions |
| **`E0194`** | `MalformedConstructorChaining` | Expected 'super(...)' after ':' in constructor |
| **`E0195`** | `InvalidTopLevelToken` | Unexpected token at top-level compilation unit |
| **`E0196`** | `InvalidClassModifier` | Modifier '{modifier}' is not allowed on a class declaration |
| **`E0197`** | `InvalidInterfaceModifier` | Modifier '{modifier}' is not allowed on an interface declaration |
| **`E0198`** | `InvalidEnumModifier` | Modifier '{modifier}' is not allowed on an enum declaration |
| **`E0199`** | `InvalidFreeFunctionModifier` | Modifier '{modifier}' is not allowed on a top-level free function |
| **`E0200`** | `InvalidOperatorModifier` | Modifier '{modifier}' is not allowed on an operator overload declaration |

### 3.3 Semantic & Binder Diagnostic Codes (`solix-common::diagnostics::binding`)

| Error Code | Variant Name | Trigger Condition & Description |
| :--- | :--- | :--- |
| **`E0201`** | `UndefinedSymbol` | Unresolved identifier or unknown type in expression/statement. |
| **`E0202`** | `DuplicateSymbol` | Identifier already declared in the same lexical or nominal scope. |
| **`E0203`** | `TypeMismatch` | Assigned or computed expression type incompatible with expected type. |
| **`E0204`** | `CyclicInheritance` | Class extends itself directly or transitively (`A extends B extends A`). |
| **`E0205`** | `InvalidBaseClass` | Class extends an interface, enum, primitive, or final type. |
| **`E0206`** | `UnimplementedInterfaceMethod` | Concrete class fails to implement all required interface/abstract methods. |
| **`E0207`** | `MethodOverrideMismatch` | Method override signature (parameters or return type) does not match superclass. |
| **`E0208`** | `PrivateAccessViolation` | Accessing private class member from outside declaring class. |
| **`E0209`** | `ProtectedAccessViolation` | Accessing protected class member from non-subclass. |
| **`E0210`** | `CannotAssignToConst` | Attempting to mutate or reassign a `const` field or local variable. |
| **`E0211`** | `ReturnOutsideFunction` | Return statement outside function, method, or closure. |
| **`E0212`** | `MissingReturn` | Non-void function/method has code path that does not return a value. |
| **`E0213`** | `ReturnTypeMismatch` | Returned expression type incompatible with declared return type. |
| **`E0214`** | `OverloadAmbiguity` | Multiple overloaded methods match argument list with equal precision. |
| **`E0215`** | `NoMatchingOverload` | No method, constructor, or operator overload matches provided arguments. |
| **`E0216`** | `GenericArityMismatch` | Generic instantiation passes wrong number of type arguments. |
| **`E0217`** | `CannotInstantiateAbstractType` | Attempting to instantiate an abstract class or interface directly (`new Base()`). |
| **`E0218`** | `SuperConstructorMustBeFirst` | `: super(...)` constructor chaining is not the first statement. |
| **`E0219`** | `BreakOrContinueOutsideLoop` | `break` or `continue` outside a `for`, `while`, or `do-while` loop. |
| **`E0220`** | `InvalidOperatorOverload` | Operator overload has wrong arity or un-overloadable operator token. |
| **`E0221`** | `NonIndexableType` | Array subscript `[i]` applied to non-array, non-tuple type. |
| **`E0222`** | `NonCallableType` | Function call `(...)` applied to non-callable value. |
| **`E0223`** | `CannotInferType` | Type inference failed to deduce concrete type. |
| **`E0224`** | `AmbiguousImport` | Wildcard imports from multiple packages define colliding symbol names. |
| **`E0225`** | `NonConstantCaseValue` | Switch `case` expression is not a compile-time constant or enum variant. |
| **`E0226`** | `StaticAccessViaDot` | Static member or enum variant accessed via `.` instead of `::`. |
| **`E0227`** | `InstanceAccessViaScopeResolution` | Instance field or method accessed via `::` instead of `.`. |
| **`E0228`** | `InvalidSuperCall` | Calling `super(...)` in class without a superclass. |
| **`E0229`** | `InvalidWeakModifier` | `weak` modifier applied to non-reference primitive or value type. |
| **`E0230`** | `CircularAlias` | Circular type alias chain detected (`alias A = B; alias B = A;`). |
| **`E0231`** | `UnresolvedAliasTarget` | Target type of alias statement cannot be resolved. |
| **`E0232`** | `CannotResolveImport` | Imported package or module path does not exist. |
| **`E0233`** | `UnresolvedParameterType` | Method, function, constructor, or lambda parameter has an unknown/unresolved type. |
| **`E0234`** | `UnresolvedReturnType` | Method or function declared with an unknown/unresolved return type. |
| **`E0235`** | `UnresolvedFieldType` | Class field or global variable declared with an unknown/unresolved type. |
| **`E0236`** | `UnresolvedInterfaceType` | Class implements or interface extends an unresolved or non-interface type. |
| **`E0237`** | `UnresolvedGenericArgument` | Generic type instantiation contains an unresolved type argument. |
| **`E0238`** | `FunctionArgumentCountMismatch` | Free function or callable invoked with incorrect number of arguments. |
| **`E0239`** | `FunctionArgumentTypeMismatch` | Argument passed to free function or callable is incompatible with parameter type. |
| **`E0240`** | `FieldInitializerTypeMismatch` | Class field or global variable initializer incompatible with declared type. |
| **`E0241`** | `MemberAccessOnPrimitiveOrNonObject` | Accessing `.member` on a primitive, void, or non-composite type. |
| **`E0242`** | `SuperConstructorArgumentMismatch` | Chained `super(...)` call arguments do not match any superclass constructor. |
| **`E0243`** | `InvalidEntryPointSignature` | `main` function has invalid parameters (not `()` or `(string[])` / `(char[][])`) or return (not `void` or `int32`). |
| **`E0244`** | `AmbiguousEntryPoint` | Multiple `main` entry points found across compilation units without disambiguation. |
| **`E0245`** | `UnreachableCatchClause` | `catch` clause is unreachable because preceding catch catches superclass. |
| **`E0246`** | `DuplicateCatchClause` | Multiple `catch` clauses catch the exact same exception type. |
| **`E0247`** | `CatchNonExceptionType` | `catch` clause specifies a non-class or non-exception type. |
| **`E0248`** | `ThrowNonExceptionType` | `throw` expression is a primitive, void, uninstantiated class name, or non-object. |
| **`E0249`** | `DuplicateSwitchCaseValue` | Multiple `case` labels evaluate to the same constant value. |
| **`E0250`** | `SwitchCaseTypeMismatch` | `case` label type is incompatible with `switch` discriminant expression type. |
| **`E0251`** | `InvalidCastBetweenUnrelatedTypes` | Cast between unrelated classes or between primitive and class. |
| **`E0252`** | `CannotTakeAddressOfInstanceMethod` | Referencing an instance method as a bare function pointer without a closure. |
| **`E0253`** | `VoidVariableOrParameter` | Variable, field, or parameter declared with type `void`. |
| **`E0254`** | `DefaultOfVoidType` | `default(void)` is illegal in expressions. |
| **`E0255`** | `SizeofVoidOrUnresolvedType` | `sizeof(void)` or `sizeof(UnknownType)`. |
| **`E0256`** | `CannotIndexPrimitive` | Array subscript `[...]` applied to a scalar primitive (`int32`, `bool`, etc.). |
| **`E0257`** | `CannotCallNonCallable` | Parentheses `(...)` call applied to a non-callable variable. |
| **`E0258`** | `InvalidConditionType` | Condition expression in `if`, `while`, or `do-while` does not evaluate to `bool`. |
| **`E0259`** | `InvalidLogicalOperand` | Logical operator (`&&`, `\|\|`, `!`) applied to a non-boolean expression. |
| **`E0260`** | `NonConstantSwitchDiscriminant` | Switch discriminant evaluates to a non-integral and non-enum type. |
| **`E0261`** | `NonConstantCaseLabel` | Switch case label contains a dynamic variable or non-constant call. |
| **`E0262`** | `DuplicateEnumVariantValue` | Explicit enum variant integer values collide. |
| **`E0263`** | `InvalidArraySizeExpression` | Array allocation size expression is non-integer. |
| **`E0264`** | `GenericTypeConstraintViolation` | Generic argument fails to satisfy declared interface/class bound (`<T : Bound>`). |
| **`E0265`** | `AbstractMethodInConcreteClass` | Abstract method declared in non-abstract class. |
| **`E0266`** | `InvalidVirtualModifier` | `virtual` or `override` applied to static method, private method, or free function. |
| **`E0267`** | `SuperWithoutInheritance` | `super` member call or constructor call in a class with no base class. |
| **`E0268`** | `DestructorOnNonClass` | Destructor declared on enum, interface, or struct. |
| **`E0269`** | `DuplicateFieldInClass` | Two fields declared with the exact same identifier in the same class. |
| **`E0270`** | `RecursiveTypeAlias` | Type alias transitively refers to itself (`alias A = A[];`). |
| **`E0271`** | `UnusedVariableWarningOrError` | Unused variable detection in strict compilation mode. |
| **`E0272`** | `InvalidTupleIndex` | Tuple index out of range during compile-time constant indexing. |
| **`E0273`** | `InvalidConstructorModifier` | Constructor declared with illegal modifier (`static`, `virtual`, `override`, `abstract`, `const`, `weak`). |
| **`E0274`** | `InvalidFieldModifier` | Field declared with method-only modifier (`virtual`, `override`, `abstract`, `inline`). |
| **`E0275`** | `InvalidNativeDeclaration` | `native` modifier applied to class, interface, enum, field, constructor, destructor, or operator. |
| **`E0276`** | `InvalidMethodModifier` | Method or free function declared with field-only modifier (`const`, `weak`). |
| **`E0277`** | `InvalidExternTarget` | `extern` modifier applied to enum, destructor, or operator overload. |
| **`E0278`** | `ConflictingNativeModifier` | `native` combined with `abstract`, `virtual`, `override`, `inline`, or `extern`. |
| **`E0279`** | `GenericNativeFunction` | `native` function or method cannot declare generic type parameters `<T>`. |
| **`E0280`** | `ConflictingAbstractModifier` | `abstract` combined with `static`, `private`, `inline`, `native`, or `extern`. |
| **`E0281`** | `InvalidInterfaceMemberModifier` | Interface method declared with `private`, `protected`, `static`, `native`, `inline`, or `override`. |
| **`E0282`** | `OverrideWithoutBaseMethod` | Method marked `override` does not match any method in any superclass. |
| **`E0283`** | `OverrideNonVirtualMethod` | Method marked `override` targets a superclass method that is neither `virtual` nor `abstract`. |
| **`E0284`** | `MissingOverrideModifier` | Method hides or overrides a `virtual` / `abstract` superclass method without the `override` keyword. |
| **`E0285`** | `EnumVariantValueNotInteger` | Explicit enum variant value is not a compile-time integer constant (`enum E { A = "str" }`). |
| **`E0286`** | `ParameterDefaultValueTypeMismatch` | Parameter default value expression is incompatible with declared parameter type (`int32 x = "bad"`). |
| **`E0287`** | `MethodCallWithoutBody` | Calling an unimplemented method or `extern` / `native` method without a linked target. |
| **`E0288`** | `BitwiseOperandNotInteger` | Bitwise operator (`&`, `\|`, `^`, `~`, `<<`, `>>`) applied to non-integer type (e.g. `float64` or `string`). |

---

## 4. Audit of `TESTS.md`: Unimplemented & Fake-Passing Test Cases

An exhaustive cross-audit of `crates/solix-tests/TESTS.md` against `crates/solix-tests/tests/` revealed the following cases that are claimed `[IMPLEMENTED]` in `TESTS.md` but are **missing from the test files** or **fake-passing on loose substring assertions**:

1. **ImportStatement Case 3.5:** Cross-Platform Binary Path Resolution (`test_import_statement.rs` skips 3.5).
2. **ClassDeclaration Case 3.6:** Nested Class Declaration Inside Class (`Outer.Inner obj = new Outer.Inner(42)` — parser does not support nested classes inside `parse_class_member` yet!).
3. **DestructorDeclaration Case 3.8:** Exception Rewinding RAII across Multiple Call Frames (`test_destructor_declaration.rs` skips 3.8 and 3.9!).
4. **DestructorDeclaration Case 3.9:** Exception in Try Block RAII (`ScopedLock` destroyed before `catch`).
5. **DestructorDeclaration Case 4.4:** Destructor Name Mismatch (`~Other()` in `class Widget` — missing from `test_destructor_declaration.rs`).
6. **EnumDeclaration Case 4.4:** Enum Declared Inside Function Body (`enum LocalState { ON, OFF }` — missing from `test_enum_declaration.rs`).
7. **FieldDeclaration Case 3.7:** Deterministic Live Object Count Reclamation on Scope Exit.
8. **FieldDeclaration Case 3.10:** Segregated Free Lists and Adjacent Block Coalescing.
9. **FieldDeclaration Case 4.2:** Field Access on Null Reference (Runtime Fault).
10. **FieldDeclaration Case 4.5:** Incompatible Field Initializer Type (`int32 count = "invalid";` — `visit_field_declaration` in `pass3_type_checking.rs:754` evaluates `init` **and never checks if `init` is assignable to `field_type`!**).
11. **FieldDeclaration Case 4.8:** Standalone Field Declared at File Top Level with Access Modifier (`public int32 globalField = 42;`).
12. **InterfaceDeclaration Case 3.2:** Interface Hierarchy with Sub-Interface Extension (`interface AdvancedReader extends Reader` — `pass2_type_hierarchy.rs` ignores `iface_decl.super_interfaces`!).
13. **InterfaceDeclaration Case 3.4:** Nested Interface Declaration Inside Class.
14. **InterfaceDeclaration Case 3.7:** Sub-Interface Dynamic Dispatch and Cast.
15. **InterfaceDeclaration Case 3.8:** Class Inheritance with Interface Implementation Override.
16. **MethodDeclaration Case 4.4:** Incompatible Override Signature Return Type (`pass2_type_hierarchy.rs` only checks `existing_entry.symbol_type == method_symbol.symbol_type`; if return type differs, it treats it as a new overload instead of emitting `E0207: MethodOverrideMismatch`!).
17. **MethodDeclaration Case 4.7:** Nested Method Declaration Inside Another Method (missing from `test_method_declaration.rs`).
18. **OperatorDeclaration Case 4.5:** Operator Overload Declared Inside Method Body.
19. **OperatorDeclaration Case 4.6, 4.7, 4.8:** Package-Level Native Compatibility, Variadic Adapters, Value-Type Structural Equality.
20. **Generics Suite 48 Cases 48.4, 48.5, 48.6:** Generic Methods with Explicit Type Arguments (`Utils.convert<int32>(55)`) and Implicit Template Argument Deduction (`Deduce.identity(123)`).
21. **Generics Suite 48 Cases 48.11, 48.12:** Conflicting Template Deduction at Call Site & Unbound Type Parameter Identifier.
22. **Native Interop Suite 49 Cases 49.1 – 49.9:** Static/Instance native calls and shared library error handling.
23. **Suite 53 Cases 53.1 & 53.2:** Free Function Entry Point Without `static` & Rejection of `static` on Free Function Outside Class.
24. **Suite 54 Cases 54.1 & 54.2:** Single Unambiguous Entry Point & Ambiguous Entry Point Detection Across Modules.
25. **Suite 55 Cases 55.5, 55.6, 55.11, 55.12, 55.15:** Function Template Lowering with `<args...>`, Call-Site Argument Bundling, Tuple `TypeCastException`, Tuple `IndexOutOfBoundsException`.

---

## 5. Phased Implementation Plan

### Phase 1: Lexer Completeness, Diagnostic Formatting Cleanup & Granular Parser Error Codes (`solix-common` & `solix-compilation`)
- [ ] `[P0]` **Task 1.1:** Strip `at {span:?}` from all `#[error("...")]` strings in `lexing.rs`, `parsing.rs`, and `binding.rs`.
- [ ] `[P0]` **Task 1.2:** Add missing bitwise tokens (`OperatorBitwiseOr` `|`, `OperatorBitwiseXor` `^`, `OperatorShiftLeft` `<<`, `OperatorShiftRight` `>>`) to `TokenKind`, `lexer.rs`, `parser.rs`, `pass3_type_checking.rs`, and `emitter.rs`, and add `E0011`–`E0015` to `LexingError`.
- [ ] `[P0]` **Task 1.3:** Add `E0160` through `E0200` to `ParsingError` in `crates/solix-common/src/diagnostics/parsing.rs`.
- [ ] `[P0]` **Task 1.4:** Replace all 27 misused `ExpectedClosingDelimiter` calls in `crates/solix-compilation/src/phases/parser/parser.rs` with the exact specific delimiter/construct error (`ExpectedOpenParen`, `ExpectedCloseParen`, `ExpectedCloseBracket`, `ExpectedCloseAngle`, `ExpectedColonInTernary`, `UnexpectedQuestionOperator`, etc.).
- [ ] `[P0]` **Task 1.5:** Implement `validate_modifiers()` in `parser.rs` and `pass1`/`pass2` enforcing the complete **13-Modifier Legality Matrix** (Section 2.1) across classes, interfaces, enums, fields, methods, free functions, constructors, destructors, operators, local variables, and module statements (`E0172`–`E0175`, `E0185`, `E0196`–`E0200`, `E0273`–`E0281`).
- [ ] `[P0]` **Task 1.6:** Reject duplicate `default:` branches in `switch` (`E0184`), empty enums (`E0191`), and support nested `class` and `interface` declarations inside `parse_class_member`.

### Phase 2: Recursive Type Validation & Closing All 24 Semantic Binder Bypasses (`solix-common` & `solix-compilation`)
- [ ] `[P0]` **Task 2.1:** Add helper `Type::first_unresolved_name(&self) -> Option<&str>` in `crates/solix-common/src/types.rs` that recursively inspects `Type::Unresolved`, `Type::Array`, `Type::Tuple`, `Type::FunctionPointer`, and `Type::Class / Interface` `type_arguments`.
- [ ] `[P0]` **Task 2.2:** Add `E0233` through `E0288` to `BindingError` in `crates/solix-common/src/diagnostics/binding.rs`.
- [ ] `[P0]` **Task 2.3:** Validate all resolved types in Pass 1, Pass 2, and Pass 3:
  - Every function/method/constructor/operator/lambda parameter type **MUST** be checked via `first_unresolved_name()` -> emit `E0233` (`UnresolvedParameterType`) and reject `void` parameters with `E0253`.
  - Every parameter `default_value` **MUST** be type-checked against `param_type` -> emit `E0286` (`ParameterDefaultValueTypeMismatch`).
  - Every function/method/operator return type **MUST** be checked via `first_unresolved_name()` -> emit `E0234` (`UnresolvedReturnType`).
  - Every class field and global variable type **MUST** be checked via `first_unresolved_name()` -> emit `E0235` (`UnresolvedFieldType`).
  - Every local variable type **MUST** be checked recursively via `first_unresolved_name()` -> emit `E0201` / `E0237`.
  - Every `implements` clause on a class and `extends` clause on an interface **MUST** verify the target exists and is a `Type::Interface` -> emit `E0236`.
- [ ] `[P0]` **Task 2.4:** Enforce complete call-site argument validation in `pass3_type_checking.rs`:
  - Validate argument count (`E0238`) and argument type assignability (`E0239`) for **top-level free function calls** (`fn(args)`).
  - Validate method existence, argument count, and argument types for **interface method calls** (`Type::Interface`), **function pointer calls**, and **closure calls** (`BUG-05`, `BUG-20`).
  - Validate `super(...)` constructor call argument count and types against superclass constructors (`E0242`).
- [ ] `[P0]` **Task 2.5:** Enforce `virtual` / `override` / `abstract` rules in `pass2_type_hierarchy.rs`:
  - Reject `override` when no superclass method matches (`E0282`).
  - Reject `override` when the superclass method is not `virtual` or `abstract` (`E0283`).
  - Reject overriding a method with a mismatched return type (`E0207`).
  - Reject `abstract` methods inside concrete (non-abstract) classes (`E0265`).
- [ ] `[P0]` **Task 2.6:** Enforce enum explicit value evaluation & duplicate value checking (`E0262`, `E0285`), field initializer type checking (`E0240`), non-composite member access rejection (`E0241`), generic type bound checking (`E0264`), and `main` entry-point signature/ambiguity checking (`E0243`, `E0244`).

### Phase 3: Audit `TESTS.md` Tags & Implement All Missing `TESTS.md` Scenarios
- [ ] `[P1]` **Task 3.1:** Update `crates/solix-tests/TESTS.md` to tag all 25 currently unimplemented scenarios as `[NOT IMPLEMENTED]`.
- [ ] `[P1]` **Task 3.2:** Implement the missing language features required by `TESTS.md` (sub-interface inheritance method collection, nested classes/interfaces, generic static methods with explicit and deduced type arguments, destructor unwinding during exceptions, explicit enum variant values in emitter).
- [ ] `[P1]` **Task 3.3:** Write all missing `TESTS.md` test functions in `crates/solix-tests/tests/` and transition their tags in `TESTS.md` to `[IMPLEMENTED]` as each test passes.

### Phase 4: 250+ Granular Positive & Negative Conformance Tests (`crates/solix-tests`)
- [ ] `[P1]` **Task 4.1:** Upgrade `assert_compile_error` in `crates/solix-tests/src/common.rs` so tests assert **exact error codes** (`E00XX`, `E01XX`, `E02XX`) and verify that no error string contains `SourceSpan {`.
- [ ] `[P1]` **Task 4.2:** Add 250+ new positive and negative edge-case tests across `crates/solix-tests/tests/compilation/` and document every single one in `crates/solix-tests/TESTS.md`, covering every cell of the **13-Modifier Legality Matrix**, every `TokenKind` syntax error, and every semantic type check (`E0000`–`E0288`).

---

## 6. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Zero Silent Type Bypasses** | `void main(tretertertreter[] args)` & `void fn(fdsfds args)` rejected with `E0233` | `cargo test -p solix-tests` | [ ] Pending |
| **13-Modifier Matrix Enforced** | All illegal `native`, `extern`, `virtual`, `override`, `abstract`, `weak`, `const`, `static` combinations rejected | `cargo test -p solix-tests` | [ ] Pending |
| **Clean Diagnostic Messages** | Zero occurrences of `SourceSpan {` in any error message | Diagnostic formatting test | [ ] Pending |
| **Specific Delimiter Codes** | `E0104`, `E0160`–`E0200` tested individually | `cargo test --test parser_tests` | [ ] Pending |
| **100% `TESTS.md` Accuracy** | Every `[IMPLEMENTED]` case in `TESTS.md` has a real test in `solix-tests` | Automated `TESTS.md` audit | [ ] Pending |
| **250+ New Edge-Case Tests** | All new positive & negative tests pass with exact error code assertions | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy & Rustfmt Clean** | Zero warnings across workspace | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |