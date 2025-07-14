//! # Store Module
//!
//! This module provides the core Redux-like store functionality for centralized state management.
//!
//! ## Features
//!
//! - Thread-safe state management with `Arc<Mutex<T>>`
//! - Subscriber pattern for reactive updates
//! - Dynamic reducer replacement
//! - Immutable state updates through reducers
//!
//! ## Example
//!
//! ```rust
//! use zed::{Store, create_reducer};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct AppState {
//!     count: i32,
//! }
//!
//! #[derive(Clone)]
//! enum Action {
//!     Increment,
//!     Decrement,
//! }
//!
//! fn reducer(state: &AppState, action: &Action) -> AppState {
//!     match action {
//!         Action::Increment => AppState { count: state.count + 1 },
//!         Action::Decrement => AppState { count: state.count - 1 },
//!     }
//! }
//!
//! # fn main() {
//! let store = Store::new(AppState { count: 0 }, Box::new(create_reducer(reducer)));
//!
//! store.subscribe(|state: &AppState| {
//!     println!("Count: {}", state.count);
//! });
//!
//! store.dispatch(Action::Increment);
//! assert_eq!(store.get_state().count, 1);
//! # }
//! ```

use crate::reducer::Reducer;
use std::sync::{Arc, Mutex};

type SharedState<S> = Arc<Mutex<S>>;
type Subscriber<State> = Box<dyn Fn(&State) + Send + Sync>;
type Subscribers<State> = Arc<Mutex<Vec<Subscriber<State>>>>;

/// A Redux-like store that manages application state through reducers and subscriptions.
///
/// The Store provides centralized, thread-safe state management with the following guarantees:
/// - State updates are atomic and consistent
/// - Subscribers are notified of all state changes
/// - State can only be modified through dispatched actions
/// - Reducers can be replaced at runtime for hot-reloading scenarios
pub struct Store<State, Action> {
    state: SharedState<State>,
    reducer: Arc<Mutex<Box<dyn Reducer<State, Action> + Send + Sync>>>,
    subscribers: Subscribers<State>,
}

impl<State: Clone + Send + 'static, Action: Send + 'static> Store<State, Action> {
    /// Creates a new store with the given initial state and reducer.
    ///
    /// # Arguments
    ///
    /// * `initial_state` - The initial state of the store
    /// * `reducer` - A boxed reducer that handles state transitions
    ///
    /// # Example
    ///
    /// ```rust
    /// use zed::{Store, create_reducer};
    ///
    /// #[derive(Clone)]
    /// struct State { count: i32 }
    ///
    /// #[derive(Clone)]
    /// enum Action { Increment }
    ///
    /// let store = Store::new(
    ///     State { count: 0 },
    ///     Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 }))
    /// );
    /// ```
    pub fn new(
        initial_state: State,
        reducer: Box<dyn Reducer<State, Action> + Send + Sync>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
            reducer: Arc::new(Mutex::new(reducer)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Dispatches an action to update the state.
    ///
    /// This method applies the action to the current state using the reducer,
    /// updates the store's state, and notifies all subscribers.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to dispatch
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// store.dispatch(Action::Increment);
    /// ```
    pub fn dispatch(&self, action: Action) {
        let mut state = self.state.lock().unwrap();
        let current_reducer = self.reducer.lock().unwrap();
        let new_state = current_reducer.reduce(&state, &action);
        *state = new_state.clone();

        for subscriber in self.subscribers.lock().unwrap().iter() {
            subscriber(&new_state);
        }
    }

    /// Subscribes to state changes.
    ///
    /// The provided function will be called whenever the state is updated
    /// through a dispatch action.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that will be called with the new state
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// store.subscribe(|state: &State| {
    ///     println!("Count is now: {}", state.count);
    /// });
    /// ```
    pub fn subscribe<F>(&self, f: F)
    where
        F: Fn(&State) + Send + Sync + 'static,
    {
        self.subscribers.lock().unwrap().push(Box::new(f));
    }

    /// Gets the current state.
    ///
    /// Returns a clone of the current state. This is safe to call from
    /// multiple threads concurrently.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// let current_state = store.get_state();
    /// println!("Current count: {}", current_state.count);
    /// ```
    pub fn get_state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    /// Replaces the current reducer with a new one.
    ///
    /// This is useful for hot-reloading scenarios or dynamic behavior changes.
    ///
    /// # Arguments
    ///
    /// * `new_reducer` - The new reducer to use for future dispatches
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// let new_reducer = create_reducer(|state: &State, _: &Action| State { count: state.count + 2 });
    /// store.replace_reducer(Box::new(new_reducer));
    /// ```
    pub fn replace_reducer(&self, new_reducer: Box<dyn Reducer<State, Action> + Send + Sync>) {
        let mut reducer = self.reducer.lock().unwrap();
        *reducer = new_reducer;
    }
}
