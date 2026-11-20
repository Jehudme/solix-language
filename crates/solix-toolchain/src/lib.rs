//! Solix Command-Line Toolchain library interface.

#![forbid(unsafe_code)]

pub mod assets;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod diagnostics;
pub mod process;
pub mod registry;
pub mod solver;
pub mod telemetry;

pub use cli::{
    BuildArgs, CleanArgs, Cli, Commands, CompileArgs, CompilePackageArgs, CompileSourceArgs,
    CompileSubcommand, DapArgs, DocArgs, ExportArgs, ExportBinariesArgs, ExportPackageArgs,
    ExportSubcommand, FmtArgs, InfoArgs, InstallArgs, InstallBinariesArgs, InstallPackageArgs,
    InstallSubcommand, ListArgs, LspArgs, NewArgs, NewNativeArgs, NewPackageArgs, NewSubcommand,
    RunArgs, RunPackageArgs, RunSourceArgs, RunSubcommand, UninstallArgs, UninstallBinariesArgs,
    UninstallPackageArgs, UninstallSubcommand, WikiArgs,
};
pub use commands::{
    execute_build, execute_clean, execute_compile, execute_compile_package, execute_compile_source,
    execute_dap, execute_doc, execute_export, execute_fmt, execute_info, execute_install,
    execute_install_binaries, execute_install_package, execute_list, execute_lsp, execute_new,
    execute_new_native, execute_new_package, execute_run, execute_run_package, execute_run_source,
    execute_uninstall, execute_uninstall_binaries, execute_uninstall_package, execute_wiki,
    install_package_artifact, resolve_bin_store_dir,
};
pub use diagnostics::render_diagnostics;
pub use process::{
    create_process_command, find_executable_in_path, find_executable_in_search_paths,
};
pub use solver::{ClientModuleSolver, resolve_native_cache_dir, resolve_package_store_dir};
pub use telemetry::{
    CompileMetricsReport, ProcessTelemetry, RunMetricsReport, capture_process_telemetry,
};

/// Default Git tag stamped into CMake `FetchContent` templates for `solix::ffi`.
///
/// Resolves to `dev` on development builds and `v<CARGO_PKG_VERSION>` on official
/// release builds (`SOLIX_RELEASE_CHANNEL=release`).
pub const DEFAULT_FFI_GIT_TAG: &str = env!("SOLIX_FFI_GIT_TAG");

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
        Commands::Export(args) => {
            execute_export(args)?;
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
        Commands::Dap(args) => {
            execute_dap(args)?;
            Ok(0)
        }
        Commands::Fmt(args) => {
            let code = execute_fmt(args)?;
            Ok(code)
        }
        Commands::Doc(args) => {
            execute_doc(args)?;
            Ok(0)
        }
        Commands::Wiki(args) => {
            execute_wiki(args)?;
            Ok(0)
        }
        Commands::Clean(args) => {
            execute_clean(args)?;
            Ok(0)
        }
    }
}
