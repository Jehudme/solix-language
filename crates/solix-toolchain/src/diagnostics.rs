//! High-fidelity compiler diagnostic reporting powered by `miette`.

#![forbid(unsafe_code)]

use miette::{NamedSource, Report};
use solix_compilation::CompilationDiagnostic;

/// Formats and renders a list of [`CompilationDiagnostic`]s to standard error.
pub fn render_diagnostics(
    source_names: &[String],
    sources: &[String],
    diagnostics: &[CompilationDiagnostic],
) {
    for diagnostic in diagnostics {
        let (file_name, file_src) = if let Some(source_id) = diagnostic.source_id()
            && let Some(name) = source_names.get(source_id as usize)
            && let Some(src) = sources.get(source_id as usize)
        {
            (name.as_str(), src.as_str())
        } else {
            // Fallback: prioritize non-builtin user source files
            let non_builtin = source_names
                .iter()
                .zip(sources.iter())
                .find(|(name, _)| !name.starts_with("blt@"));
            if let Some((name, src)) = non_builtin {
                (name.as_str(), src.as_str())
            } else if let (Some(name), Some(src)) = (source_names.first(), sources.first()) {
                (name.as_str(), src.as_str())
            } else {
                ("", "")
            }
        };

        if !file_name.is_empty() {
            let report = Report::new(diagnostic.clone())
                .with_source_code(NamedSource::new(file_name, file_src.to_string()));
            eprintln!("{report:?}");
        } else {
            let report = Report::new(diagnostic.clone());
            eprintln!("{report:?}");
        }
    }
}
