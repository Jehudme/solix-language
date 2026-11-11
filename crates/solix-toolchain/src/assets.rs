//! Self-contained asset management and runtime materialization for `solix-toolchain`.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Compile-time embedded copy of `init_project.py`.
pub const EMBEDDED_INIT_PROJECT_SCRIPT: &str =
    include_str!("../assets/scripts/init_project.py");

/// Compile-time embedded copy of `solix_toolbox.py`.
pub const EMBEDDED_SOLIX_TOOLBOX_SCRIPT: &str =
    include_str!("../assets/scripts/solix_toolbox.py");

/// Resolves the filesystem path to `init_project.py`.
///
/// Priority:
/// 1. `SOLIX_INIT_PROJECT_PY` environment variable override.
/// 2. Sibling `<exe_dir>/assets/scripts/init_project.py` (staged by `build.rs`).
/// 3. Materialized fallback in `$SOLIX_HOME/cache/scripts/` or system temp dir.
pub fn resolve_init_project_script_path() -> Result<PathBuf> {
    if let Ok(custom_script) = std::env::var("SOLIX_INIT_PROJECT_PY") {
        let candidate_path = PathBuf::from(custom_script);
        if candidate_path.exists() {
            return Ok(candidate_path);
        }
    }

    // Check sibling assets/ directory next to current executable
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|current_executable| current_executable.parent().map(Path::to_path_buf))
    {
        let candidate = exe_dir.join("assets").join("scripts").join("init_project.py");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // Materialize embedded scripts into user cache directory
    materialize_embedded_scripts()
}

/// Materializes embedded `init_project.py` and `solix_toolbox.py` into a cache folder.
fn materialize_embedded_scripts() -> Result<PathBuf> {
    let cache_dir = resolve_scripts_cache_dir();
    fs::create_dir_all(&cache_dir).with_context(|| {
        format!(
            "Failed to create scripts cache directory at '{}'",
            cache_dir.display()
        )
    })?;

    let init_path = cache_dir.join("init_project.py");
    let toolbox_path = cache_dir.join("solix_toolbox.py");

    // Write scripts if missing or updated
    if !init_path.exists()
        || fs::read_to_string(&init_path).map_or(true, |content| content != EMBEDDED_INIT_PROJECT_SCRIPT)
    {
        fs::write(&init_path, EMBEDDED_INIT_PROJECT_SCRIPT)?;
    }

    if !toolbox_path.exists()
        || fs::read_to_string(&toolbox_path).map_or(true, |content| content != EMBEDDED_SOLIX_TOOLBOX_SCRIPT)
    {
        fs::write(&toolbox_path, EMBEDDED_SOLIX_TOOLBOX_SCRIPT)?;
    }

    Ok(init_path)
}

/// Resolves the cache directory where materialized scripts should reside.
fn resolve_scripts_cache_dir() -> PathBuf {
    if let Ok(solix_home) = std::env::var("SOLIX_HOME") {
        PathBuf::from(solix_home).join("cache").join("scripts")
    } else if let Some(home_dir) = dirs_home_dir() {
        home_dir.join(".solix").join("cache").join("scripts")
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
