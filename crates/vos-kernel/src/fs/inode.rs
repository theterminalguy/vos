//! Inode implementation.
//!
//! An inode (index node) stores metadata about a file or directory.

use vos_core::Address;

/// Inode number (unique identifier).
pub type InodeNumber = u32;

/// Inode type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Device file
    Device,
}

/// Inode - stores file/directory metadata.
///
/// Similar to Unix inodes, stores everything about a file except its name.
#[derive(Debug, Clone)]
pub struct Inode {
    /// Unique inode number
    pub number: InodeNumber,

    /// Type (file, directory, device)
    pub inode_type: InodeType,

    /// Size in bytes
    pub size: usize,

    /// Number of hard links
    pub link_count: u32,

    /// Permissions (Unix-style: rwxrwxrwx)
    pub permissions: u16,

    /// Owner user ID
    pub uid: u32,

    /// Owner group ID
    pub gid: u32,

    /// Creation time (in cycles)
    pub created: u64,

    /// Last modification time
    pub modified: u64,

    /// Last access time
    pub accessed: u64,

    /// Block pointers (direct blocks)
    pub blocks: Vec<Address>,

    /// Indirect block pointer (for large files)
    pub indirect_block: Option<Address>,
}

impl Inode {
    /// Creates a new inode.
    pub fn new(number: InodeNumber, inode_type: InodeType, created: u64) -> Self {
        let permissions = match inode_type {
            InodeType::File => 0o644,      // rw-r--r--
            InodeType::Directory => 0o755, // rwxr-xr-x
            InodeType::Device => 0o666,    // rw-rw-rw-
        };

        Self {
            number,
            inode_type,
            size: 0,
            link_count: 1,
            permissions,
            uid: 0,
            gid: 0,
            created,
            modified: created,
            accessed: created,
            blocks: Vec::new(),
            indirect_block: None,
        }
    }

    /// Checks if inode is a file.
    pub fn is_file(&self) -> bool {
        self.inode_type == InodeType::File
    }

    /// Checks if inode is a directory.
    pub fn is_directory(&self) -> bool {
        self.inode_type == InodeType::Directory
    }

    /// Checks if inode is a device.
    pub fn is_device(&self) -> bool {
        self.inode_type == InodeType::Device
    }

    /// Adds a block to the inode.
    pub fn add_block(&mut self, block_addr: Address) {
        self.blocks.push(block_addr);
    }

    /// Gets the number of blocks used.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Updates modification time.
    pub fn touch(&mut self, time: u64) {
        self.modified = time;
    }

    /// Updates access time.
    pub fn access(&mut self, time: u64) {
        self.accessed = time;
    }

    /// Checks if user has read permission.
    pub fn can_read(&self, uid: u32) -> bool {
        if uid == self.uid {
            (self.permissions & 0o400) != 0 // Owner read
        } else {
            (self.permissions & 0o004) != 0 // Other read
        }
    }

    /// Checks if user has write permission.
    pub fn can_write(&self, uid: u32) -> bool {
        if uid == self.uid {
            (self.permissions & 0o200) != 0 // Owner write
        } else {
            (self.permissions & 0o002) != 0 // Other write
        }
    }

    /// Checks if user has execute permission.
    pub fn can_execute(&self, uid: u32) -> bool {
        if uid == self.uid {
            (self.permissions & 0o100) != 0 // Owner execute
        } else {
            (self.permissions & 0o001) != 0 // Other execute
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inode_creation() {
        let inode = Inode::new(1, InodeType::File, 1000);

        assert_eq!(inode.number, 1);
        assert_eq!(inode.inode_type, InodeType::File);
        assert_eq!(inode.size, 0);
        assert_eq!(inode.link_count, 1);
        assert_eq!(inode.created, 1000);
    }

    #[test]
    fn test_inode_type_checks() {
        let file = Inode::new(1, InodeType::File, 0);
        assert!(file.is_file());
        assert!(!file.is_directory());

        let dir = Inode::new(2, InodeType::Directory, 0);
        assert!(dir.is_directory());
        assert!(!dir.is_file());
    }

    #[test]
    fn test_add_blocks() {
        let mut inode = Inode::new(1, InodeType::File, 0);

        assert_eq!(inode.block_count(), 0);

        inode.add_block(0x1000);
        inode.add_block(0x2000);

        assert_eq!(inode.block_count(), 2);
        assert_eq!(inode.blocks[0], 0x1000);
        assert_eq!(inode.blocks[1], 0x2000);
    }

    #[test]
    fn test_permissions() {
        let mut inode = Inode::new(1, InodeType::File, 0);
        inode.uid = 100;
        inode.permissions = 0o644; // rw-r--r--

        // Owner can read and write
        assert!(inode.can_read(100));
        assert!(inode.can_write(100));
        assert!(!inode.can_execute(100));

        // Others can only read
        assert!(inode.can_read(200));
        assert!(!inode.can_write(200));
        assert!(!inode.can_execute(200));
    }

    #[test]
    fn test_touch() {
        let mut inode = Inode::new(1, InodeType::File, 1000);
        assert_eq!(inode.modified, 1000);

        inode.touch(2000);
        assert_eq!(inode.modified, 2000);
    }
}
