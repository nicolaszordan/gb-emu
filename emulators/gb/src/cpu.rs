mod alu;
mod instructions;
mod interrupts;
mod registers;
mod stack;

use emu::MemoryBus;

use crate::interrupts::InterruptLine;
use interrupts::{IME, INTERRUPT_DISPATCH_CYCLES, InterruptJumpVector};
use registers::Registers;
use stack::StackController;

use instructions::parameter::CallParam;

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
    pub(crate) fn new() -> Self {
        Self {
            registers: Registers::new(),
            sp: 0,
            pc: 0,
            ime: IME::Disabled,
        }
    }

    /// Execute one instruction cycle, including checking for and dispatching any pending interrupts.
    ///
    /// Returns the number of cycles taken to execute the instruction or handle any interrupts.
    pub(crate) fn step<M: MemoryBus + InterruptLine>(&mut self, mem_bus: &mut M) -> u32 {
        if let Some(interrupt_cycles) = self.try_dispatch_pending_interrupt(mem_bus) {
            return interrupt_cycles;
        }

        self.ime.commit_pending();

        let opcode = self.fetch_byte(mem_bus);
        self.execute_instruction(mem_bus, opcode)
    }

    /// Dispatch the highest-priority pending interrupt when IME is enabled.
    ///
    /// Returns the number of cycles taken to handle the interrupt, or 0 if no interrupt was dispatched.
    ///
    /// This method will disable IME and acknowledge the interrupt on the bus if an interrupt is dispatched.
    fn try_dispatch_pending_interrupt<M: MemoryBus + InterruptLine>(
        &mut self,
        mem_bus: &mut M,
    ) -> Option<u32> {
        if self.ime != IME::Enabled {
            return None;
        }

        let Some(pending) = mem_bus.pending_interrupt() else {
            return None;
        };

        self.ime.disable();
        mem_bus.acknowledge_interrupt(pending);

        self.instr_call(
            mem_bus,
            CallParam::VEC(InterruptJumpVector::from(pending).addr()),
        );

        Some(INTERRUPT_DISPATCH_CYCLES)
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

    use crate::interrupts::Interrupt;
    use emu::mem::test_utilities::MockMemoryBus as MockBus;

    /// MockBus implementing InterruptLine trait
    ///
    /// It holds only 1 pending interrupt
    struct MockInterruptBus {
        mem: MockBus,
        pending: Option<Interrupt>,
        acknowledged: Option<Interrupt>,
    }

    impl MockInterruptBus {
        fn new() -> Self {
            Self {
                mem: MockBus::new(),
                pending: None,
                acknowledged: None,
            }
        }

        fn with_pending(interrupt: Interrupt) -> Self {
            Self {
                pending: Some(interrupt),
                ..Self::new()
            }
        }
    }

    impl MemoryBus for MockInterruptBus {
        fn read(&self, address: u16) -> u8 {
            self.mem.read(address)
        }

        fn write(&mut self, address: u16, value: u8) {
            self.mem.write(address, value)
        }
    }

    impl InterruptLine for MockInterruptBus {
        fn pending_interrupt(&self) -> Option<Interrupt> {
            self.pending
        }

        fn acknowledge_interrupt(&mut self, interrupt: Interrupt) {
            self.acknowledged = Some(interrupt);
            self.pending = None;
        }
    }

    mod interrupt {
        use super::*;

        /// Tests for the EI instruction and its latency.
        mod enable {
            use super::*;

            #[test]
            fn ime_enable() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::LCDStat);
                cpu.ime = IME::Disabled;
                cpu.pc = 0x0100;
                cpu.sp = 0xFFFE;

                bus.write(cpu.pc, 0xFB); // EI opcode

                cpu.step(&mut bus); // should set IME to PendingEnable

                assert_eq!(cpu.ime, IME::PendingEnable);
                assert_eq!(cpu.pc, 0x0101);

                cpu.step(&mut bus); // noop (bus is zeroed out), but should transition IME to Enabled

                assert_eq!(cpu.ime, IME::Enabled);
                assert_eq!(cpu.pc, 0x0102);

                cpu.step(&mut bus); // should dispatch the pending LCDStat interrupt

                assert_eq!(cpu.ime, IME::Disabled);
                assert_eq!(cpu.pc, 0x0048); // LCDStat interrupt vector
            }
        }

        mod try_dispatch {
            use super::*;

            #[test]
            fn noop_when_ime_disabled() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);
                cpu.ime = IME::Disabled;
                cpu.pc = 0x0100;
                cpu.sp = 0xFFFE;

                let cycles = cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.pc, 0x0100);
                assert_eq!(cpu.sp, 0xFFFE);
                assert_eq!(cpu.ime, IME::Disabled);
                assert_eq!(bus.acknowledged, None);
            }

            #[test]
            fn noop_when_ime_pending_enable() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);
                cpu.ime = IME::PendingEnable;
                cpu.pc = 0x0100;
                cpu.sp = 0xFFFE;

                let cycles = cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.pc, 0x0100);
                assert_eq!(cpu.sp, 0xFFFE);
                assert_eq!(cpu.ime, IME::PendingEnable);
                assert_eq!(bus.acknowledged, None);
            }

            #[test]
            fn noop_when_no_pending_interrupt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::new();
                cpu.ime = IME::Enabled;
                cpu.pc = 0x0100;
                cpu.sp = 0xFFFE;

                let cycles = cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.pc, 0x0100);
                assert_eq!(cpu.sp, 0xFFFE);
                assert_eq!(cpu.ime, IME::Enabled);
            }

            #[test]
            fn dispatch_returns_interrupt_dispatch_cycles() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);
                cpu.ime = IME::Enabled;
                cpu.sp = 0xFFFE;

                let cycles = cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cycles, Some(INTERRUPT_DISPATCH_CYCLES));
            }

            #[test]
            fn dispatch_disables_ime() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::Timer);
                cpu.ime = IME::Enabled;
                cpu.sp = 0xFFFE;

                cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cpu.ime, IME::Disabled);
            }

            #[test]
            fn dispatch_acknowledges_interrupt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::Serial);
                cpu.ime = IME::Enabled;
                cpu.sp = 0xFFFE;

                cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(bus.acknowledged, Some(Interrupt::Serial));
            }

            #[test]
            fn dispatch_pushes_return_address_and_jumps_to_vector() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);
                cpu.ime = IME::Enabled;
                cpu.pc = 0x0150;
                cpu.sp = 0xFFFE;

                cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cpu.pc, 0x0040);
                assert_eq!(cpu.sp, 0xFFFC);
                assert_eq!(cpu.stack().peek_word(&bus), 0x0150);
            }

            #[test]
            fn dispatch_interrupt_vectors() {
                let cases = [
                    (Interrupt::VBlank, 0x0040_u16),
                    (Interrupt::LCDStat, 0x0048),
                    (Interrupt::Timer, 0x0050),
                    (Interrupt::Serial, 0x0058),
                    (Interrupt::Joypad, 0x0060),
                ];

                for (interrupt, expected_vector) in cases {
                    let mut cpu = CPU::new();
                    let mut bus = MockInterruptBus::with_pending(interrupt);
                    cpu.ime = IME::Enabled;
                    cpu.sp = 0xFFFE;

                    cpu.try_dispatch_pending_interrupt(&mut bus);

                    assert_eq!(
                        cpu.pc, expected_vector,
                        "{interrupt:?} should jump to 0x{expected_vector:04X}"
                    );
                }
            }
        }
    }

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
