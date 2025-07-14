//! # State Mesh Module
//!
//! This module provides distributed state management through interconnected state nodes.
//! It's designed for collaborative applications where different parts of the state need
//! to be synchronized across multiple sources with intelligent conflict resolution.
//!
//! ## Features
//!
//! - **Distributed State**: State represented as nodes in a graph
//! - **Conflict Resolution**: Pluggable conflict resolution strategies
//! - **State Propagation**: Automatic propagation of updates to connected nodes
//! - **Flexible Topology**: Arbitrary connection patterns between nodes
//!
//! ## Use Cases
//!
//! - Collaborative editing (like Google Docs)
//! - Multiplayer games with state synchronization
//! - Distributed systems with eventual consistency
//! - P2P applications with shared state
//!
//! ## Example
//!
//! ```rust
//! use zed::StateNode;
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct Document {
//!     content: String,
//!     version: u32,
//! }
//!
//! # fn main() {
//! let mut node1 = StateNode::new("user1".to_string(), Document {
//!     content: "Hello".to_string(),
//!     version: 1,
//! });
//!
//! let node2 = StateNode::new("user2".to_string(), Document {
//!     content: "Hi".to_string(),
//!     version: 2,
//! });
//!
//! // Set up last-write-wins conflict resolution
//! node1.set_conflict_resolver(|current: &mut Document, remote: &Document| {
//!     if remote.version > current.version {
//!         *current = remote.clone();
//!     }
//! });
//!
//! node1.connect(node2);
//! node1.propagate_update(); // Sync states
//! # }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for node identifiers
pub type NodeId = String;

/// Type alias for conflict resolution functions
///
/// The function takes a mutable reference to the current state and an immutable
/// reference to the remote state, allowing you to implement various conflict
/// resolution strategies like last-write-wins, merge, or custom logic.
pub type ConflictResolver<T> = Arc<dyn Fn(&mut T, &T) + Send + Sync>;

/// Type alias for the connections map
pub type StateNodeConnections<T> = HashMap<NodeId, StateNode<T>>;

/// A node in the state mesh representing a piece of distributed state.
///
/// Each node maintains its own state and connections to other nodes. When conflicts
/// arise between different versions of state, the node uses its conflict resolver
/// to determine how to merge or choose between conflicting states.
#[derive(Clone)]
pub struct StateNode<T: Clone> {
    /// Unique identifier for this node
    pub id: NodeId,
    /// The current state stored in this node
    pub state: T,
    /// Map of connected nodes by their IDs
    pub connections: StateNodeConnections<T>,
    /// Optional conflict resolution strategy
    pub on_conflict: Option<ConflictResolver<T>>,
}

impl<T: Clone> StateNode<T> {
    /// Creates a new state node with the given ID and initial state.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this node
    /// * `initial_state` - The starting state for this node
    ///
    /// # Example
    ///
    /// ```rust
    /// use zed::StateNode;
    ///
    /// #[derive(Clone)]
    /// struct MyState { value: i32 }
    ///
    /// let node = StateNode::new("node1".to_string(), MyState { value: 42 });
    /// ```
    pub fn new(id: NodeId, initial_state: T) -> Self {
        Self {
            id,
            state: initial_state,
            connections: HashMap::new(),
            on_conflict: None,
        }
    }

    /// Connects this node to another node.
    ///
    /// This creates a one-way connection from this node to the other node.
    /// For bidirectional connections, you need to call connect on both nodes.
    ///
    /// # Arguments
    ///
    /// * `other` - The node to connect to
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::StateNode;
    /// # #[derive(Clone)] struct MyState { value: i32 }
    /// let mut node1 = StateNode::new("node1".to_string(), MyState { value: 1 });
    /// let node2 = StateNode::new("node2".to_string(), MyState { value: 2 });
    ///
    /// node1.connect(node2);
    /// ```
    pub fn connect(&mut self, other: StateNode<T>) {
        self.connections.insert(other.id.clone(), other);
    }

    /// Removes a connection to another node.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the node to disconnect
    ///
    /// # Returns
    ///
    /// The removed node if it existed, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::StateNode;
    /// # #[derive(Clone)] struct MyState { value: i32 }
    /// # let mut node1 = StateNode::new("node1".to_string(), MyState { value: 1 });
    /// # let node2 = StateNode::new("node2".to_string(), MyState { value: 2 });
    /// # node1.connect(node2);
    /// let removed = node1.remove_connection(&"node2".to_string());
    /// ```
    pub fn remove_connection(&mut self, id: &NodeId) -> Option<StateNode<T>> {
        self.connections.remove(id)
    }

    /// Sets a conflict resolution strategy for this node.
    ///
    /// The resolver function will be called whenever there's a conflict between
    /// this node's state and incoming remote state. Common strategies include:
    /// - Last write wins (based on timestamp)
    /// - Merge strategies (for structured data)
    /// - Custom business logic
    ///
    /// # Arguments
    ///
    /// * `resolver` - Function that takes (current_state, remote_state) and modifies current_state
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::StateNode;
    /// # #[derive(Clone)] struct MyState { value: i32, version: u32 }
    /// # let mut node = StateNode::new("node1".to_string(), MyState { value: 1, version: 1 });
    /// // Last-write-wins based on version
    /// node.set_conflict_resolver(|current: &mut MyState, remote: &MyState| {
    ///     if remote.version > current.version {
    ///         *current = remote.clone();
    ///     }
    /// });
    /// ```
    pub fn set_conflict_resolver<F>(&mut self, resolver: F)
    where
        F: 'static + Fn(&mut T, &T) + Send + Sync,
    {
        self.on_conflict = Some(Arc::new(resolver));
    }

    /// Resolves a conflict with remote state using the configured strategy.
    ///
    /// If no conflict resolver is set, this defaults to replacing the current
    /// state with the remote state.
    ///
    /// # Arguments
    ///
    /// * `remote_state` - The conflicting state from a remote source
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::StateNode;
    /// # #[derive(Clone)] struct MyState { value: i32 }
    /// # let mut node = StateNode::new("node1".to_string(), MyState { value: 1 });
    /// let remote_state = MyState { value: 42 };
    /// node.resolve_conflict(remote_state);
    /// ```
    pub fn resolve_conflict(&mut self, remote_state: T) {
        if let Some(ref resolver) = self.on_conflict {
            resolver(&mut self.state, &remote_state);
        } else {
            self.state = remote_state;
        }
    }

    /// Propagates this node's current state to all connected nodes.
    ///
    /// This triggers conflict resolution on each connected node, potentially
    /// updating their states based on their conflict resolution strategies.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::StateNode;
    /// # #[derive(Clone)] struct MyState { value: i32 }
    /// # let mut node1 = StateNode::new("node1".to_string(), MyState { value: 1 });
    /// # let node2 = StateNode::new("node2".to_string(), MyState { value: 2 });
    /// # node1.connect(node2);
    /// node1.propagate_update(); // All connected nodes receive this node's state
    /// ```
    pub fn propagate_update(&mut self) {
        for node in self.connections.values_mut() {
            node.resolve_conflict(self.state.clone());
        }
    }

    /// Merges state from another node using conflict resolution.
    ///
    /// This is a convenience method that calls resolve_conflict with the other node's state.
    ///
    /// # Arguments
    ///
    /// * `other` - The node whose state to merge with
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zed::StateNode;
    /// # #[derive(Clone)] struct MyState { value: i32 }
    /// # let mut node1 = StateNode::new("node1".to_string(), MyState { value: 1 });
    /// # let node2 = StateNode::new("node2".to_string(), MyState { value: 2 });
    /// node1.merge(&node2); // Merge node2's state into node1
    /// ```
    pub fn merge(&mut self, other: &StateNode<T>) {
        self.resolve_conflict(other.state.clone());
    }
}
