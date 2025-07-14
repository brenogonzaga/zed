use serde::{Deserialize, Serialize};
use zed::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterState {
    pub value: i32,
    pub is_loading: bool,
    pub error: Option<String>,
}

create_slice! {
    enum_name: CounterActions,
    fn_base: counter,
    state: CounterState,
    initial_state: CounterState { value: 0, is_loading: false, error: None },
    actions: {
        StartLoading,
        Incremented,
        Decremented,
        SetValue { value: i32 },
        SetError { error: String },
        Reset,
    },
    reducer: |state: &mut CounterState, action: &CounterActions| {
        match action {
            CounterActions::StartLoading => {
                state.is_loading = true;
                state.error = None;
            },
            CounterActions::Incremented => {
                state.is_loading = false;
                state.value += 1;
                state.error = None;
            },
            CounterActions::Decremented => {
                state.is_loading = false;
                state.value -= 1;
                state.error = None;
            },
            CounterActions::SetValue { value } => {
                state.is_loading = false;
                state.value = *value;
                state.error = None;
            },
            CounterActions::SetError { error } => {
                state.is_loading = false;
                state.error = Some(error.clone());
            },
            CounterActions::Reset => {
                state.value = 0;
                state.is_loading = false;
                state.error = None;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_initial_state() {
        let expected = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };
        assert_eq!(COUNTER_INITIAL_STATE, expected);
    }

    #[test]
    fn test_slice_reducer_increment() {
        let initial_state = CounterState {
            value: 5,
            is_loading: true,
            error: Some("test error".to_string()),
        };

        let new_state = counter_reducer(&initial_state, &CounterActions::Incremented);

        assert_eq!(new_state.value, 6);
        assert!(!new_state.is_loading);
        assert!(new_state.error.is_none());
    }

    #[test]
    fn test_slice_reducer_decrement() {
        let initial_state = CounterState {
            value: 5,
            is_loading: true,
            error: Some("test error".to_string()),
        };

        let new_state = counter_reducer(&initial_state, &CounterActions::Decremented);

        assert_eq!(new_state.value, 4);
        assert!(!new_state.is_loading);
        assert!(new_state.error.is_none());
    }

    #[test]
    fn test_slice_reducer_set_value() {
        let initial_state = CounterState {
            value: 5,
            is_loading: true,
            error: Some("test error".to_string()),
        };

        let new_state = counter_reducer(&initial_state, &CounterActions::SetValue { value: 42 });

        assert_eq!(new_state.value, 42);
        assert!(!new_state.is_loading);
        assert!(new_state.error.is_none());
    }

    #[test]
    fn test_slice_reducer_start_loading() {
        let initial_state = CounterState {
            value: 5,
            is_loading: false,
            error: Some("test error".to_string()),
        };

        let new_state = counter_reducer(&initial_state, &CounterActions::StartLoading);

        assert_eq!(new_state.value, 5);
        assert!(new_state.is_loading);
        assert!(new_state.error.is_none());
    }

    #[test]
    fn test_slice_reducer_set_error() {
        let initial_state = CounterState {
            value: 5,
            is_loading: true,
            error: None,
        };

        let error_msg = "Network error".to_string();
        let new_state = counter_reducer(
            &initial_state,
            &CounterActions::SetError {
                error: error_msg.clone(),
            },
        );

        assert_eq!(new_state.value, 5);
        assert!(!new_state.is_loading);
        assert_eq!(new_state.error, Some(error_msg));
    }

    #[test]
    fn test_slice_reducer_reset() {
        let initial_state = CounterState {
            value: 42,
            is_loading: true,
            error: Some("test error".to_string()),
        };

        let new_state = counter_reducer(&initial_state, &CounterActions::Reset);

        assert_eq!(new_state.value, 0);
        assert!(!new_state.is_loading);
        assert!(new_state.error.is_none());
    }

    #[test]
    fn test_generated_store() {
        let store = counter_store();

        // Test initial state
        assert_eq!(store.get_state(), COUNTER_INITIAL_STATE);

        // Test dispatching actions
        store.dispatch(CounterActions::Incremented);
        assert_eq!(store.get_state().value, 1);

        store.dispatch(CounterActions::SetValue { value: 10 });
        assert_eq!(store.get_state().value, 10);

        store.dispatch(CounterActions::StartLoading);
        assert!(store.get_state().is_loading);

        store.dispatch(CounterActions::Reset);
        assert_eq!(store.get_state().value, 0);
        assert!(!store.get_state().is_loading);
    }
}
