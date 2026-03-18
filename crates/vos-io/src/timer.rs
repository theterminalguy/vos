//! Timer device.
//!
//! Provides a programmable timer for periodic interrupts.

use serde::{Deserialize, Serialize};
use vos_core::{Address, Byte, Device, Result, Word};

/// Base address for timer memory-mapped I/O.
pub const TIMER_BASE: Address = 0x8000_3000;

/// Timer device size.
pub const TIMER_SIZE: usize = 16;

/// Timer device registers.
const REG_COUNTER: u32 = 0;   // Current counter value (read-only)
const REG_RELOAD: u32 = 4;    // Reload value (read/write)
const REG_CONTROL: u32 = 8;   // Control register
const REG_STATUS: u32 = 12;   // Status register

/// Control flags.
const CTRL_ENABLE: u8 = 0x01;     // Timer enabled
const CTRL_INTERRUPT: u8 = 0x02;  // Generate interrupt on expire

/// Status flags.
const STATUS_EXPIRED: u8 = 0x01;  // Timer has expired

/// Timer device.
///
/// Provides a programmable countdown timer that can generate interrupts.
///
/// # Memory Layout
///
/// - Base: 0x80003000
/// - Size: 16 bytes
///
/// ## Registers
///
/// - 0x00: COUNTER - Current counter value (read-only, counts down)
/// - 0x04: RELOAD - Reload value (read/write)
/// - 0x08: CONTROL - Control flags (enable, interrupt)
/// - 0x0C: STATUS - Status flags (expired)
///
/// # Operation
///
/// 1. Write reload value to RELOAD register
/// 2. Set ENABLE bit in CONTROL register
/// 3. Counter counts down each cycle
/// 4. When counter reaches 0, it reloads and STATUS_EXPIRED is set
/// 5. If INTERRUPT bit is set, an interrupt is generated
///
/// # Examples
///
/// ```
/// use vos_io::timer::Timer;
/// use vos_core::Device;
///
/// let mut timer = Timer::new();
///
/// // Set reload value to 1000
/// timer.write_word(4, 1000).unwrap();
///
/// // Enable timer with interrupts
/// timer.write_byte(8, 0x03).unwrap();
///
/// // Tick the timer
/// for _ in 0..1000 {
///     timer.tick(1);
/// }
///
/// // Check if expired
/// let status = timer.read_byte(12).unwrap();
/// assert_eq!(status & 0x01, 0x01);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    /// Current counter value
    counter: u32,

    /// Reload value
    reload: u32,

    /// Control register
    control: u8,

    /// Status register
    status: u8,

    /// Total cycles elapsed
    total_cycles: u64,
}

impl Timer {
    /// Creates a new timer device.
    pub fn new() -> Self {
        Self {
            counter: 0,
            reload: 0,
            control: 0,
            status: 0,
            total_cycles: 0,
        }
    }

    /// Returns true if the timer is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.control & CTRL_ENABLE) != 0
    }

    /// Returns true if interrupts are enabled.
    pub fn interrupt_enabled(&self) -> bool {
        (self.control & CTRL_INTERRUPT) != 0
    }

    /// Returns true if the timer has expired.
    pub fn has_expired(&self) -> bool {
        (self.status & STATUS_EXPIRED) != 0
    }

    /// Clears the expired flag.
    pub fn clear_expired(&mut self) {
        self.status &= !STATUS_EXPIRED;
    }

    /// Gets the current counter value.
    pub fn counter(&self) -> u32 {
        self.counter
    }

    /// Gets the reload value.
    pub fn reload(&self) -> u32 {
        self.reload
    }

    /// Returns the total number of cycles elapsed.
    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    /// Ticks the timer forward by the given number of cycles.
    fn tick_internal(&mut self, cycles: u64) {
        if !self.is_enabled() {
            return;
        }

        self.total_cycles += cycles;

        for _ in 0..cycles {
            if self.counter > 0 {
                self.counter -= 1;

                // Check if we just reached 0
                if self.counter == 0 {
                    // Timer expired
                    self.status |= STATUS_EXPIRED;

                    // Reload counter
                    self.counter = self.reload;
                }
            }
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Timer {
    fn read_byte(&mut self, offset: u32) -> Result<Byte> {
        if offset <= REG_COUNTER + 3 {
            let shift = (offset - REG_COUNTER) * 8;
            Ok(((self.counter >> shift) & 0xFF) as Byte)
        } else if (REG_RELOAD..=REG_RELOAD + 3).contains(&offset) {
            let shift = (offset - REG_RELOAD) * 8;
            Ok(((self.reload >> shift) & 0xFF) as Byte)
        } else if offset == REG_CONTROL {
            Ok(self.control)
        } else if offset == REG_STATUS {
            Ok(self.status)
        } else {
            Ok(0)
        }
    }

    fn write_byte(&mut self, offset: u32, value: Byte) -> Result<()> {
        if (REG_RELOAD..=REG_RELOAD + 3).contains(&offset) {
            let shift = (offset - REG_RELOAD) * 8;
            let mask = !(0xFF << shift);
            self.reload = (self.reload & mask) | ((value as u32) << shift);
        } else if offset == REG_CONTROL {
            let was_enabled = self.is_enabled();
            self.control = value;

            // If transitioning from disabled to enabled, load counter
            if !was_enabled && self.is_enabled() {
                self.counter = self.reload;
            }
        } else if offset == REG_STATUS {
            // Writing to status clears expired flag
            if value & STATUS_EXPIRED != 0 {
                self.clear_expired();
            }
        }
        // Other registers are read-only
        Ok(())
    }

    fn read_word(&mut self, offset: u32) -> Result<Word> {
        match offset {
            REG_COUNTER => Ok(self.counter),
            REG_RELOAD => Ok(self.reload),
            _ => {
                // For other registers, use default byte-based implementation
                let b0 = self.read_byte(offset)? as Word;
                let b1 = self.read_byte(offset + 1)? as Word;
                let b2 = self.read_byte(offset + 2)? as Word;
                let b3 = self.read_byte(offset + 3)? as Word;
                Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
            }
        }
    }

    fn write_word(&mut self, offset: u32, value: Word) -> Result<()> {
        match offset {
            REG_RELOAD => {
                self.reload = value;
                Ok(())
            }
            _ => {
                // For other registers, use default byte-based implementation
                self.write_byte(offset, (value & 0xFF) as Byte)?;
                self.write_byte(offset + 1, ((value >> 8) & 0xFF) as Byte)?;
                self.write_byte(offset + 2, ((value >> 16) & 0xFF) as Byte)?;
                self.write_byte(offset + 3, ((value >> 24) & 0xFF) as Byte)?;
                Ok(())
            }
        }
    }

    fn base_address(&self) -> Address {
        TIMER_BASE
    }

    fn size(&self) -> usize {
        TIMER_SIZE
    }

    fn name(&self) -> &str {
        "Timer"
    }

    fn tick(&mut self, cycles: u64) {
        self.tick_internal(cycles);
    }

    fn reset(&mut self) {
        self.counter = 0;
        self.reload = 0;
        self.control = 0;
        self.status = 0;
        self.total_cycles = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new();
        assert!(!timer.is_enabled());
        assert!(!timer.has_expired());
    }

    #[test]
    fn test_set_reload() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 1000).unwrap();
        assert_eq!(timer.reload(), 1000);
    }

    #[test]
    fn test_enable() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 100).unwrap();
        timer.write_byte(REG_CONTROL, CTRL_ENABLE).unwrap();

        assert!(timer.is_enabled());
        assert_eq!(timer.counter(), 100);
    }

    #[test]
    fn test_countdown() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 10).unwrap();
        timer.write_byte(REG_CONTROL, CTRL_ENABLE).unwrap();

        // Tick 10 times
        for _ in 0..10 {
            timer.tick(1);
        }

        // Should have expired and reloaded
        assert!(timer.has_expired());
        assert_eq!(timer.counter(), 10);
    }

    #[test]
    fn test_interrupt_flag() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 5).unwrap();
        timer
            .write_byte(REG_CONTROL, CTRL_ENABLE | CTRL_INTERRUPT)
            .unwrap();

        assert!(timer.interrupt_enabled());

        // Tick until expired
        timer.tick(5);

        assert!(timer.has_expired());
    }

    #[test]
    fn test_clear_expired() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 1).unwrap();
        timer.write_byte(REG_CONTROL, CTRL_ENABLE).unwrap();

        timer.tick(1);
        assert!(timer.has_expired());

        // Clear by writing to status
        timer.write_byte(REG_STATUS, STATUS_EXPIRED).unwrap();
        assert!(!timer.has_expired());
    }

    #[test]
    fn test_disabled_timer() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 10).unwrap();
        // Don't enable

        timer.tick(100);

        // Counter shouldn't change
        assert_eq!(timer.counter(), 0);
        assert!(!timer.has_expired());
    }

    #[test]
    fn test_total_cycles() {
        let mut timer = Timer::new();

        timer.write_word(REG_RELOAD, 100).unwrap();
        timer.write_byte(REG_CONTROL, CTRL_ENABLE).unwrap();

        timer.tick(50);
        assert_eq!(timer.total_cycles(), 50);

        timer.tick(50);
        assert_eq!(timer.total_cycles(), 100);
    }

    #[test]
    fn test_base_address() {
        let timer = Timer::new();
        assert_eq!(timer.base_address(), TIMER_BASE);
        assert_eq!(timer.size(), TIMER_SIZE);
    }
}
