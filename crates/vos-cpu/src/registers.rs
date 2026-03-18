//! CPU Register file implementation.
//!
//! The VOS CPU has 16 general-purpose registers and several special registers.

use serde::{Deserialize, Serialize};
use vos_core::{Address, CpuError, RegisterIndex, Result, Word};

/// CPU register file.
///
/// Contains 16 general-purpose registers plus special registers.
///
/// # Register Conventions
///
/// - R0: Always zero (hardwired)
/// - R1-R14: General purpose
/// - R15: Stack pointer (by convention)
///
/// # Special Registers
///
/// - PC: Program Counter
/// - IR: Instruction Register (current instruction)
/// - FLAGS: Status flags (zero, negative, carry, overflow)
///
/// # Examples
///
/// ```
/// use vos_cpu::registers::Registers;
///
/// let mut regs = Registers::new();
///
/// // R0 is always zero
/// regs.write(0, 42);
/// assert_eq!(regs.read(0), 0);
///
/// // Other registers work normally
/// regs.write(1, 42);
/// assert_eq!(regs.read(1), 42);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registers {
    /// General-purpose registers (R0-R15)
    gpr: [Word; 16],

    /// Program Counter
    pub pc: Address,

    /// Instruction Register (current instruction being executed)
    pub ir: Word,

    /// Status flags
    pub flags: Flags,
}

/// CPU status flags.
///
/// These flags are set by arithmetic and comparison operations
/// and used by conditional branch instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flags {
    /// Zero flag: Set when result is zero
    pub zero: bool,

    /// Negative flag: Set when result is negative
    pub negative: bool,

    /// Carry flag: Set when unsigned overflow occurs
    pub carry: bool,

    /// Overflow flag: Set when signed overflow occurs
    pub overflow: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            zero: false,
            negative: false,
            carry: false,
            overflow: false,
        }
    }
}

impl Flags {
    /// Creates new flags with all bits cleared.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates flags based on a result value.
    ///
    /// Sets zero and negative flags appropriately.
    pub fn update_zn(&mut self, result: Word) {
        self.zero = result == 0;
        self.negative = (result as i32) < 0;
    }

    /// Converts flags to a single byte value.
    ///
    /// Format: [0:4][overflow:1][carry:1][negative:1][zero:1]
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.zero { byte |= 0x01; }
        if self.negative { byte |= 0x02; }
        if self.carry { byte |= 0x04; }
        if self.overflow { byte |= 0x08; }
        byte
    }

    /// Creates flags from a byte value.
    pub fn from_byte(byte: u8) -> Self {
        Self {
            zero: (byte & 0x01) != 0,
            negative: (byte & 0x02) != 0,
            carry: (byte & 0x04) != 0,
            overflow: (byte & 0x08) != 0,
        }
    }
}

impl Registers {
    /// Creates a new register file with all registers initialized to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::registers::Registers;
    ///
    /// let regs = Registers::new();
    /// assert_eq!(regs.read(0), 0);
    /// assert_eq!(regs.pc, 0);
    /// ```
    pub fn new() -> Self {
        Self {
            gpr: [0; 16],
            pc: 0,
            ir: 0,
            flags: Flags::new(),
        }
    }

    /// Reads a value from a general-purpose register.
    ///
    /// # Parameters
    ///
    /// - `index`: Register number (0-15)
    ///
    /// # Returns
    ///
    /// The value in the register. R0 always returns 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::registers::Registers;
    ///
    /// let mut regs = Registers::new();
    /// regs.write(5, 42);
    /// assert_eq!(regs.read(5), 42);
    /// ```
    pub fn read(&self, index: RegisterIndex) -> Word {
        if index == 0 {
            0  // R0 is hardwired to zero
        } else if index < 16 {
            self.gpr[index as usize]
        } else {
            0  // Invalid index returns 0 (defensive)
        }
    }

    /// Writes a value to a general-purpose register.
    ///
    /// # Parameters
    ///
    /// - `index`: Register number (0-15)
    /// - `value`: Value to write
    ///
    /// # Notes
    ///
    /// Writes to R0 are silently ignored (R0 is hardwired to zero).
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::registers::Registers;
    ///
    /// let mut regs = Registers::new();
    ///
    /// regs.write(1, 42);
    /// assert_eq!(regs.read(1), 42);
    ///
    /// // R0 cannot be changed
    /// regs.write(0, 42);
    /// assert_eq!(regs.read(0), 0);
    /// ```
    pub fn write(&mut self, index: RegisterIndex, value: Word) {
        if index > 0 && index < 16 {
            self.gpr[index as usize] = value;
        }
        // Writes to R0 or invalid indices are ignored
    }

    /// Attempts to read a register, returning an error for invalid indices.
    ///
    /// This is the checked version of `read()` that returns an error
    /// instead of defaulting to 0 for invalid indices.
    pub fn try_read(&self, index: RegisterIndex) -> Result<Word> {
        if index < 16 {
            Ok(self.read(index))
        } else {
            Err(CpuError::InvalidRegister(index).into())
        }
    }

    /// Attempts to write a register, returning an error for invalid indices.
    ///
    /// This is the checked version of `write()`.
    pub fn try_write(&mut self, index: RegisterIndex, value: Word) -> Result<()> {
        if index < 16 {
            self.write(index, value);
            Ok(())
        } else {
            Err(CpuError::InvalidRegister(index).into())
        }
    }

    /// Resets all registers to their initial state.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::registers::Registers;
    ///
    /// let mut regs = Registers::new();
    /// regs.write(5, 42);
    /// regs.pc = 0x1000;
    ///
    /// regs.reset();
    /// assert_eq!(regs.read(5), 0);
    /// assert_eq!(regs.pc, 0);
    /// ```
    pub fn reset(&mut self) {
        self.gpr = [0; 16];
        self.pc = 0;
        self.ir = 0;
        self.flags = Flags::new();
    }

    /// Returns a formatted string showing all register values.
    ///
    /// Useful for debugging and displaying CPU state.
    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str("Registers:\n");

        for i in 0..16 {
            output.push_str(&format!(
                "  R{:2}: 0x{:08X} ({})\n",
                i,
                self.read(i as RegisterIndex),
                self.read(i as RegisterIndex) as i32
            ));
        }

        output.push_str(&format!("\nPC:    0x{:08X}\n", self.pc));
        output.push_str(&format!("IR:    0x{:08X}\n", self.ir));
        output.push_str(&format!(
            "FLAGS: Z={} N={} C={} V={}\n",
            if self.flags.zero { 1 } else { 0 },
            if self.flags.negative { 1 } else { 0 },
            if self.flags.carry { 1 } else { 0 },
            if self.flags.overflow { 1 } else { 0 }
        ));

        output
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r0_always_zero() {
        let mut regs = Registers::new();

        // Try to write to R0
        regs.write(0, 42);

        // R0 should still be zero
        assert_eq!(regs.read(0), 0);
    }

    #[test]
    fn test_general_register_read_write() {
        let mut regs = Registers::new();

        // Write and read from R1
        regs.write(1, 12345);
        assert_eq!(regs.read(1), 12345);

        // Write and read from R15 (stack pointer)
        regs.write(15, 0xFFFF0000);
        assert_eq!(regs.read(15), 0xFFFF0000);
    }

    #[test]
    fn test_invalid_register_index() {
        let mut regs = Registers::new();

        // Reading invalid index returns 0
        assert_eq!(regs.read(20), 0);

        // Writing to invalid index is ignored (no panic)
        regs.write(20, 42);
    }

    #[test]
    fn test_try_read_error() {
        let regs = Registers::new();

        // Valid index
        assert!(regs.try_read(0).is_ok());

        // Invalid index
        assert!(regs.try_read(16).is_err());
    }

    #[test]
    fn test_try_write_error() {
        let mut regs = Registers::new();

        // Valid index
        assert!(regs.try_write(1, 42).is_ok());

        // Invalid index
        assert!(regs.try_write(16, 42).is_err());
    }

    #[test]
    fn test_flags_update_zn() {
        let mut flags = Flags::new();

        // Zero result
        flags.update_zn(0);
        assert!(flags.zero);
        assert!(!flags.negative);

        // Positive result
        flags.update_zn(42);
        assert!(!flags.zero);
        assert!(!flags.negative);

        // Negative result (when interpreted as signed)
        flags.update_zn(0xFFFFFFFF);
        assert!(!flags.zero);
        assert!(flags.negative);
    }

    #[test]
    fn test_flags_to_from_byte() {
        let flags = Flags {
            zero: true,
            negative: false,
            carry: true,
            overflow: false,
        };

        let byte = flags.to_byte();
        let restored = Flags::from_byte(byte);

        assert_eq!(flags, restored);
    }

    #[test]
    fn test_reset() {
        let mut regs = Registers::new();

        // Modify registers
        regs.write(1, 42);
        regs.write(5, 100);
        regs.pc = 0x1000;
        regs.flags.zero = true;

        // Reset
        regs.reset();

        // Check all reset to zero
        assert_eq!(regs.read(1), 0);
        assert_eq!(regs.read(5), 0);
        assert_eq!(regs.pc, 0);
        assert!(!regs.flags.zero);
    }

    #[test]
    fn test_dump() {
        let mut regs = Registers::new();
        regs.write(1, 42);
        regs.pc = 0x1000;

        let dump = regs.dump();
        assert!(dump.contains("R 1: 0x0000002A"));
        assert!(dump.contains("PC:    0x00001000"));
    }
}
