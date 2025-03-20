pub type Logic<T, Action> = Box<dyn Fn(&mut T, Action)>;

pub type CacheBox<T> = Box<dyn Cache<T>>;

pub trait Cache<T> {
    fn get(&self) -> Option<T>;
    fn set(&mut self, value: T);
}

pub struct Capsule<T, Action> {
    state: T,
    logic: Option<Logic<T, Action>>,
    cache: Option<CacheBox<T>>,
}

impl<T: Clone, Action: Clone> Capsule<T, Action> {
    pub fn new(initial_state: T) -> Self {
        Self {
            state: initial_state,
            logic: None,
            cache: None,
        }
    }

    pub fn with_logic<F>(mut self, logic: F) -> Self
    where
        F: 'static + Fn(&mut T, Action),
    {
        self.logic = Some(Box::new(logic));
        self
    }

    pub fn with_cache<C>(mut self, cache: C) -> Self
    where
        C: 'static + Cache<T>,
    {
        self.cache = Some(Box::new(cache));
        self
    }

    pub fn dispatch(&mut self, action: Action) {
        if let Some(ref logic) = self.logic {
            logic(&mut self.state, action);
        }
        if let Some(ref mut cache) = self.cache {
            cache.set(self.state.clone());
        }
    }

    pub fn get_state(&self) -> &T {
        &self.state
    }
}
