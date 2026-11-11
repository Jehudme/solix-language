//! Standalone stdio binary entrypoint for the Solix Language Server (`solix-lsp`).

#![forbid(unsafe_code)]

#[tokio::main]
async fn main() {
    solix_lsp::run_stdio_server(None, None).await;
}
