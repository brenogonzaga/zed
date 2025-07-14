use std::any::Any;
use zed::StateManager;

#[derive(Clone, Debug, PartialEq)]
struct TestState {
    counter: i32,
    name: String,
}

#[derive(Clone, Debug)]
enum TestAction {
    Increment,
    Decrement,
    SetName(String),
    Reset,
}

fn test_reducer(state: &TestState, action: &dyn Any) -> TestState {
    if let Some(action) = action.downcast_ref::<TestAction>() {
        match action {
            TestAction::Increment => TestState {
                counter: state.counter + 1,
                name: state.name.clone(),
            },
            TestAction::Decrement => TestState {
                counter: state.counter - 1,
                name: state.name.clone(),
            },
            TestAction::SetName(name) => TestState {
                counter: state.counter,
                name: name.clone(),
            },
            TestAction::Reset => TestState {
                counter: 0,
                name: "reset".to_string(),
            },
        }
    } else {
        state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let initial_state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        let manager = StateManager::new(initial_state.clone(), test_reducer);
        assert_eq!(manager.current_state(), &initial_state);
    }

    #[test]
    fn test_state_manager_dispatch() {
        let initial_state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        let mut manager = StateManager::new(initial_state, test_reducer);

        manager.dispatch(TestAction::Increment);
        assert_eq!(manager.current_state().counter, 1);

        manager.dispatch(TestAction::Increment);
        assert_eq!(manager.current_state().counter, 2);

        manager.dispatch(TestAction::Decrement);
        assert_eq!(manager.current_state().counter, 1);
    }

    #[test]
    fn test_state_manager_rewind() {
        let initial_state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        let mut manager = StateManager::new(initial_state, test_reducer);

        // Create some history
        manager.dispatch(TestAction::Increment);
        manager.dispatch(TestAction::Increment);
        manager.dispatch(TestAction::SetName("test".to_string()));

        assert_eq!(manager.current_state().counter, 2);
        assert_eq!(manager.current_state().name, "test");

        // Rewind 1 step
        manager.rewind(1);
        assert_eq!(manager.current_state().counter, 2);
        assert_eq!(manager.current_state().name, "initial");

        // Rewind 2 more steps
        manager.rewind(2);
        assert_eq!(manager.current_state().counter, 0);
        assert_eq!(manager.current_state().name, "initial");
    }

    #[test]
    fn test_state_manager_rewind_too_many_steps() {
        let initial_state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        let mut manager = StateManager::new(initial_state.clone(), test_reducer);

        manager.dispatch(TestAction::Increment);
        manager.dispatch(TestAction::Increment);

        // Try to rewind more steps than available
        manager.rewind(10);
        assert_eq!(manager.current_state(), &initial_state);
    }

    #[test]
    fn test_state_manager_branch() {
        let initial_state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        let mut manager = StateManager::new(initial_state, test_reducer);

        // Create some history
        manager.dispatch(TestAction::Increment);
        manager.dispatch(TestAction::Increment);

        // Rewind and create a branch
        manager.rewind(1);
        let mut branch = manager.branch();

        // Both should be at the same state now
        assert_eq!(manager.current_state().counter, 1);
        assert_eq!(branch.current_state().counter, 1);

        // Diverge the branches
        manager.dispatch(TestAction::SetName("original".to_string()));
        branch.dispatch(TestAction::SetName("branch".to_string()));

        assert_eq!(manager.current_state().name, "original");
        assert_eq!(branch.current_state().name, "branch");
    }

    #[test]
    fn test_state_manager_dispatch_after_rewind() {
        let initial_state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        let mut manager = StateManager::new(initial_state, test_reducer);

        // Create history
        manager.dispatch(TestAction::Increment);
        manager.dispatch(TestAction::Increment);
        manager.dispatch(TestAction::Increment);

        assert_eq!(manager.current_state().counter, 3);

        // Rewind and dispatch new action
        manager.rewind(2);
        assert_eq!(manager.current_state().counter, 1);

        manager.dispatch(TestAction::SetName("new_branch".to_string()));
        assert_eq!(manager.current_state().counter, 1);
        assert_eq!(manager.current_state().name, "new_branch");

        // The future history should be truncated
        manager.dispatch(TestAction::Increment);
        assert_eq!(manager.current_state().counter, 2);
    }

    #[test]
    fn test_state_manager_complex_workflow() {
        let initial_state = TestState {
            counter: 0,
            name: "start".to_string(),
        };

        let mut manager = StateManager::new(initial_state, test_reducer);

        // Build up some state
        for _ in 1..=5 {
            manager.dispatch(TestAction::Increment);
        }
        manager.dispatch(TestAction::SetName("final".to_string()));

        // Current state should be counter: 5, name: "final"
        assert_eq!(manager.current_state().counter, 5);
        assert_eq!(manager.current_state().name, "final");

        // Go back to middle
        manager.rewind(3);
        assert_eq!(manager.current_state().counter, 3);
        assert_eq!(manager.current_state().name, "start");

        // Create alternative branch
        manager.dispatch(TestAction::SetName("alternative".to_string()));
        manager.dispatch(TestAction::Reset);

        assert_eq!(manager.current_state().counter, 0);
        assert_eq!(manager.current_state().name, "reset");
    }
}
