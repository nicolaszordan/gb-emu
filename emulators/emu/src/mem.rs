pub trait MemoryBus {
    /// Read a single byte from the given address.
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// assert_eq!(bus.read(0x1234), 0x56);
    /// ```
    fn read(&self, address: u16) -> u8;

    /// Write a single byte to the given address.
    ///
    /// ## Example
    /// ```no_run
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// assert_eq!(bus.read(0x1234), 0x56);
    /// ```
    fn write(&mut self, address: u16, value: u8);

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
    /// ```no_run
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

    fn read_range(&self, start: u16, end: u16) -> Vec<u8> {
        (start..=end).map(|addr| self.read(addr)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMemoryBus {
        memory: [u8; 0x10000],
    }

    impl MockMemoryBus {
        fn new() -> Self {
            Self {
                memory: [0; 0x10000],
            }
        }
    }

    impl MemoryBus for MockMemoryBus {
        fn read(&self, address: u16) -> u8 {
            self.memory[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.memory[address as usize] = value
        }
    }

    #[test]
    fn memory_bus_read_word() {
        let mut bus = MockMemoryBus::new();
        bus.write(0x00, 0x12);
        bus.write(0x01, 0x34);
        assert_eq!(bus.read_word(0x00), 0x1234);
    }

    #[test]
    fn memory_bus_write_word() {
        let mut bus = MockMemoryBus::new();
        bus.write_word(0x00, 0x1234);
        assert_eq!(bus.read(0x00), 0x12);
        assert_eq!(bus.read(0x01), 0x34);
    }

    #[test]
    fn memory_bus_read_range() {
        let mut bus = MockMemoryBus::new();
        bus.write(0x00, 0x12);
        bus.write(0x01, 0x34);
        bus.write(0x02, 0x56);
        let range = bus.read_range(0x00, 0x02);
        assert_eq!(range, vec![0x12, 0x34, 0x56]);
    }
}
