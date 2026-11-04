//! Solix Command-Line Toolchain library interface.

#![forbid(unsafe_code)]

pub mod cli;
pub mod commands;
pub mod diagnostics;
pub mod telemetry;

pub use cli::{Cli, Commands, CompileArgs, InfoArgs, RunArgs};
pub use commands::{execute_compile, execute_info, execute_run};
pub use diagnostics::render_diagnostics;
pub use telemetry::{
    CompileMetricsReport, ProcessTelemetry, RunMetricsReport, capture_process_telemetry,
};

/// Main programmatic entry point for dispatching CLI commands.
///
/// # Errors
/// Returns an error if any subcommand execution fails.
pub fn run(cli: Cli) -> anyhow::Result<i32> {
    match cli.command {
        Commands::Compile(args) => {
            execute_compile(args)?;
            Ok(0)
        }
        Commands::Run(args) => {
            let code = execute_run(args)?;
            Ok(code)
        }
        Commands::Info(args) => {
            execute_info(args)?;
            Ok(0)
        }
    }
}
