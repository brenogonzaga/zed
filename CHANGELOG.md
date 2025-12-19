# Changelog

All notable changes to the Zed state management library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-19

### Added

- **Unified Store with Enhanced Features**:
  - `subscribe()` now returns a `SubscriptionId` for unsubscription
  - `unsubscribe(id)` to remove subscribers and prevent memory leaks
  - `dispatch_batch(actions)` for efficient batch operations (single notification)
  - `with_state(f)` for read-only state access without cloning
  - `subscriber_count()` to monitor active subscriptions
  - Export `SubscriptionId` type alias
- **Production-Ready Test Suite**:
  - 15 comprehensive edge case tests covering integer overflow, saturation, and concurrent access
  - 8 integration tests demonstrating real-world e-commerce scenarios
  - Complete serialization/deserialization roundtrip tests
  - Stress tests with 50+ concurrent threads
  - Tests for large data structures (10k+ items)
- **CI/CD Pipeline**:
  - GitHub Actions workflow with multi-platform testing (Linux, macOS, Windows)
  - Documentation validation with rustdoc warnings as errors
  - Automated benchmarking on main branch
  - Rust stable and beta channel testing

### Changed

- **Store API**: `subscribe()` now returns `SubscriptionId` instead of `()` (breaking change)
- Removed `OptimizedStore` - all its features are now in the main `Store`
- Consolidated examples: removed redundant `example_advanced_patterns.rs`, `example_game_state.rs`, and `example_collaborative_editor.rs`

### Fixed

- Fixed race condition in `dispatch()` - now holds lock for entire read-modify-write cycle
- Fixed test assertions for concurrent operations to be deterministic
- Improved error handling in edge cases
- Enhanced state consistency validation

### Performance

- Optimized `dispatch()` for thread-safe atomic updates
- Verified performance under extreme loads (1000+ concurrent operations)
- Benchmarks included in CI/CD

## [0.1.17] - 2024-07-14

### Added

- **Comprehensive Test Suite**: Added extensive unit tests for all modules
  - Store tests with concurrent access, subscription, and reducer replacement
  - Timeline tests with rewind, branching, and complex workflows
  - State Mesh tests with conflict resolution and propagation
  - Capsule tests with logic and caching integration
  - Reactive System tests with cascading effects and ordering
  - Slice tests for macro-generated code
- **Performance Benchmarks**: Added Criterion-based benchmarks
  - Store performance under various loads and conditions
  - Timeline memory usage and operation performance
  - State Mesh throughput and conflict resolution performance
- **Enhanced Documentation**:
  - Comprehensive module-level documentation with examples
  - Detailed API documentation for all public methods
  - Updated README with advanced examples and use cases
  - Doctests for critical functionality
- **Advanced Examples**:
  - Todo application demonstrating Redux-like patterns
  - Collaborative document editor showcasing State Mesh capabilities
  - Reactive system examples with cascading effects
- **Improved Error Handling**: Better error messages and edge case handling

### Enhanced

- **Store Module**: Added detailed documentation and examples for all methods
- **Timeline Module**: Improved documentation with time travel use cases
- **State Mesh Module**: Enhanced conflict resolution documentation and examples
- **Type Safety**: Improved type annotations and bounds checking
- **Code Quality**: Applied clippy suggestions and Rust best practices

### Performance

- **Memory Efficiency**: Optimized state cloning and memory usage
- **Concurrent Access**: Improved thread safety and performance
- **Conflict Resolution**: Optimized state mesh propagation algorithms

### Documentation

- **README**: Complete rewrite with modern examples and clear use cases
- **API Docs**: Comprehensive documentation for all public APIs
- **Examples**: Real-world usage patterns and advanced scenarios
- **Benchmarks**: Performance characteristics and optimization guides

### Testing

- Unit tests for all core modules
- Integration tests for real-world scenarios
- Stress tests with concurrent access
- Documentation tests (doctests)
