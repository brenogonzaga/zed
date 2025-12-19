//! # Store Module
//!
//! Redux-like store for centralized state management.
//!
//! ## Features
//!
//! - Thread-safe with `Arc<Mutex<T>>`
//! - Subscribe/unsubscribe to state changes
//! - Batch dispatch operations
//! - Dynamic reducer replacement
//! - Read-only state access
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
//! // Subscribe and get an ID for later unsubscription
//! let subscription_id = store.subscribe(|state: &AppState| {
//!     println!("Count: {}", state.count);
//! });
//!
//! store.dispatch(Action::Increment);
//! assert_eq!(store.get_state().count, 1);
//!
//! // Unsubscribe when no longer needed
//! store.unsubscribe(subscription_id);
//! # }
//! ```

use crate::reducer::Reducer;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Type alias for subscription IDs
pub type SubscriptionId = usize;

type SharedState<S> = Arc<Mutex<S>>;
type Subscriber<State> = Box<dyn Fn(&State) + Send + Sync>;
type SubscriberMap<State> = Arc<Mutex<HashMap<SubscriptionId, Subscriber<State>>>>;

/// Redux-like store for centralized state management.
///
/// Thread-safe store with:
/// - Atomic state updates
/// - Subscriber notifications
/// - Batch dispatch support
/// - Dynamic reducer replacement
pub struct Store<State, Action> {
    state: SharedState<State>,
    reducer: Arc<Mutex<Box<dyn Reducer<State, Action> + Send + Sync>>>,
    subscribers: SubscriberMap<State>,
    next_subscriber_id: AtomicUsize,
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
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            next_subscriber_id: AtomicUsize::new(0),
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
        // Hold state lock for the entire read-modify-write cycle to ensure atomicity
        let new_state = {
            let mut state = self.state.lock().unwrap();
            let reducer = self.reducer.lock().unwrap();
            let new_state = reducer.reduce(&state, &action);
            *state = new_state.clone();
            new_state
        };

        // Notify subscribers (separate lock to reduce contention)
        self.notify_subscribers(&new_state);
    }

    /// Dispatches multiple actions in a batch.
    ///
    /// This is more efficient than dispatching actions individually because
    /// subscribers are only notified once after all actions have been applied.
    ///
    /// # Arguments
    ///
    /// * `actions` - A vector of actions to dispatch
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// // All three increments, but subscribers notified only once
    /// store.dispatch_batch(vec![Action::Increment, Action::Increment, Action::Increment]);
    /// assert_eq!(store.get_state().count, 3);
    /// ```
    pub fn dispatch_batch(&self, actions: Vec<Action>) {
        if actions.is_empty() {
            return;
        }

        let new_state = {
            let mut state = self.state.lock().unwrap();
            let reducer = self.reducer.lock().unwrap();

            for action in actions {
                let temp_state = reducer.reduce(&state, &action);
                *state = temp_state;
            }

            state.clone()
        };

        // Notify subscribers once after all actions
        self.notify_subscribers(&new_state);
    }

    /// Subscribes to state changes.
    ///
    /// The provided function will be called whenever the state is updated
    /// through a dispatch action. Returns a subscription ID that can be used
    /// to unsubscribe later.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that will be called with the new state
    ///
    /// # Returns
    ///
    /// A `SubscriptionId` that can be used with `unsubscribe()` to cancel the subscription.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// let id = store.subscribe(|state: &State| {
    ///     println!("Count is now: {}", state.count);
    /// });
    ///
    /// // Later, when you no longer need the subscription
    /// store.unsubscribe(id);
    /// ```
    pub fn subscribe<F>(&self, f: F) -> SubscriptionId
    where
        F: Fn(&State) + Send + Sync + 'static,
    {
        let id = self.next_subscriber_id.fetch_add(1, Ordering::SeqCst);
        self.subscribers.lock().unwrap().insert(id, Box::new(f));
        id
    }

    /// Unsubscribes a previously registered subscriber.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription ID returned by `subscribe()`
    ///
    /// # Returns
    ///
    /// `true` if the subscriber was found and removed, `false` if no subscriber
    /// with that ID exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// let id = store.subscribe(|_| {});
    ///
    /// assert!(store.unsubscribe(id));  // Returns true - subscriber removed
    /// assert!(!store.unsubscribe(id)); // Returns false - already removed
    /// ```
    pub fn unsubscribe(&self, id: SubscriptionId) -> bool {
        self.subscribers.lock().unwrap().remove(&id).is_some()
    }

    /// Gets the current state.
    ///
    /// Returns a clone of the current state. This is safe to call from
    /// multiple threads concurrently.
    ///
    /// For read-only access without cloning, consider using `with_state()`.
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

    /// Accesses the state without cloning.
    ///
    /// This is useful for read-only access to the state when you don't need
    /// to keep a copy. The provided function receives an immutable reference
    /// to the state and can return a value.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes an immutable reference to the state
    ///
    /// # Returns
    ///
    /// The return value of the provided function.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// // Read state without cloning
    /// let double_count = store.with_state(|state| state.count * 2);
    ///
    /// // Check a condition without cloning
    /// let is_positive = store.with_state(|state| state.count > 0);
    /// ```
    pub fn with_state<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&State) -> R,
    {
        let state = self.state.lock().unwrap();
        f(&state)
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
    /// // Replace with a reducer that increments by 2
    /// let new_reducer = create_reducer(|state: &State, _: &Action| State { count: state.count + 2 });
    /// store.replace_reducer(Box::new(new_reducer));
    /// ```
    pub fn replace_reducer(&self, new_reducer: Box<dyn Reducer<State, Action> + Send + Sync>) {
        let mut reducer = self.reducer.lock().unwrap();
        *reducer = new_reducer;
    }

    /// Returns the number of active subscribers.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::{Store, create_reducer};
    /// # #[derive(Clone)] struct State { count: i32 }
    /// # #[derive(Clone)] enum Action { Increment }
    /// # let store = Store::new(State { count: 0 }, Box::new(create_reducer(|state: &State, _: &Action| State { count: state.count + 1 })));
    /// assert_eq!(store.subscriber_count(), 0);
    ///
    /// let id = store.subscribe(|_| {});
    /// assert_eq!(store.subscriber_count(), 1);
    ///
    /// store.unsubscribe(id);
    /// assert_eq!(store.subscriber_count(), 0);
    /// ```
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.lock().unwrap().len()
    }

    /// Internal helper to notify all subscribers
    fn notify_subscribers(&self, new_state: &State) {
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.values() {
            subscriber(new_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_reducer;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        counter: i32,
    }

    #[derive(Clone)]
    enum TestAction {
        Increment,
        Decrement,
        SetValue(i32),
    }

    fn create_test_store() -> Store<TestState, TestAction> {
        let reducer = create_reducer(|state: &TestState, action: &TestAction| match action {
            TestAction::Increment => TestState {
                counter: state.counter + 1,
            },
            TestAction::Decrement => TestState {
                counter: state.counter - 1,
            },
            TestAction::SetValue(val) => TestState { counter: *val },
        });

        Store::new(TestState { counter: 0 }, Box::new(reducer))
    }

    #[test]
    fn test_basic_operations() {
        let store = create_test_store();

        assert_eq!(store.get_state().counter, 0);

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().counter, 1);

        store.dispatch(TestAction::Decrement);
        assert_eq!(store.get_state().counter, 0);

        store.dispatch(TestAction::SetValue(42));
        assert_eq!(store.get_state().counter, 42);
    }

    #[test]
    fn test_subscribe_and_unsubscribe() {
        let store = create_test_store();
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        assert_eq!(store.subscriber_count(), 0);

        let id = store.subscribe(move |state| {
            notifications_clone.lock().unwrap().push(state.counter);
        });

        assert_eq!(store.subscriber_count(), 1);

        store.dispatch(TestAction::Increment);
        store.dispatch(TestAction::Increment);

        thread::sleep(Duration::from_millis(10));

        {
            let notifs = notifications.lock().unwrap();
            assert_eq!(notifs.len(), 2);
            assert_eq!(notifs[0], 1);
            assert_eq!(notifs[1], 2);
        }

        // Unsubscribe
        assert!(store.unsubscribe(id));
        assert_eq!(store.subscriber_count(), 0);
        assert!(!store.unsubscribe(id)); // Should return false for non-existent ID

        // Dispatch after unsubscribe - no more notifications
        store.dispatch(TestAction::Increment);
        thread::sleep(Duration::from_millis(10));

        let notifs = notifications.lock().unwrap();
        assert_eq!(notifs.len(), 2); // Still 2, not 3
    }

    #[test]
    fn test_dispatch_batch() {
        let store = create_test_store();
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        store.subscribe(move |state| {
            notifications_clone.lock().unwrap().push(state.counter);
        });

        // Batch dispatch - should only notify once
        store.dispatch_batch(vec![
            TestAction::Increment,
            TestAction::Increment,
            TestAction::Increment,
        ]);

        thread::sleep(Duration::from_millis(10));

        let notifs = notifications.lock().unwrap();
        assert_eq!(notifs.len(), 1); // Only one notification
        assert_eq!(notifs[0], 3); // Final state after all actions
        assert_eq!(store.get_state().counter, 3);
    }

    #[test]
    fn test_with_state() {
        let store = create_test_store();
        store.dispatch(TestAction::SetValue(100));

        // Read without cloning
        let result = store.with_state(|state| state.counter * 2);
        assert_eq!(result, 200);

        // Original state unchanged
        assert_eq!(store.get_state().counter, 100);
    }

    #[test]
    fn test_concurrent_access() {
        let store = Arc::new(create_test_store());
        let mut handles = vec![];

        for _ in 0..10 {
            let store_clone = store.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    store_clone.dispatch(TestAction::Increment);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.get_state().counter, 1000);
    }

    #[test]
    fn test_replace_reducer() {
        let store = create_test_store();

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().counter, 1);

        // Replace with a reducer that increments by 10
        let new_reducer = create_reducer(|state: &TestState, action: &TestAction| match action {
            TestAction::Increment => TestState {
                counter: state.counter + 10,
            },
            _ => state.clone(),
        });

        store.replace_reducer(Box::new(new_reducer));

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().counter, 11); // 1 + 10
    }
}
