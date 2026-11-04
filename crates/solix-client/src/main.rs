//! Solix compiler and virtual machine CLI binary executable.

#![forbid(unsafe_code)]

use clap::Parser;
use solix_client::cli::Cli;

fn main() {
    let cli = Cli::parse();
    match solix_client::run(cli) {
        Ok(code) => {
            std::process::exit(code);
        }
        Err(err) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
    }
}
