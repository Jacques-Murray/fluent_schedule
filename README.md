# Fluent Schedule

[![Rust CI](https://github.com/Jacques-Murray/fluent_schedule/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/Jacques-Murray/fluent_schedule/actions/workflows/rust-ci.yml)
[![Crates.io](https://img.shields.io/crates/v/fluent_schedule.svg)](https://crates.io/crates/fluent_schedule)
[![Documentation](https://docs.rs/fluent_schedule/badge.svg)](https://docs.rs/fluent_schedule)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A human-readable, fluent task scheduling library for Rust. This library provides a simple API for scheduling tasks without using complex cron syntax.

## Features

- **Fluent API**: Chain methods to build schedules in a readable way
- **Flexible Scheduling**: Support for intervals, specific times, and day-of-week scheduling
- **Type Safety**: Compile-time guarantees for valid configurations
- **Error Handling**: Clear error messages for invalid configurations
- **Zero Dependencies**: Only depends on `chrono` for time handling
- **Thread Safe**: Jobs can be safely shared across threads

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluent_schedule = "1.0.0"
```

## Quick Start

### Basic Interval Scheduling

```rust
use fluent_schedule::{Job, Scheduler, FluentDuration};

fn main() {
    // Create a job that runs every 5 seconds
    let job = Job::new()
        .every(5u32.seconds())
        .run(|| println!("Task executed every 5 seconds!"));

    // Create and start the scheduler
    let mut scheduler = Scheduler::new();
    scheduler.add(job).expect("Failed to add job");

    // This blocks the current thread
    scheduler.run_forever();
}
```

### Time-Based Scheduling

```rust
use fluent_schedule::{Job, Scheduler};
use chrono::Weekday;

fn main() {
    // Create a job that runs at 5:00 PM on weekdays
    let job = Job::new()
        .on_weekday()
        .at("17:00")
        .run(|| println!("End of workday!"));

    let mut scheduler = Scheduler::new();
    scheduler.add(job).expect("Failed to add job");

    scheduler.run_forever();
}
```

### Multiple Jobs

```rust
use fluent_schedule::{Job, Scheduler, FluentDuration};
use chrono::Weekday;

fn main() {
    let job1 = Job::new()
        .every(10u32.seconds())
        .run(|| println!("Heartbeat every 10 seconds"));

    let job2 = Job::new()
        .on(Weekday::Mon)
        .at("09:00")
        .run(|| println!("Monday morning meeting"));

    let job3 = Job::new()
        .on_weekend()
        .at("10:00")
        .run(|| println!("Weekend task"));

    let mut scheduler = Scheduler::new();
    scheduler.add(job1).expect("Failed to add job1");
    scheduler.add(job2).expect("Failed to add job2");
    scheduler.add(job3).expect("Failed to add job3");

    scheduler.run_forever();
}
```

## API Overview

### Job Builder Methods

- `Job::new()` - Create a new job
- `.every(duration)` - Run at fixed intervals
- `.at(time_str)` - Run at specific time (HH:MM or HH:MM:SS)
- `.on(weekday)` - Run on specific days of the week
- `.on_weekday()` - Run Monday through Friday
- `.on_weekend()` - Run Saturday and Sunday
- `.run(closure)` - Set the task to execute

### Fluent Duration Extensions

The library extends unsigned integers with time unit methods:

```rust
use fluent_schedule::FluentDuration;

let five_seconds = 5u32.seconds();
let ten_minutes = 10u32.minutes();
let two_hours = 2u32.hours();
```

### Scheduler Methods

- `Scheduler::new()` - Create a new scheduler
- `.add(job)` - Add a configured job (returns Result)
- `.run_forever()` - Start the scheduler (blocks current thread)

### Error Handling

The library uses `SchedulerError` for configuration issues:

```rust
use fluent_schedule::{Job, Scheduler, SchedulerError};

let invalid_job = Job::new().at("99:99").run(|| {});
let mut scheduler = Scheduler::new();

match scheduler.add(invalid_job) {
    Ok(()) => println!("Job added successfully"),
    Err(SchedulerError::InvalidTimeFormat(time)) => {
        eprintln!("Invalid time format: {}", time);
    }
    Err(SchedulerError::TaskNotSet) => {
        eprintln!("Job must have a task set with .run()");
    }
}
```

## Examples

See the `examples/` directory for more usage examples:

- `simple.rs` - Basic usage with multiple job types

Run an example:

```bash
cargo run --example simple
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with verbose output:

```bash
cargo test --verbose
```

## Documentation

Generate and view documentation:

```bash
cargo doc --open
```

## Performance Considerations

- The scheduler runs in a single thread and blocks on `run_forever()`
- Jobs should be lightweight to avoid blocking other scheduled tasks
- For CPU-intensive tasks, consider spawning threads within the job closure
- The scheduler calculates sleep durations dynamically based on the next job's schedule

## Limitations

- Single-threaded execution (jobs run sequentially)
- No persistence (schedules are lost on restart)
- Time precision is limited to seconds
- No support for complex cron-like expressions

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.
