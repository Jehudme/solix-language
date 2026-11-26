//! Comprehensive integration test suite for the Solix AST parser and grammar engine.

#![forbid(unsafe_code)]

mod common;

use common::LanguageFixture;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::lexing::{lex, lex_source};
use solix_compilation::phases::parser::{
    AstVisitor, ClassDeclaration, EnumDeclaration, Expression, FieldDeclaration,
    InterfaceDeclaration, MethodDeclaration, Statement, Visitable, parse, parse_source,
};
use std::collections::HashSet;
use std::path::Path;

/// Test visitor that counts occurrences of various AST node types.
#[derive(Default)]
struct AstCounterVisitor {
    classes_count: usize,
    interfaces_count: usize,
    enums_count: usize,
    methods_count: usize,
    fields_count: usize,
    statements_count: usize,
    expressions_count: usize,
}

impl AstVisitor for AstCounterVisitor {
    fn visit_class_declaration(&mut self, node: &ClassDeclaration) {
        self.classes_count += 1;
        solix_compilation::phases::parser::visitor::walk_class_declaration(self, node);
    }

    fn visit_interface_declaration(&mut self, node: &InterfaceDeclaration) {
        self.interfaces_count += 1;
        solix_compilation::phases::parser::visitor::walk_interface_declaration(self, node);
    }

    fn visit_enum_declaration(&mut self, node: &EnumDeclaration) {
        self.enums_count += 1;
        solix_compilation::phases::parser::visitor::walk_enum_declaration(self, node);
    }

    fn visit_method_declaration(&mut self, node: &MethodDeclaration) {
        self.methods_count += 1;
        solix_compilation::phases::parser::visitor::walk_method_declaration(self, node);
    }

    fn visit_field_declaration(&mut self, node: &FieldDeclaration) {
        self.fields_count += 1;
        solix_compilation::phases::parser::visitor::walk_field_declaration(self, node);
    }

    fn visit_statement(&mut self, node: &Statement) {
        self.statements_count += 1;
        solix_compilation::phases::parser::visitor::walk_statement(self, node);
    }

    fn visit_expression(&mut self, node: &Expression) {
        self.expressions_count += 1;
        solix_compilation::phases::parser::visitor::walk_expression(self, node);
    }
}

#[test]
fn test_positive_master_syntax_fixture_parses_completely() {
    let fixture_path = Path::new("tests/fixtures/syntax/positive/master_all_statements.slx");
    let fixture = LanguageFixture::load_from_path(fixture_path)
        .expect("Failed to load master_all_statements.slx positive fixture");

    let tokens = lex_source(0, &fixture.source_content)
        .expect("Lexing master_all_statements.slx should succeed without errors");

    let parse_result = parse_source(0, &tokens, &fixture.source_content);
    assert!(
        parse_result.is_ok(),
        "Parsing master_all_statements.slx failed: {:#?}",
        parse_result.err()
    );

    let ast = parse_result.expect("Expected valid ProgramAst");

    // Verify Package Statement
    assert!(ast.package.is_some(), "Package statement must be present");
    let package = ast.package.as_ref().expect("Package statement");
    assert_eq!(
        package.path.len(),
        3,
        "Package path should have 3 segments: solix.tests.master"
    );

    // Verify Imports
    assert_eq!(ast.imports.len(), 4, "Expected 4 import statements");
    assert!(
        ast.imports[2].alias.is_some(),
        "Expected alias on 3rd import"
    );
    assert!(
        ast.imports[3].is_wildcard,
        "Expected wildcard on 4th import"
    );

    // Verify Aliases
    assert_eq!(ast.aliases.len(), 4, "Expected 4 alias statements");

    // Verify Declarations
    assert!(
        ast.declarations.len() >= 4,
        "Expected at least 4 top-level declarations (interfaces, enums, classes)"
    );

    // Verify Visitor Traversal
    let mut counter = AstCounterVisitor::default();
    ast.accept(&mut counter);

    assert!(
        counter.classes_count >= 2,
        "Expected at least 2 classes counted"
    );
    assert!(
        counter.interfaces_count >= 1,
        "Expected at least 1 interface counted"
    );
    assert!(counter.enums_count >= 1, "Expected at least 1 enum counted");
    assert!(
        counter.methods_count >= 3,
        "Expected at least 3 methods counted"
    );
    assert!(
        counter.fields_count >= 1,
        "Expected at least 1 field counted"
    );
    assert!(
        counter.statements_count >= 20,
        "Expected at least 20 statements counted"
    );
    assert!(
        counter.expressions_count >= 50,
        "Expected at least 50 expressions counted"
    );
}

#[test]
fn test_positive_compilation_context_pipeline() {
    let fixture_path = Path::new("tests/fixtures/syntax/positive/master_all_statements.slx");
    let fixture =
        LanguageFixture::load_from_path(fixture_path).expect("Failed to load positive fixture");

    let options =
        CompilationOptions::new().with_source("master_all_statements.slx", &fixture.source_content);
    let mut context = CompilationContext::new(options);

    let lex_meta = lex(&mut context).expect("Lexing context should succeed");
    assert!(lex_meta.tokens_count > 0);

    let parse_meta = parse(&mut context).expect("Parsing context should succeed");
    assert!(parse_meta.declarations_count > 0);
    assert_eq!(context.ast.len(), 1);
    assert!(context.get_ast(0).is_some());
    assert!(
        context
            .get_ast(0)
            .and_then(|ast| ast.package.as_ref())
            .is_some()
    );
}

#[test]
fn test_negative_master_syntax_fixture_detects_all_errors() {
    let fixture_path = Path::new("tests/fixtures/syntax/negative/master_invalid_syntax.slx");
    let fixture = LanguageFixture::load_from_path(fixture_path)
        .expect("Failed to load master_invalid_syntax.slx negative fixture");

    let tokens = lex_source(0, &fixture.source_content)
        .expect("Lexing negative fixture should produce token stream");

    let parse_result = parse_source(0, &tokens, &fixture.source_content);
    assert!(
        parse_result.is_err(),
        "master_invalid_syntax.slx must fail parsing and return errors"
    );

    let errors = parse_result.expect_err("Expected accumulated syntax errors");
    assert!(
        errors.len() >= 25,
        "Expected at least 25 accumulated syntax errors, found {}: {:#?}",
        errors.len(),
        errors
    );

    let mut detected_codes = HashSet::new();
    for error in &errors {
        detected_codes.insert(error.error_code());
        let span = error.source_span();
        assert!(span.offset() > 0, "Error span offset must be valid");
    }

    // Verify key error codes from specification are all detected
    let expected_codes = [
        "E0118", // Operator outside class
        "E0119", // Constructor outside class
        "E0124", // Return outside function
        "E0146", // Malformed alias
        "E0111", // Package not first
        "E0112", // Duplicate package
        "E0113", // Import after declaration
        "E0125", // Interface destructor
        "E0126", // Interface field
        "E0127", // Interface method with body
        "E0134", // Constructor return type
        "E0130", // Invalid destructor modifier
        "E0131", // Destructor name mismatch
        "E0132", // Destructor parameters
        "E0129", // Multiple destructors
        "E0138", // Abstract method with body
        "E0136", // Variadic parameter not last
        "E0114", // Local class
        "E0115", // Local enum
        "E0116", // Local interface
        "E0117", // Modifier on local variable
        "E0120", // Solitary declaration in branch
        "E0123", // Method inside method
        "E0140", // Try missing catch or finally
        "E0141", // Malformed catch parameter
        "E0143", // Malformed for loop
        "E0144", // Malformed while
        "E0145", // Malformed do-while
        "E0106", // Invalid assignment target
        "E0107", // Invalid increment target
        "E0108", // Array allocation requires size
        "E0110", // Cast missing type
        "E0150", // Integer overflow
        "E0151", // Invalid escape sequence
        "E0152", // Empty char literal
        "E0153", // Multi char literal
    ];

    for expected_code in expected_codes {
        assert!(
            detected_codes.contains(expected_code),
            "Expected error code {} was not emitted. Detected codes: {:#?}",
            expected_code,
            detected_codes
        );
    }
}

#[test]
fn test_strict_namespace_type_path_syntax() {
    // 1. Dot syntax in type paths MUST fail with E0157
    let invalid_dot_type_source = "solix.collections.List<int32> items;";
    let tokens_invalid = lex_source(0, invalid_dot_type_source).expect("Lexing succeeds");
    let parse_result_invalid = parse_source(0, &tokens_invalid, invalid_dot_type_source);
    assert!(
        parse_result_invalid.is_err(),
        "Dot in type annotation MUST be rejected"
    );
    let detected_errors = parse_result_invalid.unwrap_err();
    assert!(
        detected_errors
            .iter()
            .any(|error| error.error_code() == "E0157"),
        "Expected E0157 (DotNotAllowedInTypePath) when using '.' in type annotation"
    );

    // 2. Scope resolution syntax '::' in type paths MUST succeed
    let valid_colon_type_source = "solix::collections::List<int32> items;";
    let tokens_valid = lex_source(0, valid_colon_type_source).expect("Lexing succeeds");
    let parse_result_valid = parse_source(0, &tokens_valid, valid_colon_type_source);
    assert!(
        parse_result_valid.is_ok(),
        "Scope resolution '::' in type annotation MUST succeed"
    );

    // 3. Both '::' and '.' in package statements MUST succeed
    let package_dot_source = "package game.logic; class Player {}";
    let tokens_pkg_dot = lex_source(0, package_dot_source).expect("Lexing succeeds");
    assert!(parse_source(0, &tokens_pkg_dot, package_dot_source).is_ok());

    let package_colon_source = "package game::logic; class Player {}";
    let tokens_pkg_colon = lex_source(0, package_colon_source).expect("Lexing succeeds");
    assert!(parse_source(0, &tokens_pkg_colon, package_colon_source).is_ok());

    // 4. Both '::' and '.' in import statements MUST succeed
    let import_dot_source = "import solix.collections.List;";
    let tokens_import_dot = lex_source(0, import_dot_source).expect("Lexing succeeds");
    assert!(parse_source(0, &tokens_import_dot, import_dot_source).is_ok());

    let import_colon_source = "import solix::collections::List;";
    let tokens_import_colon = lex_source(0, import_colon_source).expect("Lexing succeeds");
    assert!(parse_source(0, &tokens_import_colon, import_colon_source).is_ok());
}

#[test]
fn test_char_literal_escaped_character_bounds_validation() {
    // 1. Valid single escape sequences and hex escape sequences
    let valid_snippets = [
        "char newline = '\\n';",
        "char tab = '\\t';",
        "char carriage_return = '\\r';",
        "char backslash = '\\\\';",
        "char single_quote = '\\'';",
        "char double_quote = '\\\"';",
        "char null_byte = '\\0';",
        "char ascii_capital_a = '\\x41';",
        "char standard_char = 'Z';",
    ];

    for snippet in valid_snippets {
        let tokens =
            lex_source(0, snippet).expect("Lexing valid character literal snippet must succeed");
        let parse_result = parse_source(0, &tokens, snippet);
        assert!(
            parse_result.is_ok(),
            "Expected valid character literal '{snippet}' to parse cleanly, got {:?}",
            parse_result.err()
        );
    }

    // 2. Invalid multi-character literals with leading escapes
    let invalid_snippets = [
        "char invalid_newline = '\\nabc';",
        "char invalid_tab = '\\txyz';",
        "char invalid_null = '\\00';",
        "char invalid_hex = '\\x41B';",
        "char regular_multi = 'ab';",
    ];

    for snippet in invalid_snippets {
        let tokens = lex_source(0, snippet).expect("Lexing snippet must succeed");
        let parse_result = parse_source(0, &tokens, snippet);
        assert!(
            parse_result.is_err(),
            "Expected multi-character literal '{snippet}' to be rejected"
        );
        let errors = parse_result.expect_err("Parsing errors");
        assert!(
            errors.iter().any(|error| error.error_code() == "E0153"),
            "Expected E0153 (MultiCharacterLiteral) for '{snippet}', got {errors:?}"
        );
    }
}

#[test]
fn test_unicode_braced_escape_sequences_in_string_and_char_literals() {
    use solix_compilation::phases::parser::statements::declarations::Declaration;
    use solix_compilation::phases::parser::statements::expressions::{Expression, LiteralValue};

    let valid_source = r#"
        char check = '\u{2713}';
        char emoji = '\u{1F680}';
        string text = "A\u{0042}\u{2713}\u{1F680}";
    "#;
    let tokens = lex_source(0, valid_source).expect("Lexing valid unicode escapes must succeed");
    let ast = parse_source(0, &tokens, valid_source)
        .expect("Valid unicode escapes in char and string literals must parse cleanly");
    assert_eq!(ast.declarations.len(), 3);

    let extracted_values: Vec<LiteralValue> = ast
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            Declaration::Field(field) => match field.initializer.as_ref()? {
                Expression::Literal { value, .. } => Some(value.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(
        extracted_values,
        vec![
            LiteralValue::Char('\u{2713}'),
            LiteralValue::Char('\u{1F680}'),
            LiteralValue::String("AB\u{2713}\u{1F680}".to_string()),
        ]
    );

    let invalid_escapes = [
        r#"string empty_brace = "\u{}";"#,
        r#"string missing_brace = "\u1234";"#,
        r#"string too_many_digits = "\u{1234567}";"#,
        r#"string surrogate = "\u{D800}";"#,
        r#"string out_of_range = "\u{110000}";"#,
        r#"char bad_char_unicode = '\u{110000}';"#,
        r#"char invalid_hex = '\u{ZZZZ}';"#,
    ];
    for snippet in invalid_escapes {
        let bad_tokens = lex_source(0, snippet).expect("Lexing invalid escape string succeeds");
        let bad_parse = parse_source(0, &bad_tokens, snippet);
        assert!(
            bad_parse.is_err(),
            "Invalid unicode escape must fail: {snippet}"
        );
        let errors = bad_parse.expect_err("Parsing error expected");
        assert!(
            errors.iter().any(|error| error.error_code() == "E0151"),
            "Expected E0151 for '{snippet}', got {errors:?}"
        );
    }
}

#[test]
fn test_issue_248_parser_error_recovery_and_cascade_prevention() {
    let source = r#"
package pong;

public class Main {
    public static int32 main() {
        whilje(true) {
            int32 counter = 1;
        };

        return 0;
    }
}
"#;
    let tokens = lex_source(0, source).expect("Lexing source must succeed");
    let parse_result = parse_source(0, &tokens, source);
    assert!(
        parse_result.is_err(),
        "Must report parse error for misspelled control flow keyword"
    );
    let errors = parse_result.expect_err("Parsing errors");

    // Exactly one diagnostic must be emitted: E0158 (UnknownControlFlowKeyword) with suggestion "while"
    assert_eq!(
        errors.len(),
        1,
        "Expected exactly 1 diagnostic for misspelled keyword, but got {}: {errors:?}",
        errors.len()
    );
    assert_eq!(errors[0].error_code(), "E0158");
    let formatted_error = format!("{}", errors[0]);
    assert!(formatted_error.contains("whilje"));

    // Crucially: no cascading StatementDirectlyInClass (E0128) or InvalidTopLevelToken (E0195)
    assert!(
        !errors
            .iter()
            .any(|error_entry| error_entry.error_code() == "E0128"),
        "Must NOT emit StatementDirectlyInClass"
    );
    assert!(
        !errors
            .iter()
            .any(|error_entry| error_entry.error_code() == "E0195"),
        "Must NOT emit InvalidTopLevelToken"
    );
}

#[test]
fn test_class_member_synchronize_recovers_method_after_missing_semicolon() {
    use solix_compilation::phases::parser::parse_source_resilient;
    use solix_compilation::phases::parser::statements::declarations::{ClassMember, Declaration};

    let source = r#"
public class BrokenFieldClass {
    public int32 missing_semi
    public void valid_method() {
        int32 ok = 1;
    }
}
"#;
    let tokens = lex_source(0, source).expect("Lexing source must succeed");
    let (ast, errors) = parse_source_resilient(0, &tokens, source);
    assert!(
        !errors.is_empty(),
        "Missing semicolon on field must emit a parse diagnostic"
    );

    let class_decl = ast
        .declarations
        .iter()
        .find_map(|decl| match decl {
            Declaration::Class(class_decl) => Some(class_decl),
            _ => None,
        })
        .expect("BrokenFieldClass should still be recovered in AST");

    let method_names: Vec<&str> = class_decl
        .members
        .iter()
        .filter_map(|member| match member {
            ClassMember::Method(method_decl) => Some(
                &source[method_decl.name_span.start as usize..method_decl.name_span.end as usize],
            ),
            _ => None,
        })
        .collect();
    assert!(
        method_names.contains(&"valid_method"),
        "Parser::synchronize must stop at 'public' so 'valid_method' is recovered; found {method_names:?}"
    );
}
