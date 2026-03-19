//! Virtual File System (VFS) layer.
//!
//! Provides a unified interface to different filesystem types.

use super::file::{FileDescriptor, FileTable, OpenMode};
use super::inode::InodeNumber;
use super::simple_fs::SimpleFs;
use thiserror::Error;
use vos_core::{Result, VosError};

/// VFS errors.
#[derive(Error, Debug)]
pub enum VfsError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid file descriptor: {0}")]
    InvalidFd(FileDescriptor),

    #[error("Not a directory: {0}")]
    NotADirectory(String),

    #[error("Is a directory: {0}")]
    IsADirectory(String),

    #[error("File exists: {0}")]
    FileExists(String),

    #[error("Directory not empty: {0}")]
    DirectoryNotEmpty(String),
}

impl From<VfsError> for VosError {
    fn from(err: VfsError) -> Self {
        VosError::Filesystem(err.to_string())
    }
}

/// Virtual File System.
///
/// Provides file operations abstracted from the underlying filesystem.
pub struct Vfs {
    /// The filesystem
    fs: SimpleFs,

    /// File table for open files
    file_table: FileTable,

    /// Current working directory
    cwd: InodeNumber,
}

impl Vfs {
    /// Creates a new VFS.
    pub fn new() -> Self {
        let fs = SimpleFs::new();
        let root = fs.root();

        Self {
            fs,
            file_table: FileTable::new(),
            cwd: root,
        }
    }

    /// Opens a file.
    pub fn open(&mut self, path: &str, mode: OpenMode) -> Result<FileDescriptor> {
        let inode = self
            .fs
            .resolve_path(path)
            .ok_or_else(|| VfsError::FileNotFound(path.to_string()))?;

        // Check if it's a file
        let inode_meta = self
            .fs
            .get_inode(inode)
            .ok_or_else(|| VfsError::FileNotFound(path.to_string()))?;

        if inode_meta.is_directory() {
            return Err(VfsError::IsADirectory(path.to_string()).into());
        }

        let fd = self.file_table.open(inode, mode);
        Ok(fd)
    }

    /// Closes a file.
    pub fn close(&mut self, fd: FileDescriptor) -> Result<()> {
        if self.file_table.close(fd) {
            Ok(())
        } else {
            Err(VfsError::InvalidFd(fd).into())
        }
    }

    /// Reads from a file.
    pub fn read(&mut self, fd: FileDescriptor, buffer: &mut [u8]) -> Result<usize> {
        let file = self
            .file_table
            .get(fd)
            .ok_or(VfsError::InvalidFd(fd))?;

        if !file.can_read() {
            return Err(VfsError::PermissionDenied("File not open for reading".to_string()).into());
        }

        let inode = file.inode;
        let position = file.position;

        // Read file contents
        let data = self.fs.read_file(inode)?;

        // Copy to buffer from current position
        let to_read = buffer.len().min(data.len().saturating_sub(position));
        buffer[..to_read].copy_from_slice(&data[position..position + to_read]);

        // Update position
        if let Some(file) = self.file_table.get_mut(fd) {
            file.advance(to_read);
        }

        Ok(to_read)
    }

    /// Writes to a file.
    pub fn write(&mut self, fd: FileDescriptor, data: &[u8]) -> Result<usize> {
        let file = self
            .file_table
            .get(fd)
            .ok_or(VfsError::InvalidFd(fd))?;

        if !file.can_write() {
            return Err(
                VfsError::PermissionDenied("File not open for writing".to_string()).into(),
            );
        }

        let inode = file.inode;
        let position = file.position;

        // Read existing contents
        let mut contents = self.fs.read_file(inode).unwrap_or_default();

        // Extend if needed
        if position + data.len() > contents.len() {
            contents.resize(position + data.len(), 0);
        }

        // Write data at position
        contents[position..position + data.len()].copy_from_slice(data);

        // Write back
        self.fs.write_file(inode, contents)?;

        // Update position
        if let Some(file) = self.file_table.get_mut(fd) {
            file.advance(data.len());
        }

        Ok(data.len())
    }

    /// Creates a new file.
    pub fn create(&mut self, path: &str) -> Result<InodeNumber> {
        // Split path into parent and name
        let (parent_path, name) = self.split_path(path);

        let parent = self
            .fs
            .resolve_path(parent_path)
            .ok_or_else(|| VfsError::FileNotFound(parent_path.to_string()))?;

        self.fs.create_file(parent, name.to_string())
    }

    /// Creates a new directory.
    pub fn mkdir(&mut self, path: &str) -> Result<InodeNumber> {
        // Split path into parent and name
        let (parent_path, name) = self.split_path(path);

        let parent = self
            .fs
            .resolve_path(parent_path)
            .ok_or_else(|| VfsError::FileNotFound(parent_path.to_string()))?;

        self.fs.create_directory(parent, name.to_string())
    }

    /// Lists directory contents.
    pub fn readdir(&self, path: &str) -> Result<Vec<(String, InodeNumber)>> {
        let inode = self
            .fs
            .resolve_path(path)
            .ok_or_else(|| VfsError::FileNotFound(path.to_string()))?;

        self.fs.list_directory(inode)
    }

    /// Deletes a file or directory.
    pub fn unlink(&mut self, path: &str) -> Result<()> {
        let (parent_path, name) = self.split_path(path);

        let parent = self
            .fs
            .resolve_path(parent_path)
            .ok_or_else(|| VfsError::FileNotFound(parent_path.to_string()))?;

        self.fs.delete(parent, name)
    }

    /// Changes current working directory.
    pub fn chdir(&mut self, path: &str) -> Result<()> {
        let inode = self
            .fs
            .resolve_path(path)
            .ok_or_else(|| VfsError::FileNotFound(path.to_string()))?;

        // Check if it's a directory
        let inode_meta = self
            .fs
            .get_inode(inode)
            .ok_or_else(|| VfsError::FileNotFound(path.to_string()))?;

        if !inode_meta.is_directory() {
            return Err(VfsError::NotADirectory(path.to_string()).into());
        }

        self.cwd = inode;
        Ok(())
    }

    /// Gets current working directory inode.
    pub fn getcwd(&self) -> InodeNumber {
        self.cwd
    }

    /// Splits a path into parent and name.
    fn split_path<'a>(&self, path: &'a str) -> (&'a str, &'a str) {
        if let Some(pos) = path.rfind('/') {
            let parent = if pos == 0 { "/" } else { &path[..pos] };
            let name = &path[pos + 1..];
            (parent, name)
        } else {
            ("/", path)
        }
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_creation() {
        let vfs = Vfs::new();
        assert_eq!(vfs.getcwd(), 1); // Root is inode 1
    }

    #[test]
    fn test_create_file() {
        let mut vfs = Vfs::new();

        let inode = vfs.create("/test.txt").unwrap();
        assert!(inode > 0);
    }

    #[test]
    fn test_mkdir() {
        let mut vfs = Vfs::new();

        let inode = vfs.mkdir("/testdir").unwrap();
        assert!(inode > 0);
    }

    #[test]
    fn test_open_close() {
        let mut vfs = Vfs::new();

        vfs.create("/test.txt").unwrap();

        let fd = vfs.open("/test.txt", OpenMode::ReadWrite).unwrap();
        assert!(fd >= 3); // After std streams

        vfs.close(fd).unwrap();
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
    fn test_readdir() {
        let mut vfs = Vfs::new();

        vfs.create("/file1.txt").unwrap();
        vfs.create("/file2.txt").unwrap();
        vfs.mkdir("/subdir").unwrap();

        let entries = vfs.readdir("/").unwrap();
        assert_eq!(entries.len(), 5); // ., .., file1.txt, file2.txt, subdir
    }

    #[test]
    fn test_chdir() {
        let mut vfs = Vfs::new();

        vfs.mkdir("/testdir").unwrap();
        vfs.chdir("/testdir").unwrap();

        assert_ne!(vfs.getcwd(), 1); // Not root anymore
    }

    #[test]
    fn test_unlink() {
        let mut vfs = Vfs::new();

        vfs.create("/test.txt").unwrap();
        vfs.unlink("/test.txt").unwrap();

        // Should not exist anymore
        assert!(vfs.open("/test.txt", OpenMode::ReadOnly).is_err());
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
}
