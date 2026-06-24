pub use emu::BitIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R8Param {
    /// Designate a parameter using CPU's `B` register
    B,

    /// Designate a parameter using CPU's `C` register
    C,

    /// Designate a parameter using CPU's `D` register
    D,

    /// Designate a parameter using CPU's `E` register
    E,

    /// Designate a parameter using CPU's `H` register
    H,

    /// Designate a parameter using CPU's `L` register
    L,

    /// Designate a parameter using the value stored in `RAM` at the address
    /// `HL`
    IndHL,

    /// Designate a parameter using CPU's `A` register
    A,
}

impl R8Param {
    /// Create an [`R8Param`] based on the lower **3** bits of `value`.
    ///
    /// Mapping is as follow:
    /// - `0` => [`R8Param::B`]
    /// - `1` => [`R8Param::C`]
    /// - `2` => [`R8Param::D`]
    /// - `3` => [`R8Param::E`]
    /// - `4` => [`R8Param::H`]
    /// - `5` => [`R8Param::L`]
    /// - `6` => [`R8Param::IndHL`]
    /// - `7` => [`R8Param::A`]
    pub const fn from_low_bits(value: u8) -> Self {
        match value & 0b111 {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::IndHL,
            7 => Self::A,
            _ => unreachable!(), // all possible values are mapped
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16Param {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16StackParam {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16MemParam {
    IndBC,
    IndDE,
    IndHLi,
    IndHLd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LD8SrcParam {
    R8(R8Param),
    R16Mem(R16MemParam),
    N8,
    IndHighMemC,
    IndHighMemA8,
    IndN16,
}

impl From<R8Param> for LD8SrcParam {
    fn from(value: R8Param) -> Self {
        Self::R8(value)
    }
}

impl From<R16MemParam> for LD8SrcParam {
    fn from(value: R16MemParam) -> Self {
        Self::R16Mem(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LD8DstParam {
    R8(R8Param),
    R16Mem(R16MemParam),
    IndHighMemC,
    IndHighMemA8,
    IndN16,
}

impl From<R8Param> for LD8DstParam {
    fn from(value: R8Param) -> Self {
        Self::R8(value)
    }
}

impl From<R16MemParam> for LD8DstParam {
    fn from(value: R16MemParam) -> Self {
        Self::R16Mem(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LD16SrcParam {
    HL,
    SP,
    N16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LD16DstParam {
    R16(R16Param),
    IndN16,
}

impl From<R16Param> for LD16DstParam {
    fn from(value: R16Param) -> Self {
        Self::R16(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ALU8Param {
    R8(R8Param),
    N8,
}

impl From<R8Param> for ALU8Param {
    fn from(value: R8Param) -> Self {
        Self::R8(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallParam {
    /// Designate a parameter using the next 16bits after the current
    /// instruction as an absolute address for the call.
    N16,

    /// Designate a parameter using a jump vector table. Values go from 0x00
    /// to 0x38 with a step of 0x08.
    VEC(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpParam {
    /// Designate a parameter using the next 8bits after the current
    /// instruction as a **signed** [`i8`]. The jump will add this signed to
    /// the [`CPU::pc`] to perform the jump.
    PCE8,

    /// Designate a parameter using the next 16bits after the current
    /// instruction as an absolute address for the jump.
    N16,

    /// Designate a parameter using the value contained in the CPU register
    /// [`Registers::hl`] as an absolute address for the jump.
    HL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSPe8DstParam {
    HL,
    SP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ALUOperation {
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    XOR,
    OR,
    CP,
}

impl ALUOperation {
    /// Create an [`ALUOperation`] from the lower **3** bits of `value`.
    ///
    /// Value mapping is as follow:
    /// - `0` => [`ALUOperation::ADD`]
    /// - `1` => [`ALUOperation::ADC`]
    /// - `2` => [`ALUOperation::SUB`]
    /// - `3` => [`ALUOperation::SBC`]
    /// - `4` => [`ALUOperation::AND`]
    /// - `5` => [`ALUOperation::XOR`]
    /// - `6` => [`ALUOperation::OR`]
    /// - `7` => [`ALUOperation::CP`]
    pub const fn from_low_bits(value: u8) -> Self {
        match value & 0b111 {
            0 => Self::ADD,
            1 => Self::ADC,
            2 => Self::SUB,
            3 => Self::SBC,
            4 => Self::AND,
            5 => Self::XOR,
            6 => Self::OR,
            7 => Self::CP,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitRotateOperation {
    RLC,
    RRC,
    RL,
    RR,
}

impl BitRotateOperation {
    /// Create a [`BitRotateOperation`] from the lower **2** bits of `value`.
    ///
    /// Value mapping is as follow:
    /// - `0` => [`BitRotateOperation::RLC`]
    /// - `1` => [`BitRotateOperation::RRC`]
    /// - `2` => [`BitRotateOperation::RL`]
    /// - `3` => [`BitRotateOperation::RR`]
    pub const fn from_low_bits(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::RLC,
            1 => Self::RRC,
            2 => Self::RL,
            3 => Self::RR,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitShiftOperation {
    Rotate(BitRotateOperation),
    SLA,
    SRA,
    SWAP,
    SRL,
}

impl BitShiftOperation {
    /// Create a [`BitShiftOperation`] from the lower **3** bits of `value`.
    ///
    /// Value mapping is as follow:
    /// - `0` => [`BitRotateOperation::RLC`]
    /// - `1` => [`BitRotateOperation::RRC`]
    /// - `2` => [`BitRotateOperation::RL`]
    /// - `3` => [`BitRotateOperation::RR`]
    /// - `4` => [`BitShiftOperation::SLA`]
    /// - `5` => [`BitShiftOperation::SRA`]
    /// - `6` => [`BitShiftOperation::SWAP`]
    /// - `7` => [`BitShiftOperation::SRL`]
    pub const fn from_low_bits(value: u8) -> Self {
        match value & 0b111 {
            0..=3 => Self::Rotate(BitRotateOperation::from_low_bits(value)),
            4 => Self::SLA,
            5 => Self::SRA,
            6 => Self::SWAP,
            7 => Self::SRL,
            _ => unreachable!(),
        }
    }
}
