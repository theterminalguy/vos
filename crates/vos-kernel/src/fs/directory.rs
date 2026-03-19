//! Directory implementation.

use super::inode::InodeNumber;

/// Directory entry.
///
/// Maps a name to an inode number.
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// File/directory name
    pub name: String,

    /// Inode number
    pub inode: InodeNumber,
}

impl DirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(name: String, inode: InodeNumber) -> Self {
        Self { name, inode }
    }
}

/// Directory - contains directory entries.
#[derive(Debug, Clone)]
pub struct Directory {
    /// Directory entries
    entries: Vec<DirectoryEntry>,
}

impl Directory {
    /// Creates an empty directory.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Creates a directory with . and .. entries.
    pub fn with_dots(self_inode: InodeNumber, parent_inode: InodeNumber) -> Self {
        let mut dir = Self::new();
        dir.add_entry(".".to_string(), self_inode);
        dir.add_entry("..".to_string(), parent_inode);
        dir
    }

    /// Adds an entry to the directory.
    pub fn add_entry(&mut self, name: String, inode: InodeNumber) {
        // Remove existing entry with same name
        self.remove_entry(&name);

        self.entries.push(DirectoryEntry::new(name, inode));
    }

    /// Removes an entry from the directory.
    pub fn remove_entry(&mut self, name: &str) -> Option<InodeNumber> {
        if let Some(pos) = self.entries.iter().position(|e| e.name == name) {
            let entry = self.entries.remove(pos);
            Some(entry.inode)
        } else {
            None
        }
    }

    /// Looks up an entry by name.
    pub fn lookup(&self, name: &str) -> Option<InodeNumber> {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.inode)
    }

    /// Lists all entries.
    pub fn list(&self) -> Vec<DirectoryEntry> {
        self.entries.clone()
    }

    /// Returns the number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Checks if directory is empty (excluding . and ..).
    pub fn is_empty(&self) -> bool {
        self.entries
            .iter()
            .all(|e| e.name == "." || e.name == "..")
    }

    /// Checks if an entry exists.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }
}

impl Default for Directory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_creation() {
        let dir = Directory::new();
        assert_eq!(dir.entry_count(), 0);
    }

    #[test]
    fn test_with_dots() {
        let dir = Directory::with_dots(10, 5);

        assert_eq!(dir.entry_count(), 2);
        assert_eq!(dir.lookup("."), Some(10));
        assert_eq!(dir.lookup(".."), Some(5));
    }

    #[test]
    fn test_add_entry() {
        let mut dir = Directory::new();

        dir.add_entry("file1.txt".to_string(), 100);
        dir.add_entry("file2.txt".to_string(), 101);

        assert_eq!(dir.entry_count(), 2);
        assert_eq!(dir.lookup("file1.txt"), Some(100));
        assert_eq!(dir.lookup("file2.txt"), Some(101));
    }

    #[test]
    fn test_remove_entry() {
        let mut dir = Directory::new();

        dir.add_entry("file1.txt".to_string(), 100);
        dir.add_entry("file2.txt".to_string(), 101);

        let removed = dir.remove_entry("file1.txt");
        assert_eq!(removed, Some(100));
        assert_eq!(dir.entry_count(), 1);
        assert!(dir.lookup("file1.txt").is_none());
    }

    #[test]
    fn test_lookup() {
        let mut dir = Directory::new();

        dir.add_entry("test.txt".to_string(), 42);

        assert_eq!(dir.lookup("test.txt"), Some(42));
        assert_eq!(dir.lookup("notfound.txt"), None);
    }

    #[test]
    fn test_list() {
        let mut dir = Directory::new();

        dir.add_entry("a.txt".to_string(), 1);
        dir.add_entry("b.txt".to_string(), 2);

        let entries = dir.list();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.txt");
        assert_eq!(entries[1].name, "b.txt");
    }

    #[test]
    fn test_is_empty() {
        let mut dir = Directory::with_dots(1, 1);
        assert!(dir.is_empty()); // Only . and ..

        dir.add_entry("file.txt".to_string(), 10);
        assert!(!dir.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut dir = Directory::new();
        dir.add_entry("test.txt".to_string(), 1);

        assert!(dir.contains("test.txt"));
        assert!(!dir.contains("other.txt"));
    }

    #[test]
    fn test_replace_entry() {
        let mut dir = Directory::new();

        dir.add_entry("file.txt".to_string(), 100);
        assert_eq!(dir.lookup("file.txt"), Some(100));

        // Adding same name replaces
        dir.add_entry("file.txt".to_string(), 200);
        assert_eq!(dir.lookup("file.txt"), Some(200));
        assert_eq!(dir.entry_count(), 1); // Still only one entry
    }
}
