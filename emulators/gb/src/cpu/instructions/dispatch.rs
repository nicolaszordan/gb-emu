use crate::cpu::CPU;
use crate::interrupts::InterruptBus;

use super::condition::Condition;
use super::meta;

#[allow(clippy::wildcard_imports)] // we need and use all the params
use super::parameter::*;

use emu::MemoryBus;

impl CPU {
    /// Dispatch and execute an instruction from the main instruction table
    /// designated by `opcode`.
    pub(crate) fn execute_instruction<B: MemoryBus + InterruptBus>(
        &mut self,
        bus: &mut B,
        opcode: u8,
    ) -> u32 {
        let additional_cycles = match opcode {
            0x00 => {
                // NOP
                0
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                self.instr_ld16(bus, R16Param::from(opcode >> 4).into(), LD16SrcParam::N16)
            }
            0x02 | 0x12 | 0x22 | 0x32 => self.instr_ld8(
                bus,
                R16MemParam::from(opcode >> 4).into(),
                R8Param::A.into(),
            ),
            0x03 | 0x13 | 0x23 | 0x33 => self.instr_inc16(bus, R16Param::from(opcode >> 4)),
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                self.instr_inc8(bus, R8Param::from(opcode >> 3))
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                self.instr_dec8(bus, R8Param::from(opcode >> 3))
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                self.instr_ld8(bus, R8Param::from(opcode >> 3).into(), LD8SrcParam::N8)
            }
            0x07 | 0x0F | 0x17 | 0x1F => {
                self.instr_rotate_acc(bus, BitRotateOperation::from(opcode >> 3))
            }
            0x08 => self.instr_ld16(bus, LD16DstParam::IndN16, LD16SrcParam::SP),
            0x09 | 0x19 | 0x29 | 0x39 => self.instr_add16(bus, R16Param::from(opcode >> 4)),
            0x0A | 0x1A | 0x2A | 0x3A => self.instr_ld8(
                bus,
                R8Param::A.into(),
                R16MemParam::from(opcode >> 4).into(),
            ),
            0x0B | 0x1B | 0x2B | 0x3B => self.instr_dec16(bus, R16Param::from(opcode >> 4)),
            0x10 => self.instr_stop(),
            0x18 => self.instr_jump(bus, JumpParam::PCE8),
            0x20 | 0x28 | 0x30 | 0x38 => {
                self.instr_cond_jump(bus, Condition::from(opcode >> 3), JumpParam::PCE8)
            }
            0x27 => self.instr_daa(),
            0x2F => self.instr_cpl(),
            0x37 => self.instr_scf(),
            0x3F => self.instr_ccf(),
            0x76 => self.instr_halt(bus),
            0x40..=0x7F => self.instr_ld8(
                bus,
                R8Param::from(opcode >> 3).into(),
                R8Param::from(opcode).into(),
            ),
            0x80..=0xBF => self.instr_alu(
                bus,
                ALUOperation::from(opcode >> 3),
                R8Param::from(opcode).into(),
            ),
            0xC0 | 0xC8 | 0xD0 | 0xD8 => self.instr_cond_ret(bus, Condition::from(opcode >> 3)),
            0xC1 | 0xD1 | 0xE1 | 0xF1 => self.instr_pop(bus, R16StackParam::from(opcode >> 4)),
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                self.instr_cond_jump(bus, Condition::from(opcode >> 3), JumpParam::N16)
            }
            0xC3 => self.instr_jump(bus, JumpParam::N16),
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                self.instr_cond_call(bus, Condition::from(opcode >> 3), CallParam::N16)
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => self.instr_push(bus, R16StackParam::from(opcode >> 4)),
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                self.instr_alu(bus, ALUOperation::from(opcode >> 3), ALU8Param::N8)
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                self.instr_call(bus, CallParam::VEC(u16::from(opcode) & 0b0011_1000))
            }
            0xC9 => self.instr_ret(bus),
            0xCB => self.instr_prefix(bus),
            0xCD => self.instr_call(bus, CallParam::N16),
            0xD9 => self.instr_reti(bus),
            0xE0 => self.instr_ld8(bus, LD8DstParam::IndHighMemA8, R8Param::A.into()),
            0xE2 => self.instr_ld8(bus, LD8DstParam::IndHighMemC, R8Param::A.into()),
            0xE8 => self.instr_add_spe8(bus, AddSPe8DstParam::SP),
            0xE9 => self.instr_jump(bus, JumpParam::HL),
            0xEA => self.instr_ld8(bus, LD8DstParam::IndN16, R8Param::A.into()),
            0xF0 => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::IndHighMemA8),
            0xF2 => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::IndHighMemC),
            0xF3 => self.instr_di(),
            0xF8 => self.instr_add_spe8(bus, AddSPe8DstParam::HL),
            0xF9 => self.instr_ld16(bus, R16Param::SP.into(), LD16SrcParam::HL),
            0xFA => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::IndN16),
            0xFB => self.instr_ei(),

            0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => {
                // TODO: this  panic! will be changed when we implement CPU states to enter a "bricked" state instead of panicking.
                panic!("Invalid opcode: 0x{opcode:02X}");
            }
        };

        u32::from(meta::UNPREFIXED_INSTRUCTIONS[opcode as usize].cycles) + additional_cycles
    }

    /// Dispatch and execute an instruction from the extended instruction table
    /// designated by `opcode`.
    pub(crate) fn execute_extended_instruction<M: MemoryBus>(
        &mut self,
        mem_bus: &mut M,
        opcode: u8,
    ) -> u32 {
        // NOTE: all extended hanlders return 0 => we're ignoring their return value and just return the cycles from the meta table.
        match opcode {
            0x00..=0x3F => self.instr_ext_bit_shift(
                mem_bus,
                BitShiftOperation::from(opcode >> 3),
                R8Param::from(opcode),
            ),

            0x40..=0x7F => self.instr_ext_bit(
                mem_bus,
                BitIndex::from_low_bits(opcode >> 3),
                R8Param::from(opcode),
            ),

            0x80..=0xBF => self.instr_ext_res(
                mem_bus,
                BitIndex::from_low_bits(opcode >> 3),
                R8Param::from(opcode),
            ),

            0xC0..=0xFF => self.instr_ext_set(
                mem_bus,
                BitIndex::from_low_bits(opcode >> 3),
                R8Param::from(opcode),
            ),
        };

        u32::from(meta::CBPREFIXED_INSTRUCTIONS[opcode as usize].cycles)
    }
}

#[cfg(test)]
#[allow(non_snake_case)] // help A LOT to have upper cases for instr names and regs
mod tests {
    use crate::cpu::interrupts::IME;
    use crate::cpu::state::CPUState;
    use crate::cpu::tests::MockInterruptBus as Bus;

    use super::*;

    mod ld8 {
        use super::*;

        #[test]
        fn LD_B_C() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0xFF;
            cpu.registers.c = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x41); // LD B C

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.b, 0x01);
            assert_eq!(cpu.registers.c, 0x01);
        }

        #[test]
        fn LD_C_D() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.c = 0xFF;
            cpu.registers.d = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x4A); // LD C D

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.c, 0x01);
            assert_eq!(cpu.registers.d, 0x01);
        }

        #[test]
        fn LD_D_E() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.d = 0xFF;
            cpu.registers.e = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x53); // LD D E

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.d, 0x01);
            assert_eq!(cpu.registers.e, 0x01);
        }

        #[test]
        fn LD_E_H() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.e = 0xFF;
            cpu.registers.h = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x5C); // LD E H

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.e, 0x01);
            assert_eq!(cpu.registers.h, 0x01);
        }

        #[test]
        fn LD_H_L() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0xFF;
            cpu.registers.l = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x65); // LD H L

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.h, 0x01);
            assert_eq!(cpu.registers.l, 0x01);
        }

        #[test]
        fn LD_L_indHL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            bus.write(0x1234, 0x56);
            let cycles = cpu.execute_instruction(&mut bus, 0x6E); // LD L [HL]

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.l, 0x56);
            assert_eq!(bus.read(0x1234), 0x56);
        }

        #[test]
        fn LD_indHL_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            bus.write(0x1234, 0x56);
            cpu.registers.a = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x77); // LD [HL] A

            assert_eq!(cycles, 8);
            assert_eq!(bus.read(0x1234), 0x01);
            assert_eq!(cpu.registers.a, 0x01);
        }

        #[test]
        fn LD_A_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xFF;
            let cycles = cpu.execute_instruction(&mut bus, 0x7F); // LD A A

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0xFF);
        }

        #[test]
        fn LD_E_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.e = 0x12;
            cpu.pc = 0x0034;
            bus.write(0x0034, 0x56);
            let cycles = cpu.execute_instruction(&mut bus, 0x1E); // LD E n8

            assert_eq!(cycles, 8);
            assert_eq!(cpu.pc, 0x35);
            assert_eq!(cpu.registers.e, 0x56);
            assert_eq!(bus.read(0x0034), 0x56);
        }

        #[test]
        fn LD_indHL_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            cpu.pc = 0x0034;
            bus.write(0x0034, 0x56);
            bus.write(0x1234, 0x78);
            let cycles = cpu.execute_instruction(&mut bus, 0x36); // LD [HL] n8

            assert_eq!(cycles, 12);
            assert_eq!(cpu.pc, 0x35);
            assert_eq!(cpu.registers.h, 0x12);
            assert_eq!(cpu.registers.l, 0x34);
            assert_eq!(bus.read(0x1234), 0x56);
            assert_eq!(bus.read(0x0034), 0x56);
        }

        #[test]
        fn LD_A_indBC() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x12;
            cpu.registers.c = 0x34;
            bus.write(0x1234, 0x56);
            let cycles = cpu.execute_instruction(&mut bus, 0x0A); // LD A [BC]

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.a, 0x56);
            assert_eq!(bus.read(0x1234), 0x56);
        }

        #[test]
        fn LD_A_indDE() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.d = 0x12;
            cpu.registers.e = 0x34;
            bus.write(0x1234, 0x56);
            let cycles = cpu.execute_instruction(&mut bus, 0x1A); // LD A [DE]

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.a, 0x56);
            assert_eq!(bus.read(0x1234), 0x56);
        }

        #[test]
        fn LD_A_indHLi() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            bus.write(0x1234, 0x56);

            let cycles = cpu.execute_instruction(&mut bus, 0x2A); // LD A [HL+]

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.a, 0x56);
            assert_eq!(bus.read(0x1234), 0x56);
            assert_eq!(cpu.registers.hl_get(), 0x1235);
        }

        #[test]
        fn LD_indDE_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.d = 0x12;
            cpu.registers.e = 0x34;
            bus.write(0x1234, 0x56);
            cpu.registers.a = 0x78;
            let cycles = cpu.execute_instruction(&mut bus, 0x12); // LD [DE] A

            assert_eq!(cycles, 8);
            assert_eq!(bus.read(0x1234), 0x78);
            assert_eq!(cpu.registers.a, 0x78);
        }

        #[test]
        fn LD_indHLd_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            bus.write(0x1234, 0x56);
            cpu.registers.a = 0x78;
            let cycles = cpu.execute_instruction(&mut bus, 0x32); // LD [HL-] A

            assert_eq!(cycles, 8);
            assert_eq!(bus.read(0x1234), 0x78);
            assert_eq!(cpu.registers.a, 0x78);
            assert_eq!(cpu.registers.hl_get(), 0x1233);
        }

        #[test]
        fn LD_highmem_n8_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write(0x1234, 0x56);
            cpu.registers.a = 0x78;
            let cycles = cpu.execute_instruction(&mut bus, 0xE0); // LD [0xFF00 + n8] A

            assert_eq!(cycles, 12);
            assert_eq!(bus.read(0xFF56), 0x78);
            assert_eq!(cpu.registers.a, 0x78);
            assert_eq!(cpu.pc, 0x1235);
        }

        #[test]
        fn LD_A_highmem_C() {
            {
                // LD A [0xFF00 + C]
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0x34;
                bus.write(0xFF34, 0x56);
                let cycles = cpu.execute_instruction(&mut bus, 0xF2); // LD A [0xFF00 + C]

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.a, 0x56);
                assert_eq!(bus.read(0xFF34), 0x56);
            }
        }

        #[test]
        fn LD_A_indn16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);
            bus.write(0x5678, 0x9A);
            let cycles = cpu.execute_instruction(&mut bus, 0xFA); // LD A [N16]

            assert_eq!(cycles, 16);
            assert_eq!(cpu.registers.a, 0x9A);
            assert_eq!(bus.read(0x5678), 0x9A);
            assert_eq!(cpu.pc, 0x1236);
        }

        #[test]
        fn LD_indn16_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);
            cpu.registers.a = 0x9A;
            let cycles = cpu.execute_instruction(&mut bus, 0xEA); // LD [N16] A

            assert_eq!(cycles, 16);
            assert_eq!(bus.read(0x5678), 0x9A);
            assert_eq!(cpu.registers.a, 0x9A);
            assert_eq!(cpu.pc, 0x1236);
        }
    }

    mod ld16 {
        use super::*;

        #[test]
        fn LD_BC_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);

            let cycles = cpu.execute_instruction(&mut bus, 0x01); // LD BC n16

            assert_eq!(cycles, 12);
            assert_eq!(cpu.registers.bc_get(), 0x5678);
            assert_eq!(bus.read(0x1234), 0x78);
            assert_eq!(bus.read(0x1235), 0x56);
            assert_eq!(cpu.pc, 0x1236);
        }

        #[test]
        fn LD_HL_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);

            let cycles = cpu.execute_instruction(&mut bus, 0x21); // LD HL n16

            assert_eq!(cycles, 12);
            assert_eq!(cpu.registers.hl_get(), 0x5678);
            assert_eq!(bus.read(0x1234), 0x78);
            assert_eq!(bus.read(0x1235), 0x56);
            assert_eq!(cpu.pc, 0x1236);
        }

        #[test]
        fn LD_SP_n16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);

            let cycles = cpu.execute_instruction(&mut bus, 0x31); // LD SP n16

            assert_eq!(cycles, 12);
            assert_eq!(cpu.sp, 0x5678);
            assert_eq!(bus.read(0x1234), 0x78);
            assert_eq!(bus.read(0x1235), 0x56);
            assert_eq!(cpu.pc, 0x1236);
        }

        #[test]
        fn LD_indN16_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);
            cpu.sp = 0x9ABC;

            let cycles = cpu.execute_instruction(&mut bus, 0x08); // LD [N16] SP

            assert_eq!(cycles, 20);
            assert_eq!(bus.read_word(0x5678), 0x9ABC);
            assert_eq!(cpu.sp, 0x9ABC);
            assert_eq!(cpu.pc, 0x1236);
        }

        #[test]
        fn LD_SP_HL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x9ABC);

            let cycles = cpu.execute_instruction(&mut bus, 0xF9); // LD SP HL

            assert_eq!(cycles, 8);
            assert_eq!(cpu.sp, 0x9ABC);
            assert_eq!(cpu.registers.hl_get(), 0x9ABC);
        }
    }

    mod alu8 {
        use super::*;

        #[test]
        fn ADD_B() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xFF;
            cpu.registers.b = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x80); // ADD B

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0x00);
        }

        #[test]
        fn ADC_C() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x00;
            cpu.registers.c = 0x0F;
            cpu.registers.flags.c = true;
            let cycles = cpu.execute_instruction(&mut bus, 0x89); // ADC C

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0x10);
        }

        #[test]
        fn SUB_D() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x00;
            cpu.registers.d = 0x0F;
            let cycles = cpu.execute_instruction(&mut bus, 0x92); // SUB D

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0xF1);
        }

        #[test]
        fn SBC_E() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x00;
            cpu.registers.e = 0x0F;
            cpu.registers.flags.c = true;
            let cycles = cpu.execute_instruction(&mut bus, 0x9B); // SBC E

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0xF0);
        }

        #[test]
        fn AND_H() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.h = 0b0000_1111;
            let cycles = cpu.execute_instruction(&mut bus, 0xA4); // AND H

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0b0000_1100);
        }

        #[test]
        fn XOR_L() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.l = 0b0000_1111;
            let cycles = cpu.execute_instruction(&mut bus, 0xAD); // XOR L

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0b0011_0011);
        }

        #[test]
        fn OR_indHL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            bus.write(0x1234, 0b0000_1111);
            let cycles = cpu.execute_instruction(&mut bus, 0xB6); // OR [HL]

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.a, 0b0011_1111);
        }

        #[test]
        fn CP_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            let cycles = cpu.execute_instruction(&mut bus, 0xBF); // CP A

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0b0011_1100);
            assert!(cpu.registers.flags.z);
            assert!(cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);
        }

        #[test]
        fn ADD_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xFF;
            cpu.pc = 0x1234;
            bus.write(0x1234, 0x01);
            let cycles = cpu.execute_instruction(&mut bus, 0xC6); // ADD N8

            assert_eq!(cycles, 8);
            assert_eq!(cpu.pc, 0x1235);
            assert_eq!(cpu.registers.a, 0x00);
        }

        #[test]
        fn XOR_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.pc = 0x1234;
            bus.write(0x1234, 0b0000_1111);
            let cycles = cpu.execute_instruction(&mut bus, 0xEE); // XOR N8

            assert_eq!(cycles, 8);
            assert_eq!(cpu.pc, 0x1235);
            assert_eq!(cpu.registers.a, 0b0011_0011);
        }

        #[test]
        fn INC_B() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0xFF;
            let cycles = cpu.execute_instruction(&mut bus, 0x04); // INC B

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.b, 0x00);
        }

        #[test]
        fn INC_L() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.l = 0xF0;
            let cycles = cpu.execute_instruction(&mut bus, 0x2C); // INC L

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.l, 0xF1);
        }

        #[test]
        fn DEC_H() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0xFF;
            let cycles = cpu.execute_instruction(&mut bus, 0x25); // DEC H

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.h, 0xFE);
        }

        #[test]
        fn DEC_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x20;
            let cycles = cpu.execute_instruction(&mut bus, 0x3D); // DEC A

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0x1F);
        }

        mod shift {
            use super::*;

            #[test]
            fn RLCA() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;

                let cycles = cpu.execute_instruction(&mut bus, 0x07); // RLCA

                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.a, 0b0010_1101);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn RRCA() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;

                let cycles = cpu.execute_instruction(&mut bus, 0x0F); // RRCA

                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.a, 0b0100_1011);
            }

            #[test]
            fn RLA() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1000_0000;
                cpu.registers.flags.c = false;

                let cycles = cpu.execute_instruction(&mut bus, 0x17); // RLA

                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.a, 0b0000_0000);
                assert!(!cpu.registers.flags.z); // RLA does not set the Z flag when a is 0
                assert!(cpu.registers.flags.c);
            }
        }
    }

    mod alu16 {
        use super::*;

        #[test]
        fn ADD_HL_BC() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x1234);
            cpu.registers.bc_set(0x1111);
            let cycles = cpu.execute_instruction(&mut bus, 0x09); // ADD HL BC

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.hl_get(), 0x2345);
        }

        #[test]
        fn ADD_HL_HL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x1234);
            let cycles = cpu.execute_instruction(&mut bus, 0x29); // ADD HL HL

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.hl_get(), 0x2468);
        }

        #[test]
        fn ADD_HL_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x1234);
            cpu.sp = 0x1111;
            let cycles = cpu.execute_instruction(&mut bus, 0x39); // ADD HL SP

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.hl_get(), 0x2345);
        }

        #[test]
        fn ADD_SPpE8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0x0008;
            cpu.pc = 0x1234;
            bus.write(cpu.pc, (-0x0A_i8).cast_unsigned());

            let cycles = cpu.execute_instruction(&mut bus, 0xE8); // ADD SP E8

            assert_eq!(cycles, 16);
            assert_eq!(cpu.pc, 0x1235); // moved over e8
            assert_eq!(cpu.sp, 0xFFFE);
        }

        #[test]
        fn LD_HL_SPpE8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0xFFF8;
            cpu.pc = 0x1234;
            bus.write(cpu.pc, 0x0A);

            let cycles = cpu.execute_instruction(&mut bus, 0xF8); // LD HL SP+E8

            assert_eq!(cycles, 12);
            assert_eq!(cpu.pc, 0x1235); // moved over e8
            assert_eq!(cpu.sp, 0xFFF8);
            assert_eq!(cpu.registers.hl_get(), 0x0002);
        }

        #[test]
        fn INC_HL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            let cycles = cpu.execute_instruction(&mut bus, 0x23); // INC HL

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.hl_get(), 0x1235);
        }

        #[test]
        fn INC_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0x12FF;
            let cycles = cpu.execute_instruction(&mut bus, 0x33); // INC SP

            assert_eq!(cycles, 8);
            assert_eq!(cpu.sp, 0x1300);
        }

        #[test]
        fn DEC_BC() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x12;
            cpu.registers.c = 0x00;
            let cycles = cpu.execute_instruction(&mut bus, 0x0B); // DEC BC

            assert_eq!(cycles, 8);
            assert_eq!(cpu.registers.bc_get(), 0x11FF);
        }

        #[test]
        fn DEC_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0x1234;
            let cycles = cpu.execute_instruction(&mut bus, 0x3B); // DEC SP

            assert_eq!(cycles, 8);
            assert_eq!(cpu.sp, 0x1233);
        }
    }

    mod stack {
        use super::*;

        #[test]
        fn PUSH_DE() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.de_set(0x1234);
            cpu.sp = 0xFFFE;

            let cycles = cpu.execute_instruction(&mut bus, 0xD5); // PUSH DE

            assert_eq!(cycles, 16);
            assert_eq!(bus.read(0xFFFC), 0x34); // lo - E
            assert_eq!(bus.read(0xFFFD), 0x12); // hi - D
            assert_eq!(cpu.sp, 0xFFFC);
        }

        #[test]
        fn PUSH_AF() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x56;
            cpu.registers.flags.z = true;
            cpu.registers.flags.n = false;
            cpu.registers.flags.h = true;
            cpu.registers.flags.c = false;
            cpu.sp = 0xFFF6;

            let cycles = cpu.execute_instruction(&mut bus, 0xF5); // PUSH AF

            assert_eq!(cycles, 16);
            assert_eq!(bus.read(0xFFF4), 0b1010_0000); // lo - F
            assert_eq!(bus.read(0xFFF5), 0x56); // hi - A
            assert_eq!(cpu.sp, 0xFFF4);
        }

        #[test]
        fn POP_BC() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0xFFFC;
            bus.write(0xFFFC, 0x34); // lo - C
            bus.write(0xFFFD, 0x12); // hi - B

            let cycles = cpu.execute_instruction(&mut bus, 0xC1); // POP BC

            assert_eq!(cycles, 12);
            assert_eq!(cpu.registers.bc_get(), 0x1234);
            assert_eq!(cpu.sp, 0xFFFE); // sp should have moved up by 2
        }

        #[test]
        fn POP_HL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0xFFF2;
            bus.write(0xFFF2, 0x34); // lo - L
            bus.write(0xFFF3, 0x12); // hi - H

            let cycles = cpu.execute_instruction(&mut bus, 0xE1); // POP HL

            assert_eq!(cycles, 12);
            assert_eq!(cpu.registers.hl_get(), 0x1234);
            assert_eq!(cpu.sp, 0xFFF4); // sp should have moved up by 2
        }
    }

    mod jump {
        use super::*;

        mod jr {
            use super::*;

            #[test]
            fn JR_e8() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(0x1234, (-2_i8).cast_unsigned());

                let cycles = cpu.execute_instruction(&mut bus, 0x18); // JR e8

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x1233); // +1 from e8 fetch -2 from jr
            }

            #[test]
            fn JR_cond_Z() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(0x1234, (-0x11_i8).cast_unsigned());

                cpu.registers.flags.z = true;
                let cycles = cpu.execute_instruction(&mut bus, 0x28); // JR Z e8
                // jump should succeed

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x1224); // +1 from e8 fetch -11 from jr

                bus.write(0x1224, 0x20);

                cpu.registers.flags.z = false;
                let cycles = cpu.execute_instruction(&mut bus, 0x28); // JR Z e8
                // jump should fail

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1225); // +1 from e8 fetch
            }

            #[test]
            fn JR_cond_NC() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(0x1234, (-0x11_i8).cast_unsigned());

                cpu.registers.flags.c = true;
                let cycles = cpu.execute_instruction(&mut bus, 0x30); // JR NC e8
                // jump should fail

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1235); // +1 from e8

                bus.write(0x1235, 0x20);

                cpu.registers.flags.c = false;
                let cycles = cpu.execute_instruction(&mut bus, 0x30); // JR NC e8
                // jump should succeed

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x1256); // +1 from e8 fetch + 20 from jr
            }
        }

        mod jp {
            use super::*;

            #[test]
            fn JP_n16() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x2345);

                let cycles = cpu.execute_instruction(&mut bus, 0xC3); // JP N16

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x2345);
            }

            #[test]
            fn JP_HL() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.registers.hl_set(0x4321);

                let cycles = cpu.execute_instruction(&mut bus, 0xE9); // JP HL

                assert_eq!(cycles, 4);
                assert_eq!(cpu.pc, 0x4321);
            }

            #[test]
            fn JP_cond_NZ() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);
                cpu.registers.flags.z = false; // NZ condition met

                let cycles = cpu.execute_instruction(&mut bus, 0xC2); // JP NZ N16

                assert_eq!(cycles, 16); // 12 base + 4 for taken branch
                assert_eq!(cpu.pc, 0x5678);

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.z = true; // NZ condition not met — jump not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xC2); // JP NZ N16

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x1236);
            }

            #[test]
            fn JP_cond_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.c = true; // C condition met
                let cycles = cpu.execute_instruction(&mut bus, 0xDA); // JP C N16

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x5678);

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.c = false; // C condition not met — jump not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xDA); // JP C N16

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x1236);
            }
        }
    }

    mod call_ret {
        use super::*;

        mod call {
            use super::*;

            #[test]
            fn CALL_n16() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                bus.write_word(cpu.pc, 0x5678);

                let cycles = cpu.execute_instruction(&mut bus, 0xCD); // CALL N16

                assert_eq!(cycles, 24);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFC);
                assert_eq!(cpu.stack().peek_word(&bus), 0x1236); // return address is pc after N16 fetch
            }

            #[test]
            fn CALL_cond_Z() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.z = true; // Z condition met
                let cycles = cpu.execute_instruction(&mut bus, 0xCC); // CALL Z N16

                assert_eq!(cycles, 24); // 12 base + 12 for taken branch
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFC); // stack + 2 in size
                assert_eq!(cpu.stack().peek_word(&bus), 0x1236); // old pc after N16

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                cpu.registers.flags.z = false; // Z condition not met — call not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xCC); // CALL Z N16

                assert_eq!(cycles, 12); // 12 base
                assert_eq!(cpu.pc, 0x1236); // pc after N16
                assert_eq!(cpu.sp, 0xFFFE); // stack unchanged
            }

            #[test]
            fn CALL_cond_NC() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.c = false; // NC condition met
                let cycles = cpu.execute_instruction(&mut bus, 0xD4); // CALL NC N16

                assert_eq!(cycles, 24); // 12 base + 12 for taken branch
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFC); // stack + 2 in size
                assert_eq!(cpu.stack().peek_word(&bus), 0x1236); // old pc after N16

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                cpu.registers.flags.c = true; // NC condition not met — call not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xD4); // CALL NC N16

                assert_eq!(cycles, 12); // 12 base
                assert_eq!(cpu.pc, 0x1236); // pc after N16
                assert_eq!(cpu.sp, 0xFFFE); // stack unchanged
            }
        }

        mod ret {
            use super::*;

            #[test]
            fn RET() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678); // fake return address

                let cycles = cpu.execute_instruction(&mut bus, 0xC9); // RET

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);
            }

            #[test]
            fn RETI() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678);

                let cycles = cpu.execute_instruction(&mut bus, 0xD9); // RETI

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);
                assert!(matches!(cpu.ime, IME::Enabled));
            }

            #[test]
            fn RET_cond_NZ() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678);

                cpu.registers.flags.z = false; // NZ condition met
                let cycles = cpu.execute_instruction(&mut bus, 0xC0); // RET NZ

                assert_eq!(cycles, 20); // 8 base + 12 for taken branch
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFC;

                cpu.registers.flags.z = true; // NZ condition not met — ret not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xC0); // RET NZ

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1234);
                assert_eq!(cpu.sp, 0xFFFC); // stack unchanged
            }

            #[test]
            fn RET_cond_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678);

                cpu.registers.flags.c = true; // C condition met
                let cycles = cpu.execute_instruction(&mut bus, 0xD8); // RET C

                assert_eq!(cycles, 20);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFC;

                cpu.registers.flags.c = false; // C condition not met — ret not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xD8); // RET C

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1234);
                assert_eq!(cpu.sp, 0xFFFC);
            }
        }

        mod rst {
            use super::*;

            #[test]
            fn RST_00() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                let cycles = cpu.execute_instruction(&mut bus, 0xC7); // RST $00

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x0000);
                assert_eq!(cpu.sp, 0xFFFC);
                assert_eq!(cpu.stack().peek_word(&bus), 0x1234);
            }

            #[test]
            fn RST_18() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                let cycles = cpu.execute_instruction(&mut bus, 0xDF); // RST $18

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x0018);
                assert_eq!(cpu.sp, 0xFFFC);
                assert_eq!(cpu.stack().peek_word(&bus), 0x1234);
            }

            #[test]
            fn RST_30() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                let cycles = cpu.execute_instruction(&mut bus, 0xF7); // RST $30

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x0030);
                assert_eq!(cpu.sp, 0xFFFC);
                assert_eq!(cpu.stack().peek_word(&bus), 0x1234);
            }

            #[test]
            fn RST_38() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                let cycles = cpu.execute_instruction(&mut bus, 0xFF); // RST $38

                assert_eq!(cycles, 16);
                assert_eq!(cpu.pc, 0x0038);
                assert_eq!(cpu.sp, 0xFFFC);
                assert_eq!(cpu.stack().peek_word(&bus), 0x1234);
            }
        }
    }

    mod misc {
        use super::*;

        #[test]
        fn NOP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0x00);

            assert_eq!(cycles, 4);
        }

        #[test]
        fn DAA() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xF3;
            cpu.registers.flags.h = true;

            let cycles = cpu.execute_instruction(&mut bus, 0x27); // DAA

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0x59); // 0xF3 +0x66
            assert!(cpu.registers.flags.c);
            assert!(!cpu.registers.flags.h);
        }

        #[test]
        fn CPL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b1001_0110;

            let cycles = cpu.execute_instruction(&mut bus, 0x2F); // CPL

            assert_eq!(cycles, 4);
            assert_eq!(cpu.registers.a, 0b0110_1001); // a's complement
        }

        #[test]
        fn SCF() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0x37); // SCF

            assert_eq!(cycles, 4);
            assert!(cpu.registers.flags.c); // carry as been set
        }

        #[test]
        fn CCF() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0x3F); // CCF

            assert_eq!(cycles, 4);
            assert!(cpu.registers.flags.c); // carry is flipped to true

            let cycles = cpu.execute_instruction(&mut bus, 0x3F); // CCF

            assert_eq!(cycles, 4);
            assert!(!cpu.registers.flags.c); // carry is flipped to false
        }

        #[test]
        #[ignore = "requires interrupt handling -- not yet implemented"]
        fn STOP() {
            // STOP 0x10
            todo!()
        }

        #[test]
        fn HALT() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.ime.enable_now();

            let cycles = cpu.execute_instruction(&mut bus, 0x76); // HALT

            assert_eq!(cycles, 4);
            assert!(matches!(cpu.state, CPUState::Halted));
        }

        #[test]
        fn EI() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0xFB); // EI

            assert_eq!(cycles, 4);
            assert!(matches!(cpu.ime, IME::PendingEnable(1)));
        }

        #[test]
        fn DI() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.ime = IME::Enabled; // start with interrupts enabled

            let cycles = cpu.execute_instruction(&mut bus, 0xF3); // DI

            assert_eq!(cycles, 4);
            assert!(matches!(cpu.ime, IME::Disabled));
        }

        #[test]
        fn invalid() {
            let invalid_opcodes = [
                0xD3u8, 0xE3, 0xE4, 0xF4, 0xDB, 0xEB, 0xEC, 0xFC, 0xDD, 0xED, 0xFD,
            ];

            assert!(invalid_opcodes.iter().all(|&opcode| {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    cpu.execute_instruction(&mut bus, opcode)
                }))
                .is_err()
            }));
        }
    }

    mod ext {
        use super::*;

        mod prefix {
            use super::*;

            #[test]
            fn RL_L() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(cpu.pc, 0x15); // RL L

                cpu.registers.l = 0b0001_0110;
                cpu.registers.flags.c = true;

                let cycles = cpu.execute_instruction(&mut bus, 0xCB); // PREFIX

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1235); // moved over prefix param
                assert_eq!(cpu.registers.l, 0b0010_1101);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn BIT_2_indHL() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(cpu.pc, 0x56); // BIT 2 [HL]

                cpu.registers.h = 0x23;
                cpu.registers.l = 0x45;
                bus.write(0x2345, 0b0000_0100);
                cpu.registers.flags.z = true;

                let cycles = cpu.execute_instruction(&mut bus, 0xCB); // PREFIX

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x1235);
                assert!(!cpu.registers.flags.z);
            }

            #[test]
            fn RES_3_A() {
                // RES 3 A
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(cpu.pc, 0x9F); // RES 3 A

                cpu.registers.a = 0b1111_1111;

                let cycles = cpu.execute_instruction(&mut bus, 0xCB); // PREFIX

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1235); // moved over prefix param
                assert_eq!(cpu.registers.a, 0b1111_0111);
            }

            #[test]
            fn SET_6_E() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(cpu.pc, 0xF3); // SET 6 E

                cpu.registers.e = 0b1000_0000;

                let cycles = cpu.execute_instruction(&mut bus, 0xCB); // PREFIX

                assert_eq!(cycles, 8);
                assert_eq!(cpu.pc, 0x1235); // moved over prefix param
                assert_eq!(cpu.registers.e, 0b1100_0000);
            }
        }

        mod shift {
            use super::*;

            #[test]
            fn RLC_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x01); // RLC C

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.c, 0b0010_1101);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn RRC_D() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x0A); // RRC D

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.d, 0b0100_1011);
            }

            #[test]
            fn RL_E() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.e = 0b1001_0110;
                cpu.registers.flags.c = true;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x13); // RL E

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.e, 0b0010_1101);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn RR_H() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1001_0110;
                cpu.registers.flags.c = true;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x1C); // RR H

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.h, 0b1100_1011);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn SLA_L() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x25); // SLA L

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.l, 0b0010_1100);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn SRA_indHL() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0x12;
                cpu.registers.l = 0x34;
                bus.write(0x1234, 0b1001_0110);

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x2E); // SRA [HL]

                assert_eq!(cycles, 16);
                assert_eq!(bus.read(0x1234), 0b1100_1011); // bit 7 should remain unchanged
            }

            #[test]
            fn SWAP_B() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.b = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x30); // SWAP B

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.b, 0b0110_1001);
            }

            #[test]
            fn SRL_A() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                let cycles = cpu.execute_extended_instruction(&mut bus, 0x3F);

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.a, 0b0100_1011);
            }
        }

        mod bit {
            use super::*;

            #[test]
            fn BIT_0_B() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.b = 0b1111_1110;
                cpu.registers.flags.z = false;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x40); // BIT 0 B

                assert_eq!(cycles, 8);
                assert!(cpu.registers.flags.z);
            }

            #[test]
            fn BIT_3_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0b0000_1000;
                cpu.registers.flags.z = true;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x59); // BIT 3 C

                assert_eq!(cycles, 8);
                assert!(!cpu.registers.flags.z);
            }

            #[test]
            fn BIT_6_D() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1011_1111;
                cpu.registers.flags.z = false;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x72); // BIT 6 D

                assert_eq!(cycles, 8);
                assert!(cpu.registers.flags.z);
            }
        }

        mod res {
            use super::*;

            #[test]
            fn RES_1_E() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.e = 0b1111_1111;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x8B); // RES 1 E

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.e, 0b1111_1101);
            }

            #[test]
            fn RES_4_H() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1111_1111;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xA4); // RES 4 H

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.h, 0b1110_1111);
            }

            #[test]
            fn RES_7_L() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1111_1111;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xBD); // RES 7 L

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.l, 0b0111_1111);
            }
        }

        mod set {
            use super::*;

            #[test]
            fn SET_2_indHL() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0x12;
                cpu.registers.l = 0x34;
                bus.write(0x1234, 0b0000_0000);

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xD6); // SET 2 [HL]

                assert_eq!(cycles, 16);
                assert_eq!(bus.read(0x1234), 0b0000_0100);
            }

            #[test]
            fn SET_5_A() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b0000_0000;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xEF); // SET 5 A

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.a, 0b0010_0000);
            }
        }
    }
}
