//! Parser for assembly instructions.

use crate::lexer::{Lexer, Token};
use std::collections::HashMap;
use vos_core::Result;

/// Parsed assembly instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmInstruction {
    /// R-type instruction
    RType {
        mnemonic: String,
        rd: u8,
        rs: u8,
        rt: u8,
    },
    /// I-type instruction
    IType {
        mnemonic: String,
        rt: u8,
        rs: u8,
        immediate: i32,
    },
    /// J-type instruction
    JType {
        mnemonic: String,
        target: String, // Label name
    },
    /// Zero-operand instruction (HALT, NOP, etc.)
    ZeroOp { mnemonic: String },
}

/// Parsed assembly program.
#[derive(Debug)]
pub struct Program {
    /// Instructions in order
    pub instructions: Vec<AsmInstruction>,
    /// Label to instruction index mapping
    pub labels: HashMap<String, usize>,
}

/// Parser for assembly code.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Creates a new parser from source code.
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        Self {
            tokens,
            position: 0,
        }
    }

    /// Parses the entire program.
    pub fn parse(&mut self) -> Result<Program> {
        let mut instructions = Vec::new();
        let mut labels = HashMap::new();

        while !self.is_at_end() {
            self.skip_newlines();

            if self.is_at_end() {
                break;
            }

            // Check for label definition
            if let Token::LabelDef(name) = self.current_token() {
                labels.insert(name.clone(), instructions.len());
                self.advance();
                self.skip_newlines();
                continue;
            }

            // Parse instruction
            if let Some(instr) = self.parse_instruction()? {
                instructions.push(instr);
            }

            self.skip_newlines();
        }

        Ok(Program {
            instructions,
            labels,
        })
    }

    fn parse_instruction(&mut self) -> Result<Option<AsmInstruction>> {
        if self.is_at_end() {
            return Ok(None);
        }

        let mnemonic = match self.current_token() {
            Token::Mnemonic(m) => m.to_uppercase(),
            _ => return Ok(None),
        };

        self.advance();

        // Determine instruction type based on mnemonic
        let instr = match mnemonic.as_str() {
            // Zero-operand instructions
            "HALT" | "NOP" | "SYSCALL" | "BREAK" => AsmInstruction::ZeroOp { mnemonic },

            // R-type instructions (rd, rs, rt)
            "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "OR" | "XOR" | "SLL" | "SRL" | "SRA"
            | "SLT" | "SGT" => self.parse_rtype(mnemonic)?,

            // I-type instructions (rt, rs, imm)
            "ADDI" | "SUBI" | "ANDI" | "ORI" | "XORI" | "LUI" | "LW" | "SW" | "LB" | "SB" => {
                self.parse_itype(mnemonic)?
            }

            // Branch instructions (rs, rt, offset/label)
            "BEQ" | "BNE" | "BLT" | "BGT" | "BLE" | "BGE" => self.parse_branch(mnemonic)?,

            // Jump instructions
            "J" | "JAL" => self.parse_jtype(mnemonic)?,

            _ => {
                return Err(format!("Unknown mnemonic: {}", mnemonic).into());
            }
        };

        Ok(Some(instr))
    }

    fn parse_rtype(&mut self, mnemonic: String) -> Result<AsmInstruction> {
        let rd = self.expect_register()?;
        self.expect_comma()?;
        let rs = self.expect_register()?;
        self.expect_comma()?;
        let rt = self.expect_register()?;

        Ok(AsmInstruction::RType {
            mnemonic,
            rd,
            rs,
            rt,
        })
    }

    fn parse_itype(&mut self, mnemonic: String) -> Result<AsmInstruction> {
        let rt = self.expect_register()?;
        self.expect_comma()?;
        let rs = self.expect_register()?;
        self.expect_comma()?;
        let immediate = self.expect_number()?;

        Ok(AsmInstruction::IType {
            mnemonic,
            rt,
            rs,
            immediate,
        })
    }

    fn parse_branch(&mut self, mnemonic: String) -> Result<AsmInstruction> {
        let rs = self.expect_register()?;
        self.expect_comma()?;
        let rt = self.expect_register()?;
        self.expect_comma()?;

        // Branch target can be a label or immediate offset
        let target = match self.current_token() {
            Token::Label(label) => {
                self.advance();
                label
            }
            Token::Number(offset) => {
                self.advance();
                offset.to_string()
            }
            _ => return Err("Expected label or offset for branch".into()),
        };

        Ok(AsmInstruction::IType {
            mnemonic,
            rt,
            rs,
            immediate: target.parse().unwrap_or(0),
        })
    }

    fn parse_jtype(&mut self, mnemonic: String) -> Result<AsmInstruction> {
        let target = match self.current_token() {
            Token::Label(label) => {
                self.advance();
                label
            }
            _ => return Err("Expected label for jump".into()),
        };

        Ok(AsmInstruction::JType { mnemonic, target })
    }

    fn expect_register(&mut self) -> Result<u8> {
        match self.current_token() {
            Token::Register(r) => {
                self.advance();
                Ok(r)
            }
            _ => Err(format!("Expected register, got {:?}", self.current_token()).into()),
        }
    }

    fn expect_number(&mut self) -> Result<i32> {
        match self.current_token() {
            Token::Number(n) => {
                self.advance();
                Ok(n)
            }
            _ => Err(format!("Expected number, got {:?}", self.current_token()).into()),
        }
    }

    fn expect_comma(&mut self) -> Result<()> {
        match self.current_token() {
            Token::Comma => {
                self.advance();
                Ok(())
            }
            _ => Err(format!("Expected comma, got {:?}", self.current_token()).into()),
        }
    }

    fn current_token(&self) -> Token {
        if self.position < self.tokens.len() {
            self.tokens[self.position].clone()
        } else {
            Token::Eof
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn skip_newlines(&mut self) {
        while self.current_token() == Token::Newline {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.current_token() == Token::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rtype() {
        let mut parser = Parser::new("ADD R1, R2, R3");
        let program = parser.parse().unwrap();

        assert_eq!(program.instructions.len(), 1);
        match &program.instructions[0] {
            AsmInstruction::RType {
                mnemonic,
                rd,
                rs,
                rt,
            } => {
                assert_eq!(mnemonic, "ADD");
                assert_eq!(*rd, 1);
                assert_eq!(*rs, 2);
                assert_eq!(*rt, 3);
            }
            _ => panic!("Expected RType instruction"),
        }
    }

    #[test]
    fn test_parse_itype() {
        let mut parser = Parser::new("ADDI R1, R2, 42");
        let program = parser.parse().unwrap();

        assert_eq!(program.instructions.len(), 1);
        match &program.instructions[0] {
            AsmInstruction::IType {
                mnemonic,
                rt,
                rs,
                immediate,
            } => {
                assert_eq!(mnemonic, "ADDI");
                assert_eq!(*rt, 1);
                assert_eq!(*rs, 2);
                assert_eq!(*immediate, 42);
            }
            _ => panic!("Expected IType instruction"),
        }
    }

    #[test]
    fn test_parse_zero_op() {
        let mut parser = Parser::new("HALT\nNOP");
        let program = parser.parse().unwrap();

        assert_eq!(program.instructions.len(), 2);
        assert!(matches!(
            &program.instructions[0],
            AsmInstruction::ZeroOp { mnemonic } if mnemonic == "HALT"
        ));
        assert!(matches!(
            &program.instructions[1],
            AsmInstruction::ZeroOp { mnemonic } if mnemonic == "NOP"
        ));
    }

    #[test]
    fn test_parse_labels() {
        let source = r#"
start:
    ADDI R1, R0, 10
loop:
    SUBI R1, R1, 1
    BNE R1, R0, loop
    HALT
"#;
        let mut parser = Parser::new(source);
        let program = parser.parse().unwrap();

        assert_eq!(program.labels.get("start"), Some(&0));
        assert_eq!(program.labels.get("loop"), Some(&1));
        assert_eq!(program.instructions.len(), 4);
    }

    #[test]
    fn test_parse_jump() {
        let mut parser = Parser::new("J start");
        let program = parser.parse().unwrap();

        assert_eq!(program.instructions.len(), 1);
        match &program.instructions[0] {
            AsmInstruction::JType { mnemonic, target } => {
                assert_eq!(mnemonic, "J");
                assert_eq!(target, "start");
            }
            _ => panic!("Expected JType instruction"),
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let source = r#"
; This is a comment
ADDI R1, R0, 42  ; Load 42 into R1
HALT
"#;
        let mut parser = Parser::new(source);
        let program = parser.parse().unwrap();

        assert_eq!(program.instructions.len(), 2);
    }
}
