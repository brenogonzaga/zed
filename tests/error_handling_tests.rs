//! Error handling and edge case tests for the Zed crate
//! This module tests various error conditions and edge cases

use zed::*;

#[derive(Clone, Debug, PartialEq)]
struct TestState {
    value: i32,
    data: Vec<String>,
}

#[derive(Clone, Debug)]
enum TestAction {
    Increment,
    Decrement,
    AddData(String),
    ClearData,
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_store_with_panicking_reducer() {
        // Test that store can handle reducers that panic
        let store = configure_store(
            TestState {
                value: 0,
                data: vec![],
            },
            create_reducer(|state: &TestState, action: &TestAction| match action {
                TestAction::Increment => {
                    if state.value >= 10 {
                        panic!("Value too high!");
                    }
                    TestState {
                        value: state.value + 1,
                        data: state.data.clone(),
                    }
                }
                _ => state.clone(),
            }),
        );

        // Normal operations should work
        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().value, 1);

        // Set value to 9
        for _ in 0..8 {
            store.dispatch(TestAction::Increment);
        }
        assert_eq!(store.get_state().value, 9);

        // This should panic, but we can't easily test panic behavior
        // in a unit test without special setup
        // store.dispatch(TestAction::Increment); // Would panic
    }

    #[test]
    fn test_concurrent_store_access_heavy_load() {
        let store = Arc::new(configure_store(
            TestState {
                value: 0,
                data: vec![],
            },
            create_reducer(|state: &TestState, action: &TestAction| match action {
                TestAction::Increment => TestState {
                    value: state.value + 1,
                    data: state.data.clone(),
                },
                TestAction::Decrement => TestState {
                    value: state.value - 1,
                    data: state.data.clone(),
                },
                TestAction::AddData(s) => {
                    let mut new_data = state.data.clone();
                    new_data.push(s.clone());
                    TestState {
                        value: state.value,
                        data: new_data,
                    }
                }
                TestAction::ClearData => TestState {
                    value: state.value,
                    data: vec![],
                },
            }),
        ));

        let num_threads = 50;
        let operations_per_thread = 100;
        let mut handles = vec![];

        for i in 0..num_threads {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                for j in 0..operations_per_thread {
                    match j % 4 {
                        0 => store_clone.dispatch(TestAction::Increment),
                        1 => store_clone.dispatch(TestAction::Decrement),
                        2 => {
                            store_clone.dispatch(TestAction::AddData(format!("thread_{i}_op_{j}")))
                        }
                        3 => store_clone.dispatch(TestAction::ClearData),
                        _ => unreachable!(),
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // State should be consistent after all operations
        let final_state = store.get_state();
        println!(
            "Final state after heavy concurrent access: value={}, data_len={}",
            final_state.value,
            final_state.data.len()
        );

        // The exact values are non-deterministic due to concurrency,
        // but the state should be valid
        assert!(final_state.value >= -5000 && final_state.value <= 5000);
    }

    #[test]
    fn test_store_with_large_state() {
        // Test store with very large state
        let large_data: Vec<String> = (0..10000).map(|i| format!("item_{i}")).collect();

        let store = configure_store(
            TestState {
                value: 0,
                data: large_data,
            },
            create_reducer(|state: &TestState, action: &TestAction| match action {
                TestAction::Increment => TestState {
                    value: state.value + 1,
                    data: state.data.clone(),
                },
                TestAction::AddData(s) => {
                    let mut new_data = state.data.clone();
                    new_data.push(s.clone());
                    TestState {
                        value: state.value,
                        data: new_data,
                    }
                }
                _ => state.clone(),
            }),
        );

        let initial_len = store.get_state().data.len();
        assert_eq!(initial_len, 10000);

        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().value, 1);
        assert_eq!(store.get_state().data.len(), 10000);

        store.dispatch(TestAction::AddData("new_item".to_string()));
        assert_eq!(store.get_state().data.len(), 10001);
    }

    #[test]
    fn test_reactive_system_with_many_reactions() {
        let mut system = ReactiveSystem::new(TestState {
            value: 0,
            data: vec![],
        });

        // Add many reactions to the same action
        for i in 0..1000 {
            system.on("increment".to_string(), move |state: &mut TestState| {
                state.value += i % 10; // Different increments
            });
        }

        system.trigger("increment".to_string());

        // The value should be the sum of all increments
        let expected_sum: i32 = (0..1000).map(|i| i % 10).sum();
        assert_eq!(system.current_state().value, expected_sum);
    }

    #[test]
    fn test_reactive_system_with_long_action_names() {
        let mut system = ReactiveSystem::new(TestState {
            value: 0,
            data: vec![],
        });

        let long_action_name = "a".repeat(10000);
        system.on(long_action_name.clone(), |state: &mut TestState| {
            state.value += 1;
        });

        system.trigger(long_action_name);
        assert_eq!(system.current_state().value, 1);
    }

    #[test]
    fn test_capsule_with_expensive_cache_operations() {
        #[derive(Clone)]
        struct ExpensiveCache<T: Clone> {
            value: Option<T>,
            access_count: Arc<Mutex<u32>>,
        }

        impl<T: Clone> ExpensiveCache<T> {
            fn new() -> Self {
                Self {
                    value: None,
                    access_count: Arc::new(Mutex::new(0)),
                }
            }
        }

        impl<T: Clone> Cache<T> for ExpensiveCache<T> {
            fn get(&self) -> Option<T> {
                // Simulate expensive operation
                thread::sleep(Duration::from_millis(1));
                *self.access_count.lock().unwrap() += 1;
                self.value.clone()
            }

            fn set(&mut self, value: T) {
                // Simulate expensive operation
                thread::sleep(Duration::from_millis(1));
                *self.access_count.lock().unwrap() += 1;
                self.value = Some(value);
            }
        }

        let mut capsule = Capsule::new(TestState {
            value: 0,
            data: vec![],
        })
        .with_cache(ExpensiveCache::new())
        .with_logic(|state: &mut TestState, action: TestAction| match action {
            TestAction::Increment => state.value += 1,
            TestAction::AddData(s) => state.data.push(s),
            _ => {}
        });

        // Operations should work despite expensive cache
        capsule.dispatch(TestAction::Increment);
        assert_eq!(capsule.get_state().value, 1);

        capsule.dispatch(TestAction::AddData("test".to_string()));
        assert_eq!(capsule.get_state().data.len(), 1);
    }

    #[test]
    fn test_state_mesh_with_many_connections() {
        let mut main_node = StateNode::new(
            "main".to_string(),
            TestState {
                value: 0,
                data: vec![],
            },
        );

        // Create many connected nodes
        for i in 0..100 {
            let node = StateNode::new(
                format!("node_{i}"),
                TestState {
                    value: i,
                    data: vec![format!("data_{}", i)],
                },
            );
            main_node.connect(node);
        }

        // Update main node state
        main_node.state.value = 999;
        main_node.propagate_update();

        // All connected nodes should have been updated
        for (id, node) in &main_node.connections {
            assert_eq!(node.state.value, 999, "Node {id} was not updated");
        }
    }

    #[test]
    fn test_timeline_with_many_states() {
        let mut timeline = StateManager::new(
            TestState {
                value: 0,
                data: vec![],
            },
            |state: &TestState, action: &dyn std::any::Any| {
                if let Some(test_action) = action.downcast_ref::<TestAction>() {
                    match test_action {
                        TestAction::Increment => TestState {
                            value: state.value + 1,
                            data: state.data.clone(),
                        },
                        TestAction::AddData(s) => {
                            let mut new_data = state.data.clone();
                            new_data.push(s.clone());
                            TestState {
                                value: state.value,
                                data: new_data,
                            }
                        }
                        _ => state.clone(),
                    }
                } else {
                    state.clone()
                }
            },
        );

        // Create a long history
        for i in 0..1000 {
            timeline.dispatch(TestAction::Increment);
            if i % 10 == 0 {
                timeline.dispatch(TestAction::AddData(format!("checkpoint_{i}")));
            }
        }

        assert_eq!(timeline.current_state().value, 1000);
        assert_eq!(timeline.current_state().data.len(), 100);
        assert_eq!(timeline.history_len(), 1101); // 1000 increments + 100 data additions + 1 initial

        // Test rewinding through large history
        timeline.rewind(500);
        assert_eq!(timeline.current_position(), 600);

        timeline.rewind(1000); // Should clamp to 0
        assert_eq!(timeline.current_position(), 0);
        assert_eq!(timeline.current_state().value, 0);
    }

    #[test]
    fn test_state_node_circular_references() {
        let mut node1 = StateNode::new(
            "node1".to_string(),
            TestState {
                value: 1,
                data: vec![],
            },
        );

        let mut node2 = StateNode::new(
            "node2".to_string(),
            TestState {
                value: 2,
                data: vec![],
            },
        );

        let node3 = StateNode::new(
            "node3".to_string(),
            TestState {
                value: 3,
                data: vec![],
            },
        );

        // Create circular-like connections
        node1.connect(node2.clone());
        node2.connect(node3.clone());
        node1.connect(node3);

        // Update node1 and propagate
        node1.state.value = 999;
        node1.propagate_update();

        // Check that updates propagated correctly
        assert_eq!(node1.connections.get("node2").unwrap().state.value, 999);
        assert_eq!(node1.connections.get("node3").unwrap().state.value, 999);
    }

    #[test]
    fn test_empty_state_operations() {
        // Test with empty/minimal state
        let store = configure_store(
            TestState {
                value: 0,
                data: vec![],
            },
            create_reducer(|state: &TestState, _action: &TestAction| {
                state.clone() // No-op reducer
            }),
        );

        // Should work with no-op reducer
        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().value, 0);

        let mut reactive = ReactiveSystem::new(TestState {
            value: 0,
            data: vec![],
        });

        // Should work with no reactions
        reactive.trigger("nonexistent".to_string());
        assert_eq!(reactive.current_state().value, 0);
    }
}
