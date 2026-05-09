use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InterruptFlags: u8 {
        const VBLANK   = 0b0000_0001;
        const LCD_STAT = 0b0000_0010;
        const TIMER    = 0b0000_0100;
        const SERIAL   = 0b0000_1000;
        const JOYPAD   = 0b0001_0000;
    }
}

impl InterruptFlags {
    /// Returns the address of the highest priority interrupt vector
    /// corresponding to the set flags, or `None` if no interrupts flags are
    /// set.
    ///
    /// If multiple interrupts are pending, the one with the highest priority is
    /// returned (`VBLANK` > `LCD_STAT` > `TIMER` > `SERIAL` > `JOYPAD`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use emulators::gb::cpu::interrupts::{InterruptFlags, VBLANK_JUMP_VECTOR, SERIAL_JUMP_VECTOR};
    /// let flags = InterruptFlags::VBLANK | InterruptFlags::TIMER;
    /// assert_eq!(flags.try_into_jump_vector(), Some(VBLANK_JUMP_VECTOR)); // VBLANK has higher priority than TIMER
    ///
    /// let flags = InterruptFlags::SERIAL | InterruptFlags::JOYPAD;
    /// assert_eq!(flags.try_into_jump_vector(), Some(SERIAL_JUMP_VECTOR)); // SERIAL has higher priority than JOYPAD
    ///
    /// let flags = InterruptFlags::empty();
    /// assert_eq!(flags.try_into_jump_vector(), None); // no interrupts pending
    /// ```
    #[must_use]
    pub const fn try_into_jump_vector(self) -> Option<InterruptJumpVector> {
        if self.contains(Self::VBLANK) {
            Some(VBLANK_JUMP_VECTOR)
        } else if self.contains(Self::LCD_STAT) {
            Some(LCD_STAT_JUMP_VECTOR)
        } else if self.contains(Self::TIMER) {
            Some(TIMER_JUMP_VECTOR)
        } else if self.contains(Self::SERIAL) {
            Some(SERIAL_JUMP_VECTOR)
        } else if self.contains(Self::JOYPAD) {
            Some(JOYPAD_JUMP_VECTOR)
        } else {
            None
        }
    }
}

#[must_use = "This struct is used to represent an interupt to be serviced, IF bit is already cleared, and the IME flag is already disabled when this struct is created. So if it's not used, the interrupt will be lost and never serviced."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptJumpVector(u16);

impl InterruptJumpVector {
    pub const fn addr(self) -> u16 {
        self.0
    }
}

const VBLANK_JUMP_VECTOR: InterruptJumpVector = InterruptJumpVector(0x40);
const LCD_STAT_JUMP_VECTOR: InterruptJumpVector = InterruptJumpVector(0x48);
const TIMER_JUMP_VECTOR: InterruptJumpVector = InterruptJumpVector(0x50);
const SERIAL_JUMP_VECTOR: InterruptJumpVector = InterruptJumpVector(0x58);
const JOYPAD_JUMP_VECTOR: InterruptJumpVector = InterruptJumpVector(0x60);

pub struct InterruptRequest {
    pub flag: InterruptFlags,
    pub jump_vector: InterruptJumpVector,
}

/// A static array of all possible interrupt requests, ordered by priority (highest priority first).
const INTERRUPT_REQUESTS: [InterruptRequest; 5] = [
    InterruptRequest {
        flag: InterruptFlags::VBLANK,
        jump_vector: VBLANK_JUMP_VECTOR,
    },
    InterruptRequest {
        flag: InterruptFlags::LCD_STAT,
        jump_vector: LCD_STAT_JUMP_VECTOR,
    },
    InterruptRequest {
        flag: InterruptFlags::TIMER,
        jump_vector: TIMER_JUMP_VECTOR,
    },
    InterruptRequest {
        flag: InterruptFlags::SERIAL,
        jump_vector: SERIAL_JUMP_VECTOR,
    },
    InterruptRequest {
        flag: InterruptFlags::JOYPAD,
        jump_vector: JOYPAD_JUMP_VECTOR,
    },
];

impl TryFrom<InterruptFlags> for InterruptRequest {
    type Error = ();

    fn try_from(value: InterruptFlags) -> Result<Self, Self::Error> {
        for request in INTERRUPT_REQUESTS {
            if value.contains(request.flag) {
                return Ok(request);
            }
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod try_into_jump_vector {
        use super::*;

        #[test]
        fn vblank() {
            assert_eq!(
                InterruptFlags::VBLANK.try_into_jump_vector(),
                Some(VBLANK_JUMP_VECTOR)
            );

            assert_eq!(
                (InterruptFlags::VBLANK | InterruptFlags::LCD_STAT).try_into_jump_vector(),
                Some(VBLANK_JUMP_VECTOR)
            ); // VBLANK has higher priority than LCD_STAT

            assert_eq!(
                (InterruptFlags::VBLANK | InterruptFlags::TIMER).try_into_jump_vector(),
                Some(VBLANK_JUMP_VECTOR)
            ); // VBLANK has higher priority than TIMER
        }

        #[test]
        fn lcd_stat() {
            assert_eq!(
                InterruptFlags::LCD_STAT.try_into_jump_vector(),
                Some(LCD_STAT_JUMP_VECTOR)
            );

            assert_eq!(
                (InterruptFlags::LCD_STAT | InterruptFlags::TIMER).try_into_jump_vector(),
                Some(LCD_STAT_JUMP_VECTOR)
            ); // VBLANK has higher priority than LCD_STAT

            assert_eq!(
                (InterruptFlags::LCD_STAT | InterruptFlags::SERIAL).try_into_jump_vector(),
                Some(LCD_STAT_JUMP_VECTOR)
            ); // LCD_STAT has higher priority than SERIAL
        }

        #[test]
        fn timer() {
            assert_eq!(
                InterruptFlags::TIMER.try_into_jump_vector(),
                Some(TIMER_JUMP_VECTOR)
            );

            assert_eq!(
                (InterruptFlags::TIMER | InterruptFlags::SERIAL).try_into_jump_vector(),
                Some(TIMER_JUMP_VECTOR)
            ); // LCD_STAT has higher priority than TIMER

            assert_eq!(
                (InterruptFlags::TIMER | InterruptFlags::JOYPAD).try_into_jump_vector(),
                Some(TIMER_JUMP_VECTOR)
            ); // TIMER has higher priority than SERIAL
        }

        #[test]
        fn serial() {
            assert_eq!(
                InterruptFlags::SERIAL.try_into_jump_vector(),
                Some(SERIAL_JUMP_VECTOR)
            );

            assert_eq!(
                (InterruptFlags::SERIAL | InterruptFlags::JOYPAD).try_into_jump_vector(),
                Some(SERIAL_JUMP_VECTOR)
            ); // SERIAL has higher priority than JOYPAD

            assert_eq!(
                (InterruptFlags::SERIAL | InterruptFlags::VBLANK).try_into_jump_vector(),
                Some(VBLANK_JUMP_VECTOR)
            ); // VBLANK has higher priority than SERIAL
        }

        #[test]
        fn joypad() {
            assert_eq!(
                InterruptFlags::JOYPAD.try_into_jump_vector(),
                Some(JOYPAD_JUMP_VECTOR)
            );

            assert_eq!(
                (InterruptFlags::JOYPAD | InterruptFlags::VBLANK).try_into_jump_vector(),
                Some(VBLANK_JUMP_VECTOR)
            ); // VBLANK has higher priority than JOYPAD

            assert_eq!(
                (InterruptFlags::JOYPAD | InterruptFlags::LCD_STAT).try_into_jump_vector(),
                Some(LCD_STAT_JUMP_VECTOR)
            ); // LCD_STAT has higher priority than JOYPAD
        }

        #[test]
        fn none() {
            assert_eq!(InterruptFlags::empty().try_into_jump_vector(), None);

            assert_eq!(
                InterruptFlags::from_bits_retain(0b1110_0000).try_into_jump_vector(), // no valid flags set, should return None
                None
            );
        }
    }
}
