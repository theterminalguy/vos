//! VOS Kernel - Operating system kernel for VOS.
//!
//! Provides core OS functionality:
//! - Process management and scheduling
//! - System calls
//! - Boot sequence
//! - User/kernel mode separation
//!
//! # Architecture
//!
//! ```text
//! User Programs
//!      |
//!      | (system calls)
//!      ▼
//! ┌─────────────────────────┐
//! │    System Call Layer    │
//! └─────────────────────────┘
//!      |
//!      ▼
//! ┌─────────────────────────┐
//! │   Process Scheduler     │
//! └─────────────────────────┘
//!      |
//!      ▼
//! ┌─────────────────────────┐
//! │    Process Manager      │
//! └─────────────────────────┘
//! ```

pub mod boot;
pub mod fs;
pub mod process;
pub mod scheduler;
pub mod syscall;

pub use boot::boot_kernel;
pub use fs::{OpenMode, Vfs, VfsError};
pub use process::{Process, ProcessId, ProcessState};
pub use scheduler::Scheduler;
pub use syscall::{Syscall, SyscallHandler};
