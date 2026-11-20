//! Process spawning and cross-platform executable resolution.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Searches for an executable named `executable_name` across `search_path`.
///
/// On Windows, prioritizes executable extensions (`.exe`, `.cmd`, `.bat`, `.com`)
/// from `PATHEXT` to avoid accidentally executing extensionless POSIX shell scripts
/// that cause OS error 193 (`%1 is not a valid Win32 application`).
#[must_use]
pub fn find_executable_in_search_paths(
    executable_name: &str,
    search_path: &std::ffi::OsStr,
) -> Option<PathBuf> {
    let candidate_path = Path::new(executable_name);
    if candidate_path
        .parent()
        .is_some_and(|parent_dir| !parent_dir.as_os_str().is_empty())
    {
        if candidate_path.is_file() {
            #[cfg(windows)]
            {
                let extension = candidate_path
                    .extension()
                    .and_then(|ext_os| ext_os.to_str())
                    .unwrap_or("");
                if !extension.is_empty() {
                    return Some(candidate_path.to_path_buf());
                }
            }
            #[cfg(not(windows))]
            return Some(candidate_path.to_path_buf());
        }
        #[cfg(windows)]
        {
            for extension in &[".exe", ".cmd", ".bat", ".com"] {
                let with_extension = candidate_path.with_extension(&extension[1..]);
                if with_extension.is_file() {
                    return Some(with_extension);
                }
            }
        }
        return None;
    }

    #[cfg(windows)]
    let extensions: Vec<String> = if let Some(pathext_var) = std::env::var_os("PATHEXT") {
        std::env::split_paths(&pathext_var)
            .filter_map(|ext_path| {
                ext_path
                    .to_str()
                    .map(|ext_str| ext_str.to_ascii_lowercase())
            })
            .collect()
    } else {
        vec![
            ".exe".to_string(),
            ".cmd".to_string(),
            ".bat".to_string(),
            ".com".to_string(),
        ]
    };

    for path_entry in std::env::split_paths(search_path) {
        #[cfg(windows)]
        {
            // If the name already has an explicit extension (.exe, .cmd, etc.), check directly
            if candidate_path.extension().is_some() {
                let direct_candidate = path_entry.join(executable_name);
                if direct_candidate.is_file() {
                    return Some(direct_candidate);
                }
            }
            // Check Windows extensions (.exe, .cmd, .bat, .com) in order
            for extension in &extensions {
                let candidate_file = if extension.starts_with('.') {
                    path_entry.join(format!("{executable_name}{extension}"))
                } else {
                    path_entry.join(format!("{executable_name}.{extension}"))
                };
                if candidate_file.is_file() {
                    return Some(candidate_file);
                }
            }
            // Extensionless files are strictly ignored on Windows to avoid OS error 193
        }
        #[cfg(not(windows))]
        {
            let direct_candidate = path_entry.join(executable_name);
            if direct_candidate.is_file() {
                return Some(direct_candidate);
            }
        }
    }

    None
}

/// Searches for an executable named `executable_name` in the system `PATH`.
#[must_use]
pub fn find_executable_in_path(executable_name: &str) -> Option<PathBuf> {
    let path_variable = std::env::var_os("PATH").unwrap_or_default();
    find_executable_in_search_paths(executable_name, &path_variable)
}

/// Creates a `std::process::Command` prepared to execute `executable_path`.
///
/// On Windows, scripts with `.cmd` or `.bat` extensions cannot be executed directly
/// by Win32 `CreateProcessW` without wrapping in `cmd.exe /c`. This helper transparently
/// handles wrapping when necessary.
#[must_use]
pub fn create_process_command(executable_path: &Path) -> Command {
    #[cfg(windows)]
    {
        let extension = executable_path
            .extension()
            .and_then(|ext_os| ext_os.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if extension == "cmd" || extension == "bat" {
            let comspec = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
            let mut command = Command::new(comspec);
            command.arg("/c").arg(executable_path);
            return command;
        }
    }
    Command::new(executable_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_process_command_basic() {
        let dummy_path = Path::new("dummy_tool");
        let command = create_process_command(dummy_path);
        let program_name = command.get_program().to_string_lossy();
        assert!(!program_name.is_empty());
    }

    #[test]
    fn test_find_executable_in_mock_search_paths() {
        let temp_dir = tempfile::tempdir().expect("Temporary directory must be created");
        let bin_dir = temp_dir.path().join("mock_bin");
        fs::create_dir_all(&bin_dir).expect("Mock bin directory must be created");

        let test_tool_name = "mocktool";
        #[cfg(windows)]
        let file_name = format!("{test_tool_name}.exe");
        #[cfg(not(windows))]
        let file_name = test_tool_name.to_string();

        let tool_file_path = bin_dir.join(&file_name);
        fs::write(&tool_file_path, b"dummy content").expect("Mock executable must be written");

        let search_path = std::env::join_paths([&bin_dir]).expect("Search path must be joined");
        let resolved = find_executable_in_search_paths(test_tool_name, &search_path);

        assert!(resolved.is_some(), "Executable must be found in mock path");
        assert_eq!(resolved.unwrap(), tool_file_path);
    }

    #[test]
    fn test_windows_script_extension_priority_over_extensionless() {
        let temp_dir = tempfile::tempdir().expect("Temporary directory must be created");
        let bin_dir = temp_dir.path().join("vscode_bin");
        fs::create_dir_all(&bin_dir).expect("Directory creation must succeed");

        // Simulate VS Code directory layout with both extensionless script and .cmd/.exe wrappers
        let extensionless_script = bin_dir.join("code");
        let cmd_script = bin_dir.join("code.cmd");
        let exe_file = bin_dir.join("code.exe");

        fs::write(&extensionless_script, b"#!/bin/sh\nexit 0").expect("Write extensionless");
        fs::write(&cmd_script, b"@echo off\nexit /b 0").expect("Write cmd");
        fs::write(&exe_file, b"MZ dummy binary").expect("Write exe");

        let search_path = std::env::join_paths([&bin_dir]).expect("Join search path");

        #[cfg(windows)]
        {
            let resolved = find_executable_in_search_paths("code", &search_path);
            assert!(resolved.is_some());
            let resolved_path = resolved.unwrap();
            // On Windows, .exe must take precedence, and extensionless must NEVER be selected
            assert_eq!(resolved_path, exe_file);
            assert_ne!(resolved_path, extensionless_script);
        }

        #[cfg(not(windows))]
        {
            let resolved = find_executable_in_search_paths("code", &search_path);
            assert!(resolved.is_some());
            assert_eq!(resolved.unwrap(), extensionless_script);
        }
    }
}
