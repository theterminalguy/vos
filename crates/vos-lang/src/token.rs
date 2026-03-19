//! Token types for the vos script language.

use std::fmt;

/// A token in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

/// Token kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    Integer(i32),
    Float(String), // Store as string to avoid float equality issues
    String(String),
    True,
    False,

    // Identifiers and keywords
    Identifier(String),
    Let,
    Const,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Struct,

    // Types
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,
    TypeChar,

    // Operators
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    Equal,     // =
    EqualEqual, // ==
    BangEqual, // !=
    Less,      // <
    LessEqual, // <=
    Greater,   // >
    GreaterEqual, // >=
    Bang,      // !
    And,       // &&
    Or,        // ||
    Arrow,     // ->
    Pipe,      // |

    // Delimiters
    LeftParen,   // (
    RightParen,  // )
    LeftBrace,   // {
    RightBrace,  // }
    LeftBracket, // [
    RightBracket, // ]
    Comma,       // ,
    Colon,       // :
    Semicolon,   // ;
    Dot,         // .

    // Special
    Newline,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer(n) => write!(f, "INTEGER({})", n),
            TokenKind::Float(s) => write!(f, "FLOAT({})", s),
            TokenKind::String(s) => write!(f, "STRING(\"{}\")", s),
            TokenKind::Identifier(s) => write!(f, "IDENT({})", s),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::TypeInt => write!(f, "int"),
            TokenKind::TypeFloat => write!(f, "float"),
            TokenKind::TypeBool => write!(f, "bool"),
            TokenKind::TypeString => write!(f, "string"),
            TokenKind::TypeChar => write!(f, "char"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Newline => write!(f, "NEWLINE"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}
