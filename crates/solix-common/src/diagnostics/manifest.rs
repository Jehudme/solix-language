//! High-fidelity manifest parsing and validation diagnostics.

#![forbid(unsafe_code)]

use miette::Diagnostic;

/// Structured manifest parsing and validation errors with source location, field breadcrumb paths,
/// and compiler-grade formatted visual snippets.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Diagnostic)]
pub enum ManifestError {
    /// Deserialization or validation error with precise location and context.
    #[error("Failed to parse {filename}: {message}\n{snippet}")]
    #[diagnostic(
        code(solix::manifest::parse_error),
        help("Ensure the manifest follows the valid Solix manifest.yaml schema.")
    )]
    ParseFailed {
        filename: String,
        line: Option<usize>,
        column: Option<usize>,
        field_path: Option<String>,
        message: String,
        snippet: String,
    },

    /// Profile uses removed `inherits` keyword.
    #[error(
        "Failed to parse {filename}: profile '{profile_name}' uses 'inherits', which has been removed; configure each profile explicitly or use YAML anchors (& / *)\n{snippet}"
    )]
    #[diagnostic(
        code(solix::manifest::deprecated_inherits),
        help("Configure each profile explicitly or use YAML anchors (& / *).")
    )]
    DeprecatedInherits {
        filename: String,
        profile_name: String,
        line: Option<usize>,
        column: Option<usize>,
        snippet: String,
    },

    /// Manifest is missing mandatory `profiles.release`.
    #[error(
        "Failed to parse {filename}: manifest must define mandatory profiles.release profile\n{snippet}"
    )]
    #[diagnostic(
        code(solix::manifest::missing_release_profile),
        help("Define at least a 'release' profile under 'profiles' in manifest.yaml.")
    )]
    MissingReleaseProfile {
        filename: String,
        line: Option<usize>,
        column: Option<usize>,
        snippet: String,
    },

    /// Unknown field at top-level.
    #[error(
        "Failed to parse {filename}: unknown field `{field_name}` at root level, expected one of {expected}\n{snippet}"
    )]
    #[diagnostic(
        code(solix::manifest::unknown_field),
        help("Remove unrecognized keys or prefix custom vendor keys with 'x-'.")
    )]
    UnknownRootField {
        filename: String,
        field_name: String,
        expected: &'static str,
        line: Option<usize>,
        column: Option<usize>,
        snippet: String,
    },

    /// IO error reading manifest.
    #[error("Failed to read manifest file '{path}': {reason}")]
    #[diagnostic(
        code(solix::manifest::io_error),
        help("Check that the manifest file exists and has read permissions.")
    )]
    IoError { path: String, reason: String },
}

impl ManifestError {
    /// Returns the standardized compiler error code string.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::ParseFailed { .. } => "E0401",
            Self::DeprecatedInherits { .. } => "E0402",
            Self::MissingReleaseProfile { .. } => "E0403",
            Self::UnknownRootField { .. } => "E0404",
            Self::IoError { .. } => "E0405",
        }
    }

    /// Accessor for 1-indexed source line if available.
    #[must_use]
    pub const fn line(&self) -> Option<usize> {
        match self {
            Self::ParseFailed { line, .. }
            | Self::DeprecatedInherits { line, .. }
            | Self::MissingReleaseProfile { line, .. }
            | Self::UnknownRootField { line, .. } => *line,
            Self::IoError { .. } => None,
        }
    }

    /// Accessor for 1-indexed source column if available.
    #[must_use]
    pub const fn column(&self) -> Option<usize> {
        match self {
            Self::ParseFailed { column, .. }
            | Self::DeprecatedInherits { column, .. }
            | Self::MissingReleaseProfile { column, .. }
            | Self::UnknownRootField { column, .. } => *column,
            Self::IoError { .. } => None,
        }
    }

    /// Accessor for field breadcrumb path if available.
    #[must_use]
    pub fn field_path(&self) -> Option<&str> {
        match self {
            Self::ParseFailed { field_path, .. } => field_path.as_deref(),
            _ => None,
        }
    }
}

/// Extracts the hierarchical key path for a given line in a YAML document.
///
/// Uses indentation-based hierarchy traversal backwards from `target_line`.
#[must_use]
pub fn extract_yaml_breadcrumb_path(yaml_content: &str, target_line: usize) -> Vec<String> {
    let source_lines: Vec<&str> = yaml_content.lines().collect();
    if target_line == 0 || target_line > source_lines.len() {
        return Vec::new();
    }

    let mut current_line_idx = target_line - 1;
    let mut current_indent = None;
    let mut path_segments = Vec::new();

    let target_str = source_lines[current_line_idx];
    let trimmed = target_str.trim();
    if !trimmed.is_empty() && !trimmed.starts_with('#') {
        let indent = target_str.len() - target_str.trim_start().len();
        current_indent = Some(indent);

        if let Some(colon_pos) = trimmed.find(':') {
            let potential_key = trimmed[..colon_pos].trim();
            if !potential_key.is_empty()
                && !potential_key.starts_with('-')
                && !potential_key.contains(' ')
            {
                path_segments.push(potential_key.to_string());
            }
        }
    }

    let mut required_indent = current_indent.unwrap_or(usize::MAX);

    while current_line_idx > 0 {
        current_line_idx -= 1;
        let line_str = source_lines[current_line_idx];
        let trimmed_line = line_str.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        let line_indent = line_str.len() - line_str.trim_start().len();
        if line_indent < required_indent
            && let Some(colon_pos) = trimmed_line.find(':')
        {
            let potential_key = trimmed_line[..colon_pos].trim();
            if !potential_key.is_empty()
                && !potential_key.starts_with('-')
                && !potential_key.contains(' ')
            {
                path_segments.push(potential_key.to_string());
                required_indent = line_indent;
                if line_indent == 0 {
                    break;
                }
            }
        }
    }

    path_segments.reverse();
    path_segments
}

/// Enriches a raw serde deserialization message with hierarchical breadcrumbs.
#[must_use]
pub fn enrich_manifest_error_message(
    raw_message: &str,
    breadcrumb_path: &[String],
) -> (String, Option<String>) {
    if breadcrumb_path.is_empty() {
        return (raw_message.to_string(), None);
    }

    let mut path_field_str = breadcrumb_path.join(".");

    // Extract potential field name from raw message
    let extracted_missing_field = if let Some(prefix_pos) = raw_message.find("missing field `") {
        let remainder = &raw_message[prefix_pos + "missing field `".len()..];
        remainder.find('`').map(|end_pos| &remainder[..end_pos])
    } else {
        None
    };

    let extracted_unknown_field = if let Some(prefix_pos) = raw_message.find("unknown field `") {
        let remainder = &raw_message[prefix_pos + "unknown field `".len()..];
        remainder.find('`').map(|end_pos| &remainder[..end_pos])
    } else {
        None
    };

    // Check for profiles.<profile_name>.dependencies.<dep_name>
    if breadcrumb_path.len() >= 4
        && breadcrumb_path[0] == "profiles"
        && breadcrumb_path[2] == "dependencies"
    {
        let profile_name = &breadcrumb_path[1];
        let dep_name = &breadcrumb_path[3];
        path_field_str = format!("profiles.{profile_name}.dependencies.{dep_name}");

        if let Some(field_name) = extracted_missing_field {
            return (
                format!(
                    "missing required field `{field_name}` for dependency '{dep_name}' in profile '{profile_name}'"
                ),
                Some(path_field_str),
            );
        }
        if let Some(field_name) = extracted_unknown_field {
            return (
                format!(
                    "unknown field `{field_name}` for dependency '{dep_name}' in profile '{profile_name}'"
                ),
                Some(path_field_str),
            );
        }
        if raw_message.contains("invalid type") {
            return (
                format!(
                    "invalid type for dependency '{dep_name}' in profile '{profile_name}': {raw_message}"
                ),
                Some(path_field_str),
            );
        }
    }

    // Check for profiles.<profile_name>.natives.<native_name>
    if breadcrumb_path.len() >= 4
        && breadcrumb_path[0] == "profiles"
        && breadcrumb_path[2] == "natives"
    {
        let profile_name = &breadcrumb_path[1];
        let native_name = &breadcrumb_path[3];
        path_field_str = format!("profiles.{profile_name}.natives.{native_name}");

        if let Some(field_name) = extracted_missing_field {
            return (
                format!(
                    "missing required field `{field_name}` for native target '{native_name}' in profile '{profile_name}'"
                ),
                Some(path_field_str),
            );
        }
        if let Some(field_name) = extracted_unknown_field {
            return (
                format!(
                    "unknown field `{field_name}` for native target '{native_name}' in profile '{profile_name}'"
                ),
                Some(path_field_str),
            );
        }
        if raw_message.contains("invalid type") {
            return (
                format!(
                    "invalid type for native target '{native_name}' in profile '{profile_name}': {raw_message}"
                ),
                Some(path_field_str),
            );
        }
    }

    // Check for profiles.<profile_name>
    if breadcrumb_path.len() >= 2 && breadcrumb_path[0] == "profiles" {
        let profile_name = &breadcrumb_path[1];
        path_field_str = format!("profiles.{profile_name}");

        if let Some(field_name) = extracted_missing_field {
            return (
                format!("missing required field `{field_name}` in profile '{profile_name}'"),
                Some(path_field_str),
            );
        }
        if let Some(field_name) = extracted_unknown_field {
            return (
                format!("unknown field `{field_name}` in profile '{profile_name}'"),
                Some(path_field_str),
            );
        }
    }

    // Check for configuration
    if !breadcrumb_path.is_empty() && breadcrumb_path[0] == "configuration" {
        path_field_str = "configuration".to_string();

        if let Some(field_name) = extracted_missing_field {
            return (
                format!("missing required field `{field_name}` in project 'configuration'"),
                Some(path_field_str),
            );
        }
        if let Some(field_name) = extracted_unknown_field {
            return (
                format!("unknown field `{field_name}` in project 'configuration'"),
                Some(path_field_str),
            );
        }
    }

    (
        format!("{raw_message} at '{path_field_str}'"),
        Some(path_field_str),
    )
}

/// Renders a compiler-grade source snippet pointing to an exact line and column.
#[must_use]
pub fn render_source_snippet(
    yaml_content: &str,
    filename: &str,
    target_line: usize,
    target_column: usize,
    label: &str,
) -> String {
    let source_lines: Vec<&str> = yaml_content.lines().collect();
    if source_lines.is_empty() {
        return format!("  --> {filename}:{target_line}:{target_column}");
    }

    let mut output = String::new();
    output.push_str(&format!("  --> {filename}:{target_line}:{target_column}\n"));

    let start_line = if target_line > 2 { target_line - 2 } else { 1 };
    let end_line = std::cmp::min(target_line + 1, source_lines.len());

    let max_line_str = format!("{end_line}");
    let gutter_width = max_line_str.len();

    let indent_space = " ".repeat(gutter_width);
    output.push_str(&format!("   {indent_space}|\n"));

    for current_line_num in start_line..=end_line {
        if current_line_num > source_lines.len() {
            break;
        }
        let line_content = source_lines[current_line_num - 1];
        let line_num_str = format!("{current_line_num}");
        let padding = " ".repeat(gutter_width - line_num_str.len());
        output.push_str(&format!(" {padding}{line_num_str} | {line_content}\n"));

        if current_line_num == target_line {
            let pointer_indent = if target_column > 0 {
                target_column - 1
            } else {
                0
            };
            let pointer_spaces = " ".repeat(pointer_indent);
            output.push_str(&format!("   {indent_space}| {pointer_spaces}^ {label}\n"));
        }
    }

    output.trim_end().to_string()
}
