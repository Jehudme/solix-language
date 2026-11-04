//! Shared test harness and language fixture loaders for solix-compilation integration tests.

#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// Represents an automated Solix language test fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageFixture {
    pub file_path: PathBuf,
    pub source_content: String,
    pub expected_error_code: Option<String>,
}

impl LanguageFixture {
    /// Loads an individual fixture file from disk and parses diagnostic metadata headers.
    pub fn load_from_path(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path_reference = file_path.as_ref();
        let source_content = fs::read_to_string(path_reference)?;

        // Negative fixtures declare expected error in header: // EXPECT_ERROR: E0001
        let expected_error_code = source_content
            .lines()
            .find(|line_reference| line_reference.starts_with("// EXPECT_ERROR:"))
            .map(|line_reference| {
                line_reference
                    .replace("// EXPECT_ERROR:", "")
                    .trim()
                    .to_string()
            });

        Ok(Self {
            file_path: path_reference.to_path_buf(),
            source_content,
            expected_error_code,
        })
    }
}
