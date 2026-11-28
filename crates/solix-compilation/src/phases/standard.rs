//! Standard built-in compilation phases implementing [`CompilationPhase`].

#![forbid(unsafe_code)]

use super::phase::CompilationPhase;
use crate::CompilationContext::CompilationContext;
use crate::CompilationDiagnostic;
use rayon::prelude::*;

pub use super::dependency::DependencyResolutionPhase;
pub use super::optimizer::{AstInlinerPhase, BytecodeOptimizerPhase};

/// Minimum pending source file count before engaging `rayon` parallelism in lexing/parsing.
const PARALLEL_FRONTEND_THRESHOLD: usize = 4;

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

        let mut pending_indices = Vec::new();
        for source_index in 0..sources_count {
            if u16::try_from(source_index).is_err() {
                accumulated_diagnostics.push(CompilationDiagnostic::from(
                    crate::LexingError::source_limit_exceeded(),
                ));
                return Err(accumulated_diagnostics);
            }
            if context
                .ast
                .get(source_index)
                .and_then(|ast_option| ast_option.as_ref())
                .is_none()
            {
                pending_indices.push(source_index);
            }
        }

        if pending_indices.len() >= PARALLEL_FRONTEND_THRESHOLD {
            let sources = &context.sources;
            let mut results: Vec<_> = pending_indices
                .par_iter()
                .map(|&source_index| {
                    let source_id = source_index as u16;
                    (
                        source_index,
                        crate::phases::lexing::lex_source(source_id, &sources[source_index]),
                    )
                })
                .collect();
            results.sort_by_key(|(source_index, _)| *source_index);
            for (source_index, outcome) in results {
                let source_id = source_index as u16;
                match outcome {
                    Ok(tokens) => {
                        context.tokens[source_index] = tokens;
                    }
                    Err(errors) => {
                        for error in errors {
                            accumulated_diagnostics
                                .push(CompilationDiagnostic::from(error).with_source_id(source_id));
                        }
                    }
                }
            }
        } else {
            for source_index in pending_indices {
                let source_id = source_index as u16;
                match crate::phases::lexing::lex_source(source_id, &context.sources[source_index]) {
                    Ok(tokens) => {
                        context.tokens[source_index] = tokens;
                    }
                    Err(errors) => {
                        for error in errors {
                            accumulated_diagnostics
                                .push(CompilationDiagnostic::from(error).with_source_id(source_id));
                        }
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

        let mut pending_indices = Vec::new();
        for source_index in 0..sources_count {
            if u16::try_from(source_index).is_err() {
                accumulated_diagnostics.push(CompilationDiagnostic::from(
                    crate::ParsingError::SourceLimitExceeded {
                        span: miette::SourceSpan::from((0usize, 0usize)),
                    },
                ));
                return Err(accumulated_diagnostics);
            }
            if context
                .ast
                .get(source_index)
                .and_then(|ast_option| ast_option.as_ref())
                .is_none()
            {
                pending_indices.push(source_index);
            }
        }

        if pending_indices.len() >= PARALLEL_FRONTEND_THRESHOLD {
            let sources = &context.sources;
            let tokens_slice = &context.tokens;
            let mut results: Vec<_> = pending_indices
                .par_iter()
                .map(|&source_index| {
                    let source_id = source_index as u16;
                    let tokens = tokens_slice
                        .get(source_index)
                        .map(|tok_vec| tok_vec.as_slice())
                        .unwrap_or(&[]);
                    (
                        source_index,
                        crate::phases::parser::parse_source(
                            source_id,
                            tokens,
                            &sources[source_index],
                        ),
                    )
                })
                .collect();
            results.sort_by_key(|(source_index, _)| *source_index);
            for (source_index, outcome) in results {
                let source_id = source_index as u16;
                match outcome {
                    Ok(ast) => {
                        context.ast[source_index] = Some(ast);
                    }
                    Err(errors) => {
                        context.ast[source_index] = None;
                        for error in errors {
                            accumulated_diagnostics
                                .push(CompilationDiagnostic::from(error).with_source_id(source_id));
                        }
                    }
                }
            }
        } else {
            for source_index in pending_indices {
                let source_id = source_index as u16;
                let tokens = context
                    .tokens
                    .get(source_index)
                    .map(|tok_vec| tok_vec.as_slice())
                    .unwrap_or(&[]);

                match crate::phases::parser::parse_source(
                    source_id,
                    tokens,
                    &context.sources[source_index],
                ) {
                    Ok(ast) => {
                        context.ast[source_index] = Some(ast);
                    }
                    Err(errors) => {
                        context.ast[source_index] = None;
                        for error in errors {
                            accumulated_diagnostics
                                .push(CompilationDiagnostic::from(error).with_source_id(source_id));
                        }
                    }
                }
            }
        }

        let mut resolved_packages = std::collections::HashSet::new();
        let mut ast_cursor = 0;
        while ast_cursor < context.ast.len() {
            let mut new_extern_imports = Vec::new();
            if let Some(ast) = &context.ast[ast_cursor] {
                for import_statement in &ast.imports {
                    if import_statement.is_extern {
                        let package_name =
                            if let Some(first_segment) = import_statement.path.first() {
                                context.get_lexeme(*first_segment).to_string()
                            } else {
                                String::new()
                            };
                        if !package_name.is_empty()
                            && resolved_packages.insert(package_name.clone())
                        {
                            new_extern_imports.push((package_name, import_statement.span));
                        }
                    }
                }
            }
            ast_cursor += 1;

            for (package_name, import_span) in new_extern_imports {
                if let Err(resolution_errors) = crate::phases::dependency::resolve_extern_import(
                    &package_name,
                    import_span,
                    context,
                ) {
                    accumulated_diagnostics.extend(resolution_errors);
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
            Err(_) => {
                let diagnostics = context
                    .binding_errors
                    .iter()
                    .map(|(source_id, error)| {
                        CompilationDiagnostic::from(error.clone()).with_source_id(*source_id)
                    })
                    .collect();
                Err(diagnostics)
            }
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
