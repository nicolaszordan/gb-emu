use emu::MemoryBus;

pub struct Bus {
    memory: [u8; 0x10000],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x10000],
        }
    }
}

impl MemoryBus for Bus {
    /// Read a single byte from the given address.
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// assert_eq!(bus.read(0x1234), 0x56);
    /// ```
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Write a single byte to the given address.
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// assert_eq!(bus.read(0x1234), 0x56);
    /// ```
    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    /// Read a 16-bit word from the given address.
    ///
    /// The 16-bit word is buit from the byte at the given address (high byte)
    /// and the byte at the next address (low byte).
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x12);
    /// bus.write(0x1235, 0x34);
    /// assert_eq!(bus.read_word(0x1234), 0x1234);
    /// ```
    fn read_word(&self, address: u16) -> u16 {
        let high = self.read(address) as u16;
        let low = self.read(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    /// Write a 16-bit word to the given address.
    ///
    /// The high byte of the word is written to the given address, and the low
    /// byte is written to the next address.
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write_word(0x1234, 0x1234);
    /// assert_eq!(bus.read(0x1234), 0x12);
    /// assert_eq!(bus.read(0x1235), 0x34);
    /// ```
    fn write_word(&mut self, address: u16, value: u16) {
        let low = (value & 0x00FF) as u8;
        let high = (value >> 8) as u8;
        self.write(address, high);
        self.write(address.wrapping_add(1), low);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus_is_zeroed() {
        let bus = Bus::new();
        for byte in bus.memory.iter() {
            assert_eq!(*byte, 0);
        }
    }

    #[test]
    fn read_byte() {
        let mut bus = Bus::new();

        bus.memory[0x1234] = 0x12;
        bus.memory[0x0000] = 0x23;
        bus.memory[0xFFFF] = 0x00;

        assert_eq!(bus.read(0x1234), 0x12);
        assert_eq!(bus.read(0x0000), 0x23);
        assert_eq!(bus.read(0xFFFF), 0x00);
    }

    #[test]
    fn write_byte() {
        let mut bus = Bus::new();

        bus.write(0x1234, 0x12);
        bus.write(0x0000, 0x23);
        bus.write(0xFFFF, 0x00);

        assert_eq!(bus.memory[0x1234], 0x12);
        assert_eq!(bus.memory[0x0000], 0x23);
        assert_eq!(bus.memory[0xFFFF], 0x00);
    }

    #[test]
    fn read_word() {
        let mut bus = Bus::new();

        bus.memory[0xFFFE] = 0xFF;
        bus.memory[0xFFFF] = 0x00;
        bus.memory[0x0000] = 0x23;
        bus.memory[0x0001] = 0x45;
        bus.memory[0x1234] = 0x12;
        bus.memory[0x1235] = 0x34;

        assert_eq!(bus.read_word(0x1234), 0x1234);
        assert_eq!(bus.read_word(0x0000), 0x2345);
        assert_eq!(bus.read_word(0xFFFE), 0xFF00);
        assert_eq!(bus.read_word(0xFFFF), 0x0023); // wraps around to 0x0000
    }

    #[test]
    fn write_word() {
        let mut bus = Bus::new();

        bus.write_word(0x1234, 0x1234);
        bus.write_word(0x0001, 0x2345);
        bus.write_word(0xFFFF, 0xFF00); // wraps around to 0x0000

        assert_eq!(bus.memory[0xFFFF], 0xFF);
        assert_eq!(bus.memory[0x0000], 0x00);
        assert_eq!(bus.memory[0x0001], 0x23);
        assert_eq!(bus.memory[0x0002], 0x45);
        assert_eq!(bus.memory[0x1234], 0x12);
        assert_eq!(bus.memory[0x1235], 0x34);
    }
}
