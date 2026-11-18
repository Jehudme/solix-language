//! Executes the `solix wiki` subcommand for offline versioned documentation.

#![forbid(unsafe_code)]

use crate::assets::{export_wiki_docs_to_dir, resolve_wiki_docs_dir};
use crate::cli::WikiArgs;
use crate::commands::doc::{open_in_browser, open_target_in_browser};
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Executes the `solix wiki` command, resolving or exporting the offline documentation portal.
///
/// # Errors
/// Returns an error if embedded documentation could not be resolved or exported cleanly.
pub fn execute_wiki(args: WikiArgs) -> Result<()> {
    let source_dir = resolve_wiki_docs_dir()?;
    let active_dir = if let Some(export_dir) = args.output {
        let exported_count = export_wiki_docs_to_dir(&source_dir, &export_dir)?;
        if !args.path {
            println!(
                "Exported {exported_count} offline documentation file(s) to '{}'.",
                export_dir.display()
            );
        }
        export_dir
    } else {
        source_dir
    };

    let index_path = active_dir.join("index.html");
    let canonical_index = std::fs::canonicalize(&index_path).unwrap_or(index_path);
    let active_query = args
        .search
        .as_deref()
        .or(args.query.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let browser_target = build_browser_launch_target(&canonical_index, active_query)?;

    if args.path {
        println!("{}", canonical_index.display());
    } else if args.no_open {
        println!(
            "Offline Solix v{} documentation ready: {}",
            env!("CARGO_PKG_VERSION"),
            canonical_index.display()
        );
        if let Some(query_text) = active_query {
            println!("Search target: {browser_target} (query: '{query_text}')");
        }
    } else {
        println!(
            "Opening Solix v{} offline wiki: {}",
            env!("CARGO_PKG_VERSION"),
            canonical_index.display()
        );
        if active_query.is_some() {
            open_target_in_browser(&browser_target);
        } else {
            open_in_browser(&canonical_index);
        }
    }

    Ok(())
}

fn build_browser_launch_target(index_path: &Path, query: Option<&str>) -> Result<String> {
    let Some(raw_query) = query else {
        return Ok(index_path.to_string_lossy().into_owned());
    };

    let file_uri = path_to_file_uri(index_path)?;
    let encoded_query = percent_encode_query(raw_query);
    Ok(format!("{file_uri}?q={encoded_query}"))
}

fn path_to_file_uri(path: &Path) -> Result<String> {
    let normalized = normalize_for_file_uri(path);
    let slash_path = normalized.replace('\\', "/");
    let escaped = percent_encode_path(&slash_path);
    if escaped.starts_with('/') {
        Ok(format!("file://{escaped}"))
    } else {
        Ok(format!("file:///{escaped}"))
    }
}

fn normalize_for_file_uri(path: &Path) -> String {
    let raw_string = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        if let Some(stripped) = raw_string.strip_prefix("\\\\?\\UNC\\") {
            return format!("//{stripped}");
        }
        if let Some(stripped) = raw_string.strip_prefix("\\\\?\\") {
            return stripped.to_string();
        }
    }
    raw_string.into_owned()
}

fn percent_encode_path(raw_path: &str) -> String {
    let mut encoded = String::with_capacity(raw_path.len());
    for byte in raw_path.as_bytes() {
        let is_unreserved_or_slash =
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'/' | b':');
        if is_unreserved_or_slash {
            encoded.push(char::from(*byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn percent_encode_query(raw_query: &str) -> String {
    let mut encoded = String::with_capacity(raw_query.len());
    for byte in raw_query.as_bytes() {
        if *byte == b' ' {
            encoded.push_str("%20");
        } else if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(*byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

/// Resolves the path to the root `index.html` of the offline documentation portal.
///
/// # Errors
/// Returns an error if embedded documentation assets are not available.
pub fn resolve_wiki_index_path() -> Result<PathBuf> {
    let docs_dir = resolve_wiki_docs_dir()?;
    let index_path = docs_dir.join("index.html");
    if !index_path.is_file() {
        return Err(anyhow::anyhow!(
            "Offline wiki directory '{}' does not contain index.html",
            docs_dir.display()
        ))
        .context("Failed to locate rendered offline documentation index");
    }
    Ok(index_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode_query_preserves_safe_chars_and_encodes_spaces() {
        let encoded = percent_encode_query("arc weak refs + error E0201?");
        assert_eq!(encoded, "arc%20weak%20refs%20%2B%20error%20E0201%3F");
    }

    #[test]
    fn test_build_browser_launch_target_with_and_without_query() {
        let sample_index = PathBuf::from("/tmp/solix docs/index.html");
        let plain_target = build_browser_launch_target(&sample_index, None);
        assert!(plain_target.is_ok());
        assert_eq!(
            plain_target.unwrap_or_default(),
            "/tmp/solix docs/index.html"
        );

        let search_target = build_browser_launch_target(&sample_index, Some("classes & enums"));
        assert!(search_target.is_ok());
        let search_uri = search_target.unwrap_or_default();
        assert!(search_uri.starts_with("file:///tmp/solix%20docs/index.html?q="));
        assert!(search_uri.ends_with("classes%20%26%20enums"));
    }
}
