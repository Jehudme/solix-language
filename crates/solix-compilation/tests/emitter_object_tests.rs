//! Unit and integration tests for Phase 6: Object Model, Classes, VTables & Flat ITables.

use solix_common::bytecode::{CompiledFunction, OpCode};
use solix_common::span::Span;
use solix_common::tokens::TokenKind;
use solix_common::types::Type;
use solix_compilation::phases::binder::layout::ClassLayout;
use solix_compilation::phases::binder::symbols::SymbolId;
use solix_compilation::phases::emitter::{
    ConstantPoolBuilder, Emitter, RegisterAllocator, compile_class_layout,
};
use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};
use std::collections::HashMap;

fn make_span(start: u32, end: u32) -> Span {
    Span { id: 0, start, end }
}

#[test]
fn test_object_allocation_and_constructor_window_call() {
    let mut func = CompiledFunction::new("test_new_instance()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["User".to_string()];

    let mut class_layouts = HashMap::new();
    let mut user_layout = ClassLayout::new(SymbolId(1), "User", 5);
    user_layout.instance_size_bytes = 48;
    class_layouts.insert("User".to_string(), user_layout);

    let mut function_ids = HashMap::new();
    function_ids.insert("User::constructor".to_string(), 10);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_class_layouts(&class_layouts)
        .with_function_ids(&function_ids);

    let expr_new = Expression::NewInstance {
        span: make_span(0, 15),
        instance_type:
            solix_compilation::phases::parser::statements::types::TypeAnnotation::Named {
                span: make_span(0, 4), // "User"
                path: vec![make_span(0, 4)],
            },
        arguments: vec![Expression::Literal {
            span: make_span(5, 7),
            value: LiteralValue::Int(100),
        }],
    };

    let dst = emitter
        .emit_expression(&expr_new, None)
        .expect("emit new User(100)");

    // 1. First instruction: ALLOC_OBJECT dst, vtable_id=5, size=48
    let alloc_inst = emitter.function.instructions[0];
    assert_eq!(alloc_inst.opcode(0).unwrap(), OpCode::AllocObject);
    assert_eq!(alloc_inst.dst(), dst);
    assert_eq!(alloc_inst.src1(), 5); // vtable_id
    assert_eq!(alloc_inst.src2(), 48); // size

    // 2. Window call: CALL_DIRECT 0, ctor_id=10, arg_base, arg_count=2 (this + 100)
    let call_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(call_inst.opcode(0).unwrap(), OpCode::CallDirect);
    assert_eq!(call_inst.call_target(), 10);
    assert_eq!(call_inst.arg_count(), 2);
}

#[test]
fn test_instance_field_get_and_set() {
    let mut func = CompiledFunction::new("test_field_access()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["u price".to_string()];

    let u_span = make_span(0, 1);
    let price_span = make_span(2, 7);

    // Register local 'u'
    allocator.enter_scope();
    let u_reg = allocator.alloc_local("u").unwrap();

    let mut class_layouts = HashMap::new();
    let mut user_layout = ClassLayout::new(SymbolId(1), "User", 1);
    user_layout.field_offsets.insert("price".to_string(), 16);
    class_layouts.insert("User".to_string(), user_layout);

    let mut node_types = HashMap::new();
    node_types.insert(
        u_span,
        Type::Class {
            symbol_name: "User".to_string(),
            type_arguments: Vec::new(),
        },
    );

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_class_layouts(&class_layouts)
        .with_node_types(&node_types);

    // 1. Read: u.price
    let expr_get_field = Expression::MemberAccess {
        span: make_span(0, 7),
        object: Box::new(Expression::Identifier { span: u_span }),
        member_span: price_span,
        is_scope_resolution: false,
    };

    let dst = emitter
        .emit_expression(&expr_get_field, None)
        .expect("emit u.price");
    let get_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(get_inst.opcode(0).unwrap(), OpCode::GetField);
    assert_eq!(get_inst.dst(), dst);
    assert_eq!(get_inst.src1(), u_reg);
    assert_eq!(get_inst.src2(), 16); // byte offset

    // 2. Write: u.price = 50
    let expr_set_field = Expression::Assignment {
        span: make_span(0, 15),
        target: Box::new(expr_get_field),
        operator: TokenKind::OperatorAssign,
        value: Box::new(Expression::Literal {
            span: make_span(10, 12),
            value: LiteralValue::Int(50),
        }),
    };

    emitter
        .emit_expression(&expr_set_field, None)
        .expect("emit u.price = 50");
    let mut has_set_field = false;
    for inst in &emitter.function.instructions {
        if let Ok(OpCode::SetField) = inst.opcode(0) {
            has_set_field = true;
            assert_eq!(inst.src1(), u_reg); // object pointer
            assert_eq!(inst.src2(), 16); // field offset
        }
    }
    assert!(has_set_field, "Must emit SET_FIELD");
}

#[test]
fn test_virtual_and_interface_dispatch() {
    let mut func = CompiledFunction::new("test_calls()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["u calc p print".to_string()];

    let u_span = make_span(0, 1);
    let calc_span = make_span(2, 6);
    let p_span = make_span(7, 8);
    let print_span = make_span(9, 14);

    allocator.enter_scope();
    allocator.alloc_local("u").unwrap();
    allocator.alloc_local("p").unwrap();

    let mut node_types = HashMap::new();
    node_types.insert(
        u_span,
        Type::Class {
            symbol_name: "User".to_string(),
            type_arguments: Vec::new(),
        },
    );
    node_types.insert(
        p_span,
        Type::Interface {
            symbol_name: "Printable".to_string(),
            type_arguments: Vec::new(),
        },
    );

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_node_types(&node_types);

    // 1. Virtual call: u.calc(10)
    let expr_virtual = Expression::MethodCall {
        span: make_span(0, 10),
        callee: Box::new(Expression::MemberAccess {
            span: make_span(0, 6),
            object: Box::new(Expression::Identifier { span: u_span }),
            member_span: calc_span,
            is_scope_resolution: false,
        }),
        generic_arguments: Vec::new(),
        arguments: vec![Expression::Literal {
            span: make_span(7, 9),
            value: LiteralValue::Int(10),
        }],
    };

    emitter
        .emit_expression(&expr_virtual, None)
        .expect("emit u.calc(10)");
    let vcall_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(vcall_inst.opcode(0).unwrap(), OpCode::CallVirtual);
    assert_eq!(vcall_inst.arg_count(), 2); // this + 10

    // 2. Interface call: p.print()
    let expr_interface = Expression::MethodCall {
        span: make_span(15, 25),
        callee: Box::new(Expression::MemberAccess {
            span: make_span(15, 22),
            object: Box::new(Expression::Identifier { span: p_span }),
            member_span: print_span,
            is_scope_resolution: false,
        }),
        generic_arguments: Vec::new(),
        arguments: Vec::new(),
    };

    emitter
        .emit_expression(&expr_interface, None)
        .expect("emit p.print()");
    let iface_call_inst = emitter.function.instructions.last().unwrap();
    assert_eq!(iface_call_inst.opcode(0).unwrap(), OpCode::CallInterface);
    assert_eq!(iface_call_inst.arg_count(), 1); // this only
}

#[test]
fn test_instanceof_rtti_check() {
    let mut func = CompiledFunction::new("test_instanceof()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["u User".to_string()];

    let u_span = make_span(0, 1);
    let user_span = make_span(2, 6);

    allocator.enter_scope();
    let u_reg = allocator.alloc_local("u").unwrap();

    let mut class_layouts = HashMap::new();
    let user_layout = ClassLayout::new(SymbolId(1), "User", 7);
    class_layouts.insert("User".to_string(), user_layout);

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_class_layouts(&class_layouts);

    // u instanceof User
    let expr_instanceof = Expression::Instanceof {
        span: make_span(0, 15),
        expression: Box::new(Expression::Identifier { span: u_span }),
        target_type: solix_compilation::phases::parser::statements::types::TypeAnnotation::Named {
            span: user_span,
            path: vec![user_span],
        },
    };

    let dst = emitter
        .emit_expression(&expr_instanceof, None)
        .expect("emit instanceof");
    let inst = emitter.function.instructions.last().unwrap();
    assert_eq!(inst.opcode(0).unwrap(), OpCode::Instanceof);
    assert_eq!(inst.dst(), dst);
    assert_eq!(inst.src1(), u_reg);
    assert_eq!(inst.src2(), 7); // vtable_id
}

#[test]
fn test_compile_class_layout_to_compiled_class() {
    let mut layout = ClassLayout::new(SymbolId(10), "solix::models::User", 3);
    layout.base_vtable_id = Some(1);
    layout.instance_size_bytes = 64;
    layout.reference_field_offsets = vec![8, 24];
    layout.vtable_methods = vec![SymbolId(101), SymbolId(102)];
    layout
        .itable_methods
        .insert("IComparable".to_string(), vec![SymbolId(102)]);

    let mut symbol_to_func_id = HashMap::new();
    symbol_to_func_id.insert(SymbolId(101), 12);
    symbol_to_func_id.insert(SymbolId(102), 14);

    let mut iface_to_id = HashMap::new();
    iface_to_id.insert("IComparable".to_string(), 99);

    let compiled_class = compile_class_layout(&layout, &symbol_to_func_id, &iface_to_id, Some(15));

    assert_eq!(compiled_class.symbol, "solix::models::User");
    assert_eq!(compiled_class.vtable_id, 3);
    assert_eq!(compiled_class.base_vtable_id, Some(1));
    assert_eq!(compiled_class.instance_size, 64);
    assert_eq!(compiled_class.reference_field_offsets, vec![8, 24]);
    assert_eq!(compiled_class.vtable, vec![12, 14]);
    assert_eq!(compiled_class.itable, vec![(99, vec![14])]);
    assert_eq!(compiled_class.destructor_id, Some(15));
}

#[test]
fn test_weak_reference_field_layout_and_assignment_emission() {
    let mut func = CompiledFunction::new("test_weak_assignment()", 0, 0, 0);
    let mut allocator = RegisterAllocator::new(0);
    let mut pool = ConstantPoolBuilder::new();
    let sources = vec!["node.parent = parent;".to_string()];

    let mut class_layouts = HashMap::new();
    let mut node_layout = ClassLayout::new(SymbolId(10), "Node", 2);
    node_layout.instance_size_bytes = 32;
    node_layout.field_offsets.insert("child".to_string(), 8);
    node_layout.field_offsets.insert("parent".to_string(), 16);
    node_layout.reference_field_offsets.push(8); // child is strong
    node_layout.weak_field_offsets.push(16); // parent is weak
    class_layouts.insert("Node".to_string(), node_layout);

    let mut node_types = HashMap::new();
    let node_type = Type::Class {
        symbol_name: "Node".to_string(),
        type_arguments: Vec::new(),
    };
    node_types.insert(make_span(0, 4), node_type.clone()); // "node"
    node_types.insert(make_span(14, 20), node_type); // "parent"

    let local_node = allocator.alloc_local("node").unwrap();
    let local_parent = allocator.alloc_local("parent").unwrap();

    let mut emitter = Emitter::new(&mut func, &mut allocator, &mut pool)
        .with_sources(&sources)
        .with_class_layouts(&class_layouts)
        .with_node_types(&node_types);

    // Assignment: node.parent = parent
    let assign_expr = Expression::Assignment {
        span: make_span(0, 20),
        target: Box::new(Expression::MemberAccess {
            span: make_span(0, 11), // "node.parent"
            object: Box::new(Expression::Identifier {
                span: make_span(0, 4),
            }),
            member_span: make_span(5, 11), // "parent"
            is_scope_resolution: false,
        }),
        operator: TokenKind::OperatorAssign,
        value: Box::new(Expression::Identifier {
            span: make_span(14, 20),
        }),
    };

    emitter.emit_expression(&assign_expr, None).unwrap();

    // Verify SetField has FLAG_WEAK set
    let set_field_inst = func
        .instructions
        .iter()
        .find(|i| i.opcode(0).ok() == Some(OpCode::SetField))
        .expect("Expected SetField instruction");

    assert!(
        set_field_inst.is_weak(),
        "Expected FLAG_WEAK to be set on weak SetField"
    );
    assert_eq!(
        set_field_inst.flags(),
        solix_common::bytecode::instruction::FLAG_WEAK
    );
    assert_eq!(set_field_inst.dst(), local_parent); // r_val
    assert_eq!(set_field_inst.src1(), local_node); // r_obj
    assert_eq!(set_field_inst.src2(), 16); // field offset
}

#[test]
fn test_overloaded_string_vs_boxed_methods_static_null_and_super_ctor() {
    use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};

    let source_code = r#"
public class BoxedStr {
    public string raw_value;
    public BoxedStr(string initial) {
        this.raw_value = initial;
    }
    public string unwrap() {
        return this.raw_value;
    }
}

public class GenericMap<KeyType, ValueType> {
    public GenericMap() {}
}

public class BaseCustomException {
    public string message;
    public BaseCustomException() {
        this.message = "";
    }
    public BaseCustomException(string message) {
        this.message = message;
    }
}

public class DerivedCustomException extends BaseCustomException {
    public DerivedCustomException() : super("default") {}
    public DerivedCustomException(string message) : super(message) {}
}

public class OverloadTarget {
    public int32 process(string raw_text) {
        return this.process(new BoxedStr(raw_text));
    }
    public int32 process(BoxedStr boxed) {
        return boxed.unwrap().length;
    }

    public static int32 dispatch(string verb, string path) {
        return OverloadTarget::dispatch(new BoxedStr(verb), new BoxedStr(path), null, null);
    }
    public static int32 dispatch(
        BoxedStr verb,
        BoxedStr path,
        BoxedStr body,
        GenericMap<BoxedStr, BoxedStr> headers
    ) {
        int32 total = verb.unwrap().length + path.unwrap().length;
        if (body != null) {
            total = total + body.unwrap().length;
        }
        return total;
    }
}

public class Main {
    public static int32 main() {
        OverloadTarget target = new OverloadTarget();
        DerivedCustomException err = new DerivedCustomException("boom");
        return target.process("hello") + OverloadTarget::dispatch("GET", "/ok") + err.message.length;
    }
}
"#;

    let mut options = CompilationOptions::new().with_source("overload_regression.slx", source_code);
    options.profile = solix_compilation::CompilationContext::BuildProfile::Debug;
    options.inline_threshold = 0;
    options.bytecode_optimization = false;
    let mut context = CompilationContext::new(options);
    context
        .execute()
        .expect("Overloaded (string) vs (BoxedStr), static null generic arg, and super(msg) must compile cleanly");

    let program = &context
        .emission_artifacts
        .as_ref()
        .expect("emission artifacts present")
        .program;

    let find_func = |prefix: &str| {
        program
            .functions
            .iter()
            .enumerate()
            .find(|(_, func)| func.symbol.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing function starting with '{prefix}'"))
    };

    let (process_str_idx, process_str_fn) = find_func("OverloadTarget::process(string)");
    let (process_boxed_idx, _) = find_func("OverloadTarget::process(BoxedStr)");
    assert_ne!(
        process_str_idx, process_boxed_idx,
        "Overloaded instance methods differing by parameter types must receive distinct IDs"
    );
    let calls_self_virtual_or_direct = process_str_fn.instructions.iter().any(|inst| {
        inst.opcode(0).ok() == Some(OpCode::CallDirect)
            && inst.call_target() as usize == process_str_idx
    });
    assert!(
        !calls_self_virtual_or_direct,
        "process(string) must not emit recursive CallDirect to itself"
    );

    let (dispatch_2_idx, dispatch_2_fn) = find_func("OverloadTarget::dispatch(string,string)");
    let (dispatch_4_idx, _) =
        find_func("OverloadTarget::dispatch(BoxedStr,BoxedStr,BoxedStr,GenericMap");
    let dispatch_2_calls_4 = dispatch_2_fn.instructions.iter().any(|inst| {
        inst.opcode(0).ok() == Some(OpCode::CallDirect)
            && inst.call_target() as usize == dispatch_4_idx
    });
    assert!(
        dispatch_2_calls_4,
        "2-arg OverloadTarget::dispatch(string, string) passing null generic args must call 4-arg target #{dispatch_4_idx}, not itself #{dispatch_2_idx}"
    );

    let base_ctor_empty_idx = program
        .functions
        .iter()
        .enumerate()
        .find(|(_, func)| func.symbol == "BaseCustomException::constructor")
        .map(|(index, _)| index)
        .expect("BaseCustomException::constructor present");
    let base_ctor_str_idx = program
        .functions
        .iter()
        .enumerate()
        .find(|(_, func)| func.symbol == "BaseCustomException::constructor(string)")
        .map(|(index, _)| index)
        .expect("BaseCustomException::constructor(string) present");
    let (_, derived_ctor_fn) = find_func("DerivedCustomException::constructor(string)");
    assert_ne!(base_ctor_empty_idx, base_ctor_str_idx);
    let calls_base_str_ctor = derived_ctor_fn.instructions.iter().any(|inst| {
        inst.opcode(0).ok() == Some(OpCode::CallDirect)
            && inst.call_target() as usize == base_ctor_str_idx
    });
    assert!(
        calls_base_str_ctor,
        "DerivedCustomException(string) : super(message) must call BaseCustomException::constructor(string)"
    );
}
