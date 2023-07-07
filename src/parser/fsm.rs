use std::{fmt, error::Error, collections::HashMap, sync::LazyLock};

use vnlex::{token::Token, ParseError};

use super::{CellValue, cell_value::{ByteValue, DoubleValue, FloatValue, IntValue, LStringValue, SByteValue, ShortValue, StringValue, UShortValue, CustomValue, UIntValue, ArrayValue}, stack::Stack};

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
            if !T::ALPHA_TABLE.contains_key(key) { continue; }
            let input = &T::ALPHA_TABLE[key];
            
            if let Some(v) = self.impl_obj.transfer(&self.state, input) {
                self.impl_obj.output(&self.state, &v, input);
                self.state = v;
            } else {
                return Err(TransferFailedError);
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
    stack: Stack<CellValue>
}
#[derive(PartialEq)]
pub enum TypeMachineState {
    Stop,
    Begin,
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
#[derive(PartialEq)]
pub enum TypeMachineInput {
    List,
    Byte,
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
        Self { stack: Stack::new() }
    }
}

impl StateMachineImpl for TypeMachine {
    type Input = TypeMachineInput;
    type Output = CellValue;
    type State = TypeMachineState;

    const INITIAL_STATE: Self::State = Self::State::Stop;
    const END_TOKEN: Self::Input = Self::Input::Empty;

    const ALPHA_TABLE: LazyLock<HashMap<&'static str, Self::Input>> = {
        LazyLock::new(|| {
            let mut table = HashMap::new();
            table.insert("List", Self::Input::List);
            table.insert("byte", Self::Input::Byte);
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

    fn transfer(&mut self, state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            // Basic-Type
            (Self::State::Stop, Self::Input::Byte | Self::Input::Double | Self::Input::Float | Self::Input::Int | 
             Self::Input::LString | Self::Input::SByte | Self::Input::Short | Self::Input::String | Self::Input::UShort |
             Self::Input::UInt | Self::Input::Custom) => { Some(Self::State::Begin) }
            
            // -------------List-----------------
            (Self::State::Stop | Self::State::InList | Self::State::InTuple, Self::Input::List) => { Some(Self::State::ListBegin) }
            (Self::State::ListBegin, Self::Input::LBracket) => { Some(Self::State::InList) }
            (Self::State::InList, Self::Input::RBracket) => { Some(Self::State::ListEnd) }
            // -------------List-----------------

            // -------------Array-----------------
            (Self::State::Begin, Self::Input::LMidBracket) => { Some(Self::State::ArrayBegin) }
            (Self::State::ArrayBegin, Self::Input::RMidBracket) => { Some(Self::State::ArrayEnd) }
            // -------------Array-----------------

            // End
            (Self::State::Begin | Self::State::ListEnd | Self::State::ArrayEnd | Self::State::TupleEnd | Self::State::ValueTupleEnd, Self::Input::Empty) => { Some(Self::State::End) }
            (_, Self::Input::WhiteSpace) => { Some(Self::State::Skip) }
            _ => { None }
        }
    }

    fn output(&mut self, bef: &Self::State, aft: &Self::State, input: &Self::Input) {
        match (bef, aft) {
            (Self::State::Stop, Self::State::Begin) => {
                match input {
                    Self::Input::Byte => { self.stack.push(CellValue::DByte(ByteValue::default())) }
                    Self::Input::Double => { self.stack.push(CellValue::DDouble(DoubleValue::default())) }
                    Self::Input::Float => { self.stack.push(CellValue::DFloat(FloatValue::default())) }
                    Self::Input::Int => { self.stack.push(CellValue::DInt(IntValue::default())) }
                    Self::Input::LString => { self.stack.push(CellValue::DLString(LStringValue::default())) }
                    Self::Input::SByte => { self.stack.push(CellValue::DSByte(SByteValue::default())) }
                    Self::Input::Short => { self.stack.push(CellValue::DShort(ShortValue::default())) }
                    Self::Input::String => { self.stack.push(CellValue::DString(StringValue::default())) }
                    Self::Input::UShort => { self.stack.push(CellValue::DUShort(UShortValue::default())) }
                    Self::Input::UInt => { self.stack.push(CellValue::DUInt(UIntValue::default())) }
                    Self::Input::Custom => { self.stack.push(CellValue::DCustom(CustomValue::default())) }
                    _ => {}
                }
            }
            (Self::State::ArrayBegin, Self::State::ArrayEnd) => {
                if let Ok(basic) = self.stack.pop() {
                    let mut vec = ArrayValue::default();
                    vec.0.push(basic);
                    self.stack.push(CellValue::DArray(vec));
                } else {
                    unreachable!()
                }
            }

            _ => {}
        }
    }

    fn produce(&mut self) -> Option<Self::Output> {
        if let Ok(v) = self.stack.pop() {
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