use crate::interrupts::Interrupt;

// #[must_use = "This struct is used to represent an interupt to be serviced, IF bit is already cleared, and the IME flag is already disabled when this struct is created. So if it's not used, the interrupt will be lost and never serviced."]
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
/// but the next cycle
#[allow(clippy::upper_case_acronyms)] // we're suppressing this lint to keep the naming consistent with the pan docs
#[derive(Debug, PartialEq, Eq)]
pub enum IME {
    /// IME flag is reset, and will be set after the next
    /// instruction is executed.
    PendingEnable,

    /// IME flag is set.
    Enabled,

    /// IME flag is reset.
    Disabled,
}

impl IME {
    /// Enable the IME flag after the next instruction is executed. Default
    /// behavior of the EI instruction.
    pub const fn enable(&mut self) {
        *self = Self::PendingEnable;
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

    /// Transition from `PendingEnable` to `Enabled`. No-op in other states.
    ///
    /// Call once per CPU step, after interrupt dispatch, to honour the one-cycle
    /// delay introduced by the EI instruction.
    pub const fn commit_pending(&mut self) {
        if matches!(self, Self::PendingEnable) {
            *self = Self::Enabled;
        }
    }
}

#[cfg(test)]
mod tests {}
