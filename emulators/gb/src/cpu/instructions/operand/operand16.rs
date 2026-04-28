use crate::cpu::CPU;

use crate::cpu::instructions::parameter::{
    AddSPe8DstParam, CallParam, JumpParam, LD16DstParam, LD16SrcParam, R16Param, R16StackParam,
};

use emu::MemoryBus;

pub(crate) enum Operand16 {
    BC,
    DE,
    HL,
    SP,
    AF,
    N16,
    IndN16,
    PCE8,
    VEC(u16),
}

impl Operand16 {
    pub(crate) fn read<M: MemoryBus>(&self, cpu: &mut CPU, bus: &M) -> u16 {
        match self {
            Operand16::BC => cpu.registers.bc_get(),
            Operand16::DE => cpu.registers.de_get(),
            Operand16::HL => cpu.registers.hl_get(),
            Operand16::SP => cpu.sp,
            Operand16::AF => cpu.registers.af_get(),
            Operand16::N16 => cpu.fetch_word(bus),
            Operand16::IndN16 => bus.read_word(cpu.fetch_word(bus)),
            Operand16::PCE8 => {
                let e8 = cpu.fetch_byte(bus) as i8;
                cpu.pc.wrapping_add_signed(e8 as i16)
            }
            Operand16::VEC(param) => *param,
        }
    }

    pub(crate) fn write<M: MemoryBus>(&self, cpu: &mut CPU, bus: &mut M, value: u16) {
        match self {
            Operand16::BC => cpu.registers.bc_set(value),
            Operand16::DE => cpu.registers.de_set(value),
            Operand16::HL => cpu.registers.hl_set(value),
            Operand16::SP => cpu.sp = value,
            Operand16::AF => cpu.registers.af_set(value),
            Operand16::IndN16 => bus.write_word(cpu.fetch_word(bus), value),
            _ => unimplemented!("unsupported write parameter"),
        }
    }
}

impl From<R16Param> for Operand16 {
    fn from(value: R16Param) -> Self {
        match value {
            R16Param::BC => Self::BC,
            R16Param::DE => Self::DE,
            R16Param::HL => Self::HL,
            R16Param::SP => Self::SP,
        }
    }
}

impl From<R16StackParam> for Operand16 {
    fn from(value: R16StackParam) -> Self {
        match value {
            R16StackParam::BC => Self::BC,
            R16StackParam::DE => Self::DE,
            R16StackParam::HL => Self::HL,
            R16StackParam::AF => Self::AF,
        }
    }
}

impl From<LD16SrcParam> for Operand16 {
    fn from(value: LD16SrcParam) -> Self {
        match value {
            LD16SrcParam::HL => Self::HL,
            LD16SrcParam::SP => Self::SP,
            LD16SrcParam::N16 => Self::N16,
        }
    }
}

impl From<LD16DstParam> for Operand16 {
    fn from(value: LD16DstParam) -> Self {
        match value {
            LD16DstParam::R16(r16) => Self::from(r16),
            LD16DstParam::IndN16 => Self::IndN16,
        }
    }
}

impl From<JumpParam> for Operand16 {
    fn from(value: JumpParam) -> Self {
        match value {
            JumpParam::PCE8 => Self::PCE8,
            JumpParam::N16 => Self::N16,
            JumpParam::HL => Self::HL,
        }
    }
}

impl From<CallParam> for Operand16 {
    fn from(value: CallParam) -> Self {
        match value {
            CallParam::N16 => Self::N16,
            CallParam::VEC(v) => Self::VEC(v),
        }
    }
}

impl From<AddSPe8DstParam> for Operand16 {
    fn from(value: AddSPe8DstParam) -> Self {
        match value {
            AddSPe8DstParam::HL => Self::HL,
            AddSPe8DstParam::SP => Self::SP,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::tests::MockBus as Bus;

    use super::*;

    mod read {
        use super::*;

        #[test]
        fn bc() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.bc_set(0x1234);

            assert_eq!(Operand16::BC.read(&mut cpu, &bus), 0x1234);
        }

        #[test]
        fn de() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.de_set(0x5678);

            assert_eq!(Operand16::DE.read(&mut cpu, &bus), 0x5678);
        }

        #[test]
        fn hl() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.hl_set(0x9ABC);

            assert_eq!(Operand16::HL.read(&mut cpu, &bus), 0x9ABC);
        }

        #[test]
        fn sp() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.sp = 0xDEF0;

            assert_eq!(Operand16::SP.read(&mut cpu, &bus), 0xDEF0);
        }

        #[test]
        fn af() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.af_set(0x1357);

            assert_eq!(Operand16::AF.read(&mut cpu, &bus), 0x1350); // lower 4 bits of F are always 0
        }

        #[test]
        fn n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8765;
            bus.mem[0x8765] = 0x21; // N16 - lo
            bus.mem[0x8766] = 0x43; // N16 - hi
            assert_eq!(Operand16::N16.read(&mut cpu, &bus), 0x4321);
            assert_eq!(cpu.pc, 0x8767);
        }

        #[test]
        fn ind_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8767;
            bus.mem[0x8767] = 0x24; // N16 - lo
            bus.mem[0x8768] = 0x13; // N16 - hi
            bus.mem[0x1324] = 0x35; // IndN16 - lo
            bus.mem[0x1325] = 0x36; // IndN16 - hi
            assert_eq!(Operand16::IndN16.read(&mut cpu, &bus), 0x3635);
            assert_eq!(cpu.pc, 0x8769);
        }

        #[test]
        fn pc_e8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8769;
            bus.mem[0x8769] = 0x35; // E8
            assert_eq!(Operand16::PCE8.read(&mut cpu, &bus), 0x876A + 0x35); // E8 is added after we retrieved it and moved PC forward
            assert_eq!(cpu.pc, 0x876A); // PC isn't moved by the read itself

            bus.mem[0x876A] = -0x0A as i8 as u8;
            assert_eq!(Operand16::PCE8.read(&mut cpu, &bus), 0x876B - 0x0A); // E8 is added after we retrieved it and moved PC forward
            assert_eq!(cpu.pc, 0x876B); // PC isn't moved by the read itself
        }

        #[test]
        fn vec() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            assert_eq!(Operand16::VEC(0x7531).read(&mut cpu, &bus), 0x7531);
        }
    }

    mod write {
        use super::*;

        #[test]
        fn bc() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand16::BC.write(&mut cpu, &mut bus, 0x1234);
            assert_eq!(cpu.registers.bc_get(), 0x1234);
        }

        #[test]
        fn de() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand16::DE.write(&mut cpu, &mut bus, 0x5678);
            assert_eq!(cpu.registers.de_get(), 0x5678);
        }

        #[test]
        fn hl() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand16::HL.write(&mut cpu, &mut bus, 0x9ABC);
            assert_eq!(cpu.registers.hl_get(), 0x9ABC);
        }

        #[test]
        fn sp() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand16::SP.write(&mut cpu, &mut bus, 0xDEF0);
            assert_eq!(cpu.sp, 0xDEF0);
        }

        #[test]
        fn af() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand16::AF.write(&mut cpu, &mut bus, 0x1357);
            assert_eq!(cpu.registers.af_get(), 0x1350); // lower 4 bits of F are always 0
        }

        #[test]
        fn ind_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8765;
            bus.mem[0x8765] = 0x44; // N16 - lo
            bus.mem[0x8766] = 0x43; // N16 - hi
            Operand16::IndN16.write(&mut cpu, &mut bus, 0x9876);
            assert_eq!(bus.mem[0x4344], 0x76); // lo byte
            assert_eq!(bus.mem[0x4345], 0x98); // hi byte
            assert_eq!(cpu.pc, 0x8767);
        }
    }
}
