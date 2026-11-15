//! Self-contained asset management and runtime materialization for `solix-toolchain`.

#![forbid(unsafe_code)]

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static IS_BOOTSTRAPPING: AtomicBool = AtomicBool::new(false);

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

/// Resolves the filesystem path to the directory containing bundled default packages.
///
/// Priority:
/// 1. `SOLIX_BUNDLED_PACKAGES_DIR` environment variable override.
/// 2. Sibling `<exe_dir>/assets/packages/`.
/// 3. Sibling `<exe_dir>/../assets/packages/`.
/// 4. Compile-time `CARGO_MANIFEST_DIR/assets/packages/` (fallback during local development).
/// 5. `<current_dir>/crates/solix-toolchain/assets/packages/`.
#[must_use]
pub fn resolve_bundled_packages_dir() -> Option<PathBuf> {
    if let Ok(custom_dir) = std::env::var("SOLIX_BUNDLED_PACKAGES_DIR") {
        let candidate = PathBuf::from(custom_dir);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    if let Some(executable_directory) = std::env::current_exe()
        .ok()
        .and_then(|current_executable| current_executable.parent().map(Path::to_path_buf))
    {
        let candidate = executable_directory.join("assets").join("packages");
        if candidate.exists() {
            return Some(candidate);
        }

        if let Some(parent_directory) = executable_directory.parent() {
            let alt_candidate = parent_directory.join("assets").join("packages");
            if alt_candidate.exists() {
                return Some(alt_candidate);
            }
        }
    }

    let manifest_assets = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("packages");
    if manifest_assets.exists() {
        return Some(manifest_assets);
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let repo_candidate = current_dir
            .join("crates")
            .join("solix-toolchain")
            .join("assets")
            .join("packages");
        if repo_candidate.exists() {
            return Some(repo_candidate);
        }
    }

    None
}

/// Discovers and installs all bundled default libraries from `assets/packages/` into `store_directory`.
///
/// # Errors
/// Returns an error if archive reading, unpacking, or registry writing fails.
pub fn install_bundled_packages(store_directory: &Path, force: bool) -> Result<usize> {
    if IS_BOOTSTRAPPING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(0);
    }

    struct BootstrapGuard;
    impl Drop for BootstrapGuard {
        fn drop(&mut self) {
            IS_BOOTSTRAPPING.store(false, Ordering::SeqCst);
        }
    }
    let _guard = BootstrapGuard;

    let bundled_directory = match resolve_bundled_packages_dir() {
        Some(dir) => dir,
        None => {
            if force {
                bail!("No bundled packages directory found in 'assets/packages/'");
            }
            return Ok(0);
        }
    };

    if !bundled_directory.exists() {
        if force {
            bail!(
                "Bundled packages directory does not exist at '{}'",
                bundled_directory.display()
            );
        }
        return Ok(0);
    }

    let mut installed_count: usize = 0;
    let registry_file = store_directory.join("registry.json");
    let existing_records: Vec<(String, String, String)> = if registry_file.exists()
        && let Ok(manifest_content) = fs::read_to_string(&registry_file)
        && let Ok(parsed_registry) =
            serde_json::from_str::<crate::registry::PackageRegistry>(&manifest_content)
    {
        parsed_registry
            .packages
            .into_iter()
            .map(|record| (record.name, record.build_profile, record.version))
            .collect()
    } else {
        Vec::new()
    };

    let manifest_path = bundled_directory.join("bundled_manifest.json");
    let mut candidates: Vec<(String, String, String, PathBuf)> = Vec::new();

    if manifest_path.exists()
        && let Ok(manifest_content) = fs::read_to_string(&manifest_path)
        && let Ok(manifest_json) = serde_json::from_str::<serde_json::Value>(&manifest_content)
        && let Some(packages_array) = manifest_json.get("packages").and_then(|val| val.as_array())
    {
        for pkg_entry in packages_array {
            let pkg_name = pkg_entry
                .get("name")
                .and_then(|val| val.as_str())
                .unwrap_or("");
            let pkg_version = pkg_entry
                .get("version")
                .and_then(|val| val.as_str())
                .unwrap_or("");
            let pkg_profile = pkg_entry
                .get("profile")
                .and_then(|val| val.as_str())
                .unwrap_or("release");
            let rel_path = pkg_entry
                .get("archive_rel_path")
                .and_then(|val| val.as_str())
                .unwrap_or("");

            if !pkg_name.is_empty() && !pkg_version.is_empty() && !rel_path.is_empty() {
                let full_archive_path = bundled_directory.join(rel_path);
                if full_archive_path.exists() {
                    candidates.push((
                        pkg_name.to_string(),
                        pkg_profile.to_string(),
                        pkg_version.to_string(),
                        full_archive_path,
                    ));
                }
            }
        }
    }

    if let Ok(entries) = fs::read_dir(&bundled_directory) {
        for entry_result in entries.flatten() {
            let entry_path = entry_result.path();
            if !entry_path.is_dir() {
                continue;
            }
            let folder_name = entry_result.file_name().to_string_lossy().to_string();
            if let Some((name_profile_part, version_part)) = folder_name.split_once('@') {
                let (pkg_name, pkg_profile) =
                    if let Some(base) = name_profile_part.strip_suffix("-debug") {
                        (base, "debug")
                    } else if let Some(base) = name_profile_part.strip_suffix("-release") {
                        (base, "release")
                    } else if let Some(base) = name_profile_part.strip_suffix("-test") {
                        (base, "test")
                    } else {
                        (name_profile_part, "release")
                    };

                let archive_candidate = entry_path.join(format!("{pkg_name}.slxarc"));
                if archive_candidate.exists()
                    && !candidates.iter().any(|(name, prof, ver, _)| {
                        name == pkg_name && prof == pkg_profile && ver == version_part
                    })
                {
                    candidates.push((
                        pkg_name.to_string(),
                        pkg_profile.to_string(),
                        version_part.to_string(),
                        archive_candidate,
                    ));
                }
            }
        }
    }

    for (package_name, package_profile, package_version, archive_path) in candidates {
        let already_installed =
            existing_records
                .iter()
                .any(|(record_name, record_profile, record_version)| {
                    record_name == &package_name
                        && record_profile == &package_profile
                        && record_version == &package_version
                });

        if already_installed && !force {
            continue;
        }

        crate::commands::install_package_artifact(
            &archive_path,
            None,
            force,
            Some(&package_profile),
            Some(store_directory),
        )?;
        installed_count += 1;
    }

    if !registry_file.exists() {
        let empty_registry = crate::registry::PackageRegistry::new_empty(store_directory);
        empty_registry.save()?;
    }

    let toolchain_version = env!("CARGO_PKG_VERSION");
    let marker_path = store_directory.join(format!(".bundled_bootstrapped_v{toolchain_version}"));
    let _ = fs::write(marker_path, "1");

    Ok(installed_count)
}

/// Ensures bundled packages are installed in `pkg_dir` on first run.
///
/// Fast no-op (<0.1ms) if `.bundled_bootstrapped_<version>` marker file exists,
/// or if `SOLIX_NO_AUTO_BOOTSTRAP=1` is set.
///
/// # Errors
/// Returns an error if installation fails.
pub fn ensure_bundled_packages_installed(pkg_dir: &Path) -> Result<()> {
    if let Ok(disabled_flag) = std::env::var("SOLIX_NO_AUTO_BOOTSTRAP") {
        let flag_lower = disabled_flag.trim().to_ascii_lowercase();
        if flag_lower == "1" || flag_lower == "true" || flag_lower == "yes" {
            return Ok(());
        }
    }

    let default_store = crate::solver::resolve_package_store_dir(None);
    let is_default_store = pkg_dir == default_store
        || std::env::var("SOLIX_FORCE_AUTO_BOOTSTRAP")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    if !is_default_store {
        return Ok(());
    }

    let toolchain_version = env!("CARGO_PKG_VERSION");
    let marker_path = pkg_dir.join(format!(".bundled_bootstrapped_v{toolchain_version}"));
    if marker_path.exists() {
        return Ok(());
    }

    if IS_BOOTSTRAPPING.load(Ordering::SeqCst) {
        return Ok(());
    }

    let _ = fs::write(&marker_path, "1");
    let result = install_bundled_packages(pkg_dir, false);
    result.map(|_| ())
}
