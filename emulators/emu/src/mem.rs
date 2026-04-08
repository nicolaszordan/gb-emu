pub trait MemoryBus {
    /// Read a single byte from the given `address`.
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// assert_eq!(bus.read(0x1234), 0x56);
    /// ```
    fn read(&self, address: u16) -> u8;

    /// Write a single byte to the given `address`.
    ///
    /// ## Example
    /// ```no_run
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// assert_eq!(bus.read(0x1234), 0x56);
    /// ```
    fn write(&mut self, address: u16, value: u8);

    /// Read a 16-bit word from the given `address`.
    ///
    /// The word is assumed to be in little-endian with the low byte at the
    /// given `address` and the high byte at `address` + 1.
    /// 
    /// Note that this function wraps arround in case of overflow.
    ///
    /// ## Example
    /// ```
    /// let mut bus = Bus::new();
    ///
    /// bus.write(0x1234, 0x56);
    /// bus.write(0x1235, 0x78);
    /// assert_eq!(bus.read_word(0x1234), 0x7856); // note the endian swap
    /// ```
    fn read_word(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a 16-bit word `value` to the given `address`.
    ///
    /// `value` will be written in little endian with the low byte at the given
    /// `address` and the high byte at `address` + 1.
    ///
    /// Note that this function wraps arround in case of overflow.
    /// 
    /// ## Example
    /// ```no_run
    /// let mut bus = Bus::new();
    ///
    /// bus.write_word(0x1234, 0x5678);
    /// assert_eq!(bus.read(0x1234), 0x78); // low byte
    /// assert_eq!(bus.read(0x1235), 0x56); // high byte
    /// ```
    fn write_word(&mut self, address: u16, value: u16) {
        let lo = value as u8;
        let hi = (value >> 8) as u8;
        self.write(address, lo);
        self.write(address.wrapping_add(1), hi);
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
        bus.write(0x00, 0x34); // lo
        bus.write(0x01, 0x12); // hi
        assert_eq!(bus.read_word(0x00), 0x1234);
    }

    #[test]
    fn memory_bus_write_word() {
        let mut bus = MockMemoryBus::new();
        bus.write_word(0x00, 0x1234);
        assert_eq!(bus.read(0x00), 0x34); // lo
        assert_eq!(bus.read(0x01), 0x12); // hi
    }

    #[test]
    fn memory_bus_write_read_word() {
        let mut bus = MockMemoryBus::new();
        bus.write_word(0x00, 0x1234);
        assert_eq!(bus.read(0x00), 0x34); // lo
        assert_eq!(bus.read(0x01), 0x12); // hi
        assert_eq!(bus.read_word(0x00), 0x1234); // read_word returns the value to BE
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
