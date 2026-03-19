//! File system module.
//!
//! Provides file and directory operations.

pub mod inode;
pub mod directory;
pub mod file;
pub mod simple_fs;
pub mod vfs;

pub use inode::{Inode, InodeType};
pub use directory::{Directory, DirectoryEntry};
pub use file::{File, OpenMode};
pub use simple_fs::SimpleFs;
pub use vfs::{Vfs, VfsError};
