//! Standard built-in compilation phases implementing [`CompilationPhase`].

#![forbid(unsafe_code)]

use super::phase::CompilationPhase;
use crate::CompilationContext::CompilationContext;
use solix_common::CompilationDiagnostic;

/// Lexical analysis phase tokenizing all registered sources in the context.
#[derive(Debug, Default, Clone)]
pub struct LexerPhase;

impl LexerPhase {
    /// Constructs a new [`LexerPhase`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for LexerPhase {
    fn name(&self) -> &'static str {
        "Lexical Analysis"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        let sources_count = context.sources.len();
        if context.tokens.len() < sources_count {
            context.tokens.resize_with(sources_count, Vec::new);
        }

        let mut accumulated_diagnostics = Vec::new();

        for source_index in 0..sources_count {
            let source_id = source_index as u16;
            let source_text = &context.sources[source_index];

            match crate::phases::lexing::lex_source(source_id, source_text) {
                Ok(tokens) => {
                    context.tokens[source_index] = tokens;
                }
                Err(errors) => {
                    for err in errors {
                        accumulated_diagnostics
                            .push(CompilationDiagnostic::from(err).with_source_id(source_id));
                    }
                }
            }
        }

        if !accumulated_diagnostics.is_empty() {
            Err(accumulated_diagnostics)
        } else {
            Ok(())
        }
    }
}

/// Syntax parsing phase parsing token streams into ASTs for all registered sources.
#[derive(Debug, Default, Clone)]
pub struct ParserPhase;

impl ParserPhase {
    /// Constructs a new [`ParserPhase`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for ParserPhase {
    fn name(&self) -> &'static str {
        "Syntax Parsing"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        let sources_count = context.sources.len();
        if context.ast.len() < sources_count {
            context.ast.resize_with(sources_count, || None);
        }

        let mut accumulated_diagnostics = Vec::new();

        for source_index in 0..sources_count {
            let source_id = source_index as u16;
            let source_text = &context.sources[source_index];
            let tokens = context
                .tokens
                .get(source_index)
                .map(|tok_vec| tok_vec.as_slice())
                .unwrap_or(&[]);

            match crate::phases::parser::parse_source(source_id, tokens, source_text) {
                Ok(ast) => {
                    context.ast[source_index] = Some(ast);
                }
                Err(errors) => {
                    context.ast[source_index] = None;
                    for err in errors {
                        accumulated_diagnostics
                            .push(CompilationDiagnostic::from(err).with_source_id(source_id));
                    }
                }
            }
        }

        if !accumulated_diagnostics.is_empty() {
            Err(accumulated_diagnostics)
        } else {
            Ok(())
        }
    }
}

/// Semantic binding phase resolving symbols, types, generics, and class layouts.
#[derive(Debug, Default, Clone)]
pub struct BinderPhase;

impl BinderPhase {
    /// Constructs a new [`BinderPhase`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for BinderPhase {
    fn name(&self) -> &'static str {
        "Semantic Analysis & Type Binding"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        match crate::phases::binder::bind_context(context) {
            Ok(_) => Ok(()),
            Err(errors) => Err(errors
                .into_iter()
                .map(CompilationDiagnostic::from)
                .collect()),
        }
    }
}

/// Bytecode emission phase allocating registers and generating executable binary container.
#[derive(Debug, Default, Clone)]
pub struct EmitterPhase;

impl EmitterPhase {
    /// Constructs a new [`EmitterPhase`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for EmitterPhase {
    fn name(&self) -> &'static str {
        "Bytecode Emission"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        match crate::phases::emitter::pipeline::emit_context(context) {
            Ok(artifacts) => {
                context.emission_artifacts = Some(artifacts);
                Ok(())
            }
            Err(err) => Err(vec![CompilationDiagnostic::from(err)]),
        }
    }
}
