//! Compilation phase trait and lifecycle flow control models.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use solix_common::CompilationDiagnostic;
use std::fmt::Debug;

/// Flow control decision returned by lifecycle hooks to guide pipeline execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseDecision {
    /// Proceed to execute the next phase in the pipeline sequence.
    Continue,
    /// Halt compilation immediately and return accumulated diagnostics to caller.
    Halt,
}

/// Core trait representing an isolated, modular stage of the compilation lifecycle.
pub trait CompilationPhase: Send + Sync + Debug {
    /// Human-readable display name of the phase (e.g. "Lexical Analysis", "Syntax Parsing").
    fn name(&self) -> &'static str;

    /// Executes this compilation phase against the mutable shared context.
    ///
    /// # Errors
    /// Returns a list of [`CompilationDiagnostic`]s if errors or unrecoverable issues occur during execution.
    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>>;

    /// Lifecycle hook triggered after `run` completes successfully.
    /// Returns [`PhaseDecision::Continue`] by default to proceed to the next phase.
    fn on_success(&mut self, _context: &mut CompilationContext) -> PhaseDecision {
        PhaseDecision::Continue
    }

    /// Lifecycle hook triggered when `run` returns diagnostics.
    /// Returns [`PhaseDecision::Halt`] by default to stop pipeline execution.
    fn on_failure(
        &mut self,
        _context: &mut CompilationContext,
        _diagnostics: &[CompilationDiagnostic],
    ) -> PhaseDecision {
        PhaseDecision::Halt
    }
}
