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
}
