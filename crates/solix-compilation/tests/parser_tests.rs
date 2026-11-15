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
    // 1. Dot syntax in type paths MUST fail with E0201
    let invalid_dot_type_source = "solix.collections.List<int32> items;";
    let tokens_invalid = lex_source(0, invalid_dot_type_source).expect("Lexing succeeds");
    let parse_result_invalid = parse_source(0, &tokens_invalid, invalid_dot_type_source);
    assert!(parse_result_invalid.is_err(), "Dot in type annotation MUST be rejected");
    let detected_errors = parse_result_invalid.unwrap_err();
    assert!(
        detected_errors.iter().any(|error| error.error_code() == "E0201"),
        "Expected E0201 (DotNotAllowedInTypePath) when using '.' in type annotation"
    );

    // 2. Scope resolution syntax '::' in type paths MUST succeed
    let valid_colon_type_source = "solix::collections::List<int32> items;";
    let tokens_valid = lex_source(0, valid_colon_type_source).expect("Lexing succeeds");
    let parse_result_valid = parse_source(0, &tokens_valid, valid_colon_type_source);
    assert!(parse_result_valid.is_ok(), "Scope resolution '::' in type annotation MUST succeed");

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
