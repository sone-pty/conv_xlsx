use std::{collections::HashMap, path::Path, fs::File, io::{BufReader, BufRead}};

use crate::{parser::find_file, defs::DEFAULT_DEF_SUFFIX};

pub struct RefData {
    pub data: HashMap<String, i32>,
    pub file: Box<Path>,
    pub base_name: String,
    pub max_num: i32
}

impl RefData {
    pub fn new<P: AsRef<Path>>(dir: P, base_name: &str) -> Option<Self> {
        let mut file_name = String::from(base_name);
        file_name.push('.');
        file_name.push_str(DEFAULT_DEF_SUFFIX);
        let file = find_file(dir, &file_name);
        let mut max_num = i32::MIN;

        if let Ok(f) = File::open(&file) {
            let reader = BufReader::new(f);
            let mut data = HashMap::<String, i32>::default();
            let mut key = String::default();
            let mut ctl = false;

            for v in reader.lines() {
                if let Ok(v) = v {
                    if v.is_empty() { continue; }
                    if !ctl {
                        key = v;
                    } else if let Ok(num) = v.parse::<i32>() {
                        max_num = if max_num < num { num } else { max_num };
                        data.insert(key.clone(), num);
                    } else {
                        println!("parse failed: src = {}", v);
                    }
                }
                ctl = !ctl;
            }

            Some(RefData { data, file: file.into(), base_name: String::from(base_name), max_num })
        } else {
            None
        }
    }
}