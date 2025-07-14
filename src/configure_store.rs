//! # Configure Store Module
//!
//! This module provides utilities for easily creating and configuring Redux-style stores.
//! It simplifies the store creation process by handling the boxing of reducers and
//! providing a clean API for store initialization.
//!
//! ## Example
//!
//! ```rust
//! use zed::{configure_store, create_reducer};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct AppState {
//!     counter: i32,
//! }
//!
//! #[derive(Debug)]
//! enum AppAction {
//!     Increment,
//!     Decrement,
//!     Reset,
//! }
//!
//! // Create a reducer
//! let reducer = create_reducer(|state: &AppState, action: &AppAction| {
//!     match action {
//!         AppAction::Increment => AppState { counter: state.counter + 1 },
//!         AppAction::Decrement => AppState { counter: state.counter - 1 },
//!         AppAction::Reset => AppState { counter: 0 },
//!     }
//! });
//!
//! // Configure the store easily
//! let mut store = configure_store(AppState { counter: 0 }, reducer);
//!
//! store.dispatch(AppAction::Increment);
//! assert_eq!(store.get_state().counter, 1);
//! ```

use crate::reducer::Reducer;
use crate::store::Store;

/// Configures and creates a new Redux-style store with the given initial state and reducer.
///
/// This is a convenience function that handles the complexity of boxing the reducer
/// and provides a simple way to create stores without dealing with trait objects directly.
///
/// # Arguments
///
/// * `initial_state` - The initial state of the store
/// * `reducer` - A reducer that implements the Reducer trait
///
/// # Type Parameters
///
/// * `State` - The type of the state. Must be Clone + Send + 'static
/// * `Action` - The type of actions. Must be Send + 'static  
/// * `R` - The type of the reducer. Must implement Reducer + Send + Sync + 'static
///
/// # Returns
///
/// A new Store instance configured with the provided state and reducer.
///
/// # Example
///
/// ```rust
/// use zed::{configure_store, create_reducer};
///
/// #[derive(Clone)]
/// struct Counter { value: i32 }
///
/// enum Action { Add(i32), Reset }
///
/// let reducer = create_reducer(|state: &Counter, action: &Action| {
///     match action {
///         Action::Add(n) => Counter { value: state.value + n },
///         Action::Reset => Counter { value: 0 },
///     }
/// });
///
/// let mut store = configure_store(Counter { value: 0 }, reducer);
/// store.dispatch(Action::Add(5));
/// assert_eq!(store.get_state().value, 5);
/// ```
pub fn configure_store<State, Action, R>(initial_state: State, reducer: R) -> Store<State, Action>
where
    State: Clone + Send + 'static,
    Action: Send + 'static,
    R: Reducer<State, Action> + Send + Sync + 'static,
{
    Store::new(initial_state, Box::new(reducer))
}
