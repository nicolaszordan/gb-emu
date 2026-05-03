use emu::MemoryBus;

#[derive(Debug)]
pub struct MotherBoard {
    /// Interrupt Request Flags (IF) - 0xFF0F
    irq: u8,

    /// Interrupt Enable Flags (IE) - 0xFFFF
    ie: u8,
}

impl MotherBoard {
    pub fn new() -> Self {
        Self {
            irq: 0,
            ie: 0,
        }
    }

    pub fn step(&mut self, _cycles: u32) {
        // self.ppu.step(cycles);
        // self.apu.step(cycles);
        // self.timer.step(cycles);
    }
}

impl MemoryBus for MotherBoard {
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
            //     self.timer.read(address)
            // }
            0xFF0F => self.irq,
            // 0xFF00..=0xFF7F => {
            //     // I/O registers
            // },
            // 0xFF80..=0xFFFE => {
            //     // High RAM (HRAM)
            // },
            0xFFFF => self.ie,
            _ => 0xFF, // reading from unmapped memory returns 0xFF
        }
    }

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
            //     self.timer.write(address, value)
            // }
            0xFF0F => self.irq = value,
            // 0xFF00..=0xFF7F => {
            //     // I/O registers
            // },
            // 0xFF80..=0xFFFE => {
            //     // High RAM (HRAM)
            // },
            0xFFFF => self.ie = value,
            _ => (), // writing to unmapped memory is ignored (could also be considered a segmentation fault)
        }
    }
}

#[cfg(test)]
mod tests {}
