//! Assembler - converts assembly instructions to machine code.

use crate::parser::{AsmInstruction, Parser, Program};
use std::collections::HashMap;
use thiserror::Error;
use vos_core::{Result, VosError};
use vos_cpu::instruction::{Funct, Instruction, Opcode};

/// Assembler errors.
#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Undefined label: {0}")]
    UndefinedLabel(String),

    #[error("Unknown mnemonic: {0}")]
    UnknownMnemonic(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<AssemblerError> for VosError {
    fn from(err: AssemblerError) -> Self {
        VosError::Assembler(err.to_string())
    }
}

/// Assembler for VOS assembly language.
pub struct Assembler {
    /// Base address for code (default 0)
    base_address: u32,
}

impl Assembler {
    /// Creates a new assembler.
    pub fn new() -> Self {
        Self { base_address: 0 }
    }

    /// Sets the base address for assembled code.
    pub fn with_base_address(mut self, address: u32) -> Self {
        self.base_address = address;
        self
    }

    /// Assembles source code into machine code.
    pub fn assemble(&self, source: &str) -> Result<Vec<u8>> {
        let mut parser = Parser::new(source);
        let program = parser.parse()?;

        self.assemble_program(&program)
    }

    fn assemble_program(&self, program: &Program) -> Result<Vec<u8>> {
        let mut machine_code = Vec::new();

        for (index, instr) in program.instructions.iter().enumerate() {
            let instruction = self.assemble_instruction(instr, index, &program.labels)?;
            let word = instruction.encode();

            // Little-endian encoding
            machine_code.extend_from_slice(&word.to_le_bytes());
        }

        Ok(machine_code)
    }

    fn assemble_instruction(
        &self,
        instr: &AsmInstruction,
        _pc: usize,
        labels: &HashMap<String, usize>,
    ) -> Result<Instruction> {
        match instr {
            AsmInstruction::RType {
                mnemonic,
                rd,
                rs,
                rt,
            } => {
                let funct = self.mnemonic_to_funct(mnemonic)?;
                Ok(Instruction::RType {
                    funct,
                    rd: *rd,
                    rs: *rs,
                    rt: *rt,
                    shamt: 0,
                })
            }

            AsmInstruction::IType {
                mnemonic,
                rt,
                rs,
                immediate,
            } => {
                let opcode = self.mnemonic_to_opcode(mnemonic)?;
                Ok(Instruction::IType {
                    opcode,
                    rt: *rt,
                    rs: *rs,
                    immediate: *immediate,
                })
            }

            AsmInstruction::JType { mnemonic, target } => {
                let opcode = self.mnemonic_to_opcode(mnemonic)?;

                // Resolve label to address
                let target_addr = if let Some(&label_pc) = labels.get(target) {
                    // Convert instruction index to byte address
                    self.base_address + (label_pc as u32 * 4)
                } else {
                    // Try to parse as immediate address
                    target.parse::<u32>().map_err(|_| {
                        VosError::Assembler(format!("Undefined label: {}", target))
                    })?
                };

                Ok(Instruction::JType {
                    opcode,
                    address: target_addr,
                })
            }

            AsmInstruction::ZeroOp { mnemonic } => {
                let opcode = self.mnemonic_to_opcode(mnemonic)?;
                Ok(Instruction::IType {
                    opcode,
                    rt: 0,
                    rs: 0,
                    immediate: 0,
                })
            }
        }
    }

    fn mnemonic_to_funct(&self, mnemonic: &str) -> Result<Funct> {
        let funct = match mnemonic {
            "ADD" => Funct::ADD,
            "SUB" => Funct::SUB,
            "MUL" => Funct::MUL,
            "DIV" => Funct::DIV,
            "AND" => Funct::AND,
            "OR" => Funct::OR,
            "XOR" => Funct::XOR,
            "NOT" => Funct::NOT,
            "SLL" => Funct::SLL,
            "SRL" => Funct::SRL,
            "SRA" => Funct::SRA,
            "SLT" => Funct::SLT,
            "SGT" => Funct::SGT,
            _ => return Err(AssemblerError::UnknownMnemonic(mnemonic.to_string()).into()),
        };
        Ok(funct)
    }

    fn mnemonic_to_opcode(&self, mnemonic: &str) -> Result<Opcode> {
        let opcode = match mnemonic {
            "ADDI" => Opcode::ADDI,
            "SUBI" => Opcode::SUBI,
            "ANDI" => Opcode::ANDI,
            "ORI" => Opcode::ORI,
            "XORI" => Opcode::XORI,
            "LUI" => Opcode::LUI,
            "LW" => Opcode::LW,
            "SW" => Opcode::SW,
            "LB" => Opcode::LB,
            "SB" => Opcode::SB,
            "BEQ" => Opcode::BEQ,
            "BNE" => Opcode::BNE,
            "BLT" => Opcode::BLT,
            "BGT" => Opcode::BGT,
            "BLE" => Opcode::BLE,
            "BGE" => Opcode::BGE,
            "J" => Opcode::J,
            "JAL" => Opcode::JAL,
            "SYSCALL" => Opcode::SYSCALL,
            "BREAK" => Opcode::BREAK,
            "HALT" => Opcode::HALT,
            "NOP" => Opcode::NOP,
            _ => return Err(AssemblerError::UnknownMnemonic(mnemonic.to_string()).into()),
        };
        Ok(opcode)
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to assemble source code.
///
/// # Examples
///
/// ```
/// use vos_asm::assemble;
///
/// let source = r#"
///     ADDI R1, R0, 42
///     HALT
/// "#;
///
/// let machine_code = assemble(source).unwrap();
/// assert_eq!(machine_code.len(), 8); // 2 instructions * 4 bytes
/// ```
pub fn assemble(source: &str) -> Result<Vec<u8>> {
    Assembler::new().assemble(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_simple() {
        let source = "ADDI R1, R0, 42";
        let machine_code = assemble(source).unwrap();

        assert_eq!(machine_code.len(), 4); // 1 instruction * 4 bytes

        // Verify it's a valid instruction
        let word = u32::from_le_bytes([
            machine_code[0],
            machine_code[1],
            machine_code[2],
            machine_code[3],
        ]);

        let decoded = Instruction::decode(word).unwrap();
        assert!(matches!(decoded, Instruction::IType { .. }));
    }

    #[test]
    fn test_assemble_multiple() {
        let source = r#"
            ADDI R1, R0, 10
            ADDI R2, R0, 20
            ADD  R3, R1, R2
            HALT
        "#;

        let machine_code = assemble(source).unwrap();
        assert_eq!(machine_code.len(), 16); // 4 instructions * 4 bytes
    }

    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
start:
    ADDI R1, R0, 5
loop:
    SUBI R1, R1, 1
    BNE R1, R0, loop
    HALT
        "#;

        let machine_code = assemble(source).unwrap();
        assert_eq!(machine_code.len(), 16); // 4 instructions
    }

    #[test]
    fn test_assemble_jump() {
        let source = r#"
    J main
    NOP
main:
    HALT
        "#;

        let machine_code = assemble(source).unwrap();
        assert_eq!(machine_code.len(), 12); // 3 instructions
    }

    #[test]
    fn test_assemble_rtype() {
        let source = "ADD R1, R2, R3";
        let machine_code = assemble(source).unwrap();

        assert_eq!(machine_code.len(), 4);

        let word = u32::from_le_bytes([
            machine_code[0],
            machine_code[1],
            machine_code[2],
            machine_code[3],
        ]);

        let decoded = Instruction::decode(word).unwrap();
        match decoded {
            Instruction::RType { rd, rs, rt, .. } => {
                assert_eq!(rd, 1);
                assert_eq!(rs, 2);
                assert_eq!(rt, 3);
            }
            _ => panic!("Expected RType instruction"),
        }
    }

    #[test]
    fn test_assemble_with_comments() {
        let source = r#"
; Initialize counter
ADDI R1, R0, 10  ; R1 = 10

; Main loop
loop:
    SUBI R1, R1, 1  ; Decrement
    BNE R1, R0, loop ; Loop if not zero

HALT ; Done
        "#;

        let machine_code = assemble(source).unwrap();
        assert_eq!(machine_code.len(), 16); // 4 instructions
    }

    #[test]
    fn test_base_address() {
        let source = r#"
    J target
target:
    HALT
        "#;

        let assembler = Assembler::new().with_base_address(0x1000);
        let machine_code = assembler.assemble(source).unwrap();

        assert_eq!(machine_code.len(), 8); // 2 instructions
    }
}
