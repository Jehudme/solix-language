# Milestone 003: Recursive-Descent AST Parser & Grammar Engine

## Document Metadata
* **Milestone ID:** MS-003
* **Target Version:** `v0.1.0-alpha.3`
* **Milestone Branch:** `milestone/003-parser`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-10-25
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/specs/lexical.md` (SPEC-0001)
  * `docs/internals/specs/grammar.md` (SPEC-0002: Pending)

---

## 1. Executive Summary & Objectives

Milestone 003 implements the **Recursive-Descent Abstract Syntax Tree (AST) Parser and Grammar Engine** for the Solix language compiler in modern Rust (Edition 2024), completely superseding the legacy single-file C++ parser implementation (`core/src/processes/parser.cpp`, 2,101 lines) and monolithic AST node header (`core/src/utilities/statements.hpp`, 826 lines).

In the legacy C++ codebase, parsing suffered from substantial structural deficiencies:
1. **Monolithic AST Structure:** All 46 AST nodes were defined within a single 826-line header (`statements.hpp`), combining expressions, control flow, types, and declarations without logical separation.
2. **Untyped Heap Pointers & Mutations:** Nodes inherited from a polymorphic base `Node` struct containing 10+ mutable semantic analysis fields (`resolved_declaration`, `memory_index`, `mangled_name`, `instance_size`) intermingled with raw syntax fields, causing premature semantic coupling during parsing.
3. **Fragile Exception-Based Errors:** Syntax errors threw `ParseError : public std::runtime_error`, carrying ad-hoc string messages without machine-parseable error codes, preventing structured error recovery or multi-error accumulation.
4. **Manual Clone and Traversal Boilerplate:** The visitor pattern (`NodeVisitor`) required manual, error-prone `clone()` overrides and raw pointer parent link tracking (`parent = this`).

Milestone 003 establishes a **modular, strongly typed, zero-unsafe Rust AST engine**:
* **Group-Partitioned Node Hierarchy (`phases/parser/statements/`):** AST statements and expressions are partitioned into discrete, focused modules by category (`modules.rs`, `declarations.rs`, `control_flow.rs`, `expressions.rs`, and `types.rs`), cleanly decoupled from semantic analysis.
* **First-Class Visitor Pattern (`visitor.rs`):** An extensible `AstVisitor<R = ()>` trait equipped with default tree-walking implementations and `Visitable` trait bindings, enabling downstream compiler passes (type checking, AST lowerers, bytecoders, formatters, linters) to traverse syntax trees cleanly.
* **Checked Pratt / Recursive-Descent Expression Parsing:** Full operator precedence climbing handling all 25 Solix operators, ternary conditionals, lambda expressions, member calls, and chained indexing without recursion panics.
* **Deferred Literal Validation:** Accurate validation of string/char escape sequences (`\n`, `\xHH`), character count (`''` and `'ab'`), and numeric literal capacity (`> u64::MAX`) as mandated by the compiler architecture.
* **Resilient Error Recovery & Rich Diagnostics (`errors.rs`):** `ParsingError` enum deriving `thiserror::Error` and `miette::Diagnostic` with standardized codes (`E0101`–`E0130`), supporting panic-mode synchronization at statement delimiters (`;`, `}`) and declaration boundaries to report multiple syntax errors per compilation unit.

---

## 2. Scope & Crates Affected

* **`crates/solix-compilation`**:
  * `src/phases/parser/mod.rs`: Public API entrypoints (`parse`, `parse_source`), compilation metadata, and AST re-exports.
  * `src/phases/parser/parser.rs`: RecursiveDescentParser engine, `TokenCursor` lookahead, operator precedence tables, and synchronization recovery.
  * `src/phases/parser/visitor.rs`: `AstVisitor<R = ()>` and `Visitable` traits with default recursive walking dispatchers.
  * `src/phases/parser/errors.rs`: Hierarchical `ParsingError` enum with `E0101`–`E0130` diagnostic error codes and `miette` labels.
  * `src/phases/parser/statements/mod.rs`: Group re-exports and high-level `Statement`, `Declaration`, `Expression`, and `ProgramAst` enums.
  * `src/phases/parser/statements/modules.rs`: Module statement nodes (`PackageStatement`, `ImportStatement`, `AliasStatement`).
  * `src/phases/parser/statements/declarations.rs`: Class, Interface, Enum, Field, Method, Constructor, Destructor, Operator Overload, Generics, and Parameter AST nodes.
  * `src/phases/parser/statements/control_flow.rs`: Block, If-Else, While, Do-While, For, Switch-Case, Try-Catch-Finally, Throw, Return, Break, Continue, Variable Declaration, Expression Statement, Assert, and Exit AST nodes.
  * `src/phases/parser/statements/expressions.rs`: Binary, Unary, Assignment, Member Access, Method Call, Array Access, Array Creation, Array Literal, New Instance, Cast, Instanceof, Ternary, Lambda, Sizeof, Default, Tuple Creation, Identifier, and Literal expression nodes.
  * `src/phases/parser/statements/types.rs`: Complete type annotation model (`TypeAnnotation`: Primitives, Named Paths, Generic Arguments, Array Depth, Tuples, Variadic Packs, Function Pointers).
  * `src/CompilationContext.rs`: Multi-source AST storage (`ast: Vec<ProgramAst>`) and context-driven `parse(&mut context)`.
* **`crates/solix-compilation/tests/`**:
  * `tests/parser_tests.rs`: Comprehensive Tier 1 unit & inlined integration test suite.
  * `tests/fixtures/syntax/positive/*.slx`: Data-driven positive test programs covering all statement categories.
  * `tests/fixtures/syntax/negative/*.slx`: Consolidated negative fixtures verifying exact parser diagnostic codes (`E0101`–`E0130`).
* **`docs/internals/specs/`**:
  * `docs/internals/specs/grammar.md`: Formal EBNF Grammar Specification (SPEC-0002).

---

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)

- [x] `[P0]` **Dedicated Milestone Branch (`milestone/003-parser`):**
  Cut and protect `milestone/003-parser` from `master` adhering to GL-0002. All PRs implementing parser functionality must merge into this branch.

- [x] `[P0]` **Token Cursor & Lookahead Abstraction (`TokenCursor` in `parser.rs`):**
  Implement `TokenCursor<'tokens>` wrapping `&'tokens [Token]`:
  * Zero-copy navigation with `peek() -> Option<Token>`, `peek_next() -> Option<Token>`, `advance() -> Option<Token>`, `check(kind: TokenKind) -> bool`, and `match_token(kind: TokenKind) -> Option<Token>`.
  * Mandatory assertion `consume(expected: TokenKind, error: ParsingError) -> Result<Token, ParsingError>`.
  * `#![forbid(unsafe_code)]` with zero slice panics on unexpected EOF.
  * *Failure Mode if Omitted:* Out-of-bounds panics on malformed or truncated token streams.

- [x] `[P0]` **Group-Partitioned AST Node Architecture (`phases/parser/statements/`):**
  Organize all AST definitions into discrete files grouped by domain:
  * `modules.rs`: `PackageStatement`, `ImportStatement`, `AliasStatement`.
  * `declarations.rs`: Type declarations, member declarations, operator overloads, constructor/destructors.
  * `control_flow.rs`: Imperative statements, loops, exception handling, variable bindings.
  * `expressions.rs`: Precedence-aware expression hierarchy.
  * `types.rs`: Strongly typed type annotations including tuples and function pointers.
  * *Failure Mode if Omitted:* Monolithic header antipattern; massive compilation bottlenecks and circular dependencies.

- [x] `[P0]` **Extensible Visitor Pattern (`visitor.rs`):**
  Implement the full Visitor pattern with default tree traversal:
  * Define `AstVisitor<R = ()>` trait with visit hooks for all 46+ grammar structures (`visit_package_statement`, `visit_class_declaration`, `visit_if_statement`, `visit_binary_expression`, etc.).
  * Provide default walking logic in `AstVisitor` (e.g., `walk_class_declaration`, `walk_binary_expression`) allowing visitors to selectively override nodes of interest.
  * Implement `Visitable` trait for all AST structs and top-level enums (`Statement`, `Declaration`, `Expression`).
  * *Failure Mode if Omitted:* Subsequent semantic analysis (type checking, bytecode generation) requires invasive code duplication.

- [x] `[P0]` **Diagnostic Error Handling for Parser (`ParsingError` in `errors.rs`):**
  Implement hierarchical `ParsingError` deriving `thiserror::Error` and `miette::Diagnostic`:
  * Structured codes `E0101` through `E0153` with `SourceSpan` coordinates, descriptive messages, and actionable `#[help("...")]` annotations.
  * Standardized `error_code(&self) -> &'static str` and span resolution.
  * *Failure Mode if Omitted:* Opaque errors with no programmatic test assertions or compiler diagnostic linkage.

### 3.2 P1: High Priority (Milestone Core)

- [x] `[P1]` **Module Statements Parsing (`modules.rs`):**
  Port module statement parsing from C++ `parse_package_statement()`, `parse_import_statement()`, and `parse_alias_statement()`:
  * `package <path>;` (single statement permitted only at source file head).
  * `import <path> [as <alias>];` (imports preceding any type declarations).
  * `alias <NewName> = <OriginalPath>;` (type aliasing).
  * Enforce semicolon termination and identifier path validation.

- [x] `[P1]` **Type Declarations & Member Parsing (`declarations.rs`):**
  Port class, interface, enum, and member parsing:
  * Class declarations: `[modifiers] class <Name>[<Generics>] [extends <Super>] [implements <Interfaces>] { ... }`.
  * Interface declarations: `[modifiers] interface <Name>[<Generics>] [extends <Interfaces>] { ... }`.
  * Enum declarations: `[modifiers] enum <Name> { <Variant>[ = <Value>], ... }`.
  * Field declarations: `[modifiers] <Type> <name> [= <expr>];`.
  * Method declarations: `[modifiers] <Type> <name>[<Generics>](<params>) { <body> }`.
  * Constructors & Destructors: `[modifiers] <ClassName>(<params>) [: super(<args>)] { <body> }` and `~<ClassName>() { <body> }`.
  * Operator overloads: `[modifiers] operator <op>(<params>) -> <Type> { <body> }`.
  * Parameter lists: `(<Type> <name> [= <default>], ...)`.
  * Modifiers: `public`, `protected`, `private`, `internal`, `static`, `inline`, `native`, `const`, `virtual`, `override`, `weak`, `abstract`.

- [x] `[P1]` **Control Flow & Imperative Statement Parsing (`control_flow.rs`):**
  Port control flow statement parsing:
  * Block statements: `{ <stmt>* }`.
  * If-Else conditional: `if (<expr>) <stmt> [else <stmt>]`.
  * Loops: `while (<expr>) <stmt>`, `do <stmt> while (<expr>);`, and `for (<init>; <cond>; <step>) <stmt>`.
  * Pattern branching: `switch (<expr>) { case <expr>: <stmt>* default: <stmt>* }`.
  * Exceptions: `try <block> catch (<Type> <ident>) <block> [finally <block>]` and `throw <expr>;`.
  * Control transfer: `return [<expr>];`, `break;`, and `continue;`.
  * Variable bindings: `[const] <Type> <name> [= <expr>];`.
  * Expression statement: `<expr>;`.
  * Intrinsics & diagnostics: `assert(<expr> [, <message>]);` and `exit(<expr>);`.

- [x] `[P1]` **Precedence-Climbing Expression Parser (`expressions.rs`):**
  Implement Pratt / Precedence Climbing parser covering all 18 Solix expression types across 14 precedence levels:
  * Literals: integers, floats, characters, strings, booleans, `null`.
  * Identifiers and scoped names (`foo`, `foo::bar`, `pkg.Type`).
  * Unary operations: `!`, `-`, `+`, `~`, prefix `++`, `--`.
  * Binary operations: arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`), logical (`&&`, `||`), bitwise (`&`, `|`, `^`).
  * Assignment operations: `=`, `+=`, `-=`, `*=`, `/=`, `%=`.
  * Member access & Call: `obj.member`, `obj.method(args)`, `Type::static_member`.
  * Array operations: `arr[index]`, array literal `[a, b, c]`, array creation `new Type[size]`.
  * Object instantiation: `new Class(args)`.
  * Type inspection & conversion: `cast(expr, Type)`, `expr instanceof Type`, `sizeof(Type)`, `default(Type)`.
  * Ternary conditional: `<cond> ? <true_expr> : <false_expr>`.
  * Lambda expressions: `(params) => expr` and `(params) => { block }`.
  * Tuple expressions: `(a, b, c)`.

- [x] `[P1]` **Type Annotation Parser (`types.rs`):**
  Port `parse_type_info()` into a comprehensive `TypeAnnotation` model:
  * Primitive types (`void`, `bool`, `int8`..`int64`, `uint8`..`uint64`, `float32`, `float64`, `char`).
  * Named types with qualified paths (`std.collections.List`, `MyClass`).
  * Generic type arguments (`List<int32>`, `Map<string, User>`).
  * Multidimensional array syntax (`int32[]`, `string[][]`).
  * Strongly-typed tuples (`tuple<int32, string, bool>`).
  * First-class function pointer types: `return_type(*)(param1, param2)`.

- [x] `[P1]` **Deferred Literal Validation Engine:**
  Implement rigorous validation during AST literal node construction:
  * String/Char Escape Decoding: Validate and decode `\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`, and `\xHH`. Reject invalid escapes with `E0151` (`InvalidEscapeSequence`).
  * Character Literal Bounds: Validate exact character count. Reject empty `''` with `E0152` (`EmptyCharacterLiteral`) and multi-char `'ab'` with `E0153` (`MultiCharacterLiteral`).
  * Numeric Literal Bounds: Validate digit syntax (`0x`, `0b`) and reject integer literals exceeding `u64::MAX` with `E0150` (`IntegerLiteralOverflow`).

- [x] `[P1]` **CompilationContext Pipeline Integration:**
  Update `CompilationContext`:
  * Store parsed ASTs: `ast: Vec<ProgramAst>`.
  * Implement `parse(&mut context) -> Result<AstMetaData, Vec<ParsingError>>`.
  * Provide zero-copy access to source lexemes for AST identifier resolution.

- [x] `[P1]` **Data-Driven Syntax Fixtures:**
  * Positive Fixtures: Single master positive fixture testing every grammar construct, variation, generics, lambdas, tuples, function pointers, and modifiers (`tests/fixtures/syntax/positive/master_all_statements.slx`).
  * Negative Fixtures: Single master negative fixture testing exhaustive parser and grammar errors across standardized diagnostic error codes (`tests/fixtures/syntax/negative/master_invalid_syntax.slx`).

### 3.3 P2: Medium Priority (Ergonomics & Recovery)

- [x] `[P2]` **Panic-Mode Error Recovery & Synchronization:**
  Implement panic-mode recovery in `Parser`:
  * On encountering a syntax error, record the `ParsingError` into an accumulated error vector.
  * Advance the `TokenCursor` until reaching a synchronization boundary (semicolon `;`, closing brace `}`, or declaration keywords `class`, `interface`, `enum`, `package`, `import`).
  * Resume parsing the remainder of the compilation unit to report multiple syntax errors in a single compiler invocation.

- [x] `[P2]` **AST Visitor Traversal & Tree Walking:**
  Implement comprehensive default walker implementations for all AST declarations, statements, and expressions.

- [x] `[P2]` **Span Stitching & Exact Node Coordinates:**
  Ensure every AST node records an exact `Span` encompassing its entire source range from opening keyword/token to trailing delimiter/semicolon using `Span::combine(start, end)`, with `name_span` recorded for accurate LSP / IntelliSense support.

### 3.4 P3: Low Priority (Tooling & Benchmarking)

- [ ] `[P3]` **Parser Criterion Benchmark (`benches/parser_bench.rs`):**
  Create a baseline benchmark tracking parsing throughput across synthetic Solix source files.

- [ ] `[P3]` **CLI `--emit-ast` Subcommand Flag:**
  Add a flag to `solix compile --emit-ast` that formats and prints the parsed AST structure to standard output.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | [x] Passed |
| **Automated Test Suite** | 100% pass across all unit and integration tests | `cargo test --workspace --all-targets` | [x] Passed |
| **Clippy Static Analysis** | 0 warnings, 0 unwraps, strict pedantic pass | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Passed |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Passed |
| **Safety Invariants** | `#![forbid(unsafe_code)]` declared in all parser modules | Workspace source audit | [x] Passed |
| **Error Code Coverage** | 100% test coverage of parser error codes (`E0101`–`E0153`) | `cargo test --test parser_tests` | [x] Passed |
| **Visitor Coverage** | 100% of AST node variants covered by `AstVisitor` | Visitor trait audit & test | [x] Passed |
| **Living Documentation** | SPEC-0002 synchronized with implementation | Audit against GL-0010 | [x] Passed |

---

## 5. Architectural Blueprint & Legacy C++ Mapping

### 5.1 Architecture Pipeline Diagram

```
Token Stream (&[Token])
         │
         ▼
┌───────────────────────────────────────┐
│     TokenCursor<'tokens> (parser.rs)   │  <-- Checked lookahead, consume assertions
└──────────────────┬────────────────────┘
                   │ tokens
                   ▼
┌───────────────────────────────────────┐
│   RecursiveDescentParser (parser.rs)  │  <-- Precedence climbing, synchronization
├──────────────────┬────────────────────┤
│ statements/      │ statements/        │
│ • modules.rs     │ • control_flow.rs  │
│ • declarations.rs│ • expressions.rs   │
│ • types.rs       │                    │
└──────────────────┬────────────────────┘
                   │
                   ▼
┌───────────────────────────────────────┐
│           ProgramAst Root             │  <-- Strongly typed, memory safe
├───────────────────────────────────────┤
│ Vec<PackageStatement>                 │
│ Vec<ImportStatement>                  │
│ Vec<AliasStatement>                   │
│ Vec<Declaration>                      │
└──────────────────┬────────────────────┘
                   │ accept(visitor)
                   ▼
┌───────────────────────────────────────┐
│      AstVisitor<R> (visitor.rs)       │  <-- Traversal for type checking, codegen
└───────────────────────────────────────┘
```

### 5.2 Legacy C++ to Modern Rust Structural Mapping

| Legacy C++ (`solix/core`) | Modern Rust (`crates/solix-compilation`) | Key Architectural Improvement |
| :--- | :--- | :--- |
| `core/src/utilities/statements.hpp` (monolithic) | `src/phases/parser/statements/` (partitioned) | Clear domain separation: modules, declarations, control flow, expressions, types. |
| `struct Node` with mutable semantic fields | Pure syntax AST nodes; semantic fields moved to Binder phase | Pure syntax trees; zero premature semantic coupling. |
| `virtual void accept(NodeVisitor& v)` | `trait AstVisitor<R>` with default `walk_*` dispatch | Extensible generic return type `R`; default tree walking eliminating boilerplate. |
| `std::unique_ptr<Node> clone() const` | `#[derive(Clone, Debug, PartialEq)]` | Zero manual copy constructors; memory safe standard Rust cloning. |
| `Node* parent` raw pointers | Clean hierarchical ownership (`Box<Node>`, `Vec<Node>`) | `#![forbid(unsafe_code)]`; zero dangling parent pointer bugs. |
| `class ParseError : public std::runtime_error` | `enum ParsingError` deriving `Diagnostic` | Standardized `E01XX` error codes with `miette` source spans and actionable help. |
| Exception-based single error bail-out | Panic-mode recovery at statement & declaration boundaries | Accumulates multiple errors across the file without stopping at the first typo. |

---

## 6. Statement & Expression Group Specification Matrix

### 6.1 Module Statements (`statements/modules.rs`)

| AST Node | Trigger Token | BNF Syntax Schema | Fields & Metadata |
| :--- | :--- | :--- | :--- |
| **`PackageStatement`** | `package` | `package <path>;` | `span: Span`, `path: Vec<Span>` |
| **`ImportStatement`** | `import` | `import <path> [as <alias>];` | `span: Span`, `path: Vec<Span>`, `alias: Option<Span>` |
| **`AliasStatement`** | `alias` | `alias <Name> = <Target>;` | `span: Span`, `name: Span`, `target: TypeAnnotation` |

### 6.2 Declarations (`statements/declarations.rs`)

| AST Node | Trigger Token | BNF Syntax Schema | Fields & Metadata |
| :--- | :--- | :--- | :--- |
| **`ClassDeclaration`** | `class` | `[mods] class <Name>[<T>] [extends <S>] [implements <I>] { <members> }` | `span: Span`, `modifiers: Vec<Modifier>`, `name: Span`, `generics: Vec<GenericParam>`, `super_class: Option<TypeAnnotation>`, `interfaces: Vec<TypeAnnotation>`, `members: Vec<ClassMember>` |
| **`InterfaceDeclaration`** | `interface` | `[mods] interface <Name>[<T>] [extends <I>] { <methods> }` | `span: Span`, `modifiers: Vec<Modifier>`, `name: Span`, `generics: Vec<GenericParam>`, `super_interfaces: Vec<TypeAnnotation>`, `methods: Vec<MethodSignature>` |
| **`EnumDeclaration`** | `enum` | `[mods] enum <Name> { <variant>[ = <val>], ... }` | `span: Span`, `modifiers: Vec<Modifier>`, `name: Span`, `variants: Vec<EnumVariant>` |
| **`FieldDeclaration`** | Type name | `[mods] <Type> <name> [= <expr>];` | `span: Span`, `modifiers: Vec<Modifier>`, `field_type: TypeAnnotation`, `name: Span`, `initializer: Option<Expression>` |
| **`MethodDeclaration`** | Type name | `[mods] <Type> <name>[<T>](<params>) { <body> }` | `span: Span`, `modifiers: Vec<Modifier>`, `return_type: TypeAnnotation`, `name: Span`, `generics: Vec<GenericParam>`, `params: Vec<Parameter>`, `body: Option<BlockStatement>` |
| **`ConstructorDeclaration`** | Class name | `[mods] <Class>(<params>) [: super(<args>)] { <body> }` | `span: Span`, `modifiers: Vec<Modifier>`, `name: Span`, `params: Vec<Parameter>`, `super_call: Option<Vec<Expression>>`, `body: BlockStatement` |
| **`DestructorDeclaration`** | `~` | `~<Class>() { <body> }` | `span: Span`, `name: Span`, `body: BlockStatement` |
| **`OperatorDeclaration`** | `operator` | `[mods] operator <op>(<params>) -> <Type> { <body> }` | `span: Span`, `modifiers: Vec<Modifier>`, `operator_token: TokenKind`, `params: Vec<Parameter>`, `return_type: TypeAnnotation`, `body: BlockStatement` |

### 6.3 Control Flow & Imperative Statements (`statements/control_flow.rs`)

| AST Node | Trigger Token | BNF Syntax Schema | Fields & Metadata |
| :--- | :--- | :--- | :--- |
| **`BlockStatement`** | `{` | `{ <stmt>* }` | `span: Span`, `statements: Vec<Statement>` |
| **`IfStatement`** | `if` | `if (<expr>) <stmt> [else <stmt>]` | `span: Span`, `condition: Expression`, `then_branch: Box<Statement>`, `else_branch: Option<Box<Statement>>` |
| **`WhileStatement`** | `while` | `while (<expr>) <stmt>` | `span: Span`, `condition: Expression`, `body: Box<Statement>` |
| **`DoWhileStatement`** | `do` | `do <stmt> while (<expr>);` | `span: Span`, `body: Box<Statement>`, `condition: Expression` |
| **`ForStatement`** | `for` | `for (<init>; <cond>; <step>) <stmt>` | `span: Span`, `initializer: Option<Box<Statement>>`, `condition: Option<Expression>`, `update: Option<Expression>`, `body: Box<Statement>` |
| **`SwitchStatement`** | `switch` | `switch (<expr>) { case <expr>: <stmt>* default: <stmt>* }` | `span: Span`, `discriminant: Expression`, `cases: Vec<CaseClause>`, `default_case: Option<Vec<Statement>>` |
| **`TryStatement`** | `try` | `try <block> catch (<Type> <name>) <block> [finally <block>]` | `span: Span`, `try_block: BlockStatement`, `catch_clauses: Vec<CatchClause>`, `finally_block: Option<BlockStatement>` |
| **`ThrowStatement`** | `throw` | `throw <expr>;` | `span: Span`, `expression: Expression` |
| **`ReturnStatement`** | `return` | `return [<expr>];` | `span: Span`, `value: Option<Expression>` |
| **`BreakStatement`** | `break` | `break;` | `span: Span` |
| **`ContinueStatement`** | `continue` | `continue;` | `span: Span` |
| **`VariableDeclarationStatement`** | Type name | `[const] <Type> <name> [= <expr>];` | `span: Span`, `is_const: bool`, `var_type: TypeAnnotation`, `name: Span`, `initializer: Option<Expression>` |
| **`ExpressionStatement`** | Expression | `<expr>;` | `span: Span`, `expression: Expression` |
| **`AssertStatement`** | `assert` | `assert(<expr> [, <message>]);` | `span: Span`, `condition: Expression`, `message: Option<Expression>` |
| **`ExitStatement`** | `exit` | `exit(<expr>);` | `span: Span`, `code: Expression` |

### 6.4 Expressions (`statements/expressions.rs`)

| AST Node | Precedence Level | BNF Syntax Schema | Fields & Metadata |
| :--- | :--- | :--- | :--- |
| **`LiteralExpression`** | Primary (15) | `42`, `3.14`, `"hello"`, `'c'`, `true`, `null` | `span: Span`, `value: LiteralValue` |
| **`IdentifierExpression`** | Primary (15) | `<ident>` | `span: Span`, `name: Span` |
| **`ArrayLiteralExpression`** | Primary (15) | `[<expr>, ...]` | `span: Span`, `elements: Vec<Expression>` |
| **`TupleCreationExpression`** | Primary (15) | `tuple(<expr>, ...)` or `(<expr>, <expr>)` | `span: Span`, `elements: Vec<Expression>` |
| **`NewInstanceExpression`** | Primary (15) | `new <Type>(<args>)` | `span: Span`, `instance_type: TypeAnnotation`, `arguments: Vec<Expression>` |
| **`ArrayCreationExpression`** | Primary (15) | `new <Type>[<size>]` | `span: Span`, `element_type: TypeAnnotation`, `size: Box<Expression>` |
| **`LambdaExpression`** | Primary (15) | `(<params>) => <expr>` or `(<params>) => <block>` | `span: Span`, `params: Vec<Parameter>`, `body: LambdaBody` |
| **`MemberAccessExpression`** | Postfix (14) | `<expr>.<ident>` | `span: Span`, `object: Box<Expression>`, `member: Span` |
| **`MethodCallExpression`** | Postfix (14) | `<expr>(<args>)` | `span: Span`, `callee: Box<Expression>`, `arguments: Vec<Expression>` |
| **`ArrayAccessExpression`** | Postfix (14) | `<expr>[<index>]` | `span: Span`, `array: Box<Expression>`, `index: Box<Expression>` |
| **`UnaryExpression`** | Unary (13) | `!<expr>`, `-<expr>`, `+<expr>`, `~<expr>`, `++<expr>`, `--<expr>` | `span: Span`, `operator: TokenKind`, `operand: Box<Expression>`, `is_prefix: bool` |
| **`CastExpression`** | Cast (12) | `cast(<expr>, <Type>)` | `span: Span`, `expression: Box<Expression>`, `target_type: TypeAnnotation` |
| **`InstanceofExpression`** | Relational (9) | `<expr> instanceof <Type>` | `span: Span`, `expression: Box<Expression>`, `target_type: TypeAnnotation` |
| **`BinaryExpression`** | Multiplicative..Bitwise (3..11) | `<expr> <op> <expr>` | `span: Span`, `left: Box<Expression>`, `operator: TokenKind`, `right: Box<Expression>` |
| **`TernaryExpression`** | Conditional (2) | `<cond> ? <true_val> : <false_val>` | `span: Span`, `condition: Box<Expression>`, `then_branch: Box<Expression>`, `else_branch: Box<Expression>` |
| **`AssignmentExpression`** | Assignment (1) | `<target> <op>= <value>` | `span: Span`, `target: Box<Expression>`, `operator: TokenKind`, `value: Box<Expression>` |
| **`SizeofExpression`** | Primary (15) | `sizeof(<Type>)` | `span: Span`, `target_type: TypeAnnotation` |
| **`DefaultExpression`** | Primary (15) | `default(<Type>)` | `span: Span`, `target_type: TypeAnnotation` |

### 6.5 Type Annotations (`statements/types.rs`)

| Type Annotation Variant | BNF Syntax Schema | Example | Fields & Metadata |
| :--- | :--- | :--- | :--- |
| **`Primitive`** | `void \| bool \| int8..int64 \| float32..float64 \| char` | `int32` | `span: Span`, `kind: TokenKind` |
| **`Named`** | `<ident> (('.' \| '::') <ident>)*` | `std.collections.List` | `span: Span`, `path: Vec<Span>` |
| **`Generic`** | `<Type><<Type>, ...>` | `Map<string, User>` | `span: Span`, `base: Box<TypeAnnotation>`, `args: Vec<TypeAnnotation>` |
| **`Array`** | `<Type>[]` | `int32[][]` | `span: Span`, `element_type: Box<TypeAnnotation>`, `depth: usize` |
| **`Tuple`** | `tuple<<Type>, ...>` | `tuple<int32, string>` | `span: Span`, `elements: Vec<TypeAnnotation>` |
| **`FunctionPointer`** | `<RetType>(*)(<ParamType>, ...)` | `void(*)(int32, bool)` | `span: Span`, `return_type: Box<TypeAnnotation>`, `params: Vec<TypeAnnotation>` |

---

## 7. Diagnostic Error Code Specification Matrix

| Error Code | Diagnostic Variant | Trigger Condition | Recommended Help Message |
| :--- | :--- | :--- | :--- |
| **`E0101`** | `ExpectedSemicolon` | Statement missing closing `;` | `"Add a semicolon ';' to terminate the statement."` |
| **`E0102`** | `ExpectedIdentifier` | Grammar expected an identifier name | `"Provide a valid identifier name here."` |
| **`E0103`** | `ExpectedExpression` | Grammar expected an expression | `"Provide a valid expression (literal, variable, or calculation)."` |
| **`E0104`** | `ExpectedClosingDelimiter` | Missing matching `)`, `}`, or `]` | `"Add matching closing delimiter to balance the expression."` |
| **`E0105`** | `ExpectedTypeAnnotation` | Expected type name in declaration | `"Specify a valid type name (e.g. int32, string, void)."` |
| **`E0106`** | `InvalidAssignmentTarget` | Left-hand side of assignment is not an l-value | `"The left-hand side of an assignment must be a mutable variable, field, or array element."` |
| **`E0107`** | `UnexpectedToken` | Token violates statement grammar rule | `"Unexpected token; verify statement syntax."` |
| **`E0108`** | `PackageStatementNotFirst` | `package` statement appears after other declarations | `"The 'package' declaration must be the very first statement in the source file."` |
| **`E0109`** | `ImportStatementAfterDecl` | `import` statement appears after type declarations | `"All 'import' statements must appear before class, interface, or enum declarations."` |
| **`E0110`** | `DuplicateModifier` | Same access modifier repeated on declaration | `"Remove redundant or duplicate modifier."` |
| **`E0111`** | `ConflictingModifiers` | Incompatible modifiers (e.g. `public` and `private`) | `"Specify only one visibility modifier (public, protected, private, internal)."` |
| **`E0112`** | `ExpectedBlock` | Missing `{ ... }` block in control flow or method | `"Provide a block enclosed in curly braces '{ ... }'."` |
| **`E0113`** | `MissingCatchOrFinally` | `try` statement has neither `catch` nor `finally` | `"A 'try' statement must be followed by at least one 'catch' block or a 'finally' block."` |
| **`E0114`** | `MalformedParameterList` | Invalid syntax in function or method parameters | `"Parameters must be formatted as: (type name, type name = default)."` |
| **`E0115`** | `MalformedGenericArguments` | Invalid syntax inside `<...>` generic parameter list | `"Generic type arguments must be comma-separated type annotations."` |
| **`E0120`** | `UnterminatedStringLiteral` | String literal `"` missing closing quote before EOF | `"Add a closing quote '\"' to terminate the string literal."` |
| **`E0121`** | `InvalidEscapeSequence` | Unrecognized escape sequence (e.g. `\q`) | `"Valid escape sequences are: \\n, \\t, \\r, \\\\, \\', \\\", \\0, \\xHH."` |
| **`E0122`** | `EmptyCharacterLiteral` | Character literal `''` contains no character | `"Character literals must contain exactly one character or escape sequence."` |
| **`E0123`** | `MultiCharacterLiteral` | Character literal `'ab'` contains multiple characters | `"Character literals must contain only a single character. Use double quotes '\"' for strings."` |
| **`E0124`** | `IntegerLiteralOverflow` | Integer literal exceeds `u64::MAX` | `"Integer literal value exceeds maximum capacity of 64-bit integer."` |
| **`E0125`** | `MalformedHexEscape` | Incomplete `\x` escape missing 2 hex digits | `"Hexadecimal escape sequences must consist of exactly two hex digits (e.g. \\x1F)."` |

---

## 8. Risk Assessment & Mitigation Strategies

* **Risk 1: Recursion Stack Overflow on Deeply Nested Expressions**
  * *Impact:* Malicious or deeply nested code (e.g. 5,000 nested `((((...))))` or `1 + 1 + ...`) exhausts the call stack, crashing the compiler process with a stack overflow.
  * *Mitigation:* Implement recursion depth limit tracking (`max_recursion_depth: usize = 512`) in `Parser`. If the depth exceeds the threshold, emit diagnostic `E0107` and recover cleanly.
* **Risk 2: Ambiguity in Generic Comparisons (`<` Operator vs Generic `<T>`)**
  * *Impact:* Parsing `a < b` could be misconstrued as generic instantiation `a<b, c>(d)`.
  * *Mitigation:* Employ bounded 2-token lookahead and contextual disambiguation: generic arguments are only parsed following a type name in type positions, or following a method call name when followed by valid type tokens and a closing `>`.
* **Risk 3: Error Cascades During Syntax Recovery**
  * *Impact:* A single syntax error causes hundreds of spurious diagnostic errors across the rest of the file.
  * *Mitigation:* Implement panic-mode synchronization with strict boundary tokens (semicolons `;`, braces `}`, and top-level keywords). After emitting an error, discard tokens until a synchronization delimiter is consumed before resuming statement parsing.
* **Risk 4: Visitor Traversal Performance Overhead**
  * *Impact:* Deeply nested AST traversal in downstream passes becomes slow if dynamic dispatch (`dyn`) is used indiscriminately.
  * *Mitigation:* Design `AstVisitor` with static generic dispatch (`impl AstVisitor`) and provide inlineable default `walk_*` functions to permit aggressive compiler optimizations and dead-code elimination.
