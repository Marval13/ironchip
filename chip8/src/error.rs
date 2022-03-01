#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ChipError {
    PcOutOfBounds(u16),
    SpOutOfBounds(usize),
    RomTooBig(usize),
    UnrecognizedOpcode(u16),
}

impl std::fmt::Display for ChipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ChipError::PcOutOfBounds(n) => write!(f, "Program counter out of bounds: {}", n),
            ChipError::SpOutOfBounds(n) => write!(f, "Stack pointer out of bounds: {}", n),
            ChipError::RomTooBig(n) => write!(f, "Rom too big: {}/3584 bytes", n),
            ChipError::UnrecognizedOpcode(op) => write!(f, "Unrecognized opcode: {:#06X}", op),
        }
    }
}

impl std::error::Error for ChipError {}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum DebugChipError {
    AddrOutOfBounds(usize),
    PcOutOfBounds(u16),
    StackAddrOutOfBounds(u16),
    SpOutOfBounds(usize),
    IndexTooBig(u16),
    NoRegister(usize),
    NoKey(usize),
    NoPixel(usize, usize),
}

impl std::fmt::Display for DebugChipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DebugChipError::AddrOutOfBounds(n) => write!(f, "address out of bounds: {:#06X}", n),
            DebugChipError::PcOutOfBounds(n) => write!(f, "pc out of bounds: {:#06X}", n),
            DebugChipError::StackAddrOutOfBounds(n) => {
                write!(f, "address out of bounds: {:#06X}", n)
            }
            DebugChipError::SpOutOfBounds(n) => write!(f, "sp out of bounds: {}", n),
            DebugChipError::IndexTooBig(n) => write!(f, "index too big: {:#06X}", n),
            DebugChipError::NoRegister(n) => write!(f, "no such register: {:#03X}", n),
            DebugChipError::NoKey(n) => write!(f, "no such key: {:#03X}", n),
            DebugChipError::NoPixel(x, y) => write!(f, "pixel out of bounds: ({}, {})", x, y),
        }
    }
}

impl std::error::Error for DebugChipError {}
