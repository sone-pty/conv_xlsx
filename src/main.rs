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
use std::path::Path;

fn process_xlsx_dir<P: AsRef<Path>>(dir: P, dest: &mut Vec<(String, String)>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            process_xlsx_dir(path, dest)?;
        } else {
            let base_name = path.file_name().unwrap().to_str().unwrap();
            let mut parser = parser::Parser::new();
            parser.read_file(base_name, &path)?;
            let code = parser.generate("\r\n");
            dest.push((String::from(base_name), code));
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

            let mut codes = Vec::<(String, String)>::default();
            
            if args.name.is_empty() {
                process_xlsx_dir(SOURCE_XLSXS_DIR, &mut codes)?;
            } else {
                let base_name = args.name;
                let mut file_name = String::from(&base_name);
                file_name.push('.');
                file_name.push_str(DEFAULT_SOURCE_SUFFIX);
                let xlsx_path = parser::find_file(SOURCE_XLSXS_DIR, &file_name);

                let mut parser = parser::Parser::new();
                parser.read_file(&base_name, xlsx_path)?;

                let code = parser.generate("\r\n");
                codes.push((base_name, code));
            }

            for ref v in codes {
                let output_path = format!("{}/{}.{}", OUTPUT_DIR, v.0, DEFAULT_DEST_SUFFIX);
                if let Ok(mut file) = File::create(&output_path) {
                    file.write_all(v.1.as_bytes())?;
                } else {
                    println!("open file: {} failed", &output_path);
                }
            }
            Ok(())
        },
        args::Command::Clean => {
            fs::remove_dir_all(OUTPUT_DIR)?;
            Ok(())
        },
    }
}