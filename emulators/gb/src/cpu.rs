mod alu;
mod instructions;
mod registers;

use emu::MemoryBus;
use registers::Registers;

pub struct CPU {
    registers: Registers,

    sp: u16,
    pc: u16,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            sp: 0,
            pc: 0,
        }
    }

    pub fn step<M: MemoryBus>(&mut self, mem_bus: &mut M) -> u32 {
        todo!();

        // // this is just a rough outline of the step function
        // let opcode = mem_bus.read(self.pc);
        // let instruction = &UNPREFIXED_INSTRUCTIONS[opcode as usize];
        // // (instruction.execute)(self);
        // {
        //     let flags = alu::bit(1, self.registers.a); // just an example of how the CPU might call the ALU for a BIT instruction
        //     self.copy_flags::<{ flag_mask::Z | flag_mask::N | flag_mask::H }>(&flags);
        // }
        // {
        //     let (result, flags) = alu::add(self.registers.a, self.registers.b); // just an example of how the CPU might call the ALU for an ADD instruction
        //     self.registers.a = result;
        //     self.copy_all_flags(&flags);
        // }
        // self.pc = self.pc.wrapping_add(instruction.bytes as u16);
        // instruction.cycles[0] as u32
    }

    /// Copy chosen `alu::Flags` into `CPU::registers` flags.
    ///
    /// Use `FLAG_MASK` to chose what flags you want to copy into the CPU's
    /// flag register. Helper masks can be found in the module `flag_mask`
    ///
    /// # Flags
    /// - `flag_mask::Z` : if set, the zero flag from the `flags` parameter will
    ///     be copied into the CPU's registers.
    /// - `flag_mask::N` : if set, the substract flag from `flags` parameter
    ///     will be copied into the CPU's registers.
    /// - `flag_mask::H` : if set, the half-carry flag from `flags` parameter
    ///     will be copied into the CPU's registers.
    /// - `flag_mask::C` : if set, the carry flag from `flags` parameter will be
    ///     copied into the CPU's registers.
    /// - `flag_mask::ALL` : bitwise OR of the `Z`, `N`, `H` and `C` masks,
    ///     this will copy all the flags from the `flags` parameter into the
    ///     CPU's registers.
    ///
    /// # Example
    /// ```no_run
    /// let mut cpu = CPU::new();
    /// let (_, flags) = alu::sub(0, 1);
    /// cpu.copy_flags::<{flag_mask::Z | flag_mask::N | flag_mask::H | flag_mask::C}>(flags); // `flag_mask::ALL` could also have been used here
    /// assert!(!cpu.registers.flags().zero())
    /// assert!(cpu.registers.flags().substract())
    /// assert!(cpu.registers.flags().half_carry())
    /// assert!(cpu.registers.flags().carry())
    /// ```
    fn copy_flags<const FLAG_MASK: u8>(&mut self, flags: &alu::Flags) {
        if FLAG_MASK & flag_mask::Z != 0 {
            self.registers.flags_mut().zero().set(flags.zero().unwrap());
        }
        if FLAG_MASK & flag_mask::N != 0 {
            self.registers.flags_mut().subtract().set(flags.subtract().unwrap());
        }
        if FLAG_MASK & flag_mask::H != 0 {
            self.registers
                .flags_mut()
                .half_carry()
                .set(flags.half_carry().unwrap());
        }
        if FLAG_MASK & flag_mask::C != 0 {
            self.registers.flags_mut().carry().set(flags.carry().unwrap());
        }
    }
 
    /// Copy all given `alu::Flags` into `CPU::registers` flags.
    /// 
    /// See `CPU::copy_flags` for more granularity over which flag is copied or
    /// not.
    /// 
    /// # Example
    /// ```no_run
    /// let mut cpu = CPU::new();
    /// let (_, flags) = alu::sub(0, 1);
    /// cpu.copy_all_flags(flags);
    /// assert!(!cpu.registers.flags().zero())
    /// assert!(cpu.registers.flags().substract())
    /// assert!(cpu.registers.flags().half_carry())
    /// assert!(cpu.registers.flags().carry())
    /// ```
    fn copy_all_flags(&mut self, flags: &alu::Flags) {
        self.copy_flags::<{flag_mask::ALL}>(flags)
    }
}

/// helper module for the `CPU::apply_mask` function
///
/// # Masks
/// - `flag_mask::Z` : mask for the zero flag
/// - `flag_mask::N` : mask for the substract flag
/// - `flag_mask::H` : mask for the the half-carry flag
/// - `flag_mask::C` : mask for the carry flag
/// - `flag_mask::ALL` : bitwise OR of the `Z`, `N`, `H` and `C` masks
mod flag_mask {
    /// mask for the zero flag
    pub(crate) const Z: u8 = 0b0001;

    /// mask for the substract flag
    pub(crate) const N: u8 = 0b0010;

    /// mask for the half-carry flag
    pub(crate) const H: u8 = 0b0100;

    /// mask for the carry flag
    pub(crate) const C: u8 = 0b1000;

    /// bitwise OR of the `Z`, `N`, `H` and `C` masks
    pub(crate) const ALL: u8 = Z | N | H | C;
}

#[cfg(test)]
mod tests {
    use super::*;
}
