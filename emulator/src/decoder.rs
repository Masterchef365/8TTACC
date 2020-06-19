use common::*;
use std::io::Read;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum DecoderError {
    #[error("Unrecognized destination")]
    UnrecognizedDestination,
    #[error("Tried to read an instruction off the end of the data segment")]
    InvalidRead,
}

pub trait FromByte: Sized {
    fn from_byte(byte: u8) -> Result<Self, DecoderError>;
}

impl FromByte for Destination {
    fn from_byte(byte: u8) -> Result<Self, DecoderError> {
        match byte & 0b00_1111_00 {
            0b00_0000_00 => Ok(Destination::Memory),
            0b00_0001_00 => Ok(Destination::AccumulatorPlus),
            0b00_0010_00 => Ok(Destination::AccumulatorNand),
            0b00_0011_00 => Ok(Destination::Accumulator),
            0b00_0100_00 => Ok(Destination::ProgramCounterLatch),
            0b00_0101_00 => Ok(Destination::ProgramCounter),
            0b00_0110_00 => Ok(Destination::MemAddressLo),
            0b00_0111_00 => Ok(Destination::MemAddressHi),
            0b00_1001_00 => Ok(Destination::Led),
            0b00_1010_00 => Ok(Destination::CarrySet),
            0b00_1011_00 => Ok(Destination::CarryReset),
            _ => Err(DecoderError::UnrecognizedDestination),
        }
    }
}

impl FromByte for Source {
    fn from_byte(byte: u8) -> Result<Self, DecoderError> {
        Ok(match byte & 0b11_000000 {
            0b00_000000 => Source::Expansion,
            0b01_000000 => Source::Accumulator,
            0b10_000000 => Source::Memory,
            0b11_000000 => Source::Operand(0x00),
            _ => unreachable!(),
        })
    }
}

impl FromByte for Operation {
    fn from_byte(byte: u8) -> Result<Self, DecoderError> {
        let cond_carry = byte & 0b000000_01 == 0b000000_01;
        let cond_1 = byte & 0b000000_10 == 0b000000_10;
        Ok(Operation {
            src: Source::from_byte(byte)?,
            dest: Destination::from_byte(byte)?,
            cond_carry,
            cond_1,
        })
    }
}

/// Reads an operation and advances the instruction pointer accordingly
pub fn read_operation(buf: &[u8], program_counter: &mut usize) -> Result<Operation, DecoderError> {
    let instruction = *buf.get(*program_counter).ok_or(DecoderError::InvalidRead)?;
    *program_counter += 1;
    let mut op = Operation::from_byte(instruction)?;
    if let Source::Operand(value) = &mut op.src {
        *value = *buf.get(*program_counter).ok_or(DecoderError::InvalidRead)?;
        *program_counter += 1;
    }
    Ok(op)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assembler::assemble;

    #[test]
    fn test_roundtrip() {
        let text = "
im_a_label:
5F -> LED
00 -> PC.latch : if_1
lo@im_a_label -> PC.latch
hi@im_a_label -> PC
55 -> RAM.low : if_carry | if_1
FF -> RAM.high : if_1 | if_carry
im_also_a_label:
lo@im_also_a_label -> PC.latch";
        let bytecode = assemble(text).unwrap();
        let mut program_counter = 0;
        let mut ops = Vec::new();
        while program_counter < bytecode.len() {
            ops.push(read_operation(&bytecode, &mut program_counter).unwrap());
        }
        let expected_ops = vec![
            Operation {
                src: Source::Accumulator,
                dest: Destination::Accumulator,
                cond_1: false,
                cond_carry: false,
            },
            Operation {
                src: Source::Operand(0x5F),
                dest: Destination::Led,
                cond_1: false,
                cond_carry: false,
            },
            Operation {
                src: Source::Operand(0x00),
                dest: Destination::ProgramCounterLatch,
                cond_1: true,
                cond_carry: false,
            },
            Operation {
                src: Source::Operand(0x01),
                dest: Destination::ProgramCounterLatch,
                cond_1: false,
                cond_carry: false,
            },
            Operation {
                src: Source::Operand(0x00),
                dest: Destination::ProgramCounter,
                cond_1: false,
                cond_carry: false,
            },
            Operation {
                src: Source::Operand(0x55),
                dest: Destination::MemAddressLo,
                cond_1: true,
                cond_carry: true,
            },
            Operation {
                src: Source::Operand(0xFF),
                dest: Destination::MemAddressHi,
                cond_1: true,
                cond_carry: true,
            },
            Operation {
                src: Source::Operand(13),
                dest: Destination::ProgramCounterLatch,
                cond_1: false,
                cond_carry: false,
            },
        ];
        assert_eq!(ops, expected_ops)
    }
}
