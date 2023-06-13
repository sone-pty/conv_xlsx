#![feature(string_remove_matches)]

mod defs;
use defs::{
    OUTPUT_DIR, 
    SOURCE_XLSXS_DIR, 
    DEFAULT_SOURCE_SUFFIX, 
    DEFAULT_DEST_SUFFIX
};

mod parser;

mod args;
use args::Args;
use clap::Parser;

use std::fs::File;
use std::fs;
use std::io::Write;

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    match args.command {
        args::Command::Build => {
            let base_name = args.name;
            let mut file_name = String::from(&base_name);
            file_name.push('.');
            file_name.push_str(DEFAULT_SOURCE_SUFFIX);
            let xlsx_path = parser::find_file(SOURCE_XLSXS_DIR, &file_name);

            let mut parser = parser::Parser::new();
            let ret = parser.read_file(&base_name, xlsx_path);

            if let Err(e) = ret {
                println!("{}", e);
            }

            let code = parser.generate("\r\n");

            if let Err(_) = fs::metadata(OUTPUT_DIR) {
                fs::create_dir_all(OUTPUT_DIR)?;
            }

            let output_path = format!("{}/{}.{}", OUTPUT_DIR, base_name, DEFAULT_DEST_SUFFIX);
            if let Ok(mut file) = File::create(&output_path) {
                file.write_all(code.as_bytes())?;
            } else {
                println!("open file: {} failed", &output_path);
            }
            Ok(())
        },
        args::Command::Clean => {
            fs::remove_dir_all(OUTPUT_DIR)?;
            Ok(())
        },
    }
}