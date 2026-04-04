use emu::MemoryBus;

/// A helper struct to manage stack operations (push and pop).
pub struct StackControler<'a> {
    sp: &'a mut u16,
}

impl<'a> StackControler<'a> {
    pub fn new(sp: &'a mut u16) -> Self {
        Self { sp }
    }

    /// Push `word` into the stack.
    ///
    /// [`Self::sp`] is decremented and then `word` is written where
    /// [`Self::sp`] is currently pointing
    ///
    /// # Example
    /// ```no_run
    /// let mut cpu = CPU::new();
    /// let mut ram = RAM::new();
    ///
    /// cpu.sp = 0xFFFE;
    /// cpu.stack().push_word(&mut ram, 0x1234);
    /// assert_eq!(cpu.sp, 0xFFFC); // SP is decremented by 2 (word size) and points to the pushed value
    /// assert_eq!(ram.mem[0xFFFC..=0xFFFD], [0x12, 0x34]);
    /// ```
    pub fn push_word<M: MemoryBus>(&mut self, bus: &mut M, word: u16) {
        *self.sp = self.sp.wrapping_sub(2); // word size
        bus.write_word(*self.sp, word);
    }

    /// Pop and return the last pushed value in the stack.
    ///
    /// The value is read and [`Self::sp`] is incremented to point to the next
    /// value.
    pub fn pop_word<M: MemoryBus>(&mut self, bus: &M) -> u16 {
        let value = bus.read_word(*self.sp);
        *self.sp = self.sp.wrapping_add(2); // word size
        value
    }
}

#[cfg(test)]
impl<'a> StackControler<'a> {
    /// Peek at the last pushed value in the stack without modifying [`Self::sp`].
    pub fn peek_word<M: MemoryBus>(&self, bus: &M) -> u16 {
        bus.read_word(*self.sp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RAM {
        mem: [u8; 0x10000],
    }

    impl RAM {
        /// create a fully zero'ed bus
        fn new() -> Self {
            RAM { mem: [0; 0x10000] }
        }
    }

    impl MemoryBus for RAM {
        fn read(&self, address: u16) -> u8 {
            self.mem[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.mem[address as usize] = value
        }
    }

    struct CPU {
        sp: u16,
    }

    impl CPU {
        fn new() -> Self {
            Self { sp: 0 }
        }

        fn stack(&mut self) -> StackControler<'_> {
            StackControler::new(&mut self.sp)
        }
    }

    #[test]
    fn stack_push_pop() {
        let mut cpu = CPU::new();
        let mut ram = RAM::new();

        cpu.sp = 0xFFFE;

        cpu.stack().push_word(&mut ram, 0x1234);
        assert_eq!(cpu.sp, 0xFFFC);
        assert_eq!(ram.mem[0xFFFC..=0xFFFD], [0x12, 0x34]);

        cpu.stack().push_word(&mut ram, 0x5678);
        assert_eq!(cpu.sp, 0xFFFA);
        assert_eq!(ram.mem[0xFFFA..=0xFFFB], [0x56, 0x78]);

        cpu.stack().push_word(&mut ram, 0x9ABC);
        assert_eq!(cpu.sp, 0xFFF8);
        assert_eq!(ram.mem[0xFFF8..=0xFFF9], [0x9A, 0xBC]);

        assert_eq!(cpu.stack().pop_word(&mut ram), 0x9ABC);
        assert_eq!(cpu.sp, 0xFFFA);

        assert_eq!(cpu.stack().pop_word(&mut ram), 0x5678);
        assert_eq!(cpu.sp, 0xFFFC);

        assert_eq!(cpu.stack().pop_word(&mut ram), 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);

        assert_eq!(cpu.stack().pop_word(&mut ram), 0x0000); // "invalid" pop! -- gameboy doesn't prevent this so we just circle back
        assert_eq!(cpu.sp, 0x0000);

        assert_eq!(cpu.stack().pop_word(&mut ram), 0x0000); // "invalid" pop!
        assert_eq!(cpu.sp, 0x0002);
    }

    #[test]
    fn stack_peek() {
        let mut cpu = CPU::new();
        let mut ram = RAM::new();

        cpu.sp = 0xFFFE;

        cpu.stack().push_word(&mut ram, 0x1234);
        assert_eq!(cpu.sp, 0xFFFC); // sp is dec after the push
        assert_eq!(cpu.stack().peek_word(&mut ram), 0x1234);
        assert_eq!(cpu.sp, 0xFFFC); // sp should be unchanged

        cpu.stack().push_word(&mut ram, 0x5678);
        assert_eq!(cpu.sp, 0xFFFA); // sp is dec again after the push
        assert_eq!(cpu.stack().peek_word(&mut ram), 0x5678);
        assert_eq!(cpu.sp, 0xFFFA); // sp should be unchanged
    }
}
