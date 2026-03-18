//! Lexical analysis for assembly source code.

use std::fmt;

/// Token types in assembly language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Instruction mnemonic (ADD, SUB, etc.)
    Mnemonic(String),
    /// Register (R0-R15)
    Register(u8),
    /// Integer literal
    Number(i32),
    /// Label identifier
    Label(String),
    /// Label definition (ends with :)
    LabelDef(String),
    /// Comma separator
    Comma,
    /// Newline
    Newline,
    /// End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Mnemonic(s) => write!(f, "MNEMONIC({})", s),
            Token::Register(r) => write!(f, "R{}", r),
            Token::Number(n) => write!(f, "{}", n),
            Token::Label(s) => write!(f, "LABEL({})", s),
            Token::LabelDef(s) => write!(f, "{}:", s),
            Token::Comma => write!(f, ","),
            Token::Newline => write!(f, "\\n"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Lexer for assembly source code.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    /// Creates a new lexer from source code.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    /// Returns the next token.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_except_newline();

        if self.position >= self.input.len() {
            return Token::Eof;
        }

        let ch = self.current_char();

        match ch {
            '\n' => {
                self.advance();
                Token::Newline
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            ';' => {
                self.skip_comment();
                self.next_token()
            }
            'R' | 'r' if self.peek_char().is_some_and(|c| c.is_ascii_digit()) => {
                self.read_register()
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
            ch if ch.is_ascii_digit() || ch == '-' => self.read_number(),
            _ => {
                self.advance();
                self.next_token() // Skip unknown characters
            }
        }
    }

    /// Tokenizes entire input.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn peek_char(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace_except_newline(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // Skip until newline or EOF
        while self.position < self.input.len() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn read_register(&mut self) -> Token {
        self.advance(); // Skip 'R' or 'r'

        let mut num_str = String::new();
        while self.position < self.input.len() && self.current_char().is_ascii_digit() {
            num_str.push(self.current_char());
            self.advance();
        }

        let reg_num = num_str.parse::<u8>().unwrap_or(0);
        Token::Register(reg_num)
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a label definition
        if self.position < self.input.len() && self.current_char() == ':' {
            self.advance(); // Skip ':'
            return Token::LabelDef(ident);
        }

        // Check if it's an instruction mnemonic (uppercase common)
        if ident.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            Token::Mnemonic(ident)
        } else {
            Token::Label(ident)
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();

        // Handle negative sign
        if self.current_char() == '-' {
            num_str.push('-');
            self.advance();
        }

        // Handle hex numbers (0x prefix)
        if self.position < self.input.len()
            && self.current_char() == '0'
            && self.peek_char() == Some('x')
        {
            self.advance(); // Skip '0'
            self.advance(); // Skip 'x'
            num_str.clear(); // Remove any '-'
            let is_negative = num_str.starts_with('-');

            while self.position < self.input.len() {
                let ch = self.current_char();
                if ch.is_ascii_hexdigit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }

            let value = i32::from_str_radix(&num_str, 16).unwrap_or(0);
            return Token::Number(if is_negative { -value } else { value });
        }

        // Decimal number
        while self.position < self.input.len() && self.current_char().is_ascii_digit() {
            num_str.push(self.current_char());
            self.advance();
        }

        Token::Number(num_str.parse::<i32>().unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_registers() {
        let mut lexer = Lexer::new("R0 R1 R15");
        assert_eq!(lexer.next_token(), Token::Register(0));
        assert_eq!(lexer.next_token(), Token::Register(1));
        assert_eq!(lexer.next_token(), Token::Register(15));
    }

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("42 -10 0xFF");
        assert_eq!(lexer.next_token(), Token::Number(42));
        assert_eq!(lexer.next_token(), Token::Number(-10));
        assert_eq!(lexer.next_token(), Token::Number(255));
    }

    #[test]
    fn test_tokenize_mnemonic() {
        let mut lexer = Lexer::new("ADD SUB ADDI");
        assert_eq!(lexer.next_token(), Token::Mnemonic("ADD".to_string()));
        assert_eq!(lexer.next_token(), Token::Mnemonic("SUB".to_string()));
        assert_eq!(lexer.next_token(), Token::Mnemonic("ADDI".to_string()));
    }

    #[test]
    fn test_tokenize_label() {
        let mut lexer = Lexer::new("start: loop");
        assert_eq!(lexer.next_token(), Token::LabelDef("start".to_string()));
        assert_eq!(lexer.next_token(), Token::Label("loop".to_string()));
    }

    #[test]
    fn test_tokenize_instruction() {
        let mut lexer = Lexer::new("ADD R1, R2, R3");
        assert_eq!(lexer.next_token(), Token::Mnemonic("ADD".to_string()));
        assert_eq!(lexer.next_token(), Token::Register(1));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Register(2));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Register(3));
    }

    #[test]
    fn test_skip_comments() {
        let mut lexer = Lexer::new("; This is a comment\nADD R1, R2, R3");
        assert_eq!(lexer.next_token(), Token::Newline);
        assert_eq!(lexer.next_token(), Token::Mnemonic("ADD".to_string()));
    }

    #[test]
    fn test_newlines() {
        let mut lexer = Lexer::new("ADD R1, R2, R3\nSUB R4, R5, R6");
        lexer.next_token(); // ADD
        lexer.next_token(); // R1
        lexer.next_token(); // ,
        lexer.next_token(); // R2
        lexer.next_token(); // ,
        lexer.next_token(); // R3
        assert_eq!(lexer.next_token(), Token::Newline);
        assert_eq!(lexer.next_token(), Token::Mnemonic("SUB".to_string()));
    }
}
