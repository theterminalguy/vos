//! Simple filesystem implementation.
//!
//! A basic inode-based filesystem with directories and files.

use super::directory::Directory;
use super::inode::{Inode, InodeNumber, InodeType};
use std::collections::HashMap;
use vos_core::Result;

/// Simple filesystem.
///
/// In-memory filesystem with inodes and directories.
#[derive(Debug)]
pub struct SimpleFs {
    /// Inode table (maps inode number to inode)
    inodes: HashMap<InodeNumber, Inode>,

    /// Directory contents (maps inode number to directory)
    directories: HashMap<InodeNumber, Directory>,

    /// File contents (maps inode number to file data)
    files: HashMap<InodeNumber, Vec<u8>>,

    /// Next inode number to assign
    next_inode: InodeNumber,

    /// Current time (in cycles)
    current_time: u64,

    /// Root directory inode
    root_inode: InodeNumber,
}

impl SimpleFs {
    /// Creates a new filesystem.
    pub fn new() -> Self {
        let mut fs = Self {
            inodes: HashMap::new(),
            directories: HashMap::new(),
            files: HashMap::new(),
            next_inode: 1,
            current_time: 0,
            root_inode: 1,
        };

        // Create root directory (inode 1)
        let root_inode = fs.allocate_inode(InodeType::Directory);
        let root_dir = Directory::with_dots(root_inode, root_inode);
        fs.directories.insert(root_inode, root_dir);

        fs
    }

    /// Allocates a new inode.
    fn allocate_inode(&mut self, inode_type: InodeType) -> InodeNumber {
        let number = self.next_inode;
        self.next_inode += 1;

        let inode = Inode::new(number, inode_type, self.current_time);
        self.inodes.insert(number, inode);

        number
    }

    /// Gets an inode by number.
    pub fn get_inode(&self, inode: InodeNumber) -> Option<&Inode> {
        self.inodes.get(&inode)
    }

    /// Gets a mutable inode by number.
    pub fn get_inode_mut(&mut self, inode: InodeNumber) -> Option<&mut Inode> {
        self.inodes.get_mut(&inode)
    }

    /// Creates a file in a directory.
    pub fn create_file(&mut self, parent: InodeNumber, name: String) -> Result<InodeNumber> {
        // Allocate inode for file
        let inode = self.allocate_inode(InodeType::File);

        // Add to parent directory
        if let Some(dir) = self.directories.get_mut(&parent) {
            dir.add_entry(name, inode);
        } else {
            return Err("Parent is not a directory".into());
        }

        // Initialize empty file content
        self.files.insert(inode, Vec::new());

        Ok(inode)
    }

    /// Creates a directory.
    pub fn create_directory(&mut self, parent: InodeNumber, name: String) -> Result<InodeNumber> {
        // Allocate inode for directory
        let inode = self.allocate_inode(InodeType::Directory);

        // Add to parent directory
        if let Some(parent_dir) = self.directories.get_mut(&parent) {
            parent_dir.add_entry(name, inode);
        } else {
            return Err("Parent is not a directory".into());
        }

        // Create directory with . and ..
        let dir = Directory::with_dots(inode, parent);
        self.directories.insert(inode, dir);

        Ok(inode)
    }

    /// Reads file contents.
    pub fn read_file(&mut self, inode: InodeNumber) -> Result<Vec<u8>> {
        // Update access time
        let current_time = self.current_time;
        if let Some(inode_meta) = self.get_inode_mut(inode) {
            inode_meta.access(current_time);
        }

        self.files
            .get(&inode)
            .cloned()
            .ok_or_else(|| "File not found".into())
    }

    /// Writes file contents.
    pub fn write_file(&mut self, inode: InodeNumber, data: Vec<u8>) -> Result<()> {
        // Update inode metadata
        let current_time = self.current_time;
        if let Some(inode_meta) = self.get_inode_mut(inode) {
            inode_meta.size = data.len();
            inode_meta.touch(current_time);
        } else {
            return Err("Inode not found".into());
        }

        self.files.insert(inode, data);
        Ok(())
    }

    /// Lists directory contents.
    pub fn list_directory(&self, inode: InodeNumber) -> Result<Vec<(String, InodeNumber)>> {
        if let Some(dir) = self.directories.get(&inode) {
            let entries = dir
                .list()
                .into_iter()
                .map(|e| (e.name, e.inode))
                .collect();
            Ok(entries)
        } else {
            Err("Not a directory".into())
        }
    }

    /// Looks up a name in a directory.
    pub fn lookup(&self, parent: InodeNumber, name: &str) -> Option<InodeNumber> {
        self.directories.get(&parent)?.lookup(name)
    }

    /// Resolves a path to an inode.
    pub fn resolve_path(&self, path: &str) -> Option<InodeNumber> {
        if path == "/" {
            return Some(self.root_inode);
        }

        let mut current = self.root_inode;

        for component in path.split('/').filter(|s| !s.is_empty()) {
            current = self.lookup(current, component)?;
        }

        Some(current)
    }

    /// Deletes a file or empty directory.
    pub fn delete(&mut self, parent: InodeNumber, name: &str) -> Result<()> {
        // Remove from parent directory
        let inode = self
            .directories
            .get_mut(&parent)
            .and_then(|dir| dir.remove_entry(name))
            .ok_or("Entry not found")?;

        // Check if it's a directory
        if let Some(dir) = self.directories.get(&inode) {
            if !dir.is_empty() {
                return Err("Directory not empty".into());
            }
            self.directories.remove(&inode);
        }

        // Remove file contents
        self.files.remove(&inode);

        // Remove inode
        self.inodes.remove(&inode);

        Ok(())
    }

    /// Gets the root inode number.
    pub fn root(&self) -> InodeNumber {
        self.root_inode
    }

    /// Updates the current time.
    pub fn set_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Returns filesystem statistics.
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.inodes.len(),
            self.directories.len(),
            self.files.len(),
        )
    }
}

impl Default for SimpleFs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_creation() {
        let fs = SimpleFs::new();
        assert_eq!(fs.root(), 1);

        let (inodes, dirs, files) = fs.stats();
        assert_eq!(inodes, 1); // Root inode
        assert_eq!(dirs, 1);   // Root directory
        assert_eq!(files, 0);
    }

    #[test]
    fn test_create_file() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        let inode = fs.create_file(root, "test.txt".to_string()).unwrap();

        assert!(fs.get_inode(inode).is_some());
        assert!(fs.get_inode(inode).unwrap().is_file());
    }

    #[test]
    fn test_create_directory() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        let inode = fs.create_directory(root, "subdir".to_string()).unwrap();

        assert!(fs.get_inode(inode).is_some());
        assert!(fs.get_inode(inode).unwrap().is_directory());
    }

    #[test]
    fn test_read_write_file() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        let inode = fs.create_file(root, "test.txt".to_string()).unwrap();

        // Write
        let data = b"Hello, World!".to_vec();
        fs.write_file(inode, data.clone()).unwrap();

        // Read
        let read_data = fs.read_file(inode).unwrap();
        assert_eq!(read_data, data);

        // Check size
        let file_inode = fs.get_inode(inode).unwrap();
        assert_eq!(file_inode.size, data.len());
    }

    #[test]
    fn test_list_directory() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        fs.create_file(root, "file1.txt".to_string()).unwrap();
        fs.create_file(root, "file2.txt".to_string()).unwrap();
        fs.create_directory(root, "subdir".to_string()).unwrap();

        let entries = fs.list_directory(root).unwrap();

        // Should have ., .., file1.txt, file2.txt, subdir
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_lookup() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        let file_inode = fs.create_file(root, "test.txt".to_string()).unwrap();

        let found = fs.lookup(root, "test.txt");
        assert_eq!(found, Some(file_inode));

        let not_found = fs.lookup(root, "notfound.txt");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_resolve_path() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        let dir_inode = fs.create_directory(root, "subdir".to_string()).unwrap();
        let file_inode = fs.create_file(dir_inode, "file.txt".to_string()).unwrap();

        assert_eq!(fs.resolve_path("/"), Some(root));
        assert_eq!(fs.resolve_path("/subdir"), Some(dir_inode));
        assert_eq!(fs.resolve_path("/subdir/file.txt"), Some(file_inode));
        assert_eq!(fs.resolve_path("/notfound"), None);
    }

    #[test]
    fn test_delete_file() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        fs.create_file(root, "test.txt".to_string()).unwrap();

        fs.delete(root, "test.txt").unwrap();

        assert!(fs.lookup(root, "test.txt").is_none());
    }

    #[test]
    fn test_delete_empty_directory() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        fs.create_directory(root, "emptydir".to_string()).unwrap();

        fs.delete(root, "emptydir").unwrap();

        assert!(fs.lookup(root, "emptydir").is_none());
    }

    #[test]
    fn test_delete_nonempty_directory_fails() {
        let mut fs = SimpleFs::new();
        let root = fs.root();

        let dir_inode = fs.create_directory(root, "dir".to_string()).unwrap();
        fs.create_file(dir_inode, "file.txt".to_string()).unwrap();

        let result = fs.delete(root, "dir");
        assert!(result.is_err());
    }
}
