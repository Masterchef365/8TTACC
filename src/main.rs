use std::fs::File;
use std::io::{BufRead, BufReader, Write};
mod parser;
use parser::*;
mod assembler;
use anyhow::{bail, Result};

fn assemble(lines: &[String]) -> Result<Vec<u8>> {
    let mut statements = Vec::new();

    statements.push((
        Statement::Operation(Operation {
            src: Source::Accumulator,
            dest: Destination::Accumulator,
            cond_carry: false,
            cond_1: false,
        }),
        0,
    ));

    for (line_number, line) in lines.iter().enumerate() {
        let line_number = line_number + 1;
        match parse_line(&line) {
            Err(e) => bail!("Parser error on line {}; {:?}", line_number, e),
            Ok((_, Some(s))) => statements.push((s, line_number)),
            _ => (),
        }
    }

    Ok(assembler::assemble(statements.as_slice())?)
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let (input_path, output_path) = if let (Some(i), Some(o)) = (args.next(), args.next()) {
        (i, o)
    } else {
        bail!("Usage: <input_path> <output_path>");
    };

    let file = BufReader::new(File::open(input_path)?);
    let lines = file.lines().collect::<Result<Vec<_>, std::io::Error>>()?;
    let bytecode = assemble(&lines)?;

    File::create(output_path)?.write_all(&bytecode)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembler() {
        let lines: [String; 9] = [
            "im_a_label:".into(),
            "5F -> LED".into(),
            "00 -> PC.latch : if_1".into(),
            "lo@im_a_label -> PC.latch".into(),
            "hi@im_a_label -> PC".into(),
            "55 -> RAM.low : if_carry | if_1".into(),
            "FF -> RAM.high : if_1 | if_carry".into(),
            "im_also_a_label:".into(),
            "lo@im_also_a_label -> PC.latch".into(),
        ];
        let expected_bytecode = vec![
            0b01001100, 0b11100100, 0b01011111, 0b11010010, 0b00000000, 0b11010000, 0b00000001,
            0b11010100, 0b00000000, 0b11011011, 0b01010101, 0b11011111, 0b11111111, 0b11010000,
            0b00001101,
        ];
        assert_eq!(assemble(&lines).unwrap(), expected_bytecode);

        //8F -> RAM                       // Assembler will throw an error!
        //                 // Assembler will throw an error!
    }

    #[test]
    #[should_panic]
    fn test_assembler_err() {
        assemble(&["8F -> RAM".into()]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_assembler_err2() {
        assemble(&["ACC.plus -> LED".into()]).unwrap();
    }
}
