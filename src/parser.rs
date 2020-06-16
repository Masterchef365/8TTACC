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

pub type PortName<'s> = Vec<&'s str>;

#[derive(Debug, PartialEq)]
pub struct Operation<'s> {
    pub src: PortName<'s>,
    pub dest: PortName<'s>,
}

pub type Label<'a> = &'a str;

#[derive(Debug, PartialEq)]
pub enum Statement<'a> {
    Label(Label<'a>),
    Operation(Operation<'a>),
}

fn parse_name<'a>(s: &'a str) -> IResult<&'a str, &'a str> {
    s.split_at_position1_complete(|item| !item.is_alphanum() && item != '_', ErrorKind::NoneOf)
}

fn parse_portname<'a>(s: &'a str) -> IResult<&'a str, PortName<'a>> {
    separated_list1(tag("."), parse_name)(s)
}

fn parse_operation<'a>(s: &'a str) -> IResult<&'a str, Operation<'a>> {
    let arrow = tuple((space1, tag("->"), space1));
    let ports = tuple((terminated(parse_portname, arrow), parse_portname));
    map(ports, |(src, dest)| Operation { src, dest })(s)
}

fn parse_label<'a>(s: &'a str) -> IResult<&'a str, Label<'a>> {
    terminated(parse_name, tag(":"))(s)
}

fn parse_statement<'a>(s: &'a str) -> IResult<&'a str, Statement<'a>> {
    alt((
        map(parse_operation, Statement::Operation),
        map(parse_label, Statement::Label),
    ))(s)
}

pub fn parse_line<'a>(s: &'a str) -> IResult<&'a str, Option<Statement<'a>>> {
    alt((
        map(parse_statement, Some),
        map(tag("//"), |_| None),
        map(all_consuming(space0), |_| None),
    ))(s)
}

#[test]
fn test_parse_portname() {
    assert!(parse_portname("").is_err());
    assert_eq!(parse_portname("a"), Ok(("", vec!["a"])));
    assert_eq!(parse_portname("a.b"), Ok(("", vec!["a", "b"])));
    assert_eq!(parse_portname("a.b.c"), Ok(("", vec!["a", "b", "c"])));
}

#[test]
fn test_parse_operation() {
    assert_eq!(
        parse_operation("a -> b"),
        Ok((
            "",
            Operation {
                src: vec!["a"],
                dest: vec!["b"]
            }
        ))
    );
    assert_eq!(
        parse_operation("a.c -> b.q"),
        Ok((
            "",
            Operation {
                src: vec!["a", "c"],
                dest: vec!["b", "q"]
            }
        ))
    );
    assert!(parse_operation(" -> b.q.a").is_err());
    assert!(parse_operation("a ->  ").is_err());
}

#[test]
fn test_parse_label() {
    assert!(parse_label("").is_err());
    assert!(parse_label(":").is_err());
    assert_eq!(parse_label("thisisalabel:"), Ok(("", "thisisalabel")));
}

#[test]
fn test_parse_statement() {
    assert_eq!(
        parse_statement("a -> b"),
        Ok((
            "",
            Statement::Operation(Operation {
                src: vec!["a"],
                dest: vec!["b"]
            })
        ))
    );
    assert_eq!(
        parse_statement("thisisalabel:"),
        Ok(("", Statement::Label("thisisalabel")))
    );
}

#[test]
fn test_parse_line() {
    assert_eq!(
        parse_line("a -> b"),
        Ok((
            "",
            Some(Statement::Operation(Operation {
                src: vec!["a"],
                dest: vec!["b"]
            }))
        ))
    );
    assert_eq!(
        parse_line("this_is_a_label:"),
        Ok(("", Some(Statement::Label("this_is_a_label"))))
    );
    assert_eq!(parse_line("//"), Ok(("", None)));
    assert_eq!(
        parse_line("// This is a comment"),
        Ok((" This is a comment", None))
    );
    assert_eq!(parse_line(""), Ok(("", None)));
    assert_eq!(parse_line("\t\t     "), Ok(("", None)));
}
