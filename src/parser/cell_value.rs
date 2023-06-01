use std::rc::Rc;
use super::stack::Stack;

pub enum CellValue {
    DBool(bool),
    DLString(Rc<String>),
    DString(Rc<String>),
    DShort(i16),
    DUShort(u16),
    DSByte(i8),
    DByte(u8),
    DInt(i32),
    DUInt(u32),
    DArray(Vec<CellValue>), // first element of arr is one dumb, start from index 1
    DList(Vec<CellValue>),  // first element of list is one dumb, start from index 1
    DNone,
}

impl CellValue {
    // TODO: process error
    pub fn new(val: &Rc<String>, ty: &Rc<String>) -> CellValue {
        let val_str = val.as_str();
        let ty_str = ty.as_str();

        match ty_str {
            "bool" => {
                if val_str == "0" {
                    Self::DBool(false)
                } else if val_str == "1" {
                    Self::DBool(true)
                } else {
                    todo!()
                }
            }
            "byte" => {
                if let Ok(v) = val_str.parse::<u8>() {
                    Self::DByte(v)
                } else {
                    todo!()
                }
            }
            "sbyte" => {
                if let Ok(v) = val_str.parse::<i8>() {
                    Self::DSByte(v)
                } else {
                    todo!()
                }
            }
            "LString" => {
                todo!()
            }
            "string" => Self::DString(val.clone()),
            "short" => {
                if let Ok(v) = val_str.parse::<i16>() {
                    Self::DShort(v)
                } else {
                    todo!()
                }
            }
            "ushort" => {
                if let Ok(v) = val_str.parse::<u16>() {
                    Self::DUShort(v)
                } else {
                    todo!()
                }
            }
            "int" => {
                if let Ok(v) = val_str.parse::<i32>() {
                    Self::DInt(v)
                } else {
                    todo!()
                }
            }
            "uint" => {
                if let Ok(v) = val_str.parse::<u32>() {
                    Self::DUInt(v)
                } else {
                    todo!()
                }
            }
            // array or list
            _ => {
                let mut char_stack: Stack<char> = Stack::new();
                let mut op_stack: Stack<char> = Stack::new();
                let mut keyword_stack: Stack<String> = Stack::new();
                let mut ret = CellValue::DNone;

                let take_keyword = |st: &mut Stack<char>| -> String {
                    let mut s = String::with_capacity(10);
                    while !st.is_empty() {
                        if let Ok(r) = st.pop() {
                            s.push(r)
                        }
                    }
                    s.chars().rev().collect()
                };

                for c in ty_str.chars() {
                    match c {
                        ']' => {
                            if let Ok(key) = keyword_stack.pop() {
                                if let Ok(top) = op_stack.pop() {
                                    if top == '[' {
                                        ret = CellValue::DArray(vec![CellValue::basic_default_value(&key)]);
                                    }
                                }
                            }
                        }
                        '>' => {
                            if char_stack.is_empty() {
                                if let Ok(key) = keyword_stack.pop() {
                                    if let Ok(top) = op_stack.pop() {
                                        if top == '<' && key == "List" {
                                            ret = CellValue::DList(vec![ret]);
                                        }
                                    }
                                }
                            } else {
                                let _ = op_stack.pop();
                                let inner_keyword = take_keyword(&mut char_stack);
                                ret = CellValue::basic_default_value(&inner_keyword);
                                ret = CellValue::DList(vec![ret]);
                            }
                        }
                        '[' | '<' => {
                            op_stack.push(c);
                            keyword_stack.push(take_keyword(&mut char_stack));
                        }
                        c => {
                            char_stack.push(c);
                        }
                    }
                }
                
                if op_stack.is_empty() {
                    ret 
                } else {
                    // TODO: err
                    CellValue::DNone
                }
            }
        }
    }

    // not include list and array
    fn basic_default_value(key: &str) -> CellValue {
        match key {
            "short" => CellValue::DShort(0),
            "ushort" => CellValue::DUShort(0),
            "string" => CellValue::DString(Rc::default()),
            "LString" => CellValue::DLString(Rc::default()),
            "int" => CellValue::DInt(0),
            "uint" => CellValue::DUInt(0),
            "sbyte" => CellValue::DSByte(0),
            "byte" => CellValue::DByte(0),
            "bool" => CellValue::DBool(true),
            _ => CellValue::DNone,
        }
    }

    #[allow(dead_code)]
    fn clone_from_other_with_default(v: &CellValue) -> CellValue {
        match v {
            CellValue::DBool(_) => {
                CellValue::DBool(true)
            },
            CellValue::DByte(_) => {
                CellValue::DByte(0)
            },
            CellValue::DInt(_) => {
                CellValue::DInt(0)
            },
            CellValue::DLString(_) => {
                CellValue::DLString(Rc::default())
            },
            CellValue::DShort(_) => {
                CellValue::DShort(0)
            },
            CellValue::DSByte(_) => {
                CellValue::DSByte(0)
            },
            CellValue::DString(_) => {
                CellValue::DString(Rc::default())
            },
            CellValue::DUInt(_) => {
                CellValue::DUInt(0)
            },
            CellValue::DUShort(_) => {
                CellValue::DUShort(0)
            },
            _ => { CellValue::DNone }
        }
    }
}

#[allow(dead_code)]
fn find_block(src: &str) -> usize {
    let mut st: Stack<char> = Stack::new();

    if let Some(start_idx) = src.find('{') {
        let mut idx = start_idx;
        for ref v in src.chars() {
            idx += 1;
            match v {
                '{' => { st.push('{'); },
                '}' => {
                    if let Ok(left) = st.pop() {
                        if left == '{' && st.is_empty() {
                            return idx;
                        }
                    }
                },
                _ => {}
            }
        }

        // TODO: err
        0
    } else {
        0
    }
}

#[allow(dead_code)]
fn collect_value(val: &str, dest: &mut CellValue) {
    // fill-fn
    let fill_elements = |arr: &mut Vec<CellValue>, elements: &Vec<&str>| {
        for e in elements {
            // match type, assert arr is not empty
            match arr[0] {
                CellValue::DBool(_) => {
                    let _ = e.parse::<bool>().map(|v| arr.push(CellValue::DBool(v)));
                },
                CellValue::DByte(_) => {
                    let _ = e.parse::<u8>().map(|v| arr.push(CellValue::DByte(v)));
                },
                CellValue::DInt(_) => {
                    let _ = e.parse::<i32>().map(|v| arr.push(CellValue::DInt(v)));
                },
                CellValue::DLString(_) => {
                    todo!()
                },
                CellValue::DShort(_) => {
                    let _ = e.parse::<i16>().map(|v| arr.push(CellValue::DShort(v)));
                },
                CellValue::DSByte(_) => {
                    let _ = e.parse::<i8>().map(|v| arr.push(CellValue::DSByte(v)));
                },
                CellValue::DString(_) => {
                    arr.push(CellValue::DString(Rc::new(e.to_string())));
                },
                CellValue::DUInt(_) => {
                    let _ = e.parse::<i16>().map(|v| arr.push(CellValue::DShort(v)));
                },
                CellValue::DUShort(_) => {
                    let _ = e.parse::<i16>().map(|v| arr.push(CellValue::DShort(v)));
                },
                _ => { todo!("err") }
            }
        }
    };
    
    let mut start_idx = 1;
    let mut temp: Vec<CellValue> = vec![];

    match dest {
        CellValue::DArray(arr) => {
            let elements: Vec<&str> = val[1..val.len()-1].split(',').collect();
            fill_elements(arr, &elements);
        },
        CellValue::DList(list) => {
            match list[0] {
                CellValue::DArray(ref arr) => {
                    while start_idx < val.len() {
                        let end_idx = find_block(&val[start_idx..]) + start_idx;
                        let mut new_arr = CellValue::DArray(vec![CellValue::clone_from_other_with_default(&arr[0])]);
                        collect_value(&val[start_idx..end_idx], &mut new_arr);
                        temp.push(new_arr);
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.push(v);
                    }
                },
                CellValue::DList(ref lst) => {
                    while start_idx < val.len() {
                        let end_idx = find_block(&val[start_idx..]) + start_idx;
                        let mut new_lst = CellValue::DList(vec![CellValue::clone_from_other_with_default(&lst[0])]);
                        collect_value(&val[start_idx..end_idx], &mut new_lst);
                        temp.push(new_lst); 
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.push(v);
                    }
                },
                _ => {
                    let elements: Vec<&str> = val[1..val.len()-1].split(',').collect();
                    fill_elements(list, &elements);
                }
            }
        },
        _ => { todo!("err") }
    }
}

pub trait ValueToString {
    fn value(&self) -> String;
}

impl ValueToString for CellValue {
    fn value(&self) -> String {
        String::default()
    }
}