//! File operations.

use super::inode::InodeNumber;

/// File descriptor.
pub type FileDescriptor = u32;

/// File open mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Read only
    ReadOnly,
    /// Write only
    WriteOnly,
    /// Read and write
    ReadWrite,
    /// Append (write at end)
    Append,
}

/// Open file handle.
///
/// Represents an open file in a process.
#[derive(Debug, Clone)]
pub struct File {
    /// File descriptor number
    pub fd: FileDescriptor,

    /// Inode number
    pub inode: InodeNumber,

    /// Open mode
    pub mode: OpenMode,

    /// Current position in file
    pub position: usize,

    /// File is open
    pub is_open: bool,
}

impl File {
    /// Creates a new file handle.
    pub fn new(fd: FileDescriptor, inode: InodeNumber, mode: OpenMode) -> Self {
        Self {
            fd,
            inode,
            mode,
            position: 0,
            is_open: true,
        }
    }

    /// Checks if file is readable.
    pub fn can_read(&self) -> bool {
        matches!(self.mode, OpenMode::ReadOnly | OpenMode::ReadWrite)
    }

    /// Checks if file is writable.
    pub fn can_write(&self) -> bool {
        matches!(
            self.mode,
            OpenMode::WriteOnly | OpenMode::ReadWrite | OpenMode::Append
        )
    }

    /// Seeks to a position in the file.
    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }

    /// Advances position by offset.
    pub fn advance(&mut self, offset: usize) {
        self.position += offset;
    }

    /// Closes the file.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Checks if file is open.
    pub fn is_closed(&self) -> bool {
        !self.is_open
    }
}

/// File table - manages open files.
#[derive(Debug)]
pub struct FileTable {
    /// Open files
    files: Vec<Option<File>>,

    /// Next FD to assign
    next_fd: FileDescriptor,
}

impl FileTable {
    /// Creates a new file table with standard streams.
    pub fn new() -> Self {
        let mut table = Self {
            files: Vec::new(),
            next_fd: 3, // 0=stdin, 1=stdout, 2=stderr
        };

        // Reserve standard streams
        table.files.push(None); // stdin
        table.files.push(None); // stdout
        table.files.push(None); // stderr

        table
    }

    /// Opens a file and returns a file descriptor.
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

        let file = File::new(fd, inode, mode);

        // Ensure the vector is large enough to hold this fd
        while self.files.len() <= fd as usize {
            self.files.push(None);
        }

        // Store at the index matching the fd
        self.files[fd as usize] = Some(file);

        fd
    }

    /// Gets a file by descriptor.
    pub fn get(&self, fd: FileDescriptor) -> Option<&File> {
        self.files
            .get(fd as usize)
            .and_then(|slot| slot.as_ref())
    }

    /// Gets a mutable file by descriptor.
    pub fn get_mut(&mut self, fd: FileDescriptor) -> Option<&mut File> {
        self.files
            .get_mut(fd as usize)
            .and_then(|slot| slot.as_mut())
    }

    /// Closes a file.
    pub fn close(&mut self, fd: FileDescriptor) -> bool {
        if let Some(file) = self.get_mut(fd) {
            file.close();
            true
        } else {
            false
        }
    }

    /// Returns the number of open files.
    pub fn open_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_some()).count()
    }
}

impl Default for FileTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_creation() {
        let file = File::new(3, 100, OpenMode::ReadOnly);

        assert_eq!(file.fd, 3);
        assert_eq!(file.inode, 100);
        assert_eq!(file.mode, OpenMode::ReadOnly);
        assert_eq!(file.position, 0);
        assert!(file.is_open);
    }

    #[test]
    fn test_file_permissions() {
        let ro = File::new(3, 100, OpenMode::ReadOnly);
        assert!(ro.can_read());
        assert!(!ro.can_write());

        let wo = File::new(4, 101, OpenMode::WriteOnly);
        assert!(!wo.can_read());
        assert!(wo.can_write());

        let rw = File::new(5, 102, OpenMode::ReadWrite);
        assert!(rw.can_read());
        assert!(rw.can_write());
    }

    #[test]
    fn test_file_seek() {
        let mut file = File::new(3, 100, OpenMode::ReadOnly);

        file.seek(100);
        assert_eq!(file.position, 100);

        file.advance(50);
        assert_eq!(file.position, 150);
    }

    #[test]
    fn test_file_close() {
        let mut file = File::new(3, 100, OpenMode::ReadOnly);
        assert!(file.is_open);

        file.close();
        assert!(file.is_closed());
    }

    #[test]
    fn test_file_table_creation() {
        let table = FileTable::new();
        assert_eq!(table.open_count(), 0); // stdin/stdout/stderr are None
    }

    #[test]
    fn test_file_table_open() {
        let mut table = FileTable::new();

        let fd1 = table.open(100, OpenMode::ReadOnly);
        assert_eq!(fd1, 3); // First available after std streams

        let fd2 = table.open(101, OpenMode::WriteOnly);
        assert_eq!(fd2, 4);

        assert_eq!(table.open_count(), 2);
    }

    #[test]
    fn test_file_table_get() {
        let mut table = FileTable::new();

        let fd = table.open(100, OpenMode::ReadOnly);

        let file = table.get(fd).unwrap();
        assert_eq!(file.inode, 100);
        assert_eq!(file.mode, OpenMode::ReadOnly);
    }

    #[test]
    fn test_file_table_close() {
        let mut table = FileTable::new();

        let fd = table.open(100, OpenMode::ReadOnly);
        assert!(table.close(fd));

        let file = table.get(fd).unwrap();
        assert!(file.is_closed());
    }

    #[test]
    fn test_file_table_reuse_slot() {
        let mut table = FileTable::new();

        let fd1 = table.open(100, OpenMode::ReadOnly);
        table.close(fd1);

        // Opening new file should reuse slot
        let fd2 = table.open(101, OpenMode::WriteOnly);

        // Both FDs point to different inodes
        assert_eq!(table.get(fd2).unwrap().inode, 101);
    }
}
