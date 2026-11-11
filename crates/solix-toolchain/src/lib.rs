//! Solix Command-Line Toolchain library interface.

#![forbid(unsafe_code)]

pub mod assets;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod diagnostics;
pub mod registry;
pub mod solver;
pub mod telemetry;

pub use cli::{
    BuildArgs, Cli, Commands, CompileArgs, FmtArgs, InfoArgs, InstallArgs, ListArgs, LspArgs,
    NewArgs, RunArgs, SyncArgs, UninstallArgs,
};
pub use commands::{
    execute_build, execute_compile, execute_fmt, execute_info, execute_install, execute_list,
    execute_lsp, execute_new, execute_run, execute_sync, execute_uninstall,
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
        Commands::Sync(args) => {
            execute_sync(args)?;
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
        Commands::Lsp(args) => {
            execute_lsp(args)?;
            Ok(0)
        }
        Commands::Fmt(args) => {
            let code = execute_fmt(args)?;
            Ok(code)
        }
    }
}
