//! Subcommand execution for `solix fmt`.

#![forbid(unsafe_code)]

use crate::cli::FmtArgs;
use std::path::{Path, PathBuf};

const IGNORED_DIRECTORY_NAMES: &[&str] = &[
    ".git",
    "build",
    "target",
    ".cache",
    "node_modules",
];

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
        let current_working_directory = std::env::current_dir()?;
        collect_slx_files(&current_working_directory, &mut files_to_format)?;
    }

    let mut unformatted_count = 0;

    for file_path in &files_to_format {
        let file_content = std::fs::read_to_string(file_path)?;
        let formatted_content = solix_lsp::format_source(&file_content);
        if file_content != formatted_content {
            unformatted_count += 1;
            if args.check {
                println!("Would format: {}", file_path.display());
            } else {
                std::fs::write(file_path, &formatted_content)?;
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

fn collect_slx_files(
    source_directory: &Path,
    collected_files: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for directory_entry_result in std::fs::read_dir(source_directory)? {
        let directory_entry = directory_entry_result?;
        let entry_path = directory_entry.path();
        if entry_path.is_dir() {
            if let Some(directory_name) = entry_path.file_name().and_then(|name| name.to_str()) {
                if directory_name.starts_with('.')
                    || IGNORED_DIRECTORY_NAMES.contains(&directory_name)
                {
                    continue;
                }
            }
            collect_slx_files(&entry_path, collected_files)?;
        } else if entry_path.is_file()
            && entry_path
                .extension()
                .is_some_and(|file_extension| file_extension == "slx")
        {
            collected_files.push(entry_path);
        }
    }
    Ok(())
}
