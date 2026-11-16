//! On-the-fly generic template monomorphization and AST synthesis engine.

#![forbid(unsafe_code)]

use crate::BindingError;
use crate::phases::binder::layout::ClassLayout;
use crate::phases::binder::symbols::{SymbolEntry, SymbolId, SymbolTable};
use crate::phases::binder::type_resolver::resolve_type_annotation;
use crate::phases::parser::statements::declarations::{ClassDeclaration, ClassMember, Modifier};
use crate::phases::parser::statements::types::TypeAnnotation;
use crate::phases::parser::statements::{Expression, LambdaBody, Statement};
use solix_common::{PrimitiveKind, Type};
use std::collections::HashMap;

/// Registered AST blueprint for a generic type or method.
#[derive(Debug, Clone)]
pub struct GenericBlueprint {
    /// Source file ID where the blueprint was declared.
    pub source_id: u16,
    /// Package prefix where the blueprint was declared.
    pub package_prefix: String,
    /// Unqualified blueprint name (e.g., `"MasterContainer"`).
    pub name: String,
    /// Fully qualified blueprint name (e.g., `"solix.tests.master.MasterContainer"`).
    pub fully_qualified_name: String,
    /// Declared generic parameter names (e.g., `["K", "V"]`).
    pub parameter_names: Vec<String>,
    /// The original AST class declaration.
    pub class_ast: ClassDeclaration,
}

/// Registry storing generic blueprints and tracking instantiated concrete classes.
#[derive(Debug, Clone, Default)]
pub struct GenericBlueprintRegistry {
    /// Blueprints indexed by fully qualified name (e.g., `"solix.tests.master.MasterContainer"`).
    pub blueprints: HashMap<String, GenericBlueprint>,
    /// Instantiated monomorphized class symbol IDs indexed by mangled name.
    pub instantiated_classes: HashMap<String, SymbolId>,
    /// Instantiated monomorphized class ASTs synthesized at compile time.
    pub monomorphized_asts: Vec<ClassDeclaration>,
}

impl GenericBlueprintRegistry {
    /// Registers a new generic class blueprint.
    pub fn register_blueprint(&mut self, blueprint: GenericBlueprint) {
        self.blueprints
            .insert(blueprint.name.clone(), blueprint.clone());
        self.blueprints
            .insert(blueprint.fully_qualified_name.clone(), blueprint);
    }

    /// Checks if a blueprint exists for the given name.
    #[must_use]
    pub fn has_blueprint(&self, name: &str) -> bool {
        self.blueprints.contains_key(name)
    }

    /// Mangles a generic template name with its concrete type arguments.
    ///
    /// Example: `("solix.collections.List", [int32])` -> `"solix.collections.List<int32>"`
    #[must_use]
    pub fn mangle_name(base_name: &str, type_arguments: &[Type]) -> String {
        let mut mangled = String::with_capacity(base_name.len() + 16);
        mangled.push_str(base_name);
        mangled.push('<');
        for (index, argument) in type_arguments.iter().enumerate() {
            if index > 0 {
                mangled.push_str(", ");
            }
            mangled.push_str(&argument.to_string());
        }
        mangled.push('>');
        mangled
    }

    /// Monomorphizes a generic class blueprint on the fly for the specified concrete type arguments.
    ///
    /// # Errors
    /// Returns [`BindingError`] if arity mismatches or blueprint is unknown.
    pub fn monomorphize_class(
        &mut self,
        base_name: &str,
        type_arguments: &[Type],
        symbol_table: &mut SymbolTable,
        layouts: &mut HashMap<SymbolId, ClassLayout>,
        source_texts: &[String],
        next_vtable_id: &mut u32,
    ) -> Result<SymbolId, Vec<BindingError>> {
        let mangled_name = Self::mangle_name(base_name, type_arguments);
        let simple_base = base_name.rsplit(['.', ':']).next().unwrap_or(base_name);
        let simple_mangled = Self::mangle_name(simple_base, type_arguments);

        // Check if already monomorphized
        if let Some(&existing_id) = self
            .instantiated_classes
            .get(&mangled_name)
            .or_else(|| self.instantiated_classes.get(&simple_mangled))
        {
            return Ok(existing_id);
        }

        let blueprint = self
            .blueprints
            .get(base_name)
            .or_else(|| self.blueprints.get(simple_base))
            .or_else(|| {
                let dot_name = base_name.replace("::", ".");
                self.blueprints.get(&dot_name)
            })
            .or_else(|| {
                let colon_name = base_name.replace('.', "::");
                self.blueprints.get(&colon_name)
            })
            .cloned()
            .ok_or_else(|| {
                vec![BindingError::UndefinedSymbol {
                    symbol_name: base_name.to_string(),
                    span: miette::SourceSpan::new(0.into(), 0),
                }]
            })?;

        if blueprint.parameter_names.len() != type_arguments.len() {
            return Err(vec![BindingError::GenericArityMismatch {
                template_name: base_name.to_string(),
                expected: blueprint.parameter_names.len(),
                found: type_arguments.len(),
                span: blueprint.class_ast.span.to_source_span(),
            }]);
        }

        let mut constraint_errors = Vec::new();
        for (generic_param, concrete_type) in blueprint
            .class_ast
            .generic_params
            .iter()
            .zip(type_arguments)
        {
            for bound_annot in &generic_param.bounds {
                let bound_type = crate::phases::binder::type_resolver::resolve_type_annotation(
                    blueprint.source_id,
                    &source_texts[blueprint.source_id as usize],
                    bound_annot,
                    symbol_table,
                );
                let satisfies = concrete_type == &bound_type
                    || concrete_type.is_assignable_to(&bound_type, &|src_name, tgt_name| {
                        symbol_table
                            .resolve_symbol(blueprint.source_id, src_name)
                            .and_then(|src_symbol_id| layouts.get(&src_symbol_id))
                            .is_some_and(|layout| layout.itable_methods.contains_key(tgt_name))
                    });
                if !satisfies && bound_type != Type::Error {
                    let param_name = if generic_param.name_span.end as usize
                        <= source_texts[blueprint.source_id as usize].len()
                    {
                        &source_texts[blueprint.source_id as usize][generic_param.name_span.start
                            as usize
                            ..generic_param.name_span.end as usize]
                    } else {
                        "T"
                    };
                    constraint_errors.push(BindingError::GenericTypeConstraintViolation {
                        type_param: param_name.to_string(),
                        expected: bound_type.to_string(),
                        actual: concrete_type.to_string(),
                        span: generic_param.span.to_source_span(),
                    });
                }
            }
        }
        if !constraint_errors.is_empty() {
            return Err(constraint_errors);
        }

        // Build substitution map: "T" -> Type
        let mut substitutions = HashMap::new();
        for (param_name, concrete_type) in
            blueprint.parameter_names.iter().zip(type_arguments.iter())
        {
            substitutions.insert(param_name.clone(), concrete_type.clone());
        }

        // Clone and substitute the class AST
        let mut monomorphized_ast = blueprint.class_ast.clone();
        monomorphized_ast.generic_params.clear();
        monomorphized_ast
            .modifiers
            .retain(|modifier| *modifier != Modifier::Extern);
        for member in &mut monomorphized_ast.members {
            match member {
                ClassMember::Method(method_decl) => {
                    method_decl
                        .modifiers
                        .retain(|modifier| *modifier != Modifier::Extern);
                }
                ClassMember::Constructor(constructor_decl) => {
                    constructor_decl
                        .modifiers
                        .retain(|modifier| *modifier != Modifier::Extern);
                }
                _ => {}
            }
        }

        // Register class symbol
        let class_symbol_entry = SymbolEntry {
            id: SymbolId(0),
            name: mangled_name.clone(),
            fully_qualified_name: mangled_name.clone(),
            source_id: blueprint.source_id,
            span: monomorphized_ast.span,
            modifiers: monomorphized_ast.modifiers.clone(),
            symbol_type: Type::Class {
                symbol_name: mangled_name.clone(),
                type_arguments: type_arguments.to_vec(),
            },
            parent_symbol: None,
            members: Vec::new(),
            is_static: false,
            is_const: false,
            slot_index: None,
            parameter_names: Vec::new(),
        };

        let class_symbol_id = symbol_table.insert_symbol(class_symbol_entry);
        self.instantiated_classes
            .insert(mangled_name.clone(), class_symbol_id);
        self.instantiated_classes
            .insert(simple_mangled, class_symbol_id);

        let current_vtable_id = *next_vtable_id;
        *next_vtable_id += 1;

        let mut class_layout =
            ClassLayout::new(class_symbol_id, mangled_name.clone(), current_vtable_id);
        let mut current_field_offset = 8usize; // After 8-byte vtable pointer / header
        let mut child_symbol_ids = Vec::new();

        let source_text = source_texts
            .get(blueprint.source_id as usize)
            .map(|text| text.as_str())
            .unwrap_or("");

        // Process members
        for member in &mut monomorphized_ast.members {
            match member {
                ClassMember::Field(field_decl) => {
                    let field_name = if field_decl.name_span.end as usize <= source_text.len() {
                        &source_text
                            [field_decl.name_span.start as usize..field_decl.name_span.end as usize]
                    } else {
                        "field"
                    };

                    let field_type = resolve_type_annotation_with_subs(
                        blueprint.source_id,
                        source_text,
                        &field_decl.field_type,
                        symbol_table,
                        &substitutions,
                    );

                    let unwrapped_field_type = field_type.unwrap_alias();
                    let is_ref = unwrapped_field_type.is_reference_type();
                    let field_size = match unwrapped_field_type {
                        Type::Primitive(prim) => prim.byte_size(),
                        Type::Enum { .. } => 4,
                        _ => 8,
                    };

                    class_layout
                        .field_offsets
                        .insert(field_name.to_string(), current_field_offset);
                    if field_decl.modifiers.contains(&Modifier::Weak) {
                        class_layout.weak_field_offsets.push(current_field_offset);
                    } else if is_ref {
                        class_layout
                            .reference_field_offsets
                            .push(current_field_offset);
                    }

                    let field_type_tag: u8 = match unwrapped_field_type {
                        Type::Primitive(prim) => match prim {
                            PrimitiveKind::Int8 => 1,
                            PrimitiveKind::UInt8 | PrimitiveKind::Char => 2,
                            PrimitiveKind::Int16 => 3,
                            PrimitiveKind::UInt16 => 4,
                            PrimitiveKind::Int32 => 5,
                            PrimitiveKind::UInt32 => 6,
                            PrimitiveKind::Int64 => 7,
                            PrimitiveKind::UInt64 => 8,
                            PrimitiveKind::Float32 => 9,
                            PrimitiveKind::Float64 => 10,
                            PrimitiveKind::Bool => 11,
                            PrimitiveKind::TuplePrimitive => 12,
                            PrimitiveKind::Void => 0,
                        },
                        Type::Enum { .. } => 5,
                        _ => 12,
                    };
                    class_layout
                        .field_types
                        .insert(current_field_offset, field_type_tag);

                    current_field_offset += field_size.max(4); // 4-byte or 8-byte alignment

                    let field_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: field_name.to_string(),
                        fully_qualified_name: format!("{mangled_name}::{field_name}"),
                        source_id: blueprint.source_id,
                        span: field_decl.span,
                        modifiers: field_decl.modifiers.clone(),
                        symbol_type: field_type,
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: field_decl.modifiers.contains(&Modifier::Static),
                        is_const: field_decl.modifiers.contains(&Modifier::Const),
                        slot_index: None,
                        parameter_names: Vec::new(),
                    };
                    let field_id = symbol_table.insert_symbol(field_symbol);
                    child_symbol_ids.push(field_id);
                    field_decl.resolved_symbol_id = Some(field_id.0);
                }
                ClassMember::Method(method_decl) => {
                    let method_name = if method_decl.name_span.end as usize <= source_text.len() {
                        &source_text[method_decl.name_span.start as usize
                            ..method_decl.name_span.end as usize]
                    } else {
                        "method"
                    };

                    let return_type = resolve_type_annotation_with_subs(
                        blueprint.source_id,
                        source_text,
                        &method_decl.return_type,
                        symbol_table,
                        &substitutions,
                    );

                    let parameter_types: Vec<Type> = method_decl
                        .parameters
                        .iter()
                        .map(|parameter| {
                            resolve_type_annotation_with_subs(
                                blueprint.source_id,
                                source_text,
                                &parameter.param_type,
                                symbol_table,
                                &substitutions,
                            )
                        })
                        .collect();

                    let param_sig = parameter_types
                        .iter()
                        .map(|param_type| param_type.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let parameter_names: Vec<String> = method_decl
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let span = parameter.name_span;
                            if (span.end as usize) <= source_text.len() {
                                source_text[span.start as usize..span.end as usize].to_string()
                            } else {
                                String::new()
                            }
                        })
                        .collect();
                    let method_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: method_name.to_string(),
                        fully_qualified_name: format!("{mangled_name}::{method_name}({param_sig})"),
                        source_id: blueprint.source_id,
                        span: method_decl.span,
                        modifiers: method_decl.modifiers.clone(),
                        symbol_type: Type::FunctionPointer {
                            return_type: Box::new(return_type),
                            parameter_types: parameter_types.clone(),
                        },
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: method_decl.modifiers.contains(&Modifier::Static),
                        is_const: false,
                        slot_index: None,
                        parameter_names,
                    };
                    let method_id = symbol_table.insert_symbol(method_symbol);
                    child_symbol_ids.push(method_id);
                    method_decl.resolved_symbol_id = Some(method_id.0);

                    if !method_decl.modifiers.contains(&Modifier::Static) {
                        let slot_idx = class_layout.vtable_methods.len();
                        class_layout.vtable_methods.push(method_id);
                        class_layout.method_slots.insert(
                            format!(
                                "{method_name}({})",
                                parameter_types
                                    .iter()
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()
                                    .join(",")
                            ),
                            slot_idx,
                        );
                        class_layout
                            .method_slots
                            .insert(format!("{method_name}/{}", parameter_types.len()), slot_idx);
                        class_layout
                            .method_slots
                            .insert(method_name.to_string(), slot_idx);
                    }
                }
                ClassMember::Constructor(constructor_decl) => {
                    let parameter_names: Vec<String> = constructor_decl
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let span = parameter.name_span;
                            if (span.end as usize) <= source_text.len() {
                                source_text[span.start as usize..span.end as usize].to_string()
                            } else {
                                String::new()
                            }
                        })
                        .collect();
                    let constructor_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: "constructor".to_string(),
                        fully_qualified_name: format!("{mangled_name}::constructor"),
                        source_id: blueprint.source_id,
                        span: constructor_decl.span,
                        modifiers: constructor_decl.modifiers.clone(),
                        symbol_type: Type::Primitive(PrimitiveKind::Void),
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                        parameter_names,
                    };
                    let constructor_id = symbol_table.insert_symbol(constructor_symbol);
                    child_symbol_ids.push(constructor_id);
                }
                ClassMember::Destructor(destructor_decl) => {
                    let destructor_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: "destructor".to_string(),
                        fully_qualified_name: format!("{mangled_name}::destructor"),
                        source_id: blueprint.source_id,
                        span: destructor_decl.span,
                        modifiers: Vec::new(),
                        symbol_type: Type::Primitive(PrimitiveKind::Void),
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                        parameter_names: Vec::new(),
                    };
                    let destructor_id = symbol_table.insert_symbol(destructor_symbol);
                    child_symbol_ids.push(destructor_id);
                }
                ClassMember::Operator(operator_decl) => {
                    let parameter_names: Vec<String> = operator_decl
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let span = parameter.name_span;
                            if (span.end as usize) <= source_text.len() {
                                source_text[span.start as usize..span.end as usize].to_string()
                            } else {
                                String::new()
                            }
                        })
                        .collect();
                    let op_str = operator_decl.operator_token.as_static_str();
                    let operator_symbol = SymbolEntry {
                        id: SymbolId(0),
                        name: format!("operator_{op_str}"),
                        fully_qualified_name: format!("{mangled_name}::operator_{op_str}"),
                        source_id: blueprint.source_id,
                        span: operator_decl.span,
                        modifiers: operator_decl.modifiers.clone(),
                        symbol_type: Type::Primitive(PrimitiveKind::Void),
                        parent_symbol: Some(class_symbol_id),
                        members: Vec::new(),
                        is_static: false,
                        is_const: false,
                        slot_index: None,
                        parameter_names,
                    };
                    let operator_id = symbol_table.insert_symbol(operator_symbol);
                    child_symbol_ids.push(operator_id);
                }
                ClassMember::Enum(_) | ClassMember::Class(_) | ClassMember::Interface(_) => {}
            }
        }

        // Register implemented interfaces
        for iface_annotation in &monomorphized_ast.interfaces {
            let resolved_iface = resolve_type_annotation_with_subs(
                blueprint.source_id,
                source_text,
                iface_annotation,
                symbol_table,
                &substitutions,
            );
            if let Type::Interface {
                ref symbol_name, ..
            } = resolved_iface
            {
                let simple = symbol_name
                    .rsplit(['.', ':'])
                    .next()
                    .unwrap_or(symbol_name)
                    .to_string();
                let iface_methods = class_layout.vtable_methods.clone();
                class_layout
                    .itable_methods
                    .insert(symbol_name.clone(), iface_methods.clone());
                class_layout.itable_methods.insert(simple, iface_methods);
            }
        }

        class_layout.instance_size_bytes = current_field_offset;
        layouts.insert(class_symbol_id, class_layout);

        if let Some(class_entry) = symbol_table.get_symbol_mut(class_symbol_id) {
            class_entry.members = child_symbol_ids;
        }

        monomorphized_ast.resolved_symbol_id = Some(class_symbol_id.0);
        self.monomorphized_asts.push(monomorphized_ast);

        let mut collected_annotations = Vec::new();
        collect_type_annotations_from_class(&blueprint.class_ast, &mut collected_annotations);

        for type_annotation in collected_annotations {
            let resolved_type = resolve_type_annotation_with_subs(
                blueprint.source_id,
                source_text,
                &type_annotation,
                symbol_table,
                &substitutions,
            );
            let mut dependent_instantiations = Vec::new();
            extract_concrete_generic_instantiations(&resolved_type, &mut dependent_instantiations);
            for (dependent_name, dependent_arguments) in dependent_instantiations {
                let simple_dep_name = dependent_name
                    .rsplit(['.', ':'])
                    .next()
                    .unwrap_or(&dependent_name);
                if self.blueprints.contains_key(&dependent_name)
                    || self.blueprints.contains_key(simple_dep_name)
                {
                    let _ = self.monomorphize_class(
                        &dependent_name,
                        &dependent_arguments,
                        symbol_table,
                        layouts,
                        source_texts,
                        next_vtable_id,
                    );
                }
            }
        }

        Ok(class_symbol_id)
    }
}

/// Helper that resolves a [`TypeAnnotation`] while substituting generic parameter names with concrete types.
#[must_use]
pub fn resolve_type_annotation_with_subs(
    source_id: u16,
    source_text: &str,
    annotation: &TypeAnnotation,
    symbol_table: &SymbolTable,
    substitutions: &HashMap<String, Type>,
) -> Type {
    fn substitute(ty: Type, substitutions: &HashMap<String, Type>) -> Type {
        match ty {
            Type::GenericParameter { name, .. } | Type::Unresolved(name) => substitutions
                .get(&name)
                .cloned()
                .unwrap_or(Type::Unresolved(name)),
            Type::Class {
                symbol_name,
                type_arguments,
            } => Type::Class {
                symbol_name,
                type_arguments: type_arguments
                    .into_iter()
                    .map(|arg| substitute(arg, substitutions))
                    .collect(),
            },
            Type::Interface {
                symbol_name,
                type_arguments,
            } => Type::Interface {
                symbol_name,
                type_arguments: type_arguments
                    .into_iter()
                    .map(|arg| substitute(arg, substitutions))
                    .collect(),
            },
            Type::Array {
                element_type,
                depth,
            } => Type::Array {
                element_type: Box::new(substitute(*element_type, substitutions)),
                depth,
            },
            Type::Tuple(elements) => Type::Tuple(
                elements
                    .into_iter()
                    .map(|element| substitute(element, substitutions))
                    .collect(),
            ),
            Type::FunctionPointer {
                return_type,
                parameter_types,
            } => Type::FunctionPointer {
                return_type: Box::new(substitute(*return_type, substitutions)),
                parameter_types: parameter_types
                    .into_iter()
                    .map(|parameter| substitute(parameter, substitutions))
                    .collect(),
            },
            Type::Alias {
                alias_name,
                target_type,
            } => Type::Alias {
                alias_name,
                target_type: Box::new(substitute(*target_type, substitutions)),
            },
            other => other,
        }
    }

    let resolved = resolve_type_annotation(source_id, source_text, annotation, symbol_table);
    substitute(resolved, substitutions)
}

fn collect_type_annotations_from_class(
    class_decl: &ClassDeclaration,
    collected_annotations: &mut Vec<TypeAnnotation>,
) {
    collected_annotations.extend(class_decl.interfaces.clone());
    for member in &class_decl.members {
        match member {
            ClassMember::Field(field_decl) => {
                collected_annotations.push(field_decl.field_type.clone());
                if let Some(initializer) = &field_decl.initializer {
                    collect_type_annotations_from_expression(initializer, collected_annotations);
                }
            }
            ClassMember::Method(method_decl) => {
                collected_annotations.push(method_decl.return_type.clone());
                for parameter in &method_decl.parameters {
                    collected_annotations.push(parameter.param_type.clone());
                }
                if let Some(body) = &method_decl.body {
                    for statement in &body.statements {
                        collect_type_annotations_from_statement(statement, collected_annotations);
                    }
                }
            }
            ClassMember::Constructor(constructor_decl) => {
                for parameter in &constructor_decl.parameters {
                    collected_annotations.push(parameter.param_type.clone());
                }
                for statement in &constructor_decl.body.statements {
                    collect_type_annotations_from_statement(statement, collected_annotations);
                }
            }
            ClassMember::Destructor(destructor_decl) => {
                for statement in &destructor_decl.body.statements {
                    collect_type_annotations_from_statement(statement, collected_annotations);
                }
            }
            ClassMember::Operator(operator_decl) => {
                collected_annotations.push(operator_decl.return_type.clone());
                for parameter in &operator_decl.parameters {
                    collected_annotations.push(parameter.param_type.clone());
                }
                for statement in &operator_decl.body.statements {
                    collect_type_annotations_from_statement(statement, collected_annotations);
                }
            }
            ClassMember::Enum(_) | ClassMember::Class(_) | ClassMember::Interface(_) => {}
        }
    }
}

fn collect_type_annotations_from_statement(
    statement: &Statement,
    collected_annotations: &mut Vec<TypeAnnotation>,
) {
    match statement {
        Statement::Block(block_statement) => {
            for inner_statement in &block_statement.statements {
                collect_type_annotations_from_statement(inner_statement, collected_annotations);
            }
        }
        Statement::If(if_statement) => {
            collect_type_annotations_from_expression(
                &if_statement.condition,
                collected_annotations,
            );
            collect_type_annotations_from_statement(
                &if_statement.then_branch,
                collected_annotations,
            );
            if let Some(else_branch) = &if_statement.else_branch {
                collect_type_annotations_from_statement(else_branch, collected_annotations);
            }
        }
        Statement::While(while_statement) => {
            collect_type_annotations_from_expression(
                &while_statement.condition,
                collected_annotations,
            );
            collect_type_annotations_from_statement(&while_statement.body, collected_annotations);
        }
        Statement::DoWhile(do_while_statement) => {
            collect_type_annotations_from_statement(
                &do_while_statement.body,
                collected_annotations,
            );
            collect_type_annotations_from_expression(
                &do_while_statement.condition,
                collected_annotations,
            );
        }
        Statement::For(for_statement) => {
            if let Some(initializer) = &for_statement.initializer {
                collect_type_annotations_from_statement(initializer, collected_annotations);
            }
            if let Some(condition) = &for_statement.condition {
                collect_type_annotations_from_expression(condition, collected_annotations);
            }
            if let Some(update) = &for_statement.update {
                collect_type_annotations_from_expression(update, collected_annotations);
            }
            collect_type_annotations_from_statement(&for_statement.body, collected_annotations);
        }
        Statement::Switch(switch_statement) => {
            collect_type_annotations_from_expression(
                &switch_statement.discriminant,
                collected_annotations,
            );
            for case_clause in &switch_statement.cases {
                collect_type_annotations_from_expression(
                    &case_clause.case_value,
                    collected_annotations,
                );
                for inner_statement in &case_clause.statements {
                    collect_type_annotations_from_statement(inner_statement, collected_annotations);
                }
            }
            if let Some(default_statements) = &switch_statement.default_case {
                for inner_statement in default_statements {
                    collect_type_annotations_from_statement(inner_statement, collected_annotations);
                }
            }
        }
        Statement::Try(try_statement) => {
            for inner_statement in &try_statement.try_block.statements {
                collect_type_annotations_from_statement(inner_statement, collected_annotations);
            }
            for catch_clause in &try_statement.catch_clauses {
                collected_annotations.push(catch_clause.exception_type.clone());
                for inner_statement in &catch_clause.block.statements {
                    collect_type_annotations_from_statement(inner_statement, collected_annotations);
                }
            }
            if let Some(finally_block) = &try_statement.finally_block {
                for inner_statement in &finally_block.statements {
                    collect_type_annotations_from_statement(inner_statement, collected_annotations);
                }
            }
        }
        Statement::Throw(throw_statement) => {
            collect_type_annotations_from_expression(
                &throw_statement.expression,
                collected_annotations,
            );
        }
        Statement::Return(return_statement) => {
            if let Some(value_expression) = &return_statement.value {
                collect_type_annotations_from_expression(value_expression, collected_annotations);
            }
        }
        Statement::VariableDeclaration(variable_declaration) => {
            collected_annotations.push(variable_declaration.var_type.clone());
            if let Some(initializer) = &variable_declaration.initializer {
                collect_type_annotations_from_expression(initializer, collected_annotations);
            }
        }
        Statement::Expression(expression_statement) => {
            collect_type_annotations_from_expression(
                &expression_statement.expression,
                collected_annotations,
            );
        }
        Statement::Assert(assert_statement) => {
            collect_type_annotations_from_expression(
                &assert_statement.condition,
                collected_annotations,
            );
            if let Some(message) = &assert_statement.message {
                collect_type_annotations_from_expression(message, collected_annotations);
            }
        }
        Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Exit(_)
        | Statement::Declaration(_) => {}
    }
}

fn collect_type_annotations_from_expression(
    expression: &Expression,
    collected_annotations: &mut Vec<TypeAnnotation>,
) {
    match expression {
        Expression::Literal { .. } | Expression::Identifier { .. } => {}
        Expression::Binary { left, right, .. } => {
            collect_type_annotations_from_expression(left, collected_annotations);
            collect_type_annotations_from_expression(right, collected_annotations);
        }
        Expression::Unary { operand, .. } => {
            collect_type_annotations_from_expression(operand, collected_annotations);
        }
        Expression::Assignment { target, value, .. } => {
            collect_type_annotations_from_expression(target, collected_annotations);
            collect_type_annotations_from_expression(value, collected_annotations);
        }
        Expression::MemberAccess { object, .. } => {
            collect_type_annotations_from_expression(object, collected_annotations);
        }
        Expression::MethodCall {
            callee,
            generic_arguments,
            arguments,
            ..
        } => {
            collect_type_annotations_from_expression(callee, collected_annotations);
            collected_annotations.extend(generic_arguments.clone());
            for argument in arguments {
                collect_type_annotations_from_expression(argument, collected_annotations);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            collect_type_annotations_from_expression(array, collected_annotations);
            collect_type_annotations_from_expression(index, collected_annotations);
        }
        Expression::ArrayCreation {
            element_type, size, ..
        } => {
            collected_annotations.push(element_type.clone());
            collect_type_annotations_from_expression(size, collected_annotations);
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                collect_type_annotations_from_expression(element, collected_annotations);
            }
        }
        Expression::NewInstance {
            instance_type,
            arguments,
            ..
        } => {
            collected_annotations.push(instance_type.clone());
            for argument in arguments {
                collect_type_annotations_from_expression(argument, collected_annotations);
            }
        }
        Expression::Cast {
            expression: inner_expression,
            target_type,
            ..
        } => {
            collected_annotations.push(target_type.clone());
            collect_type_annotations_from_expression(inner_expression, collected_annotations);
        }
        Expression::Instanceof {
            expression: inner_expression,
            target_type,
            ..
        } => {
            collected_annotations.push(target_type.clone());
            collect_type_annotations_from_expression(inner_expression, collected_annotations);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_type_annotations_from_expression(condition, collected_annotations);
            collect_type_annotations_from_expression(then_branch, collected_annotations);
            collect_type_annotations_from_expression(else_branch, collected_annotations);
        }
        Expression::Lambda { body, .. } => match body {
            LambdaBody::Expression(expression_body) => {
                collect_type_annotations_from_expression(expression_body, collected_annotations);
            }
            LambdaBody::Block(block_body) => {
                for inner_statement in &block_body.statements {
                    collect_type_annotations_from_statement(inner_statement, collected_annotations);
                }
            }
        },
        Expression::Sizeof { target_type, .. } | Expression::Default { target_type, .. } => {
            collected_annotations.push(target_type.clone());
        }
        Expression::TupleCreation { elements, .. } => {
            for element in elements {
                collect_type_annotations_from_expression(element, collected_annotations);
            }
        }
        Expression::Block {
            statements, result, ..
        } => {
            for inner_statement in statements {
                collect_type_annotations_from_statement(inner_statement, collected_annotations);
            }
            if let Some(result_expression) = result {
                collect_type_annotations_from_expression(result_expression, collected_annotations);
            }
        }
    }
}

fn extract_concrete_generic_instantiations(
    target_type: &Type,
    output: &mut Vec<(String, Vec<Type>)>,
) {
    match target_type {
        Type::Class {
            symbol_name,
            type_arguments,
        } => {
            if !type_arguments.is_empty() {
                let has_unresolved_parameters = type_arguments.iter().any(|argument| {
                    matches!(
                        argument,
                        Type::GenericParameter { .. } | Type::Unresolved(_)
                    )
                });
                if !has_unresolved_parameters {
                    output.push((symbol_name.clone(), type_arguments.clone()));
                }
            }
            for argument in type_arguments {
                extract_concrete_generic_instantiations(argument, output);
            }
        }
        Type::Array { element_type, .. } => {
            extract_concrete_generic_instantiations(element_type, output);
        }
        Type::Tuple(elements) => {
            for element in elements {
                extract_concrete_generic_instantiations(element, output);
            }
        }
        _ => {}
    }
}
