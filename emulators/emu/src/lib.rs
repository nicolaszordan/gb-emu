pub trait CPU {}

pub trait Registers {}

pub trait PPU {}

pub trait APU {}

pub trait MemoryBus {
    /// Reads a single byte from the specified address.
    fn read(&self, address: u16) -> u8;

    /// Writes a single byte to the specified address.
    fn write(&mut self, address: u16, value: u8);

    /// Reads a 16-bit word (two bytes) from the specified address.
    fn read_word(&self, address: u16) -> u16;

    /// Writes a 16-bit word (two bytes) to the specified address.
    fn write_word(&mut self, address: u16, value: u16);

    fn read_range(&self, start: u16, end: u16) -> Vec<u8> {
        (start..=end).map(|addr| self.read(addr)).collect()
    }
}
