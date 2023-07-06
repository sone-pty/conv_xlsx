use std::{fmt, error::Error};

use super::CellValue;

pub trait StateMachineImpl {
    // input type
    type Input;
    // output type
    type Output;
    // state set
    type State;

    // initial state
    const INITIAL_STATE: Self::State;
    
    // transfer one state to another
    fn transfer(state: &Self::State, input: &Self::Input) -> Option<Self::State>;

    // output based from current state and inputs
    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output>;
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

pub struct StateMachine<T> where T: StateMachineImpl {
    state: T::State
}

impl<T> StateMachine<T> where T: StateMachineImpl {
    pub fn new() -> Self {
        Self { state: T::INITIAL_STATE }
    }

    pub fn state(&self) -> &T::State {
        &self.state
    }

    pub fn consume(&mut self, input: &T::Input) -> Result<Option<T::Output>, TransferFailedError> {
        if let Some(state) = T::transfer(&self.state, input) {
            let output = T::output(&self.state, input);
            self.state = state;
            Ok(output)
        } else {
            Err(TransferFailedError)
        }
    }
}

//--------------------------------CellType-----------------------------------------
pub struct TypeMachine {
    value: CellValue
}
enum TypeMachineState {
    Stop,
    Begin,
    ListBegin,
    ListEnd,
    ArrayBegin,
    ArrayEnd,
    TupleBegin,
    TupleEnd,
    ValueTupleBegin,
    ValueTupleEnd,
    End,
    Any
}
enum TypeMachineInput {
    List,
    Byte,
    SByte,
    Short,
    UShort,
    Int,
    Uint,
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
    Empty
}

impl StateMachineImpl for TypeMachine {
    type Input = TypeMachineInput;
    type Output = CellValue;
    type State = TypeMachineState;

    const INITIAL_STATE: Self::State = Self::State::Begin;

    fn transfer(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            // Basic-Type
            (Self::State::Stop, Self::Input::Byte | Self::Input::Double | Self::Input::Float | Self::Input::Int | 
             Self::Input::LString | Self::Input::SByte | Self::Input::Short | Self::Input::String | Self::Input::UShort |
             Self::Input::Uint | Self::Input::Custom) => { Some(Self::State::Begin) }
            //-------------List-----------------
            (Self::State::Stop | Self::State::ListBegin | Self::State::TupleBegin, Self::Input::List) => { 
                Some(Self::State::ListBegin)
            }
            (Self::State::ListBegin, )
            //-------------List-----------------
            
            // End
            (Self::State::Begin | Self::State::ListEnd | Self::State::ArrayEnd | Self::State::TupleEnd | Self::State::ValueTupleEnd, Self::Input::Empty) => { Some(Self::State::End) }
            _ => { Some(Self::State::Any) }
        }
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        
    }
}

impl Default for TypeMachine {
    fn default() -> Self {
        Self {
            value: CellValue::DNone(super::cell_value::NoneValue::default()),
        }
    }
}
//--------------------------------CellType-----------------------------------------