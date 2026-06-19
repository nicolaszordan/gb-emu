use std::ops::{Add, AddAssign};

/// T-cycles newtype wrapper.
///
/// 1 t-cycles represents 1 system tick at 4.19MHz
///
/// Often instruction durations are given in “M-cycles” (machine cycles)
/// instead of “T-states” (system clock ticks) because each instruction takes a
/// multiple of four T-states to complete, thus a NOP takes one M-cycle or four
/// T-states to complete.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TCycles(u32);

impl TCycles {
    pub const ZERO: Self = Self(0);

    /// Create a new TCycles with `cycles`.
    pub const fn new(cycles: u32) -> Self {
        Self(cycles)
    }

    /// Returns the underlying number of cycles.
    pub const fn count(self) -> u32 {
        self.0
    }
}

impl Add for TCycles {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign<Self> for TCycles {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[cfg(test)]
impl From<u32> for TCycles {
    fn from(cycles: u32) -> Self {
        Self(cycles)
    }
}

impl From<TCycles> for u32 {
    fn from(cycles: TCycles) -> Self {
        cycles.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_t_cycles() {
        let t1 = TCycles::from(4);
        let t2 = TCycles::from(8);

        let t3 = t1 + t2;
        assert_eq!(t3.count(), 12);
    }

    #[test]
    fn add_assign_t_cycles() {
        let mut t1 = TCycles::from(4);
        let t2 = TCycles::from(8);

        t1 += t2;
        assert_eq!(t1.count(), 12);
    }
}
