# Contributing to Zed

Thank you for your interest in contributing to Zed! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Documentation](#documentation)
- [Performance](#performance)

## Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all contributors. Please be respectful and considerate in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/zed.git
   cd zed
   ```
3. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70+ (2024 edition)
- Cargo (comes with Rust)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Running Examples

```bash
# Basic Redux example
cargo run --example example_redux

# Advanced Todo app
cargo run --example example_advanced_todo

# Collaborative editor
cargo run --example example_collaborative_editor
```

## Testing

We maintain comprehensive test coverage. Please ensure all tests pass and add tests for new functionality.

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test store_tests

# Run with output
cargo test -- --nocapture
```

### Test Categories

#### Unit Tests

- **Store Tests** (`tests/store_tests.rs`): Core store functionality
- **Timeline Tests** (`tests/timeline_tests.rs`): Time travel features
- **State Mesh Tests** (`tests/state_mesh_tests.rs`): Distributed state
- **Capsule Tests** (`tests/capsule_tests.rs`): Encapsulated domains
- **Reactive Tests** (`tests/reactive_tests.rs`): Reactive systems
- **Slice Tests** (`tests/slice_tests.rs`): Macro-generated code

#### Integration Tests

- Cross-module functionality
- Real-world usage patterns
- Performance characteristics

#### Documentation Tests

- All code examples in documentation
- API usage patterns
- Edge cases and error handling

### Writing Tests

#### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let initial_state = TestState::new();

        // Act
        let result = perform_operation(initial_state);

        // Assert
        assert_eq!(result.expected_field, expected_value);
    }
}
```

#### Test Naming

- Use descriptive names: `test_store_dispatch_updates_state`
- Group related tests: `test_timeline_rewind_*`
- Test both success and failure cases

## Benchmarks

We use Criterion for performance benchmarks:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench store_benchmarks

# Generate benchmark reports
cargo bench -- --output-format html
```

### Writing Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_your_feature(c: &mut Criterion) {
    c.bench_function("feature_name", |b| {
        b.iter(|| {
            // Code to benchmark
            your_function(black_box(input));
        })
    });
}

criterion_group!(benches, bench_your_feature);
criterion_main!(benches);
```

## Submitting Changes

### Pull Request Process

1. **Update documentation** for any new features
2. **Add or update tests** to maintain coverage
3. **Run the full test suite** and ensure all tests pass
4. **Check code formatting**:
   ```bash
   cargo fmt --check
   ```
5. **Run clippy** for linting:
   ```bash
   cargo clippy -- -D warnings
   ```
6. **Update CHANGELOG.md** with your changes
7. **Submit a Pull Request** with a clear description

### Pull Request Template

```markdown
## Description

Brief description of changes

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement

## Testing

- [ ] Tests added/updated
- [ ] All tests pass
- [ ] Benchmarks run (if performance-related)

## Documentation

- [ ] Code comments updated
- [ ] README updated (if needed)
- [ ] API documentation updated
```

## Code Style

### Rust Style Guidelines

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` for consistent formatting
- Address all `cargo clippy` warnings

### Naming Conventions

- **Types**: `PascalCase` (e.g., `StateManager`, `TodoAction`)
- **Functions/Variables**: `snake_case` (e.g., `get_state`, `current_position`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_CAPACITY`)
- **Modules**: `snake_case` (e.g., `state_mesh`, `timeline`)

### Error Handling

- Use `Result<T, E>` for fallible operations
- Provide meaningful error messages
- Use `?` operator for error propagation
- Consider creating custom error types for complex cases

### Memory Management

- Prefer borrowing over cloning when possible
- Use `Arc` and `Mutex` for shared mutable state
- Be mindful of circular references
- Profile memory usage for performance-critical code

## Documentation

### Code Documentation

- All public APIs must have documentation comments
- Include examples in documentation when helpful
- Document panics, errors, and safety requirements
- Use `cargo doc` to generate and review documentation

### Documentation Comments

````rust
/// Brief description of the function.
///
/// Longer description with more details about the behavior,
/// use cases, and any important considerations.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of what the function returns
///
/// # Examples
///
/// ```rust
/// use zed::Store;
///
/// let store = Store::new(initial_state, reducer);
/// store.dispatch(Action::Increment);
/// ```
///
/// # Panics
///
/// Describe any conditions that cause panics
///
/// # Errors
///
/// Describe error conditions for functions returning Result
pub fn documented_function(param1: Type1, param2: Type2) -> ReturnType {
    // Implementation
}
````

### README and Guides

- Keep examples up-to-date
- Include performance characteristics
- Document breaking changes clearly
- Provide migration guides for major updates

## Performance

### Performance Guidelines

- Profile before optimizing
- Use benchmarks to measure improvements
- Consider both time and memory performance
- Test with realistic data sizes
- Document performance characteristics

### Common Optimizations

- Minimize allocations in hot paths
- Use appropriate data structures
- Consider lazy evaluation
- Cache expensive computations
- Use `black_box` in benchmarks to prevent optimization

### Performance Testing

```rust
// Example benchmark structure
fn bench_performance_critical_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("feature_name");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                // Setup
                let data = setup_test_data(size);

                b.iter(|| {
                    // Critical path to benchmark
                    critical_function(black_box(&data));
                });
            },
        );
    }

    group.finish();
}
```

## Feature Development

### Adding New Features

1. **Design Phase**:

   - Open an issue to discuss the feature
   - Consider API design and backwards compatibility
   - Plan the implementation approach

2. **Implementation Phase**:

   - Start with tests (TDD approach)
   - Implement the minimal viable feature
   - Add comprehensive documentation
   - Include examples and benchmarks

3. **Review Phase**:
   - Self-review your code
   - Test edge cases thoroughly
   - Consider performance implications
   - Update documentation

### Breaking Changes

Breaking changes require special consideration:

- Bump major version number
- Provide migration guide
- Consider deprecation period
- Document in CHANGELOG.md clearly

## Getting Help

- **Issues**: Open a GitHub issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Documentation**: Check the README and API docs first
- **Examples**: Look at existing examples for usage patterns

## Recognition

Contributors are recognized in:

- CHANGELOG.md for significant contributions
- README.md acknowledgments section
- Git commit history

Thank you for contributing to Zed! 🚀
