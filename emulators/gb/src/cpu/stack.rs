use emu::MemoryBus;

/// A helper struct to manage push and pop stack operations.
pub struct StackController<'a> {
    sp: &'a mut u16,
}

impl<'a> StackController<'a> {
    pub const fn new(sp: &'a mut u16) -> Self {
        Self { sp }
    }

    /// Push `word` into the stack.
    ///
    /// [`Self::sp`] is decremented and then `word` is written where
    /// [`Self::sp`] is now currently pointing.
    ///
    /// Note that `word` is stored into the bus in little endian with the low
    /// byte stored at [`Self::sp`] and the high byte at [`Self::sp`] + 1.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.sp = 0xFFFE;
    ///
    /// cpu.stack().push_word(&mut bus, 0x1234);
    /// assert_eq!(cpu.sp, 0xFFFC); // SP is decremented by 2 (word size) and points to the pushed value
    /// assert_eq!(bus.mem[0xFFFC..=0xFFFD], [0x34, 0x12]); // note endian swap
    /// ```
    pub fn push_word<M: MemoryBus>(&mut self, bus: &mut M, word: u16) {
        *self.sp = self.sp.wrapping_sub(2); // word size
        bus.write_word(*self.sp, word);
    }

    /// Pop and return the last pushed value in the stack.
    ///
    /// The value is read and [`Self::sp`] is incremented to point to the next
    /// value.
    ///
    /// Note that the 16bit word is assumed to be stored in little endian.
    ///
    /// Note that this implementation doesn't prevent from popping from an empty
    /// stack and will just circle around.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cpu = CPU::new();
    /// let mut bus = Bus::new();
    ///
    /// cpu.sp = 0xFFFE;
    ///
    /// cpu.stack().push_word(&mut bus, 0x1234);
    /// assert_eq!(cpu.sp, 0xFFFC); // SP is decremented by word size and stack contains 0x1234.
    ///
    /// let value = cpu.stack().pop_word(&bus);
    /// assert_eq!(value, 0x1234);
    /// assert_eq!(cpu.sp, 0xFFFE); // SP is incremented by word size and is back where it started.
    /// ```
    pub fn pop_word<M: MemoryBus>(&mut self, bus: &M) -> u16 {
        let value = bus.read_word(*self.sp);
        *self.sp = self.sp.wrapping_add(2); // word size
        value
    }
}

#[cfg(test)]
impl StackController<'_> {
    /// Peek at the last pushed value in the stack without modifying [`Self::sp`].
    pub fn peek_word<M: MemoryBus>(&self, bus: &M) -> u16 {
        bus.read_word(*self.sp)
    }
}

#[cfg(test)]
#[allow(clippy::upper_case_acronyms)] // we're suppressing this lint to keep the naming consistent with the pan docs
mod tests {
    use super::*;

    use emu::mem::test_utilities::MockMemoryBus as RAM;

    struct CPU {
        sp: u16,
    }

    impl CPU {
        fn new() -> Self {
            Self { sp: 0 }
        }

        fn stack(&mut self) -> StackController<'_> {
            StackController::new(&mut self.sp)
        }
    }

    #[test]
    fn stack_push_pop() {
        let mut cpu = CPU::new();
        let mut ram = RAM::new();

        cpu.sp = 0xFFFE;

        cpu.stack().push_word(&mut ram, 0x1234);
        assert_eq!(cpu.sp, 0xFFFC);
        assert_eq!(ram.mem[0xFFFC..=0xFFFD], [0x34, 0x12]);

        cpu.stack().push_word(&mut ram, 0x5678);
        assert_eq!(cpu.sp, 0xFFFA);
        assert_eq!(ram.mem[0xFFFA..=0xFFFB], [0x78, 0x56]);

        cpu.stack().push_word(&mut ram, 0x9ABC);
        assert_eq!(cpu.sp, 0xFFF8);
        assert_eq!(ram.mem[0xFFF8..=0xFFF9], [0xBC, 0x9A]);

        assert_eq!(cpu.stack().pop_word(&ram), 0x9ABC);
        assert_eq!(cpu.sp, 0xFFFA);

        assert_eq!(cpu.stack().pop_word(&ram), 0x5678);
        assert_eq!(cpu.sp, 0xFFFC);

        assert_eq!(cpu.stack().pop_word(&ram), 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);

        assert_eq!(cpu.stack().pop_word(&ram), 0x0000); // "invalid" pop! -- gameboy doesn't prevent this so we just circle back
        assert_eq!(cpu.sp, 0x0000);

        assert_eq!(cpu.stack().pop_word(&ram), 0x0000); // "invalid" pop!
        assert_eq!(cpu.sp, 0x0002);
    }

    #[test]
    fn stack_peek() {
        let mut cpu = CPU::new();
        let mut ram = RAM::new();

        cpu.sp = 0xFFFE;

        cpu.stack().push_word(&mut ram, 0x1234);
        assert_eq!(cpu.sp, 0xFFFC); // sp is dec after the push
        assert_eq!(cpu.stack().peek_word(&ram), 0x1234);
        assert_eq!(cpu.sp, 0xFFFC); // sp should be unchanged

        cpu.stack().push_word(&mut ram, 0x5678);
        assert_eq!(cpu.sp, 0xFFFA); // sp is dec again after the push
        assert_eq!(cpu.stack().peek_word(&ram), 0x5678);
        assert_eq!(cpu.sp, 0xFFFA); // sp should be unchanged
    }
}
