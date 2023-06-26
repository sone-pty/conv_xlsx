#![feature(string_remove_matches)]

mod defs;
use dashmap::DashMap;
use defs::{
    OUTPUT_SCRIPT_CODE_DIR, 
    SOURCE_XLSXS_DIR, 
    DEFAULT_SOURCE_SUFFIX, 
    DEFAULT_DEST_SUFFIX, REF_TEXT_DIR, OUTPUT_ENUM_CODE_DIR, LINE_END_FLAG
};

mod parser;
mod reference;

mod args;
use args::Args;
use clap::Parser;
use lazy_static::lazy_static;
use reference::RefData;

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use std::path::Path;
use std::process::exit;

type ThreadHandles = Arc<Mutex<Vec<thread::JoinHandle<Result<(), std::io::Error>>>>>;
type RefDataMap = DashMap<String, Arc<RefData>>;

fn process_xlsx_dir<P: AsRef<Path>>(dir: P) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            //process_xlsx_dir(path)?;
            let handle = thread::spawn(|| -> Result<(), std::io::Error> {
                process_xlsx_dir(path)?;
                Ok(())
            });
            HANDLES.lock().unwrap().push(handle);
        } else if path.extension().is_some_and(|x| x.to_str().unwrap() == DEFAULT_SOURCE_SUFFIX) && !path.starts_with("~") {
            let base_name = path.file_name().unwrap().to_str().unwrap();
            let idx = base_name.find('.').unwrap_or_default();
            
            let mut parser = parser::Parser::new();
            if let Some(refdata) = RefData::new(REF_TEXT_DIR, &base_name[..idx]) {
                let aref = Arc::from(refdata);
                if !RDM.contains_key(&base_name[..idx]) {
                    RDM.insert(String::from(&base_name[..idx]), aref.clone());
                }
                parser.read_file(&base_name[..idx], &path, Some(aref))?;
            } else {
                parser.read_file(&base_name[..idx], &path, None)?;
            }

            let output_path = format!("{}/{}.{}", OUTPUT_SCRIPT_CODE_DIR, &base_name[..idx], DEFAULT_DEST_SUFFIX);
            let mut file = File::create(output_path)?;
            println!("Process file_name: {}", base_name);
            parser.generate("\r\n", &mut file)?;
        }
    }
    Ok(())
}

lazy_static! (
    static ref HANDLES: ThreadHandles = Arc::new(Mutex::new(Vec::new()));
    static ref RDM: RefDataMap = DashMap::default();
);

fn main() {
    let args = Args::parse();

    match args.command {
        args::Command::Build => {
            if let Err(_) = fs::metadata(OUTPUT_SCRIPT_CODE_DIR) {
                if let Err(_) = fs::create_dir_all(OUTPUT_SCRIPT_CODE_DIR) {
                    exit(-1)
                }
            }
            
            if let Err(_) = fs::metadata(OUTPUT_ENUM_CODE_DIR) {
                if let Err(_) = fs::create_dir_all(OUTPUT_ENUM_CODE_DIR) {
                    exit(-1)
                }
            }

            if args.name.is_empty() {
                if let Err(e) = process_xlsx_dir(SOURCE_XLSXS_DIR) {
                    println!("{}", e);
                    exit(-1); 
                }

                for handle in HANDLES.lock().unwrap().drain(..) {
                    let _ = handle.join();
                }
            } else {
                let base_name = args.name;
                let mut file_name = String::from(&base_name);
                file_name.push('.');
                file_name.push_str(DEFAULT_SOURCE_SUFFIX);
                let xlsx_path = parser::find_file(SOURCE_XLSXS_DIR, &file_name);

                let mut parser = parser::Parser::new();

                if let Some(refdata) = RefData::new(REF_TEXT_DIR, &base_name) {
                    if let Err(e) = parser.read_file(&base_name, xlsx_path, Some(Arc::from(refdata))) {
                        println!("{}", e);
                        exit(-1);
                    }
                } else {
                    if let Err(e) = parser.read_file(&base_name, xlsx_path, None) {
                        println!("{}", e);
                        exit(-1);
                    }
                }

                let output_path = format!("{}/{}.{}", OUTPUT_SCRIPT_CODE_DIR, base_name, DEFAULT_DEST_SUFFIX);
                if let Ok(mut file) = File::create(output_path) {
                    if let Err(e) = parser.generate(LINE_END_FLAG, &mut file) {
                        println!("{}", e);
                        exit(-1)
                    }
                } else {
                    exit(-1)
                }
            }
        },
        args::Command::Clean => {
            if let Err(e) = fs::remove_dir_all(OUTPUT_SCRIPT_CODE_DIR) {
                println!("{}", e);
                exit(-1)
            }
            if let Err(e) = fs::remove_dir_all(OUTPUT_ENUM_CODE_DIR) {
                println!("{}", e);
                exit(-1)
            }
        },
    }

    exit(0)
}