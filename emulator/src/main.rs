use anyhow::{Result, bail};
use std::fs;
use std::io::{BufReader, BufRead, Read};
use common::*;
mod decoder;
use decoder::*;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = match args.next() {
        Some(v) => v,
        None => bail!("Usage: <file_name.s>"),
    };
    let text_segment = fs::read(path)?;
    for instruction in InstructionDecoder::new(&text_segment) {
        dbg!(instruction);
    }
    Ok(())
}

struct InternalState {}

impl InternalState {
    pub fn new() -> Self {
        InternalState {}
    }

    pub fn step(&mut self, text_segment: &[Operation]) {
    }
}
