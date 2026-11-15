//! Semantic highlighting driven by lexical tokens and semantic binder classifications.

#![forbid(unsafe_code)]

use crate::diagnostics::byte_offset_to_position;
use crate::workspace::AnalysisSnapshot;
use solix_common::tokens::TokenKind;
use solix_common::types::Type;
use tower_lsp::lsp_types::{
    Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend, SemanticTokensRangeResult, SemanticTokensResult,
};

/// Canonical semantic token types supported by the Solix Language Server.
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,   // 0
    SemanticTokenType::TYPE,        // 1
    SemanticTokenType::CLASS,       // 2
    SemanticTokenType::INTERFACE,   // 3
    SemanticTokenType::ENUM,        // 4
    SemanticTokenType::ENUM_MEMBER, // 5
    SemanticTokenType::FUNCTION,    // 6
    SemanticTokenType::METHOD,      // 7
    SemanticTokenType::PROPERTY,    // 8
    SemanticTokenType::VARIABLE,    // 9
    SemanticTokenType::PARAMETER,   // 10
    SemanticTokenType::KEYWORD,     // 11
    SemanticTokenType::MODIFIER,    // 12
    SemanticTokenType::COMMENT,     // 13
    SemanticTokenType::STRING,      // 14
    SemanticTokenType::NUMBER,      // 15
    SemanticTokenType::OPERATOR,    // 16
];

/// Canonical semantic token modifiers supported by the Solix Language Server.
pub const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,     // 1 << 0
    SemanticTokenModifier::STATIC,          // 1 << 1
    SemanticTokenModifier::READONLY,        // 1 << 2
    SemanticTokenModifier::DEFAULT_LIBRARY, // 1 << 3
];

/// Constructs the official LSP [`SemanticTokensLegend`] for Solix.
#[must_use]
pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: TOKEN_MODIFIERS.to_vec(),
    }
}

/// Computes full-document semantic tokens.
#[must_use]
pub fn semantic_tokens_full(snapshot: &AnalysisSnapshot) -> Option<SemanticTokensResult> {
    let tokens = compute_semantic_tokens(snapshot, None);
    Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    }))
}

/// Computes range-bounded semantic tokens.
#[must_use]
pub fn semantic_tokens_range(
    snapshot: &AnalysisSnapshot,
    range: Range,
) -> Option<SemanticTokensRangeResult> {
    let tokens = compute_semantic_tokens(snapshot, Some(range));
    Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    }))
}

#[derive(Debug, Clone, Copy)]
struct RawSemanticToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

fn compute_semantic_tokens(
    snapshot: &AnalysisSnapshot,
    range_filter: Option<Range>,
) -> Vec<SemanticToken> {
    let mut raw_tokens = Vec::new();
    let semantic = snapshot.last_good_semantic.as_ref();

    for token in &snapshot.tokens {
        let start_pos = byte_offset_to_position(&snapshot.source_text, token.span.start as usize);
        let token_len = token.span.end.saturating_sub(token.span.start);

        if let Some(range) = range_filter
            && (start_pos.line < range.start.line || start_pos.line > range.end.line)
        {
            continue;
        }

        let mut token_type_index = None;
        let mut modifier_mask = 0u32;

        match token.kind {
            TokenKind::Identifier => {
                // Check if resolved by binder
                if let Some(sem) = semantic
                    && let Some(&symbol_id) = sem.resolved_symbols.get(&token.span)
                    && let Some(symbol) = sem.symbol_table.get_symbol(symbol_id)
                {
                    if symbol.is_static {
                        modifier_mask |= 1 << 1; // static
                    }
                    if symbol.is_const {
                        modifier_mask |= 1 << 2; // readonly
                    }

                    token_type_index = match &symbol.symbol_type {
                        Type::Class { .. } => Some(2), // class
                        Type::FunctionPointer { .. } => {
                            if symbol.parent_symbol.is_some() {
                                Some(7) // method
                            } else {
                                Some(6) // function
                            }
                        }
                        _ => {
                            if symbol.parent_symbol.is_some() {
                                Some(8) // property/field
                            } else {
                                Some(9) // variable
                            }
                        }
                    };
                }
            }
            TokenKind::String => {
                token_type_index = Some(14); // string
            }
            TokenKind::Char => {
                token_type_index = Some(14); // string
            }
            TokenKind::Number => {
                token_type_index = Some(15); // number
            }
            TokenKind::CommentSingleLine | TokenKind::CommentMultiLine => {
                token_type_index = Some(13); // comment
            }
            _ => {
                if token.kind.is_keyword() {
                    let text = get_span_text(&snapshot.source_text, token.span);
                    if text == "class" || text == "interface" || text == "enum" {
                        token_type_index = Some(11); // keyword
                    } else if text == "public"
                        || text == "private"
                        || text == "static"
                        || text == "virtual"
                        || text == "override"
                    {
                        token_type_index = Some(12); // modifier
                    } else {
                        token_type_index = Some(11); // keyword
                    }
                } else if token.kind.is_operator() {
                    token_type_index = Some(16); // operator
                }
            }
        }

        if let Some(type_idx) = token_type_index {
            raw_tokens.push(RawSemanticToken {
                line: start_pos.line,
                start_char: start_pos.character,
                length: token_len,
                token_type: type_idx,
                modifiers: modifier_mask,
            });
        }
    }

    // Sort tokens strictly: line first, then start_char
    raw_tokens.sort_by(|a, b| {
        a.line
            .cmp(&b.line)
            .then_with(|| a.start_char.cmp(&b.start_char))
    });

    // Delta-encode
    let mut encoded = Vec::new();
    let mut previous_line = 0u32;
    let mut previous_start = 0u32;

    for raw in raw_tokens {
        let delta_line = raw.line - previous_line;
        let delta_start = if delta_line == 0 {
            raw.start_char.saturating_sub(previous_start)
        } else {
            raw.start_char
        };

        encoded.push(SemanticToken {
            delta_line,
            delta_start,
            length: raw.length,
            token_type: raw.token_type,
            token_modifiers_bitset: raw.modifiers,
        });

        previous_line = raw.line;
        previous_start = raw.start_char;
    }

    encoded
}

fn get_span_text(source: &str, span: solix_common::Span) -> &str {
    let start = span.start as usize;
    let end = span.end as usize;
    if start <= end && end <= source.len() {
        &source[start..end]
    } else {
        ""
    }
}
