//! Solix Command-Line Toolchain library interface.

#![forbid(unsafe_code)]

pub mod assets;
pub mod cli;
pub mod commands;
pub mod diagnostics;
pub mod solver;
pub mod telemetry;

pub use cli::{
    BuildArgs, Cli, Commands, CompileArgs, InfoArgs, InstallArgs, ListArgs, NewArgs, RunArgs,
    UninstallArgs,
};
pub use commands::{
    execute_build, execute_compile, execute_info, execute_install, execute_list, execute_new,
    execute_run, execute_uninstall,
};
pub use diagnostics::render_diagnostics;
pub use solver::{ClientModuleSolver, resolve_native_cache_dir, resolve_package_store_dir};
pub use telemetry::{
    CompileMetricsReport, ProcessTelemetry, RunMetricsReport, capture_process_telemetry,
};

/// Main programmatic entry point for dispatching CLI commands.
///
/// # Errors
/// Returns an error if any subcommand execution fails.
pub fn run(cli: Cli) -> anyhow::Result<i32> {
    match cli.command {
        Commands::New(args) => {
            execute_new(args)?;
            Ok(0)
        }
        Commands::Compile(args) => {
            execute_compile(args)?;
            Ok(0)
        }
        Commands::Build(args) => {
            execute_build(args)?;
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
        Commands::Install(args) => {
            execute_install(args)?;
            Ok(0)
        }
        Commands::Uninstall(args) => {
            execute_uninstall(args)?;
            Ok(0)
        }
        Commands::List(args) => {
            execute_list(args)?;
            Ok(0)
        }
    }
}
