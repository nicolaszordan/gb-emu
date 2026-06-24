use crate::cpu::CPU;
use crate::cycles::TCycles;
use crate::interrupts::InterruptBus;

use super::Opcode;
use super::condition::Condition;

#[allow(clippy::wildcard_imports)] // we need and use all the params
use super::parameter::*;

use emu::MemoryBus;

impl CPU {
    /// Dispatch and execute an instruction from the main instruction table
    /// designated by `opcode`.
    #[allow(clippy::too_many_lines)] // matching all values from an u8 is bound to take more than 100 lines
    pub(crate) fn execute_instruction<B: MemoryBus + InterruptBus>(
        &mut self,
        bus: &mut B,
        opcode: Opcode,
    ) -> TCycles {
        let additional_cycles = match opcode.value() {
            // NOP
            0x00 => TCycles::ZERO,

            // LD R16 N16
            0x01 => self.instr_ld16(bus, R16Param::BC.into(), LD16SrcParam::N16),
            0x11 => self.instr_ld16(bus, R16Param::DE.into(), LD16SrcParam::N16),
            0x21 => self.instr_ld16(bus, R16Param::HL.into(), LD16SrcParam::N16),
            0x31 => self.instr_ld16(bus, R16Param::SP.into(), LD16SrcParam::N16),

            // LD *R16 A
            0x02 => self.instr_ld8(bus, R16MemParam::IndBC.into(), R8Param::A.into()),
            0x12 => self.instr_ld8(bus, R16MemParam::IndDE.into(), R8Param::A.into()),
            0x22 => self.instr_ld8(bus, R16MemParam::IndHLi.into(), R8Param::A.into()),
            0x32 => self.instr_ld8(bus, R16MemParam::IndHLd.into(), R8Param::A.into()),

            // LD A *R16
            0x0A => self.instr_ld8(bus, R8Param::A.into(), R16MemParam::IndBC.into()),
            0x1A => self.instr_ld8(bus, R8Param::A.into(), R16MemParam::IndDE.into()),
            0x2A => self.instr_ld8(bus, R8Param::A.into(), R16MemParam::IndHLi.into()),
            0x3A => self.instr_ld8(bus, R8Param::A.into(), R16MemParam::IndHLd.into()),

            // INC R16
            0x03 => self.instr_inc16(bus, R16Param::BC),
            0x13 => self.instr_inc16(bus, R16Param::DE),
            0x23 => self.instr_inc16(bus, R16Param::HL),
            0x33 => self.instr_inc16(bus, R16Param::SP),

            // DEC R16
            0x0B => self.instr_dec16(bus, R16Param::BC),
            0x1B => self.instr_dec16(bus, R16Param::DE),
            0x2B => self.instr_dec16(bus, R16Param::HL),
            0x3B => self.instr_dec16(bus, R16Param::SP),

            // INC R8
            0x04 => self.instr_inc8(bus, R8Param::B),
            0x0C => self.instr_inc8(bus, R8Param::C),
            0x14 => self.instr_inc8(bus, R8Param::D),
            0x1C => self.instr_inc8(bus, R8Param::E),
            0x24 => self.instr_inc8(bus, R8Param::H),
            0x2C => self.instr_inc8(bus, R8Param::L),
            0x34 => self.instr_inc8(bus, R8Param::IndHL),
            0x3C => self.instr_inc8(bus, R8Param::A),

            // DEC R8
            0x05 => self.instr_dec8(bus, R8Param::B),
            0x0D => self.instr_dec8(bus, R8Param::C),
            0x15 => self.instr_dec8(bus, R8Param::D),
            0x1D => self.instr_dec8(bus, R8Param::E),
            0x25 => self.instr_dec8(bus, R8Param::H),
            0x2D => self.instr_dec8(bus, R8Param::L),
            0x35 => self.instr_dec8(bus, R8Param::IndHL),
            0x3D => self.instr_dec8(bus, R8Param::A),

            // LD R8 N8
            0x06 => self.instr_ld8(bus, R8Param::B.into(), LD8SrcParam::N8),
            0x0E => self.instr_ld8(bus, R8Param::C.into(), LD8SrcParam::N8),
            0x16 => self.instr_ld8(bus, R8Param::D.into(), LD8SrcParam::N8),
            0x1E => self.instr_ld8(bus, R8Param::E.into(), LD8SrcParam::N8),
            0x26 => self.instr_ld8(bus, R8Param::H.into(), LD8SrcParam::N8),
            0x2E => self.instr_ld8(bus, R8Param::L.into(), LD8SrcParam::N8),
            0x36 => self.instr_ld8(bus, R8Param::IndHL.into(), LD8SrcParam::N8),
            0x3E => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::N8),

            // ROTATE A
            0x07 => self.instr_rotate_acc(bus, BitRotateOperation::RLC),
            0x0F => self.instr_rotate_acc(bus, BitRotateOperation::RRC),
            0x17 => self.instr_rotate_acc(bus, BitRotateOperation::RL),
            0x1F => self.instr_rotate_acc(bus, BitRotateOperation::RR),

            0x08 => self.instr_ld16(bus, LD16DstParam::IndN16, LD16SrcParam::SP),

            // ADD R16
            0x09 => self.instr_add16(bus, R16Param::BC),
            0x19 => self.instr_add16(bus, R16Param::DE),
            0x29 => self.instr_add16(bus, R16Param::HL),
            0x39 => self.instr_add16(bus, R16Param::SP),

            0x10 => self.instr_stop(),

            // JMP COND? PC+E8
            0x18 => self.instr_jump(bus, JumpParam::PCE8),
            0x20 => self.instr_cond_jump(bus, Condition::NZ, JumpParam::PCE8),
            0x28 => self.instr_cond_jump(bus, Condition::Z, JumpParam::PCE8),
            0x30 => self.instr_cond_jump(bus, Condition::NC, JumpParam::PCE8),
            0x38 => self.instr_cond_jump(bus, Condition::C, JumpParam::PCE8),

            0x27 => self.instr_daa(),
            0x2F => self.instr_cpl(),
            0x37 => self.instr_scf(),
            0x3F => self.instr_ccf(),

            0x76 => self.instr_halt(bus),

            // LD R8 R8
            // NOTE: pretty big block (64instr): should we split this block like we did the others?
            // NOTE: 0x76 halt needs to be before this block as it is contained in the range 0x40..=0x7F
            0x40..=0x7F => self.instr_ld8(
                bus,
                R8Param::from_low_bits(opcode.value() >> 3).into(), // dst r8 param is encoded in bits 5-3
                R8Param::from_low_bits(opcode.value()).into(),
            ),

            // ALU OP R8
            // NOTE: pretty big block (64instr): should we split this block like we did the others?
            0x80..=0xBF => self.instr_alu(
                bus,
                ALUOperation::from_low_bits(opcode.value() >> 3), // alu operation is encoded in bits 5-3
                R8Param::from_low_bits(opcode.value()).into(),
            ),

            // RET COND?
            0xC9 => self.instr_ret(bus),
            0xC0 => self.instr_cond_ret(bus, Condition::NZ),
            0xC8 => self.instr_cond_ret(bus, Condition::Z),
            0xD0 => self.instr_cond_ret(bus, Condition::NC),
            0xD8 => self.instr_cond_ret(bus, Condition::C),

            // POP R16
            0xC1 => self.instr_pop(bus, R16StackParam::BC),
            0xD1 => self.instr_pop(bus, R16StackParam::DE),
            0xE1 => self.instr_pop(bus, R16StackParam::HL),
            0xF1 => self.instr_pop(bus, R16StackParam::AF),

            // JMP COND? N16
            0xC3 => self.instr_jump(bus, JumpParam::N16),
            0xC2 => self.instr_cond_jump(bus, Condition::NZ, JumpParam::N16),
            0xCA => self.instr_cond_jump(bus, Condition::Z, JumpParam::N16),
            0xD2 => self.instr_cond_jump(bus, Condition::NC, JumpParam::N16),
            0xDA => self.instr_cond_jump(bus, Condition::C, JumpParam::N16),

            // CALL COND? N16
            0xCD => self.instr_call(bus, CallParam::N16),
            0xC4 => self.instr_cond_call(bus, Condition::NZ, CallParam::N16),
            0xCC => self.instr_cond_call(bus, Condition::Z, CallParam::N16),
            0xD4 => self.instr_cond_call(bus, Condition::NC, CallParam::N16),
            0xDC => self.instr_cond_call(bus, Condition::C, CallParam::N16),

            // PUSH R16
            0xC5 => self.instr_push(bus, R16StackParam::BC),
            0xD5 => self.instr_push(bus, R16StackParam::DE),
            0xE5 => self.instr_push(bus, R16StackParam::HL),
            0xF5 => self.instr_push(bus, R16StackParam::AF),

            // ALU OP N8
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => self.instr_alu(
                bus,
                ALUOperation::from_low_bits(opcode.value() >> 3), // alu operation is encoded in bits 5-3
                ALU8Param::N8,
            ),

            // CALL VEC
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                self.instr_call(bus, CallParam::VEC(u16::from(opcode.value()) & 0b0011_1000))
            }

            // LD HMEM A
            0xE0 => self.instr_ld8(bus, LD8DstParam::IndHighMemA8, R8Param::A.into()),
            0xE2 => self.instr_ld8(bus, LD8DstParam::IndHighMemC, R8Param::A.into()),
            0xF0 => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::IndHighMemA8),
            0xF2 => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::IndHighMemC),

            // LD *N16 A
            0xEA => self.instr_ld8(bus, LD8DstParam::IndN16, R8Param::A.into()),
            0xFA => self.instr_ld8(bus, R8Param::A.into(), LD8SrcParam::IndN16),

            // EI/DI
            0xF3 => self.instr_di(),
            0xFB => self.instr_ei(),

            // ADD SP E8
            0xE8 => self.instr_add_spe8(bus, AddSPe8DstParam::SP),
            0xF8 => self.instr_add_spe8(bus, AddSPe8DstParam::HL),

            0xCB => self.instr_prefix(bus),
            0xD9 => self.instr_reti(bus),
            0xE9 => self.instr_jump(bus, JumpParam::HL),
            0xF9 => self.instr_ld16(bus, R16Param::SP.into(), LD16SrcParam::HL),

            // INVALID
            0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => {
                // TODO: this  panic! will be changed when we implement CPU states to enter a "bricked" state instead of panicking.
                panic!("Invalid opcode: 0x{:02X}", opcode.value());
            }
        };

        opcode.meta().cycles + additional_cycles
    }

    /// Dispatch and execute an instruction from the extended instruction table
    /// designated by `opcode`.
    pub(crate) fn execute_extended_instruction<M: MemoryBus>(
        &mut self,
        mem_bus: &mut M,
        opcode: Opcode,
    ) -> TCycles {
        // NOTE: all extended hanlders return 0 => we're ignoring their return value and just return the cycles from the meta table.
        match opcode.value() {
            0x00..=0x3F => self.instr_ext_bit_shift(
                mem_bus,
                BitShiftOperation::from_low_bits(opcode.value() >> 3),
                R8Param::from_low_bits(opcode.value()),
            ),

            0x40..=0x7F => self.instr_ext_bit(
                mem_bus,
                BitIndex::from_low_bits(opcode.value() >> 3),
                R8Param::from_low_bits(opcode.value()),
            ),

            0x80..=0xBF => self.instr_ext_res(
                mem_bus,
                BitIndex::from_low_bits(opcode.value() >> 3),
                R8Param::from_low_bits(opcode.value()),
            ),

            0xC0..=0xFF => self.instr_ext_set(
                mem_bus,
                BitIndex::from_low_bits(opcode.value() >> 3),
                R8Param::from_low_bits(opcode.value()),
            ),
        };

        opcode.ext_meta().cycles
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
            let cycles = cpu.execute_instruction(&mut bus, 0x41.into()); // LD B C

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.b, 0x01);
            assert_eq!(cpu.registers.c, 0x01);
        }

        #[test]
        fn LD_C_D() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.c = 0xFF;
            cpu.registers.d = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x4A.into()); // LD C D

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.c, 0x01);
            assert_eq!(cpu.registers.d, 0x01);
        }

        #[test]
        fn LD_D_E() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.d = 0xFF;
            cpu.registers.e = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x53.into()); // LD D E

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.d, 0x01);
            assert_eq!(cpu.registers.e, 0x01);
        }

        #[test]
        fn LD_E_H() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.e = 0xFF;
            cpu.registers.h = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x5C.into()); // LD E H

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.e, 0x01);
            assert_eq!(cpu.registers.h, 0x01);
        }

        #[test]
        fn LD_H_L() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0xFF;
            cpu.registers.l = 0x01;
            let cycles = cpu.execute_instruction(&mut bus, 0x65.into()); // LD H L

            assert_eq!(cycles, 4.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x6E.into()); // LD L [HL]

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x77.into()); // LD [HL] A

            assert_eq!(cycles, 8.into());
            assert_eq!(bus.read(0x1234), 0x01);
            assert_eq!(cpu.registers.a, 0x01);
        }

        #[test]
        fn LD_A_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xFF;
            let cycles = cpu.execute_instruction(&mut bus, 0x7F.into()); // LD A A

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0xFF);
        }

        #[test]
        fn LD_E_n8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.e = 0x12;
            cpu.pc = 0x0034;
            bus.write(0x0034, 0x56);
            let cycles = cpu.execute_instruction(&mut bus, 0x1E.into()); // LD E n8

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x36.into()); // LD [HL] n8

            assert_eq!(cycles, 12.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x0A.into()); // LD A [BC]

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x1A.into()); // LD A [DE]

            assert_eq!(cycles, 8.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x2A.into()); // LD A [HL+]

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x12.into()); // LD [DE] A

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x32.into()); // LD [HL-] A

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0xE0.into()); // LD [0xFF00 + n8] A

            assert_eq!(cycles, 12.into());
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
                let cycles = cpu.execute_instruction(&mut bus, 0xF2.into()); // LD A [0xFF00 + C]

                assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0xFA.into()); // LD A [N16]

            assert_eq!(cycles, 16.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0xEA.into()); // LD [N16] A

            assert_eq!(cycles, 16.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x01.into()); // LD BC n16

            assert_eq!(cycles, 12.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x21.into()); // LD HL n16

            assert_eq!(cycles, 12.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x31.into()); // LD SP n16

            assert_eq!(cycles, 12.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x08.into()); // LD [N16] SP

            assert_eq!(cycles, 20.into());
            assert_eq!(bus.read_word(0x5678), 0x9ABC);
            assert_eq!(cpu.sp, 0x9ABC);
            assert_eq!(cpu.pc, 0x1236);
        }

        #[test]
        fn LD_SP_HL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x9ABC);

            let cycles = cpu.execute_instruction(&mut bus, 0xF9.into()); // LD SP HL

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x80.into()); // ADD B

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0x00);
        }

        #[test]
        fn ADC_C() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x00;
            cpu.registers.c = 0x0F;
            cpu.registers.flags.c = true;
            let cycles = cpu.execute_instruction(&mut bus, 0x89.into()); // ADC C

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0x10);
        }

        #[test]
        fn SUB_D() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x00;
            cpu.registers.d = 0x0F;
            let cycles = cpu.execute_instruction(&mut bus, 0x92.into()); // SUB D

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0xF1);
        }

        #[test]
        fn SBC_E() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x00;
            cpu.registers.e = 0x0F;
            cpu.registers.flags.c = true;
            let cycles = cpu.execute_instruction(&mut bus, 0x9B.into()); // SBC E

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0xF0);
        }

        #[test]
        fn AND_H() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.h = 0b0000_1111;
            let cycles = cpu.execute_instruction(&mut bus, 0xA4.into()); // AND H

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0b0000_1100);
        }

        #[test]
        fn XOR_L() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.l = 0b0000_1111;
            let cycles = cpu.execute_instruction(&mut bus, 0xAD.into()); // XOR L

            assert_eq!(cycles, 4.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0xB6.into()); // OR [HL]

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.registers.a, 0b0011_1111);
        }

        #[test]
        fn CP_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            let cycles = cpu.execute_instruction(&mut bus, 0xBF.into()); // CP A

            assert_eq!(cycles, 4.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0xC6.into()); // ADD N8

            assert_eq!(cycles, 8.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0xEE.into()); // XOR N8

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.pc, 0x1235);
            assert_eq!(cpu.registers.a, 0b0011_0011);
        }

        #[test]
        fn INC_B() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0xFF;
            let cycles = cpu.execute_instruction(&mut bus, 0x04.into()); // INC B

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.b, 0x00);
        }

        #[test]
        fn INC_L() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.l = 0xF0;
            let cycles = cpu.execute_instruction(&mut bus, 0x2C.into()); // INC L

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.l, 0xF1);
        }

        #[test]
        fn DEC_H() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0xFF;
            let cycles = cpu.execute_instruction(&mut bus, 0x25.into()); // DEC H

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.h, 0xFE);
        }

        #[test]
        fn DEC_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x20;
            let cycles = cpu.execute_instruction(&mut bus, 0x3D.into()); // DEC A

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0x1F);
        }

        mod shift {
            use super::*;

            #[test]
            fn RLCA() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;

                let cycles = cpu.execute_instruction(&mut bus, 0x07.into()); // RLCA

                assert_eq!(cycles, 4.into());
                assert_eq!(cpu.registers.a, 0b0010_1101);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn RRCA() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;

                let cycles = cpu.execute_instruction(&mut bus, 0x0F.into()); // RRCA

                assert_eq!(cycles, 4.into());
                assert_eq!(cpu.registers.a, 0b0100_1011);
            }

            #[test]
            fn RLA() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1000_0000;
                cpu.registers.flags.c = false;

                let cycles = cpu.execute_instruction(&mut bus, 0x17.into()); // RLA

                assert_eq!(cycles, 4.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x09.into()); // ADD HL BC

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.registers.hl_get(), 0x2345);
        }

        #[test]
        fn ADD_HL_HL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x1234);
            let cycles = cpu.execute_instruction(&mut bus, 0x29.into()); // ADD HL HL

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.registers.hl_get(), 0x2468);
        }

        #[test]
        fn ADD_HL_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.hl_set(0x1234);
            cpu.sp = 0x1111;
            let cycles = cpu.execute_instruction(&mut bus, 0x39.into()); // ADD HL SP

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.registers.hl_get(), 0x2345);
        }

        #[test]
        fn ADD_SPpE8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0x0008;
            cpu.pc = 0x1234;
            bus.write(cpu.pc, (-0x0A_i8).cast_unsigned());

            let cycles = cpu.execute_instruction(&mut bus, 0xE8.into()); // ADD SP E8

            assert_eq!(cycles, 16.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0xF8.into()); // LD HL SP+E8

            assert_eq!(cycles, 12.into());
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
            let cycles = cpu.execute_instruction(&mut bus, 0x23.into()); // INC HL

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.registers.hl_get(), 0x1235);
        }

        #[test]
        fn INC_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0x12FF;
            let cycles = cpu.execute_instruction(&mut bus, 0x33.into()); // INC SP

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.sp, 0x1300);
        }

        #[test]
        fn DEC_BC() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x12;
            cpu.registers.c = 0x00;
            let cycles = cpu.execute_instruction(&mut bus, 0x0B.into()); // DEC BC

            assert_eq!(cycles, 8.into());
            assert_eq!(cpu.registers.bc_get(), 0x11FF);
        }

        #[test]
        fn DEC_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.sp = 0x1234;
            let cycles = cpu.execute_instruction(&mut bus, 0x3B.into()); // DEC SP

            assert_eq!(cycles, 8.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0xD5.into()); // PUSH DE

            assert_eq!(cycles, 16.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0xF5.into()); // PUSH AF

            assert_eq!(cycles, 16.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0xC1.into()); // POP BC

            assert_eq!(cycles, 12.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0xE1.into()); // POP HL

            assert_eq!(cycles, 12.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0x18.into()); // JR e8

                assert_eq!(cycles, 12.into());
                assert_eq!(cpu.pc, 0x1233); // +1 from e8 fetch -2 from jr
            }

            #[test]
            fn JR_cond_Z() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(0x1234, (-0x11_i8).cast_unsigned());

                cpu.registers.flags.z = true;
                let cycles = cpu.execute_instruction(&mut bus, 0x28.into()); // JR Z e8
                // jump should succeed

                assert_eq!(cycles, 12.into());
                assert_eq!(cpu.pc, 0x1224); // +1 from e8 fetch -11 from jr

                bus.write(0x1224, 0x20);

                cpu.registers.flags.z = false;
                let cycles = cpu.execute_instruction(&mut bus, 0x28.into()); // JR Z e8
                // jump should fail

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.pc, 0x1225); // +1 from e8 fetch
            }

            #[test]
            fn JR_cond_NC() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write(0x1234, (-0x11_i8).cast_unsigned());

                cpu.registers.flags.c = true;
                let cycles = cpu.execute_instruction(&mut bus, 0x30.into()); // JR NC e8
                // jump should fail

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.pc, 0x1235); // +1 from e8

                bus.write(0x1235, 0x20);

                cpu.registers.flags.c = false;
                let cycles = cpu.execute_instruction(&mut bus, 0x30.into()); // JR NC e8
                // jump should succeed

                assert_eq!(cycles, 12.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xC3.into()); // JP N16

                assert_eq!(cycles, 16.into());
                assert_eq!(cpu.pc, 0x2345);
            }

            #[test]
            fn JP_HL() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.registers.hl_set(0x4321);

                let cycles = cpu.execute_instruction(&mut bus, 0xE9.into()); // JP HL

                assert_eq!(cycles, 4.into());
                assert_eq!(cpu.pc, 0x4321);
            }

            #[test]
            fn JP_cond_NZ() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);
                cpu.registers.flags.z = false; // NZ condition met

                let cycles = cpu.execute_instruction(&mut bus, 0xC2.into()); // JP NZ N16

                assert_eq!(cycles, 16.into()); // 12 base + 4 for taken branch
                assert_eq!(cpu.pc, 0x5678);

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.z = true; // NZ condition not met — jump not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xC2.into()); // JP NZ N16

                assert_eq!(cycles, 12.into());
                assert_eq!(cpu.pc, 0x1236);
            }

            #[test]
            fn JP_cond_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.c = true; // C condition met
                let cycles = cpu.execute_instruction(&mut bus, 0xDA.into()); // JP C N16

                assert_eq!(cycles, 16.into());
                assert_eq!(cpu.pc, 0x5678);

                cpu.pc = 0x1234;
                bus.write_word(cpu.pc, 0x5678);

                cpu.registers.flags.c = false; // C condition not met — jump not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xDA.into()); // JP C N16

                assert_eq!(cycles, 12.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xCD.into()); // CALL N16

                assert_eq!(cycles, 24.into());
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
                let cycles = cpu.execute_instruction(&mut bus, 0xCC.into()); // CALL Z N16

                assert_eq!(cycles, 24.into()); // 12 base + 12 for taken branch
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFC); // stack + 2 in size
                assert_eq!(cpu.stack().peek_word(&bus), 0x1236); // old pc after N16

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                cpu.registers.flags.z = false; // Z condition not met — call not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xCC.into()); // CALL Z N16

                assert_eq!(cycles, 12.into()); // 12 base
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
                let cycles = cpu.execute_instruction(&mut bus, 0xD4.into()); // CALL NC N16

                assert_eq!(cycles, 24.into()); // 12 base + 12 for taken branch
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFC); // stack + 2 in size
                assert_eq!(cpu.stack().peek_word(&bus), 0x1236); // old pc after N16

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                cpu.registers.flags.c = true; // NC condition not met — call not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xD4.into()); // CALL NC N16

                assert_eq!(cycles, 12.into()); // 12 base
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

                let cycles = cpu.execute_instruction(&mut bus, 0xC9.into()); // RET

                assert_eq!(cycles, 16.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xD9.into()); // RETI

                assert_eq!(cycles, 16.into());
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
                let cycles = cpu.execute_instruction(&mut bus, 0xC0.into()); // RET NZ

                assert_eq!(cycles, 20.into()); // 8 base + 12 for taken branch
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFC;

                cpu.registers.flags.z = true; // NZ condition not met — ret not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xC0.into()); // RET NZ

                assert_eq!(cycles, 8.into());
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
                let cycles = cpu.execute_instruction(&mut bus, 0xD8.into()); // RET C

                assert_eq!(cycles, 20.into());
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFC;

                cpu.registers.flags.c = false; // C condition not met — ret not taken
                let cycles = cpu.execute_instruction(&mut bus, 0xD8.into()); // RET C

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xC7.into()); // RST $00

                assert_eq!(cycles, 16.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xDF.into()); // RST $18

                assert_eq!(cycles, 16.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xF7.into()); // RST $30

                assert_eq!(cycles, 16.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xFF.into()); // RST $38

                assert_eq!(cycles, 16.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x00.into());

            assert_eq!(cycles, 4.into());
        }

        #[test]
        fn DAA() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xF3;
            cpu.registers.flags.h = true;

            let cycles = cpu.execute_instruction(&mut bus, 0x27.into()); // DAA

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0x59); // 0xF3 +0x66
            assert!(cpu.registers.flags.c);
            assert!(!cpu.registers.flags.h);
        }

        #[test]
        fn CPL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b1001_0110;

            let cycles = cpu.execute_instruction(&mut bus, 0x2F.into()); // CPL

            assert_eq!(cycles, 4.into());
            assert_eq!(cpu.registers.a, 0b0110_1001); // a's complement
        }

        #[test]
        fn SCF() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0x37.into()); // SCF

            assert_eq!(cycles, 4.into());
            assert!(cpu.registers.flags.c); // carry as been set
        }

        #[test]
        fn CCF() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0x3F.into()); // CCF

            assert_eq!(cycles, 4.into());
            assert!(cpu.registers.flags.c); // carry is flipped to true

            let cycles = cpu.execute_instruction(&mut bus, 0x3F.into()); // CCF

            assert_eq!(cycles, 4.into());
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

            let cycles = cpu.execute_instruction(&mut bus, 0x76.into()); // HALT

            assert_eq!(cycles, 4.into());
            assert!(matches!(cpu.state, CPUState::Halted));
        }

        #[test]
        fn EI() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            let cycles = cpu.execute_instruction(&mut bus, 0xFB.into()); // EI

            assert_eq!(cycles, 4.into());
            assert!(matches!(cpu.ime, IME::PendingEnable(1)));
        }

        #[test]
        fn DI() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.ime = IME::Enabled; // start with interrupts enabled

            let cycles = cpu.execute_instruction(&mut bus, 0xF3.into()); // DI

            assert_eq!(cycles, 4.into());
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
                    cpu.execute_instruction(&mut bus, Opcode::new(opcode))
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

                let cycles = cpu.execute_instruction(&mut bus, 0xCB.into()); // PREFIX

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xCB.into()); // PREFIX

                assert_eq!(cycles, 12.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xCB.into()); // PREFIX

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_instruction(&mut bus, 0xCB.into()); // PREFIX

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x01.into()); // RLC C

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.c, 0b0010_1101);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn RRC_D() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x0A.into()); // RRC D

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.d, 0b0100_1011);
            }

            #[test]
            fn RL_E() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.e = 0b1001_0110;
                cpu.registers.flags.c = true;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x13.into()); // RL E

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.e, 0b0010_1101);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn RR_H() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1001_0110;
                cpu.registers.flags.c = true;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x1C.into()); // RR H

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.h, 0b1100_1011);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn SLA_L() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x25.into()); // SLA L

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x2E.into()); // SRA [HL]

                assert_eq!(cycles, 16.into());
                assert_eq!(bus.read(0x1234), 0b1100_1011); // bit 7 should remain unchanged
            }

            #[test]
            fn SWAP_B() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.b = 0b1001_0110;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x30.into()); // SWAP B

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.b, 0b0110_1001);
            }

            #[test]
            fn SRL_A() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                let cycles = cpu.execute_extended_instruction(&mut bus, 0x3F.into());

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x40.into()); // BIT 0 B

                assert_eq!(cycles, 8.into());
                assert!(cpu.registers.flags.z);
            }

            #[test]
            fn BIT_3_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0b0000_1000;
                cpu.registers.flags.z = true;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x59.into()); // BIT 3 C

                assert_eq!(cycles, 8.into());
                assert!(!cpu.registers.flags.z);
            }

            #[test]
            fn BIT_6_D() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1011_1111;
                cpu.registers.flags.z = false;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x72.into()); // BIT 6 D

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_extended_instruction(&mut bus, 0x8B.into()); // RES 1 E

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.e, 0b1111_1101);
            }

            #[test]
            fn RES_4_H() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1111_1111;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xA4.into()); // RES 4 H

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.h, 0b1110_1111);
            }

            #[test]
            fn RES_7_L() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1111_1111;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xBD.into()); // RES 7 L

                assert_eq!(cycles, 8.into());
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

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xD6.into()); // SET 2 [HL]

                assert_eq!(cycles, 16.into());
                assert_eq!(bus.read(0x1234), 0b0000_0100);
            }

            #[test]
            fn SET_5_A() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b0000_0000;

                let cycles = cpu.execute_extended_instruction(&mut bus, 0xEF.into()); // SET 5 A

                assert_eq!(cycles, 8.into());
                assert_eq!(cpu.registers.a, 0b0010_0000);
            }
        }
    }
}
