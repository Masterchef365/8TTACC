use anyhow::{bail, Result};
use std::fs;
use emulator::Emulator;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = match args.next() {
        Some(v) => v,
        None => bail!("Usage: <file_name.s>"),
    };
    let program = fs::read(path)?;
    let mut emulator = Emulator::from_program(program.into_boxed_slice());
    loop {
        dbg!(&emulator);
        emulator.step()?;
    }
    //Ok(())
}
