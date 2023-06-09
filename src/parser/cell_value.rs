use std::rc::Rc;
use super::{stack::Stack, LSMap};

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
    DCustom(CustomValue),
    DArray(ArrayValue), // first element of arr is one dumb, start from index 1
    DList(ListValue),  // first element of list is one dumb, start from index 1
    DNone(NoneValue),
}

impl CellValue {
    // TODO: process error
    pub fn new(val: &Rc<String>, ty: &Rc<String>, ls_map: &LSMap) -> Self {
        let val_str = val.as_str();
        let ty_str = ty.as_str();

        if val_str.is_empty() {
            return Self::DNone(NoneValue);
        }

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
                    Self::DByte(ByteValue(0))
                }
            }
            "sbyte" => {
                if let Ok(v) = val_str.parse::<i8>() {
                    Self::DSByte(SByteValue(v))
                } else {
                    Self::DSByte(SByteValue(0))
                }
            }
            "LString" => {
                let ls_data = ls_map.as_ref().borrow();
                if ls_data.contains_key(val) {
                    Self::DLString(LStringValue(val.clone(), ls_data[val]))
                } else {
                    Self::DLString(LStringValue(val.clone(), 0))
                }
            },
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
                    Self::DUShort(UShortValue(0))
                }
            }
            "int" => {
                if let Ok(v) = val_str.parse::<i32>() {
                    Self::DInt(IntValue(v))
                } else {
                    Self::DInt(IntValue(0))
                }
            }
            "uint" => {
                if let Ok(v) = val_str.parse::<u32>() {
                    Self::DUInt(UIntValue(v))
                } else {
                    Self::DUInt(UIntValue(0))
                }
            }
            // array or list
            s if s.contains("List") || s.contains("[]") => {
                let mut char_stack: Stack<char> = Stack::new();
                let mut op_stack: Stack<char> = Stack::new();
                let mut keyword_stack: Stack<String> = Stack::new();
                let mut ret = Self::DNone(NoneValue);

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
                                        ret = Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)]));
                                    }
                                }
                            }
                        }
                        '>' => {
                            if char_stack.is_empty() {
                                if let Ok(key) = keyword_stack.pop() {
                                    if let Ok(top) = op_stack.pop() {
                                        if top == '<' && key == "List" {
                                            ret = Self::DList(ListValue(vec![ret]));
                                        }
                                    }
                                }
                            } else {
                                let _ = op_stack.pop();
                                let inner_keyword = take_keyword(&mut char_stack);
                                ret = Self::basic_default_value(&inner_keyword);
                                ret = Self::DList(ListValue(vec![ret]));
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
                    collect_value(val, &mut ret, &ls_map);
                    ret
                } else {
                    // TODO: err
                    Self::DNone(NoneValue)
                }
            },
            // custom
            s => {
                Self::DCustom(CustomValue(Rc::from(String::from(s)), val.clone()))
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
            CellValue::DCustom,
            CellValue::DArray,
            CellValue::DList,
            CellValue::DNone
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

    pub fn is_none(&self) -> bool {
        if let Self::DNone(_) = self {
            true
        } else {
            false
        }
    }

    pub fn get_type(ty: &Rc<String>) -> Self {
        match ty.as_str() {
            "int" => Self::DInt(IntValue::default()),
            "uint" => Self::DUInt(UIntValue::default()),
            "bool" => Self::DBool(BoolValue::default()),
            "byte" => Self::DByte(ByteValue::default()),
            "sbyte" => Self::DSByte(SByteValue::default()),
            "short" => Self::DShort(ShortValue::default()),
            "ushort" => Self::DUShort(UShortValue::default()),
            "string" => Self::DString(StringValue::default()),
            "LString" => Self::DLString(LStringValue::default()),
            // array or list
            s if s.contains("List") || s.contains("[]") => {
                let mut char_stack: Stack<char> = Stack::new();
                let mut op_stack: Stack<char> = Stack::new();
                let mut keyword_stack: Stack<String> = Stack::new();
                let mut ret = Self::DNone(NoneValue);

                let take_keyword = |st: &mut Stack<char>| -> String {
                    let mut s = String::with_capacity(10);
                    while !st.is_empty() {
                        if let Ok(r) = st.pop() {
                            s.push(r)
                        }
                    }
                    s.chars().rev().collect()
                };

                for c in ty.chars() {
                    match c {
                        ']' => {
                            if let Ok(key) = keyword_stack.pop() {
                                if let Ok(top) = op_stack.pop() {
                                    if top == '[' {
                                        ret = Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)]));
                                    }
                                }
                            }
                        }
                        '>' => {
                            if char_stack.is_empty() {
                                if let Ok(key) = keyword_stack.pop() {
                                    if let Ok(top) = op_stack.pop() {
                                        if top == '<' && key == "List" {
                                            ret = Self::DList(ListValue(vec![ret]));
                                        }
                                    }
                                }
                            } else {
                                let _ = op_stack.pop();
                                let inner_keyword = take_keyword(&mut char_stack);
                                ret = Self::basic_default_value(&inner_keyword);
                                ret = Self::DList(ListValue(vec![ret]));
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
                    Self::DNone(NoneValue)
                }
            },
            // custom
            s => {
                Self::DCustom(CustomValue(Rc::from(String::from(s)), Rc::default()))
            }
        }
    }

    //--------------------------------internal---------------------------------------------
    
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
            CellValue::DString,
            CellValue::DLString,
            CellValue::DCustom
        )
    }

    // not include list and array
    fn basic_default_value(key: &str) -> CellValue {
        match key {
            "short" => CellValue::DShort(ShortValue(0)),
            "ushort" => CellValue::DUShort(UShortValue(0)),
            "string" => CellValue::DString(StringValue(Rc::default())),
            "LString" => CellValue::DLString(LStringValue(Rc::default(), usize::default())),
            "int" => CellValue::DInt(IntValue(0)),
            "uint" => CellValue::DUInt(UIntValue(0)),
            "sbyte" => CellValue::DSByte(SByteValue(0)),
            "byte" => CellValue::DByte(ByteValue(0)),
            "bool" => CellValue::DBool(BoolValue(true)),
            "" => CellValue::DNone(NoneValue),
            custom => CellValue::DCustom(CustomValue(Rc::from(String::from(custom)), Rc::default()))
        }
    }

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
                CellValue::DLString(LStringValue(Rc::default(), usize::default()))
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
            CellValue::DCustom(d) => {
                CellValue::DCustom(CustomValue(d.0.clone(), Rc::default()))
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

pub fn find_block(src: &str) -> usize {
    let mut st: Stack<char> = Stack::new();

    if let Some(start_idx) = src.find('{') {
        let mut idx = start_idx;
        for ref v in src.chars() {
            idx += v.len_utf8();
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
fn collect_value(val: &str, dest: &mut CellValue, ls_map: &LSMap) {
    if val.is_empty() {
        return
    }

    // filter whitespace
    let filter_val: String = val.chars().filter(|&c| !c.is_whitespace()).collect();
    let ls_data = ls_map.as_ref().borrow();

    // fill-fn
    let fill_elements = |arr: &mut Vec<CellValue>| {
        if let CellValue::DCustom(ref v) = arr[0] {
            // pattern: {{x,x,x},...}
            let mut idx = 0;
            let slice_val = &filter_val[1..filter_val.len()];
            let ty = v.0.clone();

            while idx < slice_val.len() - 1 {
                let off = find_block(&slice_val[idx..]);
                if off != 0 {
                    arr.push(CellValue::DCustom(CustomValue(ty.clone(), Rc::from(String::from(&slice_val[idx..idx+off])))));
                    idx += off + 1;
                } else {
                    break;
                }
            }
        } else {
            let elements: Vec<&str> = filter_val[1..filter_val.len()-1].split(',').collect();
            for e in elements {
                // match type, assert arr is not empty
                match arr[0] {
                    CellValue::DBool(_) => {
                        if let Err(err) = e.parse::<bool>().map(|v| arr.push(CellValue::DBool( BoolValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    CellValue::DByte(_) => {
                        if let Err(err) = e.parse::<u8>().map(|v| arr.push(CellValue::DByte( ByteValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    CellValue::DInt(_) => {
                        if let Err(err) = e.parse::<i32>().map(|v| arr.push(CellValue::DInt( IntValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    CellValue::DLString(_) => {
                        let key = Rc::from(String::from(e));
                        if ls_data.contains_key(&key) {
                            arr.push(CellValue::DLString(LStringValue(key.clone(), ls_data[&key])))
                        } else {
                            println!("LString translate err");
                        }
                    },
                    CellValue::DShort(_) => {
                        if let Err(err) = e.parse::<i16>().map(|v| arr.push(CellValue::DShort( ShortValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    CellValue::DSByte(_) => {
                        if let Err(err) = e.parse::<i8>().map(|v| arr.push(CellValue::DSByte( SByteValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    CellValue::DString(_) => {
                        arr.push(CellValue::DString( StringValue(Rc::new(e.to_string())) ));
                    },
                    CellValue::DUInt(_) => {
                        if let Err(err) = e.parse::<u32>().map(|v| arr.push(CellValue::DUInt( UIntValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    CellValue::DUShort(_) => {
                        if let Err(err) = e.parse::<u16>().map(|v| arr.push(CellValue::DUShort( UShortValue(v) ))) {
                            println!("{}: src val= {}", err, e);
                        }
                    },
                    _ => { todo!("err") }
                }
            }
        }
    };
    
    let mut start_idx = 1;
    let mut temp: Vec<CellValue> = vec![];

    match dest {
        CellValue::DArray(arr) => {
            fill_elements(&mut arr.0);
        },
        CellValue::DList(list) => {
            match (list.0)[0] {
                CellValue::DArray(ref arr) => {
                    while start_idx < filter_val.len() {
                        let end_idx = find_block(&filter_val[start_idx..]) + start_idx;
                        let mut new_arr = CellValue::DArray(ArrayValue(vec![CellValue::clone_from_other_with_default(&(arr.0)[0])]));
                        collect_value(&filter_val[start_idx..end_idx], &mut new_arr, &ls_map);
                        temp.push(new_arr);
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                },
                CellValue::DList(ref lst) => {
                    while start_idx < filter_val.len() {
                        let end_idx = find_block(&filter_val[start_idx..]) + start_idx;
                        let mut new_lst = CellValue::DList(ListValue(vec![CellValue::clone_from_other_with_default(&(lst.0)[0])]));
                        collect_value(&filter_val[start_idx..end_idx], &mut new_lst, &ls_map);
                        temp.push(new_lst); 
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                },
                _ => {
                    fill_elements(&mut list.0);
                }
            }
        },
        _ => { todo!("err") }
    }
}

pub trait ValueInfo {
    // code
    fn value(&self) -> String;
    // used by array/list code
    fn ty(&self) -> String;
}
#[derive(Default)]
pub struct BoolValue(pub bool);

#[derive(Default)]
pub struct LStringValue(pub Rc<String>, pub usize);

#[derive(Default)]
pub struct StringValue(pub Rc<String>);

#[derive(Default)]
pub struct CustomValue(pub Rc<String>, pub Rc<String>); // (type_str, params)

#[derive(Default)]
pub struct ShortValue(pub i16);

#[derive(Default)]
pub struct UShortValue(pub u16);

#[derive(Default)]
pub struct IntValue(pub i32);

#[derive(Default)]
pub struct UIntValue(pub u32);

#[derive(Default)]
pub struct ByteValue(pub u8);

#[derive(Default)]
pub struct SByteValue(pub i8);

#[derive(Default)]
pub struct ArrayValue(pub Vec<CellValue>);

#[derive(Default)]
pub struct ListValue(pub Vec<CellValue>);

#[derive(Default)]
pub struct NoneValue;

//----------------------------------impl-------------------------------------------

impl ValueInfo for NoneValue {
    fn value(&self) -> String {
        String::from("null")
    }

    fn ty(&self) -> String {
        String::from("none")
    }
}

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
        self.1.to_string()
    }

    fn ty(&self) -> String {
        String::from("int")
    }
}

impl ValueInfo for StringValue {
    fn value(&self) -> String {
        if self.0.is_empty() {
            String::from("null")
        } else if self.0.as_str() == "\"\"" {
            String::from("\"\"")
        } else {
            let mut ret = String::from('\"');
            ret.push_str(&self.0);
            ret.push('\"');
            ret
        }
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

impl ValueInfo for CustomValue {
    fn value(&self) -> String {
        let mut ret = String::from("new ");
        ret.push_str(self.0.as_str());
        ret.push('(');
        ret.push_str(&self.1.as_str()[1..self.1.len() - 1]);
        ret.push(')');
        ret
    }

    fn ty(&self) -> String {
        String::from(self.0.as_str())
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
                    CellValue::DLString(v) => {v.value()},
                    CellValue::DShort(v) => {v.value()},
                    CellValue::DUShort(v) => {v.value()},
                    CellValue::DCustom(v) => {v.value()},
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
                    CellValue::DLString(v) => {v.value()},
                    CellValue::DShort(v) => {v.value()},
                    CellValue::DUShort(v) => {v.value()},
                    CellValue::DCustom(v) => {v.value()},
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