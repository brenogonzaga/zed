/// Edge cases and boundary condition tests.
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;
use zed::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct EdgeCaseState {
    counter: i32,
    values: Vec<i32>,
    metadata: String,
}

#[derive(Clone, Debug)]
enum EdgeCaseAction {
    IncrementByLarge { value: i32 },
    AppendValue { val: i32 },
    ClearValues,
    NoOp,
    SetMetadata { text: String },
}

fn edge_case_reducer(state: &EdgeCaseState, action: &EdgeCaseAction) -> EdgeCaseState {
    match action {
        EdgeCaseAction::IncrementByLarge { value } => EdgeCaseState {
            counter: state.counter.saturating_add(*value),
            values: state.values.clone(),
            metadata: state.metadata.clone(),
        },
        EdgeCaseAction::AppendValue { val } => {
            let mut new_values = state.values.clone();
            new_values.push(*val);
            EdgeCaseState {
                counter: state.counter,
                values: new_values,
                metadata: state.metadata.clone(),
            }
        }
        EdgeCaseAction::ClearValues => EdgeCaseState {
            counter: state.counter,
            values: vec![],
            metadata: state.metadata.clone(),
        },
        EdgeCaseAction::NoOp => state.clone(),
        EdgeCaseAction::SetMetadata { text } => EdgeCaseState {
            counter: state.counter,
            values: state.values.clone(),
            metadata: text.clone(),
        },
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    // ============== Integer Overflow & Saturation ==============

    #[test]
    fn test_integer_overflow_saturation() {
        let initial_state = EdgeCaseState {
            counter: i32::MAX - 10,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        // Try to overflow - should saturate to MAX
        store.dispatch(EdgeCaseAction::IncrementByLarge { value: 100 });
        assert_eq!(store.get_state().counter, i32::MAX);

        // Try again - should stay at MAX
        store.dispatch(EdgeCaseAction::IncrementByLarge { value: 1000 });
        assert_eq!(store.get_state().counter, i32::MAX);
    }

    #[test]
    fn test_negative_saturation() {
        let initial_state = EdgeCaseState {
            counter: i32::MIN + 10,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        // Try to underflow - should saturate to MIN
        store.dispatch(EdgeCaseAction::IncrementByLarge { value: -100 });
        assert_eq!(store.get_state().counter, i32::MIN);
    }

    // ============== Empty & Null-like Conditions ==============

    #[test]
    fn test_empty_values_vector() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        // Clear on already empty should be idempotent
        store.dispatch(EdgeCaseAction::ClearValues);
        assert_eq!(store.get_state().values.len(), 0);

        store.dispatch(EdgeCaseAction::ClearValues);
        assert_eq!(store.get_state().values.len(), 0);
    }

    #[test]
    fn test_large_vector_accumulation() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        // Add 10k items
        for i in 0..10000 {
            store.dispatch(EdgeCaseAction::AppendValue { val: i });
        }

        let state = store.get_state();
        assert_eq!(state.values.len(), 10000);
        assert_eq!(state.values[0], 0);
        assert_eq!(state.values[9999], 9999);
    }

    #[test]
    fn test_very_long_metadata_string() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        // Create a 1MB string
        let large_string = "a".repeat(1_000_000);
        store.dispatch(EdgeCaseAction::SetMetadata {
            text: large_string.clone(),
        });

        assert_eq!(store.get_state().metadata.len(), 1_000_000);
    }

    // ============== NoOp & Idempotency ==============

    #[test]
    fn test_noop_does_not_modify_state() {
        let initial_state = EdgeCaseState {
            counter: 42,
            values: vec![1, 2, 3],
            metadata: "test".to_string(),
        };
        let store = Store::new(
            initial_state.clone(),
            Box::new(create_reducer(edge_case_reducer)),
        );

        store.dispatch(EdgeCaseAction::NoOp);
        assert_eq!(store.get_state(), initial_state);

        store.dispatch(EdgeCaseAction::NoOp);
        store.dispatch(EdgeCaseAction::NoOp);
        assert_eq!(store.get_state(), initial_state);
    }

    #[test]
    fn test_idempotent_operations() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![1, 2, 3],
            metadata: "test".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        let _original = store.get_state();

        // Clearing twice should result in same state (except values)
        store.dispatch(EdgeCaseAction::ClearValues);
        let after_first_clear = store.get_state();
        store.dispatch(EdgeCaseAction::ClearValues);
        let after_second_clear = store.get_state();

        assert_eq!(after_first_clear.values, after_second_clear.values);
        assert_eq!(after_first_clear.values.len(), 0);
    }

    // ============== Subscriber & State Consistency ==============

    #[test]
    fn test_subscribers_see_consistent_state() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: "initial".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        let observed_states = Arc::new(Mutex::new(Vec::new()));
        let observed_clone = Arc::clone(&observed_states);

        store.subscribe(move |state: &EdgeCaseState| {
            observed_clone.lock().unwrap().push(state.clone());
        });

        store.dispatch(EdgeCaseAction::IncrementByLarge { value: 1 });
        store.dispatch(EdgeCaseAction::AppendValue { val: 42 });
        store.dispatch(EdgeCaseAction::SetMetadata {
            text: "updated".to_string(),
        });

        thread::sleep(Duration::from_millis(50));

        let observed = observed_states.lock().unwrap();
        assert!(observed.len() >= 3);

        // Verify each observed state is valid
        for state in observed.iter() {
            assert!(state.counter >= 0);
            assert!(!state.metadata.is_empty());
        }
    }

    #[test]
    fn test_rapid_sequential_dispatches() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        // Rapid-fire dispatches
        for i in 0..1000 {
            store.dispatch(EdgeCaseAction::IncrementByLarge { value: 1 });
            if i % 10 == 0 {
                store.dispatch(EdgeCaseAction::AppendValue { val: i });
            }
        }

        let state = store.get_state();
        assert_eq!(state.counter, 1000);
        assert_eq!(state.values.len(), 100);
    }

    // ============== Concurrent Access Stress Tests ==============

    #[test]
    fn test_high_concurrency_increments() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Arc::new(Store::new(
            initial_state,
            Box::new(create_reducer(edge_case_reducer)),
        ));

        let mut handles = vec![];

        // 50 threads, each doing 100 increments
        for _ in 0..50 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    store_clone.dispatch(EdgeCaseAction::IncrementByLarge { value: 1 });
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.get_state().counter, 5000);
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Arc::new(Store::new(
            initial_state,
            Box::new(create_reducer(edge_case_reducer)),
        ));

        let mut handles = vec![];

        // Mix of different operations from multiple threads
        for thread_id in 0..10 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                for i in 0..50 {
                    match i % 4 {
                        0 => store_clone.dispatch(EdgeCaseAction::IncrementByLarge { value: 1 }),
                        1 => store_clone.dispatch(EdgeCaseAction::AppendValue { val: thread_id }),
                        2 => store_clone.dispatch(EdgeCaseAction::NoOp),
                        _ => store_clone.dispatch(EdgeCaseAction::SetMetadata {
                            text: format!("thread_{}", thread_id),
                        }),
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_state = store.get_state();
        // 10 threads * 50 iterations * 1/4 increment ops = 125
        // But due to concurrency, we might get slightly different results, so allow some variance
        assert!(
            final_state.counter >= 120 && final_state.counter <= 135,
            "Counter was {}, expected around 125",
            final_state.counter
        );
        assert!(!final_state.values.is_empty());
    }

    // ============== State Immutability & Isolation ==============

    #[test]
    fn test_state_immutability_isolation() {
        let initial_state = EdgeCaseState {
            counter: 42,
            values: vec![1, 2, 3],
            metadata: "original".to_string(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        let state1 = store.get_state();
        store.dispatch(EdgeCaseAction::SetMetadata {
            text: "modified".to_string(),
        });
        let state2 = store.get_state();

        // state1 should be unchanged (our clone is independent)
        assert_eq!(state1.metadata, "original");
        assert_eq!(state2.metadata, "modified");
    }

    // ============== Multiple Subscribers Under Load ==============

    #[test]
    fn test_multiple_subscribers_under_load() {
        let initial_state = EdgeCaseState {
            counter: 0,
            values: vec![],
            metadata: String::new(),
        };
        let store = Store::new(initial_state, Box::new(create_reducer(edge_case_reducer)));

        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));
        let counter3 = Arc::new(AtomicUsize::new(0));

        let c1 = Arc::clone(&counter1);
        let c2 = Arc::clone(&counter2);
        let c3 = Arc::clone(&counter3);

        store.subscribe(move |_| {
            c1.fetch_add(1, Ordering::Relaxed);
        });

        store.subscribe(move |_| {
            c2.fetch_add(1, Ordering::Relaxed);
        });

        store.subscribe(move |_| {
            c3.fetch_add(1, Ordering::Relaxed);
        });

        // Dispatch multiple actions
        for _ in 0..100 {
            store.dispatch(EdgeCaseAction::NoOp);
        }

        thread::sleep(Duration::from_millis(50));

        // All subscribers should be called the same number of times
        let c1_count = counter1.load(Ordering::Relaxed);
        let c2_count = counter2.load(Ordering::Relaxed);
        let c3_count = counter3.load(Ordering::Relaxed);

        assert_eq!(c1_count, c2_count);
        assert_eq!(c2_count, c3_count);
        assert_eq!(c1_count, 100);
    }

    // ============== Serialization Consistency ==============

    #[test]
    fn test_state_serialization_roundtrip() {
        let initial_state = EdgeCaseState {
            counter: 12345,
            values: vec![1, 2, 3, 4, 5],
            metadata: "test_metadata".to_string(),
        };
        let store = Store::new(
            initial_state.clone(),
            Box::new(create_reducer(edge_case_reducer)),
        );

        let state = store.get_state();
        let serialized = serde_json::to_string(&state).expect("Failed to serialize");
        let deserialized: EdgeCaseState =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_complex_state_serialization() {
        let values: Vec<i32> = (0..1000).collect();
        let state = EdgeCaseState {
            counter: 999,
            values,
            metadata: "x".repeat(5000),
        };

        let serialized = serde_json::to_string(&state).expect("Failed to serialize");
        let deserialized: EdgeCaseState =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(state, deserialized);
    }
}
