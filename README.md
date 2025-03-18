# Zed

**Zed** is a minimal, Redux-like state management library for Rust. Inspired by Redux Toolkit, Zed provides a generic and ergonomic API to create a central store, define reducers and actions, and manage state updates with minimal boilerplate. Its design focuses on immutability, unidirectional data flow, and ease of integration into any Rust application.

## Features

- **Centralized Store**: Manage your application state in one place.
- **Immutable State Updates**: Use reducers to produce a new state from actions.
- **Subscribers**: Easily subscribe to state changes.
- **Dynamic Reducer Replacement**: Swap out reducers at runtime.
- **Slice Creation Macro**: Use `create_slice!` to automatically generate action enums, initial state constants, and reducer functions—reducing boilerplate.
- **Serde Integration**: Derive `Serialize`/`Deserialize` on your state for persistence, debugging, or external communication.

## Usage

### Creating a Slice

Define a state slice with the `create_slice!` macro. For example, to manage a simple counter:

```rust
use serde::{Deserialize, Serialize};
use std::{thread::sleep, time::Duration};
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
```

### Configuring the Store

Use `configure_store` along with `create_reducer` to create a store with your slice:

```rust
use zed::*;

fn sync_work() -> Result<(), String> {
    sleep(Duration::from_secs(2));
    Ok(())
}

fn main() {
    let store = configure_store(COUNTER_INITIAL_STATE, create_reducer(counter_reducer));

    store.subscribe(|state: &CounterState| {
        println!("State updated: {:?}", state);
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
```

### API Overview

- **Store API:**

  - `Store::new(initial_state, reducer)`: Create a new store.
  - `store.dispatch(action)`: Dispatch an action to update the state.
  - `store.subscribe(listener)`: Register a listener that is called on state updates.
  - `store.get_state()`: Retrieve the current state.
  - `store.replace_reducer(new_reducer)`: Replace the store's reducer dynamically.

- **Slice Creation with `create_slice!`:**
  - Generates an actions enum, an initial state constant, and a reducer function.
  - Helps reduce boilerplate when setting up state slices.

## Contributing

Contributions are welcome! If you'd like to help improve Zed, please open an issue or submit a pull request. Suggestions, improvements, and fixes are all appreciated.

## License

This project is licensed under the MIT License.
