use crate::parser::*;
use anyhow::{bail, Result};
use common::*;
mod assembler;
mod parser;

pub fn assemble(text: &str) -> Result<Vec<u8>> {
    let lines = text.lines().map(|s| s.to_string()).collect::<Vec<_>>();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembler() {
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
        let expected_bytecode = vec![
            0b01001100, 0b11100100, 0b01011111, 0b11010010, 0b00000000, 0b11010000, 0b00000001,
            0b11010100, 0b00000000, 0b11011011, 0b01010101, 0b11011111, 0b11111111, 0b11010000,
            0b00001101,
        ];
        assert_eq!(assemble(&text).unwrap(), expected_bytecode);
    }

    #[test]
    fn test_assembler2() {
        let text = "
00 -> PC.latch
00 -> RAM.high
00 -> RAM.low
00 -> ACC
ACC -> RAM
main_loop:
00 -> ACC
ACC -> carry.reset
delay_loop:
01 -> ACC.plus
lo@out_of_loop -> PC : if_1
lo@delay_loop -> PC
out_of_loop:
RAM -> ACC
ACC -> LED
ACC -> carry.reset
01 -> ACC.plus
ACC -> RAM
lo@main_loop -> PC";
        let binary = vec![
            0b01_0011_00,
            0b11_0100_00,
            0b00_0000_00,
            0b11_0111_00,
            0b00_0000_00,
            0b11_0110_00,
            0b00_0000_00,
            0b11_0011_00,
            0b00_0000_00,
            0b01_0000_00,
            0b11_0011_00,
            0b00_0000_00,
            0b01_1011_00,
            0b11_0001_00,
            0b00_0000_01,
            0b11_0101_10,
            0b00_0100_11,
            0b11_0101_00,
            0b00_0011_01,
            0b10_0011_00,
            0b01_1001_00,
            0b01_1011_00,
            0b11_0001_00,
            0b00_0000_01,
            0b01_0000_00,
            0b11_0101_00,
            0b00_0010_10,
        ];
        assert_eq!(assemble(&text).unwrap(), binary);
    }

    #[test]
    #[should_panic]
    fn test_assembler_err() {
        assemble("8F -> RAM").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_assembler_err2() {
        assemble("ACC.plus -> LED").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_assembler_err3() {
        assemble(
            "loop
            lo@loop -> RAM",
        )
        .unwrap();
    }
}
