use super::OpcodeExtractionError;
use crate::cpu::registers::Flags;

/// Conditions for conditional instructions.
///
/// Possible conditions are:
/// - [`Condition::NZ`] : "Non-Zero" - check if the zero flag is cleared.
/// - [`Condition::Z`] : "Zero" - check if the zero flag is set.
/// - [`Condition::NC`] : "Non-Carry" - check if the carry flag is cleared.
/// - [`Condition::C`] : "Carry" - check if the carry flag is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Check if the condition is met by the given `flags` register.
    ///
    /// Returns true of false depending on the given condition:
    /// - [`Condition::NZ`] : true if zero flag is **not** set.
    /// - [`Condition::Z`] : true if zero flag is set.
    /// - [`Condition::NC`] : true if carry flag is **not** set.
    /// - [`Condition::C`] : true if carry flag is set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut flags = Flags::new(); // zero'ed flags
    ///
    /// assert!(Condition::NZ.check(&flags));
    /// assert!(!Condition::Z.check(&flags));
    /// assert!(Condition::NC.check(&flags));
    /// assert!(!Condition::C.check(&flags));
    ///
    /// flags.z = true;
    /// assert!(!Condition::NZ.check(&flags));
    /// assert!(Condition::Z.check(&flags));
    /// ```
    pub const fn check(self, flags: Flags) -> bool {
        match self {
            Self::NZ => !flags.z,
            Self::Z => flags.z,
            Self::NC => !flags.c,
            Self::C => flags.c,
        }
    }
}

impl TryFrom<u8> for Condition {
    type Error = OpcodeExtractionError;

    /// Try to build a [`Condition`] from the given [`u8`] value, returns [`OpcodeExtractionError`]
    /// if the given value is invalid.
    ///
    /// This function expects values to be in the range [0..=3].
    ///
    /// Values are mapped as follows:
    /// - 0 => [`Condition::NZ`]
    /// - 1 => [`Condition::Z`]
    /// - 2 => [`Condition::NC`]
    /// - 3 => [`Condition::C`]
    ///
    /// # Example
    ///
    /// ```ignore
    /// let op_jr_nz_nn = 0x20; // opcode for JR NZ, nn
    /// let op_jr_z_nn = 0x28;  // opcode for JR Z,  nn
    /// let op_jr_nc_nn = 0x30; // opcode for JR NC, nn
    /// let op_jr_c_nn = 0x38;  // opcode for JR C,  nn
    ///
    /// // The condition is encoded in bits 3 and 4 of the opcode, so we shift
    /// // right by 3 to get the value for the condition and mask it
    /// let cond_nz = Condition::try_from((op_jr_nz_nn >> 3) & 0b11).unwrap();
    /// let cond_z = Condition::try_from((op_jr_z_nn >> 3) & 0b11).unwrap();
    /// let cond_nc = Condition::try_from((op_jr_nc_nn >> 3) & 0b11).unwrap();
    /// let cond_c = Condition::try_from((op_jr_c_nn >> 3) & 0b11).unwrap();
    ///
    /// assert!(matches!(cond_nz, Condition::NZ));
    /// assert!(matches!(cond_z, Condition::Z));
    /// assert!(matches!(cond_nc, Condition::NC));
    /// assert!(matches!(cond_c, Condition::C));
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NZ),
            1 => Ok(Self::Z),
            2 => Ok(Self::NC),
            3 => Ok(Self::C),
            _ => Err(OpcodeExtractionError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check() {
        let mut flags = Flags::new(); // zero'ed flags

        assert!(Condition::NZ.check(flags));
        assert!(!Condition::Z.check(flags));
        assert!(Condition::NC.check(flags));
        assert!(!Condition::C.check(flags));

        flags.z = true;

        assert!(!Condition::NZ.check(flags));
        assert!(Condition::Z.check(flags));
        assert!(Condition::NC.check(flags));
        assert!(!Condition::C.check(flags));

        flags.c = true;

        assert!(!Condition::NZ.check(flags));
        assert!(Condition::Z.check(flags));
        assert!(!Condition::NC.check(flags));
        assert!(Condition::C.check(flags));

        flags.c = false;

        assert!(!Condition::NZ.check(flags));
        assert!(Condition::Z.check(flags));
        assert!(Condition::NC.check(flags));
        assert!(!Condition::C.check(flags));

        flags.z = false;

        assert!(Condition::NZ.check(flags));
        assert!(!Condition::Z.check(flags));
        assert!(Condition::NC.check(flags));
        assert!(!Condition::C.check(flags));
    }

    #[test]
    fn try_from() {
        let op_jr_nz_nn = 0x20; // opcode for JR NZ, nn
        let op_jr_z_nn = 0x28; // opcode for JR Z,  nn
        let op_jr_nc_nn = 0x30; // opcode for JR NC, nn
        let op_jr_c_nn = 0x38; // opcode for JR C,  nn

        // The condition is encoded in bits 3 and 4 of the opcode, so we shift
        // right by 3 to get the value for the condition
        let cond_nz = Condition::try_from((op_jr_nz_nn >> 3) & 0b11).unwrap();
        let cond_z = Condition::try_from((op_jr_z_nn >> 3) & 0b11).unwrap();
        let cond_nc = Condition::try_from((op_jr_nc_nn >> 3) & 0b11).unwrap();
        let cond_c = Condition::try_from((op_jr_c_nn >> 3) & 0b11).unwrap();

        assert!(matches!(cond_nz, Condition::NZ));
        assert!(matches!(cond_z, Condition::Z));
        assert!(matches!(cond_nc, Condition::NC));
        assert!(matches!(cond_c, Condition::C));
    }
}
