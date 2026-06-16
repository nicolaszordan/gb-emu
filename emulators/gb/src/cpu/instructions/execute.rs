use crate::cpu::CPU;
use crate::cpu::alu;

use crate::interrupts::InterruptBus;

use super::condition::Condition;
use super::operand::{Operand8, Operand16};

#[allow(clippy::wildcard_imports)] // we use all paramaters in this module
use super::parameter::*;

use emu::MemoryBus;

impl CPU {
    /// Load 8 bits from `src` into `dst`.
    ///
    /// The bits are copied, `src` will not change due to this instruction.
    ///
    /// See [`LD8DstParam`] and [`LD8SrcParam`] for possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.b = 0x01;
    /// cpu.registers.c = 0xFF;
    ///
    /// cpu.instr_ld8(&mut bus, R8Param::B.into(), R8Param::C.into()); // equivalent to LD B C
    ///
    /// assert_eq!(cpu.registers.b, 0xFF);
    /// assert_eq!(cpu.registers.c, 0xFF);
    /// ```
    pub(crate) fn instr_ld8<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        dst: LD8DstParam,
        src: LD8SrcParam,
    ) -> u32 {
        let value = Operand8::from(src).read(self, bus);
        Operand8::from(dst).write(self, bus, value);

        0
    }

    /// Load 16 bits from `src` into `dst`.
    ///
    /// See [`LD16DstParam`] and [`LD16SrcParam`] for possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0001;
    /// bus.write_word(0x0001, 0x1234); // next 16 bits after pc
    ///
    /// cpu.instr_ld16(&mut bus, LD16DstParam::BC, LD16SrcParam::N16);
    ///
    /// assert_eq!(cpu.registers.bc_get(), 0x1234);
    /// ```
    pub(crate) fn instr_ld16<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        dst: LD16DstParam,
        src: LD16SrcParam,
    ) -> u32 {
        let value = Operand16::from(src).read(self, bus);
        Operand16::from(dst).write(self, bus, value);

        0
    }

    /// Push `param` into the stack.
    ///
    /// Decrement [`CPU::sp`] and write `param` where [`CPU::sp`] is now pointing.
    ///
    /// See [`R16StackParam`] for all possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.sp = 0xFFFE;
    /// cpu.registers.hl_set(0x1234);
    ///
    /// cpu.instr_push(&mut bus, R16StackParam::HL);
    ///
    /// assert_eq!(cpu.sp, 0xFFFC); // SP is decremented by 2 (word size) and points to the pushed value
    /// assert_eq!(bus.mem[0xFFFC..=0xFFFD], [0x34, 0x12]); // note endian swap
    /// ```
    pub(crate) fn instr_push<M: MemoryBus>(&mut self, bus: &mut M, param: R16StackParam) -> u32 {
        let value = Operand16::from(param).read(self, bus);

        self.stack().push_word(bus, value);

        0
    }

    /// Pop from stack into `param`.
    ///
    /// Read and store into `param` the value currently pointed by [`CPU::sp`]
    /// and then increment [`CPU::sp`] to point the next value.
    ///
    /// See [`R16StackParam`] for all possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.sp = 0xFFFE;
    /// cpu.registers.hl_set(0x1234);
    ///
    /// cpu.instr_push(&mut bus, R16StackParam::HL);
    ///
    /// assert_eq!(cpu.sp, 0xFFFC); // SP is decremented by 2 (word size) and points to the pushed value
    /// assert_eq!(bus.mem[0xFFFC..=0xFFFD], [0x34, 0x12]); // note endian swap
    ///
    /// cpu.instr_pop(&mut bus, R16StackParam::DE);
    ///
    /// assert_eq!(cpu.sp, 0xFFFE); // SP in incremented by 2 and is back where it began
    /// assert_eq!(cpu.registers.de_get(), 0x1234);
    /// ```
    pub(crate) fn instr_pop<M: MemoryBus>(&mut self, bus: &mut M, param: R16StackParam) -> u32 {
        let value = self.stack().pop_word(bus);

        Operand16::from(param).write(self, bus, value);

        0
    }

    /// Perform an unconditional jump according to `param`.
    ///
    /// See [`JumpParam`] for all possible parameters.
    ///
    /// Jumps can either be absolute ([`JumpParam::N16`], [`JumpParam::HL`]) or relative
    /// to the current position of [`CPU::pc`] ([`JumpParam::PCE8`]).
    ///
    /// "Jumping" is performed by setting the value of [`CPU::pc`] to a different
    /// value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// cpu.registers.hl_set(0x1234);
    ///
    /// cpu.instr_jump(&mut bus, JumpParam::HL);
    ///
    /// assert_eq!(cpu.pc, 0x1234); // pc is now at 0x1234 and will continue executing from there
    /// ```
    pub(crate) fn instr_jump<M: MemoryBus>(&mut self, bus: &M, param: JumpParam) -> u32 {
        self.pc = Operand16::from(param).read(self, bus);

        0
    }

    /// Perform a conditional jump if `condition` is valid.
    ///
    /// See [`Condition`] for all possible conditions and [`JumpParam`] for all
    /// possible jump parameters.
    ///
    /// `condition` is checked against the [`CPU::registers`] and if the
    /// condition is valid, a jump to `param` is performed. If the condition
    /// isn't valid, the [`CPU::pc`] proceeds to the next instruction.
    ///
    /// Note that this instruction takes more cycles if the jump is taken.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// bus.write(cpu.pc, -0x11 as i8 as u8);
    ///
    /// cpu.registers.flags.z = false;
    /// let cycles = cpu.instr_cond_jump(&mut bus, Condition::Z, JumpParam::PCE8); // Condition::Z fails
    ///
    /// assert_eq!(cycles, 0); // no extra cycles are taken when the jump isn't performed
    /// assert_eq!(cpu.pc, 0x0101); // pc still moves over the e8 parameter
    /// bus.write(cpu.pc, -0x11 as i8 as u8);
    ///
    /// cpu.registers.flags.c = true;
    /// let cycles = cpu.instr_cond_jump(&mut bus, Condition::C, JumpParam::PCE8); // Condition::C is valid -- jump happens
    ///
    /// assert_eq!(cycles, 4); // extra cycles are taken when the jump is performed
    /// assert_eq!(cpu.pc, 0x00F1); // pc moves to +1(from e8 param) -0x11(from jump)
    /// ```
    pub(crate) fn instr_cond_jump<M: MemoryBus>(
        &mut self,
        bus: &M,
        condition: Condition,
        param: JumpParam,
    ) -> u32 {
        if condition.check(self.registers.flags) {
            self.instr_jump(bus, param) + 4
        } else {
            let _ = Operand16::from(param).read(self, bus); // we read the param to move PC forward in case of E8/N16 params
            0
        }
    }

    /// Perform an unconditional call to `param`.
    ///
    /// See [`CallParam`] for all possible parameters.
    ///
    /// A call consists of pushing the current [`CPU::pc`] into the stack and then
    /// jumping to the address designated by `param`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// bus.write_word(cpu.pc, 0x1234);
    ///
    /// cpu.instr_call(&mut bus, CallParam::N16);
    ///
    /// assert_eq!(cpu.pc, 0x1234);
    /// assert_eq!(cpu.stack().peek_word(&bus), 0x0102); // the value pushed into the stack points to the next instruction after the call (0x0102 in this case)
    /// ```
    pub(crate) fn instr_call<M: MemoryBus>(&mut self, bus: &mut M, param: CallParam) -> u32 {
        let dst = Operand16::from(param).read(self, bus);
        let pc = self.pc;

        self.stack().push_word(bus, pc);

        self.pc = dst;

        0
    }

    /// Perform a conditional call if `condition` is valid.
    ///
    /// See [`Condition`] for all possible conditions and [`CallParam`] for all
    /// possible call parameters.
    ///
    /// `condition` is checked against the [`CPU::registers`] and if the
    /// condition is valid, a call to `param` is performed. If the condition
    /// isn't valid, the [`CPU::pc`] proceeds to the next instruction.
    ///
    /// Note that this instruction takes more cycles if the call is taken.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// bus.write_word(cpu.pc, 0x1234);
    ///
    /// cpu.registers.flags.z = false;
    /// let cycles = cpu.instr_cond_call(&mut bus, Condition::Z, CallParam::N16); // Condition::Z fails
    ///
    /// assert_eq!(cycles, 0); // no extra cycles are taken when the call isn't performed
    /// assert_eq!(cpu.pc, 0x0102); // pc still moves over the n16 parameter
    /// bus.write_word(cpu.pc, 0x2345);
    ///
    /// cpu.registers.flags.z = false;
    /// let cycles = cpu.instr_cond_call(&mut bus, Condition::NZ, CallParam::N16); // Condition::NZ is valid -- call happens
    ///
    /// assert_eq!(cycles, 12); // extra cycles are taken when the call is performed
    /// assert_eq!(cpu.pc, 0x2345);
    /// assert_eq!(cpu.stack().peek_word(&bus), 0x0104); // the value pushed into the stack points to the next instruction after the call (0x0104 in this case)
    /// ```
    pub(crate) fn instr_cond_call<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        condition: Condition,
        param: CallParam,
    ) -> u32 {
        if condition.check(self.registers.flags) {
            self.instr_call(bus, param) + 12
        } else {
            let _ = Operand16::from(param).read(self, bus); // we read the param to move PC forward in case of E8/N16 params
            0
        }
    }

    /// Return from a call.
    ///
    /// The return is performed by popping the return address from the stack
    /// and jumping (ie. setting the [`CPU::pc`]) to it
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// bus.write_word(cpu.pc, 0x1234);
    ///
    /// cpu.instr_call(&mut bus, CALLParam::N16);
    ///
    /// assert_eq!(cpu.pc, 0x1234);
    ///
    /// cpu.instr_ret(&mut bus);
    ///
    /// assert_eq!(cpu.pc, 0x0102); // pc is back to the instruction after the call
    /// ```
    pub(crate) fn instr_ret<M: MemoryBus>(&mut self, bus: &M) -> u32 {
        let dst = self.stack().pop_word(bus);

        self.pc = dst;

        0
    }

    /// Return from a call if `condition` is valid.
    ///
    /// The return is performed by popping the return address from the stack
    /// and jumping (ie. setting the [`CPU::pc`]) to it
    ///
    /// Note that this instruction takes more cycle to perform if the return is
    /// performed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// bus.write_word(cpu.pc, 0x1234);
    /// cpu.instr_call(&mut bus, CALLParam::N16); // 0x0102 is pushed as the return address
    /// assert_eq!(cpu.pc, 0x1234);
    ///
    /// cpu.registers.flags.c = false;
    /// let cycles = cpu.instr_cond_ret(&mut bus, Condition::C); // Condition C fails -- no return
    /// assert_eq!(cycles, 0); // no extra cycles as return didn't happen
    ///
    /// cpu.registers.flags.c = true;
    /// let cycles = cpu.instr_cond_ret(&mut bus, Condition::C); // Condition C is valid -- return
    /// assert_eq!(cycles, 12); // extra cycles from return
    ///
    /// assert_eq!(cpu.pc, 0x0102); // pc is back to the instruction after the call
    /// ```
    pub(crate) fn instr_cond_ret<M: MemoryBus>(&mut self, bus: &M, condition: Condition) -> u32 {
        if condition.check(self.registers.flags) {
            self.instr_ret(bus) + 12
        } else {
            0
        }
    }

    /// Return from a call and enable interrupts.
    ///
    /// The return is performed by popping the return address from the stack
    /// and jumping (ie. setting the [`CPU::pc`]) to it. After that, the IME is
    /// set to enabled to allow interrupts.
    ///
    /// Note that this instruction directly enables IME without the 1
    /// instruction delay that the EI instruction has.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// cpu.ime = IME::Disabled;
    /// bus.write_word(cpu.pc, 0x1234);
    /// cpu.instr_call(&mut bus, CALLParam::N16);
    /// assert_eq!(cpu.pc, 0x1234);
    ///
    /// cpu.instr_reti(&bus);
    ///
    /// assert_eq!(cpu.pc, 0x0102); // pc is back to the instruction after the call
    /// assert!(matches!(cpu.ime, IME::Enabled)); // IME is enabled
    /// ```
    pub(crate) fn instr_reti<M: MemoryBus>(&mut self, bus: &M) -> u32 {
        self.instr_ret(bus);
        self.ime.enable_now();

        0
    }

    /// Increment the 8-bit value designated by `param`.
    ///
    /// This will also set the flags according to the result of the increment, but
    /// will not modify the carry flag.
    ///
    /// See [`alu::inc`] for more details on how the flags are set and
    /// [`R8Param`] for possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.b = 0x2F;
    ///
    /// cpu.instr_inc8(&mut bus, R8Param::B);
    ///
    /// assert_eq!(cpu.registers.b, 0x30);
    /// assert!(!cpu.registers.flags.z);
    /// assert!(!cpu.registers.flags.n);
    /// assert!(cpu.registers.flags.h);
    /// assert!(!cpu.registers.flags.c);
    /// ```
    pub(crate) fn instr_inc8<M: MemoryBus>(&mut self, bus: &mut M, param: R8Param) -> u32 {
        let op = Operand8::from(param);
        let (result, flags) = alu::inc(op.read(self, bus));
        self.copy_flags::<{ flag_mask::Z | flag_mask::N | flag_mask::H }>(&flags);
        op.write(self, bus, result);

        0
    }

    /// Decrement the 8-bit value designated by `param`.
    ///
    /// This will also set the flags according to the result of the decrement, but
    /// will not modify the carry flag.
    ///
    /// See [`alu::dec`] for more details on how the flags are set and
    /// [`R8Param`] for possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.b = 0x20;
    ///
    /// cpu.instr_dec8(&mut bus, R8Param::B);
    ///
    /// assert_eq!(cpu.registers.b, 0x1F);
    /// assert!(!cpu.registers.flags.z);
    /// assert!(cpu.registers.flags.n);
    /// assert!(cpu.registers.flags.h);
    /// assert!(!cpu.registers.flags.c);
    /// ```
    pub(crate) fn instr_dec8<M: MemoryBus>(&mut self, bus: &mut M, param: R8Param) -> u32 {
        let op = Operand8::from(param);
        let (result, flags) = alu::dec(op.read(self, bus));
        self.copy_flags::<{ flag_mask::Z | flag_mask::N | flag_mask::H }>(&flags);
        op.write(self, bus, result);

        0
    }

    /// Increment the 16-bit value designated by `param`.
    ///
    /// This instruction sets no flags and leaves flags as is.
    ///
    /// See [`alu::inc16`] for more details and [`R16Param`] for possible
    /// parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.b = 0x10;
    /// cpu.registers.c = 0x20;
    ///
    /// cpu.registers.d = 0x10;
    /// cpu.registers.e = 0xFF;
    ///
    /// cpu.instr_inc16(&mut bus, R16Param::BC);
    /// cpu.instr_inc16(&mut bus, R16Param::DE);
    ///
    /// assert_eq!(cpu.registers.bc_get(), 0x1021);
    /// assert_eq!(cpu.registers.de_get(), 0x1100);
    /// ```
    pub(crate) fn instr_inc16<M: MemoryBus>(&mut self, bus: &mut M, param: R16Param) -> u32 {
        let op = Operand16::from(param);
        let (result, _) = alu::inc16(op.read(self, bus));
        op.write(self, bus, result);

        0
    }

    /// Decrement the 16-bit value designated by `param`.
    ///
    /// This instruction sets no flags and leaves flags as is.
    ///
    /// See [`alu::dec16`] for more details and [`R16Param`] for possible
    /// parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.b = 0x11;
    /// cpu.registers.c = 0x22;
    ///
    /// cpu.registers.d = 0x00;
    /// cpu.registers.e = 0x00;
    ///
    /// cpu.instr_dec16(&mut bus, R16Param::BC);
    /// cpu.instr_dec16(&mut bus, R16Param::DE);
    ///
    /// assert_eq!(cpu.registers.bc_get(), 0x1121);
    /// assert_eq!(cpu.registers.de_get(), 0xFFFF);
    /// ```
    pub(crate) fn instr_dec16<M: MemoryBus>(&mut self, bus: &mut M, param: R16Param) -> u32 {
        let op = Operand16::from(param);
        let (result, _) = alu::dec16(op.read(self, bus));
        op.write(self, bus, result);

        0
    }

    /// Perform the given `operation` between the value in register A and the value
    /// designated by `param` and store the result in register A.
    ///
    /// The flags are set according to the result of the operation.
    ///
    /// See the documentation of each operation in the module [`alu`] for more
    /// details on how the flags are set, [`ALUOperation`] for possible
    /// operations, and [`ALU8Param`] for possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.a = 0b1010_1010;
    /// cpu.registers.b = 0b0101_1111;
    /// cpu.instr_alu(&mut bus, ALUOperation::AND, ALU8Param::R8::B);
    /// assert_eq!(cpu.registers.a, 0b0000_1010);
    /// assert!(!cpu.registers.flags.z);
    /// assert!(!cpu.registers.flags.n);
    /// assert!(cpu.registers.flags.h);
    /// assert!(!cpu.registers.flags.c);
    /// ```
    pub(crate) fn instr_alu<M: MemoryBus>(
        &mut self,
        bus: &M,
        operation: ALUOperation,
        param: ALU8Param,
    ) -> u32 {
        let a = self.registers.a;
        let b = Operand8::from(param).read(self, bus);

        let (result, flags) = match operation {
            ALUOperation::ADD => alu::add(a, b),
            ALUOperation::ADC => alu::adc(a, b, self.registers.flags.c),
            ALUOperation::SUB => alu::sub(a, b),
            ALUOperation::SBC => alu::sbc(a, b, self.registers.flags.c),
            ALUOperation::AND => alu::and(a, b),
            ALUOperation::XOR => alu::xor(a, b),
            ALUOperation::OR => alu::or(a, b),
            ALUOperation::CP => alu::cp(a, b),
        };

        self.registers.a = result; // we're still assigning cp's result as it just returns a without modifying it, so this is a noop effectively

        self.copy_all_flags(&flags);

        0
    }

    /// Perform a 16-bit addition between the value in HL and the value designated by `param`.
    /// The result is stored in HL.
    ///
    /// See [`alu::add16`] for more details on how the flags are set and
    /// [`R16Param`] for all possible parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.hl_set(0x1000);
    /// cpu.registers.de_set(0x1234);
    ///
    /// cpu.instr_add16(&bus, R16Param::DE);
    ///
    /// assert_eq!(cpu.registers.hl_get(), 0x2234);
    /// ```
    pub(crate) fn instr_add16<M: MemoryBus>(&mut self, bus: &M, param: R16Param) -> u32 {
        let value = Operand16::from(param).read(self, bus);
        let (result, flags) = alu::add16(self.registers.hl_get(), value);
        self.registers.hl_set(result);
        self.copy_flags::<{ flag_mask::N | flag_mask::H | flag_mask::C }>(&flags);

        0
    }

    /// Perform a 16-bit addition between [`CPU::sp`] and next signed 8-bit value.
    ///
    /// The result is stored according to `dst` and the flags are set according
    /// to the result.
    ///
    /// See [`alu::add16_signed`] for more details on how the flags are set and
    /// [`ADDSPE8DstParam`] for all possible destination parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.sp = 0xFFFE;
    /// cpu.pc = 0x0100;
    /// bus.write(cpu.pc, -0x11 as i8 as u8); // e8 parameter
    ///
    /// cpu.instr_add_spe8(&mut bus, ADDSPE8DstParam::HL);
    ///
    /// assert_eq!(cpu.registers.hl_get(), 0xFFED); // 0xFFFE + (-0x11) = 0xFFED
    /// ```
    pub(crate) fn instr_add_spe8<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        dst: AddSPe8DstParam,
    ) -> u32 {
        let e8 = Operand8::E8.read(self, bus).cast_signed();
        let sp = self.sp;
        let (result, flags) = alu::add16_signed(sp, e8);

        Operand16::from(dst).write(self, bus, result);

        self.copy_all_flags(&flags);

        0
    }

    /// Read and execute the next byte from the extended instruction table.
    ///
    /// The extended instruction table contains most bit shift operations and
    /// bit flag instructions.
    ///
    /// See [`bit_shift`](Self::instr_ext_bit_shift), [`bit`](Self::instr_ext_bit),
    /// [`res`](Self::instr_ext_res) and [`set`](Self::instr_ext_set) for all
    /// possible extended instructions.
    ///
    /// To see how they are dispatched check the [`CPU::execute_extended_instruction`]
    /// method.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0100;
    /// cpu.registers.c = 0b1111_1111;
    ///
    /// bus.write(cpu.pc, 0x99); // opcode of RES 3 C
    ///
    /// cpu.instr_prefix(&mut bus);
    ///
    /// assert_eq!(cpu.registers.c, 0b1111_0111); // BIT 3 of C has been reset
    ///
    /// ```
    pub(crate) fn instr_prefix<M: MemoryBus>(&mut self, bus: &mut M) -> u32 {
        let opcode = self.fetch_byte(bus);
        self.execute_extended_instruction(bus, opcode)
    }

    /// Perform the given bit shift `operation` on the given `param`.
    ///
    /// The operation is performed directly on the parameter and the result is
    /// stored back into the given parameter.
    ///
    /// The flags are set depending on the operation.
    ///
    /// See [`BitShiftOperation`] for a list of all possible operations, the
    /// [`alu`] module for their implementation and the way they each set their
    /// flags, and [`R8Param`] for all possible parameters.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.b = 0b0101_1111;
    /// cpu.instr_ext_bit_shift(&mut bus, BitShiftOperation::SWAP, R8Param::B);
    /// assert_eq!(cpu.registers.b, 0b1111_0101);
    /// ```
    pub(crate) fn instr_ext_bit_shift<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        operation: BitShiftOperation,
        param: R8Param,
    ) -> u32 {
        let op = Operand8::from(param);
        let value = op.read(self, bus);

        let (result, flags) = match operation {
            BitShiftOperation::Rotate(op) => match op {
                BitRotateOperation::RLC => alu::rlc(value),
                BitRotateOperation::RRC => alu::rrc(value),
                BitRotateOperation::RL => alu::rl(value, self.registers.flags.c),
                BitRotateOperation::RR => alu::rr(value, self.registers.flags.c),
            },
            BitShiftOperation::SLA => alu::sla(value),
            BitShiftOperation::SRA => alu::sra(value),
            BitShiftOperation::SWAP => alu::swap(value),
            BitShiftOperation::SRL => alu::srl(value),
        };

        op.write(self, bus, result);

        self.copy_all_flags(&flags);

        0
    }

    /// Check the status of the bit at `bit_index` in `param`.
    ///
    /// Flags are set accordingly to the status of the designated bit: the Z
    /// flag is set if the bit equals 0 and cleared if the bit equals 1.
    ///
    /// Note that this instruction has no effects on the register themselves
    /// and only sets flags.
    ///
    /// See [`BitIndex`] to see how to designate a single bit and [`R8Param`]
    /// for all possible parameters.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let bus = Bus::new();
    ///
    /// cpu.registers.b = 0b1101_1111;
    /// cpu.instr_ext_bit(&bus, 5u8.into(), R8Param::B);
    ///
    /// assert!(cpu.registers.flags.z); // BIT 5 of register B is set to 0
    /// ```
    pub(crate) fn instr_ext_bit<M: MemoryBus>(
        &mut self,
        bus: &M,
        bit_index: BitIndex,
        param: R8Param,
    ) -> u32 {
        let op = Operand8::from(param);
        let value = op.read(self, bus);

        let flags = alu::bit(bit_index, value);

        self.copy_flags::<{ flag_mask::Z | flag_mask::N | flag_mask::H }>(&flags);

        0
    }

    /// Reset to 0 the bit at `bit_index` in `param`.
    ///
    /// See [`BitIndex`] to see how to designate a single bit and [`R8Param`]
    /// for all possible parameters.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.e = 0b1101_1111;
    /// cpu.instr_ext_res(&mut bus, 2u8.into(), R8Param::E);
    ///
    /// assert_eq!(cpu.registers.e, 0b1101_1011); // BIT 2 of register E has been set to 0
    /// ```
    pub(crate) fn instr_ext_res<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        bit_index: BitIndex,
        param: R8Param,
    ) -> u32 {
        let op = Operand8::from(param);
        let value = op.read(self, bus);

        let (result, _) = alu::res(bit_index, value);

        op.write(self, bus, result);

        0
    }

    /// Set to 1 the bit at `bit_index` in `param`.
    ///
    /// See [`BitIndex`] to see how to designate a single bit and [`R8Param`]
    /// for all possible parameters.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.l = 0b0001_0000;
    /// cpu.instr_ext_set(&mut bus, 1u8.into(), R8Param::L);
    ///
    /// assert_eq!(cpu.registers.l, 0b0001_0010); // BIT 1 of register L has been set to 1
    /// ```
    pub(crate) fn instr_ext_set<M: MemoryBus>(
        &mut self,
        bus: &mut M,
        bit_index: BitIndex,
        param: R8Param,
    ) -> u32 {
        let op = Operand8::from(param);
        let value = op.read(self, bus);

        let (result, _) = alu::set(bit_index, value);

        op.write(self, bus, result);

        0
    }

    /// Perform a bit rotation on register A.
    ///
    /// See [`BitRotateOperation`] for the available rotations.
    ///
    /// Note that this instruction has a particuliar behaviour compared to the
    /// RLC/RRC/RL/RR instructions from the extended table as this instruction
    /// always sets the zero flag to false regardless of the end value of A.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.registers.a = 0b0001_0000;
    /// cpu.registers.flags.c = true;
    /// cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RL);
    ///
    /// assert_eq!(cpu.registers.a, 0b0010_0001); // a rotated left once with the carry flag
    /// ```
    pub(crate) fn instr_rotate_acc<M: MemoryBus>(
        &mut self,
        bus: &mut M, // bus is required by `instr_ext_bit_shift`'s signature but `instr_rotate_acc` only operates on register A
        operation: BitRotateOperation,
    ) -> u32 {
        self.instr_ext_bit_shift(bus, BitShiftOperation::Rotate(operation), R8Param::A);

        // the RLCA, RRCA, RLA, and RRA instruction always clear the zero flag
        // regardless of the result, whereas the CB-prefixed versions of these
        // instructions set the zero flag according to the result.
        self.registers.flags.z = false;

        0
    }

    /// Complement register A.
    ///
    /// This instruction flips all bits from the A register.
    ///
    /// # Flags
    /// This instruction sets the `N` and `H` flags to **true** and leave the
    /// others as they are.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    ///
    /// cpu.registers.a = 0b1010_1100;
    /// cpu.instr_cpl();
    /// assert_eq!(cpu.registers.a, 0b0101_0011);
    /// cpu.instr_cpl();
    /// assert_eq!(cpu.registers.a, 0b1010_1100); // back where we started
    /// ```
    ///
    pub(crate) const fn instr_cpl(&mut self) -> u32 {
        self.registers.a = !self.registers.a;

        self.registers.flags.n = true;
        self.registers.flags.h = true;

        0
    }

    /// Complement Carry Flag.
    ///
    /// This instruction flips the carry flag and also clears the subtract and
    /// half-carry flags, but leaves the zero flag as is.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    ///
    /// cpu.registers.flags.c = false;
    /// cpu.instr_ccf();
    /// assert!(cpu.registers.flags.c);
    /// cpu.instr_ccf();
    /// assert!(!cpu.registers.flags.c); // back where we started
    /// ```
    pub(crate) const fn instr_ccf(&mut self) -> u32 {
        self.registers.flags.n = false;
        self.registers.flags.h = false;

        self.registers.flags.c = !self.registers.flags.c;

        0
    }

    /// Set Carry Flag.
    ///
    /// This instruction sets the carry flag to true and also clears the
    /// subtract and half-carry flags, but leaves the zero flag as is.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    ///
    /// cpu.registers.flags.c = false;
    /// cpu.instr_scf();
    /// assert!(cpu.registers.flags.c);
    /// ```
    pub(crate) const fn instr_scf(&mut self) -> u32 {
        self.registers.flags.n = false;
        self.registers.flags.h = false;
        self.registers.flags.c = true;

        0
    }

    /// Enable interrupts by setting the [`CPU::ime`] flag.
    ///
    /// Note that the effect of this instruction isn't immediate and has a
    /// delay of 1 instruction.
    ///
    /// For disabling interrupts check the [`instr_di`](Self::instr_di)
    /// function.
    ///
    /// See [`CPU::ime`] and [`IME`] for more information on the `IME`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.instr_ei();
    /// assert!(matches!(cpu.ime, IME::PendingEnable));
    ///
    /// cpu.step(&mut bus); // ime will still be pending during this instruction
    ///
    /// assert!(matches!(cpu.ime, IME::Enabled)); // ime is now enabled
    ///
    /// cpu.step(&mut bus); // ime is enabled for this instruction
    /// ```
    pub(crate) const fn instr_ei(&mut self) -> u32 {
        self.ime.enable();

        0
    }

    /// Disable interrupts by clearing the [`CPU::ime`] flag.
    ///
    /// Note that compared to [`instr_ei`](Self::instr_ei), this instruction
    /// doesn't cause a 1 instruction latency and immediately disables
    /// interrupts.
    ///
    /// For enabling interrupts see the [`instr_ei`](Self::instr_ei) function.
    ///
    /// See [`CPU::ime`] and [`IME`] for more information on the `IME`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.instr_ei(); // enable ime
    /// assert!(matches!(cpu.ime, IME::PendingEnable));
    ///
    /// cpu.step(&mut bus); // ime will still be pending during this instruction
    ///
    /// assert!(matches!(cpu.ime, IME::Enabled)); // ime is now enabled
    ///
    /// cpu.instr_di(); // disable ime
    ///
    /// assert!(matches!(cpu.ime, IME::Disabled)); // ime is immediately disabled
    /// ```
    pub(crate) const fn instr_di(&mut self) -> u32 {
        self.ime.disable();

        0
    }

    pub(crate) fn instr_halt<IB: InterruptBus>(&mut self, interrupt_bus: &IB) -> u32 {
        if self.ime.is_disabled() && !interrupt_bus.requested_interrupts().is_empty() {
            self.state.halt_bug();
        } else {
            self.state.halt();
        }

        0
    }

    pub(crate) fn instr_stop(&self) -> u32 {
        todo!()
    }

    /// Decimal Adjust Accumulator instruction.
    ///
    /// Designed to be used after performing an arithmetic instruction (ADD,
    /// ADC, SUB, SBC) whose inputs were in Binary-Coded Decimal (BCD),
    /// adjusting the result to likewise be in BCD.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    ///
    /// cpu.registers.a = 0x3F;
    ///
    /// cpu.instr_daa(); // 0x3F will be converted to BCD by adding 0x06;
    ///
    /// assert_eq!(cpu.registers.a, 0x45);
    /// ```
    pub(crate) const fn instr_daa(&mut self) -> u32 {
        let mut a = self.registers.a;
        let mut carry = self.registers.flags.c;
        let half_carry = self.registers.flags.h;
        let subtract = self.registers.flags.n;
        let mut correction = 0x00;

        if subtract {
            if half_carry {
                correction += 0x06;
            }
            if carry {
                correction += 0x60;
            }
            a = a.wrapping_sub(correction);
        } else {
            if half_carry || (a & 0x0F) > 0x09 {
                correction += 0x06;
            }
            if carry || a > 0x99 {
                correction += 0x60;
                carry = true;
            }
            a = a.wrapping_add(correction);
        }

        self.registers.a = a;

        self.registers.flags.z = a == 0;
        self.registers.flags.h = false;
        self.registers.flags.c = carry;

        0
    }

    /// Copy chosen `alu::Flags` into `CPU::registers` flags.
    ///
    /// Use `FLAG_MASK` to chose what flags you want to copy into the CPU's
    /// flag register. Helper masks can be found in the module `flag_mask`
    ///
    /// # Flags
    /// - `flag_mask::Z` : if set, the zero flag from the `flags` parameter will
    ///   be copied into the CPU's registers.
    /// - `flag_mask::N` : if set, the substract flag from `flags` parameter
    ///   will be copied into the CPU's registers.
    /// - `flag_mask::H` : if set, the half-carry flag from `flags` parameter
    ///   will be copied into the CPU's registers.
    /// - `flag_mask::C` : if set, the carry flag from `flags` parameter will be
    ///   copied into the CPU's registers.
    /// - `flag_mask::ALL` : bitwise OR of the `Z`, `N`, `H` and `C` masks,
    ///   this will copy all the flags from the `flags` parameter into the
    ///   CPU's registers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let (_, flags) = alu::sub(0, 1);
    /// cpu.copy_flags::<{flag_mask::Z | flag_mask::N | flag_mask::H | flag_mask::C}>(&flags); // `flag_mask::ALL` could also have been used here
    /// assert!(!cpu.registers.flags.z);
    /// assert!(cpu.registers.flags.n);
    /// assert!(cpu.registers.flags.h);
    /// assert!(cpu.registers.flags.c);
    /// ```
    const fn copy_flags<const FLAG_MASK: u8>(&mut self, flags: &alu::Flags) {
        if FLAG_MASK & flag_mask::Z != 0 {
            self.registers.flags.z = flags.z().expect("unexpected None value for z flag");
        }
        if FLAG_MASK & flag_mask::N != 0 {
            self.registers.flags.n = flags.n().expect("unexpected None value for n flag");
        }
        if FLAG_MASK & flag_mask::H != 0 {
            self.registers.flags.h = flags.h().expect("unexpected None value for h flag");
        }
        if FLAG_MASK & flag_mask::C != 0 {
            self.registers.flags.c = flags.c().expect("unexpected None value for c flag");
        }
    }

    /// Copy all given `alu::Flags` into `CPU::registers` flags.
    ///
    /// See `CPU::copy_flags` for more granularity over which flag is copied or
    /// not.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let (_, flags) = alu::sub(0, 1);
    /// cpu.copy_all_flags(&flags);
    /// assert!(!cpu.registers.flags.z);
    /// assert!(cpu.registers.flags.n);
    /// assert!(cpu.registers.flags.h);
    /// assert!(cpu.registers.flags.c);
    /// ```
    const fn copy_all_flags(&mut self, flags: &alu::Flags) {
        self.copy_flags::<{ flag_mask::ALL }>(flags);
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
    pub const Z: u8 = 0b0001;

    /// mask for the substract flag
    pub const N: u8 = 0b0010;

    /// mask for the half-carry flag
    pub const H: u8 = 0b0100;

    /// mask for the carry flag
    pub const C: u8 = 0b1000;

    /// bitwise OR of the `Z`, `N`, `H` and `C` masks
    pub const ALL: u8 = Z | N | H | C;
}

#[cfg(test)]
#[allow(non_snake_case)] // helps a lot for regs names
mod tests {
    use crate::cpu::interrupts::IME;
    use crate::cpu::tests::MockInterruptBus as Bus;

    use crate::cpu::state::CPUState;

    use super::*;

    mod ld8 {
        use super::*;

        #[test]
        fn B_C() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x01;
            cpu.registers.c = 0xFF;

            let cycles = cpu.instr_ld8(&mut bus, R8Param::B.into(), R8Param::C.into()); // LD B C

            assert_eq!(cycles, 0);
            assert_eq!(cpu.registers.b, 0xFF);
            assert_eq!(cpu.registers.c, 0xFF);
        }

        #[test]
        fn L_indHL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x02;
            cpu.registers.l = 0x01;
            bus.write(0x0201, 0xFF);

            let cycles = cpu.instr_ld8(&mut bus, R8Param::L.into(), R8Param::IndHL.into()); // LD L [HL]

            assert_eq!(cycles, 0);
            assert_eq!(cpu.registers.l, 0xFF);
        }

        #[test]
        fn indHL_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;

            cpu.registers.a = 0xFE;

            let cycles = cpu.instr_ld8(&mut bus, R8Param::IndHL.into(), R8Param::A.into()); // LD [HL] A

            assert_eq!(cycles, 0);
            assert_eq!(bus.read(0x1234), 0xFE);
        }

        #[test]
        fn A_N8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x01;

            cpu.pc = 0x01;
            bus.write(cpu.pc, 0xFD);

            let cycles = cpu.instr_ld8(&mut bus, R8Param::A.into(), LD8SrcParam::N8); // LD A N8

            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x02);
            assert_eq!(cpu.registers.a, 0xFD);
        }

        #[test]
        fn indN16_A() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0x12;

            cpu.pc = 0x0001;
            bus.write_word(cpu.pc, 0x3456);

            let cycles = cpu.instr_ld8(&mut bus, LD8DstParam::IndN16, R8Param::A.into()); // LD [n16] A

            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x0003);
            assert_eq!(bus.read(0x3456), 0x12);
        }
    }

    mod ld16 {
        use super::*;

        #[test]
        fn HL_N16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x0001;
            bus.write_word(cpu.pc, 0x1234);

            let cycles = cpu.instr_ld16(&mut bus, R16Param::HL.into(), LD16SrcParam::N16); // LD HL N16

            assert_eq!(cycles, 0);
            assert_eq!(cpu.registers.hl_get(), 0x1234);
            assert_eq!(cpu.pc, 0x0003);
        }

        #[test]
        fn SP_N16() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x0003;
            bus.write_word(cpu.pc, 0x5678);

            let cycles = cpu.instr_ld16(&mut bus, R16Param::SP.into(), LD16SrcParam::N16); // LD SP N16

            assert_eq!(cycles, 0);
            assert_eq!(cpu.sp, 0x5678);
            assert_eq!(cpu.pc, 0x0005);
        }

        #[test]
        fn indN16_SP() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x0005;
            cpu.sp = 0x5678;
            bus.write_word(cpu.pc, 0x9ABC);

            let cycles = cpu.instr_ld16(&mut bus, LD16DstParam::IndN16, LD16SrcParam::SP); // LD [N16] SP

            assert_eq!(cycles, 0);
            assert_eq!(bus.read_word(0x9ABC), 0x5678);
            assert_eq!(cpu.pc, 0x0007);
        }
    }

    mod stack {
        use super::*;

        mod push {
            use super::*;

            #[test]
            fn BC() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.sp = 0xFFFE;

                cpu.registers.bc_set(0x1234);

                let cycles = cpu.instr_push(&mut bus, R16StackParam::BC);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.sp, 0xFFFC); // sp should have moved down by 2
                assert_eq!(cpu.stack().peek_word(&bus), 0x1234); // front of the stack should contain our value
            }

            #[test]
            fn AF() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.sp = 0xFFFC;

                cpu.registers.a = 0x56;
                cpu.registers.flags.z = true;
                cpu.registers.flags.n = false;
                cpu.registers.flags.h = true;
                cpu.registers.flags.c = false;

                let cycles = cpu.instr_push(&mut bus, R16StackParam::AF);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.sp, 0xFFFA); // sp should have moved down by 2 again
                assert_eq!(bus.read(0xFFFA), 0b1010_0000); // F
                assert_eq!(bus.read(0xFFFB), 0x56); // A
            }
        }

        mod pop {
            use super::*;

            #[test]
            fn BC() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.sp = 0xFFFE;
                bus.write(0xFFFE, 0x34);
                bus.write(0xFFFF, 0x12);

                let cycles = cpu.instr_pop(&mut bus, R16StackParam::BC);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.bc_get(), 0x1234);
                assert_eq!(cpu.sp, 0x0000); // sp should have moved up by 2
            }

            #[test]
            fn AF() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.sp = 0x0000;
                bus.write(0x0000, 0b1010_1010); // F
                bus.write(0x0001, 0x56); // A

                let cycles = cpu.instr_pop(&mut bus, R16StackParam::AF);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.a, 0x56);
                assert!(cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
                assert_eq!(cpu.sp, 0x0002); // sp should have moved up by 2 again
            }
        }
    }

    mod jump {
        use super::*;

        #[test]
        fn instr_jump() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.pc = 0x1234;
            bus.write_word(cpu.pc, 0x5678);

            let cycles = cpu.instr_jump(&bus, JumpParam::N16);

            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x5678);

            bus.write(cpu.pc, (-2_i8).cast_unsigned());
            let cycles = cpu.instr_jump(&bus, JumpParam::PCE8);

            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x5677); // pc +1 -2

            bus.write(cpu.pc, 5);
            let cycles = cpu.instr_jump(&bus, JumpParam::PCE8);

            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x567D); // pc +1 +5
        }

        #[test]
        fn instr_cond_jump() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.pc = 0x1234;
            cpu.registers.flags.z = false;

            // fail this condition -- should not jump
            let cycles = cpu.instr_cond_jump(&bus, Condition::Z, JumpParam::N16);
            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x1236); // pc should still move forward past the argument

            // fail this one too -- should not jump
            let cycles = cpu.instr_cond_jump(&bus, Condition::Z, JumpParam::PCE8);
            assert_eq!(cycles, 0);
            assert_eq!(cpu.pc, 0x1237); // pc should still move forward past the argument

            cpu.registers.hl_set(0x789A);

            // check the opposite condition -- should jump this time
            let cycles = cpu.instr_cond_jump(&bus, Condition::NZ, JumpParam::HL);
            assert_eq!(cycles, 4);
            assert_eq!(cpu.pc, 0x789A);
        }
    }

    mod call_ret {
        use super::*;

        mod call {
            use super::*;

            #[test]
            fn N16() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                bus.write_word(cpu.pc, 0x5678);

                let cycles = cpu.instr_call(&mut bus, CallParam::N16);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.pc, 0x5678); // pc should now be at the call address
                assert_eq!(cpu.sp, 0xFFFC); // sp should have moved down by 2 to push the return address
                assert_eq!(cpu.stack().peek_word(&bus), 0x1236); // check if the return address is the old pc after N16
            }

            #[test]
            fn vec() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                let cycles = cpu.instr_call(&mut bus, CallParam::VEC(0x789A));

                assert_eq!(cycles, 0);
                assert_eq!(cpu.pc, 0x789A); // pc should now be at the call address
                assert_eq!(cpu.sp, 0xFFFC); // sp should have moved down by 2 to push the return address
                assert_eq!(cpu.stack().peek_word(&bus), 0x1234); // check if the return address is the old pc
            }

            #[test]
            fn cond_C_N16() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;

                cpu.registers.flags.c = false;

                // fail this condition -- should not call
                let cycles = cpu.instr_cond_call(&mut bus, Condition::C, CallParam::N16);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.pc, 0x1236); // pc should still move forward past the argument

                bus.write_word(cpu.pc, 0x789A);

                // check the opposite condition -- should call this time
                let cycles = cpu.instr_cond_call(&mut bus, Condition::NC, CallParam::N16);

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x789A); // pc should now be at the call address
                assert_eq!(cpu.sp, 0xFFFC); // sp should have moved down by 2 to push the return address
                assert_eq!(cpu.stack().peek_word(&bus), 0x1238); // check if the return address is the old pc after arg
            }
        }

        mod ret {
            use super::*;

            #[test]
            fn ret() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678); // fake call pc pushed onto the stack

                let cycles = cpu.instr_ret(&bus);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE); // sp should be back to where it was before the fake call
            }

            #[test]
            fn cond_Z() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678); // fake call pc pushed onto the stack

                cpu.registers.flags.z = false;

                // fail this condition -- should not ret
                let cycles = cpu.instr_cond_ret(&bus, Condition::Z);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.pc, 0x1234);
                assert_eq!(cpu.sp, 0xFFFC); // sp should be unchanged and still containing the pc
                assert_eq!(cpu.stack().peek_word(&bus), 0x5678);

                // check the opposite condition -- should ret this time
                let cycles = cpu.instr_cond_ret(&bus, Condition::NZ);

                assert_eq!(cycles, 12);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE);
            }

            #[test]
            fn reti() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x1234;
                cpu.sp = 0xFFFE;
                cpu.stack().push_word(&mut bus, 0x5678); // fake call pc pushed onto the stack

                let cycles = cpu.instr_reti(&bus);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.pc, 0x5678);
                assert_eq!(cpu.sp, 0xFFFE); // sp should be back to where it was before the fake call
                assert!(matches!(cpu.ime, IME::Enabled)); // ime is directly enabled after RETI, not pending enable like EI
            }
        }
    }

    mod alu8 {
        use super::*;

        #[test]
        fn add_B() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0xFF;
            cpu.registers.b = 0x01;

            let cycles = cpu.instr_alu(&bus, ALUOperation::ADD, R8Param::B.into());

            assert_eq!(cpu.registers.a, 0x00);

            assert!(cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn adc_C() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0xFF;
            cpu.registers.c = 0x01;
            cpu.registers.flags.c = true;

            let cycles = cpu.instr_alu(&bus, ALUOperation::ADC, R8Param::C.into());

            assert_eq!(cpu.registers.a, 0x01);

            assert!(!cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn sub_D() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0xFF;
            cpu.registers.d = 0x01;

            let cycles = cpu.instr_alu(&bus, ALUOperation::SUB, R8Param::D.into());

            assert_eq!(cpu.registers.a, 0xFE);

            assert!(!cpu.registers.flags.z);
            assert!(cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn sbc_E() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0x01;
            cpu.registers.e = 0x00;
            cpu.registers.flags.c = true;

            let cycles = cpu.instr_alu(&bus, ALUOperation::SBC, R8Param::E.into());

            assert_eq!(cpu.registers.a, 0x00);

            assert!(cpu.registers.flags.z);
            assert!(cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn and_H() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.h = 0b0000_1111;

            let cycles = cpu.instr_alu(&bus, ALUOperation::AND, R8Param::H.into());

            assert_eq!(cpu.registers.a, 0b0000_1100);

            assert!(!cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn xor_L() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.l = 0b0000_1111;

            let cycles = cpu.instr_alu(&bus, ALUOperation::XOR, R8Param::L.into());

            assert_eq!(cpu.registers.a, 0b0011_0011);

            assert!(!cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn or_indHL() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0b0011_1100;
            cpu.registers.h = 0x12;
            cpu.registers.l = 0x34;
            bus.write(0x1234, 0b0000_1111);

            let cycles = cpu.instr_alu(&bus, ALUOperation::OR, R8Param::IndHL.into());

            assert_eq!(cpu.registers.a, 0b0011_1111);

            assert!(!cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn cp_A() {
            let mut cpu = CPU::new();
            let bus = Bus::new();

            cpu.registers.a = 0xFF;

            let cycles = cpu.instr_alu(&bus, ALUOperation::CP, R8Param::A.into());

            assert_eq!(cpu.registers.a, 0xFF);

            assert!(cpu.registers.flags.z);
            assert!(cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn add_N8() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.a = 0xFF;
            cpu.pc = 0x1234;
            bus.write(0x1234, 0x01);

            let cycles = cpu.instr_alu(&bus, ALUOperation::ADD, ALU8Param::N8);

            assert_eq!(cpu.registers.a, 0x00);

            assert!(cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn inc_B() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x1F;

            let cycles = cpu.instr_inc8(&mut bus, R8Param::B);

            assert_eq!(cpu.registers.b, 0x20);
            assert!(!cpu.registers.flags.z);
            assert!(!cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        #[test]
        fn dec_B() {
            let mut cpu = CPU::new();
            let mut bus = Bus::new();

            cpu.registers.b = 0x20;

            let cycles = cpu.instr_dec8(&mut bus, R8Param::B);

            assert_eq!(cpu.registers.b, 0x1F);
            assert!(!cpu.registers.flags.z);
            assert!(cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            assert_eq!(cycles, 0);
        }

        mod shift {
            use super::*;

            #[test]
            fn rlca() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RLC);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.a, 0b0010_1101);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn rrca() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RRC);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.a, 0b0100_1011);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn rla() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                cpu.registers.flags.c = true;
                let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RL);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.a, 0b0010_1101);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn rra() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                cpu.registers.flags.c = true;
                let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RR);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.a, 0b1100_1011);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }

            mod zero_flag {
                use super::*;

                #[test]
                fn rlca() {
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.registers.a = 0b0000_0000;
                    let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RLC);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.a, 0b0000_0000);
                    assert!(!cpu.registers.flags.z); // RLCA do not set the zero flag even if the result is zero
                }

                #[test]
                fn rrca() {
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.registers.a = 0b0000_0000;
                    let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RRC);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.a, 0b0000_0000);
                    assert!(!cpu.registers.flags.z); // RRCA do not set the zero flag even if the result is zero
                }

                #[test]
                fn rla() {
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.registers.a = 0b0000_0000;
                    let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RL);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.a, 0b0000_0000);
                    assert!(!cpu.registers.flags.z); // RLA do not set the zero flag even if the result is zero
                }

                #[test]
                fn rra() {
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.registers.a = 0b0000_0000;
                    let cycles = cpu.instr_rotate_acc(&mut bus, BitRotateOperation::RR);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.a, 0b0000_0000);
                    assert!(!cpu.registers.flags.z); // RRA do not set the zero flag even if the result is zero
                }
            }
        }
    }

    mod alu16 {
        use super::*;

        mod inc {
            use super::*;

            #[test]
            fn DE() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0x10;
                cpu.registers.e = 0xFF;
                let cycles = cpu.instr_inc16(&mut bus, R16Param::DE);

                assert_eq!(cpu.registers.de_get(), 0x1100);

                assert_eq!(cycles, 0);
            }
        }

        mod dec {
            use super::*;

            #[test]
            fn DE() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0x11;
                cpu.registers.e = 0x00;
                let cycles = cpu.instr_dec16(&mut bus, R16Param::DE);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.de_get(), 0x10FF);
            }

            #[test]
            fn SP() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.sp = 0x0000;
                let cycles = cpu.instr_dec16(&mut bus, R16Param::SP);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.sp, 0xFFFF);
            }
        }

        mod add {
            use super::*;

            #[test]
            fn HL() {
                let mut cpu = CPU::new();
                let bus = Bus::new();

                cpu.registers.hl_set(0x1234);

                let cycles = cpu.instr_add16(&bus, R16Param::HL);

                assert_eq!(cpu.registers.hl_get(), 0x2468);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
                assert_eq!(cycles, 0);
            }

            #[test]
            fn DE() {
                let mut cpu = CPU::new();
                let bus = Bus::new();

                cpu.registers.hl_set(0x2468);
                cpu.registers.de_set(0x4321);

                let cycles = cpu.instr_add16(&bus, R16Param::DE);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.hl_get(), 0x6789);
            }

            #[test]
            fn SP() {
                let mut cpu = CPU::new();
                let bus = Bus::new();

                cpu.sp = 0xFFFF;
                cpu.registers.hl_set(0x6789);

                let cycles = cpu.instr_add16(&bus, R16Param::SP);

                assert!(cpu.registers.flags.h);
                assert!(cpu.registers.flags.c);
                assert_eq!(cpu.registers.hl_get(), 0x6788);
                assert_eq!(cycles, 0);
            }

            mod SPpE8 {
                use super::*;

                #[test]
                fn SP() {
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.sp = 0xFFF8;
                    cpu.pc = 0x0001;
                    bus.write(cpu.pc, 0x08);

                    let cycles = cpu.instr_add_spe8(&mut bus, AddSPe8DstParam::SP);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.pc, 0x0002);
                    assert_eq!(cpu.sp, 0x0000); // should wrap around to 0
                    assert!(cpu.registers.flags.h);
                    assert!(cpu.registers.flags.c);
                }

                #[test]
                fn HL() {
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.sp = 0x0000;
                    cpu.pc = 0x0001;

                    bus.write(cpu.pc, (-8_i8).cast_unsigned());

                    let cycles = cpu.instr_add_spe8(&mut bus, AddSPe8DstParam::HL);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.pc, 0x0002);
                    assert_eq!(cpu.registers.hl_get(), 0xFFF8);
                    assert_eq!(cpu.sp, 0x0000); // should remain unchanged
                    assert!(!cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
        }
    }

    mod ext {
        use super::*;

        mod prefix {
            use super::*;

            #[test]
            fn rl_D() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x0001;
                bus.write(cpu.pc, 0x12); // RL D
                cpu.registers.d = 0b1001_0110;

                let cycles = cpu.instr_prefix(&mut bus);

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.d, 0b0010_1100); // the RL D instruction should have been executed correctly
                assert!(cpu.registers.flags.c); // the carry flag should have been set by the RL D instruction
                assert_eq!(cpu.pc, 0x0002); // pc should have moved forward by 1 to read the prefix byte
            }
            #[test]
            fn set_7_H() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.pc = 0x0001;
                bus.write(cpu.pc, 0xFC); // SET 7 H

                let cycles = cpu.instr_prefix(&mut bus);

                assert_eq!(cycles, 8);
                assert_eq!(cpu.registers.h, 0b1000_0000);
                assert_eq!(cpu.pc, 0x0002); // pc should have moved forward by 1 to read the prefix byte
            }
        }

        mod shift {
            use super::*;

            #[test]
            fn rlc_B() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.b = 0b1001_0110;
                let cycles = cpu.instr_ext_bit_shift(
                    &mut bus,
                    BitShiftOperation::Rotate(BitRotateOperation::RLC),
                    R8Param::B,
                );

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.b, 0b0010_1101);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn rrc_C() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0b1001_0110;
                let cycles = cpu.instr_ext_bit_shift(
                    &mut bus,
                    BitShiftOperation::Rotate(BitRotateOperation::RRC),
                    R8Param::C,
                );

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.c, 0b0100_1011);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn rl_D() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1001_0110;
                cpu.registers.flags.c = true;
                let cycles = cpu.instr_ext_bit_shift(
                    &mut bus,
                    BitShiftOperation::Rotate(BitRotateOperation::RL),
                    R8Param::D,
                );

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.d, 0b0010_1101);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn rr_E() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.e = 0b1001_0110;
                cpu.registers.flags.c = true;
                let cycles = cpu.instr_ext_bit_shift(
                    &mut bus,
                    BitShiftOperation::Rotate(BitRotateOperation::RR),
                    R8Param::E,
                );

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.e, 0b1100_1011);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn sla_H() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1001_0110;
                let cycles = cpu.instr_ext_bit_shift(&mut bus, BitShiftOperation::SLA, R8Param::H);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.h, 0b0010_1100);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(cpu.registers.flags.c);
            }

            #[test]
            fn sra_L() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1001_0110;
                let cycles = cpu.instr_ext_bit_shift(&mut bus, BitShiftOperation::SRA, R8Param::L);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.l, 0b1100_1011); // bit 7 should remain unchanged
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn swap_A() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.a = 0b1001_0110;
                let cycles = cpu.instr_ext_bit_shift(&mut bus, BitShiftOperation::SWAP, R8Param::A);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.a, 0b0110_1001);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }

            #[test]
            fn srl_indHL() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0x12;
                cpu.registers.l = 0x34;
                bus.write(0x1234, 0b1001_0110);
                let cycles =
                    cpu.instr_ext_bit_shift(&mut bus, BitShiftOperation::SRL, R8Param::IndHL);

                assert_eq!(cycles, 0);
                assert_eq!(bus.read(0x1234), 0b0100_1011);
                assert!(!cpu.registers.flags.z);
                assert!(!cpu.registers.flags.n);
                assert!(!cpu.registers.flags.h);
                assert!(!cpu.registers.flags.c);
            }
        }

        mod bit {
            use super::*;

            // NOTE: these tests are more testing the dispatching than actually testing the bit function...

            #[test]
            fn B_0() {
                {
                    // BIT 0 B
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.b = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 0.try_into().unwrap(), R8Param::B);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.b, 0b1001_0110); // should not modify the value
                    assert!(cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn C_1() {
                {
                    // BIT 1 C
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.c = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 1.try_into().unwrap(), R8Param::C);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.c, 0b1001_0110); // should not modify the value
                    assert!(!cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn D_2() {
                {
                    // BIT 2 D
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.d = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 2.try_into().unwrap(), R8Param::D);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.d, 0b1001_0110); // should not modify the value
                    assert!(!cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn E_3() {
                {
                    // BIT 3 E
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.e = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 3.try_into().unwrap(), R8Param::E);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.e, 0b1001_0110); // should not modify the value
                    assert!(cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn H_4() {
                {
                    // BIT 4 H
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.h = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 4.try_into().unwrap(), R8Param::H);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.h, 0b1001_0110); // should not modify the value
                    assert!(!cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn L_5() {
                {
                    // BIT 5 L
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.l = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 5.try_into().unwrap(), R8Param::L);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.l, 0b1001_0110); // should not modify the value
                    assert!(cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn indHL_6() {
                {
                    // BIT 6 [HL]
                    let mut cpu = CPU::new();
                    let mut bus = Bus::new();

                    cpu.registers.h = 0x12;
                    cpu.registers.l = 0x34;
                    bus.write(0x1234, 0b1001_0110);
                    let cycles = cpu.instr_ext_bit(&bus, 6.try_into().unwrap(), R8Param::IndHL);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.h, 0x12);
                    assert_eq!(cpu.registers.l, 0x34);
                    assert_eq!(bus.read(0x1234), 0b1001_0110); // should not modify the value

                    assert!(cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
            #[test]
            fn A_7() {
                {
                    // BIT 7 A
                    let mut cpu = CPU::new();
                    let bus = Bus::new();

                    cpu.registers.a = 0b1001_0110;
                    let cycles = cpu.instr_ext_bit(&bus, 7.try_into().unwrap(), R8Param::A);

                    assert_eq!(cycles, 0);
                    assert_eq!(cpu.registers.a, 0b1001_0110); // should not modify the value
                    assert!(!cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.n);
                    assert!(cpu.registers.flags.h);
                    assert!(!cpu.registers.flags.c);
                }
            }
        }

        mod res {
            use super::*;

            #[test]
            fn B_0() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.b = 0b1001_0110;
                let cycles = cpu.instr_ext_res(&mut bus, 0.try_into().unwrap(), R8Param::B);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.b, 0b1001_0110 & !(1 << 0));
            }

            #[test]
            fn C_1() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0b1001_0110;
                let cycles = cpu.instr_ext_res(&mut bus, 1.try_into().unwrap(), R8Param::C);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.c, 0b1001_0110 & !(1 << 1));
            }

            #[test]
            fn D_2() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1001_0110;
                let cycles = cpu.instr_ext_res(&mut bus, 2.try_into().unwrap(), R8Param::D);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.d, 0b1001_0110 & !(1 << 2));
            }

            #[test]
            fn E_3() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.e = 0b1001_0110;
                let cycles = cpu.instr_ext_res(&mut bus, 3.try_into().unwrap(), R8Param::E);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.e, 0b1001_0110 & !(1 << 3));
            }

            #[test]
            fn H_4() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1001_0110;
                let cycles = cpu.instr_ext_res(&mut bus, 4.try_into().unwrap(), R8Param::H);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.h, 0b1001_0110 & !(1 << 4));
            }

            #[test]
            fn L_5() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1001_0110;
                let cycles = cpu.instr_ext_res(&mut bus, 5.try_into().unwrap(), R8Param::L);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.l, 0b1001_0110 & !(1 << 5));
            }

            #[test]
            fn indHL_7() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0x12;
                cpu.registers.l = 0x34;
                bus.write(0x1234, 0b1001_0110);
                let cycles = cpu.instr_ext_res(&mut bus, 7.try_into().unwrap(), R8Param::IndHL);

                assert_eq!(cycles, 0);
                assert_eq!(bus.read(0x1234), 0b1001_0110 & !(1 << 7));
            }
        }

        mod set {
            use super::*;

            #[test]
            fn B_0() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.b = 0b1001_0110;
                let cycles = cpu.instr_ext_set(&mut bus, 0.try_into().unwrap(), R8Param::B);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.b, 0b1001_0110 | (1 << 0));
            }

            #[test]
            fn C_1() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.c = 0b1001_0110;
                let cycles = cpu.instr_ext_set(&mut bus, 1.try_into().unwrap(), R8Param::C);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.c, 0b1001_0110 | (1 << 1));
            }

            #[test]
            fn D_2() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.d = 0b1001_0110;
                let cycles = cpu.instr_ext_set(&mut bus, 2.try_into().unwrap(), R8Param::D);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.d, 0b1001_0110 | (1 << 2));
            }

            #[test]
            fn E_3() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.e = 0b1001_0110;
                let cycles = cpu.instr_ext_set(&mut bus, 3.try_into().unwrap(), R8Param::E);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.e, 0b1001_0110 | (1 << 3));
            }

            #[test]
            fn H_4() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0b1001_0110;
                let cycles = cpu.instr_ext_set(&mut bus, 4.try_into().unwrap(), R8Param::H);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.h, 0b1001_0110 | (1 << 4));
            }

            #[test]
            fn L_5() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.l = 0b1001_0110;
                let cycles = cpu.instr_ext_set(&mut bus, 5.try_into().unwrap(), R8Param::L);

                assert_eq!(cycles, 0);
                assert_eq!(cpu.registers.l, 0b1001_0110 | (1 << 5));
            }

            #[test]
            fn indHL_7() {
                let mut cpu = CPU::new();
                let mut bus = Bus::new();

                cpu.registers.h = 0x12;
                cpu.registers.l = 0x34;
                bus.write(0x1234, 0b1001_0110);
                let cycles = cpu.instr_ext_set(&mut bus, 7.try_into().unwrap(), R8Param::IndHL);

                assert_eq!(cycles, 0);
                assert_eq!(bus.read(0x1234), 0b1001_0110 | (1 << 7));
            }
        }
    }

    mod misc {
        use super::*;

        #[test]
        fn cpl() {
            let mut cpu = CPU::new();

            cpu.registers.a = 0b1001_0110;
            let cycles = cpu.instr_cpl();

            assert_eq!(cycles, 0);
            assert_eq!(cpu.registers.a, 0b0110_1001);
            assert!(!cpu.registers.flags.z);
            assert!(cpu.registers.flags.n);
            assert!(cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);
        }

        #[test]
        fn ccf() {
            let mut cpu = CPU::new();

            cpu.registers.flags.n = true;
            cpu.registers.flags.h = true;
            cpu.registers.flags.c = true;

            let cycles = cpu.instr_ccf();

            assert_eq!(cycles, 0);
            assert!(!cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(!cpu.registers.flags.c);

            let cycles = cpu.instr_ccf();

            assert_eq!(cycles, 0);
            assert!(!cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(cpu.registers.flags.c);
        }

        #[test]
        fn scf() {
            let mut cpu = CPU::new();

            cpu.registers.flags.n = true;
            cpu.registers.flags.h = true;
            cpu.registers.flags.c = false;

            let cycles = cpu.instr_scf();

            assert_eq!(cycles, 0);
            assert!(!cpu.registers.flags.n);
            assert!(!cpu.registers.flags.h);
            assert!(cpu.registers.flags.c);
        }

        mod halt {
            use crate::interrupts::Interrupt;

            use super::*;

            #[test]
            fn ime_enabled_no_interrupt() {
                let mut cpu = CPU::new();
                let bus = Bus::new();

                cpu.ime.enable_now();

                let cycles = cpu.instr_halt(&bus);

                assert_eq!(cycles, 0);
                assert!(matches!(cpu.state, CPUState::Halted));
            }

            #[test]
            fn ime_disabled_no_interrupt() {
                let mut cpu = CPU::new();
                let bus = Bus::new();

                let cycles = cpu.instr_halt(&bus);

                assert_eq!(cycles, 0);
                assert!(matches!(cpu.state, CPUState::Halted));
            }

            #[test]
            fn ime_disabled_with_interrupt() {
                let mut cpu = CPU::new();
                let bus = Bus::with_pending(Interrupt::VBlank);

                let cycles = cpu.instr_halt(&bus);

                assert_eq!(cycles, 0);
                assert!(matches!(cpu.state, CPUState::HaltBug));
            }
        }

        #[test]
        #[ignore = "requires interrupt handling -- not yet implemented"]
        fn stop() {
            todo!()
        }

        #[test]
        fn ei() {
            let mut cpu = CPU::new();

            let cycles = cpu.instr_ei();

            assert_eq!(cycles, 0);
            assert!(matches!(cpu.ime, IME::PendingEnable(1)));
        }

        #[test]
        fn di() {
            let mut cpu = CPU::new();

            cpu.ime = IME::Enabled;

            let cycles = cpu.instr_di();

            assert_eq!(cycles, 0);
            assert!(matches!(cpu.ime, IME::Disabled));
        }

        mod daa {
            use super::*;

            mod add {
                use super::*;

                #[test]
                fn lo_nibble() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0x0A;

                    cpu.instr_daa(); // only the lowest nibble will overflow in base 10

                    assert_eq!(cpu.registers.a, 0x10);
                    assert!(!cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.c); // no carry
                    assert!(!cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }

                #[test]
                fn both_nibles_wrap_to_zero() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0x9A;

                    cpu.instr_daa(); // both nibbles will overflow in base 10

                    assert_eq!(cpu.registers.a, 0x00);
                    assert!(cpu.registers.flags.z); // zero flag is set as a == 0
                    assert!(cpu.registers.flags.c); // carry flag is set from overlfow
                    assert!(!cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }

                #[test]
                fn half_carry_correction() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0x12;
                    cpu.registers.flags.h = true;

                    cpu.instr_daa(); // half carry correction +0x06

                    assert_eq!(cpu.registers.a, 0x18);
                    assert!(!cpu.registers.flags.z);
                    assert!(!cpu.registers.flags.c);
                    assert!(!cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }

                #[test]
                fn carry_correction() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0x30;
                    cpu.registers.flags.c = true;

                    cpu.instr_daa(); // carry correction +0x60

                    assert_eq!(cpu.registers.a, 0x90);
                    assert!(!cpu.registers.flags.z);
                    assert!(cpu.registers.flags.c);
                    assert!(!cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }

                #[test]
                fn both_nibbles_over_nine() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0xFB;

                    cpu.instr_daa(); // 0xFB +0x66

                    assert_eq!(cpu.registers.a, 0x61);
                    assert!(!cpu.registers.flags.z);
                    assert!(cpu.registers.flags.c);
                    assert!(!cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }
            }

            mod sub {
                use super::*;

                #[test]
                fn both_correction_wrap_to_zero() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0x66;
                    cpu.registers.flags.n = true;
                    cpu.registers.flags.h = true;
                    cpu.registers.flags.c = true;

                    cpu.instr_daa(); // 0x66 -0x66

                    assert_eq!(cpu.registers.a, 0x00);
                    assert!(cpu.registers.flags.z);
                    assert!(cpu.registers.flags.c);
                    assert!(cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }

                #[test]
                fn carry_only() {
                    let mut cpu = CPU::new();

                    cpu.registers.a = 0x72;
                    cpu.registers.flags.n = true;
                    cpu.registers.flags.c = true;

                    cpu.instr_daa(); // 0x72 -0x60

                    assert_eq!(cpu.registers.a, 0x12);
                    assert!(!cpu.registers.flags.z);
                    assert!(cpu.registers.flags.c);
                    assert!(cpu.registers.flags.n);
                    assert!(!cpu.registers.flags.h);
                }
            }
        }
    }
}
