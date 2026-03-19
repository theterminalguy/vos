//! Lexer for the vos script language.

use crate::token::{Token, TokenKind};

/// Lexer state.
pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Creates a new lexer from source code.
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenizes the entire source.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Gets the next token.
    fn next_token(&mut self) -> Result<Token, String> {
        // Skip whitespace (except newlines)
        self.skip_whitespace();

        // Check for EOF
        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof, ""));
        }

        let start_line = self.line;
        let start_column = self.column;
        let c = self.advance();

        // Newline
        if c == '\n' {
            return Ok(Token::new(TokenKind::Newline, "\n".to_string(), start_line, start_column));
        }

        // Comments
        if c == '/' && self.peek() == Some('/') {
            self.skip_line_comment();
            return self.next_token(); // Get next token after comment
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.number(c, start_line, start_column);
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            return self.identifier(c, start_line, start_column);
        }

        // Strings
        if c == '"' {
            return self.string(start_line, start_column);
        }

        // Operators and delimiters
        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::And
                } else {
                    return Err(format!("Unexpected character '&' at line {}, column {}", start_line, start_column));
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::Or
                } else {
                    TokenKind::Pipe
                }
            }
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            _ => {
                return Err(format!(
                    "Unexpected character '{}' at line {}, column {}",
                    c, start_line, start_column
                ));
            }
        };

        Ok(self.make_token(kind, &c.to_string()))
    }

    /// Parses a number (integer or float).
    fn number(&mut self, first: char, line: usize, column: usize) -> Result<Token, String> {
        let mut num_str = first.to_string();

        // Collect digits
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point
        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            num_str.push('.');
            self.advance();

            // Collect fractional digits
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            Ok(Token::new(
                TokenKind::Float(num_str.clone()),
                num_str,
                line,
                column,
            ))
        } else {
            // Integer
            let value = num_str
                .parse::<i32>()
                .map_err(|_| format!("Invalid integer '{}' at line {}, column {}", num_str, line, column))?;

            Ok(Token::new(TokenKind::Integer(value), num_str, line, column))
        }
    }

    /// Parses an identifier or keyword.
    fn identifier(&mut self, first: char, line: usize, column: usize) -> Result<Token, String> {
        let mut ident = first.to_string();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "struct" => TokenKind::Struct,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "int" => TokenKind::TypeInt,
            "float" => TokenKind::TypeFloat,
            "bool" => TokenKind::TypeBool,
            "string" => TokenKind::TypeString,
            "char" => TokenKind::TypeChar,
            _ => TokenKind::Identifier(ident.clone()),
        };

        Ok(Token::new(kind, ident, line, column))
    }

    /// Parses a string literal.
    fn string(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let mut value = String::new();

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance(); // Consume closing quote
                return Ok(Token::new(
                    TokenKind::String(value.clone()),
                    format!("\"{}\"", value),
                    line,
                    column,
                ));
            } else if c == '\\' {
                self.advance();
                if let Some(escaped) = self.peek() {
                    self.advance();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => {
                            return Err(format!(
                                "Invalid escape sequence '\\{}' at line {}, column {}",
                                escaped, self.line, self.column
                            ));
                        }
                    }
                } else {
                    return Err(format!("Unterminated string at line {}, column {}", line, column));
                }
            } else if c == '\n' {
                return Err(format!("Unterminated string at line {}, column {}", line, column));
            } else {
                value.push(c);
                self.advance();
            }
        }

        Err(format!("Unterminated string at line {}, column {}", line, column))
    }

    /// Skips whitespace (but not newlines).
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skips a line comment.
    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Creates a token with the current lexeme.
    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token::new(kind, lexeme.to_string(), self.line, self.column)
    }

    /// Advances to the next character.
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;

        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        c
    }

    /// Peeks at the current character without advancing.
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source[self.current])
        }
    }

    /// Peeks at the next character.
    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            Some(self.source[self.current + 1])
        }
    }

    /// Checks if we're at the end of the source.
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integers() {
        let mut lexer = Lexer::new("42 123 0");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // 3 integers + EOF
        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
        assert_eq!(tokens[1].kind, TokenKind::Integer(123));
        assert_eq!(tokens[2].kind, TokenKind::Integer(0));
    }

    #[test]
    fn test_floats() {
        let mut lexer = Lexer::new("3.14 0.5 100.0");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0].kind, TokenKind::Float(s) if s == "3.14"));
        assert!(matches!(&tokens[1].kind, TokenKind::Float(s) if s == "0.5"));
        assert!(matches!(&tokens[2].kind, TokenKind::Float(s) if s == "100.0"));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" "world" "hello\nworld""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello"));
        assert!(matches!(&tokens[1].kind, TokenKind::String(s) if s == "world"));
        assert!(matches!(&tokens[2].kind, TokenKind::String(s) if s == "hello\nworld"));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("let const fn if else while for in return");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Const);
        assert_eq!(tokens[2].kind, TokenKind::Fn);
        assert_eq!(tokens[3].kind, TokenKind::If);
        assert_eq!(tokens[4].kind, TokenKind::Else);
        assert_eq!(tokens[5].kind, TokenKind::While);
        assert_eq!(tokens[6].kind, TokenKind::For);
        assert_eq!(tokens[7].kind, TokenKind::In);
        assert_eq!(tokens[8].kind, TokenKind::Return);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("x foo bar_123 _test");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "foo"));
        assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "bar_123"));
        assert!(matches!(&tokens[3].kind, TokenKind::Identifier(s) if s == "_test"));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % = == != < <= > >= ! && || ->");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Percent);
        assert_eq!(tokens[5].kind, TokenKind::Equal);
        assert_eq!(tokens[6].kind, TokenKind::EqualEqual);
        assert_eq!(tokens[7].kind, TokenKind::BangEqual);
        assert_eq!(tokens[8].kind, TokenKind::Less);
        assert_eq!(tokens[9].kind, TokenKind::LessEqual);
        assert_eq!(tokens[10].kind, TokenKind::Greater);
        assert_eq!(tokens[11].kind, TokenKind::GreaterEqual);
        assert_eq!(tokens[12].kind, TokenKind::Bang);
        assert_eq!(tokens[13].kind, TokenKind::And);
        assert_eq!(tokens[14].kind, TokenKind::Or);
        assert_eq!(tokens[15].kind, TokenKind::Arrow);
    }

    #[test]
    fn test_delimiters() {
        let mut lexer = Lexer::new("( ) { } [ ] , : ; .");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::RightParen);
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[3].kind, TokenKind::RightBrace);
        assert_eq!(tokens[4].kind, TokenKind::LeftBracket);
        assert_eq!(tokens[5].kind, TokenKind::RightBracket);
        assert_eq!(tokens[6].kind, TokenKind::Comma);
        assert_eq!(tokens[7].kind, TokenKind::Colon);
        assert_eq!(tokens[8].kind, TokenKind::Semicolon);
        assert_eq!(tokens[9].kind, TokenKind::Dot);
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("let x = 42 // this is a comment\nlet y = 10");
        let tokens = lexer.tokenize().unwrap();

        // Should have: let, x, =, 42, newline, let, y, =, 10, EOF
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "x"));
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Integer(42));
        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[5].kind, TokenKind::Let);
    }

    #[test]
    fn test_simple_program() {
        let source = r#"
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        // Should successfully tokenize
        assert!(tokens.len() > 20); // Many tokens
        assert_eq!(tokens[1].kind, TokenKind::Fn); // First non-newline token
    }
}
