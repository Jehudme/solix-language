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

/// Computes the block nesting indentation level at the specified zero-based line index.
#[must_use]
pub fn compute_indent_level_at_line(source_text: &str, target_line_index: u32) -> usize {
    let mut indent_level = 0usize;
    let mut in_block_comment = false;
    for (line_index, raw_line) in source_text.lines().enumerate() {
        if line_index as u32 >= target_line_index {
            break;
        }
        let trimmed_line = raw_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }
        if in_block_comment {
            if trimmed_line.contains("*/") {
                let (_, _) = count_code_braces(trimmed_line, &mut in_block_comment);
            }
            continue;
        }

        let starts_with_closing_brace =
            trimmed_line.starts_with('}') || trimmed_line.starts_with(']');
        if starts_with_closing_brace {
            indent_level = indent_level.saturating_sub(1);
        }

        let (open_brace_count, close_brace_count) =
            count_code_braces(trimmed_line, &mut in_block_comment);

        if starts_with_closing_brace {
            let remaining_close_count = close_brace_count.saturating_sub(1);
            if open_brace_count > remaining_close_count {
                indent_level += open_brace_count - remaining_close_count;
            } else if remaining_close_count > open_brace_count {
                indent_level =
                    indent_level.saturating_sub(remaining_close_count - open_brace_count);
            }
        } else if open_brace_count > close_brace_count {
            indent_level += open_brace_count - close_brace_count;
        } else if close_brace_count > open_brace_count {
            indent_level = indent_level.saturating_sub(close_brace_count - open_brace_count);
        }
    }
    indent_level
}

/// Formats a specific line range within a Solix source document preserving base indentation.
#[must_use]
pub fn format_range(snapshot: &AnalysisSnapshot, range: Range) -> Vec<TextEdit> {
    let start_line = range.start.line;
    let end_line = if range.end.character == 0 && range.end.line > range.start.line {
        range.end.line - 1
    } else {
        range.end.line
    };

    let document_lines: Vec<&str> = snapshot.source_text.lines().collect();
    if start_line as usize >= document_lines.len() {
        return Vec::new();
    }
    let clamped_end_line = (end_line as usize).min(document_lines.len().saturating_sub(1));

    let initial_indent_level = compute_indent_level_at_line(&snapshot.source_text, start_line);
    let selected_slice = document_lines[start_line as usize..=clamped_end_line].join("\n");
    let formatted_text = format_source_with_base_indent(&selected_slice, initial_indent_level);

    let start_offset = position_to_byte_offset(
        &snapshot.source_text,
        Position {
            line: start_line,
            character: 0,
        },
    );
    let end_offset = if clamped_end_line + 1 < document_lines.len() {
        position_to_byte_offset(
            &snapshot.source_text,
            Position {
                line: (clamped_end_line + 1) as u32,
                character: 0,
            },
        )
    } else {
        snapshot.source_text.len()
    };

    let target_range = Range {
        start: Position {
            line: start_line,
            character: 0,
        },
        end: byte_offset_to_position(&snapshot.source_text, end_offset),
    };

    let original_slice = &snapshot.source_text[start_offset..end_offset];
    if original_slice == formatted_text.trim_end_matches('\n') || original_slice == formatted_text {
        return Vec::new();
    }

    vec![TextEdit {
        range: target_range,
        new_text: formatted_text,
    }]
}

/// Formats raw Solix source code according to the official style guide.
///
/// Indentation: 4 spaces per block level.
/// Spacing: normalizes binary operators, ensures 1 space after commas and semicolons.
/// Comments: preserves all inline and block comments.
#[must_use]
pub fn format_source(source: &str) -> String {
    format_source_with_base_indent(source, 0)
}

/// Formats raw Solix source code starting from an initial base indentation level.
#[must_use]
pub fn format_source_with_base_indent(source: &str, initial_indent_level: usize) -> String {
    let mut formatted_lines = Vec::new();
    let mut indent_level = initial_indent_level;
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

        // Adjust indent level for trailing opening braces (lexical-aware, ignoring comments & strings)
        let (open_count, close_count) = count_code_braces(trimmed, &mut in_block_comment);

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

fn count_code_braces(line: &str, in_block_comment: &mut bool) -> (usize, usize) {
    let mut open_braces = 0usize;
    let mut close_braces = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_active = false;

    let chars: Vec<char> = line.chars().collect();
    let mut index = 0usize;

    while index < chars.len() {
        let current = chars[index];

        if *in_block_comment {
            if current == '*' && index + 1 < chars.len() && chars[index + 1] == '/' {
                *in_block_comment = false;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }

        if in_string || in_char {
            if escape_active {
                escape_active = false;
            } else if current == '\\' {
                escape_active = true;
            } else if in_string && current == '"' {
                in_string = false;
            } else if in_char && current == '\'' {
                in_char = false;
            }
            index += 1;
            continue;
        }

        if current == '/' && index + 1 < chars.len() {
            if chars[index + 1] == '/' {
                break;
            }
            if chars[index + 1] == '*' {
                *in_block_comment = true;
                index += 2;
                continue;
            }
        }

        if current == '"' {
            in_string = true;
        } else if current == '\'' {
            in_char = true;
        } else if current == '{' {
            open_braces += 1;
        } else if current == '}' {
            close_braces += 1;
        }
        index += 1;
    }

    (open_braces, close_braces)
}

fn format_line_content(line: &str) -> String {
    // If it's a comment line, return as is
    if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len());
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_active = false;
    let mut previous_char = ' ';

    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let current = chars[index];

        if in_string || in_char {
            result.push(current);
            if escape_active {
                escape_active = false;
            } else if current == '\\' {
                escape_active = true;
            } else if in_string && current == '"' {
                in_string = false;
            } else if in_char && current == '\'' {
                in_char = false;
            }
            previous_char = current;
            index += 1;
            continue;
        }

        if current == '"' {
            in_string = true;
            result.push(current);
            previous_char = current;
            index += 1;
            continue;
        }

        if current == '\'' {
            in_char = true;
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
