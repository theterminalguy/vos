//! Error types used throughout VOS.
//!
//! This module defines common error types that can occur during
//! virtual machine operation, hardware simulation, and OS operations.

use thiserror::Error;

use crate::types::Address;

/// Result type alias using VosError.
pub type Result<T> = std::result::Result<T, VosError>;

/// Top-level error type for VOS operations.
///
/// This enum encompasses all possible errors that can occur in the VOS system.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum VosError {
    /// CPU-related errors
    #[error("CPU error: {0}")]
    Cpu(#[from] CpuError),

    /// Memory-related errors
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    /// I/O device errors
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    /// Kernel errors
    #[error("Kernel error: {0}")]
    Kernel(#[from] KernelError),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// CPU-specific errors.
///
/// These errors occur during CPU instruction execution.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CpuError {
    /// Invalid register index (must be 0-15)
    #[error("Invalid register index: {0} (must be 0-15)")]
    InvalidRegister(u8),

    /// Invalid or unrecognized instruction
    #[error("Invalid instruction: 0x{0:08X}")]
    InvalidInstruction(u32),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Arithmetic overflow
    #[error("Arithmetic overflow")]
    Overflow,

    /// Invalid CPU state
    #[error("Invalid CPU state: {0}")]
    InvalidState(String),

    /// Attempted to execute from non-executable memory
    #[error("Execution protection violation at address 0x{0:08X}")]
    ExecutionProtection(Address),

    /// Halted (not an error, but signals CPU should stop)
    #[error("CPU halted")]
    Halted,
}

/// Memory-related errors.
///
/// These errors occur during memory access operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    /// Address out of bounds
    #[error("Address out of bounds: 0x{address:08X} (size: 0x{size:08X})")]
    OutOfBounds { address: Address, size: usize },

    /// Unaligned memory access (word access must be 4-byte aligned)
    #[error("Unaligned memory access: 0x{address:08X} (alignment: {alignment})")]
    Unaligned { address: Address, alignment: usize },

    /// Access violation (attempted to access protected memory)
    #[error("Access violation at address 0x{0:08X}")]
    AccessViolation(Address),

    /// Attempted to write to read-only memory
    #[error("Write to read-only memory at address 0x{0:08X}")]
    ReadOnly(Address),

    /// Page fault
    #[error("Page fault at address 0x{address:08X} (present: {present}, write: {write})")]
    PageFault {
        address: Address,
        present: bool,
        write: bool,
    },

    /// Memory not mapped
    #[error("Memory not mapped at address 0x{0:08X}")]
    NotMapped(Address),
}

/// I/O device errors.
///
/// These errors occur during I/O operations with virtual devices.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum IoError {
    /// Device not found
    #[error("Device not found at address 0x{0:08X}")]
    DeviceNotFound(Address),

    /// Invalid device operation
    #[error("Invalid device operation: {0}")]
    InvalidOperation(String),

    /// Device not ready
    #[error("Device not ready: {0}")]
    NotReady(String),

    /// Device timeout
    #[error("Device timeout: {0}")]
    Timeout(String),

    /// Device-specific error
    #[error("Device error: {0}")]
    DeviceError(String),
}

/// Kernel-level errors.
///
/// These errors occur during OS kernel operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// Process not found
    #[error("Process not found: {0}")]
    ProcessNotFound(u32),

    /// Invalid system call number
    #[error("Invalid syscall: {0}")]
    InvalidSyscall(u32),

    /// System call failed
    #[error("Syscall failed: {0}")]
    SyscallFailed(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Not a directory
    #[error("Not a directory: {0}")]
    NotADirectory(String),

    /// Is a directory
    #[error("Is a directory: {0}")]
    IsADirectory(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_error_display() {
        let err = CpuError::InvalidRegister(20);
        assert_eq!(err.to_string(), "Invalid register index: 20 (must be 0-15)");
    }

    #[test]
    fn test_memory_error_display() {
        let err = MemoryError::OutOfBounds {
            address: 0x12345678,
            size: 0x1000,
        };
        assert_eq!(
            err.to_string(),
            "Address out of bounds: 0x12345678 (size: 0x00001000)"
        );
    }

    #[test]
    fn test_vos_error_conversion() {
        let cpu_err = CpuError::DivisionByZero;
        let vos_err: VosError = cpu_err.into();
        assert!(matches!(vos_err, VosError::Cpu(_)));
    }
}
