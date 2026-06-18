use std::ops::{Add, AddAssign};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TCycles(u32);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MCycles(u32);

impl TCycles {
    pub const ZERO: Self = Self(0);

    /// Create a new TCycles with `cycles`
    pub const fn new(cycles: u32) -> Self {
        Self(cycles)
    }

    /// Returns the underlying number of cycles
    pub const fn count(self) -> u32 {
        self.0
    }
}

impl MCycles {
    pub const ZERO: Self = Self(0);

    /// Create a new MCycles with `cycles`.
    pub const fn new(cycles: u32) -> Self {
        Self(cycles)
    }

    /// Returns the underlying number of cycles
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

impl From<MCycles> for TCycles {
    fn from(m_cycles: MCycles) -> Self {
        Self(m_cycles.0 * 4)
    }
}

impl Add for MCycles {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign<Self> for MCycles {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<TCycles> for MCycles {
    fn from(t_cycles: TCycles) -> Self {
        Self(t_cycles.0 / 4)
    }
}

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

impl From<u32> for MCycles {
    fn from(cycles: u32) -> Self {
        Self(cycles)
    }
}

impl From<MCycles> for u32 {
    fn from(cycles: MCycles) -> Self {
        cycles.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_cycles_to_m_cycles() {
        let t_cycles = TCycles::from(8);
        let m_cycles: MCycles = t_cycles.into();
        assert_eq!(m_cycles.count(), 2);
    }

    #[test]
    fn m_cycles_to_t_cycles() {
        let m_cycles = MCycles::from(3);
        let t_cycles: TCycles = m_cycles.into();
        assert_eq!(t_cycles.count(), 12);
    }

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

    #[test]
    fn add_m_cycles() {
        let m1 = MCycles::from(2);
        let m2 = MCycles::from(3);

        let m3 = m1 + m2;
        assert_eq!(m3.count(), 5);
    }

    #[test]
    fn add_assign_m_cycles() {
        let mut m1 = MCycles::from(2);
        let m2 = MCycles::from(3);

        m1 += m2;
        assert_eq!(m1.count(), 5);
    }
}
