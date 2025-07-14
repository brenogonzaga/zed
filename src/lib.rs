//! # Zed - Advanced State Management for Rust
//!
//! Zed is a comprehensive state management library that provides Redux-like patterns
//! with advanced features for modern Rust applications.
//!
//! ## Features
//!
//! - **Redux-like Store**: Centralized, predictable state management
//! - **Timeline**: Time-reversible state with history and branching
//! - **State Mesh**: Distributed state nodes with conflict resolution
//! - **Capsules**: Encapsulated state domains with caching
//! - **Reactive System**: Cascade-triggered reactive updates
//!
//! ## Quick Start
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use zed::*;
//!
//! #[derive(Clone, Debug, Serialize, Deserialize)]
//! pub struct CounterState {
//!     pub value: i32,
//!     pub is_loading: bool,
//! }
//!
//! create_slice! {
//!     enum_name: CounterActions,
//!     fn_base: counter,
//!     state: CounterState,
//!     initial_state: CounterState { value: 0, is_loading: false },
//!     actions: {
//!         Increment,
//!         Decrement,
//!     },
//!     reducer: |state: &mut CounterState, action: &CounterActions| {
//!         match action {
//!             CounterActions::Increment => state.value += 1,
//!             CounterActions::Decrement => state.value -= 1,
//!         }
//!     }
//! }
//!
//! # fn main() {
//! let store = counter_store();
//!
//! store.subscribe(|state: &CounterState| {
//!     println!("State: {:?}", state);
//! });
//!
//! store.dispatch(CounterActions::Increment);
//! # }
//! ```

pub mod capsule;
pub mod configure_store;
pub mod create_slice;
pub mod reactive;
pub mod reducer;
pub mod simple_cache;
pub mod state_mesh;
pub mod store;
pub mod timeline;

pub use capsule::{Cache, Capsule};
pub use configure_store::configure_store;
pub use paste::paste;
pub use reactive::ReactiveSystem;
pub use reducer::{ClosureReducer, Reducer, create_reducer};
pub use simple_cache::SimpleCache;
pub use state_mesh::StateNode;
pub use store::Store;
pub use timeline::StateManager;
