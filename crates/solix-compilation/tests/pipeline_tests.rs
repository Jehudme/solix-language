//! Unit tests for modular compilation phases and lifecycle flow control.

#![forbid(unsafe_code)]

use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationDiagnostic;
use solix_compilation::CompilationPipeline;
use solix_compilation::phases::phase::{CompilationPhase, PhaseDecision};
use solix_compilation::phases::standard::{BinderPhase, EmitterPhase, LexerPhase, ParserPhase};

#[derive(Debug)]
struct MockRecordingPhase {
    executed: bool,
}

impl CompilationPhase for MockRecordingPhase {
    fn name(&self) -> &'static str {
        "MockRecordingPhase"
    }

    fn run(&mut self, _context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        self.executed = true;
        Ok(())
    }
}

#[derive(Debug)]
struct MockFailingPhase;

impl CompilationPhase for MockFailingPhase {
    fn name(&self) -> &'static str {
        "MockFailingPhase"
    }

    fn run(&mut self, _context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        Err(vec![CompilationDiagnostic::custom(
            "TEST01",
            "mock failure triggered",
            None,
        )])
    }

    fn on_failure(
        &mut self,
        _context: &mut CompilationContext,
        _diagnostics: &[CompilationDiagnostic],
    ) -> PhaseDecision {
        PhaseDecision::Halt
    }
}

#[test]
fn test_standard_phases_sequential_execution() {
    let code = r#"
        class Calculator {
            public static int32 add(int32 a, int32 b) {
                return a + b;
            }
        }
    "#;
    let options = CompilationOptions::new().with_source("calc.slx", code);
    let mut context = CompilationContext::new(options);

    let mut lexer = LexerPhase::new();
    assert_eq!(lexer.name(), "Lexical Analysis");
    assert!(lexer.run(&mut context).is_ok());
    assert!(!context.tokens[0].is_empty());

    let mut parser = ParserPhase::new();
    assert_eq!(parser.name(), "Syntax Parsing");
    assert!(parser.run(&mut context).is_ok());
    assert!(context.ast[0].is_some());

    let mut binder = BinderPhase::new();
    assert_eq!(binder.name(), "Semantic Analysis & Type Binding");
    assert!(binder.run(&mut context).is_ok());
    assert!(context.binding_artifacts.is_some());

    let mut emitter = EmitterPhase::new();
    assert_eq!(emitter.name(), "Bytecode Emission");
    assert!(emitter.run(&mut context).is_ok());
    assert!(context.emission_artifacts.is_some());
}

#[test]
fn test_custom_phase_lifecycle_hooks() {
    let options = CompilationOptions::new().with_source("empty.slx", "");
    let mut context = CompilationContext::new(options);

    let mut mock = MockRecordingPhase { executed: false };
    assert_eq!(mock.on_success(&mut context), PhaseDecision::Continue);
    assert!(mock.run(&mut context).is_ok());
    assert!(mock.executed);

    let mut failing = MockFailingPhase;
    let errs = failing.run(&mut context).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].error_code(), "TEST01");
    assert_eq!(failing.on_failure(&mut context, &errs), PhaseDecision::Halt);
}

#[test]
fn test_pipeline_standard_preset_and_context_execute() {
    let code = r#"
        class App {
            public static int32 main() {
                return 42;
            }
        }
    "#;
    let options = CompilationOptions::new().with_source("main.slx", code);
    let mut context = CompilationContext::new(options);

    assert!(context.execute().is_ok());
    assert!(context.emission_artifacts.is_some());
    let prog = &context.emission_artifacts.as_ref().unwrap().program;
    assert!(prog.functions.iter().any(|f| f.symbol.contains("main")));
}

#[test]
fn test_pipeline_lsp_and_syntax_only_presets() {
    let code = r#"
        class Greeter {
            public string say_hello() {
                return "hello";
            }
        }
    "#;

    // 1. Syntax-only preset
    let opt_syntax = CompilationOptions::new().with_source("greeter.slx", code);
    let mut ctx_syntax = CompilationContext::new(opt_syntax);
    let mut syntax_pipeline = CompilationPipeline::syntax_only();
    assert_eq!(syntax_pipeline.len(), 2);
    assert!(syntax_pipeline.execute(&mut ctx_syntax).is_ok());
    assert!(ctx_syntax.ast[0].is_some());
    assert!(ctx_syntax.binding_artifacts.is_none());
    assert!(ctx_syntax.emission_artifacts.is_none());

    // 2. LSP preset
    let opt_lsp = CompilationOptions::new().with_source("greeter.slx", code);
    let mut ctx_lsp = CompilationContext::new(opt_lsp);
    let mut lsp_pipeline = CompilationPipeline::lsp();
    assert_eq!(lsp_pipeline.len(), 3);
    assert!(lsp_pipeline.execute(&mut ctx_lsp).is_ok());
    assert!(ctx_lsp.ast[0].is_some());
    assert!(ctx_lsp.binding_artifacts.is_some());
    assert!(ctx_lsp.emission_artifacts.is_none()); // Does not emit bytecode!
}

#[derive(Debug, PartialEq, Eq)]
struct AstNodeCounterExtension {
    count: usize,
}

#[derive(Debug)]
struct AstCounterPhase;

impl CompilationPhase for AstCounterPhase {
    fn name(&self) -> &'static str {
        "AstCounterPhase"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        let count = context.ast.iter().flatten().count();
        context.insert_extension(AstNodeCounterExtension { count });
        Ok(())
    }
}

#[test]
fn test_context_blackboard_extensions_and_custom_phase_injection() {
    let code = r#"
        class A {}
        class B {}
    "#;
    let options = CompilationOptions::new()
        .with_source("blackboard_test.slx", code)
        .with_pipeline(
            CompilationPipeline::syntax_only()
                .add_phase(AstCounterPhase)
                .add_phase(BinderPhase::new()),
        );

    let mut context = CompilationContext::new(options);
    assert!(context.execute().is_ok());

    // Inspect custom extension stored in blackboard
    let counter = context.get_extension::<AstNodeCounterExtension>();
    assert!(counter.is_some());
    assert_eq!(counter.unwrap().count, 1);

    // Remove extension
    let removed = context.remove_extension::<AstNodeCounterExtension>();
    assert_eq!(removed, Some(AstNodeCounterExtension { count: 1 }));
    assert!(context.get_extension::<AstNodeCounterExtension>().is_none());
}

#[test]
fn test_pipeline_halts_immediately_on_first_error_phase() {
    let mut pipeline = CompilationPipeline::empty()
        .add_phase(MockFailingPhase)
        .add_phase(MockRecordingPhase { executed: false });

    let options = CompilationOptions::new().with_source("halt_test.slx", "");
    let mut context = CompilationContext::new(options);

    let result = pipeline.execute(&mut context);
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].error_code(), "TEST01");
}

#[test]
fn test_builtins_inclusion_and_compilation() {
    let code = r#"
        import blt.io.println;

        class App {
            public static void main() {
                println("Hello from builtins!");
            }
        }
    "#;
    let options = CompilationOptions::new()
        .with_source("main.slx", code)
        .with_builtins(true);

    let mut context = CompilationContext::new(options);
    assert!(context.execute().is_ok());
    assert!(context.emission_artifacts.is_some());
}

#[test]
fn test_pipeline_telemetry_recording() {
    let source_code = r#"
        class MathUtils {
            public static int32 add(int32 left_operand, int32 right_operand) {
                return left_operand + right_operand;
            }
        }
    "#;
    let options = CompilationOptions::new()
        .with_package_metadata("math_pkg", "1.0.0", "Math utilities")
        .with_source("math.slx", source_code);

    let mut context = CompilationContext::new(options);
    let mut pipeline = CompilationPipeline::standard();
    let result = pipeline.execute(&mut context);
    assert!(result.is_ok());

    let telemetry = context
        .telemetry
        .as_ref()
        .expect("telemetry should be populated");
    assert_eq!(telemetry.package_name, "math_pkg");
    assert!(telemetry.source_bytes > 0);
    assert!(telemetry.total_tokens > 0);
    assert!(telemetry.total_instructions > 0);
    assert!(telemetry.binary_size_bytes > 0);
    assert_eq!(telemetry.phases.len(), 7);
    assert_eq!(telemetry.phases[0].name, "Dependency Resolution");
    assert_eq!(telemetry.phases[1].name, "Lexical Analysis");
    assert_eq!(telemetry.phases[2].name, "Syntax Parsing");
    assert_eq!(telemetry.phases[3].name, "Semantic Analysis & Type Binding");
    assert_eq!(telemetry.phases[4].name, "AST Function & Method Inliner");
    assert_eq!(telemetry.phases[5].name, "Bytecode Emission");
    assert_eq!(
        telemetry.phases[6].name,
        "Bytecode Peephole & Superinstruction Optimizer"
    );
}
