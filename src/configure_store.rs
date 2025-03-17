use crate::reducer::Reducer;
use crate::store::Store;

pub fn configure_store<State, Action, R>(initial_state: State, reducer: R) -> Store<State, Action>
where
    State: Clone + Send + 'static,
    Action: Send + 'static,
    R: Reducer<State, Action> + Send + Sync + 'static,
{
    Store::new(initial_state, Box::new(reducer))
}
