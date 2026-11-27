//! Diagnostic translation and source coordinate conversions for the Solix Language Server.

#![forbid(unsafe_code)]

use solix_common::Span;
use solix_compilation::phases::binder::symbols::SymbolTable;
use solix_compilation::phases::parser::statements::{Declaration, ProgramAst, Statement};
use solix_compilation::{BindingError, LexingError, ParsingError};
use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, DiagnosticTag, NumberOrString, Position,
    Range, Url,
};

/// Base URL for canonical repository documentation of error codes.
pub const ERROR_WIKI_BASE_URL: &str =
    "https://github.com/Jehudme/solix-language/blob/master/docs/wiki/errors/";

/// Converts a 0-based byte offset within a source string into an LSP 0-based [`Position`].
///
/// Note: Characters in LSP [`Position`] are measured in UTF-16 code units according
/// to the Language Server Protocol specification.
#[must_use]
pub fn byte_offset_to_position(source_text: &str, byte_offset: usize) -> Position {
    let clamped_offset = byte_offset.min(source_text.len());
    let mut line_number: u32 = 0;
    let mut character_offset: u32 = 0;

    for (current_byte_index, character) in source_text.char_indices() {
        if current_byte_index >= clamped_offset {
            break;
        }
        if character == '\n' {
            line_number += 1;
            character_offset = 0;
        } else if character != '\r' {
            character_offset += character.len_utf16() as u32;
        }
    }

    Position {
        line: line_number,
        character: character_offset,
    }
}

/// Converts an LSP 0-based [`Position`] into a 0-based byte offset within `source_text`.
#[must_use]
pub fn position_to_byte_offset(source_text: &str, position: Position) -> usize {
    let mut current_line: u32 = 0;
    let mut current_character_utf16: u32 = 0;

    for (byte_index, character) in source_text.char_indices() {
        if current_line == position.line {
            if current_character_utf16 >= position.character {
                return byte_index;
            }
            if character == '\n' {
                return byte_index;
            }
            current_character_utf16 += character.len_utf16() as u32;
        } else if character == '\n' {
            current_line += 1;
            if current_line == position.line && position.character == 0 {
                return byte_index + 1;
            }
        }
    }

    source_text.len()
}

/// Converts a [`Span`] and its associated source text into an LSP [`Range`].
#[must_use]
pub fn span_to_range(source_text: &str, span: Span) -> Range {
    let start_position = byte_offset_to_position(source_text, span.start as usize);
    let end_position = byte_offset_to_position(source_text, span.end as usize);
    Range {
        start: start_position,
        end: end_position,
    }
}

/// Generates a canonical documentation [`CodeDescription`] for an error code and explicit wiki page.
#[must_use]
pub fn make_code_description_for_page(page: &str, error_code: &str) -> Option<CodeDescription> {
    let anchor = error_code.to_ascii_lowercase();
    let target_url = format!("{ERROR_WIKI_BASE_URL}{page}#{anchor}");
    Url::parse(&target_url)
        .ok()
        .map(|parsed_url| CodeDescription { href: parsed_url })
}

/// Generates a canonical documentation [`CodeDescription`] for an error code if valid.
#[must_use]
pub fn make_code_description(error_code: &str) -> Option<CodeDescription> {
    let is_binder_300_code = error_code
        .strip_prefix('E')
        .and_then(|num_str| num_str.parse::<u32>().ok())
        .is_some_and(|code_num| (301..=316).contains(&code_num));
    let page = if error_code.starts_with("E00") {
        "lexer-errors.md"
    } else if error_code.starts_with("E01") {
        "parser-errors.md"
    } else if error_code.starts_with("E02") || is_binder_300_code {
        "binder-errors.md"
    } else if error_code.starts_with("E03") {
        "runtime-errors.md"
    } else {
        "README.md"
    };
    make_code_description_for_page(page, error_code)
}

/// Converts a canonical [`LexingError`] into an LSP [`Diagnostic`].
#[must_use]
pub fn lexing_error_to_diagnostic(source_text: &str, error: &LexingError) -> Diagnostic {
    let source_span = error.source_span();
    let start_offset = source_span.offset();
    let end_offset = start_offset + source_span.len();
    let start_position = byte_offset_to_position(source_text, start_offset);
    let end_position = byte_offset_to_position(source_text, end_offset);
    let error_code = error.error_code();

    Diagnostic {
        range: Range {
            start: start_position,
            end: end_position,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error_code.to_string())),
        code_description: make_code_description_for_page("lexer-errors.md", error_code),
        source: Some("solix".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Converts a canonical [`ParsingError`] into an LSP [`Diagnostic`].
#[must_use]
pub fn parsing_error_to_diagnostic(source_text: &str, error: &ParsingError) -> Diagnostic {
    let source_span = error.source_span();
    let start_offset = source_span.offset();
    let end_offset = start_offset + source_span.len();
    let start_position = byte_offset_to_position(source_text, start_offset);
    let end_position = byte_offset_to_position(source_text, end_offset);
    let error_code = error.error_code();

    Diagnostic {
        range: Range {
            start: start_position,
            end: end_position,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error_code.to_string())),
        code_description: make_code_description_for_page("parser-errors.md", error_code),
        source: Some("solix".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Converts a canonical [`BindingError`] into an LSP [`Diagnostic`].
#[must_use]
pub fn binding_error_to_diagnostic(source_text: &str, error: &BindingError) -> Diagnostic {
    let source_span = error.source_span();
    let start_offset = source_span.offset();
    let end_offset = start_offset + source_span.len();
    let start_position = byte_offset_to_position(source_text, start_offset);
    let end_position = byte_offset_to_position(source_text, end_offset);
    let error_code = error.error_code();

    Diagnostic {
        range: Range {
            start: start_position,
            end: end_position,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error_code.to_string())),
        code_description: make_code_description_for_page("binder-errors.md", error_code),
        source: Some("solix".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Scans the given program AST for unreachable / dead statements following terminal control flow (`return`, `break`, `continue`, `throw`, `exit`).
#[must_use]
pub fn collect_dead_code_diagnostics(source_text: &str, ast: &ProgramAst) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for declaration in &ast.declarations {
        match declaration {
            Declaration::Method(method_declaration) => {
                if let Some(body_block) = &method_declaration.body {
                    inspect_block_for_dead_code(
                        source_text,
                        &body_block.statements,
                        &mut diagnostics,
                    );
                }
            }
            Declaration::Constructor(constructor_declaration) => {
                inspect_block_for_dead_code(
                    source_text,
                    &constructor_declaration.body.statements,
                    &mut diagnostics,
                );
            }
            Declaration::Destructor(destructor_declaration) => {
                inspect_block_for_dead_code(
                    source_text,
                    &destructor_declaration.body.statements,
                    &mut diagnostics,
                );
            }
            Declaration::Operator(operator_declaration) => {
                inspect_block_for_dead_code(
                    source_text,
                    &operator_declaration.body.statements,
                    &mut diagnostics,
                );
            }
            Declaration::Class(class_declaration) => {
                for member in &class_declaration.members {
                    match member {
                        solix_compilation::phases::parser::statements::ClassMember::Method(
                            method_decl,
                        ) => {
                            if let Some(body) = &method_decl.body {
                                inspect_block_for_dead_code(
                                    source_text,
                                    &body.statements,
                                    &mut diagnostics,
                                );
                            }
                        }
                        solix_compilation::phases::parser::statements::ClassMember::Constructor(
                            constructor_decl,
                        ) => {
                            inspect_block_for_dead_code(
                                source_text,
                                &constructor_decl.body.statements,
                                &mut diagnostics,
                            );
                        }
                        solix_compilation::phases::parser::statements::ClassMember::Destructor(
                            destructor_decl,
                        ) => {
                            inspect_block_for_dead_code(
                                source_text,
                                &destructor_decl.body.statements,
                                &mut diagnostics,
                            );
                        }
                        solix_compilation::phases::parser::statements::ClassMember::Operator(
                            operator_decl,
                        ) => {
                            inspect_block_for_dead_code(
                                source_text,
                                &operator_decl.body.statements,
                                &mut diagnostics,
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}

fn inspect_statement_for_dead_code(
    source_text: &str,
    statement: &Statement,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::Block(inner_block) => {
            inspect_block_for_dead_code(source_text, &inner_block.statements, diagnostics);
        }
        Statement::If(if_statement) => {
            inspect_statement_for_dead_code(source_text, &if_statement.then_branch, diagnostics);
            if let Some(else_branch) = &if_statement.else_branch {
                inspect_statement_for_dead_code(source_text, else_branch, diagnostics);
            }
        }
        Statement::While(while_statement) => {
            inspect_statement_for_dead_code(source_text, &while_statement.body, diagnostics);
        }
        Statement::DoWhile(do_while_statement) => {
            inspect_statement_for_dead_code(source_text, &do_while_statement.body, diagnostics);
        }
        Statement::For(for_statement) => {
            inspect_statement_for_dead_code(source_text, &for_statement.body, diagnostics);
        }
        _ => {}
    }
}

fn inspect_block_for_dead_code(
    source_text: &str,
    statements: &[Statement],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut terminated = false;

    for statement in statements {
        if terminated {
            let statement_span = statement.span();
            let diagnostic_range = span_to_range(source_text, statement_span);
            diagnostics.push(Diagnostic {
                range: diagnostic_range,
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("W0001".to_string())),
                code_description: make_code_description("W0001"),
                source: Some("solix".to_string()),
                message: "Unreachable code detected after terminating statement".to_string(),
                related_information: None,
                tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                data: None,
            });
            continue;
        }

        match statement {
            Statement::Return(_)
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Throw(_)
            | Statement::Exit(_) => {
                terminated = true;
            }
            _ => {
                inspect_statement_for_dead_code(source_text, statement, diagnostics);
            }
        }
    }
}

/// Identifies unused private symbols or unused local imports for `DiagnosticTag::UNNECESSARY`.
#[must_use]
pub fn collect_unused_symbols_diagnostics(
    source_text: &str,
    source_id: u16,
    symbol_table: &SymbolTable,
    resolved_symbols: &std::collections::HashMap<
        Span,
        solix_compilation::phases::binder::symbols::SymbolId,
    >,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let used_ids: std::collections::HashSet<_> = resolved_symbols.values().copied().collect();

    for symbol_entry in &symbol_table.symbols {
        if symbol_entry.source_id == source_id
            && symbol_entry.is_private()
            && !used_ids.contains(&symbol_entry.id)
            && symbol_entry.name != "main"
        {
            let range = span_to_range(source_text, symbol_entry.span);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::HINT),
                code: Some(NumberOrString::String("W0002".to_string())),
                code_description: make_code_description("W0002"),
                source: Some("solix".to_string()),
                message: format!(
                    "Private symbol '{}' is declared but never used",
                    symbol_entry.name
                ),
                related_information: None,
                tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                data: None,
            });
        }
    }

    diagnostics
}
