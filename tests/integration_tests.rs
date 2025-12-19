//! Integration tests for combining multiple Zed features together
//! This tests realistic scenarios where multiple components work together

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use zed::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct SharedAppState {
    user_count: i32,
    messages: Vec<String>,
    is_connected: bool,
    last_update: Option<String>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum AppAction {
    UserJoined(String),
    UserLeft(String),
    MessageSent(String),
    ConnectionLost,
    ConnectionRestored,
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_store_with_capsule_integration() {
        // Test that Store and Capsule can work together
        let store = configure_store(
            SharedAppState {
                user_count: 0,
                messages: vec![],
                is_connected: true,
                last_update: None,
            },
            create_reducer(|state: &SharedAppState, action: &AppAction| {
                let mut new_state = state.clone();
                match action {
                    AppAction::UserJoined(name) => {
                        new_state.user_count += 1;
                        new_state.messages.push(format!("{name} joined"));
                        new_state.last_update = Some(format!("user_joined:{name}"));
                    }
                    AppAction::UserLeft(name) => {
                        new_state.user_count -= 1;
                        new_state.messages.push(format!("{name} left"));
                        new_state.last_update = Some(format!("user_left:{name}"));
                    }
                    AppAction::MessageSent(msg) => {
                        new_state.messages.push(msg.clone());
                        new_state.last_update = Some(format!("message:{msg}"));
                    }
                    AppAction::ConnectionLost => {
                        new_state.is_connected = false;
                        new_state.last_update = Some("connection_lost".to_string());
                    }
                    AppAction::ConnectionRestored => {
                        new_state.is_connected = true;
                        new_state.last_update = Some("connection_restored".to_string());
                    }
                }
                new_state
            }),
        );

        let mut capsule = Capsule::new("session_data".to_string()).with_cache(SimpleCache::new());

        store.dispatch(AppAction::UserJoined("Alice".to_string()));
        capsule.dispatch("update_session".to_string());

        let state = store.get_state();
        assert_eq!(state.user_count, 1);
        assert_eq!(state.messages.len(), 1);
        assert!(state.messages[0].contains("Alice joined"));
    }

    #[test]
    fn test_timeline_with_state_mesh_integration() {
        // Test Timeline and StateNode working together
        let mut timeline = StateManager::new(
            SharedAppState {
                user_count: 0,
                messages: vec![],
                is_connected: true,
                last_update: None,
            },
            |state: &SharedAppState, action: &dyn std::any::Any| {
                if let Some(app_action) = action.downcast_ref::<AppAction>() {
                    let mut new_state = state.clone();
                    match app_action {
                        AppAction::UserJoined(_) => {
                            new_state.user_count += 1;
                        }
                        AppAction::UserLeft(_) => {
                            new_state.user_count -= 1;
                        }
                        _ => {}
                    }
                    new_state
                } else {
                    state.clone()
                }
            },
        );

        let mut node = StateNode::new("main".to_string(), timeline.current_state().clone());

        timeline.dispatch(AppAction::UserJoined("Bob".to_string()));
        node.resolve_conflict(timeline.current_state().clone());

        assert_eq!(timeline.current_state().user_count, 1);
        assert_eq!(node.state.user_count, 1);

        // Test rewind functionality
        timeline.rewind(1);
        assert_eq!(timeline.current_state().user_count, 0);
    }

    #[test]
    fn test_concurrent_store_access() {
        // Test thread safety of Store
        let store = Arc::new(configure_store(
            SharedAppState {
                user_count: 0,
                messages: vec![],
                is_connected: true,
                last_update: None,
            },
            create_reducer(|state: &SharedAppState, action: &AppAction| {
                let mut new_state = state.clone();
                match action {
                    AppAction::UserJoined(_) => {
                        new_state.user_count += 1;
                    }
                    AppAction::UserLeft(_) => {
                        new_state.user_count -= 1;
                    }
                    _ => {}
                }
                new_state
            }),
        ));

        let num_threads = 10;
        let mut handles = vec![];

        for i in 0..num_threads {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                store_clone.dispatch(AppAction::UserJoined(format!("User{i}")));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_state = store.get_state();
        assert_eq!(final_state.user_count, num_threads);
    }

    #[test]
    fn test_reactive_system_with_store() {
        // Test ReactiveSystem integration with Store patterns
        let mut reactive = ReactiveSystem::new(SharedAppState {
            user_count: 0,
            messages: vec![],
            is_connected: true,
            last_update: None,
        });

        let message_count = Arc::new(Mutex::new(0));
        let message_count_clone = Arc::clone(&message_count);

        reactive.on(
            "message_sent".to_string(),
            move |state: &mut SharedAppState| {
                state.messages.push("New message".to_string());
                *message_count_clone.lock().unwrap() += 1;
            },
        );

        reactive.on("user_joined".to_string(), |state: &mut SharedAppState| {
            state.user_count += 1;
        });

        reactive.trigger("message_sent".to_string());
        reactive.trigger("user_joined".to_string());

        assert_eq!(reactive.current_state().user_count, 1);
        assert_eq!(reactive.current_state().messages.len(), 1);
        assert_eq!(*message_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_serialization_deserialization() {
        // Test that states can be serialized and deserialized correctly
        let original_state = SharedAppState {
            user_count: 42,
            messages: vec!["Hello".to_string(), "World".to_string()],
            is_connected: false,
            last_update: Some("test_update".to_string()),
        };

        let serialized = serde_json::to_string(&original_state).unwrap();
        let deserialized: SharedAppState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original_state, deserialized);
    }
}
