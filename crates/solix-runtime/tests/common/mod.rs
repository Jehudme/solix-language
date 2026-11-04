//! Shared test harness for solix-runtime virtual machine integration tests.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Fixture descriptor for runtime execution tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeExecutionFixture {
    pub binary_path: PathBuf,
    pub expected_exit_code: i32,
}

impl RuntimeExecutionFixture {
    pub fn new(binary_path: impl AsRef<Path>, expected_exit_code: i32) -> Self {
        Self {
            binary_path: binary_path.as_ref().to_path_buf(),
            expected_exit_code,
        }
    }
}
