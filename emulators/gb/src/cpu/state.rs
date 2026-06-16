pub const HALTED_CYCLES: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CPUState {
    Running,
    Halted,
    HaltBug,
    // Stopped,
}

impl CPUState {
    /// Wake the CPU from a halted state.
    pub const fn wake(&mut self) {
        *self = Self::Running;
    }

    /// Put the CPU into a halted state.
    pub const fn halt(&mut self) {
        *self = Self::Halted;
    }

    /// Put the CPU into a halt bug state.
    ///
    /// This state occurs when the CPU is halted while an interrupt is pending
    /// and IME is disabled. In this state, the CPU will execute the next
    /// instruction after the HALT instruction, but will not increment the
    /// program counter after executing that instruction.
    pub const fn halt_bug(&mut self) {
        *self = Self::HaltBug;
    }

    /// Checks if the CPU is in a halted state.
    pub const fn is_halted(self) -> bool {
        matches!(self, Self::Halted)
    }

    /// Checks if the CPU is in a halt bug state.
    pub const fn is_halt_bug(self) -> bool {
        matches!(self, Self::HaltBug)
    }
}
