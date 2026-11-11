//! Subcommand execution for `solix dap`.

#![forbid(unsafe_code)]

use crate::cli::DapArgs;

/// Runs the debug adapter protocol service over stdio or a TCP port.
pub fn execute_dap(args: DapArgs) -> anyhow::Result<()> {
    let root_path = if args.path.as_os_str().is_empty() || args.path.to_str() == Some(".") {
        std::env::current_dir().ok()
    } else {
        Some(args.path)
    };

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        solix_dap::run_dap_server(args.port, root_path).await
    })
}
