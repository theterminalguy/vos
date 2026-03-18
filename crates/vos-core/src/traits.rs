//! Core traits for VOS components.
//!
//! This module defines the fundamental traits that components of the VOS system
//! implement, providing common interfaces for devices, executable code, and more.

use crate::error::Result;
use crate::types::{Address, Byte, Word};

/// A memory-mapped I/O device.
///
/// All I/O devices in VOS are memory-mapped, meaning they are accessed through
/// reads and writes to specific memory addresses. This trait defines the interface
/// that all devices must implement.
///
/// # Examples
///
/// ```ignore
/// use vos_core::traits::Device;
///
/// struct Timer {
///     counter: u32,
/// }
///
/// impl Device for Timer {
///     fn read_byte(&mut self, offset: u32) -> Result<u8> {
///         // Read from device register at offset
///         Ok((self.counter >> (offset * 8)) as u8)
///     }
///
///     fn write_byte(&mut self, offset: u32, value: u8) -> Result<()> {
///         // Write to device register at offset
///         Ok(())
///     }
///
///     fn base_address(&self) -> Address {
///         0x80000000
///     }
///
///     fn size(&self) -> usize {
///         4 // 4 bytes for the counter register
///     }
/// }
/// ```
pub trait Device {
    /// Reads a byte from the device at the given offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: Byte offset from the device's base address
    ///
    /// # Returns
    ///
    /// The byte value at the specified offset, or an error.
    fn read_byte(&mut self, offset: u32) -> Result<Byte>;

    /// Writes a byte to the device at the given offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: Byte offset from the device's base address
    /// - `value`: The byte value to write
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error.
    fn write_byte(&mut self, offset: u32, value: Byte) -> Result<()>;

    /// Reads a word (4 bytes) from the device at the given offset.
    ///
    /// Default implementation reads 4 bytes and combines them (little-endian).
    ///
    /// # Parameters
    ///
    /// - `offset`: Byte offset from the device's base address (should be word-aligned)
    ///
    /// # Returns
    ///
    /// The word value at the specified offset, or an error.
    fn read_word(&mut self, offset: u32) -> Result<Word> {
        let b0 = self.read_byte(offset)? as Word;
        let b1 = self.read_byte(offset + 1)? as Word;
        let b2 = self.read_byte(offset + 2)? as Word;
        let b3 = self.read_byte(offset + 3)? as Word;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    /// Writes a word (4 bytes) to the device at the given offset.
    ///
    /// Default implementation writes 4 bytes separately (little-endian).
    ///
    /// # Parameters
    ///
    /// - `offset`: Byte offset from the device's base address (should be word-aligned)
    /// - `value`: The word value to write
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error.
    fn write_word(&mut self, offset: u32, value: Word) -> Result<()> {
        self.write_byte(offset, (value & 0xFF) as Byte)?;
        self.write_byte(offset + 1, ((value >> 8) & 0xFF) as Byte)?;
        self.write_byte(offset + 2, ((value >> 16) & 0xFF) as Byte)?;
        self.write_byte(offset + 3, ((value >> 24) & 0xFF) as Byte)?;
        Ok(())
    }

    /// Returns the base address of this device in the memory map.
    fn base_address(&self) -> Address;

    /// Returns the size of the device's address space in bytes.
    fn size(&self) -> usize;

    /// Returns the name of the device (for debugging/display).
    fn name(&self) -> &str {
        "Unknown Device"
    }

    /// Called on each clock tick (for devices that need periodic updates).
    ///
    /// # Parameters
    ///
    /// - `cycles`: Number of CPU cycles since last tick
    fn tick(&mut self, _cycles: u64) {
        // Default: do nothing
    }

    /// Resets the device to its initial state.
    fn reset(&mut self) {
        // Default: do nothing
    }
}

/// A component that can be executed (CPU, virtual machine, etc.).
///
/// This trait abstracts over things that can run instructions or code.
pub trait Executable {
    /// Executes a single step (e.g., one instruction).
    ///
    /// # Returns
    ///
    /// Ok(true) if execution should continue, Ok(false) if execution is complete,
    /// or an error if something went wrong.
    fn step(&mut self) -> Result<bool>;

    /// Runs execution until completion or error.
    ///
    /// Default implementation calls `step()` repeatedly.
    fn run(&mut self) -> Result<()> {
        while self.step()? {
            // Continue execution
        }
        Ok(())
    }

    /// Resets the executable to its initial state.
    fn reset(&mut self);
}

/// A component that can be clocked (ticked forward in time).
///
/// This trait is for components that need to track the passage of time
/// or CPU cycles.
pub trait Clockable {
    /// Advances the component by the given number of cycles.
    ///
    /// # Parameters
    ///
    /// - `cycles`: Number of clock cycles to advance
    fn tick(&mut self, cycles: u64);

    /// Returns the current cycle count.
    fn cycle_count(&self) -> u64;
}

/// A component that can be inspected for debugging.
///
/// This trait provides a way to examine the internal state of a component
/// for debugging and educational purposes.
pub trait Inspectable {
    /// Returns a human-readable string representation of the component's state.
    fn inspect(&self) -> String;

    /// Returns detailed state information as key-value pairs.
    fn state(&self) -> Vec<(String, String)> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock device for testing
    struct MockDevice {
        data: [u8; 16],
    }

    impl Device for MockDevice {
        fn read_byte(&mut self, offset: u32) -> Result<Byte> {
            Ok(self.data[offset as usize])
        }

        fn write_byte(&mut self, offset: u32, value: Byte) -> Result<()> {
            self.data[offset as usize] = value;
            Ok(())
        }

        fn base_address(&self) -> Address {
            0x8000_0000
        }

        fn size(&self) -> usize {
            16
        }

        fn name(&self) -> &str {
            "MockDevice"
        }
    }

    #[test]
    fn test_device_read_write_byte() {
        let mut device = MockDevice { data: [0; 16] };

        device.write_byte(0, 0x42).unwrap();
        assert_eq!(device.read_byte(0).unwrap(), 0x42);
    }

    #[test]
    fn test_device_read_write_word() {
        let mut device = MockDevice { data: [0; 16] };

        device.write_word(0, 0x12345678).unwrap();
        assert_eq!(device.read_word(0).unwrap(), 0x12345678);

        // Check little-endian byte order
        assert_eq!(device.read_byte(0).unwrap(), 0x78);
        assert_eq!(device.read_byte(1).unwrap(), 0x56);
        assert_eq!(device.read_byte(2).unwrap(), 0x34);
        assert_eq!(device.read_byte(3).unwrap(), 0x12);
    }
}
