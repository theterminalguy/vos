//! System call interface.
//!
//! Provides the interface between user programs and the kernel.

use vos_core::{Result, Word};

/// System call numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Syscall {
    /// Exit process
    Exit = 1,

    /// Get process ID
    GetPid = 2,

    /// Write to file/device
    Write = 3,

    /// Read from file/device
    Read = 4,

    /// Allocate memory
    Malloc = 5,

    /// Free memory
    Free = 6,

    /// Sleep for N cycles
    Sleep = 7,

    /// Yield CPU to other processes
    Yield = 8,

    /// Get current time (cycles)
    GetTime = 9,

    /// Fork process (create child)
    Fork = 10,
}

impl Syscall {
    /// Converts a system call number to a Syscall enum.
    pub fn from_number(num: u32) -> Option<Self> {
        match num {
            1 => Some(Syscall::Exit),
            2 => Some(Syscall::GetPid),
            3 => Some(Syscall::Write),
            4 => Some(Syscall::Read),
            5 => Some(Syscall::Malloc),
            6 => Some(Syscall::Free),
            7 => Some(Syscall::Sleep),
            8 => Some(Syscall::Yield),
            9 => Some(Syscall::GetTime),
            10 => Some(Syscall::Fork),
            _ => None,
        }
    }

    /// Gets the system call number.
    pub fn number(&self) -> u32 {
        *self as u32
    }

    /// Gets the system call name.
    pub fn name(&self) -> &str {
        match self {
            Syscall::Exit => "exit",
            Syscall::GetPid => "getpid",
            Syscall::Write => "write",
            Syscall::Read => "read",
            Syscall::Malloc => "malloc",
            Syscall::Free => "free",
            Syscall::Sleep => "sleep",
            Syscall::Yield => "yield",
            Syscall::GetTime => "gettime",
            Syscall::Fork => "fork",
        }
    }
}

/// System call handler.
///
/// Processes system calls from user programs.
pub struct SyscallHandler {
    /// Total system calls executed
    syscall_count: u64,

    /// Syscall counts by type
    syscall_stats: [u64; 11],
}

impl SyscallHandler {
    /// Creates a new system call handler.
    pub fn new() -> Self {
        Self {
            syscall_count: 0,
            syscall_stats: [0; 11],
        }
    }

    /// Handles a system call.
    ///
    /// # Arguments
    ///
    /// * `syscall` - The system call to execute
    /// * `arg1` - First argument (usage depends on syscall)
    /// * `arg2` - Second argument
    /// * `arg3` - Third argument
    ///
    /// # Returns
    ///
    /// The return value in R1 (by convention)
    pub fn handle(
        &mut self,
        syscall: Syscall,
        arg1: Word,
        arg2: Word,
        arg3: Word,
    ) -> Result<Word> {
        self.syscall_count += 1;
        self.syscall_stats[syscall.number() as usize] += 1;

        match syscall {
            Syscall::Exit => self.sys_exit(arg1),
            Syscall::GetPid => self.sys_getpid(),
            Syscall::Write => self.sys_write(arg1, arg2, arg3),
            Syscall::Read => self.sys_read(arg1, arg2, arg3),
            Syscall::Malloc => self.sys_malloc(arg1),
            Syscall::Free => self.sys_free(arg1),
            Syscall::Sleep => self.sys_sleep(arg1),
            Syscall::Yield => self.sys_yield(),
            Syscall::GetTime => self.sys_gettime(),
            Syscall::Fork => self.sys_fork(),
        }
    }

    /// Exit the current process.
    ///
    /// # Arguments
    ///
    /// * `status` - Exit status code
    fn sys_exit(&mut self, status: Word) -> Result<Word> {
        // In a real implementation, this would terminate the process
        // For now, we just return the status code
        Ok(status)
    }

    /// Get the current process ID.
    fn sys_getpid(&mut self) -> Result<Word> {
        // In a real implementation, this would return the actual PID
        // For now, we return a placeholder
        Ok(1)
    }

    /// Write data to a file descriptor.
    ///
    /// # Arguments
    ///
    /// * `fd` - File descriptor (1 = stdout, 2 = stderr)
    /// * `buffer` - Address of data to write
    /// * `length` - Number of bytes to write
    ///
    /// # Returns
    ///
    /// Number of bytes written
    fn sys_write(&mut self, fd: Word, buffer: Word, length: Word) -> Result<Word> {
        // In a real implementation, this would write to the actual device
        // For now, we just return the length as if it was written
        let _ = (fd, buffer); // Suppress unused warnings
        Ok(length)
    }

    /// Read data from a file descriptor.
    ///
    /// # Arguments
    ///
    /// * `fd` - File descriptor (0 = stdin)
    /// * `buffer` - Address to write data to
    /// * `length` - Maximum number of bytes to read
    ///
    /// # Returns
    ///
    /// Number of bytes read
    fn sys_read(&mut self, fd: Word, buffer: Word, length: Word) -> Result<Word> {
        // In a real implementation, this would read from the actual device
        let _ = (fd, buffer, length); // Suppress unused warnings
        Ok(0)
    }

    /// Allocate memory.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    ///
    /// # Returns
    ///
    /// Address of allocated memory, or 0 on failure
    fn sys_malloc(&mut self, size: Word) -> Result<Word> {
        // In a real implementation, this would use a heap allocator
        let _ = size;
        Ok(0)
    }

    /// Free allocated memory.
    ///
    /// # Arguments
    ///
    /// * `address` - Address of memory to free
    fn sys_free(&mut self, address: Word) -> Result<Word> {
        // In a real implementation, this would return memory to the heap
        let _ = address;
        Ok(0)
    }

    /// Sleep for a number of cycles.
    ///
    /// # Arguments
    ///
    /// * `cycles` - Number of cycles to sleep
    fn sys_sleep(&mut self, cycles: Word) -> Result<Word> {
        // In a real implementation, this would block the process
        let _ = cycles;
        Ok(0)
    }

    /// Yield CPU to other processes.
    fn sys_yield(&mut self) -> Result<Word> {
        // In a real implementation, this would trigger scheduler
        Ok(0)
    }

    /// Get the current time in cycles.
    fn sys_gettime(&mut self) -> Result<Word> {
        // In a real implementation, this would return actual cycle count
        Ok(0)
    }

    /// Fork the current process (create a child).
    ///
    /// # Returns
    ///
    /// Child PID in parent, 0 in child
    fn sys_fork(&mut self) -> Result<Word> {
        // In a real implementation, this would create a new process
        Ok(0)
    }

    /// Returns the total number of system calls executed.
    pub fn total_syscalls(&self) -> u64 {
        self.syscall_count
    }

    /// Returns the count for a specific system call.
    pub fn syscall_stat(&self, syscall: Syscall) -> u64 {
        self.syscall_stats[syscall.number() as usize]
    }
}

impl Default for SyscallHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_from_number() {
        assert_eq!(Syscall::from_number(1), Some(Syscall::Exit));
        assert_eq!(Syscall::from_number(2), Some(Syscall::GetPid));
        assert_eq!(Syscall::from_number(3), Some(Syscall::Write));
        assert_eq!(Syscall::from_number(99), None);
    }

    #[test]
    fn test_syscall_number() {
        assert_eq!(Syscall::Exit.number(), 1);
        assert_eq!(Syscall::GetPid.number(), 2);
        assert_eq!(Syscall::Write.number(), 3);
    }

    #[test]
    fn test_syscall_name() {
        assert_eq!(Syscall::Exit.name(), "exit");
        assert_eq!(Syscall::GetPid.name(), "getpid");
        assert_eq!(Syscall::Write.name(), "write");
    }

    #[test]
    fn test_handler_creation() {
        let handler = SyscallHandler::new();
        assert_eq!(handler.total_syscalls(), 0);
    }

    #[test]
    fn test_sys_exit() {
        let mut handler = SyscallHandler::new();

        let result = handler.handle(Syscall::Exit, 42, 0, 0).unwrap();
        assert_eq!(result, 42);
        assert_eq!(handler.total_syscalls(), 1);
        assert_eq!(handler.syscall_stat(Syscall::Exit), 1);
    }

    #[test]
    fn test_sys_getpid() {
        let mut handler = SyscallHandler::new();

        let result = handler.handle(Syscall::GetPid, 0, 0, 0).unwrap();
        assert_eq!(result, 1);
        assert_eq!(handler.total_syscalls(), 1);
    }

    #[test]
    fn test_sys_write() {
        let mut handler = SyscallHandler::new();

        // Write 10 bytes to stdout (fd=1)
        let result = handler.handle(Syscall::Write, 1, 0x1000, 10).unwrap();
        assert_eq!(result, 10); // Returns bytes written
        assert_eq!(handler.syscall_stat(Syscall::Write), 1);
    }

    #[test]
    fn test_syscall_stats() {
        let mut handler = SyscallHandler::new();

        handler.handle(Syscall::Exit, 0, 0, 0).unwrap();
        handler.handle(Syscall::GetPid, 0, 0, 0).unwrap();
        handler.handle(Syscall::GetPid, 0, 0, 0).unwrap();
        handler.handle(Syscall::Write, 1, 0, 10).unwrap();

        assert_eq!(handler.total_syscalls(), 4);
        assert_eq!(handler.syscall_stat(Syscall::Exit), 1);
        assert_eq!(handler.syscall_stat(Syscall::GetPid), 2);
        assert_eq!(handler.syscall_stat(Syscall::Write), 1);
    }
}
