use std::ops::Deref;

/// A newtype wrapper around `u8` that represents a valid bit index in the range `[0, 7]`.
///
/// Bit indices are used to address individual bits within a byte. Since a byte has 8 bits,
/// only values 0 through 7 are meaningful. `BitIndex` enforces this constraint by masking
/// the lower 3 bits on construction (`value & 0b111`), so any `u8` can be safely converted
/// into a `BitIndex` without panicking.
///
/// # Examples
///
/// ```no_run
/// let idx = BitIndex::new(5u8);
/// assert_eq!(*idx, 5);
///
/// // Values outside [0, 7] are masked down automatically.
/// let idx = BitIndex::from(10u8); // 10 & 0b111 == 2
/// assert_eq!(*idx, 2);
///
/// // Convert back to u8.
/// let raw: u8 = idx.into();
/// assert_eq!(raw, 2);
/// ```
pub struct BitIndex(u8);

impl BitIndex {
    /// Creates a new [BitIndex] from the given `value`.
    /// 
    /// Note that the value is clamped to `[0, 7]` by masking the lower 3 bits.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// let idx = BitIndex::new(2u8);
    /// assert_eq!(*idx, 2);
    /// ```
    pub fn new(value: u8) -> Self {
        value.into()
    }

    /// Returns a bitmask with only the bit at this index set.
    ///
    /// Equivalent to `1 << self`, useful for isolating or toggling a specific
    /// bit within a byte.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let idx = BitIndex::new(3u8);
    /// assert_eq!(idx.bit_mask(), 0b0000_1000);
    /// ```
    pub fn bit_mask(&self) -> u8 {
        1 << self.0
    }
}

impl From<u8> for BitIndex {
    /// Converts a `u8` into a `BitIndex`, clamping the value to `[0, 7]` by masking
    /// the lower 3 bits.
    fn from(value: u8) -> Self {
        Self { 0: value & 0b111 }
    }
}

impl From<BitIndex> for u8 {
    /// Extracts the underlying `u8` value from a `BitIndex`.
    fn from(value: BitIndex) -> Self {
        value.0
    }
}

impl Deref for BitIndex {
    type Target = u8;

    /// Dereferences to the underlying `u8`, allowing `BitIndex` to be used
    /// transparently wherever a `&u8` is expected.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_mask() {
        let idx = BitIndex::new(4);
        assert_eq!(idx.bit_mask(), 0b1_0000);
        let idx = BitIndex::new(3);
        assert_eq!(idx.bit_mask(), 0b1000);
        let idx = BitIndex::new(7);
        assert_eq!(idx.bit_mask(), 0b1000_0000);
        let idx = BitIndex::new(0);
        assert_eq!(idx.bit_mask(), 0b1);
    }
}