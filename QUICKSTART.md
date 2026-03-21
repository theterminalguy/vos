# 🚀 VOS Quick Start Guide

**Get up and running with VOS in 5 minutes!**

## What is VOS?

VOS (Virtual Operating System) is a complete computer system built in software. It's perfect for learning how operating systems and computers really work.

Think of it as a real computer inside your computer, with:
- 🧠 A CPU that executes instructions
- 💾 Memory for storing data
- 📁 A filesystem for organizing files
- 💻 A shell for running commands
- 🌐 Even a simple web browser!

## Prerequisites

You need Rust installed. If you don't have it:

```bash
# Install Rust (takes 2 minutes)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Then restart your terminal or run:
source $HOME/.cargo/env

# Verify it worked:
rustc --version
```

## Get VOS Running

### 1. Clone and Build

```bash
# Get the code
git clone https://github.com/theterminalguy/vos.git
cd vos

# Build it (takes 1-2 minutes first time)
cargo build --release

# Verify everything works (236 tests)
cargo test
```

### 2. Start the Shell

```bash
cargo run --release --bin vos-cli
```

You should see:
```
VOS Shell v0.1.0
Type 'help' for available commands, 'exit' to quit.

vos>
```

**🎉 Congratulations! You're now inside VOS!**

## Your First Commands

Try these commands to explore:

```bash
# See all available commands
vos> help

# Where am I?
vos> pwd

# What's in the root directory?
vos> ls /

# Create your own space
vos> mkdir /myspace
vos> cd /myspace
vos> pwd

# Create some files
vos> touch hello.txt
vos> touch notes.txt
vos> ls

# Create nested directories
vos> mkdir projects
vos> mkdir projects/vos-demo
vos> ls projects

# Print something
vos> echo Hello, VOS!

# Exit when done
vos> exit
```

## 🎯 Quick Challenges

Try these to test your understanding:

### Challenge 1: Build Your Directory Tree
Create this structure:
```
/
├── home/
│   └── user/
│       ├── documents/
│       └── projects/
└── tmp/
```

<details>
<summary>Solution (click to expand)</summary>

```bash
vos> mkdir /home
vos> mkdir /home/user
vos> mkdir /home/user/documents
vos> mkdir /home/user/projects
vos> mkdir /tmp
vos> ls /
vos> ls /home/user
```
</details>

### Challenge 2: File Operations
1. Create a file called `readme.txt` in `/home/user/documents`
2. Navigate to that directory
3. List the files to verify it exists

<details>
<summary>Solution</summary>

```bash
vos> touch /home/user/documents/readme.txt
vos> cd /home/user/documents
vos> pwd
vos> ls
```
</details>

## What Just Happened?

When you ran those commands, here's what happened inside VOS:

```
    You typed a command
           ↓
    ┌──────────────┐
    │    Shell     │ ← Parsed your command
    └──────┬───────┘
           ↓
    ┌──────────────┐
    │   Kernel     │ ← Managed the operation
    └──────┬───────┘
           ↓
    ┌──────────────┐
    │ File System  │ ← Stored the file
    └──────┬───────┘
           ↓
    ┌──────────────┐
    │   Memory     │ ← Held the data
    └──────────────┘
```

Every command went through multiple layers, just like in a real OS!

## Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `help` | Show all commands | `help` |
| `pwd` | Print working directory | `pwd` |
| `cd [path]` | Change directory | `cd /home` |
| `ls [path]` | List files | `ls /` |
| `mkdir <path>` | Create directory | `mkdir /docs` |
| `touch <file>` | Create file | `touch file.txt` |
| `cat <file>` | Show file contents | `cat readme.txt` |
| `rm <path>` | Remove file/directory | `rm old.txt` |
| `echo <text>` | Print text | `echo Hello!` |
| `exit` | Quit shell | `exit` |

## What Can You Explore?

### 1. **Run Tests** - See the components in action
```bash
# Test the CPU
cargo test --package vos-cpu

# Test the filesystem
cargo test --package vos-kernel

# Test the HTML parser
cargo test --package vos-userspace test_parse
```

### 2. **Read the Code** - Learn how it works
```bash
# CPU implementation
cat crates/vos-cpu/src/cpu.rs

# Filesystem code
cat crates/vos-kernel/src/fs/vfs.rs

# Shell implementation
cat crates/vos-userspace/src/shell.rs
```

### 3. **Check Out Examples**
```bash
# vos script programs
ls examples/*.vos

# Assembly programs
ls examples/asm/*.asm
```

## Next Steps

Ready to learn more? Choose your path:

### 🎓 **For Beginners** - Start Here!
Interactive, visual, hands-on learning:
- **[Beginner's Guide](docs/beginners-guide/README.md)** ← Start here!
- Learn by doing with interactive challenges
- Lots of diagrams and examples
- No prior CS knowledge needed

### 📚 **For Technical Deep Dive** - CS Students
Comprehensive technical documentation:
- **[Technical Tutorials](tutorials-technical/README.md)**
- Detailed implementation explanations
- Computer architecture concepts
- OS theory and practice

### 🔍 **For Explorers**
- Read the [Architecture Overview](README.md#architecture)
- Explore the [codebase](crates/)
- Try modifying the code!

## Troubleshooting

### Build Fails?
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Want More Output?
```bash
# Run with debug logging
RUST_LOG=debug cargo run --bin vos-cli

# See test output
cargo test -- --nocapture
```

### Shell Crashes?
The shell is pretty robust, but if something goes wrong:
1. Exit and restart: `vos> exit`, then rerun
2. Check the error message - it usually tells you what's wrong
3. Report issues: [GitHub Issues](https://github.com/theterminalguy/vos/issues)

## Tips & Tricks

💡 **Tab Completion**: Not yet implemented, but on the roadmap!

💡 **Command History**: The shell remembers your commands within a session

💡 **Exploring Files**: Use `ls` liberally to see what's where

💡 **Getting Lost**: `pwd` shows where you are, `cd /` goes back to root

💡 **Learning**: Try breaking things! The worst that happens is you restart the shell

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────┐
│                    VOS QUICK REFERENCE                  │
├─────────────────────────────────────────────────────────┤
│  Navigation          │  Files                           │
│  pwd                 │  touch <file>    Create file     │
│  cd <path>           │  cat <file>      Show contents   │
│  ls [path]           │  rm <path>       Delete          │
│                      │                                   │
│  Directories         │  Utilities                       │
│  mkdir <path>        │  echo <text>     Print text      │
│  rm <path>           │  help            Show commands   │
│                      │  exit            Quit shell      │
└─────────────────────────────────────────────────────────┘
```

## What Makes VOS Special?

Unlike other learning tools, VOS is:

✅ **Complete** - Full stack from CPU to applications
✅ **Working** - 236 tests, real implementation
✅ **Educational** - Built specifically for learning
✅ **Accessible** - Well-documented and beginner-friendly
✅ **Hackable** - Modify and experiment freely

## Get Help

- 📖 **Beginner's Guide**: [docs/beginners-guide/](docs/beginners-guide/)
- 📚 **Technical Docs**: [tutorials-technical/](tutorials-technical/)
- 💬 **Issues**: [GitHub Issues](https://github.com/theterminalguy/vos/issues)
- 🌟 **Star the repo**: Help others find VOS!

---

**Ready to dive deeper?** → [Beginner's Guide - Chapter 1: Hello VOS!](docs/beginners-guide/01-hello-vos.md)

**Want technical details?** → [Technical Tutorial - Chapter 1: Introduction](tutorials-technical/chapter-01-introduction.md)

---

*Built with ❤️ and Rust 🦀 • Making OS concepts accessible to everyone*
