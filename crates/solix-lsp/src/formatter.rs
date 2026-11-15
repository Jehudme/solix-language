//! Comment-preserving Solix code formatter powering LSP formatting and the `solix fmt` CLI tool.

#![forbid(unsafe_code)]

use crate::diagnostics::{byte_offset_to_position, position_to_byte_offset};
use crate::workspace::AnalysisSnapshot;
use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Standard indentation unit for Solix source code (4 spaces).
pub const INDENT_SPACES: &str = "    ";

/// Formats an entire Solix source document, returning LSP text edits.
#[must_use]
pub fn format_document(snapshot: &AnalysisSnapshot) -> Vec<TextEdit> {
    let formatted = format_source(&snapshot.source_text);
    if formatted == snapshot.source_text {
        return Vec::new();
    }

    let end_pos = byte_offset_to_position(&snapshot.source_text, snapshot.source_text.len());
    vec![TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: end_pos,
        },
        new_text: formatted,
    }]
}

/// Formats a specific line range within a Solix source document.
#[must_use]
pub fn format_range(snapshot: &AnalysisSnapshot, range: Range) -> Vec<TextEdit> {
    let start_offset = position_to_byte_offset(&snapshot.source_text, range.start);
    let end_offset = position_to_byte_offset(&snapshot.source_text, range.end);

    if start_offset >= end_offset || end_offset > snapshot.source_text.len() {
        return Vec::new();
    }

    let slice = &snapshot.source_text[start_offset..end_offset];
    let formatted = format_source(slice);

    vec![TextEdit {
        range,
        new_text: formatted,
    }]
}

/// Formats raw Solix source code according to the official style guide.
///
/// Indentation: 4 spaces per block level.
/// Spacing: normalizes binary operators, ensures 1 space after commas and semicolons.
/// Comments: preserves all inline and block comments.
#[must_use]
pub fn format_source(source: &str) -> String {
    let mut formatted_lines = Vec::new();
    let mut indent_level = 0usize;
    let mut in_block_comment = false;

    for raw_line in source.lines() {
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            formatted_lines.push(String::new());
            continue;
        }

        // Handle block comments without touching their inner indentation
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            formatted_lines.push(format!("{}{trimmed}", INDENT_SPACES.repeat(indent_level)));
            continue;
        }

        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            formatted_lines.push(format!("{}{trimmed}", INDENT_SPACES.repeat(indent_level)));
            continue;
        }

        // Adjust indent level down before emitting if line starts with closing brace
        let starts_with_closing_brace = trimmed.starts_with('}') || trimmed.starts_with(']');
        if starts_with_closing_brace {
            indent_level = indent_level.saturating_sub(1);
        }

        // Apply statement formatting
        let formatted_content = format_line_content(trimmed);
        formatted_lines.push(format!(
            "{}{formatted_content}",
            INDENT_SPACES.repeat(indent_level)
        ));

        // Adjust indent level for trailing opening braces
        let open_count = trimmed
            .chars()
            .filter(|&character| character == '{')
            .count();
        let close_count = trimmed
            .chars()
            .filter(|&character| character == '}')
            .count();

        if starts_with_closing_brace {
            let remaining_close = close_count.saturating_sub(1);
            if open_count > remaining_close {
                indent_level += open_count - remaining_close;
            } else if remaining_close > open_count {
                indent_level = indent_level.saturating_sub(remaining_close - open_count);
            }
        } else if open_count > close_count {
            indent_level += open_count - close_count;
        } else if close_count > open_count {
            indent_level = indent_level.saturating_sub(close_count - open_count);
        }
    }

    let mut result = formatted_lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn format_line_content(line: &str) -> String {
    // If it's a comment line, return as is
    if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len());
    let mut in_string = false;
    let mut in_char = false;
    let mut previous_char = ' ';

    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let current = chars[index];

        if current == '"' && previous_char != '\\' {
            in_string = !in_string;
            result.push(current);
            previous_char = current;
            index += 1;
            continue;
        }

        if current == '\'' && previous_char != '\\' {
            in_char = !in_char;
            result.push(current);
            previous_char = current;
            index += 1;
            continue;
        }

        if in_string || in_char {
            result.push(current);
            previous_char = current;
            index += 1;
            continue;
        }

        // Collapse consecutive spaces
        if current == ' ' && previous_char == ' ' {
            index += 1;
            continue;
        }

        // Inline comment start
        if current == '/' && index + 1 < chars.len() && chars[index + 1] == '/' {
            result.extend(&chars[index..]);
            break;
        }

        // Comma spacing
        if current == ',' {
            result.push(',');
            if index + 1 < chars.len() && chars[index + 1] != ' ' && chars[index + 1] != '\n' {
                result.push(' ');
            }
            previous_char = ',';
            index += 1;
            continue;
        }

        // Semicolon spacing in for loops
        if current == ';' {
            result.push(';');
            if index + 1 < chars.len()
                && chars[index + 1] != ' '
                && chars[index + 1] != '\n'
                && chars[index + 1] != '}'
            {
                result.push(' ');
            }
            previous_char = ';';
            index += 1;
            continue;
        }

        // Scope resolution :: must have zero spaces
        if current == ':' && index + 1 < chars.len() && chars[index + 1] == ':' {
            if result.ends_with(' ') {
                result.pop();
            }
            result.push_str("::");
            index += 2;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            previous_char = ':';
            continue;
        }

        // Member access dot . must have zero spaces
        if current == '.'
            && !previous_char.is_ascii_digit()
            && (index + 1 >= chars.len() || !chars[index + 1].is_ascii_digit())
        {
            if result.ends_with(' ') {
                result.pop();
            }
            result.push('.');
            index += 1;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            previous_char = '.';
            continue;
        }

        // Lambda arrow '=>'
        if current == '=' && index + 1 < chars.len() && chars[index + 1] == '>' {
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            result.push_str("=>");
            index += 2;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            if index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                result.push(' ');
            }
            previous_char = ' ';
            continue;
        }

        // Equality operator '=='
        if current == '=' && index + 1 < chars.len() && chars[index + 1] == '=' {
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            result.push_str("==");
            index += 2;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            if index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                result.push(' ');
            }
            previous_char = ' ';
            continue;
        }

        // Relational compound operators '!=', '<=', '>='
        if (current == '!' || current == '<' || current == '>')
            && index + 1 < chars.len()
            && chars[index + 1] == '='
        {
            let operator_string = format!("{current}=");
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&operator_string);
            index += 2;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            if index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                result.push(' ');
            }
            previous_char = ' ';
            continue;
        }

        // Shift compound assignment '<<=', '>>='
        if (current == '<'
            && index + 2 < chars.len()
            && chars[index + 1] == '<'
            && chars[index + 2] == '=')
            || (current == '>'
                && index + 2 < chars.len()
                && chars[index + 1] == '>'
                && chars[index + 2] == '=')
        {
            let operator_string = format!("{current}{current}=");
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&operator_string);
            index += 3;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            if index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                result.push(' ');
            }
            previous_char = ' ';
            continue;
        }

        // Arithmetic & bitwise compound assignments '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^='
        if matches!(current, '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^')
            && index + 1 < chars.len()
            && chars[index + 1] == '='
        {
            let operator_string = format!("{current}=");
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&operator_string);
            index += 2;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            if index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                result.push(' ');
            }
            previous_char = ' ';
            continue;
        }

        // Standard assignment '=' operator spacing
        if current == '=' {
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            result.push('=');
            index += 1;
            while index < chars.len() && chars[index] == ' ' {
                index += 1;
            }
            if index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                result.push(' ');
            }
            previous_char = ' ';
            continue;
        }

        // Spacing inside empty parens `( )` -> `()`
        if current == '(' && index + 1 < chars.len() && chars[index + 1] == ' ' {
            let mut scan = index + 1;
            while scan < chars.len() && chars[scan] == ' ' {
                scan += 1;
            }
            if scan < chars.len() && chars[scan] == ')' {
                result.push_str("()");
                index = scan + 1;
                previous_char = ')';
                continue;
            }
        }

        // Ensure space before opening brace '{'
        if current == '{' && !result.ends_with(' ') && !result.is_empty() {
            result.push(' ');
        }

        result.push(current);
        previous_char = current;
        index += 1;
    }

    result
}
