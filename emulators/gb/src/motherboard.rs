mod interrupts;

use emu::MemoryBus;

use crate::cycles::TCycles;

use crate::interrupts::{Interrupt, InterruptBus, InterruptFlags};
use interrupts::{IE_ADDRESS, IF_ADDRESS, InterruptRegisters};

pub(crate) mod timer;

use timer::Timer;

#[derive(Debug)]
pub struct MotherBoard {
    interrupt_registers: InterruptRegisters,
    timer: Timer,
}

impl MotherBoard {
    pub const fn new() -> Self {
        Self {
            interrupt_registers: InterruptRegisters::new(),
            timer: Timer::new(),
        }
    }

    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)] // suppressing these lints while we wait for the components to be implemented
    pub const fn step(&mut self, _cycles: TCycles) {
        // self.ppu.step(cycles);
        // self.apu.step(cycles);
        self.timer.step(cycles);
    }
}

impl MemoryBus for MotherBoard {
    #[allow(clippy::match_single_binding)] // suppressing this lint while we wait for the components to be implemented
    fn read(&self, address: u16) -> u8 {
        match address {
            // 0x0000..=0x7FFF => {
            //     // ROM bank 0 (cartridge)
            // },
            // 0x8000..=0x9FFF => {
            //     // VRAM (PPU)
            // },
            // 0xA000..=0xBFFF => {
            //     // External RAM (cartridge)
            // },
            // 0xC000..=0xDFFF => {
            //     // Work RAM
            // },
            // 0xE000..=0xFDFF => {
            //     // Echo RAM (mirrors 0xC000..=0xDDFF)
            // },
            // 0xFE00..=0xFE9F => {
            //     // OAM (PPU)
            // },
            // 0xFEA0..=0xFEFF => {
            //     // Unusable memory
            // },
            // 0xFF04..=0xFF07 => {
            //     // Timer registers
            // },
            IE_ADDRESS | IF_ADDRESS => {
                // Interrupt registers
                self.interrupt_registers.read(address)
            }
            0xFF04..=0xFF07 => {
                // Timer registers
                self.timer.read(address)
            }
            // 0xFF00..=0xFF7F => {
            //     // I/O registers
            // },
            // 0xFF80..=0xFFFE => {
            //     // High RAM (HRAM)
            // },
            _ => 0xFF, // reading from unmapped memory returns 0xFF
        }
    }

    #[allow(clippy::match_single_binding)] // suppressing this lint while we wait for the components to be implemented
    fn write(&mut self, address: u16, value: u8) {
        match address {
            // 0x0000..=0x7FFF => {
            //     // ROM bank 0 (cartridge)
            // },
            // 0x8000..=0x9FFF => {
            //     // VRAM (PPU)
            // },
            // 0xA000..=0xBFFF => {
            //     // External RAM (cartridge)
            // },
            // 0xC000..=0xDFFF => {
            //     // Work RAM
            // },
            // 0xE000..=0xFDFF => {
            //     // Echo RAM (mirrors 0xC000..=0xDDFF)
            // },
            // 0xFE00..=0xFE9F => {
            //     // OAM (PPU)
            // },
            // 0xFEA0..=0xFEFF => {
            //     // Unusable memory
            // },
            // 0xFF04..=0xFF07 => {
            //     // Timer registers
            // },
            IE_ADDRESS | IF_ADDRESS => {
                // Interrupt registers
                self.interrupt_registers.write(address, value);
            }
            0xFF04..=0xFF07 => {
                // Timer registers
                self.timer.write(address, value)
            }
            // 0xFF00..=0xFF7F => {
            //     // I/O registers
            // },
            // 0xFF80..=0xFFFE => {
            //     // High RAM (HRAM)
            // },
            _ => (), // writing to unmapped memory is ignored (could also be considered a segmentation fault)
        }
    }
}

impl InterruptBus for MotherBoard {
    fn requested_interrupts(&self) -> InterruptFlags {
        self.interrupt_registers.requested_interrupts()
    }

    fn enabled_interrupts(&self) -> InterruptFlags {
        self.interrupt_registers.enabled_interrupts()
    }

    fn acknowledge_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_registers.acknowledge_interrupt(interrupt);
    }
}

#[cfg(test)]
mod tests {}
