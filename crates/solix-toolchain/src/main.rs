//! Solix compiler and virtual machine CLI binary executable.

#![forbid(unsafe_code)]

use clap::Parser;
use solix_toolchain::cli::Cli;

fn main() {
    let cli = Cli::parse();
    match solix_toolchain::run(cli) {
        Ok(code) => {
            std::process::exit(code);
        }
        Err(err) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
    }
}
