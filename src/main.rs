use serde::{Deserialize, Serialize};
use zed::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CounterState {
    value: i32,
}

// Aqui, definimos o slice do contador:
// - `enum_name`: o nome do enum de ações (CounterActions)
// - `fn_base`: o nome base em snake_case (counter) para gerar os itens: `COUNTER_INITIAL_STATE` e `counter_reducer`
create_slice! {
    enum_name: CounterActions,
    fn_base: counter,
    state: CounterState,
    initial_state: CounterState { value: 0 },
    actions: {
        Incremented,
        Decremented,
        SetValue(i32),
    },
    reducer: |state: &mut CounterState, action: &CounterActions| {
        match action {
            CounterActions::Incremented => {
                state.value += 1;
            },
            CounterActions::Decremented => {
                state.value -= 1;
            },
            CounterActions::SetValue(value) => {
                state.value = *value;
            },
        }
    }
}

fn main() {
    let store = configure_store(COUNTER_INITIAL_STATE, create_reducer(counter_reducer));

    store.subscribe(|state: &CounterState| {
        println!("Estado atualizado: {:?}", state);
    });

    store.dispatch(CounterActions::Incremented);
    store.dispatch(CounterActions::Incremented);
    store.dispatch(CounterActions::Decremented);
    store.dispatch(CounterActions::SetValue(42));

    println!("Estado final: {:?}", store.get_state());

    store.dispatch(CounterActions::Incremented);
    println!("Estado após replace_reducer: {:?}", store.get_state());
}
