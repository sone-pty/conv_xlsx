use std::{collections::HashMap, path::PathBuf};
use std::cell::RefCell;
use std::rc::Rc;

use crate::defs::{DATA_START_ROW, DEFAULT_SOURCE_SUFFIX, DATA_DEFAULT_ROW, SOURCE_XLSXS_DIR};

use super::cell_value::CellValue;
use super::stack::Stack;

type FKMap = HashMap<String, HashMap<Rc<String>, Rc<String>>>;
// <col, (fk_pattern, vals, type_info)>
pub type RawValData<'a> = (usize, (&'a str, Vec<&'a str>, CellValue));

pub struct FKValue<'a> {
    rawdata: HashMap<usize, ColRawData<'a>>, // <col, data>
    fk_map: RefCell<FKMap>,
    outvals: RefCell<HashMap<usize, Vec<String>>>
}

struct ColRawData<'a> {
    fk_pattern: &'a str,
    vals: Vec<&'a str>,
    ty: CellValue
}

impl<'a> FKValue<'a> {
    pub fn new(vals: Vec<RawValData<'a>>) -> Self {
        let mut rawdata: HashMap<usize, ColRawData<'a>> = HashMap::default();
        let fk_map: RefCell<FKMap> = RefCell::from(HashMap::default());
        let outvals: RefCell<HashMap<usize, Vec<String>>> = RefCell::from(HashMap::default());

        for v in vals {
            if !rawdata.contains_key(&v.0) {
                rawdata.insert(v.0, ColRawData { fk_pattern: v.1.0, vals: Vec::default(), ty: v.1.2 });
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
                self.parse_internal(*vv, v.fk_pattern, col, &v.ty);
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
    fn parse_internal(&'a self, val: &'a str, pattern: &'a str, col: &'a usize, ty: &'a CellValue) {
        let take_and_replace_value = |st: &mut Stack<char>, dest: &mut String, fks: &HashMap<Rc<String>, Rc<String>>| {
            let mut s = String::with_capacity(10);
            while !st.is_empty() {
                if let Ok(r) = st.pop() {
                    s.push(r)
                }
            }

            let rev: String = s.chars().rev().collect();
            if !rev.is_empty() {
                if let Some(vv) = fks.get(&rev) {
                    dest.push_str(vv);
                } else {
                    // TODO
                    dest.push_str("-1");
                }
            }
        };

        let rval = val.chars().filter(|c| *c != ' ').collect::<String>();
        // new value
        let mut rs = String::default();

        if is_simple_pattern(pattern) {
            let mut ch_stack = Stack::<char>::new();

            if !self.fk_map.borrow().contains_key(pattern) {
                let base_name = pattern.chars().filter(|c| *c != '{' && *c != '}').collect();
                self.read_fk_table(base_name);
            }
            if let Some(fks) = self.fk_map.borrow().get(pattern) {
                for v in rval.chars() {
                    match v {
                        '{' => { rs.push(v); },
                        '}' | ',' => {
                            take_and_replace_value(&mut ch_stack, &mut rs, fks);
                            rs.push(v);
                        },
                        _ => {
                            ch_stack.push(v);
                        }
                    }
                }
                if !ch_stack.is_empty() {
                    take_and_replace_value(&mut ch_stack, &mut rs, fks);
                }
            } else {
                todo!("err")
            }
        } else if pattern.contains('?') || pattern.contains('#') {

        } else {
            self.format_value_2(ty, pattern, &rval, &mut rs);
        }

        // push in outvals
        let mut outvals_mut = self.outvals.borrow_mut();
        if !outvals_mut.contains_key(col) {
            outvals_mut.insert(*col, Vec::default());
        }
        let nvals = outvals_mut.get_mut(col).unwrap();
        nvals.push(rs);
    }

    fn read_fk_table(&self, name: String) {
        let mut xlsxs_path = PathBuf::from(SOURCE_XLSXS_DIR);
        xlsxs_path.push(&name);
        xlsxs_path.set_extension(DEFAULT_SOURCE_SUFFIX);

        if let Ok(table) = super::Parser::get_table_with_id(xlsxs_path, "Template") {
            let mut fk_map = self.fk_map.borrow_mut();
            let mut fks = HashMap::<Rc<String>, Rc<String>>::default();
            let height = table.height();
            for row in DATA_START_ROW..height - 1 {
                if let Some(val) = table.cell(0, row) {
                    //fks.insert(Rc::from((row - DATA_START_ROW).to_string()), val.clone());
                    fks.insert(val.clone(), Rc::from((row - DATA_START_ROW).to_string()));
                }
            }
            fk_map.insert(name, fks);
        } else {
            println!("read_fk_table: {} failed", name);
        }
    }

    fn format_value_2(&self, ty: &CellValue, pattern: &str, val: &str, rs: &mut String) 
    {
        let mut ch_stack = Stack::<char>::new();
        
        let take_and_replace_value = |st: &mut Stack<char>, dest: &mut String, fks: &HashMap<Rc<String>, Rc<String>>| {
            let mut s = String::with_capacity(10);
            while !st.is_empty() {
                if let Ok(r) = st.pop() {
                    s.push(r)
                }
            }

            let rev: String = s.chars().rev().collect();
            if !rev.is_empty() {
                if let Some(vv) = fks.get(&rev) {
                    dest.push_str(vv);
                } else {
                    // TODO
                    dest.push_str("-1");
                }
            }
        };

        let take_value = |st: &mut Stack<char>| -> String {
            let mut s = String::with_capacity(10);
            while !st.is_empty() {
                if let Ok(r) = st.pop() {
                    s.push(r)
                }
            }
            s.chars().rev().collect()
        };

        let mut push_basic_value = |ty: &CellValue, dest: &mut String| {
            match ty {
                CellValue::DInt(_) | CellValue::DByte(_) | CellValue::DSByte(_) | 
                CellValue::DShort(_) | CellValue::DUInt(_) | CellValue::DUShort(_) => {
                    let item_pattern = &pattern[1..pattern.len()-1].chars().filter(|c| *c != ' ').collect::<String>();
                    let indexs = item_pattern.split(',').collect::<Vec<&str>>();
                    let mut cnt = 0;
                    
                    for v in indexs.iter() {
                        if !v.is_empty() && !self.fk_map.borrow().contains_key(*v) {
                            self.read_fk_table(String::from(*v));
                        }
                    }

                    for v in val.chars() {
                        match v {
                            '{' => { dest.push(v); },
                            '}' | ',' => {
                                if cnt >= indexs.len() {
                                    cnt = indexs.len() - 1;
                                }
                                if let Some(fks) = self.fk_map.borrow().get(indexs[cnt]) {
                                    take_and_replace_value(&mut ch_stack, dest, fks);
                                } else if indexs[cnt].is_empty() {
                                    dest.push_str(&take_value(&mut ch_stack));
                                }
                                cnt += 1;
                                dest.push(v);
                            },
                            _ => {
                                ch_stack.push(v);
                            }
                        }
                    }
                },
                CellValue::DCustom(_) => {

                },
                _ => {todo!("err")}
            }
        };

        match ty {
            CellValue::DArray(ref arr) => {
                push_basic_value(&arr.0[0], rs);
            },
            CellValue::DList (ref lst) => {
                match &lst.0[0] {
                    CellValue::DList(_) | CellValue::DArray(_) => { 
                        rs.push('{');
                        let mut idx = 1;
                        while idx < val.len() - 1 {
                            let off = super::cell_value::find_block(&val[idx..]);
                            if off != 0 {
                                self.format_value_2(&lst.0[0], &pattern[1..pattern.len()-1], &val[idx..idx+off], rs);
                                idx += off + 1;
                            } else {
                                break;
                            }
                            rs.push(',');
                        }
                        if rs.ends_with(',') {
                            rs.remove(rs.len() - 1);
                        }
                        rs.push('}');
                    },
                    ty => { push_basic_value(ty, rs); }
                }
            },
            _ => {}
        }
    }
}

fn is_simple_pattern(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric()) ||
    s.chars().filter(|c| *c != '{' && *c != '}').all(|c| c.is_alphanumeric())
}