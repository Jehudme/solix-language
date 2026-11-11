//! Solix Language Server Protocol implementation library.

#![forbid(unsafe_code)]

pub mod code_actions;
pub mod code_lens;
pub mod completion;
pub mod diagnostics;
pub mod folding;
pub mod formatter;
pub mod hierarchy;
pub mod hover;
pub mod inlay_hints;
pub mod navigation;
pub mod rename;
pub mod selection_range;
pub mod semantic_tokens;
pub mod server;
pub mod signature_help;
pub mod symbols;
pub mod workspace;

pub use diagnostics::{
    binding_error_to_diagnostic, byte_offset_to_position, lexing_error_to_diagnostic,
    parsing_error_to_diagnostic, position_to_byte_offset, span_to_range,
};
pub use formatter::format_source;
pub use server::SolixLanguageServer;
pub use workspace::{AnalysisSnapshot, BoundSemanticSnapshot, VersionedDocument, WorkspaceManager};

use solix_common::solver::ModuleSolver;
use std::path::PathBuf;
use std::sync::Arc;
use tower_lsp::{ClientSocket, LspService, Server};

/// Builds an in-memory LSP service pair suitable for zero-process integration testing.
#[must_use]
pub fn build_lsp_service(
    workspace_root: Option<PathBuf>,
    solver: Option<Arc<dyn ModuleSolver>>,
) -> (LspService<SolixLanguageServer>, ClientSocket) {
    let workspace = Arc::new(WorkspaceManager::new(workspace_root, solver));
    LspService::new(|client| SolixLanguageServer::new(client, workspace))
}

/// Runs the language server over standard input and output streams.
pub async fn run_stdio_server(
    workspace_root: Option<PathBuf>,
    solver: Option<Arc<dyn ModuleSolver>>,
) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = build_lsp_service(workspace_root, solver);
    Server::new(stdin, stdout, socket).serve(service).await;
}
