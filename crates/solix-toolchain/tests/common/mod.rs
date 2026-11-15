//! Shared test utilities for hermetic solix-toolchain CLI integration tests.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// Ephemeral test workspace ensuring zero disk pollution during CLI testing.
pub struct HermeticCliWorkspace {
    temporary_directory: TempDir,
}

impl HermeticCliWorkspace {
    /// Creates a brand-new unique temporary directory.
    pub fn create() -> std::io::Result<Self> {
        let temporary_directory = tempdir()?;
        Ok(Self {
            temporary_directory,
        })
    }

    /// Returns the root path of the isolated temporary directory.
    pub fn root_path(&self) -> PathBuf {
        self.temporary_directory.path().to_path_buf()
    }

    /// Resolves an absolute path within the hermetic workspace.
    pub fn file_path(&self, rel_path: &str) -> PathBuf {
        self.temporary_directory.path().join(rel_path)
    }

    /// Writes content to a file at the given relative path within the workspace.
    pub fn write_file(&self, rel_path: &str, content: &str) -> std::io::Result<PathBuf> {
        let destination_path = self.file_path(rel_path);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&destination_path, content)?;
        Ok(destination_path)
    }

    /// Reads content from a file at the given relative path within the workspace.
    #[allow(dead_code)]
    pub fn read_file(&self, rel_path: &str) -> std::io::Result<String> {
        fs::read_to_string(self.file_path(rel_path))
    }
}
