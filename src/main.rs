use std::{sync::Mutex, thread::sleep, time::Duration};
use zed::prelude::*;

fn sync_work() -> Result<(), String> {
    sleep(Duration::from_secs(2));
    Ok(())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
        SetValue(i32),
        SetError(String),
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
            CounterActions::SetValue(val) => {
                state.is_loading = false;
                state.value = *val;
                state.error = None;
            },
            CounterActions::SetError(err) => {
                state.is_loading = false;
                state.error = Some(err.clone());

            },
        }
    }
}

fn main() {
    let store = configure_store(COUNTER_INITIAL_STATE, create_reducer(counter_reducer));

    store.subscribe(|state: &CounterState| {
        println!("Estado atualizado: {:?}", state);
    });

    store.dispatch(CounterActions::StartLoading);

    let result = sync_work();

    match result {
        Ok(_) => store.dispatch(CounterActions::Incremented),
        Err(err) => store.dispatch(CounterActions::SetError(err)),
    }

    loop {
        std::thread::park();
    }
}
