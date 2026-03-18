//! Memory Management Unit (MMU) implementation.
//!
//! Provides virtual to physical address translation with simple paging.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vos_core::{Address, MemoryError, Result};

/// Page size in bytes (4KB).
pub const PAGE_SIZE: usize = 4096;

/// Page number type.
pub type PageNumber = u32;

/// Page frame number type.
pub type FrameNumber = u32;

/// Converts a virtual address to a page number and offset.
///
/// # Examples
///
/// ```
/// use vos_memory::mmu::{page_number, page_offset, PAGE_SIZE};
///
/// let address = 0x1234;
/// let page = page_number(address);
/// let offset = page_offset(address);
///
/// // Reconstruct address
/// assert_eq!((page as usize * PAGE_SIZE) + offset as usize, address as usize);
/// ```
pub fn page_number(address: Address) -> PageNumber {
    (address / PAGE_SIZE as u32) as PageNumber
}

/// Extracts the offset within a page from an address.
pub fn page_offset(address: Address) -> u32 {
    address % PAGE_SIZE as u32
}

/// Page table entry.
///
/// Contains the mapping from a virtual page to a physical frame,
/// along with permission and status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageTableEntry {
    /// Physical frame number
    pub frame: FrameNumber,

    /// Is the page present in memory?
    pub present: bool,

    /// Can the page be written to?
    pub writable: bool,

    /// Can the page be executed?
    pub executable: bool,

    /// Has the page been accessed?
    pub accessed: bool,

    /// Has the page been modified (dirty)?
    pub dirty: bool,
}

impl PageTableEntry {
    /// Creates a new page table entry.
    pub fn new(frame: FrameNumber) -> Self {
        Self {
            frame,
            present: true,
            writable: true,
            executable: true,
            accessed: false,
            dirty: false,
        }
    }

    /// Creates a read-only page table entry.
    pub fn read_only(frame: FrameNumber) -> Self {
        Self {
            frame,
            present: true,
            writable: false,
            executable: false,
            accessed: false,
            dirty: false,
        }
    }

    /// Creates an executable page table entry.
    pub fn executable(frame: FrameNumber) -> Self {
        Self {
            frame,
            present: true,
            writable: false,
            executable: true,
            accessed: false,
            dirty: false,
        }
    }
}

/// Simple page table using a hash map.
///
/// Maps virtual page numbers to physical frame numbers with permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTable {
    /// Page table entries
    entries: HashMap<PageNumber, PageTableEntry>,
}

impl PageTable {
    /// Creates a new empty page table.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Maps a virtual page to a physical frame.
    pub fn map(&mut self, page: PageNumber, entry: PageTableEntry) {
        self.entries.insert(page, entry);
    }

    /// Unmaps a virtual page.
    pub fn unmap(&mut self, page: PageNumber) {
        self.entries.remove(&page);
    }

    /// Looks up a page table entry.
    pub fn lookup(&self, page: PageNumber) -> Option<&PageTableEntry> {
        self.entries.get(&page)
    }

    /// Looks up a page table entry mutably.
    pub fn lookup_mut(&mut self, page: PageNumber) -> Option<&mut PageTableEntry> {
        self.entries.get_mut(&page)
    }

    /// Clears all mappings.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of mapped pages.
    pub fn mapped_pages(&self) -> usize {
        self.entries.len()
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory Management Unit.
///
/// Translates virtual addresses to physical addresses using page tables.
///
/// # Examples
///
/// ```
/// use vos_memory::mmu::{Mmu, PageTableEntry};
///
/// let mut mmu = Mmu::new();
///
/// // Identity map first 1MB (256 pages of 4KB each)
/// mmu.identity_map(0, 256);
///
/// // Translate virtual to physical
/// let physical = mmu.translate(0x1234, false).unwrap();
/// assert_eq!(physical, 0x1234); // Identity mapped
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmu {
    /// Page table
    page_table: PageTable,

    /// Is paging enabled?
    paging_enabled: bool,
}

impl Mmu {
    /// Creates a new MMU with paging disabled.
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            paging_enabled: false,
        }
    }

    /// Enables paging.
    pub fn enable_paging(&mut self) {
        self.paging_enabled = true;
    }

    /// Disables paging (all addresses are identity mapped).
    pub fn disable_paging(&mut self) {
        self.paging_enabled = false;
    }

    /// Returns true if paging is enabled.
    pub fn is_paging_enabled(&self) -> bool {
        self.paging_enabled
    }

    /// Translates a virtual address to a physical address.
    ///
    /// # Parameters
    ///
    /// - `virtual_address`: The virtual address to translate
    /// - `write`: Is this a write access?
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::PageFault` if:
    /// - The page is not mapped
    /// - The page is not present
    /// - Write access to a read-only page
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::mmu::Mmu;
    ///
    /// let mut mmu = Mmu::new();
    /// mmu.identity_map(0, 1); // Map page 0
    /// mmu.enable_paging();
    ///
    /// let physical = mmu.translate(0x100, false).unwrap();
    /// assert_eq!(physical, 0x100);
    /// ```
    pub fn translate(&mut self, virtual_address: Address, write: bool) -> Result<Address> {
        // If paging is disabled, use identity mapping
        if !self.paging_enabled {
            return Ok(virtual_address);
        }

        let page = page_number(virtual_address);
        let offset = page_offset(virtual_address);

        // Look up page table entry
        let entry = self
            .page_table
            .lookup_mut(page)
            .ok_or(MemoryError::PageFault {
                address: virtual_address,
                present: false,
                write,
            })?;

        // Check if page is present
        if !entry.present {
            return Err(MemoryError::PageFault {
                address: virtual_address,
                present: false,
                write,
            }
            .into());
        }

        // Check write permission
        if write && !entry.writable {
            return Err(MemoryError::PageFault {
                address: virtual_address,
                present: true,
                write: true,
            }
            .into());
        }

        // Update accessed and dirty flags
        entry.accessed = true;
        if write {
            entry.dirty = true;
        }

        // Calculate physical address
        let physical_address = (entry.frame as usize * PAGE_SIZE) as Address + offset;

        Ok(physical_address)
    }

    /// Maps a virtual page to a physical frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::mmu::{Mmu, PageTableEntry};
    ///
    /// let mut mmu = Mmu::new();
    ///
    /// // Map virtual page 10 to physical frame 20
    /// mmu.map_page(10, PageTableEntry::new(20));
    /// ```
    pub fn map_page(&mut self, page: PageNumber, entry: PageTableEntry) {
        self.page_table.map(page, entry);
    }

    /// Unmaps a virtual page.
    pub fn unmap_page(&mut self, page: PageNumber) {
        self.page_table.unmap(page);
    }

    /// Identity maps a range of pages (virtual page == physical frame).
    ///
    /// # Parameters
    ///
    /// - `start_page`: First page to map
    /// - `count`: Number of pages to map
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::mmu::Mmu;
    ///
    /// let mut mmu = Mmu::new();
    ///
    /// // Identity map first 256 pages (1MB)
    /// mmu.identity_map(0, 256);
    /// ```
    pub fn identity_map(&mut self, start_page: PageNumber, count: usize) {
        for i in 0..count {
            let page = start_page + i as PageNumber;
            self.map_page(page, PageTableEntry::new(page));
        }
    }

    /// Identity maps a range of pages as read-only.
    pub fn identity_map_readonly(&mut self, start_page: PageNumber, count: usize) {
        for i in 0..count {
            let page = start_page + i as PageNumber;
            self.map_page(page, PageTableEntry::read_only(page));
        }
    }

    /// Identity maps a range of pages as executable.
    pub fn identity_map_executable(&mut self, start_page: PageNumber, count: usize) {
        for i in 0..count {
            let page = start_page + i as PageNumber;
            self.map_page(page, PageTableEntry::executable(page));
        }
    }

    /// Clears all page table entries.
    pub fn clear(&mut self) {
        self.page_table.clear();
    }

    /// Returns the number of mapped pages.
    pub fn mapped_pages(&self) -> usize {
        self.page_table.mapped_pages()
    }

    /// Returns a reference to the page table.
    pub fn page_table(&self) -> &PageTable {
        &self.page_table
    }

    /// Returns a mutable reference to the page table.
    pub fn page_table_mut(&mut self) -> &mut PageTable {
        &mut self.page_table
    }
}

impl Default for Mmu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_number_and_offset() {
        let address = 0x1234;
        let page = page_number(address);
        let offset = page_offset(address);

        // Page 0, offset 0x1234 (since page size is 4096 = 0x1000)
        assert_eq!(page, 1);
        assert_eq!(offset, 0x234);

        // Reconstruct
        let reconstructed = (page as usize * PAGE_SIZE) as Address + offset;
        assert_eq!(reconstructed, address);
    }

    #[test]
    fn test_mmu_disabled() {
        let mut mmu = Mmu::new();

        // With paging disabled, all addresses are identity mapped
        assert_eq!(mmu.translate(0x1000, false).unwrap(), 0x1000);
        assert_eq!(mmu.translate(0x5000, false).unwrap(), 0x5000);
    }

    #[test]
    fn test_mmu_identity_map() {
        let mut mmu = Mmu::new();
        mmu.identity_map(0, 10);
        mmu.enable_paging();

        // Should work for mapped pages
        assert_eq!(mmu.translate(0x1000, false).unwrap(), 0x1000);
        assert_eq!(mmu.translate(0x5000, false).unwrap(), 0x5000);
    }

    #[test]
    fn test_mmu_page_fault() {
        let mut mmu = Mmu::new();
        mmu.identity_map(0, 10);
        mmu.enable_paging();

        // Try to access unmapped page (page 20)
        let result = mmu.translate(0x14000, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_mmu_write_protection() {
        let mut mmu = Mmu::new();

        // Map page 0 as read-only
        mmu.map_page(0, PageTableEntry::read_only(0));
        mmu.enable_paging();

        // Read should work
        assert!(mmu.translate(0x100, false).is_ok());

        // Write should fail
        assert!(mmu.translate(0x100, true).is_err());
    }

    #[test]
    fn test_mmu_remap() {
        let mut mmu = Mmu::new();
        mmu.enable_paging();

        // Map virtual page 10 to physical frame 20
        mmu.map_page(10, PageTableEntry::new(20));

        let virtual_addr = 10 * PAGE_SIZE as Address + 0x100;
        let physical_addr = mmu.translate(virtual_addr, false).unwrap();

        // Should translate to frame 20
        let expected = 20 * PAGE_SIZE as Address + 0x100;
        assert_eq!(physical_addr, expected);
    }

    #[test]
    fn test_page_table_entry_flags() {
        let mut mmu = Mmu::new();
        mmu.map_page(0, PageTableEntry::new(0));
        mmu.enable_paging();

        // Access the page
        mmu.translate(0x100, false).unwrap();

        let entry = mmu.page_table().lookup(0).unwrap();
        assert!(entry.accessed);
        assert!(!entry.dirty);

        // Write to the page
        mmu.translate(0x100, true).unwrap();

        let entry = mmu.page_table().lookup(0).unwrap();
        assert!(entry.accessed);
        assert!(entry.dirty);
    }

    #[test]
    fn test_unmap() {
        let mut mmu = Mmu::new();
        mmu.identity_map(0, 10);
        mmu.enable_paging();

        // Page 5 is mapped
        assert!(mmu.translate(0x5000, false).is_ok());

        // Unmap page 5
        mmu.unmap_page(5);

        // Should now fail
        assert!(mmu.translate(0x5000, false).is_err());
    }

    #[test]
    fn test_clear() {
        let mut mmu = Mmu::new();
        mmu.identity_map(0, 10);

        assert_eq!(mmu.mapped_pages(), 10);

        mmu.clear();

        assert_eq!(mmu.mapped_pages(), 0);
    }

    #[test]
    fn test_readonly_mapping() {
        let mut mmu = Mmu::new();
        mmu.identity_map_readonly(0, 5);
        mmu.enable_paging();

        // Read works
        assert!(mmu.translate(0x1000, false).is_ok());

        // Write fails
        assert!(mmu.translate(0x1000, true).is_err());
    }

    #[test]
    fn test_executable_mapping() {
        let mut mmu = Mmu::new();
        mmu.identity_map_executable(0, 5);

        let entry = mmu.page_table().lookup(0).unwrap();
        assert!(entry.executable);
        assert!(!entry.writable);
    }
}
