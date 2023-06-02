use std::rc::Rc;
use super::stack::Stack;

macro_rules! get_basic_type_string {
    ($self:ident, $($enum:ident::$variant:ident),+) => {
        match $self {
            $( $enum::$variant(v) => v.ty() ),+,
            _ => String::default(),
        }
    };
}

macro_rules! gen_code {
    ($self:ident, $($enum:ident::$variant:ident),+) => {
        match $self {
            $( $enum::$variant(v) => v.value() ),+,
            _ => String::default(),
        }
    };
}

pub enum CellValue {
    DBool(BoolValue),
    DLString(LStringValue),
    DString(StringValue),
    DShort(ShortValue),
    DUShort(UShortValue),
    DSByte(SByteValue),
    DByte(ByteValue),
    DInt(IntValue),
    DUInt(UIntValue),
    DArray(ArrayValue), // first element of arr is one dumb, start from index 1
    DList(ListValue),  // first element of list is one dumb, start from index 1
    DNone(NoneValue),
}

impl CellValue {
    // TODO: process error
    pub fn new(val: &Rc<String>, ty: &Rc<String>) -> CellValue {
        let val_str = val.as_str();
        let ty_str = ty.as_str();

        match ty_str {
            "bool" => {
                if val_str == "0" {
                    Self::DBool(BoolValue(false))
                } else if val_str == "1" {
                    Self::DBool(BoolValue(true))
                } else {
                    todo!()
                }
            }
            "byte" => {
                if let Ok(v) = val_str.parse::<u8>() {
                    Self::DByte(ByteValue(v))
                } else {
                    todo!()
                }
            }
            "sbyte" => {
                if let Ok(v) = val_str.parse::<i8>() {
                    Self::DSByte(SByteValue(v))
                } else {
                    Self::DSByte(SByteValue(0))
                }
            }
            "LString" => Self::DLString(LStringValue(val.clone())),
            "string" => Self::DString(StringValue(val.clone())),
            "short" => {
                if let Ok(v) = val_str.parse::<i16>() {
                    Self::DShort(ShortValue(v))
                } else {
                    Self::DShort(ShortValue(0))
                }
            }
            "ushort" => {
                if let Ok(v) = val_str.parse::<u16>() {
                    Self::DUShort(UShortValue(v))
                } else {
                    todo!()
                }
            }
            "int" => {
                if let Ok(v) = val_str.parse::<i32>() {
                    Self::DInt(IntValue(v))
                } else {
                    todo!()
                }
            }
            "uint" => {
                if let Ok(v) = val_str.parse::<u32>() {
                    Self::DUInt(UIntValue(v))
                } else {
                    todo!()
                }
            }
            // array or list
            _ => {
                let mut char_stack: Stack<char> = Stack::new();
                let mut op_stack: Stack<char> = Stack::new();
                let mut keyword_stack: Stack<String> = Stack::new();
                let mut ret = CellValue::DNone(NoneValue);

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
                                        ret = CellValue::DArray(ArrayValue(vec![CellValue::basic_default_value(&key)]));
                                    }
                                }
                            }
                        }
                        '>' => {
                            if char_stack.is_empty() {
                                if let Ok(key) = keyword_stack.pop() {
                                    if let Ok(top) = op_stack.pop() {
                                        if top == '<' && key == "List" {
                                            ret = CellValue::DList(ListValue(vec![ret]));
                                        }
                                    }
                                }
                            } else {
                                let _ = op_stack.pop();
                                let inner_keyword = take_keyword(&mut char_stack);
                                ret = CellValue::basic_default_value(&inner_keyword);
                                ret = CellValue::DList(ListValue(vec![ret]));
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
                    collect_value(val_str, &mut ret);
                    ret
                } else {
                    // TODO: err
                    CellValue::DNone(NoneValue)
                }
            }
        }
    }

    pub fn gen_code(&self) -> String {
        gen_code!(
            self,
            CellValue::DBool,
            CellValue::DByte, 
            CellValue::DSByte, 
            CellValue::DInt, 
            CellValue::DUInt, 
            CellValue::DShort, 
            CellValue::DUShort, 
            CellValue::DString,
            CellValue::DLString,
            CellValue::DArray,
            CellValue::DList
        )
    }

    pub fn is_lstring(&self) -> bool {
        if let Self::DLString(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_lstring_arr(&self) -> bool {
        match self {
            Self::DList(v) => {
                if let Self::DLString(_) = (v.0)[0] {
                    true
                } else {
                    false
                }
            },
            Self::DArray(v) => {
                if let Self::DLString(_) = (v.0)[0] {
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
    
    fn get_basic_type_string(&self) -> String {
        get_basic_type_string!(
            self, 
            CellValue::DBool, 
            CellValue::DByte, 
            CellValue::DSByte, 
            CellValue::DInt, 
            CellValue::DUInt, 
            CellValue::DShort, 
            CellValue::DUShort, 
            CellValue::DString
        )
    }

    // not include list and array
    fn basic_default_value(key: &str) -> CellValue {
        match key {
            "short" => CellValue::DShort(ShortValue(0)),
            "ushort" => CellValue::DUShort(UShortValue(0)),
            "string" => CellValue::DString(StringValue(Rc::default())),
            "LString" => CellValue::DLString(LStringValue(Rc::default())),
            "int" => CellValue::DInt(IntValue(0)),
            "uint" => CellValue::DUInt(UIntValue(0)),
            "sbyte" => CellValue::DSByte(SByteValue(0)),
            "byte" => CellValue::DByte(ByteValue(0)),
            "bool" => CellValue::DBool(BoolValue(true)),
            _ => CellValue::DNone(NoneValue),
        }
    }

    #[allow(dead_code)]
    fn clone_from_other_with_default(v: &CellValue) -> CellValue {
        match v {
            CellValue::DBool(_) => {
                CellValue::DBool(BoolValue(true))
            },
            CellValue::DByte(_) => {
                CellValue::DByte(ByteValue(0))
            },
            CellValue::DInt(_) => {
                CellValue::DInt(IntValue(0))
            },
            CellValue::DLString(_) => {
                CellValue::DLString(LStringValue(Rc::default()))
            },
            CellValue::DShort(_) => {
                CellValue::DShort(ShortValue(0))
            },
            CellValue::DSByte(_) => {
                CellValue::DSByte(SByteValue(0))
            },
            CellValue::DString(_) => {
                CellValue::DString(StringValue(Rc::default()))
            },
            CellValue::DUInt(_) => {
                CellValue::DUInt(UIntValue(0))
            },
            CellValue::DUShort(_) => {
                CellValue::DUShort(UShortValue(0))
            },
            CellValue::DArray(arr) => {
                if arr.0.is_empty() {
                    CellValue::DNone(NoneValue)
                } else {
                    CellValue::DArray(ArrayValue(vec![CellValue::clone_from_other_with_default(&(arr.0)[0])]))
                }
            },
            CellValue::DList(lst) => {
                if lst.0.is_empty() {
                    CellValue::DNone(NoneValue)
                } else {
                    CellValue::DList(ListValue(vec![CellValue::clone_from_other_with_default(&(lst.0)[0])]))
                }
            },
            _ => { CellValue::DNone(NoneValue) }
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
    if val.is_empty() {
        return
    }
    // fill-fn
    let fill_elements = |arr: &mut Vec<CellValue>, elements: &Vec<&str>| {
        for e in elements {
            // match type, assert arr is not empty
            match arr[0] {
                CellValue::DBool(_) => {
                    let _ = e.parse::<bool>().map(|v| arr.push(CellValue::DBool( BoolValue(v) )));
                },
                CellValue::DByte(_) => {
                    let _ = e.parse::<u8>().map(|v| arr.push(CellValue::DByte( ByteValue(v) )));
                },
                CellValue::DInt(_) => {
                    let _ = e.parse::<i32>().map(|v| arr.push(CellValue::DInt( IntValue(v) )));
                },
                CellValue::DLString(_) => {
                    todo!()
                },
                CellValue::DShort(_) => {
                    let _ = e.parse::<i16>().map(|v| arr.push(CellValue::DShort( ShortValue(v) )));
                },
                CellValue::DSByte(_) => {
                    let _ = e.parse::<i8>().map(|v| arr.push(CellValue::DSByte( SByteValue(v) )));
                },
                CellValue::DString(_) => {
                    arr.push(CellValue::DString( StringValue(Rc::new(e.to_string())) ));
                },
                CellValue::DUInt(_) => {
                    let _ = e.parse::<u32>().map(|v| arr.push(CellValue::DUInt( UIntValue(v) )));
                },
                CellValue::DUShort(_) => {
                    let _ = e.parse::<u16>().map(|v| arr.push(CellValue::DUShort( UShortValue(v) )));
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
            fill_elements(&mut arr.0, &elements);
        },
        CellValue::DList(list) => {
            match (list.0)[0] {
                CellValue::DArray(ref arr) => {
                    while start_idx < val.len() {
                        let end_idx = find_block(&val[start_idx..]) + start_idx;
                        let mut new_arr = CellValue::DArray(ArrayValue(vec![CellValue::clone_from_other_with_default(&(arr.0)[0])]));
                        collect_value(&val[start_idx..end_idx], &mut new_arr);
                        temp.push(new_arr);
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                },
                CellValue::DList(ref lst) => {
                    while start_idx < val.len() {
                        let end_idx = find_block(&val[start_idx..]) + start_idx;
                        let mut new_lst = CellValue::DList(ListValue(vec![CellValue::clone_from_other_with_default(&(lst.0)[0])]));
                        collect_value(&val[start_idx..end_idx], &mut new_lst);
                        temp.push(new_lst); 
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                },
                _ => {
                    let elements: Vec<&str> = val[1..val.len()-1].split(',').collect();
                    fill_elements(&mut list.0, &elements);
                }
            }
        },
        _ => { todo!("err") }
    }
}

pub trait ValueInfo {
    fn value(&self) -> String;
    fn ty(&self) -> String;
}
pub struct BoolValue(bool);
pub struct LStringValue(Rc<String>);
pub struct StringValue(Rc<String>);
pub struct ShortValue(i16);
pub struct UShortValue(u16);
pub struct IntValue(i32);
pub struct UIntValue(u32);
pub struct ByteValue(u8);
pub struct SByteValue(i8);
pub struct ArrayValue(Vec<CellValue>);
pub struct ListValue(Vec<CellValue>);
pub struct NoneValue;

//----------------------------------impl-------------------------------------------

impl ValueInfo for BoolValue {
    fn value(&self) -> String {
        if self.0 == false {
            String::from("false")
        } else {
            String::from("true")
        }
    }

    fn ty(&self) -> String {
        String::from("bool")
    }
}

impl ValueInfo for LStringValue {
    fn value(&self) -> String {
        todo!()
    }

    fn ty(&self) -> String {
        todo!()
    }
}

impl ValueInfo for StringValue {
    fn value(&self) -> String {
        let mut ret = String::from('\"');
        ret.push_str(&self.0);
        ret.push('\"');
        ret
    }

    fn ty(&self) -> String {
        String::from("string")
    }
}

impl ValueInfo for ShortValue {
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn ty(&self) -> String {
        String::from("short")
    }
}

impl ValueInfo for UShortValue {
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn ty(&self) -> String {
        String::from("ushort")
    }
}

impl ValueInfo for IntValue {
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn ty(&self) -> String {
        String::from("int")
    }
}

impl ValueInfo for UIntValue {
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn ty(&self) -> String {
        String::from("uint")
    }
}

impl ValueInfo for ByteValue {
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn ty(&self) -> String {
        String::from("byte")
    }
}

impl ValueInfo for SByteValue {
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn ty(&self) -> String {
        String::from("sbyte")
    }
}

impl ValueInfo for ArrayValue {
    fn value(&self) -> String {
        if self.0.is_empty() {
            String::default()
        } else {
            let mut ret = String::from("new ");
            ret.push_str(&self.ty());
            ret.push('{');

            for v in self.0.iter().skip(1) {
                let s = match v {
                    CellValue::DBool(v) => {v.value()},
                    CellValue::DByte(v) => {v.value()},
                    CellValue::DSByte(v) => {v.value()},
                    CellValue::DInt(v) => {v.value()},
                    CellValue::DUInt(v) => {v.value()},
                    CellValue::DString(v) => {v.value()},
                    CellValue::DShort(v) => {v.value()},
                    CellValue::DUShort(v) => {v.value()},
                    _ => {String::default()}
                };
                ret.push_str(&s);
                ret.push(',');
            }
            if ret.ends_with(',') {
                ret.remove(ret.len() - 1);
            }
            ret.push('}');
            ret
        }
    }

    fn ty(&self) -> String {
        let mut ty = (self.0)[0].get_basic_type_string();
        ty.push_str("[]");
        ty
    }
}

impl ValueInfo for ListValue {
    fn value(&self) -> String {
        if self.0.is_empty() {
            String::default()
        } else {
            let mut ret = String::from("new ");
            ret.push_str(&self.ty());
            ret.push('{');

            for v in self.0.iter().skip(1) {
                let s = match v {
                    CellValue::DBool(v) => {v.value()},
                    CellValue::DByte(v) => {v.value()},
                    CellValue::DSByte(v) => {v.value()},
                    CellValue::DInt(v) => {v.value()},
                    CellValue::DUInt(v) => {v.value()},
                    CellValue::DString(v) => {v.value()},
                    CellValue::DShort(v) => {v.value()},
                    CellValue::DUShort(v) => {v.value()},
                    CellValue::DArray(v) => {v.value()},
                    CellValue::DList(v) => {v.value()}
                    _ => {String::default()}
                };
                ret.push_str(&s);
                ret.push(',');
            }

            if ret.ends_with(',') {
                ret.remove(ret.len() - 1);
            }
            ret.push('}');
            ret
        }
    }

    fn ty(&self) -> String {
        let mut ty = String::default();
        ty.push_str("List<");

        let first = self.0.first().unwrap();
        match first {
            CellValue::DArray(v) => { ty.push_str(&v.ty()); }
            CellValue::DList(v) => { ty.push_str(&v.ty()); }
            // basic type
            _ => {
                ty.push_str(&first.get_basic_type_string());
            }
        }
        ty.push('>');
        ty
    }
}