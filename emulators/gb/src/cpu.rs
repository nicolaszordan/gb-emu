mod alu;
mod instructions;
mod interrupts;
mod registers;
mod stack;

use emu::MemoryBus;

use crate::interrupts::InterruptLine;
use interrupts::{IME, InterruptJumpVector};
use registers::Registers;
use stack::StackController;

use instructions::parameter::CallParam;

#[allow(clippy::upper_case_acronyms)] // we're suppressing this lint to keep the naming consistent with the pan docs
#[derive(Debug)]
pub struct CPU {
    /// CPU Registers
    registers: Registers,

    /// Stack Pointer
    ///
    /// Points to the top of the stack.
    sp: u16,

    /// Program Counter
    ///
    /// Points to the current instruction being executed.
    pc: u16,

    /// Interrupt Master Enable flag
    ime: IME,
    // state: CPUState, // for HALT and STOP states?
}

impl CPU {
    pub(crate) fn new() -> Self {
        Self {
            registers: Registers::new(),
            sp: 0,
            pc: 0,
            ime: IME::Disabled,
        }
    }

    pub(crate) fn step<M: MemoryBus + InterruptLine>(&mut self, mem_bus: &mut M) -> u32 {
        let interrupt_cycles = self.dispatch_pending_interrupt(mem_bus);

        self.ime.commit_pending();

        let opcode = self.fetch_byte(mem_bus);
        self.execute_instruction(mem_bus, opcode) + interrupt_cycles
    }

    /// Dispatch the highest-priority pending interrupt when IME is enabled.
    ///
    /// Returns the number of cycles taken to handle the interrupt, or 0 if no interrupt was dispatched.
    ///
    /// This method will disable IME and acknowledge the interrupt on the bus if an interrupt is dispatched.
    fn dispatch_pending_interrupt<M: MemoryBus + InterruptLine>(&mut self, mem_bus: &mut M) -> u32 {
        if self.ime != IME::Enabled {
            return 0;
        }

        let Some(pending) = mem_bus.pending_interrupt() else {
            return 0;
        };

        self.ime.disable();
        mem_bus.acknowledge_interrupt(pending);

        self.instr_call(
            mem_bus,
            CallParam::VEC(InterruptJumpVector::from(pending).addr()),
        );

        5
    }

    /// Get a stack controller for the CPU's stack pointer.
    const fn stack(&mut self) -> StackController<'_> {
        StackController::new(&mut self.sp)
    }

    /// Read the current byte at `pc` and increment `pc` to the next byte.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0000;
    /// bus.write(0x0000, 0x12);
    /// bus.write(0x0001, 0x34);
    ///
    /// let value = cpu.fetch(&bus);
    /// assert_eq!(value, 0x12);
    /// assert_eq!(cpu.pc, 0x0001);
    ///
    /// let value = cpu.fetch(&bus);
    /// assert_eq!(value, 0x34);
    /// assert_eq!(cpu.pc, 0x0002);
    /// ```
    fn fetch_byte<M: MemoryBus>(&mut self, mem_bus: &M) -> u8 {
        let value = mem_bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    /// Read the current word pointed by `pc` and move `pc` to the next word.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.pc = 0x0000;
    /// bus.write_word(0x0000, 0x1234);
    /// bus.write_word(0x0002, 0x5678);
    ///
    /// let value = cpu.fetch_word(&bus);
    /// assert_eq!(value, 0x1234);
    /// assert_eq!(cpu.pc, 0x0002);
    ///
    /// let value = cpu.fetch_word(&bus);
    /// assert_eq!(value, 0x5678);
    /// assert_eq!(cpu.pc, 0x0004);
    /// ```
    fn fetch_word<M: MemoryBus>(&mut self, mem_bus: &M) -> u16 {
        let value = mem_bus.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        value
    }
}

// TODO: move this to a more appropriate module
const HIGH_MEM_OFFSET: u16 = 0xFF00;

#[cfg(test)]
mod tests {
    use super::*;

    use emu::mem::test_utilities::MockMemoryBus as MockBus;

    #[test]
    fn fetch_byte() {
        let mut cpu = CPU::new();
        let mut bus = MockBus::new();

        cpu.pc = 0xFFFF;
        bus.mem[0xFFFF] = 0xEF;
        bus.mem[0x0000] = 0x12;
        bus.mem[0x0001] = 0x34;
        bus.mem[0x0002] = 0x56;

        assert_eq!(cpu.fetch_byte(&bus), 0xEF);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.fetch_byte(&bus), 0x12);
        assert_eq!(cpu.pc, 0x0001);
        assert_eq!(cpu.fetch_byte(&bus), 0x34);
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.fetch_byte(&bus), 0x56);
        assert_eq!(cpu.pc, 0x0003);
    }

    #[test]
    fn fetch_word() {
        let mut cpu = CPU::new();
        let mut bus = MockBus::new();

        cpu.pc = 0xFFFE;
        bus.mem[0xFFFE] = 0xEF; // lo
        bus.mem[0xFFFF] = 0xCD; // hi
        bus.mem[0x0000] = 0x34; // lo
        bus.mem[0x0001] = 0x12; // hi
        bus.mem[0x0002] = 0x78; // lo
        bus.mem[0x0003] = 0x56; // hi

        assert_eq!(cpu.fetch_word(&bus), 0xCDEF);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.fetch_word(&bus), 0x1234);
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.fetch_word(&bus), 0x5678);
        assert_eq!(cpu.pc, 0x0004);
    }
}
