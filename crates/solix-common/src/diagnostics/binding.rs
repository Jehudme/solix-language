//! Semantic binding and type-checking diagnostics emitted during semantic analysis.

#![forbid(unsafe_code)]

use crate::span::Span;
use miette::{Diagnostic, SourceSpan};

/// Structured diagnostic errors emitted during semantic binding and type resolution.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Diagnostic)]
pub enum BindingError {
    #[error("Undefined symbol '{symbol_name}'")]
    #[diagnostic(
        code(solix::binder::undefined_symbol),
        help("Ensure the symbol is declared or imported into the current scope.")
    )]
    UndefinedSymbol {
        symbol_name: String,
        #[label("unresolved symbol")]
        span: SourceSpan,
    },

    #[error("Duplicate symbol declaration '{symbol_name}'")]
    #[diagnostic(
        code(solix::binder::duplicate_symbol),
        help("Choose a unique identifier or remove the redundant declaration.")
    )]
    DuplicateSymbol {
        symbol_name: String,
        #[label("redeclared symbol")]
        span: SourceSpan,
    },

    #[error("Type mismatch: expected '{expected}', found '{found}'")]
    #[diagnostic(
        code(solix::binder::type_mismatch),
        help("Provide an expression compatible with the expected type or use an explicit cast.")
    )]
    TypeMismatch {
        expected: String,
        found: String,
        #[label("expected '{expected}', found '{found}'")]
        span: SourceSpan,
    },

    #[error("Cyclic inheritance detected in class '{class_name}'")]
    #[diagnostic(
        code(solix::binder::cyclic_inheritance),
        help("Break the cyclic dependency in class inheritance.")
    )]
    CyclicInheritance {
        class_name: String,
        #[label("cyclic inheritance loop")]
        span: SourceSpan,
    },

    #[error("Invalid superclass '{base_name}': {reason}")]
    #[diagnostic(
        code(solix::binder::invalid_base_class),
        help(
            "Classes may only inherit from concrete or abstract classes, not interfaces or enums."
        )
    )]
    InvalidBaseClass {
        base_name: String,
        reason: String,
        #[label("invalid base class")]
        span: SourceSpan,
    },

    #[error(
        "Class '{class_name}' does not implement interface method '{method_signature}' from '{interface_name}'"
    )]
    #[diagnostic(
        code(solix::binder::unimplemented_interface_method),
        help("Implement all required interface methods in the concrete class.")
    )]
    UnimplementedInterfaceMethod {
        class_name: String,
        interface_name: String,
        method_signature: String,
        #[label("unimplemented interface method")]
        span: SourceSpan,
    },

    #[error("Method override mismatch for '{method_name}' in superclass '{super_class}': {reason}")]
    #[diagnostic(
        code(solix::binder::override_mismatch),
        help(
            "Ensure the overriding method's parameter and return types match the superclass definition."
        )
    )]
    MethodOverrideMismatch {
        method_name: String,
        super_class: String,
        reason: String,
        #[label("invalid method override")]
        span: SourceSpan,
    },

    #[error(
        "Private member '{member_name}' of class '{owner_class}' cannot be accessed from this scope"
    )]
    #[diagnostic(
        code(solix::binder::private_access_violation),
        help("Private members can only be accessed within their declaring class.")
    )]
    PrivateAccessViolation {
        member_name: String,
        owner_class: String,
        #[label("private member access denied")]
        span: SourceSpan,
    },

    #[error(
        "Protected member '{member_name}' of class '{owner_class}' cannot be accessed from this scope"
    )]
    #[diagnostic(
        code(solix::binder::protected_access_violation),
        help(
            "Protected members can only be accessed within the declaring class or its subclasses."
        )
    )]
    ProtectedAccessViolation {
        member_name: String,
        owner_class: String,
        #[label("protected member access denied")]
        span: SourceSpan,
    },

    #[error("Cannot assign to immutable or const variable '{variable_name}'")]
    #[diagnostic(
        code(solix::binder::assign_to_const),
        help("Remove the 'const' modifier if this variable must be mutable.")
    )]
    CannotAssignToConst {
        variable_name: String,
        #[label("cannot assign to const variable")]
        span: SourceSpan,
    },

    #[error("Return statement outside function or method")]
    #[diagnostic(
        code(solix::binder::return_outside_function),
        help("Move this return statement inside a function or method body.")
    )]
    ReturnOutsideFunction {
        #[label("return outside function")]
        span: SourceSpan,
    },

    #[error("Non-void method '{method_name}' must return a value of type '{expected_type}'")]
    #[diagnostic(
        code(solix::binder::missing_return),
        help("Ensure all code paths return an expression matching the method's return type.")
    )]
    MissingReturn {
        method_name: String,
        expected_type: String,
        #[label("missing return statement")]
        span: SourceSpan,
    },

    #[error("Return type mismatch: expected '{expected}', found '{found}'")]
    #[diagnostic(
        code(solix::binder::return_type_mismatch),
        help("Change the returned expression to match the declared return type.")
    )]
    ReturnTypeMismatch {
        expected: String,
        found: String,
        #[label("returned expression type mismatch")]
        span: SourceSpan,
    },

    #[error("Ambiguous method call '{method_name}' for arguments '({argument_types})'")]
    #[diagnostic(
        code(solix::binder::overload_ambiguity),
        help("Disambiguate the call with explicit type casts.")
    )]
    OverloadAmbiguity {
        method_name: String,
        argument_types: String,
        #[label("ambiguous method overload")]
        span: SourceSpan,
    },

    #[error("No matching overload for '{method_name}' with arguments '({argument_types})'")]
    #[diagnostic(
        code(solix::binder::no_matching_overload),
        help("Verify method argument types or provide a suitable overload.")
    )]
    NoMatchingOverload {
        method_name: String,
        argument_types: String,
        #[label("no matching method signature")]
        span: SourceSpan,
    },

    #[error(
        "Generic arity mismatch for '{template_name}': expected {expected} type arguments, got {found}"
    )]
    #[diagnostic(
        code(solix::binder::generic_arity_mismatch),
        help("Provide the exact number of type arguments expected by the generic definition.")
    )]
    GenericArityMismatch {
        template_name: String,
        expected: usize,
        found: usize,
        #[label("arity mismatch")]
        span: SourceSpan,
    },

    #[error("Cannot instantiate abstract class or interface '{type_name}'")]
    #[diagnostic(
        code(solix::binder::instantiate_abstract),
        help("Instantiate a concrete subclass that implements all required methods.")
    )]
    CannotInstantiateAbstractType {
        type_name: String,
        #[label("cannot instantiate abstract type")]
        span: SourceSpan,
    },

    #[error(
        "Super constructor call ': super(...)' must be the first statement in constructor body"
    )]
    #[diagnostic(
        code(solix::binder::super_must_be_first),
        help("Invoke superclass constructors before executing any subclass logic.")
    )]
    SuperConstructorMustBeFirst {
        #[label("super constructor must be first")]
        span: SourceSpan,
    },

    #[error("Break or continue statement outside of loop")]
    #[diagnostic(
        code(solix::binder::break_outside_loop),
        help("Break and continue statements can only appear within loop bodies.")
    )]
    BreakOrContinueOutsideLoop {
        #[label("break or continue outside loop")]
        span: SourceSpan,
    },

    #[error("Invalid operator overload '{operator_name}': {reason}")]
    #[diagnostic(
        code(solix::binder::invalid_operator_overload),
        help("Binary operators require 1 parameter; unary operators require 0 parameters.")
    )]
    InvalidOperatorOverload {
        operator_name: String,
        reason: String,
        #[label("invalid operator overload signature")]
        span: SourceSpan,
    },

    #[error("Type '{found}' cannot be indexed with array brackets")]
    #[diagnostic(
        code(solix::binder::non_indexable_type),
        help("Array subscript indexing is only valid on arrays and tuples.")
    )]
    NonIndexableType {
        found: String,
        #[label("non-indexable type")]
        span: SourceSpan,
    },

    #[error("Type '{found}' cannot be invoked as a function")]
    #[diagnostic(
        code(solix::binder::non_callable_type),
        help("Only function pointers, methods, and closures can be invoked.")
    )]
    NonCallableType {
        found: String,
        #[label("non-callable expression")]
        span: SourceSpan,
    },

    #[error("Cannot infer type for expression")]
    #[diagnostic(
        code(solix::binder::cannot_infer_type),
        help("Provide an explicit type annotation.")
    )]
    CannotInferType {
        #[label("type inference failure")]
        span: SourceSpan,
    },

    #[error("Ambiguous imported symbol '{symbol_name}'; matches candidates: {candidates:?}")]
    #[diagnostic(
        code(solix::binder::ambiguous_import),
        help(
            "Disambiguate by importing or referencing the symbol with its fully-qualified package prefix."
        )
    )]
    AmbiguousImport {
        symbol_name: String,
        candidates: Vec<String>,
        #[label("ambiguous symbol import")]
        span: SourceSpan,
    },

    #[error("Case switch label must be a compile-time constant expression")]
    #[diagnostic(
        code(solix::binder::non_constant_case_value),
        help("Use literal constants or enum variants as case values.")
    )]
    NonConstantCaseValue {
        #[label("non-constant switch case")]
        span: SourceSpan,
    },

    #[error(
        "Static member '{member_name}' of '{owner}' must be accessed using scope operator '::' instead of '.'"
    )]
    #[diagnostic(
        code(solix::binder::static_access_via_dot),
        help("Replace '.' with '::' to access static members or enum variants.")
    )]
    StaticAccessViaDot {
        member_name: String,
        owner: String,
        #[label("static member accessed via '.'")]
        span: SourceSpan,
    },

    #[error(
        "Instance member '{member_name}' of '{owner}' must be accessed using '.' instead of '::'"
    )]
    #[diagnostic(
        code(solix::binder::instance_access_via_scope_resolution),
        help("Replace '::' with '.' to access instance fields or methods.")
    )]
    InstanceAccessViaScopeResolution {
        member_name: String,
        owner: String,
        #[label("instance member accessed via '::'")]
        span: SourceSpan,
    },

    #[error("Keyword 'super' cannot be used in a class with no superclass")]
    #[diagnostic(
        code(solix::binder::invalid_super_call),
        help("Remove the ': super(...)' invocation or specify a base class with 'extends'.")
    )]
    InvalidSuperCall {
        #[label("no superclass to invoke")]
        span: SourceSpan,
    },

    #[error("Modifier 'weak' can only be applied to reference types, found '{found}'")]
    #[diagnostic(
        code(solix::binder::invalid_weak_modifier),
        help("Remove 'weak' or change the field to a class or interface reference type.")
    )]
    InvalidWeakModifier {
        found: String,
        #[label("weak on non-reference type")]
        span: SourceSpan,
    },

    #[error("Circular alias detected in '{alias_name}'")]
    #[diagnostic(
        code(solix::binder::circular_alias),
        help("Break the cyclic alias chain.")
    )]
    CircularAlias {
        alias_name: String,
        #[label("circular alias")]
        span: SourceSpan,
    },

    #[error("Cannot resolve alias target '{target_name}'")]
    #[diagnostic(
        code(solix::binder::unresolved_alias_target),
        help("Ensure the aliased type exists and is in scope.")
    )]
    UnresolvedAliasTarget {
        target_name: String,
        #[label("cannot resolve alias target")]
        span: SourceSpan,
    },

    #[error("Cannot resolve import '{import_path}': package or symbol not found")]
    #[diagnostic(
        code(solix::binder::cannot_resolve_import),
        help("Ensure the imported package and symbol exist.")
    )]
    CannotResolveImport {
        import_path: String,
        #[label("cannot resolve import")]
        span: SourceSpan,
    },

    // ------------------------------------------------------------------------
    // Granular Binder Errors (E0233 - E0288)
    // ------------------------------------------------------------------------
    #[error("Duplicate local variable declaration '{name}'")]
    #[diagnostic(
        code(solix::binder::duplicate_local_variable),
        help(
            "Rename the variable to avoid shadowing or duplicate declarations in the same scope."
        )
    )]
    DuplicateLocalVariable {
        name: String,
        #[label("duplicate local variable")]
        span: SourceSpan,
    },

    #[error("Duplicate parameter name '{name}' in parameter list")]
    #[diagnostic(
        code(solix::binder::duplicate_parameter_name),
        help("Parameter names within a declaration must be unique.")
    )]
    DuplicateParameterName {
        name: String,
        #[label("duplicate parameter name")]
        span: SourceSpan,
    },

    #[error("Cannot inherit from class '{class_name}'")]
    #[diagnostic(
        code(solix::binder::cannot_inherit_from_final_class),
        help("Final classes cannot be extended.")
    )]
    CannotInheritFromFinalClass {
        class_name: String,
        #[label("cannot inherit from final class")]
        span: SourceSpan,
    },

    #[error("Type '{found}' cannot be used as a superclass; expected a class")]
    #[diagnostic(
        code(solix::binder::superclass_must_be_class),
        help("Only classes can be extended via 'extends'.")
    )]
    SuperclassMustBeClass {
        found: String,
        #[label("expected a class")]
        span: SourceSpan,
    },

    #[error("Type '{found}' cannot be used in 'implements' clause; expected an interface")]
    #[diagnostic(
        code(solix::binder::implemented_type_must_be_interface),
        help("Only interfaces can be implemented via 'implements'.")
    )]
    ImplementedTypeMustBeInterface {
        found: String,
        #[label("expected an interface")]
        span: SourceSpan,
    },

    #[error("Function or method expects {expected} argument(s), but {found} were provided")]
    #[diagnostic(
        code(solix::binder::argument_count_mismatch),
        help("Provide the exact number of required arguments.")
    )]
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        #[label("argument count mismatch")]
        span: SourceSpan,
    },

    #[error("Argument {parameter_index} type mismatch: expected '{expected}', found '{found}'")]
    #[diagnostic(
        code(solix::binder::argument_type_mismatch),
        help("Pass an argument matching the expected parameter type or use an explicit cast.")
    )]
    ArgumentTypeMismatch {
        parameter_index: usize,
        expected: String,
        found: String,
        #[label("argument type mismatch")]
        span: SourceSpan,
    },

    #[error(
        "Field '{field_name}' initializer type mismatch: expected '{expected}', found '{found}'"
    )]
    #[diagnostic(
        code(solix::binder::field_initializer_type_mismatch),
        help("Provide an initializer expression matching the declared field type.")
    )]
    FieldInitializerTypeMismatch {
        field_name: String,
        expected: String,
        found: String,
        #[label("field initializer type mismatch")]
        span: SourceSpan,
    },

    #[error("Cannot access member '{member_name}' on non-composite type '{found}'")]
    #[diagnostic(
        code(solix::binder::member_access_on_non_composite_type),
        help("Member access '.' is only valid on classes, interfaces, or structs.")
    )]
    MemberAccessOnNonCompositeType {
        found: String,
        member_name: String,
        #[label("member access on non-composite type")]
        span: SourceSpan,
    },

    #[error("Super constructor expects {expected} argument(s), but {found} were provided")]
    #[diagnostic(
        code(solix::binder::super_constructor_argument_mismatch),
        help("Provide arguments matching the base class constructor definition.")
    )]
    SuperConstructorArgumentMismatch {
        expected: usize,
        found: usize,
        #[label("super constructor argument count mismatch")]
        span: SourceSpan,
    },

    #[error("Invalid 'main' entry point signature: {reason}")]
    #[diagnostic(
        code(solix::binder::main_method_signature_invalid),
        help("The 'main' function must have signature 'int32 main()' or 'void main()'.")
    )]
    MainMethodSignatureInvalid {
        reason: String,
        #[label("invalid main signature")]
        span: SourceSpan,
    },

    #[error("Multiple 'main' entry point functions discovered ({count} found across package)")]
    #[diagnostic(
        code(solix::binder::multiple_main_methods_found),
        help(
            "Ensure exactly one 'main' function is defined across all compiled files in the package."
        )
    )]
    MultipleMainMethodsFound {
        count: usize,
        #[label("conflicting main entry point")]
        span: SourceSpan,
    },

    #[error("Duplicate variant '{variant_name}' in enum '{enum_name}'")]
    #[diagnostic(
        code(solix::binder::duplicate_enum_variant),
        help("Each variant identifier in an enum must be unique.")
    )]
    DuplicateEnumVariant {
        variant_name: String,
        enum_name: String,
        #[label("duplicate enum variant")]
        span: SourceSpan,
    },

    #[error("Explicit value for enum variant '{variant_name}' must be an integer, found '{found}'")]
    #[diagnostic(
        code(solix::binder::enum_explicit_value_not_integer),
        help("Enum variant values must evaluate to integer constants.")
    )]
    EnumExplicitValueNotInteger {
        variant_name: String,
        found: String,
        #[label("non-integer enum value")]
        span: SourceSpan,
    },

    #[error("Unary operator '{operator}' cannot be applied to operand of type '{operand_type}'")]
    #[diagnostic(
        code(solix::binder::unary_operator_type_mismatch),
        help("Apply the unary operator to a compatible operand type.")
    )]
    UnaryOperatorTypeMismatch {
        operator: String,
        operand_type: String,
        #[label("incompatible operand type")]
        span: SourceSpan,
    },

    #[error(
        "Binary operator '{operator}' cannot be applied to types '{left_type}' and '{right_type}'"
    )]
    #[diagnostic(
        code(solix::binder::binary_operator_type_mismatch),
        help("Apply the binary operator to compatible operand types.")
    )]
    BinaryOperatorTypeMismatch {
        operator: String,
        left_type: String,
        right_type: String,
        #[label("incompatible operand types")]
        span: SourceSpan,
    },

    #[error("Conditional expression must evaluate to 'bool', found '{found}'")]
    #[diagnostic(
        code(solix::binder::logical_condition_not_boolean),
        help("Ensure loop or if conditions produce boolean values.")
    )]
    LogicalConditionNotBoolean {
        found: String,
        #[label("expected 'bool'")]
        span: SourceSpan,
    },

    #[error("Array size expression must evaluate to an integer type, found '{found}'")]
    #[diagnostic(
        code(solix::binder::array_length_not_integer),
        help("Specify array sizes using integer expressions.")
    )]
    ArrayLengthNotInteger {
        found: String,
        #[label("expected integer array size")]
        span: SourceSpan,
    },

    #[error("Array index expression must evaluate to an integer type, found '{found}'")]
    #[diagnostic(
        code(solix::binder::index_expression_not_integer),
        help("Subscript index expressions must evaluate to integers.")
    )]
    IndexExpressionNotInteger {
        found: String,
        #[label("expected integer index")]
        span: SourceSpan,
    },

    #[error("Tuple index {index} out of range for tuple of size {size}")]
    #[diagnostic(
        code(solix::binder::tuple_index_out_of_range),
        help("Tuple indices are 0-based and must be less than the tuple element count.")
    )]
    TupleIndexOutOfRange {
        index: usize,
        size: usize,
        #[label("tuple index out of range")]
        span: SourceSpan,
    },

    #[error("Cannot assign to read-only target '{name}'")]
    #[diagnostic(
        code(solix::binder::cannot_assign_to_read_only),
        help("Target is read-only and cannot be modified.")
    )]
    CannotAssignToReadOnly {
        name: String,
        #[label("cannot assign to read-only")]
        span: SourceSpan,
    },

    #[error("Type 'void' is only valid as a return type and cannot be used here")]
    #[diagnostic(
        code(solix::binder::void_type_not_allowed_here),
        help("Variables, parameters, and fields cannot be of type 'void'.")
    )]
    VoidTypeNotAllowedHere {
        #[label("'void' not permitted here")]
        span: SourceSpan,
    },

    #[error("Unknown type '{type_name}' in declaration")]
    #[diagnostic(
        code(solix::binder::unknown_type_in_declaration),
        help("Ensure the declared type name exists and is imported.")
    )]
    UnknownTypeInDeclaration {
        type_name: String,
        #[label("unknown type")]
        span: SourceSpan,
    },

    #[error("Duplicate field '{field_name}' in class '{class_name}'")]
    #[diagnostic(
        code(solix::binder::duplicate_field_declaration),
        help("Field names within a class must be unique.")
    )]
    DuplicateFieldDeclaration {
        field_name: String,
        class_name: String,
        #[label("duplicate field")]
        span: SourceSpan,
    },

    #[error("Duplicate method '{method_signature}' in class '{class_name}'")]
    #[diagnostic(
        code(solix::binder::duplicate_method_declaration),
        help("Method signatures within a class must be unique.")
    )]
    DuplicateMethodDeclaration {
        method_signature: String,
        class_name: String,
        #[label("duplicate method signature")]
        span: SourceSpan,
    },

    #[error("Duplicate constructor signature '{constructor_signature}'")]
    #[diagnostic(
        code(solix::binder::duplicate_constructor_declaration),
        help("Constructor overloads must have distinct parameter lists.")
    )]
    DuplicateConstructorDeclaration {
        constructor_signature: String,
        #[label("duplicate constructor signature")]
        span: SourceSpan,
    },

    #[error("Constructor chaining cycle detected in constructor '{constructor_name}'")]
    #[diagnostic(
        code(solix::binder::constructor_chaining_cycle),
        help("Avoid recursive ': this(...)' constructor delegations.")
    )]
    ConstructorChainingCycle {
        constructor_name: String,
        #[label("constructor cycle detected")]
        span: SourceSpan,
    },

    #[error(
        "Keyword 'this' can only be used inside instance methods, constructors, or destructors"
    )]
    #[diagnostic(
        code(solix::binder::this_used_outside_instance_method),
        help("Remove 'this' from static or package-level contexts.")
    )]
    ThisUsedOutsideInstanceMethod {
        #[label("'this' outside instance method")]
        span: SourceSpan,
    },

    #[error("Keyword 'super' can only be used inside subclasses with a declared superclass")]
    #[diagnostic(
        code(solix::binder::super_used_outside_subclass),
        help("Ensure the enclosing class extends a base class before using 'super'.")
    )]
    SuperUsedOutsideSubclass {
        #[label("'super' outside subclass")]
        span: SourceSpan,
    },

    #[error(
        "Static method '{method_name}' of class '{class_name}' should be invoked via '::' rather than an instance"
    )]
    #[diagnostic(
        code(solix::binder::static_method_access_via_instance),
        help("Invoke static methods using the class name: 'ClassName::method()'.")
    )]
    StaticMethodAccessViaInstance {
        method_name: String,
        class_name: String,
        #[label("static method called on instance")]
        span: SourceSpan,
    },

    #[error(
        "Instance method '{method_name}' of class '{class_name}' cannot be invoked statically without an instance"
    )]
    #[diagnostic(
        code(solix::binder::instance_method_access_via_static),
        help("Instance methods require an object instance to be invoked.")
    )]
    InstanceMethodAccessViaStatic {
        method_name: String,
        class_name: String,
        #[label("instance method called statically")]
        span: SourceSpan,
    },

    #[error("Abstract method '{method_name}' declared in non-abstract class '{class_name}'")]
    #[diagnostic(
        code(solix::binder::abstract_method_in_non_abstract_class),
        help("Declare the enclosing class 'abstract' or provide a method body.")
    )]
    AbstractMethodInNonAbstractClass {
        method_name: String,
        class_name: String,
        #[label("abstract method in concrete class")]
        span: SourceSpan,
    },

    #[error(
        "Class '{class_name}' must either be declared abstract or implement abstract method '{missing_method}'"
    )]
    #[diagnostic(
        code(solix::binder::non_abstract_class_missing_abstract_impl),
        help("Implement all inherited abstract methods or mark the class 'abstract'.")
    )]
    NonAbstractClassMissingAbstractImpl {
        class_name: String,
        missing_method: String,
        #[label("missing abstract method implementation")]
        span: SourceSpan,
    },

    #[error("Fields in interfaces must be compile-time constants")]
    #[diagnostic(
        code(solix::binder::interface_field_not_constant),
        help("Declare interface fields 'const' with compile-time literal initializers.")
    )]
    InterfaceFieldNotConstant {
        field_name: String,
        interface_name: String,
        #[label("non-constant interface field")]
        span: SourceSpan,
    },

    #[error("Cannot override final method '{method_name}' from base class '{base_class}'")]
    #[diagnostic(
        code(solix::binder::final_method_overridden),
        help("Final methods cannot be overridden in derived classes.")
    )]
    FinalMethodOverridden {
        method_name: String,
        base_class: String,
        #[label("override of final method")]
        span: SourceSpan,
    },

    #[error(
        "Cannot reduce visibility of overridden method '{method_name}' from '{original_visibility}' to '{new_visibility}'"
    )]
    #[diagnostic(
        code(solix::binder::narrowing_visibility_in_override),
        help(
            "Overriding methods cannot declare more restrictive visibility than the parent method."
        )
    )]
    NarrowingVisibilityInOverride {
        method_name: String,
        original_visibility: String,
        new_visibility: String,
        #[label("narrowed override visibility")]
        span: SourceSpan,
    },

    #[error("Cannot cast expression of type '{source_type}' to '{target_type}'")]
    #[diagnostic(
        code(solix::binder::invalid_cast_type),
        help("Ensure types are related by inheritance or numeric conversion paths.")
    )]
    InvalidCastType {
        source_type: String,
        target_type: String,
        #[label("invalid type cast")]
        span: SourceSpan,
    },

    #[error(
        "Right-hand operand of 'instanceof' must be a class or interface type, found '{found}'"
    )]
    #[diagnostic(
        code(solix::binder::invalid_type_for_instance_of),
        help("Use 'instanceof' only with nominal class or interface types.")
    )]
    InvalidTypeForInstanceOf {
        found: String,
        #[label("invalid type for instanceof")]
        span: SourceSpan,
    },

    #[error("Unreachable statement detected after return or throw")]
    #[diagnostic(
        code(solix::binder::dead_code_after_return_or_throw),
        help("Remove unreachable statements after unconditional control flow divergence.")
    )]
    DeadCodeAfterReturnOrThrow {
        #[label("unreachable statement")]
        span: SourceSpan,
    },

    #[error("Local variable '{variable_name}' used before being initialized")]
    #[diagnostic(
        code(solix::binder::uninitialized_variable_used),
        help("Initialize local variables before reading their value.")
    )]
    UninitializedVariableUsed {
        variable_name: String,
        #[label("use of uninitialized variable")]
        span: SourceSpan,
    },

    #[error("Duplicate access modifier '{modifier}'")]
    #[diagnostic(
        code(solix::binder::duplicate_access_modifier),
        help("Remove duplicate access modifiers.")
    )]
    DuplicateAccessModifier {
        modifier: String,
        #[label("duplicate access modifier")]
        span: SourceSpan,
    },

    #[error("Cannot combine '{modifier1}' and '{modifier2}' modifiers")]
    #[diagnostic(
        code(solix::binder::conflicting_modifier_combination),
        help("Remove one of the conflicting modifiers.")
    )]
    ConflictingModifierCombination {
        modifier1: String,
        modifier2: String,
        #[label("conflicting modifiers")]
        span: SourceSpan,
    },

    #[error("Modifier '{modifier}' is not applicable to {declaration_kind} declarations")]
    #[diagnostic(
        code(solix::binder::modifier_not_applicable_to_declaration),
        help("Remove modifier invalid for this declaration kind.")
    )]
    ModifierNotApplicableToDeclaration {
        modifier: String,
        declaration_kind: String,
        #[label("inapplicable modifier")]
        span: SourceSpan,
    },

    #[error("Modifier 'virtual' can only be applied to methods inside a class")]
    #[diagnostic(
        code(solix::binder::virtual_on_non_method),
        help("Remove 'virtual'; only class methods can be virtual.")
    )]
    VirtualOnNonMethod {
        #[label("virtual on non-method")]
        span: SourceSpan,
    },

    #[error("Modifier 'override' can only be applied to methods inside a class")]
    #[diagnostic(
        code(solix::binder::override_on_non_method),
        help("Remove 'override'; only class methods can be overrides.")
    )]
    OverrideOnNonMethod {
        #[label("override on non-method")]
        span: SourceSpan,
    },

    #[error("Modifier 'native' can only be applied to functions or methods")]
    #[diagnostic(
        code(solix::binder::native_on_non_method_or_function),
        help("Remove 'native'; only callable declarations can be native.")
    )]
    NativeOnNonMethodOrFunction {
        #[label("native on non-callable")]
        span: SourceSpan,
    },

    #[error("Modifier 'inline' can only be applied to functions or methods")]
    #[diagnostic(
        code(solix::binder::inline_on_non_method_or_function),
        help("Remove 'inline'; only callable declarations can be inline.")
    )]
    InlineOnNonMethodOrFunction {
        #[label("inline on non-callable")]
        span: SourceSpan,
    },

    #[error("Modifier 'static' cannot be applied to constructors or destructors")]
    #[diagnostic(
        code(solix::binder::static_on_constructor_or_destructor),
        help("Constructors and destructors cannot be declared static.")
    )]
    StaticOnConstructorOrDestructor {
        is_destructor: bool,
        #[label("static on constructor/destructor")]
        span: SourceSpan,
    },

    #[error("Access modifiers cannot be applied to destructors")]
    #[diagnostic(
        code(solix::binder::visibility_on_destructor),
        help("Destructors have no visibility modifier; remove 'public', 'private', etc.")
    )]
    VisibilityOnDestructor {
        #[label("visibility on destructor")]
        span: SourceSpan,
    },

    #[error("Method '{method_name}' marked 'override' does not override any base class method")]
    #[diagnostic(
        code(solix::binder::override_without_base_method),
        help("Ensure the method signature matches a virtual method in a superclass.")
    )]
    OverrideWithoutBaseMethod {
        method_name: String,
        #[label("override without base method")]
        span: SourceSpan,
    },

    #[error(
        "Cannot override method '{method_name}' of class '{base_class}' because it is not declared 'virtual'"
    )]
    #[diagnostic(
        code(solix::binder::override_non_virtual_method),
        help("Declare the base class method 'virtual' or remove 'override'.")
    )]
    OverrideNonVirtualMethod {
        method_name: String,
        base_class: String,
        #[label("cannot override non-virtual method")]
        span: SourceSpan,
    },

    #[error("Destructors cannot be invoked directly; they are managed automatically")]
    #[diagnostic(
        code(solix::binder::destructor_invocation_direct),
        help("Remove explicit destructor call; ARC cleans up objects automatically.")
    )]
    DestructorInvocationDirect {
        #[label("direct destructor invocation")]
        span: SourceSpan,
    },

    #[error("Extern symbol '{symbol_name}' is missing an explicit type signature")]
    #[diagnostic(
        code(solix::binder::extern_symbol_missing_signature),
        help("Provide explicit types for extern functions and variables.")
    )]
    ExternSymbolMissingSignature {
        symbol_name: String,
        #[label("missing extern signature")]
        span: SourceSpan,
    },

    #[error(
        "Default value for parameter '{parameter_name}' type mismatch: expected '{expected}', found '{found}'"
    )]
    #[diagnostic(
        code(solix::binder::parameter_default_value_mismatch),
        help("Ensure default parameter expression matches declared parameter type.")
    )]
    ParameterDefaultValueMismatch {
        parameter_name: String,
        expected: String,
        found: String,
        #[label("parameter default value mismatch")]
        span: SourceSpan,
    },

    #[error("Expression cannot be evaluated in a constant context: {reason}")]
    #[diagnostic(
        code(solix::binder::invalid_expression_in_constant_context),
        help(
            "Only literal constants, basic arithmetic, and const references are allowed in constant expressions."
        )
    )]
    InvalidExpressionInConstantContext {
        reason: String,
        #[label("invalid constant expression")]
        span: SourceSpan,
    },

    #[error("Bitwise operator '{operator}' can only be applied to integer types, found '{found}'")]
    #[diagnostic(
        code(solix::binder::bitwise_operator_on_non_integer),
        help(
            "Bitwise operators ('&', '|', '^', '<<', '>>', '~') are only valid on integer types."
        )
    )]
    BitwiseOperatorOnNonInteger {
        operator: String,
        found: String,
        #[label("bitwise operator on non-integer")]
        span: SourceSpan,
    },
}

impl BindingError {
    /// Returns the standardized diagnostic error code string.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::UndefinedSymbol { .. } => "E0201",
            Self::DuplicateSymbol { .. } => "E0202",
            Self::TypeMismatch { .. } => "E0203",
            Self::CyclicInheritance { .. } => "E0204",
            Self::InvalidBaseClass { .. } => "E0205",
            Self::UnimplementedInterfaceMethod { .. } => "E0206",
            Self::MethodOverrideMismatch { .. } => "E0207",
            Self::PrivateAccessViolation { .. } => "E0208",
            Self::ProtectedAccessViolation { .. } => "E0209",
            Self::CannotAssignToConst { .. } => "E0210",
            Self::ReturnOutsideFunction { .. } => "E0211",
            Self::MissingReturn { .. } => "E0212",
            Self::ReturnTypeMismatch { .. } => "E0213",
            Self::OverloadAmbiguity { .. } => "E0214",
            Self::NoMatchingOverload { .. } => "E0215",
            Self::GenericArityMismatch { .. } => "E0216",
            Self::CannotInstantiateAbstractType { .. } => "E0217",
            Self::SuperConstructorMustBeFirst { .. } => "E0218",
            Self::BreakOrContinueOutsideLoop { .. } => "E0219",
            Self::InvalidOperatorOverload { .. } => "E0220",
            Self::NonIndexableType { .. } => "E0221",
            Self::NonCallableType { .. } => "E0222",
            Self::CannotInferType { .. } => "E0223",
            Self::AmbiguousImport { .. } => "E0224",
            Self::NonConstantCaseValue { .. } => "E0225",
            Self::StaticAccessViaDot { .. } => "E0226",
            Self::InstanceAccessViaScopeResolution { .. } => "E0227",
            Self::InvalidSuperCall { .. } => "E0228",
            Self::InvalidWeakModifier { .. } => "E0229",
            Self::CircularAlias { .. } => "E0230",
            Self::UnresolvedAliasTarget { .. } => "E0231",
            Self::CannotResolveImport { .. } => "E0232",

            Self::DuplicateLocalVariable { .. } => "E0233",
            Self::DuplicateParameterName { .. } => "E0234",
            Self::CannotInheritFromFinalClass { .. } => "E0235",
            Self::SuperclassMustBeClass { .. } => "E0236",
            Self::ImplementedTypeMustBeInterface { .. } => "E0237",
            Self::ArgumentCountMismatch { .. } => "E0238",
            Self::ArgumentTypeMismatch { .. } => "E0239",
            Self::FieldInitializerTypeMismatch { .. } => "E0240",
            Self::MemberAccessOnNonCompositeType { .. } => "E0241",
            Self::SuperConstructorArgumentMismatch { .. } => "E0242",
            Self::MainMethodSignatureInvalid { .. } => "E0243",
            Self::MultipleMainMethodsFound { .. } => "E0244",
            Self::DuplicateEnumVariant { .. } => "E0245",
            Self::EnumExplicitValueNotInteger { .. } => "E0246",
            Self::UnaryOperatorTypeMismatch { .. } => "E0247",
            Self::BinaryOperatorTypeMismatch { .. } => "E0248",
            Self::LogicalConditionNotBoolean { .. } => "E0249",
            Self::ArrayLengthNotInteger { .. } => "E0250",
            Self::IndexExpressionNotInteger { .. } => "E0251",
            Self::TupleIndexOutOfRange { .. } => "E0252",
            Self::CannotAssignToReadOnly { .. } => "E0253",
            Self::VoidTypeNotAllowedHere { .. } => "E0254",
            Self::UnknownTypeInDeclaration { .. } => "E0255",
            Self::DuplicateFieldDeclaration { .. } => "E0256",
            Self::DuplicateMethodDeclaration { .. } => "E0257",
            Self::DuplicateConstructorDeclaration { .. } => "E0258",
            Self::ConstructorChainingCycle { .. } => "E0259",
            Self::ThisUsedOutsideInstanceMethod { .. } => "E0260",
            Self::SuperUsedOutsideSubclass { .. } => "E0261",
            Self::StaticMethodAccessViaInstance { .. } => "E0262",
            Self::InstanceMethodAccessViaStatic { .. } => "E0263",
            Self::AbstractMethodInNonAbstractClass { .. } => "E0264",
            Self::NonAbstractClassMissingAbstractImpl { .. } => "E0265",
            Self::InterfaceFieldNotConstant { .. } => "E0266",
            Self::FinalMethodOverridden { .. } => "E0267",
            Self::NarrowingVisibilityInOverride { .. } => "E0268",
            Self::InvalidCastType { .. } => "E0269",
            Self::InvalidTypeForInstanceOf { .. } => "E0270",
            Self::DeadCodeAfterReturnOrThrow { .. } => "E0271",
            Self::UninitializedVariableUsed { .. } => "E0272",
            Self::DuplicateAccessModifier { .. } => "E0273",
            Self::ConflictingModifierCombination { .. } => "E0274",
            Self::ModifierNotApplicableToDeclaration { .. } => "E0275",
            Self::VirtualOnNonMethod { .. } => "E0276",
            Self::OverrideOnNonMethod { .. } => "E0277",
            Self::NativeOnNonMethodOrFunction { .. } => "E0278",
            Self::InlineOnNonMethodOrFunction { .. } => "E0279",
            Self::StaticOnConstructorOrDestructor { .. } => "E0280",
            Self::VisibilityOnDestructor { .. } => "E0281",
            Self::OverrideWithoutBaseMethod { .. } => "E0282",
            Self::OverrideNonVirtualMethod { .. } => "E0283",
            Self::DestructorInvocationDirect { .. } => "E0284",
            Self::ExternSymbolMissingSignature { .. } => "E0285",
            Self::ParameterDefaultValueMismatch { .. } => "E0286",
            Self::InvalidExpressionInConstantContext { .. } => "E0287",
            Self::BitwiseOperatorOnNonInteger { .. } => "E0288",
        }
    }

    /// Returns the diagnostic source span.
    #[must_use]
    pub const fn source_span(&self) -> SourceSpan {
        match self {
            Self::UndefinedSymbol { span, .. }
            | Self::DuplicateSymbol { span, .. }
            | Self::TypeMismatch { span, .. }
            | Self::CyclicInheritance { span, .. }
            | Self::InvalidBaseClass { span, .. }
            | Self::UnimplementedInterfaceMethod { span, .. }
            | Self::MethodOverrideMismatch { span, .. }
            | Self::PrivateAccessViolation { span, .. }
            | Self::ProtectedAccessViolation { span, .. }
            | Self::CannotAssignToConst { span, .. }
            | Self::ReturnOutsideFunction { span }
            | Self::MissingReturn { span, .. }
            | Self::ReturnTypeMismatch { span, .. }
            | Self::OverloadAmbiguity { span, .. }
            | Self::NoMatchingOverload { span, .. }
            | Self::GenericArityMismatch { span, .. }
            | Self::CannotInstantiateAbstractType { span, .. }
            | Self::SuperConstructorMustBeFirst { span }
            | Self::BreakOrContinueOutsideLoop { span }
            | Self::InvalidOperatorOverload { span, .. }
            | Self::NonIndexableType { span, .. }
            | Self::NonCallableType { span, .. }
            | Self::CannotInferType { span }
            | Self::AmbiguousImport { span, .. }
            | Self::NonConstantCaseValue { span }
            | Self::StaticAccessViaDot { span, .. }
            | Self::InstanceAccessViaScopeResolution { span, .. }
            | Self::InvalidSuperCall { span }
            | Self::InvalidWeakModifier { span, .. }
            | Self::CircularAlias { span, .. }
            | Self::UnresolvedAliasTarget { span, .. }
            | Self::CannotResolveImport { span, .. }
            | Self::DuplicateLocalVariable { span, .. }
            | Self::DuplicateParameterName { span, .. }
            | Self::CannotInheritFromFinalClass { span, .. }
            | Self::SuperclassMustBeClass { span, .. }
            | Self::ImplementedTypeMustBeInterface { span, .. }
            | Self::ArgumentCountMismatch { span, .. }
            | Self::ArgumentTypeMismatch { span, .. }
            | Self::FieldInitializerTypeMismatch { span, .. }
            | Self::MemberAccessOnNonCompositeType { span, .. }
            | Self::SuperConstructorArgumentMismatch { span, .. }
            | Self::MainMethodSignatureInvalid { span, .. }
            | Self::MultipleMainMethodsFound { span, .. }
            | Self::DuplicateEnumVariant { span, .. }
            | Self::EnumExplicitValueNotInteger { span, .. }
            | Self::UnaryOperatorTypeMismatch { span, .. }
            | Self::BinaryOperatorTypeMismatch { span, .. }
            | Self::LogicalConditionNotBoolean { span, .. }
            | Self::ArrayLengthNotInteger { span, .. }
            | Self::IndexExpressionNotInteger { span, .. }
            | Self::TupleIndexOutOfRange { span, .. }
            | Self::CannotAssignToReadOnly { span, .. }
            | Self::VoidTypeNotAllowedHere { span }
            | Self::UnknownTypeInDeclaration { span, .. }
            | Self::DuplicateFieldDeclaration { span, .. }
            | Self::DuplicateMethodDeclaration { span, .. }
            | Self::DuplicateConstructorDeclaration { span, .. }
            | Self::ConstructorChainingCycle { span, .. }
            | Self::ThisUsedOutsideInstanceMethod { span }
            | Self::SuperUsedOutsideSubclass { span }
            | Self::StaticMethodAccessViaInstance { span, .. }
            | Self::InstanceMethodAccessViaStatic { span, .. }
            | Self::AbstractMethodInNonAbstractClass { span, .. }
            | Self::NonAbstractClassMissingAbstractImpl { span, .. }
            | Self::InterfaceFieldNotConstant { span, .. }
            | Self::FinalMethodOverridden { span, .. }
            | Self::NarrowingVisibilityInOverride { span, .. }
            | Self::InvalidCastType { span, .. }
            | Self::InvalidTypeForInstanceOf { span, .. }
            | Self::DeadCodeAfterReturnOrThrow { span }
            | Self::UninitializedVariableUsed { span, .. }
            | Self::DuplicateAccessModifier { span, .. }
            | Self::ConflictingModifierCombination { span, .. }
            | Self::ModifierNotApplicableToDeclaration { span, .. }
            | Self::VirtualOnNonMethod { span }
            | Self::OverrideOnNonMethod { span }
            | Self::NativeOnNonMethodOrFunction { span }
            | Self::InlineOnNonMethodOrFunction { span }
            | Self::StaticOnConstructorOrDestructor { span, .. }
            | Self::VisibilityOnDestructor { span }
            | Self::OverrideWithoutBaseMethod { span, .. }
            | Self::OverrideNonVirtualMethod { span, .. }
            | Self::DestructorInvocationDirect { span }
            | Self::ExternSymbolMissingSignature { span, .. }
            | Self::ParameterDefaultValueMismatch { span, .. }
            | Self::InvalidExpressionInConstantContext { span, .. }
            | Self::BitwiseOperatorOnNonInteger { span, .. } => *span,
        }
    }

    /// Convenience helper to create a diagnostic from a [`Span`].
    #[must_use]
    pub fn undefined_symbol(name: impl Into<String>, span: Span) -> Self {
        Self::UndefinedSymbol {
            symbol_name: name.into(),
            span: span.to_source_span(),
        }
    }

    /// Convenience helper for type mismatch diagnostics.
    #[must_use]
    pub fn type_mismatch(
        expected: impl Into<String>,
        found: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
            span: span.to_source_span(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_binding_error_codes() {
        let dummy_span = SourceSpan::new(0.into(), 1);
        let error_undefined = BindingError::UndefinedSymbol {
            symbol_name: "foo".to_string(),
            span: dummy_span,
        };
        assert_eq!(error_undefined.error_code(), "E0201");

        let error_dup_local = BindingError::DuplicateLocalVariable {
            name: "counter".to_string(),
            span: dummy_span,
        };
        assert_eq!(error_dup_local.error_code(), "E0233");

        let error_arg_count = BindingError::ArgumentCountMismatch {
            expected: 2,
            found: 1,
            span: dummy_span,
        };
        assert_eq!(error_arg_count.error_code(), "E0238");

        let error_bitwise = BindingError::BitwiseOperatorOnNonInteger {
            operator: "|".to_string(),
            found: "bool".to_string(),
            span: dummy_span,
        };
        assert_eq!(error_bitwise.error_code(), "E0288");
    }

    #[test]
    fn test_negative_no_raw_span_in_error_message() {
        let dummy_span = SourceSpan::new(20.into(), 8);
        let error = BindingError::UndefinedSymbol {
            symbol_name: "bar".to_string(),
            span: dummy_span,
        };
        let formatted = format!("{error}");
        assert!(!formatted.contains("SourceSpan"));
        assert!(!formatted.contains("offset:"));
    }
}
