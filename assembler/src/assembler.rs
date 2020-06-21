use crate::parser::*;
use common::*;
use std::collections::HashMap;
use thiserror::Error;

pub trait IntoInstruction {
    fn instruction_bits(&self) -> u8;
}

impl IntoInstruction for Source {
    fn instruction_bits(&self) -> u8 {
        match self {
            Source::Expansion => 0b00_000000,
            Source::Accumulator => 0b01_000000,
            Source::Memory => 0b10_000000,
            Source::Operand(_) | Source::LabelHi(_) | Source::LabelLo(_) => 0b11_000000,
        }
    }
}

impl IntoInstruction for Destination {
    fn instruction_bits(&self) -> u8 {
        match self {
            Destination::Memory => 0b00_0000_00,
            Destination::AccumulatorPlus => 0b00_0001_00,
            Destination::AccumulatorNand => 0b00_0010_00,
            Destination::Accumulator => 0b00_0011_00,
            Destination::ProgramCounterLatch => 0b00_0100_00,
            Destination::ProgramCounter => 0b00_0101_00,
            Destination::MemAddressLo => 0b00_0110_00,
            Destination::MemAddressHi => 0b00_0111_00,
            Destination::Serial => 0b00_1000_00,
            Destination::Led => 0b00_1001_00,
            Destination::CarrySet => 0b00_1010_00,
            Destination::CarryReset => 0b00_1011_00,
            Destination::ExpansionSelect => 0b00_1100_00,
        }
    }
}

impl IntoInstruction for Operation {
    fn instruction_bits(&self) -> u8 {
        let src = self.src.instruction_bits();
        let dest = self.dest.instruction_bits();
        let if_carry = if self.cond_carry { 0b000000_01 } else { 0 };
        let if_one = if self.cond_1 { 0b000000_10 } else { 0 };
        src | dest | if_carry | if_one
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AssemblerError {
    #[error("Label repeated {label}, line: {line}")]
    RepeatLabel { label: String, line: usize },
    #[error("Unrecognized label \"{label}\", line: {line}")]
    UnrecognizedLabel { label: String, line: usize },
    #[error("Forbidden instruction, line: {line}")]
    ForbiddenInstruction { line: usize },
}

pub fn assemble(lines: &[(Statement, usize)]) -> Result<Vec<u8>, AssemblerError> {
    let mut labels = HashMap::new();
    let mut pc: u16 = 0;
    for (statement, line) in lines {
        match statement {
            Statement::Label(label) => {
                if labels.insert(label.clone(), pc).is_some() {
                    Err(AssemblerError::RepeatLabel {
                        line: *line,
                        label: label.clone(),
                    })?;
                }
            }
            Statement::Operation(op) => match op.src {
                Source::Operand(_) | Source::LabelLo(_) | Source::LabelHi(_) => pc += 2,
                _ => pc += 1,
            },
        }
    }

    let mut bytecode = Vec::new();
    for (statement, line) in lines {
        let op = match statement {
            Statement::Operation(op) => op,
            Statement::Label(_) => continue,
        };
        bytecode.push(op.instruction_bits());
        let get_label_pc = |label: &String| match labels.get(label) {
            Some(pc) => Ok(*pc),
            None => Err(AssemblerError::UnrecognizedLabel {
                label: label.clone(),
                line: *line,
            }),
        };
        match op.src {
            Source::Operand(op) => bytecode.push(op),
            Source::LabelHi(ref label) => bytecode.push((get_label_pc(label)? >> 8) as u8),
            Source::LabelLo(ref label) => bytecode.push((get_label_pc(label)? & 0x00FF) as u8),
            _ => (),
        }
        if let Source::Operand(_) | Source::LabelLo(_) | Source::LabelHi(_) = op.src {
            if op.dest == Destination::Memory {
                Err(AssemblerError::ForbiddenInstruction { line: *line })?;
            }
        }
    }
    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_operation_bits() {
        assert_eq!(
            Operation {
                src: Source::Accumulator,
                dest: Destination::Accumulator,
                cond_1: false,
                cond_carry: false
            }
            .instruction_bits(),
            0b01_0011_00
        );
        assert_eq!(
            Operation {
                src: Source::Operand(0x00),
                dest: Destination::CarrySet,
                cond_1: false,
                cond_carry: true
            }
            .instruction_bits(),
            0b11_1010_01
        );
    }

    #[test]
    fn test_assembler() {
        let instructions = [
            (
                Statement::Operation(Operation {
                    src: Source::Accumulator,
                    dest: Destination::Accumulator,
                    cond_carry: false,
                    cond_1: false,
                }),
                1,
            ),
            (Statement::Label("loop".into()), 2),
            (
                Statement::Operation(Operation {
                    src: Source::LabelLo("loop".into()),
                    dest: Destination::ProgramCounterLatch,
                    cond_carry: false,
                    cond_1: false,
                }),
                2,
            ),
            (
                Statement::Operation(Operation {
                    src: Source::LabelHi("loop".into()),
                    dest: Destination::ProgramCounter,
                    cond_carry: false,
                    cond_1: false,
                }),
                3,
            ),
        ];
        let expected_bytecode = vec![
            0b01_0011_00,
            0b11_0100_00,
            0b0000000001,
            0b11_0101_00,
            0b0000000000,
        ];
        assert_eq!(assemble(&instructions), Ok(expected_bytecode));
    }
}
