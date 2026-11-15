//! Self-contained asset management and runtime materialization for `solix-toolchain`.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Compile-time embedded copy of `solix_scaffold.py`.
pub const EMBEDDED_SOLIX_SCAFFOLD_SCRIPT: &str =
    include_str!("../assets/scripts/solix_scaffold.py");

/// Compile-time embedded copy of `solix_build.py`.
pub const EMBEDDED_SOLIX_BUILD_SCRIPT: &str = include_str!("../assets/scripts/solix_build.py");

/// Backward-compatibility alias for `EMBEDDED_SOLIX_SCAFFOLD_SCRIPT`.
pub const EMBEDDED_INIT_PROJECT_SCRIPT: &str = EMBEDDED_SOLIX_SCAFFOLD_SCRIPT;

/// Backward-compatibility alias for `EMBEDDED_SOLIX_BUILD_SCRIPT`.
pub const EMBEDDED_SOLIX_TOOLBOX_SCRIPT: &str = EMBEDDED_SOLIX_BUILD_SCRIPT;

/// Resolves the filesystem path to `solix_scaffold.py`.
///
/// Priority:
/// 1. `SOLIX_SCAFFOLD_PY` or legacy `SOLIX_INIT_PROJECT_PY` environment variable override.
/// 2. Sibling `<exe_dir>/assets/scripts/solix_scaffold.py` (staged by `build.rs`).
/// 3. Materialized fallback in `$SOLIX_HOME/cache/scripts/` or system temp dir.
///
/// # Errors
/// Returns an error if filesystem operations during script materialization fail.
pub fn resolve_scaffold_script_path() -> Result<PathBuf> {
    if let Ok(custom_script) =
        std::env::var("SOLIX_SCAFFOLD_PY").or_else(|_| std::env::var("SOLIX_INIT_PROJECT_PY"))
    {
        let candidate_path = PathBuf::from(custom_script);
        if candidate_path.exists() {
            return Ok(candidate_path);
        }
    }

    // Check sibling assets/ directory next to current executable
    if let Some(executable_directory) = std::env::current_exe()
        .ok()
        .and_then(|current_executable| current_executable.parent().map(Path::to_path_buf))
    {
        let candidate = executable_directory
            .join("assets")
            .join("scripts")
            .join("solix_scaffold.py");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // Materialize embedded scripts into user cache directory
    materialize_embedded_scripts()
}

/// Backward-compatibility alias for `resolve_scaffold_script_path`.
///
/// # Errors
/// Returns an error if filesystem operations during script materialization fail.
pub fn resolve_init_project_script_path() -> Result<PathBuf> {
    resolve_scaffold_script_path()
}

/// Materializes embedded `solix_scaffold.py` and `solix_build.py` into a cache folder.
fn materialize_embedded_scripts() -> Result<PathBuf> {
    let cache_directory = resolve_scripts_cache_dir();
    fs::create_dir_all(&cache_directory).with_context(|| {
        format!(
            "Failed to create scripts cache directory at '{}'",
            cache_directory.display()
        )
    })?;

    let scaffold_path = cache_directory.join("solix_scaffold.py");
    let build_path = cache_directory.join("solix_build.py");

    // Write scripts if missing or updated
    if !scaffold_path.exists()
        || fs::read_to_string(&scaffold_path)
            .map_or(true, |content| content != EMBEDDED_SOLIX_SCAFFOLD_SCRIPT)
    {
        fs::write(&scaffold_path, EMBEDDED_SOLIX_SCAFFOLD_SCRIPT)?;
    }

    if !build_path.exists()
        || fs::read_to_string(&build_path)
            .map_or(true, |content| content != EMBEDDED_SOLIX_BUILD_SCRIPT)
    {
        fs::write(&build_path, EMBEDDED_SOLIX_BUILD_SCRIPT)?;
    }

    Ok(scaffold_path)
}

/// Resolves the cache directory where materialized scripts should reside.
fn resolve_scripts_cache_dir() -> PathBuf {
    if let Ok(solix_home) = std::env::var("SOLIX_HOME") {
        PathBuf::from(solix_home).join("cache").join("scripts")
    } else if let Some(home_directory) = dirs_home_dir() {
        home_directory.join(".solix").join("cache").join("scripts")
    } else {
        std::env::temp_dir().join("solix_cache_scripts")
    }
}

/// Helper to get user home directory cross-platform.
fn dirs_home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
