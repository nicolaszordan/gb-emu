use crate::cpu::instructions::meta::CBPREFIXED_INSTRUCTIONS;
use crate::cpu::instructions::meta::InstructionMeta;
use crate::cpu::instructions::meta::UNPREFIXED_INSTRUCTIONS;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Opcode(u8);

impl Opcode {
    /// Create a new [`Opcode`] from the given `opcode`.
    pub const fn new(opcode: u8) -> Self {
        Self(opcode)
    }

    /// Get the underlying raw opcode value.
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Get the [`InstructionMeta`] corresponding to the underlying opcode from the main instruction table.
    pub const fn meta(self) -> &'static InstructionMeta {
        &UNPREFIXED_INSTRUCTIONS[self.0 as usize]
    }

    /// Get the [`InstructionMeta`] corresponding to the underlying opcode from the extended instruction table.
    pub const fn ext_meta(self) -> &'static InstructionMeta {
        &CBPREFIXED_INSTRUCTIONS[self.0 as usize]
    }
}

#[cfg(test)]
impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
