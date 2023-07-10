use std::{fmt, error::Error, collections::{HashMap, VecDeque}, sync::LazyLock};

use vnlex::{token::Token, ParseError};

use super::{CellValue, cell_value::{ByteValue, DoubleValue, FloatValue, IntValue, LStringValue, SByteValue, ShortValue, StringValue, UShortValue, CustomValue, UIntValue, ArrayValue, ListValue, TupleValue, BoolValue, ShortListValue, ValueTupleValue}, stack::Stack};

pub trait StateMachineImpl {
    // input type
    type Input;
    // output type
    type Output;
    // state set
    type State;

    // transfer table(str -> keyword)
    const ALPHA_TABLE: LazyLock<HashMap<&'static str, Self::Input>>;

    // initial state
    const INITIAL_STATE: Self::State;

    // token-end
    const END_TOKEN: Self::Input;

    // custom-token
    const CUSTOM_TOKEN: Self::Input;
    
    // transfer one state to another
    fn transfer(&mut self, state: &Self::State, input: &Self::Input) -> Option<Self::State>;

    // output based from current state and inputs
    fn output(&mut self, bef: &Self::State, aft: &Self::State, input: &Self::Input);

    // produce if state in END_TOKEN
    fn produce(&mut self) -> Option<Self::Output>;

    // is end
    fn is_end(state: &Self::State) -> bool;
}

#[derive(Debug)]
pub struct TransferFailedError;

impl fmt::Display for TransferFailedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cannot perform a state transition from the current state with the provided input"
        )
    }
}

impl Error for TransferFailedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub struct StateMachine<T> where T: StateMachineImpl + Default {
    state: T::State,
    impl_obj: T
}

impl<T> StateMachine<T> where T: StateMachineImpl + Default {
    pub fn new() -> Self {
        Self { state: T::INITIAL_STATE, impl_obj: T::default() }
    }

    pub fn state(&self) -> &T::State {
        &self.state
    }

    pub fn tick<'r, I, R>(&mut self, mut tokens: I) -> Result<Option<T::Output>, TransferFailedError>
        where I: Iterator<Item = Result<Token<'r, R>, ParseError<'r>>> {
        
        while let Some(token) = tokens.next() {
            if token.is_err() { return Err(TransferFailedError); }
            let key = token.unwrap().content;
            // skip x of [x]
            if let Ok(_) = key.parse::<i32>() { continue; }
            // custom type
            if !T::ALPHA_TABLE.contains_key(key) {
                if let Some(v) = self.impl_obj.transfer(&self.state, &T::CUSTOM_TOKEN) {
                    self.impl_obj.output(&self.state, &v, &T::CUSTOM_TOKEN);
                    self.state = v;
                } else {
                    return Err(TransferFailedError);
                }
            } else {
                if let Some(v) = self.impl_obj.transfer(&self.state, &T::ALPHA_TABLE[key]) {
                    self.impl_obj.output(&self.state, &v, &T::ALPHA_TABLE[key]);
                    self.state = v;
                } else {
                    return Err(TransferFailedError);
                }
            }
        }

        self.impl_obj.transfer(&self.state, &T::END_TOKEN).map(|v| self.state = v );
        if T::is_end(&self.state) {
            Ok(self.impl_obj.produce())
        } else {
            Err(TransferFailedError)
        }
    }
}

/* CellType */
pub struct TypeMachine {
    vals: VecDeque<CellValue>,
    saved: Stack<TypeMachineState>
}
#[derive(PartialEq, Clone, Copy)]
pub enum TypeMachineState {
    Stop,
    Basic,
    BasicInList,
    ListBegin,
    InList,
    ListEnd,
    ArrayBegin,
    ArrayEnd,
    TupleBegin,
    InTuple,
    TupleEnd,
    ValueTupleBegin,
    InValueTuple,
    ValueTupleEnd,
    End,
    Skip
}
#[derive(PartialEq, Clone, Copy)]
pub enum TypeMachineInput {
    List,
    Byte,
    Bool,
    SByte,
    Short,
    UShort,
    Int,
    UInt,
    LString,
    String,
    Float,
    Double,
    Tuple,
    ValueTuple,
    Semicolon,
    Comma,
    LBracket,
    RBracket,
    LMidBracket,
    RMidBracket,
    ShortList,
    Custom,
    WhiteSpace,
    Empty
}

impl Default for TypeMachine {
    fn default() -> Self {
        Self { vals: VecDeque::new(), saved: Stack::new() }
    }
}

impl StateMachineImpl for TypeMachine {
    type Input = TypeMachineInput;
    type Output = CellValue;
    type State = TypeMachineState;

    const INITIAL_STATE: Self::State = Self::State::Stop;
    const END_TOKEN: Self::Input = Self::Input::Empty;
    const CUSTOM_TOKEN: Self::Input = Self::Input::Custom;

    const ALPHA_TABLE: LazyLock<HashMap<&'static str, Self::Input>> = {
        LazyLock::new(|| {
            let mut table = HashMap::new();
            table.insert("List", Self::Input::List);
            table.insert("byte", Self::Input::Byte);
            table.insert("bool", Self::Input::Bool);
            table.insert("sbyte", Self::Input::SByte);
            table.insert("short", Self::Input::Short);
            table.insert("ushort", Self::Input::UShort);
            table.insert("int", Self::Input::Int);
            table.insert("uint", Self::Input::UInt);
            table.insert("Lstring", Self::Input::LString);
            table.insert("LString", Self::Input::LString); 
            table.insert("String", Self::Input::String); 
            table.insert("float", Self::Input::Float); 
            table.insert("double", Self::Input::Double); 
            table.insert("Tuple", Self::Input::Tuple); 
            table.insert("ValueTuple", Self::Input::ValueTuple);
            table.insert(";", Self::Input::Semicolon); 
            table.insert(",", Self::Input::Comma); 
            table.insert("<", Self::Input::LBracket); 
            table.insert(">", Self::Input::RBracket); 
            table.insert("[", Self::Input::LMidBracket); 
            table.insert("]", Self::Input::RMidBracket); 
            table.insert("ShortList", Self::Input::ShortList);
            table.insert(" ", Self::Input::WhiteSpace);
            table
        })
    };

    #[allow(unused_must_use)]
    fn transfer(&mut self, state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            // Basic
            (Self::State::Stop, Self::Input::Byte | Self::Input::Double | Self::Input::Float | Self::Input::Int | 
             Self::Input::LString | Self::Input::SByte | Self::Input::Short | Self::Input::String | Self::Input::UShort |
             Self::Input::UInt | Self::Input::Custom | Self::Input::Bool | Self::Input::ShortList) => { Some(Self::State::Basic) }
            
            // -------------List------------------
            (Self::State::Stop, Self::Input::List) => { Some(Self::State::ListBegin) }
            (Self::State::InTuple, Self::Input::List) => { self.saved.push(Self::State::InTuple); Some(Self::State::ListBegin) }
            (Self::State::InValueTuple, Self::Input::List) => { self.saved.push(Self::State::InValueTuple); Some(Self::State::ListBegin) }
            (Self::State::InList, Self::Input::List) => { Some(Self::State::ListBegin) }

            (Self::State::ListBegin, Self::Input::LBracket) => { self.saved.push(Self::State::InList); Some(Self::State::InList) }

            (Self::State::InList, Self::Input::Byte | Self::Input::Double | Self::Input::Float | Self::Input::Int | 
             Self::Input::LString | Self::Input::SByte | Self::Input::Short | Self::Input::String | Self::Input::UShort |
             Self::Input::UInt | Self::Input::Custom | Self::Input::Bool | Self::Input::ShortList) => { Some(Self::State::BasicInList) }

            (Self::State::BasicInList | Self::State::ArrayEnd | Self::State::ListEnd, Self::Input::RBracket) => {
                if let Ok(v) = self.saved.pop() {
                    match v {
                        Self::State::InTuple => { Some(Self::State::TupleEnd) }
                        Self::State::InValueTuple => { Some(Self::State::ValueTupleEnd) }
                        _ => { Some(Self::State::ListEnd) }
                    }
                } else {
                    Some(Self::State::ListEnd)
                }
            }
            // -------------List------------------

            // -------------Array-----------------
            (Self::State::InTuple, Self::Input::LMidBracket) => { self.saved.push(Self::State::InTuple); Some(Self::State::ArrayBegin) }
            (Self::State::InValueTuple, Self::Input::LMidBracket) => { self.saved.push(Self::State::InValueTuple); Some(Self::State::ArrayBegin) }
            (Self::State::BasicInList, Self::Input::LMidBracket) => { Some(Self::State::ArrayBegin) }
            (Self::State::Basic, Self::Input::LMidBracket) => { Some(Self::State::ArrayBegin) }
            (Self::State::ArrayBegin, Self::Input::RMidBracket) => { Some(Self::State::ArrayEnd) }
            (Self::State::ArrayEnd | Self::State::ListEnd, Self::Input::Comma) => { 
                if let Some(v) = self.saved.peek() {
                    match v {
                        Self::State::InTuple => { Some(Self::State::InTuple) }
                        Self::State::InValueTuple => { Some(Self::State::InValueTuple) }
                        _ => { None }
                    }
                } else { None }
            }
            // -------------Array-----------------

            // -------------Tuple-----------------
            (Self::State::Stop | Self::State::InList | Self::State::InTuple, Self::Input::Tuple) => { Some(Self::State::TupleBegin) }
            (Self::State::TupleBegin, Self::Input::LBracket) => { Some(Self::State::InTuple) }
            (Self::State::InTuple, Self::Input::Byte | Self::Input::Double | Self::Input::Float | Self::Input::Int | 
             Self::Input::LString | Self::Input::SByte | Self::Input::Short | Self::Input::String | Self::Input::UShort |
             Self::Input::UInt | Self::Input::Custom | Self::Input::Comma | Self::Input::Bool | Self::Input::ShortList) => { Some(Self::State::InTuple) }
            (Self::State::InTuple, Self::Input::RBracket) => { Some(Self::State::TupleEnd) }
            // -------------Tuple-----------------

            // -------------ValueTuple------------
            (Self::State::Stop | Self::State::InList | Self::State::InValueTuple, Self::Input::ValueTuple) => { Some(Self::State::ValueTupleBegin) }
            (Self::State::ValueTupleBegin, Self::Input::LBracket) => { Some(Self::State::InValueTuple) }
            (Self::State::InValueTuple, Self::Input::Byte | Self::Input::Double | Self::Input::Float | Self::Input::Int | 
             Self::Input::LString | Self::Input::SByte | Self::Input::Short | Self::Input::String | Self::Input::UShort |
             Self::Input::UInt | Self::Input::Custom | Self::Input::Comma | Self::Input::Bool | Self::Input::ShortList) => { Some(Self::State::InValueTuple) }
            (Self::State::InValueTuple, Self::Input::RBracket) => { Some(Self::State::ValueTupleEnd) }
            // -------------ValueTuple------------

            // End
            (Self::State::Basic | Self::State::ListEnd | Self::State::ArrayEnd | Self::State::TupleEnd | Self::State::ValueTupleEnd, Self::Input::Empty) => { Some(Self::State::End) }
            (state, Self::Input::WhiteSpace) => { Some(*state) }
            _ => { None }
        }
    }

    fn output(&mut self, bef: &Self::State, aft: &Self::State, input: &Self::Input) {
        match (bef, aft) {
            (Self::State::Stop | Self::State::InList, Self::State::BasicInList) => {
                match input {
                    Self::Input::Byte => { self.vals.push_back(CellValue::DByte(ByteValue::default())) }
                    Self::Input::Double => { self.vals.push_back(CellValue::DDouble(DoubleValue::default())) }
                    Self::Input::Float => { self.vals.push_back(CellValue::DFloat(FloatValue::default())) }
                    Self::Input::Int => { self.vals.push_back(CellValue::DInt(IntValue::default())) }
                    Self::Input::LString => { self.vals.push_back(CellValue::DLString(LStringValue::default())) }
                    Self::Input::SByte => { self.vals.push_back(CellValue::DSByte(SByteValue::default())) }
                    Self::Input::Short => { self.vals.push_back(CellValue::DShort(ShortValue::default())) }
                    Self::Input::String => { self.vals.push_back(CellValue::DString(StringValue::default())) }
                    Self::Input::UShort => { self.vals.push_back(CellValue::DUShort(UShortValue::default())) }
                    Self::Input::UInt => { self.vals.push_back(CellValue::DUInt(UIntValue::default())) }
                    Self::Input::Custom => { self.vals.push_back(CellValue::DCustom(CustomValue::default())) }
                    Self::Input::Bool => { self.vals.push_back(CellValue::DBool(BoolValue::default())) }
                    Self::Input::ShortList => { self.vals.push_back(CellValue::DShortList(ShortListValue::default())) }
                    _ => {}
                }
            }
            (Self::State::ArrayBegin, Self::State::ArrayEnd) => {
                if let Some(basic) = self.vals.pop_back() {
                    let mut vec = ArrayValue::default();
                    vec.0.push(basic);
                    self.vals.push_back(CellValue::DArray(vec));
                } else {
                    unreachable!()
                }
            }
            (Self::State::ListBegin, Self::State::InList) => {
                self.vals.push_back(CellValue::DList(ListValue::default()))
            }
            (Self::State::BasicInList | Self::State::ArrayEnd | Self::State::ListEnd, Self::State::ListEnd) => {
                if let Some(element) = self.vals.pop_back() {
                    self.vals.back_mut().map(|list| {
                        if let CellValue::DList(ListValue(vec)) = list {
                            vec.push(element);
                        }
                    });
                }
            }
            /* Tuple */
            (Self::State::TupleBegin, Self::State::InTuple) => {
                self.vals.push_back(CellValue::DTuple(TupleValue::default()))
            }
            (Self::State::InTuple, Self::State::InTuple) => {
                match input {
                    Self::Input::Byte => { self.vals.push_back(CellValue::DByte(ByteValue::default())) }
                    Self::Input::Double => { self.vals.push_back(CellValue::DDouble(DoubleValue::default())) }
                    Self::Input::Float => { self.vals.push_back(CellValue::DFloat(FloatValue::default())) }
                    Self::Input::Int => { self.vals.push_back(CellValue::DInt(IntValue::default())) }
                    Self::Input::LString => { self.vals.push_back(CellValue::DLString(LStringValue::default())) }
                    Self::Input::SByte => { self.vals.push_back(CellValue::DSByte(SByteValue::default())) }
                    Self::Input::Short => { self.vals.push_back(CellValue::DShort(ShortValue::default())) }
                    Self::Input::String => { self.vals.push_back(CellValue::DString(StringValue::default())) }
                    Self::Input::UShort => { self.vals.push_back(CellValue::DUShort(UShortValue::default())) }
                    Self::Input::UInt => { self.vals.push_back(CellValue::DUInt(UIntValue::default())) }
                    Self::Input::Custom => { self.vals.push_back(CellValue::DCustom(CustomValue::default())) }
                    Self::Input::Bool => { self.vals.push_back(CellValue::DBool(BoolValue::default())) }
                    Self::Input::ShortList => { self.vals.push_back(CellValue::DShortList(ShortListValue::default())) }
                    _ => {}
                }
            }
            (Self::State::InTuple | Self::State::ArrayEnd | Self::State::ListEnd, Self::State::TupleEnd) => {
                let mut tuple = Vec::<CellValue>::default();
                while let Some(v) = self.vals.pop_back() {
                    match v {
                        CellValue::DTuple(_) => { break; }
                        CellValue::DNone(_) | CellValue::DError(_) => {}
                        _ => {
                            tuple.insert(0, v);
                        }
                    }
                }
                self.vals.push_back(CellValue::DTuple(TupleValue(tuple)));
            }

            /* ValueTuple */
            (Self::State::ValueTupleBegin, Self::State::InValueTuple) => {
                self.vals.push_back(CellValue::DValueTuple(ValueTupleValue::default()))
            }
            (Self::State::InValueTuple, Self::State::InValueTuple) => {
                match input {
                    Self::Input::Byte => { self.vals.push_back(CellValue::DByte(ByteValue::default())) }
                    Self::Input::Double => { self.vals.push_back(CellValue::DDouble(DoubleValue::default())) }
                    Self::Input::Float => { self.vals.push_back(CellValue::DFloat(FloatValue::default())) }
                    Self::Input::Int => { self.vals.push_back(CellValue::DInt(IntValue::default())) }
                    Self::Input::LString => { self.vals.push_back(CellValue::DLString(LStringValue::default())) }
                    Self::Input::SByte => { self.vals.push_back(CellValue::DSByte(SByteValue::default())) }
                    Self::Input::Short => { self.vals.push_back(CellValue::DShort(ShortValue::default())) }
                    Self::Input::String => { self.vals.push_back(CellValue::DString(StringValue::default())) }
                    Self::Input::UShort => { self.vals.push_back(CellValue::DUShort(UShortValue::default())) }
                    Self::Input::UInt => { self.vals.push_back(CellValue::DUInt(UIntValue::default())) }
                    Self::Input::Custom => { self.vals.push_back(CellValue::DCustom(CustomValue::default())) }
                    Self::Input::Bool => { self.vals.push_back(CellValue::DBool(BoolValue::default())) }
                    Self::Input::ShortList => { self.vals.push_back(CellValue::DShortList(ShortListValue::default())) }
                    _ => {}
                }
            }
            (Self::State::InValueTuple | Self::State::ArrayEnd | Self::State::ListEnd, Self::State::ValueTupleEnd) => {
                let mut tuple = Vec::<CellValue>::default();
                while let Some(v) = self.vals.pop_back() {
                    match v {
                        CellValue::DValueTuple(_) => { break; }
                        CellValue::DNone(_) | CellValue::DError(_) => {}
                        _ => {
                            tuple.insert(0, v);
                        }
                    }
                }
                self.vals.push_back(CellValue::DValueTuple(ValueTupleValue(tuple)));
            }
            _ => {}
        }
    }

    fn produce(&mut self) -> Option<Self::Output> {
        if let Some(v) = self.vals.pop_front() {
            Some(v)
        } else {
            None
        }
    }

    fn is_end(state: &Self::State) -> bool {
        *state == Self::State::End
    }
}
/* CellType */