use zed::{Cache, Capsule, SimpleCache};

#[derive(Clone, Debug, PartialEq)]
struct CounterState {
    value: i32,
    history: Vec<i32>,
}

#[derive(Clone, Debug)]
enum CounterAction {
    Increment,
    Decrement,
    Reset,
    SetValue(i32),
}

struct TestCache<T: Clone> {
    value: Option<T>,
    access_count: u32,
}

impl<T: Clone> TestCache<T> {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            value: None,
            access_count: 0,
        }
    }

    #[allow(dead_code)]
    fn get_access_count(&self) -> u32 {
        self.access_count
    }
}

impl<T: Clone> Cache<T> for TestCache<T> {
    fn get(&self) -> Option<T> {
        self.value.clone()
    }

    fn set(&mut self, value: T) {
        self.value = Some(value);
        self.access_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_creation() {
        let initial_state = CounterState {
            value: 0,
            history: vec![],
        };

        let capsule: Capsule<CounterState, CounterAction> = Capsule::new(initial_state.clone());
        assert_eq!(capsule.get_state(), &initial_state);
    }

    #[test]
    fn test_capsule_with_logic() {
        let initial_state = CounterState {
            value: 0,
            history: vec![],
        };

        let mut capsule = Capsule::new(initial_state).with_logic(
            |state: &mut CounterState, action: CounterAction| match action {
                CounterAction::Increment => {
                    state.value += 1;
                    state.history.push(state.value);
                }
                CounterAction::Decrement => {
                    state.value -= 1;
                    state.history.push(state.value);
                }
                CounterAction::Reset => {
                    state.value = 0;
                    state.history.clear();
                }
                CounterAction::SetValue(v) => {
                    state.value = v;
                    state.history.push(v);
                }
            },
        );

        capsule.dispatch(CounterAction::Increment);
        assert_eq!(capsule.get_state().value, 1);
        assert_eq!(capsule.get_state().history, vec![1]);

        capsule.dispatch(CounterAction::Increment);
        assert_eq!(capsule.get_state().value, 2);
        assert_eq!(capsule.get_state().history, vec![1, 2]);

        capsule.dispatch(CounterAction::Decrement);
        assert_eq!(capsule.get_state().value, 1);
        assert_eq!(capsule.get_state().history, vec![1, 2, 1]);
    }

    #[test]
    fn test_capsule_with_simple_cache() {
        let initial_state = CounterState {
            value: 0,
            history: vec![],
        };

        let cache = SimpleCache::new();
        let mut capsule = Capsule::new(initial_state)
            .with_logic(
                |state: &mut CounterState, action: CounterAction| match action {
                    CounterAction::Increment => state.value += 1,
                    CounterAction::Decrement => state.value -= 1,
                    CounterAction::Reset => state.value = 0,
                    CounterAction::SetValue(v) => state.value = v,
                },
            )
            .with_cache(cache);

        capsule.dispatch(CounterAction::Increment);
        assert_eq!(capsule.get_state().value, 1);

        capsule.dispatch(CounterAction::SetValue(42));
        assert_eq!(capsule.get_state().value, 42);
    }

    #[test]
    fn test_capsule_with_custom_cache() {
        let initial_state = CounterState {
            value: 0,
            history: vec![],
        };

        let cache = TestCache::new();
        let mut capsule = Capsule::new(initial_state)
            .with_logic(|state: &mut CounterState, action: CounterAction| {
                if let CounterAction::Increment = action {
                    state.value += 1;
                }
            })
            .with_cache(cache);

        // Initially cache should not have been accessed
        // Note: We can't directly access the cache from outside the capsule
        // This test verifies the cache integration works

        capsule.dispatch(CounterAction::Increment);
        assert_eq!(capsule.get_state().value, 1);

        capsule.dispatch(CounterAction::Increment);
        assert_eq!(capsule.get_state().value, 2);
    }

    #[test]
    fn test_capsule_without_logic() {
        let initial_state = CounterState {
            value: 42,
            history: vec![],
        };

        let mut capsule = Capsule::new(initial_state.clone());

        // Dispatching without logic should not change state
        capsule.dispatch(CounterAction::Increment);
        assert_eq!(capsule.get_state(), &initial_state);
    }

    #[test]
    fn test_capsule_reset_functionality() {
        let initial_state = CounterState {
            value: 0,
            history: vec![],
        };

        let mut capsule = Capsule::new(initial_state).with_logic(
            |state: &mut CounterState, action: CounterAction| match action {
                CounterAction::Increment => {
                    state.value += 1;
                    state.history.push(state.value);
                }
                CounterAction::Reset => {
                    state.value = 0;
                    state.history.clear();
                }
                _ => {}
            },
        );

        // Build up some state
        capsule.dispatch(CounterAction::Increment);
        capsule.dispatch(CounterAction::Increment);
        capsule.dispatch(CounterAction::Increment);

        assert_eq!(capsule.get_state().value, 3);
        assert_eq!(capsule.get_state().history, vec![1, 2, 3]);

        // Reset
        capsule.dispatch(CounterAction::Reset);

        assert_eq!(capsule.get_state().value, 0);
        assert!(capsule.get_state().history.is_empty());
    }

    #[test]
    fn test_simple_cache_basic_operations() {
        let mut cache = SimpleCache::<i32>::new();

        // Initially empty
        assert!(cache.get().is_none());

        // Set a value
        cache.set(42);
        assert_eq!(cache.get(), Some(42));

        // Update value
        cache.set(100);
        assert_eq!(cache.get(), Some(100));
    }

    #[test]
    fn test_simple_cache_with_complex_type() {
        let mut cache = SimpleCache::<CounterState>::new();

        let state1 = CounterState {
            value: 10,
            history: vec![1, 2, 3],
        };

        let state2 = CounterState {
            value: 20,
            history: vec![4, 5, 6],
        };

        assert!(cache.get().is_none());

        cache.set(state1.clone());
        assert_eq!(cache.get(), Some(state1));

        cache.set(state2.clone());
        assert_eq!(cache.get(), Some(state2));
    }
}
