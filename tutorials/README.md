# VOS Tutorials

Welcome to the VOS tutorial series! These 12 chapters will guide you through understanding how computers and operating systems work from the ground up.

## Prerequisites

- Basic programming knowledge (any language)
- Familiarity with command-line interfaces
- Curiosity about how computers work!

No prior knowledge of operating systems, computer architecture, or Rust is required. We'll teach you everything you need as we go.

## Tutorial Structure

Each tutorial chapter includes:

1. **Learning Objectives** - What you'll learn
2. **Concepts** - Theory and explanations
3. **Code Walkthrough** - Understanding the implementation
4. **Hands-On Exercise** - Practice what you learned
5. **Challenge Problems** - Optional advanced exercises
6. **Further Reading** - Resources to dive deeper

## Chapters

### Part 1: Hardware Foundations

1. **[Introduction](chapter-01-introduction.md)**
   - What is VOS?
   - Computer architecture overview
   - Setting up your environment
   - Running your first program

2. **[CPU Basics](chapter-02-cpu-basics.md)**
   - Registers and the ALU
   - Fetch-decode-execute cycle
   - Instruction formats
   - Writing assembly code

3. **[Memory](chapter-03-memory.md)**
   - RAM and address spaces
   - Virtual memory concepts
   - Page tables and translation
   - Memory protection

4. **[I/O Devices](chapter-04-io-devices.md)**
   - Memory-mapped I/O
   - Device drivers
   - Interrupts and polling
   - Keyboard and display

### Part 2: Operating System Fundamentals

5. **[Boot Process](chapter-05-boot-process.md)**
   - System initialization
   - Bootloader design
   - Kernel loading
   - Starting the first process

6. **[Processes](chapter-06-processes.md)**
   - Process abstraction
   - Process Control Blocks (PCBs)
   - Process states and lifecycle
   - Context switching

7. **[Scheduling](chapter-07-scheduling.md)**
   - CPU scheduling algorithms
   - Round-robin scheduling
   - Priority scheduling
   - Performance metrics

8. **[File Systems](chapter-08-file-systems.md)**
   - File system abstraction
   - Inodes and directories
   - File operations
   - Disk I/O

9. **[System Calls](chapter-09-system-calls.md)**
   - User mode vs kernel mode
   - System call mechanism
   - Implementing syscalls
   - Standard I/O

### Part 3: High-Level Software

10. **[Shell](chapter-10-shell.md)**
    - Command-line interfaces
    - REPL design
    - Command parsing and execution
    - Built-in commands

11. **[Programming Language](chapter-11-programming-language.md)**
    - Language design choices
    - Lexing and parsing
    - Type checking
    - Code generation

12. **[Applications](chapter-12-applications.md)**
    - Application architecture
    - Simple web browser
    - Text editor
    - Putting it all together

## How to Use These Tutorials

### For Learners

If you're here to learn about operating systems:

1. **Start from Chapter 1** - The tutorials build on each other
2. **Read the code** - Understanding comes from seeing how it works
3. **Do the exercises** - Hands-on practice is essential
4. **Experiment** - Modify the code and see what happens
5. **Take your time** - There's a lot of material here

### For Educators

If you're teaching:

- Each chapter can be 1-2 class sessions
- Exercises make good homework assignments
- Challenge problems are suitable for projects
- Code is well-documented for self-study

### For Implementers

If you're building your own OS:

- Use VOS as a reference implementation
- The architecture is designed to be understandable
- Code is modular and well-tested
- Extend or modify to explore ideas

## Estimated Time

- **Quick read** (just reading): ~10-15 hours
- **With exercises**: ~30-40 hours
- **With challenges**: ~50-60 hours
- **Building from scratch**: 100+ hours

Don't rush! The goal is understanding, not speed.

## Getting Help

If you get stuck:

1. Re-read the relevant chapter section
2. Check the code documentation (`cargo doc --open`)
3. Run the tests to see examples (`cargo test`)
4. Open an issue on GitHub
5. Join our community discussions

## Contributing

Found an error or have a suggestion?

- Submit a pull request
- Open an issue
- Suggest improvements

All contributions welcome!

## Let's Begin!

Ready to understand how computers really work? Start with [Chapter 1: Introduction](chapter-01-introduction.md)!
