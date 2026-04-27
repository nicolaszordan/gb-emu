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

impl From<u8> for R8Param {
    /// Create a [`R8Param`] from an [`u8`].
    ///
    /// Only the last 3 bits from `value` are checked and therefore the values
    /// go from 0 to 7.
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
    fn from(value: u8) -> Self {
        match value & 0b111 {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::IndHL,
            7 => Self::A,
            _ => unreachable!("all values after mask are mapped"),
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

impl From<u8> for R16Param {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 => Self::SP,
            _ => unreachable!("all values after mask are mapped"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16StackParam {
    BC,
    DE,
    HL,
    AF,
}

impl From<u8> for R16StackParam {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 => Self::AF,
            _ => unreachable!("all values after mask are mapped"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16MemParam {
    IndBC,
    IndDE,
    IndHLi,
    IndHLd,
}

impl From<u8> for R16MemParam {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::IndBC,
            1 => Self::IndDE,
            2 => Self::IndHLi,
            3 => Self::IndHLd,
            _ => unreachable!("all values after mask are mapped"),
        }
    }
}

pub use emu::BitIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LD8SrcParam {
    R8(R8Param),
    R16Mem(R16MemParam),
    N8,
    IndHighMemC,
    IndHighMemA8,
    IndN16,
}

/// Note: N8 isn't a valid destination
pub type LD8DstParam = LD8SrcParam;

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

    /// Designate a parameter using
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

impl From<u8> for ALUOperation {
    /// Create an [`ALUOperation`] from an [`u8`]
    ///
    /// ALU main instructions calls can be deduced from the opcode and are
    /// called in the following order {add, adc, sub, sbc, and, xor, or, cp}
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
    ///
    fn from(value: u8) -> Self {
        match value & 0b111 {
            0 => ALUOperation::ADD,
            1 => ALUOperation::ADC,
            2 => ALUOperation::SUB,
            3 => ALUOperation::SBC,
            4 => ALUOperation::AND,
            5 => ALUOperation::XOR,
            6 => ALUOperation::OR,
            7 => ALUOperation::CP,
            _ => unreachable!("all possible values after mask are mapped"),
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

impl From<u8> for BitRotateOperation {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::RLC,
            1 => Self::RRC,
            2 => Self::RL,
            3 => Self::RR,
            _ => unreachable!("all possible values after mask are mapped"),
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

impl From<u8> for BitShiftOperation {
    fn from(value: u8) -> Self {
        match value & 0b111 {
            0..=3 => Self::Rotate(BitRotateOperation::from(value)),
            4 => Self::SLA,
            5 => Self::SRA,
            6 => Self::SWAP,
            7 => Self::SRL,
            _ => unreachable!("all possible values after mask are mapped"),
        }
    }
}
