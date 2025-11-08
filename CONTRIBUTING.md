# Contributing to Fluent Schedule

Thank you for your interest in contributing to Fluent Schedule! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.70 or later (stable toolchain recommended)
- Git

### Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/fluent_schedule.git
   cd fluent_schedule
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/Jacques-Murray/fluent_schedule.git
   ```

4. Create a development branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Building and Testing

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Run tests with verbose output:
```bash
cargo test --verbose
```

Run a specific example:
```bash
cargo run --example simple
```

Generate documentation:
```bash
cargo doc --open
```

## Code Style

This project follows standard Rust conventions:

- Use `rustfmt` for code formatting
- Use `clippy` for linting
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

Format code before committing:
```bash
cargo fmt
```

Check for linting issues:
```bash
cargo clippy
```

## Testing

- All new features must include comprehensive tests
- Tests should cover both happy path and error cases
- Use `cargo test` to run the full test suite
- Doc tests are also required for public APIs

### Test Coverage

Aim for high test coverage. Key areas to test:
- Job builder methods and validation
- Scheduler functionality
- Error handling
- Time calculations
- Edge cases (invalid times, missing tasks, etc.)

## Pull Request Process

1. Ensure all tests pass and code is formatted
2. Update documentation if needed
3. Write clear commit messages
4. Create a pull request with a descriptive title
5. Wait for review and address feedback

### Commit Messages

Use conventional commit format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test additions/changes
- `refactor:` for code refactoring
- `chore:` for maintenance tasks

Example: `feat: add support for monthly scheduling`

## Architecture Overview

### Core Components

- **`Job`**: Builder pattern for creating scheduled tasks
- **`Scheduler`**: Runtime that executes jobs at their scheduled times
- **`FluentDuration`**: Extension trait for human-readable time durations
- **`SchedulerError`**: Error types for configuration issues

### Key Design Decisions

- **Builder Pattern**: Jobs use a fluent builder for readable configuration
- **Single Threaded**: Scheduler runs in one thread for simplicity
- **Error Collection**: Invalid configurations are caught early with clear error messages
- **Zero-Copy**: Tasks are stored as boxed closures to avoid unnecessary allocations

## Adding New Features

### Scheduling Features

When adding new scheduling capabilities:

1. Extend the `ScheduleRules` struct in `job.rs`
2. Add builder methods to `Job` impl block
3. Implement time calculation logic in `calculate_next_run`
4. Add comprehensive tests
5. Update documentation and examples

### Error Types

New error conditions should:
- Be added to the `SchedulerError` enum
- Include descriptive error messages
- Have appropriate `Display` implementations
- Be tested for proper error propagation

## Documentation

- All public APIs must have documentation comments
- Include code examples where helpful
- Keep the README updated with new features
- Update doc tests when APIs change

## Performance Considerations

- Keep the scheduler loop efficient
- Avoid blocking operations in job closures
- Consider memory usage for long-running applications
- Profile performance-critical sections

## Code Review Checklist

Before submitting a PR, ensure:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Documentation is updated
- [ ] New features include tests
- [ ] Commit messages follow conventions
- [ ] Breaking changes are clearly documented

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Check the documentation for API details

Thank you for contributing to Fluent Schedule! 🎉