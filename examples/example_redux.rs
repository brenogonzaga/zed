use serde::{Deserialize, Serialize};
use std::{thread::sleep, time::Duration};
use zed::*;

fn sync_work() -> Result<(), String> {
    sleep(Duration::from_secs(2));
    Ok(())
}
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
        StartLoading,
        Incremented,
        Decremented,
        SetValue { value: i32 },
        SetError { error: String },
    },
    reducer: |state: &mut CounterState, action: &CounterActions| {
        match action {
            CounterActions::StartLoading => {
                state.is_loading = true;
                state.error = None;
            },
            CounterActions::Incremented => {
                state.is_loading = false;
                state.value += 1;
                state.error = None;
            },
            CounterActions::Decremented => {
                state.is_loading = false;
                state.value -= 1;
                state.error = None;
            },
            CounterActions::SetValue { value } => {
                state.is_loading = false;
                state.value = *value;
                state.error = None;
            },
            CounterActions::SetError { error } => {
                state.is_loading = false;
                state.error = Some(error.clone());
            },
        }
    }
}

fn main() {
    println!("=== Redux-like Store Example (Counter) ===");

    let store = counter_store();

    store.subscribe(|state: &CounterState| {
        println!("[Redux] Updated state: {state:?}");
    });

    store.dispatch(CounterActions::StartLoading);

    let result = sync_work();
    match result {
        Ok(_) => store.dispatch(CounterActions::Incremented),
        Err(err) => store.dispatch(CounterActions::SetError { error: err }),
    }
}
