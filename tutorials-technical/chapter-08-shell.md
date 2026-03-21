# Chapter 8: Shell and User Programs

## Learning Objectives

After completing this chapter, you will understand:
- What a shell is and why it's important
- How to build a REPL (Read-Eval-Print Loop)
- Command parsing and execution
- Built-in commands vs external programs
- Command history and line editing
- How the shell interacts with the filesystem

## Introduction

The **shell** is the user's primary interface to an operating system. It's a command-line interpreter that reads user commands, executes them, and displays results. While modern systems often have graphical interfaces, the shell remains essential for:

- System administration
- Automation (shell scripts)
- Remote access (SSH)
- Power user workflows
- Development and debugging

In this chapter, we'll build VOS's interactive shell with familiar Unix-like commands.

## What is a Shell?

A shell is a program that:
1. **Reads** commands from the user
2. **Parses** the command line into components
3. **Executes** the command (built-in or external program)
4. **Displays** the output
5. **Repeats** (the "loop" in REPL)

### Shell vs Kernel

```
┌─────────────────────────────────────┐
│            User                     │
└─────────────────────────────────────┘
                │
                │ Types commands
                ▼
┌─────────────────────────────────────┐
│          Shell (vos-cli)            │  ← Command interpreter
│   - Parse commands                  │
│   - Execute built-ins               │
│   - Launch programs                 │
└─────────────────────────────────────┘
                │
                │ System calls
                ▼
┌─────────────────────────────────────┐
│      Kernel (vos-kernel)            │  ← OS core
│   - Process management              │
│   - File system                     │
│   - System calls                    │
└─────────────────────────────────────┘
```

The shell is **userspace** software. It uses the kernel's services (filesystem, processes) but doesn't run with kernel privileges.

## Shell Architecture

### Shell Structure

```rust
pub struct Shell {
    vfs: Vfs,                  // Filesystem access
    history: Vec<String>,      // Command history
    should_exit: bool,         // Exit flag
}
```

The shell maintains:
- **VFS**: Access to the filesystem
- **History**: Previous commands for recall
- **State**: Whether to exit

### The REPL Loop

```rust
loop {
    // 1. Read: Display prompt and get input
    print!("vos> ");
    let input = read_line();

    // 2. Eval: Execute the command
    match shell.execute(input) {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    // 3. Print: Output was already displayed above

    // 4. Loop: Check exit condition
    if shell.should_exit() {
        break;
    }
}
```

This is the classic REPL pattern: **R**ead-**E**val-**P**rint-**L**oop.

## Command Parsing

When the user types:
```
mkdir /home/user/documents
```

The shell must parse this into:
- **Command**: `mkdir`
- **Arguments**: `["/home/user/documents"]`

### Simple Parsing

```rust
pub fn execute(&mut self, line: &str) -> Result<String> {
    let line = line.trim();

    // Skip empty lines
    if line.is_empty() {
        return Ok(String::new());
    }

    // Split by whitespace
    let parts: Vec<&str> = line.split_whitespace().collect();

    let command = parts[0];
    let args = &parts[1..];

    // Execute based on command name
    match command {
        "mkdir" => self.cmd_mkdir(args),
        "ls" => self.cmd_ls(args),
        // ... other commands
        _ => Ok(format!("command not found: {}", command)),
    }
}
```

This simple parser:
1. Trims whitespace
2. Splits by whitespace
3. First word is the command
4. Rest are arguments

### Limitations

Our simple parser doesn't handle:
- Quoted strings: `echo "hello world"` (treated as two args)
- Escape sequences: `echo hello\ world`
- Pipes: `cat file | grep pattern`
- Redirection: `echo hello > file`
- Background jobs: `sleep 10 &`

These features would require a more sophisticated parser. For now, simplicity is key!

## Built-in Commands

**Built-in commands** are implemented inside the shell itself. They don't launch separate programs. Examples: `cd`, `pwd`, `exit`.

### Why Built-ins?

Some commands **must** be built-ins:
- **cd**: Changes the shell's current directory (can't be external)
- **exit**: Terminates the shell itself
- **export**: Sets shell environment variables

Others are built-ins for efficiency:
- **echo**: Faster than launching a program
- **pwd**: Simple state read

### Implementing Built-ins

#### Exit Command

```rust
fn cmd_exit(&mut self, _args: &[&str]) -> Result<String> {
    self.should_exit = true;
    Ok("Goodbye!".to_string())
}
```

Sets the exit flag. The REPL checks this and breaks the loop.

#### PWD Command

```rust
fn cmd_pwd(&self, _args: &[&str]) -> Result<String> {
    Ok(format!("/ (inode {})", self.vfs.getcwd()))
}
```

Returns the current working directory. In our implementation, we show the inode number. A real shell would track the full path string.

#### CD Command

```rust
fn cmd_cd(&mut self, args: &[&str]) -> Result<String> {
    if args.is_empty() {
        // Go to root
        self.vfs.chdir("/")?;
    } else {
        self.vfs.chdir(args[0])?;
    }
    Ok(String::new())
}
```

Changes the shell's current directory using the VFS.

#### Echo Command

```rust
fn cmd_echo(&self, args: &[&str]) -> Result<String> {
    Ok(args.join(" "))
}
```

Simply joins arguments with spaces and returns them.

#### Help Command

```rust
fn cmd_help(&self, _args: &[&str]) -> Result<String> {
    let help = r#"VOS Shell - Available Commands:

  help         Display this help message
  exit         Exit the shell
  pwd          Print working directory
  cd [PATH]    Change directory
  ls [PATH]    List directory contents
  cat FILE     Display file contents
  echo [ARGS]  Echo arguments to output
  mkdir PATH   Create directory
  touch FILE   Create empty file
  rm PATH      Remove file or directory
"#;
    Ok(help.to_string())
}
```

Returns help text describing available commands.

## Filesystem Commands

These commands interact with the VFS to manipulate files and directories.

### List Directory (ls)

```rust
fn cmd_ls(&self, args: &[&str]) -> Result<String> {
    let path = if args.is_empty() { "/" } else { args[0] };

    let entries = self.vfs.readdir(path)?;

    let mut output = String::new();
    for (name, inode) in entries {
        // Skip . and .. for cleaner output
        if name != "." && name != ".." {
            output.push_str(&format!("{}  (inode {})\n", name, inode));
        }
    }

    Ok(output)
}
```

**Steps:**
1. Get path (default to "/")
2. Call VFS `readdir`
3. Format entries as string
4. Return output

**Example:**
```
vos> ls /
mydir  (inode 2)
test.txt  (inode 3)
```

### Make Directory (mkdir)

```rust
fn cmd_mkdir(&mut self, args: &[&str]) -> Result<String> {
    if args.is_empty() {
        return Ok("mkdir: missing operand".to_string());
    }

    for path in args {
        self.vfs.mkdir(path)?;
    }

    Ok(String::new())
}
```

**Features:**
- Supports multiple arguments: `mkdir dir1 dir2 dir3`
- Returns error if no arguments
- Creates each directory via VFS

**Example:**
```
vos> mkdir /projects /documents /photos
vos> ls /
projects  (inode 2)
documents  (inode 3)
photos  (inode 4)
```

### Create File (touch)

```rust
fn cmd_touch(&mut self, args: &[&str]) -> Result<String> {
    if args.is_empty() {
        return Ok("touch: missing file operand".to_string());
    }

    for path in args {
        self.vfs.create(path)?;
    }

    Ok(String::new())
}
```

Creates empty files. Similar to `mkdir` but for files.

**Example:**
```
vos> touch /projects/plan.txt /projects/notes.txt
vos> ls /projects
plan.txt  (inode 5)
notes.txt  (inode 6)
```

### Display File (cat)

```rust
fn cmd_cat(&mut self, args: &[&str]) -> Result<String> {
    if args.is_empty() {
        return Ok("cat: missing file operand".to_string());
    }

    let path = args[0];

    // Open file
    let fd = self.vfs.open(path, OpenMode::ReadOnly)?;

    // Read contents
    let mut buffer = vec![0u8; 4096];
    let bytes_read = self.vfs.read(fd, &mut buffer)?;

    // Close file
    self.vfs.close(fd)?;

    // Convert to string
    let content = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    Ok(content)
}
```

**Steps:**
1. Open file for reading
2. Read into buffer
3. Close file
4. Convert bytes to string
5. Return content

**Example:**
```
vos> cat /projects/plan.txt
Phase 1: Design
Phase 2: Implementation
Phase 3: Testing
```

### Remove File/Directory (rm)

```rust
fn cmd_rm(&mut self, args: &[&str]) -> Result<String> {
    if args.is_empty() {
        return Ok("rm: missing operand".to_string());
    }

    for path in args {
        self.vfs.unlink(path)?;
    }

    Ok(String::new())
}
```

Removes files or empty directories via VFS `unlink`.

**Example:**
```
vos> rm /projects/old.txt
vos> ls /projects
plan.txt  (inode 5)
notes.txt  (inode 6)
```

## Command History

The shell maintains a history of executed commands:

```rust
pub struct Shell {
    history: Vec<String>,
    // ...
}

pub fn execute(&mut self, line: &str) -> Result<String> {
    // ... parse command ...

    // Add to history
    self.history.push(line.to_string());

    // ... execute command ...
}

pub fn history(&self) -> &[String] {
    &self.history
}
```

This allows:
- Viewing past commands
- Repeating commands
- Debugging (what did I just run?)

A real shell would add:
- History persistence (save to file)
- History navigation (arrow keys)
- History search (Ctrl+R)
- History expansion (!!, !$, !n)

## The REPL Implementation

Let's look at the complete REPL in `vos-cli`:

```rust
fn main() {
    println!("VOS Shell v0.1.0");
    println!("Type 'help' for available commands, 'exit' to quit.\n");

    let mut shell = Shell::new();

    loop {
        // 1. Display prompt
        print!("vos> ");
        io::stdout().flush().unwrap();

        // 2. Read input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            break;
        }

        // 3. Execute command
        match shell.execute(input.trim()) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }

        // 4. Check exit condition
        if shell.should_exit() {
            break;
        }
    }
}
```

### Key Details

**Prompt Display:**
```rust
print!("vos> ");
io::stdout().flush().unwrap();
```
- `print!` doesn't flush automatically
- Must call `flush()` to show prompt immediately

**Input Reading:**
```rust
let mut input = String::new();
io::stdin().read_line(&mut input)
```
- Creates buffer
- Reads entire line (including newline)
- That's why we `trim()` before executing

**Error Handling:**
```rust
match shell.execute(input.trim()) {
    Ok(output) => println!("{}", output),
    Err(e) => eprintln!("Error: {}", e),
}
```
- Success: Print output
- Error: Print to stderr with `eprintln!`

## Running the Shell

Build and run:
```bash
cargo build --package vos-cli
cargo run --package vos-cli
```

Example session:
```
VOS Shell v0.1.0
Type 'help' for available commands, 'exit' to quit.

vos> help
VOS Shell - Available Commands:

  help         Display this help message
  exit         Exit the shell
  pwd          Print working directory
  cd [PATH]    Change directory
  ls [PATH]    List directory contents
  cat FILE     Display file contents
  echo [ARGS]  Echo arguments to output
  mkdir PATH   Create directory
  touch FILE   Create empty file
  rm PATH      Remove file or directory

vos> mkdir /projects
vos> mkdir /documents
vos> ls /
projects  (inode 2)
documents  (inode 3)
vos> cd /projects
vos> pwd
/ (inode 2)
vos> touch plan.txt
vos> echo Hello VOS
Hello VOS
vos> exit
Goodbye!
```

## Advanced Shell Features (Not Implemented Yet)

Real shells like bash and zsh have many advanced features:

### 1. Pipes and Redirection
```bash
cat file.txt | grep "error" > errors.txt
```
- **Pipes**: Connect output of one program to input of another
- **Redirection**: Send output to file instead of terminal

### 2. Environment Variables
```bash
export PATH=/usr/local/bin:/usr/bin
echo $PATH
```
- Store configuration
- Pass data to child processes

### 3. Job Control
```bash
sleep 100 &        # Background job
jobs               # List jobs
fg %1              # Foreground job
```
- Run programs in background
- Manage multiple processes

### 4. Command Substitution
```bash
files=$(ls)
echo "Files: $files"
```
- Use output of command as input to another

### 5. Script Execution
```bash
#!/bin/sh
for file in *.txt; do
    echo "Processing $file"
done
```
- Execute sequence of commands
- Variables, loops, conditionals

### 6. Tab Completion
```bash
cat /home/us<TAB>
# Completes to: cat /home/user/
```
- Auto-complete paths and commands

### 7. Line Editing
- Arrow keys: Navigate command
- Ctrl+A: Move to start
- Ctrl+E: Move to end
- Ctrl+K: Delete to end
- Ctrl+U: Delete to start

These would require significant additional implementation!

## Testing the Shell

Our shell includes comprehensive tests:

```rust
#[test]
fn test_echo() {
    let mut shell = Shell::new();
    let output = shell.execute("echo hello world").unwrap();
    assert_eq!(output, "hello world");
}

#[test]
fn test_mkdir_ls() {
    let mut shell = Shell::new();
    shell.execute("mkdir /testdir").unwrap();

    let output = shell.execute("ls /").unwrap();
    assert!(output.contains("testdir"));
}

#[test]
fn test_touch_cat() {
    let mut shell = Shell::new();

    // Create file
    shell.execute("touch /test.txt").unwrap();

    // Write to it (direct VFS access for testing)
    let fd = shell.vfs_mut().open("/test.txt", OpenMode::WriteOnly).unwrap();
    shell.vfs_mut().write(fd, b"Hello, Shell!").unwrap();
    shell.vfs_mut().close(fd).unwrap();

    // Cat file
    let output = shell.execute("cat /test.txt").unwrap();
    assert_eq!(output, "Hello, Shell!");
}
```

Run shell tests:
```bash
cargo test --package vos-userspace
```

## Hands-On Exercise

Let's create a shell session that demonstrates the filesystem:

```rust
use vos_userspace::Shell;

fn demo_session() {
    let mut shell = Shell::new();

    // Create directory structure
    println!("=== Creating directory structure ===");
    shell.execute("mkdir /home").unwrap();
    shell.execute("mkdir /home/user").unwrap();
    shell.execute("mkdir /home/user/documents").unwrap();
    shell.execute("mkdir /home/user/projects").unwrap();

    // List home directory
    println!("\n=== Listing /home/user ===");
    let output = shell.execute("ls /home/user").unwrap();
    println!("{}", output);

    // Create files
    println!("\n=== Creating files ===");
    shell.execute("touch /home/user/documents/readme.txt").unwrap();
    shell.execute("touch /home/user/projects/main.rs").unwrap();

    // Write to file
    let fd = shell.vfs_mut()
        .open("/home/user/documents/readme.txt", OpenMode::WriteOnly)
        .unwrap();
    shell.vfs_mut()
        .write(fd, b"Welcome to VOS!\n\nThis is a simple operating system.\n")
        .unwrap();
    shell.vfs_mut().close(fd).unwrap();

    // Display file
    println!("\n=== Reading readme.txt ===");
    let output = shell.execute("cat /home/user/documents/readme.txt").unwrap();
    println!("{}", output);

    // Navigate directories
    println!("\n=== Navigating ===");
    shell.execute("cd /home/user").unwrap();
    println!("Changed to /home/user");

    let output = shell.execute("ls").unwrap();
    println!("Contents:\n{}", output);
}
```

Run this to see the shell in action!

## Challenge Problems

1. **Write Command**: Implement a `write` command that writes text to a file:
   ```rust
   fn cmd_write(&mut self, args: &[&str]) -> Result<String> {
       // Usage: write <path> <text>
       // Write text to file at path
   }
   ```

2. **Copy Command**: Implement `cp` to copy files:
   ```rust
   fn cmd_cp(&mut self, args: &[&str]) -> Result<String> {
       // Usage: cp <src> <dst>
       // Copy file from src to dst
   }
   ```

3. **Tree Command**: Display directory tree recursively:
   ```rust
   fn cmd_tree(&self, args: &[&str]) -> Result<String> {
       // Usage: tree [path]
       // Display directory structure as tree
   }
   ```

4. **Find Command**: Search for files by name:
   ```rust
   fn cmd_find(&self, args: &[&str]) -> Result<String> {
       // Usage: find <path> <name>
       // Find all files matching name under path
   }
   ```

5. **History Command**: Display command history:
   ```rust
   fn cmd_history(&self, args: &[&str]) -> Result<String> {
       // Display numbered list of previous commands
   }
   ```

## Improving the Shell

### Better Line Editing

Use the `rustyline` crate for advanced line editing:

```toml
[dependencies]
rustyline = "12.0"
```

```rust
use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new().unwrap();
    let mut shell = Shell::new();

    loop {
        match rl.readline("vos> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                match shell.execute(&line) {
                    Ok(output) => println!("{}", output),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(_) => break,
        }

        if shell.should_exit() {
            break;
        }
    }
}
```

Features:
- Arrow keys for history
- Line editing (Ctrl+A, Ctrl+E, etc.)
- History persistence
- Tab completion (with configuration)

### Color Output

Use `colored` crate for colorful output:

```rust
use colored::*;

println!("{}", "Error:".red());
println!("{}", "Success!".green());
println!("{}", filename.blue());
```

### Argument Parsing

Use `clap` for complex command arguments:

```rust
use clap::Parser;

#[derive(Parser)]
struct LsArgs {
    /// Show hidden files
    #[arg(short, long)]
    all: bool,

    /// Long format
    #[arg(short, long)]
    long: bool,

    /// Path to list
    path: Option<String>,
}
```

## Shell Execution Flow Diagram

```
User Input: "mkdir /projects"
        │
        ▼
┌─────────────────────┐
│   Read Input        │  read_line()
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Parse Command     │  "mkdir" + ["/projects"]
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Match Command     │  match "mkdir"
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Execute Built-in  │  cmd_mkdir()
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Call VFS          │  vfs.mkdir("/projects")
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Return Result     │  Ok(String::new())
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Display Output    │  (empty output)
└─────────────────────┘
        │
        ▼
        Loop back to Read
```

## Key Takeaways

1. **Shell is userspace software**: Runs with normal user privileges, uses kernel services
2. **REPL pattern is simple**: Read → Eval → Print → Loop
3. **Built-in commands must be internal**: Like `cd`, `exit`, `export`
4. **Command parsing can be complex**: Our simple version splits by whitespace
5. **Shell maintains state**: Current directory, history, environment
6. **Error handling is important**: Display errors without crashing

## Next Steps

Now that we have a working shell, we can:
- Implement system calls for the shell to use (Chapter 9)
- Add external programs that the shell can execute (Chapter 10)
- Create a programming language for scripts (Chapter 11)

## Further Reading

- "The UNIX Programming Environment" by Kernighan & Pike
- Bash source code: https://git.savannah.gnu.org/cgit/bash.git
- Fish shell: https://github.com/fish-shell/fish-shell
- "Writing a Unix Shell" tutorial series

## Summary

In this chapter, we built a complete interactive shell with:
- REPL implementation
- Command parsing
- 10 built-in commands (exit, pwd, cd, ls, cat, echo, mkdir, touch, rm, help)
- Command history
- Filesystem integration via VFS
- Comprehensive tests

Our shell provides a familiar Unix-like interface for interacting with VOS. Users can create directories, manage files, navigate the filesystem, and display content—all through simple text commands.

The shell is the bridge between the user and the operating system. While simple compared to bash or zsh, our implementation demonstrates the core concepts used in all shells. Understanding how shells work gives you insight into how users interact with operating systems at the command line.

In the next chapter, we'll explore how to extend the shell with external programs and process execution!
