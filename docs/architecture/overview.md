# VOS Architecture Overview

This document provides a high-level overview of the VOS (Virtual Operating System) architecture.

## System Architecture

VOS simulates a complete computer system with the following major components:

```
┌─────────────────────────────────────────────────────────────┐
│                     VOS System Stack                         │
├─────────────────────────────────────────────────────────────┤
│  Applications (vos-userspace)                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │  Shell   │  │  Browser │  │  Editor  │  ...              │
│  └──────────┘  └──────────┘  └──────────┘                  │
├─────────────────────────────────────────────────────────────┤
│  Programming Language (vos-lang)                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  vos script: Lexer → Parser → Type Checker → Codegen │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Operating System Kernel (vos-kernel)                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Process  │  │ Scheduler│  │   File   │  │ Syscalls │  │
│  │   Mgmt   │  │          │  │  System  │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Hardware Abstraction (vos-hardware)                         │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          Virtual Machine Integration Layer            │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Hardware Components                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────────┐ │
│  │   CPU    │  │  Memory  │  │  I/O Devices             │ │
│  │(vos-cpu) │  │(vos-mem) │  │  (vos-io)                │ │
│  └──────────┘  └──────────┘  └──────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Core Abstractions (vos-core)                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Types, Traits, Error Handling                        │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

         ┌──────────────┐           ┌──────────────┐
         │  Assembler   │           │   Debugger   │
         │  (vos-asm)   │           │ (vos-debugger)│
         └──────────────┘           └──────────────┘
              Tools                      Tools
```

## Component Details

### 1. vos-core: Foundation Layer

The foundation of the entire system, providing:

- **Core Types**: `Word`, `Address`, `Byte`, `AddressRange`
- **Error Types**: Comprehensive error handling
- **Traits**: Common interfaces (`Device`, `Executable`, `Clockable`, `Inspectable`)
- **Memory Layout**: Definitions for system memory regions

**Key Design Decisions:**
- 32-bit words for simplicity
- Byte-addressable memory
- Memory-mapped I/O model
- Strongly typed errors with context

### 2. vos-cpu: Processor Emulation

Simulates a 32-bit RISC CPU with:

- **Registers**: 16 general-purpose registers
  - R0: Always zero
  - R1-R14: General purpose
  - R15: Stack pointer (by convention)
- **Special Registers**: PC (Program Counter), FLAGS, IR (Instruction Register)
- **ALU**: Arithmetic and logic operations
- **Instruction Formats**:
  - R-Type: Register operations
  - I-Type: Immediate operations
  - J-Type: Jump operations

**Instruction Set** (~30 instructions):
- Arithmetic: ADD, SUB, MUL, DIV
- Logic: AND, OR, XOR, NOT
- Shifts: SLL, SRL, SRA
- Memory: LW, SW, LB, SB
- Control: BEQ, BNE, BLT, BGT, J, JAL, JR
- System: SYSCALL, BREAK, HALT, NOP

**Fetch-Decode-Execute Cycle:**
```
1. Fetch: Read instruction from memory at PC
2. Decode: Parse opcode and operands
3. Execute: Perform operation
4. Update: Increment PC, update registers
5. Repeat
```

### 3. vos-memory: Memory Subsystem

Implements the memory hierarchy:

- **RAM**: Physical memory storage
- **MMU**: Memory Management Unit
  - Virtual to physical address translation
  - Simple paging (4KB pages)
  - Page tables
  - Protection bits (read/write/execute)

**Memory Layout** (4GB address space):
```
0x00000000 - 0x00000FFF   Interrupt Vector Table (4KB)
0x00001000 - 0x000FFFFF   Kernel Code + Data (~1MB)
0x00100000 - 0x001FFFFF   Kernel Stack (1MB)
0x00200000 - 0x3FFFFFFF   User Space (~1GB)
0x40000000 - 0x7FFFFFFF   Heap (1GB)
0x80000000 - 0xBFFFFFFF   Memory-Mapped I/O (1GB)
0xC0000000 - 0xFFFFFFFF   Reserved (1GB)
```

**Key Features:**
- Byte-addressable (can read/write individual bytes)
- Word-aligned access for performance
- Bounds checking for safety
- Memory-mapped I/O region

### 4. vos-io: I/O Devices

All devices implement the `Device` trait for memory-mapped I/O:

**Device Roster:**
1. **Display** (0x80000000 - 0x80001F40)
   - Text mode: 80×25 characters
   - Character + attribute pairs
   - Direct memory access

2. **Keyboard** (0x80002000 - 0x80002010)
   - Input buffer (queue)
   - Interrupt-driven
   - Scancode translation

3. **Timer** (0x80003000 - 0x80003010)
   - Configurable frequency
   - Periodic interrupts
   - Counter register

4. **Disk** (0x80004000 - 0x80004100)
   - Block device (512-byte sectors)
   - DMA controller
   - Command/status registers

5. **Serial Port** (0x80005000 - 0x80005010)
   - Debugging output
   - Byte-at-a-time I/O

**Interrupt System:**
- 256 interrupt vectors
- Hardware interrupts: 0-31
- Software interrupts (syscalls): 32-255

### 5. vos-hardware: Virtual Machine

Integrates all hardware components:

```rust
struct VirtualMachine {
    cpu: CPU,
    memory: Memory,
    devices: Vec<Box<dyn Device>>,
    interrupt_controller: InterruptController,
}
```

Provides high-level operations:
- Load programs into memory
- Execute until halt or error
- Handle interrupts
- Manage device I/O

### 6. vos-kernel: Operating System

Provides OS functionality:

**Process Management:**
- Process Control Blocks (PCBs)
- Process states: Ready, Running, Blocked, Terminated
- Context switching
- Process creation and destruction

**Scheduler:**
- Round-robin scheduling (configurable quantum)
- Ready queue management
- CPU time accounting

**File System:**
- Simple inode-based design (Unix-like)
- Directories: /, /bin, /home, /dev
- File operations: open, read, write, close
- Directory operations: mkdir, readdir

**System Calls** (~20-30 core syscalls):
- Process: fork, exec, exit, wait, getpid
- File: open, close, read, write, seek
- Directory: mkdir, rmdir, readdir
- Memory: brk, mmap
- Time: time, sleep

### 7. vos-lang: Programming Language

The vos script language with TypeScript/Ruby-like syntax:

**Pipeline:**
```
Source Code
    ↓
Lexer (Tokenization)
    ↓
Parser (AST Generation)
    ↓
Type Checker
    ↓
Compiler/Codegen
    ↓
Machine Code
```

**Type System:**
- Primitives: int, float, bool, char, string
- Compound: Arrays, Tuples, Structs
- Functions: First-class, closures
- Type inference

**Example Code:**
```typescript
fn factorial(n: int) -> int {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}

struct Point {
    x: int
    y: int

    fn distance(self) -> float {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

let numbers = [1, 2, 3, 4, 5]
for num in numbers {
    print(num)
}
```

### 8. vos-asm: Assembler

Converts assembly code to machine code:

**Assembly Syntax:**
```asm
    .text
    .global _start

_start:
    addi R1, R0, 10      ; R1 = 10
    addi R2, R0, 20      ; R2 = 20
    add  R3, R1, R2      ; R3 = R1 + R2
    halt

    .data
message:
    .string "Hello!"
```

**Features:**
- Labels and symbols
- Directives (.text, .data, .global)
- Pseudo-instructions (expanded to real instructions)
- Immediate values and addressing modes

### 9. vos-debugger: Interactive Debugger

Debugging tools:

**Features:**
- Step execution (single instruction)
- Breakpoints (address-based)
- Register inspection
- Memory inspection (hex dump)
- Disassembly
- Execution history

**Commands:**
```
step           - Execute one instruction
continue       - Run until breakpoint
break 0x1000   - Set breakpoint
print R1       - Print register value
mem 0x1000 64  - Dump 64 bytes of memory
backtrace      - Show call stack
```

### 10. vos-userspace: User Programs

Application layer:

**Shell:**
- REPL (Read-Eval-Print Loop)
- Command parsing
- Built-in commands: cd, pwd, ls, cat, echo
- Program execution

**Utilities:**
- File management: cp, mv, rm
- Text processing: grep, sort
- System info: ps, top

**Applications:**
- Text editor
- Simple web browser (HTML parser, renderer)
- Calculator
- Games (optional)

### 11. vos-cli: Command-Line Interface

Main entry point for interacting with VOS:

**Commands:**
```bash
vos-cli run program.bin           # Run program
vos-cli asm program.asm           # Assemble code
vos-cli debug program.bin         # Debug program
vos-cli shell                     # Start shell
vos-cli compile program.vos       # Compile vos script
```

## Data Flow Examples

### Example 1: Running a Program

```
1. User: vos-cli run hello.bin
2. CLI loads binary into vos-hardware VM
3. VM initializes CPU, memory, devices
4. CPU starts fetch-decode-execute loop
5. Program executes, makes syscall (write)
6. Kernel handles syscall, writes to display device
7. Display device updates character buffer
8. CLI reads display buffer and prints to terminal
9. Program exits (HALT instruction)
10. VM returns control to CLI
```

### Example 2: Keyboard Input

```
1. User presses key in terminal
2. CLI captures input, passes to VM
3. VM triggers keyboard device interrupt (IRQ 1)
4. CPU saves state, jumps to interrupt handler
5. Kernel interrupt handler reads scancode from keyboard
6. Kernel translates scancode to character
7. Kernel places character in input buffer
8. Kernel signals waiting process (if any)
9. Process reads character via read() syscall
10. CPU returns from interrupt
```

### Example 3: File System Operation

```
1. Program calls open("/home/user/file.txt", READ)
2. Syscall instruction triggers trap to kernel
3. Kernel syscall handler validates arguments
4. Kernel looks up file in directory structure
5. Kernel reads inode from disk (via disk device)
6. Kernel creates file descriptor entry
7. Kernel returns file descriptor to program
8. Program calls read(fd, buffer, size)
9. Kernel reads data from disk into buffer
10. Kernel copies data to user space
11. Returns number of bytes read
```

## Design Principles

### Educational First

VOS prioritizes understanding over performance:
- Simple, clear implementations
- Well-commented code
- Comprehensive documentation
- Tests as examples

### Modularity

Each component is independent:
- Clear interfaces (traits)
- Minimal dependencies
- Easy to test in isolation
- Can be understood separately

### Correctness

Safety and correctness matter:
- Strong type system (Rust)
- Comprehensive error handling
- Extensive testing
- Defensive programming

### Realism

VOS models real systems:
- Based on actual architectures (MIPS-inspired)
- Real OS concepts (processes, scheduling, file systems)
- Industry-standard designs
- Teaches transferable knowledge

## Performance Considerations

VOS is **not** designed for performance, but rather for:
- **Clarity**: Code should be easy to read
- **Simplicity**: Avoid complex optimizations
- **Correctness**: Safe, correct behavior first

Expected performance:
- ~10K-100K instructions/second (depends on host)
- Sufficient for educational purposes
- Can run simple programs interactively

## Future Extensions

Potential areas for expansion:

1. **Networking**: TCP/IP stack, socket API
2. **Graphics**: Pixel-based display mode
3. **Sound**: Audio device and synthesis
4. **Multicore**: SMP support, locks, threading
5. **Advanced FS**: Journaling, permissions, links
6. **Virtual Memory**: Demand paging, swap
7. **Security**: Capabilities, sandboxing
8. **Performance**: JIT compilation for vos script

## Learning Path

Recommended order to understand VOS:

1. Start with vos-core (types, traits)
2. Understand vos-cpu (how instructions work)
3. Study vos-memory (addressing, paging)
4. Explore vos-io (devices, interrupts)
5. Examine vos-kernel (processes, syscalls)
6. Try vos-asm (write assembly programs)
7. Use vos-debugger (step through execution)
8. Learn vos-lang (high-level programming)
9. Write vos-userspace programs
10. Build your own extensions!

## Conclusion

VOS is a complete, working computer system implemented in software. Every component is designed to teach real concepts while remaining accessible to learners.

The architecture follows industry-standard designs (RISC CPU, Unix-like OS) but simplifies them enough to understand in full. By studying VOS, you gain deep knowledge of how computers actually work from the transistors up to applications.

---

For more details, see:
- [CPU Design](cpu-design.md)
- [Memory Model](memory-model.md)
- [Instruction Set](instruction-set.md)
