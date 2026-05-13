use bitflags::bitflags;

bitflags! {
    /// Interrupt flags used for both the IE and IF registers. Each bit corresponds to a specific interrupt.
    ///
    /// Bits 7-5 are unused and should always read as 0.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InterruptFlags: u8 {
        const VBLANK   = 0b0000_0001;
        const LCD_STAT = 0b0000_0010;
        const TIMER    = 0b0000_0100;
        const SERIAL   = 0b0000_1000;
        const JOYPAD   = 0b0001_0000;
    }
}

/// Represents a single interrupt that can be serviced by the CPU.
///
/// This struct is used to represent an interrupt to be serviced or requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

/// Abstracts over the Game Boy interrupt controller registers (IE and IF).
///
/// Implementors expose the combined state of the Interrupt Enable (IE) and
/// Interrupt Flag (IF) registers so the CPU can determine which interrupt to
/// service next and mark it as handled.
///
/// Priority follows GB hardware convention: `VBlank` > `LCDStat` > `Timer` > `Serial` > `Joypad`.
pub trait InterruptLine {
    /// Returns the highest priority enabled pending interrupt, if any. Returns `None`
    /// if no enabled interrupts are pending.
    fn pending_interrupt(&self) -> Option<Interrupt>;

    /// Acknowledge an interrupt.
    ///
    /// This will typically involve clearing the corresponding bit in the IF
    /// register to indicate that the interrupt is being serviced.
    fn acknowledge_interrupt(&mut self, interrupt: Interrupt);
}

impl InterruptFlags {
    /// Returns the highest priority [`Interrupt`] set in these flags, or `None` if no interrupt is pending.
    ///
    /// Priority follows GB hardware convention: lower bit position = higher priority
    /// (`VBlank` > `LCDStat` > `Timer` > `Serial` > `Joypad`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let flags = InterruptFlags::VBLANK | InterruptFlags::TIMER;
    /// assert_eq!(flags.highest_priority(), Some(Interrupt::VBlank));
    ///
    /// let flags = InterruptFlags::LCD_STAT | InterruptFlags::SERIAL;
    /// assert_eq!(flags.highest_priority(), Some(Interrupt::LCDStat));
    ///
    /// let flags = InterruptFlags::empty();
    /// assert_eq!(flags.highest_priority(), None);
    /// ```
    pub const fn highest_priority(self) -> Option<Interrupt> {
        match self.bits().trailing_zeros() {
            0 => Some(Interrupt::VBlank),
            1 => Some(Interrupt::LCDStat),
            2 => Some(Interrupt::Timer),
            3 => Some(Interrupt::Serial),
            4 => Some(Interrupt::Joypad),
            _ => None,
        }
    }
}

impl From<Interrupt> for InterruptFlags {
    fn from(interrupt: Interrupt) -> Self {
        match interrupt {
            Interrupt::VBlank => Self::VBLANK,
            Interrupt::LCDStat => Self::LCD_STAT,
            Interrupt::Timer => Self::TIMER,
            Interrupt::Serial => Self::SERIAL,
            Interrupt::Joypad => Self::JOYPAD,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_priority() {
        let flags = InterruptFlags::VBLANK | InterruptFlags::TIMER;
        assert_eq!(flags.highest_priority(), Some(Interrupt::VBlank));

        let flags = InterruptFlags::LCD_STAT | InterruptFlags::SERIAL;
        assert_eq!(flags.highest_priority(), Some(Interrupt::LCDStat));

        let flags = InterruptFlags::empty();
        assert_eq!(flags.highest_priority(), None);
    }
}
