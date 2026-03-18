//! VOS Assembler - Assembly language to machine code compiler.
//!
//! This crate provides an assembler for the VOS instruction set, converting
//! human-readable assembly code into executable machine code.
//!
//! # Assembly Syntax
//!
//! ```text
//! ; Comments start with semicolon
//! label:              ; Labels end with colon
//!     ADD  R1, R2, R3 ; R-type: ADD R1 = R2 + R3
//!     ADDI R1, R2, 10 ; I-type: ADD Immediate
//!     J    label      ; Jump to label
//!     NOP             ; No operation
//! ```
//!
//! # Instruction Formats
//!
//! - **R-type**: `OPCODE RD, RS, RT` - Register operations
//! - **I-type**: `OPCODE RT, RS, IMM` - Immediate operations
//! - **J-type**: `OPCODE LABEL` - Jump operations
//!
//! # Examples
//!
//! ```
//! use vos_asm::{Assembler, assemble};
//!
//! let source = r#"
//!     ADDI R1, R0, 42    ; R1 = 42
//!     ADDI R2, R0, 8     ; R2 = 8
//!     ADD  R3, R1, R2    ; R3 = R1 + R2
//!     HALT
//! "#;
//!
//! let machine_code = assemble(source).unwrap();
//! assert_eq!(machine_code.len(), 16); // 4 instructions * 4 bytes
//! ```

pub mod assembler;
pub mod lexer;
pub mod parser;

pub use assembler::{assemble, Assembler, AssemblerError};
pub use lexer::{Lexer, Token};
pub use parser::Parser;
