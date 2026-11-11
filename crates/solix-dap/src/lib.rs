//! Solix Debug Adapter Protocol (DAP) implementation library.

#![forbid(unsafe_code)]
#![allow(clippy::collapsible_if, clippy::collapsible_match)]

pub mod controller;
pub mod evaluator;
pub mod server;
pub mod state;

use std::path::PathBuf;

pub use controller::DapController;
pub use server::run_dap_server_blocking;

/// Starts the Solix Debug Adapter Protocol server over stdio or a TCP port.
///
/// # Errors
/// Returns an error if server execution fails.
pub async fn run_dap_server(
    port: Option<u16>,
    project_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        server::run_dap_server_blocking(port, project_path)
    })
    .await?
}
