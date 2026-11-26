//! Tests for incremental multi-source compilation and dependency tracking.

#![forbid(unsafe_code)]

use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::lexing::{lex_context, lex_source};
use solix_compilation::phases::parser::{parse_context, parse_source};

#[test]
fn test_single_source_lex_and_parse_helpers() {
    let source_code = "public class Calculator { public int32 calculate() { return 42; } }";
    let tokens =
        lex_source(0, source_code).expect("Single source lexing should succeed on valid code");
    assert!(!tokens.is_empty());

    let program_ast = parse_source(0, &tokens, source_code)
        .expect("Single source parsing should succeed on valid token stream");
    assert_eq!(program_ast.declarations.len(), 1);
}

#[test]
fn test_incremental_source_update_invalidates_only_target() {
    let options = CompilationOptions::new()
        .with_source(
            "file_a.slx",
            "public class Alpha { public int32 run() { return 1; } }",
        )
        .with_source(
            "file_b.slx",
            "public class Beta { public int32 run() { return 2; } }",
        );

    let mut context = CompilationContext::new(options);

    lex_context(&mut context).expect("Initial context lexing should succeed");
    parse_context(&mut context).expect("Initial context parsing should succeed");

    assert!(context.get_ast(0).is_some());
    assert!(context.get_ast(1).is_some());

    let initial_alpha_tokens = context.tokens[0].clone();
    let initial_beta_ast_decls = context
        .get_ast(1)
        .expect("Ast 1 must exist")
        .declarations
        .len();
    assert_eq!(initial_beta_ast_decls, 1);

    // Mutate file 1 (file_b.slx)
    context.update_source(
        1,
        "public class BetaUpdated { public int32 run() { return 999; } }",
    );

    // File 0 must remain completely untouched
    assert_eq!(context.tokens[0], initial_alpha_tokens);
    assert!(context.get_ast(0).is_some());

    // File 1 must be cleared / invalidated
    assert!(context.tokens[1].is_empty());
    assert!(context.get_ast(1).is_none());

    // Incrementally re-lex and re-parse file 1
    context
        .lex_source(1)
        .expect("Re-lexing source 1 should succeed");
    assert!(!context.tokens[1].is_empty());

    context
        .parse_source(1)
        .expect("Re-parsing source 1 should succeed");
    assert!(context.get_ast(1).is_some());

    // File 0 must still remain valid
    assert_eq!(context.tokens[0], initial_alpha_tokens);
    assert!(context.get_ast(0).is_some());
}

#[test]
fn test_incremental_compilation_error_resilience() {
    let options = CompilationOptions::new()
        .with_source(
            "valid.slx",
            "public class ValidClass { public int32 run() { return 10; } }",
        )
        .with_source(
            "target.slx",
            "public class TargetClass { public int32 run() { return 20; } }",
        );

    let mut context = CompilationContext::new(options);

    lex_context(&mut context).expect("Initial context lexing should succeed");
    parse_context(&mut context).expect("Initial context parsing should succeed");

    assert!(context.get_ast(0).is_some());
    assert!(context.get_ast(1).is_some());

    // Introduce a syntax violation in file 1
    context.update_source(1, "public class BrokenClass { public int32 broken(");

    // Re-lex target file
    context
        .lex_source(1)
        .expect("Lexing broken file produces tokens without lex error");

    // Re-parsing target file must return errors
    let parse_result = context.parse_source(1);
    assert!(parse_result.is_err());
    assert!(context.get_ast(1).is_none());

    // File 0 must remain intact despite file 1 failing
    assert!(context.get_ast(0).is_some());
    assert_eq!(context.valid_asts().count(), 1);

    // Repair file 1
    context.update_source(
        1,
        "public class RepairedClass { public int32 run() { return 30; } }",
    );
    context
        .lex_source(1)
        .expect("Lexing repaired source should succeed");
    context
        .parse_source(1)
        .expect("Parsing repaired source should succeed");

    assert!(context.get_ast(1).is_some());
    assert_eq!(context.valid_asts().count(), 2);
}

#[test]
fn test_dependency_and_transitive_dependents_graph() {
    let mut context = CompilationContext::new(CompilationOptions::new());

    // Add 4 sources: 0, 1, 2, 3
    let source_zero = context
        .add_source("base.slx", "class Base {}")
        .expect("source_zero registration");
    let source_one = context
        .add_source("middle_a.slx", "import base; class MiddleA {}")
        .expect("source_one registration");
    let source_two = context
        .add_source("leaf.slx", "import middle_a; class Leaf {}")
        .expect("source_two registration");
    let source_three = context
        .add_source("middle_b.slx", "import base; class MiddleB {}")
        .expect("source_three registration");

    assert_eq!(source_zero, 0);
    assert_eq!(source_one, 1);
    assert_eq!(source_two, 2);
    assert_eq!(source_three, 3);

    // MiddleA (1) depends on Base (0)
    context.register_dependency(1, 0);
    // MiddleB (3) depends on Base (0)
    context.register_dependency(3, 0);
    // Leaf (2) depends on MiddleA (1)
    context.register_dependency(2, 1);

    // Check immediate dependents
    let base_dependents = context.get_dependents(0);
    assert!(base_dependents.contains(&1));
    assert!(base_dependents.contains(&3));
    assert_eq!(base_dependents.len(), 2);

    let middle_a_dependents = context.get_dependents(1);
    assert_eq!(middle_a_dependents, &[2]);

    let leaf_dependents = context.get_dependents(2);
    assert!(leaf_dependents.is_empty());

    // Check transitive dependents (downstream invalidation set)
    // Changing base (0) requires rebinding MiddleA (1), MiddleB (3), and Leaf (2)
    let transitive_base_dependents = context.get_transitive_dependents(0);
    assert_eq!(transitive_base_dependents, vec![1, 2, 3]);

    // Changing MiddleA (1) requires rebinding only Leaf (2)
    let transitive_middle_dependents = context.get_transitive_dependents(1);
    assert_eq!(transitive_middle_dependents, vec![2]);

    // Clear dependencies for MiddleA (1)
    context.clear_dependencies_for(1);
    assert!(context.get_dependents(0).contains(&3));
    assert!(!context.get_dependents(0).contains(&1));

    let updated_transitive_base = context.get_transitive_dependents(0);
    assert_eq!(updated_transitive_base, vec![3]);
}
