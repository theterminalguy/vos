//! Keyboard input device.
//!
//! Provides keyboard input through a memory-mapped buffer with interrupt support.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use vos_core::{Address, Byte, Device, Result};

/// Base address for keyboard memory-mapped I/O.
pub const KEYBOARD_BASE: Address = 0x8000_2000;

/// Keyboard device size.
pub const KEYBOARD_SIZE: usize = 16;

/// Maximum keyboard buffer size.
const BUFFER_SIZE: usize = 16;

/// Keyboard device registers.
const REG_DATA: u32 = 0;      // Read: get character, Write: ignored
const REG_STATUS: u32 = 4;    // Read: status flags
const REG_CONTROL: u32 = 8;   // Read/Write: control flags

/// Status flags.
const STATUS_DATA_READY: u8 = 0x01;  // Data available in buffer
const STATUS_BUFFER_FULL: u8 = 0x02; // Buffer is full

/// Keyboard input device.
///
/// Provides keyboard input through a memory-mapped interface.
/// Characters are queued in a FIFO buffer.
///
/// # Memory Layout
///
/// - Base: 0x80002000
/// - Size: 16 bytes
///
/// ## Registers
///
/// - 0x00: DATA - Read to get next character (or 0 if none)
/// - 0x04: STATUS - Read status flags
/// - 0x08: CONTROL - Control register (interrupt enable, etc.)
///
/// # Examples
///
/// ```
/// use vos_io::keyboard::Keyboard;
/// use vos_core::Device;
///
/// let mut keyboard = Keyboard::new();
///
/// // Simulate key press
/// keyboard.push_key(b'A');
///
/// // Read from device
/// let data = keyboard.read_byte(0).unwrap();
/// assert_eq!(data, b'A');
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyboard {
    /// Input buffer (FIFO queue)
    buffer: VecDeque<u8>,

    /// Control register
    control: u8,

    /// Should generate interrupt on key press?
    interrupt_enabled: bool,
}

impl Keyboard {
    /// Creates a new keyboard device.
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(BUFFER_SIZE),
            control: 0,
            interrupt_enabled: false,
        }
    }

    /// Pushes a key into the buffer.
    ///
    /// Returns true if the key was added, false if buffer is full.
    pub fn push_key(&mut self, key: u8) -> bool {
        if self.buffer.len() < BUFFER_SIZE {
            self.buffer.push_back(key);
            true
        } else {
            false
        }
    }

    /// Pops a key from the buffer.
    pub fn pop_key(&mut self) -> Option<u8> {
        self.buffer.pop_front()
    }

    /// Returns true if data is available.
    pub fn has_data(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Returns true if buffer is full.
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= BUFFER_SIZE
    }

    /// Gets the status register value.
    fn status(&self) -> u8 {
        let mut status = 0u8;
        if self.has_data() {
            status |= STATUS_DATA_READY;
        }
        if self.is_full() {
            status |= STATUS_BUFFER_FULL;
        }
        status
    }

    /// Returns true if interrupts are enabled.
    pub fn interrupt_enabled(&self) -> bool {
        self.interrupt_enabled
    }

    /// Enables or disables interrupts.
    pub fn set_interrupt_enabled(&mut self, enabled: bool) {
        self.interrupt_enabled = enabled;
    }

    /// Clears the input buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Keyboard {
    fn read_byte(&mut self, offset: u32) -> Result<Byte> {
        match offset {
            REG_DATA => {
                // Read character from buffer (or 0 if empty)
                Ok(self.pop_key().unwrap_or(0))
            }
            REG_STATUS => {
                // Read status flags
                Ok(self.status())
            }
            REG_CONTROL => {
                // Read control register
                Ok(self.control)
            }
            _ => Ok(0),
        }
    }

    fn write_byte(&mut self, offset: u32, value: Byte) -> Result<()> {
        match offset {
            REG_CONTROL => {
                // Write control register
                self.control = value;
                self.interrupt_enabled = (value & 0x01) != 0;
            }
            _ => {
                // Other registers are read-only
            }
        }
        Ok(())
    }

    fn base_address(&self) -> Address {
        KEYBOARD_BASE
    }

    fn size(&self) -> usize {
        KEYBOARD_SIZE
    }

    fn name(&self) -> &str {
        "Keyboard"
    }

    fn reset(&mut self) {
        self.clear();
        self.control = 0;
        self.interrupt_enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_creation() {
        let keyboard = Keyboard::new();
        assert!(!keyboard.has_data());
        assert!(!keyboard.is_full());
    }

    #[test]
    fn test_push_pop() {
        let mut keyboard = Keyboard::new();

        keyboard.push_key(b'A');
        assert!(keyboard.has_data());

        let key = keyboard.pop_key();
        assert_eq!(key, Some(b'A'));
        assert!(!keyboard.has_data());
    }

    #[test]
    fn test_buffer_full() {
        let mut keyboard = Keyboard::new();

        // Fill buffer
        for _ in 0..BUFFER_SIZE {
            assert!(keyboard.push_key(b'X'));
        }

        assert!(keyboard.is_full());

        // Should fail to add more
        assert!(!keyboard.push_key(b'Y'));
    }

    #[test]
    fn test_device_read() {
        let mut keyboard = Keyboard::new();

        keyboard.push_key(b'H');
        keyboard.push_key(b'i');

        // Read first character
        assert_eq!(keyboard.read_byte(REG_DATA).unwrap(), b'H');

        // Read second character
        assert_eq!(keyboard.read_byte(REG_DATA).unwrap(), b'i');

        // Buffer empty, should return 0
        assert_eq!(keyboard.read_byte(REG_DATA).unwrap(), 0);
    }

    #[test]
    fn test_status_register() {
        let mut keyboard = Keyboard::new();

        // Empty: no flags set
        assert_eq!(keyboard.read_byte(REG_STATUS).unwrap(), 0);

        // Add data
        keyboard.push_key(b'A');
        assert_eq!(
            keyboard.read_byte(REG_STATUS).unwrap(),
            STATUS_DATA_READY
        );

        // Fill buffer
        for _ in 1..BUFFER_SIZE {
            keyboard.push_key(b'X');
        }
        assert_eq!(
            keyboard.read_byte(REG_STATUS).unwrap(),
            STATUS_DATA_READY | STATUS_BUFFER_FULL
        );
    }

    #[test]
    fn test_control_register() {
        let mut keyboard = Keyboard::new();

        // Enable interrupts
        keyboard.write_byte(REG_CONTROL, 0x01).unwrap();
        assert!(keyboard.interrupt_enabled());

        // Disable interrupts
        keyboard.write_byte(REG_CONTROL, 0x00).unwrap();
        assert!(!keyboard.interrupt_enabled());
    }

    #[test]
    fn test_clear() {
        let mut keyboard = Keyboard::new();

        keyboard.push_key(b'A');
        keyboard.push_key(b'B');

        keyboard.clear();

        assert!(!keyboard.has_data());
    }

    #[test]
    fn test_base_address() {
        let keyboard = Keyboard::new();
        assert_eq!(keyboard.base_address(), KEYBOARD_BASE);
        assert_eq!(keyboard.size(), KEYBOARD_SIZE);
    }
}
