//! Main memory module integrating RAM and MMU.

use serde::{Deserialize, Serialize};
use vos_core::{Address, Byte, Result, Word};

use crate::mmu::Mmu;
use crate::ram::Ram;

/// Main memory subsystem.
///
/// Integrates RAM (physical memory) with MMU (address translation).
/// Provides the memory interface required by the CPU.
///
/// # Examples
///
/// ```
/// use vos_memory::memory::Memory;
///
/// // Create 16MB of memory
/// let mut memory = Memory::new(16 * 1024 * 1024);
///
/// // Write and read
/// memory.write_word(0x1000, 0x12345678).unwrap();
/// assert_eq!(memory.read_word(0x1000).unwrap(), 0x12345678);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Physical RAM
    ram: Ram,

    /// Memory Management Unit
    mmu: Mmu,
}

impl Memory {
    /// Creates a new memory subsystem with the specified size.
    ///
    /// # Parameters
    ///
    /// - `size`: Size of physical RAM in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::memory::Memory;
    ///
    /// let memory = Memory::new(16 * 1024 * 1024); // 16MB
    /// ```
    pub fn new(size: usize) -> Self {
        let mut mmu = Mmu::new();

        // Identity map all physical memory (disabled by default)
        let pages = size / crate::mmu::PAGE_SIZE;
        mmu.identity_map(0, pages);

        Self {
            ram: Ram::new(size),
            mmu,
        }
    }

    /// Returns the size of physical memory in bytes.
    pub fn size(&self) -> usize {
        self.ram.size()
    }

    /// Returns a reference to the MMU.
    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    /// Returns a mutable reference to the MMU.
    pub fn mmu_mut(&mut self) -> &mut Mmu {
        &mut self.mmu
    }

    /// Returns a reference to the RAM.
    pub fn ram(&self) -> &Ram {
        &self.ram
    }

    /// Returns a mutable reference to the RAM.
    pub fn ram_mut(&mut self) -> &mut Ram {
        &mut self.ram
    }

    /// Reads a byte from memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Virtual address to read from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Address is out of bounds
    /// - Page fault occurs (if paging is enabled)
    pub fn read_byte(&mut self, address: Address) -> Result<Byte> {
        let physical = self.mmu.translate(address, false)?;
        self.ram.read_byte(physical)
    }

    /// Writes a byte to memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Virtual address to write to
    /// - `value`: Byte value to write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Address is out of bounds
    /// - Page fault occurs (if paging is enabled)
    /// - Write to read-only page
    pub fn write_byte(&mut self, address: Address, value: Byte) -> Result<()> {
        let physical = self.mmu.translate(address, true)?;
        self.ram.write_byte(physical, value)
    }

    /// Reads a word from memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Virtual address to read from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Address is out of bounds
    /// - Page fault occurs (if paging is enabled)
    pub fn read_word(&mut self, address: Address) -> Result<Word> {
        let physical = self.mmu.translate(address, false)?;
        self.ram.read_word(physical)
    }

    /// Writes a word to memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Virtual address to write to
    /// - `value`: Word value to write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Address is out of bounds
    /// - Page fault occurs (if paging is enabled)
    /// - Write to read-only page
    pub fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
        let physical = self.mmu.translate(address, true)?;
        self.ram.write_word(physical, value)
    }

    /// Reads multiple bytes from memory.
    pub fn read_bytes(&mut self, address: Address, buffer: &mut [Byte]) -> Result<()> {
        // For simplicity, read byte by byte (could be optimized)
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = self.read_byte(address + i as Address)?;
        }
        Ok(())
    }

    /// Writes multiple bytes to memory.
    pub fn write_bytes(&mut self, address: Address, data: &[Byte]) -> Result<()> {
        // For simplicity, write byte by byte (could be optimized)
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(address + i as Address, byte)?;
        }
        Ok(())
    }

    /// Loads data into memory at the specified address.
    ///
    /// This bypasses the MMU and writes directly to physical memory.
    /// Useful for loading programs before paging is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::memory::Memory;
    ///
    /// let mut memory = Memory::new(1024 * 1024);
    /// let program = vec![0x01, 0x02, 0x03, 0x04];
    ///
    /// memory.load(0x1000, &program).unwrap();
    /// ```
    pub fn load(&mut self, address: Address, data: &[Byte]) -> Result<()> {
        self.ram.load(address, data)
    }

    /// Clears all memory.
    pub fn clear(&mut self) {
        self.ram.clear();
    }

    /// Creates a hexdump of a memory region.
    ///
    /// # Parameters
    ///
    /// - `start`: Starting virtual address
    /// - `length`: Number of bytes to dump
    pub fn hexdump(&self, start: Address, length: usize) -> String {
        // For hexdump, we access physical memory directly
        self.ram.hexdump(start, length)
    }
}

// Implement the CPU Memory trait
impl vos_cpu::cpu::Memory for Memory {
    fn read_word(&mut self, address: Address) -> Result<Word> {
        Memory::read_word(self, address)
    }

    fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
        Memory::write_word(self, address, value)
    }

    fn read_byte(&mut self, address: Address) -> Result<Byte> {
        Memory::read_byte(self, address)
    }

    fn write_byte(&mut self, address: Address, value: Byte) -> Result<()> {
        Memory::write_byte(self, address, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(1024 * 1024);
        assert_eq!(memory.size(), 1024 * 1024);
    }

    #[test]
    fn test_byte_operations() {
        let mut memory = Memory::new(1024);

        memory.write_byte(100, 42).unwrap();
        assert_eq!(memory.read_byte(100).unwrap(), 42);
    }

    #[test]
    fn test_word_operations() {
        let mut memory = Memory::new(1024);

        memory.write_word(0x100, 0x12345678).unwrap();
        assert_eq!(memory.read_word(0x100).unwrap(), 0x12345678);
    }

    #[test]
    fn test_load() {
        let mut memory = Memory::new(1024);

        let program = vec![0xAA, 0xBB, 0xCC, 0xDD];
        memory.load(0x200, &program).unwrap();

        assert_eq!(memory.read_byte(0x200).unwrap(), 0xAA);
        assert_eq!(memory.read_byte(0x203).unwrap(), 0xDD);
    }

    #[test]
    fn test_with_paging() {
        let mut memory = Memory::new(1024 * 1024);

        // Enable paging
        memory.mmu_mut().enable_paging();

        // Should still work with identity mapping
        memory.write_word(0x1000, 0x12345678).unwrap();
        assert_eq!(memory.read_word(0x1000).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_write_bytes() {
        let mut memory = Memory::new(1024);

        let data = vec![1, 2, 3, 4, 5];
        memory.write_bytes(0x100, &data).unwrap();

        let mut buffer = vec![0; 5];
        memory.read_bytes(0x100, &mut buffer).unwrap();

        assert_eq!(buffer, data);
    }

    #[test]
    fn test_cpu_memory_trait() {
        use vos_cpu::cpu::Memory as CpuMemory;

        let mut memory = Memory::new(1024);

        // Use through CPU Memory trait
        CpuMemory::write_word(&mut memory, 0x100, 0x12345678).unwrap();
        assert_eq!(CpuMemory::read_word(&mut memory, 0x100).unwrap(), 0x12345678);
    }
}
