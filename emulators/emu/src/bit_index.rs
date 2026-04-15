/// A validated bit index in the range `0..=7`, used to address individual bits within a byte.
///
/// Wraps a `u8` and guarantees it is always less than 8, making bit manipulation
/// operations safe and explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BitIndex(u8);

impl BitIndex {
    /// Creates a `BitIndex` from a `u8`, returning an error if the value is out of range.
    ///
    /// ```
    /// # use emu::{BitIndex, BitIndexOutOfRange};
    /// assert!(BitIndex::new(3).is_ok());
    /// assert_eq!(BitIndex::new(8), Err(BitIndexOutOfRange(8)));
    /// ```
    pub fn new(value: u8) -> Result<Self, BitIndexOutOfRange> {
        if value < 8 {
            Ok(Self(value))
        } else {
            Err(BitIndexOutOfRange(value))
        }
    }

    /// Creates a `BitIndex` without checking that the value is in range.
    ///
    /// # Safety
    ///
    /// The caller must ensure `value < 8`. Passing a value `>= 8` results in
    /// undefined behavior when the index is later used in bit operations.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = unsafe { BitIndex::new_unchecked(5) };
    /// assert_eq!(idx.index(), 5);
    /// ```
    pub unsafe fn new_unchecked(value: u8) -> Self {
        Self(value)
    }

    /// Creates a `BitIndex` by masking the low 3 bits of `value`, so the result is
    /// always in range `0..=7`.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// // Only the low 3 bits (0b110 = 6) are kept.
    /// assert_eq!(BitIndex::from_low_bits(0b1010_0110).index(), 6);
    /// ```
    pub fn from_low_bits(value: u8) -> Self {
        Self(value & 0b111)
    }

    /// Returns the underlying bit index as a `u8`.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = BitIndex::new(4).unwrap();
    /// assert_eq!(idx.index(), 4);
    /// ```
    pub fn index(self) -> u8 {
        self.0
    }

    /// Returns a bitmask with only the bit at this index set.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = BitIndex::new(3).unwrap();
    /// assert_eq!(idx.to_bit_mask(), 0b0000_1000);
    /// ```
    pub fn to_bit_mask(self) -> u8 {
        1 << self.0
    }

    /// Returns `true` if the bit at this index is set in `byte`.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = BitIndex::new(4).unwrap();
    /// assert!(idx.get(0b0001_0000));
    /// assert!(!idx.get(0b1110_1111));
    /// ```
    pub fn get(self, byte: u8) -> bool {
        byte & self.to_bit_mask() != 0
    }

    /// Returns `byte` with the bit at this index set to `1`.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = BitIndex::new(1).unwrap();
    /// assert_eq!(idx.set(0b0000_0000), 0b0000_0010);
    /// ```
    pub fn set(self, byte: u8) -> u8 {
        byte | self.to_bit_mask()
    }

    /// Returns `byte` with the bit at this index set to `0`.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = BitIndex::new(2).unwrap();
    /// assert_eq!(idx.clear(0b1111_1111), 0b1111_1011);
    /// ```
    pub fn clear(self, byte: u8) -> u8 {
        byte & !self.to_bit_mask()
    }

    /// Returns `byte` with the bit at this index toggled.
    ///
    /// ```
    /// # use emu::BitIndex;
    /// let idx = BitIndex::new(5).unwrap();
    /// assert_eq!(idx.flip(0b0000_0000), 0b0010_0000);
    /// assert_eq!(idx.flip(0b1111_1111), 0b1101_1111);
    /// ```
    pub fn flip(self, byte: u8) -> u8 {
        byte ^ self.to_bit_mask()
    }
}

/// Converts a `u8` into a `BitIndex`, returning [`BitIndexOutOfRange`] if the value is `>= 8`.
///
/// ```
/// # use emu::{BitIndex, BitIndexOutOfRange};
/// let idx: Result<BitIndex, _> = 5u8.try_into();
/// assert!(idx.is_ok());
///
/// let err: Result<BitIndex, _> = 8u8.try_into();
/// assert_eq!(err, Err(BitIndexOutOfRange(8)));
/// ```
impl TryFrom<u8> for BitIndex {
    type Error = BitIndexOutOfRange;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Converts a `BitIndex` back into its raw `u8` value.
///
/// ```
/// # use emu::BitIndex;
/// let idx = BitIndex::new(6).unwrap();
/// assert_eq!(u8::from(idx), 6u8);
/// ```
impl From<BitIndex> for u8 {
    fn from(value: BitIndex) -> Self {
        value.0
    }
}

/// Error returned when a value `>= 8` is used to construct a [`BitIndex`].
///
/// ```
/// # use emu::{BitIndex, BitIndexOutOfRange};
/// let err = BitIndex::new(9).unwrap_err();
/// assert_eq!(err, BitIndexOutOfRange(9));
/// assert_eq!(err.to_string(), "bit index 9 is out of range (expected 0..=7)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitIndexOutOfRange(pub u8);

impl std::fmt::Display for BitIndexOutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bit index {} is out of range (expected 0..=7)", self.0)
    }
}

impl std::error::Error for BitIndexOutOfRange {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert!(matches!(BitIndex::new(0), Ok(BitIndex(0))));
        assert!(matches!(BitIndex::new(3), Ok(BitIndex(3))));
        assert!(matches!(BitIndex::new(7), Ok(BitIndex(7))));
        assert!(matches!(BitIndex::new(8), Err(BitIndexOutOfRange(8))));
        assert!(matches!(BitIndex::new(10), Err(BitIndexOutOfRange(10))));
        assert!(matches!(BitIndex::new(67), Err(BitIndexOutOfRange(67))));
        assert!(matches!(BitIndex::new(155), Err(BitIndexOutOfRange(155))));
    }

    #[test]
    fn from_low_bits() {
        assert_eq!(BitIndex::from_low_bits(0b1010_0001), BitIndex(1));
        assert_eq!(BitIndex::from_low_bits(0b110), BitIndex(6));
        assert_eq!(BitIndex::from_low_bits(0b1111_1111), BitIndex(7));
    }

    #[test]
    fn try_from_u8() {
        let idx: Result<BitIndex, BitIndexOutOfRange> = 5u8.try_into();
        assert!(matches!(idx, Ok(BitIndex(5))));

        let idx: Result<BitIndex, _> = 2u8.try_into();
        assert!(matches!(idx, Ok(BitIndex(2))));

        let idx: Result<BitIndex, _> = 8u8.try_into();
        assert!(matches!(idx, Err(BitIndexOutOfRange(8))));

        let idx: Result<BitIndex, _> = 15u8.try_into();
        assert!(matches!(idx, Err(BitIndexOutOfRange(15))));
    }

    #[test]
    fn to_bit_mask() {
        let idx = BitIndex::new(4).unwrap();
        assert_eq!(idx.to_bit_mask(), 0b1_0000);

        let idx = BitIndex::new(3).unwrap();
        assert_eq!(idx.to_bit_mask(), 0b1000);

        let idx = BitIndex::new(7).unwrap();
        assert_eq!(idx.to_bit_mask(), 0b1000_0000);

        let idx = BitIndex::new(0).unwrap();
        assert_eq!(idx.to_bit_mask(), 0b1);
    }

    #[test]
    fn get() {
        let idx = BitIndex::new(4).unwrap();
        assert!(idx.get(0b0001_0000));
        assert!(!idx.get(0b1110_1111));

        let idx = BitIndex::new(0).unwrap();
        assert!(idx.get(0b0000_0001));
        assert!(!idx.get(0b1111_1110));

        let idx = BitIndex::new(7).unwrap();
        assert!(idx.get(0b1000_0000));
        assert!(!idx.get(0b0111_1111));
    }

    #[test]
    fn set() {
        let idx = BitIndex::new(1).unwrap();
        assert_eq!(idx.set(0b0000_0000), 0b0000_0010);

        let idx = BitIndex::new(0).unwrap();
        assert_eq!(idx.set(0b0000_0000), 0b0000_0001);

        let idx = BitIndex::new(7).unwrap();
        assert_eq!(idx.set(0b0000_1111), 0b1000_1111);
    }

    #[test]
    fn clear() {
        let idx = BitIndex::new(2).unwrap();
        assert_eq!(idx.clear(0b1111_1111), 0b1111_1011);

        let idx = BitIndex::new(5).unwrap();
        assert_eq!(idx.clear(0b1111_1111), 0b1101_1111);

        let idx = BitIndex::new(6).unwrap();
        assert_eq!(idx.clear(0b1111_0000), 0b1011_0000);
    }

    #[test]
    fn flip() {
        let idx = BitIndex::new(2).unwrap();
        assert_eq!(idx.flip(0b1111_1111), 0b1111_1011);
        assert_eq!(idx.flip(0b0000_0000), 0b0000_0100);

        let idx = BitIndex::new(5).unwrap();
        assert_eq!(idx.flip(0b1111_1111), 0b1101_1111);
        assert_eq!(idx.flip(0b0000_0000), 0b0010_0000);

        let idx = BitIndex::new(6).unwrap();
        assert_eq!(idx.flip(0b1111_0000), 0b1011_0000);
        assert_eq!(idx.flip(0b0000_1111), 0b0100_1111);
    }
}
