use std::collections::HashMap;
use std::sync::Arc;

pub type NodeId = String;

pub type ConflictResolver<T> = Arc<dyn Fn(&mut T, &T) + Send + Sync>;

pub type StateNodeConnections<T> = HashMap<NodeId, StateNode<T>>;

#[derive(Clone)]
pub struct StateNode<T: Clone> {
    pub id: NodeId,
    pub state: T,
    pub connections: StateNodeConnections<T>,
    pub on_conflict: Option<ConflictResolver<T>>,
}

impl<T: Clone> StateNode<T> {
    pub fn new(id: NodeId, initial_state: T) -> Self {
        Self {
            id,
            state: initial_state,
            connections: HashMap::new(),
            on_conflict: None,
        }
    }

    pub fn connect(&mut self, other: StateNode<T>) {
        self.connections.insert(other.id.clone(), other);
    }

    pub fn set_conflict_resolver<F>(&mut self, resolver: F)
    where
        F: 'static + Fn(&mut T, &T) + Send + Sync,
    {
        self.on_conflict = Some(Arc::new(resolver));
    }

    pub fn resolve_conflict(&mut self, remote_state: T) {
        if let Some(ref resolver) = self.on_conflict {
            resolver(&mut self.state, &remote_state);
        } else {
            self.state = remote_state;
        }
    }
}
