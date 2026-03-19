# Chapter 6: Kernel Fundamentals

## Learning Objectives

By the end of this chapter, you will:
- Understand how an operating system boots
- Learn about processes and process management
- Understand scheduling and CPU time-slicing
- Learn how system calls work
- Implement user/kernel mode separation
- Write programs that use system calls

## Introduction

The **kernel** is the core of an operating system. It manages system resources (CPU, memory, I/O), provides services to user programs through **system calls**, and ensures processes don't interfere with each other. In this chapter, we'll build VOS's kernel from the ground up.

## What is a Kernel?

The kernel sits between hardware and user programs:

```
┌────────────────────────────┐
│     User Programs          │  Application layer
│   (shell, editor, etc.)    │
└────────────┬───────────────┘
             │ System calls
             ▼
┌────────────────────────────┐
│         Kernel             │  Operating system
│  (scheduler, syscalls,     │
│   process mgmt, drivers)   │
└────────────┬───────────────┘
             │ Hardware access
             ▼
┌────────────────────────────┐
│    Hardware (CPU,          │  Physical hardware
│     Memory, I/O)           │
└────────────────────────────┘
```

### Kernel Responsibilities

1. **Process Management**: Create, schedule, and terminate processes
2. **Memory Management**: Allocate and protect memory
3. **I/O Management**: Control access to devices
4. **System Calls**: Provide services to user programs
5. **Security**: Enforce permissions and isolation

## Boot Sequence

When VOS starts, it goes through a boot sequence:

### Boot Steps

```rust
pub fn boot_kernel(boot_info: &BootInfo) -> Scheduler {
    // 1. Hardware initialization (CPU, Memory, I/O)
    //    Assumed done by VirtualMachine

    // 2. Set up kernel data structures
    let mut scheduler = Scheduler::new();

    // 3. Create init process (PID 1)
    let init_base = boot_info.kernel_base + boot_info.kernel_size as u32;
    let init_size = 0x10000; // 64KB

    scheduler.create_process(
        boot_info.init_entry,  // Entry point
        init_base,              // Base address
        init_size               // Memory size
    );

    // 4. Return scheduler (caller will start it)
    scheduler
}
```

### Boot Information

```rust
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
```

**Example boot:**
```rust
let boot_info = BootInfo::new(
    0x1000,              // Kernel at 0x1000
    0x10000,             // Kernel is 64KB
    16 * 1024 * 1024,    // 16MB total RAM
    0x20000              // Init starts at 0x20000
);

let scheduler = boot_kernel(&boot_info);
```

## Processes

A **process** is a running program. Each process has its own memory, registers, and state.

### Process Control Block (PCB)

The PCB stores all information about a process:

```rust
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
```

### Process States

A process can be in one of four states:

```rust
pub enum ProcessState {
    /// Ready to run (waiting for CPU)
    Ready,

    /// Currently running on CPU
    Running,

    /// Blocked (waiting for I/O)
    Blocked,

    /// Terminated (finished)
    Terminated,
}
```

**State Diagram:**
```
     ┌─────────┐
     │  Ready  │◄──────┐
     └────┬────┘       │
          │ schedule   │ time quantum
          │            │ expired
          ▼            │
     ┌─────────┐       │
     │ Running ├───────┘
     └────┬────┘
          │ exit/terminate
          ▼
     ┌─────────┐
     │Terminated│
     └─────────┘
```

### Creating a Process

```rust
let process = Process::new(
    1,          // PID
    0x1000,     // Entry point (PC)
    0x10000,    // Base address
    0x10000     // Memory size (64KB)
);

assert_eq!(process.pid, 1);
assert_eq!(process.state, ProcessState::Ready);
assert_eq!(process.pc, 0x1000);
```

### Context Switching

When switching between processes, the kernel must save and restore context:

```rust
// Save current process context
let pc = cpu.registers.pc;
let registers = cpu.registers.gpr;
current_process.save_context(pc, &registers);

// Load next process context
let (new_pc, new_registers) = next_process.restore_context();
cpu.registers.pc = new_pc;
cpu.registers.gpr = new_registers;
```

## Process Scheduling

The **scheduler** decides which process runs next. VOS uses **round-robin** scheduling.

### Round-Robin Scheduling

Each process gets a fixed **time quantum** (e.g., 100 CPU cycles). When the quantum expires, the scheduler switches to the next ready process.

```
Process 1 ─────────┐
                   │ (100 cycles)
                   ▼
Process 2 ─────────┐
                   │ (100 cycles)
                   ▼
Process 3 ─────────┐
                   │ (100 cycles)
                   ▼
Process 1 ─────────  (back to start)
```

### Scheduler Structure

```rust
pub struct Scheduler {
    /// All processes in the system
    processes: Vec<Process>,

    /// Ready queue (FIFO)
    ready_queue: VecDeque<ProcessId>,

    /// Currently running process
    current_pid: Option<ProcessId>,

    /// Next PID to assign
    next_pid: ProcessId,

    /// Context switch counter
    context_switches: u64,
}
```

### Scheduling Algorithm

```rust
pub fn schedule(&mut self) -> Option<ProcessId> {
    // If current process has time left, keep running
    if let Some(current_pid) = self.current_pid {
        if let Some(process) = self.get_process_mut(current_pid) {
            if process.state == ProcessState::Running
                && process.time_quantum > 0
            {
                return Some(current_pid);
            }

            // Quantum expired, move to back of queue
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

    None  // No ready processes
}
```

### Example: Round-Robin

```rust
let mut scheduler = Scheduler::new();

// Create 3 processes
let pid1 = scheduler.create_process(0x1000, 0x10000, 0x1000);
let pid2 = scheduler.create_process(0x2000, 0x20000, 0x1000);
let pid3 = scheduler.create_process(0x3000, 0x30000, 0x1000);

// First schedule: pid1
assert_eq!(scheduler.schedule(), Some(pid1));

// Exhaust pid1's quantum
scheduler.current_process_mut().unwrap().time_quantum = 0;

// Next schedule: pid2
assert_eq!(scheduler.schedule(), Some(pid2));

// Exhaust pid2's quantum
scheduler.current_process_mut().unwrap().time_quantum = 0;

// Next schedule: pid3
assert_eq!(scheduler.schedule(), Some(pid3));

// Exhaust pid3's quantum
scheduler.current_process_mut().unwrap().time_quantum = 0;

// Wrap back to pid1
assert_eq!(scheduler.schedule(), Some(pid1));
```

## System Calls

**System calls** (syscalls) are the interface between user programs and the kernel. They allow programs to request OS services.

### System Call Numbers

```rust
pub enum Syscall {
    Exit = 1,      // Exit process
    GetPid = 2,    // Get process ID
    Write = 3,     // Write to file/device
    Read = 4,      // Read from file/device
    Malloc = 5,    // Allocate memory
    Free = 6,      // Free memory
    Sleep = 7,     // Sleep for N cycles
    Yield = 8,     // Yield CPU
    GetTime = 9,   // Get current time
    Fork = 10,     // Fork process
}
```

### Making a System Call

From assembly:
```assembly
; Call write(fd=1, buffer=0x1000, length=10)
ADDI R1, R0, 3      ; R1 = syscall number (3 = Write)
ADDI R2, R0, 1      ; R2 = fd (1 = stdout)
LUI  R3, 0x1000     ; R3 = buffer address
ADDI R4, R0, 10     ; R4 = length
SYSCALL             ; Execute system call
; Return value in R1
```

### System Call Handler

```rust
pub fn handle(
    &mut self,
    syscall: Syscall,
    arg1: Word,
    arg2: Word,
    arg3: Word,
) -> Result<Word> {
    self.syscall_count += 1;

    match syscall {
        Syscall::Exit => self.sys_exit(arg1),
        Syscall::GetPid => self.sys_getpid(),
        Syscall::Write => self.sys_write(arg1, arg2, arg3),
        // ... other syscalls
    }
}
```

### Implementing a System Call

**Example: `exit`**

```rust
fn sys_exit(&mut self, status: Word) -> Result<Word> {
    // Mark process as terminated
    // (In real implementation, would clean up process)
    Ok(status)
}
```

**Example: `getpid`**

```rust
fn sys_getpid(&mut self) -> Result<Word> {
    // Return current process ID
    Ok(self.current_pid as Word)
}
```

**Example: `write`**

```rust
fn sys_write(&mut self, fd: Word, buffer: Word, length: Word) -> Result<Word> {
    // fd: file descriptor (1 = stdout, 2 = stderr)
    // buffer: memory address of data
    // length: number of bytes

    match fd {
        1 => {
            // Write to stdout (display)
            for offset in 0..length {
                let byte = memory.read_byte(buffer + offset)?;
                display.put_char(byte);
            }
            Ok(length)
        }
        _ => Err("Invalid file descriptor".into()),
    }
}
```

## User Mode vs Kernel Mode

Modern CPUs have privilege levels to protect the kernel:

```
┌─────────────────────────┐
│    User Mode (Ring 3)   │  User programs
│  - Limited privileges   │  - Can't access hardware
│  - Uses syscalls        │  - Can't modify kernel
└────────────┬────────────┘
             │ SYSCALL
             ▼
┌─────────────────────────┐
│   Kernel Mode (Ring 0)  │  Kernel code
│  - Full privileges      │  - Direct hardware access
│  - Can access hardware  │  - Can modify anything
└─────────────────────────┘
```

**VOS Simplified Model:**
- User programs run with restricted memory access
- SYSCALL instruction transitions to kernel mode
- Kernel validates requests and performs operations
- Return to user mode after syscall completes

## Example: Using System Calls

### Example 1: Hello World with Syscalls

```assembly
; Write "Hello" to stdout using syscall

start:
    ; Prepare string in memory
    LUI  R10, 0x1000
    ADDI R2, R0, 72     ; 'H'
    SB   R2, R10, 0
    ADDI R2, R0, 101    ; 'e'
    SB   R2, R10, 1
    ADDI R2, R0, 108    ; 'l'
    SB   R2, R10, 2
    SB   R2, R10, 3     ; second 'l'
    ADDI R2, R0, 111    ; 'o'
    SB   R2, R10, 4

    ; Call write(1, 0x1000, 5)
    ADDI R1, R0, 3      ; syscall 3 = write
    ADDI R2, R0, 1      ; fd = 1 (stdout)
    ADD  R3, R0, R10    ; buffer = 0x1000
    ADDI R4, R0, 5      ; length = 5
    SYSCALL

    ; Exit with status 0
    ADDI R1, R0, 1      ; syscall 1 = exit
    ADDI R2, R0, 0      ; status = 0
    SYSCALL
```

### Example 2: Get Process ID

```assembly
; Get and display PID

start:
    ; Call getpid()
    ADDI R1, R0, 2      ; syscall 2 = getpid
    SYSCALL
    ; R1 now contains PID

    ; Store PID in R10
    ADD  R10, R0, R1

    ; ... use PID

    HALT
```

### Example 3: Process Yielding

```assembly
; Yield CPU to other processes

loop:
    ; Do some work
    ADDI R5, R5, 1

    ; Check if should yield
    ADDI R6, R0, 100
    BLT  R5, R6, loop   ; Continue if < 100

    ; Yield CPU
    ADDI R1, R0, 8      ; syscall 8 = yield
    SYSCALL

    ; Reset counter
    ADDI R5, R0, 0
    J    loop
```

## Hands-On Exercise: Multi-Process Program

Write two programs that share CPU time through the scheduler.

**Program 1: Counter**
```assembly
; Count from 0 to 100, yielding every 10

counter:
    ADDI R10, R0, 0     ; Counter = 0

loop1:
    ADDI R10, R10, 1    ; Increment
    ADDI R11, R0, 10
    ; Yield every 10 counts
    ; (Implementation left as exercise)
    ADDI R12, R0, 100
    BLT  R10, R12, loop1

    HALT
```

**Program 2: Timer**
```assembly
; Read timer and update every quantum

timer:
    ; Get time
    ADDI R1, R0, 9      ; syscall 9 = gettime
    SYSCALL
    ADD  R10, R0, R1    ; Save time

    ; Yield
    ADDI R1, R0, 8
    SYSCALL

    J    timer
```

## Code Walkthrough: Kernel Implementation

### Process Creation

```rust
impl Process {
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
            time_quantum: 100,  // 100 cycles default
        }
    }
}
```

### Time Quantum Management

```rust
impl Process {
    pub fn use_quantum(&mut self) -> bool {
        if self.time_quantum > 0 {
            self.time_quantum -= 1;
            self.cpu_time += 1;
            true
        } else {
            false  // Quantum expired
        }
    }

    pub fn reset_quantum(&mut self) {
        self.time_quantum = 100;
    }
}
```

### Scheduler Create Process

```rust
impl Scheduler {
    pub fn create_process(
        &mut self,
        pc: Address,
        base_address: Address,
        memory_size: usize,
    ) -> ProcessId {
        let pid = self.next_pid;
        self.next_pid += 1;

        let process = Process::new(pid, pc, base_address, memory_size);
        self.processes.push(process);
        self.ready_queue.push_back(pid);

        pid
    }
}
```

## Challenge Problems

### Challenge 1: Priority Scheduling

Modify the scheduler to support priorities:
- Higher priority processes run first
- Same priority uses round-robin
- Test with 5 processes of varying priorities

### Challenge 2: Process Statistics

Add statistics tracking:
- Total CPU time per process
- Number of context switches per process
- Average wait time
- Display stats for all processes

### Challenge 3: Sleep System Call

Implement the `sleep` syscall:
- Process sleeps for N cycles
- Moves to Blocked state
- Wakes up after N cycles elapse
- Returns to Ready state

## Summary

In this chapter, you learned:

✅ The kernel is the core of the operating system
✅ Boot sequence initializes the kernel and creates init process
✅ Processes have states (Ready, Running, Blocked, Terminated)
✅ Process Control Blocks (PCBs) store process information
✅ Round-robin scheduling gives each process fair CPU time
✅ Time quantums prevent processes from monopolizing CPU
✅ System calls provide services to user programs
✅ User/kernel mode separation protects the system

## Next Steps

In Chapter 7 (coming soon), we'll explore **File Systems**: how the OS organizes and stores data persistently on disk, implements directories and files, and provides file operations.

## Further Reading

- `crates/vos-kernel/src/process.rs` - Process management
- `crates/vos-kernel/src/scheduler.rs` - Scheduling algorithm
- `crates/vos-kernel/src/syscall.rs` - System call interface
- `crates/vos-kernel/src/boot.rs` - Boot sequence

## Testing Your Understanding

1. What happens when a process's time quantum expires?
2. Why does the scheduler use a ready queue?
3. How does a system call differ from a regular function call?
4. What information is stored in the Process Control Block?
5. Why is user/kernel mode separation important?

**Answers:**
1. Process moves to back of ready queue; next process gets CPU
2. To implement fair round-robin scheduling (FIFO order)
3. Syscall transitions to kernel mode with privilege escalation
4. PID, state, PC, registers, stack pointer, memory info, timing
5. Prevents user programs from crashing system or accessing others' memory
