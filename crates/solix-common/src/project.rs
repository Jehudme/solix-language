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

/// User-editable project identity metadata declared in the `[info]` table of `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Package or project canonical identifier name.
    pub name: String,
    /// Semantic version string (e.g., `"0.1.0"`).
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

/// Entry point symbol overrides declared in the `[entrypoints]` table of `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProjectEntryPoints {
    /// Canonical entry point symbol for standard execution (`debug` and `release`).
    #[serde(default)]
    pub main: Option<String>,
    /// Canonical entry point symbol for test suite execution (`test`).
    #[serde(default)]
    pub test: Option<String>,
}

/// Minimal, user-editable project manifest contract parsed from `manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    /// Project identity and metadata.
    pub info: ProjectInfo,
    /// Entry point symbol mappings.
    #[serde(default)]
    pub entrypoints: ProjectEntryPoints,
}

impl ProjectManifest {
    /// Parses a `manifest.toml` string into a strongly-typed `ProjectManifest`.
    pub fn from_toml_str(toml_content: &str) -> Result<Self, EmissionError> {
        toml::from_str(toml_content).map_err(|parse_error| {
            EmissionError::ContainerSerializationFailed {
                reason: format!("failed to parse manifest.toml: {parse_error}"),
            }
        })
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
                // Extract host native library to cache
                let target_path = extract_cache_directory.join(
                    Path::new(&entry_name)
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new("native.so")),
                );

                let mut out_file = File::create(&target_path).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to extract native library '{}': {io_error}",
                            target_path.display()
                        ),
                    }
                })?;

                let mut library_bytes = Vec::new();
                file_entry
                    .read_to_end(&mut library_bytes)
                    .map_err(|io_error| EmissionError::ContainerSerializationFailed {
                        reason: format!("failed to read native library '{entry_name}': {io_error}"),
                    })?;

                out_file.write_all(&library_bytes).map_err(|io_error| {
                    EmissionError::ContainerSerializationFailed {
                        reason: format!(
                            "failed to write native library '{}': {io_error}",
                            target_path.display()
                        ),
                    }
                })?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755));
                }

                native_libraries.push(target_path.clone());
                extracted_files.push(target_path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse_minimal_valid() {
        let toml_sample = r#"
[info]
name = "my_app"
version = "1.2.3"
description = "A sample Solix application"
license = "Apache-2.0"
authors = ["Jane Doe <jane@example.com>"]

[entrypoints]
main = "Main::main"
test = "TestRunner::main"
"#;
        let manifest = ProjectManifest::from_toml_str(toml_sample).expect("valid toml");
        assert_eq!(manifest.info.name, "my_app");
        assert_eq!(manifest.info.version, "1.2.3");
        assert_eq!(
            manifest.info.description.as_deref(),
            Some("A sample Solix application")
        );
        assert_eq!(manifest.info.license.as_deref(), Some("Apache-2.0"));
        assert_eq!(manifest.info.authors, vec!["Jane Doe <jane@example.com>"]);
        assert_eq!(manifest.entrypoints.main.as_deref(), Some("Main::main"));
        assert_eq!(
            manifest.entrypoints.test.as_deref(),
            Some("TestRunner::main")
        );
    }

    #[test]
    fn test_manifest_parse_defaults() {
        let toml_sample = r#"
[info]
name = "minimal"
version = "0.0.1"
"#;
        let manifest = ProjectManifest::from_toml_str(toml_sample).expect("valid toml");
        assert_eq!(manifest.info.name, "minimal");
        assert_eq!(manifest.info.version, "0.0.1");
        assert_eq!(manifest.info.description, None);
        assert!(manifest.info.authors.is_empty());
        assert_eq!(manifest.entrypoints.main, None);
        assert_eq!(manifest.entrypoints.test, None);
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
[info]
name = "pack_test"
version = "0.1.0"
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
        assert_eq!(inspection.manifest.unwrap().info.name, "pack_test");
        assert_eq!(inspection.binaries.len(), 1);
        assert_eq!(inspection.native_libraries.len(), 1);

        let extract_cache = temp_dir.path().join("cache");
        let loaded =
            SlxarcArchive::extract_and_load(&archive_path, &extract_cache).expect("extract");
        assert_eq!(loaded.binaries.len(), 1);
        assert_eq!(loaded.native_libraries.len(), 1);
        assert!(loaded.native_libraries[0].exists());
    }
}
