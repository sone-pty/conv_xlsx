use std::{rc::Rc, io::{Write, Result}, cell::RefCell, collections::HashMap, vec};
use vnlex::{cursor, lexer::{Lexer, self}, token::tokenizers::{self, KIND_WHITESPACE_OR_COMMENT, KIND_KEYWORD}};

use super::{stack::Stack, LSMap, ENMap, fk_value::split_val, LSEmptyMap, fsm::{StateMachine, TypeMachine}};

macro_rules! get_basic_type_string {
    ($self:ident, $stream:ident, $($enum:ident::$variant:ident),+) => {
        match $self {
            $( $enum::$variant(v) => v.ty($stream) ),+,
            _ => Ok(())
        }
    };
}

macro_rules! gen_code {
    ($self:ident, $stream:ident, $($enum:ident::$variant:ident),+) => {
        match $self {
            $( $enum::$variant(v) => v.value($stream) ),+,
        }
    };
}

macro_rules! define_getters {
    ($name1:ident, $name2:ident, $variant:ident, $type:ty) => {
        fn $name1(self) -> Option<$type> {
            match self {
                Self::$variant(v) => Some(v),
                _ => None,
            }
        }

        fn $name2(&mut self) -> Option<&mut $type> {
            match self {
                Self::$variant(ref mut v) => Some(v),
                _ => None,
            }
        }
    };
}

macro_rules! write_value_to_stream {
    ($vv:ident, $stream:ident, $($enum:ident::$variant:ident),+) => {
        match $vv {
            $( $enum::$variant(v) => { v.value($stream)?; } ),+,
            _ => { $stream.write("".as_bytes())?; }
        }
    };
}

pub enum CellValue {
    DEnum(EnumValue),
    DBool(BoolValue),
    DLString(LStringValue),
    DString(StringValue),
    DShort(ShortValue),
    DUShort(UShortValue),
    DSByte(SByteValue),
    DByte(ByteValue),
    DInt(IntValue),
    DUInt(UIntValue),
    DFloat(FloatValue),
    DDouble(DoubleValue),
    DCustom(CustomValue),
    DShortList(ShortListValue),
    DTuple(TupleValue),
    DValueTuple(ValueTupleValue),
    DArray(ArrayValue), // first element of arr is one dumb, start from index 1
    DList(ListValue),  // first element of list is one dumb, start from index 1
    DNone(NoneValue),
    DError(ErrorValue)
}

impl CellValue {
    pub fn new(val: &Rc<String>, ty: &Rc<String>, ls_map: &LSMap, ls_empty_map: &LSEmptyMap, ident: &Rc<String>, enmaps: &Rc<RefCell<HashMap<String, ENMap>>>, base_name: &str, row: usize, col: usize) -> Self {
        let val_str = val.as_str();
        let ty_str = ty.as_str();
        let pos = (row, col);

        if val_str.is_empty() && ty_str != "LString" && ty_str != "Lstring" || val_str == "None" {
            return Self::DNone(NoneValue(ty.clone()));
        }

        match ty_str {
            "bool" => {
                if val_str == "0" || val_str == "false" || val_str == "FALSE" {
                    Self::DBool(BoolValue(false))
                } else if val_str == "1" || val_str == "true" || val_str == "TRUE" {
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
            "LString" | "Lstring" => {
                let ls_data = ls_map.as_ref().borrow();
                if val.is_empty() {
                    if ls_empty_map.contains_key(&pos) && !ls_empty_map[&pos].is_empty() {
                        Self::DLString(LStringValue(val.clone(), ls_empty_map[&pos][0]))
                    } else {
                        Self::DLString(LStringValue(val.clone(), -1))
                    }
                } else {
                    if ls_data.contains_key(val) {
                        Self::DLString(LStringValue(val.clone(), ls_data[val]))
                    } else {
                        Self::DLString(LStringValue(val.clone(), -1))
                    }
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
            "float" => {
                if let Ok(v) = val_str.parse::<f32>() {
                    Self::DFloat(FloatValue(v))
                } else {
                    Self::DFloat(FloatValue(0_f32))
                }
            }
            "double" => {
                if let Ok(v) = val_str.parse::<f64>() {
                    Self::DDouble(DoubleValue(v))
                } else {
                    Self::DDouble(DoubleValue(0_f64))
                }
            }
            "enum" => {
                enmaps.borrow().get(ident.as_str()).map(|map| {
                    let binding = map.borrow();
                    let v = binding.get(&Some(val.clone()));
                    if v.is_none() {
                        Self::DEnum(EnumValue(ident.clone(), Rc::default(), Rc::default()))
                    } else {
                        Self::DEnum(EnumValue(ident.clone(), v.as_ref().unwrap().as_ref().unwrap().clone(), Rc::from(String::from(base_name))))
                    }
                }).unwrap_or(Self::DEnum(EnumValue(Rc::default(), Rc::default(), Rc::default())))
            }
            "ShortList" => {
                let mut ret = Self::DShortList(ShortListValue::default());
                collect_value(val, &mut ret, &ls_map, &ls_empty_map, &pos);
                ret
            }
            s if s.contains("ValueTuple") => {
                let mut ch_stack = Stack::<char>::new();
                // 0-List, 1-Tuple
                let mut key_stack = Stack::<u8>::new();
                let mut obj_stack = Stack::<CellValue>::new();

                for v in s.chars() {
                    match v {
                        '<' => {
                            let key = Self::take_keyword(&mut ch_stack);
                            match &key[..] {
                                "ValueTuple" => {
                                    key_stack.push(1);
                                    obj_stack.push(Self::DValueTuple(ValueTupleValue::default()));
                                }
                                "List" => {
                                    key_stack.push(0);
                                    obj_stack.push(Self::DList(ListValue::default()));
                                }
                                _ => {}
                            }
                        }
                        // in tuple
                        ',' => {
                            if !ch_stack.is_empty() {
                                if let Some(ValueTupleValue(data)) = obj_stack.peek_mut().unwrap().get_valuetuple_ref_value() {
                                    let key = Self::take_keyword(&mut ch_stack);
                                    data.push(Self::basic_default_value(&key));
                                }
                            }
                        }
                        '>' => {
                            if let Ok(k) = key_stack.pop() {
                                if k == 0 {
                                    // list
                                    if let Some(ListValue(mut data)) = obj_stack.pop().unwrap().get_list_value() {
                                        if !ch_stack.is_empty() {
                                            let key = Self::take_keyword(&mut ch_stack);
                                            data.push(Self::basic_default_value(&key));
                                        }

                                        if let Some(idx) = key_stack.peek() {
                                            if *idx == 0 {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ListValue(varr)) = varr.get_list_ref_value() {
                                                        varr.push(Self::DList(ListValue(data)));
                                                    }
                                                });
                                            } else {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ValueTupleValue(varr)) = varr.get_valuetuple_ref_value() {
                                                        varr.push(Self::DList(ListValue(data)));
                                                    }
                                                });
                                            }
                                        } else {
                                            obj_stack.push(Self::DList(ListValue(data)));
                                            key_stack.push(0);
                                        }
                                    } else {
                                        todo!("err")
                                    }
                                } else if k == 1 {
                                    // tuple
                                    if let Some(ValueTupleValue(mut data)) = obj_stack.pop().unwrap().get_valuetuple_value() {
                                        if !ch_stack.is_empty() {
                                            let key = Self::take_keyword(&mut ch_stack);
                                            data.push(Self::basic_default_value(&key));
                                        }

                                        if let Some(idx) = key_stack.peek() {
                                            if *idx == 0 {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ListValue(varr)) = varr.get_list_ref_value() {
                                                        varr.push(Self::DValueTuple(ValueTupleValue(data)));
                                                    }
                                                });
                                            } else {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ValueTupleValue(varr)) = varr.get_valuetuple_ref_value() {
                                                        varr.push(Self::DValueTuple(ValueTupleValue(data)));
                                                    }
                                                });
                                            }
                                        } else {
                                            obj_stack.push(Self::DValueTuple(ValueTupleValue(data)));
                                            key_stack.push(1);
                                        }
                                    } else {
                                        todo!("err")
                                    }
                                }
                            }
                        }
                        ']' => {
                            if !ch_stack.is_empty() {
                                let key = Self::take_keyword(&mut ch_stack);
                                if let Some(idx) = key_stack.peek() {
                                    if *idx == 0 {
                                        if let Some(ListValue(data)) = obj_stack.peek_mut().unwrap().get_list_ref_value() {
                                            data.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                        }
                                    } else {
                                        if let Some(ValueTupleValue(data)) = obj_stack.peek_mut().unwrap().get_valuetuple_ref_value() {
                                            data.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                        }
                                    }
                                } else {
                                    obj_stack.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                }
                            } else if let Ok(idx) = key_stack.pop() {
                                if idx == 1 {
                                    if let Some(ValueTupleValue(data)) = obj_stack.pop().unwrap().get_valuetuple_value() {
                                        obj_stack.push(Self::DArray(ArrayValue(vec![Self::DValueTuple(ValueTupleValue(data))])));
                                    }
                                }
                            } else {
                                todo!("err")
                            }
                        }
                        // skip whitespace && '['
                        ' ' | '[' => {}
                        _ => { ch_stack.push(v); }
                    }
                }

                obj_stack.pop().map(|mut v| {
                    collect_value(val, &mut v, ls_map, ls_empty_map, &pos);
                    v
                }).unwrap()
            }
            s if s.contains("Tuple") => {
                let mut ch_stack = Stack::<char>::new();
                // 0-List, 1-Tuple
                let mut key_stack = Stack::<u8>::new();
                let mut obj_stack = Stack::<CellValue>::new();

                for v in s.chars() {
                    match v {
                        '<' => {
                            let key = Self::take_keyword(&mut ch_stack);
                            match &key[..] {
                                "Tuple" => {
                                    key_stack.push(1);
                                    obj_stack.push(Self::DTuple(TupleValue::default()));
                                }
                                "List" => {
                                    key_stack.push(0);
                                    obj_stack.push(Self::DList(ListValue::default()));
                                }
                                _ => {}
                            }
                        }
                        // in tuple
                        ',' => {
                            if !ch_stack.is_empty() {
                                if let Some(TupleValue(data)) = obj_stack.peek_mut().unwrap().get_tuple_ref_value() {
                                    let key = Self::take_keyword(&mut ch_stack);
                                    data.push(Self::basic_default_value(&key));
                                }
                            }
                        }
                        '>' => {
                            if let Ok(k) = key_stack.pop() {
                                if k == 0 {
                                    // list
                                    if let Some(ListValue(mut data)) = obj_stack.pop().unwrap().get_list_value() {
                                        if !ch_stack.is_empty() {
                                            let key = Self::take_keyword(&mut ch_stack);
                                            data.push(Self::basic_default_value(&key));
                                        }

                                        if let Some(idx) = key_stack.peek() {
                                            if *idx == 0 {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ListValue(varr)) = varr.get_list_ref_value() {
                                                        varr.push(Self::DList(ListValue(data)));
                                                    }
                                                });
                                            } else {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(TupleValue(varr)) = varr.get_tuple_ref_value() {
                                                        varr.push(Self::DList(ListValue(data)));
                                                    }
                                                });
                                            }
                                        } else {
                                            obj_stack.push(Self::DList(ListValue(data)));
                                            key_stack.push(0);
                                        }
                                    } else {
                                        todo!("err")
                                    }
                                } else if k == 1 {
                                    // tuple
                                    if let Some(TupleValue(mut data)) = obj_stack.pop().unwrap().get_tuple_value() {
                                        if !ch_stack.is_empty() {
                                            let key = Self::take_keyword(&mut ch_stack);
                                            data.push(Self::basic_default_value(&key));
                                        }

                                        if let Some(idx) = key_stack.peek() {
                                            if *idx == 0 {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ListValue(varr)) = varr.get_list_ref_value() {
                                                        varr.push(Self::DTuple(TupleValue(data)));
                                                    }
                                                });
                                            } else {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(TupleValue(varr)) = varr.get_tuple_ref_value() {
                                                        varr.push(Self::DTuple(TupleValue(data)));
                                                    }
                                                });
                                            }
                                        } else {
                                            obj_stack.push(Self::DTuple(TupleValue(data)));
                                            key_stack.push(1);
                                        }
                                    } else {
                                        todo!("err")
                                    }
                                }
                            }
                        }
                        ']' => {
                            if !ch_stack.is_empty() {
                                let key = Self::take_keyword(&mut ch_stack);
                                if let Some(idx) = key_stack.peek() {
                                    if *idx == 0 {
                                        if let Some(ListValue(data)) = obj_stack.peek_mut().unwrap().get_list_ref_value() {
                                            data.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                        }
                                    } else {
                                        if let Some(TupleValue(data)) = obj_stack.peek_mut().unwrap().get_tuple_ref_value() {
                                            data.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                        }
                                    }
                                } else {
                                    obj_stack.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                }
                            } else if let Ok(idx) = key_stack.pop() {
                                if idx == 1 {
                                    if let Some(TupleValue(data)) = obj_stack.pop().unwrap().get_tuple_value() {
                                        obj_stack.push(Self::DArray(ArrayValue(vec![Self::DTuple(TupleValue(data))])));
                                    }
                                }
                            } else {
                                todo!("err")
                            }
                        }
                        // skip whitespace && '['
                        ' ' | '[' => {}
                        _ => { ch_stack.push(v); }
                    }
                }

                obj_stack.pop().map(|mut v| {
                    collect_value(val, &mut v, ls_map, ls_empty_map, &pos);
                    v
                }).unwrap()
            }
            // array or list
            s if s.contains("List") || s.contains("[]") => {
                let mut char_stack: Stack<char> = Stack::new();
                let mut op_stack: Stack<char> = Stack::new();
                let mut keyword_stack: Stack<String> = Stack::new();
                let mut ret = Self::DError(ErrorValue);

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
                                let inner_keyword = Self::take_keyword(&mut char_stack);
                                ret = Self::basic_default_value(&inner_keyword);
                                ret = Self::DList(ListValue(vec![ret]));
                            }
                        }
                        '[' | '<' => {
                            op_stack.push(c);
                            keyword_stack.push(Self::take_keyword(&mut char_stack));
                        }
                        c => {
                            char_stack.push(c);
                        }
                    }
                }
                
                if op_stack.is_empty() {
                    collect_value(val, &mut ret, &ls_map, ls_empty_map, &pos);
                    ret
                } else {
                    // TODO: err
                    Self::DError(ErrorValue)
                }
            },
            // custom
            s => {
                Self::DCustom(CustomValue(Rc::from(String::from(s)), val.clone()))
            }
        }
    }

    pub fn gen_code<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        gen_code!(
            self,
            stream,
            CellValue::DEnum,
            CellValue::DBool,
            CellValue::DByte, 
            CellValue::DSByte, 
            CellValue::DInt, 
            CellValue::DUInt,
            CellValue::DFloat,
            CellValue::DDouble,
            CellValue::DShort, 
            CellValue::DUShort, 
            CellValue::DString,
            CellValue::DLString,
            CellValue::DCustom,
            CellValue::DShortList,
            CellValue::DTuple,
            CellValue::DValueTuple,
            CellValue::DArray,
            CellValue::DList,
            CellValue::DNone,
            CellValue::DError
        )
    }

    pub fn is_enum(&self) -> bool {
        if let Self::DEnum(_) = self {
            true
        } else {
            false
        }
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

    pub fn is_arr_or_list(&self) -> bool {
        if let Self::DArray(_) = self {
            true
        } else if let Self::DList(_) = self {
            true
        } else {
            false
        }
    }

    pub fn get_type(ty: &Rc<String>) -> Self {
        match ty.as_str() {
            "int" => Self::DInt(IntValue::default()),
            "uint" => Self::DUInt(UIntValue::default()),
            "enum" => Self::DEnum(EnumValue::default()),
            "bool" => Self::DBool(BoolValue::default()),
            "byte" => Self::DByte(ByteValue::default()),
            "sbyte" => Self::DSByte(SByteValue::default()),
            "short" => Self::DShort(ShortValue::default()),
            "ushort" => Self::DUShort(UShortValue::default()),
            "float" => Self::DFloat(FloatValue::default()),
            "double" => Self::DDouble(DoubleValue::default()),
            "string" => Self::DString(StringValue::default()),
            "LString" | "Lstring" => Self::DLString(LStringValue::default()),
            "ShortList" => Self::DShortList(ShortListValue::default()),
            s if s.contains("Tuple") => {
                let mut ch_stack = Stack::<char>::new();
                // 0-List, 1-Tuple
                let mut key_stack = Stack::<u8>::new();
                let mut obj_stack = Stack::<CellValue>::new();

                for v in s.chars() {
                    match v {
                        '<' => {
                            let key = Self::take_keyword(&mut ch_stack);
                            match &key[..] {
                                "Tuple" => {
                                    key_stack.push(1);
                                    obj_stack.push(Self::DTuple(TupleValue::default()));
                                }
                                "List" => {
                                    key_stack.push(0);
                                    obj_stack.push(Self::DList(ListValue::default()));
                                }
                                _ => {}
                            }
                        }
                        // in tuple
                        ',' => {
                            if !ch_stack.is_empty() {
                                if let Some(TupleValue(data)) = obj_stack.peek_mut().unwrap().get_tuple_ref_value() {
                                    let key = Self::take_keyword(&mut ch_stack);
                                    data.push(Self::basic_default_value(&key));
                                }
                            }
                        }
                        '>' => {
                            if let Ok(k) = key_stack.pop() {
                                if k == 0 {
                                    // list
                                    if let Some(ListValue(mut data)) = obj_stack.pop().unwrap().get_list_value() {
                                        if !ch_stack.is_empty() {
                                            let key = Self::take_keyword(&mut ch_stack);
                                            data.push(Self::basic_default_value(&key));
                                        }

                                        if let Some(idx) = key_stack.peek() {
                                            if *idx == 0 {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ListValue(varr)) = varr.get_list_ref_value() {
                                                        varr.push(Self::DList(ListValue(data)));
                                                    }
                                                });
                                            } else {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(TupleValue(varr)) = varr.get_tuple_ref_value() {
                                                        varr.push(Self::DList(ListValue(data)));
                                                    }
                                                });
                                            }
                                        } else {
                                            obj_stack.push(Self::DList(ListValue(data)));
                                            key_stack.push(0);
                                        }
                                    } else {
                                        todo!("err")
                                    }
                                } else if k == 1 {
                                    // tuple
                                    if let Some(TupleValue(mut data)) = obj_stack.pop().unwrap().get_tuple_value() {
                                        if !ch_stack.is_empty() {
                                            let key = Self::take_keyword(&mut ch_stack);
                                            data.push(Self::basic_default_value(&key));
                                        }

                                        if let Some(idx) = key_stack.peek() {
                                            if *idx == 0 {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(ListValue(varr)) = varr.get_list_ref_value() {
                                                        varr.push(Self::DTuple(TupleValue(data)));
                                                    }
                                                });
                                            } else {
                                                obj_stack.peek_mut().map(|varr| {
                                                    if let Some(TupleValue(varr)) = varr.get_tuple_ref_value() {
                                                        varr.push(Self::DTuple(TupleValue(data)));
                                                    }
                                                });
                                            }
                                        } else {
                                            obj_stack.push(Self::DTuple(TupleValue(data)));
                                            key_stack.push(1);
                                        }
                                    } else {
                                        todo!("err")
                                    }
                                }
                            }
                        }
                        ']' => {
                            if !ch_stack.is_empty() {
                                let key = Self::take_keyword(&mut ch_stack);
                                if let Some(idx) = key_stack.peek() {
                                    if *idx == 0 {
                                        if let Some(ListValue(data)) = obj_stack.peek_mut().unwrap().get_list_ref_value() {
                                            data.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                        }
                                    } else {
                                        if let Some(TupleValue(data)) = obj_stack.peek_mut().unwrap().get_tuple_ref_value() {
                                            data.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                        }
                                    }
                                } else {
                                    obj_stack.push(Self::DArray(ArrayValue(vec![Self::basic_default_value(&key)])));
                                }
                            } else if let Ok(idx) = key_stack.pop() {
                                if idx == 1 {
                                    if let Some(TupleValue(data)) = obj_stack.pop().unwrap().get_tuple_value() {
                                        obj_stack.push(Self::DArray(ArrayValue(vec![Self::DTuple(TupleValue(data))])));
                                    }
                                }
                            } else {
                                todo!("err")
                            }
                        }
                        // skip whitespace && '['
                        ' ' | '[' => {}
                        _ => { ch_stack.push(v); }
                    }
                }

                obj_stack.pop().unwrap()
            }
            // array or list
            s if s.contains("List") || s.contains("[]") => {
                let mut char_stack: Stack<char> = Stack::new();
                let mut op_stack: Stack<char> = Stack::new();
                let mut keyword_stack: Stack<String> = Stack::new();
                let mut ret = Self::DError(ErrorValue);

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
                    Self::DError(ErrorValue)
                }
            },
            // custom
            s => {
                Self::DCustom(CustomValue(Rc::from(String::from(s)), Rc::default()))
            }
        }
    }

    //--------------------------------internal---------------------------------------------

    // getters
    define_getters!(get_tuple_value, get_tuple_ref_value, DTuple, TupleValue);
    define_getters!(get_valuetuple_value, get_valuetuple_ref_value, DValueTuple, ValueTupleValue);
    define_getters!(get_list_value, get_list_ref_value, DList, ListValue);
    // getters

    fn take_keyword(st: &mut Stack<char>) -> String {
        let mut s = String::with_capacity(10);
        while !st.is_empty() {
            if let Ok(r) = st.pop() {
                s.push(r)
            }
        }
        s.chars().rev().collect()
    }
    
    fn get_basic_type_string<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        get_basic_type_string!(
            self,
            stream,
            CellValue::DBool, 
            CellValue::DByte, 
            CellValue::DSByte, 
            CellValue::DInt, 
            CellValue::DUInt, 
            CellValue::DShort, 
            CellValue::DUShort,
            CellValue::DFloat,
            CellValue::DDouble,
            CellValue::DString,
            CellValue::DLString,
            CellValue::DShortList,
            CellValue::DCustom,
            CellValue::DTuple,
            CellValue::DValueTuple
        )
    }

    // not include list and array
    fn basic_default_value(key: &str) -> CellValue {
        match key {
            "short" => CellValue::DShort(ShortValue(0)),
            "ushort" => CellValue::DUShort(UShortValue(0)),
            "string" => CellValue::DString(StringValue(Rc::default())),
            "LString" | "Lstring" => CellValue::DLString(LStringValue(Rc::default(), i32::default())),
            "int" => CellValue::DInt(IntValue(0)),
            "uint" => CellValue::DUInt(UIntValue(0)),
            "float" => CellValue::DFloat(FloatValue(0_f32)),
            "double" => CellValue::DDouble(DoubleValue(0_f64)),
            "sbyte" => CellValue::DSByte(SByteValue(0)),
            "byte" => CellValue::DByte(ByteValue(0)),
            "bool" => CellValue::DBool(BoolValue(true)),
            "ShortList" => CellValue::DShortList(ShortListValue::default()),
            "" => CellValue::DError(ErrorValue),
            custom => CellValue::DCustom(CustomValue(Rc::from(String::from(custom)), Rc::default()))
        }
    }

    pub fn clone_from_other_with_default(v: &CellValue) -> CellValue {
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
                CellValue::DLString(LStringValue(Rc::default(), i32::default()))
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
            CellValue::DFloat(_) => {
                CellValue::DFloat(FloatValue(0_f32))
            },
            CellValue::DDouble(_) => {
                CellValue::DDouble(DoubleValue(0_f64))
            },
            CellValue::DUShort(_) => {
                CellValue::DUShort(UShortValue(0))
            },
            CellValue::DCustom(d) => {
                CellValue::DCustom(CustomValue(d.0.clone(), Rc::default()))
            },
            CellValue::DShortList(_) => {
                CellValue::DShortList(ShortListValue::default())
            },
            CellValue::DArray(arr) => {
                if arr.0.is_empty() {
                    CellValue::DError(ErrorValue)
                } else {
                    CellValue::DArray(ArrayValue(vec![CellValue::clone_from_other_with_default(&(arr.0)[0])]))
                }
            },
            CellValue::DList(lst) => {
                if lst.0.is_empty() {
                    CellValue::DError(ErrorValue)
                } else {
                    CellValue::DList(ListValue(vec![CellValue::clone_from_other_with_default(&(lst.0)[0])]))
                }
            },
            _ => { CellValue::DError(ErrorValue) }
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

fn collect_basic_value(e: &str, arr: &mut Vec<CellValue>, ls_map: &LSMap, ls_empty_map: &LSEmptyMap, pos: &(usize, usize), idx: usize) {
    let ls_data = ls_map.as_ref().borrow();

    match arr[0] {
        CellValue::DBool(_) => {
            if e == "0" {
                arr.push(CellValue::DBool( BoolValue(false) ));
            } else if e == "1" {
                arr.push(CellValue::DBool( BoolValue(true) ));
            } else {
                if let Err(_err) = e.parse::<bool>().map(|v| arr.push(CellValue::DBool( BoolValue(v) ))) {
                    //println!("{}: src val= {}", err, e);
                }
            }
        }
        CellValue::DByte(_) => {
            if let Err(_err) = e.parse::<u8>().map(|v| arr.push(CellValue::DByte( ByteValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DInt(_) => {
            if let Err(_err) = e.parse::<i32>().map(|v| arr.push(CellValue::DInt( IntValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DLString(_) => {
            let key: Rc<String> = Rc::from(String::from(e));
            if e.is_empty() {
                ls_empty_map.get(pos).map(|v| {
                    if idx < v.len() {
                        arr.push(CellValue::DLString(LStringValue(key.clone(), v[idx])));
                    } else {
                        arr.push(CellValue::DLString(LStringValue(key.clone(), -1)));
                    }
                });
            } else {
                if ls_data.contains_key(&key) { 
                    arr.push(CellValue::DLString(LStringValue(key.clone(), ls_data[&key])))
                } else {
                    arr.push(CellValue::DLString(LStringValue(key.clone(), -1)));
                }
            }
        }
        CellValue::DShort(_) => {
            if let Err(_err) = e.parse::<i16>().map(|v| arr.push(CellValue::DShort( ShortValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DSByte(_) => {
            if let Err(_err) = e.parse::<i8>().map(|v| arr.push(CellValue::DSByte( SByteValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DFloat(_) => {
            if let Err(_err) = e.parse::<f32>().map(|v| arr.push(CellValue::DFloat( FloatValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DDouble(_) => {
            if let Err(_err) = e.parse::<f64>().map(|v| arr.push(CellValue::DDouble( DoubleValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DString(_) => {
            arr.push(CellValue::DString( StringValue(Rc::new(e.to_string())) ));
        }
        CellValue::DUInt(_) => {
            if let Err(_err) = e.parse::<u32>().map(|v| arr.push(CellValue::DUInt( UIntValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        CellValue::DUShort(_) => {
            if let Err(_err) = e.parse::<u16>().map(|v| arr.push(CellValue::DUShort( UShortValue(v) ))) {
                //println!("{}: src val= {}", err, e);
            }
        }
        _ => { todo!("err") }
    }
}

fn collect_vec_value(arr: &mut Vec<CellValue>, ls_map: &LSMap, filter_val: &String, ls_empty_map: &LSEmptyMap, pos: &(usize, usize)) {
    let ls_data = ls_map.as_ref().borrow();

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
    } else if let CellValue::DTuple(ref v) = arr[0] {
        let vals = split_val(&filter_val[1..filter_val.len()-1]);
        let mut idx;
        let mut temp = Vec::<CellValue>::default();

        for s in vals.iter() {
            idx = 0;
            let vs = split_val(&s[1..s.len()-1]);
            if vs.len() != v.0.len() { continue; }
            let mut tuple = TupleValue::default();

            for item in v.0.iter() {
                match item {
                    CellValue::DBool(BoolValue(_)) => { tuple.0.push(CellValue::DBool(BoolValue(vs[idx].parse::<bool>().unwrap()))) }
                    CellValue::DByte(ByteValue(_)) => { tuple.0.push(CellValue::DByte(ByteValue(vs[idx].parse::<u8>().unwrap()))) }
                    CellValue::DSByte(SByteValue(_)) => { tuple.0.push(CellValue::DSByte(SByteValue(vs[idx].parse::<i8>().unwrap()))) }
                    CellValue::DFloat(FloatValue(_)) => { tuple.0.push(CellValue::DFloat(FloatValue(vs[idx].parse::<f32>().unwrap()))) }
                    CellValue::DDouble(DoubleValue(_)) => { tuple.0.push(CellValue::DDouble(DoubleValue(vs[idx].parse::<f64>().unwrap()))) }
                    CellValue::DInt(IntValue(_)) => { tuple.0.push(CellValue::DInt(IntValue(vs[idx].parse::<i32>().unwrap()))) }
                    CellValue::DUInt(UIntValue(_)) => { tuple.0.push(CellValue::DUInt(UIntValue(vs[idx].parse::<u32>().unwrap()))) }
                    CellValue::DShort(ShortValue(_)) => { tuple.0.push(CellValue::DShort(ShortValue(vs[idx].parse::<i16>().unwrap()))) }
                    CellValue::DUShort(UShortValue(_)) => { tuple.0.push(CellValue::DUShort(UShortValue(vs[idx].parse::<u16>().unwrap()))) }
                    CellValue::DString(StringValue(_)) => { tuple.0.push(CellValue::DString(StringValue(Rc::from(String::from(&vs[idx]))))) }
                    CellValue::DLString(LStringValue(_, _)) => {
                        let key = Rc::from(String::from(&vs[idx]));
                        if ls_data.contains_key(&key) {
                            tuple.0.push(CellValue::DLString(LStringValue(key.clone(), ls_data[&key])));
                        } else {
                            println!("LString translate err");
                        }
                    }
                    _ => { todo!("err") }
                }
                idx += 1;
            }
            temp.push(CellValue::DTuple(tuple));
        }

        for v in temp {
            arr.push(v);
        }
    } else if let CellValue::DValueTuple(ref v) = arr[0] {
        let vals = split_val(&filter_val[1..filter_val.len()-1]);
        let mut idx;
        let mut temp = Vec::<CellValue>::default();

        for s in vals.iter() {
            idx = 0;
            let vs = split_val(&s[1..s.len()-1]);
            if vs.len() != v.0.len() { continue; }
            let mut tuple = ValueTupleValue::default();

            for item in v.0.iter() {
                match item {
                    CellValue::DBool(BoolValue(_)) => { tuple.0.push(CellValue::DBool(BoolValue(vs[idx].parse::<bool>().unwrap()))) }
                    CellValue::DByte(ByteValue(_)) => { tuple.0.push(CellValue::DByte(ByteValue(vs[idx].parse::<u8>().unwrap()))) }
                    CellValue::DSByte(SByteValue(_)) => { tuple.0.push(CellValue::DSByte(SByteValue(vs[idx].parse::<i8>().unwrap()))) }
                    CellValue::DFloat(FloatValue(_)) => { tuple.0.push(CellValue::DFloat(FloatValue(vs[idx].parse::<f32>().unwrap()))) }
                    CellValue::DDouble(DoubleValue(_)) => { tuple.0.push(CellValue::DDouble(DoubleValue(vs[idx].parse::<f64>().unwrap()))) }
                    CellValue::DInt(IntValue(_)) => { tuple.0.push(CellValue::DInt(IntValue(vs[idx].parse::<i32>().unwrap()))) }
                    CellValue::DUInt(UIntValue(_)) => { tuple.0.push(CellValue::DUInt(UIntValue(vs[idx].parse::<u32>().unwrap()))) }
                    CellValue::DShort(ShortValue(_)) => { tuple.0.push(CellValue::DShort(ShortValue(vs[idx].parse::<i16>().unwrap()))) }
                    CellValue::DUShort(UShortValue(_)) => { tuple.0.push(CellValue::DUShort(UShortValue(vs[idx].parse::<u16>().unwrap()))) }
                    CellValue::DString(StringValue(_)) => { tuple.0.push(CellValue::DString(StringValue(Rc::from(String::from(&vs[idx]))))) }
                    CellValue::DLString(LStringValue(_, _)) => {
                        let key = Rc::from(String::from(&vs[idx]));
                        if ls_data.contains_key(&key) {
                            tuple.0.push(CellValue::DLString(LStringValue(key.clone(), ls_data[&key])));
                        } else {
                            println!("LString translate err");
                        }
                    }
                    _ => { todo!("err") }
                }
                idx += 1;
            }
            temp.push(CellValue::DValueTuple(tuple));
        }

        for v in temp {
            arr.push(v);
        }
    } else {
        let elements: Vec<&str> = filter_val[1..filter_val.len()-1].split(',').collect();
        for (idx, e) in elements.iter().enumerate() {
            // if e.is_empty() { continue; }
            // match type, assert arr is not empty
            collect_basic_value(e, arr, ls_map, ls_empty_map, pos, idx);
        }
    }
}

#[allow(dead_code)]
fn collect_value(val: &str, dest: &mut CellValue, ls_map: &LSMap, ls_empty_map: &LSEmptyMap, pos: &(usize, usize)) {
    if val.is_empty() { return; }

    // filter whitespace
    let filter_val: String = val.chars().filter(|&c| !c.is_whitespace()).collect();
    let mut start_idx = 1;
    let mut temp: Vec<CellValue> = vec![];
    let ls_data = ls_map.as_ref().borrow();

    match dest {
        CellValue::DArray(arr) => {
            collect_vec_value(&mut arr.0, ls_map, &filter_val, ls_empty_map, pos);
        }
        CellValue::DList(list) => {
            match (list.0)[0] {
                CellValue::DArray(ref arr) => {
                    while start_idx < filter_val.len() {
                        let end_idx = find_block(&filter_val[start_idx..]) + start_idx;
                        if end_idx != start_idx {
                            let mut new_arr = CellValue::DArray(ArrayValue(vec![CellValue::clone_from_other_with_default(&(arr.0)[0])]));
                            collect_value(&filter_val[start_idx..end_idx], &mut new_arr, &ls_map, ls_empty_map, pos);
                            temp.push(new_arr);
                        }
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                }
                CellValue::DList(ref lst) => {
                    while start_idx < filter_val.len() {
                        let end_idx = find_block(&filter_val[start_idx..]) + start_idx;
                        if end_idx != start_idx {
                            let mut new_lst = CellValue::DList(ListValue(vec![CellValue::clone_from_other_with_default(&(lst.0)[0])]));
                            collect_value(&filter_val[start_idx..end_idx], &mut new_lst, &ls_map, ls_empty_map, pos);
                            temp.push(new_lst);
                        }
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                }
                CellValue::DShortList(_) => {
                    while start_idx < filter_val.len() {
                        let end_idx = find_block(&filter_val[start_idx..]) + start_idx;
                        if end_idx != start_idx {
                            let mut new_sl = CellValue::DShortList(ShortListValue::default());
                            collect_value(&filter_val[start_idx..end_idx], &mut new_sl, &ls_map, ls_empty_map, pos);
                            temp.push(new_sl);
                        }
                        start_idx = end_idx + 1;
                    }

                    for v in temp {
                        list.0.push(v);
                    }
                }
                _ => {
                    collect_vec_value(&mut list.0, ls_map, &filter_val, ls_empty_map, pos);
                }
            }
        }
        CellValue::DShortList(ShortListValue(arr)) => {
            collect_vec_value(&mut arr.0, ls_map, &filter_val, ls_empty_map, pos);
        }
        CellValue::DValueTuple(ValueTupleValue(arr)) => {
            let vals = split_val(&filter_val[1..filter_val.len()-1]);
            for (idx, v) in arr.iter_mut().enumerate() {
                match v {
                    CellValue::DBool(BoolValue(ref mut v)) => { *v = vals[idx].parse::<bool>().unwrap(); }
                    CellValue::DByte(ByteValue(ref mut v)) => { *v = vals[idx].parse::<u8>().unwrap(); }
                    CellValue::DSByte(SByteValue(ref mut v)) => { *v = vals[idx].parse::<i8>().unwrap(); }
                    CellValue::DFloat(FloatValue(ref mut v)) => { *v = vals[idx].parse::<f32>().unwrap(); }
                    CellValue::DDouble(DoubleValue(ref mut v)) => { *v = vals[idx].parse::<f64>().unwrap(); }
                    CellValue::DInt(IntValue(ref mut v)) => { *v = vals[idx].parse::<i32>().unwrap(); }
                    CellValue::DUInt(UIntValue(ref mut v)) => { *v = vals[idx].parse::<u32>().unwrap(); }
                    CellValue::DShort(ShortValue(ref mut v)) => { *v = vals[idx].parse::<i16>().unwrap(); }
                    CellValue::DUShort(UShortValue(ref mut v)) => { *v = vals[idx].parse::<u16>().unwrap(); }
                    CellValue::DString(StringValue(ref mut v)) => { *v = Rc::from(String::from(&vals[idx]));  }
                    CellValue::DLString(LStringValue(ref mut k, ref mut v)) => {
                        let key = Rc::from(String::from(&vals[idx]));
                        if ls_data.contains_key(&key) {
                            *k = key.clone();
                            *v = ls_data[&key];
                        } else {
                            println!("LString translate err");
                        }
                    }
                    _ => { todo!("err") }
                }
            }
        }
        _ => { todo!("err") }
    }
}

pub trait ValueInfo {
    // code
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()>;
    // used by array/list code
    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()>;
}
#[derive(Default)]
pub struct EnumValue(pub Rc<String>, pub Rc<String>, pub Rc<String>); // (enum_name, val, base_name)

#[derive(Default)]
pub struct BoolValue(pub bool);

#[derive(Default)]
pub struct LStringValue(pub Rc<String>, pub i32);

#[derive(Default)]
pub struct StringValue(pub Rc<String>);

pub struct ShortListValue(pub ArrayValue);
impl Default for ShortListValue {
    fn default() -> Self {
        ShortListValue(ArrayValue(vec![CellValue::DShort(ShortValue::default())]))
    }
}

#[derive(Default)]
pub struct TupleValue(pub Vec<CellValue>);

#[derive(Default)]
pub struct ValueTupleValue(pub Vec<CellValue>);

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
pub struct FloatValue(pub f32);

#[derive(Default)]
pub struct DoubleValue(pub f64);

#[derive(Default)]
pub struct ByteValue(pub u8);

#[derive(Default)]
pub struct SByteValue(pub i8);

#[derive(Default)]
pub struct ArrayValue(pub Vec<CellValue>);

#[derive(Default)]
pub struct ListValue(pub Vec<CellValue>);

#[derive(Default)]
pub struct NoneValue(pub Rc<String>);

#[derive(Default)]
pub struct ErrorValue;

//----------------------------------impl-------------------------------------------

impl ValueInfo for ErrorValue {
    fn value<W: Write + ?Sized>(&self, _stream: &mut W) -> Result<()> {
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, _stream: &mut W) -> Result<()> {
        Ok(())
    }
}

impl ValueInfo for NoneValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        match CellValue::get_type(&self.0) {
            CellValue::DBool(_) => { stream.write("false".as_bytes())?; }
            CellValue::DSByte(_) | CellValue::DLString(_) | CellValue::DInt(_) | CellValue::DShort(_) => { stream.write("-1".as_bytes())?; }
            CellValue::DArray(_) | CellValue::DEnum(_) | CellValue::DList(_) | 
            CellValue::DShortList(_) | CellValue::DTuple(_) | CellValue::DCustom(_) | CellValue::DString(_) => { stream.write("null".as_bytes())?; }
            CellValue::DDouble(_) | CellValue::DFloat(_) => { stream.write("0.0".as_bytes())?; }
            CellValue::DUInt(_) | CellValue::DByte(_) | CellValue::DUShort(_) => { stream.write("0".as_bytes())?; }
            _ => {}
        }
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("none".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for EnumValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write_fmt(format_args!("E{}{}.{}", self.2, self.0, self.1))?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, _stream: &mut W) -> Result<()> {
        Ok(())
    }
}

impl ValueInfo for ShortListValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("new ShortList(".as_bytes())?;
        let mut cnt = 1;
        for v in self.0.0.iter().skip(1) {
            if let CellValue::DShort(vv) = v{
                stream.write(vv.0.to_string().as_bytes())?;
                if cnt < self.0.0.len()-1 {
                    stream.write(",".as_bytes())?;
                }
            } else {
                println!("ShortList value format failed");
            }
            cnt += 1;
        }
        stream.write(")".as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("ShortList".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for BoolValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        if self.0 == false {
            stream.write("false".as_bytes())?;
        } else {
            stream.write("true".as_bytes())?;
        }
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("bool".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for LStringValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.1.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("int".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for StringValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        if self.0.is_empty() {
            stream.write("\"\"".as_bytes())?;
        } else if self.0.as_str() == "\"\"" {
            stream.write("\"\"".as_bytes())?;
        } else if self.0.as_str().contains("\"") {
            stream.write(self.0.as_bytes())?;
        } else {
            stream.write("\"".as_bytes())?;
            stream.write(self.0.as_bytes())?;
            stream.write("\"".as_bytes())?;
        }
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("string".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for ShortValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("short".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for UShortValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("ushort".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for IntValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("int".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for UIntValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("uint".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for FloatValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write_fmt(format_args!("{:E}f", self.0))?;
        //stream.write(self.0.to_string().as_bytes())?;
        //stream.write("f".as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("float".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for DoubleValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write_fmt(format_args!("{:E}d", self.0))?;
        //stream.write(self.0.to_string().as_bytes())?;
        //stream.write("d".as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("double".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for ByteValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("byte".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for SByteValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.to_string().as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("sbyte".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for CustomValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("new ".as_bytes())?;
        stream.write(self.0.to_string().as_bytes())?;
        stream.write("(".as_bytes())?;
        for v in self.1.as_str()[1..self.1.len()-1].chars() {
            match v {
                '{' => { stream.write("new []{".as_bytes())?; },
                _ => { stream.write(v.to_string().as_bytes())?; }
            }
        }
        stream.write(")".as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write(self.0.as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for ArrayValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        if self.0.is_empty() {
            stream.write("".as_bytes())?;
        } else {
            stream.write("new ".as_bytes())?;
            self.ty(stream)?;
            stream.write("{".as_bytes())?;
            let mut cnt = 1;

            for v in self.0.iter().skip(1) {
                write_value_to_stream!(
                    v,
                    stream,
                    CellValue::DBool,
                    CellValue::DByte, 
                    CellValue::DSByte, 
                    CellValue::DInt, 
                    CellValue::DUInt,
                    CellValue::DFloat,
                    CellValue::DDouble,
                    CellValue::DShort, 
                    CellValue::DUShort, 
                    CellValue::DString,
                    CellValue::DLString,
                    CellValue::DCustom,
                    CellValue::DTuple,
                    CellValue::DValueTuple
                );

                if cnt < self.0.len()-1 {
                    stream.write(",".as_bytes())?;
                }
                cnt += 1;
            }
            stream.write("}".as_bytes())?;
        }
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        (self.0)[0].get_basic_type_string(stream)?;
        stream.write("[]".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for ListValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        if self.0.is_empty() {
            stream.write("".as_bytes())?;
        } else {
            stream.write("new ".as_bytes())?;
            self.ty(stream)?;
            stream.write("{".as_bytes())?;
            let mut cnt = 1;

            for v in self.0.iter().skip(1) {
                write_value_to_stream!(
                    v,
                    stream,
                    CellValue::DBool,
                    CellValue::DByte, 
                    CellValue::DSByte, 
                    CellValue::DInt, 
                    CellValue::DUInt,
                    CellValue::DFloat,
                    CellValue::DDouble,
                    CellValue::DShort, 
                    CellValue::DUShort, 
                    CellValue::DString,
                    CellValue::DLString,
                    CellValue::DCustom,
                    CellValue::DShortList,
                    CellValue::DTuple,
                    CellValue::DValueTuple,
                    CellValue::DArray,
                    CellValue::DList
                );

                if cnt < self.0.len()-1 {
                    stream.write(",".as_bytes())?;
                }
                cnt += 1;
            }
            
            stream.write("}".as_bytes())?;
        }
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write("List<".as_bytes())?;
        let first = self.0.first().unwrap();
        match first {
            CellValue::DArray(v) => { v.ty(stream)?; }
            CellValue::DList(v) => { v.ty(stream)?; }
            // basic type
            _ => {
                first.get_basic_type_string(stream)?;
            }
        }
        stream.write(">".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for TupleValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        let mut cnt = 0;
        stream.write("new ".as_bytes())?;
        self.ty(stream)?;
        stream.write("(".as_bytes())?;
        for v in self.0.iter() {
            write_value_to_stream!(
                v,
                stream,
                CellValue::DBool,
                CellValue::DByte, 
                CellValue::DSByte, 
                CellValue::DInt, 
                CellValue::DUInt,
                CellValue::DFloat,
                CellValue::DDouble,
                CellValue::DShort, 
                CellValue::DUShort, 
                CellValue::DString,
                CellValue::DLString,
                CellValue::DCustom
            );

            if cnt < self.0.len()-1 {
                stream.write(",".as_bytes())?;
            }
            cnt += 1;
        }
        stream.write(")".as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        let mut cnt = 0;
        stream.write("Tuple<".as_bytes())?;
        for v in self.0.iter() {
            v.get_basic_type_string(stream)?;
            if cnt < self.0.len() - 1 {
                stream.write(",".as_bytes())?;
            }
            cnt += 1;
        }
        stream.write(">".as_bytes())?;
        Ok(())
    }
}

impl ValueInfo for ValueTupleValue {
    fn value<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        let mut cnt = 0;
        stream.write("new ".as_bytes())?;
        self.ty(stream)?;
        stream.write("(".as_bytes())?;
        for v in self.0.iter() {
            write_value_to_stream!(
                v,
                stream,
                CellValue::DBool,
                CellValue::DByte, 
                CellValue::DSByte, 
                CellValue::DInt, 
                CellValue::DUInt,
                CellValue::DFloat,
                CellValue::DDouble,
                CellValue::DShort, 
                CellValue::DUShort, 
                CellValue::DString,
                CellValue::DLString,
                CellValue::DCustom
            );

            if cnt < self.0.len()-1 {
                stream.write(",".as_bytes())?;
            }
            cnt += 1;
        }
        stream.write(")".as_bytes())?;
        Ok(())
    }

    fn ty<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        let mut cnt = 0;
        stream.write("ValueTuple<".as_bytes())?;
        for v in self.0.iter() {
            v.get_basic_type_string(stream)?;
            if cnt < self.0.len() - 1 {
                stream.write(",".as_bytes())?;
            }
            cnt += 1;
        }
        stream.write(">".as_bytes())?;
        Ok(())
    }
}

pub const DEF_KEYWORDS: &[(&str, u32)] = &[
    ("byte", 1),
    ("List", 2),
];

pub const DEF_SYMBOLS: &[(char, u32)] = &[
    ('!', 1),
    ('#', 2),
    (',', 3),
    (':', 4),
    (';', 5),
    ('<', 6),
    ('=', 7),
    ('>', 8),
    ('@', 9),
    ('^', 10),
    ('|', 11),
    ('[', 12),
    (']', 13),
];

#[test]
fn te() {
    let mut symbols = Box::new(Vec::<(char, u32)>::default());
    symbols.extend_from_slice(DEF_SYMBOLS);
    symbols.sort_by_key(|v| v.0);
    let mut keywords = Box::new(Vec::<(&str, u32)>::default());
    keywords.extend_from_slice(DEF_KEYWORDS);
    keywords.sort_by_key(|v| v.0);

    let code = "List<byte[3]>";
    let mut cursor = cursor::Cursor::new(code, 0, 0, None);
    let lexer: Lexer<(), ()> = lexer::Builder::whitespace()
                .append(tokenizers::Number)
                .append(tokenizers::identifier_keyword_with_sorted_array(Box::leak(keywords)))
                .append(tokenizers::symbol_with_sorted_array(Box::leak(symbols)))
                .build();

    let mut sm: StateMachine::<TypeMachine> = StateMachine::new();
    if let Ok(v) = sm.tick(lexer.tokenizing(&mut cursor, &mut ())) {
        match v.unwrap() {
            CellValue::DList(vec) => {
                if let CellValue::DArray(arr) = &vec.0[0] {
                    if let CellValue::DByte(_) = &arr.0[0] {
                        println!("parse successfully")
                    }
                }
            }
            _ => {}
        }
    }
}