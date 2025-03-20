use serde::{Deserialize, Serialize};
use std::any::Any;
use zed::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CounterState {
    pub value: i32,
    pub is_loading: bool,
    pub error: Option<String>,
}

create_slice! {
    enum_name: CounterActions,
    fn_base: counter,
    state: CounterState,
    initial_state: CounterState { value: 0, is_loading: false, error: None },
    actions: {
        Incremented,
        Decremented,
    },
    reducer: |state: &mut CounterState, action: &CounterActions| {
        match action {
            CounterActions::Incremented => {
                state.value += 1;
            },
            CounterActions::Decremented => {
                state.value -= 1;
            },
        }
    }
}

fn main() {
    println!("=== Timeline Example ===");

    let reducer = |state: &CounterState, action: &dyn Any| -> CounterState {
        if let Some(action) = action.downcast_ref::<CounterActions>() {
            counter_reducer(state, action)
        } else {
            state.clone()
        }
    };

    let mut timeline = timeline::StateManager::new(COUNTER_INITIAL_STATE, reducer);

    timeline.dispatch(CounterActions::Incremented);
    timeline.dispatch(CounterActions::Incremented);
    timeline.dispatch(CounterActions::Decremented);

    println!("[Timeline] Current state: {:?}", timeline.current_state());

    timeline.rewind(1);
    println!("[Timeline] After rewind: {:?}", timeline.current_state());

    let branch = timeline.branch();
    println!("[Timeline] Branch state: {:?}", branch.current_state());
}
