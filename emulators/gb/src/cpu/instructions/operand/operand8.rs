use crate::cpu::CPU;
use crate::cpu::HIGH_MEM_OFFSET;
use crate::cpu::alu;

use crate::cpu::instructions::parameter::{
    ALU8Param, LD8DstParam, LD8SrcParam, R8Param, R16MemParam,
};

use emu::MemoryBus;

pub(crate) enum Operand8 {
    B,
    C,
    D,
    E,
    H,
    L,
    IndHL,
    A,
    IndBC,
    IndDE,
    IndHLi,
    IndHLd,
    N8,
    E8,
    IndN16,
    IndHighMemC,
    IndHighMemA8,
}

impl Operand8 {
    pub(crate) fn read<M: MemoryBus>(&self, cpu: &mut CPU, bus: &M) -> u8 {
        match self {
            Operand8::B => cpu.registers.b,
            Operand8::C => cpu.registers.c,
            Operand8::D => cpu.registers.d,
            Operand8::E => cpu.registers.e,
            Operand8::H => cpu.registers.h,
            Operand8::L => cpu.registers.l,
            Operand8::IndHL => bus.read(cpu.registers.hl_get()),
            Operand8::A => cpu.registers.a,
            Operand8::IndBC => bus.read(cpu.registers.bc_get()),
            Operand8::IndDE => bus.read(cpu.registers.de_get()),
            Operand8::IndHLi => {
                let value = bus.read(cpu.registers.hl_get());
                let (hli, _) = alu::inc16(cpu.registers.hl_get());
                cpu.registers.hl_set(hli);
                value
            }
            Operand8::IndHLd => {
                let value = bus.read(cpu.registers.hl_get());
                let (hld, _) = alu::dec16(cpu.registers.hl_get());
                cpu.registers.hl_set(hld);
                value
            }
            Operand8::N8 => cpu.fetch_byte(bus),
            Operand8::E8 => cpu.fetch_byte(bus), // cast to i8 is left to the caller
            Operand8::IndN16 => bus.read(cpu.fetch_word(bus)),
            Operand8::IndHighMemC => bus.read(HIGH_MEM_OFFSET + cpu.registers.c as u16),
            Operand8::IndHighMemA8 => bus.read(HIGH_MEM_OFFSET + cpu.fetch_byte(bus) as u16),
        }
    }

    pub(crate) fn write<M: MemoryBus>(&self, cpu: &mut CPU, bus: &mut M, value: u8) {
        match self {
            Operand8::B => cpu.registers.b = value,
            Operand8::C => cpu.registers.c = value,
            Operand8::D => cpu.registers.d = value,
            Operand8::E => cpu.registers.e = value,
            Operand8::H => cpu.registers.h = value,
            Operand8::L => cpu.registers.l = value,
            Operand8::IndHL => bus.write(cpu.registers.hl_get(), value),
            Operand8::A => cpu.registers.a = value,
            Operand8::IndBC => bus.write(cpu.registers.bc_get(), value),
            Operand8::IndDE => bus.write(cpu.registers.de_get(), value),
            Operand8::IndHLi => {
                bus.write(cpu.registers.hl_get(), value);
                let (hli, _) = alu::inc16(cpu.registers.hl_get());
                cpu.registers.hl_set(hli);
            }
            Operand8::IndHLd => {
                bus.write(cpu.registers.hl_get(), value);
                let (hld, _) = alu::dec16(cpu.registers.hl_get());
                cpu.registers.hl_set(hld);
            }
            Operand8::IndN16 => bus.write(cpu.fetch_word(bus), value),
            Operand8::IndHighMemC => bus.write(HIGH_MEM_OFFSET + cpu.registers.c as u16, value),
            Operand8::IndHighMemA8 => {
                bus.write(HIGH_MEM_OFFSET + cpu.fetch_byte(bus) as u16, value)
            }
            _ => unimplemented!("unsupported write parameter"),
        }
    }
}

impl From<R8Param> for Operand8 {
    fn from(value: R8Param) -> Self {
        match value {
            R8Param::B => Self::B,
            R8Param::C => Self::C,
            R8Param::D => Self::D,
            R8Param::E => Self::E,
            R8Param::H => Self::H,
            R8Param::L => Self::L,
            R8Param::IndHL => Self::IndHL,
            R8Param::A => Self::A,
        }
    }
}

impl From<LD8SrcParam> for Operand8 {
    fn from(value: LD8SrcParam) -> Self {
        match value {
            LD8SrcParam::R8(r8) => Self::from(r8),
            LD8SrcParam::R16Mem(r16_mem) => match r16_mem {
                R16MemParam::IndBC => Self::IndBC,
                R16MemParam::IndDE => Self::IndDE,
                R16MemParam::IndHLi => Self::IndHLi,
                R16MemParam::IndHLd => Self::IndHLd,
            },
            LD8SrcParam::N8 => Self::N8,
            LD8SrcParam::IndHighMemC => Self::IndHighMemC,
            LD8SrcParam::IndHighMemA8 => Self::IndHighMemA8,
            LD8SrcParam::IndN16 => Self::IndN16,
        }
    }
}

impl From<LD8DstParam> for Operand8 {
    fn from(value: LD8DstParam) -> Self {
        match value {
            LD8DstParam::R8(r8) => Self::from(r8),
            LD8DstParam::R16Mem(r16_mem) => match r16_mem {
                R16MemParam::IndBC => Self::IndBC,
                R16MemParam::IndDE => Self::IndDE,
                R16MemParam::IndHLi => Self::IndHLi,
                R16MemParam::IndHLd => Self::IndHLd,
            },
            LD8DstParam::IndHighMemC => Self::IndHighMemC,
            LD8DstParam::IndHighMemA8 => Self::IndHighMemA8,
            LD8DstParam::IndN16 => Self::IndN16,
        }
    }
}

impl From<ALU8Param> for Operand8 {
    fn from(value: ALU8Param) -> Self {
        match value {
            ALU8Param::R8(r8) => Self::from(r8),
            ALU8Param::N8 => Self::N8,
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
        fn b() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.b = 0x12;

            assert_eq!(Operand8::B.read(&mut cpu, &bus), 0x12);
        }

        #[test]
        fn c() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.c = 0x34;

            assert_eq!(Operand8::C.read(&mut cpu, &bus), 0x34);
        }

        #[test]
        fn d() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.d = 0x56;

            assert_eq!(Operand8::D.read(&mut cpu, &bus), 0x56);
        }

        #[test]
        fn e() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.e = 0x78;

            assert_eq!(Operand8::E.read(&mut cpu, &bus), 0x78);
        }

        #[test]
        fn h() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.h = 0x9A;

            assert_eq!(Operand8::H.read(&mut cpu, &bus), 0x9A);
        }

        #[test]
        fn l() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.l = 0xBC;

            assert_eq!(Operand8::L.read(&mut cpu, &bus), 0xBC);
        }

        #[test]
        fn ind_hl() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x9A;
            cpu.registers.l = 0xBC;
            bus.mem[0x9ABC] = 0xDE; // IndHL

            assert_eq!(Operand8::IndHL.read(&mut cpu, &bus), 0xDE);
        }

        #[test]
        fn a() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0xF0;

            assert_eq!(Operand8::A.read(&mut cpu, &bus), 0xF0);
        }

        #[test]
        fn ind_bc() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x12;
            cpu.registers.c = 0x34;
            bus.mem[0x1234] = 0x0F; // IndBC

            assert_eq!(Operand8::IndBC.read(&mut cpu, &bus), 0x0F);
        }

        #[test]
        fn ind_de() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.d = 0x56;
            cpu.registers.e = 0x78;
            bus.mem[0x5678] = 0xED; // IndDE

            assert_eq!(Operand8::IndDE.read(&mut cpu, &bus), 0xED);
        }

        #[test]
        fn ind_hl_i() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x9A;
            cpu.registers.l = 0xBC;
            bus.mem[0x9ABC] = 0xCB; // IndHL

            assert_eq!(Operand8::IndHLi.read(&mut cpu, &bus), 0xCB);
            assert_eq!(cpu.registers.hl_get(), 0x9ABD);
        }

        #[test]
        fn ind_hl_d() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x9A;
            cpu.registers.l = 0xBC;
            bus.mem[0x9ABC] = 0xA9; // IndHL

            assert_eq!(Operand8::IndHLd.read(&mut cpu, &bus), 0xA9);
            assert_eq!(cpu.registers.hl_get(), 0x9ABB);
        }

        #[test]
        fn n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8765;
            bus.mem[0x8765] = 0x43; // N8

            assert_eq!(Operand8::N8.read(&mut cpu, &bus), 0x43);
            assert_eq!(cpu.pc, 0x8766);
        }

        #[test]
        fn e8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8766;
            bus.mem[0x8766] = 0x21; // N8

            assert_eq!(Operand8::E8.read(&mut cpu, &bus), 0x21);
            assert_eq!(cpu.pc, 0x8767);
        }

        #[test]
        fn ind_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8767;
            bus.mem[0x8767] = 0x24; // N8 - lo
            bus.mem[0x8768] = 0x13; // N8 - hi
            bus.mem[0x1324] = 0x35; // IndN16

            assert_eq!(Operand8::IndN16.read(&mut cpu, &bus), 0x35);
            assert_eq!(cpu.pc, 0x8769);
        }

        #[test]
        fn ind_high_mem_c() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.c = 0x46;
            bus.mem[HIGH_MEM_OFFSET as usize + 0x46] = 0x57; // IndHighMemC

            assert_eq!(Operand8::IndHighMemC.read(&mut cpu, &bus), 0x57);
        }

        #[test]
        fn ind_high_mem_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8769;
            bus.mem[0x8769] = 0x68; // A8
            bus.mem[HIGH_MEM_OFFSET as usize + 0x68] = 0x79; // IndHighMemA8

            assert_eq!(Operand8::IndHighMemA8.read(&mut cpu, &bus), 0x79);
            assert_eq!(cpu.pc, 0x876A);
        }
    }

    mod write {
        use super::*;

        #[test]
        fn b() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::B.write(&mut cpu, &mut bus, 0x12);
            assert_eq!(cpu.registers.b, 0x12);
        }

        #[test]
        fn c() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::C.write(&mut cpu, &mut bus, 0x34);
            assert_eq!(cpu.registers.c, 0x34);
        }

        #[test]
        fn d() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::D.write(&mut cpu, &mut bus, 0x56);
            assert_eq!(cpu.registers.d, 0x56);
        }

        #[test]
        fn e() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::E.write(&mut cpu, &mut bus, 0x78);
            assert_eq!(cpu.registers.e, 0x78);
        }

        #[test]
        fn h() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::H.write(&mut cpu, &mut bus, 0x9A);
            assert_eq!(cpu.registers.h, 0x9A);
        }

        #[test]
        fn l() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::L.write(&mut cpu, &mut bus, 0xBC);
            assert_eq!(cpu.registers.l, 0xBC);
        }

        #[test]
        fn ind_hl() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x9A;
            cpu.registers.l = 0xBC;

            Operand8::IndHL.write(&mut cpu, &mut bus, 0xDE);
            assert_eq!(bus.mem[0x9ABC], 0xDE);
        }

        #[test]
        fn a() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            Operand8::A.write(&mut cpu, &mut bus, 0xF0);
            assert_eq!(cpu.registers.a, 0xF0);
        }

        #[test]
        fn ind_bc() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x12;
            cpu.registers.c = 0x34;

            Operand8::IndBC.write(&mut cpu, &mut bus, 0x0F);
            assert_eq!(bus.mem[0x1234], 0x0F);
        }

        #[test]
        fn ind_de() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.d = 0x56;
            cpu.registers.e = 0x78;

            Operand8::IndDE.write(&mut cpu, &mut bus, 0xED);
            assert_eq!(bus.mem[0x5678], 0xED);
        }

        #[test]
        fn ind_hl_i() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x9A;
            cpu.registers.l = 0xBC;

            Operand8::IndHLi.write(&mut cpu, &mut bus, 0xCB);
            assert_eq!(bus.mem[0x9ABC], 0xCB);
            assert_eq!(cpu.registers.hl_get(), 0x9ABD);
        }

        #[test]
        fn ind_hl_d() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x9A;
            cpu.registers.l = 0xBC;

            Operand8::IndHLd.write(&mut cpu, &mut bus, 0xA9);
            assert_eq!(bus.mem[0x9ABC], 0xA9);
            assert_eq!(cpu.registers.hl_get(), 0x9ABB);
        }

        #[test]
        fn ind_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8765;
            bus.mem[0x8765] = 0x44; // N8 - lo
            bus.mem[0x8766] = 0x43; // N8 - hi
            Operand8::IndN16.write(&mut cpu, &mut bus, 0x35);
            assert_eq!(bus.mem[0x4344], 0x35);
            assert_eq!(cpu.pc, 0x8767);
        }

        #[test]
        fn ind_high_mem_c() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.c = 0x46;
            Operand8::IndHighMemC.write(&mut cpu, &mut bus, 0x57);
            assert_eq!(bus.mem[HIGH_MEM_OFFSET as usize + 0x46], 0x57);
        }

        #[test]
        fn ind_high_mem_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x8767;
            bus.mem[0x8767] = 0x45; // N8
            Operand8::IndHighMemA8.write(&mut cpu, &mut bus, 0x79);
            assert_eq!(bus.mem[HIGH_MEM_OFFSET as usize + 0x45], 0x79);
        }
    }
}
