//! Text-mode display device.
//!
//! A simple 80x25 character display with attributes (foreground/background colors).

use serde::{Deserialize, Serialize};
use vos_core::{Address, Byte, Device, Result};

/// Display dimensions.
pub const DISPLAY_WIDTH: usize = 80;
pub const DISPLAY_HEIGHT: usize = 25;
pub const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Base address for display memory-mapped I/O.
pub const DISPLAY_BASE: Address = 0x8000_0000;

/// Display device size (2 bytes per character: char + attribute).
pub const DISPLAY_DEVICE_SIZE: usize = DISPLAY_SIZE * 2;

/// Character with attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharCell {
    /// ASCII character
    pub character: u8,
    /// Attribute byte (foreground and background color)
    pub attribute: u8,
}

impl CharCell {
    /// Creates a new character cell.
    pub fn new(character: u8, attribute: u8) -> Self {
        Self {
            character,
            attribute,
        }
    }

    /// Creates a character cell with default attributes (white on black).
    pub fn with_char(character: u8) -> Self {
        Self {
            character,
            attribute: 0x07, // Light gray on black
        }
    }
}

impl Default for CharCell {
    fn default() -> Self {
        Self {
            character: b' ',
            attribute: 0x07,
        }
    }
}

/// Text-mode display device.
///
/// Provides a 80x25 character display with color attributes.
/// Each character occupies 2 bytes: character code and attribute.
///
/// # Memory Layout
///
/// - Base address: 0x80000000
/// - Size: 4000 bytes (80 * 25 * 2)
/// - Format: [char, attr, char, attr, ...]
///
/// # Attributes
///
/// The attribute byte contains foreground and background colors:
/// - Bits 0-3: Foreground color
/// - Bits 4-7: Background color
///
/// # Examples
///
/// ```
/// use vos_io::display::{Display, CharCell};
/// use vos_core::Device;
///
/// let mut display = Display::new();
///
/// // Write 'A' with attribute 0x0F (white on black)
/// display.write_byte(0, b'A').unwrap();
/// display.write_byte(1, 0x0F).unwrap();
///
/// // Read it back
/// assert_eq!(display.read_byte(0).unwrap(), b'A');
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    /// Character buffer (80x25)
    buffer: Vec<CharCell>,

    /// Cursor position (0-1999)
    cursor: usize,
}

impl Display {
    /// Creates a new display device.
    pub fn new() -> Self {
        Self {
            buffer: vec![CharCell::default(); DISPLAY_SIZE],
            cursor: 0,
        }
    }

    /// Gets the cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, position: usize) {
        self.cursor = position.min(DISPLAY_SIZE - 1);
    }

    /// Gets a character cell at the given position.
    pub fn get_cell(&self, index: usize) -> Option<CharCell> {
        self.buffer.get(index).copied()
    }

    /// Sets a character cell at the given position.
    pub fn set_cell(&mut self, index: usize, cell: CharCell) {
        if index < DISPLAY_SIZE {
            self.buffer[index] = cell;
        }
    }

    /// Clears the display.
    pub fn clear(&mut self) {
        self.buffer.fill(CharCell::default());
        self.cursor = 0;
    }

    /// Writes a character at the cursor and advances.
    pub fn put_char(&mut self, character: u8) {
        if character == b'\n' {
            // Newline: move to start of next line
            self.cursor = ((self.cursor / DISPLAY_WIDTH) + 1) * DISPLAY_WIDTH;
        } else {
            self.buffer[self.cursor] = CharCell::with_char(character);
            self.cursor += 1;
        }

        // Scroll if needed
        if self.cursor >= DISPLAY_SIZE {
            self.scroll();
            self.cursor = DISPLAY_SIZE - DISPLAY_WIDTH;
        }
    }

    /// Scrolls the display up by one line.
    fn scroll(&mut self) {
        self.buffer.copy_within(DISPLAY_WIDTH..DISPLAY_SIZE, 0);
        let last_line_start = DISPLAY_SIZE - DISPLAY_WIDTH;
        for i in last_line_start..DISPLAY_SIZE {
            self.buffer[i] = CharCell::default();
        }
    }

    /// Gets the display buffer as a string (for debugging/testing).
    pub fn display_to_string(&self) -> String {
        let mut output = String::with_capacity(DISPLAY_SIZE + DISPLAY_HEIGHT);

        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                let idx = row * DISPLAY_WIDTH + col;
                let ch = self.buffer[idx].character;
                output.push(if (32..=126).contains(&ch) {
                    ch as char
                } else {
                    ' '
                });
            }
            output.push('\n');
        }

        output
    }

    /// Gets a single line as a string.
    pub fn line(&self, row: usize) -> String {
        if row >= DISPLAY_HEIGHT {
            return String::new();
        }

        let start = row * DISPLAY_WIDTH;
        let end = start + DISPLAY_WIDTH;

        self.buffer[start..end]
            .iter()
            .map(|cell| {
                if (32..=126).contains(&cell.character) {
                    cell.character as char
                } else {
                    ' '
                }
            })
            .collect()
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Display {
    fn read_byte(&mut self, offset: u32) -> Result<Byte> {
        let offset = offset as usize;
        if offset >= DISPLAY_DEVICE_SIZE {
            return Ok(0);
        }

        let cell_index = offset / 2;
        let byte_index = offset % 2;

        let cell = self.buffer[cell_index];
        Ok(if byte_index == 0 {
            cell.character
        } else {
            cell.attribute
        })
    }

    fn write_byte(&mut self, offset: u32, value: Byte) -> Result<()> {
        let offset = offset as usize;
        if offset >= DISPLAY_DEVICE_SIZE {
            return Ok(());
        }

        let cell_index = offset / 2;
        let byte_index = offset % 2;

        if byte_index == 0 {
            self.buffer[cell_index].character = value;
        } else {
            self.buffer[cell_index].attribute = value;
        }

        Ok(())
    }

    fn base_address(&self) -> Address {
        DISPLAY_BASE
    }

    fn size(&self) -> usize {
        DISPLAY_DEVICE_SIZE
    }

    fn name(&self) -> &str {
        "Display"
    }

    fn reset(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_creation() {
        let display = Display::new();
        assert_eq!(display.cursor(), 0);
        assert_eq!(display.buffer.len(), DISPLAY_SIZE);
    }

    #[test]
    fn test_char_cell() {
        let cell = CharCell::new(b'A', 0x0F);
        assert_eq!(cell.character, b'A');
        assert_eq!(cell.attribute, 0x0F);
    }

    #[test]
    fn test_put_char() {
        let mut display = Display::new();

        display.put_char(b'H');
        display.put_char(b'i');

        assert_eq!(display.get_cell(0).unwrap().character, b'H');
        assert_eq!(display.get_cell(1).unwrap().character, b'i');
        assert_eq!(display.cursor(), 2);
    }

    #[test]
    fn test_newline() {
        let mut display = Display::new();

        display.put_char(b'A');
        display.put_char(b'\n');

        assert_eq!(display.cursor(), DISPLAY_WIDTH);
    }

    #[test]
    fn test_clear() {
        let mut display = Display::new();

        display.put_char(b'A');
        display.put_char(b'B');

        display.clear();

        assert_eq!(display.cursor(), 0);
        assert_eq!(display.get_cell(0).unwrap().character, b' ');
    }

    #[test]
    fn test_device_read_write() {
        let mut display = Display::new();

        // Write character
        display.write_byte(0, b'X').unwrap();
        assert_eq!(display.read_byte(0).unwrap(), b'X');

        // Write attribute
        display.write_byte(1, 0x0F).unwrap();
        assert_eq!(display.read_byte(1).unwrap(), 0x0F);
    }

    #[test]
    fn test_scroll() {
        let mut display = Display::new();

        // Fill entire display
        for _ in 0..DISPLAY_SIZE {
            display.put_char(b'X');
        }

        // Cursor should be at last line after scroll
        assert!(display.cursor() < DISPLAY_SIZE);

        // First line should be clear (scrolled off)
        assert_eq!(display.get_cell(0).unwrap().character, b'X');
    }

    #[test]
    fn test_line_extraction() {
        let mut display = Display::new();

        display.put_char(b'H');
        display.put_char(b'e');
        display.put_char(b'l');
        display.put_char(b'l');
        display.put_char(b'o');

        let line = display.line(0);
        assert!(line.starts_with("Hello"));
    }

    #[test]
    fn test_base_address() {
        let display = Display::new();
        assert_eq!(display.base_address(), DISPLAY_BASE);
        assert_eq!(display.size(), DISPLAY_DEVICE_SIZE);
    }
}
