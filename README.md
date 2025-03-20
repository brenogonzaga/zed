# Zed

**Zed** is a minimal, Redux-like state management library for Rust—with innovative extensions for advanced use cases. In addition to a centralized store and slice-based state updates (inspired by Redux Toolkit), Zed offers:

- **Time-Reversible State Management (Timeline)**: Maintain a full history of state changes and “travel back in time” (or branch off) to previous states.
- **Distributed Granular State (State Mesh)**: Model parts of your state as nodes in a distributed graph to support collaborative applications with intelligent conflict resolution.
- **Domain Capsules**: Encapsulate state, update logic, and caching in self-contained modules for high modularity and minimal coupling.
- **Reactive Cascade Trees**: Create reactive systems where actions trigger cascades of state updates via registered callbacks.

Zed’s design emphasizes immutability, unidirectional data flow, and ease of integration into any Rust application.

## Features

- **Centralized Store (Redux-like)**: Manage your entire application state in one place.
- **Immutable State Updates**: Use reducers to derive a new state from dispatched actions.
- **Subscribers**: Easily listen for state updates.
- **Dynamic Reducer Replacement**: Swap out reducers at runtime.
- **Slice Creation Macro**: The `create_slice!` macro automatically generates an actions enum, initial state constant, and reducer function—cutting down on boilerplate.
- **Time-Reversible State (Timeline)**: Keep a history of all state changes and support “rewinding” or branching of state.
- **State Mesh**: Represent parts of your state as interconnected nodes, ideal for collaborative scenarios.
- **Domain Capsules**: Encapsulate a piece of state with its own logic and caching, promoting modularity.
- **Reactive Cascade Trees**: Automatically trigger cascaded state updates in response to actions.
- **Serde Integration**: Easily derive `Serialize`/`Deserialize` on your state for persistence, debugging, or external communication.

## Usage

Zed can be used in several ways. Below are examples demonstrating each of its functionalities. (Each example is also provided as a separate file in the `examples/` directory.)

---

### 1. Creating a Slice (Redux-like)

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

```

---

### 2. Configuring the Store

After creating your slice using the `create_slice!` macro, a helper function (e.g. `counter_store()`) is automatically generated based on the `counter` identifier. Use this helper function to get your configured store.

```rust
use zed::*;
use std::{thread::sleep, time::Duration};

fn sync_work() -> Result<(), String> {
    sleep(Duration::from_secs(2));
    Ok(())
}

fn main() {
    let store = counter_store();

    store.subscribe(|state: &CounterState| {
        println!("[Redux] Updated state: {:?}", state);
    });

    store.dispatch(CounterActions::StartLoading);

    let result = sync_work();
    match result {
        Ok(_) => store.dispatch(CounterActions::Incremented),
        Err(err) => store.dispatch(CounterActions::SetError { error: err }),
    }

    std::thread::park();
}
```

---

### 3. Time-Reversible State Management (Timeline)

The Timeline feature allows you to keep a complete history of state changes and “rewind” or branch from any point, much like a Git commit history.

```rust
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
            CounterActions::Incremented => state.value += 1,
            CounterActions::Decremented => state.value -= 1,
            _ => {}
        }
    }
}

fn main() {
    // The reducer accepts actions as a trait object.
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

    println!("Current state: {:?}", timeline.current_state());

    timeline.rewind(1);
    println!("After rewind: {:?}", timeline.current_state());

    let branch = timeline.branch();
    println!("Branch state: {:?}", branch.current_state());
}
```

---

### 4. Distributed Granular State (State Mesh)

State Mesh models your state as a graph of nodes. This is useful for collaborative applications where different parts of the state must be synchronized and conflicts intelligently resolved.

```rust
use zed::*;

#[derive(Clone, Debug)]
struct DocumentState {
    content: String,
}

fn main() {
    let mut node1 = state_mesh::StateNode::new("node1".to_string(), DocumentState { content: "Hello".into() });
    let node2 = state_mesh::StateNode::new("node2".to_string(), DocumentState { content: "World".into() });

    node1.connect(node2);

    // Define a conflict resolver that concatenates the document content.
    node1.set_conflict_resolver(|local, remote| {
        local.content = format!("{} {}", local.content, remote.content);
    });

    node1.resolve_conflict(DocumentState { content: "from Mesh".into() });
    println!("Document state: {:?}", node1.state);
}
```

---

### 5. Domain Capsules

Capsules encapsulate state, its update logic, and an optional caching mechanism into a self-contained module, promoting modularity and separation of concerns.

```rust
use zed::*;

#[derive(Clone, Debug)]
struct CapsuleCounterState {
    count: i32,
}

#[derive(Clone, Debug)]
enum CapsuleCounterAction {
    Increment,
    Decrement,
}

fn capsule_logic(state: &mut CapsuleCounterState, action: CapsuleCounterAction) {
    match action {
        CapsuleCounterAction::Increment => state.count += 1,
        CapsuleCounterAction::Decrement => state.count -= 1,
    }
}

fn main() {
    let mut capsule = capsule::Capsule::new(CapsuleCounterState { count: 0 })
        .with_logic(capsule_logic)
        .with_cache(simple_cache::SimpleCache::new());

    capsule.dispatch(CapsuleCounterAction::Increment);
    capsule.dispatch(CapsuleCounterAction::Increment);
    capsule.dispatch(CapsuleCounterAction::Decrement);

    println!("Capsule state: {:?}", capsule.get_state());
}
```

---

### 6. Reactive Cascade Tree

The Reactive Cascade Tree feature lets you register reactive callbacks that automatically trigger a cascade of state updates when specific actions occur.

```rust
use zed::*;

#[derive(Clone, Debug)]
struct ReactiveCounter {
    value: i32,
}

fn main() {
    let mut reactive_system = reactive::ReactiveSystem::new(ReactiveCounter { value: 0 });

    // Register a reaction for the "increment" action.
    reactive_system.on("increment".to_string(), |state| {
        state.value += 1;
        println!("Reactive increment: {}", state.value);
    });

    // Additional reaction to alert when the value reaches 2.
    reactive_system.on("increment".to_string(), |state| {
        if state.value == 2 {
            println!("Alert: value reached 2!");
        }
    });

    reactive_system.trigger("increment".to_string());
    reactive_system.trigger("increment".to_string());
}
```

---

## API Overview

- **Store API:**

  - `Store::new(initial_state, reducer)`: Create a new store.
  - `store.dispatch(action)`: Dispatch an action to update the state.
  - `store.subscribe(listener)`: Register a listener that is called on state updates.
  - `store.get_state()`: Retrieve the current state.
  - `store.replace_reducer(new_reducer)`: Dynamically replace the store's reducer.

- **Timeline API:**

  - `StateManager::new(initial_state, reducer)`: Create a state manager with history.
  - `state_manager.dispatch(action)`: Dispatch an action and record the change.
  - `state_manager.rewind(steps)`: Rewind the state by a given number of steps.
  - `state_manager.branch()`: Branch off the current state history.
  - `state_manager.current_state()`: Get the current state.

- **State Mesh API:**

  - `StateNode::new(id, initial_state)`: Create a new state node.
  - `node.connect(other)`: Connect another state node.
  - `node.set_conflict_resolver(resolver)`: Define a custom conflict resolver.
  - `node.resolve_conflict(remote_state)`: Resolve a conflict with a remote state.

- **Domain Capsules API:**

  - `Capsule::new(initial_state)`: Create a new capsule.
  - `capsule.with_logic(logic)`: Attach domain-specific update logic.
  - `capsule.with_cache(cache)`: Attach a caching mechanism.
  - `capsule.dispatch(action)`: Dispatch an action within the capsule.
  - `capsule.get_state()`: Retrieve the capsule’s current state.

- **Reactive Cascade Tree API:**
  - `ReactiveSystem::new(initial_state)`: Create a new reactive system.
  - `reactive_system.on(action_type, callback)`: Register a callback for a specific action type.
  - `reactive_system.trigger(action_type)`: Trigger all callbacks associated with an action type.
  - `reactive_system.current_state()`: Retrieve the current state.

## Contributing

Contributions are welcome! If you'd like to help improve Zed, please open an issue or submit a pull request. Suggestions, improvements, and fixes are all appreciated.

## License

This project is licensed under the MIT License.
