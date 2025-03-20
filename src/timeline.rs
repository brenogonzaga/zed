use std::any::Any;

pub struct StateManager<T: Clone> {
    history: Vec<T>,
    current: usize,
    reducer: fn(&T, &dyn Any) -> T,
}

impl<T: Clone> StateManager<T> {
    pub fn new(initial_state: T, reducer: fn(&T, &dyn Any) -> T) -> Self {
        Self {
            history: vec![initial_state],
            current: 0,
            reducer,
        }
    }

    pub fn dispatch<A: 'static + Clone>(&mut self, action: A) {
        let current_state = self.history[self.current].clone();
        let new_state = (self.reducer)(&current_state, &action);
        self.history.truncate(self.current + 1);
        self.history.push(new_state);
        self.current += 1;
    }

    pub fn rewind(&mut self, steps: usize) {
        if steps > self.current {
            self.current = 0;
        } else {
            self.current -= steps;
        }
    }

    pub fn branch(&self) -> Self {
        let branch_history = self.history[..=self.current].to_vec();
        Self {
            history: branch_history,
            current: self.current,
            reducer: self.reducer,
        }
    }

    pub fn current_state(&self) -> &T {
        &self.history[self.current]
    }
}
