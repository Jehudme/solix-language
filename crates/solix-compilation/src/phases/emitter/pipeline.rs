//! Full compilation pipeline driver for register bytecode emission and SLXB container generation.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::phases::binder::SymbolId;
use crate::phases::binder::layout::ClassLayout;
use crate::phases::binder::passes::pass1_symbol_discovery::{
    compute_effective_package, extract_namespace_path,
};
use crate::phases::binder::symbols::SymbolTable;
use crate::phases::binder::type_resolver::resolve_type_annotation;
use crate::phases::emitter::constant_pool::ConstantPoolBuilder;
use crate::phases::emitter::control_flow::LoopContext;
use crate::phases::emitter::emitter::Emitter;
use crate::phases::emitter::object_model::compile_class_layout;
use crate::phases::emitter::register_allocator::RegisterAllocator;
use crate::phases::emitter::static_layout::{
    StaticLayout, StaticVariableDecl, synthesize_module_init,
};
use crate::phases::lexing::TokenKind;
use crate::phases::parser::statements::declarations::{
    ClassDeclaration, ClassMember, ConstructorDeclaration, Declaration, DestructorDeclaration,
    MethodDeclaration, Modifier, OperatorDeclaration,
};
use crate::phases::parser::statements::expressions::{Expression, LiteralValue};
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::parser::statements::{ProgramAst, Statement};
use solix_common::bytecode::{
    BinaryProgram, CompiledFunction, DeclarationModifier, ExportManifest, ExportedClass,
    ExportedEnum, ExportedField, ExportedFunction, ExportedGenericTemplate, ExportedParameter,
    ExportedStatic, Instruction, OpCode,
};
use solix_common::diagnostics::EmissionError;
use solix_common::types::{PrimitiveKind, Type};
use std::collections::HashMap;

/// Emitted artifacts container capturing per-source function metadata and the final binary program.
#[derive(Debug, Clone, PartialEq)]
pub struct EmissionArtifacts {
    /// Compiled functions partitioned by source file index: `functions_by_source[source_id]`
    /// contains the list of compiled function metadata for that specific source file.
    pub functions_by_source: Vec<Vec<CompiledFunction>>,
    /// Hermetic serialized virtual register executable binary program container (`SLXB`).
    pub program: BinaryProgram,
}

const fn to_decl_modifier(modifier: Modifier) -> Option<DeclarationModifier> {
    match modifier {
        Modifier::Public => Some(DeclarationModifier::Public),
        Modifier::Protected => Some(DeclarationModifier::Protected),
        Modifier::Private => Some(DeclarationModifier::Private),
        Modifier::Static => Some(DeclarationModifier::Static),
        Modifier::Const => Some(DeclarationModifier::Const),
        Modifier::Virtual => Some(DeclarationModifier::Virtual),
        Modifier::Override => Some(DeclarationModifier::Override),
        Modifier::Abstract => Some(DeclarationModifier::Abstract),
        Modifier::Weak => Some(DeclarationModifier::Weak),
        Modifier::Inline => Some(DeclarationModifier::Inline),
        Modifier::Extern => Some(DeclarationModifier::Extern),
        Modifier::Internal => Some(DeclarationModifier::Private),
        Modifier::Native => None,
    }
}

fn evaluate_integer_constant(expression: &Expression) -> Option<i64> {
    match expression {
        Expression::Literal {
            value: LiteralValue::Int(value),
            ..
        } => Some(*value),
        Expression::Literal {
            value: LiteralValue::UInt(value),
            ..
        } => Some(*value as i64),
        Expression::Unary {
            operator: TokenKind::OperatorMinus,
            operand,
            ..
        } => evaluate_integer_constant(operand).map(|val| -val),
        Expression::Unary {
            operator: TokenKind::OperatorPlus,
            operand,
            ..
        } => evaluate_integer_constant(operand),
        _ => None,
    }
}

/// Executes full register bytecode emission across all registered source files in the context.
///
/// # Errors
/// Returns `EmissionError` if constant pool limits, register allocation limits, or binary
/// encoding constraints are violated.
pub fn emit_context(context: &mut CompilationContext) -> Result<EmissionArtifacts, EmissionError> {
    // Ensure semantic binding artifacts are available
    if context.binding_artifacts.is_none() {
        let _ = crate::phases::binder::bind_context(context);
    }

    let sources_count = context.sources.len();
    let mut functions_by_source: Vec<Vec<CompiledFunction>> = vec![Vec::new(); sources_count];
    let mut synthesized_lambdas: Vec<CompiledFunction> = Vec::new();

    // 1. Collect static fields across all compilation units
    let mut static_decls = Vec::new();
    for ast in context.ast.iter().flatten() {
        collect_static_declarations(ast, &context.sources, &mut static_decls);
    }

    let mut static_layout = StaticLayout::new();
    for decl in &static_decls {
        static_layout.assign_static(&decl.symbol);
    }

    let mut pool = ConstantPoolBuilder::new();

    let node_types = context
        .binding_artifacts
        .as_ref()
        .map(|b| b.node_types.clone())
        .unwrap_or_default();

    // 2. Collect class layouts and function symbols to allocate IDs
    let class_layouts: HashMap<String, ClassLayout> = context
        .binding_artifacts
        .as_ref()
        .map(|b| {
            b.layouts
                .values()
                .flat_map(|l| {
                    let simple = l
                        .class_name
                        .rsplit(['.', ':'])
                        .next()
                        .unwrap_or(&l.class_name)
                        .to_string();
                    vec![(l.class_name.clone(), l.clone()), (simple, l.clone())]
                })
                .collect()
        })
        .unwrap_or_default();

    let mut function_ids: HashMap<String, u32> = HashMap::new();
    function_ids.insert("@init".to_string(), 0);
    let mut next_func_id = 1u32;

    let mut generic_invocations: HashMap<(String, String), Vec<String>> = HashMap::new();
    for ast in context.ast.iter().flatten() {
        collect_generic_calls_in_ast(ast, &context.sources, &mut generic_invocations);
    }

    // Index function symbols to allocate deterministic function IDs
    for ast in context.ast.iter().flatten() {
        index_function_symbols(
            ast,
            &context.sources,
            &mut function_ids,
            &mut next_func_id,
            &generic_invocations,
        );
    }

    if let Some(artifacts) = &context.binding_artifacts {
        for mono_cls in &artifacts.monomorphized_classes {
            index_monomorphized_class_symbols(
                mono_cls,
                &artifacts.symbol_table,
                &context.sources,
                &mut function_ids,
                &mut next_func_id,
            );
        }
    }

    // 3. Synthesize Module Static Initializer (@init - Function ID 0)
    let init_func = synthesize_module_init(
        &static_decls,
        &static_layout,
        &mut pool,
        Some(&node_types),
        Some(&context.sources),
        Some(&function_ids),
        Some(&class_layouts),
    )?;

    let mut enum_variants: HashMap<String, u32> = HashMap::new();
    for ast in context.ast.iter().flatten() {
        let source_index = ast.source_id as usize;
        if source_index < context.sources.len() {
            let code = &context.sources[source_index];
            for declaration in &ast.declarations {
                match declaration {
                    Declaration::Enum(enum_decl) => {
                        let enum_name = get_span_text(enum_decl.name_span, code);
                        let mut current_variant_value = 0u32;
                        for (variant_index, variant) in enum_decl.variants.iter().enumerate() {
                            let variant_name = get_span_text(variant.name_span, code);
                            if let Some(explicit) = &variant.explicit_value {
                                if let Some(integer_value) = evaluate_integer_constant(explicit) {
                                    current_variant_value = integer_value as u32;
                                }
                            } else if variant_index > 0 {
                                current_variant_value += 1;
                            }
                            let full_symbol = format!("{enum_name}::{variant_name}");
                            enum_variants.insert(full_symbol, current_variant_value);
                        }
                    }
                    Declaration::Class(class_decl) => {
                        let class_name = get_span_text(class_decl.name_span, code);
                        for member in &class_decl.members {
                            if let ClassMember::Enum(enum_decl) = member {
                                let enum_name = get_span_text(enum_decl.name_span, code);
                                let mut current_variant_value = 0u32;
                                for (variant_index, variant) in
                                    enum_decl.variants.iter().enumerate()
                                {
                                    let variant_name = get_span_text(variant.name_span, code);
                                    if let Some(explicit) = &variant.explicit_value {
                                        if let Some(integer_value) =
                                            evaluate_integer_constant(explicit)
                                        {
                                            current_variant_value = integer_value as u32;
                                        }
                                    } else if variant_index > 0 {
                                        current_variant_value += 1;
                                    }
                                    let full_symbol =
                                        format!("{class_name}::{enum_name}::{variant_name}");
                                    enum_variants.insert(full_symbol, current_variant_value);
                                    enum_variants.insert(
                                        format!("{enum_name}::{variant_name}"),
                                        current_variant_value,
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 4. Compile functions for each source file
    for source_id in 0..sources_count as u16 {
        if let Some(ast) = context.ast.get(source_id as usize).and_then(|a| a.as_ref()) {
            compile_source_functions(
                ast,
                &context.sources,
                &class_layouts,
                &function_ids,
                &enum_variants,
                &static_layout,
                &node_types,
                &mut pool,
                &mut functions_by_source[source_id as usize],
                &mut synthesized_lambdas,
                &generic_invocations,
            )?;
        }
    }

    let mut monomorphized_functions = Vec::new();
    if let Some(artifacts) = &context.binding_artifacts {
        for mono_cls in &artifacts.monomorphized_classes {
            compile_monomorphized_class_functions(
                mono_cls,
                &artifacts.symbol_table,
                &context.sources,
                &class_layouts,
                &function_ids,
                &enum_variants,
                &static_layout,
                &node_types,
                &mut pool,
                &mut monomorphized_functions,
                &mut synthesized_lambdas,
            )?;
        }
    }

    // 5. Compile class descriptors
    let mut compiled_classes = Vec::new();
    let mut sym_to_func = HashMap::<SymbolId, u32>::new();
    if let Some(artifacts) = &context.binding_artifacts {
        for entry in &artifacts.symbol_table.symbols {
            let signature = if let Type::FunctionPointer {
                parameter_types, ..
            } = &entry.symbol_type
            {
                Some(
                    parameter_types
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                )
            } else {
                None
            };
            let mangled_name = signature
                .as_ref()
                .map(|parameters| format!("{}({parameters})", entry.fully_qualified_name));
            if let Some(func_id) = mangled_name
                .as_ref()
                .and_then(|name| function_ids.get(name))
                .or_else(|| function_ids.get(&entry.fully_qualified_name))
                .copied()
            {
                sym_to_func.insert(entry.id, func_id);
            } else if let Some(parent_id) = entry.parent_symbol
                && let Some(parent) = artifacts.symbol_table.get_symbol(parent_id)
            {
                let short_full = format!("{}::{}", parent.name, entry.name);
                let short_mangled = signature
                    .as_ref()
                    .map(|parameters| format!("{short_full}({parameters})"));
                if let Some(func_id) = short_mangled
                    .as_ref()
                    .and_then(|name| function_ids.get(name))
                    .or_else(|| function_ids.get(&short_full))
                    .copied()
                {
                    sym_to_func.insert(entry.id, func_id);
                }
            }
        }
    }

    if let Some(artifacts) = &context.binding_artifacts {
        let mut iface_to_id = HashMap::new();
        for layout in artifacts.layouts.values() {
            let simple = layout
                .class_name
                .rsplit(['.', ':'])
                .next()
                .unwrap_or(&layout.class_name);
            iface_to_id.insert(layout.class_name.clone(), layout.vtable_id);
            iface_to_id.insert(simple.to_string(), layout.vtable_id);
        }

        for layout in artifacts.layouts.values() {
            if let Some(entry) = artifacts.symbol_table.get_symbol(layout.class_symbol_id)
                && entry.modifiers.contains(&Modifier::Extern)
            {
                continue;
            }
            let class_name = &layout.class_name;
            let simple = class_name.rsplit(['.', ':']).next().unwrap_or(class_name);
            let dtor_sym = format!("{class_name}::~destructor");
            let dtor_sym_simple = format!("{simple}::~destructor");
            let dtor_id = function_ids
                .get(&dtor_sym)
                .or_else(|| function_ids.get(&dtor_sym_simple))
                .copied();
            compiled_classes.push(compile_class_layout(
                layout,
                &sym_to_func,
                &iface_to_id,
                dtor_id,
            ));
        }
    }

    // 6. Assemble complete flat functions list for the binary container
    let mut all_functions = Vec::new();
    all_functions.push(init_func);

    for file_funcs in &functions_by_source {
        all_functions.extend(file_funcs.clone());
    }
    all_functions.extend(monomorphized_functions);
    all_functions.extend(synthesized_lambdas);

    // 7. Collect public export manifest and metadata
    let mut exported_classes = Vec::new();
    let mut exported_enums = Vec::new();
    let mut exported_functions = Vec::new();
    let mut exported_statics = Vec::new();
    let mut generic_templates = Vec::new();
    let dummy_sym_table = SymbolTable::with_sources_count(0);
    let sym_table = context
        .binding_artifacts
        .as_ref()
        .map(|ba| &ba.symbol_table)
        .unwrap_or(&dummy_sym_table);

    for (source_index, ast_opt) in context.ast.iter().enumerate() {
        let source_id = source_index as u16;
        if context.is_external_source(source_id) {
            continue;
        }
        if let Some(ast) = ast_opt {
            let code = &context.sources[source_index];
            for decl in &ast.declarations {
                match decl {
                    Declaration::Class(cls) => {
                        if cls.modifiers.contains(&Modifier::Extern) {
                            continue;
                        }
                        let cls_name = get_span_text(cls.name_span, code);
                        if !cls.generic_params.is_empty() {
                            let template_src = get_span_text(cls.span, code).to_string();
                            let param_names = cls
                                .generic_params
                                .iter()
                                .map(|p| get_span_text(p.name_span, code).to_string())
                                .collect();
                            generic_templates.push(ExportedGenericTemplate {
                                name: cls_name.to_string(),
                                parameter_names: param_names,
                                template_source: template_src,
                            });
                        } else {
                            let mut fields = Vec::new();
                            let mut methods = Vec::new();

                            let layout_opt = context.binding_artifacts.as_ref().and_then(|ba| {
                                ba.layouts.values().find(|l| l.class_name == cls_name)
                            });

                            for member in &cls.members {
                                match member {
                                    ClassMember::Field(f) => {
                                        let field_name =
                                            get_span_text(f.name_span, code).to_string();
                                        let f_type = resolve_type_annotation(
                                            ast.source_id,
                                            code,
                                            &f.field_type,
                                            sym_table,
                                        );
                                        let offset = layout_opt
                                            .and_then(|l| l.field_offsets.get(&field_name))
                                            .copied()
                                            .unwrap_or(8);
                                        let mods = f
                                            .modifiers
                                            .iter()
                                            .filter_map(|&modifier| to_decl_modifier(modifier))
                                            .collect();
                                        fields.push(ExportedField {
                                            name: field_name,
                                            field_type: f_type,
                                            modifiers: mods,
                                            offset,
                                        });
                                    }
                                    ClassMember::Method(method_declaration) => {
                                        let method_name =
                                            get_span_text(method_declaration.name_span, code)
                                                .to_string();

                                        // Skip private methods — they are implementation details
                                        // and must not appear in the extern class header.
                                        if method_declaration.modifiers.contains(&Modifier::Private)
                                        {
                                            continue;
                                        }

                                        // Skip destructor-named methods from extern headers;
                                        // destructors are not callable from extern code.
                                        if method_name.starts_with('~') {
                                            continue;
                                        }

                                        // Generic or variadic methods cannot be expressed as
                                        // bodiless extern prototypes — skip them from the class
                                        // stub and emit the whole class source as a generic
                                        // template so callers can use them via monomorphization.
                                        let has_variadic_param = method_declaration
                                            .parameters
                                            .iter()
                                            .any(|param| param.is_variadic);
                                        if !method_declaration.generic_params.is_empty()
                                            || has_variadic_param
                                        {
                                            continue;
                                        }

                                        let full_sym = format!("{cls_name}::{method_name}");
                                        let ret_type = resolve_type_annotation(
                                            ast.source_id,
                                            code,
                                            &method_declaration.return_type,
                                            sym_table,
                                        );
                                        let params = method_declaration
                                            .parameters
                                            .iter()
                                            .map(|param| {
                                                let param_name =
                                                    get_span_text(param.name_span, code)
                                                        .to_string();
                                                let param_type = resolve_type_annotation(
                                                    ast.source_id,
                                                    code,
                                                    &param.param_type,
                                                    sym_table,
                                                );
                                                ExportedParameter {
                                                    name: param_name,
                                                    param_type,
                                                }
                                            })
                                            .collect();
                                        let mods = method_declaration
                                            .modifiers
                                            .iter()
                                            .filter_map(|&modifier| to_decl_modifier(modifier))
                                            .collect();
                                        methods.push(ExportedFunction {
                                            symbol: full_sym,
                                            simple_name: method_name,
                                            modifiers: mods,
                                            parameters: params,
                                            return_type: ret_type,
                                            is_constructor: false,
                                            is_destructor: false,
                                            is_operator: false,
                                        });
                                    }
                                    ClassMember::Constructor(ctor) => {
                                        let arity = ctor.parameters.len();
                                        let full_sym = format!("{cls_name}::constructor/{arity}");
                                        let params = ctor
                                            .parameters
                                            .iter()
                                            .map(|p| {
                                                let p_name =
                                                    get_span_text(p.name_span, code).to_string();
                                                let p_type = resolve_type_annotation(
                                                    ast.source_id,
                                                    code,
                                                    &p.param_type,
                                                    sym_table,
                                                );
                                                ExportedParameter {
                                                    name: p_name,
                                                    param_type: p_type,
                                                }
                                            })
                                            .collect();
                                        let mods = ctor
                                            .modifiers
                                            .iter()
                                            .filter_map(|&modifier| to_decl_modifier(modifier))
                                            .collect();
                                        methods.push(ExportedFunction {
                                            symbol: full_sym,
                                            simple_name: "constructor".to_string(),
                                            modifiers: mods,
                                            parameters: params,
                                            return_type: Type::Primitive(PrimitiveKind::Void),
                                            is_constructor: true,
                                            is_destructor: false,
                                            is_operator: false,
                                        });
                                    }
                                    ClassMember::Destructor(_) => {
                                        // Destructors are implementation details and must not be
                                        // exported into extern class headers — they are not
                                        // callable from external code and would cause syntax
                                        // errors if emitted as bodiless extern declarations.
                                    }
                                    ClassMember::Operator(operator_declaration) => {
                                        let operator_text =
                                            get_span_text(operator_declaration.operator_span, code)
                                                .to_string();
                                        let arity = operator_declaration.parameters.len();
                                        let full_sym =
                                            format!("{cls_name}::operator{operator_text}/{arity}");
                                        let ret_type = resolve_type_annotation(
                                            ast.source_id,
                                            code,
                                            &operator_declaration.return_type,
                                            sym_table,
                                        );
                                        let params = operator_declaration
                                            .parameters
                                            .iter()
                                            .map(|param| {
                                                let param_name =
                                                    get_span_text(param.name_span, code)
                                                        .to_string();
                                                let param_type = resolve_type_annotation(
                                                    ast.source_id,
                                                    code,
                                                    &param.param_type,
                                                    sym_table,
                                                );
                                                ExportedParameter {
                                                    name: param_name,
                                                    param_type,
                                                }
                                            })
                                            .collect();
                                        let mods = operator_declaration
                                            .modifiers
                                            .iter()
                                            .filter_map(|&modifier| to_decl_modifier(modifier))
                                            .collect();
                                        methods.push(ExportedFunction {
                                            symbol: full_sym,
                                            simple_name: format!("operator{operator_text}"),
                                            modifiers: mods,
                                            parameters: params,
                                            return_type: ret_type,
                                            is_constructor: false,
                                            is_destructor: false,
                                            is_operator: true,
                                        });
                                    }
                                    _ => {}
                                }
                            }

                            let base_class = cls
                                .super_class
                                .as_ref()
                                .map(|sc| get_span_text(sc.span(), code).to_string());
                            let ifaces = cls
                                .interfaces
                                .iter()
                                .map(|iface| get_span_text(iface.span(), code).to_string())
                                .collect();
                            let mods = cls
                                .modifiers
                                .iter()
                                .filter_map(|&modifier| to_decl_modifier(modifier))
                                .collect();

                            exported_classes.push(ExportedClass {
                                symbol: cls_name.to_string(),
                                name: cls_name.to_string(),
                                modifiers: mods,
                                is_interface: false,
                                base_class,
                                interfaces: ifaces,
                                generic_parameters: Vec::new(),
                                fields,
                                methods,
                            });
                        }
                    }
                    Declaration::Method(m) => {
                        if m.modifiers.contains(&Modifier::Extern) {
                            continue;
                        }
                        // Skip private top-level functions from the export manifest.
                        if m.modifiers.contains(&Modifier::Private) {
                            continue;
                        }
                        let func_name = get_span_text(m.name_span, code).to_string();
                        let arity = m.parameters.len();
                        let full_sym = format!("{func_name}/{arity}");
                        let ret_type =
                            resolve_type_annotation(ast.source_id, code, &m.return_type, sym_table);
                        let params = m
                            .parameters
                            .iter()
                            .map(|p| {
                                let p_name = get_span_text(p.name_span, code).to_string();
                                let p_type = resolve_type_annotation(
                                    ast.source_id,
                                    code,
                                    &p.param_type,
                                    sym_table,
                                );
                                ExportedParameter {
                                    name: p_name,
                                    param_type: p_type,
                                }
                            })
                            .collect();
                        let mods = m
                            .modifiers
                            .iter()
                            .filter_map(|&modifier| to_decl_modifier(modifier))
                            .collect();
                        exported_functions.push(ExportedFunction {
                            symbol: full_sym,
                            simple_name: func_name,
                            modifiers: mods,
                            parameters: params,
                            return_type: ret_type,
                            is_constructor: false,
                            is_destructor: false,
                            is_operator: false,
                        });
                    }
                    Declaration::Field(f) => {
                        if f.modifiers.contains(&Modifier::Extern) {
                            continue;
                        }
                        let var_name = get_span_text(f.name_span, code).to_string();
                        let var_type =
                            resolve_type_annotation(ast.source_id, code, &f.field_type, sym_table);
                        let slot = static_layout.lookup_slot(&var_name).unwrap_or(0);
                        let mods = f
                            .modifiers
                            .iter()
                            .filter_map(|&modifier| to_decl_modifier(modifier))
                            .collect();
                        exported_statics.push(ExportedStatic {
                            symbol: var_name.clone(),
                            name: var_name,
                            var_type,
                            modifiers: mods,
                            slot,
                        });
                    }
                    Declaration::Interface(interface_declaration) => {
                        if interface_declaration.modifiers.contains(&Modifier::Extern) {
                            continue;
                        }
                        let iface_name =
                            get_span_text(interface_declaration.name_span, code).to_string();

                        // Generic interfaces (e.g. IIterator<T>, IMap<K,V>) cannot be represented
                        // as non-generic extern class stubs — emit their full source as a generic
                        // template so downstream consumers can monomorphize them correctly.
                        if !interface_declaration.generic_params.is_empty() {
                            let template_source =
                                get_span_text(interface_declaration.span, code).to_string();
                            let parameter_names = interface_declaration
                                .generic_params
                                .iter()
                                .map(|generic_param| {
                                    get_span_text(generic_param.name_span, code).to_string()
                                })
                                .collect();
                            generic_templates.push(ExportedGenericTemplate {
                                name: iface_name,
                                parameter_names,
                                template_source,
                            });
                            continue;
                        }

                        let mut methods = Vec::new();
                        for method_declaration in &interface_declaration.methods {
                            let func_name =
                                get_span_text(method_declaration.name_span, code).to_string();
                            let arity = method_declaration.parameters.len();
                            let full_sym = format!("{iface_name}::{func_name}/{arity}");
                            let ret_type = resolve_type_annotation(
                                ast.source_id,
                                code,
                                &method_declaration.return_type,
                                sym_table,
                            );
                            let params = method_declaration
                                .parameters
                                .iter()
                                .map(|param| {
                                    let param_name =
                                        get_span_text(param.name_span, code).to_string();
                                    let param_type = resolve_type_annotation(
                                        ast.source_id,
                                        code,
                                        &param.param_type,
                                        sym_table,
                                    );
                                    ExportedParameter {
                                        name: param_name,
                                        param_type,
                                    }
                                })
                                .collect();
                            let mods = method_declaration
                                .modifiers
                                .iter()
                                .filter_map(|&modifier| to_decl_modifier(modifier))
                                .collect();
                            methods.push(ExportedFunction {
                                symbol: full_sym,
                                simple_name: func_name,
                                modifiers: mods,
                                parameters: params,
                                return_type: ret_type,
                                is_constructor: false,
                                is_destructor: false,
                                is_operator: false,
                            });
                        }
                        let mods = interface_declaration
                            .modifiers
                            .iter()
                            .filter_map(|&modifier| to_decl_modifier(modifier))
                            .collect();
                        exported_classes.push(ExportedClass {
                            symbol: iface_name.clone(),
                            name: iface_name,
                            modifiers: mods,
                            is_interface: true,
                            base_class: None,
                            interfaces: Vec::new(),
                            generic_parameters: Vec::new(),
                            fields: Vec::new(),
                            methods,
                        });
                    }
                    Declaration::Enum(enum_declaration) => {
                        if enum_declaration.modifiers.contains(&Modifier::Extern) {
                            continue;
                        }
                        let enum_name = get_span_text(enum_declaration.name_span, code).to_string();
                        let mut variants = Vec::new();
                        for variant in &enum_declaration.variants {
                            let variant_name = get_span_text(variant.name_span, code).to_string();
                            let explicit_disc =
                                variant.explicit_value.as_ref().and_then(|expr| match expr {
                                    Expression::Literal {
                                        value: LiteralValue::Int(val),
                                        ..
                                    } => Some(*val),
                                    _ => None,
                                });
                            variants.push((variant_name, explicit_disc));
                        }
                        let mods = enum_declaration
                            .modifiers
                            .iter()
                            .filter_map(|&modifier| to_decl_modifier(modifier))
                            .collect();
                        exported_enums.push(ExportedEnum {
                            name: enum_name,
                            modifiers: mods,
                            variants,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    let constant_pool = pool.finish();
    let mut program = BinaryProgram::new();
    program.functions = all_functions;
    program.classes = compiled_classes;
    program.static_space_size = static_layout.static_space_size;
    program.static_symbols_map = static_layout.static_symbols_map;
    program.constant_pool = constant_pool;
    for (function_index, function) in program.functions.iter().enumerate() {
        if function.symbol == "main" || function.symbol.ends_with("::main") {
            program.entry_point_id = Some(function_index as u32);
            program.entry_point_symbol = Some(function.symbol.clone());
            break;
        }
    }
    program.package_name = context.options.package_name.clone();
    program.package_version = context.options.package_version.clone();
    program.package_description = context.options.package_description.clone();

    let mut dependencies: Vec<solix_common::solver::DependencyRequirement> =
        context.dependencies_map.values().cloned().collect();
    dependencies
        .sort_by(|requirement_a, requirement_b| requirement_a.name.cmp(&requirement_b.name));

    let (api_header, exported_ast_bytes) =
        crate::phases::export_api::generate_api_header_and_ast(context);

    program.export_manifest = ExportManifest {
        package_name: program.package_name.clone(),
        package_version: program.package_version.clone(),
        package_description: program.package_description.clone(),
        dependencies,
        exported_classes,
        exported_enums,
        exported_functions,
        exported_statics,
        generic_templates,
        api_header,
        exported_ast_bytes,
    };

    Ok(EmissionArtifacts {
        functions_by_source,
        program,
    })
}

fn collect_static_declarations(
    ast: &ProgramAst,
    sources: &[String],
    decls: &mut Vec<StaticVariableDecl>,
) {
    let src_idx = ast.source_id as usize;
    if src_idx >= sources.len() {
        return;
    }
    let code = &sources[src_idx];

    for decl in &ast.declarations {
        match decl {
            Declaration::Class(cls) => {
                if cls.modifiers.contains(&Modifier::Extern) {
                    continue;
                }
                let cls_name = get_span_text(cls.name_span, code);
                for member in &cls.members {
                    if let ClassMember::Field(field) = member
                        && !field.modifiers.contains(&Modifier::Extern)
                        && (field.modifiers.contains(&Modifier::Static)
                            || field.modifiers.contains(&Modifier::Const))
                    {
                        let field_name = get_span_text(field.name_span, code);
                        let full_sym = format!("{cls_name}::{field_name}");
                        decls.push(StaticVariableDecl {
                            symbol: full_sym,
                            span: field.span,
                            initializer: field.initializer.clone(),
                        });
                    }
                }
            }
            Declaration::Field(field) => {
                if field.modifiers.contains(&Modifier::Extern) {
                    continue;
                }
                let field_name = get_span_text(field.name_span, code);
                decls.push(StaticVariableDecl {
                    symbol: field_name.to_string(),
                    span: field.span,
                    initializer: field.initializer.clone(),
                });
            }
            _ => {}
        }
    }
}

fn collect_classes(declarations: &[Declaration]) -> Vec<&ClassDeclaration> {
    let mut classes = Vec::new();
    for decl in declarations {
        if let Declaration::Class(cls) = decl {
            classes.push(cls);
            for member in &cls.members {
                if let ClassMember::Class(nested) = member {
                    classes.push(nested);
                }
            }
        }
    }
    classes
}

fn collect_generic_calls_in_expression(
    expression: &Expression,
    sources: &[String],
    invocations: &mut HashMap<(String, String), Vec<String>>,
) {
    match expression {
        Expression::MethodCall {
            callee,
            generic_arguments,
            arguments,
            ..
        } => {
            if !generic_arguments.is_empty() {
                let argument_strings: Vec<String> = generic_arguments
                    .iter()
                    .map(|argument| {
                        get_span_text(argument.span(), &sources[argument.span().id as usize])
                            .to_string()
                    })
                    .collect();
                let joined_arguments = argument_strings.join(", ");
                match callee.as_ref() {
                    Expression::MemberAccess {
                        object,
                        member_span,
                        ..
                    } => {
                        let object_name =
                            get_span_text(object.span(), &sources[object.span().id as usize]);
                        let method_name =
                            get_span_text(*member_span, &sources[member_span.id as usize]);
                        let entry = invocations
                            .entry((object_name.to_string(), method_name.to_string()))
                            .or_default();
                        if !entry.contains(&joined_arguments) {
                            entry.push(joined_arguments);
                        }
                    }
                    Expression::Identifier { span } => {
                        let method_name = get_span_text(*span, &sources[span.id as usize]);
                        let entry = invocations
                            .entry((String::new(), method_name.to_string()))
                            .or_default();
                        if !entry.contains(&joined_arguments) {
                            entry.push(joined_arguments);
                        }
                    }
                    _ => {}
                }
            }
            collect_generic_calls_in_expression(callee, sources, invocations);
            for argument in arguments {
                collect_generic_calls_in_expression(argument, sources, invocations);
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_generic_calls_in_expression(left, sources, invocations);
            collect_generic_calls_in_expression(right, sources, invocations);
        }
        Expression::Unary { operand, .. } => {
            collect_generic_calls_in_expression(operand, sources, invocations);
        }
        Expression::Assignment { target, value, .. } => {
            collect_generic_calls_in_expression(target, sources, invocations);
            collect_generic_calls_in_expression(value, sources, invocations);
        }
        Expression::MemberAccess { object, .. } => {
            collect_generic_calls_in_expression(object, sources, invocations);
        }
        Expression::ArrayAccess { array, index, .. } => {
            collect_generic_calls_in_expression(array, sources, invocations);
            collect_generic_calls_in_expression(index, sources, invocations);
        }
        Expression::ArrayCreation { size, .. } => {
            collect_generic_calls_in_expression(size, sources, invocations);
        }
        Expression::ArrayLiteral { elements, .. } | Expression::TupleCreation { elements, .. } => {
            for element in elements {
                collect_generic_calls_in_expression(element, sources, invocations);
            }
        }
        Expression::NewInstance { arguments, .. } => {
            for argument in arguments {
                collect_generic_calls_in_expression(argument, sources, invocations);
            }
        }
        Expression::Cast {
            expression: inner, ..
        }
        | Expression::Instanceof {
            expression: inner, ..
        } => {
            collect_generic_calls_in_expression(inner, sources, invocations);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_generic_calls_in_expression(condition, sources, invocations);
            collect_generic_calls_in_expression(then_branch, sources, invocations);
            collect_generic_calls_in_expression(else_branch, sources, invocations);
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements {
                collect_generic_calls_in_statement(statement, sources, invocations);
            }
            if let Some(result_expression) = result {
                collect_generic_calls_in_expression(result_expression, sources, invocations);
            }
        }
        _ => {}
    }
}

fn collect_generic_calls_in_statement(
    statement: &Statement,
    sources: &[String],
    invocations: &mut HashMap<(String, String), Vec<String>>,
) {
    match statement {
        Statement::Block(block_statement) => {
            for nested in &block_statement.statements {
                collect_generic_calls_in_statement(nested, sources, invocations);
            }
        }
        Statement::If(if_statement) => {
            collect_generic_calls_in_expression(&if_statement.condition, sources, invocations);
            collect_generic_calls_in_statement(&if_statement.then_branch, sources, invocations);
            if let Some(ref else_branch) = if_statement.else_branch {
                collect_generic_calls_in_statement(else_branch, sources, invocations);
            }
        }
        Statement::While(while_statement) => {
            collect_generic_calls_in_expression(&while_statement.condition, sources, invocations);
            collect_generic_calls_in_statement(&while_statement.body, sources, invocations);
        }
        Statement::DoWhile(do_while_statement) => {
            collect_generic_calls_in_statement(&do_while_statement.body, sources, invocations);
            collect_generic_calls_in_expression(
                &do_while_statement.condition,
                sources,
                invocations,
            );
        }
        Statement::For(for_statement) => {
            if let Some(ref initializer) = for_statement.initializer {
                collect_generic_calls_in_statement(initializer, sources, invocations);
            }
            if let Some(ref condition) = for_statement.condition {
                collect_generic_calls_in_expression(condition, sources, invocations);
            }
            if let Some(ref update) = for_statement.update {
                collect_generic_calls_in_expression(update, sources, invocations);
            }
            collect_generic_calls_in_statement(&for_statement.body, sources, invocations);
        }
        Statement::Switch(switch_statement) => {
            collect_generic_calls_in_expression(
                &switch_statement.discriminant,
                sources,
                invocations,
            );
            for switch_case in &switch_statement.cases {
                collect_generic_calls_in_expression(&switch_case.case_value, sources, invocations);
                for case_statement in &switch_case.statements {
                    collect_generic_calls_in_statement(case_statement, sources, invocations);
                }
            }
        }
        Statement::Try(try_statement) => {
            for try_stmt in &try_statement.try_block.statements {
                collect_generic_calls_in_statement(try_stmt, sources, invocations);
            }
            for catch_clause in &try_statement.catch_clauses {
                for catch_stmt in &catch_clause.block.statements {
                    collect_generic_calls_in_statement(catch_stmt, sources, invocations);
                }
            }
            if let Some(ref finally_block) = try_statement.finally_block {
                for finally_stmt in &finally_block.statements {
                    collect_generic_calls_in_statement(finally_stmt, sources, invocations);
                }
            }
        }
        Statement::Throw(throw_statement) => {
            collect_generic_calls_in_expression(&throw_statement.expression, sources, invocations);
        }
        Statement::Return(return_statement) => {
            if let Some(ref value) = return_statement.value {
                collect_generic_calls_in_expression(value, sources, invocations);
            }
        }
        Statement::VariableDeclaration(variable_statement) => {
            if let Some(ref initializer) = variable_statement.initializer {
                collect_generic_calls_in_expression(initializer, sources, invocations);
            }
        }
        Statement::Expression(expression_statement) => {
            collect_generic_calls_in_expression(
                &expression_statement.expression,
                sources,
                invocations,
            );
        }
        Statement::Assert(assert_statement) => {
            collect_generic_calls_in_expression(&assert_statement.condition, sources, invocations);
            if let Some(ref message) = assert_statement.message {
                collect_generic_calls_in_expression(message, sources, invocations);
            }
        }
        Statement::Exit(exit_statement) => {
            collect_generic_calls_in_expression(&exit_statement.code, sources, invocations);
        }
        Statement::Declaration(declaration) => match declaration {
            Declaration::Method(method_declaration) => {
                if let Some(ref body) = method_declaration.body {
                    for stmt in &body.statements {
                        collect_generic_calls_in_statement(stmt, sources, invocations);
                    }
                }
            }
            Declaration::Class(class_declaration) => {
                for member in &class_declaration.members {
                    match member {
                        ClassMember::Method(method_declaration) => {
                            if let Some(ref body) = method_declaration.body {
                                for stmt in &body.statements {
                                    collect_generic_calls_in_statement(stmt, sources, invocations);
                                }
                            }
                        }
                        ClassMember::Constructor(constructor_declaration) => {
                            for stmt in &constructor_declaration.body.statements {
                                collect_generic_calls_in_statement(stmt, sources, invocations);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn collect_generic_calls_in_ast(
    ast: &ProgramAst,
    sources: &[String],
    invocations: &mut HashMap<(String, String), Vec<String>>,
) {
    for declaration in &ast.declarations {
        match declaration {
            Declaration::Method(method_declaration) => {
                if let Some(ref body) = method_declaration.body {
                    for statement in &body.statements {
                        collect_generic_calls_in_statement(statement, sources, invocations);
                    }
                }
            }
            Declaration::Class(class_declaration) => {
                for member in &class_declaration.members {
                    match member {
                        ClassMember::Method(method_declaration) => {
                            if let Some(ref body) = method_declaration.body {
                                for statement in &body.statements {
                                    collect_generic_calls_in_statement(
                                        statement,
                                        sources,
                                        invocations,
                                    );
                                }
                            }
                        }
                        ClassMember::Constructor(constructor_declaration) => {
                            for statement in &constructor_declaration.body.statements {
                                collect_generic_calls_in_statement(statement, sources, invocations);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn index_function_symbols(
    ast: &ProgramAst,
    sources: &[String],
    function_ids: &mut HashMap<String, u32>,
    next_func_id: &mut u32,
    generic_invocations: &HashMap<(String, String), Vec<String>>,
) {
    let source_index = ast.source_id as usize;
    if source_index >= sources.len() {
        return;
    }
    let source_code = &sources[source_index];

    let raw_file_package = ast
        .package
        .as_ref()
        .map(|pkg| {
            pkg.path
                .iter()
                .map(|span| get_span_text(*span, source_code))
                .collect::<Vec<_>>()
                .join("::")
        })
        .unwrap_or_default();

    for declaration in &ast.declarations {
        if let Declaration::Method(method_declaration) = declaration {
            if method_declaration.modifiers.contains(&Modifier::Extern) {
                continue;
            }
            let method_name = get_span_text(method_declaration.name_span, source_code);
            let decl_namespace =
                extract_namespace_path(&method_declaration.namespace_path, source_code);
            let effective_package = compute_effective_package(&raw_file_package, &decl_namespace);

            let func_id = *next_func_id;
            *next_func_id += 1;
            function_ids.insert(method_name.to_string(), func_id);
            if !effective_package.is_empty() {
                let full_colon_name = format!("{effective_package}::{method_name}");
                let full_dot_name =
                    format!("{}.{method_name}", effective_package.replace("::", "."));
                function_ids.insert(full_colon_name, func_id);
                function_ids.insert(full_dot_name, func_id);
            }
            if !method_declaration.generic_params.is_empty()
                && let Some(specialization_args) =
                    generic_invocations.get(&(String::new(), method_name.to_string()))
            {
                for args_str in specialization_args {
                    let specialized_sym = format!("{method_name}<{args_str}>");
                    let spec_id = *next_func_id;
                    *next_func_id += 1;
                    function_ids.insert(specialized_sym.clone(), spec_id);
                }
            }
        }
    }

    for class_declaration in collect_classes(&ast.declarations) {
        if class_declaration.modifiers.contains(&Modifier::Extern) {
            continue;
        }
        let class_name = get_span_text(class_declaration.name_span, source_code);
        let decl_namespace = extract_namespace_path(&class_declaration.namespace_path, source_code);
        let effective_package = compute_effective_package(&raw_file_package, &decl_namespace);
        let qualified_class_name = if effective_package.is_empty() {
            class_name.to_string()
        } else {
            format!("{effective_package}::{class_name}")
        };
        let dot_class_name = if effective_package.is_empty() {
            class_name.to_string()
        } else {
            format!("{}.{class_name}", effective_package.replace("::", "."))
        };

        for member in &class_declaration.members {
            match member {
                ClassMember::Method(method_declaration) => {
                    if method_declaration.modifiers.contains(&Modifier::Extern) {
                        continue;
                    }
                    let method_name = get_span_text(method_declaration.name_span, source_code);
                    let param_sig = method_declaration
                        .parameters
                        .iter()
                        .map(|parameter| get_span_text(parameter.param_type.span(), source_code))
                        .collect::<Vec<_>>()
                        .join(",");
                    let full_sym_mangled = if param_sig.is_empty() {
                        format!("{class_name}::{method_name}")
                    } else {
                        format!("{class_name}::{method_name}({param_sig})")
                    };
                    let full_sym = format!("{class_name}::{method_name}");
                    let func_id = *next_func_id;
                    *next_func_id += 1;
                    function_ids.insert(full_sym_mangled.clone(), func_id);
                    function_ids.entry(full_sym.clone()).or_insert(func_id);

                    if !effective_package.is_empty() {
                        let qualified_sym_mangled = if param_sig.is_empty() {
                            format!("{qualified_class_name}::{method_name}")
                        } else {
                            format!("{qualified_class_name}::{method_name}({param_sig})")
                        };
                        let qualified_sym = format!("{qualified_class_name}::{method_name}");
                        function_ids.insert(qualified_sym_mangled, func_id);
                        function_ids.entry(qualified_sym).or_insert(func_id);

                        let dot_sym_mangled = if param_sig.is_empty() {
                            format!("{dot_class_name}::{method_name}")
                        } else {
                            format!("{dot_class_name}::{method_name}({param_sig})")
                        };
                        let dot_sym = format!("{dot_class_name}::{method_name}");
                        function_ids.insert(dot_sym_mangled, func_id);
                        function_ids.entry(dot_sym).or_insert(func_id);
                    }

                    if !method_declaration.generic_params.is_empty()
                        && let Some(specialization_args) = generic_invocations
                            .get(&(class_name.to_string(), method_name.to_string()))
                    {
                        for args_str in specialization_args {
                            let specialized_sym =
                                format!("{class_name}::{method_name}<{args_str}>");
                            let spec_id = *next_func_id;
                            *next_func_id += 1;
                            function_ids.insert(specialized_sym.clone(), spec_id);
                            function_ids.insert(format!("{method_name}<{args_str}>"), spec_id);
                            if !effective_package.is_empty() {
                                function_ids.insert(
                                    format!("{qualified_class_name}::{method_name}<{args_str}>"),
                                    spec_id,
                                );
                            }
                        }
                    }
                }
                ClassMember::Constructor(constructor_declaration) => {
                    if constructor_declaration
                        .modifiers
                        .contains(&Modifier::Extern)
                    {
                        continue;
                    }
                    let arity = constructor_declaration.parameters.len();
                    let param_sig = constructor_declaration
                        .parameters
                        .iter()
                        .map(|parameter| get_span_text(parameter.param_type.span(), source_code))
                        .collect::<Vec<_>>()
                        .join(",");
                    let full_sym_mangled = if param_sig.is_empty() {
                        format!("{class_name}::constructor")
                    } else {
                        format!("{class_name}::constructor({param_sig})")
                    };
                    let full_sym_arity = format!("{class_name}::constructor/{arity}");
                    let full_sym = format!("{class_name}::constructor");
                    let func_id = *next_func_id;
                    *next_func_id += 1;
                    function_ids.insert(full_sym_mangled.clone(), func_id);
                    function_ids
                        .entry(full_sym_arity.clone())
                        .or_insert(func_id);
                    function_ids.entry(full_sym.clone()).or_insert(func_id);

                    if !effective_package.is_empty() {
                        let qualified_sym_mangled = if param_sig.is_empty() {
                            format!("{qualified_class_name}::constructor")
                        } else {
                            format!("{qualified_class_name}::constructor({param_sig})")
                        };
                        let qualified_sym_arity =
                            format!("{qualified_class_name}::constructor/{arity}");
                        let qualified_sym = format!("{qualified_class_name}::constructor");
                        function_ids.insert(qualified_sym_mangled, func_id);
                        function_ids.entry(qualified_sym_arity).or_insert(func_id);
                        function_ids.entry(qualified_sym).or_insert(func_id);

                        let dot_sym_mangled = if param_sig.is_empty() {
                            format!("{dot_class_name}::constructor")
                        } else {
                            format!("{dot_class_name}::constructor({param_sig})")
                        };
                        let dot_sym_arity = format!("{dot_class_name}::constructor/{arity}");
                        let dot_sym = format!("{dot_class_name}::constructor");
                        function_ids.insert(dot_sym_mangled, func_id);
                        function_ids.entry(dot_sym_arity).or_insert(func_id);
                        function_ids.entry(dot_sym).or_insert(func_id);
                    }
                }
                ClassMember::Destructor(_) => {
                    let full_sym = format!("{class_name}::~destructor");
                    let func_id = *next_func_id;
                    *next_func_id += 1;
                    function_ids.insert(full_sym, func_id);
                    if !effective_package.is_empty() {
                        let qualified_sym = format!("{qualified_class_name}::~destructor");
                        let dot_sym = format!("{dot_class_name}::~destructor");
                        function_ids.insert(qualified_sym, func_id);
                        function_ids.insert(dot_sym, func_id);
                    }
                }
                ClassMember::Operator(operator_declaration) => {
                    let op_str = operator_declaration.operator_token.as_static_str();
                    let param_sig = operator_declaration
                        .parameters
                        .iter()
                        .map(|parameter| get_span_text(parameter.param_type.span(), source_code))
                        .collect::<Vec<_>>()
                        .join(",");
                    let full_sym = if param_sig.is_empty() {
                        format!("{class_name}::operator_{op_str}")
                    } else {
                        format!("{class_name}::operator_{op_str}({param_sig})")
                    };
                    let unsuffixed_sym = format!("{class_name}::operator_{op_str}");
                    let func_id = *next_func_id;
                    *next_func_id += 1;
                    function_ids.insert(full_sym, func_id);
                    function_ids.entry(unsuffixed_sym).or_insert(func_id);
                    if !effective_package.is_empty() {
                        let qualified_sym = format!("{qualified_class_name}::operator_{op_str}");
                        let qualified_sym_mangled = if param_sig.is_empty() {
                            qualified_sym.clone()
                        } else {
                            format!("{qualified_sym}({param_sig})")
                        };
                        function_ids.insert(qualified_sym_mangled, func_id);
                        function_ids.entry(qualified_sym).or_insert(func_id);

                        let dot_sym = format!("{dot_class_name}::operator_{op_str}");
                        let dot_sym_mangled = if param_sig.is_empty() {
                            dot_sym.clone()
                        } else {
                            format!("{dot_sym}({param_sig})")
                        };
                        function_ids.insert(dot_sym_mangled, func_id);
                        function_ids.entry(dot_sym).or_insert(func_id);
                    }
                }
                _ => {}
            }
        }

        let has_explicit_ctor = class_declaration
            .members
            .iter()
            .any(|member| matches!(member, ClassMember::Constructor(_)));
        if !has_explicit_ctor {
            let full_sym_arity = format!("{class_name}::constructor/0");
            let full_sym = format!("{class_name}::constructor");
            let func_id = *next_func_id;
            *next_func_id += 1;
            function_ids.insert(full_sym_arity, func_id);
            function_ids.entry(full_sym).or_insert(func_id);
            if !effective_package.is_empty() {
                let qualified_sym_arity = format!("{qualified_class_name}::constructor/0");
                let qualified_sym = format!("{qualified_class_name}::constructor");
                function_ids.insert(qualified_sym_arity, func_id);
                function_ids.entry(qualified_sym).or_insert(func_id);

                let dot_sym_arity = format!("{dot_class_name}::constructor/0");
                let dot_sym = format!("{dot_class_name}::constructor");
                function_ids.insert(dot_sym_arity, func_id);
                function_ids.entry(dot_sym).or_insert(func_id);
            }
        }
    }

    if !ast.root_statements.is_empty() {
        let entry_sym = format!("<entry_{}>", ast.source_id);
        let func_id = *next_func_id;
        *next_func_id += 1;
        function_ids.insert(entry_sym, func_id);
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_source_functions(
    ast: &ProgramAst,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    output: &mut Vec<CompiledFunction>,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
    generic_invocations: &HashMap<(String, String), Vec<String>>,
) -> Result<(), EmissionError> {
    let src_idx = ast.source_id as usize;
    if src_idx >= sources.len() {
        return Ok(());
    }
    let code = &sources[src_idx];

    for decl in &ast.declarations {
        if let Declaration::Method(m) = decl {
            if m.modifiers.contains(&Modifier::Extern) {
                continue;
            }
            let name = get_span_text(m.name_span, code).to_string();
            let func = compile_single_method(
                &name,
                ast.source_id,
                m,
                false,
                sources,
                class_layouts,
                function_ids,
                enum_variants,
                static_layout,
                node_types,
                pool,
                synthesized_lambdas,
            )?;
            output.push(func);

            if !m.generic_params.is_empty()
                && let Some(specialization_args) =
                    generic_invocations.get(&(String::new(), name.clone()))
            {
                for args_str in specialization_args {
                    let specialized_sym = format!("{name}<{args_str}>");
                    let func = compile_single_method(
                        &specialized_sym,
                        ast.source_id,
                        m,
                        false,
                        sources,
                        class_layouts,
                        function_ids,
                        enum_variants,
                        static_layout,
                        node_types,
                        pool,
                        synthesized_lambdas,
                    )?;
                    output.push(func);
                }
            }
        }
    }

    for cls in collect_classes(&ast.declarations) {
        if cls.modifiers.contains(&Modifier::Extern) {
            continue;
        }
        let cls_name = get_span_text(cls.name_span, code);
        for member in &cls.members {
            match member {
                ClassMember::Method(m) => {
                    if m.modifiers.contains(&Modifier::Extern) {
                        continue;
                    }
                    let method_name = get_span_text(m.name_span, code);
                    let param_sig = m
                        .parameters
                        .iter()
                        .map(|p| get_span_text(p.param_type.span(), code))
                        .collect::<Vec<_>>()
                        .join(",");
                    let full_sym = if param_sig.is_empty() {
                        format!("{cls_name}::{method_name}")
                    } else {
                        format!("{cls_name}::{method_name}({param_sig})")
                    };
                    let is_instance = !m.modifiers.contains(&Modifier::Static);
                    let func = compile_single_method(
                        &full_sym,
                        ast.source_id,
                        m,
                        is_instance,
                        sources,
                        class_layouts,
                        function_ids,
                        enum_variants,
                        static_layout,
                        node_types,
                        pool,
                        synthesized_lambdas,
                    )?;
                    output.push(func);

                    if !m.generic_params.is_empty()
                        && let Some(specialization_args) = generic_invocations
                            .get(&(cls_name.to_string(), method_name.to_string()))
                    {
                        for args_str in specialization_args {
                            let specialized_sym = format!("{cls_name}::{method_name}<{args_str}>");
                            let func = compile_single_method(
                                &specialized_sym,
                                ast.source_id,
                                m,
                                is_instance,
                                sources,
                                class_layouts,
                                function_ids,
                                enum_variants,
                                static_layout,
                                node_types,
                                pool,
                                synthesized_lambdas,
                            )?;
                            output.push(func);
                        }
                    }
                }
                ClassMember::Constructor(ctor) => {
                    if ctor.modifiers.contains(&Modifier::Extern) {
                        continue;
                    }
                    let param_sig = ctor
                        .parameters
                        .iter()
                        .map(|parameter| get_span_text(parameter.param_type.span(), code))
                        .collect::<Vec<_>>()
                        .join(",");
                    let full_sym = if param_sig.is_empty() {
                        format!("{cls_name}::constructor")
                    } else {
                        format!("{cls_name}::constructor({param_sig})")
                    };
                    let func = compile_constructor(
                        &full_sym,
                        ast.source_id,
                        ctor,
                        sources,
                        class_layouts,
                        function_ids,
                        enum_variants,
                        static_layout,
                        node_types,
                        pool,
                        synthesized_lambdas,
                    )?;
                    output.push(func);
                }
                ClassMember::Destructor(dtor) => {
                    let full_sym = format!("{cls_name}::~destructor");
                    let func = compile_destructor(
                        &full_sym,
                        ast.source_id,
                        dtor,
                        sources,
                        class_layouts,
                        function_ids,
                        enum_variants,
                        static_layout,
                        node_types,
                        pool,
                        synthesized_lambdas,
                    )?;
                    output.push(func);
                }
                ClassMember::Operator(op) => {
                    let op_str = op.operator_token.as_static_str();
                    let full_sym = format!("{cls_name}::operator_{op_str}");
                    let func = compile_operator(
                        &full_sym,
                        ast.source_id,
                        op,
                        sources,
                        class_layouts,
                        function_ids,
                        enum_variants,
                        static_layout,
                        node_types,
                        pool,
                        synthesized_lambdas,
                    )?;
                    output.push(func);
                }
                _ => {}
            }
        }

        let has_explicit_ctor = cls
            .members
            .iter()
            .any(|m| matches!(m, ClassMember::Constructor(_)));
        if !has_explicit_ctor {
            let full_sym = format!("{cls_name}::constructor");
            let func = compile_default_constructor(
                &full_sym,
                ast.source_id,
                cls,
                sources,
                class_layouts,
                function_ids,
                enum_variants,
                static_layout,
                node_types,
                pool,
                synthesized_lambdas,
            )?;
            output.push(func);
        }
    }

    if !ast.root_statements.is_empty() {
        let entry_sym = format!("<entry_{}>", ast.source_id);
        let mut func = CompiledFunction::new(entry_sym, ast.source_id, 0, 0);
        let mut allocator = RegisterAllocator::new(0);
        allocator.enter_scope();

        {
            let mut emitter = Emitter::new(&mut func, &mut allocator, pool)
                .with_sources(sources)
                .with_class_layouts(class_layouts)
                .with_function_ids(function_ids)
                .with_enum_variants(enum_variants)
                .with_static_layout(static_layout)
                .with_node_types(node_types)
                .with_synthesized_functions(synthesized_lambdas);

            let mut loop_stack = Vec::<LoopContext>::new();
            for stmt in &ast.root_statements {
                emitter.emit_statement(stmt, &mut loop_stack)?;
            }

            let has_return = emitter
                .function
                .instructions
                .last()
                .and_then(|inst| inst.opcode(0).ok())
                .is_some_and(|op| matches!(op, OpCode::Return | OpCode::ReturnValue));

            if !has_return {
                emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
            }
        }

        func.num_registers = allocator.peak_registers();
        output.push(func);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_single_method(
    symbol: &str,
    source_id: u16,
    m: &MethodDeclaration,
    is_instance: bool,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
) -> Result<CompiledFunction, EmissionError> {
    let param_offset = if is_instance { 1 } else { 0 };
    let param_count = (param_offset + m.parameters.len()) as u16;
    let mut func = CompiledFunction::new(symbol.to_string(), source_id, param_count, 0);
    let mut allocator = RegisterAllocator::new(param_count);

    if is_instance {
        allocator.alloc_param("this")?;
    }

    for param in &m.parameters {
        let name = get_span_text(param.name_span, &sources[param.name_span.id as usize]);
        allocator.alloc_param(name)?;
    }

    allocator.enter_scope();

    if let Some(body) = &m.body {
        let cls_name = symbol.split("::").next().unwrap_or("");
        let mut emitter = Emitter::new(&mut func, &mut allocator, pool)
            .with_sources(sources)
            .with_class_layouts(class_layouts)
            .with_function_ids(function_ids)
            .with_enum_variants(enum_variants)
            .with_static_layout(static_layout)
            .with_node_types(node_types)
            .with_synthesized_functions(synthesized_lambdas);

        if !cls_name.is_empty() {
            emitter = emitter.with_enclosing_class(cls_name);
        }

        let mut loop_stack = Vec::<LoopContext>::new();
        emitter.emit_block_statement(body, &mut loop_stack)?;

        let has_return = emitter
            .function
            .instructions
            .last()
            .and_then(|inst| inst.opcode(0).ok())
            .is_some_and(|op| matches!(op, OpCode::Return | OpCode::ReturnValue));

        if !has_return {
            emitter.emit_return_arc_cleanup(None);
            emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
        }
    } else if m.modifiers.contains(&Modifier::Native) {
        let clean_symbol = symbol.split('(').next().unwrap_or(symbol);
        let sym_idx = pool.add_string(clean_symbol)?;
        let dst = allocator.alloc_temp()?;
        func.emit(
            Instruction::format_call(OpCode::CallSymbol, dst, sym_idx, 0, param_count as u8),
            None,
        );
        let is_void = matches!(
            m.return_type,
            TypeAnnotation::Primitive {
                kind: TokenKind::PrimitiveVoid,
                ..
            }
        );
        if is_void {
            func.emit(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
        } else {
            func.emit(
                Instruction::format_a(OpCode::ReturnValue, 0, 0, dst, 0),
                None,
            );
        }
    } else {
        func.emit(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
    }

    func.num_registers = allocator.peak_registers();
    Ok(func)
}

#[allow(clippy::too_many_arguments)]
fn compile_constructor(
    symbol: &str,
    source_id: u16,
    ctor: &ConstructorDeclaration,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
) -> Result<CompiledFunction, EmissionError> {
    let param_count = (1 + ctor.parameters.len()) as u16; // this is r0
    let mut func = CompiledFunction::new(symbol.to_string(), source_id, param_count, 0);
    let mut allocator = RegisterAllocator::new(param_count);

    allocator.alloc_param("this")?;
    for param in &ctor.parameters {
        let name = get_span_text(param.name_span, &sources[param.name_span.id as usize]);
        allocator.alloc_param(name)?;
    }

    allocator.enter_scope();

    {
        let cls_name = symbol.split("::").next().unwrap_or("");
        let mut emitter = Emitter::new(&mut func, &mut allocator, pool)
            .with_sources(sources)
            .with_class_layouts(class_layouts)
            .with_function_ids(function_ids)
            .with_enum_variants(enum_variants)
            .with_static_layout(static_layout)
            .with_node_types(node_types)
            .with_synthesized_functions(synthesized_lambdas)
            .with_enclosing_class(cls_name);

        let mut loop_stack = Vec::<LoopContext>::new();

        // Superclass constructor chaining: super(...)
        if let Some(super_args) = &ctor.super_call_args {
            let cls_name = symbol.split("::").next().unwrap_or("");
            let simple_cls = cls_name.rsplit(['.', ':']).next().unwrap_or(cls_name);
            if let Some(layout) = class_layouts
                .get(cls_name)
                .or_else(|| class_layouts.get(simple_cls))
                && let Some(base_vtable_id) = layout.base_vtable_id
                && let Some(base_cls) = class_layouts
                    .values()
                    .find(|l| l.vtable_id == base_vtable_id)
            {
                let base_name = &base_cls.class_name;
                let simple_base = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
                let super_sym = format!("{base_name}::constructor");
                let super_sym_simple = format!("{simple_base}::constructor");
                if let Some(&super_func_id) = function_ids
                    .get(&super_sym)
                    .or_else(|| function_ids.get(&super_sym_simple))
                {
                    let window_size = 1 + super_args.len();
                    let arg_base = emitter.allocator.alloc_window(window_size)?;
                    emitter.emit_instruction(
                        Instruction::format_a(OpCode::Mov, 0, arg_base, 0, 0),
                        Some(ctor.span),
                    );
                    for (i, arg) in super_args.iter().enumerate() {
                        emitter.emit_expression(arg, Some(arg_base + 1 + i as u16))?;
                    }
                    let dummy = emitter.allocator.alloc_temp()?;
                    emitter.emit_instruction(
                        Instruction::format_call(
                            OpCode::CallDirect,
                            dummy,
                            super_func_id as u16,
                            arg_base,
                            window_size as u8,
                        ),
                        Some(ctor.span),
                    );
                    emitter.allocator.free_temp(dummy);
                    emitter.allocator.free_window(arg_base, window_size);
                }
            }
        }

        emitter.emit_block_statement(&ctor.body, &mut loop_stack)?;

        let has_return = emitter
            .function
            .instructions
            .last()
            .and_then(|inst| inst.opcode(0).ok())
            .is_some_and(|op| matches!(op, OpCode::Return | OpCode::ReturnValue));

        if !has_return {
            emitter.emit_return_arc_cleanup(None);
            emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
        }
    }

    func.num_registers = allocator.peak_registers();
    Ok(func)
}

#[allow(clippy::too_many_arguments)]
fn compile_default_constructor(
    symbol: &str,
    source_id: u16,
    cls: &ClassDeclaration,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
) -> Result<CompiledFunction, EmissionError> {
    let mut func = CompiledFunction::new(symbol.to_string(), source_id, 1, 0);
    let mut allocator = RegisterAllocator::new(1);
    allocator.alloc_param("this")?;
    allocator.enter_scope();

    {
        let cls_name = symbol.split("::").next().unwrap_or("");
        let mut emitter = Emitter::new(&mut func, &mut allocator, pool)
            .with_sources(sources)
            .with_class_layouts(class_layouts)
            .with_function_ids(function_ids)
            .with_enum_variants(enum_variants)
            .with_static_layout(static_layout)
            .with_node_types(node_types)
            .with_synthesized_functions(synthesized_lambdas)
            .with_enclosing_class(cls_name);

        let simple_cls = cls_name.rsplit(['.', ':']).next().unwrap_or(cls_name);
        if let Some(layout) = class_layouts
            .get(cls_name)
            .or_else(|| class_layouts.get(simple_cls))
            && let Some(base_vtable_id) = layout.base_vtable_id
            && let Some(base_cls) = class_layouts
                .values()
                .find(|l| l.vtable_id == base_vtable_id)
        {
            let base_name = &base_cls.class_name;
            let simple_base = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
            let super_sym = format!("{base_name}::constructor");
            let super_sym_simple = format!("{simple_base}::constructor");
            if let Some(&super_func_id) = function_ids
                .get(&super_sym)
                .or_else(|| function_ids.get(&super_sym_simple))
            {
                let arg_base = emitter.allocator.alloc_window(1)?;
                emitter.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base, 0, 0),
                    Some(cls.span),
                );
                let dummy = emitter.allocator.alloc_temp()?;
                emitter.emit_instruction(
                    Instruction::format_call(
                        OpCode::CallDirect,
                        dummy,
                        super_func_id as u16,
                        arg_base,
                        1,
                    ),
                    Some(cls.span),
                );
                emitter.allocator.free_temp(dummy);
                emitter.allocator.free_window(arg_base, 1);
            }
        }

        // Initialize fields with default values/initializers
        for member in &cls.members {
            if let ClassMember::Field(f) = member
                && !f.modifiers.iter().any(|m| matches!(m, Modifier::Static))
                && let Some(init) = &f.initializer
            {
                let field_name = get_span_text(f.name_span, &sources[f.name_span.id as usize]);
                let offset = if let Some(layout) = class_layouts
                    .get(cls_name)
                    .or_else(|| class_layouts.get(simple_cls))
                {
                    layout.field_offsets.get(field_name).copied().unwrap_or(0) as u16
                } else {
                    0
                };
                let r_val = emitter.emit_expression(init, None)?;
                emitter.emit_instruction(
                    Instruction::format_a(OpCode::SetField, 0, r_val, 0, offset),
                    Some(f.span),
                );
                if emitter.query_type(init.span()).is_reference_type()
                    && !matches!(init, Expression::Identifier { .. })
                {
                    emitter.emit_instruction(
                        Instruction::format_a(OpCode::DecRef, 0, r_val, 0, 0),
                        Some(f.span),
                    );
                }
                emitter.allocator.free_temp(r_val);
            }
        }

        emitter.emit_return_arc_cleanup(None);
        emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
    }

    func.num_registers = allocator.peak_registers();
    Ok(func)
}

#[allow(clippy::too_many_arguments)]
fn compile_destructor(
    symbol: &str,
    source_id: u16,
    dtor: &DestructorDeclaration,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
) -> Result<CompiledFunction, EmissionError> {
    let param_count = 1; // this is r0
    let mut func = CompiledFunction::new(symbol.to_string(), source_id, param_count, 0);
    let mut allocator = RegisterAllocator::new(param_count);

    allocator.alloc_param("this")?;
    allocator.enter_scope();

    {
        let cls_name = symbol.split("::").next().unwrap_or("");
        let mut emitter = Emitter::new(&mut func, &mut allocator, pool)
            .with_sources(sources)
            .with_class_layouts(class_layouts)
            .with_function_ids(function_ids)
            .with_enum_variants(enum_variants)
            .with_static_layout(static_layout)
            .with_node_types(node_types)
            .with_synthesized_functions(synthesized_lambdas)
            .with_enclosing_class(cls_name);

        let mut loop_stack = Vec::<LoopContext>::new();
        emitter.emit_block_statement(&dtor.body, &mut loop_stack)?;

        let simple_cls = cls_name.rsplit(['.', ':']).next().unwrap_or(cls_name);
        if let Some(layout) = class_layouts
            .get(cls_name)
            .or_else(|| class_layouts.get(simple_cls))
            && let Some(base_vtable_id) = layout.base_vtable_id
            && let Some(base_cls) = class_layouts
                .values()
                .find(|l| l.vtable_id == base_vtable_id)
        {
            let base_name = &base_cls.class_name;
            let simple_base = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
            let super_dtor = format!("{base_name}::~destructor");
            let super_dtor_simple = format!("{simple_base}::~destructor");
            if let Some(&super_func_id) = function_ids
                .get(&super_dtor)
                .or_else(|| function_ids.get(&super_dtor_simple))
            {
                let arg_base = emitter.allocator.alloc_window(1)?;
                emitter.emit_instruction(
                    Instruction::format_a(OpCode::Mov, 0, arg_base, 0, 0),
                    Some(dtor.span),
                );
                let dummy = emitter.allocator.alloc_temp()?;
                emitter.emit_instruction(
                    Instruction::format_call(
                        OpCode::CallDirect,
                        dummy,
                        super_func_id as u16,
                        arg_base,
                        1,
                    ),
                    Some(dtor.span),
                );
                emitter.allocator.free_temp(dummy);
                emitter.allocator.free_window(arg_base, 1);
            }
        }

        let has_return = emitter
            .function
            .instructions
            .last()
            .and_then(|inst| inst.opcode(0).ok())
            .is_some_and(|op| matches!(op, OpCode::Return | OpCode::ReturnValue));

        if !has_return {
            emitter.emit_return_arc_cleanup(None);
            emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
        }
    }

    func.num_registers = allocator.peak_registers();
    Ok(func)
}

#[allow(clippy::too_many_arguments)]
fn compile_operator(
    symbol: &str,
    source_id: u16,
    op: &OperatorDeclaration,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
) -> Result<CompiledFunction, EmissionError> {
    let param_count = (1 + op.parameters.len()) as u16;
    let mut func = CompiledFunction::new(symbol.to_string(), source_id, param_count, 0);
    let mut allocator = RegisterAllocator::new(param_count);

    allocator.alloc_param("this")?;

    for param in &op.parameters {
        let name = get_span_text(param.name_span, &sources[param.name_span.id as usize]);
        allocator.alloc_param(name)?;
    }

    allocator.enter_scope();

    {
        let cls_name = symbol.split("::").next().unwrap_or("");
        let mut emitter = Emitter::new(&mut func, &mut allocator, pool)
            .with_sources(sources)
            .with_class_layouts(class_layouts)
            .with_function_ids(function_ids)
            .with_enum_variants(enum_variants)
            .with_static_layout(static_layout)
            .with_node_types(node_types)
            .with_synthesized_functions(synthesized_lambdas)
            .with_enclosing_class(cls_name);

        let mut loop_stack = Vec::<LoopContext>::new();
        emitter.emit_block_statement(&op.body, &mut loop_stack)?;

        let has_return = emitter
            .function
            .instructions
            .last()
            .and_then(|inst| inst.opcode(0).ok())
            .is_some_and(|op| matches!(op, OpCode::Return | OpCode::ReturnValue));

        if !has_return {
            emitter.emit_return_arc_cleanup(None);
            emitter.emit_instruction(Instruction::format_a(OpCode::Return, 0, 0, 0, 0), None);
        }
    }

    func.num_registers = allocator.peak_registers();
    Ok(func)
}

fn get_span_text(span: solix_common::span::Span, code: &str) -> &str {
    let start = span.start as usize;
    let end = span.end as usize;
    if start <= end && end <= code.len() {
        &code[start..end]
    } else {
        ""
    }
}

fn index_monomorphized_class_symbols(
    mono_cls: &ClassDeclaration,
    symbol_table: &SymbolTable,
    sources: &[String],
    function_ids: &mut HashMap<String, u32>,
    next_func_id: &mut u32,
) {
    let cls_id = mono_cls.resolved_symbol_id.map(SymbolId);
    let mangled_name = cls_id
        .and_then(|id| symbol_table.get_symbol(id))
        .map(|s| s.fully_qualified_name.clone())
        .unwrap_or_else(|| {
            if (mono_cls.name_span.id as usize) < sources.len() {
                get_span_text(mono_cls.name_span, &sources[mono_cls.name_span.id as usize])
                    .to_string()
            } else {
                "GenericClass".to_string()
            }
        });
    let simple_name = mangled_name
        .rsplit(['.', ':'])
        .next()
        .unwrap_or(&mangled_name)
        .to_string();

    let source_id = mono_cls.name_span.id;
    let code = sources
        .get(source_id as usize)
        .map(|s| s.as_str())
        .unwrap_or("");

    for member in &mono_cls.members {
        match member {
            ClassMember::Method(m) => {
                let method_name = get_span_text(m.name_span, code);
                let param_sig = m
                    .parameters
                    .iter()
                    .map(|p| get_span_text(p.param_type.span(), code))
                    .collect::<Vec<_>>()
                    .join(",");
                let full_sym_mangled = format!("{mangled_name}::{method_name}({param_sig})");
                let simple_sym_mangled = format!("{simple_name}::{method_name}({param_sig})");
                let full_sym = format!("{mangled_name}::{method_name}");
                let simple_sym = format!("{simple_name}::{method_name}");
                let func_id = *next_func_id;
                *next_func_id += 1;
                function_ids.insert(full_sym_mangled, func_id);
                function_ids.insert(simple_sym_mangled, func_id);
                function_ids.entry(full_sym).or_insert(func_id);
                function_ids.entry(simple_sym).or_insert(func_id);
                if let Some((generic_base, generic_args)) = simple_name.split_once('<') {
                    let generic_suffix = format!("<{generic_args}");
                    let generic_sym = format!("{generic_base}::{method_name}{generic_suffix}");
                    let generic_arity = format!(
                        "{generic_base}::{method_name}{generic_suffix}/{}",
                        m.parameters.len()
                    );
                    function_ids.insert(generic_sym, func_id);
                    function_ids.insert(generic_arity, func_id);
                }
            }
            ClassMember::Constructor(ctor) => {
                let arity = ctor.parameters.len();
                let param_sig = ctor
                    .parameters
                    .iter()
                    .map(|parameter| get_span_text(parameter.param_type.span(), code))
                    .collect::<Vec<_>>()
                    .join(",");
                let full_sym_mangled = if param_sig.is_empty() {
                    format!("{mangled_name}::constructor")
                } else {
                    format!("{mangled_name}::constructor({param_sig})")
                };
                let simple_sym_mangled = if param_sig.is_empty() {
                    format!("{simple_name}::constructor")
                } else {
                    format!("{simple_name}::constructor({param_sig})")
                };
                let full_sym_arity = format!("{mangled_name}::constructor/{arity}");
                let simple_sym_arity = format!("{simple_name}::constructor/{arity}");
                let full_sym = format!("{mangled_name}::constructor");
                let simple_sym = format!("{simple_name}::constructor");
                let func_id = *next_func_id;
                *next_func_id += 1;
                function_ids.insert(full_sym_mangled, func_id);
                function_ids.insert(simple_sym_mangled, func_id);
                function_ids.entry(full_sym_arity).or_insert(func_id);
                function_ids.entry(simple_sym_arity).or_insert(func_id);
                function_ids.entry(full_sym).or_insert(func_id);
                function_ids.entry(simple_sym).or_insert(func_id);
            }
            ClassMember::Destructor(_) => {
                let full_sym = format!("{mangled_name}::~destructor");
                let simple_sym = format!("{simple_name}::~destructor");
                let func_id = *next_func_id;
                *next_func_id += 1;
                function_ids.insert(full_sym, func_id);
                function_ids.insert(simple_sym, func_id);
            }
            ClassMember::Operator(op) => {
                let op_str = op.operator_token.as_static_str();
                let param_sig = op
                    .parameters
                    .iter()
                    .map(|parameter| get_span_text(parameter.param_type.span(), code))
                    .collect::<Vec<_>>()
                    .join(",");
                let full_sym = if param_sig.is_empty() {
                    format!("{mangled_name}::operator_{op_str}")
                } else {
                    format!("{mangled_name}::operator_{op_str}({param_sig})")
                };
                let simple_sym = if param_sig.is_empty() {
                    format!("{simple_name}::operator_{op_str}")
                } else {
                    format!("{simple_name}::operator_{op_str}({param_sig})")
                };
                let func_id = *next_func_id;
                *next_func_id += 1;
                function_ids.insert(full_sym, func_id);
                function_ids.insert(simple_sym, func_id);
                function_ids
                    .entry(format!("{mangled_name}::operator_{op_str}"))
                    .or_insert(func_id);
                function_ids
                    .entry(format!("{simple_name}::operator_{op_str}"))
                    .or_insert(func_id);
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_monomorphized_class_functions(
    mono_cls: &ClassDeclaration,
    symbol_table: &SymbolTable,
    sources: &[String],
    class_layouts: &HashMap<String, ClassLayout>,
    function_ids: &HashMap<String, u32>,
    enum_variants: &HashMap<String, u32>,
    static_layout: &StaticLayout,
    node_types: &HashMap<solix_common::span::Span, solix_common::types::Type>,
    pool: &mut ConstantPoolBuilder,
    output: &mut Vec<CompiledFunction>,
    synthesized_lambdas: &mut Vec<CompiledFunction>,
) -> Result<(), EmissionError> {
    let cls_id = mono_cls.resolved_symbol_id.map(SymbolId);
    let mangled_name = cls_id
        .and_then(|id| symbol_table.get_symbol(id))
        .map(|s| s.fully_qualified_name.clone())
        .unwrap_or_else(|| {
            if (mono_cls.name_span.id as usize) < sources.len() {
                get_span_text(mono_cls.name_span, &sources[mono_cls.name_span.id as usize])
                    .to_string()
            } else {
                "GenericClass".to_string()
            }
        });

    let source_id = mono_cls.name_span.id;
    let code = sources
        .get(source_id as usize)
        .map(|s| s.as_str())
        .unwrap_or("");

    for member in &mono_cls.members {
        match member {
            ClassMember::Method(m) => {
                let method_name = get_span_text(m.name_span, code);
                let param_sig = m
                    .parameters
                    .iter()
                    .map(|p| get_span_text(p.param_type.span(), code))
                    .collect::<Vec<_>>()
                    .join(",");
                let full_sym = m
                    .resolved_symbol_id
                    .and_then(|id| symbol_table.get_symbol(SymbolId(id)))
                    .map(|symbol| symbol.fully_qualified_name.clone())
                    .unwrap_or_else(|| format!("{mangled_name}::{method_name}({param_sig})"));
                let is_instance = !m.modifiers.contains(&Modifier::Static);
                let func = compile_single_method(
                    &full_sym,
                    source_id,
                    m,
                    is_instance,
                    sources,
                    class_layouts,
                    function_ids,
                    enum_variants,
                    static_layout,
                    node_types,
                    pool,
                    synthesized_lambdas,
                )?;
                output.push(func);
            }
            ClassMember::Constructor(ctor) => {
                let full_sym = format!("{mangled_name}::constructor");
                let func = compile_constructor(
                    &full_sym,
                    source_id,
                    ctor,
                    sources,
                    class_layouts,
                    function_ids,
                    enum_variants,
                    static_layout,
                    node_types,
                    pool,
                    synthesized_lambdas,
                )?;
                output.push(func);
            }
            ClassMember::Destructor(dtor) => {
                let full_sym = format!("{mangled_name}::~destructor");
                let func = compile_destructor(
                    &full_sym,
                    source_id,
                    dtor,
                    sources,
                    class_layouts,
                    function_ids,
                    enum_variants,
                    static_layout,
                    node_types,
                    pool,
                    synthesized_lambdas,
                )?;
                output.push(func);
            }
            ClassMember::Operator(op) => {
                let op_str = op.operator_token.as_static_str();
                let full_sym = format!("{mangled_name}::operator_{op_str}");
                let func = compile_operator(
                    &full_sym,
                    source_id,
                    op,
                    sources,
                    class_layouts,
                    function_ids,
                    enum_variants,
                    static_layout,
                    node_types,
                    pool,
                    synthesized_lambdas,
                )?;
                output.push(func);
            }
            _ => {}
        }
    }
    Ok(())
}
