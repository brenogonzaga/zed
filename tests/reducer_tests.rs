#[cfg(test)]
mod reducer_tests {
    use zed::{Reducer, create_reducer};

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        counter: i32,
        message: String,
    }

    #[derive(Debug)]
    enum TestAction {
        Increment,
        Decrement,
        SetMessage(String),
        Reset,
    }

    fn test_reducer(state: &TestState, action: &TestAction) -> TestState {
        match action {
            TestAction::Increment => TestState {
                counter: state.counter + 1,
                message: state.message.clone(),
            },
            TestAction::Decrement => TestState {
                counter: state.counter - 1,
                message: state.message.clone(),
            },
            TestAction::SetMessage(msg) => TestState {
                counter: state.counter,
                message: msg.clone(),
            },
            TestAction::Reset => TestState {
                counter: 0,
                message: "reset".to_string(),
            },
        }
    }

    #[test]
    fn test_create_reducer_basic() {
        let reducer = create_reducer(test_reducer);

        let initial_state = TestState {
            counter: 0,
            message: "initial".to_string(),
        };

        let new_state = reducer.reduce(&initial_state, &TestAction::Increment);
        assert_eq!(new_state.counter, 1);
        assert_eq!(new_state.message, "initial");
    }

    #[test]
    fn test_create_reducer() {
        let reducer = create_reducer(test_reducer);

        let initial_state = TestState {
            counter: 5,
            message: "test".to_string(),
        };

        // Test increment
        let state1 = reducer.reduce(&initial_state, &TestAction::Increment);
        assert_eq!(state1.counter, 6);
        assert_eq!(state1.message, "test");

        // Test decrement
        let state2 = reducer.reduce(&state1, &TestAction::Decrement);
        assert_eq!(state2.counter, 5);
        assert_eq!(state2.message, "test");

        // Test message change
        let state3 = reducer.reduce(&state2, &TestAction::SetMessage("new message".to_string()));
        assert_eq!(state3.counter, 5);
        assert_eq!(state3.message, "new message");

        // Test reset
        let state4 = reducer.reduce(&state3, &TestAction::Reset);
        assert_eq!(state4.counter, 0);
        assert_eq!(state4.message, "reset");
    }

    #[test]
    fn test_create_reducer_with_closure() {
        let reducer = create_reducer(|state: &TestState, action: &TestAction| {
            match action {
                TestAction::Increment => TestState {
                    counter: state.counter + 10, // Different increment for testing
                    message: state.message.clone(),
                },
                _ => state.clone(),
            }
        });

        let initial_state = TestState {
            counter: 0,
            message: "closure test".to_string(),
        };

        let new_state = reducer.reduce(&initial_state, &TestAction::Increment);
        assert_eq!(new_state.counter, 10);
        assert_eq!(new_state.message, "closure test");

        // Test that other actions return the same state
        let same_state = reducer.reduce(&initial_state, &TestAction::Reset);
        assert_eq!(same_state, initial_state);
    }

    #[test]
    fn test_reducer_trait_implementation() {
        let reducer = create_reducer(|state: &i32, action: &i32| state + action);

        // Test that the reducer implements the Reducer trait
        assert_eq!(reducer.reduce(&5, &3), 8);
        assert_eq!(reducer.reduce(&10, &-2), 8);
        assert_eq!(reducer.reduce(&0, &100), 100);
    }

    #[test]
    fn test_reducer_immutability() {
        let reducer = create_reducer(|state: &Vec<i32>, action: &i32| {
            let mut new_vec = state.clone();
            new_vec.push(*action);
            new_vec
        });

        let initial_state = vec![1, 2, 3];
        let new_state = reducer.reduce(&initial_state, &4);

        // Original state should be unchanged
        assert_eq!(initial_state, vec![1, 2, 3]);
        // New state should have the added element
        assert_eq!(new_state, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_complex_state_reducer() {
        #[derive(Clone, Debug, PartialEq)]
        struct ComplexState {
            users: Vec<String>,
            active_user: Option<String>,
            settings: std::collections::HashMap<String, String>,
        }

        enum ComplexAction {
            AddUser(String),
            SetActiveUser(String),
            SetSetting(String, String),
        }

        let reducer = create_reducer(
            |state: &ComplexState, action: &ComplexAction| match action {
                ComplexAction::AddUser(user) => {
                    let mut new_users = state.users.clone();
                    new_users.push(user.clone());
                    ComplexState {
                        users: new_users,
                        active_user: state.active_user.clone(),
                        settings: state.settings.clone(),
                    }
                }
                ComplexAction::SetActiveUser(user) => ComplexState {
                    users: state.users.clone(),
                    active_user: Some(user.clone()),
                    settings: state.settings.clone(),
                },
                ComplexAction::SetSetting(key, value) => {
                    let mut new_settings = state.settings.clone();
                    new_settings.insert(key.clone(), value.clone());
                    ComplexState {
                        users: state.users.clone(),
                        active_user: state.active_user.clone(),
                        settings: new_settings,
                    }
                }
            },
        );

        let initial_state = ComplexState {
            users: vec![],
            active_user: None,
            settings: std::collections::HashMap::new(),
        };

        // Add user
        let state1 = reducer.reduce(&initial_state, &ComplexAction::AddUser("alice".to_string()));
        assert_eq!(state1.users, vec!["alice"]);
        assert_eq!(state1.active_user, None);

        // Set active user
        let state2 = reducer.reduce(&state1, &ComplexAction::SetActiveUser("alice".to_string()));
        assert_eq!(state2.users, vec!["alice"]);
        assert_eq!(state2.active_user, Some("alice".to_string()));

        // Set setting
        let state3 = reducer.reduce(
            &state2,
            &ComplexAction::SetSetting("theme".to_string(), "dark".to_string()),
        );
        assert_eq!(state3.users, vec!["alice"]);
        assert_eq!(state3.active_user, Some("alice".to_string()));
        assert_eq!(state3.settings.get("theme"), Some(&"dark".to_string()));
    }
}
