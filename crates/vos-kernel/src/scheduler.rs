//! Process scheduler.
//!
//! Implements round-robin scheduling with configurable time quantum.

use crate::process::{Process, ProcessId, ProcessState};
use std::collections::VecDeque;

/// Process scheduler.
///
/// Uses round-robin scheduling: each process gets a fixed time quantum,
/// then the scheduler switches to the next ready process.
#[derive(Debug)]
pub struct Scheduler {
    /// All processes in the system
    processes: Vec<Process>,

    /// Ready queue (process IDs)
    ready_queue: VecDeque<ProcessId>,

    /// Currently running process ID
    current_pid: Option<ProcessId>,

    /// Next process ID to assign
    next_pid: ProcessId,

    /// Total context switches
    context_switches: u64,
}

impl Scheduler {
    /// Creates a new scheduler.
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            ready_queue: VecDeque::new(),
            current_pid: None,
            next_pid: 1,
            context_switches: 0,
        }
    }

    /// Creates a new process and adds it to the ready queue.
    pub fn create_process(
        &mut self,
        pc: vos_core::Address,
        base_address: vos_core::Address,
        memory_size: usize,
    ) -> ProcessId {
        let pid = self.next_pid;
        self.next_pid += 1;

        let process = Process::new(pid, pc, base_address, memory_size);
        self.processes.push(process);
        self.ready_queue.push_back(pid);

        pid
    }

    /// Gets the currently running process.
    pub fn current_process(&self) -> Option<&Process> {
        self.current_pid
            .and_then(|pid| self.processes.iter().find(|p| p.pid == pid))
    }

    /// Gets the currently running process (mutable).
    pub fn current_process_mut(&mut self) -> Option<&mut Process> {
        self.current_pid
            .and_then(|pid| self.processes.iter_mut().find(|p| p.pid == pid))
    }

    /// Gets a process by ID.
    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Gets a process by ID (mutable).
    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }

    /// Schedules the next process to run.
    ///
    /// Returns the PID of the next process, or None if no processes are ready.
    pub fn schedule(&mut self) -> Option<ProcessId> {
        // If current process is running, check its time quantum
        if let Some(current_pid) = self.current_pid {
            if let Some(process) = self.get_process_mut(current_pid) {
                if process.state == ProcessState::Running && process.time_quantum > 0 {
                    // Process still has time left
                    return Some(current_pid);
                }

                // Time quantum expired or process stopped
                if process.state == ProcessState::Running {
                    process.stop_running();
                    process.reset_quantum();
                    self.ready_queue.push_back(current_pid);
                }
            }
        }

        // Get next ready process
        while let Some(pid) = self.ready_queue.pop_front() {
            if let Some(process) = self.get_process_mut(pid) {
                if process.is_runnable() {
                    process.start_running();
                    self.current_pid = Some(pid);
                    self.context_switches += 1;
                    return Some(pid);
                }
            }
        }

        // No ready processes
        self.current_pid = None;
        None
    }

    /// Terminates the current process.
    pub fn terminate_current(&mut self) {
        if let Some(pid) = self.current_pid {
            if let Some(process) = self.get_process_mut(pid) {
                process.terminate();
            }
            self.current_pid = None;
        }
    }

    /// Yields the current process (puts it back in ready queue).
    pub fn yield_current(&mut self) {
        if let Some(pid) = self.current_pid {
            if let Some(process) = self.get_process_mut(pid) {
                if process.state == ProcessState::Running {
                    process.stop_running();
                    process.reset_quantum();
                    self.ready_queue.push_back(pid);
                }
            }
            self.current_pid = None;
        }
    }

    /// Returns the number of processes.
    pub fn process_count(&self) -> usize {
        self.processes
            .iter()
            .filter(|p| p.state != ProcessState::Terminated)
            .count()
    }

    /// Returns the number of ready processes.
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }

    /// Returns the total number of context switches.
    pub fn context_switches(&self) -> u64 {
        self.context_switches
    }

    /// Lists all processes.
    pub fn list_processes(&self) -> Vec<&Process> {
        self.processes.iter().collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.process_count(), 0);
        assert_eq!(scheduler.ready_count(), 0);
        assert!(scheduler.current_process().is_none());
    }

    #[test]
    fn test_create_process() {
        let mut scheduler = Scheduler::new();

        let pid1 = scheduler.create_process(0x1000, 0x10000, 0x1000);
        assert_eq!(pid1, 1);
        assert_eq!(scheduler.process_count(), 1);
        assert_eq!(scheduler.ready_count(), 1);

        let pid2 = scheduler.create_process(0x2000, 0x20000, 0x2000);
        assert_eq!(pid2, 2);
        assert_eq!(scheduler.process_count(), 2);
        assert_eq!(scheduler.ready_count(), 2);
    }

    #[test]
    fn test_schedule_single_process() {
        let mut scheduler = Scheduler::new();
        let pid = scheduler.create_process(0x1000, 0x10000, 0x1000);

        let scheduled = scheduler.schedule();
        assert_eq!(scheduled, Some(pid));

        let current = scheduler.current_process().unwrap();
        assert_eq!(current.pid, pid);
        assert_eq!(current.state, ProcessState::Running);
    }

    #[test]
    fn test_round_robin() {
        let mut scheduler = Scheduler::new();

        let pid1 = scheduler.create_process(0x1000, 0x10000, 0x1000);
        let pid2 = scheduler.create_process(0x2000, 0x20000, 0x1000);
        let pid3 = scheduler.create_process(0x3000, 0x30000, 0x1000);

        // First schedule
        assert_eq!(scheduler.schedule(), Some(pid1));

        // Exhaust time quantum
        let process = scheduler.current_process_mut().unwrap();
        process.time_quantum = 0;

        // Should switch to pid2
        assert_eq!(scheduler.schedule(), Some(pid2));

        // Exhaust time quantum
        let process = scheduler.current_process_mut().unwrap();
        process.time_quantum = 0;

        // Should switch to pid3
        assert_eq!(scheduler.schedule(), Some(pid3));

        // Exhaust time quantum
        let process = scheduler.current_process_mut().unwrap();
        process.time_quantum = 0;

        // Should wrap back to pid1
        assert_eq!(scheduler.schedule(), Some(pid1));
    }

    #[test]
    fn test_terminate_current() {
        let mut scheduler = Scheduler::new();

        let _pid1 = scheduler.create_process(0x1000, 0x10000, 0x1000);
        let pid2 = scheduler.create_process(0x2000, 0x20000, 0x1000);

        scheduler.schedule(); // Start pid1
        scheduler.terminate_current();

        assert!(scheduler.current_process().is_none());
        assert_eq!(scheduler.process_count(), 1); // Only pid2 active

        // Next schedule should be pid2
        assert_eq!(scheduler.schedule(), Some(pid2));
    }

    #[test]
    fn test_yield_current() {
        let mut scheduler = Scheduler::new();

        let _pid1 = scheduler.create_process(0x1000, 0x10000, 0x1000);
        let pid2 = scheduler.create_process(0x2000, 0x20000, 0x1000);

        scheduler.schedule(); // Start pid1
        scheduler.yield_current();

        assert!(scheduler.current_process().is_none());
        assert_eq!(scheduler.ready_count(), 2); // Both back in queue

        // Should schedule pid2 next (round-robin)
        assert_eq!(scheduler.schedule(), Some(pid2));
    }

    #[test]
    fn test_context_switches() {
        let mut scheduler = Scheduler::new();

        scheduler.create_process(0x1000, 0x10000, 0x1000);
        scheduler.create_process(0x2000, 0x20000, 0x1000);

        assert_eq!(scheduler.context_switches(), 0);

        scheduler.schedule();
        assert_eq!(scheduler.context_switches(), 1);

        scheduler.yield_current();
        scheduler.schedule();
        assert_eq!(scheduler.context_switches(), 2);
    }
}
