//! # Reducer Module
//!
//! This module provides traits and utilities for creating reducers - pure functions that
//! take a current state and an action, and return a new state. This is a core concept
//! in Redux-style state management.
//!
//! ## Example
//!
//! ```rust
//! use zed::{Reducer, create_reducer};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct CounterState {
//!     value: i32,
//! }
//!
//! #[derive(Debug)]
//! enum CounterAction {
//!     Increment,
//!     Decrement,
//!     Set(i32),
//! }
//!
//! // Create a reducer using a closure
//! let counter_reducer = create_reducer(|state: &CounterState, action: &CounterAction| {
//!     match action {
//!         CounterAction::Increment => CounterState { value: state.value + 1 },
//!         CounterAction::Decrement => CounterState { value: state.value - 1 },
//!         CounterAction::Set(val) => CounterState { value: *val },
//!     }
//! });
//!
//! let state = CounterState { value: 0 };
//! let new_state = counter_reducer.reduce(&state, &CounterAction::Increment);
//! assert_eq!(new_state.value, 1);
//! ```

use std::marker::PhantomData;

/// A trait for implementing reducers that transform state based on actions.
///
/// Reducers are pure functions that take the current state and an action,
/// and return a new state. They should not have side effects and should
/// be deterministic.
pub trait Reducer<State, Action> {
    /// Applies an action to the current state and returns a new state.
    ///
    /// # Arguments
    ///
    /// * `state` - The current state
    /// * `action` - The action to apply
    ///
    /// # Returns
    ///
    /// A new state after applying the action
    fn reduce(&self, state: &State, action: &Action) -> State;
}

/// A reducer implementation that wraps a closure function.
///
/// This allows you to easily create reducers from functions or closures
/// without having to implement the Reducer trait manually.
///
/// # Example
///
/// ```rust
/// use zed::{create_reducer, Reducer};
///
/// #[derive(Clone)]
/// struct State { count: i32 }
///
/// enum Action { Increment }
///
/// let reducer = create_reducer(|state: &State, _action: &Action| State { count: state.count + 1 });
///
/// let state = State { count: 0 };
/// let new_state = reducer.reduce(&state, &Action::Increment);
/// assert_eq!(new_state.count, 1);
/// ```
pub struct ClosureReducer<State, Action, F>
where
    F: Fn(&State, &Action) -> State,
{
    /// The closure function that performs the reduction
    pub f: F,
    /// Phantom data to maintain type information
    _phantom: PhantomData<(State, Action)>,
}

impl<State, Action, F> Reducer<State, Action> for ClosureReducer<State, Action, F>
where
    F: Fn(&State, &Action) -> State,
{
    fn reduce(&self, state: &State, action: &Action) -> State {
        (self.f)(state, action)
    }
}

/// Creates a new reducer from a closure function.
///
/// This is a convenience function that wraps a closure in a ClosureReducer
/// and handles the phantom data automatically.
///
/// # Arguments
///
/// * `f` - A function that takes a state reference and action reference, returns new state
///
/// # Returns
///
/// A ClosureReducer that implements the Reducer trait
///
/// # Example
///
/// ```rust
/// use zed::{create_reducer, Reducer};
///
/// #[derive(Clone)]
/// struct AppState { counter: i32 }
///
/// enum AppAction { Increment, Decrement }
///
/// let reducer = create_reducer(|state: &AppState, action: &AppAction| {
///     match action {
///         AppAction::Increment => AppState { counter: state.counter + 1 },
///         AppAction::Decrement => AppState { counter: state.counter - 1 },
///     }
/// });
///
/// let initial_state = AppState { counter: 0 };
/// let new_state = reducer.reduce(&initial_state, &AppAction::Increment);
/// assert_eq!(new_state.counter, 1);
/// ```
pub fn create_reducer<State, Action, F>(f: F) -> ClosureReducer<State, Action, F>
where
    F: Fn(&State, &Action) -> State,
{
    ClosureReducer {
        f,
        _phantom: PhantomData,
    }
}
