mod cpu;
mod mem;
// mod apu;
// mod ppu;
// mod timer;

pub struct GameBoy {
    mem: mem::Bus,
    cpu: cpu::CPU,
    // ppu: ppu::PPU,
    // apu: apu::APU,
    // timer: timer::Timer,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            mem: mem::Bus::new(),
            cpu: cpu::CPU::new(),
            // ppu: ppu::PPU::new(),
            // apu: apu::APU::new(),
            // timer: timer::Timer::new(),
        }
    }

    pub fn tick(&mut self) {
        let cycles = self.cpu.step(&mut self.mem);
        // self.ppu.step(&mut self.mem, cycles);
        // self.apu.step(&mut self.mem, cycles);
        // self.timer.step(&mut self.mem, cycles);
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
