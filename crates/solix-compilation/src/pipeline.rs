//! Sequential compilation pipeline executor managing phase lifecycles and flow control.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::phases::phase::{CompilationPhase, PhaseDecision};
use crate::phases::standard::{BinderPhase, EmitterPhase, LexerPhase, ParserPhase};
use solix_common::CompilationDiagnostic;

/// Sequential pipeline holding an ordered sequence of compilation phases.
#[derive(Debug)]
pub struct CompilationPipeline {
    phases: Vec<Box<dyn CompilationPhase>>,
}

impl Default for CompilationPipeline {
    fn default() -> Self {
        Self::standard()
    }
}

impl CompilationPipeline {
    /// Creates an empty compilation pipeline without any phases.
    #[must_use]
    pub fn empty() -> Self {
        Self { phases: Vec::new() }
    }

    /// Appends a new phase to the pipeline sequence.
    #[must_use]
    pub fn add_phase<P: CompilationPhase + 'static>(mut self, phase: P) -> Self {
        self.phases.push(Box::new(phase));
        self
    }

    /// Appends a boxed dynamic phase to the pipeline sequence.
    pub fn push_phase(&mut self, phase: Box<dyn CompilationPhase>) {
        self.phases.push(phase);
    }

    /// Returns the number of registered phases in the pipeline.
    #[must_use]
    pub fn len(&self) -> usize {
        self.phases.len()
    }

    /// Returns whether the pipeline has no registered phases.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.phases.is_empty()
    }

    /// Pre-configured full compilation pipeline:
    /// Lexical Analysis -> Syntax Parsing -> Semantic Binding -> Bytecode Emission.
    #[must_use]
    pub fn standard() -> Self {
        Self::empty()
            .add_phase(LexerPhase::new())
            .add_phase(ParserPhase::new())
            .add_phase(BinderPhase::new())
            .add_phase(EmitterPhase::new())
    }

    /// Pre-configured language server (LSP) pipeline:
    /// Lexical Analysis -> Syntax Parsing -> Semantic Binding (omits bytecode emission).
    #[must_use]
    pub fn lsp() -> Self {
        Self::empty()
            .add_phase(LexerPhase::new())
            .add_phase(ParserPhase::new())
            .add_phase(BinderPhase::new())
    }

    /// Pre-configured syntax-only pipeline:
    /// Lexical Analysis -> Syntax Parsing.
    #[must_use]
    pub fn syntax_only() -> Self {
        Self::empty()
            .add_phase(LexerPhase::new())
            .add_phase(ParserPhase::new())
    }

    /// Pre-configured lexer-only pipeline:
    /// Lexical Analysis.
    #[must_use]
    pub fn lex_only() -> Self {
        Self::empty().add_phase(LexerPhase::new())
    }

    /// Executes all registered phases in sequence against the compilation context.
    ///
    /// At each phase:
    /// - If the phase succeeds (`Ok`), its `on_success` hook is invoked.
    ///   If `on_success` returns [`PhaseDecision::Halt`], execution stops early with `Ok(())`.
    /// - If the phase fails (`Err`), its `on_failure` hook is invoked.
    ///   If `on_failure` returns [`PhaseDecision::Halt`], execution halts immediately and
    ///   returns the accumulated diagnostics. If it returns [`PhaseDecision::Continue`],
    ///   diagnostics are accumulated and subsequent phases are attempted.
    ///
    /// # Errors
    /// Returns the accumulated list of [`CompilationDiagnostic`]s if any phase fails and halts,
    /// or if errors were accumulated during pipeline execution.
    pub fn execute(
        &mut self,
        context: &mut CompilationContext,
    ) -> Result<(), Vec<CompilationDiagnostic>> {
        let mut accumulated_diagnostics = Vec::new();

        for phase in &mut self.phases {
            match phase.run(context) {
                Ok(()) => {
                    let decision = phase.on_success(context);
                    if decision == PhaseDecision::Halt {
                        break;
                    }
                }
                Err(diagnostics) => {
                    let decision = phase.on_failure(context, &diagnostics);
                    accumulated_diagnostics.extend(diagnostics);

                    if decision == PhaseDecision::Halt {
                        return Err(accumulated_diagnostics);
                    }
                }
            }
        }

        if accumulated_diagnostics.is_empty() {
            Ok(())
        } else {
            Err(accumulated_diagnostics)
        }
    }
}
