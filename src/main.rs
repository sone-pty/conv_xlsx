#![feature(string_remove_matches)]

mod defs;
use defs::{
    OUTPUT_PATH, 
    SOURCE_XLSXS_DIR, 
    DEFAULT_SOURCE_SUFFIX, 
    DEFAULT_DEST_SUFFIX
};

mod parser;

use std::fs::File;
use std::{io::Write, path::PathBuf};

fn main() {
    let base_name = "Animal";
    let mut xlsxs_path = PathBuf::from(SOURCE_XLSXS_DIR);
    xlsxs_path.push(base_name);
    xlsxs_path.set_extension(DEFAULT_SOURCE_SUFFIX);

    let mut parser = parser::Parser::new();
    let ret = parser.read_file(base_name, xlsxs_path);

    if let Err(e) = ret {
        println!("{}", e);
    }

    let code = parser.generate("\r\n");
    let mut output_path = PathBuf::from(OUTPUT_PATH);
    output_path.push(base_name);
    output_path.set_extension(DEFAULT_DEST_SUFFIX);
    let mut file = File::create(output_path.as_path()).unwrap();
    file.write_all(code.as_bytes()).unwrap();
}