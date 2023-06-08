use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cell::RefCell;
use std::io::{Error, ErrorKind};

use xlsx_read::{excel_file::ExcelFile, excel_table::ExcelTable};

use super::stack::Stack;

type FKMap<'a> = HashMap<&'a str, HashMap<&'a str, &'a str>>;
// <col, (fk_pattern, vals)>
pub type RawValData<'a> = (usize, (&'a str, Vec<&'a str>));

pub struct FKValue<'a> {
    rawdata: HashMap<usize, ColRawData<'a>>, // <col, data>
    fk_map: RefCell<FKMap<'a>>,
    outvals: HashMap<usize, Vec<String>>
}

struct ColRawData<'a> {
    fk_pattern: &'a str,
    vals: Vec<&'a str>
}

impl<'a> FKValue<'a> {
    pub fn new(vals: Vec<RawValData<'a>>) -> Self {
        let mut rawdata: HashMap<usize, ColRawData<'a>> = HashMap::default();
        let fk_map: RefCell<FKMap<'a>> = RefCell::from(HashMap::default());
        let outvals: HashMap<usize, Vec<String>> = HashMap::default();

        for v in vals {
            match rawdata.entry(v.0) {
                Entry::Occupied(mut e) => {
                    for vv in v.1.1 {
                        e.get_mut().vals.push(vv);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(ColRawData { fk_pattern: v.1.0, vals: Vec::default() });
                }
            }
        }

        Self { rawdata, fk_map, outvals }
    }

    pub fn parse(&self) {
        for (_, v) in self.rawdata.iter() {
            for vv in v.vals.iter() {
                self.parse_internal(*vv, v.fk_pattern);
            }
        }
    }

    pub fn get_value(&'a self, col: usize, row: usize) -> &'a str {
        if self.outvals.contains_key(&col) {
            let vals = self.outvals.get(&col).unwrap();
            if row < vals.len() {
                &vals[row]
            } else {
                ""
            }
        } else {
            ""
        }
    }

    //----------------------------private-------------------------------
    fn parse_internal(&self, val: &'a str, pattern: &'a str) {
        let fk_map = self.fk_map.borrow();
        let mut ch_stack = Stack::<char>::new();

        let take_value = |st: &mut Stack<char>| -> String {
            let mut s = String::with_capacity(10);
            while !st.is_empty() {
                if let Ok(r) = st.pop() {
                    s.push(r)
                }
            }
            s.chars().rev().collect()
        };

        if is_word(pattern) {
            if !fk_map.contains_key(pattern) {
                self.read_fk_table(pattern);
            }
            if let Some(fks) = fk_map.get(pattern) {
                let mut rs = String::default();
                for v in val.chars() {
                    match v {
                        '{' => { rs.push(v); },
                        '}' | ',' => {
                            rs.push(v);
                            let keyword = take_value(&mut ch_stack);
                            if !keyword.is_empty() {
                                rs.push_str(fks.get(keyword.as_str()).unwrap());
                            }
                        },
                        _ => {
                            ch_stack.push(v);
                        }
                    }
                }
            } else {
                todo!("err")
            }
        } else {

        }
    }

    fn read_fk_table(&self, name: &str) {
        if let Ok(table) = super::Parser::get_table_with_id(name, "Template") {
            
        } else {
            println!("read_fk_table: {} failed", name);
        }
    }
}

fn is_word(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric())
}