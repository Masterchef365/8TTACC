use thiserror::Error;
mod decoder;
use common::*;
use std::io::{self, Read, Write};

#[derive(Default, Debug)]
pub struct Emulator {
    pub flag_1: bool,
    pub flag_carry: bool,
    pub program: Box<[u8]>,
    pub pc: ProgramCounter,
    pub acc: Accumulator,
    pub mem: Memory,
    pub led: Led,
    pub ser: Serial,
}

impl Emulator {
    pub fn from_program(program: Box<[u8]>) -> Result<Self, EmulatorError> {
        let first_instruction = *program
            .get(0)
            .ok_or(EmulatorError::Decoder(decoder::DecoderError::InvalidRead))?;
        if first_instruction != 0b01001100 {
            return Err(EmulatorError::MissingNop);
        }
        Ok(Self {
            program,
            ..Default::default()
        })
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
            let word = self.pull(op.src);
            self.push(op.dest, word);
        }

        Ok(())
    }

    pub fn pull(&mut self, src: Source) -> Word {
        match src {
            Source::Operand(value) => value,
            Source::Accumulator => self.acc.get(),
            Source::Memory => self.mem.read(),
            Source::Serial => self.ser.read(),
            _ => unreachable!("This should always be an operand..."),
        }
    }

    pub fn push(&mut self, dest: Destination, value: Word) {
        match dest {
            Destination::ProgramCounterLatch => self.pc.latch(value),
            Destination::ProgramCounter => self.pc.jump(value),
            Destination::Accumulator => self.flag_1 = self.acc.set(value),
            Destination::AccumulatorPlus => self.flag_carry = self.acc.add(value),
            Destination::AccumulatorNand => self.acc.nand(value),
            Destination::Led => self.led.set(value),
            Destination::Memory => self.mem.write(value),
            Destination::MemAddressLo => self.mem.latch_low(value),
            Destination::MemAddressHi => self.mem.latch_high(value),
            Destination::Serial => self.ser.write(value),
            Destination::CarrySet => self.flag_carry = true,
            Destination::CarryReset => self.flag_carry = false,
        }
    }
}

pub type Word = u8;

#[derive(Debug, Clone, Error)]
pub enum EmulatorError {
    #[error(transparent)]
    Decoder(#[from] decoder::DecoderError), // Derive From?
    #[error("Illegal instruction")]
    Illegal(Operation),
    #[error("Missing initial NOP")]
    MissingNop,
}

#[derive(Debug, Default)]
pub struct Accumulator {
    pub value: Word,
}

impl Accumulator {
    pub fn nand(&mut self, value: Word) {
        self.value = !(self.value & value);
    }

    /// Returns true if the carry flag is set
    pub fn add(&mut self, value: Word) -> bool {
        let (new_val, carry) = self.value.overflowing_add(value);
        self.value = new_val;
        carry
    }

    /// Returns true of the one flag is set
    pub fn set(&mut self, value: Word) -> bool {
        self.value = value;
        value == 0b1111_1111
    }

    pub fn get(&self) -> Word {
        self.value
    }
}

#[derive(Default, Debug)]
pub struct ProgramCounter {
    pub value: u16,
    pub latch: Word,
}

impl ProgramCounter {
    pub fn latch(&mut self, value: Word) {
        self.latch = value;
    }

    pub fn jump(&mut self, value: Word) {
        self.value = ((self.latch as u16) << 8) + (value as u16);
    }

    pub fn set(&mut self, value: u16) {
        self.value = value;
    }

    pub fn advance(&mut self, advance: u16) {
        self.value = self.value.wrapping_add(advance);
    }

    pub fn get(&self) -> u16 {
        self.value
    }
}

#[derive(Default, Debug)]
pub struct Led {
    pub value: Word,
}

impl Led {
    pub fn set(&mut self, value: Word) {
        self.value = value;
    }

    pub fn get(&self) -> Word {
        self.value
    }
}

#[derive(Default, Debug)]
pub struct Memory {
    pub values: Vec<Word>,
    pub low_latch: Word,
    pub hi_latch: Word,
}

impl Memory {
    pub fn latch_low(&mut self, value: Word) {
        self.low_latch = value;
    }

    pub fn latch_high(&mut self, value: Word) {
        self.hi_latch = value;
    }

    pub fn address(&self) -> u16 {
        (self.low_latch as u16) + ((self.hi_latch as u16) << 8)
    }

    fn expand(&mut self) {
        let address = self.address() as usize;
        if address >= self.values.len() {
            self.values.resize_with(address + 1, || 0);
        }
    }

    pub fn read(&mut self) -> Word {
        self.expand();
        self.values[self.address() as usize]
    }

    pub fn write(&mut self, value: Word) {
        self.expand();
        let addr = self.address() as usize;
        self.values[addr] = value;
    }
}

#[derive(Debug)]
pub struct Serial {
    in_stream: io::Stdin,
    out_stream: io::Stdout,
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            in_stream: io::stdin(),
            out_stream: io::stdout(),
        }
    }
}

impl Serial {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, value: Word) {
        self.out_stream.lock().write(&[value]).expect("Stdout error");
        self.out_stream.flush().unwrap();
    }

    pub fn read(&mut self) -> Word {
        let mut buf = [0u8];
        self.in_stream.lock().read(&mut buf).expect("Stdin error");
        buf[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_counter() {
        let mut pc = ProgramCounter::default();
        pc.latch(0x5F);
        assert_eq!(pc.get(), 0x0000);
        pc.jump(0xA7);
        assert_eq!(pc.get(), 0x5FA7);
        pc.advance(0x02);
        assert_eq!(pc.get(), 0x5FA9);
    }

    #[test]
    fn test_accumulator() {
        let mut acc = Accumulator::default();
        assert_eq!(acc.get(), 0x00);
        acc.set(0b10100101);
        acc.nand(0b01101110);
        assert_eq!(acc.get(), 0b11011011);
        acc.set(0x5F);
        assert!(acc.add(0xF0));
        assert_eq!(acc.get(), 0x4F);
    }

    #[test]
    fn test_memory() {
        let mut mem = Memory::default();
        assert_eq!(mem.address(), 0x0000);
        mem.latch_high(0xFF);
        assert_eq!(mem.address(), 0xFF00);
        assert_eq!(mem.read(), 0x00);
        mem.write(0x57);
        assert_eq!(mem.read(), 0x57);
        mem.latch_low(0x88);
        assert_eq!(mem.address(), 0xFF88);
        assert_eq!(mem.read(), 0x00);
        mem.write(0x55);
        assert_eq!(mem.read(), 0x55);
    }
}
