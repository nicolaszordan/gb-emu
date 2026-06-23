use crate::cpu::instructions::meta::CBPREFIXED_INSTRUCTIONS;
use crate::cpu::instructions::meta::InstructionMeta;
use crate::cpu::instructions::meta::UNPREFIXED_INSTRUCTIONS;

use bitvec::prelude::*;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Opcode(u8);

impl Opcode {
    pub const fn new(opcode: u8) -> Self {
        Self(opcode)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn meta(self) -> &'static InstructionMeta {
        &UNPREFIXED_INSTRUCTIONS[self.0 as usize]
    }

    pub const fn ext_meta(self) -> &'static InstructionMeta {
        &CBPREFIXED_INSTRUCTIONS[self.0 as usize]
    }

    pub fn bits(&self) -> &BitSlice<u8, Lsb0> {
        self.0.view_bits::<Lsb0>()
    }
}

#[cfg(test)]
impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_ranges() {
        let opcode = Opcode(0b101_010);

        let src: u8 = opcode.bits()[0..3].load();
        let dst: u8 = opcode.bits()[3..6].load();

        assert_eq!(src, 0b010);
        assert_eq!(dst, 0b101);
    }
}
