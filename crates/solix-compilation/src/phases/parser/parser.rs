//! Recursive-Descent AST Parser and Precedence-Climbing Expression Engine for Solix.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::phases::lexing::{Span, Token, TokenKind};
use crate::phases::parser::errors::ParsingError;
use crate::phases::parser::statements::*;
use std::time::{Duration, Instant};

/// Metadata and statistics summarizing the completion of the parsing phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AstMetaData {
    /// Number of source files parsed into ASTs.
    pub files_parsed: usize,
    /// Total count of top-level declarations parsed.
    pub declarations_count: usize,
    /// Total count of statements parsed across all files.
    pub statements_count: usize,
    /// Wall-clock duration of the parsing phase.
    pub duration: Duration,
}

/// Tokenizes and parses all source files in the compilation context into Abstract Syntax Trees.
///
/// # Errors
/// Returns an accumulated list of all syntax errors across all files if parsing failed.
pub fn parse_context(context: &mut CompilationContext) -> Result<AstMetaData, Vec<ParsingError>> {
    let start_time = Instant::now();
    let mut total_declarations = 0;
    let mut total_statements = 0;
    let mut accumulated_errors = Vec::new();

    let sources_count = context.sources.len();
    if context.ast.len() < sources_count {
        context.ast.resize_with(sources_count, || None);
    }

    for source_index in 0..sources_count {
        let source_id = source_index as u16;
        let source_text = &context.sources[source_index];
        let tokens: &[Token] = context
            .tokens
            .get(source_index)
            .map(|tok_vec| tok_vec.as_slice())
            .unwrap_or(&[]);

        match parse_source(source_id, tokens, source_text) {
            Ok(ast) => {
                total_declarations += ast.declarations.len();
                total_statements += ast.root_statements.len();
                context.ast[source_index] = Some(ast);
            }
            Err(errors) => {
                context.ast[source_index] = None;
                accumulated_errors.extend(errors);
            }
        }
    }

    if !accumulated_errors.is_empty() {
        return Err(accumulated_errors);
    }

    Ok(AstMetaData {
        files_parsed: sources_count,
        declarations_count: total_declarations,
        statements_count: total_statements,
        duration: start_time.elapsed(),
    })
}

/// Tokenizes and parses all source files in the compilation context into Abstract Syntax Trees.
///
/// Alias for [`parse_context`].
///
/// # Errors
/// Returns an accumulated list of all syntax errors across all files if parsing failed.
#[inline]
pub fn parse(context: &mut CompilationContext) -> Result<AstMetaData, Vec<ParsingError>> {
    parse_context(context)
}

/// Parses a token stream from a single source file into an Abstract Syntax Tree.
///
/// # Errors
/// Returns an accumulated list of [`ParsingError`] diagnostics if syntax errors were encountered.
pub fn parse_source(
    source_id: u16,
    tokens: &[Token],
    source_text: &str,
) -> Result<ProgramAst, Vec<ParsingError>> {
    let mut state = ParserState::new(source_id, tokens, source_text);
    let ast = state.parse_program();

    if !state.errors.is_empty() {
        return Err(state.errors);
    }

    Ok(ast)
}

/// Parses a token stream from a single source file in resilient mode, always returning the parsed AST.
#[must_use]
pub fn parse_source_resilient(
    source_id: u16,
    tokens: &[Token],
    source_text: &str,
) -> (ProgramAst, Vec<ParsingError>) {
    let mut state = ParserState::new(source_id, tokens, source_text);
    let ast = state.parse_program();
    (ast, state.errors)
}

/// Parses all sources in the compilation context in resilient mode, always populating `context.ast`.
pub fn parse_context_resilient(context: &mut CompilationContext) -> Vec<ParsingError> {
    let mut accumulated_errors = Vec::new();
    let sources_count = context.sources.len();
    if context.ast.len() < sources_count {
        context.ast.resize_with(sources_count, || None);
    }

    for source_index in 0..sources_count {
        let source_id = source_index as u16;
        let source_text = &context.sources[source_index];
        let tokens: &[Token] = context
            .tokens
            .get(source_index)
            .map(|tok_vec| tok_vec.as_slice())
            .unwrap_or(&[]);

        let (ast, errors) = parse_source_resilient(source_id, tokens, source_text);
        context.ast[source_index] = Some(ast);
        accumulated_errors.extend(errors);
    }

    accumulated_errors
}

/// Token stream cursor providing checked lookahead and consumption.
struct TokenCursor<'tokens> {
    tokens: &'tokens [Token],
    position: usize,
    pending_greater_thans: usize,
}

impl<'tokens> TokenCursor<'tokens> {
    fn new(tokens: &'tokens [Token]) -> Self {
        Self {
            tokens,
            position: 0,
            pending_greater_thans: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.pending_greater_thans > 0 {
            self.previous()
        } else {
            self.tokens.get(self.position)
        }
    }

    fn peek_kind(&self) -> TokenKind {
        if self.pending_greater_thans > 0 {
            TokenKind::OperatorGreaterThan
        } else {
            self.peek()
                .map(|token| token.kind)
                .unwrap_or(TokenKind::Eof)
        }
    }

    fn peek_next(&self) -> Option<&Token> {
        if self.pending_greater_thans > 0 {
            self.tokens.get(self.position)
        } else {
            self.tokens.get(self.position + 1)
        }
    }

    fn peek_next_kind(&self) -> TokenKind {
        self.peek_next()
            .map(|token| token.kind)
            .unwrap_or(TokenKind::Eof)
    }

    fn peek_nth(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }

    fn previous(&self) -> Option<&Token> {
        if self.position > 0 {
            self.tokens.get(self.position - 1)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.pending_greater_thans > 0 {
            self.pending_greater_thans -= 1;
            return self.previous();
        }
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        if self.pending_greater_thans > 0 {
            false
        } else {
            self.peek_kind() == TokenKind::Eof || self.position >= self.tokens.len()
        }
    }

    fn check(&self, expected_kind: TokenKind) -> bool {
        if expected_kind == TokenKind::OperatorGreaterThan {
            if self.pending_greater_thans > 0 {
                return true;
            }
            if let Some(token) = self.tokens.get(self.position) {
                return token.kind == TokenKind::OperatorGreaterThan
                    || token.kind == TokenKind::OperatorShiftRight;
            }
            return false;
        }
        if self.is_at_end() {
            expected_kind == TokenKind::Eof
        } else {
            self.peek_kind() == expected_kind
        }
    }

    fn check_next(&self, expected_kind: TokenKind) -> bool {
        self.peek_next_kind() == expected_kind
    }

    fn check_nth(&self, offset: usize, expected_kind: TokenKind) -> bool {
        self.tokens
            .get(self.position + offset)
            .map(|tok| tok.kind == expected_kind)
            .unwrap_or(false)
    }

    fn match_token(&mut self, expected_kind: TokenKind) -> Option<&Token> {
        if expected_kind == TokenKind::OperatorGreaterThan {
            if self.pending_greater_thans > 0 {
                self.pending_greater_thans -= 1;
                return self.previous();
            }
            if let Some(token) = self.tokens.get(self.position) {
                if token.kind == TokenKind::OperatorGreaterThan {
                    return self.advance();
                } else if token.kind == TokenKind::OperatorShiftRight {
                    self.advance();
                    self.pending_greater_thans = 1;
                    return self.previous();
                }
            }
            return None;
        }
        if self.check(expected_kind) {
            self.advance()
        } else {
            None
        }
    }

    fn current_span(&self) -> Span {
        self.peek()
            .map(|token| token.span)
            .unwrap_or_else(|| self.previous().map(|token| token.span).unwrap_or_default())
    }

    fn previous_span(&self) -> Span {
        self.previous()
            .map(|token| token.span)
            .unwrap_or_else(|| self.current_span())
    }
}

/// Stateful recursive-descent parser engine.
struct ParserState<'tokens, 'source> {
    cursor: TokenCursor<'tokens>,
    source_id: u16,
    source_text: &'source str,
    errors: Vec<ParsingError>,
    scope_depth: usize,
    inside_class: bool,
    inside_extern_class: bool,
    inside_generic_class: bool,
    inside_interface: bool,
    enclosing_class_name: Option<String>,
    has_parsed_package: bool,
    has_parsed_declaration: bool,
    destructors_in_class: usize,
}

impl<'tokens, 'source> ParserState<'tokens, 'source> {
    fn new(source_id: u16, tokens: &'tokens [Token], source_text: &'source str) -> Self {
        Self {
            cursor: TokenCursor::new(tokens),
            source_id,
            source_text,
            errors: Vec::new(),
            scope_depth: 0,
            inside_class: false,
            inside_extern_class: false,
            inside_generic_class: false,
            inside_interface: false,
            enclosing_class_name: None,
            has_parsed_package: false,
            has_parsed_declaration: false,
            destructors_in_class: 0,
        }
    }

    fn get_lexeme(&self, span: Span) -> &'source str {
        let start = span.start as usize;
        let end = span.end as usize;
        if start <= end && end <= self.source_text.len() {
            &self.source_text[start..end]
        } else {
            ""
        }
    }

    fn record_error(&mut self, error: ParsingError) {
        self.errors.push(error);
    }

    fn synchronize(&mut self) {
        if self.cursor.check(TokenKind::PunctuationCloseBrace) {
            return;
        }
        self.cursor.advance();
        while !self.cursor.is_at_end() {
            if matches!(self.cursor.previous(), Some(prev) if prev.kind == TokenKind::PunctuationSemicolon)
            {
                return;
            }
            match self.cursor.peek_kind() {
                TokenKind::KeywordClass
                | TokenKind::KeywordInterface
                | TokenKind::KeywordEnum
                | TokenKind::KeywordPackage
                | TokenKind::KeywordImport
                | TokenKind::KeywordAlias
                | TokenKind::KeywordFor
                | TokenKind::KeywordIf
                | TokenKind::KeywordWhile
                | TokenKind::KeywordReturn
                | TokenKind::KeywordTry
                | TokenKind::PunctuationCloseBrace => return,
                _ => {
                    self.cursor.advance();
                }
            }
        }
    }

    fn parse_program(&mut self) -> ProgramAst {
        let file_span = self.cursor.current_span();
        let mut program = ProgramAst::new(self.source_id, file_span);

        while !self.cursor.is_at_end() {
            match self.cursor.peek_kind() {
                TokenKind::KeywordPackage => match self.parse_package_statement() {
                    Ok(package_stmt) => {
                        if program.package.is_some() {
                            self.record_error(ParsingError::DuplicatePackageStatement {
                                span: package_stmt.span.to_source_span(),
                            });
                        } else if self.has_parsed_declaration || !program.imports.is_empty() {
                            self.record_error(ParsingError::PackageStatementNotFirst {
                                span: package_stmt.span.to_source_span(),
                            });
                            program.package = Some(package_stmt);
                            self.has_parsed_package = true;
                        } else {
                            program.package = Some(package_stmt);
                            self.has_parsed_package = true;
                        }
                    }
                    Err(error) => {
                        self.record_error(error);
                        self.synchronize();
                    }
                },
                TokenKind::KeywordImport => match self.parse_import_statement(false, None) {
                    Ok(import_stmt) => {
                        if self.has_parsed_declaration {
                            self.record_error(ParsingError::ImportAfterDeclaration {
                                span: import_stmt.span.to_source_span(),
                            });
                        }
                        program.imports.push(import_stmt);
                    }
                    Err(error) => {
                        self.record_error(error);
                        self.synchronize();
                    }
                },
                TokenKind::KeywordAlias => match self.parse_alias_statement() {
                    Ok(alias_stmt) => {
                        program.aliases.push(alias_stmt);
                    }
                    Err(error) => {
                        self.record_error(error);
                        self.synchronize();
                    }
                },
                _ => match self.parse_top_level_element() {
                    Ok(TopLevelElement::Declaration(decl)) => {
                        self.has_parsed_declaration = true;
                        program.declarations.push(decl);
                    }
                    Ok(TopLevelElement::Statement(stmt)) => {
                        program.root_statements.push(stmt);
                    }
                    Ok(TopLevelElement::Import(import_stmt)) => {
                        if self.has_parsed_declaration {
                            self.record_error(ParsingError::ImportAfterDeclaration {
                                span: import_stmt.span.to_source_span(),
                            });
                        }
                        program.imports.push(import_stmt);
                    }
                    Err(error) => {
                        self.record_error(error);
                        self.synchronize();
                    }
                },
            }
        }

        program
    }

    fn parse_package_statement(&mut self) -> Result<PackageStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'package'

        let mut path = Vec::new();
        let first_ident = self.consume_identifier()?;
        path.push(first_ident);

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::PunctuationDot | TokenKind::PunctuationDoubleColon
        ) {
            self.cursor.advance();
            let segment = self.consume_identifier()?;
            path.push(segment);
        }

        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(PackageStatement { span, path })
    }

    fn parse_import_statement(
        &mut self,
        is_extern: bool,
        extern_span: Option<Span>,
    ) -> Result<ImportStatement, ParsingError> {
        let start_span = extern_span.unwrap_or_else(|| self.cursor.current_span());
        self.cursor.advance(); // consume 'import'

        let mut path = Vec::new();
        let mut is_wildcard = false;

        let first_ident = self.consume_identifier()?;
        path.push(first_ident);

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::PunctuationDot | TokenKind::PunctuationDoubleColon
        ) {
            self.cursor.advance();
            if self
                .cursor
                .match_token(TokenKind::OperatorMultiply)
                .is_some()
            {
                is_wildcard = true;
                break;
            }
            let segment = self.consume_identifier()?;
            path.push(segment);
        }

        let alias = if self.cursor.check(TokenKind::Identifier)
            && self.get_lexeme(self.cursor.current_span()) == "as"
        {
            self.cursor.advance();
            Some(self.consume_identifier()?)
        } else {
            None
        };

        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(ImportStatement {
            span,
            path,
            is_wildcard,
            alias,
            is_extern,
        })
    }

    fn parse_alias_statement(&mut self) -> Result<AliasStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'alias'

        let (namespace_path, name_span) = self.parse_qualified_name()?;
        let mut generic_params = Vec::new();

        if self
            .cursor
            .match_token(TokenKind::OperatorLessThan)
            .is_some()
        {
            loop {
                generic_params.push(self.consume_identifier()?);
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
            if self
                .cursor
                .match_token(TokenKind::OperatorGreaterThan)
                .is_none()
            {
                return Err(ParsingError::ExpectedCloseAngle {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
        }

        if self.cursor.match_token(TokenKind::OperatorAssign).is_none() {
            return Err(ParsingError::MalformedAlias {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let target_type = self.parse_type_annotation()?;
        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(AliasStatement {
            span,
            namespace_path,
            name_span,
            generic_params,
            target_type,
        })
    }

    fn parse_top_level_element(&mut self) -> Result<TopLevelElement, ParsingError> {
        let modifiers = self.parse_modifiers();

        if !modifiers.is_empty()
            && matches!(
                self.cursor.peek_kind(),
                TokenKind::KeywordPackage | TokenKind::KeywordImport | TokenKind::KeywordAlias
            )
        {
            if modifiers.len() == 1
                && modifiers[0] == Modifier::Extern
                && self.cursor.peek_kind() == TokenKind::KeywordImport
            {
                let import_statement = self.parse_import_statement(true, None)?;
                return Ok(TopLevelElement::Import(import_statement));
            }
            let span = self.cursor.current_span();
            return Err(ParsingError::ModifierOnModuleStatement {
                span: span.to_source_span(),
            });
        }

        if modifiers.is_empty() && self.cursor.check(TokenKind::PunctuationSemicolon) {
            let semi = self.cursor.advance().copied().unwrap_or_default();
            return Err(ParsingError::UnexpectedSemicolon {
                span: semi.span.to_source_span(),
            });
        }

        if modifiers.is_empty()
            && matches!(
                self.cursor.peek_kind(),
                TokenKind::KeywordCatch
                    | TokenKind::KeywordFinally
                    | TokenKind::KeywordCase
                    | TokenKind::KeywordDefault
                    | TokenKind::PunctuationCloseBrace
                    | TokenKind::PunctuationCloseParen
                    | TokenKind::PunctuationCloseBracket
            )
        {
            let token = self.cursor.advance().copied().unwrap_or_default();
            return Err(ParsingError::InvalidTopLevelToken {
                token: token.kind.to_string(),
                span: token.span.to_source_span(),
            });
        }

        if self.is_constructor_lookahead() {
            let name_span = self
                .cursor
                .advance()
                .map(|token| token.span)
                .unwrap_or_default();
            let ctor = self.parse_constructor_declaration(modifiers, name_span)?;
            self.record_error(ParsingError::ConstructorOutsideClass {
                span: ctor.span.to_source_span(),
            });
            return Ok(TopLevelElement::Declaration(Declaration::Constructor(ctor)));
        }

        match self.cursor.peek_kind() {
            TokenKind::KeywordClass => {
                let class_decl = self.parse_class_declaration(modifiers)?;
                Ok(TopLevelElement::Declaration(Declaration::Class(class_decl)))
            }
            TokenKind::KeywordInterface => {
                let iface_decl = self.parse_interface_declaration(modifiers)?;
                Ok(TopLevelElement::Declaration(Declaration::Interface(
                    iface_decl,
                )))
            }
            TokenKind::KeywordEnum => {
                let enum_decl = self.parse_enum_declaration(modifiers)?;
                Ok(TopLevelElement::Declaration(Declaration::Enum(enum_decl)))
            }
            TokenKind::OperatorBitwiseNot => {
                let dtor = self.parse_destructor_declaration()?;
                if !self.inside_class {
                    self.record_error(ParsingError::ConstructorOutsideClass {
                        span: dtor.span.to_source_span(),
                    });
                }
                Ok(TopLevelElement::Declaration(Declaration::Destructor(dtor)))
            }
            TokenKind::KeywordOperator => {
                let err_span = self.cursor.current_span();
                self.cursor.advance(); // consume 'operator'
                let operator_token = self.cursor.peek_kind();
                let operator_span = self.cursor.current_span();
                self.cursor.advance();
                let parameters = self.parse_parameter_list().unwrap_or_default();
                let body = self.parse_block()?;
                let span = err_span.combine(body.span);
                self.record_error(ParsingError::OperatorOutsideClass {
                    span: span.to_source_span(),
                });
                Ok(TopLevelElement::Declaration(Declaration::Operator(
                    OperatorDeclaration {
                        span,
                        operator_span,
                        operator_token,
                        modifiers,
                        parameters,
                        return_type: TypeAnnotation::Primitive {
                            span: err_span,
                            kind: TokenKind::PrimitiveVoid,
                        },
                        body,
                    },
                )))
            }
            _ => {
                // Check if this is a top-level field or method declaration
                if !modifiers.is_empty() || self.is_declaration_start() {
                    let type_annot = self.parse_type_annotation()?;
                    if self.cursor.check(TokenKind::KeywordOperator) {
                        let op_decl =
                            self.parse_operator_declaration(modifiers, Some(type_annot))?;
                        return Err(ParsingError::OperatorOutsideClass {
                            span: op_decl.span.to_source_span(),
                        });
                    }
                    let (namespace_path, name_span) = self.parse_qualified_name()?;

                    // Check for method declaration: ( or <
                    if self.cursor.check(TokenKind::PunctuationOpenParen)
                        || self.cursor.check(TokenKind::OperatorLessThan)
                    {
                        self.validate_modifiers(
                            &modifiers,
                            &[
                                Modifier::Public,
                                Modifier::Internal,
                                Modifier::Native,
                                Modifier::Inline,
                                Modifier::Extern,
                            ],
                            "free_function",
                            name_span,
                        );
                        let method_decl = self.parse_method_declaration(
                            modifiers,
                            type_annot,
                            namespace_path,
                            name_span,
                        )?;
                        return Ok(TopLevelElement::Declaration(Declaration::Method(
                            method_decl,
                        )));
                    }

                    // Otherwise field declaration
                    for modifier in &modifiers {
                        if matches!(
                            modifier,
                            Modifier::Public
                                | Modifier::Protected
                                | Modifier::Private
                                | Modifier::Internal
                        ) {
                            self.record_error(ParsingError::StandaloneFieldWithAccessModifier {
                                modifier: modifier.as_str().to_string(),
                                span: name_span.to_source_span(),
                            });
                        }
                    }

                    let initializer =
                        if self.cursor.match_token(TokenKind::OperatorAssign).is_some() {
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };

                    if modifiers.contains(&Modifier::Extern) && initializer.is_some() {
                        self.record_error(ParsingError::ExternWithInitializer {
                            span: name_span.to_source_span(),
                        });
                    }

                    if modifiers.contains(&Modifier::Extern) {
                        for modifier in &modifiers {
                            match modifier {
                                Modifier::Native => {
                                    self.record_error(ParsingError::InvalidExternModifier {
                                        modifier: "native".to_string(),
                                        span: name_span.to_source_span(),
                                    });
                                }
                                Modifier::Inline => {
                                    self.record_error(ParsingError::InvalidExternModifier {
                                        modifier: "inline".to_string(),
                                        span: name_span.to_source_span(),
                                    });
                                }
                                Modifier::Abstract => {
                                    self.record_error(ParsingError::InvalidExternModifier {
                                        modifier: "abstract".to_string(),
                                        span: name_span.to_source_span(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }

                    let semi = self.consume_semicolon()?;
                    let span = type_annot.span().combine(semi.span);

                    return Ok(TopLevelElement::Declaration(Declaration::Field(
                        FieldDeclaration {
                            span,
                            namespace_path,
                            name_span,
                            modifiers,
                            field_type: type_annot,
                            initializer,
                            resolved_symbol_id: None,
                        },
                    )));
                }

                // Normal statement
                let stmt = self.parse_statement()?;
                if !modifiers.is_empty() {
                    let modifier_str = modifiers[0].as_str().to_string();
                    self.record_error(ParsingError::StandaloneFieldWithAccessModifier {
                        modifier: modifier_str,
                        span: stmt.span().to_source_span(),
                    });
                }
                Ok(TopLevelElement::Statement(stmt))
            }
        }
    }

    fn parse_modifiers(&mut self) -> Vec<Modifier> {
        let mut modifiers = Vec::new();
        loop {
            let modifier = match self.cursor.peek_kind() {
                TokenKind::KeywordPublic => Some(Modifier::Public),
                TokenKind::KeywordProtected => Some(Modifier::Protected),
                TokenKind::KeywordPrivate => Some(Modifier::Private),
                TokenKind::KeywordInternal => Some(Modifier::Internal),
                TokenKind::KeywordStatic => Some(Modifier::Static),
                TokenKind::KeywordInline => Some(Modifier::Inline),
                TokenKind::KeywordNative => Some(Modifier::Native),
                TokenKind::KeywordConst => Some(Modifier::Const),
                TokenKind::KeywordVirtual => Some(Modifier::Virtual),
                TokenKind::KeywordOverride => Some(Modifier::Override),
                TokenKind::KeywordWeak => Some(Modifier::Weak),
                TokenKind::KeywordAbstract => Some(Modifier::Abstract),
                TokenKind::KeywordExtern => Some(Modifier::Extern),
                _ => None,
            };

            if let Some(parsed_modifier) = modifier {
                self.cursor.advance();
                modifiers.push(parsed_modifier);
            } else {
                break;
            }
        }
        modifiers
    }

    fn validate_modifiers(
        &mut self,
        modifiers: &[Modifier],
        allowed: &[Modifier],
        decl_kind: &str,
        span: Span,
    ) {
        let mut seen = Vec::new();
        for &modifier in modifiers {
            if seen.contains(&modifier) {
                self.record_error(ParsingError::DuplicateModifierOnDeclaration {
                    modifier: modifier.as_str().to_string(),
                    span: span.to_source_span(),
                });
            } else {
                seen.push(modifier);
            }
        }

        let visibility_mods: Vec<Modifier> = modifiers
            .iter()
            .copied()
            .filter(|modifier| {
                matches!(
                    modifier,
                    Modifier::Public | Modifier::Protected | Modifier::Private | Modifier::Internal
                )
            })
            .collect();
        if visibility_mods.len() > 1 {
            self.record_error(ParsingError::ConflictingVisibilityModifiers {
                modifier1: visibility_mods[0].as_str().to_string(),
                modifier2: visibility_mods[1].as_str().to_string(),
                span: span.to_source_span(),
            });
        }

        for &modifier in modifiers {
            if !allowed.contains(&modifier) {
                match decl_kind {
                    "class" => {
                        self.record_error(ParsingError::InvalidClassModifier {
                            modifier: modifier.as_str().to_string(),
                            span: span.to_source_span(),
                        });
                    }
                    "interface" => {
                        self.record_error(ParsingError::InvalidInterfaceModifier {
                            modifier: modifier.as_str().to_string(),
                            span: span.to_source_span(),
                        });
                    }
                    "enum" => {
                        self.record_error(ParsingError::InvalidEnumModifier {
                            modifier: modifier.as_str().to_string(),
                            span: span.to_source_span(),
                        });
                    }
                    "free_function" => {
                        if modifier == Modifier::Static {
                            self.record_error(ParsingError::StaticOnFreeFunction {
                                span: span.to_source_span(),
                            });
                        } else {
                            self.record_error(ParsingError::InvalidFreeFunctionModifier {
                                modifier: modifier.as_str().to_string(),
                                span: span.to_source_span(),
                            });
                        }
                    }
                    "operator" => {
                        self.record_error(ParsingError::InvalidOperatorModifier {
                            modifier: modifier.as_str().to_string(),
                            span: span.to_source_span(),
                        });
                    }
                    "destructor" => {
                        self.record_error(ParsingError::InvalidDestructorModifier {
                            span: span.to_source_span(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    fn parse_class_declaration(
        &mut self,
        modifiers: Vec<Modifier>,
    ) -> Result<ClassDeclaration, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'class'

        self.validate_modifiers(
            &modifiers,
            &[
                Modifier::Public,
                Modifier::Internal,
                Modifier::Abstract,
                Modifier::Extern,
            ],
            "class",
            start_span,
        );

        if self.inside_function() {
            self.record_error(ParsingError::ClassInLocalScope {
                span: start_span.to_source_span(),
            });
        }

        let (namespace_path, name_span) = self.parse_qualified_name()?;
        let class_name = self.get_lexeme(name_span).to_string();

        let generic_params = self.parse_generic_params()?;

        let super_class = if self.cursor.match_token(TokenKind::KeywordExtends).is_some() {
            if !self.is_type_start(self.cursor.peek_kind()) {
                return Err(ParsingError::ExpectedSuperclassList {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let mut interfaces = Vec::new();
        if self
            .cursor
            .match_token(TokenKind::KeywordImplements)
            .is_some()
        {
            if !self.is_type_start(self.cursor.peek_kind()) {
                return Err(ParsingError::ExpectedInterfaceList {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
            loop {
                interfaces.push(self.parse_type_annotation()?);
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
        }

        self.consume_open_brace()?;

        let previous_class_name = self.enclosing_class_name.replace(class_name);
        let previous_inside_class = self.inside_class;
        let previous_inside_extern = self.inside_extern_class;
        let previous_inside_generic = self.inside_generic_class;
        let previous_destructors = self.destructors_in_class;
        self.inside_class = true;
        self.inside_extern_class = modifiers.contains(&Modifier::Extern);
        self.inside_generic_class = !generic_params.is_empty();
        self.destructors_in_class = 0;

        let mut members = Vec::new();
        while !self.cursor.check(TokenKind::PunctuationCloseBrace) && !self.cursor.is_at_end() {
            match self.parse_class_member() {
                Ok(member) => members.push(member),
                Err(error) => {
                    self.record_error(error);
                    self.synchronize();
                }
            }
        }

        let close_brace = self.consume_close_brace()?;
        let span = start_span.combine(close_brace.span);

        self.enclosing_class_name = previous_class_name;
        self.inside_class = previous_inside_class;
        self.inside_extern_class = previous_inside_extern;
        self.inside_generic_class = previous_inside_generic;
        self.destructors_in_class = previous_destructors;

        Ok(ClassDeclaration {
            span,
            namespace_path,
            name_span,
            modifiers,
            generic_params,
            super_class,
            interfaces,
            members,
            resolved_symbol_id: None,
        })
    }

    fn parse_class_member(&mut self) -> Result<ClassMember, ParsingError> {
        if matches!(
            self.cursor.peek_kind(),
            TokenKind::KeywordBreak
                | TokenKind::KeywordContinue
                | TokenKind::KeywordReturn
                | TokenKind::KeywordThrow
                | TokenKind::KeywordIf
                | TokenKind::KeywordWhile
                | TokenKind::KeywordDo
                | TokenKind::KeywordFor
                | TokenKind::KeywordSwitch
                | TokenKind::KeywordTry
                | TokenKind::PunctuationOpenBrace
                | TokenKind::Number
                | TokenKind::String
                | TokenKind::Char
        ) {
            let span = self.cursor.current_span();
            return Err(ParsingError::StatementDirectlyInClass {
                span: span.to_source_span(),
            });
        }

        let member_modifiers = self.parse_modifiers();

        // Destructor: ~ClassName()
        if self.cursor.check(TokenKind::OperatorBitwiseNot) {
            let dtor = self.parse_destructor_declaration()?;
            if !member_modifiers.is_empty() {
                self.validate_modifiers(&member_modifiers, &[], "destructor", dtor.span);
            }
            self.destructors_in_class += 1;
            if self.destructors_in_class > 1 {
                self.record_error(ParsingError::MultipleDestructorsInClass {
                    span: dtor.span.to_source_span(),
                });
            }
            if let Some(enclosing) = &self.enclosing_class_name {
                let dtor_name = self.get_lexeme(dtor.name_span);
                if dtor_name != enclosing {
                    self.record_error(ParsingError::DestructorNameMismatch {
                        span: dtor.name_span.to_source_span(),
                    });
                }
            }
            return Ok(ClassMember::Destructor(dtor));
        }

        // Nested enum declaration inside class
        if self.cursor.check(TokenKind::KeywordEnum) {
            let enum_decl = self.parse_enum_declaration(member_modifiers)?;
            return Ok(ClassMember::Enum(enum_decl));
        }

        // Nested class declaration inside class
        if self.cursor.check(TokenKind::KeywordClass) {
            let class_decl = self.parse_class_declaration(member_modifiers)?;
            return Ok(ClassMember::Class(class_decl));
        }

        // Nested interface declaration inside class
        if self.cursor.check(TokenKind::KeywordInterface) {
            let interface_decl = self.parse_interface_declaration(member_modifiers)?;
            return Ok(ClassMember::Interface(interface_decl));
        }

        // Operator overload: operator +(...)
        if self.cursor.check(TokenKind::KeywordOperator) {
            let op_decl = self.parse_operator_declaration(member_modifiers, None)?;
            return Ok(ClassMember::Operator(op_decl));
        }

        // Check for constructor: ClassName(...) with no preceding type
        if self.cursor.check(TokenKind::Identifier)
            && self.cursor.check_next(TokenKind::PunctuationOpenParen)
            && !self.cursor.check_nth(2, TokenKind::OperatorMultiply)
        {
            let ident_span = self.cursor.current_span();
            let ident_name = self.get_lexeme(ident_span);

            if self.enclosing_class_name.as_deref() == Some(ident_name) {
                self.cursor.advance();
                let ctor = self.parse_constructor_declaration(member_modifiers, ident_span)?;
                return Ok(ClassMember::Constructor(ctor));
            } else {
                return Err(ParsingError::ConstructorNameMismatch {
                    span: ident_span.to_source_span(),
                });
            }
        }

        // Otherwise field or method
        let type_annot = self.parse_type_annotation()?;

        // Operator overload with return type: Type operator +(...)
        if self.cursor.check(TokenKind::KeywordOperator) {
            let op_decl = self.parse_operator_declaration(member_modifiers, Some(type_annot))?;
            return Ok(ClassMember::Operator(op_decl));
        }

        let name_span = self.consume_identifier()?;

        // Constructor returning a type error: void ClassName()
        if self.enclosing_class_name.as_deref() == Some(self.get_lexeme(name_span))
            && self.cursor.check(TokenKind::PunctuationOpenParen)
        {
            self.record_error(ParsingError::ConstructorReturnType {
                span: type_annot.span().to_source_span(),
            });
        }

        if self.cursor.check(TokenKind::PunctuationOpenParen)
            || self.cursor.check(TokenKind::OperatorLessThan)
            || self.cursor.check(TokenKind::PunctuationOpenBrace)
        {
            let method =
                self.parse_method_declaration(member_modifiers, type_annot, Vec::new(), name_span)?;
            return Ok(ClassMember::Method(method));
        }

        // Field
        let span = type_annot.span().combine(name_span);
        self.validate_modifiers(
            &member_modifiers,
            &[
                Modifier::Public,
                Modifier::Protected,
                Modifier::Private,
                Modifier::Internal,
                Modifier::Static,
                Modifier::Const,
                Modifier::Weak,
                Modifier::Extern,
            ],
            "field",
            span,
        );
        let initializer = if self.cursor.match_token(TokenKind::OperatorAssign).is_some() {
            Some(self.parse_expression()?)
        } else {
            None
        };
        if (self.inside_extern_class || member_modifiers.contains(&Modifier::Extern))
            && initializer.is_some()
        {
            self.record_error(ParsingError::ExternWithInitializer {
                span: name_span.to_source_span(),
            });
        }
        let semi = self.consume_semicolon()?;
        let span = type_annot.span().combine(semi.span);

        Ok(ClassMember::Field(FieldDeclaration {
            span,
            namespace_path: Vec::new(),
            name_span,
            modifiers: member_modifiers,
            field_type: type_annot,
            initializer,
            resolved_symbol_id: None,
        }))
    }

    fn parse_interface_declaration(
        &mut self,
        modifiers: Vec<Modifier>,
    ) -> Result<InterfaceDeclaration, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'interface'

        self.validate_modifiers(
            &modifiers,
            &[Modifier::Public, Modifier::Internal, Modifier::Extern],
            "interface",
            start_span,
        );

        if self.inside_function() {
            self.record_error(ParsingError::InterfaceInLocalScope {
                span: start_span.to_source_span(),
            });
        }

        let (namespace_path, name_span) = self.parse_qualified_name()?;

        let generic_params = self.parse_generic_params()?;

        let mut super_interfaces = Vec::new();
        if self.cursor.match_token(TokenKind::KeywordExtends).is_some() {
            if !self.is_type_start(self.cursor.peek_kind()) {
                return Err(ParsingError::ExpectedInterfaceList {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
            loop {
                super_interfaces.push(self.parse_type_annotation()?);
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
        }

        self.consume_open_brace()?;

        let previous_inside_interface = self.inside_interface;
        self.inside_interface = true;

        let mut methods = Vec::new();
        while !self.cursor.check(TokenKind::PunctuationCloseBrace) && !self.cursor.is_at_end() {
            if self.cursor.check(TokenKind::OperatorBitwiseNot) {
                let dtor = self.parse_destructor_declaration();
                let err_span = match &dtor {
                    Ok(decl) => decl.span.to_source_span(),
                    Err(error) => error.source_span(),
                };
                self.record_error(ParsingError::InterfaceDestructor { span: err_span });
                continue;
            }

            let member_mods = self.parse_modifiers();
            let return_type = self.parse_type_annotation()?;
            let method_name = self.consume_identifier()?;

            if !self.cursor.check(TokenKind::PunctuationOpenParen)
                && !self.cursor.check(TokenKind::OperatorLessThan)
            {
                self.record_error(ParsingError::InterfaceField {
                    span: method_name.to_source_span(),
                });
                if self.cursor.match_token(TokenKind::OperatorAssign).is_some() {
                    let _ = self.parse_expression();
                }
                let _ = self.consume_semicolon();
                continue;
            }

            let method =
                self.parse_method_declaration(member_mods, return_type, Vec::new(), method_name)?;
            if method.body.is_some() {
                self.record_error(ParsingError::InterfaceMethodWithBody {
                    span: method.span.to_source_span(),
                });
            }
            methods.push(method);
        }

        let close_brace = self.consume_close_brace()?;
        let span = start_span.combine(close_brace.span);
        self.inside_interface = previous_inside_interface;

        Ok(InterfaceDeclaration {
            span,
            namespace_path,
            name_span,
            modifiers,
            generic_params,
            super_interfaces,
            methods,
            resolved_symbol_id: None,
        })
    }

    fn parse_enum_declaration(
        &mut self,
        modifiers: Vec<Modifier>,
    ) -> Result<EnumDeclaration, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'enum'

        self.validate_modifiers(
            &modifiers,
            &[Modifier::Public, Modifier::Internal],
            "enum",
            start_span,
        );

        if self.inside_function() {
            self.record_error(ParsingError::EnumInLocalScope {
                span: start_span.to_source_span(),
            });
        }

        let (namespace_path, name_span) = self.parse_qualified_name()?;

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenBrace)
            .is_none()
        {
            return Err(ParsingError::ExpectedEnumBrace {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let mut variants = Vec::new();
        while !self.cursor.check(TokenKind::PunctuationCloseBrace) && !self.cursor.is_at_end() {
            let variant_name = self.consume_identifier()?;
            let explicit_value = if self.cursor.match_token(TokenKind::OperatorAssign).is_some() {
                Some(self.parse_expression()?)
            } else {
                None
            };
            let var_span = if let Some(val) = &explicit_value {
                variant_name.combine(val.span())
            } else {
                variant_name
            };

            variants.push(EnumVariant {
                span: var_span,
                name_span: variant_name,
                explicit_value,
            });

            if self
                .cursor
                .match_token(TokenKind::PunctuationComma)
                .is_none()
            {
                break;
            }
        }

        let close_brace = self.consume_close_brace()?;
        if variants.is_empty() {
            return Err(ParsingError::EmptyEnumDeclaration {
                span: start_span.combine(close_brace.span).to_source_span(),
            });
        }
        let span = start_span.combine(close_brace.span);

        Ok(EnumDeclaration {
            span,
            namespace_path,
            name_span,
            modifiers,
            variants,
            resolved_symbol_id: None,
        })
    }

    fn parse_method_declaration(
        &mut self,
        modifiers: Vec<Modifier>,
        return_type: TypeAnnotation,
        namespace_path: Vec<Span>,
        name_span: Span,
    ) -> Result<MethodDeclaration, ParsingError> {
        if self.inside_function() {
            self.record_error(ParsingError::MethodInsideMethod {
                span: name_span.to_source_span(),
            });
        }

        let member_span = return_type.span().combine(name_span);
        self.validate_modifiers(
            &modifiers,
            &[
                Modifier::Public,
                Modifier::Protected,
                Modifier::Private,
                Modifier::Internal,
                Modifier::Static,
                Modifier::Virtual,
                Modifier::Override,
                Modifier::Abstract,
                Modifier::Inline,
                Modifier::Native,
                Modifier::Extern,
            ],
            "method",
            member_span,
        );

        let generic_params = self.parse_generic_params()?;
        let parameters = self.parse_parameter_list()?;

        let is_abstract_or_native =
            modifiers.contains(&Modifier::Abstract) || modifiers.contains(&Modifier::Native);
        let is_extern = self.inside_extern_class || modifiers.contains(&Modifier::Extern);

        if is_extern {
            for modifier in &modifiers {
                match modifier {
                    Modifier::Native => {
                        self.record_error(ParsingError::InvalidExternModifier {
                            modifier: "native".to_string(),
                            span: name_span.to_source_span(),
                        });
                    }
                    Modifier::Inline => {
                        self.record_error(ParsingError::InvalidExternModifier {
                            modifier: "inline".to_string(),
                            span: name_span.to_source_span(),
                        });
                    }
                    Modifier::Abstract => {
                        self.record_error(ParsingError::InvalidExternModifier {
                            modifier: "abstract".to_string(),
                            span: name_span.to_source_span(),
                        });
                    }
                    _ => {}
                }
            }
        }

        let body = if self.cursor.check(TokenKind::PunctuationOpenBrace) {
            if is_abstract_or_native {
                self.record_error(ParsingError::AbstractOrNativeMethodWithBody {
                    span: name_span.to_source_span(),
                });
            }
            let has_variadic_param = parameters.iter().any(|param| param.is_variadic);
            let is_generic_method =
                !generic_params.is_empty() || has_variadic_param || self.inside_generic_class;
            if is_extern && !is_generic_method && !modifiers.contains(&Modifier::Inline) {
                self.record_error(ParsingError::ExternWithBody {
                    span: name_span.to_source_span(),
                });
            }
            Some(self.parse_block()?)
        } else {
            self.consume_semicolon()?;
            None
        };

        let span = return_type
            .span()
            .combine(body.as_ref().map(|block| block.span).unwrap_or(name_span));

        Ok(MethodDeclaration {
            span,
            namespace_path,
            name_span,
            modifiers,
            return_type,
            generic_params,
            parameters,
            body,
            resolved_symbol_id: None,
        })
    }

    fn parse_constructor_declaration(
        &mut self,
        modifiers: Vec<Modifier>,
        name_span: Span,
    ) -> Result<ConstructorDeclaration, ParsingError> {
        self.validate_modifiers(
            &modifiers,
            &[
                Modifier::Public,
                Modifier::Protected,
                Modifier::Private,
                Modifier::Internal,
                Modifier::Inline,
                Modifier::Extern,
            ],
            "constructor",
            name_span,
        );
        let parameters = self.parse_parameter_list()?;

        let super_call_args = if self
            .cursor
            .match_token(TokenKind::PunctuationColon)
            .is_some()
        {
            if self.cursor.check(TokenKind::KeywordSuper) {
                self.cursor.advance();
                Some(self.parse_argument_list()?)
            } else {
                return Err(ParsingError::MalformedConstructorChaining {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
        } else if self.cursor.check(TokenKind::KeywordSuper) {
            return Err(ParsingError::ExpectedColonAfterSuper {
                span: self.cursor.current_span().to_source_span(),
            });
        } else {
            None
        };

        let is_extern = self.inside_extern_class || modifiers.contains(&Modifier::Extern);
        let body = if self.cursor.check(TokenKind::PunctuationOpenBrace) {
            let has_variadic_param = parameters.iter().any(|param| param.is_variadic);
            if is_extern
                && !has_variadic_param
                && !self.inside_generic_class
                && !modifiers.contains(&Modifier::Inline)
            {
                self.record_error(ParsingError::ExternWithBody {
                    span: name_span.to_source_span(),
                });
            }
            self.parse_block()?
        } else if is_extern || self.cursor.check(TokenKind::PunctuationSemicolon) {
            self.consume_semicolon()?;
            BlockStatement {
                span: name_span,
                statements: Vec::new(),
            }
        } else {
            self.parse_block()?
        };
        let span = name_span.combine(body.span);

        Ok(ConstructorDeclaration {
            span,
            name_span,
            modifiers,
            parameters,
            super_call_args,
            body,
        })
    }

    fn parse_destructor_declaration(&mut self) -> Result<DestructorDeclaration, ParsingError> {
        let tilde_span = self.cursor.current_span();
        self.cursor.advance(); // consume '~'

        let name_span = self.consume_identifier()?;

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedOpenParen {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        if !self.cursor.check(TokenKind::PunctuationCloseParen) {
            self.record_error(ParsingError::DestructorParameters {
                span: self.cursor.current_span().to_source_span(),
            });
            while !self.cursor.check(TokenKind::PunctuationCloseParen) && !self.cursor.is_at_end() {
                self.cursor.advance();
            }
        }
        self.cursor.advance(); // consume ')'

        let body = if self.cursor.check(TokenKind::PunctuationOpenBrace) {
            self.parse_block()?
        } else {
            self.consume_semicolon()?;
            BlockStatement {
                span: name_span,
                statements: Vec::new(),
            }
        };
        let span = tilde_span.combine(body.span);

        Ok(DestructorDeclaration {
            span,
            name_span,
            body,
        })
    }

    fn parse_operator_declaration(
        &mut self,
        modifiers: Vec<Modifier>,
        return_type: Option<TypeAnnotation>,
    ) -> Result<OperatorDeclaration, ParsingError> {
        let op_kw_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'operator'

        let operator_token = self.cursor.peek_kind();
        let operator_span = self.cursor.current_span();
        self.cursor.advance();

        self.validate_modifiers(
            &modifiers,
            &[Modifier::Public, Modifier::Inline],
            "operator",
            operator_span,
        );

        let is_overloadable = matches!(
            operator_token,
            TokenKind::OperatorPlus
                | TokenKind::OperatorMinus
                | TokenKind::OperatorMultiply
                | TokenKind::OperatorDivide
                | TokenKind::OperatorModulo
                | TokenKind::OperatorEqual
                | TokenKind::OperatorNotEqual
                | TokenKind::OperatorLessThan
                | TokenKind::OperatorLessEqual
                | TokenKind::OperatorGreaterThan
                | TokenKind::OperatorGreaterEqual
                | TokenKind::PunctuationAmpersand
                | TokenKind::OperatorBitwiseNot
                | TokenKind::OperatorLogicalNot
                | TokenKind::OperatorIncrement
                | TokenKind::OperatorDecrement
                | TokenKind::PunctuationOpenBracket
        );
        if !is_overloadable {
            return Err(ParsingError::InvalidOperatorOverload {
                span: operator_span.to_source_span(),
            });
        }

        let parameters = self.parse_parameter_list()?;

        let ret_type = if let Some(explicit_return) = return_type {
            explicit_return
        } else if self
            .cursor
            .match_token(TokenKind::OperatorFatArrow)
            .is_some()
        {
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Primitive {
                span: op_kw_span,
                kind: TokenKind::PrimitiveVoid,
            }
        };

        let body = if self.cursor.check(TokenKind::PunctuationSemicolon) {
            let semi_span = self.cursor.current_span();
            self.cursor.advance();
            BlockStatement {
                span: semi_span,
                statements: Vec::new(),
            }
        } else {
            self.parse_block()?
        };
        let span = op_kw_span.combine(body.span);

        Ok(OperatorDeclaration {
            span,
            operator_span,
            operator_token,
            modifiers,
            parameters,
            return_type: ret_type,
            body,
        })
    }

    fn parse_statement_in_block(&mut self) -> Result<Vec<Statement>, ParsingError> {
        match self.cursor.peek_kind() {
            TokenKind::PunctuationSemicolon => {
                let semi = self.cursor.advance().copied().unwrap_or_default();
                Err(ParsingError::UnexpectedSemicolon {
                    span: semi.span.to_source_span(),
                })
            }
            TokenKind::PunctuationOpenBrace => Ok(vec![Statement::Block(self.parse_block()?)]),
            TokenKind::KeywordIf => Ok(vec![Statement::If(self.parse_if_statement()?)]),
            TokenKind::KeywordWhile => Ok(vec![Statement::While(self.parse_while_statement()?)]),
            TokenKind::KeywordDo => Ok(vec![Statement::DoWhile(self.parse_do_while_statement()?)]),
            TokenKind::KeywordFor => Ok(vec![Statement::For(self.parse_for_statement()?)]),
            TokenKind::KeywordSwitch => Ok(vec![Statement::Switch(self.parse_switch_statement()?)]),
            TokenKind::KeywordTry => Ok(vec![Statement::Try(self.parse_try_statement()?)]),
            TokenKind::KeywordThrow => Ok(vec![Statement::Throw(self.parse_throw_statement()?)]),
            TokenKind::KeywordReturn => Ok(vec![Statement::Return(self.parse_return_statement()?)]),
            TokenKind::KeywordBreak => Ok(vec![Statement::Break(self.parse_break_statement()?)]),
            TokenKind::KeywordContinue => {
                Ok(vec![Statement::Continue(self.parse_continue_statement()?)])
            }
            TokenKind::KeywordAssert => Ok(vec![Statement::Assert(self.parse_assert_statement()?)]),
            TokenKind::KeywordExit => Ok(vec![Statement::Exit(self.parse_exit_statement()?)]),
            TokenKind::KeywordConst => {
                let start_span = self.cursor.current_span();
                self.cursor.advance(); // consume 'const'
                let var_type = self.parse_type_annotation()?;
                let name_span = self.consume_identifier()?;
                let mut decls = Vec::new();
                let first_init = if self.cursor.match_token(TokenKind::OperatorAssign).is_some() {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                decls.push(VariableDeclarationStatement {
                    span: start_span.combine(name_span),
                    name_span,
                    is_const: true,
                    var_type: var_type.clone(),
                    initializer: first_init,
                });

                while self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_some()
                {
                    let next_name_span = self.consume_identifier()?;
                    let next_init = if self.cursor.match_token(TokenKind::OperatorAssign).is_some()
                    {
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                    decls.push(VariableDeclarationStatement {
                        span: next_name_span,
                        name_span: next_name_span,
                        is_const: true,
                        var_type: var_type.clone(),
                        initializer: next_init,
                    });
                }

                let semi = self.consume_semicolon()?;
                if let Some(last) = decls.last_mut() {
                    last.span = last.span.combine(semi.span);
                }
                Ok(decls
                    .into_iter()
                    .map(Statement::VariableDeclaration)
                    .collect())
            }
            TokenKind::KeywordClass => {
                let span = self.cursor.current_span();
                self.record_error(ParsingError::ClassInLocalScope {
                    span: span.to_source_span(),
                });
                let class_decl = self.parse_class_declaration(Vec::new())?;
                Ok(vec![Statement::Block(BlockStatement {
                    span: class_decl.span,
                    statements: Vec::new(),
                })])
            }
            TokenKind::KeywordInterface => {
                let span = self.cursor.current_span();
                self.record_error(ParsingError::InterfaceInLocalScope {
                    span: span.to_source_span(),
                });
                let iface_decl = self.parse_interface_declaration(Vec::new())?;
                Ok(vec![Statement::Block(BlockStatement {
                    span: iface_decl.span,
                    statements: Vec::new(),
                })])
            }
            TokenKind::KeywordEnum => {
                let span = self.cursor.current_span();
                self.record_error(ParsingError::EnumInLocalScope {
                    span: span.to_source_span(),
                });
                let enum_decl = self.parse_enum_declaration(Vec::new())?;
                Ok(vec![Statement::Block(BlockStatement {
                    span: enum_decl.span,
                    statements: Vec::new(),
                })])
            }
            TokenKind::KeywordPackage => {
                let span = self.cursor.current_span();
                self.record_error(ParsingError::PackageStatementNotFirst {
                    span: span.to_source_span(),
                });
                let pkg = self.parse_package_statement()?;
                Ok(vec![Statement::Block(BlockStatement {
                    span: pkg.span,
                    statements: Vec::new(),
                })])
            }
            TokenKind::KeywordImport => {
                let span = self.cursor.current_span();
                self.record_error(ParsingError::ImportAfterDeclaration {
                    span: span.to_source_span(),
                });
                let imp = self.parse_import_statement(false, None)?;
                Ok(vec![Statement::Block(BlockStatement {
                    span: imp.span,
                    statements: Vec::new(),
                })])
            }
            TokenKind::KeywordOperator => {
                let span = self.cursor.current_span();
                self.record_error(ParsingError::OperatorOutsideClass {
                    span: span.to_source_span(),
                });
                let op = self.parse_operator_declaration(Vec::new(), None)?;
                Ok(vec![Statement::Block(op.body)])
            }
            _ => {
                let mut had_modifier = false;
                while self.cursor.check(TokenKind::KeywordExtern)
                    || self.is_non_const_modifier(self.cursor.peek_kind())
                {
                    let mod_span = self.cursor.current_span();
                    if self.cursor.check(TokenKind::KeywordExtern) {
                        self.record_error(ParsingError::ExternInsideFunction {
                            span: mod_span.to_source_span(),
                        });
                    } else {
                        self.record_error(ParsingError::ModifierOnLocalVariable {
                            span: mod_span.to_source_span(),
                        });
                    }
                    self.cursor.advance();
                    had_modifier = true;
                }

                if self.is_constructor_lookahead() {
                    let name_span = self
                        .cursor
                        .advance()
                        .map(|token| token.span)
                        .unwrap_or_default();
                    let ctor = self.parse_constructor_declaration(Vec::new(), name_span)?;
                    self.record_error(ParsingError::ConstructorOutsideClass {
                        span: ctor.span.to_source_span(),
                    });
                    return Ok(vec![Statement::Block(ctor.body)]);
                }

                // Check for typed variable declaration: int32 x = 5;
                if had_modifier || self.is_declaration_start() {
                    let var_type = self.parse_type_annotation()?;
                    let name_span = self.consume_identifier()?;

                    if self.cursor.check(TokenKind::PunctuationOpenParen)
                        || self.cursor.check(TokenKind::OperatorLessThan)
                    {
                        self.record_error(ParsingError::MethodInsideMethod {
                            span: name_span.to_source_span(),
                        });
                        let method = self.parse_method_declaration(
                            Vec::new(),
                            var_type,
                            Vec::new(),
                            name_span,
                        )?;
                        return Ok(vec![Statement::Block(method.body.unwrap_or(
                            BlockStatement {
                                span: name_span,
                                statements: Vec::new(),
                            },
                        ))]);
                    }

                    let mut decls = Vec::new();
                    let first_init = if self.cursor.match_token(TokenKind::OperatorAssign).is_some()
                    {
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                    decls.push(VariableDeclarationStatement {
                        span: var_type.span().combine(name_span),
                        name_span,
                        is_const: false,
                        var_type: var_type.clone(),
                        initializer: first_init,
                    });

                    while self
                        .cursor
                        .match_token(TokenKind::PunctuationComma)
                        .is_some()
                    {
                        let next_name_span = self.consume_identifier()?;
                        let next_init =
                            if self.cursor.match_token(TokenKind::OperatorAssign).is_some() {
                                Some(self.parse_expression()?)
                            } else {
                                None
                            };
                        decls.push(VariableDeclarationStatement {
                            span: next_name_span,
                            name_span: next_name_span,
                            is_const: false,
                            var_type: var_type.clone(),
                            initializer: next_init,
                        });
                    }

                    let semi = self.consume_semicolon()?;
                    if let Some(last) = decls.last_mut() {
                        last.span = last.span.combine(semi.span);
                    }
                    return Ok(decls
                        .into_iter()
                        .map(Statement::VariableDeclaration)
                        .collect());
                }

                // Otherwise expression statement
                let expr = self.parse_expression()?;
                let semi = self.consume_semicolon()?;
                let span = expr.span().combine(semi.span);
                Ok(vec![Statement::Expression(ExpressionStatement {
                    span,
                    expression: expr,
                })])
            }
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParsingError> {
        let mut stmts = self.parse_statement_in_block()?;
        if stmts.len() == 1
            && let Some(single_stmt) = stmts.pop()
        {
            return Ok(single_stmt);
        }
        let start_span = stmts.first().map(|stmt| stmt.span()).unwrap_or_default();
        let end_span = stmts.last().map(|stmt| stmt.span()).unwrap_or_default();
        Ok(Statement::Block(BlockStatement {
            span: start_span.combine(end_span),
            statements: stmts,
        }))
    }

    fn parse_block(&mut self) -> Result<BlockStatement, ParsingError> {
        let open_brace = self.cursor.current_span();
        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenBrace)
            .is_none()
        {
            return Err(ParsingError::ExpectedBlock {
                span: open_brace.to_source_span(),
            });
        }

        self.scope_depth += 1;
        let mut statements = Vec::new();

        while !self.cursor.check(TokenKind::PunctuationCloseBrace) && !self.cursor.is_at_end() {
            match self.parse_statement_in_block() {
                Ok(stmts) => statements.extend(stmts),
                Err(error) => {
                    self.record_error(error);
                    self.synchronize();
                }
            }
        }

        let close_brace_span = if self.cursor.check(TokenKind::PunctuationCloseBrace) {
            self.cursor.advance().copied().unwrap_or_default().span
        } else {
            self.record_error(ParsingError::ExpectedCloseBrace {
                span: self.cursor.current_span().to_source_span(),
            });
            self.cursor.current_span()
        };
        self.scope_depth -= 1;
        let span = open_brace.combine(close_brace_span);

        Ok(BlockStatement { span, statements })
    }

    fn parse_if_statement(&mut self) -> Result<IfStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'if'

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedOpenParen {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        if self.cursor.check(TokenKind::PunctuationCloseParen) {
            return Err(ParsingError::ExpectedConditionInIf {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let condition = self.parse_expression()?;

        if self
            .cursor
            .match_token(TokenKind::PunctuationCloseParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let then_branch = self.parse_branch_statement()?;

        let else_branch = if self.cursor.match_token(TokenKind::KeywordElse).is_some() {
            Some(Box::new(self.parse_branch_statement()?))
        } else {
            None
        };

        let end_span = else_branch
            .as_ref()
            .map(|node| node.span())
            .unwrap_or_else(|| then_branch.span());
        let span = start_span.combine(end_span);

        Ok(IfStatement {
            span,
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_branch_statement(&mut self) -> Result<Statement, ParsingError> {
        if self.cursor.check(TokenKind::KeywordConst) || self.is_declaration_start() {
            let err_span = self.cursor.current_span();
            self.record_error(ParsingError::SolitaryDeclarationInBranchOrLoop {
                span: err_span.to_source_span(),
            });
        }
        self.parse_statement()
    }

    fn parse_while_statement(&mut self) -> Result<WhileStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'while'

        let has_paren = self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_some();
        if !has_paren {
            self.record_error(ParsingError::MalformedWhileCondition {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        if has_paren && self.cursor.check(TokenKind::PunctuationCloseParen) {
            return Err(ParsingError::ExpectedConditionInWhile {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let condition = self.parse_expression()?;

        if has_paren
            && self
                .cursor
                .match_token(TokenKind::PunctuationCloseParen)
                .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let body = self.parse_branch_statement()?;
        let span = start_span.combine(body.span());

        Ok(WhileStatement {
            span,
            condition,
            body: Box::new(body),
        })
    }

    fn parse_do_while_statement(&mut self) -> Result<DoWhileStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'do'

        let body = self.parse_branch_statement()?;

        if self.cursor.match_token(TokenKind::KeywordWhile).is_none() {
            return Err(ParsingError::MalformedDoWhile {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        if self.cursor.check(TokenKind::PunctuationSemicolon) {
            let semi = self.consume_semicolon()?;
            self.record_error(ParsingError::MalformedDoWhile {
                span: semi.span.to_source_span(),
            });
            let span = start_span.combine(semi.span);
            return Ok(DoWhileStatement {
                span,
                body: Box::new(body),
                condition: Expression::Literal {
                    span: semi.span,
                    value: LiteralValue::Bool(false),
                },
            });
        }

        let has_paren = self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_some();
        if !has_paren {
            self.record_error(ParsingError::MalformedWhileCondition {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        if has_paren && self.cursor.check(TokenKind::PunctuationCloseParen) {
            return Err(ParsingError::ExpectedConditionInWhile {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let condition = self.parse_expression()?;

        if has_paren
            && self
                .cursor
                .match_token(TokenKind::PunctuationCloseParen)
                .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(DoWhileStatement {
            span,
            body: Box::new(body),
            condition,
        })
    }

    fn parse_for_statement(&mut self) -> Result<ForStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'for'

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_none()
        {
            return Err(ParsingError::MalformedForLoop {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let initializer = if self.cursor.check(TokenKind::PunctuationSemicolon) {
            self.cursor.advance();
            None
        } else {
            match self.parse_statement() {
                Ok(stmt) => Some(Box::new(stmt)),
                Err(_) => {
                    self.record_error(ParsingError::MalformedForLoop {
                        span: self.cursor.current_span().to_source_span(),
                    });
                    while !self.cursor.check(TokenKind::PunctuationSemicolon)
                        && !self.cursor.is_at_end()
                    {
                        self.cursor.advance();
                    }
                    if self.cursor.check(TokenKind::PunctuationSemicolon) {
                        self.cursor.advance();
                    }
                    None
                }
            }
        };

        let condition = if self.cursor.check(TokenKind::PunctuationSemicolon) {
            None
        } else {
            match self.parse_expression() {
                Ok(expr) => Some(expr),
                Err(_) => {
                    self.record_error(ParsingError::MalformedForLoop {
                        span: self.cursor.current_span().to_source_span(),
                    });
                    while !self.cursor.check(TokenKind::PunctuationSemicolon)
                        && !self.cursor.is_at_end()
                    {
                        self.cursor.advance();
                    }
                    None
                }
            }
        };

        if self
            .cursor
            .match_token(TokenKind::PunctuationSemicolon)
            .is_none()
        {
            self.record_error(ParsingError::MalformedForLoop {
                span: self.cursor.current_span().to_source_span(),
            });
            while !self.cursor.check(TokenKind::PunctuationCloseParen)
                && !self.cursor.check(TokenKind::PunctuationOpenBrace)
                && !self.cursor.is_at_end()
            {
                self.cursor.advance();
            }
        }

        let update = if self.cursor.check(TokenKind::PunctuationCloseParen) {
            None
        } else {
            self.parse_expression().ok()
        };

        if self
            .cursor
            .match_token(TokenKind::PunctuationCloseParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let body = self.parse_branch_statement()?;
        let span = start_span.combine(body.span());

        Ok(ForStatement {
            span,
            initializer,
            condition,
            update,
            body: Box::new(body),
        })
    }

    fn parse_switch_statement(&mut self) -> Result<SwitchStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'switch'

        let has_paren = self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_some();
        if !has_paren {
            self.record_error(ParsingError::MalformedSwitch {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let discriminant = self.parse_expression()?;

        if has_paren
            && self
                .cursor
                .match_token(TokenKind::PunctuationCloseParen)
                .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenBrace)
            .is_none()
        {
            return Err(ParsingError::ExpectedSwitchBrace {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let mut cases = Vec::new();
        let mut default_case = None;
        let mut seen_case = false;

        while !self.cursor.check(TokenKind::PunctuationCloseBrace) && !self.cursor.is_at_end() {
            if self.cursor.match_token(TokenKind::KeywordCase).is_some() {
                seen_case = true;
                let case_start = self.cursor.current_span();
                let case_value = if self
                    .cursor
                    .match_token(TokenKind::KeywordInstanceof)
                    .is_some()
                {
                    let type_annotation = self.parse_type_annotation()?;
                    let type_span = type_annotation.span();
                    Expression::Instanceof {
                        span: case_start.combine(type_span),
                        expression: Box::new(discriminant.clone()),
                        target_type: type_annotation,
                    }
                } else {
                    self.parse_expression()?
                };

                let is_valid_case = matches!(
                    case_value,
                    Expression::Literal { .. }
                        | Expression::MemberAccess { .. }
                        | Expression::Identifier { .. }
                        | Expression::TupleCreation { .. }
                        | Expression::Instanceof { .. }
                ) || (matches!(&case_value, Expression::Unary { operator, operand, .. }
                    if matches!(operator, TokenKind::OperatorMinus | TokenKind::OperatorPlus)
                        && matches!(&**operand, Expression::Literal { .. })));

                if !is_valid_case {
                    self.record_error(ParsingError::CaseLabelNotConstant {
                        span: case_value.span().to_source_span(),
                    });
                }

                if self
                    .cursor
                    .match_token(TokenKind::PunctuationColon)
                    .is_none()
                {
                    return Err(ParsingError::ExpectedColonInCase {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }

                let mut case_stmts = Vec::new();
                while !self.cursor.check(TokenKind::KeywordCase)
                    && !self.cursor.check(TokenKind::KeywordDefault)
                    && !self.cursor.check(TokenKind::PunctuationCloseBrace)
                    && !self.cursor.is_at_end()
                {
                    case_stmts.extend(self.parse_statement_in_block()?);
                }

                let case_end = case_stmts
                    .last()
                    .map(|stmt| stmt.span())
                    .unwrap_or_else(|| case_value.span());
                cases.push(CaseClause {
                    span: case_start.combine(case_end),
                    case_value,
                    statements: case_stmts,
                });
            } else if self.cursor.match_token(TokenKind::KeywordDefault).is_some() {
                if default_case.is_some() {
                    return Err(ParsingError::DuplicateDefaultInSwitch {
                        span: self.cursor.previous_span().to_source_span(),
                    });
                }
                seen_case = true;
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationColon)
                    .is_none()
                {
                    return Err(ParsingError::ExpectedColonInCase {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }

                let mut def_stmts = Vec::new();
                while !self.cursor.check(TokenKind::KeywordCase)
                    && !self.cursor.check(TokenKind::KeywordDefault)
                    && !self.cursor.check(TokenKind::PunctuationCloseBrace)
                    && !self.cursor.is_at_end()
                {
                    def_stmts.extend(self.parse_statement_in_block()?);
                }
                default_case = Some(def_stmts);
            } else {
                if !seen_case {
                    self.record_error(ParsingError::StatementBeforeCaseInSwitch {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }
                self.parse_statement()?;
            }
        }

        let close_brace = self.consume_close_brace()?;
        let span = start_span.combine(close_brace.span);

        Ok(SwitchStatement {
            span,
            discriminant,
            cases,
            default_case,
        })
    }

    fn parse_try_statement(&mut self) -> Result<TryStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'try'

        let try_block = self.parse_block()?;
        let mut catch_clauses = Vec::new();

        while self.cursor.match_token(TokenKind::KeywordCatch).is_some() {
            let catch_start = self.cursor.current_span();
            if self
                .cursor
                .match_token(TokenKind::PunctuationOpenParen)
                .is_none()
            {
                return Err(ParsingError::MalformedCatchParameter {
                    span: catch_start.to_source_span(),
                });
            }

            if self.cursor.check(TokenKind::PunctuationCloseParen) {
                self.record_error(ParsingError::MalformedCatchParameter {
                    span: catch_start.to_source_span(),
                });
                self.cursor.advance(); // consume ')'
                let block = self.parse_block()?;
                let span = catch_start.combine(block.span);
                catch_clauses.push(CatchClause {
                    span,
                    exception_type: TypeAnnotation::Primitive {
                        span: catch_start,
                        kind: TokenKind::PrimitiveVoid,
                    },
                    variable_name: catch_start,
                    block,
                });
                continue;
            }

            let exception_type = self.parse_type_annotation()?;
            let variable_name = self.consume_identifier()?;

            if self
                .cursor
                .match_token(TokenKind::PunctuationCloseParen)
                .is_none()
            {
                return Err(ParsingError::ExpectedClosingDelimiter {
                    span: self.cursor.current_span().to_source_span(),
                });
            }

            if !self.cursor.check(TokenKind::PunctuationOpenBrace) {
                return Err(ParsingError::ExpectedCatchOrFinallyBlock {
                    span: self.cursor.current_span().to_source_span(),
                });
            }

            let block = self.parse_block()?;
            let span = catch_start.combine(block.span);

            catch_clauses.push(CatchClause {
                span,
                exception_type,
                variable_name,
                block,
            });
        }

        let finally_block = if self.cursor.match_token(TokenKind::KeywordFinally).is_some() {
            if !self.cursor.check(TokenKind::PunctuationOpenBrace) {
                return Err(ParsingError::ExpectedCatchOrFinallyBlock {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
            Some(self.parse_block()?)
        } else {
            None
        };

        if catch_clauses.is_empty() && finally_block.is_none() {
            self.record_error(ParsingError::TryMissingCatchOrFinally {
                span: start_span.to_source_span(),
            });
        }

        let end_span = finally_block
            .as_ref()
            .map(|block| block.span)
            .or_else(|| catch_clauses.last().map(|clause| clause.span))
            .unwrap_or(try_block.span);

        let span = start_span.combine(end_span);

        Ok(TryStatement {
            span,
            try_block,
            catch_clauses,
            finally_block,
        })
    }

    fn parse_throw_statement(&mut self) -> Result<ThrowStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'throw'

        let expression = self.parse_expression()?;
        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(ThrowStatement { span, expression })
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'return'

        if !self.inside_function() {
            self.record_error(ParsingError::ReturnOutsideFunction {
                span: start_span.to_source_span(),
            });
        }

        let value = if self.cursor.check(TokenKind::PunctuationSemicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(ReturnStatement { span, value })
    }

    fn parse_break_statement(&mut self) -> Result<BreakStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'break'
        let semi = self.consume_semicolon()?;
        Ok(BreakStatement {
            span: start_span.combine(semi.span),
        })
    }

    fn parse_continue_statement(&mut self) -> Result<ContinueStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'continue'
        let semi = self.consume_semicolon()?;
        Ok(ContinueStatement {
            span: start_span.combine(semi.span),
        })
    }

    fn parse_assert_statement(&mut self) -> Result<AssertStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'assert'

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedOpenParen {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let condition = self.parse_expression()?;

        let message = if self
            .cursor
            .match_token(TokenKind::PunctuationComma)
            .is_some()
        {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if self
            .cursor
            .match_token(TokenKind::PunctuationCloseParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(AssertStatement {
            span,
            condition,
            message,
        })
    }

    fn parse_exit_statement(&mut self) -> Result<ExitStatement, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume 'exit'

        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedOpenParen {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let code = self.parse_expression()?;

        if self
            .cursor
            .match_token(TokenKind::PunctuationCloseParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            });
        }

        let semi = self.consume_semicolon()?;
        let span = start_span.combine(semi.span);

        Ok(ExitStatement { span, code })
    }

    // =========================================================================
    // Expressions & Operator Precedence Climbing
    // =========================================================================

    fn parse_expression(&mut self) -> Result<Expression, ParsingError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParsingError> {
        let expression = self.parse_ternary()?;

        if let Some(op_kind) = self.match_assignment_operator() {
            if !self.is_lvalue(&expression) {
                self.record_error(ParsingError::InvalidAssignmentTarget {
                    span: expression.span().to_source_span(),
                });
            }
            let value = self.parse_assignment()?;
            let span = expression.span().combine(value.span());
            return Ok(Expression::Assignment {
                span,
                target: Box::new(expression),
                operator: op_kind,
                value: Box::new(value),
            });
        }

        Ok(expression)
    }

    fn match_assignment_operator(&mut self) -> Option<TokenKind> {
        match self.cursor.peek_kind() {
            TokenKind::OperatorAssign
            | TokenKind::OperatorPlusAssign
            | TokenKind::OperatorMinusAssign
            | TokenKind::OperatorMultiplyAssign
            | TokenKind::OperatorDivideAssign
            | TokenKind::OperatorModuloAssign => {
                let kind = self.cursor.peek_kind();
                self.cursor.advance();
                Some(kind)
            }
            _ => None,
        }
    }

    fn is_lvalue(&self, expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::Identifier { .. }
                | Expression::MemberAccess { .. }
                | Expression::ArrayAccess { .. }
        )
    }

    fn parse_ternary(&mut self) -> Result<Expression, ParsingError> {
        let condition = self.parse_logical_or()?;

        if self
            .cursor
            .match_token(TokenKind::OperatorQuestion)
            .is_some()
        {
            let then_branch = self.parse_expression()?;
            if self
                .cursor
                .match_token(TokenKind::PunctuationColon)
                .is_none()
            {
                return Err(ParsingError::ExpectedColonInTernary {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
            let else_branch = self.parse_ternary()?;
            let span = condition.span().combine(else_branch.span());
            return Ok(Expression::Ternary {
                span,
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            });
        }

        Ok(condition)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_logical_and()?;

        while self
            .cursor
            .match_token(TokenKind::OperatorLogicalOr)
            .is_some()
        {
            let right = self.parse_logical_and()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator: TokenKind::OperatorLogicalOr,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_bitwise_or()?;

        while self
            .cursor
            .match_token(TokenKind::OperatorLogicalAnd)
            .is_some()
        {
            let right = self.parse_bitwise_or()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator: TokenKind::OperatorLogicalAnd,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_bitwise_xor()?;

        while self
            .cursor
            .match_token(TokenKind::OperatorBitwiseOr)
            .is_some()
        {
            let right = self.parse_bitwise_xor()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator: TokenKind::OperatorBitwiseOr,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_bitwise_and()?;

        while self
            .cursor
            .match_token(TokenKind::OperatorBitwiseXor)
            .is_some()
        {
            let right = self.parse_bitwise_and()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator: TokenKind::OperatorBitwiseXor,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_equality()?;

        while self
            .cursor
            .match_token(TokenKind::PunctuationAmpersand)
            .is_some()
        {
            let right = self.parse_equality()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator: TokenKind::PunctuationAmpersand,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_comparison()?;

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::OperatorEqual | TokenKind::OperatorNotEqual
        ) {
            let operator = self
                .cursor
                .advance()
                .map(|token| token.kind)
                .unwrap_or(TokenKind::OperatorEqual);
            let right = self.parse_comparison()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_shift()?;

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::OperatorLessThan
                | TokenKind::OperatorGreaterThan
                | TokenKind::OperatorLessEqual
                | TokenKind::OperatorGreaterEqual
                | TokenKind::KeywordInstanceof
        ) {
            let op_kind = self.cursor.peek_kind();
            if op_kind == TokenKind::KeywordInstanceof {
                self.cursor.advance();
                let target_type = self.parse_type_annotation()?;
                let span = left.span().combine(target_type.span());
                left = Expression::Instanceof {
                    span,
                    expression: Box::new(left),
                    target_type,
                };
            } else {
                let operator = self
                    .cursor
                    .advance()
                    .map(|token| token.kind)
                    .unwrap_or(TokenKind::OperatorLessThan);
                let right = self.parse_shift()?;
                let span = left.span().combine(right.span());
                left = Expression::Binary {
                    span,
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            }
        }

        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_term()?;

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::OperatorShiftLeft | TokenKind::OperatorShiftRight
        ) {
            let operator = self
                .cursor
                .advance()
                .map(|token| token.kind)
                .unwrap_or(TokenKind::OperatorShiftLeft);
            let right = self.parse_term()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_factor()?;

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::OperatorPlus | TokenKind::OperatorMinus
        ) {
            let operator = self
                .cursor
                .advance()
                .map(|token| token.kind)
                .unwrap_or(TokenKind::OperatorPlus);
            let right = self.parse_factor()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParsingError> {
        let mut left = self.parse_unary()?;

        while matches!(
            self.cursor.peek_kind(),
            TokenKind::OperatorMultiply | TokenKind::OperatorDivide | TokenKind::OperatorModulo
        ) {
            let operator = self
                .cursor
                .advance()
                .map(|token| token.kind)
                .unwrap_or(TokenKind::OperatorMultiply);
            let right = self.parse_unary()?;
            let span = left.span().combine(right.span());
            left = Expression::Binary {
                span,
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParsingError> {
        if matches!(
            self.cursor.peek_kind(),
            TokenKind::OperatorLogicalNot
                | TokenKind::OperatorMinus
                | TokenKind::OperatorPlus
                | TokenKind::OperatorBitwiseNot
                | TokenKind::OperatorIncrement
                | TokenKind::OperatorDecrement
        ) {
            let op_token = self.cursor.advance().copied().unwrap_or_default();
            let operand = self.parse_unary()?;

            if matches!(
                op_token.kind,
                TokenKind::OperatorIncrement | TokenKind::OperatorDecrement
            ) && !self.is_lvalue(&operand)
            {
                self.record_error(ParsingError::InvalidIncrementTarget {
                    span: operand.span().to_source_span(),
                });
            }

            let span = op_token.span.combine(operand.span());
            return Ok(Expression::Unary {
                span,
                operator: op_token.kind,
                operand: Box::new(operand),
                is_prefix: true,
            });
        }

        self.parse_postfix()
    }

    fn is_generic_call_lookahead(&self) -> bool {
        if self.cursor.peek_kind() != TokenKind::OperatorLessThan {
            return false;
        }
        let mut nesting_depth = 1usize;
        let mut token_offset = 1usize;
        while let Some(token) = self.cursor.peek_nth(token_offset) {
            match token.kind {
                TokenKind::OperatorLessThan => {
                    nesting_depth += 1;
                }
                TokenKind::OperatorGreaterThan => {
                    nesting_depth -= 1;
                    if nesting_depth == 0 {
                        if let Some(next_token) = self.cursor.peek_nth(token_offset + 1) {
                            return next_token.kind == TokenKind::PunctuationOpenParen;
                        }
                        return false;
                    }
                }
                TokenKind::Identifier
                | TokenKind::PunctuationComma
                | TokenKind::PunctuationDot
                | TokenKind::PunctuationDoubleColon
                | TokenKind::PunctuationOpenBracket
                | TokenKind::PunctuationCloseBracket
                | TokenKind::OperatorQuestion
                | TokenKind::PrimitiveVoid
                | TokenKind::PrimitiveBool
                | TokenKind::PrimitiveInt8
                | TokenKind::PrimitiveInt16
                | TokenKind::PrimitiveInt32
                | TokenKind::PrimitiveInt64
                | TokenKind::PrimitiveUint8
                | TokenKind::PrimitiveUint16
                | TokenKind::PrimitiveUint32
                | TokenKind::PrimitiveUint64
                | TokenKind::PrimitiveFloat32
                | TokenKind::PrimitiveFloat64
                | TokenKind::PrimitiveChar
                | TokenKind::PrimitiveTuple => {
                    // Allowed tokens within generic type argument list
                }
                _ => return false,
            }
            token_offset += 1;
        }
        false
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParsingError> {
        let mut expression = self.parse_primary()?;

        loop {
            if self
                .cursor
                .match_token(TokenKind::PunctuationOpenParen)
                .is_some()
            {
                // Method call invocation
                let mut arguments = Vec::new();
                if !self.cursor.check(TokenKind::PunctuationCloseParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                }
                let close_paren = self.consume_close_paren()?;
                let span = expression.span().combine(close_paren.span);
                expression = Expression::MethodCall {
                    span,
                    callee: Box::new(expression),
                    generic_arguments: Vec::new(),
                    arguments,
                };
            } else if self.cursor.peek_kind() == TokenKind::OperatorLessThan
                && self.is_generic_call_lookahead()
            {
                self.cursor.advance();
                let mut generic_arguments = Vec::new();
                loop {
                    generic_arguments.push(self.parse_type_annotation()?);
                    if self
                        .cursor
                        .match_token(TokenKind::PunctuationComma)
                        .is_none()
                    {
                        break;
                    }
                }
                if self
                    .cursor
                    .match_token(TokenKind::OperatorGreaterThan)
                    .is_none()
                {
                    return Err(ParsingError::ExpectedCloseAngle {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }
                self.consume_open_paren()?;
                let mut arguments = Vec::new();
                if !self.cursor.check(TokenKind::PunctuationCloseParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                }
                let close_paren = self.consume_close_paren()?;
                let span = expression.span().combine(close_paren.span);
                expression = Expression::MethodCall {
                    span,
                    callee: Box::new(expression),
                    generic_arguments,
                    arguments,
                };
            } else if let Some(dot_token) = self.cursor.match_token(TokenKind::PunctuationDot) {
                let dot_span = dot_token.span;
                // Member access
                let member_span = match self.consume_identifier() {
                    Ok(ident_span) => ident_span,
                    Err(parsing_error) => {
                        self.errors.push(parsing_error);
                        Span::new(dot_span.id, dot_span.end, dot_span.end)
                    }
                };
                let span = expression.span().combine(member_span);
                expression = Expression::MemberAccess {
                    span,
                    object: Box::new(expression),
                    member_span,
                    is_scope_resolution: false,
                };
            } else if let Some(colon_token) =
                self.cursor.match_token(TokenKind::PunctuationDoubleColon)
            {
                let colon_span = colon_token.span;
                // Static member access
                let member_span = match self.consume_identifier() {
                    Ok(ident_span) => ident_span,
                    Err(parsing_error) => {
                        self.errors.push(parsing_error);
                        Span::new(colon_span.id, colon_span.end, colon_span.end)
                    }
                };
                let span = expression.span().combine(member_span);
                expression = Expression::MemberAccess {
                    span,
                    object: Box::new(expression),
                    member_span,
                    is_scope_resolution: true,
                };
            } else if self
                .cursor
                .match_token(TokenKind::PunctuationOpenBracket)
                .is_some()
            {
                // Array indexing
                if self.cursor.check(TokenKind::PunctuationCloseBracket) {
                    return Err(ParsingError::ExpectedExpressionInIndex {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }
                let index = self.parse_expression()?;
                let close_bracket = self.consume_close_bracket()?;
                let span = expression.span().combine(close_bracket.span);
                expression = Expression::ArrayAccess {
                    span,
                    array: Box::new(expression),
                    index: Box::new(index),
                };
            } else if self
                .cursor
                .match_token(TokenKind::PunctuationArrayBrackets)
                .is_some()
            {
                return Err(ParsingError::ExpectedExpressionInIndex {
                    span: self.cursor.current_span().to_source_span(),
                });
            } else if matches!(
                self.cursor.peek_kind(),
                TokenKind::OperatorIncrement | TokenKind::OperatorDecrement
            ) {
                let op_token = self.cursor.advance().copied().unwrap_or_default();
                if !self.is_lvalue(&expression) {
                    self.record_error(ParsingError::InvalidIncrementTarget {
                        span: expression.span().to_source_span(),
                    });
                }
                let span = expression.span().combine(op_token.span);
                expression = Expression::Unary {
                    span,
                    operator: op_token.kind,
                    operand: Box::new(expression),
                    is_prefix: false,
                };
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParsingError> {
        let token = self.cursor.peek().copied().unwrap_or_default();

        match token.kind {
            TokenKind::Number => {
                self.cursor.advance();
                let literal_val = self.parse_numeric_literal(token.span)?;
                Ok(Expression::Literal {
                    span: token.span,
                    value: literal_val,
                })
            }
            TokenKind::String => {
                self.cursor.advance();
                let decoded_str = self.parse_string_literal(token.span)?;
                Ok(Expression::Literal {
                    span: token.span,
                    value: LiteralValue::String(decoded_str),
                })
            }
            TokenKind::Char => {
                self.cursor.advance();
                let decoded_char = self.parse_char_literal(token.span)?;
                Ok(Expression::Literal {
                    span: token.span,
                    value: LiteralValue::Char(decoded_char),
                })
            }
            TokenKind::Identifier => {
                let lexeme = self.get_lexeme(token.span);
                if lexeme == "true" {
                    self.cursor.advance();
                    Ok(Expression::Literal {
                        span: token.span,
                        value: LiteralValue::Bool(true),
                    })
                } else if lexeme == "false" {
                    self.cursor.advance();
                    Ok(Expression::Literal {
                        span: token.span,
                        value: LiteralValue::Bool(false),
                    })
                } else if lexeme == "null" {
                    self.cursor.advance();
                    Ok(Expression::Literal {
                        span: token.span,
                        value: LiteralValue::Null,
                    })
                } else if lexeme == "cast"
                    && self.cursor.check_next(TokenKind::PunctuationOpenParen)
                {
                    self.cursor.advance(); // consume 'cast'
                    self.cursor.advance(); // consume '('
                    let expr = self.parse_expression()?;
                    if self
                        .cursor
                        .match_token(TokenKind::PunctuationComma)
                        .is_none()
                    {
                        return Err(ParsingError::ExpectedCommaInCast {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    }
                    if !self.is_type_start(self.cursor.peek_kind()) {
                        return Err(ParsingError::ExpectedTypeInCast {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    }
                    let target_type = self.parse_type_annotation()?;
                    let close_paren = self.consume_close_paren()?;
                    let span = token.span.combine(close_paren.span);
                    Ok(Expression::Cast {
                        span,
                        expression: Box::new(expr),
                        target_type,
                    })
                } else {
                    self.cursor.advance();
                    Ok(Expression::Identifier { span: token.span })
                }
            }
            TokenKind::KeywordSuper => {
                self.cursor.advance();
                Ok(Expression::Identifier { span: token.span })
            }
            TokenKind::KeywordNew => {
                let new_span = token.span;
                self.cursor.advance(); // consume 'new'
                let inst_type = self.parse_type_annotation()?;

                if self
                    .cursor
                    .match_token(TokenKind::PunctuationOpenBracket)
                    .is_some()
                {
                    // Array creation: new Type[size]
                    if self.cursor.check(TokenKind::PunctuationCloseBracket) {
                        return Err(ParsingError::ArrayAllocationRequiresSize {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    }
                    let size_expr = self.parse_expression()?;
                    let close_bracket = self.consume_close_bracket()?;
                    let span = new_span.combine(close_bracket.span);
                    Ok(Expression::ArrayCreation {
                        span,
                        element_type: inst_type,
                        size: Box::new(size_expr),
                    })
                } else if matches!(inst_type, TypeAnnotation::Array { .. }) {
                    self.record_error(ParsingError::ArrayAllocationRequiresSize {
                        span: inst_type.span().to_source_span(),
                    });
                    Ok(Expression::ArrayCreation {
                        span: new_span.combine(inst_type.span()),
                        element_type: inst_type,
                        size: Box::new(Expression::Literal {
                            span: new_span,
                            value: LiteralValue::Int(0),
                        }),
                    })
                } else if self
                    .cursor
                    .match_token(TokenKind::PunctuationOpenParen)
                    .is_some()
                {
                    // Object instantiation: new Type(args)
                    let arguments = self.parse_argument_list_tail()?;
                    let close_paren = self
                        .cursor
                        .previous()
                        .map(|token| token.span)
                        .unwrap_or(new_span);
                    let span = new_span.combine(close_paren);
                    Ok(Expression::NewInstance {
                        span,
                        instance_type: inst_type,
                        arguments,
                    })
                } else {
                    Err(ParsingError::ExpectedConstructorCallOrSize {
                        span: self.cursor.current_span().to_source_span(),
                    })
                }
            }
            TokenKind::PunctuationArrayBrackets => {
                if self.cursor.check_next(TokenKind::PunctuationOpenParen) {
                    let start_span = token.span;
                    self.cursor.advance(); // consume '[]'
                    self.consume_open_paren()?;
                    let parameters = self.parse_lambda_parameters()?;

                    if self
                        .cursor
                        .match_token(TokenKind::PunctuationColon)
                        .is_some()
                    {
                        let _return_type = self.parse_type_annotation()?;
                    }

                    let body = if self
                        .cursor
                        .match_token(TokenKind::OperatorFatArrow)
                        .is_some()
                    {
                        if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                            LambdaBody::Block(self.parse_block()?)
                        } else {
                            LambdaBody::Expression(Box::new(self.parse_expression()?))
                        }
                    } else if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                        LambdaBody::Block(self.parse_block()?)
                    } else {
                        return Err(ParsingError::ExpectedLambdaArrow {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    };

                    let end_span = match &body {
                        LambdaBody::Expression(body_expr) => body_expr.span(),
                        LambdaBody::Block(block) => block.span,
                    };

                    Ok(Expression::Lambda {
                        span: start_span.combine(end_span),
                        captures: Vec::new(),
                        parameters,
                        body,
                    })
                } else {
                    self.cursor.advance(); // consume '[]'
                    Ok(Expression::ArrayLiteral {
                        span: token.span,
                        elements: Vec::new(),
                    })
                }
            }
            TokenKind::PunctuationOpenBracket => {
                if self.is_lambda_with_captures_lookahead() {
                    return self.parse_lambda_with_captures();
                }
                // Array literal: [a, b, c]
                let start_span = token.span;
                self.cursor.advance(); // consume '['
                let mut elements = Vec::new();
                if !self.cursor.check(TokenKind::PunctuationCloseBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                }
                let close_bracket = self.consume_close_bracket()?;
                let span = start_span.combine(close_bracket.span);
                Ok(Expression::ArrayLiteral { span, elements })
            }
            TokenKind::PunctuationOpenBrace => {
                // Array literal with braces: {a, b, c}
                let start_span = token.span;
                self.cursor.advance(); // consume '{'
                let mut elements = Vec::new();
                if !self.cursor.check(TokenKind::PunctuationCloseBrace) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                }
                let close_brace = self.consume_close_brace()?;
                let span = start_span.combine(close_brace.span);
                Ok(Expression::ArrayLiteral { span, elements })
            }
            TokenKind::KeywordSizeof => {
                let start_span = token.span;
                self.cursor.advance(); // consume 'sizeof'
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationOpenParen)
                    .is_none()
                {
                    return Err(ParsingError::ExpectedOpenParen {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }
                let target_type = self.parse_type_annotation()?;
                let close_paren = self.consume_close_paren()?;
                let span = start_span.combine(close_paren.span);
                Ok(Expression::Sizeof { span, target_type })
            }
            TokenKind::KeywordDefault => {
                let start_span = token.span;
                self.cursor.advance(); // consume 'default'
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationOpenParen)
                    .is_none()
                {
                    return Err(ParsingError::ExpectedOpenParen {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }
                let target_type = self.parse_type_annotation()?;
                let close_paren = self.consume_close_paren()?;
                let span = start_span.combine(close_paren.span);
                Ok(Expression::Default { span, target_type })
            }
            TokenKind::PrimitiveTuple => {
                // tuple(a, b, c)
                let start_span = token.span;
                self.cursor.advance(); // consume 'tuple'
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationOpenParen)
                    .is_none()
                {
                    return Err(ParsingError::ExpectedOpenParen {
                        span: self.cursor.current_span().to_source_span(),
                    });
                }
                let mut elements = Vec::new();
                if !self.cursor.check(TokenKind::PunctuationCloseParen) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                }
                let close_paren = self.consume_close_paren()?;
                if elements.is_empty() {
                    return Err(ParsingError::EmptyTupleExpression {
                        span: start_span.combine(close_paren.span).to_source_span(),
                    });
                }
                let span = start_span.combine(close_paren.span);
                Ok(Expression::TupleCreation { span, elements })
            }
            TokenKind::PunctuationOpenParen => {
                let open_paren = token.span;
                self.cursor.advance(); // consume '('

                // Disambiguate: could be grouping `(expr)`, tuple `(a, b)`, or lambda `(x, y) => ...`
                if self.cursor.check(TokenKind::PunctuationCloseParen) {
                    // Empty parens followed by => is a zero-arg lambda
                    self.cursor.advance(); // consume ')'
                    if self
                        .cursor
                        .match_token(TokenKind::OperatorFatArrow)
                        .is_some()
                    {
                        let body = if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                            LambdaBody::Block(self.parse_block()?)
                        } else {
                            LambdaBody::Expression(Box::new(self.parse_expression()?))
                        };
                        let end_span = match &body {
                            LambdaBody::Expression(body_expr) => body_expr.span(),
                            LambdaBody::Block(block) => block.span,
                        };
                        return Ok(Expression::Lambda {
                            span: open_paren.combine(end_span),
                            captures: Vec::new(),
                            parameters: Vec::new(),
                            body,
                        });
                    }
                    return Err(ParsingError::ExpectedExpression {
                        span: open_paren.to_source_span(),
                    });
                }

                // Check for typed or untyped parameters followed by ')' and '=>'
                if self.is_lambda_lookahead() {
                    let parameters = self.parse_lambda_parameters()?;
                    if self
                        .cursor
                        .match_token(TokenKind::PunctuationColon)
                        .is_some()
                    {
                        let _return_type = self.parse_type_annotation()?;
                    }
                    let body = if self
                        .cursor
                        .match_token(TokenKind::OperatorFatArrow)
                        .is_some()
                    {
                        if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                            LambdaBody::Block(self.parse_block()?)
                        } else {
                            LambdaBody::Expression(Box::new(self.parse_expression()?))
                        }
                    } else if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                        LambdaBody::Block(self.parse_block()?)
                    } else {
                        return Err(ParsingError::ExpectedLambdaArrow {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    };
                    let end_span = match &body {
                        LambdaBody::Expression(body_expr) => body_expr.span(),
                        LambdaBody::Block(block) => block.span,
                    };
                    return Ok(Expression::Lambda {
                        span: open_paren.combine(end_span),
                        captures: Vec::new(),
                        parameters,
                        body,
                    });
                }

                // Check for C-style cast: (Type)expr
                if self.is_type_start(self.cursor.peek_kind()) {
                    let saved_pos = self.cursor.position;
                    let saved_errors = self.errors.len();
                    let cast_attempt =
                        (|| -> Result<(TypeAnnotation, Expression), ParsingError> {
                            let target_type = self.parse_type_annotation()?;
                            self.consume_close_paren()?;
                            let peek = self.cursor.peek_kind();
                            if !matches!(
                                peek,
                                TokenKind::PunctuationSemicolon
                                    | TokenKind::PunctuationCloseParen
                                    | TokenKind::PunctuationCloseBracket
                                    | TokenKind::PunctuationCloseBrace
                                    | TokenKind::PunctuationComma
                                    | TokenKind::OperatorFatArrow
                            ) && self.is_expression_start(peek)
                            {
                                let expr = self.parse_unary()?;
                                Ok((target_type, expr))
                            } else {
                                Err(ParsingError::ExpectedExpression {
                                    span: self.cursor.current_span().to_source_span(),
                                })
                            }
                        })();
                    if let Ok((target_type, expression)) = cast_attempt {
                        return Ok(Expression::Cast {
                            span: open_paren.combine(expression.span()),
                            expression: Box::new(expression),
                            target_type,
                        });
                    }
                    self.cursor.position = saved_pos;
                    self.errors.truncate(saved_errors);
                }

                let first_expr = self.parse_expression()?;

                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_some()
                {
                    // Tuple: (a, b, ...)
                    let mut elements = vec![first_expr];
                    loop {
                        elements.push(self.parse_expression()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                    let close_paren = self.consume_close_paren()?;
                    let span = open_paren.combine(close_paren.span);
                    return Ok(Expression::TupleCreation { span, elements });
                }

                let _close_paren = self.consume_close_paren()?;

                // Check if this was a single parameter lambda: (x) => expr
                if self
                    .cursor
                    .match_token(TokenKind::OperatorFatArrow)
                    .is_some()
                {
                    let param_name_span = first_expr.span();
                    let body = if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                        LambdaBody::Block(self.parse_block()?)
                    } else {
                        LambdaBody::Expression(Box::new(self.parse_expression()?))
                    };
                    let end_span = match &body {
                        LambdaBody::Expression(body_expr) => body_expr.span(),
                        LambdaBody::Block(block) => block.span,
                    };
                    return Ok(Expression::Lambda {
                        span: open_paren.combine(end_span),
                        captures: Vec::new(),
                        parameters: vec![Parameter {
                            span: param_name_span,
                            name_span: param_name_span,
                            param_type: TypeAnnotation::Named {
                                span: param_name_span,
                                path: vec![param_name_span],
                            },
                            default_value: None,
                            is_variadic: false,
                        }],
                        body,
                    });
                }

                Ok(first_expr)
            }
            TokenKind::OperatorQuestion => {
                self.cursor.advance();
                Err(ParsingError::UnexpectedQuestionOperator {
                    span: token.span.to_source_span(),
                })
            }
            _ => Err(ParsingError::ExpectedExpression {
                span: token.span.to_source_span(),
            }),
        }
    }

    fn is_lambda_with_captures_lookahead(&self) -> bool {
        let mut depth = 1;
        let mut idx = self.cursor.position + 1;

        while idx < self.cursor.tokens.len() && depth > 0 {
            match self.cursor.tokens[idx].kind {
                TokenKind::PunctuationOpenBracket => depth += 1,
                TokenKind::PunctuationCloseBracket => depth -= 1,
                TokenKind::PunctuationSemicolon
                | TokenKind::PunctuationOpenBrace
                | TokenKind::PunctuationCloseBrace
                | TokenKind::Eof => return false,
                _ => {}
            }
            idx += 1;
        }

        if depth == 0 && idx < self.cursor.tokens.len() {
            self.cursor.tokens[idx].kind == TokenKind::PunctuationOpenParen
        } else {
            false
        }
    }

    fn parse_lambda_with_captures(&mut self) -> Result<Expression, ParsingError> {
        let start_span = self.cursor.current_span();
        self.cursor.advance(); // consume '['
        let mut captures = Vec::new();

        if !self.cursor.check(TokenKind::PunctuationCloseBracket) {
            loop {
                let cap_span = self.consume_identifier()?;
                captures.push(cap_span);

                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
        }
        self.consume_close_bracket()?;

        self.consume_open_paren()?;
        let parameters = self.parse_lambda_parameters()?;

        if self
            .cursor
            .match_token(TokenKind::PunctuationColon)
            .is_some()
        {
            let _return_type = self.parse_type_annotation()?;
        }

        let body = if self
            .cursor
            .match_token(TokenKind::OperatorFatArrow)
            .is_some()
        {
            if self.cursor.check(TokenKind::PunctuationOpenBrace) {
                LambdaBody::Block(self.parse_block()?)
            } else {
                LambdaBody::Expression(Box::new(self.parse_expression()?))
            }
        } else if self.cursor.check(TokenKind::PunctuationOpenBrace) {
            LambdaBody::Block(self.parse_block()?)
        } else {
            return Err(ParsingError::ExpectedLambdaArrow {
                span: self.cursor.current_span().to_source_span(),
            });
        };

        let end_span = match &body {
            LambdaBody::Expression(body_expr) => body_expr.span(),
            LambdaBody::Block(block) => block.span,
        };

        Ok(Expression::Lambda {
            span: start_span.combine(end_span),
            captures,
            parameters,
            body,
        })
    }

    fn is_lambda_lookahead(&self) -> bool {
        // Scans forward to find matching ')' followed by '=>'
        let mut depth = 1;
        let mut idx = self.cursor.position;

        while idx < self.cursor.tokens.len() && depth > 0 {
            match self.cursor.tokens[idx].kind {
                TokenKind::PunctuationOpenParen => depth += 1,
                TokenKind::PunctuationCloseParen => depth -= 1,
                TokenKind::PunctuationSemicolon | TokenKind::PunctuationOpenBrace => return false,
                _ => {}
            }
            idx += 1;
        }

        if depth == 0 && idx < self.cursor.tokens.len() {
            self.cursor.tokens[idx].kind == TokenKind::OperatorFatArrow
        } else {
            false
        }
    }

    fn parse_lambda_parameters(&mut self) -> Result<Vec<Parameter>, ParsingError> {
        let mut parameters = Vec::new();

        if !self.cursor.check(TokenKind::PunctuationCloseParen) {
            loop {
                let start_span = self.cursor.current_span();
                let (param_type, name_span) = if self.is_type_start(self.cursor.peek_kind())
                    && self.cursor.check_next(TokenKind::Identifier)
                {
                    let parsed_type = self.parse_type_annotation()?;
                    let parsed_name = self.consume_identifier()?;
                    (parsed_type, parsed_name)
                } else {
                    let parsed_name = self.consume_identifier()?;
                    (
                        TypeAnnotation::Named {
                            span: parsed_name,
                            path: vec![parsed_name],
                        },
                        parsed_name,
                    )
                };

                let span = start_span.combine(name_span);
                parameters.push(Parameter {
                    span,
                    name_span,
                    param_type,
                    default_value: None,
                    is_variadic: false,
                });

                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
        }

        self.consume_close_paren()?;
        Ok(parameters)
    }

    fn parse_argument_list(&mut self) -> Result<Vec<Expression>, ParsingError> {
        self.consume_open_paren()?;
        self.parse_argument_list_tail()
    }

    fn parse_argument_list_tail(&mut self) -> Result<Vec<Expression>, ParsingError> {
        let mut arguments = Vec::new();
        if !self.cursor.check(TokenKind::PunctuationCloseParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
        }
        self.consume_close_paren()?;
        Ok(arguments)
    }

    // =========================================================================
    // Type Annotations & Generics Parsing
    // =========================================================================

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParsingError> {
        let token = self.cursor.peek().copied().unwrap_or_default();

        let mut base_type = match token.kind {
            TokenKind::PunctuationOpenParen => {
                let open_span = token.span;
                self.cursor.advance();
                let mut elements = Vec::new();
                while !self.cursor.check(TokenKind::PunctuationCloseParen)
                    && !self.cursor.is_at_end()
                {
                    elements.push(self.parse_type_annotation()?);
                    if self
                        .cursor
                        .match_token(TokenKind::PunctuationComma)
                        .is_none()
                    {
                        break;
                    }
                }
                let Some(close_token) = self.cursor.match_token(TokenKind::PunctuationCloseParen)
                else {
                    return Err(ParsingError::ExpectedClosingDelimiter {
                        span: self.cursor.current_span().to_source_span(),
                    });
                };
                let span = open_span.combine(close_token.span);
                TypeAnnotation::Tuple { span, elements }
            }
            TokenKind::PrimitiveVoid
            | TokenKind::PrimitiveBool
            | TokenKind::PrimitiveInt8
            | TokenKind::PrimitiveInt16
            | TokenKind::PrimitiveInt32
            | TokenKind::PrimitiveInt64
            | TokenKind::PrimitiveUint8
            | TokenKind::PrimitiveUint16
            | TokenKind::PrimitiveUint32
            | TokenKind::PrimitiveUint64
            | TokenKind::PrimitiveFloat32
            | TokenKind::PrimitiveFloat64
            | TokenKind::PrimitiveChar => {
                self.cursor.advance();
                TypeAnnotation::Primitive {
                    span: token.span,
                    kind: token.kind,
                }
            }
            TokenKind::PrimitiveTuple => {
                self.cursor.advance();
                let mut elements = Vec::new();
                if self
                    .cursor
                    .match_token(TokenKind::OperatorLessThan)
                    .is_some()
                {
                    loop {
                        elements.push(self.parse_type_annotation()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                    if self
                        .cursor
                        .match_token(TokenKind::OperatorGreaterThan)
                        .is_none()
                    {
                        return Err(ParsingError::ExpectedCloseAngle {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    }
                }
                let span = token.span.combine(
                    elements
                        .last()
                        .map(|elem| elem.span())
                        .unwrap_or(token.span),
                );
                TypeAnnotation::Tuple { span, elements }
            }
            TokenKind::Identifier => {
                let mut path = vec![token.span];
                self.cursor.advance();

                if self.cursor.check(TokenKind::PunctuationDot) {
                    let dot_token = self.cursor.advance().copied().unwrap_or_default();
                    return Err(ParsingError::DotNotAllowedInTypePath {
                        span: dot_token.span.to_source_span(),
                    });
                }

                while self
                    .cursor
                    .match_token(TokenKind::PunctuationDoubleColon)
                    .is_some()
                {
                    let segment = self.consume_identifier()?;
                    path.push(segment);
                    if self.cursor.check(TokenKind::PunctuationDot) {
                        let dot_token = self.cursor.advance().copied().unwrap_or_default();
                        return Err(ParsingError::DotNotAllowedInTypePath {
                            span: dot_token.span.to_source_span(),
                        });
                    }
                }

                let named_span = token
                    .span
                    .combine(path.last().copied().unwrap_or(token.span));
                let named = TypeAnnotation::Named {
                    span: named_span,
                    path,
                };

                if self
                    .cursor
                    .match_token(TokenKind::OperatorLessThan)
                    .is_some()
                {
                    let mut arguments = Vec::new();
                    loop {
                        arguments.push(self.parse_type_annotation()?);
                        if self
                            .cursor
                            .match_token(TokenKind::PunctuationComma)
                            .is_none()
                        {
                            break;
                        }
                        if self.cursor.check(TokenKind::PunctuationCloseParen) {
                            break;
                        }
                    }
                    if self
                        .cursor
                        .match_token(TokenKind::OperatorGreaterThan)
                        .is_none()
                    {
                        return Err(ParsingError::ExpectedCloseAngle {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    }
                    let end_span = self
                        .cursor
                        .previous()
                        .map(|token| token.span)
                        .unwrap_or(named_span);
                    TypeAnnotation::Generic {
                        span: named_span.combine(end_span),
                        base: Box::new(named),
                        arguments,
                    }
                } else {
                    named
                }
            }
            _ => {
                return Err(ParsingError::ExpectedTypeName {
                    span: token.span.to_source_span(),
                });
            }
        };

        // Function pointer extension: RetType(*)(ParamTypes)
        if self.cursor.check(TokenKind::PunctuationOpenParen)
            && self.cursor.check_next(TokenKind::OperatorMultiply)
        {
            self.cursor.advance(); // consume '('
            self.cursor.advance(); // consume '*'
            if self
                .cursor
                .match_token(TokenKind::PunctuationCloseParen)
                .is_none()
            {
                return Err(ParsingError::MalformedFunctionPointerType {
                    span: self.cursor.current_span().to_source_span(),
                });
            }

            let mut parameters = Vec::new();
            if self
                .cursor
                .match_token(TokenKind::PunctuationOpenParen)
                .is_none()
            {
                return Err(ParsingError::MalformedFunctionPointerType {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
            if !self.cursor.check(TokenKind::PunctuationCloseParen) {
                loop {
                    parameters.push(self.parse_type_annotation()?);
                    if self
                        .cursor
                        .match_token(TokenKind::PunctuationComma)
                        .is_none()
                    {
                        break;
                    }
                }
            }
            let close_paren = self.consume_close_paren()?;
            let span = base_type.span().combine(close_paren.span);
            base_type = TypeAnnotation::FunctionPointer {
                span,
                return_type: Box::new(base_type),
                parameters,
            };
        }

        // Array depth handling: int32[] or string[][]
        while self
            .cursor
            .match_token(TokenKind::PunctuationArrayBrackets)
            .is_some()
        {
            let end_span = self
                .cursor
                .previous()
                .map(|token| token.span)
                .unwrap_or_else(|| base_type.span());
            let span = base_type.span().combine(end_span);
            let current_depth = match &base_type {
                TypeAnnotation::Array { depth, .. } => *depth + 1,
                _ => 1,
            };
            base_type = TypeAnnotation::Array {
                span,
                element_type: Box::new(base_type),
                depth: current_depth,
            };
        }

        Ok(base_type)
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParameter>, ParsingError> {
        let mut params = Vec::new();
        if self
            .cursor
            .match_token(TokenKind::OperatorLessThan)
            .is_some()
        {
            loop {
                let name_span = self.consume_identifier()?;
                let is_variadic = self
                    .cursor
                    .match_token(TokenKind::PunctuationEllipsis)
                    .is_some();
                let mut bounds = Vec::new();
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationColon)
                    .is_some()
                {
                    if self.cursor.check(TokenKind::OperatorGreaterThan)
                        || self.cursor.check(TokenKind::PunctuationComma)
                    {
                        return Err(ParsingError::MalformedTypeBound {
                            span: self.cursor.current_span().to_source_span(),
                        });
                    }
                    loop {
                        bounds.push(self.parse_type_annotation()?);
                        if self.cursor.match_token(TokenKind::OperatorPlus).is_none() {
                            break;
                        }
                    }
                }
                let span =
                    name_span.combine(bounds.last().map(|node| node.span()).unwrap_or(name_span));
                if is_variadic && self.cursor.check(TokenKind::PunctuationComma) {
                    self.record_error(ParsingError::VariadicParameterNotLast {
                        span: name_span.to_source_span(),
                    });
                }
                params.push(GenericParameter {
                    span,
                    name_span,
                    bounds,
                    is_variadic,
                });
                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
            if self
                .cursor
                .match_token(TokenKind::OperatorGreaterThan)
                .is_none()
            {
                return Err(ParsingError::ExpectedCloseAngle {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
        }
        Ok(params)
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParsingError> {
        if self
            .cursor
            .match_token(TokenKind::PunctuationOpenParen)
            .is_none()
        {
            return Err(ParsingError::ExpectedParameterList {
                span: self.cursor.current_span().to_source_span(),
            });
        }
        let mut parameters = Vec::new();

        if !self.cursor.check(TokenKind::PunctuationCloseParen) {
            loop {
                let start_span = self.cursor.current_span();
                let mut param_type = self.parse_type_annotation()?;

                let is_variadic = self
                    .cursor
                    .match_token(TokenKind::PunctuationEllipsis)
                    .is_some();
                let name_span = if is_variadic
                    && (self.cursor.check(TokenKind::PunctuationCloseParen)
                        || self.cursor.check(TokenKind::PunctuationComma))
                {
                    let span = param_type.span();
                    param_type = TypeAnnotation::Named {
                        span,
                        path: vec![span],
                    };
                    span
                } else if self.cursor.check(TokenKind::Identifier) {
                    self.consume_identifier()?
                } else {
                    return Err(ParsingError::ExpectedIdentifierInParameter {
                        span: self.cursor.current_span().to_source_span(),
                    });
                };

                let default_value = if self.cursor.match_token(TokenKind::OperatorAssign).is_some()
                {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                let span = start_span.combine(
                    default_value
                        .as_ref()
                        .map(|expr| expr.span())
                        .unwrap_or(name_span),
                );

                if is_variadic && self.cursor.check(TokenKind::PunctuationComma) {
                    self.record_error(ParsingError::VariadicParameterNotLast {
                        span: name_span.to_source_span(),
                    });
                }

                parameters.push(Parameter {
                    span,
                    name_span,
                    param_type,
                    default_value,
                    is_variadic,
                });

                if self
                    .cursor
                    .match_token(TokenKind::PunctuationComma)
                    .is_none()
                {
                    break;
                }
            }
        }

        self.consume_close_paren()?;
        Ok(parameters)
    }

    // =========================================================================
    // Literal Parsing & Validation Helpers
    // =========================================================================

    fn parse_numeric_literal(&mut self, span: Span) -> Result<LiteralValue, ParsingError> {
        let raw = self.get_lexeme(span);
        let stripped_lexeme;
        let raw = if raw.contains('_') {
            stripped_lexeme = raw.replace('_', "");
            stripped_lexeme.as_str()
        } else {
            raw
        };

        // Check for float
        if raw.contains('.')
            || raw.contains('e')
            || raw.contains('E')
            || raw.ends_with('f')
            || raw.ends_with('F')
        {
            let clean = raw.trim_end_matches(['f', 'F']);
            if let Ok(float_val) = clean.parse::<f64>() {
                return Ok(LiteralValue::Float(float_val));
            }
        }

        // Check for hex or binary
        if raw.starts_with("0x") || raw.starts_with("0X") {
            let clean = raw[2..].trim_end_matches(['l', 'L']);
            if let Ok(hex_val) = u64::from_str_radix(clean, 16) {
                return Ok(LiteralValue::UInt(hex_val));
            }
            self.record_error(ParsingError::IntegerLiteralOverflow {
                span: span.to_source_span(),
            });
            return Ok(LiteralValue::UInt(u64::MAX));
        }

        if raw.starts_with("0b") || raw.starts_with("0B") {
            let clean = raw[2..].trim_end_matches(['l', 'L']);
            if let Ok(bin_val) = u64::from_str_radix(clean, 2) {
                return Ok(LiteralValue::UInt(bin_val));
            }
            self.record_error(ParsingError::IntegerLiteralOverflow {
                span: span.to_source_span(),
            });
            return Ok(LiteralValue::UInt(u64::MAX));
        }

        // Decimal integer
        let clean = raw.trim_end_matches(['l', 'L']);
        match clean.parse::<i64>() {
            Ok(int_val) => Ok(LiteralValue::Int(int_val)),
            Err(_) => match clean.parse::<u64>() {
                Ok(uint_val) => Ok(LiteralValue::UInt(uint_val)),
                Err(_) => {
                    self.record_error(ParsingError::IntegerLiteralOverflow {
                        span: span.to_source_span(),
                    });
                    Ok(LiteralValue::UInt(u64::MAX))
                }
            },
        }
    }

    fn parse_string_literal(&mut self, span: Span) -> Result<String, ParsingError> {
        let raw = self.get_lexeme(span);
        if !raw.starts_with('"') || !raw.ends_with('"') || raw.len() < 2 {
            return Ok(raw.to_string());
        }

        let inner = &raw[1..raw.len() - 1];
        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('\'') => result.push('\''),
                    Some('"') => result.push('"'),
                    Some('0') => result.push('\0'),
                    Some('x') => {
                        let hex_one = chars.next();
                        let hex_two = chars.next();
                        if let (Some(first), Some(second)) = (hex_one, hex_two) {
                            let hex_str = format!("{}{}", first, second);
                            if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                                result.push(byte_val as char);
                            } else {
                                self.record_error(ParsingError::InvalidEscapeSequence {
                                    span: span.to_source_span(),
                                });
                            }
                        } else {
                            self.record_error(ParsingError::InvalidEscapeSequence {
                                span: span.to_source_span(),
                            });
                        }
                    }
                    _ => {
                        self.record_error(ParsingError::InvalidEscapeSequence {
                            span: span.to_source_span(),
                        });
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn parse_char_literal(&mut self, span: Span) -> Result<char, ParsingError> {
        let raw = self.get_lexeme(span);
        if !raw.starts_with('\'') || !raw.ends_with('\'') || raw.len() < 2 {
            self.record_error(ParsingError::EmptyCharacterLiteral {
                span: span.to_source_span(),
            });
            return Ok('\0');
        }

        let inner = &raw[1..raw.len() - 1];
        if inner.is_empty() {
            self.record_error(ParsingError::EmptyCharacterLiteral {
                span: span.to_source_span(),
            });
            return Ok('\0');
        }

        let mut chars = inner.chars();
        let first = chars.next().unwrap_or('\0');

        if first == '\\' {
            match chars.next() {
                Some('n') => Ok('\n'),
                Some('t') => Ok('\t'),
                Some('r') => Ok('\r'),
                Some('\\') => Ok('\\'),
                Some('\'') => Ok('\''),
                Some('"') => Ok('"'),
                Some('0') => Ok('\0'),
                Some('x') => {
                    let hex_one = chars.next();
                    let hex_two = chars.next();
                    if let (Some(first_hex), Some(second_hex)) = (hex_one, hex_two) {
                        let hex_str = format!("{}{}", first_hex, second_hex);
                        if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                            Ok(byte_val as char)
                        } else {
                            self.record_error(ParsingError::InvalidEscapeSequence {
                                span: span.to_source_span(),
                            });
                            Ok('\0')
                        }
                    } else {
                        self.record_error(ParsingError::InvalidEscapeSequence {
                            span: span.to_source_span(),
                        });
                        Ok('\0')
                    }
                }
                _ => {
                    self.record_error(ParsingError::InvalidEscapeSequence {
                        span: span.to_source_span(),
                    });
                    Ok('\0')
                }
            }
        } else {
            if chars.next().is_some() {
                self.record_error(ParsingError::MultiCharacterLiteral {
                    span: span.to_source_span(),
                });
            }
            Ok(first)
        }
    }

    // =========================================================================
    // Delimiter & Token Helpers
    // =========================================================================

    fn consume_identifier(&mut self) -> Result<Span, ParsingError> {
        if self.cursor.check(TokenKind::Identifier) {
            let token = self.cursor.advance().copied().unwrap_or_default();
            Ok(token.span)
        } else {
            Err(ParsingError::ExpectedIdentifier {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    /// Parses a potentially namespace-qualified identifier sequence (`a::b::Name`).
    ///
    /// Returns `(namespace_path, name_span)` where `namespace_path` contains the leading
    /// namespace segments and `name_span` points to the final declared identifier.
    fn parse_qualified_name(&mut self) -> Result<(Vec<Span>, Span), ParsingError> {
        let mut segments = Vec::new();
        let first_segment = self.consume_identifier()?;
        segments.push(first_segment);
        while self
            .cursor
            .match_token(TokenKind::PunctuationDoubleColon)
            .is_some()
        {
            let next_segment = self.consume_identifier()?;
            segments.push(next_segment);
        }
        let name_span = match segments.pop() {
            Some(last_span) => last_span,
            None => {
                return Err(ParsingError::ExpectedIdentifier {
                    span: self.cursor.current_span().to_source_span(),
                });
            }
        };
        let namespace_path = segments;
        Ok((namespace_path, name_span))
    }

    fn consume_semicolon(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationSemicolon) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else if self.is_expression_start(self.cursor.peek_kind()) {
            Err(ParsingError::MissingOperatorInExpression {
                span: self.cursor.current_span().to_source_span(),
            })
        } else {
            Err(ParsingError::ExpectedSemicolon {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    fn consume_open_paren(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationOpenParen) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else {
            Err(ParsingError::ExpectedOpenParen {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    fn consume_close_paren(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationCloseParen) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else {
            Err(ParsingError::ExpectedClosingDelimiter {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    #[allow(dead_code)]
    fn consume_open_bracket(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationOpenBracket) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else {
            Err(ParsingError::ExpectedOpenBracket {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    fn consume_close_bracket(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationCloseBracket) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else {
            Err(ParsingError::ExpectedCloseBracket {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    fn consume_open_brace(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationOpenBrace) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else {
            Err(ParsingError::ExpectedOpenBrace {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    fn consume_close_brace(&mut self) -> Result<Token, ParsingError> {
        if self.cursor.check(TokenKind::PunctuationCloseBrace) {
            Ok(self.cursor.advance().copied().unwrap_or_default())
        } else {
            Err(ParsingError::ExpectedCloseBrace {
                span: self.cursor.current_span().to_source_span(),
            })
        }
    }

    fn is_constructor_lookahead(&self) -> bool {
        if !self.cursor.check(TokenKind::Identifier)
            || !self.cursor.check_next(TokenKind::PunctuationOpenParen)
        {
            return false;
        }

        let mut scan = self.cursor.position + 2;
        let mut depth = 1;
        while scan < self.cursor.tokens.len() && depth > 0 {
            match self.cursor.tokens[scan].kind {
                TokenKind::PunctuationOpenParen => depth += 1,
                TokenKind::PunctuationCloseParen => depth -= 1,
                _ => {}
            }
            scan += 1;
        }

        if depth == 0 && scan < self.cursor.tokens.len() {
            let next_kind = self.cursor.tokens[scan].kind;
            return next_kind == TokenKind::PunctuationOpenBrace
                || next_kind == TokenKind::PunctuationColon;
        }

        false
    }

    fn is_declaration_start(&mut self) -> bool {
        let peek = self.cursor.peek_kind();
        if peek == TokenKind::PunctuationOpenParen {
            let saved_pos = self.cursor.position;
            let saved_errors = self.errors.len();
            let is_decl = match self.parse_type_annotation() {
                Ok(_) => self.cursor.check(TokenKind::Identifier),
                Err(_) => false,
            };
            self.cursor.position = saved_pos;
            self.errors.truncate(saved_errors);
            if is_decl {
                return true;
            }
        }
        if peek.is_primitive() && peek != TokenKind::PrimitiveTuple {
            return true;
        }
        if peek == TokenKind::PrimitiveTuple
            && (self.cursor.check_next(TokenKind::OperatorLessThan)
                || self.cursor.check_next(TokenKind::Identifier))
        {
            return true;
        }
        if peek == TokenKind::Identifier {
            let saved_pos = self.cursor.position;
            let saved_errors = self.errors.len();

            let mut had_dot_type_error = false;
            let is_decl = match self.parse_type_annotation() {
                Ok(_) => self.cursor.check(TokenKind::Identifier),
                Err(ParsingError::DotNotAllowedInTypePath { .. }) => {
                    // Continue scanning past dotted/scoped path to see if followed by generic args or variable identifier
                    while self.cursor.check(TokenKind::Identifier)
                        || self.cursor.check(TokenKind::PunctuationDot)
                        || self.cursor.check(TokenKind::PunctuationDoubleColon)
                    {
                        self.cursor.advance();
                    }
                    if self.cursor.check(TokenKind::OperatorLessThan) {
                        let mut angle_depth = 0;
                        while !self.cursor.is_at_end() {
                            if self.cursor.check(TokenKind::OperatorLessThan) {
                                angle_depth += 1;
                                self.cursor.advance();
                            } else if self.cursor.check(TokenKind::OperatorGreaterThan) {
                                angle_depth -= 1;
                                self.cursor.advance();
                                if angle_depth == 0 {
                                    break;
                                }
                            } else if self.cursor.check(TokenKind::PunctuationSemicolon) {
                                break;
                            } else {
                                self.cursor.advance();
                            }
                        }
                    }
                    if self.cursor.check(TokenKind::Identifier) {
                        had_dot_type_error = true;
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };

            self.cursor.position = saved_pos;
            self.errors.truncate(saved_errors);
            if had_dot_type_error {
                return true;
            }
            return is_decl;
        }
        false
    }

    fn is_type_start(&self, kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::PrimitiveVoid
                | TokenKind::PrimitiveBool
                | TokenKind::PrimitiveInt8
                | TokenKind::PrimitiveInt16
                | TokenKind::PrimitiveInt32
                | TokenKind::PrimitiveInt64
                | TokenKind::PrimitiveUint8
                | TokenKind::PrimitiveUint16
                | TokenKind::PrimitiveUint32
                | TokenKind::PrimitiveUint64
                | TokenKind::PrimitiveFloat32
                | TokenKind::PrimitiveFloat64
                | TokenKind::PrimitiveChar
                | TokenKind::PrimitiveTuple
                | TokenKind::Identifier
        )
    }

    fn is_expression_start(&self, kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Identifier
                | TokenKind::Number
                | TokenKind::String
                | TokenKind::Char
                | TokenKind::PunctuationOpenParen
                | TokenKind::PunctuationOpenBracket
                | TokenKind::PunctuationOpenBrace
                | TokenKind::OperatorLogicalNot
                | TokenKind::OperatorBitwiseNot
                | TokenKind::OperatorMinus
                | TokenKind::OperatorPlus
                | TokenKind::OperatorIncrement
                | TokenKind::OperatorDecrement
                | TokenKind::KeywordNew
                | TokenKind::KeywordDefault
                | TokenKind::KeywordSizeof
                | TokenKind::KeywordSuper
        )
    }

    fn is_non_const_modifier(&self, kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::KeywordPublic
                | TokenKind::KeywordProtected
                | TokenKind::KeywordPrivate
                | TokenKind::KeywordInternal
                | TokenKind::KeywordStatic
                | TokenKind::KeywordInline
                | TokenKind::KeywordNative
                | TokenKind::KeywordVirtual
                | TokenKind::KeywordOverride
                | TokenKind::KeywordWeak
                | TokenKind::KeywordAbstract
        )
    }

    fn inside_function(&self) -> bool {
        self.scope_depth > 0
    }
}

enum TopLevelElement {
    Declaration(Declaration),
    Statement(Statement),
    Import(ImportStatement),
}
