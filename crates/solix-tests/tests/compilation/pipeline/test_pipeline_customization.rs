//! Conformance tests for Compiler Pipeline Modularization and Phase Customization (TESTS.md Section 58).

#![forbid(unsafe_code)]

use solix_compilation::CompilationDiagnostic;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::CompilationPipeline;
use solix_compilation::phases::phase::{CompilationPhase, PhaseDecision};
use solix_compilation::phases::standard::{BinderPhase, EmitterPhase, LexerPhase, ParserPhase};

#[derive(Debug)]
struct MockLintPhase;

impl CompilationPhase for MockLintPhase {
    fn name(&self) -> &'static str {
        "MockLintPhase"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        // Inspect symbol table produced by BinderPhase
        let artifacts = context
            .binding_artifacts
            .as_ref()
            .expect("Binder artifacts must be present");

        let has_secret = artifacts
            .symbol_table
            .lookup_global("SecretClass")
            .is_some();
        if has_secret {
            return Err(vec![CompilationDiagnostic::custom(
                "L001",
                "SecretClass is deprecated in public API",
                None,
            )]);
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct PipelineRecorderPhase {
    ran: bool,
}

impl CompilationPhase for PipelineRecorderPhase {
    fn name(&self) -> &'static str {
        "PipelineRecorderPhase"
    }

    fn run(&mut self, _context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        self.ran = true;
        Ok(())
    }
}

#[derive(Debug)]
struct FailingHaltPhase;

impl CompilationPhase for FailingHaltPhase {
    fn name(&self) -> &'static str {
        "FailingHaltPhase"
    }

    fn run(&mut self, _context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        Err(vec![CompilationDiagnostic::custom(
            "E9001",
            "Critical stage failure",
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

#[derive(Debug)]
struct NonFatalWarningPhase;

impl CompilationPhase for NonFatalWarningPhase {
    fn name(&self) -> &'static str {
        "NonFatalWarningPhase"
    }

    fn run(&mut self, _context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        Err(vec![CompilationDiagnostic::custom(
            "W1001",
            "Non-fatal warning diagnostic",
            None,
        )])
    }

    fn on_failure(
        &mut self,
        _context: &mut CompilationContext,
        _diagnostics: &[CompilationDiagnostic],
    ) -> PhaseDecision {
        PhaseDecision::Continue
    }
}

#[test]
fn test_case_58_1_custom_lint_phase_insertion_and_execution() {
    let code = r#"
        class SecretClass {
            public int32 key;
        }
    "#;
    let options = CompilationOptions::new()
        .with_source("secret.slx", code)
        .with_pipeline(
            CompilationPipeline::empty()
                .add_phase(LexerPhase::new())
                .add_phase(ParserPhase::new())
                .add_phase(BinderPhase::new())
                .add_phase(MockLintPhase)
                .add_phase(EmitterPhase::new()),
        );

    let mut context = CompilationContext::new(options);
    let result = context.execute();
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].error_code(), "L001");
    assert!(
        diags[0]
            .to_string()
            .contains("SecretClass is deprecated in public API")
    );
}

#[test]
fn test_case_58_2_pipeline_early_termination_on_halt() {
    let options = CompilationOptions::new().with_source("test.slx", "class X {}");
    let mut context = CompilationContext::new(options);

    let mut pipeline = CompilationPipeline::empty()
        .add_phase(FailingHaltPhase)
        .add_phase(PipelineRecorderPhase::default());

    let res = pipeline.execute(&mut context);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err()[0].error_code(), "E9001");
}

#[test]
fn test_case_58_3_pipeline_error_accumulation_on_continue() {
    let options = CompilationOptions::new().with_source("test.slx", "class X {}");
    let mut context = CompilationContext::new(options);

    let mut pipeline = CompilationPipeline::empty()
        .add_phase(NonFatalWarningPhase)
        .add_phase(FailingHaltPhase);

    let res = pipeline.execute(&mut context);
    assert!(res.is_err());
    let diags = res.unwrap_err();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].error_code(), "W1001");
    assert_eq!(diags[1].error_code(), "E9001");
}

#[derive(Debug, PartialEq, Eq)]
struct CompilerStatsExtension {
    total_phases_run: usize,
    class_count: usize,
}

#[derive(Debug)]
struct StatsCollectorPhase;

impl CompilationPhase for StatsCollectorPhase {
    fn name(&self) -> &'static str {
        "StatsCollectorPhase"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        let count = context.ast.iter().flatten().count();
        context.insert_extension(CompilerStatsExtension {
            total_phases_run: 3,
            class_count: count,
        });
        Ok(())
    }
}

#[test]
fn test_case_58_4_blackboard_extension_mutation_across_phases() {
    let code = r#"
        class A {}
        class B {}
    "#;
    let options = CompilationOptions::new()
        .with_source("stats.slx", code)
        .with_pipeline(
            CompilationPipeline::syntax_only()
                .add_phase(StatsCollectorPhase)
                .add_phase(BinderPhase::new()),
        );

    let mut context = CompilationContext::new(options);
    assert!(context.execute().is_ok());

    let stats = context.get_extension::<CompilerStatsExtension>().unwrap();
    assert_eq!(stats.class_count, 1);
    assert_eq!(stats.total_phases_run, 3);
}

#[test]
fn test_case_58_5_lsp_preset_pipeline_omits_bytecode_emission() {
    let code = r#"
        class Service {
            public int32 compute(int32 x) {
                return x * 2;
            }
        }
    "#;
    let options = CompilationOptions::new()
        .with_source("service.slx", code)
        .with_pipeline(CompilationPipeline::lsp());

    let mut context = CompilationContext::new(options);
    assert!(context.execute().is_ok());

    assert!(!context.tokens[0].is_empty());
    assert!(context.ast[0].is_some());
    assert!(context.binding_artifacts.is_some());
    assert!(
        context.emission_artifacts.is_none(),
        "LSP pipeline must never run EmitterPhase"
    );
}

#[test]
fn test_case_58_6_syntax_only_preset_pipeline() {
    let code = r#"
        class FastCheck {
            public void ping() {}
        }
    "#;
    let options = CompilationOptions::new()
        .with_source("fast.slx", code)
        .with_pipeline(CompilationPipeline::syntax_only());

    let mut context = CompilationContext::new(options);
    assert!(context.execute().is_ok());

    assert!(!context.tokens[0].is_empty());
    assert!(context.ast[0].is_some());
    assert!(
        context.binding_artifacts.is_none(),
        "Syntax-only pipeline must never run BinderPhase"
    );
    assert!(
        context.emission_artifacts.is_none(),
        "Syntax-only pipeline must never run EmitterPhase"
    );
}
