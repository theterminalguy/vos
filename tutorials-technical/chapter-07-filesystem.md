# Chapter 7: File Systems

## Learning Objectives

After completing this chapter, you will understand:
- How filesystems organize data on disk
- Inode-based filesystem architecture
- File operations: open, read, write, close
- Directory structure and path resolution
- Virtual File System (VFS) abstraction
- File descriptors and file tables

## Introduction

A filesystem is one of the most essential components of an operating system. It provides a structured way to store, organize, and retrieve data on storage devices. Without a filesystem, data would be just a sequence of bytes with no organization or meaning.

In this chapter, we'll explore VOS's inode-based filesystem, inspired by Unix filesystems like ext2 and ext3. We'll see how files and directories are represented, how the VFS provides abstraction, and how processes interact with files through system calls.

## Why We Need File Systems

Consider these fundamental problems that filesystems solve:

1. **Organization**: How do we organize millions of files?
2. **Naming**: How do we give files meaningful names?
3. **Persistence**: How do we ensure data survives after power off?
4. **Access Control**: How do we control who can access what?
5. **Efficiency**: How do we make file operations fast?

The filesystem is the answer to all these questions.

## Filesystem Architecture Overview

VOS's filesystem has a layered architecture:

```
┌─────────────────────────────────────┐
│     User Programs (Shell, etc.)     │
└─────────────────────────────────────┘
                 │
                 │ System calls (open, read, write, close)
                 ▼
┌─────────────────────────────────────┐
│   Virtual File System (VFS) Layer   │  ← Abstraction layer
└─────────────────────────────────────┘
                 │
                 │ VFS operations
                 ▼
┌─────────────────────────────────────┐
│      SimpleFs Implementation        │  ← Actual filesystem
└─────────────────────────────────────┘
                 │
                 │ Block I/O
                 ▼
┌─────────────────────────────────────┐
│         Disk Controller             │  ← Hardware
└─────────────────────────────────────┘
```

### Key Components

1. **Inodes**: Metadata about files (size, permissions, timestamps)
2. **Directories**: Maps filenames to inode numbers
3. **File Table**: Tracks open files per process
4. **VFS**: Provides unified interface regardless of underlying filesystem

## Inodes: The Heart of the Filesystem

An **inode** (index node) is a data structure that stores metadata about a file or directory. Every file has exactly one inode, and every inode has a unique number.

### Inode Structure

```rust
pub struct Inode {
    pub number: InodeNumber,           // Unique identifier
    pub inode_type: InodeType,         // File or Directory
    pub size: usize,                   // Size in bytes
    pub link_count: u32,               // Number of hard links
    pub permissions: u16,              // Read/write/execute bits
    pub uid: u32,                      // Owner user ID
    pub gid: u32,                      // Owner group ID
    pub created: u64,                  // Creation time
    pub modified: u64,                 // Last modification time
    pub accessed: u64,                 // Last access time
    pub blocks: Vec<Address>,          // Data block addresses
    pub indirect_block: Option<Address>, // For large files
}
```

### What Inodes Store

Notice what's **in** an inode:
- File metadata (size, permissions, timestamps)
- Owner information
- Pointers to data blocks

Notice what's **not** in an inode:
- The filename (stored in directories!)
- The actual file data (stored in data blocks)

### Inode Types

```rust
pub enum InodeType {
    File,       // Regular file
    Directory,  // Directory (special file containing entries)
}
```

### Permission Bits

Permissions are stored as a 16-bit integer:

```
Bits: UUUGGGOOO
      │││││││││
      │││││││└─→ Others: Execute
      ││││││└──→ Others: Write
      │││││└───→ Others: Read
      ││││└────→ Group: Execute
      │││└─────→ Group: Write
      ││└──────→ Group: Read
      │└───────→ User: Execute
      └────────→ User: Write
             ...→ User: Read

Example: 0o644 = rw-r--r--
- Owner: read + write (6 = 110)
- Group: read (4 = 100)
- Others: read (4 = 100)
```

### Inode Operations

```rust
// Create a new inode
let inode = Inode::new(1, InodeType::File, current_time);

// Check permissions
if inode.can_read(uid, gid) {
    // Read file
}

// Update timestamps
inode.access(current_time);  // Update accessed time
inode.touch(current_time);   // Update modified and accessed
```

## Directories: Mapping Names to Inodes

A **directory** is a special file that maps filenames to inode numbers. It's the bridge between human-readable names and the filesystem's internal structure.

### Directory Structure

```rust
pub struct Directory {
    entries: Vec<DirectoryEntry>,
}

pub struct DirectoryEntry {
    pub name: String,          // Filename
    pub inode: InodeNumber,    // Inode number
}
```

### Special Directory Entries

Every directory contains two special entries:
- `.` (dot): Points to the directory itself
- `..` (dot-dot): Points to the parent directory

```rust
// Create directory with . and ..
let dir = Directory::with_dots(self_inode, parent_inode);
```

### Example Directory Layout

Consider this directory structure:

```
/
├── home/
│   └── user/
│       ├── file.txt
│       └── notes/
└── bin/
    └── shell
```

The root directory `/` (inode 1) contains:

```
Entry       Inode
.           1      (itself)
..          1      (parent is also root)
home        2
bin         3
```

The `/home` directory (inode 2) contains:

```
Entry       Inode
.           2      (itself)
..          1      (parent is root)
user        4
```

### Directory Operations

```rust
// Add entry
dir.add_entry("file.txt".to_string(), 100);

// Look up entry
if let Some(inode) = dir.lookup("file.txt") {
    println!("file.txt is inode {}", inode);
}

// Remove entry
dir.remove_entry("file.txt");

// List all entries
for entry in dir.list() {
    println!("{} -> {}", entry.name, entry.inode);
}

// Check if directory is empty (excluding . and ..)
if dir.is_empty() {
    // Can safely delete
}
```

## File Descriptors and File Tables

When a process opens a file, it receives a **file descriptor** (fd), a small integer that represents the open file. The process uses this fd for all subsequent operations.

### File Descriptor Structure

```rust
pub struct File {
    pub fd: FileDescriptor,      // File descriptor number
    pub inode: InodeNumber,      // Which file
    pub mode: OpenMode,          // How it's opened
    pub position: usize,         // Current read/write position
    pub is_open: bool,           // Open or closed
}
```

### Open Modes

```rust
pub enum OpenMode {
    ReadOnly,    // Can only read
    WriteOnly,   // Can only write
    ReadWrite,   // Can read and write
    Append,      // Write at end of file
}
```

### File Table

Each process has a **file table** that tracks all its open files:

```rust
pub struct FileTable {
    files: Vec<Option<File>>,     // Array of open files
    next_fd: FileDescriptor,      // Next fd to assign
}
```

### Standard File Descriptors

By convention, the first three file descriptors are reserved:
- `fd 0`: stdin (standard input)
- `fd 1`: stdout (standard output)
- `fd 2`: stderr (standard error)

User files start at fd 3.

### File Descriptor Lifecycle

```rust
// 1. Create file table
let mut table = FileTable::new();  // Reserves fds 0, 1, 2

// 2. Open a file
let fd = table.open(inode_num, OpenMode::ReadWrite);
// Returns fd = 3 (first available)

// 3. Get file for operations
let file = table.get(fd).unwrap();
println!("Position: {}", file.position);

// 4. Close the file
table.close(fd);

// 5. Reuse fd
let fd2 = table.open(another_inode, OpenMode::ReadOnly);
// Returns fd = 3 (reuses closed fd)
```

## SimpleFs: The Filesystem Implementation

`SimpleFs` is VOS's in-memory inode-based filesystem. It's "simple" because it stores everything in memory (no actual disk persistence yet), but it demonstrates all the key filesystem concepts.

### SimpleFs Structure

```rust
pub struct SimpleFs {
    inodes: HashMap<InodeNumber, Inode>,         // Inode table
    directories: HashMap<InodeNumber, Directory>, // Directory contents
    files: HashMap<InodeNumber, Vec<u8>>,        // File data
    next_inode: InodeNumber,                     // Next inode to allocate
    current_time: u64,                           // Current time
    root_inode: InodeNumber,                     // Root directory (always 1)
}
```

### Creating Files and Directories

```rust
// Create a file
let inode = fs.create_file(parent_inode, "test.txt".to_string())?;
// Returns new inode number

// Create a directory
let dir_inode = fs.create_directory(parent_inode, "mydir".to_string())?;
```

What happens when creating a file:

1. **Allocate inode**: Assign next available inode number
2. **Initialize inode**: Set type to File, timestamps, permissions
3. **Add to parent directory**: Create directory entry mapping name → inode
4. **Initialize file data**: Create empty data buffer

### Path Resolution

Path resolution converts a path string (e.g., `/home/user/file.txt`) into an inode number:

```rust
pub fn resolve_path(&self, path: &str) -> Option<InodeNumber> {
    if path == "/" {
        return Some(self.root_inode);
    }

    let mut current = self.root_inode;

    // Split path and look up each component
    for component in path.split('/').filter(|s| !s.is_empty()) {
        current = self.lookup(current, component)?;
    }

    Some(current)
}
```

Example: Resolving `/home/user/file.txt`

1. Start at root (inode 1)
2. Look up "home" in root directory → inode 2
3. Look up "user" in directory 2 → inode 5
4. Look up "file.txt" in directory 5 → inode 10
5. Return inode 10

### Reading and Writing Files

```rust
// Write file
let data = b"Hello, VOS!".to_vec();
fs.write_file(inode, data)?;

// Read file
let contents = fs.read_file(inode)?;
println!("{}", String::from_utf8_lossy(&contents));
```

What happens during write:
1. Update file size in inode
2. Store data in files HashMap
3. Update modified timestamp

What happens during read:
1. Update accessed timestamp
2. Retrieve data from files HashMap
3. Return copy of data

### Listing Directories

```rust
// List directory
let entries = fs.list_directory(inode)?;
for (name, inode) in entries {
    println!("{} -> {}", name, inode);
}
```

### Deleting Files

```rust
// Delete a file or empty directory
fs.delete(parent_inode, "file.txt")?;
```

What happens during delete:
1. Remove entry from parent directory
2. If directory, check it's empty (only . and ..)
3. Remove from directories HashMap (if dir)
4. Remove from files HashMap
5. Remove inode from inode table

## Virtual File System (VFS)

The VFS provides a high-level abstraction over the filesystem. It adds:
- Current working directory tracking
- File descriptor management
- Path manipulation utilities
- Error handling with typed errors

### VFS Structure

```rust
pub struct Vfs {
    fs: SimpleFs,              // Underlying filesystem
    file_table: FileTable,     // Open files
    cwd: InodeNumber,          // Current working directory
}
```

### VFS Errors

```rust
pub enum VfsError {
    FileNotFound(String),
    PermissionDenied(String),
    InvalidFd(FileDescriptor),
    NotADirectory(String),
    IsADirectory(String),
    FileExists(String),
    DirectoryNotEmpty(String),
}
```

### VFS Operations

#### Opening Files

```rust
let fd = vfs.open("/home/user/file.txt", OpenMode::ReadWrite)?;
```

Steps:
1. Resolve path to inode
2. Verify it's a file (not directory)
3. Allocate file descriptor
4. Add to file table
5. Return fd

#### Reading Files

```rust
let mut buffer = vec![0u8; 100];
let bytes_read = vfs.read(fd, &mut buffer)?;
```

Steps:
1. Look up file in file table
2. Verify read permission (mode)
3. Get current position
4. Read from filesystem at position
5. Copy to buffer
6. Update position
7. Return bytes read

#### Writing Files

```rust
let data = b"Hello, World!";
let bytes_written = vfs.write(fd, data)?;
```

Steps:
1. Look up file in file table
2. Verify write permission (mode)
3. Get current position
4. Read existing file contents
5. Expand buffer if needed
6. Write data at position
7. Write back to filesystem
8. Update position
9. Return bytes written

#### Closing Files

```rust
vfs.close(fd)?;
```

Simply marks the file as closed in the file table. The fd can be reused.

#### Creating Files

```rust
let inode = vfs.create("/home/user/newfile.txt")?;
```

Steps:
1. Split path into parent and name
2. Resolve parent directory
3. Create file in parent
4. Return new inode

#### Creating Directories

```rust
let inode = vfs.mkdir("/home/user/newdir")?;
```

Similar to create, but creates a directory with . and .. entries.

#### Changing Directory

```rust
vfs.chdir("/home/user")?;
```

Steps:
1. Resolve path to inode
2. Verify it's a directory
3. Update cwd to new inode

#### Listing Directory

```rust
let entries = vfs.readdir("/home/user")?;
for (name, inode) in entries {
    println!("{} (inode {})", name, inode);
}
```

#### Removing Files

```rust
vfs.unlink("/home/user/oldfile.txt")?;
```

## Complete File Operation Example

Let's trace through a complete example:

```rust
// Create VFS
let mut vfs = Vfs::new();

// Create a file in root directory
let inode = vfs.create("/test.txt")?;
println!("Created file with inode {}", inode);

// Open file for writing
let fd = vfs.open("/test.txt", OpenMode::ReadWrite)?;
println!("Opened with fd {}", fd);

// Write data
let data = b"Hello, VOS filesystem!";
let written = vfs.write(fd, data)?;
println!("Wrote {} bytes", written);

// Close file
vfs.close(fd)?;

// Reopen for reading
let fd = vfs.open("/test.txt", OpenMode::ReadOnly)?;

// Read data
let mut buffer = vec![0u8; 100];
let read = vfs.read(fd, &mut buffer)?;
println!("Read {} bytes: {}", read, String::from_utf8_lossy(&buffer[..read]));

// Close
vfs.close(fd)?;

// Create directory
vfs.mkdir("/mydir")?;

// Create file in directory
vfs.create("/mydir/nested.txt")?;

// List root directory
let entries = vfs.readdir("/")?;
println!("Root directory:");
for (name, inode) in entries {
    println!("  {} -> inode {}", name, inode);
}
```

Output:
```
Created file with inode 2
Opened with fd 3
Wrote 22 bytes
Read 22 bytes: Hello, VOS filesystem!
Root directory:
  . -> inode 1
  .. -> inode 1
  test.txt -> inode 2
  mydir -> inode 3
```

## Nested Directories Example

```rust
let mut vfs = Vfs::new();

// Create nested directory structure
vfs.mkdir("/usr")?;
vfs.mkdir("/usr/local")?;
vfs.mkdir("/usr/local/bin")?;

// Create file in nested directory
vfs.create("/usr/local/bin/program")?;

// Open and write
let fd = vfs.open("/usr/local/bin/program", OpenMode::WriteOnly)?;
vfs.write(fd, b"#!/bin/sh\necho Hello")?;
vfs.close(fd)?;

// Read back
let fd = vfs.open("/usr/local/bin/program", OpenMode::ReadOnly)?;
let mut buffer = vec![0u8; 100];
let n = vfs.read(fd, &mut buffer)?;
println!("{}", String::from_utf8_lossy(&buffer[..n]));
```

## Implementation Details

### Memory Layout

In SimpleFs, everything is in memory:

```
Inodes HashMap:
  1 → Inode { type: Directory, size: 0, ... }
  2 → Inode { type: File, size: 22, ... }
  3 → Inode { type: Directory, size: 0, ... }

Directories HashMap:
  1 → Directory { entries: [(".", 1), ("..", 1), ("test.txt", 2)] }
  3 → Directory { entries: [(".", 3), ("..", 1)] }

Files HashMap:
  2 → vec![72, 101, 108, 108, 111, ...]  // "Hello..."
```

### Inode Allocation

Inodes are allocated sequentially:

```rust
fn allocate_inode(&mut self, inode_type: InodeType) -> InodeNumber {
    let number = self.next_inode;
    self.next_inode += 1;  // Increment for next allocation

    let inode = Inode::new(number, inode_type, self.current_time);
    self.inodes.insert(number, inode);

    number
}
```

Root directory always gets inode 1.

### File Descriptor Reuse

The FileTable reuses closed file descriptors:

```rust
pub fn open(&mut self, inode: InodeNumber, mode: OpenMode) -> FileDescriptor {
    // Try to find a closed file slot to reuse
    for (index, slot) in self.files.iter_mut().enumerate() {
        if let Some(file) = slot {
            if file.is_closed() {
                let fd = index as FileDescriptor;
                *slot = Some(File::new(fd, inode, mode));
                return fd;
            }
        }
    }

    // No closed slot found, allocate new fd
    let fd = self.next_fd;
    self.next_fd += 1;
    // ... store in table
}
```

This mimics Unix behavior where `open()` returns the lowest available fd.

## Filesystem Limitations and Future Work

Our current SimpleFs has several limitations:

1. **No Persistence**: Everything is in memory, lost on shutdown
2. **No Disk Blocks**: Files stored as continuous byte arrays
3. **Limited File Size**: No support for very large files
4. **No Hard Links**: Each file has exactly one name
5. **No Symbolic Links**: Can't create links to other files
6. **No File Locking**: Multiple processes can interfere
7. **No Journaling**: No protection against crashes
8. **In-Memory Only**: Limited by RAM size

Future improvements:
- Add disk persistence with actual block I/O
- Implement indirect blocks for large files
- Add hard link support (increment link_count)
- Implement symbolic links
- Add file locking mechanisms
- Implement write-ahead logging or journaling

## Comparing to Real Filesystems

### ext2 (Linux)

Our SimpleFs is inspired by ext2:
- **Inodes**: ✓ Similar structure
- **Directories**: ✓ Name → inode mapping
- **Superblock**: ✗ We don't have one yet
- **Block Groups**: ✗ All in one group (HashMap)
- **Indirect Blocks**: ✗ Not implemented

### FAT32

FAT32 is quite different:
- **File Allocation Table**: Uses a table instead of inodes
- **Clusters**: Fixed-size allocation units
- **Directory Entries**: Store metadata in directory, not separate inode
- **No Permissions**: No Unix-style permissions

## Testing the Filesystem

Our implementation includes comprehensive tests:

```rust
#[test]
fn test_vfs_creation() {
    let vfs = Vfs::new();
    assert_eq!(vfs.getcwd(), 1); // Root is inode 1
}

#[test]
fn test_write_read() {
    let mut vfs = Vfs::new();
    vfs.create("/test.txt").unwrap();

    let fd = vfs.open("/test.txt", OpenMode::ReadWrite).unwrap();

    // Write
    let data = b"Hello, VFS!";
    let written = vfs.write(fd, data).unwrap();
    assert_eq!(written, data.len());

    // Close and reopen
    vfs.close(fd).unwrap();
    let fd = vfs.open("/test.txt", OpenMode::ReadOnly).unwrap();

    // Read
    let mut buffer = vec![0u8; 20];
    let read = vfs.read(fd, &mut buffer).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buffer[..read], data);

    vfs.close(fd).unwrap();
}

#[test]
fn test_nested_directories() {
    let mut vfs = Vfs::new();

    vfs.mkdir("/dir1").unwrap();
    vfs.mkdir("/dir1/dir2").unwrap();
    vfs.create("/dir1/dir2/file.txt").unwrap();

    let fd = vfs.open("/dir1/dir2/file.txt", OpenMode::WriteOnly).unwrap();
    vfs.write(fd, b"nested").unwrap();
    vfs.close(fd).unwrap();

    let fd = vfs.open("/dir1/dir2/file.txt", OpenMode::ReadOnly).unwrap();
    let mut buffer = vec![0u8; 10];
    let read = vfs.read(fd, &mut buffer).unwrap();
    assert_eq!(&buffer[..read], b"nested");
}
```

Run tests:
```bash
cargo test --package vos-kernel
```

## Hands-On Exercise

Let's implement a simple "file manager" that uses the VFS:

```rust
use vos_kernel::{Vfs, OpenMode};

fn main() -> vos_core::Result<()> {
    let mut vfs = Vfs::new();

    // Create directory structure
    vfs.mkdir("/home")?;
    vfs.mkdir("/home/user")?;
    vfs.mkdir("/home/user/documents")?;

    // Create a file
    vfs.create("/home/user/documents/notes.txt")?;

    // Write to file
    let fd = vfs.open("/home/user/documents/notes.txt", OpenMode::WriteOnly)?;
    vfs.write(fd, b"Meeting at 3pm\n")?;
    vfs.write(fd, b"Buy groceries\n")?;
    vfs.write(fd, b"Call mom\n")?;
    vfs.close(fd)?;

    // Read file
    let fd = vfs.open("/home/user/documents/notes.txt", OpenMode::ReadOnly)?;
    let mut buffer = vec![0u8; 1024];
    let n = vfs.read(fd, &mut buffer)?;
    println!("=== notes.txt ===");
    println!("{}", String::from_utf8_lossy(&buffer[..n]));
    vfs.close(fd)?;

    // List directory
    println!("\n=== /home/user/documents ===");
    let entries = vfs.readdir("/home/user/documents")?;
    for (name, inode) in entries {
        if name != "." && name != ".." {
            println!("{} (inode {})", name, inode);
        }
    }

    Ok(())
}
```

## Challenge Problems

1. **File Copy**: Implement a function that copies a file:
   ```rust
   fn copy_file(vfs: &mut Vfs, src: &str, dst: &str) -> Result<()> {
       // Your implementation here
   }
   ```

2. **Recursive Delete**: Implement a function that deletes a directory and all its contents:
   ```rust
   fn remove_recursive(vfs: &mut Vfs, path: &str) -> Result<()> {
       // Your implementation here
   }
   ```

3. **File Search**: Implement a function that searches for files by name:
   ```rust
   fn find_file(vfs: &Vfs, name: &str) -> Vec<String> {
       // Return all paths containing files with this name
   }
   ```

4. **Disk Usage**: Implement a function that calculates total size of all files:
   ```rust
   fn disk_usage(vfs: &Vfs, path: &str) -> usize {
       // Return total bytes used by path and descendants
   }
   ```

## Key Takeaways

1. **Inodes separate metadata from data**: The inode stores file information, directories map names to inodes
2. **File descriptors are process-specific**: Each process has its own file table
3. **Path resolution is iterative**: Walk through directory hierarchy one component at a time
4. **VFS provides abstraction**: Hide implementation details behind clean interface
5. **Directories are special files**: They contain mappings instead of arbitrary data
6. **Unix-style design is elegant**: Everything is a file, simple and powerful

## Next Steps

Now that we understand filesystems, we're ready to:
- Build a shell that uses the filesystem (Chapter 8)
- Implement system calls for file operations (Chapter 9)
- Create user programs that read and write files (Chapter 10)

## Further Reading

- "The Design and Implementation of the 4.4BSD Operating System"
- "Linux Kernel Development" by Robert Love (Chapter on VFS)
- ext2 filesystem specification
- "Operating Systems: Three Easy Pieces" - File Systems chapter

## Summary

In this chapter, we built a complete inode-based filesystem from scratch. We learned:
- How inodes store file metadata
- How directories map names to inodes
- How file descriptors work
- How to implement file operations (open, read, write, close)
- How the VFS provides abstraction
- How path resolution works

Our filesystem implements the core concepts used in real systems like ext2, ext3, and ext4. While simplified (no disk persistence, no indirect blocks), it demonstrates all the fundamental principles of modern filesystems.

In the next chapter, we'll build a shell that allows users to interact with our filesystem using familiar commands like `ls`, `cat`, `cd`, and more!
