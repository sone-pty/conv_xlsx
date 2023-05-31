use crate::defs::ItemStr;
use std::rc::Rc;

use super::stack::Stack;

enum DataType {
    DBool(bool),
    DLString(Rc<String>),
    DString(Rc<String>),
    DShort(i16),
    DUShort(u16),
    DSByte(i8),
    DByte(u8),
    DInt(i32),
    DUInt(u32),
    DArray(Vec<DataType>), // first element of arr is one dumb, start from index 1
    DList(Vec<DataType>),  // first element of list is one dumb, start from index 1
    DNone,
}

impl DataType {
    // TODO: process error
    fn new(val: &Rc<String>, ty: &Rc<String>) -> DataType {
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
                let mut ret = DataType::DNone;

                let take_keyword = |st: &mut Stack<char>| {
                    let mut s = String::with_capacity(10);
                    while !st.is_empty() {
                        if let Ok(r) = st.pop() {
                            s.push(r);
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
                                        ret = DataType::DArray(vec![DataType::basic_default_value(&key)]);
                                    }
                                }
                            }
                        }
                        '>' => {
                            if char_stack.is_empty() {
                            } else {
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
                    // TODO
                    DataType::DNone
                }
            }
        }
    }

    // not include list and array
    fn basic_default_value(key: &str) -> DataType {
        match key {
            "short" => DataType::DShort(0),
            "ushort" => DataType::DUShort(0),
            "string" => DataType::DString(Rc::default()),
            "LString" => DataType::DLString(Rc::default()),
            "int" => DataType::DInt(0),
            "uint" => DataType::DUInt(0),
            "sbyte" => DataType::DSByte(0),
            "byte" => DataType::DByte(0),
            "bool" => DataType::DBool(true),
            _ => DataType::DNone,
        }
    }
}

pub trait ValueToString {
    fn value(&self) -> String;
}

pub struct CellValue<T: ValueToString> {
    raw_data: ItemStr,
    data_type: DataType,
    impl_obj: Option<T>,
}

fn find_block<'a>(src: &'a str) -> Option<&'a str> {
    if src.starts_with('{') {
        if let Some(idx) = src.find('}') {
            Some(&src[1..idx])
        } else { None }
    } else {
        None
    }
}

fn collect_value(val: &str, dest: &mut DataType) {
    match dest {
        DataType::DArray(arr) => {
            let elements: Vec<&str> = val.split(',').collect();
            
            for e in elements {
                // match type, assert arr is not empty
                match arr[0] {
                    DataType::DBool(_) => {
                        let _ = e.parse::<bool>().map(|v| arr.push(DataType::DBool(v)));
                    },
                    DataType::DByte(_) => {
                        let _ = e.parse::<u8>().map(|v| arr.push(DataType::DByte(v)));
                    },
                    DataType::DInt(_) => {
                        let _ = e.parse::<i32>().map(|v| arr.push(DataType::DInt(v)));
                    },
                    DataType::DLString(_) => {
                        todo!()
                    },
                    DataType::DShort(_) => {
                        let _ = e.parse::<i16>().map(|v| arr.push(DataType::DShort(v)));
                    },
                    DataType::DSByte(_) => {
                        let _ = e.parse::<i8>().map(|v| arr.push(DataType::DSByte(v)));
                    },
                    DataType::DString(_) => {
                        arr.push(DataType::DString(Rc::new(e.to_string())));
                    },
                    DataType::DUInt(_) => {
                        let _ = e.parse::<i16>().map(|v| arr.push(DataType::DShort(v)));
                    },
                    DataType::DUShort(_) => {
                        let _ = e.parse::<i16>().map(|v| arr.push(DataType::DShort(v)));
                    },
                    _ => { todo!("err") }
                }
            }
        },
        DataType::DList(list) => {
            
        },
        _ => { todo!("err") }
    }
}

#[test]
fn test_parse() {
    //let val_str = "{{1,2,3}, {3,4,5}}";
    //let ty_str = "List<short[]>";
    let val_str = "{1,2,3,4}";
    let ty_str = "short[]";
    
    let mut char_stack: Stack<char> = Stack::new();
    let mut op_stack: Stack<char> = Stack::new();
    let mut keyword_stack: Stack<String> = Stack::new();
    let mut ret = DataType::DNone;

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
                            ret = DataType::DArray(vec![DataType::basic_default_value(&key)]);
                        }
                    }
                }
            }
            '>' => {
                if char_stack.is_empty() {
                    if let Ok(key) = keyword_stack.pop() {
                        if let Ok(top) = op_stack.pop() {
                            if top == '<' && key == "List" {
                                ret = DataType::DList(vec![ret]);
                            }
                        }
                    }
                } else {
                    let inner_keyword = take_keyword(&mut char_stack);
                    ret = DataType::basic_default_value(&inner_keyword);
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
        collect_value(&val_str[1..val_str.len() - 1], &mut ret);

        match ret {
            DataType::DList(ref mut list) => {

            },
            DataType::DArray(arr) => {
                for e in arr {
                    match e {
                        DataType::DShort(v) => {
                            println!("{}", v);
                        },
                        _ => {}
                    }
                }
            },
            _ => { todo!("err") }
        }
    } else {
        println!("format err");
    }
}