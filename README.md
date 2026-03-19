# VOS - Virtual Operating System

A complete virtual operating system simulator for learning OS and computer architecture concepts.

![Status](https://img.shields.io/badge/status-complete-success)
![Tests](https://img.shields.io/badge/tests-236%20passing-success)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)

## Overview

VOS is a comprehensive educational project that implements a full computer system in software, from CPU to applications. Built entirely in Rust, it provides a working implementation of:

- **Custom Computer Architecture**: Simple 32-bit RISC CPU with 16 registers
- **Operating System Kernel**: Process management, scheduling, filesystem, system calls
- **System Programs**: Interactive shell, assembler, debugger
- **Programming Language**: vos script - a TypeScript/Ruby-like language
- **Application Programs**: Calculator, web browser with HTML parser
- **Comprehensive Tutorials**: 10 detailed chapters covering everything

## 🎯 Project Goals

This project is designed to:

1. **Teach** fundamental concepts of operating systems and computer architecture
2. **Demonstrate** how computers work from the ground up
3. **Provide** hands-on learning through working, tested code
4. **Enable** experimentation with OS and language design

Perfect for computer science students, self-learners, and anyone curious about how operating systems work!

## ✨ Features

### Complete Implementation

- ✅ **CPU Emulator** - 32-bit RISC architecture with ALU and 16 registers
- ✅ **Memory Management** - Virtual memory with paging (4KB pages)
- ✅ **I/O System** - Display, keyboard, timer devices
- ✅ **File System** - Inode-based Unix-style filesystem
- ✅ **Process Management** - PCB, context switching, round-robin scheduler
- ✅ **System Calls** - 10 core syscalls (exit, fork, read, write, etc.)
- ✅ **Interactive Shell** - 10 built-in commands (cd, ls, cat, mkdir, etc.)
- ✅ **Assembler** - Full assembly language support
- ✅ **Debugger** - Interactive debugger with breakpoints
- ✅ **Programming Language** - vos script lexer and AST
- ✅ **Web Browser** - HTML parser and text renderer

### Quality Metrics

- **236 tests** across all components
- **Zero clippy warnings**
- **~15,000+ lines** of well-documented Rust code
- **10 comprehensive tutorials** (~8,000+ lines of documentation)
- **6 example programs** demonstrating language features

## 🏗️ Architecture

### CPU Specification

- **Architecture**: 32-bit RISC
- **Registers**: 16 general-purpose (R0-R15), PC, FLAGS
- **Instructions**: ~30 core instructions
  - Arithmetic: ADD, SUB, MUL, DIV
  - Logic: AND, OR, XOR, NOT
  - Memory: LW, SW, LB, SB
  - Control: BEQ, BNE, J, JAL, JR
  - System: SYSCALL, HALT
- **Instruction Formats**: R-type, I-type, J-type

### Memory Model

```
0x00000000 - 0x00000FFF  (4KB)     Interrupt Vector Table
0x00001000 - 0x000FFFFF  (1020KB)  Kernel Code + Data
0x00100000 - 0x001FFFFF  (1MB)     Kernel Stack
0x00200000 - 0x3FFFFFFF  (~1GB)    User Space
0x40000000 - 0x7FFFFFFF  (1GB)     Heap
0x80000000 - 0xBFFFFFFF  (1GB)     Memory-Mapped I/O
```

- Virtual memory with simple paging
- 4KB page size
- Kernel/User mode separation

### I/O Devices

- **Display**: Text mode (80x25 characters)
- **Keyboard**: Interrupt-driven input queue
- **Timer**: Periodic interrupts for scheduling
- **Disk**: Block I/O (512-byte sectors)
- **Serial**: Debug output

### Operating System

- **Process Management**: PCB, states (Ready, Running, Blocked, Terminated)
- **Scheduler**: Round-robin with configurable time quantum
- **File System**: Inode-based with directories
- **System Calls**: 10 core syscalls
- **Boot Sequence**: Kernel initialization and init process

### vos script Language

TypeScript/Ruby-inspired syntax:

```vos
// Variables and types
let x: int = 42
let name = "VOS"  // Type inference
const PI = 3.14159

// Functions
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

// Control flow
while i < 10 {
    print(i)
    i = i + 1
}
```

Features:
- Static typing with inference
- First-class functions
- Structs and methods
- Arrays and control flow

## 📁 Project Structure

```
vos/
├── crates/                    # Rust workspace crates
│   ├── vos-core/              # Core types, traits, error handling
│   ├── vos-cpu/               # CPU emulator (42 tests)
│   ├── vos-memory/            # Memory and MMU (29 tests)
│   ├── vos-io/                # I/O devices (26 tests)
│   ├── vos-hardware/          # Virtual machine (9 tests)
│   ├── vos-kernel/            # OS kernel (63 tests)
│   ├── vos-asm/               # Assembler (20 tests)
│   ├── vos-debugger/          # Interactive debugger (6 tests)
│   ├── vos-lang/              # vos script language (9 tests)
│   ├── vos-userspace/         # Shell and programs (20 tests)
│   └── vos-cli/               # Main CLI application
├── tutorials/                 # Tutorial chapters
│   ├── chapter-01-introduction.md
│   ├── chapter-02-cpu-basics.md
│   ├── chapter-03-memory.md
│   ├── chapter-04-io-devices.md
│   ├── chapter-05-assembler-debugger.md
│   ├── chapter-06-kernel-fundamentals.md
│   ├── chapter-07-filesystem.md
│   ├── chapter-08-shell.md
│   ├── chapter-09-vos-script.md
│   └── chapter-10-applications.md
├── examples/                  # Example programs
│   ├── asm/                   # Assembly examples
│   │   ├── hello.asm
│   │   ├── fibonacci.asm
│   │   └── countdown.asm
│   ├── hello_world.vos        # vos script examples
│   ├── fibonacci.vos
│   ├── calculator.vos
│   ├── loops.vos
│   └── variables.vos
└── docs/                      # Additional documentation
```

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs/))
- Cargo (comes with Rust)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/theterminalguy/vos.git
cd vos

# Build the project
cargo build --release

# Run all tests
cargo test

# Start the interactive shell
cargo run --release --bin vos-cli
```

### Running Examples

```bash
# Tokenize a vos script program
cargo run --bin vos-cli -- examples/fibonacci.vos

# View all example programs
ls examples/
ls examples/asm/
```

### Using the Shell

```bash
cargo run --bin vos-cli
```

```
VOS Shell v0.1.0
Type 'help' for available commands, 'exit' to quit.

vos> help
VOS Shell - Available Commands:
  help, exit, pwd, cd, ls, cat, echo, mkdir, touch, rm

vos> mkdir /mydir
vos> touch /mydir/file.txt
vos> ls /mydir
file.txt  (inode 2)

vos> exit
```

## 📚 Tutorials

The project includes 10 comprehensive tutorial chapters (~8,000 lines):

| Chapter | Topic | Lines |
|---------|-------|-------|
| 1 | Introduction | ~400 |
| 2 | CPU Basics | ~650 |
| 3 | Memory Systems | ~650 |
| 4 | I/O Devices | ~650 |
| 5 | Assembler & Debugger | ~650 |
| 6 | Kernel Fundamentals | ~700 |
| 7 | File Systems | ~910 |
| 8 | Shell & Userspace | ~913 |
| 9 | vos script Language | ~801 |
| 10 | Building Applications | ~794 |

**Start here:** [tutorials/chapter-01-introduction.md](tutorials/chapter-01-introduction.md)

Each chapter includes:
- Learning objectives
- Concept explanations
- Code walkthroughs
- Hands-on exercises
- Challenge problems

## 🧪 Development

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p vos-cpu

# Build with optimizations
cargo build --release
```

### Testing

```bash
# Run all tests (236 tests)
cargo test

# Run tests for specific crate
cargo test -p vos-kernel

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --open
```

## 🎓 Learning Path

1. **Start with Tutorials**: Read Chapter 1 for overview
2. **Follow Along**: Chapters 2-4 cover hardware (CPU, Memory, I/O)
3. **Build Understanding**: Chapters 5-7 cover system software
4. **Explore Applications**: Chapters 8-10 cover userspace
5. **Experiment**: Modify the code, add features, break things!

## 📊 Project Statistics

- **Total Lines of Code**: ~15,000+ (Rust)
- **Total Tests**: 236
- **Test Coverage**: Comprehensive unit and integration tests
- **Documentation**: ~8,000+ lines of tutorials
- **Crates**: 11 workspace crates
- **Example Programs**: 6 programs (3 assembly, 6 vos script)
- **Clippy Warnings**: 0
- **Development Time**: 4-6 months
- **Educational Value**: Priceless! 🎓

## 🏆 What You'll Learn

By exploring VOS, you'll understand:

### Computer Architecture
- How CPUs execute instructions
- Register files and ALU operations
- Memory addressing and virtual memory
- Interrupt handling
- Device I/O

### Operating Systems
- Process management and scheduling
- System calls and kernel/user mode
- File systems and inodes
- Memory paging
- Boot process

### Compilers & Languages
- Lexical analysis (tokenization)
- Parsing and AST construction
- Assembly language
- Type systems

### Software Engineering
- Rust programming
- Test-driven development
- Error handling
- API design
- Documentation

## 🔮 Future Enhancements

Possible extensions (PRs welcome!):

- [ ] Complete vos script parser and interpreter
- [ ] Network stack (TCP/IP)
- [ ] Multi-core CPU support
- [ ] Graphical display mode
- [ ] More system calls
- [ ] Persistent disk storage
- [ ] Shell pipes and redirection
- [ ] Package manager
- [ ] More example applications

## 🤝 Contributing

Contributions are welcome! This is an educational project, so clarity and documentation are as important as functionality.

**How to contribute:**

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for your changes
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Check for warnings: `cargo clippy`
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

**Guidelines:**
- Keep educational value in mind
- Add tests for new features
- Update documentation/tutorials if needed
- Follow Rust best practices
- Write clear commit messages

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

This project is inspired by:

- **MIPS architecture** - Simple, clean RISC design
- **xv6 operating system (MIT)** - Educational Unix-like OS
- **Linux kernel** - Real-world OS design patterns
- **Rust** - Excellent type system and tooling
- **"Operating Systems: Three Easy Pieces"** - Great OS textbook
- **"Crafting Interpreters"** - Language implementation guide

## 📖 Resources

### Operating Systems
- [Operating Systems: Three Easy Pieces](https://pages.cs.wisc.edu/~remzi/OSTEP/)
- [xv6: A simple Unix-like OS](https://pdos.csail.mit.edu/6.828/2020/xv6.html)
- [OSDev Wiki](https://wiki.osdev.org/)

### Computer Architecture
- [Computer Systems: A Programmer's Perspective](https://csapp.cs.cmu.edu/)
- [MIPS Reference](https://www.mips.com/)

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Writing an OS in Rust](https://os.phil-opp.com/)

### Compilers
- [Crafting Interpreters](https://craftinginterpreters.com/)
- [Engineering a Compiler](https://www.elsevier.com/books/engineering-a-compiler/cooper/978-0-12-088478-0)

## ⭐ Star History

If you find this project helpful, please consider giving it a star! ⭐

It helps others discover the project and motivates continued development.

## 📬 Contact

- **GitHub**: [@theterminalguy](https://github.com/theterminalguy)
- **Issues**: [GitHub Issues](https://github.com/theterminalguy/vos/issues)

## 🎉 Status

**✅ Project Complete!** - All 10 phases implemented and documented.

VOS is a fully functional virtual operating system with:
- Working CPU, memory, and I/O
- Complete kernel with processes and filesystem
- Interactive shell and utilities
- Programming language foundation
- Example applications
- Comprehensive tutorials

Start learning today with [Chapter 1: Introduction](tutorials/chapter-01-introduction.md)!

---

**Built with ❤️ and Rust 🦀**

*Making operating systems accessible to everyone, one instruction at a time.*
