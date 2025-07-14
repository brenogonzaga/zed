use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zed::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct TestState {
    count: i32,
    name: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum TestAction {
    Increment,
    Decrement,
    SetName(String),
    Reset,
}

fn test_reducer(state: &TestState, action: &TestAction) -> TestState {
    match action {
        TestAction::Increment => TestState {
            count: state.count + 1,
            name: state.name.clone(),
        },
        TestAction::Decrement => TestState {
            count: state.count - 1,
            name: state.name.clone(),
        },
        TestAction::SetName(name) => TestState {
            count: state.count,
            name: name.clone(),
        },
        TestAction::Reset => TestState {
            count: 0,
            name: "reset".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let initial_state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let store = Store::new(
            initial_state.clone(),
            Box::new(create_reducer(test_reducer)),
        );

        assert_eq!(store.get_state(), initial_state);
    }

    #[test]
    fn test_store_dispatch() {
        let initial_state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(test_reducer)));

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().count, 1);

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().count, 2);

        store.dispatch(TestAction::Decrement);
        assert_eq!(store.get_state().count, 1);
    }

    #[test]
    fn test_store_subscription() {
        let initial_state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(test_reducer)));

        let received_states = Arc::new(Mutex::new(Vec::new()));
        let received_states_clone = Arc::clone(&received_states);

        store.subscribe(move |state: &TestState| {
            received_states_clone.lock().unwrap().push(state.clone());
        });

        store.dispatch(TestAction::Increment);
        store.dispatch(TestAction::SetName("test".to_string()));

        // Give some time for subscribers to be called
        thread::sleep(Duration::from_millis(10));

        let states = received_states.lock().unwrap();
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].count, 1);
        assert_eq!(states[1].name, "test");
    }

    #[test]
    fn test_store_reducer_replacement() {
        let initial_state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(test_reducer)));

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().count, 1);

        // Replace reducer with one that doubles the increment
        let double_increment_reducer = |state: &TestState, action: &TestAction| match action {
            TestAction::Increment => TestState {
                count: state.count + 2,
                name: state.name.clone(),
            },
            _ => test_reducer(state, action),
        };

        store.replace_reducer(Box::new(create_reducer(double_increment_reducer)));
        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().count, 3); // 1 + 2
    }

    #[test]
    fn test_concurrent_access() {
        let initial_state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let store = Arc::new(Store::new(
            initial_state,
            Box::new(create_reducer(test_reducer)),
        ));

        let mut handles = vec![];

        // Spawn multiple threads that increment the counter
        for _ in 0..10 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    store_clone.dispatch(TestAction::Increment);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.get_state().count, 100);
    }

    #[test]
    fn test_multiple_subscribers() {
        let initial_state = TestState {
            count: 0,
            name: "initial".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(test_reducer)));

        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let counter1_clone = Arc::clone(&counter1);
        let counter2_clone = Arc::clone(&counter2);

        store.subscribe(move |_| {
            *counter1_clone.lock().unwrap() += 1;
        });

        store.subscribe(move |_| {
            *counter2_clone.lock().unwrap() += 1;
        });

        store.dispatch(TestAction::Increment);
        store.dispatch(TestAction::Decrement);

        thread::sleep(Duration::from_millis(10));

        assert_eq!(*counter1.lock().unwrap(), 2);
        assert_eq!(*counter2.lock().unwrap(), 2);
    }
}
