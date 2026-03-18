//! VOS CPU - 32-bit RISC processor emulator.
//!
//! This crate implements a complete CPU emulator for the VOS (Virtual Operating System).
//! The CPU is a simple RISC design with 16 registers and approximately 30 instructions.
//!
//! # Architecture
//!
//! - **32-bit word size**: All operations work on 32-bit values
//! - **16 registers**: R0 (always zero), R1-R14 (general purpose), R15 (stack pointer)
//! - **Simple instruction set**: Three instruction formats (R-type, I-type, J-type)
//! - **Status flags**: Zero, Negative, Carry, Overflow
//!
//! # Instruction Set
//!
//! ## Arithmetic
//! - ADD, SUB, MUL, DIV
//! - ADDI, SUBI (immediate versions)
//!
//! ## Logic
//! - AND, OR, XOR, NOT
//! - ANDI, ORI, XORI (immediate versions)
//!
//! ## Shifts
//! - SLL (Shift Left Logical)
//! - SRL (Shift Right Logical)
//! - SRA (Shift Right Arithmetic)
//!
//! ## Memory
//! - LW, SW (Load/Store Word)
//! - LB, SB (Load/Store Byte)
//!
//! ## Control Flow
//! - BEQ, BNE, BLT, BGT, BLE, BGE (conditional branches)
//! - J, JAL, JR (jumps)
//!
//! ## System
//! - SYSCALL, BREAK, HALT, NOP
//!
//! # Examples
//!
//! ## Basic CPU Usage
//!
//! ```
//! use vos_cpu::cpu::{Cpu, Memory};
//! use vos_cpu::instruction::{Instruction, Opcode};
//! use vos_core::{Address, Result, Word};
//!
//! // Simple memory implementation
//! struct SimpleMemory {
//!     data: Vec<Word>,
//! }
//!
//! impl Memory for SimpleMemory {
//!     fn read_word(&mut self, address: Address) -> Result<Word> {
//!         Ok(self.data[(address / 4) as usize])
//!     }
//!
//!     fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
//!         self.data[(address / 4) as usize] = value;
//!         Ok(())
//!     }
//!
//!     fn read_byte(&mut self, address: Address) -> Result<u8> {
//!         let word = self.read_word(address & !3)?;
//!         Ok(((word >> ((address & 3) * 8)) & 0xFF) as u8)
//!     }
//!
//!     fn write_byte(&mut self, address: Address, value: u8) -> Result<()> {
//!         let word_addr = address & !3;
//!         let mut word = self.read_word(word_addr)?;
//!         let shift = (address & 3) * 8;
//!         word = (word & !(0xFF << shift)) | ((value as Word) << shift);
//!         self.write_word(word_addr, word)
//!     }
//! }
//!
//! let mut cpu = Cpu::new();
//! let mut memory = SimpleMemory { data: vec![0; 256] };
//!
//! // Load a simple program: ADDI R1, R0, 42
//! let instr = Instruction::IType {
//!     opcode: Opcode::ADDI,
//!     rt: 1,
//!     rs: 0,
//!     immediate: 42,
//! };
//! memory.data[0] = instr.encode();
//!
//! // Execute one instruction
//! cpu.step(&mut memory).unwrap();
//!
//! // Check result
//! assert_eq!(cpu.registers.read(1), 42);
//! assert_eq!(cpu.pc(), 4);
//! ```
//!
//! ## Working with Instructions
//!
//! ```
//! use vos_cpu::instruction::{Instruction, Funct};
//!
//! // Create an ADD instruction
//! let instr = Instruction::RType {
//!     funct: Funct::ADD,
//!     rd: 1,  // R1 = destination
//!     rs: 2,  // R2 = operand 1
//!     rt: 3,  // R3 = operand 2
//!     shamt: 0,
//! };
//!
//! // Encode to machine code
//! let encoded = instr.encode();
//!
//! // Decode back
//! let decoded = Instruction::decode(encoded).unwrap();
//! assert_eq!(instr, decoded);
//!
//! // Disassemble for display
//! assert_eq!(instr.disassemble(), "ADD R1, R2, R3");
//! ```

pub mod alu;
pub mod cpu;
pub mod instruction;
pub mod registers;

// Re-export commonly used items
pub use alu::{Alu, AluResult};
pub use cpu::{Cpu, Memory};
pub use instruction::{Funct, Instruction, Opcode};
pub use registers::{Flags, Registers};
