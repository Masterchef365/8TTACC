use anyhow::{bail, Result};
use common::*;
use std::fs;
use std::io::{BufRead, BufReader, Read};
mod decoder;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = match args.next() {
        Some(v) => v,
        None => bail!("Usage: <file_name.s>"),
    };
    let text_segment = fs::read(path)?;
    //let instructions: Vec<_> = InstructionDecoder::new(&text_segment).collect();
    Ok(())
}

type Word = u8;

#[derive(Debug, Default)]
struct Accumulator {
    value: Word,
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

struct InstructionPointer {
    value: u16,
}

impl InstructionPointer {
    fn latch(&mut self, value: Word) {

    }

    fn set(&mut self, value: Word) {
    }
}

#[derive(Default)]
struct Emulator {
    program: Box<[u8]>,
    acc: Accumulator,
    flag_1: bool,
    flag_carry: bool,
    pc: usize,
}

impl Emulator {
    pub fn from_program(program: Box<[u8]>) -> Self {
        Self {
            program,
            ..Default::default()
        }
    }

    pub fn step(&mut self) {
        let instruction = decoder::read_operation(&self.program, &mut self.pc);
    }
}
