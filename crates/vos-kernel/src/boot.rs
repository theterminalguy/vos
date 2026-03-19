//! Kernel boot sequence.
//!
//! Handles system initialization and boot process.

use crate::scheduler::Scheduler;
use vos_core::Address;

/// Boot information.
#[derive(Debug, Clone)]
pub struct BootInfo {
    /// Kernel base address
    pub kernel_base: Address,

    /// Kernel size in bytes
    pub kernel_size: usize,

    /// Total memory available
    pub total_memory: usize,

    /// Initial process entry point
    pub init_entry: Address,
}

impl BootInfo {
    /// Creates boot information.
    pub fn new(
        kernel_base: Address,
        kernel_size: usize,
        total_memory: usize,
        init_entry: Address,
    ) -> Self {
        Self {
            kernel_base,
            kernel_size,
            total_memory,
            init_entry,
        }
    }
}

/// Boot the kernel.
///
/// # Boot Sequence
///
/// 1. Initialize hardware (CPU, Memory, I/O)
/// 2. Set up kernel data structures
/// 3. Initialize scheduler
/// 4. Create init process
/// 5. Start scheduler
///
/// # Arguments
///
/// * `boot_info` - Boot configuration
///
/// # Returns
///
/// Initialized scheduler with init process
pub fn boot_kernel(boot_info: &BootInfo) -> Scheduler {
    // 1. Hardware initialization (assumed already done)

    // 2. Set up kernel data structures
    let mut scheduler = Scheduler::new();

    // 3. Create init process
    let init_base = boot_info.kernel_base + boot_info.kernel_size as u32;
    let init_size = 0x10000; // 64KB for init process

    scheduler.create_process(boot_info.init_entry, init_base, init_size);

    // 4. Return scheduler (caller will start it)
    scheduler
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_info_creation() {
        let boot_info = BootInfo::new(0x1000, 0x10000, 16 * 1024 * 1024, 0x20000);

        assert_eq!(boot_info.kernel_base, 0x1000);
        assert_eq!(boot_info.kernel_size, 0x10000);
        assert_eq!(boot_info.total_memory, 16 * 1024 * 1024);
        assert_eq!(boot_info.init_entry, 0x20000);
    }

    #[test]
    fn test_boot_kernel() {
        let boot_info = BootInfo::new(0x1000, 0x10000, 16 * 1024 * 1024, 0x20000);

        let scheduler = boot_kernel(&boot_info);

        assert_eq!(scheduler.process_count(), 1);
        assert_eq!(scheduler.ready_count(), 1);

        // Init process should be ready
        let processes = scheduler.list_processes();
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].pid, 1);
    }
}
