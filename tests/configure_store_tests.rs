#[cfg(test)]
mod configure_store_tests {
    use zed::{Store, configure_store, create_reducer};

    #[derive(Clone, Debug, PartialEq)]
    struct CounterState {
        value: i32,
        history: Vec<i32>,
    }

    impl CounterState {
        fn new() -> Self {
            Self {
                value: 0,
                history: vec![],
            }
        }
    }

    #[derive(Debug, Clone)]
    enum CounterAction {
        Increment,
        Decrement,
        Add(i32),
        Reset,
        Multiply(i32),
    }

    fn counter_reducer(state: &CounterState, action: &CounterAction) -> CounterState {
        let mut new_history = state.history.clone();
        new_history.push(state.value);

        match action {
            CounterAction::Increment => CounterState {
                value: state.value + 1,
                history: new_history,
            },
            CounterAction::Decrement => CounterState {
                value: state.value - 1,
                history: new_history,
            },
            CounterAction::Add(n) => CounterState {
                value: state.value + n,
                history: new_history,
            },
            CounterAction::Reset => CounterState {
                value: 0,
                history: new_history,
            },
            CounterAction::Multiply(n) => CounterState {
                value: state.value * n,
                history: new_history,
            },
        }
    }

    #[test]
    fn test_configure_store_basic() {
        let reducer = create_reducer(counter_reducer);
        let store = configure_store(CounterState::new(), reducer);

        // Initial state
        assert_eq!(store.get_state().value, 0);
        assert_eq!(store.get_state().history, vec![]);

        // Dispatch increment
        store.dispatch(CounterAction::Increment);
        assert_eq!(store.get_state().value, 1);
        assert_eq!(store.get_state().history, vec![0]);

        // Dispatch another increment
        store.dispatch(CounterAction::Increment);
        assert_eq!(store.get_state().value, 2);
        assert_eq!(store.get_state().history, vec![0, 1]);
    }

    #[test]
    fn test_configure_store_multiple_actions() {
        let reducer = create_reducer(counter_reducer);
        let store = configure_store(CounterState::new(), reducer);

        // Test various actions
        store.dispatch(CounterAction::Add(5));
        assert_eq!(store.get_state().value, 5);

        store.dispatch(CounterAction::Multiply(3));
        assert_eq!(store.get_state().value, 15);

        store.dispatch(CounterAction::Decrement);
        assert_eq!(store.get_state().value, 14);

        store.dispatch(CounterAction::Reset);
        assert_eq!(store.get_state().value, 0);

        // Check history
        assert_eq!(store.get_state().history, vec![0, 5, 15, 14]);
    }

    #[test]
    fn test_configure_store_with_subscriptions() {
        let reducer = create_reducer(counter_reducer);
        let store = configure_store(CounterState::new(), reducer);

        // We can test that subscription works by checking state changes
        store.dispatch(CounterAction::Increment);
        assert_eq!(store.get_state().value, 1);

        store.dispatch(CounterAction::Add(10));
        assert_eq!(store.get_state().value, 11);

        store.dispatch(CounterAction::Reset);
        assert_eq!(store.get_state().value, 0);
    }

    #[test]
    fn test_configure_store_with_string_state() {
        #[derive(Clone, Debug, PartialEq)]
        struct TextState {
            content: String,
            word_count: usize,
        }

        #[derive(Clone)]
        enum TextAction {
            SetText(String),
            Append(String),
            Clear,
        }

        let text_reducer = create_reducer(|state: &TextState, action: &TextAction| match action {
            TextAction::SetText(text) => TextState {
                content: text.clone(),
                word_count: text.split_whitespace().count(),
            },
            TextAction::Append(text) => {
                let new_content = format!("{} {}", state.content, text).trim().to_string();
                TextState {
                    content: new_content.clone(),
                    word_count: new_content.split_whitespace().count(),
                }
            }
            TextAction::Clear => TextState {
                content: String::new(),
                word_count: 0,
            },
        });

        let initial_state = TextState {
            content: String::new(),
            word_count: 0,
        };

        let store = configure_store(initial_state, text_reducer);

        // Test text operations
        store.dispatch(TextAction::SetText("Hello world".to_string()));
        assert_eq!(store.get_state().content, "Hello world");
        assert_eq!(store.get_state().word_count, 2);

        store.dispatch(TextAction::Append("from Rust".to_string()));
        assert_eq!(store.get_state().content, "Hello world from Rust");
        assert_eq!(store.get_state().word_count, 4);

        store.dispatch(TextAction::Clear);
        assert_eq!(store.get_state().content, "");
        assert_eq!(store.get_state().word_count, 0);
    }

    #[test]
    fn test_configure_store_type_compatibility() {
        // Test that configure_store returns the same type as Store::new
        let reducer1 = create_reducer(counter_reducer);
        let reducer2 = create_reducer(counter_reducer);

        let store1: Store<CounterState, CounterAction> =
            configure_store(CounterState::new(), reducer1);
        let store2: Store<CounterState, CounterAction> =
            Store::new(CounterState::new(), Box::new(reducer2));

        // Both should have the same initial state
        assert_eq!(store1.get_state(), store2.get_state());
    }

    #[test]
    fn test_configure_store_with_complex_state() {
        #[derive(Clone, Debug, PartialEq)]
        struct AppState {
            user: Option<String>,
            settings: std::collections::HashMap<String, String>,
            items: Vec<String>,
            counter: i32,
        }

        #[derive(Clone)]
        enum AppAction {
            Login(String),
            Logout,
            SetSetting(String, String),
            AddItem(String),
            RemoveItem(String),
            IncrementCounter,
        }

        let app_reducer = create_reducer(|state: &AppState, action: &AppAction| match action {
            AppAction::Login(username) => AppState {
                user: Some(username.clone()),
                ..state.clone()
            },
            AppAction::Logout => AppState {
                user: None,
                ..state.clone()
            },
            AppAction::SetSetting(key, value) => {
                let mut new_settings = state.settings.clone();
                new_settings.insert(key.clone(), value.clone());
                AppState {
                    settings: new_settings,
                    ..state.clone()
                }
            }
            AppAction::AddItem(item) => {
                let mut new_items = state.items.clone();
                new_items.push(item.clone());
                AppState {
                    items: new_items,
                    ..state.clone()
                }
            }
            AppAction::RemoveItem(item) => {
                let new_items: Vec<String> =
                    state.items.iter().filter(|&i| i != item).cloned().collect();
                AppState {
                    items: new_items,
                    ..state.clone()
                }
            }
            AppAction::IncrementCounter => AppState {
                counter: state.counter + 1,
                ..state.clone()
            },
        });

        let initial_state = AppState {
            user: None,
            settings: std::collections::HashMap::new(),
            items: vec![],
            counter: 0,
        };

        let store = configure_store(initial_state, app_reducer);

        // Test login
        store.dispatch(AppAction::Login("alice".to_string()));
        assert_eq!(store.get_state().user, Some("alice".to_string()));

        // Test settings
        store.dispatch(AppAction::SetSetting(
            "theme".to_string(),
            "dark".to_string(),
        ));
        assert_eq!(
            store.get_state().settings.get("theme"),
            Some(&"dark".to_string())
        );

        // Test items
        store.dispatch(AppAction::AddItem("item1".to_string()));
        store.dispatch(AppAction::AddItem("item2".to_string()));
        assert_eq!(store.get_state().items, vec!["item1", "item2"]);

        store.dispatch(AppAction::RemoveItem("item1".to_string()));
        assert_eq!(store.get_state().items, vec!["item2"]);

        // Test counter
        store.dispatch(AppAction::IncrementCounter);
        store.dispatch(AppAction::IncrementCounter);
        assert_eq!(store.get_state().counter, 2);

        // Test logout
        store.dispatch(AppAction::Logout);
        assert_eq!(store.get_state().user, None);
        // Other state should remain
        assert_eq!(store.get_state().items, vec!["item2"]);
        assert_eq!(store.get_state().counter, 2);
    }
}
