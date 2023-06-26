use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::defs::{DATA_START_ROW, DEFAULT_SOURCE_SUFFIX, DATA_DEFAULT_ROW, SOURCE_XLSXS_DIR};

use super::cell_value::{CellValue, ShortListValue};
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
        let rval = val.chars().filter(|c| *c != ' ').collect::<String>();
        // new value
        let mut rs = String::default();

        if is_simple_pattern(pattern) {
            let mut ch_stack = Stack::<char>::new();
            let base_name = pattern.chars().filter(|c| *c != '{' && *c != '}').collect::<String>();

            if !self.fk_map.borrow().contains_key(&base_name) {
                self.read_fk_table(base_name.clone());
            }
            if let Some(fks) = self.fk_map.borrow().get(&base_name) {
                for v in rval.chars() {
                    match v {
                        '{' => { rs.push(v); },
                        '}' | ',' | '，'=> {
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
                println!("cant find fk table: {}.xlsx", &base_name);
            }
        } else if pattern.contains('?') || pattern.contains('#') {
            self.format_value_1(ty, pattern, &rval, &mut rs);
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
        let mut file_name = String::from(&name);
        file_name.push('.');
        file_name.push_str(DEFAULT_SOURCE_SUFFIX);
        let xlsxs_path = super::find_file(SOURCE_XLSXS_DIR, &file_name);

        if let Ok(table) = super::Parser::get_table_with_id(xlsxs_path, "") {
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

    fn format_value_1(&self, ty: &CellValue, pattern: &str, val: &str, rs: &mut String) {
        // handle with custom objects
        let push_basic_value = |ty: &CellValue, dest: &mut String, is_arr: bool| {
            match ty {
                CellValue::DCustom(_) => {
                    let patterns = split_pattern(&pattern[if is_arr {1} else {0}..pattern.len()-(if is_arr {1} else {0})]);
                    let mut pidx = 0;

                    let mut idx = if is_arr {1} else {0};
                    let mut fk_names = Vec::<String>::with_capacity(1);

                    if val.is_empty() { return; }

                    while idx < val.len() - 1 {
                        if pidx >= patterns.len() { pidx = patterns.len()-1; }
                        let empty = patterns[pidx].is_empty();
                        let item_pattern = &patterns[pidx][(if empty {0} else {1})..(if empty {0} else {patterns[pidx].len()-1})].chars().filter(|c| *c != ' ').collect::<String>();
                        let indexs = item_pattern.split(',').collect::<Vec<&str>>();
                        let off = super::cell_value::find_block(&val[idx..]);
                        
                        if off != 0 {
                            dest.push('{');
                            let val_str = val[idx+1..idx+off-1].chars().filter(|c| *c != ' ').collect::<String>();
                            let vals = split_val(&val_str);
                            let mut cnt = 0;

                            // get fks, assert len of vals >= len of indexs
                            for v in indexs.iter() {
                                if v.starts_with('?') {
                                    if v.len() == 1 {
                                        fk_names.push(String::from(&vals[cnt]));
                                    } else {
                                        if let Ok(num) = v[1..].parse::<usize>() {
                                            if fk_names.capacity() < num {
                                                fk_names.reserve(num << 1);
                                                unsafe { fk_names.set_len(num << 1); }
                                            }
                                            fk_names[num] = String::from(&vals[cnt]);
                                        } else {
                                            println!("parse from {} to usize failed", &v[1..]);
                                        }
                                    }

                                    if !vals[cnt].is_empty() && !self.fk_map.borrow().contains_key(&vals[cnt][1..vals[cnt].len()-1]) {
                                        self.read_fk_table(String::from(&vals[cnt][1..vals[cnt].len()-1]));
                                    }
                                }
                                cnt += 1;
                            }

                            cnt = 0;
                            // push str
                            for v in vals.iter() {
                                if cnt >= indexs.len() { cnt = indexs.len()-1; }
                                if indexs[cnt].starts_with('?') { // push table name
                                    dest.push_str(v);
                                } else if indexs[cnt].starts_with('#') { // push val in fks
                                    let mut id = 0;
                                    if indexs[cnt].len() > 1 {
                                        if let Ok(num) = indexs[cnt][1..].parse::<usize>() {
                                            id = num;
                                        } else {
                                            println!("parse from {} to usize failed", &indexs[cnt][1..]);
                                        }
                                    }

                                    // assert id is in the [0..fk_names.len()]
                                    if let Some(fks) = self.fk_map.borrow().get(&fk_names[id][1..fk_names[id].len()-1]) {
                                        if let Some(vv) = fks.get(&String::from(v)) {
                                            dest.push_str(vv);
                                        }
                                    } else {
                                        println!("cant find the fks by the keyname = {}", &fk_names[id]);
                                    }
                                } else if indexs[cnt].contains('#') { // push original val
                                    self.process_cmp_value(dest, indexs[cnt], v, &fk_names);
                                } else {
                                    dest.push_str(v);
                                }
                                cnt += 1;
                                dest.push(',');
                            }

                            if dest.ends_with(',') { dest.remove(dest.len()-1); }
                            dest.push('}');
                        } else {
                            break;
                        }

                        dest.push(',');
                        idx += off + 1; // skip ','
                        pidx += 1;
                    }

                    if dest.ends_with(',') { dest.remove(dest.len()-1); }
                },
                _ => { todo!("err") }
            }
        };

        match ty {
            CellValue::DArray(arr) => {
                rs.push('{');
                push_basic_value(&arr.0[0], rs, true);
                rs.push('}');
            },
            CellValue::DList(ref lst) => {
                rs.push('{');
                match &lst.0[0] {
                    CellValue::DList(_) | CellValue::DArray(_) => {},
                    CellValue::DCustom(_) => { push_basic_value(&lst.0[0], rs, true); },
                    _ => {}
                }
                rs.push('}');
            },
            CellValue::DCustom(_) => {
                push_basic_value(ty, rs, false);
            },
            _ => { todo!("err") }
        }
    }

    fn process_cmp_value(&self, dest: &mut String, pat: &str, val: &str, fk_names: &Vec<String>) {
        let item_pattern = &pat[1..pat.len()-1].chars().filter(|c| *c != ' ').collect::<String>();
        let indexs = item_pattern.split(',').collect::<Vec<&str>>();
        let val_str = val[1..val.len()-1].chars().filter(|c| *c != ' ').collect::<String>();
        let vals = split_val(&val_str);
        let mut cnt = 0;

        dest.push('{');
        for v in vals.iter() {
            if cnt >= indexs.len() { cnt = indexs.len()-1; }
            if indexs[cnt].starts_with('?') { // push table name
                dest.push_str(v);
            } else if indexs[cnt].starts_with('#') { // push val in fks
                let mut id = 0;
                if indexs[cnt].len() > 1 {
                    if let Ok(num) = indexs[cnt][1..].parse::<usize>() {
                        id = num;
                    } else {
                        println!("parse from {} to usize failed", &indexs[cnt][1..]);
                    }
                }

                // assert id is in the [0..fk_names.len()]
                self.fk_map.borrow().get(&fk_names[id][1..fk_names[id].len()-1]).map(|fks| {
                    fks.get(&String::from(v)).map(|vv| {
                        dest.push_str(vv);
                    });
                });
            } else if indexs[cnt].contains('#') { // push original val
                self.process_cmp_value(dest, indexs[cnt], v, fk_names);
            } else {
                dest.push_str(v);
            }
        }
        dest.push('}');
    }

    fn format_value_2(&self, ty: &CellValue, pattern: &str, val: &str, rs: &mut String) 
    {
        let mut ch_stack = Stack::<char>::new();
        let mut push_basic_value = |ty: &CellValue, dest: &mut String, is_arr: bool| {
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
                            _ => { ch_stack.push(v); }
                        }
                    }
                }
                CellValue::DCustom(_) => {
                    let item_pattern = &pattern[(if is_arr {2} else {1})..pattern.len()-(if is_arr {2} else {1})].chars().filter(|c| *c != ' ').collect::<String>();
                    let indexs = item_pattern.split(',').collect::<Vec<&str>>();
                    let mut cnt = 0;
                    let mut tmp: &str;
            
                    for v in indexs.iter() {
                        if v.is_empty() { continue; }
                        
                        if v.starts_with('{') {
                            tmp = &v[1..v.len()-1];
                        } else {
                            tmp = &v;
                        }

                        if !tmp.is_empty() && !self.fk_map.borrow().contains_key(tmp) {
                            self.read_fk_table(String::from(tmp));
                        }
                    }

                    let mut braces = 0;
                    for v in val.chars() {
                        match v {
                            '{' => { dest.push(v); braces += 1; }
                            '}' | ',' => {
                                if cnt >= indexs.len() {
                                    cnt = indexs.len() - 1;
                                }
                                
                                if braces < (if is_arr {3} else {2}) {
                                    if braces == (if is_arr {1} else {0}) && v == ',' { dest.push(v); continue; }

                                    if let Some(fks) = self.fk_map.borrow().get(indexs[cnt]) {
                                        take_and_replace_value(&mut ch_stack, dest, fks);
                                    } else if indexs[cnt].is_empty() {
                                        dest.push_str(&take_value(&mut ch_stack));
                                    }

                                    if v == '}' {
                                        braces -= 1;
                                        cnt = 0;
                                    } else {
                                        cnt += 1;
                                    }
                                } else if indexs[cnt].starts_with('{') {
                                    tmp = &indexs[cnt][1..indexs[cnt].len()-1];
                                    self.fk_map.borrow().get(tmp).map(|fks| {
                                        take_and_replace_value(&mut ch_stack, dest, fks);
                                    });
                                    if v == '}' { braces -= 1; }
                                } else {
                                    dest.push_str(&take_value(&mut ch_stack));
                                    if v == '}' { braces -= 1; }
                                }

                                dest.push(v);
                            }
                            _ => { ch_stack.push(v); }
                        }
                    }
                }
                CellValue::DTuple(_) => {
                    let item_pattern = &pattern[(if is_arr {2} else {1})..pattern.len()-(if is_arr {2} else {1})].chars().filter(|c| *c != ' ').collect::<String>();
                    let indexs = item_pattern.split(',').collect::<Vec<&str>>();
                    let mut cnt;
                    let mut idx = 0;
                    let items = split_val(&val[(if is_arr {1} else {0})..val.len()-(if is_arr {1} else {0})]);
                    if is_arr { dest.push('{'); }

                    for v in indexs.iter() {
                        if !v.is_empty() && !self.fk_map.borrow().contains_key(*v) && !v.starts_with('{') {
                            self.read_fk_table(String::from(*v));
                        }
                    }

                    if !indexs.is_empty() {
                        if indexs[0].is_empty() {
                            let nums = indexs.len() - 1;

                            for v in items.iter() {
                                cnt = nums;
                                dest.push('{');
                                let vs = split_val(&v[1..v.len()-1]);

                                for i in 0..vs.len() - nums {
                                    dest.push_str(&vs[i]);
                                    dest.push(',');
                                }

                                for i in nums..vs.len() {
                                    if cnt >= indexs.len() { cnt = indexs.len()-1; }
                                    if let Some(fks) = self.fk_map.borrow().get(indexs[cnt]) {
                                        if let Some(vv) = fks.get(&vs[i]) {
                                            dest.push_str(vv);
                                        }
                                    } else if indexs[cnt].is_empty() {
                                        dest.push_str(&vs[i]);
                                    }
                                    if cnt < vs.len()-1 {
                                        dest.push(',');
                                    }
                                    cnt += 1;
                                }
                                dest.push('}');
                            }
                        } else {
                            for v in items.iter() {
                                cnt = 0;
                                let vs = split_val(&v[1..v.len()-1]);

                                dest.push('{');
                                for vv in vs.iter() {
                                    if cnt >= indexs.len() { cnt = indexs.len()-1; }
                                    if let Some(fks) = self.fk_map.borrow().get(indexs[cnt]) {
                                        if let Some(vv) = fks.get(vv) {
                                            dest.push_str(vv);
                                        }
                                    } else if indexs[cnt].is_empty() {
                                        dest.push_str(vv);
                                    }
                                    if cnt < vs.len()-1 {
                                        dest.push(',');
                                    }
                                    cnt += 1;
                                }
                                dest.push('}');
                                if idx < items.len()-1 {
                                    dest.push(',');
                                }
                                idx += 1;
                            }
                        }
                    }

                    if is_arr { dest.push('}'); }
                }
                _ => { todo!("err") }
            }
        };

        match ty {
            CellValue::DArray(ref arr) => {
                push_basic_value(&arr.0[0], rs, true);
            },
            CellValue::DList(ref lst) => {
                match &lst.0[0] {
                    CellValue::DList(_) | CellValue::DArray(_) | CellValue::DShortList(_) => { 
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
                    ty => { push_basic_value(ty, rs, true); }
                }
            },
            CellValue::DShortList(ShortListValue(ref arr)) => {
                push_basic_value(&arr.0[0], rs, true);
            },
            CellValue::DCustom(_) => { push_basic_value(ty, rs, false); },
            _ => { todo!("err") }
        }
    }
}

fn is_simple_pattern(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric()) ||
    s.chars().filter(|c| *c != '{' && *c != '}').all(|c| c.is_alphanumeric())
}

fn take_value(st: &mut Stack<char>) -> String {
    let mut s = String::with_capacity(10);
    while !st.is_empty() {
        if let Ok(r) = st.pop() {
            s.push(r)
        }
    }
    s.chars().rev().collect()
}

fn take_and_replace_value(st: &mut Stack<char>, dest: &mut String, fks: &HashMap<Rc<String>, Rc<String>>) {
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
}

pub fn split_val(val: &str) -> Vec<String> {
    let mut ch_stack = Stack::<char>::new();
    let mut ret = Vec::<String>::default();
    let mut is_bracket = false;

    for v in val.chars() {
        match v {
            '{' => {
                ch_stack.push(v);
                is_bracket = true;
            }
            ',' => {
                if is_bracket { 
                    ch_stack.push(v);
                } else if !ch_stack.is_empty() {
                    ret.push(take_value(&mut ch_stack));
                }
            }
            '}' => {
                ch_stack.push(v);
                ret.push(take_value(&mut ch_stack));
                is_bracket = false;
            }
            ' ' => {}
            _ => { ch_stack.push(v); }
        }
    }

    if !ret.is_empty() {
        if ret[ret.len()-1].is_empty() { ret.remove(ret.len()-1); }
    }
    if !ch_stack.is_empty() { ret.push(take_value(&mut ch_stack)); }

    ret
}

fn split_pattern(pat: &str) -> Vec<&str> {
    let mut ret = Vec::<&str>::default();
    let mut cur = 0;
    let mut prev = 0;
    let mut bracket_stack = Stack::<char>::default();

    for ref v in pat.chars() {
        match v {
            '{' => {
                if bracket_stack.is_empty() {
                    prev = cur;
                }
                bracket_stack.push(*v);
            }
            ',' => {
                if bracket_stack.is_empty() {
                    ret.push(&pat[prev..cur]);
                }
            },
            '}' => {
                if let Ok(v) = bracket_stack.pop() {
                    if v == '{' && bracket_stack.is_empty() {
                        ret.push(&pat[prev..]);
                    }
                }
            }
            _ => {}
        }
        cur += 1;
    }

    if pat.ends_with(',') {
        ret.push("");
    }

    ret
}