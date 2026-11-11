//! Subcommand execution for `solix lsp`.

#![forbid(unsafe_code)]

use crate::cli::LspArgs;
use crate::solver::{resolve_package_store_dir, ClientModuleSolver};
use std::sync::Arc;

/// Runs the language server over stdio.
///
/// # Errors
/// Returns an error if the async runtime cannot be initialized or the package store cannot be resolved.
pub fn execute_lsp(args: LspArgs) -> anyhow::Result<()> {
    let store_dir = resolve_package_store_dir(args.pkg_dir.as_deref());
    let solver = Arc::new(ClientModuleSolver::with_store_dir(store_dir));
    let root_path = if args.path.as_os_str().is_empty() || args.path.to_str() == Some(".") {
        std::env::current_dir().ok()
    } else {
        Some(args.path)
    };

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        solix_lsp::run_stdio_server(root_path, Some(solver)).await;
    });

    Ok(())
}
