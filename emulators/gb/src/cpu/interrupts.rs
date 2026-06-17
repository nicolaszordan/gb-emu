use crate::interrupts::Interrupt;

pub const INTERRUPT_DISPATCH_CYCLES: u32 = 20;

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

impl From<Interrupt> for InterruptJumpVector {
    fn from(interrupt: Interrupt) -> Self {
        match interrupt {
            Interrupt::VBlank => VBLANK_JUMP_VECTOR,
            Interrupt::LCDStat => LCD_STAT_JUMP_VECTOR,
            Interrupt::Timer => TIMER_JUMP_VECTOR,
            Interrupt::Serial => SERIAL_JUMP_VECTOR,
            Interrupt::Joypad => JOYPAD_JUMP_VECTOR,
        }
    }
}

/// Interrupt Master Enable (IME) flag.
///
/// The meaning of the IME flag is not to enable or disable interrupts. In fact,
/// what it does is enable the jump to the interrupt vectors.
///
/// IME can only be enabled by the instructions EI and RETI, and can only be
/// disabled by DI (and the CPU when jumping to an interrupt vector).
///
/// Note that EI doesn't enable the interrupts the same cycle it is executed,
/// but the next cycle.
#[allow(clippy::upper_case_acronyms)] // we're suppressing this lint to keep the naming consistent with the pan docs
#[derive(Debug, PartialEq, Eq)]
pub enum IME {
    /// IME flag is reset, and will be set after the next
    /// instruction is executed.
    PendingEnable(u8), // `EI` own step counts as a cycle, so we had to add a counter to track the number of cycles before enabling IME.

    /// IME flag is set.
    Enabled,

    /// IME flag is reset.
    Disabled,
}

impl IME {
    /// Enable the IME flag after the next instruction is executed. Default
    /// behavior of the EI instruction.
    pub const fn enable(&mut self) {
        *self = Self::PendingEnable(1);
    }

    /// Disable the IME flag immediately.
    pub const fn disable(&mut self) {
        *self = Self::Disabled;
    }

    /// Enable the IME flag immediately, without waiting for the next instruction
    /// to be executed.
    pub const fn enable_now(&mut self) {
        *self = Self::Enabled;
    }

    /// Tick the IME flag.
    ///
    /// Transition from `PendingEnable` to `Enabled`. No-op in other states.
    ///
    /// Call once per CPU step, after instruction execution, to honour the one
    /// cycle delay introduced by the EI instruction.
    pub const fn tick(&mut self) {
        if matches!(self, Self::PendingEnable(0)) {
            *self = Self::Enabled;
        } else if let Self::PendingEnable(n) = self {
            *self = Self::PendingEnable(*n - 1);
        }
    }

    /// Checks if the IME is in `Enabled` state.
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// Checks if the IME is either in `Disabled` or `PendingEnable` state.
    pub const fn is_disabled(&self) -> bool {
        !self.is_enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ime {
        use super::*;

        #[test]
        fn ime_enable() {
            let mut ime = IME::Disabled;
            ime.enable();
            assert_eq!(ime, IME::PendingEnable(1));
            ime.tick();
            assert_eq!(ime, IME::PendingEnable(0));
            ime.tick();
            assert_eq!(ime, IME::Enabled);
        }

        #[test]
        fn ime_disable() {
            let mut ime = IME::Enabled;
            ime.disable();
            assert_eq!(ime, IME::Disabled);
        }

        #[test]
        fn ime_enable_now() {
            let mut ime = IME::Disabled;
            ime.enable_now();
            assert_eq!(ime, IME::Enabled);
        }

        #[test]
        fn ime_tick_noop() {
            let mut ime = IME::Disabled;
            ime.tick();
            assert_eq!(ime, IME::Disabled);

            ime.enable_now();
            ime.tick();
            assert_eq!(ime, IME::Enabled);
        }
    }
}
