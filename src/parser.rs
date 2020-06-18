#![allow(unused_imports)]
#![allow(dead_code)]
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::character::*;
use nom::combinator::*;
use nom::error::ErrorKind;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;
use nom::{AsChar, InputTakeAtPosition};

type Label = String;

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    Expansion,
    Accumulator,
    Memory,
    Operand(u8),
    LabelLo(Label),
    LabelHi(Label),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Destination {
    Memory,
    MemAddressLo,
    MemAddressHi,
    Accumulator,
    AccumulatorPlus,
    AccumulatorNand,
    ProgramCounter,
    ProgramCounterLatch,
    Led,
    CarrySet,
    CarryReset,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Operation {
    pub src: Source,
    pub dest: Destination,
    pub cond_1: bool,
    pub cond_carry: bool,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Label(Label),
    Operation(Operation),
}

fn parse_hex(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, |c: char| c.is_digit(16)), |s| {
        u8::from_str_radix(s, 16)
    })(input)
}

fn parse_name<'a>(s: &'a str) -> IResult<&'a str, &'a str> {
    s.split_at_position1_complete(|item| !item.is_alphanum() && item != '_', ErrorKind::NoneOf)
}

fn parse_source(s: &str) -> IResult<&str, Source> {
    alt((
        map(tag("EXP"), |_| Source::Expansion),
        map(tag("ACC"), |_| Source::Accumulator),
        map(tag("RAM"), |_| Source::Memory),
        map(preceded(tag("lo@"), parse_name), |label| {
            Source::LabelLo(label.to_string())
        }),
        map(preceded(tag("hi@"), parse_name), |label| {
            Source::LabelHi(label.to_string())
        }),
        map(parse_hex, |op| Source::Operand(op)),
    ))(s)
}

fn parse_destination(s: &str) -> IResult<&str, Destination> {
    alt((
        map(tag("RAM.low"), |_| Destination::MemAddressLo),
        map(tag("RAM.high"), |_| Destination::MemAddressHi),
        map(tag("RAM"), |_| Destination::Memory),
        map(tag("ACC.plus"), |_| Destination::AccumulatorPlus),
        map(tag("ACC.nand"), |_| Destination::AccumulatorNand),
        map(tag("ACC"), |_| Destination::Accumulator),
        map(tag("PC.latch"), |_| Destination::ProgramCounterLatch),
        map(tag("PC"), |_| Destination::ProgramCounter),
        map(tag("LED"), |_| Destination::Led),
        map(tag("carry.set"), |_| Destination::CarrySet),
        map(tag("carry.reset"), |_| Destination::CarryReset),
    ))(s)
}

fn parse_operation(s: &str) -> IResult<&str, Operation> {
    let arrow = delimited(space1, tag("->"), space1);
    let colon = delimited(space1, tag(":"), space1);
    let bar = || delimited(space1, tag("|"), space1);
    let mov = separated_pair(parse_source, arrow, parse_destination);
    let one = || tag("if_1");
    let carry = || tag("if_carry");

    let conditions = alt((
        map(separated_pair(carry(), bar(), one()), |_| (true, true)),
        map(separated_pair(one(), bar(), carry()), |_| (true, true)),
        map(one(), |_| (true, false)),
        map(carry(), |_| (false, true)),
    ));

    let conditions = alt((
        preceded(colon, conditions),
        map(tag(""), |_| (false, false)),
    ));

    map(
        tuple((mov, conditions)),
        |((src, dest), (cond_1, cond_carry))| Operation {
            src,
            dest,
            cond_1,
            cond_carry,
        },
    )(s)
}

fn parse_label(s: &str) -> IResult<&str, Label> {
    map(terminated(parse_name, tag(":")), |s| s.to_string())(s)
}

fn parse_statement(s: &str) -> IResult<&str, Statement> {
    alt((
        map(parse_operation, Statement::Operation),
        map(parse_label, Statement::Label),
    ))(s)
}

pub fn parse_line(s: &str) -> IResult<&str, Option<Statement>> {
    alt((
        map(parse_statement, Some),
        map(tag("//"), |_| None),
        map(all_consuming(space0), |_| None),
    ))(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_name() {
        assert!(all_consuming(parse_name)("this_is_an_outrage!").is_err());
        assert_eq!(
            parse_name("This_is_just_Fine"),
            Ok(("", "This_is_just_Fine"))
        );
    }

    #[test]
    fn test_parse_source() {
        assert!(parse_source("").is_err());
        assert!(parse_source("0").is_err());
        assert_eq!(parse_source("EXP"), Ok(("", Source::Expansion)));
        assert_eq!(parse_source("ACC"), Ok(("", Source::Accumulator)));
        assert_eq!(parse_source("RAM"), Ok(("", Source::Memory)));
        assert_eq!(parse_source("00"), Ok(("", Source::Operand(0x00))));
        assert_eq!(
            parse_source("lo@my_label"),
            Ok(("", Source::LabelLo("my_label".into())))
        );
        assert_eq!(
            parse_source("hi@my_label"),
            Ok(("", Source::LabelHi("my_label".into())))
        );
        assert_eq!(parse_source("5F"), Ok(("", Source::Operand(0x5F))));
    }

    #[test]
    fn test_parse_destination() {
        assert_eq!(parse_destination("RAM"), Ok(("", Destination::Memory)));
        assert_eq!(
            parse_destination("RAM.low"),
            Ok(("", Destination::MemAddressLo))
        );
        assert_eq!(
            parse_destination("RAM.high"),
            Ok(("", Destination::MemAddressHi))
        );
        assert_eq!(parse_destination("ACC"), Ok(("", Destination::Accumulator)));
        assert_eq!(
            parse_destination("ACC.plus"),
            Ok(("", Destination::AccumulatorPlus))
        );
        assert_eq!(
            parse_destination("ACC.nand"),
            Ok(("", Destination::AccumulatorNand))
        );
        assert_eq!(
            parse_destination("PC"),
            Ok(("", Destination::ProgramCounter))
        );
        assert_eq!(
            parse_destination("PC.latch"),
            Ok(("", Destination::ProgramCounterLatch))
        );
        assert_eq!(parse_destination("LED"), Ok(("", Destination::Led)));
        assert_eq!(
            parse_destination("carry.set"),
            Ok(("", Destination::CarrySet))
        );
        assert_eq!(
            parse_destination("carry.reset"),
            Ok(("", Destination::CarryReset))
        );
    }

    #[test]
    fn test_parse_operation() {
        assert!(parse_operation("-> RAM").is_err());
        assert!(parse_operation("RAM -> : if_carry").is_err());
        assert!(parse_operation("-> : ").is_err());
        assert!(parse_operation("RAM.low -> RAM").is_err());
        assert_eq!(
            parse_operation("5F -> ACC // Comment"),
            Ok((
                " // Comment",
                Operation {
                    src: Source::Operand(0x5F),
                    dest: Destination::Accumulator,
                    cond_1: false,
                    cond_carry: false,
                }
            ))
        );
        assert_eq!(
            parse_operation("RAM -> RAM : if_carry // Comment"),
            Ok((
                " // Comment",
                Operation {
                    src: Source::Memory,
                    dest: Destination::Memory,
                    cond_1: false,
                    cond_carry: true,
                }
            ))
        );
        assert_eq!(
            parse_operation("lo@some_label -> ACC.nand : if_carry | if_1 // Comment"),
            Ok((
                " // Comment",
                Operation {
                    src: Source::LabelLo("some_label".into()),
                    dest: Destination::AccumulatorNand,
                    cond_1: true,
                    cond_carry: true,
                }
            ))
        );
    }

    #[test]
    fn test_parse_label() {
        assert!(parse_label("").is_err());
        assert!(parse_label(":").is_err());
        assert_eq!(
            parse_label("thisisalabel:"),
            Ok(("", "thisisalabel".into()))
        );
    }

    #[test]
    fn test_parse_statement() {
        assert_eq!(
            parse_statement("ACC -> ACC"),
            Ok((
                "",
                Statement::Operation(Operation {
                    src: Source::Accumulator,
                    dest: Destination::Accumulator,
                    cond_carry: false,
                    cond_1: false,
                })
            ))
        );
        assert_eq!(
            parse_statement("thisisalabel:"),
            Ok(("", Statement::Label("thisisalabel".into())))
        );
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(
            parse_line("RAM -> RAM : if_carry // Comment"),
            Ok((
                " // Comment",
                Some(Statement::Operation(Operation {
                    src: Source::Memory,
                    dest: Destination::Memory,
                    cond_1: false,
                    cond_carry: true,
                }))
            ))
        );
        assert_eq!(
            parse_line("this_is_a_label:"),
            Ok(("", Some(Statement::Label("this_is_a_label".into()))))
        );
        assert_eq!(parse_line("//"), Ok(("", None)));
        assert_eq!(
            parse_line("// This is a comment"),
            Ok((" This is a comment", None))
        );
        assert_eq!(parse_line(""), Ok(("", None)));
        assert_eq!(parse_line("\t\t     "), Ok(("", None)));
    }
}
