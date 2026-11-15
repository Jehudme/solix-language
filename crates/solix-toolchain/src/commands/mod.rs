//! Subcommand execution modules for the Solix CLI.

#![forbid(unsafe_code)]

pub mod build;
pub mod compile;
pub mod dap;
pub mod fmt;
pub mod info;
pub mod lsp;
pub mod new;
pub mod pkg;
pub mod run;

pub use build::execute_build;
pub use compile::execute_compile;
pub use dap::execute_dap;
pub use fmt::execute_fmt;
pub use info::execute_info;
pub use lsp::execute_lsp;
pub use new::execute_new;
pub use pkg::{
    execute_install, execute_list, execute_reinstall, execute_sync, execute_uninstall,
    install_package_artifact,
};
pub use run::execute_run;
