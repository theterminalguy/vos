//! RAM (Random Access Memory) implementation.
//!
//! Provides byte-addressable memory storage with configurable size.

use serde::{Deserialize, Serialize};
use vos_core::{Address, Byte, MemoryError, Result, Word};

/// Random Access Memory.
///
/// The RAM is byte-addressable and provides both byte and word access.
/// Memory is allocated as a contiguous vector of bytes.
///
/// # Examples
///
/// ```
/// use vos_memory::ram::Ram;
///
/// // Create 1MB of RAM
/// let mut ram = Ram::new(1024 * 1024);
///
/// // Write a word
/// ram.write_word(0x1000, 0x12345678).unwrap();
///
/// // Read it back
/// let value = ram.read_word(0x1000).unwrap();
/// assert_eq!(value, 0x12345678);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ram {
    /// Memory storage
    data: Vec<Byte>,

    /// Size of RAM in bytes
    size: usize,
}

impl Ram {
    /// Creates a new RAM with the specified size in bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: Size of RAM in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::ram::Ram;
    ///
    /// // Create 16MB of RAM
    /// let ram = Ram::new(16 * 1024 * 1024);
    /// assert_eq!(ram.size(), 16 * 1024 * 1024);
    /// ```
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            size,
        }
    }

    /// Returns the size of RAM in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Checks if an address is valid (within bounds).
    fn check_address(&self, address: Address) -> Result<()> {
        if (address as usize) >= self.size {
            Err(MemoryError::OutOfBounds {
                address,
                size: self.size,
            }
            .into())
        } else {
            Ok(())
        }
    }

    /// Reads a byte from memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Memory address to read from
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfBounds` if address is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::ram::Ram;
    ///
    /// let mut ram = Ram::new(1024);
    /// ram.write_byte(100, 42).unwrap();
    /// assert_eq!(ram.read_byte(100).unwrap(), 42);
    /// ```
    pub fn read_byte(&self, address: Address) -> Result<Byte> {
        self.check_address(address)?;
        Ok(self.data[address as usize])
    }

    /// Writes a byte to memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Memory address to write to
    /// - `value`: Byte value to write
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfBounds` if address is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::ram::Ram;
    ///
    /// let mut ram = Ram::new(1024);
    /// ram.write_byte(100, 42).unwrap();
    /// assert_eq!(ram.read_byte(100).unwrap(), 42);
    /// ```
    pub fn write_byte(&mut self, address: Address, value: Byte) -> Result<()> {
        self.check_address(address)?;
        self.data[address as usize] = value;
        Ok(())
    }

    /// Reads a word (4 bytes) from memory.
    ///
    /// The address should be word-aligned (multiple of 4) for best performance,
    /// but unaligned access is supported with a performance penalty.
    ///
    /// Bytes are read in little-endian order.
    ///
    /// # Parameters
    ///
    /// - `address`: Memory address to read from
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfBounds` if address is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::ram::Ram;
    ///
    /// let mut ram = Ram::new(1024);
    /// ram.write_word(0x100, 0x12345678).unwrap();
    /// assert_eq!(ram.read_word(0x100).unwrap(), 0x12345678);
    /// ```
    pub fn read_word(&self, address: Address) -> Result<Word> {
        // Check bounds for all 4 bytes
        self.check_address(address)?;
        self.check_address(address + 3)?;

        // Read 4 bytes in little-endian order
        let b0 = self.data[address as usize] as Word;
        let b1 = self.data[(address + 1) as usize] as Word;
        let b2 = self.data[(address + 2) as usize] as Word;
        let b3 = self.data[(address + 3) as usize] as Word;

        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    /// Writes a word (4 bytes) to memory.
    ///
    /// The address should be word-aligned (multiple of 4) for best performance,
    /// but unaligned access is supported with a performance penalty.
    ///
    /// Bytes are written in little-endian order.
    ///
    /// # Parameters
    ///
    /// - `address`: Memory address to write to
    /// - `value`: Word value to write
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfBounds` if address is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::ram::Ram;
    ///
    /// let mut ram = Ram::new(1024);
    /// ram.write_word(0x100, 0x12345678).unwrap();
    ///
    /// // Verify individual bytes (little-endian)
    /// assert_eq!(ram.read_byte(0x100).unwrap(), 0x78);
    /// assert_eq!(ram.read_byte(0x101).unwrap(), 0x56);
    /// assert_eq!(ram.read_byte(0x102).unwrap(), 0x34);
    /// assert_eq!(ram.read_byte(0x103).unwrap(), 0x12);
    /// ```
    pub fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
        // Check bounds for all 4 bytes
        self.check_address(address)?;
        self.check_address(address + 3)?;

        // Write 4 bytes in little-endian order
        self.data[address as usize] = (value & 0xFF) as Byte;
        self.data[(address + 1) as usize] = ((value >> 8) & 0xFF) as Byte;
        self.data[(address + 2) as usize] = ((value >> 16) & 0xFF) as Byte;
        self.data[(address + 3) as usize] = ((value >> 24) & 0xFF) as Byte;

        Ok(())
    }

    /// Reads multiple bytes from memory into a buffer.
    ///
    /// # Parameters
    ///
    /// - `address`: Starting memory address
    /// - `buffer`: Buffer to read into
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfBounds` if any address is out of range.
    pub fn read_bytes(&self, address: Address, buffer: &mut [Byte]) -> Result<()> {
        let start = address as usize;
        let end = start + buffer.len();

        if end > self.size {
            return Err(MemoryError::OutOfBounds {
                address: end as Address,
                size: self.size,
            }
            .into());
        }

        buffer.copy_from_slice(&self.data[start..end]);
        Ok(())
    }

    /// Writes multiple bytes from a buffer to memory.
    ///
    /// # Parameters
    ///
    /// - `address`: Starting memory address
    /// - `buffer`: Buffer to write from
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfBounds` if any address is out of range.
    pub fn write_bytes(&mut self, address: Address, buffer: &[Byte]) -> Result<()> {
        let start = address as usize;
        let end = start + buffer.len();

        if end > self.size {
            return Err(MemoryError::OutOfBounds {
                address: end as Address,
                size: self.size,
            }
            .into());
        }

        self.data[start..end].copy_from_slice(buffer);
        Ok(())
    }

    /// Clears all memory (sets to zero).
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Returns a slice of memory for reading.
    ///
    /// # Safety
    ///
    /// The returned slice is valid for the lifetime of the borrow.
    pub fn as_slice(&self) -> &[Byte] {
        &self.data
    }

    /// Returns a mutable slice of memory.
    ///
    /// # Safety
    ///
    /// The returned slice is valid for the lifetime of the mutable borrow.
    pub fn as_mut_slice(&mut self) -> &mut [Byte] {
        &mut self.data
    }

    /// Loads data from a slice into memory at the specified address.
    ///
    /// This is useful for loading programs or data into memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_memory::ram::Ram;
    ///
    /// let mut ram = Ram::new(1024);
    /// let program = vec![0x01, 0x02, 0x03, 0x04];
    ///
    /// ram.load(0x100, &program).unwrap();
    ///
    /// assert_eq!(ram.read_byte(0x100).unwrap(), 0x01);
    /// assert_eq!(ram.read_byte(0x103).unwrap(), 0x04);
    /// ```
    pub fn load(&mut self, address: Address, data: &[Byte]) -> Result<()> {
        self.write_bytes(address, data)
    }

    /// Creates a hexdump of a memory region.
    ///
    /// Useful for debugging and inspection.
    ///
    /// # Parameters
    ///
    /// - `start`: Starting address
    /// - `length`: Number of bytes to dump
    ///
    /// # Returns
    ///
    /// A formatted string showing the memory contents.
    pub fn hexdump(&self, start: Address, length: usize) -> String {
        let mut output = String::new();
        let end = ((start as usize + length).min(self.size)) as Address;

        for addr in (start..end).step_by(16) {
            output.push_str(&format!("{:08X}  ", addr));

            // Hex bytes
            for i in 0..16 {
                if addr + i < end {
                    let byte = self.data[(addr + i) as usize];
                    output.push_str(&format!("{:02X} ", byte));
                } else {
                    output.push_str("   ");
                }

                if i == 7 {
                    output.push(' ');
                }
            }

            output.push_str(" |");

            // ASCII representation
            for i in 0..16 {
                if addr + i < end {
                    let byte = self.data[(addr + i) as usize];
                    if (32..=126).contains(&byte) {
                        output.push(byte as char);
                    } else {
                        output.push('.');
                    }
                }
            }

            output.push_str("|\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_creation() {
        let ram = Ram::new(1024);
        assert_eq!(ram.size(), 1024);
    }

    #[test]
    fn test_byte_read_write() {
        let mut ram = Ram::new(1024);

        ram.write_byte(100, 42).unwrap();
        assert_eq!(ram.read_byte(100).unwrap(), 42);
    }

    #[test]
    fn test_byte_out_of_bounds() {
        let ram = Ram::new(1024);

        let result = ram.read_byte(1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_word_read_write() {
        let mut ram = Ram::new(1024);

        ram.write_word(0x100, 0x12345678).unwrap();
        assert_eq!(ram.read_word(0x100).unwrap(), 0x12345678);
    }

    #[test]
    fn test_word_little_endian() {
        let mut ram = Ram::new(1024);

        ram.write_word(0x100, 0x12345678).unwrap();

        // Check little-endian byte order
        assert_eq!(ram.read_byte(0x100).unwrap(), 0x78);
        assert_eq!(ram.read_byte(0x101).unwrap(), 0x56);
        assert_eq!(ram.read_byte(0x102).unwrap(), 0x34);
        assert_eq!(ram.read_byte(0x103).unwrap(), 0x12);
    }

    #[test]
    fn test_word_unaligned() {
        let mut ram = Ram::new(1024);

        // Unaligned word access (address not multiple of 4)
        ram.write_word(0x101, 0x12345678).unwrap();
        assert_eq!(ram.read_word(0x101).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_write_bytes() {
        let mut ram = Ram::new(1024);

        let data = vec![1, 2, 3, 4, 5];
        ram.write_bytes(0x100, &data).unwrap();

        let mut buffer = vec![0; 5];
        ram.read_bytes(0x100, &mut buffer).unwrap();

        assert_eq!(buffer, data);
    }

    #[test]
    fn test_load() {
        let mut ram = Ram::new(1024);

        let program = vec![0xAA, 0xBB, 0xCC, 0xDD];
        ram.load(0x200, &program).unwrap();

        assert_eq!(ram.read_byte(0x200).unwrap(), 0xAA);
        assert_eq!(ram.read_byte(0x203).unwrap(), 0xDD);
    }

    #[test]
    fn test_clear() {
        let mut ram = Ram::new(1024);

        // Write some data
        ram.write_word(0x100, 0x12345678).unwrap();
        ram.write_word(0x200, 0xAABBCCDD).unwrap();

        // Clear
        ram.clear();

        // Check everything is zero
        assert_eq!(ram.read_word(0x100).unwrap(), 0);
        assert_eq!(ram.read_word(0x200).unwrap(), 0);
    }

    #[test]
    fn test_hexdump() {
        let mut ram = Ram::new(1024);

        // Write some recognizable data
        for i in 0..32 {
            ram.write_byte(i, i as u8).unwrap();
        }

        let dump = ram.hexdump(0, 32);
        assert!(dump.contains("00000000"));
        assert!(dump.contains("00 01 02 03"));
    }

    #[test]
    fn test_bounds_checking() {
        let mut ram = Ram::new(1024);

        // Try to write at boundary
        assert!(ram.write_byte(1023, 42).is_ok());

        // Try to write just beyond boundary
        assert!(ram.write_byte(1024, 42).is_err());

        // Try to write word that would overflow
        assert!(ram.write_word(1022, 0x12345678).is_err());
    }
}
