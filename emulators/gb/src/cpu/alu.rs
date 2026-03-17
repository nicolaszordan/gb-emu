/// ALU instruction Flags.
/// 
/// All ALU instructions return a value and the Flags that should be set by the
/// CPU.
#[derive(Debug, Default)]
pub(crate) struct Flags {
    zero: Option<bool>,
    subtract: Option<bool>,
    half_carry: Option<bool>,
    carry: Option<bool>,
}

impl Flags {
    /// Get the value of the zero flag.
    ///
    /// The zero flag is set if the result of the last ALU operation was 0.
    /// 
    /// Returns `None` if the instruction does not influence this flag.
    pub(crate) fn zero(&self) -> Option<bool> {
        self.zero
    }

    /// Get the value of the subtract flag.
    ///
    /// The subtract flag is set if the last ALU operation was a subtraction
    /// (eg. SUB, SBC, DEC).
    /// 
    /// Returns `None` if the instruction does not influence this flag.
    pub(crate) fn subtract(&self) -> Option<bool> {
        self.subtract
    }

    /// Get the value of the half carry flag.
    ///
    /// The half carry flag is set if the last ALU operation caused a carry
    /// from bit 3 to bit 4 (for 8-bit operations) or from bit 11 to bit 12
    /// (for 16-bit operations).
    /// 
    /// Returns `None` if the instruction does not influence this flag.
    pub(crate) fn half_carry(&self) -> Option<bool> {
        self.half_carry
    }

    /// Get the value of the carry flag.
    ///
    /// The carry flag is set if the last ALU operation caused a carry from bit
    /// 7 to bit 8 (for 8-bit operations) or from bit 15 to bit 16 (for 16-bit
    /// operations), or if a subtraction operation caused a borrow (ie. if the
    /// subtracted value was greater than the original value).
    /// 
    /// Returns `None` if the instruction does not influence this flag.
    pub(crate) fn carry(&self) -> Option<bool> {
        self.carry
    }
}

/// An Index of a bit in a byte, used for bit manipulation instructions
/// (eg. BIT, RES, SET).
/// Values are expected to be in the range 0-7, where 0 corresponds to the least
/// significant bit
type BitIndex = u8; // 0-7

/// Add two 8-bit values.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : Set if overflow from bit 3.
/// - C : Set if overflow from bit 7.
pub fn add(a: u8, b: u8) -> (u8, Flags) {
    let (result, carry) = a.overflowing_add(b);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some((a & 0xF) + (b & 0xF) > 0xF),
            carry: Some(carry),
        },
    )
}

/// Add two 8-bit values and the given carry flag.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : Set if overflow from bit 3.
/// - C : Set if overflow from bit 7.
pub fn adc(a: u8, b: u8, carry: bool) -> (u8, Flags) {
    let (result, result_carry) = a.carrying_add(b, carry);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some((a & 0xF) + (b & 0xF) + carry as u8 > 0xF),
            carry: Some(result_carry),
        },
    )
}

/// Add two 16-bit values.
///
/// # Flags
/// - Z : Not affected.
/// - N : 0 (cleared).
/// - H : Set if overflow from bit 11.
/// - C : Set if overflow from bit 15.
pub fn add16(a: u16, b: u16) -> (u16, Flags) {
    let (result, carry) = a.overflowing_add(b);
    (
        result,
        Flags {
            zero: None,
            subtract: Some(false),
            half_carry: Some((a & 0xFFF) + (b & 0xFFF) > 0xFFF),
            carry: Some(carry),
        },
    )
}

/// Subtract two 8-bit values.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 1 (set).
/// - H : Set if borrow from bit 4.
/// - C : Set if borrow (ie. B > A).
pub fn sub(a: u8, b: u8) -> (u8, Flags) {
    let (result, carry) = a.overflowing_sub(b);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(true),
            half_carry: Some((a & 0xF) < (b & 0xF)),
            carry: Some(carry),
        },
    )
}

/// Subtract two 8-bit values and the given carry flag.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 1 (set).
/// - H : Set if borrow from bit 4.
/// - C : Set if borrow (ie. B + C > A).
pub fn sbc(a: u8, b: u8, carry: bool) -> (u8, Flags) {
    let result = a.wrapping_sub(b).wrapping_sub(carry as u8);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(true),
            half_carry: Some((a & 0xF) < (b & 0xF) + carry as u8),
            carry: Some((a as u16) < (b as u16) + (carry as u16)),
        },
    )
}

/// Perform a bitwise AND on two 8-bit values.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 1 (set).
/// - C : 0 (cleared).
pub fn and(a: u8, b: u8) -> (u8, Flags) {
    let result = a & b;
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(true),
            carry: Some(false),
        },
    )
}

/// Perform a bitwise OR on two 8-bit values.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : 0 (cleared).
pub fn or(a: u8, b: u8) -> (u8, Flags) {
    let result = a | b;
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some(false),
        },
    )
}

/// Perform a bitwise XOR on two 8-bit values.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : 0 (cleared).
pub fn xor(a: u8, b: u8) -> (u8, Flags) {
    let result = a ^ b;
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some(false),
        },
    )
}

/// ComPare two 8-bit values.
///
/// Perform a subtraction of `b` from `a` and discard the result. Returns the
/// flags set according to the result of the operation.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 1 (set).
/// - H : Set if borrow from bit 4.
/// - C : Set if borrow (ie. B > A).
pub fn cp(a: u8, b: u8) -> Flags {
    let (result, carry) = a.overflowing_sub(b);
    Flags {
        zero: Some(result == 0),
        subtract: Some(true),
        half_carry: Some((a & 0xF) < (b & 0xF)),
        carry: Some(carry),
    }
}

/// Increment an 8-bit value.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : Set if overflow from bit 3.
/// - C : Not affected.
pub fn inc(value: u8) -> (u8, Flags) {
    let result = value.wrapping_add(1);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some((result & 0xF) == 0), // check carry from bit 4
            carry: None,
        },
    )
}

/// Increment a 16-bit value.
///
/// # Flags
/// **None**
///
/// # Notes
/// The `inc16` instructions **do not** affect any flag.
pub fn inc16(value: u16) -> (u16, Flags) {
    let result = value.wrapping_add(1);
    (
        result,
        Flags {
            zero: None,
            subtract: None,
            half_carry: None,
            carry: None,
        }
    ) 
}

/// Decrement an 8-bit value.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 1 (set).
/// - H : Set if borrow from bit 4.
/// - C : Not affected.
pub fn dec(value: u8) -> (u8, Flags) {
    let result = value.wrapping_sub(1);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(true),
            half_carry: Some((result & 0xF) == 0xF), // check borrow from bit 4
            carry: None,
        },
    )
}

/// Decrement a 16-bit value.
///
/// # Flags
/// **None**
///
/// # Notes
/// The `dec16` instructions **do not** affect any flag.
pub fn dec16(value: u16) -> (u16, Flags) {
    let result = value.wrapping_sub(1);
    (
        result,
        Flags {
            zero: None,
            subtract: None,
            half_carry: None,
            carry: None,
        }
    ) 
}

/// Rotate value left.
///
/// ```text
/// ┏━ Flags ━┓   ┏━━━━━━ val ━━━━━━┓
/// ┃    C   ←╂─┬─╂─ b7 ← ... ← b0 ←╂─┐
/// ┗━━━━━━━━━┛ │ ┗━━━━━━━━━━━━━━━━━┛ │
///             └─────────────────────┘
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
///
/// # Note
/// The RLCA instruction (opcode: 0x07) is a special case of the RLC instruction
/// that rotates the A register and contrary to the regular RLC instruction, it
/// **should not affect** the zero flag.
pub fn rlc(value: u8) -> (u8, Flags) {
    let result = value.rotate_left(1);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0b1000_0000) != 0), // check the 8th bit of the original value
        },
    )
}

/// Rotate value right.
///
/// ```text
///   ┏━━━━━━ [HL] ━━━━━┓   ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─┬─╂→   C    ┃
/// │ ┗━━━━━━━━━━━━━━━━━┛ │ ┗━━━━━━━━━┛
/// └─────────────────────┘
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
///
/// # Note
/// The RRCA instruction (opcode: 0x0F) is a special case of the RRC instruction
/// that only rotates the A register and contrary to the regular RRC
/// instruction, it **does not affect** the zero flag.
pub fn rrc(value: u8) -> (u8, Flags) {
    let result = value.rotate_right(1);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0x1) != 0), // check the 1st bit of the original value
        },
    )
}

/// Rotate bits in value left, through the given carry flag.
///
/// ```text
///   ┏━ Flags ━┓ ┏━━━━━━━ r8 ━━━━━━┓
/// ┌─╂─   C   ←╂─╂─ b7 ← ... ← b0 ←╂─┐
/// │ ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━┛ │
/// └─────────────────────────────────┘
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
///
/// # Note
/// The RLA instruction (opcode: 0x17) is a special case of the RL instruction
/// that only rotates the A register and contrary to the regular RL
/// instruction, it **does not affect** the zero flag.
pub fn rl(value: u8, carry: bool) -> (u8, Flags) {
    let result = (value << 1) | carry as u8;
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0b1000_0000) != 0), // check the 8th bit of the original value
        },
    )
}

/// Rotate bits in value right, through the given carry flag.
///
/// ```text
///   ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └─────────────────────────────────┘
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
///
/// # Note
/// The RRA instruction (opcode: 0x1F) is a special case of the RR instruction
/// that only rotates the A register and contrary to the regular RR instruction,
/// it **does not affect** the zero flag.
pub fn rr(value: u8, carry: bool) -> (u8, Flags) {
    let result = (value >> 1) | ((carry as u8) << 7);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0x1) != 0), // check the 1st bit of the original value
        },
    )
}

/// Logical Left Shift value.
///
/// ```text
/// ┏━ Flags ━┓ ┏━━━━━━━ r8 ━━━━━━┓
/// ┃    C   ←╂─╂─ b7 ← ... ← b0 ←╂─ 0
/// ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━┛
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
pub fn sla(value: u8) -> (u8, Flags) {
    let result = value << 1;
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0b1000_0000) != 0), // check the 8th bit of the original value
        },
    )
}

/// Arithmetic Right Shift value (bit7 remains unchanged).
///
/// ```text
/// ┏━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// ┃ b7 → ... → b0 ─╂─╂→   C    ┃
/// ┗━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
pub fn sra(value: u8) -> (u8, Flags) {
    let result = (value >> 1) | (value & 0b1000_0000); // keep the 8th bit unchanged
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0b1) != 0), // check the 1st bit of the original value
        },
    )
}

/// Swap the upper 4 bits and lower 4 bits of value.
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : 0 (cleared).
pub fn swap(value: u8) -> (u8, Flags) {
    let result = (value << 4) | (value >> 4);
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some(false),
        },
    )
}

/// Logical Right Shift value.
///
/// ```text
///    ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// 0 ─╂→ b7 → ... → b0 ─╂─╂→   C    ┃
///    ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
/// ```
///
/// # Flags
/// - Z : Set if result is 0.
/// - N : 0 (cleared).
/// - H : 0 (cleared).
/// - C : Set according to result.
pub fn srl(value: u8) -> (u8, Flags) {
    let result = value >> 1;
    (
        result,
        Flags {
            zero: Some(result == 0),
            subtract: Some(false),
            half_carry: Some(false),
            carry: Some((value & 0b1) != 0), // check the 1st bit of the original value
        },
    )
}

/// Test bit at `index` in `value`.
///
/// Indexes go from 0 to 7 where 0 is the right most bit (LSB).
///
/// # Flags
/// - Z : Set if the selected bit is 0.
/// - N : 0 (cleared).
/// - H : 1 (set).
/// - C : Not affected.
pub fn bit(index: BitIndex, value: u8) -> Flags {
    let is_bit_set = (value & (1 << index)) != 0;
    Flags {
        zero: Some(!is_bit_set), // flag is set if the tested bit is 0
        subtract: Some(false),
        half_carry: Some(true),
        carry: None,
    }
}

/// Set bit at `index` in `value` to 1.
///
/// Indexes go from 0 to 7 where 0 is the right most bit (LSB).
///
/// # Flags
/// **None**
///
/// # Notes
/// The `set` instructions **do not** affect any flag.
pub fn set(index: BitIndex, value: u8) -> (u8, Flags) {
    let result = value | (1 << index);
    (
        result,
        Flags { // Note: The SET instruction does not affect any flags.
            zero: None,
            subtract: None,
            half_carry: None,
            carry: None,
        }
    )
}

/// Set bit at `index` in `value` to 0.
///
/// Indexes go from 0 to 7 where 0 is the right most bit (LSB).
///
/// # Flags
/// **None**
///
/// # Notes
/// The `res` instructions **do not** affect any flag.
pub fn res(index: BitIndex, value: u8) -> (u8, Flags) {
    let result = value & !(1 << index);
    (
        result,
        Flags { // Note: The RES instruction does not affect any flags.
            zero: None,
            subtract: None,
            half_carry: None,
            carry: None,
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- add ---

    #[test]
    fn add_basic() {
        let (result, flags) = add(1, 2);
        assert_eq!(result, 3);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn add_zero_flag() {
        let (result, flags) = add(0, 0);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = add(1, 2);
        assert_eq!(result, 3);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        // wrapping adds who result in 0 also set the z-flag
        let (result, flags) = add(0xFF, 1);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn add_half_carry() {
        let (result, flags) = add(0x0F, 0x01);
        assert_eq!(result, 0x10);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = add(0x0E, 0x01);
        assert_eq!(result, 0x0F);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        // h-flag only cares if the lower nibble overflows
        let (_, flags) = add(0xFE, 0x11);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
        let (_, flags) = add(0xFF, 0x33);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn add_carry_flag() {
        let (result, flags) = add(0x80, 0x80);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));

        let (_, flags) = add(1, 2);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (_, flags) = add(0xFF, 0xFF);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    // --- adc ---

    #[test]
    fn adc_basic() {
        let (result, flags) = adc(1, 1, true);
        assert_eq!(result, 3);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = adc(1, 1, false);
        assert_eq!(result, 2);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn adc_zero_flag() {
        let (result, flags) = adc(0, 0, false);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = adc(0, 0, true);
        assert_eq!(result, 0x01);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = adc(0xFE, 0x01, true);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn adc_carry_flag() {
        let (result, flags) = adc(0xFE, 0x01, true);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));

        let (result, flags) = adc(0xFE, 0x01, false);
        assert_eq!(result, 0xFF);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = adc(0xF0, 0x0F, true);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));

        let (_, flags) = adc(0xF0, 0xF0, false);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn adc_half_carry() {
        // 0x0F + 0x30 + carry(1) = 0x40 => half carry
        let (result, flags) = adc(0x0F, 0x30, true);
        assert_eq!(result, 0x40);
        assert_eq!(flags.half_carry(), Some(true));

        let (result, flags) = adc(0x0F, 0x30, false);
        assert_eq!(result, 0x3F);
        assert_eq!(flags.half_carry(), Some(false));

        let (_, flags) = adc(0xFF, 0x01, true);
        assert_eq!(flags.half_carry(), Some(true));

        let (_, flags) = adc(0xFE, 0x10, true);
        assert_eq!(flags.half_carry(), Some(false));
    }

    // --- add16 ---

    #[test]
    fn add16_basic() {
        let (result, flags) = add16(0x1234, 0x0001);
        assert_eq!(result, 0x1235);
        assert_eq!(flags.zero(), None);
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn add16_half_carry() {
        let (result, flags) = add16(0x0FFF, 0x0001);
        assert_eq!(result, 0x1000);
        assert_eq!(flags.zero(), None);
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = add16(0x0FFE, 0x0001);
        assert_eq!(result, 0x0FFF);
        assert_eq!(flags.zero(), None);
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (_, flags) = add16(0xFFFE, 0x1001);
        assert_eq!(flags.zero(), None);
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn add16_carry() {
        let (result, flags) = add16(0xFFFF, 0x0001);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), None);
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));

        let (_, flags) = add16(0x0012, 0x0001);
        assert_eq!(flags.zero(), None);
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- sub ---

    #[test]
    fn sub_basic() {
        let (result, flags) = sub(5, 3);
        assert_eq!(result, 2);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn sub_zero_flag() {
        let (result, flags) = sub(5, 5);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (_, flags) = sub(5, 4);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = sub(0, 0);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn sub_carry_flag() {
        let (result, flags) = sub(0x00, 0x01);
        assert_eq!(result, 0xFF);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn sub_half_carry() {
        // Lower nibble borrows: 0x10 - 0x01
        let (result, flags) = sub(0x10, 0x01);
        assert_eq!(result, 0x0F);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));

        // Lower nibble doesn't borrow: 0x22 - 0x11
        let (_, flags) = sub(0x22, 0x11);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- sbc ---

    #[test]
    fn sbc_basic() {
        let (result, flags) = sbc(5, 3, true);
        assert_eq!(result, 1);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn sbc_zero_flag() {
        let (result, flags) = sbc(5, 4, true);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (_, flags) = sbc(5, 4, false);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = sbc(4, 4, false);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn sbc_carry_flag() {
        let (result, flags) = sbc(0x00, 0x00, true);
        assert_eq!(result, 0xFF);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));

        let (result, flags) = sbc(0x00, 0x00, false);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = sbc(0xFF, 0xFF, true); // we subtract 1 from 0xFF - 0xFF
        assert_eq!(result, 0xFF);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn sbc_half_carry_flag() {
        let (result, flags) = sbc(0x10, 0x00, true);
        assert_eq!(result, 0x0F);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));

        let (_, flags) = sbc(0x00, 0x00, false);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = sbc(0xFF, 0xFF, true); // we subtract 1 from 0xFF - 0xFF
        assert_eq!(result, 0xFF);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    // --- and ---

    #[test]
    fn and_basic() {
        let (result, flags) = and(0b1100, 0b1010);
        assert_eq!(result, 0b1000);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn and_zero_flag() {
        let (result, flags) = and(0b1100, 0b0011);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- or ---

    #[test]
    fn or_basic() {
        let (result, flags) = or(0b1100, 0b0011);
        assert_eq!(result, 0b1111);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn or_zero_flag() {
        let (result, flags) = or(0, 0);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- xor ---

    #[test]
    fn xor_basic() {
        let (result, flags) = xor(0b1100, 0b1010);
        assert_eq!(result, 0b0110);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn xor_zero_flag() {
        let (result, flags) = xor(0xAB, 0xAB);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- cp ---

    #[test]
    fn cp_basic() {
        let flags = cp(5, 3);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn cp_zero_flag() {
        let flags = cp(5, 5);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let flags = cp(5, 4);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let flags = cp(0, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn cp_carry_flag() {
        let flags = cp(0x00, 0x01);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn cp_half_carry() {
        // Lower nibble borrows: 0x10 - 0x01
        let flags = cp(0x10, 0x01);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), Some(false));

        // Lower nibble doesn't borrow: 0x22 - 0x11
        let flags = cp(0x22, 0x11);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- inc ---

    #[test]
    fn inc_basic() {
        let (result, flags) = inc(5);
        assert_eq!(result, 6);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), None);
    }

    #[test]
    fn inc_zero_flag() {
        let (result, flags) = inc(0xFF);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), None);
    }

    #[test]
    fn inc_half_carry() {
        let (result, flags) = inc(0x0F);
        assert_eq!(result, 0x10); // 0x0F + 1 = 0x10 => half carry
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), None);

        let (_, flags) = inc(0x0E);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), None);
    }

    // --- inc16 ---

    #[test]
    fn inc16_basic() {
        let (result, flags) = inc16(2);
        assert_eq!(result, 3);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());

        let (result, flags) = inc16(0xFFFF);
        assert_eq!(result, 0x0000);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());
    }

    // --- dec ---

    #[test]
    fn dec_basic() {
        let (result, flags) = dec(5);
        assert_eq!(result, 4);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), None);
    }

    #[test]
    fn dec_zero_flag() {
        let (result, flags) = dec(1);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), None);
    }

    #[test]
    fn dec_half_carry() {
        let (_, flags) = dec(0x10); // 0x10 - 1 = 0x0F => borrow from bit 4
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), None);

        let (_, flags) = dec(0x11);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(true));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), None);
    }

    // --- dec16 ---

    #[test]
    fn dec16_basic() {
        let (result, flags) = dec16(2);
        assert_eq!(result, 1);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());

        let (result, flags) = dec16(0x0000);
        assert_eq!(result, 0xFFFF);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());
    }


    // --- rlc ---

    #[test]
    fn rlc_basic() {
        // 0b0000_0010 rotated left => 0b0000_0100
        let (result, flags) = rlc(0b0000_0010);
        assert_eq!(result, 0b0000_0100);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn rlc_carry_flag() {
        // 0b1000_0001 rotated left => 0b0000_0011, carry = 1
        let (result, flags) = rlc(0b1000_0001);
        assert_eq!(result, 0b0000_0011);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn rlc_zero_flag() {
        let (result, flags) = rlc(0);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- rrc ---

    #[test]
    fn rrc_basic() {
        // 0b0000_0100 rotated right => 0b0000_0010
        let (result, flags) = rrc(0b0000_0100);
        assert_eq!(result, 0b0000_0010);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn rrc_carry_from_bit0() {
        // 0b1000_0001 rotated right => 0b1100_0000, carry = 1
        let (result, flags) = rrc(0b1000_0001);
        assert_eq!(result, 0b1100_0000);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn rrc_zero_flag() {
        let (result, flags) = rrc(0);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- rl ---

    #[test]
    fn rl_no_carry_in_no_carry_out() {
        // carry flag = 0; shift 0b0000_0010 left => 0b0000_0100
        let (result, flags) = rl(0b0000_0010, false);
        assert_eq!(result, 0b0000_0100);
        assert_eq!(flags.carry(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.zero(), Some(false));
    }

    #[test]
    fn rl_carry_in_enters_bit0() {
        // 0b0000_0000 | carry => 0b0000_0001
        let (result, flags) = rl(0b0000_0000, true);
        assert_eq!(result, 0b0000_0001);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn rl_bit7_becomes_carry() {
        let (result, flags) = rl(0b1000_0000, false);
        assert_eq!(result, 0b0000_0000);
        assert_eq!(flags.carry(), Some(true));
        assert_eq!(flags.zero(), Some(true));
    }

    // --- rr ---

    #[test]
    fn rr_no_carry_in_no_carry_out() {
        let (result, flags) = rr(0b0000_0100, false);
        assert_eq!(result, 0b0000_0010);
        assert_eq!(flags.carry(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.zero(), Some(false));
    }

    #[test]
    fn rr_carry_in_enters_bit7() {
        let (result, flags) = rr(0b0000_0000, true);
        assert_eq!(result, 0b1000_0000);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn rr_bit0_becomes_carry() {
        let (result, flags) = rr(0b0000_0001, false);
        assert_eq!(result, 0b0000_0000);
        assert_eq!(flags.carry(), Some(true));
        assert_eq!(flags.zero(), Some(true));
    }

    // --- sla ---

    #[test]
    fn sla_basic() {
        let (result, flags) = sla(0b0000_0010);
        assert_eq!(result, 0b0000_0100);
        assert_eq!(flags.carry(), Some(false));
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
    }

    #[test]
    fn sla_bit7_becomes_carry() {
        let (result, flags) = sla(0b1000_0001);
        assert_eq!(result, 0b0000_0010);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn sla_zero_flag() {
        let (result, flags) = sla(0b1000_0000);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    // --- sra ---

    #[test]
    fn sra_preserves_msb() {
        // bit 7 is preserved
        let (result, flags) = sra(0b1000_0010);
        assert_eq!(result, 0b1100_0001);
        assert_eq!(flags.carry(), Some(false));
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
    }

    #[test]
    fn sra_bit0_becomes_carry() {
        let (result, flags) = sra(0b0000_0001);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    #[test]
    fn sra_zero_flag() {
        let (result, flags) = sra(0b0000_0000);
        assert_eq!(result, 0);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- swap ---

    #[test]
    fn swap_basic() {
        let (result, flags) = swap(0xAB);
        assert_eq!(result, 0xBA);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));

        let (result, flags) = swap(0x0A);
        assert_eq!(result, 0xA0);
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    #[test]
    fn swap_zero_flag() {
        let (result, flags) = swap(0x00);
        assert_eq!(result, 0x00);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(false));
    }

    // --- srl ---

    #[test]
    fn srl_basic() {
        let (result, flags) = srl(0b1000_0010);
        assert_eq!(result, 0b0100_0001);
        assert_eq!(flags.carry(), Some(false));
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
    }

    #[test]
    fn srl_bit0_becomes_carry() {
        let (result, flags) = srl(0b0000_0001);
        assert_eq!(result, 0b0000_0000);
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(false));
        assert_eq!(flags.carry(), Some(true));
    }

    // --- bit ---

    #[test]
    fn bit_set_clears_zero() {
        let flags = bit(3, 0b0000_1000); // bit 3 is 1 => zero flag = false
        assert_eq!(flags.zero(), Some(false));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), None);
    }

    #[test]
    fn bit_clear_sets_zero() {
        let flags = bit(3, 0b0000_0000); // bit 3 is 0 => zero flag = true
        assert_eq!(flags.zero(), Some(true));
        assert_eq!(flags.subtract(), Some(false));
        assert_eq!(flags.half_carry(), Some(true));
        assert_eq!(flags.carry(), None);
    }

    // --- res ---

    #[test]
    fn res_clears_bit() {
        let (result, flags) = res(3, 0b0000_1111);
        assert_eq!(result, 0b0000_0111);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());
    }

    #[test]
    fn res_idempotent_when_already_clear() {
        let (result, flags) = res(3, 0b0000_0000);
        assert_eq!(result, 0b0000_0000);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());
    }

    // --- set ---

    #[test]
    fn set_sets_bit() {
        let (result, flags) = set(3, 0b0000_0000);
        assert_eq!(result, 0b0000_1000);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());
    }

    #[test]
    fn set_idempotent_when_already_set() {
        let (result, flags) = set(3, 0b0000_1000);
        assert_eq!(result, 0b0000_1000);
        assert!(flags.zero().is_none());
        assert!(flags.subtract().is_none());
        assert!(flags.half_carry().is_none());
        assert!(flags.carry().is_none());
    }
}
