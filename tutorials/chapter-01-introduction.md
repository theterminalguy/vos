# Chapter 1: Introduction

> **Learning Objectives**
> - Understand what VOS is and why it exists
> - Learn about computer architecture basics
> - Set up your development environment
> - Run your first VOS program

## What is VOS?

VOS (Virtual Operating System) is a complete computer system simulator that runs entirely in software. It includes:

- A **virtual CPU** that executes instructions
- **Virtual memory** (RAM) for storing data
- **I/O devices** like keyboards and displays
- An **operating system kernel** that manages everything
- A **shell** for interacting with the system
- A **programming language** for writing programs

Think of VOS as a computer inside your computer - it's fully functional but designed specifically for learning.

## Why Build a Virtual OS?

Learning operating systems and computer architecture from textbooks alone can be abstract and difficult. VOS makes these concepts concrete by providing:

1. **Working code** - See exactly how things work
2. **Safe experimentation** - Break things without consequences
3. **Complete control** - Modify anything you want
4. **Educational design** - Built for understanding, not performance

You can't easily break a real operating system to see how it works. With VOS, you can!

## How Computers Work (The Big Picture)

Let's start with a simple mental model of a computer:

```
┌─────────────────────────────────────────┐
│              CPU (Processor)            │
│  ┌────────────────────────────────┐    │
│  │  Registers (Fast Storage)      │    │
│  │  - R0, R1, R2, ... (16 total) │    │
│  │  - PC (Program Counter)        │    │
│  └────────────────────────────────┘    │
│                                          │
│  ┌────────────────────────────────┐    │
│  │  ALU (Arithmetic Logic Unit)   │    │
│  │  - Performs calculations        │    │
│  │  - Comparisons                  │    │
│  └────────────────────────────────┘    │
└──────────────┬──────────────────────────┘
               │ Bus
               ├─────────────────┐
               │                 │
      ┌────────▼────────┐  ┌─────▼──────┐
      │   Memory (RAM)  │  │  I/O       │
      │                 │  │  Devices   │
      │  - Programs     │  │            │
      │  - Data         │  │  - Display │
      │                 │  │  - Keyboard│
      └─────────────────┘  └────────────┘
```

### The CPU

The **CPU (Central Processing Unit)** is the brain of the computer. It:

1. **Fetches** instructions from memory
2. **Decodes** them to understand what to do
3. **Executes** the instruction
4. Repeats forever

This is called the **fetch-decode-execute cycle**.

The CPU contains:

- **Registers**: Super-fast storage inside the CPU (like variables)
- **ALU**: Does math and logic (add, subtract, compare, etc.)
- **Control Unit**: Orchestrates everything

### Memory (RAM)

**Memory** stores both programs (instructions) and data. Think of it as a huge array:

```
Address     Data
0x0000      [42]
0x0001      [17]
0x0002      [99]
...
0xFFFF      [13]
```

Each address points to one byte of data. The CPU can:
- **Read** from an address (get the value)
- **Write** to an address (change the value)

### I/O Devices

**I/O (Input/Output) devices** let the computer interact with the world:

- **Input**: Keyboard, mouse, disk
- **Output**: Display, printer, speakers

In VOS, devices are **memory-mapped**, meaning you interact with them by reading and writing to special memory addresses.

### The Operating System

An **operating system (OS)** is a program that:

1. Manages hardware resources (CPU, memory, devices)
2. Provides a nice interface for other programs
3. Keeps programs from interfering with each other
4. Makes programming easier

Examples: Linux, Windows, macOS, or... VOS!

## The VOS Architecture

VOS implements a simple but complete computer architecture:

### CPU Specification

- **32-bit RISC architecture** (simple instructions)
- **16 general-purpose registers** (R0-R15)
- **~30 instructions** (just enough to be useful)
- **Word size**: 32 bits (4 bytes)

### Memory Layout

VOS divides its 4GB address space into regions:

```
0x00000000 - 0x00000FFF   Interrupt Vector Table (4KB)
0x00001000 - 0x000FFFFF   Kernel Code (1MB)
0x00100000 - 0x001FFFFF   Kernel Stack (1MB)
0x00200000 - 0x3FFFFFFF   User Programs (~1GB)
0x40000000 - 0x7FFFFFFF   Heap (1GB)
0x80000000 - 0xBFFFFFFF   Memory-Mapped I/O (1GB)
0xC0000000 - 0xFFFFFFFF   Reserved (1GB)
```

This organization keeps different parts of the system separate and organized.

## Setting Up Your Environment

### Prerequisites

You need Rust installed. If you don't have it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version  # Should show 1.70 or later
cargo --version
```

### Getting VOS

Clone the repository:

```bash
git clone https://github.com/[username]/vos.git
cd vos
```

Build the project:

```bash
cargo build --release
```

This compiles all the VOS components. It may take a few minutes the first time.

### Project Structure

```
vos/
├── crates/           # All the VOS components
│   ├── vos-core/     # Core types and traits
│   ├── vos-cpu/      # CPU emulator
│   ├── vos-memory/   # Memory system
│   ├── vos-kernel/   # OS kernel
│   └── ...
├── tutorials/        # This is where you are now!
├── examples/         # Example programs
└── tests/            # Integration tests
```

## Your First VOS Program

Let's run a simple program to make sure everything works.

### Example: Hello World (Assembly)

Create a file `examples/asm/hello.asm`:

```asm
; Hello World in VOS assembly
; This program prints "Hello, VOS!" to the display

    .text
    .global _start

_start:
    ; Load address of message into R1
    lui R1, hi(message)
    ori R1, R1, lo(message)

    ; Load syscall number (1 = write)
    addi R2, R0, 1

    ; File descriptor (1 = stdout)
    addi R3, R0, 1

    ; Buffer address
    mov R4, R1

    ; Length of message
    addi R5, R0, 12

    ; Make syscall
    syscall

    ; Exit
    addi R2, R0, 0    ; syscall 0 = exit
    addi R3, R0, 0    ; exit code 0
    syscall
    halt

    .data
message:
    .string "Hello, VOS!\n"
```

Don't worry if you don't understand this yet! We'll learn assembly in Chapter 2.

### Running the Program

```bash
# Assemble the program
cargo run --bin vos-cli -- asm examples/asm/hello.asm -o hello.bin

# Run it
cargo run --bin vos-cli -- run hello.bin
```

You should see:

```
Hello, VOS!
```

Congratulations! You just ran your first program on VOS.

## What Just Happened?

Let's break down what occurred:

1. **Assembler** converted assembly code to machine code (binary instructions)
2. **VOS CLI** loaded the program into virtual memory
3. **Virtual CPU** executed the instructions one by one
4. **System call** (syscall) asked the OS to print text
5. **Virtual display** showed the output

All of this happened in software, but it's exactly how a real computer works!

## Understanding the Code

Let's look at vos-core, the foundation of VOS. Open `crates/vos-core/src/types.rs`:

```rust
/// A 32-bit word - the native data type of the VOS CPU.
pub type Word = u32;

/// A memory address in the VOS system.
pub type Address = u32;

/// A single byte of data.
pub type Byte = u8;
```

These are **type aliases** - we give meaningful names to basic types:

- `Word` is a 32-bit unsigned integer (0 to 4,294,967,295)
- `Address` is also 32 bits, giving us 4GB of address space
- `Byte` is 8 bits (0 to 255)

Now look at `AddressRange`:

```rust
pub struct AddressRange {
    pub start: Address,
    pub end: Address,
}

impl AddressRange {
    pub fn contains(&self, address: Address) -> bool {
        address >= self.start && address < self.end
    }
}
```

This represents a range of memory addresses and can check if an address falls within it. Simple but useful!

## Hands-On Exercise

Let's verify VOS is working correctly.

### Exercise 1: Run the Tests

```bash
cargo test -p vos-core
```

You should see all tests pass. These tests verify that the core types work correctly.

### Exercise 2: Explore the Code

1. Open `crates/vos-core/src/types.rs`
2. Find the `memory_regions` module
3. Answer these questions:
   - What address does the kernel code start at?
   - How large is the user space region?
   - What addresses are used for memory-mapped I/O?

<details>
<summary>Click to reveal answers</summary>

- Kernel code starts at `0x00001000`
- User space is `0x3FFFFFFF - 0x00200000 = 0x3FDFFFFF` bytes (~1GB)
- MMIO uses `0x80000000` to `0xBFFFFFFF`
</details>

### Exercise 3: Test Address Ranges

Add a test to `crates/vos-core/src/types.rs`:

```rust
#[test]
fn test_my_first_test() {
    use crate::types::memory_regions;

    // Kernel code region
    assert!(memory_regions::KERNEL_CODE.contains(0x00001000));

    // User space
    assert!(memory_regions::USER_SPACE.contains(0x00200000));

    // This should NOT be in kernel code
    assert!(!memory_regions::KERNEL_CODE.contains(0x80000000));
}
```

Run it:

```bash
cargo test -p vos-core test_my_first_test
```

Congratulations! You just wrote your first VOS test.

## Challenge Problems

Ready for more? Try these:

### Challenge 1: Add a New Memory Region

Add a new memory region called `VIDEO_MEMORY` from `0x80000000` to `0x80100000` (1MB for the display buffer).

1. Add it to the `memory_regions` module
2. Write a test for it
3. Make sure all tests still pass

### Challenge 2: Implement an Address Iterator

Implement an iterator that yields all 4KB-aligned addresses in a range:

```rust
impl AddressRange {
    pub fn aligned_addresses(&self, alignment: usize) -> impl Iterator<Item = Address> {
        // Your code here
    }
}
```

Test it:

```rust
#[test]
fn test_aligned_addresses() {
    let range = AddressRange::new(0x1000, 0x3000);
    let addresses: Vec<_> = range.aligned_addresses(0x1000).collect();
    assert_eq!(addresses, vec![0x1000, 0x2000]);
}
```

### Challenge 3: Research

Read about the Von Neumann architecture and answer:

- What is the key idea of Von Neumann architecture?
- How does VOS follow this architecture?
- What is the alternative (Harvard architecture)?

## Summary

In this chapter, you learned:

- ✅ What VOS is and why it's useful for learning
- ✅ Basic computer architecture (CPU, memory, I/O)
- ✅ How to set up the VOS development environment
- ✅ How to run and test VOS programs
- ✅ The structure of the VOS codebase

## Next Steps

In [Chapter 2: CPU Basics](chapter-02-cpu-basics.md), we'll dive into how the CPU works:

- Registers and the ALU
- The fetch-decode-execute cycle
- Instruction formats
- Writing assembly programs

## Further Reading

- [Computer Systems: A Programmer's Perspective](https://csapp.cs.cmu.edu/)
- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Operating Systems: Three Easy Pieces](https://pages.cs.wisc.edu/~remzi/OSTEP/)
- [MIPS Assembly Language](https://en.wikipedia.org/wiki/MIPS_architecture) (VOS is MIPS-inspired)

---

**[← Back to Tutorials](README.md) | [Next: CPU Basics →](chapter-02-cpu-basics.md)**
