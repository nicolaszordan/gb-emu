mod flags;
mod registers;

use emu::MemoryBus;

use flags::{InterruptFlags, InterruptJumpVector, InterruptRequest};

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
}

#[derive(Debug)]
pub struct InterruptController {
    ime: IME,
}

impl InterruptController {
    pub const fn new() -> Self {
        Self { ime: IME::Disabled }
    }

    pub const fn step(&mut self) {
        if matches!(self.ime, IME::PendingEnable) {
            self.ime = IME::Enabled;
        }
    }

    pub const fn ime_mut(&mut self) -> &mut IME {
        &mut self.ime
    }

    /// Returns the address of the interrupt vector corresponding to the highest
    /// priority pending interrupt. Or `None` if no interrupts are pending or
    /// if IME is not enabled.
    ///
    pub fn service_pending_interrupt<M: MemoryBus>(
        &mut self,
        bus: &mut M,
    ) -> Option<InterruptJumpVector> {
        if matches!(self.ime, IME::Enabled) {
            let requested_interrupts = registers::flags::read(bus);
            let enabled_interrupts = registers::enable::read(bus);
            let pending_interrupts = requested_interrupts & enabled_interrupts;

            if pending_interrupts.is_empty() {
                None
            } else {
                self.ime.disable(); // disable IME immediately when servicing an interrupt

                // SAFETY: we just checked that pending_interrupts is not empty, so we know that find_highest_priority_interrupt_request will return Some, so unwrap_unchecked is safe here.
                let InterruptRequest { flag, jump_vector } =
                    unsafe { pending_interrupts.try_into().unwrap_unchecked() };

                // Clear the IF flag corresponding to the interrupt being serviced
                registers::flags::write(bus, requested_interrupts & !flag);

                Some(jump_vector)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
impl InterruptController {
    pub const fn ime(&self) -> &IME {
        &self.ime
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {}
