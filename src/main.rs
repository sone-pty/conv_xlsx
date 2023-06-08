#![feature(string_remove_matches)]

mod defs;
mod parser;

use std::fs::File;
use std::{io::Write, path::PathBuf};

use crate::{defs::OUTPUT_PATH, parser::Parser};

fn main() {
    let mut parser = Parser::new();
    let ret = parser.read_file("Animal.xlsx");

    if let Err(e) = ret {
        println!("{}", e);
    }

    let code = parser.generate("\r\n");

    let mut path = PathBuf::from(OUTPUT_PATH);
    path.push("Animal.cs");
    let mut file = File::create(path.as_path()).unwrap();
    file.write_all(code.as_bytes()).unwrap();
}