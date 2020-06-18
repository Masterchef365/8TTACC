use std::fs::File;
use std::io::{BufReader, BufRead};
mod parser;
use parser::*;
mod assembler;
use assembler::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().skip(1).next().expect("Expects path");
    let file = BufReader::new(File::open(path)?);
    for line in file.lines() {
        let line = line?;
        let res = parse_line(&line).map_err(|s| s.to_owned())?;
        if let Some(stmt) = res.1 {
            println!("{:#?}", stmt);
        }
    }
    Ok(())
}
