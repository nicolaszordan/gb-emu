/// The Game Boy CPU's general-purpose 8-bit registers.
///
/// The Sharp SM83 CPU (used in the Game Boy) has eight 8-bit registers: `A`, `F`,
/// `B`, `C`, `D`, `E`, `H`, and `L`. They can be read and written individually, or
/// accessed as four 16-bit register pairs: `AF`, `BC`, `DE`, and `HL`.
///
/// | Register | Role                                                              |
/// |----------|-------------------------------------------------------------------|
/// | `A`      | Accumulator — most ALU operations write their result here        |
/// | `F`      | Flags — bits 7-4 encode Z, N, H, C; bits 3-0 are always 0       |
/// | `B`/`C`  | General purpose / 16-bit pair `BC`                               |
/// | `D`/`E`  | General purpose / 16-bit pair `DE`                               |
/// | `H`/`L`  | General purpose / 16-bit pair `HL` (often used as a pointer)    |
///
/// # Pair encoding
///
/// 16-bit register pairs store the high byte in the first register and the low
/// byte in the second (big-endian within the pair):
///
/// ```text
/// BC = (B << 8) | C
/// ```
#[derive(Debug, Default)]
pub(crate) struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    /// Creates a new [`Registers`] with all registers initialised to `0`.
    ///
    /// > **Note:** Real Game Boy hardware initialises registers to specific
    /// > post-boot-ROM values (e.g. `A = 0x01`, `F = 0xB0` for the DMG). This
    /// > constructor starts from a clean slate; any boot-ROM simulation should
    /// > set the appropriate values afterwards.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let regs = Registers::new();
    /// assert_eq!(regs.a, 0x00);
    /// assert_eq!(regs.f, 0x00);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a read-only 16-bit view of the `AF` register pair.
    ///
    /// `AF` combines the accumulator (`A`, high byte) with the flags register
    /// (`F`, low byte). Use [`RegisterPairView::get`] to read the combined 16-bit
    /// value.
    ///
    /// For a mutable view see [`af_mut`](Self::af_mut).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let regs = Registers { a: 0x12, f: 0x34, ..Registers::new() };
    /// assert_eq!(regs.af().get(), 0x1234);
    /// ```
    pub fn af(&self) -> RegisterPairView<'_> {
        RegisterPairView {
            high: &self.a,
            low: &self.f,
        }
    }

    /// Returns a mutable 16-bit view of the `AF` register pair.
    ///
    /// `AF` combines the accumulator (`A`, high byte) with the flags register
    /// (`F`, low byte). Use [`RegisterPairViewMut::set`] to write a 16-bit value;
    /// the high byte is stored in `A` and the low byte in `F`.
    ///
    /// For a read-only view see [`af`](Self::af).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    /// regs.af_mut().set(0x1234);
    /// assert_eq!(regs.a, 0x12);
    /// assert_eq!(regs.f, 0x34);
    /// ```
    pub fn af_mut(&mut self) -> RegisterPairViewMut<'_> {
        RegisterPairViewMut {
            high: &mut self.a,
            low: &mut self.f,
        }
    }

    /// Returns a read-only 16-bit view of the `BC` register pair.
    ///
    /// `BC` combines `B` (high byte) and `C` (low byte). Use
    /// [`RegisterPairView::get`] to read the combined 16-bit value.
    ///
    /// For a mutable view see [`bc_mut`](Self::bc_mut).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let regs = Registers { b: 0x56, c: 0x78, ..Registers::new() };
    /// assert_eq!(regs.bc().get(), 0x5678);
    /// ```
    pub fn bc(&self) -> RegisterPairView<'_> {
        RegisterPairView {
            high: &self.b,
            low: &self.c,
        }
    }

    /// Returns a mutable 16-bit view of the `BC` register pair.
    ///
    /// `BC` combines `B` (high byte) and `C` (low byte). Use
    /// [`RegisterPairViewMut::set`] to write a 16-bit value; the high byte is
    /// stored in `B` and the low byte in `C`.
    ///
    /// For a read-only view see [`bc`](Self::bc).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    /// regs.bc_mut().set(0x5678);
    /// assert_eq!(regs.b, 0x56);
    /// assert_eq!(regs.c, 0x78);
    /// ```
    pub fn bc_mut(&mut self) -> RegisterPairViewMut<'_> {
        RegisterPairViewMut {
            high: &mut self.b,
            low: &mut self.c,
        }
    }

    /// Returns a read-only 16-bit view of the `DE` register pair.
    ///
    /// `DE` combines `D` (high byte) and `E` (low byte). Use
    /// [`RegisterPairView::get`] to read the combined 16-bit value.
    ///
    /// For a mutable view see [`de_mut`](Self::de_mut).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let regs = Registers { d: 0x9A, e: 0xBC, ..Registers::new() };
    /// assert_eq!(regs.de().get(), 0x9ABC);
    /// ```
    pub fn de(&self) -> RegisterPairView<'_> {
        RegisterPairView {
            high: &self.d,
            low: &self.e,
        }
    }

    /// Returns a mutable 16-bit view of the `DE` register pair.
    ///
    /// `DE` combines `D` (high byte) and `E` (low byte). Use
    /// [`RegisterPairViewMut::set`] to write a 16-bit value; the high byte is
    /// stored in `D` and the low byte in `E`.
    ///
    /// For a read-only view see [`de`](Self::de).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    /// regs.de_mut().set(0x9ABC);
    /// assert_eq!(regs.d, 0x9A);
    /// assert_eq!(regs.e, 0xBC);
    /// ```
    pub fn de_mut(&mut self) -> RegisterPairViewMut<'_> {
        RegisterPairViewMut {
            high: &mut self.d,
            low: &mut self.e,
        }
    }

    /// Returns a read-only 16-bit view of the `HL` register pair.
    ///
    /// `HL` combines `H` (high byte) and `L` (low byte). Use
    /// [`RegisterPairView::get`] to read the combined 16-bit value.
    ///
    /// `HL` is frequently used as a 16-bit memory pointer by instructions such
    /// as `LD (HL), r` and `LD r, (HL)`.
    ///
    /// For a mutable view see [`hl_mut`](Self::hl_mut).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let regs = Registers { h: 0xDE, l: 0xF0, ..Registers::new() };
    /// assert_eq!(regs.hl().get(), 0xDEF0);
    /// ```
    pub fn hl(&self) -> RegisterPairView<'_> {
        RegisterPairView {
            high: &self.h,
            low: &self.l,
        }
    }

    /// Returns a mutable 16-bit view of the `HL` register pair.
    ///
    /// `HL` combines `H` (high byte) and `L` (low byte). Use
    /// [`RegisterPairViewMut::set`] to write a 16-bit value; the high byte is
    /// stored in `H` and the low byte in `L`.
    ///
    /// For a read-only view see [`hl`](Self::hl).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    /// regs.hl_mut().set(0xDEF0);
    /// assert_eq!(regs.h, 0xDE);
    /// assert_eq!(regs.l, 0xF0);
    /// ```
    pub fn hl_mut(&mut self) -> RegisterPairViewMut<'_> {
        RegisterPairViewMut {
            high: &mut self.h,
            low: &mut self.l,
        }
    }

    /// Returns a read-only view of the CPU flags encoded in the `F` register.
    ///
    /// The `F` register uses its upper nibble to store four CPU flags:
    ///
    /// | Bit | Flag        | Symbol | Meaning                              |
    /// |-----|-------------|--------|--------------------------------------|
    /// | 7   | Zero        | `Z`    | Result was zero                      |
    /// | 6   | Subtract    | `N`    | Last instruction was a subtraction   |
    /// | 5   | Half-carry  | `H`    | Carry from bit 3 to bit 4            |
    /// | 4   | Carry       | `C`    | Carry out of bit 7 (or borrow)       |
    ///
    /// Bits 3–0 of `F` are always `0` and are ignored.
    ///
    /// See [`FlagsRegisterView`] for the available flag accessors.
    ///
    /// For a mutable view see [`flags_mut`](Self::flags_mut).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // F = 0b1010_0000 → Z=1, N=0, H=1, C=0
    /// let regs = Registers { f: 0b1010_0000, ..Registers::new() };
    /// assert!(regs.flags().zero());
    /// assert!(!regs.flags().subtract());
    /// assert!(regs.flags().half_carry());
    /// assert!(!regs.flags().carry());
    /// ```
    pub fn flags(&self) -> FlagsRegisterView<'_> {
        FlagsRegisterView {
            flags_register: &self.f,
        }
    }

    /// Returns a mutable view of the CPU flags encoded in the `F` register.
    ///
    /// The `F` register uses its upper nibble to store four CPU flags:
    ///
    /// | Bit | Flag        | Symbol | Meaning                              |
    /// |-----|-------------|--------|--------------------------------------|
    /// | 7   | Zero        | `Z`    | Result was zero                      |
    /// | 6   | Subtract    | `N`    | Last instruction was a subtraction   |
    /// | 5   | Half-carry  | `H`    | Carry from bit 3 to bit 4            |
    /// | 4   | Carry       | `C`    | Carry out of bit 7 (or borrow)       |
    ///
    /// Each flag accessor on [`FlagsRegisterViewMut`] returns a
    /// [`FlagRegisterViewMut`] whose [`set`](FlagRegisterViewMut::set) method
    /// updates only that flag's bit, leaving all others unchanged.
    ///
    /// For a read-only view see [`flags`](Self::flags).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    /// regs.flags_mut().zero().set(true);      // bit 7 → 1
    /// regs.flags_mut().subtract().set(false); // bit 6 → 0
    /// regs.flags_mut().half_carry().set(true); // bit 5 → 1
    /// regs.flags_mut().carry().set(false);    // bit 4 → 0
    /// assert_eq!(regs.f, 0b1010_0000);
    /// ```
    pub fn flags_mut(&mut self) -> FlagsRegisterViewMut<'_> {
        FlagsRegisterViewMut {
            flags_register: &mut self.f,
        }
    }
}

/// A read-only 16-bit view over a pair of 8-bit registers.
///
/// The pair is encoded as `(high << 8) | low`, matching the Game Boy's
/// big-endian register-pair convention. The view holds shared references to the
/// two underlying bytes, so it cannot outlive the [`Registers`] that created it.
///
/// This struct is created by [`Registers::af`], [`Registers::bc`],
/// [`Registers::de`], and [`Registers::hl`].
pub(crate) struct RegisterPairView<'a> {
    high: &'a u8,
    low: &'a u8,
}

impl<'a> RegisterPairView<'a> {
    /// Reads the 16-bit value of the register pair.
    ///
    /// The high-byte register occupies bits 15–8 and the low-byte register
    /// occupies bits 7–0.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // BC = (B << 8) | C = (0x56 << 8) | 0x78 = 0x5678
    /// let regs = Registers { b: 0x56, c: 0x78, ..Registers::new() };
    /// assert_eq!(regs.bc().get(), 0x5678);
    ///
    /// // A zero-initialised pair reads as 0x0000
    /// let regs = Registers::new();
    /// assert_eq!(regs.hl().get(), 0x0000);
    /// ```
    pub fn get(&self) -> u16 {
        ((*self.high as u16) << 8) | (*self.low as u16)
    }
}

/// A mutable 16-bit view over a pair of 8-bit registers.
///
/// Writes split the 16-bit value across the two underlying bytes, storing the
/// high byte in the first register and the low byte in the second. The view
/// holds exclusive references to the two bytes, so it cannot outlive the
/// [`Registers`] that created it.
///
/// This struct is created by [`Registers::af_mut`], [`Registers::bc_mut`],
/// [`Registers::de_mut`], and [`Registers::hl_mut`].
pub(crate) struct RegisterPairViewMut<'a> {
    high: &'a mut u8,
    low: &'a mut u8,
}

impl<'a> RegisterPairViewMut<'a> {
    /// Writes a 16-bit value into the register pair.
    ///
    /// Bits 15–8 of `value` are stored in the high-byte register, and
    /// bits 7–0 are stored in the low-byte register.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    ///
    /// // Write 0xDEF0 into HL → H=0xDE, L=0xF0
    /// regs.hl_mut().set(0xDEF0);
    /// assert_eq!(regs.h, 0xDE);
    /// assert_eq!(regs.l, 0xF0);
    ///
    /// // Overwriting with 0x0000 clears both bytes
    /// regs.hl_mut().set(0x0000);
    /// assert_eq!(regs.h, 0x00);
    /// assert_eq!(regs.l, 0x00);
    ///
    /// // Only the low byte changes when the high byte is 0x00
    /// regs.bc_mut().set(0x00FF);
    /// assert_eq!(regs.b, 0x00);
    /// assert_eq!(regs.c, 0xFF);
    /// ```
    pub fn set(&mut self, value: u16) {
        *self.high = (value >> 8) as u8;
        *self.low = (value & 0xFF) as u8;
    }
}

const ZERO_FLAG_OFFSET: u8 = 7;
const SUBTRACT_FLAG_OFFSET: u8 = 6;
const HALF_CARRY_FLAG_OFFSET: u8 = 5;
const CARRY_FLAG_OFFSET: u8 = 4;

/// A read-only view into the F register that allows access to its individual flag bits.
///
/// The `F` register encodes four CPU condition flags in its upper nibble:
///
/// ```text
/// Bit:  7   6   5   4   3   2   1   0
///       Z   N   H   C   0   0   0   0
/// ```
///
/// Bits 3–0 are always `0`. Each accessor consumes `self` and returns a single
/// `bool`, so a fresh view must be obtained from [`Registers::flags`] for each
/// flag you want to inspect.
///
/// This `struct` is created by the [`Registers::flags`] method.
pub(crate) struct FlagsRegisterView<'a> {
    flags_register: &'a u8,
}

impl<'a> FlagsRegisterView<'a> {
    /// Returns `true` if the **Zero** flag (`Z`, bit 7) is set.
    ///
    /// The Zero flag is set by the CPU when an arithmetic or logical operation
    /// produces a result of `0`. Many conditional branch instructions (e.g.
    /// `JR Z, r8`) test this flag.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // F = 0b1000_0000 → Z=1, N=0, H=0, C=0
    /// let regs = Registers { f: 0b1000_0000, ..Registers::new() };
    /// assert!(regs.flags().zero());
    ///
    /// // F = 0b0111_0000 → Z=0
    /// let regs = Registers { f: 0b0111_0000, ..Registers::new() };
    /// assert!(!regs.flags().zero());
    /// ```
    pub fn zero(self) -> bool {
        *self.flags_register & (1 << ZERO_FLAG_OFFSET) != 0
    }

    /// Returns `true` if the **Subtract** flag (`N`, bit 6) is set.
    ///
    /// The Subtract flag is set when the last operation was a subtraction. It
    /// is used by the DAA (Decimal Adjust Accumulator) instruction to determine
    /// the direction of the correction.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // F = 0b0100_0000 → Z=0, N=1, H=0, C=0
    /// let regs = Registers { f: 0b0100_0000, ..Registers::new() };
    /// assert!(regs.flags().subtract());
    ///
    /// // F = 0b1011_0000 → N=0
    /// let regs = Registers { f: 0b1011_0000, ..Registers::new() };
    /// assert!(!regs.flags().subtract());
    /// ```
    pub fn subtract(self) -> bool {
        *self.flags_register & (1 << SUBTRACT_FLAG_OFFSET) != 0
    }

    /// Returns `true` if the **Half-carry** flag (`H`, bit 5) is set.
    ///
    /// The Half-carry flag is set when there is a carry out of bit 3 into bit 4
    /// (for additions) or a borrow from bit 4 into bit 3 (for subtractions). It
    /// is primarily used by the DAA instruction.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // F = 0b0010_0000 → Z=0, N=0, H=1, C=0
    /// let regs = Registers { f: 0b0010_0000, ..Registers::new() };
    /// assert!(regs.flags().half_carry());
    ///
    /// // F = 0b1101_0000 → H=0
    /// let regs = Registers { f: 0b1101_0000, ..Registers::new() };
    /// assert!(!regs.flags().half_carry());
    /// ```
    pub fn half_carry(self) -> bool {
        *self.flags_register & (1 << HALF_CARRY_FLAG_OFFSET) != 0
    }

    /// Returns `true` if the **Carry** flag (`C`, bit 4) is set.
    ///
    /// The Carry flag is set when an addition produces a carry out of bit 7, or
    /// when a subtraction requires a borrow. It is also affected by rotate and
    /// shift instructions, and is tested by conditional branches such as
    /// `JR C, r8`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // F = 0b0001_0000 → Z=0, N=0, H=0, C=1
    /// let regs = Registers { f: 0b0001_0000, ..Registers::new() };
    /// assert!(regs.flags().carry());
    ///
    /// // F = 0b1110_0000 → C=0
    /// let regs = Registers { f: 0b1110_0000, ..Registers::new() };
    /// assert!(!regs.flags().carry());
    /// ```
    pub fn carry(self) -> bool {
        *self.flags_register & (1 << CARRY_FLAG_OFFSET) != 0
    }
}

/// A mutable view into the F register that allows mutable access to its individual flag bits.
///
/// Each accessor method consumes `self` and returns a [`FlagRegisterViewMut`]
/// targeting that flag's bit. Call [`FlagRegisterViewMut::set`] on the returned
/// value to update only that flag without disturbing the others.
///
/// Because each accessor consumes `self`, you must obtain a fresh view from
/// [`Registers::flags_mut`] for each flag you want to modify.
///
/// This `struct` is created by the [`Registers::flags_mut`] method.
pub(crate) struct FlagsRegisterViewMut<'a> {
    flags_register: &'a mut u8,
}

impl<'a> FlagsRegisterViewMut<'a> {
    /// Returns a mutable handle targeting the **Zero** flag (`Z`, bit 7) of the F register.
    ///
    /// Call [`FlagRegisterViewMut::set`] on the returned value to set or clear
    /// the Zero flag without affecting any other flags.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    ///
    /// regs.flags_mut().zero().set(true);
    /// assert!(regs.flags().zero());
    /// assert_eq!(regs.f & 0b1000_0000, 0b1000_0000); // only bit 7 changed
    ///
    /// regs.flags_mut().zero().set(false);
    /// assert!(!regs.flags().zero());
    /// ```
    pub fn zero(self) -> FlagRegisterViewMut<'a> {
        FlagRegisterViewMut {
            flags_register: self.flags_register,
            offset: ZERO_FLAG_OFFSET,
        }
    }

    /// Returns a mutable handle targeting the **Subtract** flag (`N`, bit 6) of the F register.
    ///
    /// Call [`FlagRegisterViewMut::set`] on the returned value to set or clear
    /// the Subtract flag without affecting any other flags.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    ///
    /// regs.flags_mut().subtract().set(true);
    /// assert!(regs.flags().subtract());
    /// assert_eq!(regs.f & 0b0100_0000, 0b0100_0000); // only bit 6 changed
    ///
    /// regs.flags_mut().subtract().set(false);
    /// assert!(!regs.flags().subtract());
    /// ```
    pub fn subtract(self) -> FlagRegisterViewMut<'a> {
        FlagRegisterViewMut {
            flags_register: self.flags_register,
            offset: SUBTRACT_FLAG_OFFSET,
        }
    }

    /// Returns a mutable handle targeting the **Half-carry** flag (`H`, bit 5) of the F register.
    ///
    /// Call [`FlagRegisterViewMut::set`] on the returned value to set or clear
    /// the Half-carry flag without affecting any other flags.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    ///
    /// regs.flags_mut().half_carry().set(true);
    /// assert!(regs.flags().half_carry());
    /// assert_eq!(regs.f & 0b0010_0000, 0b0010_0000); // only bit 5 changed
    ///
    /// regs.flags_mut().half_carry().set(false);
    /// assert!(!regs.flags().half_carry());
    /// ```
    pub fn half_carry(self) -> FlagRegisterViewMut<'a> {
        FlagRegisterViewMut {
            flags_register: self.flags_register,
            offset: HALF_CARRY_FLAG_OFFSET,
        }
    }

    /// Returns a mutable handle targeting the **Carry** flag (`C`, bit 4) of the F register.
    ///
    /// Call [`FlagRegisterViewMut::set`] on the returned value to set or clear
    /// the Carry flag without affecting any other flags.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new();
    ///
    /// regs.flags_mut().carry().set(true);
    /// assert!(regs.flags().carry());
    /// assert_eq!(regs.f & 0b0001_0000, 0b0001_0000); // only bit 4 changed
    ///
    /// regs.flags_mut().carry().set(false);
    /// assert!(!regs.flags().carry());
    /// ```
    pub fn carry(self) -> FlagRegisterViewMut<'a> {
        FlagRegisterViewMut {
            flags_register: self.flags_register,
            offset: CARRY_FLAG_OFFSET,
        }
    }
}

/// A mutable view into a single flag bit of the F register.
///
/// Obtained by calling one of the flag-accessor methods on [`FlagsRegisterViewMut`]
/// (e.g. [`FlagsRegisterViewMut::zero`]). Holds an exclusive borrow of the `F`
/// register byte and a bit offset identifying which flag to operate on.
///
/// This `struct` is created by methods on [`FlagsRegisterViewMut`].
pub(crate) struct FlagRegisterViewMut<'a> {
    flags_register: &'a mut u8,
    offset: u8,
}

impl<'a> FlagRegisterViewMut<'a> {
    /// Sets or clears the flag bit this view was created for.
    ///
    /// When `value` is `true` the bit at `offset` is set using a bitwise OR;
    /// when `false` it is cleared using a bitwise AND with the complement. All
    /// other bits in the `F` register are left unchanged.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut regs = Registers::new(); // F = 0b0000_0000
    ///
    /// // Setting individual flags
    /// regs.flags_mut().zero().set(true);       // F = 0b1000_0000
    /// regs.flags_mut().half_carry().set(true); // F = 0b1010_0000
    /// regs.flags_mut().carry().set(true);      // F = 0b1011_0000
    /// assert_eq!(regs.f, 0b1011_0000);
    ///
    /// // Clearing a flag does not touch the others
    /// regs.flags_mut().zero().set(false);      // F = 0b0011_0000
    /// assert_eq!(regs.f, 0b0011_0000);
    ///
    /// // Setting an already-set flag is idempotent
    /// regs.flags_mut().carry().set(true);
    /// assert_eq!(regs.f, 0b0011_0000);
    ///
    /// // Clearing an already-clear flag is also idempotent
    /// regs.flags_mut().subtract().set(false);
    /// assert_eq!(regs.f, 0b0011_0000);
    /// ```
    pub fn set(&mut self, value: bool) {
        if value {
            *self.flags_register |= 1 << self.offset;
        } else {
            *self.flags_register &= !(1 << self.offset);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_pair_view_get() {
        let registers = Registers {
            a: 0x12,
            f: 0x34,
            b: 0x56,
            c: 0x78,
            d: 0x9A,
            e: 0xBC,
            h: 0xDE,
            l: 0xF0,
        };

        assert_eq!(registers.af().get(), 0x1234);
        assert_eq!(registers.bc().get(), 0x5678);
        assert_eq!(registers.de().get(), 0x9ABC);
        assert_eq!(registers.hl().get(), 0xDEF0);
    }

    #[test]
    fn register_pair_view_mut_set() {
        let mut registers = Registers::new();

        registers.af_mut().set(0x1234);
        registers.bc_mut().set(0x5678);
        registers.de_mut().set(0x9ABC);
        registers.hl_mut().set(0xDEF0);

        assert_eq!(registers.a, 0x12);
        assert_eq!(registers.f, 0x34);
        assert_eq!(registers.b, 0x56);
        assert_eq!(registers.c, 0x78);
        assert_eq!(registers.d, 0x9A);
        assert_eq!(registers.e, 0xBC);
        assert_eq!(registers.h, 0xDE);
        assert_eq!(registers.l, 0xF0);
    }

    #[test]
    fn flag_register_view_get() {
        let registers = Registers {
            f: 0b1010_0000,
            ..Registers::new()
        };

        assert!(registers.flags().zero());
        assert!(!registers.flags().subtract());
        assert!(registers.flags().half_carry());
        assert!(!registers.flags().carry());
    }

    #[test]
    fn flag_register_view_mut_set() {
        let mut registers = Registers::new();

        registers.flags_mut().zero().set(true);
        registers.flags_mut().subtract().set(false);
        registers.flags_mut().half_carry().set(true);
        registers.flags_mut().carry().set(false);

        assert_eq!(registers.f, 0b1010_0000);

        registers.flags_mut().zero().set(false);
        registers.flags_mut().subtract().set(true);
        registers.flags_mut().half_carry().set(false);
        registers.flags_mut().carry().set(true);

        assert_eq!(registers.f, 0b0101_0000);
    }
}
