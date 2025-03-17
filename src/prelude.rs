use crate::reducer::Reducer;
use std::marker::PhantomData;

pub struct ClosureReducer<State, Action, F>
where
    F: Fn(&State, &Action) -> State,
{
    pub f: F,
    _phantom: PhantomData<(State, Action)>,
}

impl<State, Action, F> Reducer<State, Action> for ClosureReducer<State, Action, F>
where
    F: Fn(&State, &Action) -> State,
{
    fn reduce(&self, state: &State, action: &Action) -> State {
        (self.f)(state, action)
    }
}

pub fn create_reducer<State, Action, F>(f: F) -> ClosureReducer<State, Action, F>
where
    F: Fn(&State, &Action) -> State,
{
    ClosureReducer {
        f,
        _phantom: PhantomData,
    }
}

pub use crate::configure_store::*;
pub use crate::create_slice;
pub use crate::reducer;
pub use crate::store::*;
