/// Interrupt-related registers and their corresponding read/write functions.
///
/// Note that these registers are not stored in the `InterruptController`
/// struct, but rather in the memory bus, and these functions are used to
/// read/write them from/to the bus.
use super::InterruptFlags;
use emu::MemoryBus;

/// Interrupt Flags register (IF) read/write functions.
pub mod flags {
    use super::*;

    /// Address of the Interrupt Flags register (IF).
    const IF_ADDR: u16 = 0xFF0F;

    /// Read the Interrupt Flags register (IF) from the memory bus and return it as
    /// an `InterruptFlags`.
    pub fn read<M: MemoryBus>(bus: &M) -> InterruptFlags {
        InterruptFlags::from_bits_truncate(bus.read(IF_ADDR))
    }

    /// Write the given `InterruptFlags` to the Interrupt Flags register (IF) on
    /// the memory bus.
    ///
    /// This functions overrides the entire IF register with the provided flags,
    /// so it should be used with care to avoid accidentally clearing pending
    /// interrupts. To clear specific interrupt flags, it's recommended to read
    /// the current IF value, modify it by clearing the desired flags, and then
    /// write it back using this function.
    pub fn write<M: MemoryBus>(bus: &mut M, flags: InterruptFlags) {
        bus.write(IF_ADDR, flags.bits());
    }
}

/// Interrupt Enable register (IE) read/write functions.
pub mod enable {
    use super::*;

    /// Address of the Interrupt Enable register (IE).
    pub const IE_ADDR: u16 = 0xFFFF;

    /// Read the Interrupt Enable register (IE) from the memory bus and return it as an `InterruptFlags`.
    pub fn read<M: MemoryBus>(bus: &M) -> InterruptFlags {
        InterruptFlags::from_bits_truncate(bus.read(IE_ADDR))
    }

    // NOTE: this might end up just being a test utility function as components don't need to write to it directly
    pub fn write<M: MemoryBus>(bus: &mut M, flags: InterruptFlags) {
        bus.write(IE_ADDR, flags.bits());
    }
}
