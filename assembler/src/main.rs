use anyhow::{bail, Result};
use assembler::assemble;
use std::fs;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let (input_path, output_path) = if let (Some(i), Some(o)) = (args.next(), args.next()) {
        (i, o)
    } else {
        bail!("Usage: <input_path> <output_path>");
    };

    let text = fs::read_to_string(input_path)?;
    if text == "" {
        bail!("Empty input file!");
    }
    let bytecode = assemble(&text)?;

    fs::write(output_path, &bytecode)?;
    Ok(())
}
