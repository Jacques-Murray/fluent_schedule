//! # Scheduler Error
//!
//! This module defines the public error types for the `fluent_schedule` library.

use std::fmt;

/// Represents errors that can occur during job configuration.
#[derive(Debug, PartialEq, Clone)]
pub enum SchedulerError {
    /// The provided time string was not a valid 'HH:MM' or 'HH:MM:SS' format.
    InvalidTimeFormat(String),
    /// The job was not given a task to run (missing `.run()`).
    TaskNotSet,
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeFormat(s) => write!(
                f,
                "Invalid time format: '{}'. Expected 'HH:MM' or 'HH:MM:SS'.",
                s
            ),
            Self::TaskNotSet => {
                write!(
                    f,
                    "Job was not given a task to run. Call .run() to set a task."
                )
            }
        }
    }
}

// This allows SchedulerError to be used as a standard error
impl std::error::Error for SchedulerError {}
