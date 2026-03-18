//! Instruction set definition for the VOS CPU.
//!
//! The VOS CPU uses a simple RISC instruction set with three formats:
//! - R-Type: Register-to-register operations
//! - I-Type: Immediate operations
//! - J-Type: Jump operations

use serde::{Deserialize, Serialize};
use vos_core::{CpuError, RegisterIndex, Result, Word};

/// Opcode values for instructions.
///
/// The opcode is the first 6 bits of every instruction and determines
/// what operation to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // R-Type instructions (opcode 0, use funct field)
    RType = 0,

    // I-Type instructions
    ADDI = 1,
    SUBI = 2,
    ANDI = 3,
    ORI = 4,
    XORI = 5,
    LUI = 6,   // Load Upper Immediate
    LW = 7,    // Load Word
    SW = 8,    // Store Word
    LB = 9,    // Load Byte
    SB = 10,   // Store Byte
    BEQ = 11,  // Branch if Equal
    BNE = 12,  // Branch if Not Equal
    BLT = 13,  // Branch if Less Than
    BGT = 14,  // Branch if Greater Than
    BLE = 15,  // Branch if Less or Equal
    BGE = 16,  // Branch if Greater or Equal

    // J-Type instructions
    J = 17,    // Jump
    JAL = 18,  // Jump and Link

    // System instructions
    SYSCALL = 30,
    BREAK = 31,
    HALT = 32,
    NOP = 33,
}

/// Function codes for R-Type instructions.
///
/// When opcode is 0 (RType), the funct field specifies the exact operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum Funct {
    // Arithmetic
    ADD = 0,
    SUB = 1,
    MUL = 2,
    DIV = 3,

    // Logic
    AND = 4,
    OR = 5,
    XOR = 6,
    NOT = 7,

    // Shifts
    SLL = 8,   // Shift Left Logical
    SRL = 9,   // Shift Right Logical
    SRA = 10,  // Shift Right Arithmetic

    // Comparison
    SLT = 11,  // Set Less Than
    SGT = 12,  // Set Greater Than

    // Move/Jump
    MOV = 13,  // Move (pseudo: ADD rd, rs, R0)
    JR = 14,   // Jump Register
}

/// Instruction formats.
///
/// The VOS CPU uses three instruction formats, all 32 bits wide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    /// R-Type: Register operations
    ///
    /// Format: [opcode:6][rs:4][rt:4][rd:4][shamt:5][funct:9]
    ///
    /// Example: ADD R1, R2, R3  (R1 = R2 + R3)
    RType {
        funct: Funct,
        rd: RegisterIndex,  // Destination register
        rs: RegisterIndex,  // Source register 1
        rt: RegisterIndex,  // Source register 2
        shamt: u8,          // Shift amount (0-31)
    },

    /// I-Type: Immediate operations
    ///
    /// Format: [opcode:6][rs:4][rt:4][immediate:18]
    ///
    /// Example: ADDI R1, R2, 100  (R1 = R2 + 100)
    IType {
        opcode: Opcode,
        rt: RegisterIndex,     // Target/destination register
        rs: RegisterIndex,     // Source register
        immediate: i32,        // 18-bit immediate (sign-extended)
    },

    /// J-Type: Jump operations
    ///
    /// Format: [opcode:6][address:26]
    ///
    /// Example: J 0x1000  (PC = 0x1000)
    JType {
        opcode: Opcode,
        address: u32,  // 26-bit address
    },
}

impl Instruction {
    /// Encodes an instruction into a 32-bit word.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::instruction::{Instruction, Funct};
    ///
    /// let instr = Instruction::RType {
    ///     funct: Funct::ADD,
    ///     rd: 1,
    ///     rs: 2,
    ///     rt: 3,
    ///     shamt: 0,
    /// };
    ///
    /// let encoded = instr.encode();
    /// assert_eq!(encoded & 0xFC000000, 0); // Opcode is 0 for R-Type
    /// ```
    pub fn encode(&self) -> Word {
        match self {
            Instruction::RType { funct, rd, rs, rt, shamt } => {
                let opcode = 0u32; // R-Type always has opcode 0
                let funct_bits = (*funct as u32) & 0x1FF;  // 9 bits
                let shamt_bits = (*shamt as u32) & 0x1F;   // 5 bits
                let rd_bits = (*rd as u32) & 0xF;          // 4 bits
                let rt_bits = (*rt as u32) & 0xF;          // 4 bits
                let rs_bits = (*rs as u32) & 0xF;          // 4 bits

                (opcode << 26) |
                (rs_bits << 22) |
                (rt_bits << 18) |
                (rd_bits << 14) |
                (shamt_bits << 9) |
                funct_bits
            }

            Instruction::IType { opcode, rt, rs, immediate } => {
                let opcode_bits = (*opcode as u32) & 0x3F;  // 6 bits
                let rs_bits = (*rs as u32) & 0xF;           // 4 bits
                let rt_bits = (*rt as u32) & 0xF;           // 4 bits
                let imm_bits = (*immediate as u32) & 0x3FFFF;  // 18 bits

                (opcode_bits << 26) |
                (rs_bits << 22) |
                (rt_bits << 18) |
                imm_bits
            }

            Instruction::JType { opcode, address } => {
                let opcode_bits = (*opcode as u32) & 0x3F;  // 6 bits
                let addr_bits = (*address) & 0x3FFFFFF;     // 26 bits

                (opcode_bits << 26) | addr_bits
            }
        }
    }

    /// Decodes a 32-bit word into an instruction.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::instruction::{Instruction, Funct};
    ///
    /// // Create and encode an instruction
    /// let original = Instruction::RType {
    ///     funct: Funct::ADD,
    ///     rd: 1,
    ///     rs: 2,
    ///     rt: 3,
    ///     shamt: 0,
    /// };
    /// let encoded = original.encode();
    ///
    /// // Decode it back
    /// let decoded = Instruction::decode(encoded).unwrap();
    /// assert_eq!(original, decoded);
    /// ```
    pub fn decode(word: Word) -> Result<Self> {
        let opcode_bits = ((word >> 26) & 0x3F) as u8;

        match opcode_bits {
            0 => {
                // R-Type instruction
                let rs = ((word >> 22) & 0xF) as u8;
                let rt = ((word >> 18) & 0xF) as u8;
                let rd = ((word >> 14) & 0xF) as u8;
                let shamt = ((word >> 9) & 0x1F) as u8;
                let funct_bits = (word & 0x1FF) as u16;

                let funct = match funct_bits {
                    0 => Funct::ADD,
                    1 => Funct::SUB,
                    2 => Funct::MUL,
                    3 => Funct::DIV,
                    4 => Funct::AND,
                    5 => Funct::OR,
                    6 => Funct::XOR,
                    7 => Funct::NOT,
                    8 => Funct::SLL,
                    9 => Funct::SRL,
                    10 => Funct::SRA,
                    11 => Funct::SLT,
                    12 => Funct::SGT,
                    13 => Funct::MOV,
                    14 => Funct::JR,
                    _ => return Err(CpuError::InvalidInstruction(word).into()),
                };

                Ok(Instruction::RType { funct, rd, rs, rt, shamt })
            }

            1..=18 | 30..=33 => {
                // I-Type or special instructions
                let opcode = match opcode_bits {
                    1 => Opcode::ADDI,
                    2 => Opcode::SUBI,
                    3 => Opcode::ANDI,
                    4 => Opcode::ORI,
                    5 => Opcode::XORI,
                    6 => Opcode::LUI,
                    7 => Opcode::LW,
                    8 => Opcode::SW,
                    9 => Opcode::LB,
                    10 => Opcode::SB,
                    11 => Opcode::BEQ,
                    12 => Opcode::BNE,
                    13 => Opcode::BLT,
                    14 => Opcode::BGT,
                    15 => Opcode::BLE,
                    16 => Opcode::BGE,
                    17 => Opcode::J,
                    18 => Opcode::JAL,
                    30 => Opcode::SYSCALL,
                    31 => Opcode::BREAK,
                    32 => Opcode::HALT,
                    33 => Opcode::NOP,
                    _ => return Err(CpuError::InvalidInstruction(word).into()),
                };

                // J-Type instructions
                if matches!(opcode, Opcode::J | Opcode::JAL) {
                    let address = word & 0x3FFFFFF;
                    return Ok(Instruction::JType { opcode, address });
                }

                // I-Type instructions
                let rs = ((word >> 22) & 0xF) as u8;
                let rt = ((word >> 18) & 0xF) as u8;
                let imm_unsigned = word & 0x3FFFF;

                // Sign-extend 18-bit immediate to 32-bit
                let immediate = if imm_unsigned & 0x20000 != 0 {
                    // Negative number - sign extend
                    (imm_unsigned | 0xFFFC0000) as i32
                } else {
                    imm_unsigned as i32
                };

                Ok(Instruction::IType { opcode, rt, rs, immediate })
            }

            _ => Err(CpuError::InvalidInstruction(word).into()),
        }
    }

    /// Returns a human-readable string representation of the instruction.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::instruction::{Instruction, Funct};
    ///
    /// let instr = Instruction::RType {
    ///     funct: Funct::ADD,
    ///     rd: 1,
    ///     rs: 2,
    ///     rt: 3,
    ///     shamt: 0,
    /// };
    ///
    /// assert_eq!(instr.disassemble(), "ADD R1, R2, R3");
    /// ```
    pub fn disassemble(&self) -> String {
        match self {
            Instruction::RType { funct, rd, rs, rt, shamt } => match funct {
                Funct::ADD => format!("ADD R{}, R{}, R{}", rd, rs, rt),
                Funct::SUB => format!("SUB R{}, R{}, R{}", rd, rs, rt),
                Funct::MUL => format!("MUL R{}, R{}, R{}", rd, rs, rt),
                Funct::DIV => format!("DIV R{}, R{}, R{}", rd, rs, rt),
                Funct::AND => format!("AND R{}, R{}, R{}", rd, rs, rt),
                Funct::OR => format!("OR R{}, R{}, R{}", rd, rs, rt),
                Funct::XOR => format!("XOR R{}, R{}, R{}", rd, rs, rt),
                Funct::NOT => format!("NOT R{}, R{}", rd, rs),
                Funct::SLL => format!("SLL R{}, R{}, {}", rd, rt, shamt),
                Funct::SRL => format!("SRL R{}, R{}, {}", rd, rt, shamt),
                Funct::SRA => format!("SRA R{}, R{}, {}", rd, rt, shamt),
                Funct::SLT => format!("SLT R{}, R{}, R{}", rd, rs, rt),
                Funct::SGT => format!("SGT R{}, R{}, R{}", rd, rs, rt),
                Funct::MOV => format!("MOV R{}, R{}", rd, rs),
                Funct::JR => format!("JR R{}", rs),
            },

            Instruction::IType { opcode, rt, rs, immediate } => match opcode {
                Opcode::ADDI => format!("ADDI R{}, R{}, {}", rt, rs, immediate),
                Opcode::SUBI => format!("SUBI R{}, R{}, {}", rt, rs, immediate),
                Opcode::ANDI => format!("ANDI R{}, R{}, {}", rt, rs, immediate),
                Opcode::ORI => format!("ORI R{}, R{}, {}", rt, rs, immediate),
                Opcode::XORI => format!("XORI R{}, R{}, {}", rt, rs, immediate),
                Opcode::LUI => format!("LUI R{}, {}", rt, immediate),
                Opcode::LW => format!("LW R{}, {}(R{})", rt, immediate, rs),
                Opcode::SW => format!("SW R{}, {}(R{})", rt, immediate, rs),
                Opcode::LB => format!("LB R{}, {}(R{})", rt, immediate, rs),
                Opcode::SB => format!("SB R{}, {}(R{})", rt, immediate, rs),
                Opcode::BEQ => format!("BEQ R{}, R{}, {}", rs, rt, immediate),
                Opcode::BNE => format!("BNE R{}, R{}, {}", rs, rt, immediate),
                Opcode::BLT => format!("BLT R{}, R{}, {}", rs, rt, immediate),
                Opcode::BGT => format!("BGT R{}, R{}, {}", rs, rt, immediate),
                Opcode::BLE => format!("BLE R{}, R{}, {}", rs, rt, immediate),
                Opcode::BGE => format!("BGE R{}, R{}, {}", rs, rt, immediate),
                Opcode::SYSCALL => "SYSCALL".to_string(),
                Opcode::BREAK => "BREAK".to_string(),
                Opcode::HALT => "HALT".to_string(),
                Opcode::NOP => "NOP".to_string(),
                _ => format!("UNKNOWN {:?}", opcode),
            },

            Instruction::JType { opcode, address } => match opcode {
                Opcode::J => format!("J 0x{:08X}", address),
                Opcode::JAL => format!("JAL 0x{:08X}", address),
                _ => format!("UNKNOWN {:?} 0x{:08X}", opcode, address),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_rtype() {
        let instr = Instruction::RType {
            funct: Funct::ADD,
            rd: 1,
            rs: 2,
            rt: 3,
            shamt: 0,
        };

        let encoded = instr.encode();
        let decoded = Instruction::decode(encoded).unwrap();

        assert_eq!(instr, decoded);
    }

    #[test]
    fn test_encode_decode_itype() {
        let instr = Instruction::IType {
            opcode: Opcode::ADDI,
            rt: 1,
            rs: 2,
            immediate: 100,
        };

        let encoded = instr.encode();
        let decoded = Instruction::decode(encoded).unwrap();

        assert_eq!(instr, decoded);
    }

    #[test]
    fn test_encode_decode_jtype() {
        let instr = Instruction::JType {
            opcode: Opcode::J,
            address: 0x1000,
        };

        let encoded = instr.encode();
        let decoded = Instruction::decode(encoded).unwrap();

        assert_eq!(instr, decoded);
    }

    #[test]
    fn test_disassemble_add() {
        let instr = Instruction::RType {
            funct: Funct::ADD,
            rd: 1,
            rs: 2,
            rt: 3,
            shamt: 0,
        };

        assert_eq!(instr.disassemble(), "ADD R1, R2, R3");
    }

    #[test]
    fn test_disassemble_addi() {
        let instr = Instruction::IType {
            opcode: Opcode::ADDI,
            rt: 1,
            rs: 2,
            immediate: 100,
        };

        assert_eq!(instr.disassemble(), "ADDI R1, R2, 100");
    }

    #[test]
    fn test_sign_extension() {
        // Test positive immediate
        let instr = Instruction::IType {
            opcode: Opcode::ADDI,
            rt: 1,
            rs: 2,
            immediate: 100,
        };
        let encoded = instr.encode();
        let decoded = Instruction::decode(encoded).unwrap();
        if let Instruction::IType { immediate, .. } = decoded {
            assert_eq!(immediate, 100);
        } else {
            panic!("Expected IType");
        }

        // Test negative immediate
        let instr = Instruction::IType {
            opcode: Opcode::ADDI,
            rt: 1,
            rs: 2,
            immediate: -100,
        };
        let encoded = instr.encode();
        let decoded = Instruction::decode(encoded).unwrap();
        if let Instruction::IType { immediate, .. } = decoded {
            assert_eq!(immediate, -100);
        } else {
            panic!("Expected IType");
        }
    }

    #[test]
    fn test_invalid_instruction() {
        let invalid_word = 0xFFFFFFFF;
        assert!(Instruction::decode(invalid_word).is_err());
    }
}
