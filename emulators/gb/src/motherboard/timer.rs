use emu::MemoryBus;

#[derive(Debug)]
pub struct Timer {
    /// Divider Register
    div: u8,

    /// Timer Counter
    tima: u8,

    /// Timer Modulo
    tma: u8,

    /// Timer Control
    tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn step(&mut self, _cycles: u32) {
        todo!();

        // FIXME: LLM generated --> check
        // self.div = self.div.wrapping_add((cycles / 256) as u8);
        // if self.tac & 0b100 != 0 {
        //     let freq = match self.tac & 0b11 {
        //         0 => 4096,
        //         1 => 262144,
        //         2 => 65536,
        //         3 => 16384,
        //         _ => unreachable!(),
        //     };
        //     if (self.div % (256 / freq)) == 0 {
        //         self.tima = self.tima.wrapping_add(1);
        //         if self.tima == 0 {
        //             self.tima = self.tma;
        //         }
        //     }
        // }
    }
}

impl MemoryBus for Timer {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("invalid timer register address: 0x{:04X}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0, // writing any value to DIV resets it to 0
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value & 0b111, // only the lower 3 bits are used
            _ => panic!("invalid timer register address: 0x{:04X}", address),
        }
    }
}
