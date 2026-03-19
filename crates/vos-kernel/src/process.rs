//! Process management.
//!
//! Handles process creation, state management, and context switching.

use vos_core::{Address, Word};

/// Process identifier.
pub type ProcessId = u32;

/// Process state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is ready to run
    Ready,
    /// Process is currently running
    Running,
    /// Process is blocked (waiting for I/O, etc.)
    Blocked,
    /// Process has terminated
    Terminated,
}

/// Process Control Block (PCB).
///
/// Stores all information about a process.
#[derive(Debug, Clone)]
pub struct Process {
    /// Unique process ID
    pub pid: ProcessId,

    /// Current state
    pub state: ProcessState,

    /// Program counter (saved when not running)
    pub pc: Address,

    /// Saved register values (R0-R15)
    pub registers: [Word; 16],

    /// Stack pointer
    pub stack_pointer: Address,

    /// Base address of process memory
    pub base_address: Address,

    /// Size of process memory
    pub memory_size: usize,

    /// Priority (0 = highest)
    pub priority: u8,

    /// CPU time used (in cycles)
    pub cpu_time: u64,

    /// Time quantum remaining
    pub time_quantum: u64,
}

impl Process {
    /// Creates a new process.
    pub fn new(
        pid: ProcessId,
        pc: Address,
        base_address: Address,
        memory_size: usize,
    ) -> Self {
        Self {
            pid,
            state: ProcessState::Ready,
            pc,
            registers: [0; 16],
            stack_pointer: base_address + memory_size as u32,
            base_address,
            memory_size,
            priority: 0,
            cpu_time: 0,
            time_quantum: 100, // Default: 100 cycles
        }
    }

    /// Saves the process context from CPU state.
    pub fn save_context(&mut self, pc: Address, registers: &[Word; 16]) {
        self.pc = pc;
        self.registers.copy_from_slice(registers);
    }

    /// Restores the process context to CPU state.
    pub fn restore_context(&self) -> (Address, [Word; 16]) {
        (self.pc, self.registers)
    }

    /// Checks if the process is runnable.
    pub fn is_runnable(&self) -> bool {
        self.state == ProcessState::Ready
    }

    /// Transitions to running state.
    pub fn start_running(&mut self) {
        self.state = ProcessState::Running;
    }

    /// Transitions to ready state.
    pub fn stop_running(&mut self) {
        if self.state == ProcessState::Running {
            self.state = ProcessState::Ready;
        }
    }

    /// Marks process as terminated.
    pub fn terminate(&mut self) {
        self.state = ProcessState::Terminated;
    }

    /// Resets time quantum.
    pub fn reset_quantum(&mut self) {
        self.time_quantum = 100;
    }

    /// Uses one cycle of time quantum.
    pub fn use_quantum(&mut self) -> bool {
        if self.time_quantum > 0 {
            self.time_quantum -= 1;
            self.cpu_time += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let process = Process::new(1, 0x1000, 0x10000, 0x1000);

        assert_eq!(process.pid, 1);
        assert_eq!(process.state, ProcessState::Ready);
        assert_eq!(process.pc, 0x1000);
        assert_eq!(process.base_address, 0x10000);
        assert_eq!(process.memory_size, 0x1000);
    }

    #[test]
    fn test_state_transitions() {
        let mut process = Process::new(1, 0, 0, 0x1000);

        assert_eq!(process.state, ProcessState::Ready);
        assert!(process.is_runnable());

        process.start_running();
        assert_eq!(process.state, ProcessState::Running);
        assert!(!process.is_runnable());

        process.stop_running();
        assert_eq!(process.state, ProcessState::Ready);
        assert!(process.is_runnable());

        process.terminate();
        assert_eq!(process.state, ProcessState::Terminated);
        assert!(!process.is_runnable());
    }

    #[test]
    fn test_context_save_restore() {
        let mut process = Process::new(1, 0x1000, 0x10000, 0x1000);

        let registers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        process.save_context(0x2000, &registers);

        assert_eq!(process.pc, 0x2000);
        assert_eq!(process.registers, registers);

        let (pc, regs) = process.restore_context();
        assert_eq!(pc, 0x2000);
        assert_eq!(regs, registers);
    }

    #[test]
    fn test_time_quantum() {
        let mut process = Process::new(1, 0, 0, 0x1000);
        process.time_quantum = 3;

        assert!(process.use_quantum());
        assert_eq!(process.time_quantum, 2);
        assert_eq!(process.cpu_time, 1);

        assert!(process.use_quantum());
        assert_eq!(process.time_quantum, 1);

        assert!(process.use_quantum());
        assert_eq!(process.time_quantum, 0);

        assert!(!process.use_quantum());
        assert_eq!(process.cpu_time, 3);
    }
}
