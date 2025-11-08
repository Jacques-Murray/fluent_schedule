# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release preparation
- Comprehensive documentation and examples
- CI/CD pipeline with GitHub Actions
- Code of Conduct and Contributing guidelines

## [0.1.0] - 2025-11-08

### Added
- **Fluent API**: Chainable builder pattern for creating scheduled jobs
- **Interval Scheduling**: Support for running jobs at fixed time intervals (`.every()`)
- **Time-Based Scheduling**: Support for running jobs at specific times (`.at()`)
- **Day-of-Week Scheduling**: Support for scheduling on specific weekdays (`.on()`, `.on_weekday()`, `.on_weekend()`)
- **Fluent Duration Extensions**: Human-readable time units (`5.seconds()`, `10.minutes()`, `2.hours()`)
- **Error Handling**: Comprehensive error types for invalid configurations
- **Thread Safety**: Jobs can be safely shared across threads
- **Documentation**: Extensive inline documentation with examples
- **Testing**: Comprehensive test suite with 26 unit tests and 14 doc tests

### Features
- Single-threaded scheduler with blocking execution
- Zero external dependencies (only chrono for time handling)
- Type-safe configuration with compile-time guarantees
- Clear error messages for configuration issues
- Support for multiple concurrent jobs

### Examples
- Basic interval scheduling example
- Time-based scheduling example
- Multiple jobs example in `examples/simple.rs`

### Technical Details
- Built with Rust 2024 edition
- Compatible with Rust 1.70+
- MIT licensed
- Published to crates.io

### Known Limitations
- Single-threaded execution (jobs run sequentially)
- No persistence (schedules lost on restart)
- Time precision limited to seconds
- No complex cron-like expressions

---

## Types of Changes
- `Added` for new features
- `Changed` for changes in existing functionality
- `Deprecated` for soon-to-be removed features
- `Removed` for now removed features
- `Fixed` for any bug fixes
- `Security` in case of vulnerabilities

## Versioning
This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes