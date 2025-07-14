//! # Timeline Module
//!
//! This module provides time-reversible state management, allowing you to:
//! - Maintain a complete history of state changes
//! - Rewind to previous states
//! - Branch off from any point in history
//! - Create alternative timelines
//!
//! This is particularly useful for:
//! - Undo/redo functionality
//! - Debugging with time travel
//! - Git-like state branching
//! - A/B testing with state variations

use std::any::Any;

/// A state manager that maintains a complete history of state changes and supports time travel.
pub struct StateManager<T: Clone> {
    /// Vector containing the complete history of states
    history: Vec<T>,
    /// Current position in the history (0-indexed)
    current: usize,
    /// Reducer function that applies actions to create new states
    reducer: fn(&T, &dyn Any) -> T,
}

impl<T: Clone> Clone for StateManager<T> {
    fn clone(&self) -> Self {
        Self {
            history: self.history.clone(),
            current: self.current,
            reducer: self.reducer,
        }
    }
}

impl<T: Clone> StateManager<T> {
    /// Creates a new StateManager with an initial state and reducer function.
    pub fn new(initial_state: T, reducer: fn(&T, &dyn Any) -> T) -> Self {
        Self {
            history: vec![initial_state],
            current: 0,
            reducer,
        }
    }

    /// Dispatches an action to create a new state.
    pub fn dispatch<A: 'static + Clone>(&mut self, action: A) {
        let current_state = &self.history[self.current];
        let new_state = (self.reducer)(current_state, &action);

        // If we're not at the end, truncate future history
        if self.current + 1 < self.history.len() {
            self.history.truncate(self.current + 1);
        }

        self.history.push(new_state);
        self.current += 1;
    }

    /// Rewinds the timeline by the specified number of steps.
    pub fn rewind(&mut self, steps: usize) {
        if steps >= self.current {
            self.current = 0;
        } else {
            self.current -= steps;
        }
    }

    /// Creates a new timeline branch from the current state.
    pub fn branch(&self) -> Self {
        Self {
            history: vec![self.current_state().clone()],
            current: 0,
            reducer: self.reducer,
        }
    }

    /// Returns a reference to the current state.
    pub fn current_state(&self) -> &T {
        &self.history[self.current]
    }

    /// Returns the length of the timeline history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Returns the current position in the timeline.
    pub fn current_position(&self) -> usize {
        self.current
    }
}
