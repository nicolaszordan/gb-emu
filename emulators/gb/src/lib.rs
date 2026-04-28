mod cpu;
// mod motherboard;
mod mem;

pub struct GameBoy {
    mem: mem::Bus,
    cpu: cpu::CPU,
    // mb: MotherBoard,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            mem: mem::Bus::new(),
            cpu: cpu::CPU::new(),
            // mb: MotherBoard::new(),
        }
    }

    pub fn step(&mut self) {
        let _cycles = self.cpu.step(&mut self.mem);
        // self.mb.step(cycles);
    }

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
