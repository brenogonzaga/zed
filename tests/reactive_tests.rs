use zed::ReactiveSystem;

#[derive(Clone, Debug, PartialEq)]
struct AppState {
    counter: i32,
    messages: Vec<String>,
    is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactive_system_creation() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let system = ReactiveSystem::new(initial_state.clone());
        assert_eq!(system.current_state(), &initial_state);
    }

    #[test]
    fn test_reactive_system_single_reaction() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state);

        system.on("increment".to_string(), |state: &mut AppState| {
            state.counter += 1;
        });

        system.trigger("increment".to_string());
        assert_eq!(system.current_state().counter, 1);

        system.trigger("increment".to_string());
        assert_eq!(system.current_state().counter, 2);
    }

    #[test]
    fn test_reactive_system_multiple_reactions_same_action() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state);

        // Multiple reactions for the same action type
        system.on("process".to_string(), |state: &mut AppState| {
            state.counter += 1;
        });

        system.on("process".to_string(), |state: &mut AppState| {
            state.messages.push("Processed".to_string());
        });

        system.on("process".to_string(), |state: &mut AppState| {
            state.is_active = true;
        });

        system.trigger("process".to_string());

        assert_eq!(system.current_state().counter, 1);
        assert_eq!(system.current_state().messages, vec!["Processed"]);
        assert!(system.current_state().is_active);
    }

    #[test]
    fn test_reactive_system_different_actions() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state);

        system.on("increment".to_string(), |state: &mut AppState| {
            state.counter += 1;
        });

        system.on("add_message".to_string(), |state: &mut AppState| {
            state.messages.push("Hello".to_string());
        });

        system.on("activate".to_string(), |state: &mut AppState| {
            state.is_active = true;
        });

        system.trigger("increment".to_string());
        assert_eq!(system.current_state().counter, 1);
        assert!(system.current_state().messages.is_empty());
        assert!(!system.current_state().is_active);

        system.trigger("add_message".to_string());
        assert_eq!(system.current_state().counter, 1);
        assert_eq!(system.current_state().messages, vec!["Hello"]);
        assert!(!system.current_state().is_active);

        system.trigger("activate".to_string());
        assert_eq!(system.current_state().counter, 1);
        assert_eq!(system.current_state().messages, vec!["Hello"]);
        assert!(system.current_state().is_active);
    }

    #[test]
    fn test_reactive_system_nonexistent_action() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state.clone());

        // Trigger a non-existent action - should do nothing
        system.trigger("nonexistent".to_string());
        assert_eq!(system.current_state(), &initial_state);
    }

    #[test]
    fn test_reactive_system_cascade_effects() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state);

        // Set up cascade: increment also logs and activates when counter reaches threshold
        system.on("increment".to_string(), |state: &mut AppState| {
            state.counter += 1;
        });

        system.on("increment".to_string(), |state: &mut AppState| {
            state
                .messages
                .push(format!("Counter is now {}", state.counter));
        });

        system.on("increment".to_string(), |state: &mut AppState| {
            if state.counter >= 3 {
                state.is_active = true;
            }
        });

        // First increment
        system.trigger("increment".to_string());
        assert_eq!(system.current_state().counter, 1);
        assert_eq!(system.current_state().messages, vec!["Counter is now 1"]);
        assert!(!system.current_state().is_active);

        // Second increment
        system.trigger("increment".to_string());
        assert_eq!(system.current_state().counter, 2);
        assert_eq!(
            system.current_state().messages,
            vec!["Counter is now 1", "Counter is now 2"]
        );
        assert!(!system.current_state().is_active);

        // Third increment - should activate
        system.trigger("increment".to_string());
        assert_eq!(system.current_state().counter, 3);
        assert_eq!(
            system.current_state().messages,
            vec!["Counter is now 1", "Counter is now 2", "Counter is now 3"]
        );
        assert!(system.current_state().is_active);
    }

    #[test]
    fn test_reactive_system_complex_state_updates() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state);

        // Reset action clears everything
        system.on("reset".to_string(), |state: &mut AppState| {
            state.counter = 0;
            state.messages.clear();
            state.is_active = false;
        });

        // Complex action that does multiple things
        system.on("complex_update".to_string(), |state: &mut AppState| {
            state.counter = 100;
        });

        system.on("complex_update".to_string(), |state: &mut AppState| {
            for i in 1..=5 {
                state.messages.push(format!("Message {i}"));
            }
        });

        system.on("complex_update".to_string(), |state: &mut AppState| {
            state.is_active = true;
        });

        // Trigger complex update
        system.trigger("complex_update".to_string());

        assert_eq!(system.current_state().counter, 100);
        assert_eq!(system.current_state().messages.len(), 5);
        assert_eq!(system.current_state().messages[0], "Message 1");
        assert_eq!(system.current_state().messages[4], "Message 5");
        assert!(system.current_state().is_active);

        // Reset everything
        system.trigger("reset".to_string());

        assert_eq!(system.current_state().counter, 0);
        assert!(system.current_state().messages.is_empty());
        assert!(!system.current_state().is_active);
    }

    #[test]
    fn test_reactive_system_order_of_execution() {
        let initial_state = AppState {
            counter: 0,
            messages: vec![],
            is_active: false,
        };

        let mut system = ReactiveSystem::new(initial_state);

        // Add reactions in a specific order to test execution order
        system.on("test".to_string(), |state: &mut AppState| {
            state.messages.push("First".to_string());
        });

        system.on("test".to_string(), |state: &mut AppState| {
            state.messages.push("Second".to_string());
        });

        system.on("test".to_string(), |state: &mut AppState| {
            state.messages.push("Third".to_string());
        });

        system.trigger("test".to_string());

        // Reactions should execute in the order they were added
        assert_eq!(
            system.current_state().messages,
            vec!["First", "Second", "Third"]
        );
    }
}
