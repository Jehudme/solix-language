//! Subcommand execution modules for the Solix CLI.

#![forbid(unsafe_code)]

pub mod build;
pub mod compile;
pub mod info;
pub mod pkg;
pub mod run;

pub use build::execute_build;
pub use compile::execute_compile;
pub use info::execute_info;
pub use pkg::{execute_install, execute_list, execute_uninstall};
pub use run::execute_run;
