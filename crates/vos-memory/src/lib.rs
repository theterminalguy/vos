//! VOS Memory - Memory subsystem for the Virtual Operating System.
//!
//! This crate implements the memory hierarchy for VOS, including:
//!
//! - **RAM**: Physical memory storage with byte and word access
//! - **MMU**: Memory Management Unit with virtual address translation
//! - **Paging**: Simple page-based memory management (4KB pages)
//! - **Memory**: Integrated memory subsystem combining RAM and MMU
//!
//! # Memory Model
//!
//! The VOS memory system uses a 32-bit address space (4GB total):
//!
//! ```text
//! 0x00000000 - 0x00000FFF   Interrupt Vector Table (4KB)
//! 0x00001000 - 0x000FFFFF   Kernel Code + Data (~1MB)
//! 0x00100000 - 0x001FFFFF   Kernel Stack (1MB)
//! 0x00200000 - 0x3FFFFFFF   User Space (~1GB)
//! 0x40000000 - 0x7FFFFFFF   Heap (1GB)
//! 0x80000000 - 0xBFFFFFFF   Memory-Mapped I/O (1GB)
//! 0xC0000000 - 0xFFFFFFFF   Reserved (1GB)
//! ```
//!
//! # Paging
//!
//! The MMU supports simple paging with:
//! - 4KB page size
//! - Page table with permissions (read, write, execute)
//! - Identity mapping or custom mappings
//! - Page fault detection
//!
//! # Examples
//!
//! ## Basic Memory Usage
//!
//! ```
//! use vos_memory::memory::Memory;
//!
//! // Create 16MB of memory
//! let mut memory = Memory::new(16 * 1024 * 1024);
//!
//! // Write and read
//! memory.write_word(0x1000, 0x12345678).unwrap();
//! assert_eq!(memory.read_word(0x1000).unwrap(), 0x12345678);
//! ```
//!
//! ## Using with the CPU
//!
//! ```
//! use vos_memory::memory::Memory;
//! use vos_cpu::cpu::Cpu;
//!
//! let mut cpu = Cpu::new();
//! let mut memory = Memory::new(1024 * 1024);
//!
//! // Load a program
//! let program = vec![0x04, 0x00, 0x40, 0x00]; // Example instruction
//! memory.load(0, &program).unwrap();
//!
//! // Execute
//! cpu.set_pc(0);
//! // cpu.step(&mut memory).unwrap();
//! ```
//!
//! ## Working with Paging
//!
//! ```
//! use vos_memory::memory::Memory;
//! use vos_memory::mmu::{PageTableEntry, PAGE_SIZE};
//!
//! let mut memory = Memory::new(16 * 1024 * 1024);
//!
//! // Enable paging
//! memory.mmu_mut().enable_paging();
//!
//! // Map virtual page 100 to physical frame 50
//! memory.mmu_mut().map_page(100, PageTableEntry::new(50));
//!
//! // Access through virtual address
//! let virtual_addr = 100 * PAGE_SIZE as u32;
//! memory.write_word(virtual_addr, 0xDEADBEEF).unwrap();
//! ```
//!
//! ## Loading Programs
//!
//! ```
//! use vos_memory::memory::Memory;
//!
//! let mut memory = Memory::new(1024 * 1024);
//!
//! // Load a program at address 0x1000
//! let program = vec![
//!     0x01, 0x02, 0x03, 0x04, // Instructions
//!     0x05, 0x06, 0x07, 0x08,
//! ];
//!
//! memory.load(0x1000, &program).unwrap();
//!
//! // Verify
//! assert_eq!(memory.read_byte(0x1000).unwrap(), 0x01);
//! ```

pub mod memory;
pub mod mmu;
pub mod ram;

// Re-export commonly used items
pub use memory::Memory;
pub use mmu::{Mmu, PageTable, PageTableEntry, PAGE_SIZE};
pub use ram::Ram;
