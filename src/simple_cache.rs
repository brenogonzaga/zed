//! # Simple Cache Module
//!
//! This module provides a basic cache implementation that can be used with the Capsule system.
//! The SimpleCache stores a single optional value and implements the Cache trait.
//!
//! This is useful for:
//! - Memoizing expensive computations
//! - Storing the last computed result
//! - Simple key-value storage within capsules
//!
//! ## Example
//!
//! ```rust
//! use zed::{SimpleCache, Capsule};
//! use zed::capsule::Cache;
//!
//! // Create a cache and use it directly
//! let mut cache = SimpleCache::new();
//! cache.set("Hello".to_string());
//! assert_eq!(cache.get(), Some("Hello".to_string()));
//!
//! // Create a capsule with simple cache
//! let capsule: Capsule<String, ()> = Capsule::new("Hello".to_string())
//!     .with_cache(SimpleCache::new());
//!
//! // Get the initial state
//! assert_eq!(capsule.get_state(), "Hello");
//! ```

/// A simple cache implementation that stores a single optional value.
///
/// This cache can hold at most one value at a time. When a value is set,
/// it replaces any previously stored value. When retrieved, it returns
/// a clone of the stored value.
///
/// # Type Parameters
///
/// * `T` - The type of value to cache. Must implement Clone.
///
/// # Example
///
/// ```rust
/// use zed::SimpleCache;
/// use zed::capsule::Cache;
///
/// let mut cache = SimpleCache::<i32>::new();
///
/// // Initially empty
/// assert_eq!(cache.get(), None);
///
/// // Set a value
/// cache.set(42);
/// assert_eq!(cache.get(), Some(42));
///
/// // Set another value (replaces the first)
/// cache.set(100);
/// assert_eq!(cache.get(), Some(100));
/// ```
#[derive(Clone)]
pub struct SimpleCache<T: Clone> {
    value: Option<T>,
}

impl<T: Clone> SimpleCache<T> {
    /// Creates a new empty SimpleCache.
    ///
    /// # Returns
    ///
    /// A new SimpleCache instance with no stored value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zed::SimpleCache;
    /// use zed::capsule::Cache;
    ///
    /// let cache: SimpleCache<String> = SimpleCache::new();
    /// assert_eq!(cache.get(), None);
    /// ```
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl<T: Clone> Default for SimpleCache<T> {
    /// Creates a default (empty) SimpleCache.
    ///
    /// This is equivalent to calling `SimpleCache::new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> super::capsule::Cache<T> for SimpleCache<T> {
    /// Retrieves a clone of the cached value, if any.
    ///
    /// # Returns
    ///
    /// `Some(value)` if a value is cached, `None` if the cache is empty.
    fn get(&self) -> Option<T> {
        self.value.clone()
    }

    /// Stores a value in the cache.
    ///
    /// If the cache already contains a value, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to store in the cache
    fn set(&mut self, value: T) {
        self.value = Some(value);
    }
}
