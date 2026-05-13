pub(crate) mod interrupts;

mod cpu;
mod motherboard;

use cpu::CPU;
use motherboard::MotherBoard;

#[derive(Debug)]
pub struct GameBoy {
    cpu: CPU,
    mb: MotherBoard,
}

impl GameBoy {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            mb: MotherBoard::new(),
        }
    }

    pub fn step(&mut self) {
        let cycles = self.cpu.step(&mut self.mb);
        self.mb.step(cycles);
    }

    #[must_use]
    pub fn get_display_buffer(&self) -> &[u8] {
        todo!()
    }

    pub fn load_rom(&mut self, _data: &[u8]) {
        todo!()
    }

    pub fn press_button(&mut self, _button: Button) {
        todo!()
    }

    pub fn release_button(&mut self, _button: Button) {
        todo!()
    }

    #[must_use]
    pub fn get_mem_dump(&self) -> &[u8] {
        todo!()
    }

    pub fn get_registers(&self) {
        todo!()
    }
}

impl Default for GameBoy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum Button {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {}
