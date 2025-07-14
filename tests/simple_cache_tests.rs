#[cfg(test)]
mod simple_cache_tests {
    use zed::capsule::Cache;
    use zed::{Capsule, SimpleCache};

    #[test]
    fn test_new_cache_is_empty() {
        let cache: SimpleCache<i32> = SimpleCache::new();
        assert_eq!(cache.get(), None);
    }

    #[test]
    fn test_default_cache_is_empty() {
        let cache: SimpleCache<String> = SimpleCache::default();
        assert_eq!(cache.get(), None);
    }

    #[test]
    fn test_set_and_get() {
        let mut cache = SimpleCache::new();

        // Initially empty
        assert_eq!(cache.get(), None);

        // Set a value
        cache.set(42);
        assert_eq!(cache.get(), Some(42));

        // Set another value (should replace)
        cache.set(100);
        assert_eq!(cache.get(), Some(100));
    }

    #[test]
    fn test_cache_with_string() {
        let mut cache = SimpleCache::new();

        cache.set("hello".to_string());
        assert_eq!(cache.get(), Some("hello".to_string()));

        cache.set("world".to_string());
        assert_eq!(cache.get(), Some("world".to_string()));
    }

    #[test]
    fn test_cache_with_vec() {
        let mut cache = SimpleCache::new();

        let vec1 = vec![1, 2, 3];
        let vec2 = vec![4, 5, 6];

        cache.set(vec1.clone());
        assert_eq!(cache.get(), Some(vec1));

        cache.set(vec2.clone());
        assert_eq!(cache.get(), Some(vec2));
    }

    #[test]
    fn test_cache_clone() {
        let mut cache1 = SimpleCache::new();
        cache1.set("original".to_string());

        let cache2 = cache1.clone();

        // Both caches should have the same value
        assert_eq!(cache1.get(), Some("original".to_string()));
        assert_eq!(cache2.get(), Some("original".to_string()));

        // Modifying one shouldn't affect the other
        cache1.set("modified".to_string());
        assert_eq!(cache1.get(), Some("modified".to_string()));
        assert_eq!(cache2.get(), Some("original".to_string()));
    }

    #[test]
    fn test_cache_with_capsule() {
        let mut capsule = Capsule::new(0)
            .with_logic(|state: &mut i32, increment: i32| {
                *state += increment;
            })
            .with_cache(SimpleCache::new());

        // Get initial state
        assert_eq!(*capsule.get_state(), 0);

        // Dispatch an action
        capsule.dispatch(5);
        assert_eq!(*capsule.get_state(), 5);

        // Dispatch another action
        capsule.dispatch(3);
        assert_eq!(*capsule.get_state(), 8);
    }

    #[test]
    fn test_cache_invalidation_with_capsule() {
        let mut capsule = Capsule::new("initial".to_string())
            .with_logic(|state: &mut String, new_value: String| {
                *state = new_value;
            })
            .with_cache(SimpleCache::new());

        // Initial state
        assert_eq!(capsule.get_state(), "initial");

        // Change state
        capsule.dispatch("changed".to_string());
        assert_eq!(capsule.get_state(), "changed");

        // Change again
        capsule.dispatch("final".to_string());
        assert_eq!(capsule.get_state(), "final");
    }

    #[test]
    fn test_cache_with_complex_type() {
        #[derive(Clone, Debug, PartialEq)]
        struct ComplexData {
            id: u32,
            name: String,
            values: Vec<f64>,
        }

        let mut cache = SimpleCache::new();

        let data = ComplexData {
            id: 1,
            name: "test".to_string(),
            values: vec![1.1, 2.2, 3.3],
        };

        cache.set(data.clone());
        assert_eq!(cache.get(), Some(data));
    }

    #[test]
    fn test_multiple_overwrites() {
        let mut cache = SimpleCache::new();

        // Set multiple values in sequence
        for i in 0..10 {
            cache.set(i);
            assert_eq!(cache.get(), Some(i));
        }

        // Final value should be the last one set
        assert_eq!(cache.get(), Some(9));
    }

    #[test]
    fn test_cache_trait_implementation() {
        use zed::capsule::Cache;

        let mut cache: Box<dyn Cache<i32>> = Box::new(SimpleCache::new());

        assert_eq!(cache.get(), None);
        cache.set(42);
        assert_eq!(cache.get(), Some(42));
    }
}
