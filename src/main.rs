#![feature(string_remove_matches)]

mod parser;
mod defs;

use std::{path::PathBuf, io::Write};
use std::fs::File;

use crate::{parser::Parser, defs::OUTPUT_PATH};

fn main() {
    let mut parser = Parser::new();
    let ret = parser.read_file("Animal.xlsx");
    
    if let Ok(_) =  ret {

    } else if let Err(e) = ret {
        println!("{}", e);
    }
    
    let code = parser.generate("\r\n");

    let mut path = PathBuf::from(OUTPUT_PATH);
    path.push("Animal.cs");
    let mut file = File::create(path.as_path()).unwrap();
    file.write_all(code.as_bytes()).unwrap();
}