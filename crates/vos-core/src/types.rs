//! Core types used throughout VOS.
//!
//! This module defines the fundamental types that represent the building blocks
//! of the virtual computer system, including words, addresses, and bytes.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-bit word - the native data type of the VOS CPU.
///
/// Words are used for:
/// - Register values
/// - Memory addresses
/// - Integer arithmetic
/// - Instruction encoding
///
/// # Examples
///
/// ```
/// use vos_core::types::Word;
///
/// let value: Word = 0x1234_5678;
/// assert_eq!(value, 305419896);
/// ```
pub type Word = u32;

/// A memory address in the VOS system.
///
/// The VOS architecture uses 32-bit addressing, allowing for a 4GB address space.
/// Memory is byte-addressable, but most operations work with word-aligned addresses.
///
/// # Examples
///
/// ```
/// use vos_core::types::Address;
///
/// let stack_pointer: Address = 0x00100000;
/// let heap_start: Address = 0x40000000;
/// ```
pub type Address = u32;

/// A single byte of data.
///
/// Used for byte-level memory operations and I/O.
///
/// # Examples
///
/// ```
/// use vos_core::types::Byte;
///
/// let character: Byte = b'A';
/// assert_eq!(character, 65);
/// ```
pub type Byte = u8;

/// Register index (0-15).
///
/// The VOS CPU has 16 general-purpose registers.
///
/// # Special Registers
///
/// - R0: Always reads as zero, writes are ignored
/// - R1-R14: General purpose
/// - R15: Stack pointer (by convention)
///
/// # Examples
///
/// ```
/// use vos_core::types::RegisterIndex;
///
/// let reg_zero: RegisterIndex = 0;
/// let stack_pointer: RegisterIndex = 15;
/// ```
pub type RegisterIndex = u8;

/// A range of memory addresses.
///
/// Used to describe contiguous regions of memory, such as segments or memory-mapped I/O regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AddressRange {
    /// Start address (inclusive)
    pub start: Address,
    /// End address (exclusive)
    pub end: Address,
}

impl AddressRange {
    /// Creates a new address range.
    ///
    /// # Panics
    ///
    /// Panics if `start >= end`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_core::types::AddressRange;
    ///
    /// let kernel_space = AddressRange::new(0x00000000, 0x00200000);
    /// assert_eq!(kernel_space.size(), 0x00200000);
    /// ```
    pub fn new(start: Address, end: Address) -> Self {
        assert!(start < end, "Invalid address range: start must be less than end");
        Self { start, end }
    }

    /// Returns the size of the address range in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_core::types::AddressRange;
    ///
    /// let range = AddressRange::new(0x1000, 0x2000);
    /// assert_eq!(range.size(), 0x1000); // 4KB
    /// ```
    pub fn size(&self) -> usize {
        (self.end - self.start) as usize
    }

    /// Checks if an address is within this range.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_core::types::AddressRange;
    ///
    /// let range = AddressRange::new(0x1000, 0x2000);
    /// assert!(range.contains(0x1500));
    /// assert!(!range.contains(0x2000)); // end is exclusive
    /// ```
    pub fn contains(&self, address: Address) -> bool {
        address >= self.start && address < self.end
    }

    /// Checks if this range overlaps with another range.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_core::types::AddressRange;
    ///
    /// let range1 = AddressRange::new(0x1000, 0x2000);
    /// let range2 = AddressRange::new(0x1500, 0x2500);
    /// let range3 = AddressRange::new(0x3000, 0x4000);
    ///
    /// assert!(range1.overlaps(&range2));
    /// assert!(!range1.overlaps(&range3));
    /// ```
    pub fn overlaps(&self, other: &AddressRange) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl fmt::Display for AddressRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08X}..0x{:08X}", self.start, self.end)
    }
}

/// Memory regions in the VOS address space.
///
/// The VOS memory layout divides the 4GB address space into specific regions
/// for different purposes.
pub mod memory_regions {
    use super::{Address, AddressRange};

    /// Interrupt Vector Table (IVT) - 4KB
    ///
    /// Contains 256 interrupt vectors, each 16 bytes.
    pub const IVT: AddressRange = AddressRange {
        start: 0x0000_0000,
        end: 0x0000_1000,
    };

    /// Kernel code and data - ~1MB
    ///
    /// Contains the kernel executable code and static data.
    pub const KERNEL_CODE: AddressRange = AddressRange {
        start: 0x0000_1000,
        end: 0x0010_0000,
    };

    /// Kernel stack - 1MB
    ///
    /// Stack space for kernel operations and interrupt handlers.
    pub const KERNEL_STACK: AddressRange = AddressRange {
        start: 0x0010_0000,
        end: 0x0020_0000,
    };

    /// User space - ~1GB
    ///
    /// Program code and data for user processes.
    pub const USER_SPACE: AddressRange = AddressRange {
        start: 0x0020_0000,
        end: 0x4000_0000,
    };

    /// Heap - 1GB
    ///
    /// Dynamic memory allocation region.
    pub const HEAP: AddressRange = AddressRange {
        start: 0x4000_0000,
        end: 0x8000_0000,
    };

    /// Memory-mapped I/O - 1GB
    ///
    /// Region for I/O device registers and buffers.
    pub const MMIO: AddressRange = AddressRange {
        start: 0x8000_0000,
        end: 0xC000_0000,
    };

    /// Reserved - 1GB
    ///
    /// Reserved for future use.
    pub const RESERVED: AddressRange = AddressRange {
        start: 0xC000_0000,
        end: 0xFFFF_FFFF,
    };

    /// Returns the memory region containing the given address, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_core::types::memory_regions;
    ///
    /// let region = memory_regions::get_region(0x00000500);
    /// assert!(region.is_some());
    /// ```
    pub fn get_region(address: Address) -> Option<AddressRange> {
        let regions = [IVT, KERNEL_CODE, KERNEL_STACK, USER_SPACE, HEAP, MMIO, RESERVED];
        regions.iter().find(|r| r.contains(address)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_range_new() {
        let range = AddressRange::new(0x1000, 0x2000);
        assert_eq!(range.start, 0x1000);
        assert_eq!(range.end, 0x2000);
    }

    #[test]
    #[should_panic(expected = "Invalid address range")]
    fn test_address_range_invalid() {
        AddressRange::new(0x2000, 0x1000);
    }

    #[test]
    fn test_address_range_size() {
        let range = AddressRange::new(0x1000, 0x2000);
        assert_eq!(range.size(), 0x1000);
    }

    #[test]
    fn test_address_range_contains() {
        let range = AddressRange::new(0x1000, 0x2000);
        assert!(range.contains(0x1000));
        assert!(range.contains(0x1500));
        assert!(range.contains(0x1FFF));
        assert!(!range.contains(0x2000));
        assert!(!range.contains(0x0FFF));
    }

    #[test]
    fn test_address_range_overlaps() {
        let range1 = AddressRange::new(0x1000, 0x2000);
        let range2 = AddressRange::new(0x1500, 0x2500);
        let range3 = AddressRange::new(0x3000, 0x4000);

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
        assert!(!range1.overlaps(&range3));
        assert!(!range3.overlaps(&range1));
    }

    #[test]
    fn test_memory_regions() {
        assert!(memory_regions::IVT.contains(0x0000_0000));
        assert!(memory_regions::KERNEL_CODE.contains(0x0000_1000));
        assert!(memory_regions::USER_SPACE.contains(0x0020_0000));
        assert!(memory_regions::MMIO.contains(0x8000_0000));
    }
}
