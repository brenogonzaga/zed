# Zed - Advanced State Management for Rust

[![Crates.io](https://img.shields.io/crates/v/zed.svg)](https://crates.io/crates/zed)
[![Documentation](https://docs.rs/zed/badge.svg)](https://docs.rs/zed)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Zed** is a comprehensive, Redux-inspired state management library for Rust with innovative extensions for advanced use cases. Beyond traditional Redux patterns, Zed provides cutting-edge features for modern Rust applications.

## 🚀 Features

### Core Features

- **🏪 Redux-like Store**: Centralized, predictable state management
- **📦 Slice Creation**: Automatic boilerplate generation with macros
- **🔄 Immutable Updates**: Safe state transitions through reducers
- **👂 Subscribers**: React to state changes with unsubscribe support
- **🔧 Dynamic Reducers**: Hot-swap reducers at runtime
- **📊 Thread-Safe**: Built for concurrent applications
- **📦 Batch Dispatch**: Efficient batch operations with single notification

### Advanced Features

- **⏰ Timeline**: Time-reversible state with history and branching
- **🕸️ State Mesh**: Distributed state nodes with conflict resolution
- **💊 Capsules**: Encapsulated state domains with caching
- **⚡ Reactive System**: Cascade-triggered reactive updates
- **🗄️ Persistence**: Serde integration for serialization

## 📚 Quick Start

Add Zed to your `Cargo.toml`:

```toml
[dependencies]
zed = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Redux Pattern

```rust
use serde::{Deserialize, Serialize};
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
        Increment,
        Decrement,
        SetValue { value: i32 },
        Reset,
    },
    reducer: |state: &mut CounterState, action: &CounterActions| {
        match action {
            CounterActions::Increment => {
                state.value += 1;
                state.is_loading = false;
                state.error = None;
            },
            CounterActions::Decrement => {
                state.value -= 1;
                state.is_loading = false;
                state.error = None;
            },
            CounterActions::SetValue { value } => {
                state.value = *value;
                state.is_loading = false;
                state.error = None;
            },
            CounterActions::Reset => {
                state.value = 0;
                state.is_loading = false;
                state.error = None;
            },
        }
    }
}

fn main() {
    let store = counter_store();

    // Subscribe to state changes (returns ID for unsubscription)
    let subscription_id = store.subscribe(|state: &CounterState| {
        println!("State updated: value = {}", state.value);
    });

    // Dispatch actions
    store.dispatch(CounterActions::Increment);
    store.dispatch(CounterActions::SetValue { value: 42 });
    store.dispatch(CounterActions::Reset);

    // Unsubscribe when no longer needed
    store.unsubscribe(subscription_id);
}
```

## 📖 Detailed Examples

### 1. Time Travel with Timeline

Perfect for undo/redo functionality or debugging:

```rust
use zed::StateManager;
use std::any::Any;

#[derive(Clone, Debug)]
struct AppState {
    counter: i32,
    history: Vec<String>,
}

fn app_reducer(state: &AppState, action: &dyn Any) -> AppState {
    if let Some(action) = action.downcast_ref::<&str>() {
        match *action {
            "increment" => AppState {
                counter: state.counter + 1,
                history: {
                    let mut h = state.history.clone();
                    h.push(format!("Incremented to {}", state.counter + 1));
                    h
                },
            },
            "decrement" => AppState {
                counter: state.counter - 1,
                history: {
                    let mut h = state.history.clone();
                    h.push(format!("Decremented to {}", state.counter - 1));
                    h
                },
            },
            _ => state.clone(),
        }
    } else {
        state.clone()
    }
}

fn main() {
    let initial = AppState { counter: 0, history: vec![] };
    let mut timeline = StateManager::new(initial, app_reducer);

    // Build up some history
    timeline.dispatch("increment");
    timeline.dispatch("increment");
    timeline.dispatch("decrement");

    println!("Current: {}", timeline.current_state().counter); // 1

    // Travel back in time
    timeline.rewind(2);
    println!("After rewind: {}", timeline.current_state().counter); // 0

    // Create alternative timeline
    let mut branch = timeline.branch();
    branch.dispatch("increment");
    branch.dispatch("increment");

    println!("Original: {}", timeline.current_state().counter); // 0
    println!("Branch: {}", branch.current_state().counter); // 2
}
```

### 2. Distributed State with State Mesh

Ideal for collaborative applications:

```rust
use zed::StateNode;

#[derive(Clone, Debug, PartialEq)]
struct DocumentState {
    content: String,
    version: u32,
}

fn main() {
    let doc1 = DocumentState { content: "Hello".to_string(), version: 1 };
    let doc2 = DocumentState { content: "Hi".to_string(), version: 2 };

    let mut node1 = StateNode::new("user1".to_string(), doc1);
    let mut node2 = StateNode::new("user2".to_string(), doc2);

    // Set up conflict resolution (last-write-wins based on version)
    let resolver = |current: &mut DocumentState, remote: &DocumentState| {
        if remote.version > current.version {
            *current = remote.clone();
        }
    };

    node1.set_conflict_resolver(resolver);
    node2.set_conflict_resolver(resolver);

    // Connect nodes
    node1.connect(node2.clone());

    // Simulate conflict resolution
    let updated_doc = DocumentState { content: "Hello World".to_string(), version: 3 };
    node1.resolve_conflict(updated_doc);

    // Propagate to connected nodes
    node1.propagate_update();
}
```

### 3. Reactive Cascades

Chain reactions to state changes:

```rust
use zed::ReactiveSystem;

#[derive(Clone, Debug)]
struct GameState {
    score: i32,
    level: i32,
    lives: i32,
    achievements: Vec<String>,
}

fn main() {
    let initial = GameState {
        score: 0,
        level: 1,
        lives: 3,
        achievements: vec![],
    };

    let mut system = ReactiveSystem::new(initial);

    // Score increases trigger level checks
    system.on("score_increase".to_string(), |state: &mut GameState| {
        state.score += 100;
    });

    system.on("score_increase".to_string(), |state: &mut GameState| {
        if state.score > 0 && state.score % 1000 == 0 {
            state.level += 1;
            state.achievements.push(format!("Reached level {}", state.level));
        }
    });

    // Level up triggers life bonus
    system.on("level_up".to_string(), |state: &mut GameState| {
        state.lives += 1;
        state.achievements.push("Extra life earned!".to_string());
    });

    // Trigger cascade
    for _ in 0..10 {
        system.trigger("score_increase".to_string());
    }

    println!("Final state: {:?}", system.current_state());
}
```

### 4. Encapsulated Domains with Capsules

Modular state management:

```rust
use zed::{Capsule, SimpleCache};

#[derive(Clone, Debug)]
struct UserProfile {
    name: String,
    email: String,
    preferences: Vec<String>,
}

#[derive(Clone, Debug)]
enum UserAction {
    UpdateName(String),
    UpdateEmail(String),
    AddPreference(String),
    ClearPreferences,
}

fn main() {
    let initial = UserProfile {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        preferences: vec![],
    };

    let mut user_capsule = Capsule::new(initial)
        .with_logic(|state: &mut UserProfile, action: UserAction| {
            match action {
                UserAction::UpdateName(name) => state.name = name,
                UserAction::UpdateEmail(email) => state.email = email,
                UserAction::AddPreference(pref) => state.preferences.push(pref),
                UserAction::ClearPreferences => state.preferences.clear(),
            }
        })
        .with_cache(SimpleCache::new());

    user_capsule.dispatch(UserAction::UpdateName("Jane".to_string()));
    user_capsule.dispatch(UserAction::AddPreference("dark_mode".to_string()));

    println!("User profile: {:?}", user_capsule.get_state());
}
```

## 🏗️ Architecture

Zed's architecture is built around several core concepts:

### Store

The central state container that holds your application state and manages updates through reducers.

### Slices

Self-contained units of state with their own actions and reducers, similar to Redux Toolkit slices.

### Timeline

A history-aware state manager that tracks all state changes and supports time travel.

### State Mesh

A distributed state system where state is represented as interconnected nodes.

### Capsules

Encapsulated state domains that combine state, logic, and caching.

### Reactive System

An event-driven system that allows cascading reactions to state changes.

## 🎯 Use Cases

- **Web Applications**: Complex state management with time travel debugging
- **Games**: Entity systems with reactive behaviors
- **Collaborative Tools**: Distributed state synchronization
- **Desktop Apps**: Modular state management with undo/redo
- **Real-time Systems**: Event-driven state cascades

## 🔧 API Reference

### Store API

```rust
impl<State, Action> Store<State, Action> {
    pub fn new(initial_state: State, reducer: Box<dyn Reducer<State, Action>>) -> Self
    pub fn dispatch(&self, action: Action)
    pub fn subscribe<F>(&self, f: F) where F: Fn(&State)
    pub fn get_state(&self) -> State
    pub fn replace_reducer(&self, new_reducer: Box<dyn Reducer<State, Action>>)
}
```

### Timeline API

```rust
impl<T> StateManager<T> {
    pub fn new(initial_state: T, reducer: fn(&T, &dyn Any) -> T) -> Self
    pub fn dispatch<A: 'static + Clone>(&mut self, action: A)
    pub fn rewind(&mut self, steps: usize)
    pub fn branch(&self) -> Self
    pub fn current_state(&self) -> &T
}
```

### State Mesh API

```rust
impl<T> StateNode<T> {
    pub fn new(id: String, initial_state: T) -> Self
    pub fn connect(&mut self, other: StateNode<T>)
    pub fn set_conflict_resolver<F>(&mut self, resolver: F)
    pub fn resolve_conflict(&mut self, remote_state: T)
    pub fn propagate_update(&mut self)
}
```

## 🚦 Testing

Zed comes with comprehensive tests. Run them with:

```bash
cargo test
```

## 📈 Performance

Zed is designed for performance:

- Zero-copy state access where possible
- Efficient conflict resolution algorithms
- Minimal memory overhead for state history
- Thread-safe concurrent operations

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## 📄 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Inspired by Redux, Redux Toolkit, and modern Rust state management patterns.
