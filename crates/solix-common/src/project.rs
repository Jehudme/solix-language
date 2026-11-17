//! Project manifest schema (`manifest.toml`) and `.slxarc` archive container manager.

#![forbid(unsafe_code)]

use crate::bytecode::BinaryProgram;
use crate::diagnostics::EmissionError;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

fn default_project_version() -> String {
    "0.1.0".to_string()
}

/// User-editable project identity metadata declared in the `[configuration]` table of `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

/// Backwards compatibility alias for `ProjectConfiguration`.
pub type ProjectInfo = ProjectConfiguration;

/// Profile-specific compilation, dependency, and source configuration declared in `[profiles.<name>]`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Optional parent profile to inherit from (defaults to `"release"` if omitted on non-release profiles).
    #[serde(default)]
    pub inherits: Option<String>,
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
    /// Profile-scoped dependencies mapped as `name => specification`.
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, DependencySpec>,
}

/// Fully resolved, flattened profile configuration with all inheritance applied.
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
    /// Profile-scoped resolved dependencies.
    pub dependencies: std::collections::BTreeMap<String, DependencySpec>,
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
            verify_checksum: true,
            link: "static".to_string(),
            memory_limit_mb: None,
            sources: vec!["src".to_string()],
            dependencies: std::collections::BTreeMap::new(),
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

/// Unified, user-editable project manifest contract parsed from `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    /// Unified project identity and metadata.
    pub configuration: ProjectConfiguration,
    /// First-class profiles table mapped as `[profiles.<name>]`.
    pub profiles: std::collections::BTreeMap<String, ProfileConfig>,
    /// Cryptographic SHA-256 digests of bundled native shared libraries.
    #[serde(default)]
    pub native_library_checksums: std::collections::HashMap<String, String>,
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

    /// Parses a `manifest.toml` string into a strongly-typed `ProjectManifest`.
    pub fn from_toml_str(toml_content: &str) -> Result<Self, EmissionError> {
        let manifest: Self = toml::from_str(toml_content).map_err(|parse_error| {
            if toml_content.contains("[info]")
                || toml_content.contains("[entrypoints]")
                || toml_content.contains("[build]")
                || toml_content.contains("[profile.")
            {
                EmissionError::ContainerSerializationFailed {
                    reason: format!(
                        "failed to parse manifest.toml: legacy manifest sections ([info], [entrypoints], [build], [profile.<name>]) are no longer supported; please update to [configuration] and [profiles]: {parse_error}"
                    ),
                }
            } else {
                EmissionError::ContainerSerializationFailed {
                    reason: format!("failed to parse manifest.toml: {parse_error}"),
                }
            }
        })?;

        if !manifest.profiles.contains_key("release") {
            return Err(EmissionError::ContainerSerializationFailed {
                reason: "manifest.toml must define mandatory [profiles.release] profile"
                    .to_string(),
            });
        }

        Ok(manifest)
    }

    /// Reads and parses `manifest.toml` from a file path on disk.
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Self, EmissionError> {
        let path_reference = file_path.as_ref();
        let content = fs::read_to_string(path_reference).map_err(|io_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "failed to read manifest at '{}': {io_error}",
                    path_reference.display()
                ),
            }
        })?;
        Self::from_toml_str(&content)
    }

    /// Serializes this manifest to a standard TOML string.
    pub fn to_toml_string(&self) -> Result<String, EmissionError> {
        toml::to_string_pretty(self).map_err(|serialize_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("failed to serialize manifest to toml: {serialize_error}"),
            }
        })
    }

    /// Resolves the hierarchical profile configuration for the requested profile name.
    ///
    /// The only mandatory base profile is `"release"`. If a profile does not define `inherits`,
    /// it automatically inherits from `"release"`. Profile sources and dependencies are
    /// inherited and merged hierarchically.
    ///
    /// # Errors
    /// Returns an error if the mandatory `"release"` profile is missing or a cyclic loop is detected.
    pub fn resolve_profile(&self, target_profile: &str) -> Result<ResolvedProfile, EmissionError> {
        if !self.profiles.contains_key("release") {
            return Err(EmissionError::ContainerSerializationFailed {
                reason: "manifest.toml must define mandatory [profiles.release] profile"
                    .to_string(),
            });
        }

        if target_profile != "release"
            && target_profile != "debug"
            && target_profile != "test"
            && !self.profiles.contains_key(target_profile)
        {
            return Err(EmissionError::ContainerSerializationFailed {
                reason: format!(
                    "Unknown build profile '{target_profile}'; expected 'debug', 'release', 'test', or a profile defined in manifest.toml"
                ),
            });
        }

        let mut visited_profiles: Vec<String> = Vec::new();
        let mut resolution_chain: Vec<String> = Vec::new();
        let mut current_profile_name = target_profile.to_string();

        while !current_profile_name.is_empty() {
            if visited_profiles.contains(&current_profile_name) {
                return Err(EmissionError::ContainerSerializationFailed {
                    reason: format!(
                        "Cyclic profile inheritance detected: {} -> {}",
                        visited_profiles.join(" -> "),
                        current_profile_name
                    ),
                });
            }

            visited_profiles.push(current_profile_name.clone());
            resolution_chain.push(current_profile_name.clone());

            if current_profile_name == "release" {
                break;
            }

            let next_parent = if let Some(config) = self.profiles.get(&current_profile_name) {
                if let Some(ref inherits_from) = config.inherits {
                    inherits_from.clone()
                } else {
                    "release".to_string()
                }
            } else {
                "release".to_string()
            };

            current_profile_name = next_parent;
        }

        let mut resolved = ResolvedProfile::default_release_baseline();

        while let Some(step_profile) = resolution_chain.pop() {
            if let Some(user_config) = self.profiles.get(&step_profile) {
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
                }
                if let Some(memory_limit_mb) = user_config.memory_limit_mb {
                    resolved.memory_limit_mb = Some(memory_limit_mb);
                }

                if let Some(ref step_sources) = user_config.sources {
                    if step_profile == "release" {
                        resolved.sources = step_sources.clone();
                    } else {
                        for source_dir in step_sources {
                            if !resolved.sources.contains(source_dir) {
                                resolved.sources.push(source_dir.clone());
                            }
                        }
                    }
                }

                for (dep_name, dep_spec) in &user_config.dependencies {
                    resolved
                        .dependencies
                        .insert(dep_name.clone(), dep_spec.clone());
                }
            } else if step_profile == "debug" {
                resolved.opt_level = 0;
                resolved.inline_threshold = 8;
                resolved.debug_symbols = true;
                resolved.is_static = false;
                resolved.link = "dynamic".to_string();
            } else if step_profile == "test" {
                resolved.opt_level = 0;
                resolved.inline_threshold = 8;
                resolved.debug_symbols = true;
                resolved.entrypoint = Some("TestRunner::test".to_string());
                if !resolved.sources.contains(&"tests".to_string())
                    && !resolved.sources.contains(&"test".to_string())
                {
                    resolved.sources.push("tests".to_string());
                }
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

/// Detailed metadata record for an individual entry inside a `.slxarc` container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlxarcFileEntry {
    /// Relative path within the archive container.
    pub path: String,
    /// Uncompressed payload size in bytes.
    pub uncompressed_size: u64,
    /// Compressed payload size stored within the archive container.
    pub compressed_size: u64,
    /// Whether this entry represents a directory structure.
    pub is_directory: bool,
}

/// Inspection summary containing decoded contents of a `.slxarc` archive container.
#[derive(Debug, Clone)]
pub struct SlxarcInspection {
    /// Project manifest metadata, if bundled at the archive root.
    pub manifest: Option<ProjectManifest>,
    /// Full list of file entries contained within the archive.
    pub file_entries: Vec<SlxarcFileEntry>,
    /// Deserialized Solix bytecode binaries embedded inside the archive.
    pub binaries: Vec<(String, BinaryProgram)>,
    /// List of native dynamic shared libraries identified within `lib/`.
    pub native_libraries: Vec<String>,
}

/// Fully extracted and loaded `.slxarc` package ready for virtual machine execution.
#[derive(Debug, Clone)]
pub struct LoadedSlxarc {
    /// Project manifest metadata bundled with the archive.
    pub manifest: Option<ProjectManifest>,
    /// Primary compiled bytecode binaries found inside `bin/` or root.
    pub binaries: Vec<BinaryProgram>,
    /// Discovered host native shared library file paths extracted to the runtime cache.
    pub native_libraries: Vec<PathBuf>,
    /// Secondary dependency `.slxb` binaries found inside `lib/`.
    pub secondary_binaries: Vec<BinaryProgram>,
    /// All files extracted to the cache directory.
    pub extracted_files: Vec<PathBuf>,
    /// Static resource files extracted from `resources/`.
    pub resource_files: Vec<PathBuf>,
    /// Total byte length of all extracted static resource files.
    pub resource_total_bytes: u64,
}

/// Archive container packager, extractor, and inspector for `.slxarc` distributions.
pub struct SlxarcArchive;

impl SlxarcArchive {
    /// Packages an entire directory tree into a deterministic `.slxarc` archive.
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

            let entry_name = file_entry.name().to_string();
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
                if entry_name == "manifest.toml" {
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

    /// Extracts host native shared libraries and loads bytecode binaries for VM execution.
    pub fn extract_and_load(
        archive_path: &Path,
        extract_cache_directory: &Path,
    ) -> Result<LoadedSlxarc, EmissionError> {
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
        let mut secondary_binaries = Vec::new();
        let mut native_libraries = Vec::new();
        let mut extracted_files = Vec::new();
        let mut qualified_native_names: std::collections::HashSet<std::ffi::OsString> =
            std::collections::HashSet::new();
        let mut resource_files = Vec::new();
        let mut resource_total_bytes = 0u64;

        for index in 0..zip_reader.len() {
            let mut file_entry = zip_reader.by_index(index).map_err(|zip_error| {
                EmissionError::ContainerSerializationFailed {
                    reason: format!("failed to read entry at index {index}: {zip_error}"),
                }
            })?;

            let entry_name = file_entry.name().to_string();
            if file_entry.is_dir() {
                continue;
            }

            if entry_name == "manifest.toml" {
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
            } else if entry_name.starts_with("bin/") && entry_name.ends_with(".slxb") {
                let mut binary_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut binary_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read binary '{entry_name}': {io_error}"),
                    })?;
                let binary = BinaryProgram::from_bytes(&binary_bytes).map_err(|deser_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to decode binary '{entry_name}': {deser_error:?}"),
                    }
                })?;
                binaries.push(binary);
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
                            "failed to decode dependency binary '{entry_name}': {deser_error:?}"
                        ),
                    }
                })?;
                secondary_binaries.push(binary);
            } else if entry_name.starts_with("lib/") && entry_name.ends_with(host_extension) {
                let relative_parts: Vec<&str> = entry_name
                    .strip_prefix("lib/")
                    .unwrap_or("")
                    .split('/')
                    .collect();
                let is_qualified_subdir = relative_parts.len() > 1;
                let matches_platform = if is_qualified_subdir {
                    matches_native_target_subdir(relative_parts[0])
                } else {
                    true
                };

                if !matches_platform {
                    continue;
                }

                // Extract host native library atomically, preferring target-qualified paths over fallback `lib/<file>`.
                let file_name_os = Path::new(&entry_name)
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new("native.so"));
                if !is_qualified_subdir && qualified_native_names.contains(file_name_os) {
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
                        ".tmp.{}.{}.{}",
                        std::process::id(),
                        seq_id,
                        file_name_lossy
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
                        return Err(EmissionError::ContainerSerializationFailed {
                            reason: format!(
                                "failed to finalize native library '{}': {rename_error}",
                                target_path.display()
                            ),
                        });
                    }
                }

                if is_qualified_subdir {
                    qualified_native_names.insert(file_name_os.to_os_string());
                }
                if !native_libraries.contains(&target_path) {
                    native_libraries.push(target_path.clone());
                }
                if !extracted_files.contains(&target_path) {
                    extracted_files.push(target_path);
                }
            } else if entry_name.starts_with("resources/") {
                let target_path = extract_cache_directory.join(&entry_name);
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

        for native_lib_path in &native_libraries {
            let file_name = native_lib_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let expected_checksum = manifest_checksums
                .iter()
                .chain(binary_checksums.iter())
                .find_map(|(key, hash)| {
                    if key.ends_with(file_name) || native_lib_path.to_string_lossy().ends_with(key)
                    {
                        Some(hash)
                    } else {
                        None
                    }
                });

            if let Some(expected) = expected_checksum {
                let actual = crate::hash::sha256_file(native_lib_path).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to hash extracted native library '{}': {io_error}",
                            native_lib_path.display()
                        ),
                    }
                })?;
                if &actual != expected {
                    return Err(EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "native library checksum verification failed for '{}': expected {expected}, found {actual}",
                            native_lib_path.display()
                        ),
                    });
                }
            }
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
                let entry_name = file_entry.name().to_string();
                if entry_name.ends_with(".slxb") && !entry_name.starts_with("lib/") {
                    let mut binary_bytes = Vec::new();
                    file_entry
                        .read_to_end(&mut binary_bytes)
                        .map_err(|io_err| EmissionError::ContainerSerializationFailed {
                            reason: format!("failed to read root binary: {io_err}"),
                        })?;
                    let binary = BinaryProgram::from_bytes(&binary_bytes).map_err(|deser_err| {
                        EmissionError::ContainerSerializationFailed {
                            reason: format!("failed to decode root binary: {deser_err:?}"),
                        }
                    })?;
                    binaries.push(binary);
                }
            }
        }

        Ok(LoadedSlxarc {
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
fn matches_native_target_subdir(subdirectory: &str) -> bool {
    let host_os = std::env::consts::OS;
    let host_arch = std::env::consts::ARCH;
    let short_pair = format!("{host_os}-{host_arch}");

    if subdirectory == short_pair || subdirectory == host_os || subdirectory == "host" {
        return true;
    }

    let triple_matches_os = match host_os {
        "linux" => subdirectory.contains("-linux-"),
        "macos" => subdirectory.contains("-apple-darwin"),
        "windows" => subdirectory.contains("-pc-windows-"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse_minimal_valid() {
        let toml_sample = r#"
[configuration]
name = "my_app"
version = "1.2.3"
description = "A sample Solix application"
license = "Apache-2.0"
authors = ["Jane Doe <jane@example.com>"]

[profiles.release]
entrypoint = "Main::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]

[profiles.test]
inherits = "release"
entrypoint = "TestRunner::main"
sources = ["tests"]
"#;
        let manifest = ProjectManifest::from_toml_str(toml_sample).expect("valid toml");
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

        let resolved_test = manifest.resolve_profile("test").expect("test profile");
        assert_eq!(resolved_test.sources, vec!["src", "tests"]);
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
        let host_lib_name = "test_lib.dll";
        #[cfg(target_os = "macos")]
        let host_lib_name = "test_lib.dylib";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let host_lib_name = "test_lib.so";

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
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            assert!(matches_native_target_subdir("x86_64-pc-windows-msvc"));
            assert!(matches_native_target_subdir("x86_64-pc-windows-gnu"));
            assert!(!matches_native_target_subdir("aarch64-pc-windows-msvc"));
        }
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
            "test_lib.dll",
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let (host_sub, wrong_sub, host_lib) = (
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "libtest_lib.dylib",
        );
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let (host_sub, wrong_sub, host_lib) = (
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "libtest_lib.dylib",
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
            "libtest_lib.so",
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
}
