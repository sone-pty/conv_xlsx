use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::defs::{DATA_START_ROW, DEFAULT_SUFFIX, DATA_DEFAULT_ROW};

use super::stack::Stack;

type FKMap<'a> = HashMap<&'a str, HashMap<Rc<String>, Rc<String>>>;
// <col, (fk_pattern, vals)>
pub type RawValData<'a> = (usize, (&'a str, Vec<&'a str>));

pub struct FKValue<'a> {
    rawdata: HashMap<usize, ColRawData<'a>>, // <col, data>
    fk_map: RefCell<FKMap<'a>>,
    outvals: RefCell<HashMap<usize, Vec<String>>>
}

struct ColRawData<'a> {
    fk_pattern: &'a str,
    vals: Vec<&'a str>
}

impl<'a> FKValue<'a> {
    pub fn new(vals: Vec<RawValData<'a>>) -> Self {
        let mut rawdata: HashMap<usize, ColRawData<'a>> = HashMap::default();
        let fk_map: RefCell<FKMap<'a>> = RefCell::from(HashMap::default());
        let outvals: RefCell<HashMap<usize, Vec<String>>> = RefCell::from(HashMap::default());

        for v in vals {
            if !rawdata.contains_key(&v.0) {
                rawdata.insert(v.0, ColRawData { fk_pattern: v.1.0, vals: Vec::default() });
            }

            let coldata = rawdata.get_mut(&v.0).unwrap();
            for vv in v.1.1 {
                coldata.vals.push(vv);
            }
        }

        Self { rawdata, fk_map, outvals }
    }

    pub fn parse(&'a self) {
        for (col, v) in self.rawdata.iter() {
            for vv in v.vals.iter() {
                self.parse_internal(*vv, v.fk_pattern, col);
            }
        }
    }

    pub fn get_value(&'a self, col: usize, row: usize) -> &'a str {
        if self.outvals.borrow().contains_key(&col) {
            let vals = unsafe { (*self.outvals.as_ptr()).get(&col).unwrap() };
            if row - DATA_DEFAULT_ROW < vals.len() {
                &vals[row - DATA_DEFAULT_ROW]
            } else {
                ""
            }
        } else {
            ""
        }
    }

    //----------------------------private-------------------------------
    fn parse_internal(&'a self, val: &'a str, pattern: &'a str, col: &'a usize) {
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
        
        // new value
        let mut rs = String::default();

        if is_word(pattern) {
            if !self.fk_map.borrow().contains_key(pattern) {
                self.read_fk_table(pattern);
            }
            if let Some(fks) = self.fk_map.borrow().get(pattern) {
                for v in val.chars() {
                    match v {
                        '{' => { rs.push(v); },
                        '}' | ',' => {
                            rs.push(v);
                            let keyword = take_value(&mut ch_stack);
                            if !keyword.is_empty() {
                                rs.push_str(fks.get(&keyword).unwrap());
                            }
                        },
                        _ => {
                            ch_stack.push(v);
                        }
                    }
                }
                if !ch_stack.is_empty() {
                    let keyword = take_value(&mut ch_stack);
                    if !keyword.is_empty() {
                        rs.push_str(fks.get(&keyword).unwrap());
                    }
                }
            } else {
                todo!("err")
            }
        } else {

        }

        // push in outvals
        let mut outvals_mut = self.outvals.borrow_mut();
        if !outvals_mut.contains_key(col) {
            outvals_mut.insert(*col, Vec::default());
        }
        let nvals = outvals_mut.get_mut(col).unwrap();
        nvals.push(rs);
    }

    fn read_fk_table(&'a self, name: &'a str) {
        let mut file_name = String::from(name);
        file_name.push_str(DEFAULT_SUFFIX);
        if let Ok(table) = super::Parser::get_table_with_id(&file_name, "Template") {
            let mut fk_map = self.fk_map.borrow_mut();
            let mut fks = HashMap::<Rc<String>, Rc<String>>::default();
            let height = table.height();
            for row in DATA_START_ROW..height - 1 {
                if let Some(val) = table.cell(0, row) {
                    fks.insert(Rc::from((row - DATA_START_ROW).to_string()), val.clone());
                    fks.insert(val.clone(), Rc::from((row - DATA_START_ROW).to_string()));
                }
            }
            fk_map.insert(name, fks);
        } else {
            println!("read_fk_table: {} failed", name);
        }
    }
}

fn is_word(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric())
}