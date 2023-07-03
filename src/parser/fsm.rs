pub trait StateMachine {
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