//! Subcommand execution for `solix fmt`.

#![forbid(unsafe_code)]

use crate::cli::FmtArgs;
use std::path::{Path, PathBuf};

/// Executes the Solix formatter over specified files or directories.
///
/// # Errors
/// Returns an error if disk I/O fails.
pub fn execute_fmt(args: FmtArgs) -> anyhow::Result<i32> {
    let mut files_to_format = Vec::new();
    for target_path in &args.files {
        if target_path.is_file() {
            files_to_format.push(target_path.clone());
        } else if target_path.is_dir() {
            collect_slx_files(target_path, &mut files_to_format)?;
        }
    }

    if files_to_format.is_empty() {
        let current_dir = std::env::current_dir()?;
        collect_slx_files(&current_dir, &mut files_to_format)?;
    }

    let mut unformatted_count = 0;

    for file_path in &files_to_format {
        let content = std::fs::read_to_string(file_path)?;
        let formatted = solix_lsp::format_source(&content);
        if content != formatted {
            unformatted_count += 1;
            if args.check {
                println!("Would format: {}", file_path.display());
            } else {
                std::fs::write(file_path, &formatted)?;
                println!("Formatted: {}", file_path.display());
            }
        }
    }

    if args.check && unformatted_count > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn collect_slx_files(dir: &Path, result: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_slx_files(&path, result)?;
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "slx") {
            result.push(path);
        }
    }
    Ok(())
}
