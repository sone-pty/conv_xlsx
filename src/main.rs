#![feature(string_remove_matches)]

mod parser;
mod defs;

use crate::parser::Parser;

fn main() {
    let mut parser = Parser::new();
    let ret = parser.read_file("Animal.xlsx");
    
    if let Ok(_) =  ret {

    } else if let Err(e) = ret {
        println!("{}", e);
    }

    println!("{}", parser.generate("\r\n"));
}