//! Self-contained asset management and runtime materialization for `solix-toolchain`.

#![forbid(unsafe_code)]

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static IS_BOOTSTRAPPING: AtomicBool = AtomicBool::new(false);
static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

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
    if !should_force_embedded_assets()
        && let Some(executable_directory) = std::env::current_exe()
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

    write_bytes_if_changed(&scaffold_path, EMBEDDED_SOLIX_SCAFFOLD_SCRIPT.as_bytes())?;
    write_bytes_if_changed(&build_path, EMBEDDED_SOLIX_BUILD_SCRIPT.as_bytes())?;

    Ok(scaffold_path)
}

/// Writes `expected_bytes` to `target_path` atomically only when missing or different.
fn write_bytes_if_changed(target_path: &Path, expected_bytes: &[u8]) -> Result<()> {
    if target_path.is_file()
        && fs::read(target_path).is_ok_and(|existing_bytes| existing_bytes == expected_bytes)
    {
        return Ok(());
    }

    let parent_directory = target_path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot determine parent directory for '{}'",
            target_path.display()
        )
    })?;
    fs::create_dir_all(parent_directory).with_context(|| {
        format!(
            "Failed to create parent directory at '{}'",
            parent_directory.display()
        )
    })?;

    let unique_id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    let temp_path = parent_directory.join(format!(
        ".{file_name}.tmp.{}.{unique_id}",
        std::process::id()
    ));

    fs::write(&temp_path, expected_bytes).with_context(|| {
        format!(
            "Failed to write temporary asset file at '{}'",
            temp_path.display()
        )
    })?;

    if let Err(rename_error) = fs::rename(&temp_path, target_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(rename_error).with_context(|| {
            format!(
                "Failed to move temporary asset file into '{}'",
                target_path.display()
            )
        });
    }

    Ok(())
}

/// Resolves a cache subdirectory inside `$SOLIX_HOME/cache/`, `~/.solix/cache/`, or temp dir.
fn resolve_solix_cache_subdir(subdir: &str, temp_fallback_name: &str) -> PathBuf {
    if let Ok(solix_home) = std::env::var("SOLIX_HOME") {
        let trimmed_home = solix_home.trim();
        if !trimmed_home.is_empty() {
            return PathBuf::from(trimmed_home).join("cache").join(subdir);
        }
    }

    if let Some(home_directory) = dirs_home_dir() {
        home_directory.join(".solix").join("cache").join(subdir)
    } else {
        std::env::temp_dir().join(temp_fallback_name)
    }
}

/// Resolves the cache directory where materialized scripts should reside.
fn resolve_scripts_cache_dir() -> PathBuf {
    resolve_solix_cache_subdir("scripts", "solix_cache_scripts")
}

/// Resolves the cache directory where materialized bundled default packages reside.
fn resolve_bundled_packages_cache_dir() -> PathBuf {
    resolve_solix_cache_subdir("bundled_packages", "solix_cache_bundled_packages")
}

/// Resolves the cache directory where the materialized VS Code extension `.vsix` resides.
fn resolve_vscode_cache_dir() -> PathBuf {
    resolve_solix_cache_subdir("vscode", "solix_cache_vscode")
}

/// Resolves the version-isolated cache directory where materialized offline wiki docs reside.
fn resolve_wiki_cache_dir() -> PathBuf {
    let version_folder = format!("v{}", env!("CARGO_PKG_VERSION"));
    resolve_solix_cache_subdir("wiki", "solix_cache_wiki").join(version_folder)
}

/// Returns `true` when `SOLIX_FORCE_EMBEDDED_ASSETS` forces skipping workspace/exe disk probes.
fn should_force_embedded_assets() -> bool {
    std::env::var("SOLIX_FORCE_EMBEDDED_ASSETS")
        .map(|value| {
            let trimmed = value.trim();
            trimmed == "1" || trimmed.eq_ignore_ascii_case("true")
        })
        .unwrap_or(false)
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

/// Decodes a packed asset blob generated by `build.rs` into `(relative_path, file_bytes)` entries.
fn decode_packed_assets(pack_bytes: &'static [u8]) -> Result<Vec<(&'static str, &'static [u8])>> {
    if pack_bytes.len() < 4 {
        return Ok(Vec::new());
    }
    let count =
        u32::from_le_bytes([pack_bytes[0], pack_bytes[1], pack_bytes[2], pack_bytes[3]]) as usize;
    let mut entries = Vec::with_capacity(count);
    let mut offset = 4usize;

    for _ in 0..count {
        if offset + 8 > pack_bytes.len() {
            bail!("Corrupted embedded asset pack header");
        }
        let path_len = u32::from_le_bytes([
            pack_bytes[offset],
            pack_bytes[offset + 1],
            pack_bytes[offset + 2],
            pack_bytes[offset + 3],
        ]) as usize;
        let data_len = u32::from_le_bytes([
            pack_bytes[offset + 4],
            pack_bytes[offset + 5],
            pack_bytes[offset + 6],
            pack_bytes[offset + 7],
        ]) as usize;
        offset += 8;

        let path_end = offset.saturating_add(path_len);
        let data_end = path_end.saturating_add(data_len);
        if data_end > pack_bytes.len() {
            bail!("Corrupted embedded asset pack entry bounds");
        }
        let rel_path = std::str::from_utf8(&pack_bytes[offset..path_end])
            .context("Invalid UTF-8 path in embedded asset pack")?;
        let payload = &pack_bytes[path_end..data_end];
        entries.push((rel_path, payload));
        offset = data_end;
    }

    Ok(entries)
}

/// Returns `true` when `EMBEDDED_BUNDLED_PACKAGES_PACK` contains at least one embedded archive.
#[must_use]
pub const fn has_embedded_bundled_packages() -> bool {
    !EMBEDDED_BUNDLED_PACKAGES_PACK.is_empty()
}

/// Returns `true` when `EMBEDDED_VSCODE_VSIX` contains an embedded VS Code extension archive.
#[must_use]
pub const fn has_embedded_vscode_extension() -> bool {
    EMBEDDED_VSCODE_VSIX.is_some()
}

/// Returns `true` when `EMBEDDED_WIKI_DOCS_PACK` contains embedded offline documentation pages.
#[must_use]
pub const fn has_embedded_wiki_docs() -> bool {
    !EMBEDDED_WIKI_DOCS_PACK.is_empty()
}

/// Materializes compile-time embedded bundled `.slxarc` archives and manifest into cache.
///
/// # Errors
/// Returns an error if cache directory creation or file writing fails.
pub fn materialize_embedded_bundled_packages() -> Result<Option<PathBuf>> {
    if EMBEDDED_BUNDLED_PACKAGES_PACK.is_empty() {
        return Ok(None);
    }
    let embedded_packages = decode_packed_assets(EMBEDDED_BUNDLED_PACKAGES_PACK)?;
    if embedded_packages.is_empty() {
        return Ok(None);
    }

    let cache_directory = resolve_bundled_packages_cache_dir();
    fs::create_dir_all(&cache_directory).with_context(|| {
        format!(
            "Failed to create bundled packages cache directory at '{}'",
            cache_directory.display()
        )
    })?;

    for (archive_rel_path, archive_bytes) in embedded_packages {
        let mut target_path = cache_directory.clone();
        for segment in archive_rel_path.split('/') {
            if !segment.is_empty() {
                target_path.push(segment);
            }
        }
        write_bytes_if_changed(&target_path, archive_bytes)?;
    }

    if let Some(manifest_bytes) = EMBEDDED_BUNDLED_MANIFEST {
        write_bytes_if_changed(
            &cache_directory.join("bundled_manifest.json"),
            manifest_bytes,
        )?;
    }

    Ok(Some(cache_directory))
}

/// Materializes the compile-time embedded Solix VS Code extension `.vsix` into cache.
///
/// # Errors
/// Returns an error if cache directory creation or file writing fails.
pub fn materialize_embedded_vscode_extension() -> Result<Option<PathBuf>> {
    let Some((extension_name, extension_bytes)) = EMBEDDED_VSCODE_VSIX else {
        return Ok(None);
    };

    let cache_directory = resolve_vscode_cache_dir();
    let target_path = cache_directory.join(extension_name);
    write_bytes_if_changed(&target_path, extension_bytes)?;

    Ok(Some(target_path))
}

/// Materializes the compile-time embedded offline documentation portal into versioned cache.
///
/// # Errors
/// Returns an error if any embedded path is malformed or filesystem writes fail.
fn materialize_relative_file(
    cache_directory: &Path,
    relative_path: &str,
    file_bytes: &[u8],
) -> Result<()> {
    let mut target_path = cache_directory.to_path_buf();
    for segment in relative_path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains('\\') || segment.contains('\0') {
            bail!("Unsafe embedded wiki asset path: '{relative_path}'");
        }
        target_path.push(segment);
    }
    write_bytes_if_changed(&target_path, file_bytes)
}

/// Materializes all embedded offline HTML wiki pages into cache if compiled in.
///
/// # Errors
/// Returns an error if filesystem creation or asset writes fail.
pub fn materialize_embedded_wiki_docs() -> Result<Option<PathBuf>> {
    if EMBEDDED_WIKI_DOCS_PACK.is_empty() {
        return Ok(None);
    }

    let cache_directory = resolve_wiki_cache_dir();

    if EMBEDDED_WIKI_DOCS_PACK.starts_with(b"PK\x03\x04") {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(EMBEDDED_WIKI_DOCS_PACK))
            .context("Failed to open embedded compressed wiki documentation archive")?;
        if archive.is_empty() {
            return Ok(None);
        }
        fs::create_dir_all(&cache_directory).with_context(|| {
            format!(
                "Failed to create wiki cache directory at '{}'",
                cache_directory.display()
            )
        })?;
        let mut file_bytes = Vec::new();
        for entry_idx in 0..archive.len() {
            let mut entry = archive
                .by_index(entry_idx)
                .context("Failed to read embedded wiki entry")?;
            if entry.is_dir() {
                continue;
            }
            let relative_path = entry
                .name()
                .context("Invalid UTF-8 path in embedded wiki archive")?
                .into_owned();
            file_bytes.clear();
            std::io::Read::read_to_end(&mut entry, &mut file_bytes)
                .with_context(|| format!("Failed to decompress wiki page '{relative_path}'"))?;
            materialize_relative_file(&cache_directory, &relative_path, &file_bytes)?;
        }
        return Ok(Some(cache_directory));
    }

    let embedded_wiki_docs = decode_packed_assets(EMBEDDED_WIKI_DOCS_PACK)?;
    if embedded_wiki_docs.is_empty() {
        return Ok(None);
    }

    fs::create_dir_all(&cache_directory).with_context(|| {
        format!(
            "Failed to create wiki cache directory at '{}'",
            cache_directory.display()
        )
    })?;

    for (relative_path, file_bytes) in embedded_wiki_docs {
        materialize_relative_file(&cache_directory, relative_path, file_bytes)?;
    }

    Ok(Some(cache_directory))
}

/// Checks whether `directory` contains a rendered `index.html` offline wiki portal.
fn directory_has_wiki_index(directory: &Path) -> bool {
    directory.is_dir() && directory.join("index.html").is_file()
}

/// Resolves the directory containing the offline HTML documentation portal for this toolchain version.
///
/// Priority:
/// 1. `SOLIX_WIKI_DIR` environment variable override.
/// 2. Sibling `<exe_dir>/assets/docs/` containing `index.html`.
/// 3. Sibling `<exe_dir>/../assets/docs/` containing `index.html`.
/// 4. Compile-time `CARGO_MANIFEST_DIR/assets/docs/` containing `index.html`.
/// 5. Workspace fallback `<current_dir>/crates/solix-toolchain/assets/docs/`.
/// 6. Materialized fallback from compile-time embedded HTML portal into `$SOLIX_HOME/cache/wiki/v<version>/`.
///
/// # Errors
/// Returns an error if no offline wiki portal was staged or embedded and cache materialization yields nothing.
pub fn resolve_wiki_docs_dir() -> Result<PathBuf> {
    if let Ok(custom_dir) = std::env::var("SOLIX_WIKI_DIR") {
        let trimmed = custom_dir.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if directory_has_wiki_index(&candidate) {
                return Ok(candidate);
            }
        }
    }

    if !should_force_embedded_assets() {
        if let Some(executable_directory) = std::env::current_exe()
            .ok()
            .and_then(|current_executable| current_executable.parent().map(Path::to_path_buf))
        {
            let candidate = executable_directory.join("assets").join("docs");
            if directory_has_wiki_index(&candidate) {
                return Ok(candidate);
            }

            if let Some(parent_directory) = executable_directory.parent() {
                let alt_candidate = parent_directory.join("assets").join("docs");
                if directory_has_wiki_index(&alt_candidate) {
                    return Ok(alt_candidate);
                }
            }
        }

        let manifest_assets = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("docs");
        if directory_has_wiki_index(&manifest_assets) {
            return Ok(manifest_assets);
        }

        if let Ok(current_dir) = std::env::current_dir() {
            let repo_candidate = current_dir
                .join("crates")
                .join("solix-toolchain")
                .join("assets")
                .join("docs");
            if directory_has_wiki_index(&repo_candidate) {
                return Ok(repo_candidate);
            }
        }
    }

    if let Some(materialized) = materialize_embedded_wiki_docs()?
        && directory_has_wiki_index(&materialized)
    {
        return Ok(materialized);
    }

    bail!(
        "No offline wiki HTML documentation found for Solix v{}. \
Run `python3 scripts/build_docs_site.py` before building or set SOLIX_WIKI_DIR.",
        env!("CARGO_PKG_VERSION")
    )
}

/// Recursively exports offline documentation files from `source_dir` into `destination_dir`.
///
/// # Errors
/// Returns an error if reading or copying files fails.
pub fn export_wiki_docs_to_dir(source_dir: &Path, destination_dir: &Path) -> Result<usize> {
    copy_directory_recursive(source_dir, destination_dir)
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<usize> {
    fs::create_dir_all(destination).with_context(|| {
        format!(
            "Failed to create documentation export directory '{}'",
            destination.display()
        )
    })?;
    let mut copied_files = 0usize;
    let entries = fs::read_dir(source).with_context(|| {
        format!(
            "Failed to read wiki source directory '{}'",
            source.display()
        )
    })?;

    for entry_result in entries {
        let entry = entry_result?;
        let entry_path = entry.path();
        let target_path = destination.join(entry.file_name());
        if entry_path.is_dir() {
            copied_files += copy_directory_recursive(&entry_path, &target_path)?;
        } else if entry_path.is_file() {
            let bytes = fs::read(&entry_path)
                .with_context(|| format!("Failed to read wiki asset '{}'", entry_path.display()))?;
            write_bytes_if_changed(&target_path, &bytes)?;
            copied_files += 1;
        }
    }

    Ok(copied_files)
}

/// Checks whether a candidate `packages/` directory actually contains at least one `.slxarc` archive.
fn directory_has_bundled_packages(directory: &Path) -> bool {
    if !directory.is_dir() {
        return false;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };

    for entry_result in entries.flatten() {
        let entry_path = entry_result.path();
        if !entry_path.is_dir() {
            continue;
        }
        let Ok(sub_entries) = fs::read_dir(&entry_path) else {
            continue;
        };
        for sub_entry in sub_entries.flatten() {
            let file_path = sub_entry.path();
            if file_path.is_file()
                && file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext_str| ext_str.eq_ignore_ascii_case("slxarc"))
            {
                return true;
            }
        }
    }

    false
}

/// Resolves the filesystem path to the directory containing bundled default packages.
///
/// Priority:
/// 1. `SOLIX_BUNDLED_PACKAGES_DIR` environment variable override.
/// 2. Sibling `<exe_dir>/assets/packages/` containing `.slxarc` archives.
/// 3. Sibling `<exe_dir>/../assets/packages/` containing `.slxarc` archives.
/// 4. Compile-time `CARGO_MANIFEST_DIR/assets/packages/` containing `.slxarc` archives.
/// 5. `<current_dir>/crates/solix-toolchain/assets/packages/` containing `.slxarc` archives.
/// 6. Materialized fallback from compile-time embedded `.slxarc` archives into cache.
#[must_use]
pub fn resolve_bundled_packages_dir() -> Option<PathBuf> {
    if let Ok(custom_dir) = std::env::var("SOLIX_BUNDLED_PACKAGES_DIR") {
        let trimmed_dir = custom_dir.trim();
        if !trimmed_dir.is_empty() {
            let candidate = PathBuf::from(trimmed_dir);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    if !should_force_embedded_assets() {
        if let Some(executable_directory) = std::env::current_exe()
            .ok()
            .and_then(|current_executable| current_executable.parent().map(Path::to_path_buf))
        {
            let candidate = executable_directory.join("assets").join("packages");
            if directory_has_bundled_packages(&candidate) {
                return Some(candidate);
            }

            if let Some(parent_directory) = executable_directory.parent() {
                let alt_candidate = parent_directory.join("assets").join("packages");
                if directory_has_bundled_packages(&alt_candidate) {
                    return Some(alt_candidate);
                }
            }
        }

        let manifest_assets = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("packages");
        if directory_has_bundled_packages(&manifest_assets) {
            return Some(manifest_assets);
        }

        if let Ok(current_dir) = std::env::current_dir() {
            let repo_candidate = current_dir
                .join("crates")
                .join("solix-toolchain")
                .join("assets")
                .join("packages");
            if directory_has_bundled_packages(&repo_candidate) {
                return Some(repo_candidate);
            }
        }
    }

    materialize_embedded_bundled_packages().ok().flatten()
}

/// Resolves the filesystem path to the bundled Solix Visual Studio Code extension (`.vsix`).
///
/// Priority:
/// 1. `SOLIX_VSCODE_VSIX_PATH` environment variable override.
/// 2. Sibling `<exe_dir>/assets/vscode/solix-language.vsix` or `<exe_dir>/assets/vscode/*.vsix`.
/// 3. Sibling `<exe_dir>/../assets/vscode/*.vsix`.
/// 4. Compile-time `CARGO_MANIFEST_DIR/assets/vscode/*.vsix`.
/// 5. Repository fallback: `<workspace_root>/editors/vscode/*.vsix`.
/// 6. Materialized fallback from the compile-time embedded `.vsix` into cache.
#[must_use]
pub fn resolve_bundled_vscode_extension_path() -> Option<PathBuf> {
    if let Ok(custom_path) = std::env::var("SOLIX_VSCODE_VSIX_PATH") {
        let trimmed_path = custom_path.trim();
        if !trimmed_path.is_empty() {
            let candidate = PathBuf::from(trimmed_path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    if !should_force_embedded_assets() {
        if let Some(executable_directory) = std::env::current_exe()
            .ok()
            .and_then(|current_executable| current_executable.parent().map(Path::to_path_buf))
        {
            let candidate_directory = executable_directory.join("assets").join("vscode");
            if let Some(extension_path) = find_vsix_in_directory(&candidate_directory) {
                return Some(extension_path);
            }

            if let Some(parent_directory) = executable_directory.parent() {
                let alternative_directory = parent_directory.join("assets").join("vscode");
                if let Some(extension_path) = find_vsix_in_directory(&alternative_directory) {
                    return Some(extension_path);
                }
            }
        }

        let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("vscode");
        if let Some(extension_path) = find_vsix_in_directory(&manifest_directory) {
            return Some(extension_path);
        }

        if let Ok(current_dir) = std::env::current_dir() {
            let editors_directory = current_dir.join("editors").join("vscode");
            if let Some(extension_path) = find_vsix_in_directory(&editors_directory) {
                return Some(extension_path);
            }
            let workspace_toolchain_directory = current_dir
                .join("crates")
                .join("solix-toolchain")
                .join("assets")
                .join("vscode");
            if let Some(extension_path) = find_vsix_in_directory(&workspace_toolchain_directory) {
                return Some(extension_path);
            }
        }
    }

    materialize_embedded_vscode_extension().ok().flatten()
}

/// Discovers a `.vsix` file in the specified directory.
fn find_vsix_in_directory(directory: &Path) -> Option<PathBuf> {
    if !directory.is_dir() {
        return None;
    }

    let canonical_file = directory.join("solix-language.vsix");
    if canonical_file.is_file() {
        return Some(canonical_file);
    }

    if let Ok(entries) = fs::read_dir(directory) {
        let mut candidates: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file()
                && entry_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext_str| ext_str.eq_ignore_ascii_case("vsix"))
            {
                candidates.push(entry_path);
            }
        }
        candidates.sort();
        if let Some(latest_file) = candidates.pop() {
            return Some(latest_file);
        }
    }

    None
}

/// Discovers and installs all bundled default libraries from `assets/packages/` into `store_directory`.
///
/// # Errors
/// Returns an error if archive reading, unpacking, or registry writing fails.
pub fn install_bundled_packages(store_directory: &Path, force: bool) -> Result<usize> {
    install_bundled_packages_inner(store_directory, force, false)
}

fn install_bundled_packages_inner(
    store_directory: &Path,
    force: bool,
    quiet: bool,
) -> Result<usize> {
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
                bail!(
                    "No bundled packages directory found in 'assets/packages/' or embedded in the Solix binary."
                );
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

    if candidates.is_empty()
        && let Ok(entries) = fs::read_dir(&bundled_directory)
    {
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

    if candidates.is_empty() {
        if force {
            bail!(
                "No bundled package archives (.slxarc) found in '{}'",
                bundled_directory.display()
            );
        }
        return Ok(0);
    }

    fs::create_dir_all(store_directory).with_context(|| {
        format!(
            "Failed to create package store directory at '{}'",
            store_directory.display()
        )
    })?;

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

        crate::commands::pkg::install_package_artifact_with_options(
            &archive_path,
            None,
            force,
            Some(&package_profile),
            Some(store_directory),
            quiet,
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

    let result = install_bundled_packages_inner(pkg_dir, false, true);
    result.map(|_| ())
}
