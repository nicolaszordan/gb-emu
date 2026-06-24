mod alu;
mod instructions;
mod interrupts;
mod registers;
mod stack;
mod state;

use emu::MemoryBus;

use crate::cycles::TCycles;
use crate::interrupts::{Interrupt, InterruptBus};
use instructions::Opcode;
use interrupts::{IME, INTERRUPT_DISPATCH_CYCLES, InterruptJumpVector};
use registers::Registers;
use stack::StackController;
use state::{CPUState, HALTED_CYCLES};

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
    ///
    /// Controls whether the CPU will jump to interrupt vectors when an interrupt is requested.
    ime: IME,

    /// Current state of the CPU
    ///
    /// Used to manage halting and stopping behavior.
    state: CPUState,
}

impl CPU {
    /// Create a new [`CPU`] with all fields set to `0`.
    ///
    /// [`CPU::ime`] is set to [`IME::Disabled`] and [`CPU::state`] is set to [`CPUState::Running`].
    pub(crate) const fn new() -> Self {
        Self {
            registers: Registers::new(),
            sp: 0x00,
            pc: 0x00,
            ime: IME::Disabled,
            state: CPUState::Running,
        }
    }

    /// Execute one instruction cycle, including checking for and dispatching any pending interrupts.
    ///
    /// Returns the number of cycles taken to execute the instruction or handle any interrupts.
    pub(crate) fn step<B: MemoryBus + InterruptBus>(&mut self, bus: &mut B) -> TCycles {
        // Halt handling needs to be done before checking for interrupts
        // since pending interrupts need to be serviced on wake up from halt.
        if let Some(cycles) = self.handle_halt(bus) {
            return cycles;
        }

        if let Some(cycles) = self.try_dispatch_pending_interrupt(bus) {
            return cycles;
        }

        let opcode = self.fetch_next_opcode(bus);

        let cycles = self.execute_instruction(bus, opcode);

        // Tick the IME flag after executing the instruction
        // Note that the EI instructions sets the delay to 2 calls to
        // `ime.tick()` as we're taking into account the current call to
        // `step` and the next one.
        self.ime.tick();

        cycles
    }

    /// Try to dispatch the highest-priority pending interrupt. Returns `None`
    /// if no interrupt was dispatched or `Some(cycles)` if an interrupt was
    /// dispatched.
    ///
    /// This method dispatches only if the IME flag is enabled and if an
    /// interrupt is pending.
    /// This method will disable IME and acknowledge the interrupt on the bus
    /// if an interrupt is dispatched.
    fn try_dispatch_pending_interrupt<B: MemoryBus + InterruptBus>(
        &mut self,
        bus: &mut B,
    ) -> Option<TCycles> {
        if self.ime != IME::Enabled {
            // IME is not enabled, nothing to do
            return None;
        }

        let Some(interrupt) = bus.highest_pending_interrupt() else {
            // No pending interrupts, nothing to do
            return None;
        };

        self.dispatch_interrupt(bus, interrupt);

        Some(INTERRUPT_DISPATCH_CYCLES)
    }

    /// Dispatch the given interrupt.
    ///
    /// This method will disable IME and acknowledge the interrupt on the bus
    /// and move the program counter to the corresponding interrupt vector.
    fn dispatch_interrupt<B: MemoryBus + InterruptBus>(
        &mut self,
        bus: &mut B,
        interrupt: Interrupt,
    ) {
        self.ime.disable();

        bus.acknowledge_interrupt(interrupt);

        let dst = InterruptJumpVector::from(interrupt).addr();
        let pc = if self.state.is_halt_bug() {
            self.state.wake();
            self.pc.wrapping_sub(1)
        } else {
            self.pc
        };

        self.stack().push_word(bus, pc);

        self.pc = dst;
    }

    /// Handle the halted state of the CPU.
    ///
    /// Wakes if any enabled interrupt is requested, otherwise consumes 4 cycles
    /// in the halted state.
    ///
    /// Returns `None` if the CPU is not halted and should continue with normal
    /// instruction dispatch (this includes when the CPU wakes up). Returns
    /// `Some(cycles)` if the CPU is halted and should consume the given number
    /// of cycles without dispatching an instruction.
    fn handle_halt<I: InterruptBus>(&mut self, bus: &I) -> Option<TCycles> {
        if !self.state.is_halted() {
            return None;
        }

        // Check if an interrupt is pending to exit the halt state
        if bus.pending_interrupts().is_empty() {
            Some(HALTED_CYCLES)
        } else {
            self.state.wake();
            None // wake up; fall through to normal dispatch
        }
    }

    /// Get a stack controller for the CPU's stack pointer.
    const fn stack(&mut self) -> StackController<'_> {
        StackController::new(&mut self.sp)
    }

    /// Fetch the next opcode to execute.
    ///
    /// In the normal case, this will read the byte at the current `pc` and
    /// increment `pc` by 1.
    /// However, if the CPU is in the `HaltBug` state, this will read the byte
    /// at the current `pc` without incrementing `pc`, and then transition the
    /// CPU back into the `Running` state.
    fn fetch_next_opcode<B: MemoryBus>(&mut self, bus: &B) -> Opcode {
        let opcode = if self.state.is_halt_bug() {
            self.state.wake();
            bus.read(self.pc)
        } else {
            self.fetch_byte(bus)
        };

        Opcode::new(opcode)
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

    use crate::interrupts::{Interrupt, InterruptFlags};
    use emu::mem::test_utilities::MockMemoryBus as MockBus;

    /// `MockBus` implementing `InterruptLine` trait
    pub struct MockInterruptBus {
        pub mem: MockBus,
        pub enabled: InterruptFlags,
        pub requested: InterruptFlags,
    }

    impl MockInterruptBus {
        pub fn new() -> Self {
            Self {
                mem: MockBus::new(),
                enabled: InterruptFlags::empty(),
                requested: InterruptFlags::empty(),
            }
        }

        pub fn with_pending(interrupt: Interrupt) -> Self {
            Self {
                enabled: InterruptFlags::from(interrupt),
                requested: InterruptFlags::from(interrupt),
                ..Self::new()
            }
        }

        pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
            self.enabled |= InterruptFlags::from(interrupt);
        }

        pub fn request_interrupt(&mut self, interrupt: Interrupt) {
            self.requested |= InterruptFlags::from(interrupt);
        }
    }

    impl MemoryBus for MockInterruptBus {
        fn read(&self, address: u16) -> u8 {
            self.mem.read(address)
        }

        fn write(&mut self, address: u16, value: u8) {
            self.mem.write(address, value);
        }
    }

    impl InterruptBus for MockInterruptBus {
        fn requested_interrupts(&self) -> InterruptFlags {
            self.requested
        }

        fn enabled_interrupts(&self) -> InterruptFlags {
            self.enabled
        }

        fn acknowledge_interrupt(&mut self, interrupt: Interrupt) {
            self.requested &= !InterruptFlags::from(interrupt);
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

                assert_eq!(cpu.ime, IME::PendingEnable(0));
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
                assert_eq!(bus.highest_pending_interrupt(), Some(Interrupt::VBlank));
            }

            #[test]
            fn noop_when_ime_pending_enable() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);
                cpu.ime = IME::PendingEnable(0);
                cpu.pc = 0x0100;
                cpu.sp = 0xFFFE;

                let cycles = cpu.try_dispatch_pending_interrupt(&mut bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.pc, 0x0100);
                assert_eq!(cpu.sp, 0xFFFE);
                assert_eq!(cpu.ime, IME::PendingEnable(0));
                assert_eq!(bus.highest_pending_interrupt(), Some(Interrupt::VBlank));
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
                assert_eq!(bus.highest_pending_interrupt(), None);
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

                assert_eq!(bus.requested_interrupts(), InterruptFlags::empty());
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

        // TODO: missing dispatching using the `step` function
    }

    mod halt {
        use super::*;

        mod handle_halt {
            use super::*;

            #[test]
            fn noop_when_running() {
                let mut cpu = CPU::new();
                let bus = MockInterruptBus::new();

                cpu.state = CPUState::Running;

                let cycles = cpu.handle_halt(&bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.state, CPUState::Running);
            }

            #[test]
            fn noop_when_halt_bug() {
                let mut cpu = CPU::new();
                let bus = MockInterruptBus::new();

                cpu.state = CPUState::HaltBug;

                let cycles = cpu.handle_halt(&bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.state, CPUState::HaltBug);
            }

            #[test]
            fn halt_when_not_enabled_interrupt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::new();

                cpu.state = CPUState::Halted;
                bus.request_interrupt(Interrupt::LCDStat); // request an interrupt, but don't enable it

                let cycles = cpu.handle_halt(&bus);

                assert_eq!(cycles, Some(HALTED_CYCLES));
                assert_eq!(cpu.state, CPUState::Halted); // still halted since the interrupt is not enabled

                bus.enable_interrupt(Interrupt::LCDStat); // now enable the interrupt

                let cycles = cpu.handle_halt(&bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake since the interrupt is now enabled
            }

            #[test]
            fn wake_on_pending_interrupt() {
                let mut cpu = CPU::new();
                let bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.state = CPUState::Halted;

                let cycles = cpu.handle_halt(&bus);

                assert_eq!(cycles, None);
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake
            }

            #[test]
            fn consume_cycles_when_no_pending_interrupt() {
                let mut cpu = CPU::new();
                let bus = MockInterruptBus::new();

                cpu.state = CPUState::Halted;

                let cycles = cpu.handle_halt(&bus);

                assert_eq!(cycles, Some(HALTED_CYCLES));
                assert_eq!(cpu.state, CPUState::Halted);
            }
        }

        mod step {
            use super::*;

            #[test]
            fn consume_cycles_when_no_pending_interrupt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::new();

                cpu.ime = IME::Enabled;
                cpu.state = CPUState::Halted;

                let cycles = cpu.step(&mut bus);

                assert_eq!(cycles, HALTED_CYCLES); // CPU is still halted
                assert_eq!(cpu.state, CPUState::Halted);
                assert_eq!(cpu.pc, 0x0000); // PC should not have changed
            }

            #[test]
            fn wake_on_pending_interrupt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.ime = IME::Enabled;
                cpu.state = CPUState::Halted;

                let cycles = cpu.step(&mut bus);

                assert_eq!(cycles, INTERRUPT_DISPATCH_CYCLES); // interrupt was dispatched
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake
                assert_eq!(cpu.pc, 0x0040); // VBlank interrupt vector
            }

            #[test]
            fn wake_on_ime_disabled_pending_interrupt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;

                cpu.state = CPUState::Halted;
                cpu.ime = IME::Disabled;

                let cycles = cpu.step(&mut bus);

                // executes the noop instruction under PC 0x0100 since everything is zeroed out

                assert_eq!(cycles, 4.into()); // instr noop cycles
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake
                assert_eq!(cpu.pc, 0x0101); // PC is incremented to the next instruction after executing the noop
            }

            #[test]
            fn ei_into_halt() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::new();

                bus.enable_interrupt(Interrupt::Joypad);

                cpu.pc = 0x0100;

                bus.write(0x0100, 0xFB); // EI opcode
                bus.write(0x0101, 0x76); // HALT opcode

                cpu.step(&mut bus); // enable ime

                assert_eq!(cpu.ime, IME::PendingEnable(0));
                assert_eq!(cpu.pc, 0x0101);

                cpu.step(&mut bus); // halt instruction

                assert_eq!(cpu.state, CPUState::Halted);
                assert_eq!(cpu.ime, IME::Enabled); // IME should now be enabled after the EI instruction's delay
                assert_eq!(cpu.pc, 0x0102);

                bus.request_interrupt(Interrupt::Joypad); // request an interrupt to wake the CPU

                let cycles = cpu.step(&mut bus); // should wake the CPU

                assert_eq!(cycles, INTERRUPT_DISPATCH_CYCLES);
                assert_eq!(cpu.state, CPUState::Running);
                assert_eq!(cpu.pc, 0x0060); // Joypad interrupt vector
            }
        }

        mod halt_bug {
            use super::*;

            #[test]
            fn halt_bug() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;
                cpu.state = CPUState::HaltBug;
                cpu.ime = IME::Disabled;

                let cycles = cpu.step(&mut bus);

                assert_eq!(cycles, 4.into()); // instr noop cycles
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake
                assert_eq!(cpu.pc, 0x0100); // PC isn't incremented due to halt bug behavior
            }

            #[test]
            fn halt_bug_inc_twice() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;
                cpu.state = CPUState::Running;
                cpu.ime = IME::Disabled;

                bus.write(0x0100, 0x76); // HALT opcode
                bus.write(0x0101, 0x04); // INC B opcode

                cpu.step(&mut bus); // HALT instruction

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU is now in halt bug state
                assert_eq!(cpu.pc, 0x0101); // PC is at the following instruction

                cpu.step(&mut bus); // INC B instruction

                assert_eq!(cpu.registers.b, 0x01); // B register should have been incremented
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake
                assert_eq!(cpu.pc, 0x0101); // PC remains the same due to halt bug behavior

                cpu.step(&mut bus); // INC B instruction again

                assert_eq!(cpu.registers.b, 0x02); // B register should have been incremented again
                assert_eq!(cpu.pc, 0x0102); // PC moves to the next instruction after INC B, as halt bug behavior should be resolved after the first instruction following HALT is executed
            }

            #[test]
            fn halt_bug_changes_param_values() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;
                cpu.state = CPUState::Running;
                cpu.ime = IME::Disabled;

                bus.write(0x0100, 0x76); // HALT opcode
                bus.write(0x0101, 0x06); // LD B, n opcode
                bus.write(0x0102, 0x04); // value 0x04 to be loaded into B register

                cpu.step(&mut bus); // HALT instruction

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU is now in halt bug state
                assert_eq!(cpu.pc, 0x0101); // PC is at the following instruction

                cpu.step(&mut bus); // LD B, n instruction

                assert_eq!(cpu.registers.b, 0x06); // B register should have been loaded with the value of the next instruction (0x06) instead of the intended value (0x04) due to halt bug behavior where PC is not incremented after the first instruction following HALT
                assert_eq!(cpu.state, CPUState::Running); // CPU is now awake
                assert_eq!(cpu.pc, 0x0102); // PC increments only once from fetching the parameter of the instruction

                cpu.step(&mut bus); // Parameter 0x04 is now executed as an instruction instead of being loaded into B register

                assert_eq!(cpu.registers.b, 0x07); // B is instead incremented by the INC B instruction (opcode 0x04)
                assert_eq!(cpu.pc, 0x0103); // PC increments normaly to the next instruction
            }

            #[test]
            fn halt_bug_pushes_incorect_return_address() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;
                cpu.state = CPUState::Running;
                cpu.ime = IME::Disabled;

                bus.write(0x0100, 0x76); // HALT opcode
                bus.write(0x0101, 0xCF); // RST $08 opcode (call to address 0x08)

                cpu.step(&mut bus); // HALT instruction

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU is now in halt bug state
                assert_eq!(cpu.pc, 0x0101); // PC is at the following instruction

                cpu.step(&mut bus); // RST $08 instruction

                assert_eq!(cpu.pc, 0x08); // PC should have jumped to the called address
                assert_eq!(cpu.stack().peek_word(&bus), 0x0101); // The return address pushed onto the stack should be the address of the RST $08 instruction (0x0101) instead of the next instruction after 0x0102.
            }

            #[test]
            fn ei_into_halt_bug() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;
                cpu.state = CPUState::Running;
                cpu.ime = IME::Disabled;

                bus.write(0x0100, 0xFB); // EI opcode
                bus.write(0x0101, 0x76); // HALT opcode -- halt is called before IME is fully enabled with an interrupt pending -> halt bug occurs
                bus.write(0x0040, 0x00); // VBlank interrupt vector -- noop to execute after the interrupt is dispatched
                bus.write(0x0041, 0xD9); // RETI opcode -- return from interrupt

                let cycles = cpu.step(&mut bus); // EI instruction

                assert_eq!(cycles, 4.into()); // EI instruction cycles
                assert_eq!(cpu.ime, IME::PendingEnable(0));
                assert_eq!(cpu.pc, 0x0101); // PC is now at the HALT instruction

                let cycles = cpu.step(&mut bus); // HALT instruction

                assert_eq!(cycles, 4.into()); // HALT instruction cycles
                assert_eq!(cpu.ime, IME::Enabled); // IME is now enabled
                assert_eq!(cpu.state, CPUState::HaltBug); // CPU is now in halt bug state
                assert_eq!(cpu.pc, 0x0102); // PC is after the HALT instruction

                let cycles = cpu.step(&mut bus); // Dispatch the pending VBlank interrupt

                assert_eq!(cycles, INTERRUPT_DISPATCH_CYCLES); // interrupt was dispatched
                assert_eq!(cpu.ime, IME::Disabled); // IME is now disabled
                assert_eq!(cpu.state, CPUState::Running); // CPU is still in halt bug state
                assert_eq!(cpu.pc, 0x0040); // PC is now at the VBlank interrupt vector
                assert_eq!(cpu.stack().peek_word(&bus), 0x0101); // The return address pushed onto the stack should be the address of the HALT instruction (0x0101) instead of the next instruction due to halt bug behavior

                let cycles = cpu.step(&mut bus); // Execute the noop at the VBlank interrupt vector

                assert_eq!(cycles, 4.into()); // noop instruction cycles
                assert_eq!(cpu.pc, 0x0041); // PC is now at the next instruction after the VBlank interrupt vector

                let cycles = cpu.step(&mut bus); // Execute the RETI instruction

                assert_eq!(cycles, 16.into()); // RETI instruction cycles
                assert_eq!(cpu.ime, IME::Enabled); // IME is now enabled through RETI instruction
                assert_eq!(cpu.pc, 0x0101); // PC is now back at the HALT instruction after returning from the interrupt due to earlier halt bug

                let cycles = cpu.step(&mut bus); // Execute the HALT instruction again

                assert_eq!(cycles, 4.into()); // HALT instruction cycles
                assert_eq!(cpu.state, CPUState::Halted); // CPU is now properly halted
            }

            /// Chaining 2 halts with a pending interrupt and IME disabled
            /// should result in a dead lock where the second halt instruction
            /// is called repeatedly.
            #[test]
            fn double_halt_bug_deadlock() {
                let mut cpu = CPU::new();
                let mut bus = MockInterruptBus::with_pending(Interrupt::VBlank);

                cpu.pc = 0x0100;
                cpu.state = CPUState::Running;
                cpu.ime = IME::Disabled;

                bus.write(0x0100, 0x76); // HALT opcode
                bus.write(0x0101, 0x76); // HALT opcode

                cpu.step(&mut bus); // 1st HALT instruction

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU is in halt bug state
                assert_eq!(cpu.pc, 0x0101); // PC is at the next instruction after the first HALT

                cpu.step(&mut bus); // Execute the 2nd HALT instruction

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU is kept in halt bug state due to the second HALT
                assert_eq!(cpu.pc, 0x0101); // PC stays on the second HALT instruction due to halt bug behavior

                cpu.step(&mut bus); // Execute the HALT instruction again

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU remains in halt bug state
                assert_eq!(cpu.pc, 0x0101); // PC is now at the next instruction after the first HALT

                cpu.step(&mut bus); // Execute the HALT instruction again

                assert_eq!(cpu.state, CPUState::HaltBug); // CPU remains in halt bug state
                assert_eq!(cpu.pc, 0x0101); // PC is now at the next instruction after the first HALT
            }
        }
    }

    mod fetch {
        use super::*;

        #[test]
        fn fetch_next_opcode_halt_bug() {
            let mut cpu = CPU::new();
            let mut bus = MockBus::new();

            cpu.pc = 0x0000;
            bus.mem[0x0000] = 0x12;
            bus.mem[0x0001] = 0x34;

            cpu.state = CPUState::HaltBug;

            let opcode1 = cpu.fetch_next_opcode(&bus);
            assert_eq!(opcode1, 0x12.into());
            assert_eq!(cpu.pc, 0x0000); // PC should not have incremented due to halt bug
            assert_eq!(cpu.state, CPUState::Running); // CPU should have woken up from halt bug state

            let opcode2 = cpu.fetch_next_opcode(&bus);
            assert_eq!(opcode2, 0x12.into()); // should fetch the same opcode again due to halt bug
            assert_eq!(cpu.pc, 0x0001); // PC should still not have incremented
        }

        #[test]
        fn fetch_next_opcode_normal() {
            let mut cpu = CPU::new();
            let mut bus = MockBus::new();

            cpu.pc = 0x0000;
            bus.mem[0x0000] = 0x12;
            bus.mem[0x0001] = 0x34;

            cpu.state = CPUState::Running;

            let opcode1 = cpu.fetch_next_opcode(&bus);
            assert_eq!(opcode1, 0x12.into());
            assert_eq!(cpu.pc, 0x0001); // PC should have incremented

            let opcode2 = cpu.fetch_next_opcode(&bus);
            assert_eq!(opcode2, 0x34.into());
            assert_eq!(cpu.pc, 0x0002); // PC should have incremented again
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
}
