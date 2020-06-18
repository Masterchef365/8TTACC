use crate::parser::*;

impl Source {
    fn instruction_bits(&self) -> u8 {
        match self {
            Source::Expansion => 0b00_000000,
            Source::Accumulator => 0b01_000000,
            Source::Memory => 0b10_000000,
            Source::Operand(_) | Source::LabelHi(_) | Source::LabelLo(_) => 0b11_000000,
        }
    }
}

/*
impl Destination {
    fn instruction_bits(&self) -> u8 {
        match self {
            Destination::Memory => 0b00_0000_00,
            Destination::AccumulatorPlus => 0b00_0001_00,
            Destination::AccumulatorNand => 0b00_0010_00,
            Destination::Accumulator => 0b00_0011_00,
            Destination::ProgramCounterLatch => 0b00_0100_00,
            Destination::ProgramCounter => 0b00_0101_00,
            Destination::MemAddressLo => 0b00_0101_00,
        }
    }
}
*/
