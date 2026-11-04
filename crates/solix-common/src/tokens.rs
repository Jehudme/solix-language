//! Token definitions, terminal grammar kinds, and keyword lookup table for Solix.

#![forbid(unsafe_code)]

use crate::span::Span;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::LazyLock;

/// Token classification category representing every terminal symbol in the Solix grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenKind {
    // ------------------------------------------------------------------------
    // Special & Unknown
    // ------------------------------------------------------------------------
    /// End-of-file sentinel indicating completion of token stream.
    #[default]
    Eof,
    /// Unrecognized character or malformed byte sequence.
    Unknown,

    // ------------------------------------------------------------------------
    // Literals and Identifiers
    // ------------------------------------------------------------------------
    /// User-defined identifier (variable, class, function name).
    Identifier,
    /// Numeric literal (decimal integer, floating point, hex, octal, binary).
    Number,
    /// Quoted string literal.
    String,
    /// Single-character literal enclosed in single quotes.
    Char,

    // ------------------------------------------------------------------------
    // Primitive Type Keywords
    // ------------------------------------------------------------------------
    /// Primitive `void` type keyword.
    PrimitiveVoid,
    /// Primitive `bool` boolean type keyword.
    PrimitiveBool,
    /// Primitive signed 8-bit integer type keyword (`int8`).
    PrimitiveInt8,
    /// Primitive signed 16-bit integer type keyword (`int16`).
    PrimitiveInt16,
    /// Primitive signed 32-bit integer type keyword (`int32`).
    PrimitiveInt32,
    /// Primitive signed 64-bit integer type keyword (`int64`).
    PrimitiveInt64,
    /// Primitive unsigned 8-bit integer type keyword (`uint8`).
    PrimitiveUint8,
    /// Primitive unsigned 16-bit integer type keyword (`uint16`).
    PrimitiveUint16,
    /// Primitive unsigned 32-bit integer type keyword (`uint32`).
    PrimitiveUint32,
    /// Primitive unsigned 64-bit integer type keyword (`uint64`).
    PrimitiveUint64,
    /// Primitive 32-bit floating point type keyword (`float32`).
    PrimitiveFloat32,
    /// Primitive 64-bit floating point type keyword (`float64`).
    PrimitiveFloat64,
    /// Primitive character type keyword (`char`).
    PrimitiveChar,
    /// Primitive composite tuple type keyword (`tuple`).
    PrimitiveTuple,

    // ------------------------------------------------------------------------
    // Control Flow & Declaration Keywords
    // ------------------------------------------------------------------------
    /// Conditional branching `if` keyword.
    KeywordIf,
    /// Alternative conditional branching `else` keyword.
    KeywordElse,
    /// Loop iteration `for` keyword.
    KeywordFor,
    /// Pre-condition loop `while` keyword.
    KeywordWhile,
    /// Post-condition loop `do` keyword.
    KeywordDo,
    /// Function exit and value return `return` keyword.
    KeywordReturn,
    /// Loop or switch termination `break` keyword.
    KeywordBreak,
    /// Loop iteration advance `continue` keyword.
    KeywordContinue,
    /// Multi-way branch selection `switch` keyword.
    KeywordSwitch,
    /// Branch pattern option `case` keyword.
    KeywordCase,
    /// Fallback branch selection `default` keyword.
    KeywordDefault,
    /// Immutable binding `const` keyword.
    KeywordConst,
    /// Object-oriented type definition `class` keyword.
    KeywordClass,
    /// Enumerated type definition `enum` keyword.
    KeywordEnum,
    /// Exception handling `try` block keyword.
    KeywordTry,
    /// Exception interception `catch` clause keyword.
    KeywordCatch,
    /// Guaranteed cleanup `finally` clause keyword.
    KeywordFinally,
    /// Exception raise `throw` keyword.
    KeywordThrow,
    /// Static lifetime member `static` keyword.
    KeywordStatic,
    /// Compiler optimization hint `inline` keyword.
    KeywordInline,
    /// Type or module renaming `alias` keyword.
    KeywordAlias,
    /// External module resolution `import` keyword.
    KeywordImport,
    /// Public visibility modifier `public` keyword.
    KeywordPublic,
    /// Foreign C-ABI binding `native` keyword.
    KeywordNative,
    /// Subclass visibility modifier `protected` keyword.
    KeywordProtected,
    /// Class-local visibility modifier `private` keyword.
    KeywordPrivate,
    /// Package-internal visibility modifier `internal` keyword.
    KeywordInternal,
    /// Compilation namespace declaration `package` keyword.
    KeywordPackage,
    /// Heap allocation and instantiation `new` keyword.
    KeywordNew,
    /// Custom operator overload declaration `operator` keyword.
    KeywordOperator,
    /// Class inheritance `extends` keyword.
    KeywordExtends,
    /// Parent class reference `super` keyword.
    KeywordSuper,
    /// Dynamic dispatchable method `virtual` keyword.
    KeywordVirtual,
    /// Inherited method replacement `override` keyword.
    KeywordOverride,
    /// Non-owning ARC reference `weak` keyword.
    KeywordWeak,
    /// Abstract base class or method `abstract` keyword.
    KeywordAbstract,
    /// Contract declaration `interface` keyword.
    KeywordInterface,
    /// Interface compliance declaration `implements` keyword.
    KeywordImplements,
    /// Runtime type introspection `instanceof` keyword.
    KeywordInstanceof,
    /// Memory byte size introspection `sizeof` keyword.
    KeywordSizeof,
    /// Invariant verification `assert` keyword.
    KeywordAssert,
    /// Process termination `exit` keyword.
    KeywordExit,
    /// External symbol declaration `extern` keyword.
    KeywordExtern,

    // ------------------------------------------------------------------------
    // Operators
    // ------------------------------------------------------------------------
    /// Assignment operator `=`.
    OperatorAssign,
    /// Ternary or optional question mark `?`.
    OperatorQuestion,
    /// Addition operator `+`.
    OperatorPlus,
    /// Subtraction or negation operator `-`.
    OperatorMinus,
    /// Multiplication or pointer dereference operator `*`.
    OperatorMultiply,
    /// Division operator `/`.
    OperatorDivide,
    /// Modulo remainder operator `%`.
    OperatorModulo,
    /// Addition assignment operator `+=`.
    OperatorPlusAssign,
    /// Subtraction assignment operator `-=`.
    OperatorMinusAssign,
    /// Multiplication assignment operator `*=`.
    OperatorMultiplyAssign,
    /// Division assignment operator `/=`.
    OperatorDivideAssign,
    /// Modulo assignment operator `%=`.
    OperatorModuloAssign,
    /// Postfix/prefix increment operator `++`.
    OperatorIncrement,
    /// Postfix/prefix decrement operator `--`.
    OperatorDecrement,
    /// Equality comparison operator `==`.
    OperatorEqual,
    /// Inequality comparison operator `!=`.
    OperatorNotEqual,
    /// Less-than comparison operator `<`.
    OperatorLessThan,
    /// Greater-than comparison operator `>`.
    OperatorGreaterThan,
    /// Less-than or equal comparison operator `<=`.
    OperatorLessEqual,
    /// Greater-than or equal comparison operator `>=`.
    OperatorGreaterEqual,
    /// Short-circuiting logical AND operator `&&`.
    OperatorLogicalAnd,
    /// Short-circuiting logical OR operator `||`.
    OperatorLogicalOr,
    /// Logical negation operator `!`.
    OperatorLogicalNot,
    /// Bitwise NOT operator `~`.
    OperatorBitwiseNot,
    /// Bitwise OR operator `|`.
    OperatorBitwiseOr,
    /// Bitwise XOR operator `^`.
    OperatorBitwiseXor,
    /// Bitwise shift left operator `<<`.
    OperatorShiftLeft,
    /// Bitwise shift right operator `>>`.
    OperatorShiftRight,
    /// Lambda expression fat arrow `=>`.
    OperatorFatArrow,

    // ------------------------------------------------------------------------
    // Punctuation & Delimiters
    // ------------------------------------------------------------------------
    /// Statement termination semicolon `;`.
    PunctuationSemicolon,
    /// Element separator comma `,`.
    PunctuationComma,
    /// Member access dot `.`.
    PunctuationDot,
    /// Type annotation or case label colon `:`.
    PunctuationColon,
    /// Scope resolution operator `::`.
    PunctuationDoubleColon,
    /// Opening parenthesis `(`.
    PunctuationOpenParen,
    /// Closing parenthesis `)`.
    PunctuationCloseParen,
    /// Opening curly brace `{`.
    PunctuationOpenBrace,
    /// Closing curly brace `}`.
    PunctuationCloseBrace,
    /// Opening square bracket `[`.
    PunctuationOpenBracket,
    /// Closing square bracket `]`.
    PunctuationCloseBracket,
    /// Optimized array dimension brackets `[]`.
    PunctuationArrayBrackets,
    /// Reference or bitwise AND ampersand `&`.
    PunctuationAmpersand,
    /// Variadic parameter ellipsis `...`.
    PunctuationEllipsis,

    // ------------------------------------------------------------------------
    // Comments
    // ------------------------------------------------------------------------
    /// Single-line comment `// ...`.
    CommentSingleLine,
    /// Multi-line block comment `/* ... */`.
    CommentMultiLine,
}

impl TokenKind {
    /// Returns whether this token kind represents a language keyword or primitive type.
    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        self.is_primitive()
            || matches!(
                self,
                Self::KeywordIf
                    | Self::KeywordElse
                    | Self::KeywordFor
                    | Self::KeywordWhile
                    | Self::KeywordDo
                    | Self::KeywordReturn
                    | Self::KeywordBreak
                    | Self::KeywordContinue
                    | Self::KeywordSwitch
                    | Self::KeywordCase
                    | Self::KeywordDefault
                    | Self::KeywordConst
                    | Self::KeywordClass
                    | Self::KeywordEnum
                    | Self::KeywordTry
                    | Self::KeywordCatch
                    | Self::KeywordFinally
                    | Self::KeywordThrow
                    | Self::KeywordStatic
                    | Self::KeywordInline
                    | Self::KeywordAlias
                    | Self::KeywordImport
                    | Self::KeywordPublic
                    | Self::KeywordNative
                    | Self::KeywordProtected
                    | Self::KeywordPrivate
                    | Self::KeywordInternal
                    | Self::KeywordPackage
                    | Self::KeywordNew
                    | Self::KeywordOperator
                    | Self::KeywordExtends
                    | Self::KeywordSuper
                    | Self::KeywordVirtual
                    | Self::KeywordOverride
                    | Self::KeywordWeak
                    | Self::KeywordAbstract
                    | Self::KeywordInterface
                    | Self::KeywordImplements
                    | Self::KeywordInstanceof
                    | Self::KeywordSizeof
                    | Self::KeywordAssert
                    | Self::KeywordExit
                    | Self::KeywordExtern
            )
    }

    /// Returns whether this token kind represents a built-in primitive type keyword.
    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::PrimitiveVoid
                | Self::PrimitiveBool
                | Self::PrimitiveInt8
                | Self::PrimitiveInt16
                | Self::PrimitiveInt32
                | Self::PrimitiveInt64
                | Self::PrimitiveUint8
                | Self::PrimitiveUint16
                | Self::PrimitiveUint32
                | Self::PrimitiveUint64
                | Self::PrimitiveFloat32
                | Self::PrimitiveFloat64
                | Self::PrimitiveChar
                | Self::PrimitiveTuple
        )
    }

    /// Returns whether this token kind represents an operator.
    #[must_use]
    pub const fn is_operator(&self) -> bool {
        matches!(
            self,
            Self::OperatorAssign
                | Self::OperatorQuestion
                | Self::OperatorPlus
                | Self::OperatorMinus
                | Self::OperatorMultiply
                | Self::OperatorDivide
                | Self::OperatorModulo
                | Self::OperatorPlusAssign
                | Self::OperatorMinusAssign
                | Self::OperatorMultiplyAssign
                | Self::OperatorDivideAssign
                | Self::OperatorModuloAssign
                | Self::OperatorIncrement
                | Self::OperatorDecrement
                | Self::OperatorEqual
                | Self::OperatorNotEqual
                | Self::OperatorLessThan
                | Self::OperatorGreaterThan
                | Self::OperatorLessEqual
                | Self::OperatorGreaterEqual
                | Self::OperatorLogicalAnd
                | Self::OperatorLogicalOr
                | Self::OperatorLogicalNot
                | Self::OperatorBitwiseNot
                | Self::OperatorBitwiseOr
                | Self::OperatorBitwiseXor
                | Self::OperatorShiftLeft
                | Self::OperatorShiftRight
                | Self::OperatorFatArrow
        )
    }

    /// Returns whether this token kind represents punctuation or a delimiter.
    #[must_use]
    pub const fn is_punctuation(&self) -> bool {
        matches!(
            self,
            Self::PunctuationSemicolon
                | Self::PunctuationComma
                | Self::PunctuationDot
                | Self::PunctuationColon
                | Self::PunctuationDoubleColon
                | Self::PunctuationOpenParen
                | Self::PunctuationCloseParen
                | Self::PunctuationOpenBrace
                | Self::PunctuationCloseBrace
                | Self::PunctuationOpenBracket
                | Self::PunctuationCloseBracket
                | Self::PunctuationArrayBrackets
                | Self::PunctuationAmpersand
                | Self::PunctuationEllipsis
        )
    }

    /// Returns whether this token kind represents a comment.
    #[must_use]
    pub const fn is_comment(&self) -> bool {
        matches!(self, Self::CommentSingleLine | Self::CommentMultiLine)
    }

    /// Returns the canonical textual representation of keywords, operators, and punctuation.
    #[must_use]
    pub const fn as_static_str(&self) -> &'static str {
        match self {
            Self::Eof => "<EOF>",
            Self::Unknown => "<unknown>",
            Self::Identifier => "<identifier>",
            Self::Number => "<number>",
            Self::String => "<string>",
            Self::Char => "<char>",

            Self::PrimitiveVoid => "void",
            Self::PrimitiveBool => "bool",
            Self::PrimitiveInt8 => "int8",
            Self::PrimitiveInt16 => "int16",
            Self::PrimitiveInt32 => "int32",
            Self::PrimitiveInt64 => "int64",
            Self::PrimitiveUint8 => "uint8",
            Self::PrimitiveUint16 => "uint16",
            Self::PrimitiveUint32 => "uint32",
            Self::PrimitiveUint64 => "uint64",
            Self::PrimitiveFloat32 => "float32",
            Self::PrimitiveFloat64 => "float64",
            Self::PrimitiveChar => "char",
            Self::PrimitiveTuple => "tuple",

            Self::KeywordIf => "if",
            Self::KeywordElse => "else",
            Self::KeywordFor => "for",
            Self::KeywordWhile => "while",
            Self::KeywordDo => "do",
            Self::KeywordReturn => "return",
            Self::KeywordBreak => "break",
            Self::KeywordContinue => "continue",
            Self::KeywordSwitch => "switch",
            Self::KeywordCase => "case",
            Self::KeywordDefault => "default",
            Self::KeywordConst => "const",
            Self::KeywordClass => "class",
            Self::KeywordEnum => "enum",
            Self::KeywordTry => "try",
            Self::KeywordCatch => "catch",
            Self::KeywordFinally => "finally",
            Self::KeywordThrow => "throw",
            Self::KeywordStatic => "static",
            Self::KeywordInline => "inline",
            Self::KeywordAlias => "alias",
            Self::KeywordImport => "import",
            Self::KeywordPublic => "public",
            Self::KeywordNative => "native",
            Self::KeywordProtected => "protected",
            Self::KeywordPrivate => "private",
            Self::KeywordInternal => "internal",
            Self::KeywordPackage => "package",
            Self::KeywordNew => "new",
            Self::KeywordOperator => "operator",
            Self::KeywordExtends => "extends",
            Self::KeywordSuper => "super",
            Self::KeywordVirtual => "virtual",
            Self::KeywordOverride => "override",
            Self::KeywordWeak => "weak",
            Self::KeywordAbstract => "abstract",
            Self::KeywordInterface => "interface",
            Self::KeywordImplements => "implements",
            Self::KeywordInstanceof => "instanceof",
            Self::KeywordSizeof => "sizeof",
            Self::KeywordAssert => "assert",
            Self::KeywordExit => "exit",
            Self::KeywordExtern => "extern",

            Self::OperatorAssign => "=",
            Self::OperatorQuestion => "?",
            Self::OperatorPlus => "+",
            Self::OperatorMinus => "-",
            Self::OperatorMultiply => "*",
            Self::OperatorDivide => "/",
            Self::OperatorModulo => "%",
            Self::OperatorPlusAssign => "+=",
            Self::OperatorMinusAssign => "-=",
            Self::OperatorMultiplyAssign => "*=",
            Self::OperatorDivideAssign => "/=",
            Self::OperatorModuloAssign => "%=",
            Self::OperatorIncrement => "++",
            Self::OperatorDecrement => "--",
            Self::OperatorEqual => "==",
            Self::OperatorNotEqual => "!=",
            Self::OperatorLessThan => "<",
            Self::OperatorGreaterThan => ">",
            Self::OperatorLessEqual => "<=",
            Self::OperatorGreaterEqual => ">=",
            Self::OperatorLogicalAnd => "&&",
            Self::OperatorLogicalOr => "||",
            Self::OperatorLogicalNot => "!",
            Self::OperatorBitwiseNot => "~",
            Self::OperatorBitwiseOr => "|",
            Self::OperatorBitwiseXor => "^",
            Self::OperatorShiftLeft => "<<",
            Self::OperatorShiftRight => ">>",
            Self::OperatorFatArrow => "=>",

            Self::PunctuationSemicolon => ";",
            Self::PunctuationComma => ",",
            Self::PunctuationDot => ".",
            Self::PunctuationColon => ":",
            Self::PunctuationDoubleColon => "::",
            Self::PunctuationOpenParen => "(",
            Self::PunctuationCloseParen => ")",
            Self::PunctuationOpenBrace => "{",
            Self::PunctuationCloseBrace => "}",
            Self::PunctuationOpenBracket => "[",
            Self::PunctuationCloseBracket => "]",
            Self::PunctuationArrayBrackets => "[]",
            Self::PunctuationAmpersand => "&",
            Self::PunctuationEllipsis => "...",

            Self::CommentSingleLine => "//",
            Self::CommentMultiLine => "/* */",
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.as_static_str())
    }
}

// ============================================================================
// Keyword Hash Map & Lookup
// ============================================================================

/// Global immutable lookup table mapping every keyword string literal to its [`TokenKind`].
pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenKind>> = LazyLock::new(|| {
    let mut keyword_mapping = HashMap::with_capacity(64);

    // Primitive Types (14 keywords)
    keyword_mapping.insert("void", TokenKind::PrimitiveVoid);
    keyword_mapping.insert("bool", TokenKind::PrimitiveBool);
    keyword_mapping.insert("int8", TokenKind::PrimitiveInt8);
    keyword_mapping.insert("int16", TokenKind::PrimitiveInt16);
    keyword_mapping.insert("int32", TokenKind::PrimitiveInt32);
    keyword_mapping.insert("int64", TokenKind::PrimitiveInt64);
    keyword_mapping.insert("uint8", TokenKind::PrimitiveUint8);
    keyword_mapping.insert("uint16", TokenKind::PrimitiveUint16);
    keyword_mapping.insert("uint32", TokenKind::PrimitiveUint32);
    keyword_mapping.insert("uint64", TokenKind::PrimitiveUint64);
    keyword_mapping.insert("float32", TokenKind::PrimitiveFloat32);
    keyword_mapping.insert("float64", TokenKind::PrimitiveFloat64);
    keyword_mapping.insert("char", TokenKind::PrimitiveChar);
    keyword_mapping.insert("tuple", TokenKind::PrimitiveTuple);

    // Language Keywords (42 keywords)
    keyword_mapping.insert("if", TokenKind::KeywordIf);
    keyword_mapping.insert("else", TokenKind::KeywordElse);
    keyword_mapping.insert("for", TokenKind::KeywordFor);
    keyword_mapping.insert("while", TokenKind::KeywordWhile);
    keyword_mapping.insert("do", TokenKind::KeywordDo);
    keyword_mapping.insert("return", TokenKind::KeywordReturn);
    keyword_mapping.insert("break", TokenKind::KeywordBreak);
    keyword_mapping.insert("continue", TokenKind::KeywordContinue);
    keyword_mapping.insert("switch", TokenKind::KeywordSwitch);
    keyword_mapping.insert("case", TokenKind::KeywordCase);
    keyword_mapping.insert("default", TokenKind::KeywordDefault);
    keyword_mapping.insert("try", TokenKind::KeywordTry);
    keyword_mapping.insert("catch", TokenKind::KeywordCatch);
    keyword_mapping.insert("finally", TokenKind::KeywordFinally);
    keyword_mapping.insert("throw", TokenKind::KeywordThrow);
    keyword_mapping.insert("const", TokenKind::KeywordConst);
    keyword_mapping.insert("class", TokenKind::KeywordClass);
    keyword_mapping.insert("enum", TokenKind::KeywordEnum);
    keyword_mapping.insert("static", TokenKind::KeywordStatic);
    keyword_mapping.insert("inline", TokenKind::KeywordInline);
    keyword_mapping.insert("alias", TokenKind::KeywordAlias);
    keyword_mapping.insert("import", TokenKind::KeywordImport);
    keyword_mapping.insert("public", TokenKind::KeywordPublic);
    keyword_mapping.insert("native", TokenKind::KeywordNative);
    keyword_mapping.insert("protected", TokenKind::KeywordProtected);
    keyword_mapping.insert("private", TokenKind::KeywordPrivate);
    keyword_mapping.insert("internal", TokenKind::KeywordInternal);
    keyword_mapping.insert("package", TokenKind::KeywordPackage);
    keyword_mapping.insert("new", TokenKind::KeywordNew);
    keyword_mapping.insert("operator", TokenKind::KeywordOperator);
    keyword_mapping.insert("extends", TokenKind::KeywordExtends);
    keyword_mapping.insert("super", TokenKind::KeywordSuper);
    keyword_mapping.insert("virtual", TokenKind::KeywordVirtual);
    keyword_mapping.insert("override", TokenKind::KeywordOverride);
    keyword_mapping.insert("weak", TokenKind::KeywordWeak);
    keyword_mapping.insert("abstract", TokenKind::KeywordAbstract);
    keyword_mapping.insert("interface", TokenKind::KeywordInterface);
    keyword_mapping.insert("implements", TokenKind::KeywordImplements);
    keyword_mapping.insert("instanceof", TokenKind::KeywordInstanceof);
    keyword_mapping.insert("sizeof", TokenKind::KeywordSizeof);
    keyword_mapping.insert("assert", TokenKind::KeywordAssert);
    keyword_mapping.insert("exit", TokenKind::KeywordExit);
    keyword_mapping.insert("extern", TokenKind::KeywordExtern);

    keyword_mapping
});

/// Looks up an identifier candidate to determine if it is a reserved Solix keyword.
#[must_use]
pub fn lookup_keyword(identifier_candidate: &str) -> Option<TokenKind> {
    KEYWORDS.get(identifier_candidate).copied()
}

// ============================================================================
// Token
// ============================================================================

/// Atomic lexical unit emitted by the lexer and consumed by the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Token {
    /// Categorical kind of the token.
    pub kind: TokenKind,
    /// Exact byte range coordinates and file origin in the source map.
    pub span: Span,
}

impl Token {
    /// Constructs a new token from a categorical kind and origin span.
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

// ============================================================================
// Unit Tests (Mandatory at bottom of module per GL-0011)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_span_creation_and_properties() {
        let span = Span::new(1, 10, 25);
        assert_eq!(span.id, 1);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 25);
        assert_eq!(span.length(), 15);
        assert!(!span.is_empty());

        let source_span = span.to_source_span();
        assert_eq!(source_span.offset(), 10);
        assert_eq!(source_span.len(), 15);
    }

    #[test]
    fn test_positive_token_instantiation() {
        let span = Span::new(0, 0, 5);
        let token = Token::new(TokenKind::KeywordClass, span);

        assert_eq!(token.kind, TokenKind::KeywordClass);
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 5);
        assert!(token.kind.is_keyword());
        assert!(!token.kind.is_primitive());
    }

    #[test]
    fn test_positive_all_keywords_registered_in_hashmap() {
        assert_eq!(KEYWORDS.len(), 57);

        // Spot check primitive types
        assert_eq!(lookup_keyword("void"), Some(TokenKind::PrimitiveVoid));
        assert_eq!(lookup_keyword("bool"), Some(TokenKind::PrimitiveBool));
        assert_eq!(lookup_keyword("int32"), Some(TokenKind::PrimitiveInt32));
        assert_eq!(lookup_keyword("float64"), Some(TokenKind::PrimitiveFloat64));
        assert_eq!(lookup_keyword("char"), Some(TokenKind::PrimitiveChar));
        assert_eq!(lookup_keyword("tuple"), Some(TokenKind::PrimitiveTuple));

        // Spot check language control flow keywords
        assert_eq!(lookup_keyword("if"), Some(TokenKind::KeywordIf));
        assert_eq!(lookup_keyword("else"), Some(TokenKind::KeywordElse));
        assert_eq!(lookup_keyword("while"), Some(TokenKind::KeywordWhile));
        assert_eq!(lookup_keyword("class"), Some(TokenKind::KeywordClass));
        assert_eq!(lookup_keyword("extends"), Some(TokenKind::KeywordExtends));
        assert_eq!(
            lookup_keyword("interface"),
            Some(TokenKind::KeywordInterface)
        );
        assert_eq!(
            lookup_keyword("instanceof"),
            Some(TokenKind::KeywordInstanceof)
        );
        assert_eq!(lookup_keyword("assert"), Some(TokenKind::KeywordAssert));
        assert_eq!(lookup_keyword("exit"), Some(TokenKind::KeywordExit));
        assert_eq!(lookup_keyword("extern"), Some(TokenKind::KeywordExtern));
    }

    #[test]
    fn test_positive_token_kind_queries() {
        assert!(TokenKind::PrimitiveInt32.is_primitive());
        assert!(TokenKind::PrimitiveInt32.is_keyword());
        assert!(!TokenKind::PrimitiveInt32.is_operator());

        assert!(TokenKind::OperatorPlus.is_operator());
        assert!(TokenKind::OperatorBitwiseOr.is_operator());
        assert!(TokenKind::OperatorBitwiseXor.is_operator());
        assert!(TokenKind::OperatorShiftLeft.is_operator());
        assert!(TokenKind::OperatorShiftRight.is_operator());
        assert!(!TokenKind::OperatorPlus.is_keyword());

        assert!(TokenKind::PunctuationSemicolon.is_punctuation());
        assert!(!TokenKind::PunctuationSemicolon.is_operator());

        assert!(TokenKind::CommentSingleLine.is_comment());
        assert!(TokenKind::CommentMultiLine.is_comment());
    }

    #[test]
    fn test_positive_token_display_formatting() {
        assert_eq!(format!("{}", TokenKind::OperatorFatArrow), "=>");
        assert_eq!(format!("{}", TokenKind::PunctuationSemicolon), ";");
        assert_eq!(format!("{}", TokenKind::KeywordReturn), "return");
        assert_eq!(format!("{}", TokenKind::PrimitiveInt32), "int32");
    }

    #[test]
    fn test_negative_lookup_unregistered_identifier_returns_none() {
        assert_eq!(lookup_keyword("not_a_keyword"), None);
        assert_eq!(lookup_keyword("variable_name"), None);
        assert_eq!(lookup_keyword("IF"), None);
        assert_eq!(lookup_keyword("Class"), None);
    }

    #[test]
    fn test_negative_empty_span_detected() {
        let empty_span = Span::new(0, 42, 42);
        assert_eq!(empty_span.length(), 0);
        assert!(empty_span.is_empty());

        let inverted_span = Span::new(0, 50, 40);
        assert_eq!(inverted_span.length(), 0);
        assert!(inverted_span.is_empty());
    }
}
