//! Comprehensive integration and unit test suite for Milestone 007: Advanced Language Features & Emitter Completion.

#![forbid(unsafe_code)]

use solix_common::bytecode::{CompiledFunction, OpCode};
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_common::types::{PrimitiveKind, Type};
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::binder::layout::ClassLayout;
use solix_compilation::phases::binder::symbols::SymbolId;
use solix_compilation::phases::emitter::{ConstantPoolBuilder, Emitter, RegisterAllocator};
use solix_compilation::phases::parser::statements::control_flow::{AssertStatement, ExitStatement};
use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};
use solix_compilation::phases::parser::statements::types::TypeAnnotation;
use std::collections::HashMap;

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_sizeof_expression_primitives_and_classes() {
    let mut func = CompiledFunction::new("test_sizeof()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["int32 int64 bool Point".to_string()];

    let mut class_layouts = HashMap::new();
    let mut point_layout = ClassLayout::new(SymbolId(1), "Point", 1);
    point_layout.instance_size_bytes = 24;
    class_layouts.insert("Point".to_string(), point_layout);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_class_layouts(&class_layouts);

    // 1. sizeof(int32) -> 4 ("int32" is 0..5)
    let expr_i32 = Expression::Sizeof {
        span: make_span(0, 5),
        target_type: TypeAnnotation::Named {
            span: make_span(0, 5),
            path: vec![make_span(0, 5)],
        },
    };
    let dst1 = emitter
        .emit_expression(&expr_i32, None)
        .expect("emit sizeof(int32)");
    let inst1 = emitter.function.instructions[0];
    assert_eq!(inst1.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst1.dst(), dst1);
    assert_eq!(inst1.imm32_signed(), 4);

    // 2. sizeof(int64) -> 8 ("int64" is 6..11)
    let expr_i64 = Expression::Sizeof {
        span: make_span(6, 11),
        target_type: TypeAnnotation::Named {
            span: make_span(6, 11),
            path: vec![make_span(6, 11)],
        },
    };
    let dst2 = emitter
        .emit_expression(&expr_i64, None)
        .expect("emit sizeof(int64)");
    let inst2 = emitter.function.instructions[1];
    assert_eq!(inst2.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst2.dst(), dst2);
    assert_eq!(inst2.imm32_signed(), 8);

    // 3. sizeof(bool) -> 1 ("bool" is 12..16)
    let expr_bool = Expression::Sizeof {
        span: make_span(12, 16),
        target_type: TypeAnnotation::Named {
            span: make_span(12, 16),
            path: vec![make_span(12, 16)],
        },
    };
    let dst3 = emitter
        .emit_expression(&expr_bool, None)
        .expect("emit sizeof(bool)");
    let inst3 = emitter.function.instructions[2];
    assert_eq!(inst3.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst3.dst(), dst3);
    assert_eq!(inst3.imm32_signed(), 1);

    // 4. sizeof(Point) -> 24 ("Point" is 17..22)
    let expr_point = Expression::Sizeof {
        span: make_span(17, 22),
        target_type: TypeAnnotation::Named {
            span: make_span(17, 22),
            path: vec![make_span(17, 22)],
        },
    };
    let dst4 = emitter
        .emit_expression(&expr_point, None)
        .expect("emit sizeof(Point)");
    let inst4 = emitter.function.instructions[3];
    assert_eq!(inst4.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst4.dst(), dst4);
    assert_eq!(inst4.imm32_signed(), 24);
}

#[test]
fn test_default_expression_primitives_floats_and_references() {
    let mut func = CompiledFunction::new("test_default()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["int32 float64 Point".to_string()];

    let mut class_layouts = HashMap::new();
    let point_layout = ClassLayout::new(SymbolId(1), "Point", 1);
    class_layouts.insert("Point".to_string(), point_layout);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_class_layouts(&class_layouts);

    // 1. default(int32) -> LoadI32 0 ("int32" is 0..5)
    let expr_i32 = Expression::Default {
        span: make_span(0, 5),
        target_type: TypeAnnotation::Named {
            span: make_span(0, 5),
            path: vec![make_span(0, 5)],
        },
    };
    let dst1 = emitter
        .emit_expression(&expr_i32, None)
        .expect("emit default(int32)");
    let inst1 = emitter.function.instructions[0];
    assert_eq!(inst1.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst1.dst(), dst1);
    assert_eq!(inst1.imm32_signed(), 0);

    // 2. default(float64) -> LoadConst pool(0.0) ("float64" is 6..13)
    let expr_f64 = Expression::Default {
        span: make_span(6, 13),
        target_type: TypeAnnotation::Named {
            span: make_span(6, 13),
            path: vec![make_span(6, 13)],
        },
    };
    let dst2 = emitter
        .emit_expression(&expr_f64, None)
        .expect("emit default(float64)");
    let inst2 = emitter.function.instructions[1];
    assert_eq!(inst2.opcode(0).unwrap(), OpCode::LoadConst);
    assert_eq!(inst2.dst(), dst2);

    // 3. default(Point) -> LoadNull ("Point" is 14..19)
    let expr_point = Expression::Default {
        span: make_span(14, 19),
        target_type: TypeAnnotation::Named {
            span: make_span(14, 19),
            path: vec![make_span(14, 19)],
        },
    };
    let dst3 = emitter
        .emit_expression(&expr_point, None)
        .expect("emit default(Point)");
    let inst3 = emitter.function.instructions[2];
    assert_eq!(inst3.opcode(0).unwrap(), OpCode::LoadNull);
    assert_eq!(inst3.dst(), dst3);
}

#[test]
fn test_exit_statement_emission() {
    let mut func = CompiledFunction::new("test_exit()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["exit(42);".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    let exit_stmt = ExitStatement {
        span: make_span(0, 9),
        code: Expression::Literal {
            span: make_span(5, 7),
            value: LiteralValue::Int(42),
        },
    };

    emitter
        .emit_statement(
            &solix_compilation::phases::parser::statements::Statement::Exit(exit_stmt),
            &mut Vec::new(),
        )
        .expect("emit exit statement");

    // Expect LoadI32 42 into temp, then Exit temp
    let load_inst = emitter.function.instructions[0];
    assert_eq!(load_inst.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(load_inst.imm32_signed(), 42);

    let exit_inst = emitter.function.instructions[1];
    assert_eq!(exit_inst.opcode(0).unwrap(), OpCode::Exit);
    assert_eq!(exit_inst.dst(), load_inst.dst());
}

#[test]
fn test_assert_statement_emission() {
    let mut func = CompiledFunction::new("test_assert()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["assert(flag, \"failed\");".to_string()];

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool).with_sources(&sources);

    let assert_stmt = AssertStatement {
        span: make_span(0, 23),
        condition: Expression::Literal {
            span: make_span(7, 11),
            value: LiteralValue::Bool(true),
        },
        message: Some(Expression::Literal {
            span: make_span(13, 21),
            value: LiteralValue::String("failed".to_string()),
        }),
    };

    emitter
        .emit_statement(
            &solix_compilation::phases::parser::statements::Statement::Assert(assert_stmt),
            &mut Vec::new(),
        )
        .expect("emit assert statement");

    // Instructions:
    // 0: LoadI32 dst, 1 (true)
    // 1: JumpIfTrue cond, offset to skip throw
    // 2: LoadConst msg_reg, const_idx
    // 3: Throw msg_reg
    assert!(emitter.function.instructions.len() >= 4);
    let jump_inst = emitter.function.instructions[1];
    assert_eq!(jump_inst.opcode(0).unwrap(), OpCode::JumpIfTrue);
    let throw_inst = emitter.function.instructions[3];
    assert_eq!(throw_inst.opcode(0).unwrap(), OpCode::Throw);
}

#[test]
fn test_enum_variant_constant_load_i32() {
    let mut func = CompiledFunction::new("test_enum()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["Color::Green".to_string()];

    let mut enum_variants = HashMap::new();
    enum_variants.insert("Color::Red".to_string(), 0);
    enum_variants.insert("Color::Green".to_string(), 1);
    enum_variants.insert("Color::Blue".to_string(), 2);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_enum_variants(&enum_variants);

    let member_access = Expression::MemberAccess {
        span: make_span(0, 12),
        object: Box::new(Expression::Identifier {
            span: make_span(0, 5), // "Color"
        }),
        member_span: make_span(7, 12), // "Green"
        is_scope_resolution: true,
    };

    let dst = emitter
        .emit_expression(&member_access, None)
        .expect("emit Color::Green");
    let inst = emitter.function.instructions[0];
    assert_eq!(inst.opcode(0).unwrap(), OpCode::LoadI32);
    assert_eq!(inst.dst(), dst);
    assert_eq!(inst.imm32_signed(), 1);
}

#[test]
fn test_tuple_element_instanceof_emission() {
    let mut func = CompiledFunction::new("test_tuple_instanceof()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["tup[0] int32".to_string()];

    let mut node_types = HashMap::new();
    let tuple_span = make_span(0, 3);
    node_types.insert(
        tuple_span,
        Type::Tuple(vec![
            Type::Primitive(PrimitiveKind::Int32),
            Type::Primitive(PrimitiveKind::Bool),
        ]),
    );

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types);

    // Register tup in local scope
    let r_tuple = emitter.allocator.alloc_local("tup").unwrap();

    let tuple_access = Expression::ArrayAccess {
        span: make_span(0, 6),
        array: Box::new(Expression::Identifier { span: tuple_span }),
        index: Box::new(Expression::Literal {
            span: make_span(4, 5),
            value: LiteralValue::Int(0),
        }),
    };

    let instanceof_expr = Expression::Instanceof {
        span: make_span(0, 12),
        expression: Box::new(tuple_access),
        target_type: TypeAnnotation::Named {
            span: make_span(7, 12), // "int32"
            path: vec![make_span(7, 12)],
        },
    };

    let dst = emitter
        .emit_expression(&instanceof_expr, None)
        .expect("emit tuple instanceof");
    let last_inst = *emitter.function.instructions.last().unwrap();
    assert_eq!(last_inst.opcode(0).unwrap(), OpCode::TupleInstanceof);
    assert_eq!(last_inst.dst(), dst);
    assert_eq!(last_inst.src1(), r_tuple);
    // int32 tag is 3
    assert_eq!(last_inst.flags(), 3);
}

#[test]
fn test_operator_overload_binary_expression_call() {
    let mut func = CompiledFunction::new("test_op()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["v1 + v2".to_string()];

    let mut node_types = HashMap::new();
    let v1_span = make_span(0, 2);
    let v2_span = make_span(5, 7);
    node_types.insert(
        v1_span,
        Type::Class {
            symbol_name: "Vec2".to_string(),
            type_arguments: Vec::new(),
        },
    );
    node_types.insert(
        v2_span,
        Type::Class {
            symbol_name: "Vec2".to_string(),
            type_arguments: Vec::new(),
        },
    );

    let mut function_ids = HashMap::new();
    function_ids.insert("Vec2::operator_+".to_string(), 42);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types)
        .with_function_ids(&function_ids);

    let _r_v1 = emitter.allocator.alloc_local("v1").unwrap();
    let _r_v2 = emitter.allocator.alloc_local("v2").unwrap();

    let binary_expr = Expression::Binary {
        span: make_span(0, 7),
        left: Box::new(Expression::Identifier { span: v1_span }),
        operator: TokenKind::OperatorPlus,
        right: Box::new(Expression::Identifier { span: v2_span }),
    };

    let dst = emitter
        .emit_expression(&binary_expr, None)
        .expect("emit binary operator+");
    // CallDirect 42 with window size 2
    let call_inst = emitter
        .function
        .instructions
        .iter()
        .find(|i| i.opcode(0).ok() == Some(OpCode::CallDirect))
        .expect("CallDirect instruction for operator+");

    assert_eq!(call_inst.call_target(), 42); // callee_id
    assert_eq!(call_inst.call_dst(), dst);
}

#[test]
fn test_end_to_end_advanced_features_compilation() {
    let source_code = r#"
enum Status {
    Pending,
    Active,
    Terminated
}

class Base {
    public int32 base_id;

    public Base(int32 id) {
        this.base_id = id;
    }
}

class Derived extends Base {
    public int32 extra;

    public Derived(int32 id, int32 extra) : super(id) {
        this.extra = extra;
    }

    public int32 operator +(Derived other) {
        return this.extra + other.extra;
    }
}

int32 main() {
    int32 s_point = sizeof(Base);
    int32 def_val = default(int32);
    int32 status = Status::Active;
    assert(s_point > 0, "size must be positive");
    return status;
}
"#;

    let options = CompilationOptions::new().with_source("advanced_test.slx", source_code);
    let mut context = CompilationContext::new(options);

    context.lex_source(0).expect("lex source");
    context.parse_source(0).expect("parse source");

    let artifacts = context.emit_context().expect("emit advanced context");
    assert_eq!(artifacts.functions_by_source.len(), 1);

    let funcs = &artifacts.functions_by_source[0];
    let symbols: Vec<&str> = funcs.iter().map(|f| f.symbol.as_str()).collect();

    // Verify constructor, super call chaining, operator overload, and main are compiled
    assert!(
        symbols.iter().any(|s| s.starts_with("Base::constructor")),
        "Base ctor compiled"
    );
    assert!(
        symbols
            .iter()
            .any(|s| s.starts_with("Derived::constructor")),
        "Derived ctor compiled"
    );
    assert!(
        symbols.contains(&"Derived::operator_+"),
        "Derived operator+ compiled"
    );
    assert!(symbols.contains(&"main"), "main compiled");

    // Inspect Derived constructor for super windowed call
    let derived_ctor = funcs
        .iter()
        .find(|f| f.symbol.starts_with("Derived::constructor"))
        .unwrap();
    let has_super_call = derived_ctor
        .instructions
        .iter()
        .any(|i| i.opcode(0).ok() == Some(OpCode::CallDirect));
    assert!(
        has_super_call,
        "Derived constructor must contain CallDirect to Base::constructor"
    );

    // Inspect main function for Sizeof (LoadI32), Default (LoadI32 0), and Enum variant (LoadI32 1)
    let main_fn = funcs.iter().find(|f| f.symbol == "main").unwrap();
    let load_i32_immediates: Vec<i32> = main_fn
        .instructions
        .iter()
        .filter(|i| i.opcode(0).ok() == Some(OpCode::LoadI32))
        .map(|i| i.imm32_signed())
        .collect();

    // Enum variant Status::Active is index 1
    assert!(
        load_i32_immediates.contains(&1),
        "Main must load enum variant 1 for Status::Active"
    );
    // Default int32 is 0
    assert!(
        load_i32_immediates.contains(&0),
        "Main must load default int32 0"
    );
}

#[test]
fn test_protected_access_control_allowed_for_subclass_denied_for_external() {
    let valid_source = r#"
class Parent {
    protected int32 secret;
    public Parent(int32 s) {
        this.secret = s;
    }
}

class Child extends Parent {
    public Child(int32 s) : super(s) {}

    public int32 read_secret() {
        return this.secret;
    }
}
"#;
    let options = CompilationOptions::new().with_source("valid_protected.slx", valid_source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let bind_res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        bind_res.is_ok(),
        "Subclass accessing protected field must succeed"
    );

    let invalid_source = r#"
class Parent {
    protected int32 secret;
    public Parent(int32 s) {
        this.secret = s;
    }
}

class Stranger {
    public int32 steal(Parent p) {
        return p.secret;
    }
}
"#;
    let options_bad =
        CompilationOptions::new().with_source("invalid_protected.slx", invalid_source);
    let mut context_bad = CompilationContext::new(options_bad);
    context_bad.lex_source(0).expect("lex");
    context_bad.parse_source(0).expect("parse");
    let bind_res_bad = solix_compilation::phases::binder::bind(&mut context_bad);
    assert!(
        bind_res_bad.is_err(),
        "External class accessing protected field must produce ProtectedAccessViolation"
    );
    let errors_bad = bind_res_bad.unwrap_err();
    assert!(
        errors_bad.iter().any(|e| e.error_code() == "E0209"),
        "Expected error code E0209 (ProtectedAccessViolation)"
    );
}

#[test]
fn test_negative_private_access_from_external_class() {
    let source = r#"
class SecretVault {
    private int32 secret_code;
    public SecretVault(int32 code) {
        this.secret_code = code;
    }
}

class Hacker {
    public int32 exploit(SecretVault v) {
        return v.secret_code;
    }
}
"#;
    let options = CompilationOptions::new().with_source("private_violation.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "External access to private member must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0208"),
        "Expected error code E0208 (PrivateAccessViolation)"
    );
}

#[test]
fn test_negative_assert_non_boolean_condition() {
    let source = r#"
void test() {
    assert("not_a_bool", "assertion message");
}
"#;
    let options = CompilationOptions::new().with_source("invalid_assert.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Assert with string condition must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0203"),
        "Expected error code E0203 (TypeMismatch for assert condition)"
    );
}

#[test]
fn test_negative_exit_non_integer_status() {
    let source = r#"
void test() {
    exit("fatal_error");
}
"#;
    let options = CompilationOptions::new().with_source("invalid_exit.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(res.is_err(), "Exit with string code must be rejected");
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0203"),
        "Expected error code E0203 (TypeMismatch for exit code)"
    );
}

#[test]
fn test_negative_weak_on_primitive_field() {
    let source = r#"
class Node {
    weak int32 count;
}
"#;
    let options = CompilationOptions::new().with_source("invalid_weak.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Weak modifier on primitive int32 must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0229"),
        "Expected error code E0229 (InvalidWeakModifier)"
    );
}

#[test]
fn test_negative_super_call_in_root_class() {
    let source = r#"
class RootClass {
    public RootClass() : super(10) {}
}
"#;
    let options = CompilationOptions::new().with_source("invalid_super.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Super call in class without superclass must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0228"),
        "Expected error code E0228 (InvalidSuperCall)"
    );
}

#[test]
fn test_negative_operator_not_implemented_on_class() {
    let source = r#"
class Vector2D {
    public int32 x;
    public int32 y;
}

void test() {
    Vector2D a = new Vector2D();
    Vector2D b = new Vector2D();
    Vector2D c = a + b;
}
"#;
    let options = CompilationOptions::new().with_source("missing_op.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Binary + on class without operator+ must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0220"),
        "Expected error code E0220 (InvalidOperatorOverload)"
    );
}

#[test]
fn test_negative_const_variable_reassignment() {
    let source = r#"
void test() {
    const int32 MAX_USERS = 100;
    MAX_USERS = 200;
}
"#;
    let options = CompilationOptions::new().with_source("const_assign.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Reassigning to a const variable must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0210"),
        "Expected error code E0210 (CannotAssignToConst)"
    );
}

#[test]
fn test_negative_abstract_class_instantiation() {
    let source = r#"
abstract class BaseProcessor {
    public abstract void process();
}

void test() {
    BaseProcessor p = new BaseProcessor();
}
"#;
    let options = CompilationOptions::new().with_source("abstract_new.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Instantiating an abstract class directly must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0217"),
        "Expected error code E0217 (CannotInstantiateAbstractType)"
    );
}

#[test]
fn test_negative_throw_primitive_value() {
    let source = r#"
void test() {
    throw 404;
}
"#;
    let options = CompilationOptions::new().with_source("throw_primitive.slx", source);
    let mut context = CompilationContext::new(options);
    context.lex_source(0).expect("lex");
    context.parse_source(0).expect("parse");
    let res = solix_compilation::phases::binder::bind(&mut context);
    assert!(
        res.is_err(),
        "Throwing a primitive integer instead of an exception reference must be rejected"
    );
    let errors = res.unwrap_err();
    assert!(
        errors.iter().any(|e| e.error_code() == "E0203"),
        "Expected error code E0203 (TypeMismatch for throw)"
    );
}
