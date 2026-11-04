//! Compiler Diagnostics Consistency Conformance Tests (TESTS.md Section 51).

#![forbid(unsafe_code)]

use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;

#[test]
fn test_case_51_1_syntax_error_reports_source_name_row_column() {
    let source = r#"class Main {
    public static int32 main() {
        int32 x = ;
        return 0;
    }
}
"#;
    let options = CompilationOptions::new().with_source("syntax_err.slx", source);
    let mut ctx = CompilationContext::new(options);
    let res = CompilationPipeline::syntax_only().execute(&mut ctx);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(!errs.is_empty());

    let err = &errs[0];
    let offset = err.source_span().expect("Source span must exist").offset();
    let line = source[..offset].lines().count();
    assert_eq!(ctx.source_names[0], "syntax_err.slx");
    assert_eq!(line, 3);
    assert!(offset > 0);
}

#[test]
fn test_case_51_2_lexical_error_reports_source_name_row_and_column() {
    let source = r#"class Main {
    public static int32 main() {
        @
        return 0;
    }
}
"#;
    let options = CompilationOptions::new().with_source("lex_err.slx", source);
    let mut ctx = CompilationContext::new(options);
    let res = CompilationPipeline::lex_only().execute(&mut ctx);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(!errs.is_empty());

    let err = &errs[0];
    let offset = err.source_span().expect("Source span must exist").offset();
    let line = source[..offset].lines().count();
    assert_eq!(ctx.source_names[0], "lex_err.slx");
    assert_eq!(line, 3);
    assert!(offset > 0);
}

#[test]
fn test_case_51_3_semantic_binding_error_reports_source_name_row_column() {
    let source = r#"class Main {
    public static int32 main() {
        int32 x = "mismatched";
        return 0;
    }
}
"#;
    let options = CompilationOptions::new().with_source("bind_err.slx", source);
    let mut ctx = CompilationContext::new(options);
    let res = CompilationPipeline::lsp().execute(&mut ctx);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(!errs.is_empty());

    let err = &errs[0];
    let offset = err.source_span().expect("Source span must exist").offset();
    let line = source[..offset].lines().count();
    assert_eq!(ctx.source_names[0], "bind_err.slx");
    assert_eq!(line, 3);
    assert!(offset > 0);
}
