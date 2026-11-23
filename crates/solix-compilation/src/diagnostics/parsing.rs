//! Diagnostic errors emitted during AST syntax parsing and deferred literal validation.

#![forbid(unsafe_code)]

use miette::{Diagnostic, SourceSpan};
use solix_common::span::Span;

/// Diagnostic error emitted when a syntactic violation or malformed AST construct is encountered.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Diagnostic)]
pub enum ParsingError {
    #[error("Expected semicolon ';' after statement")]
    #[diagnostic(
        code(solix::parser::expected_semicolon),
        help("Add a semicolon ';' to terminate this statement.")
    )]
    ExpectedSemicolon {
        #[label("missing semicolon here")]
        span: SourceSpan,
    },

    #[error("Expected identifier name")]
    #[diagnostic(
        code(solix::parser::expected_identifier),
        help("Provide a valid symbol identifier name.")
    )]
    ExpectedIdentifier {
        #[label("expected identifier")]
        span: SourceSpan,
    },

    #[error("Expected expression")]
    #[diagnostic(
        code(solix::parser::expected_expression),
        help("Provide a valid expression (literal, identifier, calculation, or function call).")
    )]
    ExpectedExpression {
        #[label("expected expression here")]
        span: SourceSpan,
    },

    #[error("Expected closing delimiter")]
    #[diagnostic(
        code(solix::parser::expected_closing_delimiter),
        help("Add the matching closing delimiter (')', '}}', or ']') to balance the expression.")
    )]
    ExpectedClosingDelimiter {
        #[label("unclosed delimiter")]
        span: SourceSpan,
    },

    #[error("Expected type name")]
    #[diagnostic(
        code(solix::parser::expected_type_name),
        help("Provide a valid primitive or user-defined type name (e.g. int32, string, void).")
    )]
    ExpectedTypeName {
        #[label("expected type name")]
        span: SourceSpan,
    },

    #[error("Invalid assignment target")]
    #[diagnostic(
        code(solix::parser::invalid_assignment_target),
        help(
            "The left-hand side of an assignment must be a valid mutable l-value (variable, field, or index)."
        )
    )]
    InvalidAssignmentTarget {
        #[label("invalid l-value target")]
        span: SourceSpan,
    },

    #[error("Invalid operand for increment or decrement operator")]
    #[diagnostic(
        code(solix::parser::invalid_increment_target),
        help(
            "Increment and decrement operators ('++', '--') can only be applied to mutable variables or fields."
        )
    )]
    InvalidIncrementTarget {
        #[label("cannot increment non-lvalue")]
        span: SourceSpan,
    },

    #[error("Array allocation requires size expression")]
    #[diagnostic(
        code(solix::parser::array_allocation_requires_size),
        help("Provide an array length expression between brackets, e.g. 'new int32[10]'.")
    )]
    ArrayAllocationRequiresSize {
        #[label("missing array size")]
        span: SourceSpan,
    },

    #[error("Expected '=>' arrow in lambda expression")]
    #[diagnostic(
        code(solix::parser::expected_lambda_arrow),
        help("Lambda expressions must use '=>' between parameters and body, e.g. '(x) => x + 1'.")
    )]
    ExpectedLambdaArrow {
        #[label("expected '=>' arrow")]
        span: SourceSpan,
    },

    #[error("Expected type name in cast expression")]
    #[diagnostic(
        code(solix::parser::expected_type_in_cast),
        help("Cast expressions require a destination type: 'cast(expr, TargetType)'.")
    )]
    ExpectedTypeInCast {
        #[label("missing cast target type")]
        span: SourceSpan,
    },

    #[error("'package' statement must be the very first statement in the file")]
    #[diagnostic(
        code(solix::parser::package_statement_not_first),
        help(
            "Move the 'package' statement to the top of the file before imports or declarations."
        )
    )]
    PackageStatementNotFirst {
        #[label("misplaced package statement")]
        span: SourceSpan,
    },

    #[error("Only one 'package' statement is allowed per file")]
    #[diagnostic(
        code(solix::parser::duplicate_package_statement),
        help("Remove duplicate package statement.")
    )]
    DuplicatePackageStatement {
        #[label("duplicate package statement")]
        span: SourceSpan,
    },

    #[error("Import statements must appear before class, interface, or enum declarations")]
    #[diagnostic(
        code(solix::parser::import_after_declaration),
        help("Move all 'import' statements to the top of the file before type declarations.")
    )]
    ImportAfterDeclaration {
        #[label("misplaced import statement")]
        span: SourceSpan,
    },

    #[error("Classes cannot be declared inside a function or method body")]
    #[diagnostic(
        code(solix::parser::class_in_local_scope),
        help("Move class declarations to the top level of the compilation unit.")
    )]
    ClassInLocalScope {
        #[label("illegal local class declaration")]
        span: SourceSpan,
    },

    #[error("Enums cannot be declared inside a function or method body")]
    #[diagnostic(
        code(solix::parser::enum_in_local_scope),
        help("Move enum declarations to the top level of the compilation unit.")
    )]
    EnumInLocalScope {
        #[label("illegal local enum declaration")]
        span: SourceSpan,
    },

    #[error("Interfaces cannot be declared inside a function or method body")]
    #[diagnostic(
        code(solix::parser::interface_in_local_scope),
        help("Move interface declarations to the top level of the compilation unit.")
    )]
    InterfaceInLocalScope {
        #[label("illegal local interface declaration")]
        span: SourceSpan,
    },

    #[error("Access modifiers are not allowed on local variables")]
    #[diagnostic(
        code(solix::parser::modifier_on_local_variable),
        help(
            "Remove 'public', 'private', 'protected', or 'internal' from local variable declaration."
        )
    )]
    ModifierOnLocalVariable {
        #[label("illegal modifier on local variable")]
        span: SourceSpan,
    },

    #[error("Operator overloads can only be declared inside a class body")]
    #[diagnostic(
        code(solix::parser::operator_outside_class),
        help("Move this operator overload inside a class definition.")
    )]
    OperatorOutsideClass {
        #[label("operator overload outside class")]
        span: SourceSpan,
    },

    #[error("Constructors can only be declared inside a class body")]
    #[diagnostic(
        code(solix::parser::constructor_outside_class),
        help("Move this constructor inside a class definition.")
    )]
    ConstructorOutsideClass {
        #[label("constructor outside class")]
        span: SourceSpan,
    },

    #[error(
        "Variable declarations are not allowed as immediate solitary branch or loop statements"
    )]
    #[diagnostic(
        code(solix::parser::solitary_declaration_in_branch_or_loop),
        help("Enclose the variable declaration in a block '{{ ... }}'.")
    )]
    SolitaryDeclarationInBranchOrLoop {
        #[label("declaration requires enclosing block")]
        span: SourceSpan,
    },

    #[error("Case label must be a constant literal")]
    #[diagnostic(
        code(solix::parser::case_label_not_constant),
        help("Case labels in switch statements must be integer, character, or string literals.")
    )]
    CaseLabelNotConstant {
        #[label("non-constant case label")]
        span: SourceSpan,
    },

    #[error("Statement without a preceding case in switch")]
    #[diagnostic(
        code(solix::parser::statement_before_case_in_switch),
        help("Move statement inside a 'case' or 'default' branch.")
    )]
    StatementBeforeCaseInSwitch {
        #[label("orphaned statement in switch")]
        span: SourceSpan,
    },

    #[error("Methods cannot be declared inside another method")]
    #[diagnostic(
        code(solix::parser::method_inside_method),
        help("Move method declaration to class or file level, or use a lambda closure instead.")
    )]
    MethodInsideMethod {
        #[label("nested method declaration")]
        span: SourceSpan,
    },

    #[error("'return' statement outside of function or method body")]
    #[diagnostic(
        code(solix::parser::return_outside_function),
        help("Return statements can only be executed inside functions, methods, or lambdas.")
    )]
    ReturnOutsideFunction {
        #[label("return outside function")]
        span: SourceSpan,
    },

    #[error("Interfaces cannot declare destructors")]
    #[diagnostic(
        code(solix::parser::interface_destructor),
        help(
            "Remove destructor from interface; interfaces cannot hold state or destructor logic."
        )
    )]
    InterfaceDestructor {
        #[label("destructor in interface")]
        span: SourceSpan,
    },

    #[error("Interfaces must not declare instance fields")]
    #[diagnostic(
        code(solix::parser::interface_field),
        help(
            "Remove instance fields from interface; interfaces can only define method signatures."
        )
    )]
    InterfaceField {
        #[label("instance field in interface")]
        span: SourceSpan,
    },

    #[error("Interface methods cannot have a body")]
    #[diagnostic(
        code(solix::parser::interface_method_with_body),
        help("Replace the method body with a semicolon ';'.")
    )]
    InterfaceMethodWithBody {
        #[label("interface method body")]
        span: SourceSpan,
    },

    #[error("Executable statements or bare blocks are not allowed directly in class body")]
    #[diagnostic(
        code(solix::parser::statement_directly_in_class),
        help("Move statements inside a method or constructor.")
    )]
    StatementDirectlyInClass {
        #[label("bare statement in class body")]
        span: SourceSpan,
    },

    #[error("Expression nesting depth exceeded maximum limit of 512")]
    #[diagnostic(
        code(solix::parser::expression_nesting_depth_exceeded),
        help("Simplify the expression by breaking it into intermediate variables.")
    )]
    ExpressionNestingDepthExceeded {
        #[label("expression nesting depth exceeded limit")]
        span: SourceSpan,
    },

    #[error("Class cannot declare multiple destructors")]
    #[diagnostic(
        code(solix::parser::multiple_destructors_in_class),
        help("A class can only have at most one destructor '~ClassName()'.")
    )]
    MultipleDestructorsInClass {
        #[label("duplicate destructor")]
        span: SourceSpan,
    },

    #[error("Destructors cannot have static, virtual, override, or abstract modifiers")]
    #[diagnostic(
        code(solix::parser::invalid_destructor_modifier),
        help("Remove invalid modifiers from destructor declaration.")
    )]
    InvalidDestructorModifier {
        #[label("invalid destructor modifier")]
        span: SourceSpan,
    },

    #[error("Destructor name does not match enclosing class name")]
    #[diagnostic(
        code(solix::parser::destructor_name_mismatch),
        help("The destructor name must exactly match the enclosing class name prefixed with '~'.")
    )]
    DestructorNameMismatch {
        #[label("destructor name mismatch")]
        span: SourceSpan,
    },

    #[error("Destructors cannot accept parameters")]
    #[diagnostic(
        code(solix::parser::destructor_parameters),
        help("Remove parameters; destructors must have an empty parameter list '()'.")
    )]
    DestructorParameters {
        #[label("illegal destructor parameters")]
        span: SourceSpan,
    },

    #[error("Constructor name does not match enclosing class name")]
    #[diagnostic(
        code(solix::parser::constructor_name_mismatch),
        help("The constructor name must exactly match the enclosing class name.")
    )]
    ConstructorNameMismatch {
        #[label("constructor name mismatch")]
        span: SourceSpan,
    },

    #[error("Constructors must not specify a return type")]
    #[diagnostic(
        code(solix::parser::constructor_return_type),
        help("Remove return type from constructor declaration.")
    )]
    ConstructorReturnType {
        #[label("return type on constructor")]
        span: SourceSpan,
    },

    #[error("Operator cannot be overloaded")]
    #[diagnostic(
        code(solix::parser::invalid_operator_overload),
        help("Only standard arithmetic, comparison, and indexing operators can be overloaded.")
    )]
    InvalidOperatorOverload {
        #[label("unsupported operator overload")]
        span: SourceSpan,
    },

    #[error("Variadic parameter must be the last parameter in parameter list")]
    #[diagnostic(
        code(solix::parser::variadic_parameter_not_last),
        help("Move the variadic pack parameter to the end of the parameter list.")
    )]
    VariadicParameterNotLast {
        #[label("variadic parameter not in final position")]
        span: SourceSpan,
    },

    #[error("Abstract or native method cannot have a body")]
    #[diagnostic(
        code(solix::parser::abstract_or_native_method_with_body),
        help("Replace the method body with a semicolon ';'.")
    )]
    AbstractOrNativeMethodWithBody {
        #[label("body not permitted on abstract/native method")]
        span: SourceSpan,
    },

    #[error("Expected block enclosed in braces '{{ ... }}'")]
    #[diagnostic(
        code(solix::parser::expected_block),
        help("Provide a block statement enclosed in curly braces '{{ ... }}'.")
    )]
    ExpectedBlock {
        #[label("expected block")]
        span: SourceSpan,
    },

    #[error("'try' statement must have at least one 'catch' or 'finally' clause")]
    #[diagnostic(
        code(solix::parser::try_missing_catch_or_finally),
        help("Add a 'catch' block or 'finally' block after the 'try' statement.")
    )]
    TryMissingCatchOrFinally {
        #[label("try without catch or finally")]
        span: SourceSpan,
    },

    #[error("Malformed catch parameter clause")]
    #[diagnostic(
        code(solix::parser::malformed_catch_parameter),
        help(
            "Catch clauses must specify an exception type and identifier: 'catch (Exception e)'."
        )
    )]
    MalformedCatchParameter {
        #[label("invalid catch parameter")]
        span: SourceSpan,
    },

    #[error("Malformed switch statement condition")]
    #[diagnostic(
        code(solix::parser::malformed_switch),
        help(
            "Enclose the switch discriminant expression in parentheses: 'switch (expr) {{ ... }}'."
        )
    )]
    MalformedSwitch {
        #[label("malformed switch condition")]
        span: SourceSpan,
    },

    #[error("Malformed for-loop statement")]
    #[diagnostic(
        code(solix::parser::malformed_for_loop),
        help("For loops require three semicolon-separated clauses: 'for (init; cond; step)'.")
    )]
    MalformedForLoop {
        #[label("malformed for loop clauses")]
        span: SourceSpan,
    },

    #[error("Malformed while-loop condition")]
    #[diagnostic(
        code(solix::parser::malformed_while_condition),
        help("Enclose while loop condition in parentheses: 'while (condition)'.")
    )]
    MalformedWhileCondition {
        #[label("malformed while condition")]
        span: SourceSpan,
    },

    #[error("Malformed do-while loop statement")]
    #[diagnostic(
        code(solix::parser::malformed_do_while),
        help("Do-while loops require 'while (condition);' after the statement body.")
    )]
    MalformedDoWhile {
        #[label("malformed do-while condition")]
        span: SourceSpan,
    },

    #[error("Malformed alias statement")]
    #[diagnostic(
        code(solix::parser::malformed_alias),
        help("Alias statements must follow the pattern: 'alias NewName = TargetType;'.")
    )]
    MalformedAlias {
        #[label("malformed alias syntax")]
        span: SourceSpan,
    },

    #[error("Unexpected token")]
    #[diagnostic(
        code(solix::parser::unexpected_token),
        help("Remove or replace this unexpected token with valid Solix grammar.")
    )]
    UnexpectedToken {
        #[label("unexpected token here")]
        span: SourceSpan,
    },

    #[error("Integer literal exceeds 64-bit capacity")]
    #[diagnostic(
        code(solix::parser::integer_literal_overflow),
        help("Reduce the integer literal value to fit within 64 bits (<= 18446744073709551615).")
    )]
    IntegerLiteralOverflow {
        #[label("integer literal overflow")]
        span: SourceSpan,
    },

    #[error("Invalid escape sequence in literal")]
    #[diagnostic(
        code(solix::parser::invalid_escape_sequence),
        help(r#"Valid escape sequences are: \n, \t, \r, \\, \', \", \0, \xHH."#)
    )]
    InvalidEscapeSequence {
        #[label("invalid escape sequence")]
        span: SourceSpan,
    },

    #[error("Empty character literal")]
    #[diagnostic(
        code(solix::parser::empty_character_literal),
        help("Character literals must contain exactly one character: 'a'.")
    )]
    EmptyCharacterLiteral {
        #[label("empty character literal")]
        span: SourceSpan,
    },

    #[error("Character literal contains multiple characters")]
    #[diagnostic(
        code(solix::parser::multi_character_literal),
        help(
            r#"Character literals can only contain one character; use double quotes '"' for strings."#
        )
    )]
    MultiCharacterLiteral {
        #[label("multi-character literal")]
        span: SourceSpan,
    },

    #[error("Extern function or method cannot have a body implementation")]
    #[diagnostic(
        code(solix::parser::extern_with_body),
        help(
            "Remove the body '{{ ... }}' and terminate the extern declaration with a semicolon ';'."
        )
    )]
    ExternWithBody {
        #[label("extern function with body")]
        span: SourceSpan,
    },

    #[error("Extern variable declaration cannot have an initializer")]
    #[diagnostic(
        code(solix::parser::extern_with_initializer),
        help("Remove the initializer expression from the extern variable declaration.")
    )]
    ExternWithInitializer {
        #[label("initializer not allowed on extern variable")]
        span: SourceSpan,
    },

    #[error("Extern modifier is not allowed inside a local function or block")]
    #[diagnostic(
        code(solix::parser::extern_inside_function),
        help("Move the extern declaration to package or module scope.")
    )]
    ExternInsideFunction {
        #[label("extern modifier in local scope")]
        span: SourceSpan,
    },

    #[error("Extern modifier cannot be combined with '{modifier}'")]
    #[diagnostic(
        code(solix::parser::invalid_extern_modifier),
        help("Remove either 'extern' or the conflicting modifier.")
    )]
    InvalidExternModifier {
        modifier: String,
        #[label("conflicting modifier combination")]
        span: SourceSpan,
    },

    // ------------------------------------------------------------------------
    // Granular Parser Errors (E0160 - E0200)
    // ------------------------------------------------------------------------
    #[error("Expected opening parenthesis '('")]
    #[diagnostic(
        code(solix::parser::expected_open_paren),
        help("Add an opening parenthesis '(' to group the expression or argument list.")
    )]
    ExpectedOpenParen {
        #[label("expected '('")]
        span: SourceSpan,
    },

    #[error("Expected closing brace '}}'")]
    #[diagnostic(
        code(solix::parser::expected_close_brace),
        help("Add a closing brace '}}' to terminate this block or body.")
    )]
    ExpectedCloseBrace {
        #[label("expected '}}'")]
        span: SourceSpan,
    },

    #[error("Expected opening brace '{{'")]
    #[diagnostic(
        code(solix::parser::expected_open_brace),
        help("Add an opening brace '{{' to begin this block or body.")
    )]
    ExpectedOpenBrace {
        #[label("expected '{{'")]
        span: SourceSpan,
    },

    #[error("Expected closing bracket ']'")]
    #[diagnostic(
        code(solix::parser::expected_close_bracket),
        help("Add a closing bracket ']' to terminate the array index or dimension.")
    )]
    ExpectedCloseBracket {
        #[label("expected ']'")]
        span: SourceSpan,
    },

    #[error("Expected opening bracket '['")]
    #[diagnostic(
        code(solix::parser::expected_open_bracket),
        help("Add an opening bracket '[' to begin the array dimension.")
    )]
    ExpectedOpenBracket {
        #[label("expected '['")]
        span: SourceSpan,
    },

    #[error("Expected closing angle bracket '>'")]
    #[diagnostic(
        code(solix::parser::expected_close_angle),
        help("Add a closing angle bracket '>' to close the type argument list.")
    )]
    ExpectedCloseAngle {
        #[label("expected '>'")]
        span: SourceSpan,
    },

    #[error("Expected ':' in ternary conditional expression")]
    #[diagnostic(
        code(solix::parser::expected_colon_in_ternary),
        help(
            "Ternary conditional expressions require a ':' separating the true and false expressions."
        )
    )]
    ExpectedColonInTernary {
        #[label("expected ':' in ternary expression")]
        span: SourceSpan,
    },

    #[error("Expected ':' after case or default label in switch")]
    #[diagnostic(
        code(solix::parser::expected_colon_in_case),
        help("Add a ':' after the case value or default label.")
    )]
    ExpectedColonInCase {
        #[label("expected ':' here")]
        span: SourceSpan,
    },

    #[error("Expected ':' before super constructor invocation")]
    #[diagnostic(
        code(solix::parser::expected_colon_after_super),
        help("Add a ':' before 'super(...)' constructor delegation.")
    )]
    ExpectedColonAfterSuper {
        #[label("expected ':' before super")]
        span: SourceSpan,
    },

    #[error("Expected ',' between expression and target type in cast expression")]
    #[diagnostic(
        code(solix::parser::expected_comma_in_cast),
        help("Cast expression syntax requires: 'cast(expression, TargetType)'.")
    )]
    ExpectedCommaInCast {
        #[label("expected ','")]
        span: SourceSpan,
    },

    #[error("Expected constructor call or array size expression after 'new'")]
    #[diagnostic(
        code(solix::parser::expected_constructor_call_or_size),
        help(
            "Provide an instantiated type call 'new Type(...)' or array allocation 'new Type[...]'."
        )
    )]
    ExpectedConstructorCallOrSize {
        #[label("expected constructor call or array size")]
        span: SourceSpan,
    },

    #[error("Unexpected '?' operator")]
    #[diagnostic(
        code(solix::parser::unexpected_question_operator),
        help("Remove or properly form the ternary conditional expression.")
    )]
    UnexpectedQuestionOperator {
        #[label("unexpected '?'")]
        span: SourceSpan,
    },

    #[error("Duplicate modifier '{modifier}' on declaration")]
    #[diagnostic(
        code(solix::parser::duplicate_modifier_on_declaration),
        help("Remove the redundant modifier.")
    )]
    DuplicateModifierOnDeclaration {
        modifier: String,
        #[label("duplicate modifier")]
        span: SourceSpan,
    },

    #[error("Conflicting visibility modifiers: cannot combine '{modifier1}' and '{modifier2}'")]
    #[diagnostic(
        code(solix::parser::conflicting_visibility_modifiers),
        help(
            "Specify at most one visibility modifier ('public', 'private', 'protected', 'internal')."
        )
    )]
    ConflictingVisibilityModifiers {
        modifier1: String,
        modifier2: String,
        #[label("conflicting visibility")]
        span: SourceSpan,
    },

    #[error("The 'static' modifier is not allowed on package-level functions")]
    #[diagnostic(
        code(solix::parser::static_on_free_function),
        help("Remove 'static'; top-level package functions are already package-scoped.")
    )]
    StaticOnFreeFunction {
        #[label("redundant or illegal static modifier")]
        span: SourceSpan,
    },

    #[error("Access modifier '{modifier}' is not allowed on standalone statements")]
    #[diagnostic(
        code(solix::parser::standalone_field_with_access_modifier),
        help("Move variable declarations with access modifiers into a class definition.")
    )]
    StandaloneFieldWithAccessModifier {
        modifier: String,
        #[label("illegal access modifier on standalone statement")]
        span: SourceSpan,
    },

    #[error("Malformed function pointer type signature")]
    #[diagnostic(
        code(solix::parser::malformed_function_pointer_type),
        help("Function pointer types must follow: 'ReturnType(*)(ParamType1, ParamType2)'.")
    )]
    MalformedFunctionPointerType {
        #[label("malformed function pointer")]
        span: SourceSpan,
    },

    #[error("Expected 'catch' or 'finally' block after 'try'")]
    #[diagnostic(
        code(solix::parser::expected_catch_or_finally_block),
        help("A 'try' block must be followed by at least one 'catch' or 'finally' block.")
    )]
    ExpectedCatchOrFinallyBlock {
        #[label("expected catch or finally")]
        span: SourceSpan,
    },

    #[error("Expected identifier in parameter declaration")]
    #[diagnostic(
        code(solix::parser::expected_identifier_in_parameter),
        help("Provide a parameter name following the parameter type.")
    )]
    ExpectedIdentifierInParameter {
        #[label("expected parameter identifier")]
        span: SourceSpan,
    },

    #[error("Expected parameter list '(...)' in function or method declaration")]
    #[diagnostic(
        code(solix::parser::expected_parameter_list),
        help("Add parameter list parentheses '()' after the function name.")
    )]
    ExpectedParameterList {
        #[label("expected parameter list")]
        span: SourceSpan,
    },

    #[error("Expected interface identifier list after 'implements'")]
    #[diagnostic(
        code(solix::parser::expected_interface_list),
        help("Provide one or more comma-separated interface names after 'implements'.")
    )]
    ExpectedInterfaceList {
        #[label("expected interface list")]
        span: SourceSpan,
    },

    #[error("Expected superclass identifier after 'extends'")]
    #[diagnostic(
        code(solix::parser::expected_superclass_list),
        help("Provide the superclass name after 'extends'.")
    )]
    ExpectedSuperclassList {
        #[label("expected superclass name")]
        span: SourceSpan,
    },

    #[error("Expected '{{' enclosing enum variant body")]
    #[diagnostic(
        code(solix::parser::expected_enum_brace),
        help("Enclose enum variants in curly braces '{{ ... }}'.")
    )]
    ExpectedEnumBrace {
        #[label("expected '{{' for enum body")]
        span: SourceSpan,
    },

    #[error("Expected '{{' enclosing switch body")]
    #[diagnostic(
        code(solix::parser::expected_switch_brace),
        help("Enclose switch cases in curly braces '{{ ... }}'.")
    )]
    ExpectedSwitchBrace {
        #[label("expected '{{' for switch body")]
        span: SourceSpan,
    },

    #[error("Switch statement cannot contain more than one 'default' clause")]
    #[diagnostic(
        code(solix::parser::duplicate_default_in_switch),
        help("Remove the redundant 'default' branch.")
    )]
    DuplicateDefaultInSwitch {
        #[label("duplicate default label")]
        span: SourceSpan,
    },

    #[error("Modifiers are not allowed on package or import statements")]
    #[diagnostic(
        code(solix::parser::modifier_on_module_statement),
        help("Remove modifiers from package or import statements.")
    )]
    ModifierOnModuleStatement {
        #[label("illegal modifier on module statement")]
        span: SourceSpan,
    },

    #[error("Expected index expression inside brackets '[]'")]
    #[diagnostic(
        code(solix::parser::expected_expression_in_index),
        help("Provide an expression between index brackets '[index]'.")
    )]
    ExpectedExpressionInIndex {
        #[label("missing index expression")]
        span: SourceSpan,
    },

    #[error("Empty tuple expressions '()' are not permitted; tuples require at least one element")]
    #[diagnostic(
        code(solix::parser::empty_tuple_expression),
        help("Add at least one element to the tuple: '(item,)' or '(item1, item2)'.")
    )]
    EmptyTupleExpression {
        #[label("empty tuple not allowed")]
        span: SourceSpan,
    },

    #[error("Malformed type bound or constraint")]
    #[diagnostic(
        code(solix::parser::malformed_type_bound),
        help("Type bounds must follow valid generic constraint syntax.")
    )]
    MalformedTypeBound {
        #[label("malformed type bound")]
        span: SourceSpan,
    },

    #[error("Expected condition expression in 'if' statement")]
    #[diagnostic(
        code(solix::parser::expected_condition_in_if),
        help("Provide a boolean condition inside parentheses: 'if (condition)'.")
    )]
    ExpectedConditionInIf {
        #[label("missing if condition")]
        span: SourceSpan,
    },

    #[error("Expected condition expression in 'while' loop")]
    #[diagnostic(
        code(solix::parser::expected_condition_in_while),
        help("Provide a boolean condition inside parentheses: 'while (condition)'.")
    )]
    ExpectedConditionInWhile {
        #[label("missing while condition")]
        span: SourceSpan,
    },

    #[error("Enum declaration must contain at least one variant")]
    #[diagnostic(
        code(solix::parser::empty_enum_declaration),
        help("Add at least one variant inside the enum body.")
    )]
    EmptyEnumDeclaration {
        #[label("empty enum declaration")]
        span: SourceSpan,
    },

    #[error("Unexpected standalone semicolon outside statement context")]
    #[diagnostic(
        code(solix::parser::unexpected_semicolon),
        help("Remove extraneous semicolon ';'.")
    )]
    UnexpectedSemicolon {
        #[label("unexpected semicolon")]
        span: SourceSpan,
    },

    #[error("Consecutive expressions without an intervening operator")]
    #[diagnostic(
        code(solix::parser::missing_operator_in_expression),
        help("Add an operator between adjacent expressions or sub-expressions.")
    )]
    MissingOperatorInExpression {
        #[label("missing operator")]
        span: SourceSpan,
    },

    #[error("Malformed constructor chaining invocation: expected 'super(...)' or 'this(...)'")]
    #[diagnostic(
        code(solix::parser::malformed_constructor_chaining),
        help(
            "Use ': super(...)' or ': this(...)' immediately following the constructor parameter list."
        )
    )]
    MalformedConstructorChaining {
        #[label("malformed constructor chaining")]
        span: SourceSpan,
    },

    #[error("Unexpected token at top level: '{token}'")]
    #[diagnostic(
        code(solix::parser::invalid_top_level_token),
        help(
            "Expected package, import, class, interface, enum, or function declaration at top level."
        )
    )]
    InvalidTopLevelToken {
        token: String,
        #[label("unexpected top-level token")]
        span: SourceSpan,
    },

    #[error("Modifier '{modifier}' is not valid on class declarations")]
    #[diagnostic(
        code(solix::parser::invalid_class_modifier),
        help(
            "Valid class modifiers are: 'public', 'private', 'protected', 'internal', 'abstract', 'final'."
        )
    )]
    InvalidClassModifier {
        modifier: String,
        #[label("invalid class modifier")]
        span: SourceSpan,
    },

    #[error("Modifier '{modifier}' is not valid on interface declarations")]
    #[diagnostic(
        code(solix::parser::invalid_interface_modifier),
        help("Valid interface modifiers are: 'public', 'private', 'protected', 'internal'.")
    )]
    InvalidInterfaceModifier {
        modifier: String,
        #[label("invalid interface modifier")]
        span: SourceSpan,
    },

    #[error("Modifier '{modifier}' is not valid on enum declarations")]
    #[diagnostic(
        code(solix::parser::invalid_enum_modifier),
        help("Valid enum modifiers are: 'public', 'private', 'protected', 'internal'.")
    )]
    InvalidEnumModifier {
        modifier: String,
        #[label("invalid enum modifier")]
        span: SourceSpan,
    },

    #[error("Modifier '{modifier}' is not valid on package-level functions")]
    #[diagnostic(
        code(solix::parser::invalid_free_function_modifier),
        help(
            "Valid free function modifiers are: 'public', 'private', 'internal', 'native', 'inline', 'extern'."
        )
    )]
    InvalidFreeFunctionModifier {
        modifier: String,
        #[label("invalid free function modifier")]
        span: SourceSpan,
    },

    #[error("Modifier '{modifier}' is not valid on operator overload declarations")]
    #[diagnostic(
        code(solix::parser::invalid_operator_modifier),
        help("Valid operator modifiers are: 'public', 'inline'.")
    )]
    InvalidOperatorModifier {
        modifier: String,
        #[label("invalid operator modifier")]
        span: SourceSpan,
    },

    #[error("Dot '.' is not allowed in type paths; use '::' instead")]
    #[diagnostic(
        code(solix::parser::dot_not_allowed_in_type_path),
        help(
            "Replace '.' with '::' for type and namespace paths (e.g. 'solix::collections::List'). Dot '.' is reserved for instance member access."
        )
    )]
    DotNotAllowedInTypePath {
        #[label("'.' not allowed in type path; use '::'")]
        span: SourceSpan,
    },
}

impl ParsingError {
    /// Returns the standardized compiler error code string (e.g. `"E0101"`).
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::ExpectedSemicolon { .. } => "E0101",
            Self::ExpectedIdentifier { .. } => "E0102",
            Self::ExpectedExpression { .. } => "E0103",
            Self::ExpectedClosingDelimiter { .. } => "E0104",
            Self::ExpectedTypeName { .. } => "E0105",
            Self::InvalidAssignmentTarget { .. } => "E0106",
            Self::InvalidIncrementTarget { .. } => "E0107",
            Self::ArrayAllocationRequiresSize { .. } => "E0108",
            Self::ExpectedLambdaArrow { .. } => "E0109",
            Self::ExpectedTypeInCast { .. } => "E0110",
            Self::PackageStatementNotFirst { .. } => "E0111",
            Self::DuplicatePackageStatement { .. } => "E0112",
            Self::ImportAfterDeclaration { .. } => "E0113",
            Self::ClassInLocalScope { .. } => "E0114",
            Self::EnumInLocalScope { .. } => "E0115",
            Self::InterfaceInLocalScope { .. } => "E0116",
            Self::ModifierOnLocalVariable { .. } => "E0117",
            Self::OperatorOutsideClass { .. } => "E0118",
            Self::ConstructorOutsideClass { .. } => "E0119",
            Self::SolitaryDeclarationInBranchOrLoop { .. } => "E0120",
            Self::CaseLabelNotConstant { .. } => "E0121",
            Self::StatementBeforeCaseInSwitch { .. } => "E0122",
            Self::MethodInsideMethod { .. } => "E0123",
            Self::ReturnOutsideFunction { .. } => "E0124",
            Self::InterfaceDestructor { .. } => "E0125",
            Self::InterfaceField { .. } => "E0126",
            Self::InterfaceMethodWithBody { .. } => "E0127",
            Self::StatementDirectlyInClass { .. } => "E0128",
            Self::MultipleDestructorsInClass { .. } => "E0129",
            Self::InvalidDestructorModifier { .. } => "E0130",
            Self::DestructorNameMismatch { .. } => "E0131",
            Self::DestructorParameters { .. } => "E0132",
            Self::ConstructorNameMismatch { .. } => "E0133",
            Self::ConstructorReturnType { .. } => "E0134",
            Self::InvalidOperatorOverload { .. } => "E0135",
            Self::VariadicParameterNotLast { .. } => "E0136",
            Self::ExternWithBody { .. } => "E0137",
            Self::AbstractOrNativeMethodWithBody { .. } => "E0138",
            Self::ExpectedBlock { .. } => "E0139",
            Self::TryMissingCatchOrFinally { .. } => "E0140",
            Self::MalformedCatchParameter { .. } => "E0141",
            Self::MalformedSwitch { .. } => "E0142",
            Self::MalformedForLoop { .. } => "E0143",
            Self::MalformedWhileCondition { .. } => "E0144",
            Self::MalformedDoWhile { .. } => "E0145",
            Self::MalformedAlias { .. } => "E0146",
            Self::UnexpectedToken { .. } => "E0147",
            Self::ExpressionNestingDepthExceeded { .. } => "E0148",
            Self::IntegerLiteralOverflow { .. } => "E0150",
            Self::InvalidEscapeSequence { .. } => "E0151",
            Self::EmptyCharacterLiteral { .. } => "E0152",
            Self::MultiCharacterLiteral { .. } => "E0153",
            Self::ExternWithInitializer { .. } => "E0154",
            Self::ExternInsideFunction { .. } => "E0155",
            Self::InvalidExternModifier { .. } => "E0156",

            Self::ExpectedOpenParen { .. } => "E0160",
            Self::ExpectedCloseBrace { .. } => "E0161",
            Self::ExpectedOpenBrace { .. } => "E0162",
            Self::ExpectedCloseBracket { .. } => "E0163",
            Self::ExpectedOpenBracket { .. } => "E0164",
            Self::ExpectedCloseAngle { .. } => "E0165",
            Self::ExpectedColonInTernary { .. } => "E0166",
            Self::ExpectedColonInCase { .. } => "E0167",
            Self::ExpectedColonAfterSuper { .. } => "E0168",
            Self::ExpectedCommaInCast { .. } => "E0169",
            Self::ExpectedConstructorCallOrSize { .. } => "E0170",
            Self::UnexpectedQuestionOperator { .. } => "E0171",
            Self::DuplicateModifierOnDeclaration { .. } => "E0172",
            Self::ConflictingVisibilityModifiers { .. } => "E0173",
            Self::StaticOnFreeFunction { .. } => "E0174",
            Self::StandaloneFieldWithAccessModifier { .. } => "E0175",
            Self::MalformedFunctionPointerType { .. } => "E0176",
            Self::ExpectedCatchOrFinallyBlock { .. } => "E0177",
            Self::ExpectedIdentifierInParameter { .. } => "E0178",
            Self::ExpectedParameterList { .. } => "E0179",
            Self::ExpectedInterfaceList { .. } => "E0180",
            Self::ExpectedSuperclassList { .. } => "E0181",
            Self::ExpectedEnumBrace { .. } => "E0182",
            Self::ExpectedSwitchBrace { .. } => "E0183",
            Self::DuplicateDefaultInSwitch { .. } => "E0184",
            Self::ModifierOnModuleStatement { .. } => "E0185",
            Self::ExpectedExpressionInIndex { .. } => "E0186",
            Self::EmptyTupleExpression { .. } => "E0187",
            Self::MalformedTypeBound { .. } => "E0188",
            Self::ExpectedConditionInIf { .. } => "E0189",
            Self::ExpectedConditionInWhile { .. } => "E0190",
            Self::EmptyEnumDeclaration { .. } => "E0191",
            Self::UnexpectedSemicolon { .. } => "E0192",
            Self::MissingOperatorInExpression { .. } => "E0193",
            Self::MalformedConstructorChaining { .. } => "E0194",
            Self::InvalidTopLevelToken { .. } => "E0195",
            Self::InvalidClassModifier { .. } => "E0196",
            Self::InvalidInterfaceModifier { .. } => "E0197",
            Self::InvalidEnumModifier { .. } => "E0198",
            Self::InvalidFreeFunctionModifier { .. } => "E0199",
            Self::InvalidOperatorModifier { .. } => "E0200",
            Self::DotNotAllowedInTypePath { .. } => "E0201",
        }
    }

    /// Returns the diagnostic source span.
    #[must_use]
    pub const fn source_span(&self) -> SourceSpan {
        match self {
            Self::ExpectedSemicolon { span }
            | Self::ExpectedIdentifier { span }
            | Self::ExpectedExpression { span }
            | Self::ExpectedClosingDelimiter { span }
            | Self::ExpectedTypeName { span }
            | Self::InvalidAssignmentTarget { span }
            | Self::InvalidIncrementTarget { span }
            | Self::ArrayAllocationRequiresSize { span }
            | Self::ExpectedLambdaArrow { span }
            | Self::ExpectedTypeInCast { span }
            | Self::PackageStatementNotFirst { span }
            | Self::DuplicatePackageStatement { span }
            | Self::ImportAfterDeclaration { span }
            | Self::ClassInLocalScope { span }
            | Self::EnumInLocalScope { span }
            | Self::InterfaceInLocalScope { span }
            | Self::ModifierOnLocalVariable { span }
            | Self::OperatorOutsideClass { span }
            | Self::ConstructorOutsideClass { span }
            | Self::SolitaryDeclarationInBranchOrLoop { span }
            | Self::CaseLabelNotConstant { span }
            | Self::StatementBeforeCaseInSwitch { span }
            | Self::MethodInsideMethod { span }
            | Self::ReturnOutsideFunction { span }
            | Self::InterfaceDestructor { span }
            | Self::InterfaceField { span }
            | Self::InterfaceMethodWithBody { span }
            | Self::StatementDirectlyInClass { span }
            | Self::MultipleDestructorsInClass { span }
            | Self::InvalidDestructorModifier { span }
            | Self::DestructorNameMismatch { span }
            | Self::DestructorParameters { span }
            | Self::ConstructorNameMismatch { span }
            | Self::ConstructorReturnType { span }
            | Self::InvalidOperatorOverload { span }
            | Self::VariadicParameterNotLast { span }
            | Self::AbstractOrNativeMethodWithBody { span }
            | Self::ExpectedBlock { span }
            | Self::TryMissingCatchOrFinally { span }
            | Self::MalformedCatchParameter { span }
            | Self::MalformedSwitch { span }
            | Self::MalformedForLoop { span }
            | Self::MalformedWhileCondition { span }
            | Self::MalformedDoWhile { span }
            | Self::MalformedAlias { span }
            | Self::UnexpectedToken { span }
            | Self::ExpressionNestingDepthExceeded { span }
            | Self::IntegerLiteralOverflow { span }
            | Self::InvalidEscapeSequence { span }
            | Self::EmptyCharacterLiteral { span }
            | Self::MultiCharacterLiteral { span }
            | Self::ExternWithBody { span }
            | Self::ExternWithInitializer { span }
            | Self::ExternInsideFunction { span }
            | Self::InvalidExternModifier { span, .. }
            | Self::ExpectedOpenParen { span }
            | Self::ExpectedCloseBrace { span }
            | Self::ExpectedOpenBrace { span }
            | Self::ExpectedCloseBracket { span }
            | Self::ExpectedOpenBracket { span }
            | Self::ExpectedCloseAngle { span }
            | Self::ExpectedColonInTernary { span }
            | Self::ExpectedColonInCase { span }
            | Self::ExpectedColonAfterSuper { span }
            | Self::ExpectedCommaInCast { span }
            | Self::ExpectedConstructorCallOrSize { span }
            | Self::UnexpectedQuestionOperator { span }
            | Self::DuplicateModifierOnDeclaration { span, .. }
            | Self::ConflictingVisibilityModifiers { span, .. }
            | Self::StaticOnFreeFunction { span }
            | Self::StandaloneFieldWithAccessModifier { span, .. }
            | Self::MalformedFunctionPointerType { span }
            | Self::ExpectedCatchOrFinallyBlock { span }
            | Self::ExpectedIdentifierInParameter { span }
            | Self::ExpectedParameterList { span }
            | Self::ExpectedInterfaceList { span }
            | Self::ExpectedSuperclassList { span }
            | Self::ExpectedEnumBrace { span }
            | Self::ExpectedSwitchBrace { span }
            | Self::DuplicateDefaultInSwitch { span }
            | Self::ModifierOnModuleStatement { span }
            | Self::ExpectedExpressionInIndex { span }
            | Self::EmptyTupleExpression { span }
            | Self::MalformedTypeBound { span }
            | Self::ExpectedConditionInIf { span }
            | Self::ExpectedConditionInWhile { span }
            | Self::EmptyEnumDeclaration { span }
            | Self::UnexpectedSemicolon { span }
            | Self::MissingOperatorInExpression { span }
            | Self::MalformedConstructorChaining { span }
            | Self::InvalidTopLevelToken { span, .. }
            | Self::InvalidClassModifier { span, .. }
            | Self::InvalidInterfaceModifier { span, .. }
            | Self::InvalidEnumModifier { span, .. }
            | Self::InvalidFreeFunctionModifier { span, .. }
            | Self::InvalidOperatorModifier { span, .. }
            | Self::DotNotAllowedInTypePath { span } => *span,
        }
    }

    /// Convenience constructor from [`Span`].
    #[must_use]
    pub fn from_span(span: Span, construct: fn(SourceSpan) -> Self) -> Self {
        construct(span.to_source_span())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_parsing_error_codes() {
        let dummy_span = SourceSpan::new(0.into(), 1);
        let error_semicolon = ParsingError::ExpectedSemicolon { span: dummy_span };
        assert_eq!(error_semicolon.error_code(), "E0101");

        let error_open_paren = ParsingError::ExpectedOpenParen { span: dummy_span };
        assert_eq!(error_open_paren.error_code(), "E0160");

        let error_close_brace = ParsingError::ExpectedCloseBrace { span: dummy_span };
        assert_eq!(error_close_brace.error_code(), "E0161");

        let error_dup_default = ParsingError::DuplicateDefaultInSwitch { span: dummy_span };
        assert_eq!(error_dup_default.error_code(), "E0184");

        let error_empty_enum = ParsingError::EmptyEnumDeclaration { span: dummy_span };
        assert_eq!(error_empty_enum.error_code(), "E0191");

        let error_invalid_op_mod = ParsingError::InvalidOperatorModifier {
            modifier: "virtual".to_string(),
            span: dummy_span,
        };
        assert_eq!(error_invalid_op_mod.error_code(), "E0200");

        let error_dot_in_type = ParsingError::DotNotAllowedInTypePath { span: dummy_span };
        assert_eq!(error_dot_in_type.error_code(), "E0201");
    }

    #[test]
    fn test_negative_no_raw_span_in_error_message() {
        let dummy_span = SourceSpan::new(10.into(), 5);
        let error = ParsingError::ExpectedSemicolon { span: dummy_span };
        let formatted = format!("{error}");
        assert!(!formatted.contains("SourceSpan"));
        assert!(!formatted.contains("offset:"));
    }
}
