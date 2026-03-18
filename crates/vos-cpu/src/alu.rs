//! Arithmetic Logic Unit (ALU) implementation.
//!
//! The ALU performs all arithmetic and logic operations for the CPU.

use vos_core::{CpuError, Result, Word};

use crate::registers::Flags;

/// Result of an ALU operation.
///
/// Contains the result value and updated CPU flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AluResult {
    /// The result of the operation
    pub value: Word,

    /// Updated CPU flags
    pub flags: Flags,
}

/// Arithmetic Logic Unit.
///
/// Performs arithmetic, logic, and shift operations.
///
/// # Examples
///
/// ```
/// use vos_cpu::alu::Alu;
///
/// let alu = Alu::new();
/// let result = alu.add(10, 20).unwrap();
/// assert_eq!(result.value, 30);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Alu;

impl Alu {
    /// Creates a new ALU.
    pub fn new() -> Self {
        Self
    }

    // ========== Arithmetic Operations ==========

    /// Adds two values.
    ///
    /// Updates flags: zero, negative, carry, overflow
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_cpu::alu::Alu;
    ///
    /// let alu = Alu::new();
    /// let result = alu.add(10, 20).unwrap();
    /// assert_eq!(result.value, 30);
    /// assert!(!result.flags.zero);
    /// ```
    pub fn add(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a.wrapping_add(b);
        let mut flags = Flags::new();

        flags.update_zn(result);

        // Carry: unsigned overflow
        flags.carry = result < a;

        // Overflow: signed overflow
        let a_sign = (a as i32) < 0;
        let b_sign = (b as i32) < 0;
        let result_sign = (result as i32) < 0;
        flags.overflow = (a_sign == b_sign) && (a_sign != result_sign);

        Ok(AluResult { value: result, flags })
    }

    /// Subtracts two values (a - b).
    ///
    /// Updates flags: zero, negative, carry, overflow
    pub fn sub(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a.wrapping_sub(b);
        let mut flags = Flags::new();

        flags.update_zn(result);

        // Carry: borrow occurred (a < b in unsigned arithmetic)
        flags.carry = a < b;

        // Overflow: signed overflow
        let a_sign = (a as i32) < 0;
        let b_sign = (b as i32) < 0;
        let result_sign = (result as i32) < 0;
        flags.overflow = (a_sign != b_sign) && (a_sign != result_sign);

        Ok(AluResult { value: result, flags })
    }

    /// Multiplies two values.
    ///
    /// Updates flags: zero, negative
    ///
    /// Note: Overflow is not detected (result is truncated to 32 bits).
    pub fn mul(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a.wrapping_mul(b);
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Divides two values (a / b).
    ///
    /// Updates flags: zero, negative
    ///
    /// # Errors
    ///
    /// Returns `CpuError::DivisionByZero` if b is zero.
    pub fn div(&self, a: Word, b: Word) -> Result<AluResult> {
        if b == 0 {
            return Err(CpuError::DivisionByZero.into());
        }

        let result = a / b;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    // ========== Logic Operations ==========

    /// Performs bitwise AND.
    ///
    /// Updates flags: zero, negative
    pub fn and(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a & b;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Performs bitwise OR.
    ///
    /// Updates flags: zero, negative
    pub fn or(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a | b;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Performs bitwise XOR.
    ///
    /// Updates flags: zero, negative
    pub fn xor(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a ^ b;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Performs bitwise NOT.
    ///
    /// Updates flags: zero, negative
    pub fn not(&self, a: Word) -> Result<AluResult> {
        let result = !a;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    // ========== Shift Operations ==========

    /// Shift Left Logical.
    ///
    /// Shifts bits left, filling with zeros.
    ///
    /// Updates flags: zero, negative
    pub fn sll(&self, value: Word, shift: u8) -> Result<AluResult> {
        let shift = shift & 0x1F; // Limit to 0-31
        let result = value << shift;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Shift Right Logical.
    ///
    /// Shifts bits right, filling with zeros.
    ///
    /// Updates flags: zero, negative
    pub fn srl(&self, value: Word, shift: u8) -> Result<AluResult> {
        let shift = shift & 0x1F; // Limit to 0-31
        let result = value >> shift;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Shift Right Arithmetic.
    ///
    /// Shifts bits right, preserving sign bit (for signed numbers).
    ///
    /// Updates flags: zero, negative
    pub fn sra(&self, value: Word, shift: u8) -> Result<AluResult> {
        let shift = shift & 0x1F; // Limit to 0-31
        let result = ((value as i32) >> shift) as Word;
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    // ========== Comparison Operations ==========

    /// Set if Less Than (signed).
    ///
    /// Returns 1 if a < b (as signed integers), 0 otherwise.
    pub fn slt(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = if (a as i32) < (b as i32) { 1 } else { 0 };
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }

    /// Set if Greater Than (signed).
    ///
    /// Returns 1 if a > b (as signed integers), 0 otherwise.
    pub fn sgt(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = if (a as i32) > (b as i32) { 1 } else { 0 };
        let mut flags = Flags::new();

        flags.update_zn(result);

        Ok(AluResult { value: result, flags })
    }
}

impl Default for Alu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let alu = Alu::new();

        let result = alu.add(10, 20).unwrap();
        assert_eq!(result.value, 30);
        assert!(!result.flags.zero);
        assert!(!result.flags.negative);
    }

    #[test]
    fn test_add_zero() {
        let alu = Alu::new();

        let result = alu.add(0, 0).unwrap();
        assert_eq!(result.value, 0);
        assert!(result.flags.zero);
    }

    #[test]
    fn test_add_carry() {
        let alu = Alu::new();

        // Unsigned overflow
        let result = alu.add(0xFFFFFFFF, 1).unwrap();
        assert_eq!(result.value, 0);
        assert!(result.flags.carry);
    }

    #[test]
    fn test_add_overflow() {
        let alu = Alu::new();

        // Signed overflow: MAX_INT + 1
        let result = alu.add(0x7FFFFFFF, 1).unwrap();
        assert!(result.flags.overflow);
    }

    #[test]
    fn test_sub() {
        let alu = Alu::new();

        let result = alu.sub(50, 20).unwrap();
        assert_eq!(result.value, 30);
    }

    #[test]
    fn test_sub_negative() {
        let alu = Alu::new();

        let result = alu.sub(20, 50).unwrap();
        assert!(result.flags.negative);
        assert!(result.flags.carry); // Borrow occurred
    }

    #[test]
    fn test_mul() {
        let alu = Alu::new();

        let result = alu.mul(6, 7).unwrap();
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_div() {
        let alu = Alu::new();

        let result = alu.div(100, 5).unwrap();
        assert_eq!(result.value, 20);
    }

    #[test]
    fn test_div_by_zero() {
        let alu = Alu::new();

        let result = alu.div(100, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_and() {
        let alu = Alu::new();

        let result = alu.and(0xFF00, 0x0FF0).unwrap();
        assert_eq!(result.value, 0x0F00);
    }

    #[test]
    fn test_or() {
        let alu = Alu::new();

        let result = alu.or(0xFF00, 0x00FF).unwrap();
        assert_eq!(result.value, 0xFFFF);
    }

    #[test]
    fn test_xor() {
        let alu = Alu::new();

        let result = alu.xor(0xFFFF, 0xFF00).unwrap();
        assert_eq!(result.value, 0x00FF);
    }

    #[test]
    fn test_not() {
        let alu = Alu::new();

        let result = alu.not(0x0000FFFF).unwrap();
        assert_eq!(result.value, 0xFFFF0000);
    }

    #[test]
    fn test_sll() {
        let alu = Alu::new();

        let result = alu.sll(1, 4).unwrap();
        assert_eq!(result.value, 16);
    }

    #[test]
    fn test_srl() {
        let alu = Alu::new();

        let result = alu.srl(16, 4).unwrap();
        assert_eq!(result.value, 1);
    }

    #[test]
    fn test_sra_positive() {
        let alu = Alu::new();

        let result = alu.sra(16, 2).unwrap();
        assert_eq!(result.value, 4);
    }

    #[test]
    fn test_sra_negative() {
        let alu = Alu::new();

        // Negative number (sign bit preserved)
        let result = alu.sra(0xFFFFFFFC_u32, 1).unwrap();
        assert_eq!(result.value, 0xFFFFFFFE);
    }

    #[test]
    fn test_slt() {
        let alu = Alu::new();

        let result = alu.slt(10, 20).unwrap();
        assert_eq!(result.value, 1);

        let result = alu.slt(20, 10).unwrap();
        assert_eq!(result.value, 0);
    }

    #[test]
    fn test_sgt() {
        let alu = Alu::new();

        let result = alu.sgt(20, 10).unwrap();
        assert_eq!(result.value, 1);

        let result = alu.sgt(10, 20).unwrap();
        assert_eq!(result.value, 0);
    }
}
