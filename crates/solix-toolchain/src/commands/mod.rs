//! Subcommand execution modules for the Solix CLI.

#![forbid(unsafe_code)]

pub mod build;
pub mod clean;
pub mod compile;
pub mod dap;
pub mod doc;
pub mod export;
pub mod fmt;
pub mod info;
pub mod lsp;
pub mod new;
pub mod pkg;
pub mod run;
pub mod wiki;
pub mod workflow;

pub use build::{
    APPLE_DARWIN_CROSS_IMAGE, CargoBuildStrategy, cross_image_for_triple,
    dockcross_image_for_triple, execute_build, osxcross_cargo_env_vars,
    prepare_effective_cross_config, resolve_cargo_target_cache_dir, resolve_cmake_target_cache_dir,
    resolve_native_triples_for_test, select_cargo_strategy, verify_docker_daemon_active,
    write_zig_toolchain_file_for_test,
};
pub use clean::execute_clean;
pub use compile::{execute_compile, execute_compile_package, execute_compile_source};
pub use dap::execute_dap;
pub use doc::execute_doc;
pub use export::execute_export;
pub use fmt::execute_fmt;
pub use info::execute_info;
pub use lsp::execute_lsp;
pub use new::{execute_new, execute_new_native, execute_new_package};
pub use pkg::{
    execute_install, execute_install_binaries, execute_install_package, execute_list,
    execute_uninstall, execute_uninstall_binaries, execute_uninstall_package,
    install_package_artifact, resolve_bin_store_dir,
};
pub use run::{execute_run, execute_run_package, execute_run_source};
pub use wiki::execute_wiki;
pub use workflow::{execute_add, execute_check, execute_init, execute_test};
