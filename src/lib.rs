//! # Fluent Schedule
//!
//! A human-readable, fluent task scheduling library for Rust.
//! This library provides a simple API for scheduling tasks without
//! using complex cron syntax.

mod job;
mod scheduler;
mod time_units;

// Public exports
pub use job::Job;
pub use scheduler::Scheduler;
pub use time_units::FluentDuration;
