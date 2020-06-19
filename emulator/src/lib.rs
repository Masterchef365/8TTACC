use thiserror::Error;
mod decoder;
use common::*;

pub type Word = u8;

#[derive(Debug, Default)]
pub struct Accumulator {
    pub value: Word,
}

impl Accumulator {
    fn nand(&mut self, value: Word) {
        self.value = !(self.value & value);
    }

    /// Returns true if the carry flag is set
    fn add(&mut self, value: Word) -> bool {
        let (new_val, carry) = self.value.overflowing_add(value);
        self.value = new_val;
        carry
    }

    /// Returns true of the one flag is set
    fn set(&mut self, value: Word) -> bool {
        self.value = value;
        value == 0b1111_1111
    }

    fn get(&self) -> Word {
        self.value
    }
}

#[derive(Default, Debug)]
pub struct ProgramCounter {
    pub value: u16,
}

impl ProgramCounter {
    fn set_lo(&mut self, value: Word) {
        self.value = self.value & 0xFF00 + value as u16;
    }

    fn set_hi(&mut self, value: Word) {
        self.value = self.value & 0x00FF + ((value as u16) << 8);
    }

    fn set(&mut self, value: u16) {
        self.value = value;
    }

    fn advance(&mut self, advance: u16) {
        self.value = self.value.wrapping_add(advance);
    }

    fn get(&self) -> u16 {
        self.value
    }
}

#[derive(Debug, Clone, Error)]
pub enum EmulatorError {
    #[error("Instruction decoder failed, {0}")]
    Decoder(#[from] decoder::DecoderError), // Derive From?
    #[error("Illegal instruction")]
    Illegal(Operation),
}

#[derive(Default, Debug)]
pub struct Emulator {
    pub flag_1: bool,
    pub flag_carry: bool,
    pub program: Box<[u8]>,
    pub pc: ProgramCounter,
    pub acc: Accumulator,
}

impl Emulator {
    pub fn from_program(program: Box<[u8]>) -> Self {
        Self {
            program,
            ..Default::default()
        }
    }

    pub fn step(&mut self) -> Result<(), EmulatorError> {
        let (op, advance) = decoder::read_operation(&self.program, self.pc.get() as usize)?;
        self.pc.advance(advance as u16);

        if let (Source::Operand(_), Destination::Memory) = (&op.src, &op.dest) {
            return Err(EmulatorError::Illegal(op));
        }

        let execute = match (op.cond_1, op.cond_carry) {
            (false, false) => true,
            (true, false) => self.flag_1,
            (false, true) => self.flag_carry,
            (true, true) => self.flag_carry || self.flag_1,
        };

        if execute {
            self.push(op.dest, self.pull(op.src));
        }

        Ok(())
    }

    pub fn pull(&self, src: Source) -> Word {
        match src {
            Source::Operand(value) => value,
            Source::Accumulator => self.acc.get(),
            _ => todo!("{:?}", src),
        }
    }

    pub fn push(&mut self, dest: Destination, value: Word) {
        match dest {
            Destination::Accumulator => self.flag_1 = self.acc.set(value),
            Destination::AccumulatorPlus => self.flag_carry = self.acc.add(value),
            Destination::AccumulatorNand => self.acc.nand(value),
            _ => todo!("{:?}", dest),
        }
    }
}
