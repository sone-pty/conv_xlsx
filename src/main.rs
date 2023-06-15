#![feature(string_remove_matches)]

mod defs;
use defs::{
    OUTPUT_DIR, 
    SOURCE_XLSXS_DIR, 
    DEFAULT_SOURCE_SUFFIX, 
    DEFAULT_DEST_SUFFIX, REF_TEXT_DIR
};

mod parser;
mod reference;

mod args;
use args::Args;
use clap::Parser;
use reference::RefData;

use std::fs::File;
use std::fs;
use std::path::Path;
use std::process::exit;

fn process_xlsx_dir<P: AsRef<Path>>(dir: P) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            process_xlsx_dir(path)?;
        } else {
            let base_name = path.file_name().unwrap().to_str().unwrap();
            let idx = base_name.find('.').unwrap_or_default();
            
            let mut parser = parser::Parser::new();
            parser.read_file(&base_name[..idx], &path, RefData::new(REF_TEXT_DIR, &base_name[..idx]))?;
            let output_path = format!("{}/{}.{}", OUTPUT_DIR, &base_name[..idx], DEFAULT_DEST_SUFFIX);
            let mut file = File::create(output_path)?;
            parser.generate("\r\n", &mut file)?;
        }
    }
    Ok(())
}

fn main() {
    let args = Args::parse();

    match args.command {
        args::Command::Build => {
            if let Err(_) = fs::metadata(OUTPUT_DIR) {
                if let Err(_) = fs::create_dir_all(OUTPUT_DIR) {
                    exit(-1)
                }
            }

            if args.name.is_empty() {
                if let Err(e) = process_xlsx_dir(SOURCE_XLSXS_DIR) {
                    println!("{}", e);
                    exit(-1); 
                }
            } else {
                let base_name = args.name;
                let mut file_name = String::from(&base_name);
                file_name.push('.');
                file_name.push_str(DEFAULT_SOURCE_SUFFIX);
                let xlsx_path = parser::find_file(SOURCE_XLSXS_DIR, &file_name);

                let mut parser = parser::Parser::new();

                if let Err(e) = parser.read_file(&base_name, xlsx_path, RefData::new(REF_TEXT_DIR, &base_name)) {
                    println!("{}", e);
                    exit(-1)
                }

                let output_path = format!("{}/{}.{}", OUTPUT_DIR, base_name, DEFAULT_DEST_SUFFIX);
                if let Ok(mut file) = File::create(output_path) {
                    if let Err(e) = parser.generate("\r\n", &mut file) {
                        println!("{}", e);
                        exit(-1)
                    }
                } else {
                    exit(-1)
                }
            }
        },
        args::Command::Clean => {
            if let Err(e) = fs::remove_dir_all(OUTPUT_DIR) {
                println!("{}", e);
                exit(-1)
            }
        },
    }

    exit(0)
}