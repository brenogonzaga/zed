use crate::reducer::Reducer;
use std::sync::{Arc, Mutex};

type SharedState<S> = Arc<Mutex<S>>;
type Subscriber<State> = Box<dyn Fn(&State) + Send + Sync>;
type Subscribers<State> = Arc<Mutex<Vec<Subscriber<State>>>>;

pub struct Store<State, Action> {
    state: SharedState<State>,
    reducer: Arc<Mutex<Box<dyn Reducer<State, Action> + Send + Sync>>>,
    subscribers: Subscribers<State>,
}

impl<State: Clone + Send + 'static, Action: Send + 'static> Store<State, Action> {
    pub fn new(
        initial_state: State,
        reducer: Box<dyn Reducer<State, Action> + Send + Sync>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
            reducer: Arc::new(Mutex::new(reducer)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn dispatch(&self, action: Action) {
        let mut state = self.state.lock().unwrap();
        let current_reducer = self.reducer.lock().unwrap();
        let new_state = current_reducer.reduce(&state, &action);
        *state = new_state.clone();

        for subscriber in self.subscribers.lock().unwrap().iter() {
            subscriber(&new_state);
        }
    }

    pub fn subscribe<F>(&self, f: F)
    where
        F: Fn(&State) + Send + Sync + 'static,
    {
        self.subscribers.lock().unwrap().push(Box::new(f));
    }

    pub fn get_state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    pub fn replace_reducer(&self, new_reducer: Box<dyn Reducer<State, Action> + Send + Sync>) {
        let mut reducer = self.reducer.lock().unwrap();
        *reducer = new_reducer;
    }
}
