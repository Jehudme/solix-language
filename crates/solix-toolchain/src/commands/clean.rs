//! Clean command implementation (`solix clean`).

#![forbid(unsafe_code)]

use crate::cli::CleanArgs;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Computes the total byte size and file count in a directory recursively.
fn count_files_and_bytes(directory: &Path) -> (usize, u64) {
    let mut file_count: usize = 0;
    let mut total_bytes: u64 = 0;

    if let Ok(entries) = fs::read_dir(directory) {
        for entry_result in entries.flatten() {
            let entry_path = entry_result.path();
            if entry_path.is_dir() {
                let (sub_count, sub_bytes) = count_files_and_bytes(&entry_path);
                file_count += sub_count;
                total_bytes += sub_bytes;
            } else if entry_path.is_file() {
                file_count += 1;
                total_bytes += entry_result.metadata().map_or(0, |metadata| metadata.len());
            }
        }
    }

    (file_count, total_bytes)
}

/// Formats a byte size into human-readable representation.
pub(crate) fn format_human_size(size_bytes: u64) -> String {
    if size_bytes < 1024 {
        format!("{size_bytes} B")
    } else if size_bytes < 1024 * 1024 {
        format!("{:.1} KB", size_bytes as f64 / 1024.0)
    } else if size_bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Executes the `solix clean` subcommand.
///
/// # Errors
/// Returns an error if removing directories or reading metadata fails.
pub fn execute_clean(args: CleanArgs) -> Result<()> {
    let project_root = args.path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve project path at '{}'",
            args.path.display()
        )
    })?;

    let build_directory = project_root.join("build");
    if !build_directory.exists() {
        println!(
            "No build directory found at '{}'. Nothing to clean.",
            build_directory.display()
        );
        return Ok(());
    }

    let target_directory = if args.cache_only {
        build_directory.join(".cache")
    } else {
        build_directory
    };

    if !target_directory.exists() {
        println!(
            "Directory '{}' does not exist. Nothing to clean.",
            target_directory.display()
        );
        return Ok(());
    }

    let (file_count, total_bytes) = count_files_and_bytes(&target_directory);
    fs::remove_dir_all(&target_directory).with_context(|| {
        format!(
            "Failed to remove build directory at '{}'",
            target_directory.display()
        )
    })?;

    let human_size = format_human_size(total_bytes);
    println!(
        "Cleaned '{}': Reclaimed {human_size} across {file_count} file(s).",
        target_directory.display()
    );

    Ok(())
}
