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
use std::path::Path;

fn process_xlsx_dir<P: AsRef<Path>>(dir: P) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            process_xlsx_dir(path)?;
        } else {
            let base_name = path.file_name().unwrap().to_str().unwrap();
            let mut parser = parser::Parser::new();
            parser.read_file(base_name, &path)?;
            let output_path = format!("{}/{}.{}", OUTPUT_DIR, base_name, DEFAULT_DEST_SUFFIX);
            let mut file = File::create(output_path)?;
            parser.generate("\r\n", &mut file)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    match args.command {
        args::Command::Build => {
            if let Err(_) = fs::metadata(OUTPUT_DIR) {
                fs::create_dir_all(OUTPUT_DIR)?;
            }

            if args.name.is_empty() {
                process_xlsx_dir(SOURCE_XLSXS_DIR)?;
            } else {
                let base_name = args.name;
                let mut file_name = String::from(&base_name);
                file_name.push('.');
                file_name.push_str(DEFAULT_SOURCE_SUFFIX);
                let xlsx_path = parser::find_file(SOURCE_XLSXS_DIR, &file_name);

                let mut parser = parser::Parser::new();
                parser.read_file(&base_name, xlsx_path)?;

                let output_path = format!("{}/{}.{}", OUTPUT_DIR, base_name, DEFAULT_DEST_SUFFIX);
                let mut file = File::create(output_path)?;
                parser.generate("\r\n", &mut file)?;
            }
            Ok(())
        },
        args::Command::Clean => {
            fs::remove_dir_all(OUTPUT_DIR)?;
            Ok(())
        },
    }
}