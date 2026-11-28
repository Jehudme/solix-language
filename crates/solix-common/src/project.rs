//! Project manifest schema (`manifest.toml`) and `.slxarc` archive container manager.

#![forbid(unsafe_code)]

use crate::bytecode::BinaryProgram;
use crate::diagnostics::EmissionError;
use crate::diagnostics::manifest::{
    ManifestError, enrich_manifest_error_message, extract_yaml_breadcrumb_path,
    render_source_snippet,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

fn strip_verbatim_prefix_text(path_text: &str) -> Option<PathBuf> {
    if let Some(unc_tail) = path_text.strip_prefix(r"\\?\UNC\") {
        return Some(PathBuf::from(format!(r"\\{unc_tail}")));
    }
    if let Some(drive_tail) = path_text.strip_prefix(r"\\?\") {
        return Some(PathBuf::from(drive_tail));
    }
    None
}

/// Strips Windows verbatim path prefixes (`\\?\` and `\\?\UNC\`) so external tools
/// such as CMake and MSBuild receive standard host paths instead of failing in `cmd.exe`.
#[must_use]
pub fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    if cfg!(windows)
        && let Some(stripped) = path.to_str().and_then(strip_verbatim_prefix_text)
    {
        return stripped;
    }
    path
}

/// Canonicalizes `path` and normalizes any Windows verbatim (`\\?\`) prefix.
///
/// # Errors
/// Propagates any I/O failure returned by [`Path::canonicalize`].
pub fn canonicalize_host_path(path: &Path) -> std::io::Result<PathBuf> {
    let canonical = path.canonicalize()?;
    Ok(strip_windows_verbatim_prefix(canonical))
}

fn default_project_version() -> String {
    "0.1.0".to_string()
}

/// Canonical desktop platform identifiers supported by cross-platform `.slxarc` archives.
pub const ALL_DESKTOP_PLATFORMS: [&str; 5] = [
    "linux-x86_64",
    "linux-aarch64",
    "windows-x86_64",
    "macos-x86_64",
    "macos-aarch64",
];

/// Returns the canonical `<os>-<arch>` platform identifier for the running host.
#[must_use]
pub fn host_platform_id() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        ("windows", "aarch64") => "windows-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        _ => "unknown",
    }
}

/// Returns the default Rust target triple for the running host.
#[must_use]
pub fn host_target_triple() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        _ => "x86_64-unknown-linux-gnu",
    }
}

/// Maps a Rust target triple (e.g. `"x86_64-unknown-linux-gnu"`) to a canonical platform identifier.
#[must_use]
pub fn triple_to_platform_id(triple: &str) -> Option<&'static str> {
    match triple.trim() {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" | "linux-x86_64" => {
            Some("linux-x86_64")
        }
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" | "linux-aarch64" => {
            Some("linux-aarch64")
        }
        "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" | "windows-x86_64" => {
            Some("windows-x86_64")
        }
        "x86_64-apple-darwin" | "macos-x86_64" => Some("macos-x86_64"),
        "aarch64-apple-darwin" | "macos-aarch64" => Some("macos-aarch64"),
        _ => None,
    }
}

/// Canonical manifest filenames searched in priority order (`manifest.yaml` first).
pub const MANIFEST_FILENAMES: [&str; 3] = ["manifest.yaml", "manifest.yml", "manifest.toml"];

/// Searches `project_root` for a manifest file in canonical priority order (`manifest.yaml`, `manifest.yml`, `manifest.toml`).
#[must_use]
pub fn find_manifest_path(project_root: &Path) -> Option<PathBuf> {
    for candidate_name in MANIFEST_FILENAMES {
        let candidate_path = project_root.join(candidate_name);
        if candidate_path.is_file() {
            return Some(candidate_path);
        }
    }
    None
}

/// Maps a canonical platform identifier (`"linux-x86_64"`, `"macos-aarch64"`, etc.) to its primary target triple.
#[must_use]
pub fn platform_id_to_target_triple(platform_id: &str) -> Option<&'static str> {
    match platform_id.trim() {
        "linux-x86_64" => Some("x86_64-unknown-linux-gnu"),
        "linux-aarch64" => Some("aarch64-unknown-linux-gnu"),
        "windows-x86_64" => Some("x86_64-pc-windows-gnu"),
        "macos-x86_64" => Some("x86_64-apple-darwin"),
        "macos-aarch64" => Some("aarch64-apple-darwin"),
        _ => None,
    }
}

/// Normalizes a platform specifier token or Rust triple to its canonical platform ID.
#[must_use]
pub fn normalize_platform_token(token: &str) -> String {
    let trimmed = token.trim().to_ascii_lowercase();
    if let Some(canonical) = triple_to_platform_id(&trimmed) {
        return canonical.to_string();
    }
    trimmed
}

/// Expands shorthand platform tokens (`"all"`, `"host"`, `"linux"`, `"windows"`, `"macos"`, `"apple"`)
/// into canonical `<os>-<arch>` identifiers covering every architecture for each declared OS platform.
#[must_use]
pub fn expand_platform_list(platforms: &[String]) -> Vec<String> {
    let mut expanded: Vec<String> = Vec::new();
    let mut push_unique = |identifier: &str| {
        if !identifier.is_empty() && !expanded.iter().any(|existing| existing == identifier) {
            expanded.push(identifier.to_string());
        }
    };

    for entry in platforms {
        let normalized = normalize_platform_token(entry);
        match normalized.as_str() {
            "all" => {
                for desktop_platform in ALL_DESKTOP_PLATFORMS {
                    push_unique(desktop_platform);
                }
            }
            "host" => {
                push_unique(host_platform_id());
            }
            "linux" => {
                push_unique("linux-x86_64");
                push_unique("linux-aarch64");
            }
            "macos" | "apple" | "darwin" => {
                push_unique("macos-x86_64");
                push_unique("macos-aarch64");
            }
            "windows" | "win" | "win32" | "win64" => {
                push_unique("windows-x86_64");
            }
            other => {
                push_unique(other);
            }
        }
    }
    expanded
}

/// Expands platform specifiers (`"linux"`, `"macos"`, `"windows"`, `"all"`, `"host"`, or explicit IDs/triples)
/// directly into target triples covering all architectures for each requested platform.
#[must_use]
pub fn expand_platforms_to_triples(platforms: &[String]) -> Vec<String> {
    let mut triples = Vec::new();
    let host_plat = host_platform_id();
    let host_triple = host_target_triple();
    let expanded_platforms = expand_platform_list(platforms);
    for platform_id in expanded_platforms {
        if platform_id == "universal" {
            continue;
        }
        if platform_id == host_plat {
            let owned_host = host_triple.to_string();
            if !triples.contains(&owned_host) {
                triples.push(owned_host);
            }
        } else if let Some(triple) = platform_id_to_target_triple(&platform_id) {
            let owned_triple = triple.to_string();
            if !triples.contains(&owned_triple) {
                triples.push(owned_triple);
            }
        } else if !platform_id.is_empty() && !triples.contains(&platform_id) {
            triples.push(platform_id);
        }
    }
    if triples.is_empty() {
        triples.push(host_triple.to_string());
    }
    triples
}

/// Validates that an archive manifest's `supported_platforms` permits execution on the current host.
///
/// Archives without explicit `supported_platforms` or marked `"universal"` / `"all"` are permitted.
pub fn validate_archive_host_compatibility(
    manifest: &ProjectManifest,
    archive_display: &str,
) -> Result<(), EmissionError> {
    if manifest.supported_platforms.is_empty() {
        return Ok(());
    }

    let expanded = expand_platform_list(&manifest.supported_platforms);
    if expanded.is_empty() {
        return Ok(());
    }

    let host = host_platform_id();
    let is_compatible = expanded
        .iter()
        .any(|platform| platform == "universal" || platform == "all" || platform == host);
    if !is_compatible {
        return Err(EmissionError::ContainerSerializationFailed {
            reason: format!(
                "Archive '{}' only supports platform(s) [{}], which is incompatible with current host '{}'.",
                archive_display,
                expanded.join(", "),
                host
            ),
        });
    }

    Ok(())
}

/// User-editable project identity metadata declared in the `[configuration]` table of `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfiguration {
    /// Package or project canonical identifier name.
    pub name: String,
    /// Semantic version string (e.g., `"0.1.0"`).
    #[serde(default = "default_project_version")]
    pub version: String,
    /// Brief description of the project purpose and functionality.
    #[serde(default)]
    pub description: Option<String>,
    /// Software distribution license identifier (e.g., `"Apache-2.0"`).
    #[serde(default)]
    pub license: Option<String>,
    /// List of project authors and contributors.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Optional legacy entrypoint retained for backwards compatibility.
    #[serde(default, skip_serializing)]
    pub entrypoint: Option<String>,
}

/// Backwards compatibility alias for `ProjectConfiguration`.
pub type ProjectInfo = ProjectConfiguration;

/// Native CMake or Cargo subproject configuration declared under `profiles.<name>.natives.<key>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTargetConfig {
    /// Relative path to native project root (defaults to `natives/<key>`).
    #[serde(default)]
    pub path: Option<String>,
    /// Native build optimization profile (`"debug"` or `"release"`; defaults to parent Solix profile).
    #[serde(default)]
    pub profile: Option<String>,
    /// Target OS platform names (`"linux"`, `"windows"`, `"macos"`, `"all"`, `"host"`, or explicit triples).
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
    /// Extra flags forwarded directly to CMake (`-D...`) or Cargo (`--features ...`).
    #[serde(default)]
    pub flags: Vec<String>,
    /// Explicit native build strategy (e.g. `"host"`, `"zigbuild"`, `"docker"`, `"container"`).
    #[serde(default)]
    pub strategy: Option<String>,
    /// Whether to force containerized execution via Docker (cross / dockcross).
    #[serde(default)]
    pub docker: Option<bool>,
    /// Optional custom Docker container image override.
    #[serde(default)]
    pub docker_image: Option<String>,
}

impl NativeTargetConfig {
    /// Returns true if this native target is configured to compile inside a Docker container.
    /// Containerized compilation via Docker is the mandated default for all native targets.
    #[must_use]
    pub fn is_docker_strategy(&self) -> bool {
        self.docker.unwrap_or(true)
            || self.strategy.as_deref().is_some_and(|strategy_name| {
                strategy_name.eq_ignore_ascii_case("docker")
                    || strategy_name.eq_ignore_ascii_case("container")
            })
    }
}

/// Profile-specific compilation, native build, hook, dependency, and source configuration declared in `profiles.<name>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileConfig {
    /// Optional canonical entrypoint method symbol (e.g. `"Main::main"`, `"TestRunner::test"`).
    #[serde(default)]
    pub entrypoint: Option<String>,
    /// Optimization level: 0..=3.
    #[serde(default, alias = "opt_level")]
    pub optimization_level: Option<u8>,
    /// Inlining threshold statement limit.
    #[serde(default)]
    pub inline_threshold: Option<usize>,
    /// Whether to generate debug line numbers and symbol information.
    #[serde(default)]
    pub debug_symbols: Option<bool>,
    /// Whether to link statically (`true`) or dynamically (`false`).
    #[serde(default, rename = "static")]
    pub is_static: Option<bool>,
    /// Explicit linking mode: `"dynamic"` or `"static"`.
    #[serde(default)]
    pub link: Option<String>,
    /// Whether to emit VM disassembly alongside the binary.
    #[serde(default)]
    pub emit_disassembly: Option<bool>,
    /// Whether warnings should be treated as fatal compilation errors.
    #[serde(default)]
    pub warnings_as_errors: Option<bool>,
    /// Verify cryptographic checksums of packages.
    #[serde(default)]
    pub verify_checksum: Option<bool>,
    /// Custom maximum memory limit in megabytes.
    #[serde(default)]
    pub memory_limit_mb: Option<usize>,
    /// List of directory paths (recursive) relative to project root containing `.slx` sources.
    #[serde(default)]
    pub sources: Option<Vec<String>>,
    /// Target platform constraints for this profile (e.g. `["all"]`, `["host"]`, `["linux", "windows", "macos"]`).
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
    /// Commands executed sequentially before native and Solix compilation.
    #[serde(default)]
    pub pre_compile: Vec<String>,
    /// Commands executed sequentially after archive packaging.
    #[serde(default)]
    pub post_compile: Vec<String>,
    /// Removed inheritance property retained for helpful diagnostic error reporting.
    #[serde(default, skip_serializing)]
    pub inherits: Option<serde_json::Value>,
    /// Profile-scoped dependencies mapped as `name => specification`.
    #[serde(default, deserialize_with = "deserialize_profile_dependencies")]
    pub dependencies: std::collections::BTreeMap<String, DependencySpec>,
    /// Profile-scoped native subprojects mapped as `name => native_config`.
    #[serde(default, deserialize_with = "deserialize_profile_natives")]
    pub natives: std::collections::BTreeMap<String, NativeTargetConfig>,
    /// Declared dynamic shared library dependencies required by this profile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dynamic_libraries: Vec<String>,
    /// External pre-compiled shared libraries required by this profile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<String>,
}

/// Fully resolved, self-contained profile configuration (no cross-profile inheritance).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProfile {
    /// Profile identifier name (e.g. `"debug"`, `"release"`, `"test"`).
    pub name: String,
    /// Canonical entrypoint symbol override.
    pub entrypoint: Option<String>,
    /// Optimization level: 0..=3.
    pub opt_level: u8,
    /// Inlining threshold statement limit.
    pub inline_threshold: usize,
    /// Whether debug symbol tables and line maps are emitted.
    pub debug_symbols: bool,
    /// Whether static linking is enabled.
    pub is_static: bool,
    /// Whether human-readable VM disassembly is emitted.
    pub emit_disassembly: bool,
    /// Whether warnings are treated as fatal errors.
    pub warnings_as_errors: bool,
    /// Verify cryptographic checksums.
    pub verify_checksum: bool,
    /// Dynamic or static link mode string.
    pub link: String,
    /// Maximum virtual heap allocation limit in megabytes.
    pub memory_limit_mb: Option<usize>,
    /// Recursive source directories to compile.
    pub sources: Vec<String>,
    /// Resolved target platform specifiers for this profile.
    pub platforms: Vec<String>,
    /// Declared dynamic shared library dependencies.
    pub dynamic_libraries: Vec<String>,
    /// Pre-compiled external shared libraries.
    pub libraries: Vec<String>,
    /// Commands executed sequentially before native and Solix compilation.
    pub pre_compile: Vec<String>,
    /// Commands executed sequentially after packaging.
    pub post_compile: Vec<String>,
    /// Profile-scoped resolved dependencies.
    pub dependencies: std::collections::BTreeMap<String, DependencySpec>,
    /// Profile-scoped native projects.
    pub natives: std::collections::BTreeMap<String, NativeTargetConfig>,
}

impl ResolvedProfile {
    /// Creates the standard default baseline profile for release builds.
    #[must_use]
    pub fn default_release_baseline() -> Self {
        Self {
            name: "release".to_string(),
            entrypoint: Some("Main::main".to_string()),
            opt_level: 3,
            inline_threshold: 16,
            debug_symbols: false,
            is_static: true,
            emit_disassembly: false,
            warnings_as_errors: false,
            verify_checksum: false,
            link: "static".to_string(),
            memory_limit_mb: None,
            sources: vec!["src".to_string()],
            platforms: vec!["all".to_string()],
            dynamic_libraries: Vec::new(),
            libraries: Vec::new(),
            pre_compile: Vec::new(),
            post_compile: Vec::new(),
            dependencies: std::collections::BTreeMap::new(),
            natives: std::collections::BTreeMap::new(),
        }
    }

    /// Creates the standard default baseline profile for debug or custom builds.
    #[must_use]
    pub fn default_debug_baseline(profile_name: &str) -> Self {
        Self {
            name: profile_name.to_string(),
            entrypoint: Some("Main::main".to_string()),
            opt_level: 0,
            inline_threshold: 8,
            debug_symbols: true,
            is_static: false,
            emit_disassembly: false,
            warnings_as_errors: false,
            verify_checksum: false,
            link: "dynamic".to_string(),
            memory_limit_mb: None,
            sources: vec!["src".to_string()],
            platforms: vec!["host".to_string()],
            dynamic_libraries: Vec::new(),
            libraries: Vec::new(),
            pre_compile: Vec::new(),
            post_compile: Vec::new(),
            dependencies: std::collections::BTreeMap::new(),
            natives: std::collections::BTreeMap::new(),
        }
    }

    /// Creates the standard default baseline profile for test builds.
    #[must_use]
    pub fn default_test_baseline() -> Self {
        Self {
            name: "test".to_string(),
            entrypoint: Some("TestRunner::test".to_string()),
            opt_level: 0,
            inline_threshold: 8,
            debug_symbols: true,
            is_static: false,
            emit_disassembly: false,
            warnings_as_errors: false,
            verify_checksum: false,
            link: "dynamic".to_string(),
            memory_limit_mb: None,
            sources: vec!["src".to_string(), "tests".to_string()],
            platforms: vec!["host".to_string()],
            dynamic_libraries: Vec::new(),
            libraries: Vec::new(),
            pre_compile: Vec::new(),
            post_compile: Vec::new(),
            dependencies: std::collections::BTreeMap::new(),
            natives: std::collections::BTreeMap::new(),
        }
    }

    /// Resolves all dependencies into structured `ResolvedDependency` records.
    #[must_use]
    pub fn resolved_dependencies(&self) -> Vec<ResolvedDependency> {
        self.dependencies
            .iter()
            .map(|(package_name, specification)| ResolvedDependency {
                package_name: package_name.clone(),
                version: specification.version.clone(),
                build_profile: specification.profile.clone(),
            })
            .collect()
    }
}

fn default_dependency_profile() -> String {
    "release".to_string()
}

/// Representation of a dependency requirement in `[profiles.<profile>.dependencies]`.
///
/// Only strictly-typed table notation is supported:
/// `raylib = { version = "5.5.0", profile = "debug" }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySpec {
    /// Semantic version requirement string (e.g. `"5.5.0"`, `"^1.2.0"`).
    pub version: String,
    /// Target build profile (defaults to `"release"` if omitted in table).
    #[serde(default = "default_dependency_profile")]
    pub profile: String,
}

impl DependencySpec {
    /// Creates a new dependency specification with version and profile.
    #[must_use]
    pub fn new(version: impl Into<String>, profile: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            profile: profile.into(),
        }
    }

    /// Retrieves the semantic version requirement string.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Retrieves the target profile name.
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }
}

/// A fully resolved dependency declaration with target build profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub package_name: String,
    pub version: String,
    pub build_profile: String,
}

/// Unified, user-editable project manifest contract parsed from `manifest.toml` or `manifest.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    /// Unified project identity and metadata.
    pub configuration: ProjectConfiguration,
    /// First-class profiles table mapped as `profiles.<name>`.
    #[serde(default, deserialize_with = "deserialize_manifest_profiles")]
    pub profiles: std::collections::BTreeMap<String, ProfileConfig>,
    /// Verified target platforms embedded into `.slxarc` archives during packaging.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_platforms: Vec<String>,
    /// Cryptographic SHA-256 digests of bundled native shared libraries.
    #[serde(default)]
    pub native_library_checksums: std::collections::HashMap<String, String>,
    /// Declared dynamic shared library dependencies required by this project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dynamic_libraries: Vec<String>,
    /// Optional top-level variable definitions for YAML anchor targets.
    #[serde(default, skip_serializing)]
    pub vars: Option<serde_json::Value>,
    /// Unrecognized top-level vendor fields (e.g. `x-*`).
    #[serde(flatten)]
    pub extra_fields: std::collections::BTreeMap<String, serde_json::Value>,
}

impl ProjectManifest {
    /// Project identifier name convenience accessor.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.configuration.name
    }

    /// Project version string convenience accessor.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.configuration.version
    }

    /// Optional project description convenience accessor.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.configuration.description.as_deref()
    }

    /// Optional software license identifier convenience accessor.
    #[must_use]
    pub fn license(&self) -> Option<&str> {
        self.configuration.license.as_deref()
    }

    /// Project authors list convenience accessor.
    #[must_use]
    pub fn authors(&self) -> &[String] {
        &self.configuration.authors
    }

    /// Canonical main entrypoint symbol.
    #[must_use]
    pub fn main_entrypoint(&self) -> Option<String> {
        self.resolve_profile("release")
            .ok()
            .and_then(|resolved| resolved.entrypoint)
            .or_else(|| Some("Main::main".to_string()))
    }

    /// Canonical test entrypoint symbol.
    #[must_use]
    pub fn test_entrypoint(&self) -> Option<String> {
        self.resolve_profile("test")
            .ok()
            .and_then(|resolved| resolved.entrypoint)
            .or_else(|| Some("TestRunner::test".to_string()))
    }

    /// Parses a `manifest.yaml` string into a strongly-typed `ProjectManifest`.
    ///
    /// Supports YAML anchors/aliases (`&` / `*`), YAML 1.1 merge keys (`<<:`), and an optional
    /// top-level `vars:` section ignored by schema deserialization. Profile inheritance via
    /// `inherits` is explicitly rejected with formatted diagnostics.
    pub fn from_yaml_str(yaml_content: &str) -> Result<Self, EmissionError> {
        Self::from_yaml_str_with_filename(yaml_content, "manifest.yaml")
    }

    /// Parses a YAML manifest with a designated filename for compiler-grade source diagnostics.
    pub fn from_yaml_str_with_filename(
        yaml_content: &str,
        filename: &str,
    ) -> Result<Self, EmissionError> {
        let manifest: Self = match serde_saphyr::from_str(yaml_content) {
            Ok(parsed_manifest) => parsed_manifest,
            Err(parse_error) => {
                let line_option = parse_error
                    .location()
                    .map(|location| location.line() as usize);
                let col_option = parse_error
                    .location()
                    .map(|location| location.column() as usize);
                let breadcrumbs = line_option
                    .map(|target_line| extract_yaml_breadcrumb_path(yaml_content, target_line))
                    .unwrap_or_default();
                let raw_message = parse_error.to_string();
                let clean_message = if let Some(colon_pos) = raw_message.find("error: line ") {
                    if let Some(msg_start) = raw_message[colon_pos..].find(": ") {
                        &raw_message[colon_pos + msg_start + 2..]
                    } else {
                        &raw_message
                    }
                } else {
                    &raw_message
                };
                let first_line = clean_message.lines().next().unwrap_or(clean_message).trim();
                let (enriched_message, field_path) =
                    enrich_manifest_error_message(first_line, &breadcrumbs);

                let rendered_full =
                    parse_error.render_with_options(serde_saphyr::render_options! {
                        source_name: Some(filename),
                    });
                let visual_snippet = rendered_full
                    .lines()
                    .skip_while(|line_str| !line_str.trim_start().starts_with("-->"))
                    .collect::<Vec<_>>()
                    .join("\n");

                return Err(ManifestError::ParseFailed {
                    filename: filename.to_string(),
                    line: line_option,
                    column: col_option,
                    field_path,
                    message: enriched_message,
                    snippet: visual_snippet,
                }
                .into());
            }
        };

        // Validate extra root fields (only x-* vendor extensions and vars allowed)
        for extra_key in manifest.extra_fields.keys() {
            if !extra_key.starts_with("x-") && extra_key != "vars" {
                let (line_num, col_num) = find_key_location(yaml_content, extra_key);
                let snippet = render_source_snippet(
                    yaml_content,
                    filename,
                    line_num,
                    col_num,
                    &format!("unknown field `{extra_key}`"),
                );
                return Err(ManifestError::UnknownRootField {
                    filename: filename.to_string(),
                    field_name: extra_key.clone(),
                    expected: "`configuration`, `profiles`, `supported_platforms`, `native_library_checksums`, `dynamic_libraries`",
                    line: Some(line_num),
                    column: Some(col_num),
                    snippet,
                }
                .into());
            }
        }

        // Validate removed inherits keyword and dependency version constraints
        for (profile_name, profile_config) in &manifest.profiles {
            if profile_config.inherits.is_some() {
                let (line_num, col_num) =
                    find_keyword_in_profile(yaml_content, profile_name, "inherits");
                let snippet = render_source_snippet(
                    yaml_content,
                    filename,
                    line_num,
                    col_num,
                    &format!(
                        "profile '{profile_name}' uses 'inherits', which has been removed; configure each profile explicitly or use YAML anchors (& / *)"
                    ),
                );
                return Err(ManifestError::DeprecatedInherits {
                    filename: filename.to_string(),
                    profile_name: profile_name.clone(),
                    line: Some(line_num),
                    column: Some(col_num),
                    snippet,
                }
                .into());
            }

            for (dep_name, dep_spec) in &profile_config.dependencies {
                if let Err(solver_err) =
                    crate::solver::PackageVersion::parse_constraint(&dep_spec.version)
                {
                    let (line_num, col_num) =
                        find_keyword_in_profile(yaml_content, profile_name, dep_name);
                    let snippet = render_source_snippet(
                        yaml_content,
                        filename,
                        line_num,
                        col_num,
                        &format!(
                            "invalid version constraint `{}` for `{dep_name}`",
                            dep_spec.version
                        ),
                    );
                    return Err(ManifestError::ParseFailed {
                        filename: filename.to_string(),
                        line: Some(line_num),
                        column: Some(col_num),
                        field_path: Some(format!(
                            "profiles.{profile_name}.dependencies.{dep_name}.version"
                        )),
                        message: solver_err.to_string(),
                        snippet,
                    }
                    .into());
                }
            }
        }

        // Validate mandatory release profile
        if !manifest.profiles.contains_key("release") {
            let (line_num, col_num) = find_key_location(yaml_content, "profiles");
            let snippet = render_source_snippet(
                yaml_content,
                filename,
                line_num,
                col_num,
                "manifest must define mandatory profiles.release profile",
            );
            return Err(ManifestError::MissingReleaseProfile {
                filename: filename.to_string(),
                line: Some(line_num),
                column: Some(col_num),
                snippet,
            }
            .into());
        }

        Ok(manifest)
    }

    /// Parses a legacy `manifest.toml` string into a strongly-typed `ProjectManifest`
    /// for backwards compatibility with previously built `.slxarc` archives.
    pub fn from_toml_str(toml_content: &str) -> Result<Self, EmissionError> {
        let mut raw_toml: toml::Value = toml::from_str(toml_content).map_err(|parse_error| {
            let message = if toml_content.contains("[info]")
                || toml_content.contains("[entrypoints]")
                || toml_content.contains("[build]")
                || toml_content.contains("[profile.")
            {
                format!(
                    "legacy manifest sections ([info], [entrypoints], [build], [profile.<name>]) are no longer supported; please migrate to manifest.yaml: {parse_error}"
                )
            } else {
                parse_error.to_string()
            };
            ManifestError::ParseFailed {
                filename: "manifest.toml".to_string(),
                line: None,
                column: None,
                field_path: None,
                message,
                snippet: String::new(),
            }
        })?;

        if let Some(toml::Value::Table(profiles_table)) = raw_toml
            .get_mut("profiles")
            .filter(|value| value.is_table())
        {
            for (_, profile_val) in profiles_table.iter_mut() {
                if let Some(profile_table) = profile_val.as_table_mut() {
                    profile_table.remove("inherits");
                }
            }
        }

        let manifest: Self =
            raw_toml
                .try_into()
                .map_err(|parse_error| ManifestError::ParseFailed {
                    filename: "manifest.toml".to_string(),
                    line: None,
                    column: None,
                    field_path: None,
                    message: parse_error.to_string(),
                    snippet: String::new(),
                })?;

        if !manifest.profiles.contains_key("release") {
            return Err(ManifestError::MissingReleaseProfile {
                filename: "manifest.toml".to_string(),
                line: None,
                column: None,
                snippet: String::new(),
            }
            .into());
        }

        Ok(manifest)
    }

    /// Reads and parses either `manifest.yaml` (preferred) or legacy `manifest.toml` from disk.
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Self, EmissionError> {
        let path_reference = file_path.as_ref();
        let content =
            fs::read_to_string(path_reference).map_err(|io_error| ManifestError::IoError {
                path: path_reference.display().to_string(),
                reason: io_error.to_string(),
            })?;
        let is_toml = path_reference
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext_str| ext_str.eq_ignore_ascii_case("toml"));
        if is_toml {
            Self::from_toml_str(&content)
        } else {
            let filename = path_reference
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("manifest.yaml");
            Self::from_yaml_str_with_filename(&content, filename)
        }
    }

    /// Serializes this manifest to a clean YAML string (`manifest.yaml`).
    pub fn to_yaml_string(&self) -> Result<String, EmissionError> {
        serde_saphyr::to_string(self).map_err(|serialize_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("failed to serialize manifest to yaml: {serialize_error}"),
            }
        })
    }

    /// Serializes this manifest to a TOML string (legacy compatibility).
    pub fn to_toml_string(&self) -> Result<String, EmissionError> {
        toml::to_string_pretty(self).map_err(|serialize_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("failed to serialize manifest to toml: {serialize_error}"),
            }
        })
    }

    /// Resolves the self-contained profile configuration for `target_profile` with zero cross-profile inheritance.
    ///
    /// # Errors
    /// Returns an error if `profiles.release` is missing or if `target_profile` is unknown.
    pub fn resolve_profile(&self, target_profile: &str) -> Result<ResolvedProfile, EmissionError> {
        if !self.profiles.contains_key("release") {
            return Err(EmissionError::ContainerSerializationFailed {
                reason: "manifest must define mandatory profiles.release profile".to_string(),
            });
        }

        if target_profile != "release"
            && target_profile != "debug"
            && target_profile != "test"
            && !self.profiles.contains_key(target_profile)
        {
            return Err(EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Unknown build profile '{target_profile}'; expected 'debug', 'release', 'test', or a profile defined in manifest.yaml"
                ),
            });
        }

        let mut resolved = match target_profile {
            "release" => ResolvedProfile::default_release_baseline(),
            "test" => ResolvedProfile::default_test_baseline(),
            other_profile => ResolvedProfile::default_debug_baseline(other_profile),
        };

        if let Some(user_config) = self.profiles.get(target_profile) {
            if let Some(ref entrypoint) = user_config.entrypoint {
                resolved.entrypoint = Some(entrypoint.clone());
            }
            if let Some(opt_level) = user_config.optimization_level {
                resolved.opt_level = opt_level;
            }
            if let Some(inline_threshold) = user_config.inline_threshold {
                resolved.inline_threshold = inline_threshold;
            }
            if let Some(debug_symbols) = user_config.debug_symbols {
                resolved.debug_symbols = debug_symbols;
            }
            if let Some(is_static) = user_config.is_static {
                resolved.is_static = is_static;
                resolved.link = if is_static {
                    "static".to_string()
                } else {
                    "dynamic".to_string()
                };
            }
            if let Some(ref link) = user_config.link {
                resolved.link = link.clone();
                resolved.is_static = link.eq_ignore_ascii_case("static");
            }
            if let Some(emit_disassembly) = user_config.emit_disassembly {
                resolved.emit_disassembly = emit_disassembly;
            }
            if let Some(warnings_as_errors) = user_config.warnings_as_errors {
                resolved.warnings_as_errors = warnings_as_errors;
            }
            if let Some(verify_checksum) = user_config.verify_checksum {
                resolved.verify_checksum = verify_checksum;
            } else {
                resolved.verify_checksum =
                    !resolved.is_static && target_profile != "debug" && target_profile != "test";
            }
            if let Some(memory_limit_mb) = user_config.memory_limit_mb {
                resolved.memory_limit_mb = Some(memory_limit_mb);
            }
            if let Some(ref step_sources) = user_config.sources {
                resolved.sources = step_sources.clone();
            }
            if let Some(ref step_platforms) = user_config.platforms {
                resolved.platforms = step_platforms.clone();
            }
            resolved.pre_compile = user_config.pre_compile.clone();
            resolved.post_compile = user_config.post_compile.clone();
            resolved.dependencies = user_config.dependencies.clone();
            resolved.natives = user_config.natives.clone();
            if !user_config.libraries.is_empty() {
                resolved.libraries = user_config.libraries.clone();
            } else if !user_config.dynamic_libraries.is_empty() {
                resolved.libraries = user_config.dynamic_libraries.clone();
            }
            if !user_config.dynamic_libraries.is_empty() {
                resolved.dynamic_libraries = user_config.dynamic_libraries.clone();
            } else if !user_config.libraries.is_empty() {
                resolved.dynamic_libraries = user_config.libraries.clone();
            }
        }

        if resolved.libraries.is_empty() && !self.dynamic_libraries.is_empty() {
            resolved.libraries = self.dynamic_libraries.clone();
        }
        if resolved.dynamic_libraries.is_empty() && !self.dynamic_libraries.is_empty() {
            resolved.dynamic_libraries = self.dynamic_libraries.clone();
        }

        for native_name in resolved.natives.keys() {
            if !resolved.dynamic_libraries.contains(native_name) {
                resolved.dynamic_libraries.push(native_name.clone());
            }
        }

        resolved.name = target_profile.to_string();
        Ok(resolved)
    }

    /// Resolves all dependencies for the default `"release"` profile.
    #[must_use]
    pub fn resolved_dependencies(&self) -> Vec<ResolvedDependency> {
        self.resolve_profile("release")
            .map(|profile| profile.resolved_dependencies())
            .unwrap_or_default()
    }

    /// Resolves all dependencies for the specified profile.
    pub fn resolved_dependencies_for_profile(
        &self,
        target_profile: &str,
    ) -> Result<Vec<ResolvedDependency>, EmissionError> {
        let profile = self.resolve_profile(target_profile)?;
        Ok(profile.resolved_dependencies())
    }
}

fn strip_library_extension(name_str: &str) -> &str {
    if let Some(stripped) = name_str.strip_suffix(".so") {
        stripped
    } else if let Some(stripped) = name_str.strip_suffix(".dylib") {
        stripped
    } else if let Some(stripped) = name_str.strip_suffix(".dll") {
        stripped
    } else {
        name_str
    }
}

fn deserialize_profile_dependencies<'de, DeserializerType>(
    deserializer: DeserializerType,
) -> Result<std::collections::BTreeMap<String, DependencySpec>, DeserializerType::Error>
where
    DeserializerType: serde::Deserializer<'de>,
{
    struct DependencyMapVisitor;

    impl<'de> serde::de::Visitor<'de> for DependencyMapVisitor {
        type Value = std::collections::BTreeMap<String, DependencySpec>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a mapping of dependency names to dependency specifications")
        }

        fn visit_map<MapAccessType>(
            self,
            mut access: MapAccessType,
        ) -> Result<Self::Value, MapAccessType::Error>
        where
            MapAccessType: serde::de::MapAccess<'de>,
        {
            let mut dependencies_map = std::collections::BTreeMap::new();
            while let Some(dependency_key) = access.next_key::<String>()? {
                let dependency_value: Option<DependencySpec> = access.next_value()?;
                if let Some(specification) = dependency_value {
                    dependencies_map.insert(dependency_key, specification);
                } else {
                    return Err(serde::de::Error::custom("missing field `version`"));
                }
            }
            Ok(dependencies_map)
        }
    }

    deserializer.deserialize_map(DependencyMapVisitor)
}

fn deserialize_profile_natives<'de, DeserializerType>(
    deserializer: DeserializerType,
) -> Result<std::collections::BTreeMap<String, NativeTargetConfig>, DeserializerType::Error>
where
    DeserializerType: serde::Deserializer<'de>,
{
    struct NativeMapVisitor;

    impl<'de> serde::de::Visitor<'de> for NativeMapVisitor {
        type Value = std::collections::BTreeMap<String, NativeTargetConfig>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a mapping of native target names to native configurations")
        }

        fn visit_map<MapAccessType>(
            self,
            mut access: MapAccessType,
        ) -> Result<Self::Value, MapAccessType::Error>
        where
            MapAccessType: serde::de::MapAccess<'de>,
        {
            let mut natives_map = std::collections::BTreeMap::new();
            while let Some(native_key) = access.next_key::<String>()? {
                let native_value: Option<NativeTargetConfig> = access.next_value()?;
                natives_map.insert(native_key, native_value.unwrap_or_default());
            }
            Ok(natives_map)
        }
    }

    deserializer.deserialize_map(NativeMapVisitor)
}

fn deserialize_manifest_profiles<'de, DeserializerType>(
    deserializer: DeserializerType,
) -> Result<std::collections::BTreeMap<String, ProfileConfig>, DeserializerType::Error>
where
    DeserializerType: serde::Deserializer<'de>,
{
    struct ProfileMapVisitor;

    impl<'de> serde::de::Visitor<'de> for ProfileMapVisitor {
        type Value = std::collections::BTreeMap<String, ProfileConfig>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a mapping of profile names to profile configurations")
        }

        fn visit_map<MapAccessType>(
            self,
            mut access: MapAccessType,
        ) -> Result<Self::Value, MapAccessType::Error>
        where
            MapAccessType: serde::de::MapAccess<'de>,
        {
            let mut profiles_map = std::collections::BTreeMap::new();
            while let Some(profile_key) = access.next_key::<String>()? {
                let profile_value: Option<ProfileConfig> = access.next_value()?;
                profiles_map.insert(profile_key, profile_value.unwrap_or_default());
            }
            Ok(profiles_map)
        }
    }

    deserializer.deserialize_map(ProfileMapVisitor)
}

fn find_key_location(yaml_content: &str, target_key: &str) -> (usize, usize) {
    let key_pattern = format!("{target_key}:");
    for (line_index, line_content) in yaml_content.lines().enumerate() {
        let trimmed = line_content.trim();
        if trimmed.starts_with(&key_pattern) {
            let column = line_content
                .find(target_key)
                .map(|index| index + 1)
                .unwrap_or(1);
            return (line_index + 1, column);
        }
    }
    (1, 1)
}

fn find_keyword_in_profile(
    yaml_content: &str,
    profile_name: &str,
    keyword: &str,
) -> (usize, usize) {
    let profile_pattern = format!("{profile_name}:");
    let keyword_pattern = format!("{keyword}:");
    let mut in_profile = false;
    let mut profile_indent = 0;

    for (line_index, line_content) in yaml_content.lines().enumerate() {
        let trimmed = line_content.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let line_indent = line_content.len() - line_content.trim_start().len();

        if in_profile {
            if line_indent <= profile_indent && !trimmed.is_empty() {
                in_profile = false;
            } else if trimmed.starts_with(&keyword_pattern) {
                let column = line_content
                    .find(keyword)
                    .map(|index| index + 1)
                    .unwrap_or(1);
                return (line_index + 1, column);
            }
        }

        if !in_profile && trimmed.starts_with(&profile_pattern) {
            in_profile = true;
            profile_indent = line_indent;
        }
    }

    find_key_location(yaml_content, keyword)
}

/// Returns `true` if the shared library filename corresponds to a Solix native module (`__native_*` or `lib__native_*`).
#[must_use]
pub fn is_solix_native_module_name(file_name: &str) -> bool {
    let lower_name = file_name.to_ascii_lowercase();
    let stem = strip_library_extension(&lower_name);
    let base = stem.strip_prefix("lib").unwrap_or(stem);
    base.starts_with("__native_")
}

/// Returns `true` if `file_name` matches a declared dynamic library specification,
/// accounting for platform prefixes (`lib`) and extensions (`.so`, `.dylib`, `.dll`).
#[must_use]
pub fn matches_declared_dynamic_library(file_name: &str, declared: &str) -> bool {
    if file_name.eq_ignore_ascii_case(declared) {
        return true;
    }
    let file_stem = strip_library_extension(file_name);
    let declared_stem = strip_library_extension(declared);

    let file_base = file_stem.strip_prefix("lib").unwrap_or(file_stem);
    let declared_base = declared_stem.strip_prefix("lib").unwrap_or(declared_stem);

    file_base.eq_ignore_ascii_case(declared_base)
        || file_name.starts_with(declared)
        || file_stem.starts_with(declared_stem)
}

/// Detailed metadata record for an individual entry inside a `.slxp` container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlxpFileEntry {
    /// Relative path within the archive container.
    pub path: String,
    /// Uncompressed payload size in bytes.
    pub uncompressed_size: u64,
    /// Compressed payload size stored within the archive container.
    pub compressed_size: u64,
    /// Whether this entry represents a directory structure.
    pub is_directory: bool,
}

/// Inspection summary containing decoded contents of a `.slxp` package container.
#[derive(Debug, Clone)]
pub struct SlxpInspection {
    /// Project manifest metadata, if bundled at the package root.
    pub manifest: Option<ProjectManifest>,
    /// Full list of file entries contained within the package.
    pub file_entries: Vec<SlxpFileEntry>,
    /// Deserialized Solix bytecode binaries embedded inside the package.
    pub binaries: Vec<(String, BinaryProgram)>,
    /// List of native dynamic shared libraries identified within `bin/` or `lib/`.
    pub native_libraries: Vec<String>,
}

/// Fully extracted and loaded `.slxp` package ready for virtual machine execution.
#[derive(Debug, Clone)]
pub struct LoadedSlxp {
    /// Project manifest metadata bundled with the package.
    pub manifest: Option<ProjectManifest>,
    /// Primary compiled bytecode binaries found inside `binaries.slxb` or root.
    pub binaries: Vec<BinaryProgram>,
    /// Discovered host native shared library file paths extracted to the runtime cache.
    pub native_libraries: Vec<PathBuf>,
    /// Secondary dependency `.slxb` binaries found inside `lib/` or `bin/`.
    pub secondary_binaries: Vec<BinaryProgram>,
    /// All files extracted to the cache directory.
    pub extracted_files: Vec<PathBuf>,
    /// Static resource files extracted from `resources/`.
    pub resource_files: Vec<PathBuf>,
    /// Total byte length of all extracted static resource files.
    pub resource_total_bytes: u64,
}

/// Backwards compatibility aliases for `.slxarc` terminology.
pub type SlxarcFileEntry = SlxpFileEntry;
pub type SlxarcInspection = SlxpInspection;
pub type LoadedSlxarc = LoadedSlxp;
pub type SlxarcArchive = SlxpPackage;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractedArchiveCacheMeta {
    archive_size_bytes: u64,
    archive_mtime_ns: u128,
    manifest: Option<ProjectManifest>,
    binary_cache_files: Vec<PathBuf>,
    secondary_binary_cache_files: Vec<PathBuf>,
    native_libraries: Vec<PathBuf>,
    extracted_files: Vec<PathBuf>,
    resource_files: Vec<PathBuf>,
    resource_total_bytes: u64,
}

fn archive_metadata_fingerprint(archive_path: &Path) -> Option<(u64, u128)> {
    let metadata = fs::metadata(archive_path).ok()?;
    let modified_ns = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some((metadata.len(), modified_ns))
}

fn archive_sidecar_prefix(archive_path: &Path) -> String {
    let canonical = fs::canonicalize(archive_path).unwrap_or_else(|_| archive_path.to_path_buf());
    let digest = crate::hash::sha256_bytes(canonical.to_string_lossy().as_bytes());
    let hex_full = crate::hash::format_hex_digest(&digest);
    hex_full[..16].to_string()
}

/// Package container packager, extractor, and inspector for `.slxp` distributions.
pub struct SlxpPackage;

impl SlxpPackage {
    /// Packages an entire directory tree into a deterministic `.slxp` package archive.
    pub fn pack_directory(
        staging_directory: &Path,
        output_archive_path: &Path,
    ) -> Result<(), EmissionError> {
        if let Some(parent_dir) = output_archive_path.parent() {
            fs::create_dir_all(parent_dir).map_err(|io_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!(
                        "failed to create destination directory '{}': {io_error}",
                        parent_dir.display()
                    ),
                }
            })?;
        }

        let archive_file = File::create(output_archive_path).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to create archive file '{}': {io_error}",
                    output_archive_path.display()
                ),
            }
        })?;

        let mut zip_writer = ZipWriter::new(archive_file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut collected_entries = Vec::<PathBuf>::new();
        collect_directory_entries(staging_directory, &mut collected_entries)?;
        collected_entries.sort();

        for entry_path in collected_entries {
            let relative_path =
                entry_path
                    .strip_prefix(staging_directory)
                    .map_err(|strip_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to determine relative path: {strip_error}"),
                    })?;

            let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
            if relative_path_str.is_empty() {
                continue;
            }

            if entry_path.is_dir() {
                let dir_path_str = if relative_path_str.ends_with('/') {
                    relative_path_str
                } else {
                    format!("{relative_path_str}/")
                };
                zip_writer
                    .add_directory(&dir_path_str, options)
                    .map_err(|zip_error| EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to write directory entry '{dir_path_str}': {zip_error}"
                        ),
                    })?;
            } else {
                zip_writer
                    .start_file(&relative_path_str, options)
                    .map_err(|zip_error| EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to write file entry '{relative_path_str}': {zip_error}"
                        ),
                    })?;

                let file_bytes = fs::read(&entry_path).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to read staged file '{}': {io_error}",
                            entry_path.display()
                        ),
                    }
                })?;
                zip_writer.write_all(&file_bytes).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to compress file '{}': {io_error}",
                            relative_path_str
                        ),
                    }
                })?;
            }
        }

        zip_writer
            .finish()
            .map_err(|zip_error| EmissionError::ContainerSerializationFailed {
                reason: format!("failed to finalize .slxarc archive: {zip_error}"),
            })?;

        Ok(())
    }

    /// Inspects the contents of a `.slxarc` container without modifying disk state.
    pub fn inspect(archive_path: &Path) -> Result<SlxarcInspection, EmissionError> {
        let archive_file = File::open(archive_path).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to open archive '{}': {io_error}",
                    archive_path.display()
                ),
            }
        })?;

        let mut zip_reader = ZipArchive::new(archive_file).map_err(|zip_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to read zip header in '{}': {zip_error}",
                    archive_path.display()
                ),
            }
        })?;

        let mut manifest_option = None;
        let mut file_entries = Vec::new();
        let mut binaries = Vec::new();
        let mut native_libraries = Vec::new();

        for index in 0..zip_reader.len() {
            let mut file_entry = zip_reader.by_index(index).map_err(|zip_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!("failed to read archive entry at index {index}: {zip_error}"),
                }
            })?;

            let entry_name = read_zip_entry_name(&file_entry, index)?;
            let is_directory = file_entry.is_dir();
            let uncompressed_size = file_entry.size();
            let compressed_size = file_entry.compressed_size();

            file_entries.push(SlxarcFileEntry {
                path: entry_name.clone(),
                uncompressed_size,
                compressed_size,
                is_directory,
            });

            if !is_directory {
                if entry_name == "manifest.yaml" || entry_name == "manifest.yml" {
                    let mut manifest_bytes = Vec::new();
                    if file_entry.read_to_end(&mut manifest_bytes).is_ok()
                        && let Ok(manifest_text) = std::str::from_utf8(&manifest_bytes)
                        && let Ok(parsed_manifest) = ProjectManifest::from_yaml_str(manifest_text)
                    {
                        manifest_option = Some(parsed_manifest);
                    }
                } else if entry_name == "manifest.toml" && manifest_option.is_none() {
                    let mut manifest_bytes = Vec::new();
                    if file_entry.read_to_end(&mut manifest_bytes).is_ok()
                        && let Ok(manifest_text) = std::str::from_utf8(&manifest_bytes)
                        && let Ok(parsed_manifest) = ProjectManifest::from_toml_str(manifest_text)
                    {
                        manifest_option = Some(parsed_manifest);
                    }
                } else if entry_name.ends_with(".slxb") {
                    let mut binary_bytes = Vec::new();
                    if file_entry.read_to_end(&mut binary_bytes).is_ok()
                        && let Ok(deserialized_binary) = BinaryProgram::from_bytes(&binary_bytes)
                    {
                        binaries.push((entry_name.clone(), deserialized_binary));
                    }
                } else if entry_name.ends_with(".so")
                    || entry_name.ends_with(".dll")
                    || entry_name.ends_with(".dylib")
                {
                    native_libraries.push(entry_name);
                }
            }
        }

        Ok(SlxarcInspection {
            manifest: manifest_option,
            file_entries,
            binaries,
            native_libraries,
        })
    }
}

/// Extracts validated UTF-8 archive entry name in `zip` v9+ safely.
fn read_zip_entry_name(
    file_entry: &zip::read::ZipFile<'_, File>,
    index: usize,
) -> Result<String, EmissionError> {
    file_entry
        .name()
        .map(std::borrow::Cow::into_owned)
        .map_err(|zip_error| EmissionError::ContainerSerializationFailed {
            reason: format!("invalid archive entry filename at index {index}: {zip_error}"),
        })
}

/// Validates and sanitizes a `.slxarc` ZIP archive member path against Zip-Slip traversal.
pub fn validate_archive_relative_path(entry_name: &str) -> Result<PathBuf, EmissionError> {
    if entry_name.is_empty()
        || entry_name.starts_with('/')
        || entry_name.contains('\\')
        || entry_name.contains('\0')
    {
        return Err(EmissionError::ContainerSerializationFailed {
            reason: format!("unsafe archive entry path rejected (path traversal): '{entry_name}'"),
        });
    }

    let mut sanitized = PathBuf::new();
    for segment in entry_name.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains(':') {
            return Err(EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "unsafe archive entry path rejected (parent/drive traversal): '{entry_name}'"
                ),
            });
        }
        sanitized.push(segment);
    }

    if sanitized.as_os_str().is_empty() {
        return Err(EmissionError::ContainerSerializationFailed {
            reason: format!("empty archive entry path rejected: '{entry_name}'"),
        });
    }

    Ok(sanitized)
}

impl SlxpPackage {
    /// Extracts host native shared libraries and loads bytecode binaries for VM execution.
    pub fn extract_and_load(
        archive_path: &Path,
        extract_cache_directory: &Path,
    ) -> Result<LoadedSlxp, EmissionError> {
        let archive_fingerprint = archive_metadata_fingerprint(archive_path);
        let sidecar_prefix = archive_sidecar_prefix(archive_path);
        let sidecar_meta_path =
            extract_cache_directory.join(format!(".slxp_meta_{sidecar_prefix}.bin"));

        if let Some((archive_size_bytes, archive_mtime_ns)) = archive_fingerprint
            && let Ok(meta_bytes) = fs::read(&sidecar_meta_path)
            && let Ok(cached_meta) =
                serde_json::from_slice::<ExtractedArchiveCacheMeta>(&meta_bytes)
            && cached_meta.archive_size_bytes == archive_size_bytes
            && cached_meta.archive_mtime_ns == archive_mtime_ns
            && !cached_meta.binary_cache_files.is_empty()
            && cached_meta
                .native_libraries
                .iter()
                .all(|lib_path| lib_path.is_file())
            && cached_meta
                .resource_files
                .iter()
                .all(|res_path| res_path.is_file())
        {
            let mut cached_binaries = Vec::with_capacity(cached_meta.binary_cache_files.len());
            let mut cached_secondary =
                Vec::with_capacity(cached_meta.secondary_binary_cache_files.len());
            let mut cache_hit_ok = true;

            for bin_path in &cached_meta.binary_cache_files {
                if let Ok(bin_bytes) = fs::read(bin_path)
                    && let Ok(decoded_bin) = BinaryProgram::from_bytes(&bin_bytes)
                {
                    cached_binaries.push(decoded_bin);
                } else {
                    cache_hit_ok = false;
                    break;
                }
            }

            if cache_hit_ok {
                for sec_path in &cached_meta.secondary_binary_cache_files {
                    if let Ok(sec_bytes) = fs::read(sec_path)
                        && let Ok(decoded_sec) = BinaryProgram::from_bytes(&sec_bytes)
                    {
                        cached_secondary.push(decoded_sec);
                    } else {
                        cache_hit_ok = false;
                        break;
                    }
                }
            }

            if cache_hit_ok {
                return Ok(LoadedSlxp {
                    manifest: cached_meta.manifest,
                    binaries: cached_binaries,
                    native_libraries: cached_meta.native_libraries,
                    secondary_binaries: cached_secondary,
                    extracted_files: cached_meta.extracted_files,
                    resource_files: cached_meta.resource_files,
                    resource_total_bytes: cached_meta.resource_total_bytes,
                });
            }
        }

        let archive_file = File::open(archive_path).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to open archive '{}': {io_error}",
                    archive_path.display()
                ),
            }
        })?;

        let mut zip_reader = ZipArchive::new(archive_file).map_err(|zip_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to decode archive '{}': {zip_error}",
                    archive_path.display()
                ),
            }
        })?;
        zip_reader
            .validate_header_consistency()
            .map_err(|zip_error| EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "archive header consistency check failed for '{}': {zip_error}",
                    archive_path.display()
                ),
            })?;

        fs::create_dir_all(extract_cache_directory).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to create cache directory '{}': {io_error}",
                    extract_cache_directory.display()
                ),
            }
        })?;

        #[cfg(target_os = "windows")]
        let host_extension = "dll";
        #[cfg(target_os = "macos")]
        let host_extension = "dylib";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let host_extension = "so";

        let mut manifest_option = None;
        let mut binaries = Vec::new();
        let mut pending_bin_slxb: Vec<(String, Vec<u8>)> = Vec::new();
        let mut seen_binary_hashes: std::collections::HashSet<[u8; 32]> =
            std::collections::HashSet::new();
        let mut secondary_binaries = Vec::new();
        let mut native_libraries = Vec::new();
        let mut native_entry_names: std::collections::HashMap<PathBuf, String> =
            std::collections::HashMap::new();
        let mut extracted_files = Vec::new();
        let mut native_entry_priorities: std::collections::HashMap<std::ffi::OsString, u8> =
            std::collections::HashMap::new();
        let mut resource_files = Vec::new();
        let mut resource_total_bytes = 0u64;
        let mut candidate_native_libraries = Vec::new();

        for index in 0..zip_reader.len() {
            let mut file_entry = zip_reader.by_index(index).map_err(|zip_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!("failed to read entry at index {index}: {zip_error}"),
                }
            })?;

            let entry_name = read_zip_entry_name(&file_entry, index)?;
            let safe_entry_rel_path = validate_archive_relative_path(&entry_name)?;
            if file_entry.is_dir() {
                continue;
            }

            if entry_name == "manifest.yaml" || entry_name == "manifest.yml" {
                let mut manifest_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut manifest_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read {entry_name} bytes: {io_error}"),
                    })?;
                let manifest_text = std::str::from_utf8(&manifest_bytes).map_err(|utf8_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!("invalid utf-8 in {entry_name}: {utf8_error}"),
                    }
                })?;
                manifest_option = Some(ProjectManifest::from_yaml_str(manifest_text)?);
            } else if entry_name == "manifest.toml" && manifest_option.is_none() {
                let mut manifest_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut manifest_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read manifest.toml bytes: {io_error}"),
                    })?;
                let manifest_text = std::str::from_utf8(&manifest_bytes).map_err(|utf8_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!("invalid utf-8 in manifest.toml: {utf8_error}"),
                    }
                })?;
                manifest_option = Some(ProjectManifest::from_toml_str(manifest_text)?);
            } else if entry_name == "binaries.slxb" {
                let mut binary_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut binary_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read root binary '{entry_name}': {io_error}"),
                    })?;
                seen_binary_hashes.insert(crate::hash::sha256_bytes(&binary_bytes));
                let binary = BinaryProgram::from_bytes(&binary_bytes).map_err(|deser_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to decode root binary '{entry_name}': {deser_error}"
                        ),
                    }
                })?;
                binaries.insert(0, binary);
            } else if entry_name.starts_with("bin/") && entry_name.ends_with(".slxb") {
                let mut binary_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut binary_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read binary '{entry_name}': {io_error}"),
                    })?;
                pending_bin_slxb.push((entry_name, binary_bytes));
            } else if entry_name.starts_with("lib/") && entry_name.ends_with(".slxb") {
                let mut binary_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut binary_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to read dependency binary '{entry_name}': {io_error}"
                        ),
                    })?;
                let binary = BinaryProgram::from_bytes(&binary_bytes).map_err(|deser_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to decode dependency binary '{entry_name}': {deser_error}"
                        ),
                    }
                })?;
                secondary_binaries.push(binary);
            } else if (entry_name.starts_with("bin/") || entry_name.starts_with("lib/"))
                && entry_name.ends_with(host_extension)
            {
                let relative_parts: Vec<&str> =
                    if let Some(stripped) = entry_name.strip_prefix("bin/") {
                        stripped.split('/').collect()
                    } else if let Some(stripped) = entry_name.strip_prefix("lib/") {
                        stripped.split('/').collect()
                    } else {
                        Vec::new()
                    };
                let is_qualified_subdir = relative_parts.len() > 1;
                let entry_priority = if is_qualified_subdir {
                    native_target_subdir_priority(relative_parts[0])
                } else {
                    1
                };

                if entry_priority == 0 {
                    continue;
                }

                // Extract host native library atomically, preferring exact ABI > compatible sibling ABI > unqualified `bin/<file>` or `lib/<file>`.
                let file_name_os = Path::new(&entry_name)
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new("native.so"));
                let existing_priority = native_entry_priorities
                    .get(file_name_os)
                    .copied()
                    .unwrap_or(0);
                if existing_priority >= entry_priority {
                    continue;
                }
                let target_path = extract_cache_directory.join(file_name_os);

                let mut library_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut library_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read native library '{entry_name}': {io_error}"),
                    })?;

                let expected_hash = crate::hash::sha256_hex(&library_bytes);
                let already_valid = target_path.exists()
                    && crate::hash::sha256_file(&target_path)
                        .is_ok_and(|existing_hash| existing_hash == expected_hash);

                if !already_valid {
                    static TEMP_EXTRACTION_SEQ: std::sync::atomic::AtomicU64 =
                        std::sync::atomic::AtomicU64::new(0);
                    let seq_id =
                        TEMP_EXTRACTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let file_name_lossy = file_name_os.to_string_lossy();
                    let temp_path = extract_cache_directory.join(format!(
                        ".{}.{}.{}.tmp",
                        file_name_lossy,
                        std::process::id(),
                        seq_id
                    ));
                    let mut out_file = File::create(&temp_path).map_err(|io_error| {
                        EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "failed to create temp native library '{}': {io_error}",
                                temp_path.display()
                            ),
                        }
                    })?;
                    out_file.write_all(&library_bytes).map_err(|io_error| {
                        let _ = fs::remove_file(&temp_path);
                        EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "failed to write native library '{}': {io_error}",
                                temp_path.display()
                            ),
                        }
                    })?;
                    drop(out_file);

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755));
                    }

                    if let Err(rename_error) = fs::rename(&temp_path, &target_path) {
                        let _ = fs::remove_file(&temp_path);
                        let raced_valid = target_path.exists()
                            && crate::hash::sha256_file(&target_path)
                                .is_ok_and(|existing_hash| existing_hash == expected_hash);
                        if !raced_valid {
                            return Err(EmissionError::ContainerSerializationFailed {
                                reason: format!(
                                    "failed to finalize native library '{}': {rename_error}",
                                    target_path.display()
                                ),
                            });
                        }
                    }
                }

                native_entry_priorities.insert(file_name_os.to_os_string(), entry_priority);
                native_entry_names.insert(target_path.clone(), entry_name.clone());

                if !candidate_native_libraries.contains(&target_path) {
                    candidate_native_libraries.push(target_path.clone());
                }
                if !extracted_files.contains(&target_path) {
                    extracted_files.push(target_path);
                }
            } else if entry_name.starts_with("resources/") {
                let target_path = extract_cache_directory.join(&safe_entry_rel_path);
                if let Some(parent_dir) = target_path.parent() {
                    let _ = fs::create_dir_all(parent_dir);
                }
                let mut out_file = File::create(&target_path).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to extract resource '{}': {io_error}",
                            target_path.display()
                        ),
                    }
                })?;
                let mut resource_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut resource_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read resource '{entry_name}': {io_error}"),
                    })?;
                out_file.write_all(&resource_bytes).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to write resource '{}': {io_error}",
                            target_path.display()
                        ),
                    }
                })?;
                resource_total_bytes += resource_bytes.len() as u64;
                resource_files.push(target_path.clone());
                extracted_files.push(target_path);
            }
        }

        // Enforce recorded archive platform compatibility before loading or trusting binaries
        if let Some(ref manifest) = manifest_option {
            validate_archive_host_compatibility(manifest, &archive_path.display().to_string())?;
        }

        // Verify native library checksums against manifest and binary metadata
        let empty_checksums = std::collections::HashMap::new();
        let manifest_checksums = manifest_option
            .as_ref()
            .map(|manifest| &manifest.native_library_checksums)
            .unwrap_or(&empty_checksums);
        let binary_checksums = binaries
            .first()
            .map(|binary| &binary.native_library_checksums)
            .unwrap_or(&empty_checksums);

        for extracted_file_path in &extracted_files {
            let file_name = extracted_file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let preferred_entry = native_entry_names
                .get(extracted_file_path)
                .map(String::as_str);
            let expected_checksum =
                resolve_expected_native_checksum(manifest_checksums, file_name, preferred_entry)
                    .or_else(|| {
                        resolve_expected_native_checksum(
                            binary_checksums,
                            file_name,
                            preferred_entry,
                        )
                    });

            if let Some(expected) = expected_checksum {
                let actual = crate::hash::sha256_file(extracted_file_path).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to hash extracted native library '{}': {io_error}",
                            extracted_file_path.display()
                        ),
                    }
                })?;
                if actual != expected {
                    return Err(EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "native library checksum verification failed for '{}': expected {expected}, found {actual}",
                            extracted_file_path.display()
                        ),
                    });
                }
            }
        }

        for (entry_name, binary_bytes) in pending_bin_slxb {
            // `solix build` writes `bin/<pkg>.slxb` as a legacy copy alongside root `binaries.slxb`;
            // deduplicating by SHA-256 preserves legacy-only archives while preventing duplicate runtime loads.
            if !seen_binary_hashes.insert(crate::hash::sha256_bytes(&binary_bytes)) {
                continue;
            }
            let binary = BinaryProgram::from_bytes(&binary_bytes).map_err(|deser_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!("failed to decode binary '{entry_name}': {deser_error}"),
                }
            })?;
            binaries.push(binary);
        }

        // If no bin/*.slxb was discovered, probe root *.slxb entries as fallback
        if binaries.is_empty() {
            let mut second_reader =
                ZipArchive::new(File::open(archive_path).map_err(|io_err| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!("re-open failed: {io_err}"),
                    }
                })?)
                .map_err(|zip_err| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!("re-decode failed: {zip_err}"),
                    }
                })?;

            for index in 0..second_reader.len() {
                let mut file_entry = second_reader.by_index(index).map_err(|zip_err| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read entry at index {index}: {zip_err}"),
                    }
                })?;
                let entry_name = read_zip_entry_name(&file_entry, index)?;
                if entry_name.ends_with(".slxb")
                    && !entry_name.starts_with("lib/")
                    && !entry_name.starts_with("bin/")
                {
                    let mut binary_bytes = Vec::new();
                    file_entry
                        .read_to_end(&mut binary_bytes)
                        .map_err(|io_err| EmissionError::ContainerSerializationFailed {
                            reason: format!("failed to read root binary: {io_err}"),
                        })?;
                    let binary = BinaryProgram::from_bytes(&binary_bytes).map_err(|deser_err| {
                        EmissionError::ContainerSerializationFailed {
                            reason: format!("failed to decode root binary: {deser_err}"),
                        }
                    })?;
                    binaries.push(binary);
                }
            }
        }

        for candidate_path in candidate_native_libraries {
            let file_name_lossy = candidate_path
                .file_name()
                .map(|name_os| name_os.to_string_lossy())
                .unwrap_or_default();
            let is_declared =
                manifest_option.as_ref().is_some_and(|manifest| {
                    manifest.dynamic_libraries.iter().any(|declared| {
                        matches_declared_dynamic_library(&file_name_lossy, declared)
                    })
                }) || binaries.iter().any(|binary| {
                    binary.dynamic_libraries.iter().any(|declared| {
                        matches_declared_dynamic_library(&file_name_lossy, declared)
                    })
                });
            let is_solix_native = is_solix_native_module_name(&file_name_lossy) || is_declared;
            if is_solix_native && !native_libraries.contains(&candidate_path) {
                native_libraries.push(candidate_path);
            }
        }

        if let Some((archive_size_bytes, archive_mtime_ns)) = archive_fingerprint
            && !binaries.is_empty()
        {
            let mut binary_cache_files = Vec::with_capacity(binaries.len());
            let mut secondary_binary_cache_files = Vec::with_capacity(secondary_binaries.len());
            let mut cached_ok = true;

            for (bin_idx, binary_prog) in binaries.iter().enumerate() {
                let bin_cache_path = extract_cache_directory
                    .join(format!(".slxb_bin_{sidecar_prefix}_{bin_idx}.slxb"));
                if let Ok(raw_bytes) = binary_prog.try_to_bytes()
                    && fs::write(&bin_cache_path, raw_bytes).is_ok()
                {
                    binary_cache_files.push(bin_cache_path);
                } else {
                    cached_ok = false;
                    break;
                }
            }
            if cached_ok {
                for (sec_idx, sec_prog) in secondary_binaries.iter().enumerate() {
                    let sec_cache_path = extract_cache_directory
                        .join(format!(".slxb_sec_{sidecar_prefix}_{sec_idx}.slxb"));
                    if let Ok(raw_bytes) = sec_prog.try_to_bytes()
                        && fs::write(&sec_cache_path, raw_bytes).is_ok()
                    {
                        secondary_binary_cache_files.push(sec_cache_path);
                    } else {
                        cached_ok = false;
                        break;
                    }
                }
            }
            if cached_ok {
                let sidecar_meta = ExtractedArchiveCacheMeta {
                    archive_size_bytes,
                    archive_mtime_ns,
                    manifest: manifest_option.clone(),
                    binary_cache_files,
                    secondary_binary_cache_files,
                    native_libraries: native_libraries.clone(),
                    extracted_files: extracted_files.clone(),
                    resource_files: resource_files.clone(),
                    resource_total_bytes,
                };
                if let Ok(encoded_meta) = serde_json::to_vec(&sidecar_meta) {
                    let _ = fs::write(&sidecar_meta_path, encoded_meta);
                }
            }
        }

        Ok(LoadedSlxp {
            manifest: manifest_option,
            binaries,
            native_libraries,
            secondary_binaries,
            extracted_files,
            resource_files,
            resource_total_bytes,
        })
    }
}

fn collect_directory_entries(
    directory: &Path,
    collected_entries: &mut Vec<PathBuf>,
) -> Result<(), EmissionError> {
    if !directory.exists() {
        return Ok(());
    }
    let read_dir = fs::read_dir(directory).map_err(|io_error| {
        EmissionError::ContainerSerializationFailed {
            reason: format!(
                "failed to read directory '{}': {io_error}",
                directory.display()
            ),
        }
    })?;

    for entry_result in read_dir {
        let dir_entry =
            entry_result.map_err(|io_error| EmissionError::ContainerSerializationFailed {
                reason: format!("failed to read directory entry: {io_error}"),
            })?;
        let entry_path = dir_entry.path();
        collected_entries.push(entry_path.clone());
        if entry_path.is_dir() {
            collect_directory_entries(&entry_path, collected_entries)?;
        }
    }

    Ok(())
}

/// Checks whether a `lib/<subdirectory>/` archive directory matches the active host OS and CPU architecture.
/// Returns `true` if `subdirectory` under `lib/` matches the current host OS, architecture,
/// and (on Windows) ABI toolchain (`-pc-windows-msvc` vs `-pc-windows-gnu`).
#[must_use]
pub fn matches_native_target_subdir(subdirectory: &str) -> bool {
    let host_os = std::env::consts::OS;
    let host_arch = std::env::consts::ARCH;
    let short_pair = format!("{host_os}-{host_arch}");

    if subdirectory == short_pair || subdirectory == host_os || subdirectory == "host" {
        return true;
    }

    let triple_matches_os = match host_os {
        "linux" => subdirectory.contains("-linux-"),
        "macos" => subdirectory.contains("-apple-darwin"),
        "windows" => {
            if cfg!(target_env = "msvc") {
                subdirectory.contains("-pc-windows-msvc")
            } else if cfg!(target_env = "gnu") {
                subdirectory.contains("-pc-windows-gnu")
            } else {
                subdirectory.contains("-pc-windows-")
            }
        }
        _ => false,
    };

    if !triple_matches_os {
        return false;
    }

    match host_arch {
        "x86_64" => subdirectory.starts_with("x86_64-"),
        "aarch64" => subdirectory.starts_with("aarch64-") || subdirectory.starts_with("arm64-"),
        _ => subdirectory.starts_with(host_arch),
    }
}

/// Scores how well a `lib/<subdirectory>/` archive directory matches the active host platform:
/// - `3`: Exact host OS, CPU architecture, and ABI (`matches_native_target_subdir`).
/// - `2`: Compatible Windows sibling ABI for C-ABI `cdylib` fallback (`-pc-windows-gnu` on MSVC or vice versa).
/// - `0`: Incompatible platform or architecture.
#[must_use]
pub fn native_target_subdir_priority(subdirectory: &str) -> u8 {
    if matches_native_target_subdir(subdirectory) {
        return 3;
    }

    if std::env::consts::OS == "windows" && subdirectory.contains("-pc-windows-") {
        let host_arch = std::env::consts::ARCH;
        let arch_matches = match host_arch {
            "x86_64" => subdirectory.starts_with("x86_64-"),
            "aarch64" => subdirectory.starts_with("aarch64-") || subdirectory.starts_with("arm64-"),
            _ => subdirectory.starts_with(host_arch),
        };
        if arch_matches {
            return 2;
        }
    }

    0
}

/// Returns `true` when an archive checksum key under `bin/` or `lib/` targets the active host platform
/// (either unqualified `bin/<file>`, `lib/<file>` or `bin/<matching-subdir>/<file>`, `lib/<matching-subdir>/<file>`).
#[must_use]
pub fn native_checksum_key_applies_to_host(checksum_key: &str) -> bool {
    let relative = if let Some(stripped) = checksum_key.strip_prefix("bin/") {
        stripped
    } else if let Some(stripped) = checksum_key.strip_prefix("lib/") {
        stripped
    } else {
        return !checksum_key.contains('/');
    };
    match relative.split_once('/') {
        Some((subdir, _)) => native_target_subdir_priority(subdir) > 0,
        None => true,
    }
}

/// Resolves the expected SHA-256 digest for `file_name` from a `native_library_checksums` map,
/// preferring `preferred_entry_name`, exact-ABI host entries (`priority = 3`), and compatible
/// sibling-ABI entries (`priority = 2`) before falling back to `bin/<file_name>` or `lib/<file_name>`.
#[must_use]
pub fn resolve_expected_native_checksum<'a>(
    checksums: &'a std::collections::HashMap<String, String>,
    file_name: &str,
    preferred_entry_name: Option<&str>,
) -> Option<&'a str> {
    if let Some(entry_name) = preferred_entry_name
        && let Some(hash) = checksums.get(entry_name)
    {
        return Some(hash.as_str());
    }
    let suffix = format!("/{file_name}");
    let mut best_match: Option<(u8, &'a str)> = None;
    for (key, hash) in checksums {
        let stripped = key
            .strip_prefix("bin/")
            .or_else(|| key.strip_prefix("lib/"));
        if let Some(rel) = stripped
            && let Some(subdir) = rel.strip_suffix(&suffix)
            && !subdir.contains('/')
        {
            let priority = native_target_subdir_priority(subdir);
            if priority > 0 && best_match.is_none_or(|(best_prio, _)| priority > best_prio) {
                best_match = Some((priority, hash.as_str()));
            }
        }
    }
    if let Some((_, hash)) = best_match {
        return Some(hash);
    }
    let unqualified_bin = format!("bin/{file_name}");
    let unqualified_lib = format!("lib/{file_name}");
    checksums
        .get(&unqualified_bin)
        .or_else(|| checksums.get(&unqualified_lib))
        .or_else(|| checksums.get(file_name))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse_minimal_valid() {
        let yaml_sample = r#"
vars:
  common_ver: &app_ver "1.2.3"

configuration:
  name: my_app
  version: *app_ver
  description: A sample Solix application
  license: Apache-2.0
  authors:
    - "Jane Doe <jane@example.com>"

profiles:
  release:
    entrypoint: Main::main
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    sources:
      - src
    platforms:
      - linux
      - macos
      - windows
    pre_compile:
      - echo pre
    post_compile:
      - echo post
    natives:
      math_native:
        profile: release
        platforms: [linux, windows]
        flags: [-DFOO=ON]

  test:
    entrypoint: TestRunner::main
    sources:
      - src
      - tests
"#;
        let manifest = ProjectManifest::from_yaml_str(yaml_sample).expect("valid yaml");
        assert_eq!(manifest.configuration.name, "my_app");
        assert_eq!(manifest.configuration.version, "1.2.3");
        assert_eq!(
            manifest.configuration.description.as_deref(),
            Some("A sample Solix application")
        );
        assert_eq!(
            manifest.configuration.license.as_deref(),
            Some("Apache-2.0")
        );
        assert_eq!(
            manifest.configuration.authors,
            vec!["Jane Doe <jane@example.com>"]
        );
        assert_eq!(manifest.main_entrypoint().as_deref(), Some("Main::main"));
        assert_eq!(
            manifest.test_entrypoint().as_deref(),
            Some("TestRunner::main")
        );

        let resolved_release = manifest
            .resolve_profile("release")
            .expect("release profile");
        assert_eq!(resolved_release.opt_level, 3);
        assert_eq!(resolved_release.inline_threshold, 16);
        assert_eq!(resolved_release.sources, vec!["src"]);
        assert_eq!(resolved_release.pre_compile, vec!["echo pre"]);
        assert_eq!(resolved_release.post_compile, vec!["echo post"]);
        assert_eq!(
            expand_platform_list(&resolved_release.platforms),
            vec![
                "linux-x86_64",
                "linux-aarch64",
                "macos-x86_64",
                "macos-aarch64",
                "windows-x86_64",
            ]
        );
        assert_eq!(
            expand_platforms_to_triples(&["linux".to_string(), "macos".to_string()]),
            vec![
                "x86_64-unknown-linux-gnu",
                "aarch64-unknown-linux-gnu",
                "x86_64-apple-darwin",
                "aarch64-apple-darwin",
            ]
        );

        let resolved_test = manifest.resolve_profile("test").expect("test profile");
        assert_eq!(resolved_test.sources, vec!["src", "tests"]);
        assert!(resolved_test.natives.is_empty());

        let rejected_inherits = r#"
configuration:
  name: legacy_yaml
profiles:
  release: {}
  debug:
    inherits: release
"#;
        assert!(
            ProjectManifest::from_yaml_str(rejected_inherits).is_err(),
            "inherits in manifest.yaml must be rejected"
        );
    }

    #[test]
    fn test_manifest_yaml_missing_version_in_dependency_has_precise_diagnostics() {
        let malformed_yaml = r#"configuration:
  name: test_app
  version: "1.0.0"
profiles:
  release:
    entrypoint: Main::main
  debug:
    dependencies:
      raylib:
        profile: debug
"#;
        let parse_result = ProjectManifest::from_yaml_str(malformed_yaml);
        assert!(parse_result.is_err());
        let error = parse_result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains(
                "missing required field `version` for dependency 'raylib' in profile 'debug'"
            ),
            "Error was: {error_message}"
        );
        assert!(
            error_message.contains("manifest.yaml:10:9")
                || error_message.contains("manifest.yaml:10:"),
            "Expected line 10 location in: {error_message}"
        );
        assert!(
            error_message.contains("raylib:"),
            "Expected snippet context in: {error_message}"
        );
    }

    #[test]
    fn test_manifest_yaml_unknown_field_in_profile_has_precise_diagnostics() {
        let malformed_yaml = r#"configuration:
  name: test_app
  version: "1.0.0"
profiles:
  release:
    invalid_option_key: true
"#;
        let parse_result = ProjectManifest::from_yaml_str(malformed_yaml);
        assert!(parse_result.is_err());
        let error = parse_result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("unknown field `invalid_option_key` in profile 'release'"),
            "Error was: {error_message}"
        );
        assert!(
            error_message.contains("manifest.yaml:6:5")
                || error_message.contains("manifest.yaml:6:"),
            "Expected line 6 location in: {error_message}"
        );
    }

    #[test]
    fn test_manifest_yaml_unknown_root_field_has_precise_diagnostics() {
        let malformed_yaml = r#"configuration:
  name: test_app
  version: "1.0.0"
unrecognized_root_key: "value"
profiles:
  release: {}
"#;
        let parse_result = ProjectManifest::from_yaml_str(malformed_yaml);
        assert!(parse_result.is_err());
        let error = parse_result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("unknown field `unrecognized_root_key` at root level"),
            "Error was: {error_message}"
        );
        assert!(
            error_message.contains("manifest.yaml:4:1"),
            "Expected line 4 location in: {error_message}"
        );
    }

    #[test]
    fn test_manifest_yaml_deprecated_inherits_has_precise_diagnostics() {
        let malformed_yaml = r#"configuration:
  name: test_app
  version: "1.0.0"
profiles:
  release: {}
  debug:
    inherits: release
"#;
        let parse_result = ProjectManifest::from_yaml_str(malformed_yaml);
        assert!(parse_result.is_err());
        let error = parse_result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("profile 'debug' uses 'inherits', which has been removed"),
            "Error was: {error_message}"
        );
        assert!(
            error_message.contains("manifest.yaml:7:5"),
            "Expected line 7 location in: {error_message}"
        );
    }

    #[test]
    fn test_manifest_yaml_missing_release_profile_has_precise_diagnostics() {
        let malformed_yaml = r#"configuration:
  name: test_app
  version: "1.0.0"
profiles:
  debug: {}
"#;
        let parse_result = ProjectManifest::from_yaml_str(malformed_yaml);
        assert!(parse_result.is_err());
        let error = parse_result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("manifest must define mandatory profiles.release profile"),
            "Error was: {error_message}"
        );
        assert!(
            error_message.contains("manifest.yaml:4:1"),
            "Expected line 4 location in: {error_message}"
        );
    }

    #[test]
    fn test_manifest_yaml_syntax_error_has_precise_diagnostics() {
        let malformed_yaml = r#"configuration:
  name: "unclosed string
profiles:
  release: {}
"#;
        let parse_result = ProjectManifest::from_yaml_str(malformed_yaml);
        assert!(parse_result.is_err());
        let error = parse_result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("manifest.yaml:2:"),
            "Expected line 2 location in: {error_message}"
        );
    }

    #[test]
    fn test_manifest_yaml_vars_and_x_custom_allowed() {
        let valid_yaml = r#"vars:
  shared_opt: &opt 3
x-custom-metadata:
  vendor_id: "example"
configuration:
  name: valid_app
  version: "2.0.0"
profiles:
  release:
    optimization_level: *opt
"#;
        let parse_result = ProjectManifest::from_yaml_str(valid_yaml);
        assert!(parse_result.is_ok(), "Expected valid YAML parsing");
        let manifest = parse_result.unwrap();
        assert_eq!(manifest.configuration.name, "valid_app");
        assert_eq!(manifest.configuration.version, "2.0.0");
        let release = manifest.resolve_profile("release").unwrap();
        assert_eq!(release.opt_level, 3);
        assert!(manifest.extra_fields.contains_key("x-custom-metadata"));
    }

    #[test]
    fn test_manifest_parse_defaults() {
        let toml_sample = r#"
[configuration]
name = "minimal"
version = "0.0.1"

[profiles.release]
"#;
        let manifest = ProjectManifest::from_toml_str(toml_sample).expect("valid toml");
        assert_eq!(manifest.configuration.name, "minimal");
        assert_eq!(manifest.configuration.version, "0.0.1");
        assert_eq!(manifest.configuration.description, None);
        assert!(manifest.configuration.authors.is_empty());
        assert_eq!(manifest.main_entrypoint().as_deref(), Some("Main::main"));
        assert_eq!(
            manifest.test_entrypoint().as_deref(),
            Some("TestRunner::test")
        );
        let resolved = manifest
            .resolve_profile("release")
            .expect("release profile");
        assert_eq!(resolved.opt_level, 3);
        assert_eq!(resolved.inline_threshold, 16);
        assert_eq!(resolved.sources, vec!["src"]);
    }

    #[test]
    fn test_slxarc_pack_inspect_and_extract() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let stage_dir = temp_dir.path().join("stage");
        let bin_dir = stage_dir.join("bin");
        let lib_dir = stage_dir.join("lib");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&lib_dir).unwrap();

        let manifest_content = r#"
[configuration]
name = "pack_test"
version = "0.1.0"

[profiles.release]
"#;
        fs::write(stage_dir.join("manifest.toml"), manifest_content).unwrap();

        let binary_program = BinaryProgram::new();
        let binary_bytes = binary_program.to_bytes();
        fs::write(bin_dir.join("pack_test.slxb"), &binary_bytes).unwrap();

        #[cfg(target_os = "windows")]
        let host_lib_name = "__native_test_lib.dll";
        #[cfg(target_os = "macos")]
        let host_lib_name = "lib__native_test_lib.dylib";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let host_lib_name = "lib__native_test_lib.so";

        fs::write(lib_dir.join(host_lib_name), b"dummy_so_bytes").unwrap();

        let archive_path = temp_dir.path().join("test_app.slxarc");
        SlxarcArchive::pack_directory(&stage_dir, &archive_path).expect("pack slxarc");
        assert!(archive_path.exists());

        let inspection = SlxarcArchive::inspect(&archive_path).expect("inspect slxarc");
        assert!(inspection.manifest.is_some());
        assert_eq!(inspection.manifest.unwrap().configuration.name, "pack_test");
        assert_eq!(inspection.binaries.len(), 1);
        assert_eq!(inspection.native_libraries.len(), 1);

        let extract_cache = temp_dir.path().join("cache");
        let loaded =
            SlxarcArchive::extract_and_load(&archive_path, &extract_cache).expect("extract");
        assert_eq!(loaded.binaries.len(), 1);
        assert_eq!(loaded.native_libraries.len(), 1);
        assert!(loaded.native_libraries[0].exists());
    }

    #[test]
    fn test_native_target_subdir_matching_prefers_host_arch() {
        let host_triple = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        assert!(matches_native_target_subdir(&host_triple));
        assert!(matches_native_target_subdir("host"));
        assert!(matches_native_target_subdir(std::env::consts::OS));
        assert!(!matches_native_target_subdir("sparc-unknown-none"));

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            assert!(matches_native_target_subdir("x86_64-unknown-linux-gnu"));
            assert!(matches_native_target_subdir("x86_64-unknown-linux-musl"));
            assert!(!matches_native_target_subdir("aarch64-unknown-linux-gnu"));
            assert!(!matches_native_target_subdir("x86_64-apple-darwin"));
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            assert!(matches_native_target_subdir("aarch64-apple-darwin"));
            assert!(!matches_native_target_subdir("x86_64-apple-darwin"));
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            assert!(matches_native_target_subdir("x86_64-apple-darwin"));
            assert!(!matches_native_target_subdir("aarch64-apple-darwin"));
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
        {
            assert!(matches_native_target_subdir("x86_64-pc-windows-msvc"));
            assert!(!matches_native_target_subdir("x86_64-pc-windows-gnu"));
            assert!(!matches_native_target_subdir("aarch64-pc-windows-msvc"));
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "gnu"))]
        {
            assert!(matches_native_target_subdir("x86_64-pc-windows-gnu"));
            assert!(!matches_native_target_subdir("x86_64-pc-windows-msvc"));
            assert!(!matches_native_target_subdir("aarch64-pc-windows-gnu"));
        }
    }

    #[test]
    fn test_resolve_expected_native_checksum_prefers_host_abi_over_sibling_targets() {
        let mut checksums = std::collections::HashMap::new();
        checksums.insert("lib/native_demo.so".to_string(), "fallback_sha".to_string());
        checksums.insert(
            "lib/aarch64-unknown-linux-gnu/native_demo.so".to_string(),
            "arm64_sha".to_string(),
        );
        checksums.insert(
            "lib/x86_64-unknown-linux-gnu/native_demo.so".to_string(),
            "x64_linux_sha".to_string(),
        );
        checksums.insert(
            "lib/x86_64-pc-windows-gnu/native_demo.so".to_string(),
            "win_gnu_sha".to_string(),
        );
        checksums.insert(
            "lib/x86_64-pc-windows-msvc/native_demo.so".to_string(),
            "win_msvc_sha".to_string(),
        );

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert_eq!(
            resolve_expected_native_checksum(&checksums, "native_demo.so", None),
            Some("x64_linux_sha")
        );
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
        assert_eq!(
            resolve_expected_native_checksum(&checksums, "native_demo.so", None),
            Some("win_msvc_sha")
        );
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "gnu"))]
        assert_eq!(
            resolve_expected_native_checksum(&checksums, "native_demo.so", None),
            Some("win_gnu_sha")
        );
        assert_eq!(
            resolve_expected_native_checksum(
                &checksums,
                "native_demo.so",
                Some("lib/x86_64-unknown-linux-gnu/native_demo.so")
            ),
            Some("x64_linux_sha")
        );
    }

    #[test]
    fn test_slxarc_prefers_qualified_target_triple_over_unqualified_fallback() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let stage_dir = temp_dir.path().join("stage");
        let bin_dir = stage_dir.join("bin");
        let lib_dir = stage_dir.join("lib");

        #[cfg(target_os = "windows")]
        let (host_sub, wrong_sub, host_lib) = (
            "x86_64-pc-windows-msvc",
            "aarch64-pc-windows-msvc",
            "__native_test_lib.dll",
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let (host_sub, wrong_sub, host_lib) = (
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "lib__native_test_lib.dylib",
        );
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let (host_sub, wrong_sub, host_lib) = (
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "lib__native_test_lib.dylib",
        );
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let (host_sub, wrong_sub, host_lib) = (
            if cfg!(target_arch = "aarch64") {
                "aarch64-unknown-linux-gnu"
            } else {
                "x86_64-unknown-linux-gnu"
            },
            if cfg!(target_arch = "aarch64") {
                "x86_64-unknown-linux-gnu"
            } else {
                "aarch64-unknown-linux-gnu"
            },
            "lib__native_test_lib.so",
        );

        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(lib_dir.join(host_sub)).unwrap();
        fs::create_dir_all(lib_dir.join(wrong_sub)).unwrap();
        fs::write(
            stage_dir.join("manifest.toml"),
            "[configuration]\nname = \"multi_arch\"\nversion = \"0.1.0\"\n[profiles.release]\n",
        )
        .unwrap();
        fs::write(
            bin_dir.join("multi_arch.slxb"),
            BinaryProgram::new().to_bytes(),
        )
        .unwrap();
        fs::write(lib_dir.join(host_lib), b"fallback_bytes").unwrap();
        fs::write(lib_dir.join(wrong_sub).join(host_lib), b"wrong_arch_bytes").unwrap();
        fs::write(lib_dir.join(host_sub).join(host_lib), b"host_triple_bytes").unwrap();

        let archive_path = temp_dir.path().join("multi.slxarc");
        SlxarcArchive::pack_directory(&stage_dir, &archive_path).expect("pack");

        let extract_cache = temp_dir.path().join("cache");
        let loaded = SlxarcArchive::extract_and_load(&archive_path, &extract_cache).expect("load");
        assert_eq!(loaded.native_libraries.len(), 1);
        assert_eq!(
            fs::read(&loaded.native_libraries[0]).unwrap(),
            b"host_triple_bytes"
        );
    }

    #[test]
    fn test_windows_sibling_abi_fallback_priority() {
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
        {
            assert_eq!(native_target_subdir_priority("x86_64-pc-windows-msvc"), 3);
            assert_eq!(native_target_subdir_priority("x86_64-pc-windows-gnu"), 2);
            let mut gnu_only = std::collections::HashMap::new();
            gnu_only.insert(
                "lib/x86_64-pc-windows-gnu/demo.dll".to_string(),
                "gnu_only_sha".to_string(),
            );
            assert_eq!(
                resolve_expected_native_checksum(&gnu_only, "demo.dll", None),
                Some("gnu_only_sha")
            );
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "gnu"))]
        {
            assert_eq!(native_target_subdir_priority("x86_64-pc-windows-gnu"), 3);
            assert_eq!(native_target_subdir_priority("x86_64-pc-windows-msvc"), 2);
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            assert_eq!(native_target_subdir_priority("x86_64-unknown-linux-gnu"), 3);
            assert_eq!(native_target_subdir_priority("x86_64-pc-windows-gnu"), 0);
        }
    }

    #[test]
    fn test_strip_verbatim_prefix_text_normalizes_drive_and_unc_paths() {
        assert_eq!(
            strip_verbatim_prefix_text(r"\\?\D:\a\solix-language\libraries\raylib"),
            Some(PathBuf::from(r"D:\a\solix-language\libraries\raylib"))
        );
        assert_eq!(
            strip_verbatim_prefix_text(r"\\?\UNC\server\share\solix"),
            Some(PathBuf::from(r"\\server\share\solix"))
        );
        assert_eq!(strip_verbatim_prefix_text("/home/jehud/solix"), None);
    }

    fn build_raw_zip_archive(archive_path: &Path, entries: &[(&str, &[u8])]) {
        let out_file = File::create(archive_path).expect("create test zip archive");
        let mut zip_writer = zip::ZipWriter::new(out_file);
        let options = zip::write::SimpleFileOptions::default();
        for (member_name, member_bytes) in entries {
            zip_writer
                .start_file(*member_name, options)
                .expect("start zip entry");
            zip_writer.write_all(member_bytes).expect("write zip entry");
        }
        zip_writer.finish().expect("finish zip archive");
    }

    #[test]
    fn test_slxarc_extract_rejects_zip_slip_resource_traversal() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let archive_path = temp_dir.path().join("malicious_resource.slxarc");
        let escaped_file = temp_dir.path().join("escaped_secret.txt");
        let manifest_bytes =
            b"[configuration]\nname = \"evil\"\nversion = \"0.1.0\"\n[profiles.release]\n";
        let binary_bytes = BinaryProgram::new().to_bytes();

        build_raw_zip_archive(
            &archive_path,
            &[
                ("manifest.toml", manifest_bytes),
                ("bin/evil.slxb", &binary_bytes),
                ("resources/../../escaped_secret.txt", b"pwned"),
            ],
        );

        let extract_cache = temp_dir.path().join("nested").join("cache");
        let extract_result = SlxarcArchive::extract_and_load(&archive_path, &extract_cache);
        assert!(
            extract_result.is_err(),
            "Zip-Slip entry inside resources/ MUST be rejected cleanly"
        );
        assert!(
            !escaped_file.exists(),
            "Traversal payload must never be written outside extract cache"
        );
    }

    #[test]
    fn test_slxarc_extract_rejects_zip_slip_native_lib_and_absolute_paths() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        for dangerous_path in [
            "lib/../../evil.so",
            "lib\\..\\evil.dll",
            "/etc/passwd",
            "C:\\Windows\\System32\\evil.dll",
        ] {
            let validation = validate_archive_relative_path(dangerous_path);
            assert!(
                validation.is_err(),
                "Expected dangerous archive member '{dangerous_path}' to be rejected"
            );
        }
        let clean = validate_archive_relative_path("resources/textures/hero.png");
        assert_eq!(
            clean.expect("valid relative archive path"),
            PathBuf::from("resources").join("textures").join("hero.png")
        );
        let _ = temp_dir;
    }

    #[test]
    fn test_slxarc_extract_rejects_tampered_native_checksum() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let archive_path = temp_dir.path().join("tampered_native.slxarc");
        #[cfg(target_os = "windows")]
        let native_rel = "lib/demo_native.dll";
        #[cfg(target_os = "macos")]
        let native_rel = "lib/libdemo_native.dylib";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let native_rel = "lib/libdemo_native.so";

        let manifest_toml = format!(
            "[configuration]\nname = \"tamper_check\"\nversion = \"0.1.0\"\n[native_library_checksums]\n\"{native_rel}\" = \"0000000000000000000000000000000000000000000000000000000000000000\"\n"
        );
        let binary_bytes = BinaryProgram::new().to_bytes();

        build_raw_zip_archive(
            &archive_path,
            &[
                ("manifest.toml", manifest_toml.as_bytes()),
                ("bin/tamper_check.slxb", &binary_bytes),
                (native_rel, b"tampered_native_binary_bytes"),
            ],
        );

        let extract_cache = temp_dir.path().join("cache");
        let extract_result = SlxarcArchive::extract_and_load(&archive_path, &extract_cache);
        assert!(
            extract_result.is_err(),
            "Mismatched native library SHA-256 checksum MUST abort archive loading"
        );
    }

    #[test]
    fn test_is_solix_native_module_name_requires_double_underscore_native_prefix() {
        assert!(is_solix_native_module_name("__native_raylib.so"));
        assert!(is_solix_native_module_name("lib__native_raylib.so"));
        assert!(is_solix_native_module_name("__native_physics.dll"));
        assert!(is_solix_native_module_name("lib__native_engine.dylib"));

        assert!(!is_solix_native_module_name("raylib_native.so"));
        assert!(!is_solix_native_module_name("libraylib_native.so"));
        assert!(!is_solix_native_module_name("libglfw.so"));
        assert!(!is_solix_native_module_name("openal.dll"));
        assert!(!is_solix_native_module_name("cblas.dylib"));
    }

    #[test]
    fn test_issue_250_conditional_checksum_defaults() {
        // Release static profile: verify_checksum defaults to false
        let yaml_static_release = r#"
configuration:
  name: test_pkg
  version: "1.0.0"
profiles:
  release:
    static: true
"#;
        let manifest_static =
            ProjectManifest::from_yaml_str(yaml_static_release).expect("valid yaml");
        let profile_static = manifest_static
            .resolve_profile("release")
            .expect("resolves");
        assert!(
            !profile_static.verify_checksum,
            "Static profile must default to verify_checksum = false"
        );

        // Release dynamic profile: verify_checksum defaults to true
        let yaml_dynamic_release = r#"
configuration:
  name: test_pkg
  version: "1.0.0"
profiles:
  release:
    static: false
"#;
        let manifest_dynamic =
            ProjectManifest::from_yaml_str(yaml_dynamic_release).expect("valid yaml");
        let profile_dynamic = manifest_dynamic
            .resolve_profile("release")
            .expect("resolves");
        assert!(
            profile_dynamic.verify_checksum,
            "Dynamic release profile must default to verify_checksum = true"
        );

        // Debug profile: verify_checksum defaults to false
        let profile_debug = manifest_static.resolve_profile("debug").expect("resolves");
        assert!(
            !profile_debug.verify_checksum,
            "Debug profile must default to verify_checksum = false"
        );

        // Test profile: verify_checksum defaults to false
        let profile_test = manifest_static.resolve_profile("test").expect("resolves");
        assert!(
            !profile_test.verify_checksum,
            "Test profile must default to verify_checksum = false"
        );

        // Explicit override: verify_checksum = true on debug
        let yaml_explicit_debug = r#"
configuration:
  name: test_pkg
  version: "1.0.0"
profiles:
  release: {}
  debug:
    verify_checksum: true
"#;
        let manifest_explicit =
            ProjectManifest::from_yaml_str(yaml_explicit_debug).expect("valid yaml");
        let profile_explicit_debug = manifest_explicit
            .resolve_profile("debug")
            .expect("resolves");
        assert!(
            profile_explicit_debug.verify_checksum,
            "Explicit verify_checksum = true must be respected"
        );
    }

    #[test]
    fn test_slxp_package_ignores_duplicate_legacy_bin_copy_when_root_binaries_present() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let stage_dir = temp_dir.path().join("stage");
        let bin_dir = stage_dir.join("bin");
        fs::create_dir_all(&bin_dir).expect("create stage bin dir");

        let manifest_yaml = r#"configuration:
  name: dup_copy_pkg
  version: "0.1.0"
profiles:
  release: {}
"#;
        fs::write(stage_dir.join("manifest.yaml"), manifest_yaml).expect("write manifest.yaml");

        let mut program = BinaryProgram::new();
        program.package_name = "dup_copy_pkg".to_string();
        let binary_bytes = program.to_bytes();
        // Stage both root `binaries.slxb` and identical legacy `bin/<pkg>.slxb` (matching `solix build`).
        fs::write(stage_dir.join("binaries.slxb"), &binary_bytes).expect("write root binaries");
        fs::write(bin_dir.join("dup_copy_pkg.slxb"), &binary_bytes).expect("write legacy bin copy");

        let package_path = temp_dir.path().join("dup_copy_pkg.slxp");
        SlxpPackage::pack_directory(&stage_dir, &package_path).expect("pack slxp");

        let extract_cache = temp_dir.path().join("cache");
        let loaded = SlxpPackage::extract_and_load(&package_path, &extract_cache)
            .expect("extract and load slxp");
        assert_eq!(
            loaded.binaries.len(),
            1,
            "Duplicate legacy bin/<pkg>.slxb copy of root binaries.slxb must not be double-loaded"
        );
    }

    #[test]
    fn test_slxp_extract_and_load_sidecar_cache_hit_and_corrupt_fallback() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let stage_dir = temp_dir.path().join("stage");
        fs::create_dir_all(&stage_dir).expect("create stage dir");

        let manifest_yaml = r#"configuration:
  name: cached_pkg
  version: "0.1.0"
profiles:
  release: {}
"#;
        fs::write(stage_dir.join("manifest.yaml"), manifest_yaml).expect("write manifest");

        let mut program = BinaryProgram::new();
        program.package_name = "cached_pkg".to_string();
        fs::write(stage_dir.join("binaries.slxb"), program.to_bytes()).expect("write binaries");

        let package_path = temp_dir.path().join("cached_pkg.slxp");
        SlxpPackage::pack_directory(&stage_dir, &package_path).expect("pack slxp");

        let extract_cache = temp_dir.path().join("cache");
        let first_load =
            SlxpPackage::extract_and_load(&package_path, &extract_cache).expect("first load");
        assert_eq!(first_load.binaries.len(), 1);

        let second_load =
            SlxpPackage::extract_and_load(&package_path, &extract_cache).expect("cached load");
        assert_eq!(second_load.binaries.len(), 1);
        assert_eq!(second_load.binaries[0].package_name, "cached_pkg");

        // Corrupt sidecar metadata and verify clean fallback to full ZIP extraction
        for entry in fs::read_dir(&extract_cache).expect("read cache dir") {
            let entry_path = entry.expect("dir entry").path();
            if entry_path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(".slxp_meta_"))
            {
                fs::write(&entry_path, b"corrupt_meta").expect("corrupt sidecar");
            }
        }
        let fallback_load =
            SlxpPackage::extract_and_load(&package_path, &extract_cache).expect("fallback load");
        assert_eq!(fallback_load.binaries.len(), 1);
    }
}
