//! Build script for `solix-toolchain`.
//!
//! Stages `assets/` into the target output directory alongside the `solix` binary and generates
//! `OUT_DIR/embedded_assets.rs` so standalone release binaries remain self-contained.

#![forbid(unsafe_code)]

use std::env;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let target_path = destination.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(&entry.path(), &target_path)?;
        } else {
            fs::copy(entry.path(), target_path)?;
        }
    }
    Ok(())
}

fn escape_rust_string(raw_value: &str) -> String {
    raw_value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn find_vsix_in_dir(directory: &Path) -> Option<(String, PathBuf)> {
    if !directory.is_dir() {
        return None;
    }

    let canonical_name = "solix-language.vsix";
    let canonical_path = directory.join(canonical_name);
    if canonical_path.is_file() {
        return Some((canonical_name.to_string(), canonical_path));
    }

    let entries = fs::read_dir(directory).ok()?;
    let mut candidates: Vec<(String, PathBuf)> = Vec::new();
    for entry_result in entries.flatten() {
        let entry_path = entry_result.path();
        if entry_path.is_file()
            && entry_path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension_str| extension_str.eq_ignore_ascii_case("vsix"))
        {
            let file_name = entry_result.file_name().to_string_lossy().to_string();
            candidates.push((file_name, entry_path));
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates.pop()
}

fn collect_bundled_packages(packages_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut packages: Vec<(String, PathBuf)> = Vec::new();
    if !packages_dir.is_dir() {
        return packages;
    }

    let Ok(entries) = fs::read_dir(packages_dir) else {
        return packages;
    };

    for entry_result in entries.flatten() {
        let subdir_path = entry_result.path();
        if !subdir_path.is_dir() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", subdir_path.display());

        let folder_name = entry_result.file_name().to_string_lossy().to_string();
        let Ok(sub_entries) = fs::read_dir(&subdir_path) else {
            continue;
        };

        for sub_entry in sub_entries.flatten() {
            let file_path = sub_entry.path();
            if file_path.is_file()
                && file_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension_str| extension_str.eq_ignore_ascii_case("slxarc"))
            {
                println!("cargo:rerun-if-changed={}", file_path.display());
                let file_name = sub_entry.file_name().to_string_lossy().to_string();
                let archive_rel_path = format!("{folder_name}/{file_name}");
                packages.push((archive_rel_path, file_path));
            }
        }
    }

    packages.sort_by(|left, right| left.0.cmp(&right.0));
    packages
}

fn collect_wiki_docs_recursive(
    base_dir: &Path,
    current_dir: &Path,
    collected: &mut Vec<(String, PathBuf)>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry_result in entries.flatten() {
        let entry_path = entry_result.path();
        if entry_path.is_dir() {
            println!("cargo:rerun-if-changed={}", entry_path.display());
            collect_wiki_docs_recursive(base_dir, &entry_path, collected);
        } else if entry_path.is_file()
            && let Ok(relative) = entry_path.strip_prefix(base_dir)
        {
            println!("cargo:rerun-if-changed={}", entry_path.display());
            let rel_posix = relative
                .components()
                .filter_map(|component| match component {
                    std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect::<Vec<String>>()
                .join("/");
            if !rel_posix.is_empty() {
                collected.push((rel_posix, entry_path));
            }
        }
    }
}

fn collect_wiki_docs(docs_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut wiki_docs: Vec<(String, PathBuf)> = Vec::new();
    if !docs_dir.is_dir() {
        return wiki_docs;
    }
    println!("cargo:rerun-if-changed={}", docs_dir.display());
    collect_wiki_docs_recursive(docs_dir, docs_dir, &mut wiki_docs);
    wiki_docs.sort_by(|left, right| left.0.cmp(&right.0));
    wiki_docs
}

fn target_matches_native_subdir(
    subdirectory: &str,
    target_os: &str,
    target_arch: &str,
    target_env: &str,
) -> bool {
    let short_pair = format!("{target_os}-{target_arch}");
    if subdirectory == short_pair || subdirectory == target_os || subdirectory == "host" {
        return true;
    }

    let triple_matches_os = match target_os {
        "linux" => subdirectory.contains("-linux-"),
        "macos" => subdirectory.contains("-apple-darwin"),
        "windows" => {
            if target_env == "msvc" {
                subdirectory.contains("-pc-windows-msvc") || subdirectory.contains("-pc-windows-")
            } else if target_env == "gnu" {
                subdirectory.contains("-pc-windows-gnu") || subdirectory.contains("-pc-windows-")
            } else {
                subdirectory.contains("-pc-windows-")
            }
        }
        _ => false,
    };

    if !triple_matches_os {
        return false;
    }

    match target_arch {
        "x86_64" => subdirectory.starts_with("x86_64-"),
        "aarch64" => subdirectory.starts_with("aarch64-") || subdirectory.starts_with("arm64-"),
        _ => subdirectory.starts_with(target_arch),
    }
}

/// Filters foreign-OS/architecture native shared libraries (`lib/<other-target>/...`) out of a
/// `.slxarc` ZIP payload so each platform's binary only embeds native libraries for its own target.
fn filter_slxarc_for_target(
    archive_bytes: &[u8],
    target_os: &str,
    target_arch: &str,
    target_env: &str,
) -> Option<Vec<u8>> {
    let reader = Cursor::new(archive_bytes);
    let mut zip_in = ZipArchive::new(reader).ok()?;

    let mut qualified_host_files: Vec<String> = Vec::new();
    let mut has_foreign_entries = false;
    for idx in 0..zip_in.len() {
        let entry = zip_in.by_index(idx).ok()?;
        let Ok(name_cow) = entry.name() else {
            continue;
        };
        let name = name_cow.as_ref();
        if let Some(after_lib) = name.strip_prefix("lib/")
            && let Some((subdir, rest)) = after_lib.split_once('/')
            && !subdir.is_empty()
            && (!rest.is_empty() || entry.is_dir())
        {
            if target_matches_native_subdir(subdir, target_os, target_arch, target_env) {
                if !rest.is_empty() {
                    qualified_host_files.push(rest.to_string());
                }
            } else {
                has_foreign_entries = true;
            }
        }
    }

    let mut has_duplicate_unqualified = false;
    if !qualified_host_files.is_empty() {
        for idx in 0..zip_in.len() {
            let entry = zip_in.by_index(idx).ok()?;
            let Ok(name_cow) = entry.name() else {
                continue;
            };
            let name = name_cow.as_ref();
            if let Some(after_lib) = name.strip_prefix("lib/")
                && !after_lib.contains('/')
                && qualified_host_files.iter().any(|f| f == after_lib)
            {
                has_duplicate_unqualified = true;
                break;
            }
        }
    }

    if !has_foreign_entries && !has_duplicate_unqualified {
        return None;
    }

    let mut out_buf = Cursor::new(Vec::with_capacity(archive_bytes.len() / 2));
    {
        let mut zip_out = ZipWriter::new(&mut out_buf);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        for idx in 0..zip_in.len() {
            let mut entry = zip_in.by_index(idx).ok()?;
            let name = entry.name().ok()?.into_owned();
            if let Some(after_lib) = name.strip_prefix("lib/") {
                if let Some((subdir, _)) = after_lib.split_once('/')
                    && !subdir.is_empty()
                    && !target_matches_native_subdir(subdir, target_os, target_arch, target_env)
                {
                    continue;
                }
                if !after_lib.contains('/') && qualified_host_files.iter().any(|f| f == after_lib) {
                    continue;
                }
            }

            if entry.is_dir() {
                zip_out.add_directory(&name, options).ok()?;
            } else {
                let mut payload = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut payload).ok()?;
                zip_out.start_file(&name, options).ok()?;
                zip_out.write_all(&payload).ok()?;
            }
        }
        zip_out.finish().ok()?;
    }

    Some(out_buf.into_inner())
}

fn write_packed_blob(out_path: &Path, entries: &[(String, Vec<u8>)]) {
    let mut packed: Vec<u8> = Vec::new();
    if !entries.is_empty() {
        packed.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for (rel_path, payload) in entries {
            let path_bytes = rel_path.as_bytes();
            packed.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
            packed.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            packed.extend_from_slice(path_bytes);
            packed.extend_from_slice(payload);
        }
    }
    let unchanged = fs::read(out_path).ok().as_deref() == Some(packed.as_slice());
    if !unchanged && let Ok(mut file) = File::create(out_path) {
        let _ = file.write_all(&packed);
    }
}

fn generate_embedded_assets(manifest_dir: &Path, out_dir: &Path) {
    let source_assets = manifest_dir.join("assets");
    let packages_dir = source_assets.join("packages");
    let vscode_dir = source_assets.join("vscode");
    let wiki_docs_dir = source_assets.join("docs");

    if packages_dir.exists() {
        println!("cargo:rerun-if-changed={}", packages_dir.display());
    }
    let manifest_json_path = packages_dir.join("bundled_manifest.json");
    if manifest_json_path.is_file() {
        println!("cargo:rerun-if-changed={}", manifest_json_path.display());
    }
    if vscode_dir.exists() {
        println!("cargo:rerun-if-changed={}", vscode_dir.display());
    }

    let packages = collect_bundled_packages(&packages_dir);
    let wiki_docs = collect_wiki_docs(&wiki_docs_dir);

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| env::consts::OS.to_string());
    let target_arch =
        env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| env::consts::ARCH.to_string());
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let mut packed_packages: Vec<(String, Vec<u8>)> = Vec::with_capacity(packages.len());
    for (rel_path, archive_path) in &packages {
        if let Ok(raw_bytes) = fs::read(archive_path) {
            let filtered =
                filter_slxarc_for_target(&raw_bytes, &target_os, &target_arch, &target_env)
                    .unwrap_or(raw_bytes);
            packed_packages.push((rel_path.clone(), filtered));
        }
    }
    let packages_pack_path = out_dir.join("embedded_packages.pack");
    write_packed_blob(&packages_pack_path, &packed_packages);

    let mut packed_wiki: Vec<(String, Vec<u8>)> = Vec::with_capacity(wiki_docs.len());
    for (rel_path, doc_file_path) in &wiki_docs {
        if let Ok(raw_bytes) = fs::read(doc_file_path) {
            packed_wiki.push((rel_path.clone(), raw_bytes));
        }
    }
    let wiki_pack_path = out_dir.join("embedded_wiki.pack");
    write_packed_blob(&wiki_pack_path, &packed_wiki);

    let fallback_editors_vscode = manifest_dir
        .join("..")
        .join("..")
        .join("editors")
        .join("vscode");
    let discovered_vsix =
        find_vsix_in_dir(&vscode_dir).or_else(|| find_vsix_in_dir(&fallback_editors_vscode));
    if let Some((_, ref vsix_path)) = discovered_vsix {
        println!("cargo:rerun-if-changed={}", vsix_path.display());
    }

    let mut generated = String::new();
    generated.push_str("// @generated by crates/solix-toolchain/build.rs\n\n");
    generated.push_str(
        "/// Compile-time embedded `bundled_manifest.json` when default archives are present.\n",
    );
    if !packed_packages.is_empty() && manifest_json_path.is_file() {
        let escaped_manifest = escape_rust_string(&manifest_json_path.to_string_lossy());
        let _ = writeln!(
            generated,
            "pub const EMBEDDED_BUNDLED_MANIFEST: Option<&[u8]> = Some(include_bytes!(\"{escaped_manifest}\"));"
        );
    } else {
        generated.push_str("pub const EMBEDDED_BUNDLED_MANIFEST: Option<&[u8]> = None;\n");
    }

    let escaped_packages_pack = escape_rust_string(&packages_pack_path.to_string_lossy());
    generated.push_str(
        "\n/// Compile-time packed binary blob of target-filtered bundled `.slxarc` archives.\n",
    );
    let _ = writeln!(
        generated,
        "pub const EMBEDDED_BUNDLED_PACKAGES_PACK: &[u8] = include_bytes!(\"{escaped_packages_pack}\");\n"
    );

    generated.push_str(
        "/// Compile-time embedded Solix VS Code extension archive as `(filename, bytes)`.\n",
    );
    if let Some((vsix_name, vsix_path)) = discovered_vsix {
        let escaped_name = escape_rust_string(&vsix_name);
        let escaped_vsix = escape_rust_string(&vsix_path.to_string_lossy());
        let _ = writeln!(
            generated,
            "pub const EMBEDDED_VSCODE_VSIX: Option<(&str, &[u8])> = Some((\"{escaped_name}\", include_bytes!(\"{escaped_vsix}\")));"
        );
    } else {
        generated.push_str("pub const EMBEDDED_VSCODE_VSIX: Option<(&str, &[u8])> = None;\n");
    }

    let escaped_wiki_pack = escape_rust_string(&wiki_pack_path.to_string_lossy());
    generated
        .push_str("\n/// Compile-time packed binary blob of offline documentation portal files.\n");
    let _ = writeln!(
        generated,
        "pub const EMBEDDED_WIKI_DOCS_PACK: &[u8] = include_bytes!(\"{escaped_wiki_pack}\");"
    );

    let generated_file_path = out_dir.join("embedded_assets.rs");
    let needs_write = fs::read_to_string(&generated_file_path).ok().as_deref() != Some(&generated);
    if needs_write {
        let _ = fs::write(&generated_file_path, generated);
    }
}

fn main() {
    println!("cargo:rerun-if-changed=assets");

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    generate_embedded_assets(&manifest_dir, &out_dir);

    // OUT_DIR is target/<profile>/build/solix-toolchain-<hash>/out
    // Navigating 3 ancestors up yields target/<profile>
    let target_dir = match out_dir.ancestors().nth(3) {
        Some(dir) => dir,
        None => return,
    };

    let source_assets = manifest_dir.join("assets");
    if source_assets.exists() {
        let target_assets = target_dir.join("assets");
        let _ = copy_dir_all(&source_assets, &target_assets);
    }
}
