//! VOS Core - Fundamental types and traits for the Virtual Operating System.
//!
//! This crate provides the foundational building blocks used throughout the VOS project:
//!
//! - **Types**: Core data types like `Word`, `Address`, `Byte`, and memory regions
//! - **Errors**: Comprehensive error types for all VOS operations
//! - **Traits**: Common interfaces for devices, executable components, and more
//!
//! # Overview
//!
//! VOS (Virtual Operating System) is an educational project that simulates a complete
//! computer system, from CPU and memory to operating system kernel and applications.
//! This crate serves as the common foundation that all other VOS components depend on.
//!
//! # Examples
//!
//! ## Using core types
//!
//! ```
//! use vos_core::types::{Word, Address, AddressRange};
//!
//! let register_value: Word = 0x12345678;
//! let memory_address: Address = 0x00100000;
//! let kernel_space = AddressRange::new(0x00000000, 0x00200000);
//!
//! assert!(kernel_space.contains(memory_address));
//! ```
//!
//! ## Working with errors
//!
//! ```
//! use vos_core::error::{Result, CpuError};
//!
//! fn validate_register(index: u8) -> Result<u8> {
//!     if index < 16 {
//!         Ok(index)
//!     } else {
//!         Err(CpuError::InvalidRegister(index).into())
//!     }
//! }
//!
//! assert!(validate_register(15).is_ok());
//! assert!(validate_register(20).is_err());
//! ```

pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used items for convenience
pub use error::{CpuError, IoError, KernelError, MemoryError, Result, VosError};
pub use traits::{Clockable, Device, Executable, Inspectable};
pub use types::{Address, AddressRange, Byte, RegisterIndex, Word};
