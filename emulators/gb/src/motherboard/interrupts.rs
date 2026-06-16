use crate::interrupts::{Interrupt, InterruptBus, InterruptFlags};
use emu::MemoryBus;

/// Interrupt Registers, which include the Interrupt Enable (IE) and Interrupt Flags (IF) registers.
///
/// The IE register determines which interrupts are allowed to trigger an interrupt, while the IF
/// register indicates which interrupts are currently pending. The CPU will check the IF register
/// against the IE register to determine which interrupts should be serviced.
#[derive(Debug)]
pub struct InterruptRegisters {
    /// Interrupt Enable (IE) register at 0xFFFF. Each bit corresponds to a
    /// different interrupt source, and determines whether that source is
    /// allowed to trigger an interrupt.
    interrupt_enable: InterruptFlags,

    /// Interrupt Flags (IF) register at 0xFF0F. Each bit corresponds to a
    /// different interrupt source, and is set when that source triggers an
    /// interrupt. The CPU will check this register against the IE register to
    /// determine which interrupts are pending and should be serviced.
    interrupt_flags: InterruptFlags,
}

impl InterruptRegisters {
    /// Creates a new `InterruptRegisters` instance with both the IE and IF registers
    /// initialized to 0 (no interrupts enabled and no interrupts pending).
    pub const fn new() -> Self {
        Self {
            interrupt_enable: InterruptFlags::empty(),
            interrupt_flags: InterruptFlags::empty(),
        }
    }

    /// Request an interrupt by setting the corresponding bit in the IF register.
    ///
    /// This method sets the corresponding bit in the IF register to indicate that
    /// the specified interrupt is pending. The CPU will check this register against the IE
    /// register to determine which interrupts are pending and should be serviced.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut interrupt_registers = InterruptRegisters::new();
    ///
    /// // Request a VBlank interrupt
    /// interrupt_registers.request_interrupt(Interrupt::VBlank);
    /// assert_eq!(interrupt_registers.interrupt_flags, InterruptFlags::VBLANK);
    ///
    /// assert_eq!(interrupt_registers.pending_interrupt(), None); // no interrupts enabled, so pending_interrupt returns None
    ///
    /// interrupt_registers.enable_interrupt(Interrupt::VBlank); // enable the VBlank interrupt (note that this function is only available for testing)
    ///
    /// assert_eq!(interrupt_registers.pending_interrupt(), Some(Interrupt::VBlank)); // now pending_interrupt returns the VBlank interrupt, since it is enabled and pending
    /// ```
    #[allow(dead_code)] // this method is not used yet, but will be when we implement other components that trigger interrupts
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_flags |= InterruptFlags::from(interrupt);
    }
}

impl InterruptBus for InterruptRegisters {
    /// Returns the currently requested interrupts (IF register).
    fn requested_interrupts(&self) -> InterruptFlags {
        self.interrupt_flags
    }

    /// Returns the currently enabled interrupts (IE register).
    fn enabled_interrupts(&self) -> InterruptFlags {
        self.interrupt_enable
    }

    /// Acknowledge an interrupt and mark it as serviced.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut interrupt_registers = InterruptRegisters::new();
    ///
    /// // Request and enable a Timer interrupt
    /// interrupt_registers.request_interrupt(Interrupt::Timer);
    /// interrupt_registers.enable_interrupt(Interrupt::Timer);
    ///
    /// assert_eq!(interrupt_registers.pending_interrupt(), Some(Interrupt::Timer)); // Timer interrupt is pending
    ///
    /// // Acknowledge the Timer interrupt
    /// interrupt_registers.acknowledge_interrupt(Interrupt::Timer);
    ///
    /// assert_eq!(interrupt_registers.pending_interrupt(), None); // no interrupts are pending after acknowledging
    /// ```
    fn acknowledge_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_flags &= !InterruptFlags::from(interrupt);
    }
}

pub(super) const IF_ADDRESS: u16 = 0xFF0F;

pub(super) const IE_ADDRESS: u16 = 0xFFFF;

impl MemoryBus for InterruptRegisters {
    fn read(&self, address: u16) -> u8 {
        match address {
            IF_ADDRESS => self.interrupt_flags.bits(),
            IE_ADDRESS => self.interrupt_enable.bits(),
            _ => unreachable!("mb dispatch error: invalid interrupt register address"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            IF_ADDRESS => self.interrupt_flags = InterruptFlags::from_bits_truncate(value),
            IE_ADDRESS => self.interrupt_enable = InterruptFlags::from_bits_truncate(value),
            _ => unreachable!("mb dispatch error: invalid interrupt register address"),
        }
    }
}

#[cfg(test)]
impl InterruptRegisters {
    /// Manually enable an interrupt in the IE register for testing purposes.
    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_enable |= InterruptFlags::from(interrupt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_interrupt_registers() {
        let interrupt_registers = InterruptRegisters::new();
        assert_eq!(
            interrupt_registers.interrupt_enable,
            InterruptFlags::empty()
        );
        assert_eq!(interrupt_registers.interrupt_flags, InterruptFlags::empty());
    }

    #[test]
    fn enable_interrupt() {
        let mut interrupt_registers = InterruptRegisters::new();

        interrupt_registers.enable_interrupt(Interrupt::Timer);
        assert_eq!(interrupt_registers.interrupt_enable, InterruptFlags::TIMER);

        interrupt_registers.enable_interrupt(Interrupt::VBlank);
        assert_eq!(
            interrupt_registers.interrupt_enable,
            InterruptFlags::TIMER | InterruptFlags::VBLANK
        ); // both Timer and VBlank interrupts are now enabled
    }

    #[test]
    fn request_interrupt() {
        let mut interrupt_registers = InterruptRegisters::new();

        // Request a VBlank interrupt
        interrupt_registers.request_interrupt(Interrupt::VBlank);
        assert_eq!(interrupt_registers.interrupt_flags, InterruptFlags::VBLANK);

        // Request a Timer interrupt
        interrupt_registers.request_interrupt(Interrupt::Timer);
        assert_eq!(
            interrupt_registers.interrupt_flags,
            InterruptFlags::VBLANK | InterruptFlags::TIMER
        ); // both VBlank and Timer interrupts are now pending
    }

    #[test]
    fn acknowledge_interrupt() {
        let mut interrupt_registers = InterruptRegisters::new();
        interrupt_registers.request_interrupt(Interrupt::VBlank);
        interrupt_registers.request_interrupt(Interrupt::Timer);

        // Acknowledge the VBlank interrupt
        interrupt_registers.acknowledge_interrupt(Interrupt::VBlank);
        assert_eq!(interrupt_registers.interrupt_flags, InterruptFlags::TIMER); // only Timer interrupt is pending

        // Acknowledge the Timer interrupt
        interrupt_registers.acknowledge_interrupt(Interrupt::Timer);
        assert_eq!(interrupt_registers.interrupt_flags, InterruptFlags::empty()); // no pending interrupts
    }

    #[test]
    fn pending_interrupt() {
        let mut interrupt_registers = InterruptRegisters::new();
        interrupt_registers.request_interrupt(Interrupt::Timer);

        // No interrupts enabled, so pending_interrupt should return None
        assert_eq!(interrupt_registers.highest_pending_interrupt(), None);

        // Enable the Timer interrupt
        interrupt_registers.enable_interrupt(Interrupt::Timer);

        // Now pending_interrupt should return the Timer interrupt
        assert_eq!(
            interrupt_registers.highest_pending_interrupt(),
            Some(Interrupt::Timer)
        );

        // Request and enable a VBlank interrupt, which has higher priority than Timer
        interrupt_registers.request_interrupt(Interrupt::VBlank);
        interrupt_registers.enable_interrupt(Interrupt::VBlank);

        // Now pending_interrupt should return the VBlank interrupt, since it has higher priority
        assert_eq!(
            interrupt_registers.highest_pending_interrupt(),
            Some(Interrupt::VBlank)
        );

        // Acknowledge the VBlank interrupt
        interrupt_registers.acknowledge_interrupt(Interrupt::VBlank);

        // Now pending_interrupt should return the Timer interrupt, since the VBlank interrupt has been acknowledged
        assert_eq!(
            interrupt_registers.highest_pending_interrupt(),
            Some(Interrupt::Timer)
        );
    }
}
