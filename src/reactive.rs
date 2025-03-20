use std::collections::HashMap;

pub type ActionType = String;

pub type Reaction<T> = Box<dyn Fn(&mut T)>;

pub type ReactionMap<T> = HashMap<ActionType, Vec<Reaction<T>>>;

pub struct ReactiveSystem<T> {
    state: T,
    reactions: ReactionMap<T>,
}

impl<T> ReactiveSystem<T> {
    pub fn new(initial_state: T) -> Self {
        Self {
            state: initial_state,
            reactions: HashMap::new(),
        }
    }

    pub fn on<F>(&mut self, action_type: ActionType, callback: F)
    where
        F: 'static + Fn(&mut T),
    {
        self.reactions
            .entry(action_type)
            .or_default()
            .push(Box::new(callback));
    }

    pub fn trigger(&mut self, action_type: ActionType) {
        if let Some(callbacks) = self.reactions.get(&action_type) {
            for callback in callbacks {
                callback(&mut self.state);
            }
        }
    }

    pub fn current_state(&self) -> &T {
        &self.state
    }
}
