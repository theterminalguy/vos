# VOS - Virtual Operating System

A complete virtual operating system simulator for learning OS and computer architecture concepts.

## Overview

VOS is an educational project that implements a full computer system in software, including:

- **Custom Computer Architecture**: Simple 32-bit RISC CPU with 16 registers
- **Operating System Kernel**: Process management, scheduling, filesystem, system calls
- **System Programs**: Interactive shell, utilities, debugger
- **Programming Language**: vos script - a TypeScript/Ruby-like language
- **Application Programs**: Text editor, simple web browser, and more
- **Comprehensive Tutorials**: 12 chapters covering everything from CPU basics to building applications

## Project Goals

This project is designed to:

1. **Teach** fundamental concepts of operating systems and computer architecture
2. **Demonstrate** how computers work from the ground up
3. **Provide** hands-on learning through working code
4. **Enable** experimentation with OS and language design

## Architecture

### CPU Specification

- 32-bit RISC architecture
- 16 general-purpose registers (R0-R15)
- ~30 core instructions (arithmetic, logic, memory, control flow, system)
- Simple instruction formats (R-type, I-type, J-type)

### Memory Model

- 4GB address space (16MB default RAM)
- Virtual memory with simple paging (4KB pages)
- Memory-mapped I/O
- Kernel/User space separation

### I/O Devices

- Text display (80x25 character mode)
- Keyboard (interrupt-driven)
- Timer (periodic interrupts)
- Disk controller (block I/O)
- Serial port (debugging)

### Operating System

- Process management with round-robin scheduling
- Simple inode-based filesystem (Unix-like)
- ~20-30 system calls
- Interrupt handling
- User/kernel mode separation

### vos script Language

TypeScript/Ruby-like syntax with:
- Strong static typing with type inference
- First-class functions and lambdas
- Structs and methods
- Arrays and control flow
- Compiles to machine code

## Project Structure

```
vos/
├── crates/
│   ├── vos-core/         # Core types and traits
│   ├── vos-cpu/          # CPU emulator
│   ├── vos-memory/       # Memory subsystem
│   ├── vos-io/           # I/O devices
│   ├── vos-hardware/     # Virtual machine integration
│   ├── vos-kernel/       # OS kernel
│   ├── vos-lang/         # vos script compiler
│   ├── vos-asm/          # Assembler
│   ├── vos-userspace/    # Shell and user programs
│   ├── vos-debugger/     # Interactive debugger
│   └── vos-cli/          # Main CLI application
├── tutorials/            # 12 tutorial chapters
├── docs/                 # Architecture documentation
├── examples/             # Example programs
└── tests/                # Integration tests
```

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/[username]/vos.git
cd vos

# Build the project
cargo build --release

# Run the virtual machine
cargo run --release --bin vos-cli
```

### Running Examples

```bash
# Run an assembly program
cargo run --bin vos-cli -- run examples/asm/hello.asm

# Run a vos script program
cargo run --bin vos-cli -- run examples/hello_world.vos

# Start the interactive shell
cargo run --bin vos-cli -- shell

# Use the debugger
cargo run --bin vos-cli -- debug examples/asm/fibonacci.asm
```

## Tutorials

The project includes 12 comprehensive tutorials:

1. **Introduction** - What is VOS, architecture overview, setup
2. **CPU Basics** - Registers, fetch-decode-execute, assembly
3. **Memory** - Addressing, virtual memory, page tables
4. **I/O Devices** - Device drivers, interrupts, keyboard/display
5. **Boot Process** - Bootloader, kernel init, first process
6. **Processes** - Process lifecycle, states, context switching
7. **Scheduling** - Round-robin, priority, time quantum
8. **File Systems** - Inodes, directories, file operations
9. **System Calls** - User/kernel mode, syscall mechanism
10. **Shell** - REPL, command parsing, execution
11. **Programming Language** - vos script, lexing, parsing, compilation
12. **Web Browser** - Application architecture, HTML parser, rendering

Start with [tutorials/chapter-01-introduction.md](tutorials/chapter-01-introduction.md).

## Development

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
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p vos-cpu

# Run with logging
RUST_LOG=debug cargo test
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate documentation for all dependencies
cargo doc --open --document-private-items
```

## Contributing

This is an educational project, and contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass: `cargo test`
5. Ensure code is formatted: `cargo fmt`
6. Ensure no clippy warnings: `cargo clippy`
7. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Operating Systems: Three Easy Pieces](https://pages.cs.wisc.edu/~remzi/OSTEP/)
- [Computer Systems: A Programmer's Perspective](https://csapp.cs.cmu.edu/)
- [Writing an OS in Rust](https://os.phil-opp.com/)

## Acknowledgments

This project is inspired by:
- MIPS architecture
- xv6 operating system (MIT)
- Linux kernel design
- Rust's excellent type system and tooling

## Status

🚧 **Work in Progress** - Currently implementing Phase 0 (Foundation)

- [x] Project structure
- [x] Workspace setup
- [ ] Core types and traits
- [ ] CPU emulator
- [ ] Memory system
- [ ] And much more...

Follow along with development in the [tutorials](tutorials/) directory!
