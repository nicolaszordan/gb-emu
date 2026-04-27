mod alu;
mod instructions;
mod registers;
mod stack;

use emu::MemoryBus;
use registers::Registers;
use stack::StackController;

#[allow(clippy::upper_case_acronyms)] // we're suppressing this lint to keep the naming consistent with the pan docs
#[derive(Debug, PartialEq, Eq)]
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
    pub fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            sp: 0,
            pc: 0,
            ime: IME::Disabled,
        }
    }

    pub fn step<M: MemoryBus>(&mut self, mem_bus: &mut M) -> u32 {
        let opcode = self.fetch_byte(mem_bus);
        let cycles = self.execute_instruction(mem_bus, opcode);
        // self.interrupt_check(mem_bus);
        self.update_ime();
        cycles
    }

    // fn interrupt_check<M: MemoryBus>(&mut self, bus: &M) {
    //     if matches!(self.ime, IME::Enabled) {
    //         let pending_irqs = self.get_pending_interrupts(bus);
    //         if pending_irqs != 0 {}
    //     }
    // }

    // /// Get the pending enabled interrupts.
    // fn get_pending_interrupts<M: MemoryBus>(&self, bus: &M) -> u8 {
    //     let ir = bus.read(IF_ADDR);
    //     let ie = bus.read(IE_ADDR);
    //     ir & ie
    // }

    /// Update IME's state from Pending to Enabled
    ///
    /// This is used to emulate the 1 instruction delay after executing the EI
    /// instruction before the IME is actually enabled.
    fn update_ime(&mut self) {
        if matches!(self.ime, IME::PendingEnable) {
            self.ime = IME::Enabled;
        }
    }

    fn stack(&mut self) -> StackController<'_> {
        StackController::new(&mut self.sp)
    }

    /// Read the current byte at `pc` and increment `pc` to the next byte.
    ///
    /// # Example
    /// ```no_run
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
    /// // the next call to `fetch` would yield 0x34 and move pc to 0x0002
    /// ```
    fn fetch_byte<M: MemoryBus>(&mut self, mem_bus: &M) -> u8 {
        let value = mem_bus.read(self.pc);
        self.pc += 1;
        value
    }

    /// Read the current word at `pc` and increment `pc` to the next word.
    ///
    /// # Example
    /// ```no_run
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
    /// // the next call to `fetch_word` would yield 0x5678 and move pc to 0x0004
    /// ```
    fn fetch_word<M: MemoryBus>(&mut self, mem_bus: &M) -> u16 {
        let value = mem_bus.read_word(self.pc);
        self.pc += 2;
        value
    }
}

#[allow(clippy::upper_case_acronyms)] // we're suppressing this lint to keep the naming consistent with the pan docs
#[derive(Debug, PartialEq, Eq)]
enum IME {
    /// Interrupt Master Enable flag is reset, and will be set after the next
    /// instruction is executed.
    PendingEnable,

    /// Interrupt Master Enable flag is set.
    Enabled,

    /// Interrupt Master Enable flag is reset.
    Disabled,
}

const HIGH_MEM_OFFSET: u16 = 0xFF00;

// /// Address of the Interrupt Request Flag register (IF).
// const IF_ADDR: u16 = 0xFF0F;

// /// Address of the Interrupt Enable register (IE).
// const IE_ADDR: u16 = 0xFFFF;

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) struct MockBus {
        pub(crate) mem: [u8; 0x10000],
    }

    impl MockBus {
        /// create a fully zero'ed bus
        pub(crate) fn new() -> Self {
            MockBus { mem: [0; 0x10000] }
        }
    }

    impl MemoryBus for MockBus {
        fn read(&self, address: u16) -> u8 {
            self.mem[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.mem[address as usize] = value
        }
    }

    #[test]
    fn fetch_byte() {
        let mut cpu = CPU::new();
        let mut bus = MockBus::new();

        cpu.pc = 0x0000;
        bus.mem[0x0000] = 0x12;
        bus.mem[0x0001] = 0x34;
        bus.mem[0x0002] = 0x56;

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

        cpu.pc = 0x0000;
        bus.mem[0x0000] = 0x34; // lo
        bus.mem[0x0001] = 0x12; // hi
        bus.mem[0x0002] = 0x78; // lo
        bus.mem[0x0003] = 0x56; // hi

        assert_eq!(cpu.fetch_word(&bus), 0x1234);
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.fetch_word(&bus), 0x5678);
        assert_eq!(cpu.pc, 0x0004);
    }
}
