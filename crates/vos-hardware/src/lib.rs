//! VOS Hardware - Virtual Machine integration layer.
//!
//! This crate integrates all VOS components into a complete virtual computer:
//!
//! - **CPU**: 32-bit RISC processor from `vos-cpu`
//! - **Memory**: RAM with MMU from `vos-memory`
//! - **I/O Devices**: Display, Keyboard, Timer from `vos-io`
//!
//! # Architecture
//!
//! The `VirtualMachine` struct combines all components and provides a unified
//! interface for running programs. Memory-mapped I/O is handled transparently
//! through the `MemoryBus` which routes addresses to the appropriate device.
//!
//! # Memory Map
//!
//! ```text
//! 0x00000000 - 0x7FFFFFFF   Main Memory (RAM)
//! 0x80000000 - 0x80000F9F   Display (4000 bytes)
//! 0x80002000 - 0x8000200F   Keyboard (16 bytes)
//! 0x80003000 - 0x8000300F   Timer (16 bytes)
//! ```
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use vos_hardware::machine::VirtualMachine;
//! use vos_cpu::instruction::{Instruction, Opcode};
//!
//! // Create a VM with 1MB of memory
//! let mut vm = VirtualMachine::new(1024 * 1024);
//!
//! // Create ADDI R1, R0, 64 and HALT instructions
//! let addi = Instruction::IType {
//!     opcode: Opcode::ADDI,
//!     rt: 1,
//!     rs: 0,
//!     immediate: 64,
//! };
//! let halt = Instruction::IType {
//!     opcode: Opcode::HALT,
//!     rt: 0,
//!     rs: 0,
//!     immediate: 0,
//! };
//!
//! // Encode and load program
//! let mut program = Vec::new();
//! program.extend_from_slice(&addi.encode().to_le_bytes());
//! program.extend_from_slice(&halt.encode().to_le_bytes());
//! vm.load_program(0x1000, &program).unwrap();
//!
//! // Set PC and run
//! vm.cpu_mut().set_pc(0x1000);
//! vm.run().unwrap();
//!
//! // Check result
//! assert_eq!(vm.cpu().registers.read(1), 64);
//! ```
//!
//! ## Interactive Execution
//!
//! ```
//! use vos_hardware::machine::VirtualMachine;
//! use vos_cpu::instruction::{Instruction, Opcode};
//!
//! let mut vm = VirtualMachine::new(1024 * 1024);
//!
//! // Load HALT instruction for testing
//! # let halt = Instruction::IType {
//! #     opcode: Opcode::HALT,
//! #     rt: 0,
//! #     rs: 0,
//! #     immediate: 0,
//! # };
//! # vm.load_program(0, &halt.encode().to_le_bytes()).unwrap();
//! # vm.cpu_mut().set_pc(0);
//!
//! // Execute step by step
//! while vm.step().unwrap() {
//!     // Check state after each instruction
//!     if vm.cpu().registers.read(1) == 42 {
//!         break;
//!     }
//! }
//! ```
//!
//! ## Using I/O Devices
//!
//! ```
//! use vos_hardware::machine::VirtualMachine;
//!
//! let mut vm = VirtualMachine::new(1024 * 1024);
//!
//! // Write to display
//! vm.display_mut().put_char(b'H');
//! vm.display_mut().put_char(b'i');
//!
//! // Get display output
//! let output = vm.display_output();
//! assert!(output.starts_with("Hi"));
//!
//! // Simulate keyboard input
//! vm.keyboard_mut().push_key(b'A');
//! assert!(vm.keyboard().has_data());
//! ```

pub mod machine;

// Re-export main types
pub use machine::VirtualMachine;

// Re-export component types for convenience
pub use vos_cpu::Cpu;
pub use vos_io::{Display, Keyboard, Timer};
pub use vos_memory::Memory;
