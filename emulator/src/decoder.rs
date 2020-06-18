use common::*;

pub trait FromByte {
    fn from_byte(byte: u8) -> Self;
}

impl FromByte for Destination {
    fn from_byte(byte: u8) -> Self {
        match byte & 0b00_1111_00 {
            0b00_0000_00 => Destination::Memory,
            0b00_0001_00 => Destination::AccumulatorPlus,
            0b00_0010_00 => Destination::AccumulatorNand,
            0b00_0011_00 => Destination::Accumulator,
            0b00_0100_00 => Destination::ProgramCounterLatch,
            0b00_0101_00 => Destination::ProgramCounter,
            0b00_0110_00 => Destination::MemAddressLo,
            0b00_0111_00 => Destination::MemAddressHi,
            0b00_1001_00 => Destination::Led,
            0b00_1010_00 => Destination::CarrySet,
            0b00_1011_00 => Destination::CarryReset,
        }
    }
}

impl FromByte for Source {
    fn from_byte(byte: u8) -> Self {
        match byte & 0b11_000000 {
            0b00_000000 => Source::Expansion,
            0b01_000000 => Source::Accumulator,
            0b10_000000 => Source::Memory,
            0b11_000000 => Source::Operand(0x00),
        }
    }
}

pub struct InstructionDecoder<'a> {
    bin: &'a [u8],
    idx: usize,
}

impl<'a> InstructionDecoder<'a> {
    pub fn new(bin: &'a [u8]) -> Self {
        Self { bin, idx: 0 }
    }
}

/*
impl Iterator for InstructionDecoder<'_> {
    type Item = Result<Operation>;
    fn next(&mut self) -> Option<Self::Item> {

    }
}
*/
