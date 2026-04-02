use crate::cpu::registers::Flags;

/// Conditions for conditional instructions.
///
/// Possible conditions are:
/// - [`Condition::NZ`] : "Non-Zero" - check if the zero flag is cleared.
/// - [`Condition::Z`] : "Zero" - check if the zero flag is set.
/// - [`Condition::NC`] : "Non-Carry" - check if the carry flag is cleared.
/// - [`Condition::C`] : "Carry" - check if the carry flag is set.
pub enum Condition {
    /// Non-Zero condition.
    ///
    /// Check if the zero flag is cleared (false) to perform the conditional
    /// instruction.
    NZ,

    /// Zero condition.
    ///
    /// Check if the zero flag is set (true) to perform the conditional
    /// instruction.
    Z,

    /// Non-Carry condition.
    ///
    /// Check if the carry flag is cleared (false) to perform the conditional
    /// instruction.
    NC,

    /// Carry condition.
    ///
    /// Check if the carry flag is set (true) to perform the conditional
    /// instruction.
    C,
}

impl Condition {
    /// Check if the condition is met by the given `flags`.
    ///
    /// Returns true of false depending on the given condition:
    /// - [`Condition::NZ`] : true if zero flag is **not** set.
    /// - [`Condition::Z`] : true if zero flag is set.
    /// - [`Condition::NC`] : true if carry flag is **not** set.
    /// - [`Condition::C`] : true if carry flag is set.
    pub fn check(&self, flags: Flags<'_>) -> bool {
        match self {
            Condition::NZ => !flags.zero(),
            Condition::Z => flags.zero(),
            Condition::NC => !flags.carry(),
            Condition::C => flags.carry(),
        }
    }
}

impl From<u8> for Condition {
    /// Build a [`Condition`] from the given [`u8`] value.
    /// 
    /// This function only considers the 2 least significant bits of the value and 
    /// is intended to be used for the decoding of the opcodes of conditional
    /// instructions.
    /// 
    /// Values are mapped as follows:
    /// - 0 => [`Condition::NZ`]
    /// - 1 => [`Condition::Z`]
    /// - 2 => [`Condition::NC`]
    /// - 3 => [`Condition::C`]
    /// 
    /// # Example
    /// ```no_run
    /// let op_jr_nz_nn = 0x20; // opcode for JR NZ, nn
    /// let op_jr_z_nn = 0x28;  // opcode for JR Z, nn
    /// let op_jr_nc_nn = 0x30;  // opcode for JR NC, nn
    /// let op_jr_c_nn = 0x38;  // opcode for JR C, nn
    /// 
    /// // The condition is encoded in bits 3 and 4 of the opcode, so we shift 
    /// // right by 3 to get the value for the condition
    /// let cond_nz = Condition::from(op_jr_nz_nn >> 3);
    /// let cond_z = Condition::from(op_jr_z_nn >> 3);
    /// let cond_nc = Condition::from(op_jr_nc_nn >> 3);
    /// let cond_c = Condition::from(op_jr_c_nn >> 3);
    /// 
    /// assert!(matches!(cond_nz, Condition::NZ));
    /// assert!(matches!(cond_z, Condition::Z));
    /// assert!(matches!(cond_nc, Condition::NC));
    /// assert!(matches!(cond_c, Condition::C));
    /// ```
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::NZ,
            1 => Self::Z,
            2 => Self::NC,
            3 => Self::C,
            _ => unreachable!("all possible values after mask are mapped"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::registers::Registers;

    #[test]
    fn check() {
        let mut registers = Registers::new(); // zero'ed flags

        assert!(Condition::NZ.check(registers.flags()));
        assert!(!Condition::Z.check(registers.flags()));
        assert!(Condition::NC.check(registers.flags()));
        assert!(!Condition::C.check(registers.flags()));

        registers.flags_mut().zero().set(true);

        assert!(!Condition::NZ.check(registers.flags()));
        assert!(Condition::Z.check(registers.flags()));
        assert!(Condition::NC.check(registers.flags()));
        assert!(!Condition::C.check(registers.flags()));

        registers.flags_mut().carry().set(true);

        assert!(!Condition::NZ.check(registers.flags()));
        assert!(Condition::Z.check(registers.flags()));
        assert!(!Condition::NC.check(registers.flags()));
        assert!(Condition::C.check(registers.flags()));

        registers.flags_mut().carry().set(false);

        assert!(!Condition::NZ.check(registers.flags()));
        assert!(Condition::Z.check(registers.flags()));
        assert!(Condition::NC.check(registers.flags()));
        assert!(!Condition::C.check(registers.flags()));

        registers.flags_mut().zero().set(false);

        assert!(Condition::NZ.check(registers.flags()));
        assert!(!Condition::Z.check(registers.flags()));
        assert!(Condition::NC.check(registers.flags()));
        assert!(!Condition::C.check(registers.flags()));
    }
}