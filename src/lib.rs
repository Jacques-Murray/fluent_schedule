//! # Fluent Schedule
//!
//! A human-readable, fluent task scheduling library for Rust.
//! This library provides a simple API for scheduling tasks without
//! using complex cron syntax.
//!
//! ## Examples
//!
//! Scheduling a task to run every 5 seconds:
//!
//! ```no_run
//! use fluent_schedule::{Job, Scheduler, FluentDuration};
//!
//! // Define a task
//! let job1 = Job::new()
//!     .every(5u32.seconds())
//!     .run(|| println!("Task 1: Running every 5 seconds."));
//!
//! // Create a scheduler
//! let mut scheduler = Scheduler::new();
//!
//! // Add the job
//! scheduler.add(job1);
//!
//! // Run the scheduler (this blocks the thread)
//! scheduler.run_forever();
//! ```
//!
//! Sceduling a task for a specific time:
//!
//! ```no_run
//! use fluent_schedule::{Job, Scheduler};
//! use chrono::Weekday;
//!
//! let job2 = Job::new()
//!     .on_weekday()
//!     .at("17:00")
//!     .run(|| println!("Task 2: Running at 5PM on weekdays."));
//!
//! let mut scheduler = Scheduler::new();
//! scheduler.add(job2);
//! scheduler.run_forever();
//! ```

mod job;
mod scheduler;
mod time_units;

// Public exports
pub use job::Job;
pub use scheduler::Scheduler;
pub use time_units::FluentDuration;
