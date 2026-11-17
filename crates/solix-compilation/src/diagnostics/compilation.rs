//! Unified compiler diagnostic enum wrapping all stage-specific diagnostics.

#![forbid(unsafe_code)]

use crate::diagnostics::{BindingError, LexingError, ParsingError};
use miette::{Diagnostic, SourceSpan};
use solix_common::diagnostics::EmissionError;
use solix_common::span::Span;

/// Unified compiler diagnostic representing any error or warning across the compilation lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Diagnostic)]
pub enum CompilationDiagnostic {
    /// Lexical tokenization error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lex(#[from] LexingError),

    /// Syntax parsing error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] ParsingError),

    /// Semantic analysis or type-binding error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Bind(#[from] BindingError),

    /// Bytecode emission or container serialization error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Emit(#[from] EmissionError),

    /// Custom diagnostic emitted by a user or plugin compilation phase.
    #[error("Custom diagnostic [{code}]: {message}")]
    #[diagnostic(code(solix::custom))]
    Custom {
        /// Standardized error code identifier.
        code: String,
        /// Descriptive diagnostic error message.
        message: String,
        /// Source code span identifying the site of the issue.
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// Diagnostic associated with a specific origin source file.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Located(#[from] LocatedDiagnostic),
}

/// Diagnostic wrapper binding a diagnostic to a specific source file index.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{diagnostic}")]
pub struct LocatedDiagnostic {
    /// Identifier indexing the originating source file in the compilation context.
    pub source_id: u16,
    /// The underlying compiler diagnostic.
    pub diagnostic: Box<CompilationDiagnostic>,
}

impl Diagnostic for LocatedDiagnostic {
    fn code<'diag>(&'diag self) -> Option<Box<dyn std::fmt::Display + 'diag>> {
        self.diagnostic.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.diagnostic.severity()
    }

    fn help<'diag>(&'diag self) -> Option<Box<dyn std::fmt::Display + 'diag>> {
        self.diagnostic.help()
    }

    fn url<'diag>(&'diag self) -> Option<Box<dyn std::fmt::Display + 'diag>> {
        self.diagnostic.url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.diagnostic.source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.diagnostic.labels()
    }

    fn related<'diag>(
        &'diag self,
    ) -> Option<Box<dyn Iterator<Item = &'diag dyn Diagnostic> + 'diag>> {
        self.diagnostic.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.diagnostic.diagnostic_source()
    }
}

impl LocatedDiagnostic {
    /// Wraps a diagnostic with an explicit source ID.
    #[must_use]
    pub fn new(source_id: u16, diagnostic: CompilationDiagnostic) -> Self {
        Self {
            source_id,
            diagnostic: Box::new(diagnostic),
        }
    }
}

impl CompilationDiagnostic {
    /// Attaches an originating source file ID to this diagnostic.
    #[must_use]
    pub fn with_source_id(self, source_id: u16) -> Self {
        Self::Located(LocatedDiagnostic::new(source_id, self))
    }

    /// Returns the originating source file ID if known.
    #[must_use]
    pub fn source_id(&self) -> Option<u16> {
        match self {
            Self::Located(loc) => Some(loc.source_id),
            Self::Lex(lex_err) => Some(lex_err.token().span.id),
            _ => None,
        }
    }
    /// Constructs a custom diagnostic with a specific error code and message.
    #[must_use]
    pub fn custom(code: impl Into<String>, message: impl Into<String>, span: Option<Span>) -> Self {
        Self::Custom {
            code: code.into(),
            message: message.into(),
            span: span.map(|item| item.to_source_span()),
        }
    }

    /// Returns the standardized compiler error code string (e.g. `"E0000"`, `"E0101"`, `"E0203"`, `"E0301"`).
    #[must_use]
    pub fn error_code(&self) -> &str {
        match self {
            Self::Lex(lex_err) => lex_err.error_code(),
            Self::Parse(parse_err) => parse_err.error_code(),
            Self::Bind(bind_err) => bind_err.error_code(),
            Self::Emit(emit_err) => emit_err.error_code(),
            Self::Custom { code, .. } => code.as_str(),
            Self::Located(loc) => loc.diagnostic.error_code(),
        }
    }

    /// Returns the optional [`SourceSpan`] for source coordinate mapping.
    #[must_use]
    pub fn source_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Lex(lex_err) => Some(lex_err.source_span()),
            Self::Parse(parse_err) => Some(parse_err.source_span()),
            Self::Bind(bind_err) => Some(bind_err.source_span()),
            Self::Emit(_) => None,
            Self::Custom { span, .. } => *span,
            Self::Located(loc) => loc.diagnostic.source_span(),
        }
    }
}
