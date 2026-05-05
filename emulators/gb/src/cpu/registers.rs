/// The Game Boy CPU's general-purpose 8-bit registers.
///
/// The Sharp SM83 CPU (used in the Game Boy) has eight 8-bit registers: `A`, `F`,
/// `B`, `C`, `D`, `E`, `H`, and `L`. They can be read and written individually,
/// or accessed as four 16-bit register pairs: `AF`, `BC`, `DE`, and `HL`.
///
/// | Register | Role                                                              |
/// |----------|-------------------------------------------------------------------|
/// | `A`      | Accumulator — most ALU operations write their result here         |
/// | `F`      | Flags — bits 7-4 encode Z, N, H, C; bits 3-0 are always 0         |
/// | `B`/`C`  | General purpose / 16-bit pair `BC`                                |
/// | `D`/`E`  | General purpose / 16-bit pair `DE`                                |
/// | `H`/`L`  | General purpose / 16-bit pair `HL` (often used as a pointer)      |
///
/// See [`Flags`] for more information on each individual flag.
///
/// # Pair encoding
///
/// 16-bit register pairs store the high byte in the first register and the low
/// byte in the second:
///
/// ```text
/// BC = (B << 8) | C
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub a: u8,
    pub flags: Flags,
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
    /// # Examples
    ///
    /// ```ignore
    /// let regs = Registers::new();
    /// assert_eq!(regs.a, 0x00);
    /// assert!(!regs.flags.z);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value stored in the 16-bit register pair `BC`.
    ///
    /// `BC` combines the `B` register (high byte) and the `C` register (low
    /// byte).
    ///
    /// For setting the value of `BC` see [`bc_set`](Self::bc_set).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let regs = Registers { b: 0x12, c: 0x34, ..Registers::new() };
    ///
    /// assert_eq!(regs.bc_get(), 0x1234);
    /// ```
    pub const fn bc_get(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    /// Sets the value stored in the 16-bit register pair `BC` to `value`.
    ///
    /// `BC` combines the `B` register (high byte) and the `C` register (low
    /// byte).
    ///
    /// To read the value of `BC` see [`bc_get`](Self::bc_get).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut regs = Registers::new();
    /// regs.bc_set(0x1234);
    ///
    /// assert_eq!(regs.b, 0x12);
    /// assert_eq!(regs.c, 0x34);
    /// ```
    pub const fn bc_set(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();

        self.b = hi;
        self.c = lo;
    }

    /// Returns the value stored in the 16-bit register pair `DE`.
    ///
    /// `DE` combines the `D` register (high byte) and the `E` register (low
    /// byte).
    ///
    /// For setting the value of `DE` see [`de_set`](Self::de_set).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let regs = Registers { d: 0x12, e: 0x34, ..Registers::new() };
    ///
    /// assert_eq!(regs.de_get(), 0x1234);
    /// ```
    pub const fn de_get(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    /// Sets the value stored in the 16-bit register pair `DE` to `value`.
    ///
    /// `DE` combines the `D` register (high byte) and the `E` register (low
    /// byte).
    ///
    /// To read the value of `DE` see [`de_get`](Self::de_get).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut regs = Registers::new();
    /// regs.de_set(0x1234);
    ///
    /// assert_eq!(regs.d, 0x12);
    /// assert_eq!(regs.e, 0x34);
    /// ```
    pub const fn de_set(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();

        self.d = hi;
        self.e = lo;
    }

    /// Returns the value stored in the 16-bit register pair `HL`.
    ///
    /// `HL` combines the `H` register (high byte) and the `L` register (low
    /// byte).
    ///
    /// For setting the value of `HL` see [`hl_set`](Self::hl_set).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let regs = Registers { h: 0x12, l: 0x34, ..Registers::new() };
    ///
    /// assert_eq!(regs.hl_get(), 0x1234);
    /// ```
    pub const fn hl_get(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    /// Sets the value stored in the 16-bit register pair `HL` to `value`.
    ///
    /// `HL` combines the `H` register (high byte) and the `L` register (low
    /// byte).
    ///
    /// To read the value of `HL` see [`hl_get`](Self::hl_get).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut regs = Registers::new();
    /// regs.hl_set(0x1234);
    ///
    /// assert_eq!(regs.h, 0x12);
    /// assert_eq!(regs.l, 0x34);
    /// ```
    pub const fn hl_set(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();

        self.h = hi;
        self.l = lo;
    }

    /// Returns the value stored in the 16-bit register pair `AF`.
    ///
    /// `AF` combines the `A` register (high byte) and the `F` flags register (low
    /// byte).
    ///
    /// Note that the lower 4 bits of the `F` register are always `0`.
    ///
    /// For setting the value of `AF` see [`af_set`](Self::af_set).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let regs = Registers {
    ///     a: 0x12,
    ///     flags: Flags {
    ///         z: false,
    ///         n: true,
    ///         h: true,
    ///         c: false
    ///     },
    ///     ..Registers::new()
    /// };
    ///
    /// assert_eq!(regs.af_get(), 0x1200 | 0b0110_0000);
    /// ```
    pub fn af_get(&self) -> u16 {
        u16::from_be_bytes([self.a, self.flags.into()])
    }

    /// Sets the value stored in the 16-bit register pair `AF` to `value`.
    ///
    /// `AF` combines the `A` register (high byte) and the `F` flags register
    /// (low byte).
    ///
    /// Note that the lower 4 bits of the `F` register are always `0`.
    ///
    /// To read the value of `AF` see [`af_get`](Self::af_get).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut regs = Registers::new();
    /// regs.af_set(0x1200 | 0b1100_1111); // the 4 lower bits are not evaluated for setting `F`
    ///
    /// assert_eq!(regs.a, 0x12);
    /// assert!(regs.flags.z);
    /// assert!(regs.flags.n);
    /// assert!(!regs.flags.h);
    /// assert!(!regs.flags.c);
    /// ```
    pub fn af_set(&mut self, value: u16) {
        let [hi, lo] = value.to_be_bytes();

        self.a = hi;
        self.flags = lo.into();
    }
}

/// CPU Flags Register.
///
/// The flags register uses its upper nibble to store four CPU flags:
///
/// | Bit | Flag        | Symbol | Meaning                              |
/// |-----|-------------|--------|--------------------------------------|
/// | 7   | Zero        | `Z`    | Result was zero                      |
/// | 6   | Subtract    | `N`    | Last instruction was a subtraction   |
/// | 5   | Half-carry  | `H`    | Carry from bit 3 to bit 4            |
/// | 4   | Carry       | `C`    | Carry out of bit 7 (or borrow)       |
///
/// Bits 3–0 of `F` are always `0` and are ignored.
#[allow(clippy::struct_excessive_bools)]
// the `Flags` struct will be changed in the future to use bitfields, so we can ignore this lint for now (issue #8)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// **Zero** Flag.
    ///
    /// The Zero flag is set by the CPU when an ALU operation produces a result
    /// of `0`.
    /// Several conditional branch instructions (eg. JR Z/NZ e8...) test this
    /// flag.
    pub z: bool,

    /// **Subtract** Flag.
    ///
    /// The Subtract flag is set by the CPU when the last operation was a
    /// subtraction. It is used by the DAA instruction to determine the
    /// direction of the correction.
    pub n: bool,

    /// **Half Carry** Flag.
    ///
    /// The Half Carry flag is set by the CPU when a carry or a burrow happens
    /// on the lower nibble (4 bits). It is primarily used by the DAA
    /// instruction.
    pub h: bool,

    /// ***Carry* Falg.
    ///
    /// The carry flag is mostly set when an addition produces carry out of bit
    /// 7 or when a subtraction requires a burrow.
    /// Several conditional branch instructions (eg. JR C/NC e8...) test this
    /// flag.
    pub c: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self::default()
    }
}

const Z_FLAG_OFFSET: u8 = 7;
const N_FLAG_OFFSET: u8 = 6;
const H_FLAG_OFFSET: u8 = 5;
const C_FLAG_OFFSET: u8 = 4;

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        let z = value & (1 << Z_FLAG_OFFSET) != 0;
        let n = value & (1 << N_FLAG_OFFSET) != 0;
        let h = value & (1 << H_FLAG_OFFSET) != 0;
        let c = value & (1 << C_FLAG_OFFSET) != 0;

        Self { z, n, h, c }
    }
}

impl From<Flags> for u8 {
    fn from(value: Flags) -> Self {
        let mut u = 0;

        if value.z {
            u |= 1 << Z_FLAG_OFFSET;
        }

        if value.n {
            u |= 1 << N_FLAG_OFFSET;
        }

        if value.h {
            u |= 1 << H_FLAG_OFFSET;
        }

        if value.c {
            u |= 1 << C_FLAG_OFFSET;
        }

        u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod pair {
        use super::*;

        mod af {
            use super::*;

            #[test]
            fn get() {
                let regs = Registers {
                    a: 0x12,
                    flags: Flags {
                        z: true,
                        n: false,
                        h: false,
                        c: true,
                    },
                    ..Registers::new()
                };

                assert_eq!(regs.af_get(), 0x1200 | 0b1001_0000);
            }

            #[test]
            fn set() {
                let mut regs = Registers::new();

                regs.af_set(0x1200 | 0b0110_1100);

                assert_eq!(
                    regs,
                    Registers {
                        a: 0x12,
                        flags: Flags {
                            z: false,
                            n: true,
                            h: true,
                            c: false
                        },
                        ..Registers::new()
                    }
                );
            }
        }

        mod bc {
            use super::*;

            #[test]
            fn get() {
                let regs = Registers {
                    b: 0x12,
                    c: 0x34,
                    ..Registers::new()
                };

                assert_eq!(regs.bc_get(), 0x1234);
            }

            #[test]
            fn set() {
                let mut regs = Registers::new();

                regs.bc_set(0x1234);

                assert_eq!(
                    regs,
                    Registers {
                        b: 0x12,
                        c: 0x34,
                        ..Registers::new()
                    }
                );
            }
        }

        mod de {
            use super::*;

            #[test]
            fn get() {
                let regs = Registers {
                    d: 0x12,
                    e: 0x34,
                    ..Registers::new()
                };

                assert_eq!(regs.de_get(), 0x1234);
            }

            #[test]
            fn set() {
                let mut regs = Registers::new();

                regs.de_set(0x1234);

                assert_eq!(
                    regs,
                    Registers {
                        d: 0x12,
                        e: 0x34,
                        ..Registers::new()
                    }
                );
            }
        }

        mod hl {
            use super::*;

            #[test]
            fn get() {
                let regs = Registers {
                    h: 0x12,
                    l: 0x34,
                    ..Registers::new()
                };

                assert_eq!(regs.hl_get(), 0x1234);
            }

            #[test]
            fn set() {
                let mut regs = Registers::new();

                regs.hl_set(0x1234);

                assert_eq!(
                    regs,
                    Registers {
                        h: 0x12,
                        l: 0x34,
                        ..Registers::new()
                    }
                );
            }
        }
    }

    mod flags {
        use super::*;

        #[test]
        fn flags_from_u8() {
            let u = 0b1100_1111;
            let flags: Flags = u.into();

            assert!(flags.z);
            assert!(flags.n);
            assert!(!flags.h);
            assert!(!flags.c);

            let u = 0b0011_0000;
            let flags: Flags = u.into();

            assert!(!flags.z);
            assert!(!flags.n);
            assert!(flags.h);
            assert!(flags.c);
        }

        #[test]
        fn u8_from_flags() {
            let flags = Flags {
                z: true,
                n: false,
                h: true,
                c: false,
            };

            let u: u8 = flags.into();

            assert_eq!(u, 0b1010_0000);

            let flags = Flags {
                z: false,
                n: true,
                h: false,
                c: true,
            };

            let u: u8 = flags.into();

            assert_eq!(u, 0b0101_0000);
        }
    }
}
